//! Rust backend for test rendering.
//!
//! Renders `TestFile` → valid Rust source compatible with `#[test]` and
//! the gunbc test harness.

use super::render::TestRenderer;
use super::test_ir::*;
use gunbc_ir::ValueExpr;

pub struct RustRenderer;

impl TestRenderer for RustRenderer {
    fn extension(&self) -> &str {
        "rs"
    }

    fn render_value(&self, expr: &ValueExpr) -> String {
        match expr {
            ValueExpr::Unit => "Value::Unit".to_string(),
            ValueExpr::Bool(b) => format!("Value::Bool({})", b),
            ValueExpr::Str(s) => format!(
                "Value::Str(\"{}\".to_string())",
                s.replace('\\', "\\\\").replace('\"', "\\\"")
            ),
            ValueExpr::Int(i) => format!("Value::Int({})", i),
            ValueExpr::List(items) => {
                let rendered: Vec<String> = items.iter().map(|v| self.render_value(v)).collect();
                format!("Value::List(vec![{}])", rendered.join(", "))
            }
            ValueExpr::Map(entries) => {
                if entries.is_empty() {
                    return "Value::Map(std::collections::BTreeMap::new())".to_string();
                }
                let rendered: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "(\"{}\".to_string(), {})",
                            k.replace('\\', "\\\\").replace('\"', "\\\""),
                            self.render_value(v)
                        )
                    })
                    .collect();
                format!(
                    "Value::Map(std::collections::BTreeMap::from([{}]))",
                    rendered.join(", ")
                )
            }
            ValueExpr::Json(json) => {
                format!("Value::Json(serde_json::json!({}))", json)
            }
            ValueExpr::Struct { name, fields } => {
                self.render_rust_struct(name, fields)
            }
            ValueExpr::Secret(s) => {
                format!(
                    "Value::Secret(gunbc_ir::SecretString::new(\"{}\"))",
                    s.replace('\\', "\\\\").replace('\"', "\\\"")
                )
            }
            ValueExpr::Skipped => "Value::Skipped".to_string(),
        }
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
                "fn {}() -> {} {{\n    {}\n}}\n\n",
                helper.name, helper.return_type, helper.body_expr
            ));
        }

        // Test sections
        for section in &file.sections {
            out.push_str(&format!(
                "// =========================================================================\n\
                 // {}\n\
                 // =========================================================================\n\n",
                section.title
            ));

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
            Expr::Str(s) => format!(
                "\"{}\"",
                s.replace('\\', "\\\\").replace('\"', "\\\"")
            ),
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
                    message.replace('\"', "\\\"")
                )
            }
            Assert::True { expr, message } => {
                format!(
                    "{}assert!({}, \"{}\");\n",
                    pad,
                    self.render_expr(expr),
                    message.replace('\"', "\\\"")
                )
            }
            Assert::NonEmpty { expr, message } => {
                format!(
                    "{}assert!(!{}.is_empty(), \"{}\");\n",
                    pad,
                    self.render_expr(expr),
                    message.replace('\"', "\\\"")
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
                    substring.replace('\"', "\\\""),
                    message.replace('\"', "\\\""),
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
    /// Render a struct-typed ValueExpr to Rust source.
    ///
    /// Handles transport types (TransportRequest::Shell, etc.) by
    /// emitting fully-qualified Rust struct construction.
    fn render_rust_struct(&self, name: &str, fields: &[(String, ValueExpr)]) -> String {
        // Transport types need wrapping in Value::Request/Response
        let (wrapper, inner_path) = match name {
            n if n.starts_with("TransportRequest::") => {
                let variant = n.strip_prefix("TransportRequest::").unwrap();
                (
                    Some("Value::Request"),
                    format!(
                        "gunbc_ir::transport::TransportRequest::{}",
                        variant
                    ),
                )
            }
            n if n.starts_with("TransportResponse::") => {
                let variant = n.strip_prefix("TransportResponse::").unwrap();
                (
                    Some("Value::Response"),
                    format!(
                        "gunbc_ir::transport::TransportResponse::{}",
                        variant
                    ),
                )
            }
            _ => (None, name.to_string()),
        };

        // Build the inner struct literal
        let struct_type = match name {
            n if n.starts_with("TransportRequest::") => {
                let variant = n.strip_prefix("TransportRequest::").unwrap();
                format!("gunbc_ir::transport::{}Request", variant)
            }
            n if n.starts_with("TransportResponse::") => {
                let variant = n.strip_prefix("TransportResponse::").unwrap();
                format!("gunbc_ir::transport::{}Response", variant)
            }
            _ => name.to_string(),
        };

        let field_strs: Vec<String> = fields
            .iter()
            .map(|(k, v)| format!("{}: {}", k, self.render_struct_field_value(v)))
            .collect();

        let struct_lit = format!("{} {{ {} }}", struct_type, field_strs.join(", "));

        match wrapper {
            Some(w) => format!("{}({}({}))", w, inner_path, struct_lit),
            None => struct_lit,
        }
    }

    /// Render a struct field value — uses native Rust types, not Value wrappers.
    fn render_struct_field_value(&self, expr: &ValueExpr) -> String {
        match expr {
            ValueExpr::Unit => "None".to_string(),
            ValueExpr::Bool(b) => format!("{}", b),
            ValueExpr::Str(s) => format!(
                "\"{}\".to_string()",
                s.replace('\\', "\\\\").replace('\"', "\\\"")
            ),
            ValueExpr::Int(i) => format!("{}", i),
            ValueExpr::List(items) => {
                let rendered: Vec<String> = items
                    .iter()
                    .map(|v| self.render_struct_field_value(v))
                    .collect();
                format!("vec![{}]", rendered.join(", "))
            }
            ValueExpr::Map(entries) => {
                if entries.is_empty() {
                    return "std::collections::HashMap::new()".to_string();
                }
                let rendered: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "(\"{}\".to_string(), {})",
                            k.replace('\\', "\\\\").replace('\"', "\\\""),
                            self.render_struct_field_value(v)
                        )
                    })
                    .collect();
                format!(
                    "std::collections::HashMap::from([{}])",
                    rendered.join(", ")
                )
            }
            ValueExpr::Json(json) => format!("serde_json::json!({})", json),
            ValueExpr::Struct { name, fields } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.render_struct_field_value(v)))
                    .collect();
                format!("{} {{ {} }}", name, field_strs.join(", "))
            }
            ValueExpr::Secret(s) => format!(
                "gunbc_ir::SecretString::new(\"{}\")",
                s.replace('\\', "\\\\").replace('\"', "\\\"")
            ),
            ValueExpr::Skipped => "Value::Skipped".to_string(),
        }
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
            r.render_value(&ValueExpr::List(vec![ValueExpr::Int(1), ValueExpr::Bool(true)])),
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
    fn render_expr_method_chain() {
        let r = RustRenderer;
        let expr = Expr::var("mocks")
            .method("insert", vec![
                Expr::str_lit("node"),
                Expr::str_lit("port"),
                Expr::Value(ValueExpr::Bool(true)),
            ]);
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
        assert_eq!(
            r.render_stmt(&stmt, 1),
            "    let dag = gist_graph();\n"
        );
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
}
