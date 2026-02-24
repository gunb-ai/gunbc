//! Runtime interpreter: converts DSL `Expr` AST nodes into `Value` instances.
//!
//! This is the runtime equivalent of `daglang_emit::test_mock_emit::emit_value_expr`,
//! which generates Rust code strings. This module produces actual `Value` values
//! directly, used by the testgen binary to build `MockSpec`s from `.dag` test blocks
//! without an intermediate codegen step.

use daglang_syntax::ast::{Expr, Literal};
use gunbc_ir::transport::{FileOp, FileResponse, RestResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;

/// Interpret a DSL expression into a runtime `Value`.
///
/// Handles the finite set of constructors used in `.dag` test mock blocks:
/// - Literals: strings, ints, floats, bools, none
/// - Transport constructors: `shell_response`, `rest_response`, `file_response`
/// - Records (named or anonymous): `{ key: "value" }`
/// - Lists, Maps, identifiers
pub fn interpret_expr(expr: &Expr) -> Value {
    match expr {
        Expr::Literal(lit) => interpret_literal(lit),

        Expr::Ident(name) => match name.as_str() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            "none" => Value::Unit,
            _ => Value::Str(name.clone()),
        },

        Expr::Call(name, args) => match name.as_str() {
            "shell_response" => interpret_shell_response(args),
            "rest_response" => interpret_rest_response(args),
            "file_response" => interpret_file_response(args),
            "bytes" => {
                if let Some((_, arg)) = args.first() {
                    interpret_expr(arg)
                } else {
                    Value::Str(String::new())
                }
            }
            _ => Value::Str(format!("{name}(...)")),
        },

        Expr::Record(name, fields) => interpret_record(name.as_deref(), fields),

        Expr::List(items) => Value::List(items.iter().map(interpret_expr).collect()),

        Expr::Map(entries) => {
            Value::Json(serde_json::Value::Object(
                entries
                    .iter()
                    .map(|(k, v)| (expr_to_json_key(k), expr_to_json(v)))
                    .collect(),
            ))
        }

        Expr::StringInterp(parts) => {
            let mut s = String::new();
            for part in parts {
                match part {
                    daglang_syntax::ast::StringPart::Literal(lit) => s.push_str(lit),
                    daglang_syntax::ast::StringPart::Expr(e) => {
                        s.push_str(&format!("{{{}}}", inline_expr_name(e)));
                    }
                }
            }
            Value::Str(s)
        }

        Expr::UnaryOp(op, inner) => match op {
            daglang_syntax::ast::UnaryOp::Neg => {
                if let Expr::Literal(Literal::Int(n)) = inner.as_ref() {
                    Value::Int(-n)
                } else {
                    interpret_expr(inner)
                }
            }
            daglang_syntax::ast::UnaryOp::Not => {
                if let Expr::Literal(Literal::Bool(b)) = inner.as_ref() {
                    Value::Bool(!b)
                } else {
                    interpret_expr(inner)
                }
            }
        },

        _ => Value::Unit,
    }
}

/// Whether the expression is a transport response constructor.
///
/// Mirrors `daglang_emit::test_mock_emit::is_transport_response_value`.
pub fn is_transport_response(expr: &Expr) -> bool {
    match expr {
        Expr::Call(name, _) => {
            matches!(
                name.as_str(),
                "rest_response" | "shell_response" | "file_response"
            )
        }
        Expr::Record(name, _) => name.as_deref().is_some_and(|n| {
            matches!(
                n,
                "rest_response"
                    | "RestResponse"
                    | "shell_response"
                    | "ShellResponse"
                    | "file_response"
                    | "FileResponse"
            )
        }),
        _ => false,
    }
}

fn interpret_literal(lit: &Literal) -> Value {
    match lit {
        Literal::String(s) => Value::Str(s.clone()),
        Literal::Int(n) => Value::Int(*n),
        Literal::Float(f) => Value::Float(*f),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::None => Value::Unit,
    }
}

fn interpret_shell_response(args: &[(Option<String>, Expr)]) -> Value {
    let exit_code = args
        .first()
        .map(|(_, e)| expr_to_i64(e))
        .unwrap_or(0);
    let stdout = args
        .get(1)
        .map(|(_, e)| expr_to_string(e))
        .unwrap_or_default();
    Value::Response(TransportResponse::Shell(ShellResponse {
        exit_code: exit_code as i32,
        stdout,
        stderr: String::new(),
    }))
}

fn interpret_rest_response(args: &[(Option<String>, Expr)]) -> Value {
    let status = args
        .first()
        .map(|(_, e)| expr_to_i64(e) as u16)
        .unwrap_or(200);
    let body = args
        .get(1)
        .map(|(_, e)| expr_to_json(e))
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    Value::Response(TransportResponse::Rest(RestResponse::new(status, body)))
}

fn interpret_file_response(args: &[(Option<String>, Expr)]) -> Value {
    let content = args
        .first()
        .map(|(_, e)| expr_to_string(e));
    Value::Response(TransportResponse::File(FileResponse {
        path: String::new(),
        operation: FileOp::Read,
        success: true,
        content,
        exists: None,
        error: None,
        bytes: None,
    }))
}

fn interpret_record(name: Option<&str>, fields: &[(String, Expr)]) -> Value {
    match name {
        Some(
            kind @ ("rest_response" | "RestResponse" | "shell_response" | "ShellResponse"
            | "file_response" | "FileResponse"),
        ) => interpret_named_transport_record(kind, fields),
        _ => {
            // Anonymous or non-transport record → JSON object
            let obj: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), expr_to_json(v)))
                .collect();
            Value::Json(serde_json::Value::Object(obj))
        }
    }
}

fn interpret_named_transport_record(kind: &str, fields: &[(String, Expr)]) -> Value {
    let find = |names: &[&str]| -> Option<&Expr> {
        fields
            .iter()
            .find(|(k, _)| names.iter().any(|n| k == n))
            .map(|(_, v)| v)
    };

    match kind {
        "rest_response" | "RestResponse" => {
            let status = find(&["status", "code"])
                .map(expr_to_i64)
                .unwrap_or(200) as u16;
            let body_fields: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .filter(|(k, _)| k != "status" && k != "code")
                .map(|(k, v)| (k.clone(), expr_to_json(v)))
                .collect();
            Value::Response(TransportResponse::Rest(RestResponse::new(
                status,
                serde_json::Value::Object(body_fields),
            )))
        }
        "shell_response" | "ShellResponse" => {
            let exit_code = find(&["exit_code"])
                .map(expr_to_i64)
                .unwrap_or(0);
            let stdout = find(&["stdout"])
                .map(expr_to_string)
                .unwrap_or_default();
            Value::Response(TransportResponse::Shell(ShellResponse {
                exit_code: exit_code as i32,
                stdout,
                stderr: String::new(),
            }))
        }
        "file_response" | "FileResponse" => {
            let content = find(&["path", "content"]).map(expr_to_string);
            Value::Response(TransportResponse::File(FileResponse {
                path: String::new(),
                operation: FileOp::Read,
                success: true,
                content,
                exists: None,
                error: None,
                bytes: None,
            }))
        }
        _ => Value::Unit,
    }
}

fn expr_to_i64(expr: &Expr) -> i64 {
    match expr {
        Expr::Literal(Literal::Int(n)) => *n,
        _ => 0,
    }
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s)) => s.clone(),
        Expr::Literal(Literal::Int(n)) => n.to_string(),
        Expr::Literal(Literal::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn expr_to_json(expr: &Expr) -> serde_json::Value {
    match expr {
        Expr::Literal(Literal::String(s)) => serde_json::Value::String(s.clone()),
        Expr::Literal(Literal::Int(n)) => serde_json::json!(*n),
        Expr::Literal(Literal::Float(f)) => serde_json::json!(*f),
        Expr::Literal(Literal::Bool(b)) => serde_json::Value::Bool(*b),
        Expr::Literal(Literal::None) => serde_json::Value::Null,
        Expr::Ident(name) => match name.as_str() {
            "true" => serde_json::Value::Bool(true),
            "false" => serde_json::Value::Bool(false),
            "none" | "null" => serde_json::Value::Null,
            _ => serde_json::Value::String(name.clone()),
        },
        Expr::Record(_, fields) => {
            let obj: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), expr_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Expr::List(items) => {
            serde_json::Value::Array(items.iter().map(expr_to_json).collect())
        }
        _ => serde_json::Value::Null,
    }
}

fn expr_to_json_key(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s)) => s.clone(),
        Expr::Ident(name) => name.clone(),
        _ => String::new(),
    }
}

fn inline_expr_name(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.clone(),
        Expr::FieldAccess(base, field) => format!("{}.{field}", inline_expr_name(base)),
        _ => "?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpret_bool_literal() {
        assert_eq!(
            interpret_expr(&Expr::Literal(Literal::Bool(true))),
            Value::Bool(true)
        );
    }

    #[test]
    fn interpret_string_literal() {
        assert_eq!(
            interpret_expr(&Expr::Literal(Literal::String("hello".into()))),
            Value::Str("hello".into())
        );
    }

    #[test]
    fn interpret_int_literal() {
        assert_eq!(
            interpret_expr(&Expr::Literal(Literal::Int(42))),
            Value::Int(42)
        );
    }

    #[test]
    fn interpret_shell_response_call() {
        let expr = Expr::Call(
            "shell_response".into(),
            vec![
                (None, Expr::Literal(Literal::Int(0))),
                (None, Expr::Literal(Literal::String("ok".into()))),
            ],
        );
        let value = interpret_expr(&expr);
        match value {
            Value::Response(TransportResponse::Shell(resp)) => {
                assert_eq!(resp.exit_code, 0);
                assert_eq!(resp.stdout, "ok");
            }
            other => panic!("expected ShellResponse, got {other:?}"),
        }
    }

    #[test]
    fn interpret_rest_response_call() {
        let expr = Expr::Call(
            "rest_response".into(),
            vec![
                (None, Expr::Literal(Literal::Int(200))),
                (
                    None,
                    Expr::Record(
                        None,
                        vec![("ok".into(), Expr::Literal(Literal::Bool(true)))],
                    ),
                ),
            ],
        );
        let value = interpret_expr(&expr);
        match value {
            Value::Response(TransportResponse::Rest(resp)) => {
                assert_eq!(resp.status, 200);
            }
            other => panic!("expected RestResponse, got {other:?}"),
        }
    }

    #[test]
    fn is_transport_detects_shell() {
        let expr = Expr::Call("shell_response".into(), vec![]);
        assert!(is_transport_response(&expr));
    }

    #[test]
    fn is_transport_rejects_plain_value() {
        let expr = Expr::Literal(Literal::Bool(true));
        assert!(!is_transport_response(&expr));
    }
}
