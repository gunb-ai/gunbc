//! Shape B OpenAPI and documentation demonstration helpers.
//!
//! OpenAPI and Markdown are deliberately not compiler emit targets: Shape B
//! artifacts are user-program outputs derived from a compiled DAG, while
//! `emit.rs` remains scoped to Shape A programming-language targets. This
//! module provides the narrow Rust-side receipt used by the R3 demo until the
//! equivalent `.dag` programs can own the artifact projections.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::dag::{Declaration, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};
use crate::Dag;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RestRoute {
    pub method: String,
    pub path: String,
    pub path_parameters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectOpenApiError {
    MalformedOperation { declaration: String, detail: String },
}

#[derive(Debug, Clone, Copy)]
struct RestRouteSchema {
    http_method: DeclarationId,
    url_path_token: DeclarationId,
}

pub fn extract_rest_routes(dag: &Dag) -> Result<BTreeSet<RestRoute>, ProjectOpenApiError> {
    let rest_endpoint_binding = canonical_rest_endpoint_binding(dag);
    let mut routes = BTreeSet::new();
    for decl in dag.declarations() {
        let Some(ValueBody::List(rows)) = &decl.value_body else {
            continue;
        };
        let Some(schema) = rest_route_schema(dag, decl, rest_endpoint_binding)? else {
            continue;
        };
        for row in rows {
            let fields = record_fields(row).ok_or_else(|| {
                malformed(
                    decl.name.as_deref(),
                    "service operation rows must be records",
                )
            })?;
            let endpoint = record_field(fields, "endpoint").ok_or_else(|| {
                malformed(
                    decl.name.as_deref(),
                    "service operation row missing `endpoint` field",
                )
            })?;
            let endpoint_fields = require_record(
                decl.name.as_deref().unwrap_or("<anonymous>"),
                "endpoint",
                endpoint,
            )?;
            let method = parse_http_method(
                dag,
                schema,
                decl.name.as_deref().unwrap_or("<anonymous>"),
                record_field(endpoint_fields, "method").ok_or_else(|| {
                    malformed(decl.name.as_deref(), "endpoint missing `method` field")
                })?,
            )?;
            let path = parse_path_template(
                dag,
                schema,
                decl.name.as_deref().unwrap_or("<anonymous>"),
                record_field(endpoint_fields, "path").ok_or_else(|| {
                    malformed(decl.name.as_deref(), "endpoint missing `path` field")
                })?,
            )?;
            routes.insert(RestRoute {
                method,
                path: path.path,
                path_parameters: path.parameters,
            });
        }
    }
    Ok(routes)
}

fn rest_route_schema(
    dag: &Dag,
    decl: &Declaration,
    rest_endpoint_binding: Option<DeclarationId>,
) -> Result<Option<RestRouteSchema>, ProjectOpenApiError> {
    let Some(element) = list_element_type(dag, decl) else {
        return Ok(None);
    };
    let TypeConnective::Conj { children } = &dag.declaration(element).connective else {
        return Ok(None);
    };
    let Some(endpoint_field) = children.iter().find(|field| field.label == "endpoint") else {
        return Ok(None);
    };
    if Some(endpoint_field.ty) != rest_endpoint_binding {
        return Ok(None);
    }
    let TypeConnective::Conj { children } = &dag.declaration(endpoint_field.ty).connective else {
        return Ok(None);
    };
    let Some(method_ty) = field_type(children, "method") else {
        return Err(malformed(
            decl.name.as_deref(),
            "RestEndpointBinding missing `method` field",
        ));
    };
    let Some(path_ty) = field_type(children, "path") else {
        return Err(malformed(
            decl.name.as_deref(),
            "RestEndpointBinding missing `path` field",
        ));
    };

    let TypeConnective::Conj { children } = &dag.declaration(path_ty).connective else {
        return Err(malformed(
            decl.name.as_deref(),
            "endpoint.path type must be a PathTemplate record",
        ));
    };
    let Some(tokens_ty) = field_type(children, "tokens") else {
        return Err(malformed(
            decl.name.as_deref(),
            "PathTemplate missing `tokens` field",
        ));
    };
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &dag.declaration(tokens_ty).connective
    else {
        return Err(malformed(
            decl.name.as_deref(),
            "PathTemplate.tokens must be a List",
        ));
    };
    if Some(*template) != dag.list_template() {
        return Err(malformed(
            decl.name.as_deref(),
            "PathTemplate.tokens must use the canonical List template",
        ));
    }
    let Some(url_path_token_ty) = arguments.first().map(|arg| arg.value) else {
        return Err(malformed(
            decl.name.as_deref(),
            "PathTemplate.tokens missing element type",
        ));
    };

    Ok(Some(RestRouteSchema {
        http_method: method_ty,
        url_path_token: url_path_token_ty,
    }))
}

fn canonical_rest_endpoint_binding(dag: &Dag) -> Option<DeclarationId> {
    let mut matches = dag.declarations().iter().filter(|decl| {
        decl.name.as_deref() == Some("RestEndpointBinding")
            && decl.span.file == "src/v3/std/services.dag"
    });
    let id = matches.next()?.id;
    if matches.next().is_some() {
        None
    } else {
        Some(id)
    }
}

fn list_element_type(dag: &Dag, decl: &Declaration) -> Option<DeclarationId> {
    match &decl.connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if Some(*template) == dag.list_template() => arguments.first().map(|arg| arg.value),
        _ => None,
    }
}

pub fn project_openapi_yaml(dag: &Dag) -> Result<String, ProjectOpenApiError> {
    let routes = extract_rest_routes(dag)?;
    let mut out = String::from(
        "openapi: 3.1.0\ninfo:\n  title: GunBC generated service\n  version: 0.1.0\npaths:\n",
    );
    if routes.is_empty() {
        out.push_str("  {}\n");
        return Ok(out);
    }

    let mut routes_by_path: BTreeMap<&str, Vec<&RestRoute>> = BTreeMap::new();
    for route in &routes {
        routes_by_path
            .entry(route.path.as_str())
            .or_default()
            .push(route);
    }

    for (path, path_routes) in routes_by_path {
        out.push_str("  ");
        out.push_str(&yaml_quoted(path));
        out.push_str(":\n");
        for route in path_routes {
            out.push_str("    ");
            out.push_str(&route.method.to_ascii_lowercase());
            out.push_str(":\n      operationId: ");
            out.push_str(&yaml_plain_operation_id(&route.method, &route.path));
            if !route.path_parameters.is_empty() {
                out.push_str("\n      parameters:\n");
                for parameter in &route.path_parameters {
                    append_path_parameter_yaml(&mut out, parameter);
                }
            }
            out.push_str("\n      responses:\n        '200':\n          description: OK\n");
        }
    }
    Ok(out)
}

pub fn project_markdown_documentation(dag: &Dag) -> Result<String, ProjectOpenApiError> {
    let routes = extract_rest_routes(dag)?;
    let mut out = String::from(
        "# GunBC generated service\n\n| Method | Path | Path parameters |\n| --- | --- | --- |\n",
    );
    if routes.is_empty() {
        out.push_str("| _none_ | _none_ | _none_ |\n");
        return Ok(out);
    }
    for route in routes {
        out.push_str("| ");
        out.push_str(&markdown_table_cell(&route.method));
        out.push_str(" | ");
        out.push_str(&markdown_code_span_table_cell(&route.path));
        out.push_str(" | ");
        if route.path_parameters.is_empty() {
            out.push_str("_none_");
        } else {
            for (index, parameter) in route.path_parameters.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&markdown_code_span_table_cell(parameter));
            }
        }
        out.push_str(" |\n");
    }
    Ok(out)
}

pub fn project_rust_backend_service(dag: &Dag) -> Result<String, ProjectOpenApiError> {
    let routes = extract_rest_routes(dag)?;
    let mut out = String::from(
        r#"const ROUTES: &[(&str, &str)] = &[
"#,
    );
    for route in &routes {
        out.push_str("    (");
        out.push_str(&rust_string_literal(&route.method));
        out.push_str(", ");
        out.push_str(&rust_string_literal(&route.path));
        out.push_str("),\n");
    }
    out.push_str(
        r#"];

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(method) = args.next() else {
        print_routes();
        return;
    };
    let Some(path) = args.next() else {
        print_routes();
        return;
    };

    for (route_method, route_template) in ROUTES {
        if method.eq_ignore_ascii_case(route_method) && path_matches(route_template, &path) {
            println!("200 {} {}", route_method, route_template);
            return;
        }
    }
    println!("404 {} {}", method.to_ascii_uppercase(), path);
}

fn print_routes() {
    for (method, route_template) in ROUTES {
        println!("{} {}", method, route_template);
    }
}

fn path_matches(template: &str, candidate: &str) -> bool {
    let template = template.as_bytes();
    let candidate = candidate.as_bytes();
    let mut t = 0;
    let mut c = 0;
    while t < template.len() {
        if template[t] == b'{' {
            let Some(close_offset) = template[t + 1..].iter().position(|byte| *byte == b'}') else {
                return false;
            };
            let close = t + 1 + close_offset;
            let after_param = close + 1;
            if after_param == template.len() {
                return c < candidate.len();
            }
            let next_param = template[after_param..]
                .iter()
                .position(|byte| *byte == b'{')
                .map(|offset| after_param + offset)
                .unwrap_or(template.len());
            let literal = &template[after_param..next_param];
            let Some(next_match) = find_subslice(&candidate[c..], literal) else {
                return false;
            };
            if next_match == 0 {
                return false;
            }
            c += next_match;
            t = after_param;
        } else {
            if candidate.get(c) != Some(&template[t]) {
                return false;
            }
            t += 1;
            c += 1;
        }
    }
    c == candidate.len()
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
"#,
    );
    Ok(out)
}

fn append_path_parameter_yaml(out: &mut String, parameter: &str) {
    out.push_str("        - name: ");
    out.push_str(&yaml_double_quoted(parameter));
    out.push_str(
        "\n          in: path\n          required: true\n          schema:\n            type: string\n",
    );
}

fn malformed(declaration: Option<&str>, detail: impl Into<String>) -> ProjectOpenApiError {
    ProjectOpenApiError::MalformedOperation {
        declaration: declaration.unwrap_or("<anonymous>").to_string(),
        detail: detail.into(),
    }
}

fn record_fields(value: &FieldValue) -> Option<&[(String, FieldValue)]> {
    match value {
        FieldValue::Record(fields) => Some(fields.as_slice()),
        _ => None,
    }
}

fn require_record<'a>(
    declaration: &str,
    field: &str,
    value: &'a FieldValue,
) -> Result<&'a [(String, FieldValue)], ProjectOpenApiError> {
    record_fields(value).ok_or_else(|| ProjectOpenApiError::MalformedOperation {
        declaration: declaration.to_string(),
        detail: format!("`{field}` must be a record"),
    })
}

fn record_field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> Option<&'a FieldValue> {
    fields.iter().find(|(l, _)| l == label).map(|(_, v)| v)
}

fn field_type(fields: &[crate::dag::Field], label: &str) -> Option<DeclarationId> {
    fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| field.ty)
}

fn parse_http_method(
    dag: &Dag,
    schema: RestRouteSchema,
    declaration: &str,
    value: &FieldValue,
) -> Result<String, ProjectOpenApiError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(ProjectOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "`endpoint.method` must be an HttpMethod variant".to_string(),
        });
    };
    if !payload.is_empty() {
        return Err(ProjectOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "HttpMethod variants must not carry payload".to_string(),
        });
    }
    variant_label_in_parent(dag, schema.http_method, *constructor).ok_or_else(|| {
        ProjectOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "HttpMethod constructor is not a variant of HttpMethod".to_string(),
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPathTemplate {
    path: String,
    parameters: Vec<String>,
}

fn parse_path_template(
    dag: &Dag,
    schema: RestRouteSchema,
    declaration: &str,
    value: &FieldValue,
) -> Result<ParsedPathTemplate, ProjectOpenApiError> {
    let fields = require_record(declaration, "endpoint.path", value)?;
    let tokens =
        record_field(fields, "tokens").ok_or_else(|| ProjectOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "PathTemplate missing `tokens` field".to_string(),
        })?;
    let FieldValue::List(tokens) = tokens else {
        return Err(ProjectOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "PathTemplate.tokens must be a list".to_string(),
        });
    };
    let mut pieces = Vec::with_capacity(tokens.len());
    let mut parameters = Vec::new();
    for token in tokens {
        let FieldValue::Variant {
            constructor,
            payload,
        } = token
        else {
            return Err(ProjectOpenApiError::MalformedOperation {
                declaration: declaration.to_string(),
                detail: "PathTemplate token must be a UrlPathToken variant".to_string(),
            });
        };
        let label =
            variant_label_in_parent(dag, schema.url_path_token, *constructor).ok_or_else(|| {
                ProjectOpenApiError::MalformedOperation {
                    declaration: declaration.to_string(),
                    detail: "token constructor is not a variant of UrlPathToken".to_string(),
                }
            })?;
        let text = single_string_payload(payload).ok_or_else(|| {
            ProjectOpenApiError::MalformedOperation {
                declaration: declaration.to_string(),
                detail: format!("{label} token payload must contain one string"),
            }
        })?;
        match label.as_str() {
            "LiteralToken" => pieces.push(text),
            "ParamToken" => {
                pieces.push(format!("{{{text}}}"));
                parameters.push(text);
            }
            other => {
                return Err(ProjectOpenApiError::MalformedOperation {
                    declaration: declaration.to_string(),
                    detail: format!("unsupported UrlPathToken variant `{other}`"),
                })
            }
        }
    }
    let path = pieces.concat();
    Ok(ParsedPathTemplate {
        path: if path.starts_with('/') {
            path
        } else {
            format!("/{path}")
        },
        parameters,
    })
}

fn variant_label_in_parent(
    dag: &Dag,
    parent: DeclarationId,
    constructor: DeclarationId,
) -> Option<String> {
    let TypeConnective::Disj { variants } = &dag.declaration(parent).connective else {
        return None;
    };
    variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .map(|variant| variant.label.clone())
}

fn single_string_payload(payload: &[FieldValue]) -> Option<String> {
    let [value] = payload else {
        return None;
    };
    match value {
        FieldValue::Literal(LiteralBits::String(value)) => Some(value.clone()),
        FieldValue::Record(fields) => fields.iter().find_map(|(label, value)| {
            if label == "text" || label == "name" {
                match value {
                    FieldValue::Literal(LiteralBits::String(value)) => Some(value.clone()),
                    _ => None,
                }
            } else {
                None
            }
        }),
        _ => None,
    }
}

fn yaml_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn yaml_double_quoted(value: &str) -> String {
    let mut quoted = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            other if other.is_control() => {
                write!(&mut quoted, "\\u{:04X}", other as u32).expect("write to String");
            }
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

fn yaml_plain_operation_id(method: &str, path: &str) -> String {
    let mut suffix = String::new();
    for ch in path.trim_matches('/').chars() {
        if ch.is_ascii_alphanumeric() {
            suffix.push(ch);
        } else {
            write!(&mut suffix, "_x{:X}_", ch as u32).expect("write to String");
        }
    }
    format!("{}_{}", method.to_ascii_lowercase(), suffix)
}

fn rust_string_literal(value: &str) -> String {
    let mut literal = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            other if other.is_control() => {
                write!(&mut literal, "\\u{{{:X}}}", other as u32).expect("write to String");
            }
            other => literal.push(other),
        }
    }
    literal.push('"');
    literal
}

fn markdown_table_cell(value: &str) -> String {
    value.replace('\\', "\\\\").replace('|', "\\|")
}

fn markdown_code_span(value: &str) -> String {
    let delimiter = "`".repeat(max_backtick_run(value) + 1);
    if value.starts_with('`') || value.ends_with('`') {
        format!("{delimiter} {value} {delimiter}")
    } else {
        format!("{delimiter}{value}{delimiter}")
    }
}

fn markdown_code_span_table_cell(value: &str) -> String {
    markdown_code_span(value).replace('|', "\\|")
}

fn max_backtick_run(value: &str) -> usize {
    let mut max = 0;
    let mut current = 0;
    for ch in value.chars() {
        if ch == '`' {
            current += 1;
            max = max.max(current);
        } else {
            current = 0;
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_to_dag;

    const SERVICE_FIXTURE: &str = r#"
module t.openapi_malformed_projection

import std.types { GET }
import std.effects { LiteralToken }
import v3.std.services { RestEndpointBinding }

type DemoOperation {
  endpoint: RestEndpointBinding
}

data service_operations: List<DemoOperation> = [
  {
    endpoint: {
      method: GET,
      path: { tokens: [LiteralToken { text: "users" }] }
    }
  }
]
"#;

    fn compiled_service_fixture() -> Dag {
        std::thread::Builder::new()
            .name("openapi_malformed_projection_compile".to_string())
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                compile_to_dag(SERVICE_FIXTURE, "openapi_malformed_projection_fixture.dag")
                    .expect("service fixture compiles")
            })
            .expect("spawn larger-stack compile thread")
            .join()
            .expect("larger-stack compile thread completes")
    }

    fn service_operations_id(dag: &Dag) -> DeclarationId {
        dag.declarations()
            .iter()
            .find(|decl| decl.name.as_deref() == Some("service_operations"))
            .expect("service_operations declaration exists")
            .id
    }

    fn replace_first_service_row(dag: &mut Dag, replacement: FieldValue) {
        let service_operations = service_operations_id(dag);
        let ValueBody::List(rows) = dag
            .declaration_mut(service_operations)
            .value_body
            .as_mut()
            .expect("service_operations has value body")
        else {
            panic!("service_operations is a list");
        };
        rows[0] = replacement;
    }

    #[test]
    fn extract_rest_routes_rejects_non_record_service_rows() {
        let mut dag = compiled_service_fixture();
        replace_first_service_row(
            &mut dag,
            FieldValue::Literal(LiteralBits::String("not a record".to_string())),
        );

        assert_eq!(
            extract_rest_routes(&dag),
            Err(ProjectOpenApiError::MalformedOperation {
                declaration: "service_operations".to_string(),
                detail: "service operation rows must be records".to_string(),
            })
        );
    }

    #[test]
    fn extract_rest_routes_rejects_service_rows_missing_endpoint() {
        let mut dag = compiled_service_fixture();
        replace_first_service_row(&mut dag, FieldValue::Record(vec![]));

        assert_eq!(
            extract_rest_routes(&dag),
            Err(ProjectOpenApiError::MalformedOperation {
                declaration: "service_operations".to_string(),
                detail: "service operation row missing `endpoint` field".to_string(),
            })
        );
    }

    #[test]
    fn path_parameter_names_are_quoted_for_yaml_structure() {
        let mut yaml = String::new();

        append_path_parameter_yaml(&mut yaml, "id:\nrequired: false");

        assert_eq!(
            yaml,
            "        - name: \"id:\\nrequired: false\"\n          in: path\n          required: true\n          schema:\n            type: string\n"
        );
    }

    #[test]
    fn markdown_code_cells_choose_safe_delimiters_and_escape_table_delimiters() {
        assert_eq!(markdown_code_span_table_cell("a|b`c\\d"), "``a\\|b`c\\d``");
        assert_eq!(
            markdown_code_span_table_cell("a``b```c"),
            "````a``b```c````"
        );
        assert_eq!(markdown_code_span_table_cell("\\path"), "`\\path`");
        assert_eq!(markdown_code_span_table_cell("`x"), "`` `x ``");
        assert_eq!(markdown_code_span_table_cell("x`"), "`` x` ``");
    }

    #[test]
    fn rust_backend_projection_escapes_route_literals() {
        assert_eq!(rust_string_literal("/a\"b\\c\n"), "\"/a\\\"b\\\\c\\n\"");
    }
}
