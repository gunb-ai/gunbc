//! Go CodeRenderer — renders Go-specific SourceFile to .go text.
//!
//! Standalone renderer for daglang-emit: takes a `SourceFile` (after
//! `lower_to_go`) and produces valid `.go` source text. Also provides
//! `render_go_mod` for generating a minimal go.mod.
//!
//! Go-specific rendering conventions:
//! - `Stmt::Let` → short variable declaration: `name := expr`
//!   (multi-return convention: name contains ", " → rendered as-is)
//! - `Stmt::TailExpr` → explicit return (handled by lowering, but fallback here)
//! - No semicolons after statements (Go doesn't use them)
//! - `Item::Raw` rendered as-is (for package declaration, const iota blocks)
//! - Go `import (...)` block from `Import` where path items are Go import strings
//!
//! **Owned by**: Task 13 (dsl-codegen-tasks.md)

use gunbc_ir::code_ir::{
    Assert, EnumDef, Expr, FnDef, ImplBlock, Import, Item, MatchArm, SourceFile, Stmt, StructDef,
};
use gunbc_ir::ValueExpr;
use std::fmt::Write;

// ===========================================================================
// Public API
// ===========================================================================

/// Render a `SourceFile` to a complete `.go` source string.
pub fn render_go_source(source: &SourceFile) -> String {
    let mut out = String::new();

    // Module-level doc comments.
    for line in &source.doc {
        writeln!(out, "// {}", line).unwrap();
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

/// Render a minimal go.mod for a generated module.
pub fn render_go_mod(module_path: &str, go_version: &str) -> String {
    let mut out = String::new();
    writeln!(out, "module {}", module_path).unwrap();
    out.push('\n');
    writeln!(out, "go {}", go_version).unwrap();
    out
}

// ===========================================================================
// String escaping
// ===========================================================================

fn escape_go_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

// ===========================================================================
// Item rendering
// ===========================================================================

fn render_item(item: &Item, indent: usize) -> String {
    let pad = "\t".repeat(indent);
    match item {
        Item::Use(import) => render_go_import(import),
        Item::Fn(f) => render_fn_def(f, indent),
        Item::Enum(e) => render_enum_def(e, indent),
        Item::Impl(i) => render_impl_block(i, indent),
        Item::Struct(s) => render_struct_def(s, indent),
        Item::Raw(code) => format!("{}{}\n", pad, code),
    }
}

/// Go import block: `import ("pkg1" "pkg2" ...)`.
/// The Import.path holds the list of Go import strings (Import.items is empty for Go).
fn render_go_import(import: &Import) -> String {
    if import.path.len() == 1 {
        format!("import \"{}\"\n", import.path[0])
    } else {
        let mut out = "import (\n".to_string();
        for pkg in &import.path {
            writeln!(out, "\t\"{}\"", pkg).unwrap();
        }
        out.push_str(")\n");
        out
    }
}

fn render_fn_def(f: &FnDef, indent: usize) -> String {
    let pad = "\t".repeat(indent);
    let mut out = String::new();

    // Doc comments.
    for line in &f.doc {
        writeln!(out, "{}// {}", pad, line).unwrap();
    }

    // Go: exported functions start with uppercase (already handled by lowering).
    let params: Vec<String> = f
        .params
        .iter()
        .map(|(name, ty)| {
            if ty.is_empty() {
                name.clone()
            } else {
                format!("{} {}", name, ty)
            }
        })
        .collect();
    let ret = match &f.return_type {
        Some(ty) => format!(" {}", ty),
        None => String::new(),
    };
    writeln!(
        out,
        "{}func {}({}){} {{",
        pad,
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
    // Go enums are rendered as const iota blocks by the lowering pass (Item::Raw).
    // If we reach here, render a basic type + const block.
    let pad = "\t".repeat(indent);
    let mut out = String::new();

    for line in &e.doc {
        writeln!(out, "{}// {}", pad, line).unwrap();
    }

    writeln!(out, "{}type {} int", pad, e.name).unwrap();
    out.push('\n');
    writeln!(out, "{}const (", pad).unwrap();
    for (i, variant) in e.variants.iter().enumerate() {
        if i == 0 {
            writeln!(out, "{}\t{}{} {} = iota", pad, e.name, variant, e.name).unwrap();
        } else {
            writeln!(out, "{}\t{}{}", pad, e.name, variant).unwrap();
        }
    }
    writeln!(out, "{})", pad).unwrap();
    out
}

fn render_struct_def(s: &StructDef, indent: usize) -> String {
    let pad = "\t".repeat(indent);
    let mut out = String::new();

    for line in &s.doc {
        writeln!(out, "{}// {}", pad, line).unwrap();
    }

    writeln!(out, "{}type {} struct {{", pad, s.name).unwrap();

    for (name, ty, _is_pub) in &s.fields {
        // Go: exported fields start with uppercase (already handled by lowering).
        writeln!(out, "{}\t{} {}", pad, name, ty).unwrap();
    }

    writeln!(out, "{}}}", pad).unwrap();
    out
}

fn render_impl_block(i: &ImplBlock, indent: usize) -> String {
    let pad = "\t".repeat(indent);
    let mut out = String::new();

    // Go doesn't have impl blocks. Render as methods with receiver.
    for func in &i.items {
        for line in &func.doc {
            writeln!(out, "{}// {}", pad, line).unwrap();
        }

        let receiver = format!("(self *{})", i.type_name);
        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| {
                if ty.is_empty() {
                    name.clone()
                } else {
                    format!("{} {}", name, ty)
                }
            })
            .collect();
        let ret = match &func.return_type {
            Some(ty) => format!(" {}", ty),
            None => String::new(),
        };
        writeln!(
            out,
            "{}func {} {}({}){} {{",
            pad,
            receiver,
            func.name,
            params.join(", "),
            ret
        )
        .unwrap();

        for stmt in &func.body {
            out.push_str(&render_stmt(stmt, indent + 1));
        }

        writeln!(out, "{}}}", pad).unwrap();
        out.push('\n');
    }

    out
}

// ===========================================================================
// Statement rendering
// ===========================================================================

fn render_stmt(stmt: &Stmt, indent: usize) -> String {
    let pad = "\t".repeat(indent);
    match stmt {
        Stmt::Let { name, expr, .. } => {
            // Go short variable declaration: `name := expr`
            format!("{}{} := {}\n", pad, name, render_expr(expr))
        }
        Stmt::Expr(expr) => {
            format!("{}{}\n", pad, render_expr(expr))
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
            format!("{}return {}\n", pad, rendered)
        }
        Stmt::TailExpr(expr) => {
            // Go doesn't have implicit returns — render as explicit return.
            format!("{}return {}\n", pad, render_expr(expr))
        }
        Stmt::For {
            binding,
            iter,
            body,
        } => {
            let mut out = format!(
                "{}for _, {} := range {} {{\n",
                pad,
                binding,
                render_expr(iter)
            );
            for s in body {
                out.push_str(&render_stmt(s, indent + 1));
            }
            writeln!(out, "{}}}", pad).unwrap();
            out
        }
        Stmt::Item(item) => render_item(item, indent),
    }
}

// ===========================================================================
// Expression rendering
// ===========================================================================

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Value(v) => render_value_expr(v),
        Expr::Var(name) => name.clone(),
        Expr::Str(s) => format!("\"{}\"", escape_go_str(s)),
        Expr::Call { func, args } => {
            let func_str = render_expr(func);
            let args_str: Vec<String> = args.iter().map(render_expr).collect();
            format!("{}({})", func_str, args_str.join(", "))
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let recv = render_expr(receiver);
            let args_str: Vec<String> = args.iter().map(render_expr).collect();
            format!("{}.{}({})", recv, method, args_str.join(", "))
        }
        Expr::Field(expr, field) => {
            format!("{}.{}", render_expr(expr), field)
        }
        Expr::Deref(expr) => format!("*{}", render_expr(expr)),
        Expr::Ref(expr) => format!("&{}", render_expr(expr)),
        Expr::RefMut(expr) => format!("&{}", render_expr(expr)), // Go has no &mut.
        Expr::Path(segments) => segments.join("."),              // Go uses dots, not ::.
        Expr::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, render_expr(v)))
                .collect();
            format!("{}{{ {} }}", name, field_strs.join(", "))
        }
        Expr::Closure { args, body } => {
            let body_str = render_expr(body);
            if args.is_empty() {
                format!("func() {{ return {} }}", body_str)
            } else {
                let params: Vec<String> = args
                    .iter()
                    .map(|a| format!("{} interface{{}}", a))
                    .collect();
                format!(
                    "func({}) interface{{}} {{ return {} }}",
                    params.join(", "),
                    body_str
                )
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
        Expr::Match { expr, arms } => render_switch(expr, arms),
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
            // Should be lowered to fmt.Sprintf by lower_go, but render directly if not.
            if args.is_empty() {
                format!("fmt.Sprintf(\"{}\")", escape_go_str(template))
            } else {
                let args_str: Vec<String> = args.iter().map(render_expr).collect();
                format!(
                    "fmt.Sprintf(\"{}\", {})",
                    escape_go_str(template),
                    args_str.join(", ")
                )
            }
        }
        Expr::MacroCall { name, args } => {
            // Go has no macros — render as function call.
            let args_str: Vec<String> = args.iter().map(render_expr).collect();
            format!("{}({})", name, args_str.join(", "))
        }
        Expr::Tuple(items) => {
            // Go doesn't have tuples — render as comma-separated values.
            let items_str: Vec<String> = items.iter().map(render_expr).collect();
            items_str.join(", ")
        }
        Expr::Array(items) => {
            let items_str: Vec<String> = items.iter().map(render_expr).collect();
            format!("[]interface{{}}{{ {} }}", items_str.join(", "))
        }
        Expr::RawCode(code) => code.clone(),
    }
}

fn render_switch(expr: &Expr, arms: &[MatchArm]) -> String {
    let mut out = format!("switch {} {{\n", render_expr(expr));
    for arm in arms {
        writeln!(out, "case {}:", arm.pattern).unwrap();
        for stmt in &arm.body {
            out.push_str(&render_stmt(stmt, 1));
        }
    }
    out.push('}');
    out
}

fn render_if(cond: &Expr, then_body: &[Stmt], else_body: Option<&[Stmt]>) -> String {
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
    let pad = "\t".repeat(indent);
    match assert {
        Assert::Eq {
            left,
            right,
            message,
        } => {
            format!(
                "{}if {} != {} {{\n{}\tt.Errorf(\"{}\")\n{}}}\n",
                pad,
                render_expr(left),
                render_expr(right),
                pad,
                escape_go_str(message),
                pad
            )
        }
        Assert::True { expr, message } => {
            format!(
                "{}if !({}) {{\n{}\tt.Errorf(\"{}\")\n{}}}\n",
                pad,
                render_expr(expr),
                pad,
                escape_go_str(message),
                pad
            )
        }
        Assert::NonEmpty { expr, message } => {
            format!(
                "{}if len({}) == 0 {{\n{}\tt.Errorf(\"{}\")\n{}}}\n",
                pad,
                render_expr(expr),
                pad,
                escape_go_str(message),
                pad
            )
        }
        Assert::Contains {
            expr,
            substring,
            message,
        } => {
            format!(
                "{}if !strings.Contains({}, \"{}\") {{\n{}\tt.Errorf(\"{}\")\n{}}}\n",
                pad,
                render_expr(expr),
                escape_go_str(substring),
                pad,
                escape_go_str(message),
                pad
            )
        }
    }
}

// ===========================================================================
// ValueExpr rendering (Go native types)
// ===========================================================================

fn render_value_expr(expr: &ValueExpr) -> String {
    match expr {
        ValueExpr::Unit => "nil".to_string(),
        ValueExpr::Bool(b) => b.to_string(),
        ValueExpr::Str(s) => format!("\"{}\"", escape_go_str(s)),
        ValueExpr::Int(i) => i.to_string(),
        ValueExpr::List(items) => {
            let rendered: Vec<String> = items.iter().map(render_value_expr).collect();
            format!("[]interface{{}}{{ {} }}", rendered.join(", "))
        }
        ValueExpr::Map(entries) => {
            if entries.is_empty() {
                return "map[string]interface{}{}".to_string();
            }
            let rendered: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", escape_go_str(k), render_value_expr(v)))
                .collect();
            format!("map[string]interface{{}}{{ {} }}", rendered.join(", "))
        }
        ValueExpr::Json(json) => format!("json.RawMessage(`{}`)", json),
        ValueExpr::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, render_value_expr(v)))
                .collect();
            format!("{}{{ {} }}", name, field_strs.join(", "))
        }
        ValueExpr::Secret(s) => format!("NewSecret(\"{}\")", escape_go_str(s)),
        ValueExpr::Skipped => "nil /* skipped */".to_string(),
    }
}

// ===========================================================================
// Tests (C2.8)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::code_ir::{FnDef, StructDef};

    // -- C2.1: Function rendering with multi-return --

    #[test]
    fn render_fn_with_error_return() {
        let f = FnDef {
            name: "Main".to_string(),
            is_pub: true,
            params: vec![("filePath".to_string(), "string".to_string())],
            return_type: Some("error".to_string()),
            body: vec![
                Stmt::Let {
                    name: "resp, err".to_string(),
                    mutable: false,
                    expr: Expr::call("transport.Execute", vec![Expr::var("req")]),
                },
                Stmt::Return(Expr::var("nil")),
            ],
            doc: vec!["Entry point.".to_string()],
            attributes: vec![],
        };
        let rendered = render_fn_def(&f, 0);
        assert!(rendered.contains("// Entry point."), "doc comment");
        assert!(
            rendered.contains("func Main(filePath string) error {"),
            "signature: got {rendered}"
        );
        assert!(
            rendered.contains("resp, err := transport.Execute(req)"),
            "multi-return: got {rendered}"
        );
        assert!(rendered.contains("return nil"), "return nil");
    }

    // -- C2.2: Struct rendering --

    #[test]
    fn render_go_struct() {
        let s = StructDef {
            name: "Config".to_string(),
            is_pub: true,
            derives: vec![], // Go has no derives.
            fields: vec![
                ("Name".to_string(), "string".to_string(), true),
                ("count".to_string(), "int64".to_string(), false),
            ],
            doc: vec!["Configuration.".to_string()],
        };
        let rendered = render_struct_def(&s, 0);
        assert!(rendered.contains("// Configuration."), "doc");
        assert!(rendered.contains("type Config struct {"), "header");
        assert!(rendered.contains("\tName string"), "field");
        assert!(rendered.contains("\tcount int64"), "field");
    }

    // -- C2.3: Error handling idiom --

    #[test]
    fn render_if_err_nil() {
        let stmt = Stmt::Expr(Expr::If {
            cond: Box::new(Expr::BinOp {
                left: Box::new(Expr::var("err")),
                op: "!=".to_string(),
                right: Box::new(Expr::var("nil")),
            }),
            then_body: vec![Stmt::Return(Expr::var("err"))],
            else_body: None,
        });
        let rendered = render_stmt(&stmt, 0);
        assert!(rendered.contains("if err != nil {"), "condition");
        assert!(rendered.contains("return err"), "return err");
    }

    // -- C2.4: Import rendering --

    #[test]
    fn render_single_import() {
        let import = Import {
            path: vec!["fmt".to_string()],
            items: vec![],
        };
        assert_eq!(render_go_import(&import), "import \"fmt\"\n");
    }

    #[test]
    fn render_multi_import() {
        let import = Import {
            path: vec![
                "fmt".to_string(),
                "github.com/gunb-ai/gunbc/transport".to_string(),
            ],
            items: vec![],
        };
        let rendered = render_go_import(&import);
        assert!(rendered.contains("import ("), "block start");
        assert!(rendered.contains("\t\"fmt\""), "fmt import");
        assert!(
            rendered.contains("\"github.com/gunb-ai/gunbc/transport\""),
            "transport import"
        );
        assert!(rendered.contains(")"), "block end");
    }

    // -- C2.5: go.mod rendering --

    #[test]
    fn render_go_mod_file() {
        let rendered = render_go_mod("github.com/gunb-ai/gunbc/generated/makegen", "1.21");
        assert!(rendered.contains("module github.com/gunb-ai/gunbc/generated/makegen"));
        assert!(rendered.contains("go 1.21"));
    }

    // -- C2.6/C2.7: Expression rendering --

    #[test]
    fn render_expr_path_uses_dots() {
        let expr = Expr::path(&["transport", "Execute"]);
        assert_eq!(render_expr(&expr), "transport.Execute");
    }

    #[test]
    fn render_for_loop_with_range() {
        let stmt = Stmt::For {
            binding: "item".to_string(),
            iter: Expr::var("items"),
            body: vec![Stmt::Expr(Expr::call(
                "fmt.Println",
                vec![Expr::var("item")],
            ))],
        };
        let rendered = render_stmt(&stmt, 0);
        assert!(rendered.contains("for _, item := range items {"), "range");
        assert!(rendered.contains("fmt.Println(item)"), "body");
    }

    // -- C2.8: Integration test --

    #[test]
    fn render_full_go_source() {
        let source = SourceFile {
            doc: vec!["Generated from makegen.dag".to_string()],
            items: vec![
                Item::Raw("package main".to_string()),
                Item::Use(Import {
                    path: vec![
                        "fmt".to_string(),
                        "github.com/gunb-ai/gunbc/transport".to_string(),
                    ],
                    items: vec![],
                }),
                Item::Fn(FnDef {
                    name: "Main".to_string(),
                    is_pub: true,
                    params: vec![("filePath".to_string(), "string".to_string())],
                    return_type: Some("error".to_string()),
                    body: vec![
                        Stmt::Let {
                            name: "req, err".to_string(),
                            mutable: false,
                            expr: Expr::call(
                                "transport.NewFileReadRequest",
                                vec![Expr::var("filePath")],
                            ),
                        },
                        Stmt::Expr(Expr::If {
                            cond: Box::new(Expr::BinOp {
                                left: Box::new(Expr::var("err")),
                                op: "!=".to_string(),
                                right: Box::new(Expr::var("nil")),
                            }),
                            then_body: vec![Stmt::Return(Expr::var("err"))],
                            else_body: None,
                        }),
                        Stmt::let_bind(
                            "msg",
                            Expr::call(
                                "fmt.Sprintf",
                                vec![Expr::str_lit("Read: %v"), Expr::var("req")],
                            ),
                        ),
                        Stmt::Expr(Expr::call("fmt.Println", vec![Expr::var("msg")])),
                        Stmt::Return(Expr::var("nil")),
                    ],
                    doc: vec!["Entry point.".to_string()],
                    attributes: vec![],
                }),
            ],
        };

        let rendered = render_go_source(&source);
        assert!(rendered.contains("// Generated from makegen.dag"));
        assert!(rendered.contains("package main"));
        assert!(rendered.contains("import ("));
        assert!(rendered.contains("\"fmt\""));
        assert!(rendered.contains("func Main(filePath string) error {"));
        assert!(rendered.contains("req, err := transport.NewFileReadRequest(filePath)"));
        assert!(rendered.contains("if err != nil {"));
        assert!(rendered.contains("return err"));
        assert!(rendered.contains("return nil"));
    }

    // -- ValueExpr rendering --

    #[test]
    fn render_value_expr_go_primitives() {
        assert_eq!(render_value_expr(&ValueExpr::Unit), "nil");
        assert_eq!(render_value_expr(&ValueExpr::Bool(true)), "true");
        assert_eq!(render_value_expr(&ValueExpr::Int(42)), "42");
        assert_eq!(
            render_value_expr(&ValueExpr::Str("hello".into())),
            "\"hello\""
        );
    }
}
