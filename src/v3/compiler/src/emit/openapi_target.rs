use std::collections::BTreeSet;

use crate::dag::{DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};
use crate::Dag;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RestRoute {
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitOpenApiError {
    MalformedOperation { declaration: String, detail: String },
}

pub fn extract_rest_routes(dag: &Dag) -> Result<BTreeSet<RestRoute>, EmitOpenApiError> {
    let mut routes = BTreeSet::new();
    for decl in dag.declarations() {
        let Some(ValueBody::List(rows)) = &decl.value_body else {
            continue;
        };
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
            routes.insert(RestRoute { method, path });
        }
    }
    Ok(routes)
}

pub fn emit_openapi_yaml(dag: &Dag) -> Result<String, EmitOpenApiError> {
    let routes = extract_rest_routes(dag)?;
    let mut out = String::from(
        "openapi: 3.1.0\ninfo:\n  title: GunBC generated service\n  version: 0.1.0\npaths:\n",
    );
    if routes.is_empty() {
        out.push_str("  {}\n");
        return Ok(out);
    }

    let mut current_path: Option<&str> = None;
    for route in &routes {
        if current_path != Some(route.path.as_str()) {
            out.push_str("  ");
            out.push_str(&yaml_quoted(&route.path));
            out.push_str(":\n");
            current_path = Some(route.path.as_str());
        }
        out.push_str("    ");
        out.push_str(&route.method.to_ascii_lowercase());
        out.push_str(":\n      operationId: ");
        out.push_str(&yaml_plain_operation_id(&route.method, &route.path));
        out.push_str("\n      responses:\n        '200':\n          description: OK\n");
    }
    Ok(out)
}

fn malformed(declaration: Option<&str>, detail: impl Into<String>) -> EmitOpenApiError {
    EmitOpenApiError::MalformedOperation {
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
) -> Result<&'a [(String, FieldValue)], EmitOpenApiError> {
    record_fields(value).ok_or_else(|| EmitOpenApiError::MalformedOperation {
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
) -> Result<String, EmitOpenApiError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "`endpoint.method` must be an HttpMethod variant".to_string(),
        });
    };
    if !payload.is_empty() {
        return Err(EmitOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "HttpMethod variants must not carry payload".to_string(),
        });
    }
    variant_label_in_parent(dag, "HttpMethod", *constructor).ok_or_else(|| {
        EmitOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "HttpMethod constructor is not a variant of HttpMethod".to_string(),
        }
    })
}

fn parse_path_template(
    dag: &Dag,
    declaration: &str,
    value: &FieldValue,
) -> Result<String, EmitOpenApiError> {
    let fields = require_record(declaration, "endpoint.path", value)?;
    let tokens =
        record_field(fields, "tokens").ok_or_else(|| EmitOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "PathTemplate missing `tokens` field".to_string(),
        })?;
    let FieldValue::List(tokens) = tokens else {
        return Err(EmitOpenApiError::MalformedOperation {
            declaration: declaration.to_string(),
            detail: "PathTemplate.tokens must be a list".to_string(),
        });
    };
    let mut segments = Vec::with_capacity(tokens.len());
    for token in tokens {
        let FieldValue::Variant {
            constructor,
            payload,
        } = token
        else {
            return Err(EmitOpenApiError::MalformedOperation {
                declaration: declaration.to_string(),
                detail: "PathTemplate token must be a UrlPathToken variant".to_string(),
            });
        };
        let label =
            variant_label_in_parent(dag, "UrlPathToken", *constructor).ok_or_else(|| {
                EmitOpenApiError::MalformedOperation {
                    declaration: declaration.to_string(),
                    detail: "token constructor is not a variant of UrlPathToken".to_string(),
                }
            })?;
        let text =
            single_string_payload(payload).ok_or_else(|| EmitOpenApiError::MalformedOperation {
                declaration: declaration.to_string(),
                detail: format!("{label} token payload must contain one string"),
            })?;
        match label.as_str() {
            "LiteralToken" => segments.push(text),
            "ParamToken" => segments.push(format!("{{{text}}}")),
            other => {
                return Err(EmitOpenApiError::MalformedOperation {
                    declaration: declaration.to_string(),
                    detail: format!("unsupported UrlPathToken variant `{other}`"),
                })
            }
        }
    }
    Ok(format!("/{}", segments.join("/")))
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
