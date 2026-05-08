//! Shape B OpenAPI demonstration helpers.
//!
//! OpenAPI is deliberately not a compiler emit target: Shape B artifacts are
//! user-program outputs derived from a compiled DAG, while `emit.rs` remains
//! scoped to Shape A programming-language targets. This module provides the
//! narrow Rust-side receipt used by the R3 demo until the equivalent `.dag`
//! program can own the artifact projection.

use std::collections::{BTreeMap, BTreeSet};

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

pub fn extract_rest_routes(dag: &Dag) -> Result<BTreeSet<RestRoute>, ProjectOpenApiError> {
    let mut routes = BTreeSet::new();
    for decl in dag.declarations() {
        let Some(ValueBody::List(rows)) = &decl.value_body else {
            continue;
        };
        if !list_element_has_rest_endpoint_binding(dag, decl) {
            continue;
        }
        for row in rows {
            let Some(fields) = record_fields(row) else {
                continue;
            };
            let Some(endpoint) = record_field(fields, "endpoint") else {
                continue;
            };
            let endpoint_fields = require_record(
                decl.name.as_deref().unwrap_or("<anonymous>"),
                "endpoint",
                endpoint,
            )?;
            let method = parse_http_method(
                dag,
                decl.name.as_deref().unwrap_or("<anonymous>"),
                record_field(endpoint_fields, "method").ok_or_else(|| {
                    malformed(decl.name.as_deref(), "endpoint missing `method` field")
                })?,
            )?;
            let path = parse_path_template(
                dag,
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

fn list_element_has_rest_endpoint_binding(dag: &Dag, decl: &Declaration) -> bool {
    let Some(element) = list_element_type(dag, decl) else {
        return false;
    };
    let TypeConnective::Conj { children } = &dag.declaration(element).connective else {
        return false;
    };
    let Some(endpoint_field) = children.iter().find(|field| field.label == "endpoint") else {
        return false;
    };
    dag.declaration(endpoint_field.ty).name.as_deref() == Some("RestEndpointBinding")
        && dag.declaration(endpoint_field.ty).span.file == "src/v3/std/services.dag"
}

fn list_element_type(dag: &Dag, decl: &Declaration) -> Option<DeclarationId> {
    match &decl.connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if dag.declaration(*template).name.as_deref() == Some("List") => {
            arguments.first().map(|arg| arg.value)
        }
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

fn append_path_parameter_yaml(out: &mut String, parameter: &str) {
    out.push_str("        - name: ");
    out.push_str(parameter);
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

fn parse_http_method(
    dag: &Dag,
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
    variant_label_in_parent(dag, "HttpMethod", *constructor).ok_or_else(|| {
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
            variant_label_in_parent(dag, "UrlPathToken", *constructor).ok_or_else(|| {
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
    parent_name: &str,
    constructor: DeclarationId,
) -> Option<String> {
    let parent = dag.declaration_by_name(parent_name)?;
    let TypeConnective::Disj { variants } = &parent.connective else {
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

fn yaml_plain_operation_id(method: &str, path: &str) -> String {
    let suffix = path
        .trim_matches('/')
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    format!("{}_{}", method.to_ascii_lowercase(), suffix)
}
