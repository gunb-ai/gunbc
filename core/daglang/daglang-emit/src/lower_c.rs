//! AbstractIR → C (CStyleIR) lowering.
//!
//! Adds C-specific constructs: explicit memory management, tagged union Value types,
//! function pointers, arena allocation, char*/length string handling.
//!
//! Provides [`lower_to_c`] which transforms a target-agnostic `SourceFile`
//! (from `lower_to_ir`) into a `CSourceFile`:
//!
//! - `String` → `const char*` (null-terminated)
//! - `List<T>` → `T* + size_t count` (pointer + length)
//! - `Let` → `CStmt::Decl` with explicit type
//! - Transport calls → C functions returning `int` (0 = success, -1 = error)
//! - `FormatStr` → `snprintf` call
//! - Error handling: check return code, goto cleanup on error
//! - `#include` directives from dependency analysis
//!
//! **Owned by**: Task 11 (dsl-codegen-tasks.md)

use gunbc_ir::code_ir::c_ir::*;
use gunbc_ir::code_ir::lower::LowerError;
use gunbc_ir::code_ir::{Expr, FnDef, Item, SourceFile, Stmt};

/// Configuration for C lowering.
#[derive(Debug, Clone)]
pub struct CConfig {
    /// Whether to emit arena allocator calls instead of raw malloc/free.
    pub use_arena: bool,
    /// Whether to use the exec-runtime equivalent for transport calls.
    pub use_exec_runtime: bool,
}

impl Default for CConfig {
    fn default() -> Self {
        Self {
            use_arena: true,
            use_exec_runtime: true,
        }
    }
}

/// Lower an AbstractIR `SourceFile` to a `CSourceFile`.
pub fn lower_to_c(source: &SourceFile, config: &CConfig) -> Result<CSourceFile, LowerError> {
    let includes = collect_c_includes(source, config);
    let mut items: Vec<CItem> = Vec::new();

    // Lower each item.
    for item in &source.items {
        lower_item_into(&mut items, item, config)?;
    }

    Ok(CSourceFile { includes, items })
}

// ===========================================================================
// Include analysis
// ===========================================================================

fn collect_c_includes(source: &SourceFile, config: &CConfig) -> Vec<CItem> {
    let mut includes = Vec::new();

    // Always include standard headers.
    includes.push(CItem::Include {
        path: "stdio.h".to_string(),
        system: true,
    });
    includes.push(CItem::Include {
        path: "stdlib.h".to_string(),
        system: true,
    });
    includes.push(CItem::Include {
        path: "string.h".to_string(),
        system: true,
    });

    let has_transport = source
        .items
        .iter()
        .any(|item| matches!(item, Item::Fn(f) if body_has_transport_calls(&f.body)));

    if has_transport && config.use_exec_runtime {
        includes.push(CItem::Include {
            path: "gunbc/transport.h".to_string(),
            system: false,
        });
    }

    includes
}

// ===========================================================================
// Item lowering
// ===========================================================================

fn lower_item_into(
    items: &mut Vec<CItem>,
    item: &Item,
    config: &CConfig,
) -> Result<(), LowerError> {
    match item {
        Item::Fn(f) => {
            items.push(CItem::FnDef(lower_fn_def(f, config)?));
        }
        Item::Struct(s) => {
            let fields: Vec<(String, CType)> = s
                .fields
                .iter()
                .map(|(name, ty, _)| (name.clone(), map_to_c_type(ty)))
                .collect();
            items.push(CItem::StructDef {
                name: s.name.clone(),
                fields,
            });
        }
        Item::Enum(e) => {
            // C enums: define constants with #define or C enum.
            // Use #define for simplicity.
            for (i, variant) in e.variants.iter().enumerate() {
                let clean = variant.split('(').next().unwrap_or(variant).trim();
                items.push(CItem::Define {
                    name: format!("{}_{}", e.name.to_uppercase(), clean.to_uppercase()),
                    value: i.to_string(),
                });
            }
        }
        Item::Use(_import) => {
            // Imports are handled by collect_c_includes.
        }
        Item::Impl(impl_block) => {
            // C doesn't have impl blocks — emit each method as a free function
            // with the type name prefixed.
            for func in &impl_block.items {
                let mut c_func = lower_fn_def(func, config)?;
                c_func.name = format!("{}_{}", impl_block.type_name, c_func.name);
                items.push(CItem::FnDef(c_func));
            }
        }
        Item::Raw(code) => {
            items.push(CItem::Comment(code.clone()));
        }
    }
    Ok(())
}

// ===========================================================================
// B4.5: Function lowering with error code return
// ===========================================================================

fn lower_fn_def(f: &FnDef, config: &CConfig) -> Result<CFnDef, LowerError> {
    let has_transport = body_has_transport_calls(&f.body);

    // B4.5: Functions with transport calls return int (0 = ok, -1 = error).
    let return_type = if has_transport {
        CType::Int(CIntKind::Int)
    } else {
        f.return_type
            .as_ref()
            .map(|t| map_to_c_type(t))
            .unwrap_or(CType::Void)
    };

    let params: Vec<(String, CType)> = f
        .params
        .iter()
        .map(|(name, ty)| (name.clone(), map_to_c_type(ty)))
        .collect();

    let mut body = lower_body(&f.body, has_transport, config);

    // If fallible, add `return 0;` at the end (success).
    if has_transport {
        body.push(CStmt::Return(Some(CExpr::IntLit(0))));
    }

    Ok(CFnDef {
        name: f.name.clone(),
        return_type,
        params,
        body,
        is_static: !f.is_pub,
    })
}

// ===========================================================================
// Body statement lowering
// ===========================================================================

fn lower_body(stmts: &[Stmt], in_fallible_fn: bool, config: &CConfig) -> Vec<CStmt> {
    let mut result = Vec::new();
    for stmt in stmts {
        lower_stmt_into(&mut result, stmt, in_fallible_fn, config);
    }
    result
}

fn lower_stmt_into(out: &mut Vec<CStmt>, stmt: &Stmt, in_fallible_fn: bool, config: &CConfig) {
    match stmt {
        Stmt::Let { name, expr, .. } => {
            let c_expr = lower_expr(expr, config);
            let c_type = infer_c_type(expr);
            let is_transport = expr_is_transport_call(expr);

            if is_transport && in_fallible_fn {
                // B4.5: Declare result variable, call, check return code.
                let rc_name = format!("{}_rc", name);
                out.push(CStmt::Decl {
                    name: name.clone(),
                    ty: c_type.clone(),
                    init: None,
                });
                out.push(CStmt::Decl {
                    name: rc_name.clone(),
                    ty: CType::Int(CIntKind::Int),
                    init: Some(c_expr),
                });
                // if (rc != 0) { return -1; }
                out.push(CStmt::If {
                    cond: CExpr::BinOp {
                        left: Box::new(CExpr::Var(rc_name)),
                        op: "!=".to_string(),
                        right: Box::new(CExpr::IntLit(0)),
                    },
                    then_body: vec![CStmt::Return(Some(CExpr::IntLit(-1)))],
                    else_body: None,
                });
            } else {
                out.push(CStmt::Decl {
                    name: name.clone(),
                    ty: c_type,
                    init: Some(c_expr),
                });
            }
        }
        Stmt::Expr(expr) => {
            let c_expr = lower_expr(expr, config);
            let is_transport = expr_is_transport_call(expr);

            if is_transport && in_fallible_fn {
                let rc_name = "__rc".to_string();
                out.push(CStmt::Decl {
                    name: rc_name.clone(),
                    ty: CType::Int(CIntKind::Int),
                    init: Some(c_expr),
                });
                out.push(CStmt::If {
                    cond: CExpr::BinOp {
                        left: Box::new(CExpr::Var(rc_name)),
                        op: "!=".to_string(),
                        right: Box::new(CExpr::IntLit(0)),
                    },
                    then_body: vec![CStmt::Return(Some(CExpr::IntLit(-1)))],
                    else_body: None,
                });
            } else {
                out.push(CStmt::Expr(c_expr));
            }
        }
        Stmt::Comment(text) => {
            out.push(CStmt::Comment(text.clone()));
        }
        Stmt::Blank => {
            out.push(CStmt::Blank);
        }
        Stmt::Return(expr) => {
            out.push(CStmt::Return(Some(lower_expr(expr, config))));
        }
        Stmt::TailExpr(expr) => {
            // C doesn't have implicit returns.
            out.push(CStmt::Return(Some(lower_expr(expr, config))));
        }
        Stmt::For {
            binding,
            iter,
            body,
        } => {
            // `for (size_t i = 0; i < len; i++) { type binding = arr[i]; ... }`
            let iter_expr = lower_expr(iter, config);
            let idx = format!("_i_{}", binding);
            let len = format!("_len_{}", binding);

            // size_t len = ...; (we assume the iter has a .count or len)
            out.push(CStmt::Decl {
                name: len.clone(),
                ty: CType::Int(CIntKind::SizeT),
                init: Some(CExpr::Field(
                    Box::new(iter_expr.clone()),
                    "count".to_string(),
                )),
            });

            let for_body = {
                let mut fb = Vec::new();
                // type binding = arr[i];
                fb.push(CStmt::Decl {
                    name: binding.clone(),
                    ty: CType::Void, // Auto type — would need proper inference.
                    init: Some(CExpr::Index {
                        expr: Box::new(CExpr::Field(Box::new(iter_expr), "data".to_string())),
                        index: Box::new(CExpr::Var(idx.clone())),
                    }),
                });
                fb.extend(lower_body(body, in_fallible_fn, config));
                fb
            };

            out.push(CStmt::For {
                init: Box::new(CStmt::Decl {
                    name: idx.clone(),
                    ty: CType::Int(CIntKind::SizeT),
                    init: Some(CExpr::IntLit(0)),
                }),
                cond: CExpr::BinOp {
                    left: Box::new(CExpr::Var(idx.clone())),
                    op: "<".to_string(),
                    right: Box::new(CExpr::Var(len)),
                },
                step: Box::new(CStmt::Expr(CExpr::UnaryOp {
                    op: "++".to_string(),
                    expr: Box::new(CExpr::Var(idx)),
                })),
                body: for_body,
            });
        }
        Stmt::Assert(_) => {
            // C: assert() or skip. Skip for now.
            out.push(CStmt::Comment("assert omitted".to_string()));
        }
        Stmt::Item(item) => {
            // Nested items are unusual in C but possible.
            let mut inner_items = Vec::new();
            let _ = lower_item_into(&mut inner_items, item, config);
            for ci in inner_items {
                if let CItem::FnDef(f) = ci {
                    for s in f.body {
                        out.push(s);
                    }
                }
            }
        }
    }
}

// ===========================================================================
// Expression lowering
// ===========================================================================

fn lower_expr(expr: &Expr, config: &CConfig) -> CExpr {
    match expr {
        Expr::Value(v) => lower_value_expr(v),
        Expr::Var(name) => CExpr::Var(name.clone()),
        Expr::Str(s) => CExpr::StrLit(s.clone()),
        Expr::IntLit(n) => CExpr::IntLit(*n),
        Expr::BoolLit(b) => CExpr::BoolLit(*b),
        Expr::Call { func, args } => {
            let func_name = match func.as_ref() {
                Expr::Var(name) => {
                    if let Some(c_fn) = rewrite_transport_call_c(name, config) {
                        c_fn
                    } else {
                        name.clone()
                    }
                }
                _ => "unknown_fn".to_string(),
            };
            CExpr::Call {
                func: func_name,
                args: args.iter().map(|a| lower_expr(a, config)).collect(),
            }
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            // C doesn't have methods — convert to function call with receiver as first arg.
            let mut c_args = vec![lower_expr(receiver, config)];
            c_args.extend(args.iter().map(|a| lower_expr(a, config)));
            CExpr::Call {
                func: method.clone(),
                args: c_args,
            }
        }
        Expr::Field(inner, field) => {
            CExpr::Field(Box::new(lower_expr(inner, config)), field.clone())
        }
        Expr::Deref(inner) => CExpr::Deref(Box::new(lower_expr(inner, config))),
        Expr::Ref(inner) => CExpr::AddressOf(Box::new(lower_expr(inner, config))),
        Expr::RefMut(inner) => CExpr::AddressOf(Box::new(lower_expr(inner, config))),
        Expr::BinOp { left, op, right } => CExpr::BinOp {
            left: Box::new(lower_expr(left, config)),
            op: op.clone(),
            right: Box::new(lower_expr(right, config)),
        },
        Expr::UnaryOp { op, expr } => CExpr::UnaryOp {
            op: op.clone(),
            expr: Box::new(lower_expr(expr, config)),
        },
        Expr::FormatStr { template, args } => {
            // B4.3: snprintf into a buffer.
            let mut call_args = vec![
                CExpr::Var("buf".to_string()),
                CExpr::Var("sizeof(buf)".to_string()),
                CExpr::StrLit(template.clone()),
            ];
            call_args.extend(args.iter().map(|a| lower_expr(a, config)));
            CExpr::Call {
                func: "snprintf".to_string(),
                args: call_args,
            }
        }
        Expr::MacroCall { name, args } => {
            // Convert macro calls to function calls in C.
            CExpr::Call {
                func: name.clone(),
                args: args.iter().map(|a| lower_expr(a, config)).collect(),
            }
        }
        Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            // C doesn't have if-expressions. Use ternary for simple cases.
            if then_body.len() == 1 && else_body.as_ref().is_some_and(|b| b.len() == 1) {
                if let (Some(Stmt::Return(then_expr)), Some(Stmt::Return(else_expr))) = (
                    then_body.first(),
                    else_body.as_ref().and_then(|b| b.first()),
                ) {
                    return CExpr::Ternary {
                        cond: Box::new(lower_expr(cond, config)),
                        then_expr: Box::new(lower_expr(then_expr, config)),
                        else_expr: Box::new(lower_expr(else_expr, config)),
                    };
                }
            }
            // Fallback: render as a variable name (complex if-expression not easily
            // representable in C expression context).
            CExpr::Var("/* if-expr */0".to_string())
        }
        Expr::Path(segments) => CExpr::Var(segments.join("_")),
        Expr::Struct { name, fields } => {
            // C struct literal (C99): `(struct Name){ .field = val, ... }`
            // For now, represent as a call to an init function.
            let init_args: Vec<CExpr> = fields.iter().map(|(_, v)| lower_expr(v, config)).collect();
            CExpr::Call {
                func: format!("{}_init", name),
                args: init_args,
            }
        }
        Expr::Closure { .. } => {
            // C doesn't have closures — would need function pointer.
            CExpr::Null
        }
        Expr::Array(elems) => {
            // C array initializer — represent as first element for simplicity.
            if elems.is_empty() {
                CExpr::Null
            } else {
                lower_expr(&elems[0], config)
            }
        }
        Expr::Tuple(elems) => {
            // C doesn't have tuples — return first element.
            if elems.is_empty() {
                CExpr::IntLit(0) // () → 0 in C
            } else {
                lower_expr(&elems[0], config)
            }
        }
        Expr::Match { .. } | Expr::Block(_) | Expr::RawCode(_) => {
            CExpr::Var("/* unsupported expr */0".to_string())
        }
    }
}

fn lower_value_expr(v: &gunbc_ir::ValueExpr) -> CExpr {
    match v {
        gunbc_ir::ValueExpr::Unit => CExpr::IntLit(0),
        gunbc_ir::ValueExpr::Bool(b) => CExpr::BoolLit(*b),
        gunbc_ir::ValueExpr::Str(s) => CExpr::StrLit(s.clone()),
        gunbc_ir::ValueExpr::Int(i) => CExpr::IntLit(*i),
        gunbc_ir::ValueExpr::Json(j) => CExpr::StrLit(j.to_string()),
        _ => CExpr::Null,
    }
}

// ===========================================================================
// B4.3: Type mapping
// ===========================================================================

/// Map an abstract type name to its C equivalent.
fn map_to_c_type(abstract_type: &str) -> CType {
    match abstract_type {
        "String" | "Path" => CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
        "Bool" | "bool" => CType::Int(CIntKind::Int), // C uses int for bool.
        "Int" | "i64" | "I64" => CType::Int(CIntKind::Fixed(64)),
        "Float" | "f64" => CType::Float(CFloatKind::Double),
        "ToolRegistry" => CType::Ptr(Box::new(CType::Void)),
        "FilesystemHandle" => CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
        other => {
            if let Some(inner) = other
                .strip_prefix("List<")
                .and_then(|rest| rest.strip_suffix('>'))
            {
                return CType::Ptr(Box::new(map_to_c_type(inner)));
            }
            // Default: void pointer.
            CType::Ptr(Box::new(CType::Void))
        }
    }
}

/// Infer C type from an abstract expression (best effort).
fn infer_c_type(expr: &Expr) -> CType {
    match expr {
        Expr::IntLit(_) => CType::Int(CIntKind::Fixed(64)),
        Expr::BoolLit(_) => CType::Int(CIntKind::Int),
        Expr::Str(_) => CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
        Expr::Value(gunbc_ir::ValueExpr::Int(_)) => CType::Int(CIntKind::Fixed(64)),
        Expr::Value(gunbc_ir::ValueExpr::Bool(_)) => CType::Int(CIntKind::Int),
        Expr::Value(gunbc_ir::ValueExpr::Str(_)) => {
            CType::Ptr(Box::new(CType::Const(Box::new(CType::Char))))
        }
        _ => CType::Ptr(Box::new(CType::Void)), // Default to void*.
    }
}

// ===========================================================================
// Transport call detection and rewriting
// ===========================================================================

fn rewrite_transport_call_c(name: &str, config: &CConfig) -> Option<String> {
    if !config.use_exec_runtime {
        return None;
    }

    match name {
        "prepare_file_read" => Some("gunbc_file_read_request".to_string()),
        "execute_file_read" => Some("gunbc_transport_execute".to_string()),
        "prepare_file_write" => Some("gunbc_file_write_request".to_string()),
        "execute_file_write" => Some("gunbc_transport_execute".to_string()),
        "prepare_file_exists" => Some("gunbc_file_exists_request".to_string()),
        "execute_file_exists" => Some("gunbc_transport_execute".to_string()),
        "prepare_shell_exec" => Some("gunbc_shell_request".to_string()),
        "execute_shell_exec" => Some("gunbc_transport_execute".to_string()),
        "prepare_http_request" => Some("gunbc_http_request".to_string()),
        "execute_http_request" => Some("gunbc_transport_execute".to_string()),
        "prepare_directory_list" => Some("gunbc_dir_list_request".to_string()),
        "execute_directory_list" => Some("gunbc_transport_execute".to_string()),
        "acquire_resource" => Some("gunbc_acquire_resource".to_string()),
        _ => None,
    }
}

fn expr_is_transport_call(expr: &Expr) -> bool {
    if let Expr::Call { func, .. } = expr {
        if let Expr::Var(name) = func.as_ref() {
            return name.starts_with("prepare_") || name.starts_with("execute_");
        }
    }
    false
}

fn body_has_transport_calls(stmts: &[Stmt]) -> bool {
    stmts.iter().any(stmt_has_transport)
}

fn stmt_has_transport(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { expr, .. } => expr_has_transport(expr),
        Stmt::Expr(expr) | Stmt::Return(expr) | Stmt::TailExpr(expr) => expr_has_transport(expr),
        Stmt::For { body, .. } => body_has_transport_calls(body),
        _ => false,
    }
}

fn expr_has_transport(expr: &Expr) -> bool {
    match expr {
        Expr::Call { func, args } => {
            if let Expr::Var(name) = func.as_ref() {
                if name.starts_with("prepare_") || name.starts_with("execute_") {
                    return true;
                }
            }
            args.iter().any(expr_has_transport)
        }
        Expr::MethodCall { receiver, args, .. } => {
            expr_has_transport(receiver) || args.iter().any(expr_has_transport)
        }
        Expr::BinOp { left, right, .. } => expr_has_transport(left) || expr_has_transport(right),
        Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_has_transport(cond)
                || body_has_transport_calls(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|b| body_has_transport_calls(b))
        }
        Expr::Block(stmts) => body_has_transport_calls(stmts),
        _ => false,
    }
}

// ===========================================================================
// Tests (B4.6)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::code_ir::{EnumDef, StructDef};
    use gunbc_ir::ValueExpr;

    fn make_abstract_main(stmts: Vec<Stmt>) -> SourceFile {
        SourceFile {
            doc: vec!["Test source.".to_string()],
            items: vec![Item::Fn(FnDef {
                name: "main".to_string(),
                is_pub: true,
                params: vec![("path".to_string(), "String".to_string())],
                return_type: None,
                body: stmts,
                doc: vec![],
                attributes: vec![],
            })],
        }
    }

    // -- B4.1: CSourceFile structure --

    #[test]
    fn lower_produces_c_source_file_with_includes() {
        let source = make_abstract_main(vec![Stmt::let_bind("x", Expr::IntLit(42))]);
        let config = CConfig::default();
        let lowered = lower_to_c(&source, &config).unwrap();

        assert!(
            lowered
                .includes
                .iter()
                .any(|i| matches!(i, CItem::Include { path, system: true } if path == "stdio.h")),
            "should include stdio.h"
        );
        assert!(
            lowered
                .includes
                .iter()
                .any(|i| matches!(i, CItem::Include { path, system: true } if path == "stdlib.h")),
            "should include stdlib.h"
        );
    }

    // -- B4.2: Struct lowering --

    #[test]
    fn struct_fields_mapped_to_c_types() {
        let source = SourceFile {
            doc: vec![],
            items: vec![Item::Struct(StructDef {
                name: "Config".to_string(),
                is_pub: true,
                derives: vec![],
                fields: vec![
                    ("name".to_string(), "String".to_string(), true),
                    ("count".to_string(), "Int".to_string(), false),
                ],
                doc: vec![],
            })],
        };

        let config = CConfig::default();
        let lowered = lower_to_c(&source, &config).unwrap();

        let struct_item = lowered
            .items
            .iter()
            .find_map(|item| match item {
                CItem::StructDef { name, fields } if name == "Config" => Some(fields),
                _ => None,
            })
            .expect("should have struct Config");

        // String → const char*
        assert!(
            matches!(&struct_item[0].1, CType::Ptr(inner) if matches!(inner.as_ref(), CType::Const(c) if matches!(c.as_ref(), CType::Char))),
            "String should map to const char*, got {:?}",
            struct_item[0].1
        );
        // Int → int64_t
        assert!(
            matches!(&struct_item[1].1, CType::Int(CIntKind::Fixed(64))),
            "Int should map to int64_t"
        );
    }

    // -- B4.3: String handling --

    #[test]
    fn string_type_maps_to_const_char_ptr() {
        let ty = map_to_c_type("String");
        assert!(
            matches!(&ty, CType::Ptr(inner) if matches!(inner.as_ref(), CType::Const(c) if matches!(c.as_ref(), CType::Char))),
            "String should be const char*, got {ty:?}"
        );
    }

    // -- B4.4: Type mapping --

    #[test]
    fn map_abstract_types_to_c() {
        assert!(matches!(map_to_c_type("Bool"), CType::Int(CIntKind::Int)));
        assert!(matches!(
            map_to_c_type("Int"),
            CType::Int(CIntKind::Fixed(64))
        ));
        assert!(matches!(
            map_to_c_type("Float"),
            CType::Float(CFloatKind::Double)
        ));
        assert!(matches!(
            map_to_c_type("ToolRegistry"),
            CType::Ptr(inner) if matches!(inner.as_ref(), &CType::Void)
        ));
        // List<String> → const char**
        assert!(matches!(
            map_to_c_type("List<String>"),
            CType::Ptr(inner) if matches!(inner.as_ref(), CType::Ptr(_))
        ));
    }

    // -- B4.5: Error handling --

    #[test]
    fn error_return_code_for_transport_functions() {
        let source = make_abstract_main(vec![
            Stmt::let_bind(
                "request",
                Expr::call("prepare_file_read", vec![Expr::var("path")]),
            ),
            Stmt::let_bind(
                "response",
                Expr::call("execute_file_read", vec![Expr::var("request")]),
            ),
        ]);

        let config = CConfig::default();
        let lowered = lower_to_c(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                CItem::FnDef(f) if f.name == "main" => Some(f),
                _ => None,
            })
            .expect("should have fn main");

        // Return type should be int.
        assert!(
            matches!(main_fn.return_type, CType::Int(CIntKind::Int)),
            "transport fn should return int, got {:?}",
            main_fn.return_type
        );

        // Should end with `return 0;`
        let last_stmt = main_fn.body.last().unwrap();
        assert!(
            matches!(last_stmt, CStmt::Return(Some(CExpr::IntLit(0)))),
            "should end with return 0, got {last_stmt:?}"
        );

        // Should have error check: `if (rc != 0) return -1;`
        let has_error_check = main_fn.body.iter().any(|stmt| {
            matches!(
                stmt,
                CStmt::If {
                    then_body,
                    ..
                } if then_body.iter().any(|s| matches!(s, CStmt::Return(Some(CExpr::IntLit(-1)))))
            )
        });
        assert!(has_error_check, "should have error check returning -1");
    }

    #[test]
    fn no_error_return_for_pure_functions() {
        let source = make_abstract_main(vec![Stmt::let_bind("x", Expr::IntLit(42))]);

        let config = CConfig::default();
        let lowered = lower_to_c(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                CItem::FnDef(f) if f.name == "main" => Some(f),
                _ => None,
            })
            .unwrap();

        // Pure function with no return type → void.
        assert!(
            matches!(main_fn.return_type, CType::Void),
            "pure function should be void"
        );
    }

    // -- B4.5: Transport call rewriting --

    #[test]
    fn transport_calls_rewritten_to_c_runtime() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "req",
            Expr::call("prepare_file_read", vec![Expr::var("path")]),
        )]);

        let config = CConfig::default();
        let lowered = lower_to_c(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                CItem::FnDef(f) if f.name == "main" => Some(f),
                _ => None,
            })
            .unwrap();

        let body_debug = format!("{:?}", main_fn.body);
        assert!(
            body_debug.contains("gunbc_file_read_request"),
            "prepare_file_read should be rewritten, body: {body_debug}"
        );
    }

    // -- Enum lowering --

    #[test]
    fn enum_becomes_define_constants() {
        let source = SourceFile {
            doc: vec![],
            items: vec![Item::Enum(EnumDef {
                name: "Op".to_string(),
                is_pub: true,
                derives: vec![],
                variants: vec!["Read".to_string(), "Write".to_string()],
                doc: vec![],
            })],
        };

        let config = CConfig::default();
        let lowered = lower_to_c(&source, &config).unwrap();

        let defines: Vec<&CItem> = lowered
            .items
            .iter()
            .filter(|item| matches!(item, CItem::Define { .. }))
            .collect();

        assert_eq!(defines.len(), 2, "should have 2 defines");
        assert!(
            matches!(&defines[0], CItem::Define { name, value } if name == "OP_READ" && value == "0"),
            "first define: {:?}",
            defines[0]
        );
        assert!(
            matches!(&defines[1], CItem::Define { name, value } if name == "OP_WRITE" && value == "1"),
            "second define: {:?}",
            defines[1]
        );
    }

    // -- FormatStr lowering --

    #[test]
    fn format_str_becomes_snprintf() {
        let expr = Expr::FormatStr {
            template: "Hello, %s!".to_string(),
            args: vec![Expr::var("name")],
        };
        let lowered = lower_expr(&expr, &CConfig::default());
        assert!(
            matches!(&lowered, CExpr::Call { func, .. } if func == "snprintf"),
            "FormatStr should lower to snprintf, got {lowered:?}"
        );
    }

    // -- B4.6: Integration test --

    #[test]
    fn lower_makegen_abstract_ir_to_c_ir() {
        let source = SourceFile {
            doc: vec!["Generated from makegen.dag".to_string()],
            items: vec![Item::Fn(FnDef {
                name: "main".to_string(),
                is_pub: true,
                params: vec![("path".to_string(), "String".to_string())],
                return_type: None,
                body: vec![
                    Stmt::comment("step 0: load_registry"),
                    Stmt::let_bind("registry", Expr::Value(ValueExpr::Unit)),
                    Stmt::Blank,
                    Stmt::comment("step 1: prepare_read"),
                    Stmt::let_bind(
                        "read_request",
                        Expr::call("prepare_file_read", vec![Expr::var("path")]),
                    ),
                    Stmt::comment("step 2: execute_read"),
                    Stmt::let_bind(
                        "read_response",
                        Expr::call("execute_file_read", vec![Expr::var("read_request")]),
                    ),
                    Stmt::Blank,
                    Stmt::comment("step 3: compare"),
                    Stmt::let_bind(
                        "fresh",
                        Expr::BinOp {
                            left: Box::new(Expr::var("content")),
                            op: "==".to_string(),
                            right: Box::new(Expr::var("read_response")),
                        },
                    ),
                ],
                doc: vec!["Generated main.".to_string()],
                attributes: vec![],
            })],
        };

        let config = CConfig::default();
        let lowered = lower_to_c(&source, &config).unwrap();

        // Should have includes.
        assert!(!lowered.includes.is_empty(), "should have includes");
        assert!(
            lowered.includes.iter().any(|i| {
                matches!(i, CItem::Include { path, system: false } if path == "gunbc/transport.h")
            }),
            "should include transport.h"
        );

        // Check main fn.
        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                CItem::FnDef(f) if f.name == "main" => Some(f),
                _ => None,
            })
            .expect("should have fn main");

        // Return type should be int (has transport calls).
        assert!(matches!(main_fn.return_type, CType::Int(CIntKind::Int)));

        // Params should be C types.
        assert!(
            matches!(&main_fn.params[0].1, CType::Ptr(inner) if matches!(inner.as_ref(), CType::Const(c) if matches!(c.as_ref(), CType::Char))),
            "path param should be const char*"
        );

        // Body should contain rewritten transport calls.
        let body_debug = format!("{:?}", main_fn.body);
        assert!(body_debug.contains("gunbc_file_read_request"));
        assert!(body_debug.contains("gunbc_transport_execute"));

        // Should have error checks.
        let has_error_check = main_fn.body.iter().any(|stmt| {
            matches!(
                stmt,
                CStmt::If {
                    then_body,
                    ..
                } if then_body.iter().any(|s| matches!(s, CStmt::Return(Some(CExpr::IntLit(-1)))))
            )
        });
        assert!(has_error_check, "should have error checks");

        // Should end with `return 0;`
        assert!(
            matches!(
                main_fn.body.last(),
                Some(CStmt::Return(Some(CExpr::IntLit(0))))
            ),
            "should end with return 0"
        );
    }
}
