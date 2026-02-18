//! Rust backend for code rendering.
//!
//! Renders `TestFile`/`SourceFile` → valid Rust source via
//! `CodeRenderer<M>` where `M: TextMedium`.

use gunbc_ir::code_ir::*;
use gunbc_ir::render_ir::{CodeRenderer, OutputMedium, TextMedium};
use gunbc_ir::ValueExpr;
use std::fmt::Write;

/// Escape a string for embedding in a Rust string literal.
fn escape_rust_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Generic Rust code renderer parameterized over output medium.
pub struct RustCodeRenderer<M: OutputMedium> {
    medium: M,
}

impl<M: OutputMedium> RustCodeRenderer<M> {
    pub fn new(medium: M) -> Self {
        Self { medium }
    }
}

/// Whether to render a ValueExpr as a `Value::X(...)` constructor or as a
/// bare Rust type (for transport struct fields).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueMode {
    /// Render as `Value` enum constructor: `Value::Str("x".to_string())`
    Wrapped,
    /// Render as bare Rust type: `"x".to_string()`
    Bare,
}

impl<M: TextMedium> CodeRenderer<M> for RustCodeRenderer<M> {
    fn medium(&self) -> &M {
        &self.medium
    }

    fn render_value(&self, expr: &ValueExpr) -> String {
        self.render_value_inner(expr, ValueMode::Wrapped)
    }

    fn render_file(&self, file: &TestFile) -> String {
        let mut out = String::new();

        // Header comments
        for line in &file.header {
            writeln!(out, "// {}", line).unwrap();
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
            writeln!(out, "fn {}() -> {} {{", helper.name, helper.return_type).unwrap();
            for stmt in &helper.body {
                out.push_str(&self.render_stmt(stmt, 1));
            }
            out.push_str("}\n\n");
        }

        // Test sections
        for section in &file.sections {
            write!(
                out,
                "// =========================================================================\n\
                 // {}\n\
                 // =========================================================================\n\n",
                section.title
            )
            .unwrap();

            for note in &section.notes {
                writeln!(out, "// {}", note).unwrap();
            }
            if !section.notes.is_empty() {
                out.push('\n');
            }

            for test_fn in &section.tests {
                // Doc comments
                for line in &test_fn.doc {
                    writeln!(out, "/// {}", line).unwrap();
                }
                // Test attribute and function signature
                out.push_str("#[test]\n");
                writeln!(out, "fn {}() {{", test_fn.name).unwrap();

                // Body
                for stmt in &test_fn.body {
                    out.push_str(&self.render_stmt(stmt, 1));
                }

                out.push_str("}\n\n");
            }
        }

        out
    }

    fn render_source_file(&self, file: &SourceFile) -> String {
        let mut out = String::new();

        // Module-level doc comments
        for line in &file.doc {
            writeln!(out, "//! {}", line).unwrap();
        }
        if !file.doc.is_empty() {
            out.push('\n');
        }

        for item in &file.items {
            out.push_str(&self.render_item(item, 0));
            out.push('\n');
        }

        out
    }

    fn render_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Value(v) => self.render_value(v),
            Expr::Var(name) => name.clone(),
            Expr::Str(s) => format!("\"{}\"", escape_rust_str(s)),
            Expr::Call { func, args, .. } => {
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
            Expr::Match { expr, arms } => {
                let mut out = format!("match {} {{\n", self.render_expr(expr));
                for arm in arms {
                    writeln!(out, "    {} => {{", arm.pattern).unwrap();
                    for stmt in &arm.body {
                        out.push_str(&self.render_stmt(stmt, 2));
                    }
                    out.push_str("    }\n");
                }
                out.push('}');
                out
            }
            Expr::If {
                cond,
                then_body,
                else_body,
            } => {
                let mut out = format!("if {} {{\n", self.render_expr(cond));
                for stmt in then_body {
                    out.push_str(&self.render_stmt(stmt, 1));
                }
                if let Some(else_stmts) = else_body {
                    out.push_str("} else {\n");
                    for stmt in else_stmts {
                        out.push_str(&self.render_stmt(stmt, 1));
                    }
                }
                out.push('}');
                out
            }
            Expr::Block(stmts) => {
                let mut out = "{\n".to_string();
                for stmt in stmts {
                    out.push_str(&self.render_stmt(stmt, 1));
                }
                out.push('}');
                out
            }
            Expr::FormatStr { template, args } => {
                if args.is_empty() {
                    format!("format!(\"{}\")", escape_rust_str(template))
                } else {
                    let args_str: Vec<String> = args.iter().map(|a| self.render_expr(a)).collect();
                    format!(
                        "format!(\"{}\", {})",
                        escape_rust_str(template),
                        args_str.join(", ")
                    )
                }
            }
            Expr::MacroCall { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.render_expr(a)).collect();
                format!("{}!({})", name, args_str.join(", "))
            }
            Expr::Tuple(items) => {
                let items_str: Vec<String> = items.iter().map(|e| self.render_expr(e)).collect();
                format!("({})", items_str.join(", "))
            }
            Expr::Array(items) => {
                let items_str: Vec<String> = items.iter().map(|e| self.render_expr(e)).collect();
                format!("[{}]", items_str.join(", "))
            }
            Expr::RawCode(code) => code.clone(),
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
                let rendered = self.render_expr(expr);
                if rendered == "()" {
                    format!("{}return;\n", pad)
                } else {
                    format!("{}return {};\n", pad, rendered)
                }
            }
            Stmt::TailExpr(expr) => {
                format!("{}{}\n", pad, self.render_expr(expr))
            }
            Stmt::For {
                binding,
                iter,
                body,
            } => {
                let mut out = format!("{}for {} in {} {{\n", pad, binding, self.render_expr(iter));
                for stmt in body {
                    out.push_str(&self.render_stmt(stmt, indent + 1));
                }
                writeln!(out, "{}}}", pad).unwrap();
                out
            }
            Stmt::Item(item) => self.render_item(item, indent),
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

    fn render_item(&self, item: &Item, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        match item {
            Item::Use(import) => format!("{}{}\n", pad, self.render_import(import)),
            Item::Fn(f) => self.render_fn_def(f, indent),
            Item::Enum(e) => self.render_enum_def(e, indent),
            Item::Impl(i) => self.render_impl_block(i, indent),
            Item::Struct(s) => self.render_struct_def(s, indent),
            Item::Raw(code) => format!("{}{}\n", pad, code),
        }
    }
}

impl<M: TextMedium> RustCodeRenderer<M> {
    /// Core value rendering: a single exhaustive match over ValueExpr.
    ///
    /// `Wrapped` mode emits `Value::X(...)` constructors (for test assertions).
    /// `Bare` mode emits native Rust types (for transport struct fields).
    pub(crate) fn render_value_inner(&self, expr: &ValueExpr, mode: ValueMode) -> String {
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
    fn render_rust_struct(&self, name: &str, fields: &[(String, ValueExpr)]) -> String {
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
                "cwd" | "stdin" | "timeout_ms" => {
                    Some(format!("{}: {}", field, self.render_option_value(value)))
                }
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

    fn render_fn_def(&self, f: &FnDef, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        let mut out = String::new();

        // Doc comments
        for line in &f.doc {
            writeln!(out, "{}/// {}", pad, line).unwrap();
        }

        // Attributes
        for attr in &f.attributes {
            writeln!(out, "{}{}", pad, attr).unwrap();
        }

        // Signature
        let vis = if f.is_pub { "pub " } else { "" };
        let params: Vec<String> = f
            .params
            .iter()
            .map(|(name, ty)| {
                if ty.is_empty() {
                    name.clone()
                } else {
                    format!("{}: {}", name, ty)
                }
            })
            .collect();
        let ret = match &f.return_type {
            Some(ty) => format!(" -> {}", ty),
            None => String::new(),
        };
        writeln!(
            out,
            "{}{}fn {}({}){} {{",
            pad,
            vis,
            f.name,
            params.join(", "),
            ret
        )
        .unwrap();

        // Body
        for stmt in &f.body {
            out.push_str(&self.render_stmt(stmt, indent + 1));
        }

        writeln!(out, "{}}}", pad).unwrap();
        out
    }

    fn render_enum_def(&self, e: &EnumDef, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        let mut out = String::new();

        // Doc comments
        for line in &e.doc {
            writeln!(out, "{}/// {}", pad, line).unwrap();
        }

        // Derives
        if !e.derives.is_empty() {
            writeln!(out, "{}#[derive({})]", pad, e.derives.join(", ")).unwrap();
        }

        // Enum header
        let vis = if e.is_pub { "pub " } else { "" };
        writeln!(out, "{}{}enum {} {{", pad, vis, e.name).unwrap();

        // Variants
        for variant in &e.variants {
            writeln!(out, "{}    {},", pad, variant).unwrap();
        }

        writeln!(out, "{}}}", pad).unwrap();
        out
    }

    fn render_impl_block(&self, i: &ImplBlock, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        let mut out = String::new();

        let header = match &i.trait_name {
            Some(tr) => format!("{}impl {} for {} {{\n", pad, tr, i.type_name),
            None => format!("{}impl {} {{\n", pad, i.type_name),
        };
        out.push_str(&header);

        for (idx, func) in i.items.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }
            out.push_str(&self.render_fn_def(func, indent + 1));
        }

        writeln!(out, "{}}}", pad).unwrap();
        out
    }

    fn render_struct_def(&self, s: &StructDef, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        let mut out = String::new();

        // Doc comments
        for line in &s.doc {
            writeln!(out, "{}/// {}", pad, line).unwrap();
        }

        // Derives
        if !s.derives.is_empty() {
            writeln!(out, "{}#[derive({})]", pad, s.derives.join(", ")).unwrap();
        }

        // Struct header
        let vis = if s.is_pub { "pub " } else { "" };
        writeln!(out, "{}{}struct {} {{", pad, vis, s.name).unwrap();

        // Fields
        for (name, ty, is_pub) in &s.fields {
            let field_vis = if *is_pub { "pub " } else { "" };
            writeln!(out, "{}    {}{}: {},", pad, field_vis, name, ty).unwrap();
        }

        writeln!(out, "{}}}", pad).unwrap();
        out
    }
}

/// Create a RustCodeRenderer with PlainText medium (most common usage).
pub fn plain_rust_renderer() -> RustCodeRenderer<gunbc_ir::render_ir::PlainText> {
    RustCodeRenderer::new(gunbc_ir::render_ir::PlainText {
        tier: gunbc_ir::symbols::Tier::Ascii,
        symbol_set: &gunbc_ir::symbols::STANDARD,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::render_ir::CodeRenderer;

    fn r() -> RustCodeRenderer<gunbc_ir::render_ir::PlainText> {
        plain_rust_renderer()
    }

    #[test]
    fn render_value_covers_all_variants() {
        let r = r();
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
        let r = r();
        assert_eq!(
            r.render_value(&ValueExpr::Str("say \"hi\"".into())),
            "Value::Str(\"say \\\"hi\\\"\".to_string())"
        );
    }

    #[test]
    fn render_bare_value_no_wrapper() {
        let r = r();
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
        let r = r();
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
        let r = r();
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
        let r = r();
        let stmt = Stmt::let_bind("dag", Expr::call("gist_graph", vec![]));
        assert_eq!(r.render_stmt(&stmt, 1), "    let dag = gist_graph();\n");
    }

    #[test]
    fn render_assert_eq() {
        let r = r();
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
        let r = r();
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
        let r = r();
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
