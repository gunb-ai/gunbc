//! Rust CodeRenderer — renders Rust-specific SourceFile to .rs text.
//!
//! Standalone renderer for daglang-emit: takes a `SourceFile` (after
//! `lower_to_rust`) and produces valid `.rs` source text. Also provides
//! `render_cargo_toml` for generating a minimal Cargo.toml.
//!
//! This module mirrors the patterns in the codegen test renderer
//! but operates on the `SourceFile` IR without depending on the `CodeRenderer`
//! trait or `OutputMedium` abstractions.
//!
//! **Owned by**: Task 12 (dsl-codegen-tasks.md)

use gunbc_ir::code_ir::{
    Assert, BindIntent, BindTarget, EnumDef, Expr, FnDef, ImplBlock, Import, Item, MatchArm,
    SourceFile, Stmt, StructDef,
};
use gunbc_ir::ValueExpr;
use std::fmt::Write;

// ===========================================================================
// Public API
// ===========================================================================

/// Render a `SourceFile` to a complete `.rs` source string.
pub fn render_rust_source(source: &SourceFile) -> String {
    let mut out = String::new();

    // Module-level doc comments.
    for line in &source.doc {
        writeln!(out, "//! {}", line).unwrap();
    }
    if !source.doc.is_empty() {
        out.push('\n');
    }

    for item in &source.items {
        out.push_str(&render_item(item, 0));
        out.push('\n');
    }

    out
}

/// Public access to expression rendering (for fn_codegen RawCode interpolation).
pub fn render_expr_pub(expr: &Expr) -> String {
    render_expr(expr)
}

/// Render a minimal Cargo.toml for a generated crate.
pub fn render_cargo_toml(crate_name: &str, dependencies: &[(&str, &str)]) -> String {
    let mut out = String::new();
    writeln!(out, "[package]").unwrap();
    writeln!(out, "name = \"{}\"", crate_name).unwrap();
    writeln!(out, "version = \"0.1.0\"").unwrap();
    writeln!(out, "edition = \"2021\"").unwrap();
    out.push('\n');
    writeln!(out, "[dependencies]").unwrap();
    for (name, version) in dependencies {
        writeln!(out, "{} = \"{}\"", name, version).unwrap();
    }
    out
}

// ===========================================================================
// String escaping
// ===========================================================================

fn escape_rust_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

// ===========================================================================
// Item rendering
// ===========================================================================

fn render_item(item: &Item, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match item {
        Item::Use(import) => format!("{}{}\n", pad, render_import(import)),
        Item::Fn(f) => render_fn_def(f, indent),
        Item::Enum(e) => render_enum_def(e, indent),
        Item::Impl(i) => render_impl_block(i, indent),
        Item::Struct(s) => render_struct_def(s, indent),
        Item::Raw(code) => format!("{}{}\n", pad, code),
    }
}

fn render_import(import: &Import) -> String {
    let path = import.path.join("::");
    if import.items.is_empty() {
        format!("use {};", path)
    } else if import.items.len() == 1 {
        format!("use {}::{};", path, import.items[0])
    } else {
        format!("use {}::{{{}}};", path, import.items.join(", "))
    }
}

fn render_fn_def(f: &FnDef, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();

    // Doc comments.
    for line in &f.doc {
        writeln!(out, "{}/// {}", pad, line).unwrap();
    }

    // Attributes.
    for attr in &f.attributes {
        writeln!(out, "{}{}", pad, attr).unwrap();
    }

    // Signature.
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

    // Body.
    for stmt in &f.body {
        out.push_str(&render_stmt(stmt, indent + 1));
    }

    writeln!(out, "{}}}", pad).unwrap();
    out
}

fn render_enum_def(e: &EnumDef, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();

    for line in &e.doc {
        writeln!(out, "{}/// {}", pad, line).unwrap();
    }

    if !e.derives.is_empty() {
        writeln!(out, "{}#[derive({})]", pad, e.derives.join(", ")).unwrap();
    }

    let vis = if e.is_pub { "pub " } else { "" };
    writeln!(out, "{}{}enum {} {{", pad, vis, e.name).unwrap();

    for variant in &e.variants {
        writeln!(out, "{}    {},", pad, variant).unwrap();
    }

    writeln!(out, "{}}}", pad).unwrap();
    out
}

fn render_struct_def(s: &StructDef, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();

    for line in &s.doc {
        writeln!(out, "{}/// {}", pad, line).unwrap();
    }

    if !s.derives.is_empty() {
        writeln!(out, "{}#[derive({})]", pad, s.derives.join(", ")).unwrap();
    }

    let vis = if s.is_pub { "pub " } else { "" };
    writeln!(out, "{}{}struct {} {{", pad, vis, s.name).unwrap();

    for (name, ty, is_pub) in &s.fields {
        let field_vis = if *is_pub { "pub " } else { "" };
        writeln!(out, "{}    {}{}: {},", pad, field_vis, name, ty).unwrap();
    }

    writeln!(out, "{}}}", pad).unwrap();
    out
}

fn render_impl_block(i: &ImplBlock, indent: usize) -> String {
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
        out.push_str(&render_fn_def(func, indent + 1));
    }

    writeln!(out, "{}}}", pad).unwrap();
    out
}

// ===========================================================================
// Statement rendering
// ===========================================================================

fn render_stmt(stmt: &Stmt, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::Let {
            name,
            mutable,
            expr,
        } => {
            let mut_kw = if *mutable { "mut " } else { "" };
            format!("{}let {}{} = {};\n", pad, mut_kw, name, render_expr(expr))
        }
        Stmt::Bind {
            targets,
            intent,
            expr,
        } => {
            let lhs = render_rust_bind_targets(targets);
            match intent {
                BindIntent::Declare => format!("{}let {} = {};\n", pad, lhs, render_expr(expr)),
                BindIntent::Assign => format!("{}{} = {};\n", pad, lhs, render_expr(expr)),
            }
        }
        Stmt::Assign { dest, value } => {
            if let Expr::BinOp { left, op, right } = value {
                let dest_str = render_expr(dest);
                if render_expr(left) == dest_str {
                    return format!("{}{} {}= {};\n", pad, dest_str, op, render_expr(right));
                }
            }
            format!("{}{} = {};\n", pad, render_expr(dest), render_expr(value))
        }
        Stmt::BlockScope(body) => {
            let mut out = format!("{} {{\n", pad);
            for s in body {
                out.push_str(&render_stmt(s, indent + 1));
            }
            writeln!(out, "{}}}\n", pad).unwrap();
            out
        }
        Stmt::Expr(expr) => {
            format!("{}{};\n", pad, render_expr(expr))
        }
        Stmt::Assert(assert) => render_assert(assert, indent),
        Stmt::Comment(text) => {
            if text.is_empty() {
                format!("{}\n", pad)
            } else {
                format!("{}// {}\n", pad, text)
            }
        }
        Stmt::Blank => "\n".to_string(),
        Stmt::Return(expr) => {
            let rendered = render_expr(expr);
            if rendered == "()" {
                format!("{}return;\n", pad)
            } else {
                format!("{}return {};\n", pad, rendered)
            }
        }
        Stmt::TailExpr(expr) => {
            format!("{}{}\n", pad, render_expr(expr))
        }
        Stmt::For {
            binding,
            iter,
            body,
        } => {
            let mut out = format!("{}for {} in {} {{\n", pad, binding, render_expr(iter));
            for s in body {
                out.push_str(&render_stmt(s, indent + 1));
            }
            writeln!(out, "{}}}", pad).unwrap();
            out
        }
        Stmt::Item(item) => render_item(item, indent),
    }
}

fn render_rust_bind_targets(targets: &[BindTarget]) -> String {
    if targets.len() == 1 {
        return render_rust_bind_target(&targets[0]);
    }
    let rendered = targets
        .iter()
        .map(render_rust_bind_target)
        .collect::<Vec<_>>()
        .join(", ");
    format!("({rendered})")
}

fn render_rust_bind_target(target: &BindTarget) -> String {
    match target {
        BindTarget::Name(name) => name.clone(),
        BindTarget::Discard => "_".to_string(),
    }
}

// ===========================================================================
// Expression rendering
// ===========================================================================

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Value(v) => render_value_expr(v),
        Expr::Var(name) => name.clone(),
        Expr::Str(s) => format!("\"{}\"", escape_rust_str(s)),
        Expr::Call { func, args, .. } => {
            let func_str = render_expr(func);
            let args_str: Vec<String> = args.iter().map(render_expr).collect();
            format!("{}({})", func_str, args_str.join(", "))
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            // C1.7: Special `?` operator rendering.
            if method == "?" {
                return format!("{}?", render_expr(receiver));
            }
            let recv = render_expr(receiver);
            let args_str: Vec<String> = args.iter().map(render_expr).collect();
            format!("{}.{}({})", recv, method, args_str.join(", "))
        }
        Expr::Field(expr, field) => {
            format!("{}.{}", render_expr(expr), field)
        }
        Expr::Deref(expr) => format!("*{}", render_expr(expr)),
        Expr::Ref(expr) => format!("&{}", render_expr(expr)),
        Expr::RefMut(expr) => format!("&mut {}", render_expr(expr)),
        Expr::Path(segments) => segments.join("::"),
        Expr::Struct { name, fields, rest } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| {
                    let rendered = render_expr(v);
                    if *k == rendered {
                        k.clone()
                    } else {
                        format!("{}: {}", k, rendered)
                    }
                })
                .collect();
            if let Some(base) = rest {
                if field_strs.is_empty() {
                    format!("{} {{ ..{} }}", name, render_expr(base))
                } else {
                    format!(
                        "{} {{ {}, ..{} }}",
                        name,
                        field_strs.join(", "),
                        render_expr(base)
                    )
                }
            } else {
                format!("{} {{ {} }}", name, field_strs.join(", "))
            }
        }
        Expr::Closure { args, body } => {
            let body_str = render_expr(body);
            if args.is_empty() {
                format!("|| {}", body_str)
            } else {
                format!("|{}| {}", args.join(", "), body_str)
            }
        }
        Expr::BinOp { left, op, right } => {
            format!("{} {} {}", render_expr(left), op, render_expr(right))
        }
        Expr::UnaryOp { op, expr } => {
            format!("{}{}", op, render_expr(expr))
        }
        Expr::IntLit(n) => n.to_string(),
        Expr::BoolLit(b) => b.to_string(),
        Expr::Match { expr, arms } => render_match(expr, arms),
        Expr::If {
            cond,
            then_body,
            else_body,
        } => render_if(cond, then_body, else_body.as_deref()),
        Expr::Block(stmts) => {
            let mut out = "{\n".to_string();
            for stmt in stmts {
                out.push_str(&render_stmt(stmt, 1));
            }
            out.push('}');
            out
        }
        Expr::FormatStr { template, args } => {
            if args.is_empty() {
                format!("format!(\"{}\")", escape_rust_str(template))
            } else {
                let args_str: Vec<String> = args.iter().map(render_expr).collect();
                format!(
                    "format!(\"{}\", {})",
                    escape_rust_str(template),
                    args_str.join(", ")
                )
            }
        }
        Expr::MacroCall { name, args } => {
            let args_str: Vec<String> = args.iter().map(render_expr).collect();
            format!("{}!({})", name, args_str.join(", "))
        }
        Expr::Tuple(items) => {
            let items_str: Vec<String> = items.iter().map(render_expr).collect();
            format!("({})", items_str.join(", "))
        }
        Expr::Array(items) => {
            let items_str: Vec<String> = items.iter().map(render_expr).collect();
            format!("[{}]", items_str.join(", "))
        }
        Expr::RawCode(code) => code.clone(),
    }
}

fn render_match(expr: &Expr, arms: &[MatchArm]) -> String {
    let mut out = format!("match {} {{\n", render_expr(expr));
    for arm in arms {
        writeln!(out, "    {} => {{", arm.pattern).unwrap();
        for stmt in &arm.body {
            out.push_str(&render_stmt(stmt, 2));
        }
        out.push_str("    }\n");
    }
    out.push('}');
    out
}

fn render_if(cond: &Expr, then_body: &[Stmt], else_body: Option<&[Stmt]>) -> String {
    if matches!(cond, Expr::Block(_)) {
        let cond_rendered = render_expr(cond);
        let mut out = format!("let __cond = {};\nif __cond {{\n", cond_rendered);
        for stmt in then_body {
            out.push_str(&render_stmt(stmt, 1));
        }
        if let Some(else_stmts) = else_body {
            out.push_str("} else {\n");
            for stmt in else_stmts {
                out.push_str(&render_stmt(stmt, 1));
            }
        }
        out.push('}');
        return out;
    }
    let mut out = format!("if {} {{\n", render_expr(cond));
    for stmt in then_body {
        out.push_str(&render_stmt(stmt, 1));
    }
    if let Some(else_stmts) = else_body {
        out.push_str("} else {\n");
        for stmt in else_stmts {
            out.push_str(&render_stmt(stmt, 1));
        }
    }
    out.push('}');
    out
}

// ===========================================================================
// Assert rendering
// ===========================================================================

fn render_assert(assert: &Assert, indent: usize) -> String {
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
                render_expr(left),
                render_expr(right),
                escape_rust_str(message)
            )
        }
        Assert::True { expr, message } => {
            format!(
                "{}assert!({}, \"{}\");\n",
                pad,
                render_expr(expr),
                escape_rust_str(message)
            )
        }
        Assert::NonEmpty { expr, message } => {
            format!(
                "{}assert!(!{}.is_empty(), \"{}\");\n",
                pad,
                render_expr(expr),
                escape_rust_str(message)
            )
        }
        Assert::Contains {
            expr,
            substring,
            message,
        } => {
            format!(
                "{}assert!({}.contains(\"{}\"), \"{}\");\n",
                pad,
                render_expr(expr),
                escape_rust_str(substring),
                escape_rust_str(message)
            )
        }
    }
}

// ===========================================================================
// ValueExpr rendering (bare mode — native Rust types)
// ===========================================================================

fn render_value_expr(expr: &ValueExpr) -> String {
    match expr {
        ValueExpr::Unit => "()".to_string(),
        ValueExpr::Bool(b) => b.to_string(),
        ValueExpr::Str(s) => format!("\"{}\".to_string()", escape_rust_str(s)),
        ValueExpr::Int(i) => i.to_string(),
        ValueExpr::List(items) => {
            let rendered: Vec<String> = items.iter().map(render_value_expr).collect();
            format!("vec![{}]", rendered.join(", "))
        }
        ValueExpr::Map(entries) => {
            if entries.is_empty() {
                return "std::collections::BTreeMap::new()".to_string();
            }
            let rendered: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    format!(
                        "(\"{}\".to_string(), {})",
                        escape_rust_str(k),
                        render_value_expr(v)
                    )
                })
                .collect();
            format!(
                "std::collections::BTreeMap::from([{}])",
                rendered.join(", ")
            )
        }
        ValueExpr::Json(json) => format!("serde_json::json!({})", json),
        ValueExpr::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| {
                    let rendered = render_value_expr(v);
                    if *k == rendered {
                        k.clone()
                    } else {
                        format!("{}: {}", k, rendered)
                    }
                })
                .collect();
            format!("{} {{ {} }}", name, field_strs.join(", "))
        }
        ValueExpr::Secret(s) => {
            format!("SecretString::new(\"{}\")", escape_rust_str(s))
        }
        ValueExpr::Enum { ty: _, variant } => {
            format!("\"{}\".to_string()", escape_rust_str(variant))
        }
        ValueExpr::Skipped => "Value::Skipped".to_string(),
    }
}

// ===========================================================================
// Tests (C1.7)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::code_ir::{EnumDef, FnDef, ImplBlock, StructDef};

    // -- C1.1: Function rendering --

    #[test]
    fn render_fn_simple() {
        let f = FnDef {
            name: "greet".to_string(),
            is_pub: true,
            params: vec![("name".to_string(), "String".to_string())],
            return_type: Some("String".to_string()),
            body: vec![Stmt::TailExpr(Expr::FormatStr {
                template: "Hello, {}!".to_string(),
                args: vec![Expr::var("name")],
            })],
            doc: vec!["Greet a user.".to_string()],
            attributes: vec![],
        };
        let rendered = render_fn_def(&f, 0);
        assert!(rendered.contains("/// Greet a user."), "doc comment");
        assert!(
            rendered.contains("pub fn greet(name: String) -> String {"),
            "signature"
        );
        assert!(rendered.contains("format!(\"Hello, {}!\", name)"), "body");
    }

    // -- C1.2: Struct rendering --

    #[test]
    fn render_struct_with_derives() {
        let s = StructDef {
            name: "Config".to_string(),
            is_pub: true,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields: vec![
                ("name".to_string(), "String".to_string(), true),
                ("count".to_string(), "i64".to_string(), false),
            ],
            doc: vec!["Configuration.".to_string()],
        };
        let rendered = render_struct_def(&s, 0);
        assert!(rendered.contains("#[derive(Debug, Clone)]"), "derives");
        assert!(rendered.contains("pub struct Config {"), "header");
        assert!(rendered.contains("pub name: String,"), "pub field");
        assert!(rendered.contains("    count: i64,"), "private field");
    }

    // -- C1.3: Enum rendering --

    #[test]
    fn render_enum_with_variants() {
        let e = EnumDef {
            name: "Op".to_string(),
            is_pub: true,
            derives: vec!["Debug".to_string()],
            variants: vec!["Read".to_string(), "Write(String)".to_string()],
            doc: vec![],
        };
        let rendered = render_enum_def(&e, 0);
        assert!(rendered.contains("#[derive(Debug)]"), "derive");
        assert!(rendered.contains("pub enum Op {"), "header");
        assert!(rendered.contains("    Read,"), "variant 1");
        assert!(rendered.contains("    Write(String),"), "variant 2");
    }

    // -- C1.4: Impl block rendering --

    #[test]
    fn render_impl_block_with_trait() {
        let i = ImplBlock {
            type_name: "Config".to_string(),
            trait_name: Some("Default".to_string()),
            items: vec![FnDef {
                name: "default".to_string(),
                is_pub: false,
                params: vec![],
                return_type: Some("Self".to_string()),
                body: vec![Stmt::TailExpr(Expr::Struct {
                    name: "Config".to_string(),
                    fields: vec![("name".to_string(), Expr::str_lit("default"))],
                    rest: None,
                })],
                doc: vec![],
                attributes: vec![],
            }],
        };
        let rendered = render_impl_block(&i, 0);
        assert!(rendered.contains("impl Default for Config {"), "header");
        assert!(rendered.contains("fn default() -> Self {"), "method sig");
    }

    // -- C1.5: Import rendering --

    #[test]
    fn render_import_single_item() {
        let import = Import {
            path: vec!["std".to_string(), "fmt".to_string()],
            items: vec!["Write".to_string()],
        };
        assert_eq!(render_import(&import), "use std::fmt::Write;");
    }

    #[test]
    fn render_import_multiple_items() {
        let import = Import {
            path: vec!["gunbc_exec".to_string()],
            items: vec!["execute_transport".to_string(), "ExecError".to_string()],
        };
        assert_eq!(
            render_import(&import),
            "use gunbc_exec::{execute_transport, ExecError};"
        );
    }

    // -- C1.6: Expression rendering --

    #[test]
    fn render_expr_method_chain() {
        let expr =
            Expr::var("mocks").method("insert", vec![Expr::str_lit("node"), Expr::str_lit("port")]);
        assert_eq!(render_expr(&expr), "mocks.insert(\"node\", \"port\")");
    }

    #[test]
    fn render_expr_deref_ref() {
        assert_eq!(render_expr(&Expr::var("x").deref()), "*x");
        assert_eq!(render_expr(&Expr::var("x").ref_of()), "&x");
        assert_eq!(render_expr(&Expr::var("x").ref_mut()), "&mut x");
    }

    #[test]
    fn render_expr_path() {
        let expr = Expr::path(&["FileRequest", "read"]);
        assert_eq!(render_expr(&expr), "FileRequest::read");
    }

    #[test]
    fn render_expr_closure() {
        let expr = Expr::Closure {
            args: vec!["n".to_string()],
            body: Box::new(Expr::var("n").bin_op(">=", Expr::int(2))),
        };
        assert_eq!(render_expr(&expr), "|n| n >= 2");
    }

    // -- C1.7: Question mark operator --

    #[test]
    fn render_question_mark_operator() {
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::call("execute_transport", vec![Expr::var("req")])),
            method: "?".to_string(),
            args: vec![],
        };
        assert_eq!(render_expr(&expr), "execute_transport(req)?");
    }

    // -- RV-1: Struct update syntax (rest) --

    #[test]
    fn render_struct_update_empty_fields_no_spurious_comma() {
        // Bug case: empty fields + rest should produce `{ ..base }`, not `{ , ..base }`.
        let expr = Expr::Struct {
            name: "Config".to_string(),
            fields: vec![],
            rest: Some(Box::new(Expr::var("defaults"))),
        };
        let rendered = render_expr(&expr);
        assert_eq!(rendered, "Config { ..defaults }");
        assert!(
            !rendered.contains(", .."),
            "must not contain spurious comma before ..base"
        );
    }

    #[test]
    fn render_struct_update_with_fields() {
        // Normal case: populated fields + rest.
        let expr = Expr::Struct {
            name: "Config".to_string(),
            fields: vec![
                ("name".to_string(), Expr::str_lit("custom")),
                ("count".to_string(), Expr::int(42)),
            ],
            rest: Some(Box::new(Expr::var("defaults"))),
        };
        let rendered = render_expr(&expr);
        assert_eq!(
            rendered,
            "Config { name: \"custom\", count: 42, ..defaults }"
        );
    }

    // -- ValueExpr rendering --

    #[test]
    fn render_value_expr_primitives() {
        assert_eq!(render_value_expr(&ValueExpr::Unit), "()");
        assert_eq!(render_value_expr(&ValueExpr::Bool(true)), "true");
        assert_eq!(render_value_expr(&ValueExpr::Int(42)), "42");
        assert_eq!(
            render_value_expr(&ValueExpr::Str("hello".into())),
            "\"hello\".to_string()"
        );
    }

    #[test]
    fn render_value_expr_list() {
        let expr = ValueExpr::List(vec![ValueExpr::Int(1), ValueExpr::Int(2)]);
        assert_eq!(render_value_expr(&expr), "vec![1, 2]");
    }

    // -- Integration: full SourceFile rendering --

    #[test]
    fn render_full_source_file() {
        let source = SourceFile {
            doc: vec!["Generated from makegen.dag".to_string()],
            items: vec![
                Item::Use(Import {
                    path: vec!["gunbc_exec".to_string()],
                    items: vec!["execute_transport".to_string(), "ExecError".to_string()],
                }),
                Item::Fn(FnDef {
                    name: "main".to_string(),
                    is_pub: true,
                    params: vec![("path".to_string(), "String".to_string())],
                    return_type: Some("Result<(), ExecError>".to_string()),
                    body: vec![
                        Stmt::let_bind(
                            "req",
                            Expr::call("FileRequest::read", vec![Expr::var("path")]),
                        ),
                        Stmt::let_bind(
                            "resp",
                            Expr::MethodCall {
                                receiver: Box::new(Expr::call(
                                    "execute_transport",
                                    vec![Expr::var("req")],
                                )),
                                method: "?".to_string(),
                                args: vec![],
                            },
                        ),
                        Stmt::TailExpr(Expr::call("Ok", vec![Expr::Tuple(vec![])])),
                    ],
                    doc: vec!["Entry point.".to_string()],
                    attributes: vec![],
                }),
            ],
        };

        let rendered = render_rust_source(&source);
        assert!(rendered.contains("//! Generated from makegen.dag"));
        assert!(rendered.contains("use gunbc_exec::{execute_transport, ExecError};"));
        assert!(rendered.contains("pub fn main(path: String) -> Result<(), ExecError> {"));
        assert!(rendered.contains("execute_transport(req)?"));
        assert!(rendered.contains("Ok(())"));
    }
}
