//! Rust backend for test rendering.
//!
//! Renders `TestFile` → valid Rust source compatible with `#[test]` and
//! the gunbc test harness.

use super::render::TestRenderer;
use super::test_ir::*;
use gunbc_ir::ValueExpr;

/// Escape a string for embedding in a Rust string literal.
fn escape_rust_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub struct RustRenderer;

/// Whether to render a ValueExpr as a `Value::X(...)` constructor or as a
/// bare Rust type (for transport struct fields).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueMode {
    /// Render as `Value` enum constructor: `Value::Str("x".to_string())`
    Wrapped,
    /// Render as bare Rust type: `"x".to_string()`
    Bare,
}

impl TestRenderer for RustRenderer {
    fn extension(&self) -> &str {
        "rs"
    }

    fn render_value(&self, expr: &ValueExpr) -> String {
        self.render_value_inner(expr, ValueMode::Wrapped)
    }

    fn render_file(&self, file: &TestFile) -> String {
        let mut out = String::new();

        // Header comments
        for line in &file.header {
            out.push_str(&format!("// {}\n", line));
        }
        out.push('\n');

        // Imports
        out.push('\n');
        for import in &file.imports {
            out.push_str(&self.render_import(import));
            out.push('\n');
        }
        out.push('\n');

        // Helper functions
        for helper in &file.helpers {
            out.push_str(&format!(
                "fn {}() -> {} {{\n",
                helper.name, helper.return_type
            ));
            for stmt in &helper.body {
                out.push_str(&self.render_stmt(stmt, 1));
            }
            out.push_str("}\n\n");
        }

        // Test sections
        for section in &file.sections {
            out.push_str(&format!(
                "// =========================================================================\n\
                 // {}\n\
                 // =========================================================================\n\n",
                section.title
            ));

            for note in &section.notes {
                out.push_str(&format!("// {}\n", note));
            }
            if !section.notes.is_empty() {
                out.push('\n');
            }

            for test_fn in &section.tests {
                // Doc comments
                for line in &test_fn.doc {
                    out.push_str(&format!("/// {}\n", line));
                }
                // Test attribute and function signature
                out.push_str("#[test]\n");
                out.push_str(&format!("fn {}() {{\n", test_fn.name));

                // Body
                for stmt in &test_fn.body {
                    out.push_str(&self.render_stmt(stmt, 1));
                }

                out.push_str("}\n\n");
            }
        }

        out
    }

    fn render_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Value(v) => self.render_value(v),
            Expr::Var(name) => name.clone(),
            Expr::Str(s) => format!("\"{}\"", escape_rust_str(s)),
            Expr::Call { func, args } => {
                let func_str = self.render_expr(func);
                let args_str: Vec<String> = args.iter().map(|a| self.render_expr(a)).collect();
                format!("{}({})", func_str, args_str.join(", "))
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let recv = self.render_expr(receiver);
                let args_str: Vec<String> = args.iter().map(|a| self.render_expr(a)).collect();
                format!("{}.{}({})", recv, method, args_str.join(", "))
            }
            Expr::Field(expr, field) => {
                format!("{}.{}", self.render_expr(expr), field)
            }
            Expr::Deref(expr) => format!("*{}", self.render_expr(expr)),
            Expr::Ref(expr) => format!("&{}", self.render_expr(expr)),
            Expr::RefMut(expr) => format!("&mut {}", self.render_expr(expr)),
            Expr::Path(segments) => segments.join("::"),
            Expr::Struct { name, fields } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.render_expr(v)))
                    .collect();
                format!("{} {{ {} }}", name, field_strs.join(", "))
            }
            Expr::Closure { args, body } => {
                let body_str = self.render_expr(body);
                if args.is_empty() {
                    format!("|| {}", body_str)
                } else {
                    format!("|{}| {}", args.join(", "), body_str)
                }
            }
            Expr::BinOp { left, op, right } => {
                format!(
                    "{} {} {}",
                    self.render_expr(left),
                    op,
                    self.render_expr(right)
                )
            }
            Expr::UnaryOp { op, expr } => {
                format!("{}{}", op, self.render_expr(expr))
            }
            Expr::IntLit(n) => n.to_string(),
            Expr::BoolLit(b) => b.to_string(),
        }
    }

    fn render_stmt(&self, stmt: &Stmt, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        match stmt {
            Stmt::Let {
                name,
                mutable,
                expr,
            } => {
                let mut_kw = if *mutable { "mut " } else { "" };
                let expr_str = self.render_expr(expr);
                format!("{}let {}{} = {};\n", pad, mut_kw, name, expr_str)
            }
            Stmt::Expr(expr) => {
                format!("{}{};\n", pad, self.render_expr(expr))
            }
            Stmt::Assert(assert) => self.render_assert(assert, indent),
            Stmt::Comment(text) => {
                if text.is_empty() {
                    format!("{}\n", pad)
                } else {
                    format!("{}// {}\n", pad, text)
                }
            }
            Stmt::Blank => "\n".to_string(),
            Stmt::Return(expr) => {
                format!("{}return {};\n", pad, self.render_expr(expr))
            }
            Stmt::TailExpr(expr) => {
                format!("{}{}\n", pad, self.render_expr(expr))
            }
        }
    }

    fn render_assert(&self, assert: &Assert, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        match assert {
            Assert::Eq {
                left,
                right,
                message,
            } => {
                format!(
                    "{}assert_eq!({}, {}, \"{}\");\n",
                    pad,
                    self.render_expr(left),
                    self.render_expr(right),
                    escape_rust_str(message)
                )
            }
            Assert::True { expr, message } => {
                format!(
                    "{}assert!({}, \"{}\");\n",
                    pad,
                    self.render_expr(expr),
                    escape_rust_str(message)
                )
            }
            Assert::NonEmpty { expr, message } => {
                format!(
                    "{}assert!(!{}.is_empty(), \"{}\");\n",
                    pad,
                    self.render_expr(expr),
                    escape_rust_str(message)
                )
            }
            Assert::Contains {
                expr,
                substring,
                message,
            } => {
                format!(
                    "{}assert!({}.as_str().map(|s| s.contains(\"{}\")).unwrap_or(false), \"{}\", {});\n",
                    pad,
                    self.render_expr(expr),
                    escape_rust_str(substring),
                    escape_rust_str(message),
                    self.render_expr(expr),
                )
            }
        }
    }

    fn render_import(&self, import: &Import) -> String {
        let path = import.path.join("::");
        if import.items.is_empty() {
            format!("use {};", path)
        } else if import.items.len() == 1 {
            format!("use {}::{};", path, import.items[0])
        } else {
            format!("use {}::{{{}}};", path, import.items.join(", "))
        }
    }
}

impl RustRenderer {
    /// Core value rendering: a single exhaustive match over ValueExpr.
    ///
    /// `Wrapped` mode emits `Value::X(...)` constructors (for test assertions).
    /// `Bare` mode emits native Rust types (for transport struct fields).
    fn render_value_inner(&self, expr: &ValueExpr, mode: ValueMode) -> String {
        let bare = mode == ValueMode::Bare;
        match expr {
            ValueExpr::Unit => if bare { "None" } else { "Value::Unit" }.to_string(),
            ValueExpr::Bool(b) => {
                if bare {
                    format!("{}", b)
                } else {
                    format!("Value::Bool({})", b)
                }
            }
            ValueExpr::Str(s) => {
                let escaped = escape_rust_str(s);
                if bare {
                    format!("\"{}\".to_string()", escaped)
                } else {
                    format!("Value::Str(\"{}\".to_string())", escaped)
                }
            }
            ValueExpr::Int(i) => {
                if bare {
                    format!("{}", i)
                } else {
                    format!("Value::Int({})", i)
                }
            }
            ValueExpr::List(items) => {
                let rendered: Vec<String> = items
                    .iter()
                    .map(|v| self.render_value_inner(v, mode))
                    .collect();
                let inner = format!("vec![{}]", rendered.join(", "));
                if bare {
                    inner
                } else {
                    format!("Value::List({})", inner)
                }
            }
            ValueExpr::Map(entries) => {
                // Wrapped uses BTreeMap (Value::Map's backing type).
                // Bare uses HashMap (transport struct field type).
                if bare {
                    if entries.is_empty() {
                        return "std::collections::HashMap::new()".to_string();
                    }
                    let rendered: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "(\"{}\".to_string(), {})",
                                escape_rust_str(k),
                                self.render_value_inner(v, mode)
                            )
                        })
                        .collect();
                    format!("std::collections::HashMap::from([{}])", rendered.join(", "))
                } else {
                    if entries.is_empty() {
                        return "Value::Map(std::collections::BTreeMap::new())".to_string();
                    }
                    let rendered: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "(\"{}\".to_string(), {})",
                                escape_rust_str(k),
                                self.render_value_inner(v, mode)
                            )
                        })
                        .collect();
                    format!(
                        "Value::Map(std::collections::BTreeMap::from([{}]))",
                        rendered.join(", ")
                    )
                }
            }
            ValueExpr::Json(json) => {
                let inner = format!("serde_json::json!({})", json);
                if bare {
                    inner
                } else {
                    format!("Value::Json({})", inner)
                }
            }
            ValueExpr::Struct { name, fields } => {
                if bare {
                    let field_strs: Vec<String> = fields
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, self.render_value_inner(v, mode)))
                        .collect();
                    format!("{} {{ {} }}", name, field_strs.join(", "))
                } else {
                    self.render_rust_struct(name, fields)
                }
            }
            ValueExpr::Secret(s) => {
                let escaped = escape_rust_str(s);
                let inner = format!("gunbc_ir::SecretString::new(\"{}\")", escaped);
                if bare {
                    inner
                } else {
                    format!("Value::Secret({})", inner)
                }
            }
            ValueExpr::Skipped => "Value::Skipped".to_string(),
        }
    }

    /// Render a struct-typed ValueExpr to Rust source.
    ///
    /// Handles transport types (TransportRequest::Shell, etc.) by
    /// emitting fully-qualified Rust struct construction wrapped in
    /// Value::Request/Response.
    fn render_rust_struct(&self, name: &str, fields: &[(String, ValueExpr)]) -> String {
        // Parse transport variant name once to determine all three values:
        // wrapper (Value::Request/Response), enum path, and inner struct type.
        let (wrapper, enum_path, struct_type) =
            if let Some(variant) = name.strip_prefix("TransportRequest::") {
                (
                    Some("Value::Request"),
                    format!("gunbc_ir::transport::TransportRequest::{}", variant),
                    format!("gunbc_ir::transport::{}Request", variant),
                )
            } else if let Some(variant) = name.strip_prefix("TransportResponse::") {
                (
                    Some("Value::Response"),
                    format!("gunbc_ir::transport::TransportResponse::{}", variant),
                    format!("gunbc_ir::transport::{}Response", variant),
                )
            } else {
                (None, name.to_string(), name.to_string())
            };

        let field_strs: Vec<String> = fields
            .iter()
            .map(|(k, v)| {
                if let Some(rendered) = self.render_transport_field(&struct_type, k, v) {
                    rendered
                } else {
                    format!("{}: {}", k, self.render_value_inner(v, ValueMode::Bare))
                }
            })
            .collect();

        let struct_lit = format!("{} {{ {} }}", struct_type, field_strs.join(", "));

        match wrapper {
            Some(w) => format!("{}({}({}))", w, enum_path, struct_lit),
            None => struct_lit,
        }
    }

    fn render_transport_field(
        &self,
        struct_type: &str,
        field: &str,
        value: &ValueExpr,
    ) -> Option<String> {
        match struct_type {
            "gunbc_ir::transport::FileRequest" => match field {
                "operation" => Some(format!("{}: {}", field, self.render_file_op(value))),
                "content" => Some(format!("{}: {}", field, self.render_option_value(value))),
                _ => None,
            },
            "gunbc_ir::transport::FileResponse" => match field {
                "operation" => Some(format!("{}: {}", field, self.render_file_op(value))),
                "content" | "exists" | "error" => {
                    Some(format!("{}: {}", field, self.render_option_value(value)))
                }
                _ => None,
            },
            "gunbc_ir::transport::ShellRequest" => match field {
                "cwd" | "stdin" => Some(format!("{}: {}", field, self.render_option_value(value))),
                _ => None,
            },
            "gunbc_ir::transport::TcpRequest" => match field {
                "data" => Some(format!("{}: {}", field, self.render_option_value(value))),
                _ => None,
            },
            "gunbc_ir::transport::TcpResponse" => match field {
                "data" | "error" => Some(format!("{}: {}", field, self.render_option_value(value))),
                _ => None,
            },
            "gunbc_ir::transport::HttpRequest" => match field {
                "method" => Some(format!("{}: {}", field, self.render_http_method(value))),
                "body" | "timeout_ms" => {
                    Some(format!("{}: {}", field, self.render_option_value(value)))
                }
                _ => None,
            },
            "gunbc_ir::transport::RestRequest" => match field {
                "method" => Some(format!("{}: {}", field, self.render_http_method(value))),
                "body" | "auth" | "timeout_ms" => {
                    Some(format!("{}: {}", field, self.render_option_value(value)))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn render_option_value(&self, value: &ValueExpr) -> String {
        match value {
            ValueExpr::Unit => "None".to_string(),
            _ => format!("Some({})", self.render_value_inner(value, ValueMode::Bare)),
        }
    }

    fn render_file_op(&self, value: &ValueExpr) -> String {
        let variant = match value {
            ValueExpr::Str(s) => s.as_str(),
            _ => "",
        };
        format!("gunbc_ir::transport::FileOp::{}", variant)
    }

    fn render_http_method(&self, value: &ValueExpr) -> String {
        let variant = match value {
            ValueExpr::Str(s) => s.as_str(),
            _ => "",
        };
        let method = match variant {
            "Get" | "GET" => "Get",
            "Post" | "POST" => "Post",
            "Put" | "PUT" => "Put",
            "Patch" | "PATCH" => "Patch",
            "Delete" | "DELETE" => "Delete",
            "Head" | "HEAD" => "Head",
            "Options" | "OPTIONS" => "Options",
            _ => variant,
        };
        format!("gunbc_ir::transport::HttpMethod::{}", method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_value_covers_all_variants() {
        let r = RustRenderer;
        assert_eq!(r.render_value(&ValueExpr::Unit), "Value::Unit");
        assert_eq!(r.render_value(&ValueExpr::Bool(true)), "Value::Bool(true)");
        assert_eq!(
            r.render_value(&ValueExpr::Str("hello".into())),
            "Value::Str(\"hello\".to_string())"
        );
        assert_eq!(r.render_value(&ValueExpr::Int(42)), "Value::Int(42)");
        assert_eq!(
            r.render_value(&ValueExpr::List(vec![
                ValueExpr::Int(1),
                ValueExpr::Bool(true)
            ])),
            "Value::List(vec![Value::Int(1), Value::Bool(true)])"
        );
        assert_eq!(r.render_value(&ValueExpr::Skipped), "Value::Skipped");
    }

    #[test]
    fn render_value_string_escaping() {
        let r = RustRenderer;
        assert_eq!(
            r.render_value(&ValueExpr::Str("say \"hi\"".into())),
            "Value::Str(\"say \\\"hi\\\"\".to_string())"
        );
    }

    #[test]
    fn render_bare_value_no_wrapper() {
        let r = RustRenderer;
        assert_eq!(
            r.render_value_inner(&ValueExpr::Unit, ValueMode::Bare),
            "None"
        );
        assert_eq!(
            r.render_value_inner(&ValueExpr::Bool(true), ValueMode::Bare),
            "true"
        );
        assert_eq!(
            r.render_value_inner(&ValueExpr::Str("hi".into()), ValueMode::Bare),
            "\"hi\".to_string()"
        );
        assert_eq!(
            r.render_value_inner(&ValueExpr::Int(42), ValueMode::Bare),
            "42"
        );
        assert_eq!(
            r.render_value_inner(
                &ValueExpr::List(vec![ValueExpr::Str("a".into())]),
                ValueMode::Bare
            ),
            "vec![\"a\".to_string()]"
        );
    }

    #[test]
    fn render_expr_method_chain() {
        let r = RustRenderer;
        let expr = Expr::var("mocks").method(
            "insert",
            vec![
                Expr::str_lit("node"),
                Expr::str_lit("port"),
                Expr::Value(ValueExpr::Bool(true)),
            ],
        );
        assert_eq!(
            r.render_expr(&expr),
            "mocks.insert(\"node\", \"port\", Value::Bool(true))"
        );
    }

    #[test]
    fn render_import() {
        let r = RustRenderer;
        let imp = Import {
            path: vec!["gunbc_exec".into()],
            items: vec!["execute_with_mode".into(), "BoundaryMocks".into()],
        };
        assert_eq!(
            r.render_import(&imp),
            "use gunbc_exec::{execute_with_mode, BoundaryMocks};"
        );
    }

    #[test]
    fn render_let_stmt() {
        let r = RustRenderer;
        let stmt = Stmt::let_bind("dag", Expr::call("gist_graph", vec![]));
        assert_eq!(r.render_stmt(&stmt, 1), "    let dag = gist_graph();\n");
    }

    #[test]
    fn render_assert_eq() {
        let r = RustRenderer;
        let a = Assert::Eq {
            left: Expr::var("output").deref(),
            right: Expr::Value(ValueExpr::Int(42)),
            message: "expected exact value".into(),
        };
        assert_eq!(
            r.render_assert(&a, 1),
            "    assert_eq!(*output, Value::Int(42), \"expected exact value\");\n"
        );
    }

    #[test]
    fn render_assert_non_empty() {
        let r = RustRenderer;
        let a = Assert::NonEmpty {
            expr: Expr::var("output"),
            message: "expected non-empty value".into(),
        };
        assert_eq!(
            r.render_assert(&a, 1),
            "    assert!(!output.is_empty(), \"expected non-empty value\");\n"
        );
    }

    #[test]
    fn render_bin_op_in_closure() {
        let r = RustRenderer;
        let expr = Expr::var("x").method("as_int", vec![]).method(
            "is_some_and",
            vec![Expr::Closure {
                args: vec!["n".to_string()],
                body: Box::new(Expr::var("n").bin_op(">=", Expr::int(2))),
            }],
        );
        assert_eq!(r.render_expr(&expr), "x.as_int().is_some_and(|n| n >= 2)");
    }
}
