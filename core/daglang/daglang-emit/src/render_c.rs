//! C CodeRenderer — renders CStyleIR (`CSourceFile`) to `.c` text.
//!
//! Standalone renderer for daglang-emit: takes a `CSourceFile` (after
//! `lower_to_c`) and produces valid C source text. Also provides
//! `render_c_makefile` for generating a minimal Makefile.
//!
//! C-specific rendering conventions:
//! - 4-space indentation (common C convention)
//! - Semicolons after every statement
//! - `#include` directives at the top
//! - `#define` for enum-like constants
//! - Forward declarations before definitions
//! - `static` qualifier for non-public functions
//!
//! **Owned by**: Task 14 (dsl-codegen-tasks.md)

use gunbc_ir::code_ir::c_ir::*;
use std::fmt::Write;

// ===========================================================================
// Public API
// ===========================================================================

/// Render a `CSourceFile` to a complete `.c` source string.
pub fn render_c_source(source: &CSourceFile) -> String {
    let mut out = String::new();

    // Render #include directives.
    for inc in &source.includes {
        out.push_str(&render_item(inc, 0));
    }
    if !source.includes.is_empty() {
        out.push('\n');
    }

    // Render items.
    for item in &source.items {
        out.push_str(&render_item(item, 0));
        out.push('\n');
    }

    out
}

/// Render a minimal Makefile for compiling the generated C source.
pub fn render_c_makefile(binary_name: &str, source_files: &[&str]) -> String {
    let mut out = String::new();
    let sources = source_files.join(" ");

    writeln!(out, "CC = gcc").unwrap();
    writeln!(out, "CFLAGS = -Wall -Wextra -std=c11 -O2").unwrap();
    writeln!(out, "TARGET = {}", binary_name).unwrap();
    writeln!(out, "SRCS = {}", sources).unwrap();
    out.push('\n');
    writeln!(out, "all: $(TARGET)").unwrap();
    out.push('\n');
    writeln!(out, "$(TARGET): $(SRCS)").unwrap();
    writeln!(out, "\t$(CC) $(CFLAGS) -o $@ $^").unwrap();
    out.push('\n');
    writeln!(out, "clean:").unwrap();
    writeln!(out, "\trm -f $(TARGET)").unwrap();
    out.push('\n');
    writeln!(out, ".PHONY: all clean").unwrap();

    out
}

// ===========================================================================
// String escaping
// ===========================================================================

fn escape_c_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\0', "\\0")
}

// ===========================================================================
// Item rendering
// ===========================================================================

fn render_item(item: &CItem, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match item {
        CItem::Include { path, system } => {
            if *system {
                format!("{}#include <{}>\n", pad, path)
            } else {
                format!("{}#include \"{}\"\n", pad, path)
            }
        }
        CItem::Typedef { name, ty } => {
            format!("{}typedef {} {};\n", pad, render_type(ty), name)
        }
        CItem::StructDef { name, fields } => render_struct_def(name, fields, indent),
        CItem::TaggedUnion {
            name,
            tag_name,
            variants,
        } => render_tagged_union(name, tag_name, variants, indent),
        CItem::FnDef(f) => render_fn_def(f, indent),
        CItem::FnDecl(d) => render_fn_decl(d, indent),
        CItem::Define { name, value } => {
            format!("{}#define {} {}\n", pad, name, value)
        }
        CItem::Comment(text) => {
            format!("{}/* {} */\n", pad, text)
        }
    }
}

// ===========================================================================
// C3.1: Function rendering
// ===========================================================================

fn render_fn_def(f: &CFnDef, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();

    let static_prefix = if f.is_static { "static " } else { "" };
    let params = render_params(&f.params);

    writeln!(
        out,
        "{}{}{} {}({}) {{",
        pad,
        static_prefix,
        render_type(&f.return_type),
        f.name,
        params
    )
    .unwrap();

    for stmt in &f.body {
        out.push_str(&render_stmt(stmt, indent + 1));
    }

    writeln!(out, "{}}}", pad).unwrap();
    out
}

fn render_fn_decl(d: &CFnDecl, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let params = render_params(&d.params);
    format!(
        "{}{} {}({});\n",
        pad,
        render_type(&d.return_type),
        d.name,
        params
    )
}

fn render_params(params: &[(String, CType)]) -> String {
    if params.is_empty() {
        return "void".to_string();
    }
    params
        .iter()
        .map(|(name, ty)| format!("{} {}", render_type(ty), name))
        .collect::<Vec<_>>()
        .join(", ")
}

// ===========================================================================
// C3.2: Struct rendering
// ===========================================================================

fn render_struct_def(name: &str, fields: &[(String, CType)], indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();

    writeln!(out, "{}typedef struct {{", pad).unwrap();
    for (field_name, field_type) in fields {
        writeln!(
            out,
            "{}    {} {};",
            pad,
            render_type(field_type),
            field_name
        )
        .unwrap();
    }
    writeln!(out, "{}}} {};", pad, name).unwrap();
    out
}

fn render_tagged_union(
    name: &str,
    tag_name: &str,
    variants: &[(String, Vec<(String, CType)>)],
    indent: usize,
) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();

    // Tag enum.
    writeln!(out, "{}typedef enum {{", pad).unwrap();
    for (i, (variant_name, _)) in variants.iter().enumerate() {
        let comma = if i + 1 < variants.len() { "," } else { "" };
        writeln!(out, "{}    {}_{}{}", pad, tag_name, variant_name, comma).unwrap();
    }
    writeln!(out, "{}}} {}_tag;", pad, name).unwrap();
    out.push('\n');

    // Tagged union struct.
    writeln!(out, "{}typedef struct {{", pad).unwrap();
    writeln!(out, "{}    {}_tag {};", pad, name, tag_name).unwrap();
    writeln!(out, "{}    union {{", pad).unwrap();

    for (variant_name, fields) in variants {
        if fields.is_empty() {
            continue;
        }
        writeln!(out, "{}        struct {{", pad).unwrap();
        for (field_name, field_type) in fields {
            writeln!(
                out,
                "{}            {} {};",
                pad,
                render_type(field_type),
                field_name
            )
            .unwrap();
        }
        writeln!(out, "{}        }} {};", pad, variant_name.to_lowercase()).unwrap();
    }

    writeln!(out, "{}    }} data;", pad).unwrap();
    writeln!(out, "{}}} {};", pad, name).unwrap();
    out
}

// ===========================================================================
// Statement rendering
// ===========================================================================

fn render_stmt(stmt: &CStmt, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        CStmt::Decl { name, ty, init } => match init {
            Some(expr) => format!(
                "{}{} {} = {};\n",
                pad,
                render_type(ty),
                name,
                render_expr(expr)
            ),
            None => format!("{}{} {};\n", pad, render_type(ty), name),
        },
        CStmt::Assign { lhs, rhs } => {
            format!("{}{} = {};\n", pad, render_expr(lhs), render_expr(rhs))
        }
        CStmt::Expr(expr) => {
            format!("{}{};\n", pad, render_expr(expr))
        }
        CStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let mut out = format!("{}if ({}) {{\n", pad, render_expr(cond));
            for s in then_body {
                out.push_str(&render_stmt(s, indent + 1));
            }
            if let Some(else_stmts) = else_body {
                writeln!(out, "{}}} else {{", pad).unwrap();
                for s in else_stmts {
                    out.push_str(&render_stmt(s, indent + 1));
                }
            }
            writeln!(out, "{}}}", pad).unwrap();
            out
        }
        CStmt::For {
            init,
            cond,
            step,
            body,
        } => {
            let init_str = render_stmt_inline(init);
            let step_str = render_stmt_inline(step);
            let mut out = format!(
                "{}for ({}; {}; {}) {{\n",
                pad,
                init_str,
                render_expr(cond),
                step_str
            );
            for s in body {
                out.push_str(&render_stmt(s, indent + 1));
            }
            writeln!(out, "{}}}", pad).unwrap();
            out
        }
        CStmt::While { cond, body } => {
            let mut out = format!("{}while ({}) {{\n", pad, render_expr(cond));
            for s in body {
                out.push_str(&render_stmt(s, indent + 1));
            }
            writeln!(out, "{}}}", pad).unwrap();
            out
        }
        CStmt::Return(Some(expr)) => {
            format!("{}return {};\n", pad, render_expr(expr))
        }
        CStmt::Return(None) => {
            format!("{}return;\n", pad)
        }
        CStmt::Goto(label) => {
            format!("{}goto {};\n", pad, label)
        }
        CStmt::Label(label) => {
            // Labels are outdented by one level (standard C convention).
            if indent > 0 {
                format!("{}{}:\n", "    ".repeat(indent - 1), label)
            } else {
                format!("{}:\n", label)
            }
        }
        CStmt::BlockScope(stmts) => {
            let mut out = format!("{} {{\n", pad);
            for s in stmts {
                out.push_str(&render_stmt(s, indent + 1));
            }
            writeln!(out, "{}}}\n", pad).unwrap();
            out
        }
        CStmt::Free(expr) => {
            format!("{}free({});\n", pad, render_expr(expr))
        }
        CStmt::Comment(text) => {
            format!("{}/* {} */\n", pad, text)
        }
        CStmt::Blank => "\n".to_string(),
    }
}

/// Render a statement inline (for `for` init/step — no trailing newline or semicolon).
fn render_stmt_inline(stmt: &CStmt) -> String {
    match stmt {
        CStmt::Decl { name, ty, init } => match init {
            Some(expr) => format!("{} {} = {}", render_type(ty), name, render_expr(expr)),
            None => format!("{} {}", render_type(ty), name),
        },
        CStmt::Assign { lhs, rhs } => {
            format!("{} = {}", render_expr(lhs), render_expr(rhs))
        }
        CStmt::Expr(expr) => render_expr(expr),
        _ => "/* unsupported inline stmt */".to_string(),
    }
}

// ===========================================================================
// Expression rendering
// ===========================================================================

fn render_expr(expr: &CExpr) -> String {
    match expr {
        CExpr::Var(name) => name.clone(),
        CExpr::IntLit(n) => n.to_string(),
        CExpr::StrLit(s) => format!("\"{}\"", escape_c_str(s)),
        CExpr::CharLit(c) => format!("'{}'", c),
        CExpr::BoolLit(b) => {
            if *b {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        CExpr::Null => "NULL".to_string(),
        CExpr::Call { func, args } => {
            let args_str: Vec<String> = args.iter().map(render_expr).collect();
            format!("{}({})", func, args_str.join(", "))
        }
        CExpr::BinOp { left, op, right } => {
            format!("({} {} {})", render_expr(left), op, render_expr(right))
        }
        CExpr::UnaryOp { op, expr } => {
            // Postfix operators like ++ go after the expression.
            if op == "++" || op == "--" {
                format!("{}{}", render_expr(expr), op)
            } else {
                format!("{}{}", op, render_expr(expr))
            }
        }
        CExpr::Field(expr, field) => {
            format!("{}.{}", render_expr(expr), field)
        }
        CExpr::Arrow(expr, field) => {
            format!("{}->{}", render_expr(expr), field)
        }
        CExpr::Index { expr, index } => {
            format!("{}[{}]", render_expr(expr), render_expr(index))
        }
        CExpr::AddressOf(expr) => format!("&{}", render_expr(expr)),
        CExpr::Deref(expr) => format!("*{}", render_expr(expr)),
        CExpr::Cast { ty, expr } => {
            format!("({}){}", render_type(ty), render_expr(expr))
        }
        CExpr::SizeOf(ty) => format!("sizeof({})", render_type(ty)),
        CExpr::Malloc(size_expr) => format!("malloc({})", render_expr(size_expr)),
        CExpr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            format!(
                "({} ? {} : {})",
                render_expr(cond),
                render_expr(then_expr),
                render_expr(else_expr)
            )
        }
    }
}

// ===========================================================================
// Type rendering
// ===========================================================================

fn render_type(ty: &CType) -> String {
    match ty {
        CType::Void => "void".to_string(),
        CType::Int(kind) => render_int_kind(kind),
        CType::Char => "char".to_string(),
        CType::Float(kind) => match kind {
            CFloatKind::Float => "float".to_string(),
            CFloatKind::Double => "double".to_string(),
        },
        CType::Ptr(inner) => format!("{}*", render_type(inner)),
        CType::Const(inner) => format!("const {}", render_type(inner)),
        CType::Array { element, size } => match size {
            Some(n) => format!("{}[{}]", render_type(element), n),
            None => format!("{}[]", render_type(element)),
        },
        CType::Named(name) => name.clone(),
        CType::FnPtr {
            return_type,
            param_types,
        } => {
            let params: Vec<String> = param_types.iter().map(render_type).collect();
            format!("{} (*)({})", render_type(return_type), params.join(", "))
        }
    }
}

fn render_int_kind(kind: &CIntKind) -> String {
    match kind {
        CIntKind::Int => "int".to_string(),
        CIntKind::Long => "long".to_string(),
        CIntKind::SizeT => "size_t".to_string(),
        CIntKind::Fixed(bits) => format!("int{}_t", bits),
        CIntKind::UFixed(bits) => format!("uint{}_t", bits),
    }
}

// ===========================================================================
// Tests (C3.6)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- C3.1: Function rendering --

    #[test]
    fn render_function_with_params_and_body() {
        let f = CFnDef {
            name: "process_file".to_string(),
            return_type: CType::Int(CIntKind::Int),
            params: vec![
                (
                    "path".to_string(),
                    CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                ),
                ("count".to_string(), CType::Int(CIntKind::Fixed(64))),
            ],
            body: vec![
                CStmt::Decl {
                    name: "rc".to_string(),
                    ty: CType::Int(CIntKind::Int),
                    init: Some(CExpr::Call {
                        func: "gunbc_file_read_request".to_string(),
                        args: vec![CExpr::Var("path".to_string())],
                    }),
                },
                CStmt::If {
                    cond: CExpr::BinOp {
                        left: Box::new(CExpr::Var("rc".to_string())),
                        op: "!=".to_string(),
                        right: Box::new(CExpr::IntLit(0)),
                    },
                    then_body: vec![CStmt::Return(Some(CExpr::IntLit(-1)))],
                    else_body: None,
                },
                CStmt::Return(Some(CExpr::IntLit(0))),
            ],
            is_static: false,
        };

        let rendered = render_fn_def(&f, 0);
        assert!(
            rendered.contains("int process_file(const char* path, int64_t count) {"),
            "signature: got {rendered}"
        );
        assert!(
            rendered.contains("int rc = gunbc_file_read_request(path);"),
            "decl: got {rendered}"
        );
        assert!(rendered.contains("if ((rc != 0)) {"), "if: got {rendered}");
        assert!(
            rendered.contains("return -1;"),
            "error return: got {rendered}"
        );
        assert!(
            rendered.contains("return 0;"),
            "success return: got {rendered}"
        );
    }

    #[test]
    fn render_static_function() {
        let f = CFnDef {
            name: "helper".to_string(),
            return_type: CType::Void,
            params: vec![],
            body: vec![],
            is_static: true,
        };

        let rendered = render_fn_def(&f, 0);
        assert!(
            rendered.contains("static void helper(void) {"),
            "static void fn: got {rendered}"
        );
    }

    #[test]
    fn render_forward_declaration() {
        let d = CFnDecl {
            name: "process_file".to_string(),
            return_type: CType::Int(CIntKind::Int),
            params: vec![(
                "path".to_string(),
                CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
            )],
        };
        let rendered = render_fn_decl(&d, 0);
        assert_eq!(rendered, "int process_file(const char* path);\n");
    }

    // -- C3.2: Struct rendering --

    #[test]
    fn render_c_struct() {
        let rendered = render_struct_def(
            "Config",
            &[
                (
                    "name".to_string(),
                    CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                ),
                ("count".to_string(), CType::Int(CIntKind::Fixed(64))),
            ],
            0,
        );
        assert!(
            rendered.contains("typedef struct {"),
            "start: got {rendered}"
        );
        assert!(
            rendered.contains("    const char* name;"),
            "field: got {rendered}"
        );
        assert!(
            rendered.contains("    int64_t count;"),
            "field: got {rendered}"
        );
        assert!(rendered.contains("} Config;"), "end: got {rendered}");
    }

    #[test]
    fn render_tagged_union_value_type() {
        let rendered = render_tagged_union(
            "Value",
            "tag",
            &[
                (
                    "Int".to_string(),
                    vec![("val".to_string(), CType::Int(CIntKind::Fixed(64)))],
                ),
                (
                    "Str".to_string(),
                    vec![(
                        "val".to_string(),
                        CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                    )],
                ),
                ("Null".to_string(), vec![]),
            ],
            0,
        );
        assert!(rendered.contains("typedef enum {"), "tag enum");
        assert!(rendered.contains("tag_Int,"), "variant Int");
        assert!(rendered.contains("tag_Str,"), "variant Str");
        assert!(rendered.contains("tag_Null"), "variant Null");
        assert!(rendered.contains("} Value_tag;"), "tag type name");
        assert!(rendered.contains("Value_tag tag;"), "tag field");
        assert!(rendered.contains("union {"), "union");
        assert!(rendered.contains("int64_t val;"), "int field");
        assert!(rendered.contains("const char* val;"), "str field");
        assert!(rendered.contains("} Value;"), "struct end");
    }

    // -- C3.3: Include directives --

    #[test]
    fn render_system_and_local_includes() {
        let source = CSourceFile {
            includes: vec![
                CItem::Include {
                    path: "stdio.h".to_string(),
                    system: true,
                },
                CItem::Include {
                    path: "stdlib.h".to_string(),
                    system: true,
                },
                CItem::Include {
                    path: "gunbc/transport.h".to_string(),
                    system: false,
                },
            ],
            items: vec![],
        };
        let rendered = render_c_source(&source);
        assert!(rendered.contains("#include <stdio.h>"), "system include");
        assert!(rendered.contains("#include <stdlib.h>"), "system include");
        assert!(
            rendered.contains("#include \"gunbc/transport.h\""),
            "local include"
        );
    }

    // -- C3.4: main() with argc/argv --

    #[test]
    fn render_main_with_argc_argv() {
        let f = CFnDef {
            name: "main".to_string(),
            return_type: CType::Int(CIntKind::Int),
            params: vec![
                ("argc".to_string(), CType::Int(CIntKind::Int)),
                (
                    "argv".to_string(),
                    CType::Ptr(Box::new(CType::Ptr(Box::new(CType::Char)))),
                ),
            ],
            body: vec![
                CStmt::If {
                    cond: CExpr::BinOp {
                        left: Box::new(CExpr::Var("argc".to_string())),
                        op: "<".to_string(),
                        right: Box::new(CExpr::IntLit(2)),
                    },
                    then_body: vec![
                        CStmt::Expr(CExpr::Call {
                            func: "fprintf".to_string(),
                            args: vec![
                                CExpr::Var("stderr".to_string()),
                                CExpr::StrLit("Usage: %s <path>\\n".to_string()),
                                CExpr::Index {
                                    expr: Box::new(CExpr::Var("argv".to_string())),
                                    index: Box::new(CExpr::IntLit(0)),
                                },
                            ],
                        }),
                        CStmt::Return(Some(CExpr::IntLit(1))),
                    ],
                    else_body: None,
                },
                CStmt::Return(Some(CExpr::IntLit(0))),
            ],
            is_static: false,
        };
        let rendered = render_fn_def(&f, 0);
        assert!(
            rendered.contains("int main(int argc, char** argv) {"),
            "main sig: got {rendered}"
        );
        assert!(
            rendered.contains("if ((argc < 2)) {"),
            "argc check: got {rendered}"
        );
        assert!(
            rendered.contains("fprintf(stderr, \"Usage: %s <path>\\\\n\", argv[0]);"),
            "fprintf: got {rendered}"
        );
    }

    // -- C3.5: Makefile generation --

    #[test]
    fn render_makefile_for_c_compilation() {
        let rendered = render_c_makefile("makegen", &["main.c"]);
        assert!(rendered.contains("CC = gcc"), "CC");
        assert!(rendered.contains("-Wall -Wextra -std=c11"), "CFLAGS");
        assert!(rendered.contains("TARGET = makegen"), "TARGET");
        assert!(rendered.contains("SRCS = main.c"), "SRCS");
        assert!(
            rendered.contains("$(CC) $(CFLAGS) -o $@ $^"),
            "compile rule"
        );
        assert!(rendered.contains(".PHONY: all clean"), "phony");
    }

    // -- Expression rendering --

    #[test]
    fn render_c_expressions() {
        assert_eq!(render_expr(&CExpr::IntLit(42)), "42");
        assert_eq!(render_expr(&CExpr::IntLit(-1)), "-1");
        assert_eq!(render_expr(&CExpr::BoolLit(true)), "1");
        assert_eq!(render_expr(&CExpr::BoolLit(false)), "0");
        assert_eq!(render_expr(&CExpr::Null), "NULL");
        assert_eq!(render_expr(&CExpr::StrLit("hello".into())), "\"hello\"");
        assert_eq!(render_expr(&CExpr::CharLit('x')), "'x'");
        assert_eq!(render_expr(&CExpr::Var("count".to_string())), "count");
    }

    #[test]
    fn render_pointer_expressions() {
        let addr = CExpr::AddressOf(Box::new(CExpr::Var("x".to_string())));
        assert_eq!(render_expr(&addr), "&x");

        let deref = CExpr::Deref(Box::new(CExpr::Var("ptr".to_string())));
        assert_eq!(render_expr(&deref), "*ptr");

        let arrow = CExpr::Arrow(Box::new(CExpr::Var("node".to_string())), "next".to_string());
        assert_eq!(render_expr(&arrow), "node->next");

        let cast = CExpr::Cast {
            ty: CType::Ptr(Box::new(CType::Void)),
            expr: Box::new(CExpr::Var("data".to_string())),
        };
        assert_eq!(render_expr(&cast), "(void*)data");

        let sz = CExpr::SizeOf(CType::Named("Config".to_string()));
        assert_eq!(render_expr(&sz), "sizeof(Config)");

        let m = CExpr::Malloc(Box::new(CExpr::SizeOf(CType::Named("Config".to_string()))));
        assert_eq!(render_expr(&m), "malloc(sizeof(Config))");
    }

    #[test]
    fn render_ternary() {
        let expr = CExpr::Ternary {
            cond: Box::new(CExpr::Var("x".to_string())),
            then_expr: Box::new(CExpr::IntLit(1)),
            else_expr: Box::new(CExpr::IntLit(0)),
        };
        assert_eq!(render_expr(&expr), "(x ? 1 : 0)");
    }

    // -- Type rendering --

    #[test]
    fn render_c_types() {
        assert_eq!(render_type(&CType::Void), "void");
        assert_eq!(render_type(&CType::Int(CIntKind::Int)), "int");
        assert_eq!(render_type(&CType::Int(CIntKind::Fixed(64))), "int64_t");
        assert_eq!(render_type(&CType::Int(CIntKind::UFixed(32))), "uint32_t");
        assert_eq!(render_type(&CType::Int(CIntKind::SizeT)), "size_t");
        assert_eq!(render_type(&CType::Char), "char");
        assert_eq!(render_type(&CType::Float(CFloatKind::Double)), "double");
        assert_eq!(
            render_type(&CType::Ptr(Box::new(CType::Const(Box::new(CType::Char))))),
            "const char*"
        );
        assert_eq!(render_type(&CType::Ptr(Box::new(CType::Void))), "void*");
        assert_eq!(
            render_type(&CType::Array {
                element: Box::new(CType::Int(CIntKind::Int)),
                size: Some(10)
            }),
            "int[10]"
        );
        assert_eq!(render_type(&CType::Named("Config".to_string())), "Config");
    }

    #[test]
    fn render_fn_ptr_type() {
        let ty = CType::FnPtr {
            return_type: Box::new(CType::Int(CIntKind::Int)),
            param_types: vec![
                CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                CType::Int(CIntKind::Int),
            ],
        };
        assert_eq!(render_type(&ty), "int (*)(const char*, int)");
    }

    // -- For loop rendering --

    #[test]
    fn render_for_loop() {
        let stmt = CStmt::For {
            init: Box::new(CStmt::Decl {
                name: "i".to_string(),
                ty: CType::Int(CIntKind::SizeT),
                init: Some(CExpr::IntLit(0)),
            }),
            cond: CExpr::BinOp {
                left: Box::new(CExpr::Var("i".to_string())),
                op: "<".to_string(),
                right: Box::new(CExpr::Var("n".to_string())),
            },
            step: Box::new(CStmt::Expr(CExpr::UnaryOp {
                op: "++".to_string(),
                expr: Box::new(CExpr::Var("i".to_string())),
            })),
            body: vec![CStmt::Expr(CExpr::Call {
                func: "printf".to_string(),
                args: vec![
                    CExpr::StrLit("%zu\\n".to_string()),
                    CExpr::Var("i".to_string()),
                ],
            })],
        };
        let rendered = render_stmt(&stmt, 0);
        assert!(
            rendered.contains("for (size_t i = 0; (i < n); i++) {"),
            "for header: got {rendered}"
        );
        assert!(
            rendered.contains("printf(\"%zu\\\\n\", i);"),
            "body: got {rendered}"
        );
    }

    // -- C3.6: Full integration test --

    #[test]
    fn render_full_c_source() {
        let source = CSourceFile {
            includes: vec![
                CItem::Include {
                    path: "stdio.h".to_string(),
                    system: true,
                },
                CItem::Include {
                    path: "stdlib.h".to_string(),
                    system: true,
                },
                CItem::Include {
                    path: "string.h".to_string(),
                    system: true,
                },
                CItem::Include {
                    path: "gunbc/transport.h".to_string(),
                    system: false,
                },
            ],
            items: vec![
                CItem::Define {
                    name: "OP_READ".to_string(),
                    value: "0".to_string(),
                },
                CItem::Define {
                    name: "OP_WRITE".to_string(),
                    value: "1".to_string(),
                },
                CItem::StructDef {
                    name: "Config".to_string(),
                    fields: vec![
                        (
                            "path".to_string(),
                            CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                        ),
                        ("count".to_string(), CType::Int(CIntKind::Fixed(64))),
                    ],
                },
                CItem::FnDef(CFnDef {
                    name: "main".to_string(),
                    return_type: CType::Int(CIntKind::Int),
                    params: vec![
                        ("argc".to_string(), CType::Int(CIntKind::Int)),
                        (
                            "argv".to_string(),
                            CType::Ptr(Box::new(CType::Ptr(Box::new(CType::Char)))),
                        ),
                    ],
                    body: vec![
                        CStmt::Decl {
                            name: "path".to_string(),
                            ty: CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                            init: Some(CExpr::Index {
                                expr: Box::new(CExpr::Var("argv".to_string())),
                                index: Box::new(CExpr::IntLit(1)),
                            }),
                        },
                        CStmt::Decl {
                            name: "rc".to_string(),
                            ty: CType::Int(CIntKind::Int),
                            init: Some(CExpr::Call {
                                func: "gunbc_transport_execute".to_string(),
                                args: vec![CExpr::Var("path".to_string())],
                            }),
                        },
                        CStmt::If {
                            cond: CExpr::BinOp {
                                left: Box::new(CExpr::Var("rc".to_string())),
                                op: "!=".to_string(),
                                right: Box::new(CExpr::IntLit(0)),
                            },
                            then_body: vec![CStmt::Return(Some(CExpr::IntLit(-1)))],
                            else_body: None,
                        },
                        CStmt::Return(Some(CExpr::IntLit(0))),
                    ],
                    is_static: false,
                }),
            ],
        };

        let rendered = render_c_source(&source);

        // Includes.
        assert!(rendered.contains("#include <stdio.h>"));
        assert!(rendered.contains("#include <stdlib.h>"));
        assert!(rendered.contains("#include <string.h>"));
        assert!(rendered.contains("#include \"gunbc/transport.h\""));

        // Defines.
        assert!(rendered.contains("#define OP_READ 0"));
        assert!(rendered.contains("#define OP_WRITE 1"));

        // Struct.
        assert!(rendered.contains("typedef struct {"));
        assert!(rendered.contains("const char* path;"));
        assert!(rendered.contains("int64_t count;"));
        assert!(rendered.contains("} Config;"));

        // Main function.
        assert!(rendered.contains("int main(int argc, char** argv) {"));
        assert!(rendered.contains("const char* path = argv[1];"));
        assert!(rendered.contains("int rc = gunbc_transport_execute(path);"));
        assert!(rendered.contains("if ((rc != 0)) {"));
        assert!(rendered.contains("return -1;"));
        assert!(rendered.contains("return 0;"));
    }

    // -- Label and goto --

    #[test]
    fn render_label_and_goto() {
        let stmts = vec![
            CStmt::Goto("cleanup".to_string()),
            CStmt::Label("cleanup".to_string()),
            CStmt::Free(CExpr::Var("buf".to_string())),
        ];
        let mut rendered = String::new();
        for s in &stmts {
            rendered.push_str(&render_stmt(s, 1));
        }
        assert!(rendered.contains("goto cleanup;"), "goto");
        assert!(rendered.contains("cleanup:"), "label");
        assert!(rendered.contains("free(buf);"), "free");
    }
}
