//! AbstractIR → Go (ManagedIR) lowering.
//!
//! Adds Go-specific constructs: multi-return errors, short declarations,
//! goroutines, package/import management, go.mod generation.
//!
//! Provides [`lower_to_go`] which transforms a target-agnostic `SourceFile`
//! (from `lower_to_ir`) into Go-flavored IR:
//!
//! - Transport calls get `(result, err)` multi-return + `if err != nil` checks
//! - `main()` returns `error` (Go convention for fallible entrypoints)
//! - Structs lose derives (Go has no derive macros)
//! - `FormatStr` → `fmt.Sprintf(...)` call
//! - Import analysis produces Go `import (...)` block
//! - Types mapped: `String` → `string`, `Int` → `int64`, `List<T>` → `[]T`
//!
//! **Owned by**: Task 10 (dsl-codegen-tasks.md)

use crate::transport_analysis::{body_has_transport_calls, expr_is_transport_call};
use gunbc_ir::code_ir::lower::LowerError;
use gunbc_ir::code_ir::{
    BindIntent, BindTarget, CallObligation, Expr, FnDef, Import, Item, SourceFile, Stmt,
};

/// Configuration for Go lowering.
#[derive(Debug, Clone)]
pub struct GoConfig {
    /// Package name for the generated Go file (default: "main").
    pub package_name: String,
    /// Whether to use an exec-runtime equivalent for transport calls.
    /// If false, emit standalone transport function names.
    pub use_exec_runtime: bool,
}

impl Default for GoConfig {
    fn default() -> Self {
        Self {
            package_name: "main".to_string(),
            use_exec_runtime: true,
        }
    }
}

/// Lower an AbstractIR `SourceFile` to a Go-specific `SourceFile`.
pub fn lower_to_go(source: &SourceFile, config: &GoConfig) -> Result<SourceFile, LowerError> {
    let imports = collect_go_imports(source, config);
    let mut items: Vec<Item> = Vec::new();

    // Raw because: Go package declarations have no Code IR node equivalent.
    items.push(Item::Raw(format!("package {}", config.package_name)));

    // B3.3: Emit import block.
    if !imports.is_empty() {
        items.push(Item::Use(Import {
            path: imports,
            items: vec![],
        }));
    }

    // Lower each item.
    for item in &source.items {
        items.push(lower_item(item, config)?);
    }

    Ok(SourceFile {
        doc: source.doc.clone(),
        items,
    })
}

// ===========================================================================
// Item lowering
// ===========================================================================

fn lower_item(item: &Item, config: &GoConfig) -> Result<Item, LowerError> {
    match item {
        Item::Fn(f) => Ok(Item::Fn(lower_fn_def(f, config)?)),
        Item::Struct(s) => {
            // Go structs: no derives, map types.
            let mut lowered = s.clone();
            lowered.derives.clear(); // Go has no derive macros.
            lowered.fields = lowered
                .fields
                .iter()
                .map(|(name, ty, is_pub)| {
                    let go_name = if *is_pub {
                        to_pascal_case(name)
                    } else {
                        name.clone()
                    };
                    (go_name, map_to_go_type(ty), *is_pub)
                })
                .collect();
            lowered.name = to_pascal_case(&lowered.name);
            Ok(Item::Struct(lowered))
        }
        Item::Enum(e) => {
            // Go: enums become const iota blocks. Represent as Raw.
            let mut lines = Vec::new();
            lines.push(format!("type {} int", to_pascal_case(&e.name)));
            lines.push(String::new());
            lines.push("const (".to_string());
            for (i, variant) in e.variants.iter().enumerate() {
                let clean_variant = variant.split('(').next().unwrap_or(variant).trim();
                if i == 0 {
                    lines.push(format!(
                        "\t{}{} {} = iota",
                        to_pascal_case(&e.name),
                        clean_variant,
                        to_pascal_case(&e.name)
                    ));
                } else {
                    lines.push(format!("\t{}{}", to_pascal_case(&e.name), clean_variant));
                }
            }
            lines.push(")".to_string());
            // Raw because: Go const/iota enum blocks have no Code IR node equivalent.
            Ok(Item::Raw(lines.join("\n")))
        }
        // Pass through other items unchanged.
        other => Ok(other.clone()),
    }
}

// ===========================================================================
// B3.2: Function lowering with multi-return error handling
// ===========================================================================

fn lower_fn_def(f: &FnDef, config: &GoConfig) -> Result<FnDef, LowerError> {
    let has_transport = body_has_transport_calls(&f.body);

    // B3.2: If the function contains transport calls, add error return.
    let return_type = if has_transport {
        Some("error".to_string())
    } else {
        f.return_type.as_ref().map(|t| map_to_go_type(t))
    };

    // Lower body statements, inserting error checks after transport calls.
    let body = lower_body(&f.body, has_transport, config);

    // If the function is fallible, add `return nil` at the end (no error).
    let mut final_body = body;
    if has_transport {
        final_body.push(Stmt::Return(Expr::var("nil")));
    }

    Ok(FnDef {
        name: to_go_func_name(&f.name, f.is_pub),
        is_pub: f.is_pub, // Go renderer maps this to uppercase first letter.
        params: lower_params(&f.params),
        return_type,
        body: final_body,
        doc: f.doc.clone(),
        attributes: vec![], // Go has no function attributes.
    })
}

/// Lower function parameters — map abstract types to Go types.
fn lower_params(params: &[(String, String)]) -> Vec<(String, String)> {
    params
        .iter()
        .map(|(name, ty)| (to_camel_case(name), map_to_go_type(ty)))
        .collect()
}

// ===========================================================================
// B3.2: Body statement lowering with error checks
// ===========================================================================

fn lower_body(stmts: &[Stmt], in_fallible_fn: bool, config: &GoConfig) -> Vec<Stmt> {
    let mut result = Vec::new();
    for stmt in stmts {
        lower_stmt_into(&mut result, stmt, in_fallible_fn, config);
    }
    result
}

/// Lower a single statement, potentially expanding it into multiple statements
/// (e.g., transport call → assignment + error check).
fn lower_stmt_into(out: &mut Vec<Stmt>, stmt: &Stmt, in_fallible_fn: bool, config: &GoConfig) {
    match stmt {
        Stmt::Let {
            name,
            mutable: _,
            expr,
        } => {
            let lowered_expr = lower_expr(expr, config);
            let is_transport = expr_is_transport_call(expr);

            if is_transport && in_fallible_fn {
                // B3.2: Multi-return error handling.
                // `result, err := transport_call(args...)`
                out.push(Stmt::Bind {
                    targets: vec![
                        BindTarget::Name(to_camel_case(name)),
                        BindTarget::Name("err".to_string()),
                    ],
                    intent: BindIntent::Declare,
                    expr: lowered_expr,
                });
                // `if err != nil { return err }`
                out.push(Stmt::Expr(Expr::If {
                    cond: Box::new(Expr::BinOp {
                        left: Box::new(Expr::var("err")),
                        op: "!=".to_string(),
                        right: Box::new(Expr::var("nil")),
                    }),
                    then_body: vec![Stmt::Return(Expr::var("err"))],
                    else_body: None,
                }));
            } else {
                out.push(Stmt::Let {
                    name: to_camel_case(name),
                    mutable: false, // Go uses := for all short decls.
                    expr: lowered_expr,
                });
            }
        }
        Stmt::Expr(expr) => {
            let lowered = lower_expr(expr, config);
            let is_transport = expr_is_transport_call(expr);

            if is_transport && in_fallible_fn {
                // Transport as expression statement → isolate `err` in a lexical block.
                out.push(Stmt::BlockScope(vec![
                    Stmt::Bind {
                        targets: vec![BindTarget::Discard, BindTarget::Name("err".to_string())],
                        intent: BindIntent::Declare,
                        expr: lowered,
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
                ]));
            } else {
                out.push(Stmt::Expr(lowered));
            }
        }
        Stmt::Return(expr) => {
            out.push(Stmt::Return(lower_expr(expr, config)));
        }
        Stmt::Bind {
            targets,
            intent,
            expr,
        } => {
            out.push(Stmt::Bind {
                targets: targets.clone(),
                intent: *intent,
                expr: lower_expr(expr, config),
            });
        }
        Stmt::For {
            binding,
            iter,
            body,
        } => {
            out.push(Stmt::For {
                binding: to_camel_case(binding),
                iter: lower_expr(iter, config),
                body: lower_body(body, in_fallible_fn, config),
            });
        }
        Stmt::Item(item) => match lower_item(item, config) {
            Ok(lowered) => out.push(Stmt::Item(lowered)),
            Err(_) => out.push(stmt.clone()),
        },
        // Go doesn't have implicit returns — convert TailExpr to explicit return.
        Stmt::TailExpr(expr) => {
            out.push(Stmt::Return(lower_expr(expr, config)));
        }
        // Pass through Comment, Blank, Assert.
        other => out.push(other.clone()),
    }
}

fn lower_expr(expr: &Expr, config: &GoConfig) -> Expr {
    match expr {
        // B3.4: Rewrite abstract transport calls to Go runtime equivalents.
        Expr::Call {
            func,
            args,
            obligation,
        } => {
            let lowered_func = lower_expr(func, config);
            let lowered_args: Vec<Expr> = args.iter().map(|a| lower_expr(a, config)).collect();

            if obligation.is_some_and(CallObligation::is_runtime_call) {
                if let Expr::Var(name) = &lowered_func {
                    if let Some(go_fn) = rewrite_transport_call_go(name, config) {
                        return Expr::Call {
                            func: Box::new(Expr::Var(go_fn)),
                            args: lowered_args,
                            obligation: *obligation,
                        };
                    }
                }
            }

            Expr::Call {
                func: Box::new(lowered_func),
                args: lowered_args,
                obligation: *obligation,
            }
        }

        // B3.4: FormatStr → fmt.Sprintf.
        // FC-2: Convert Rust-style `{}` placeholders to Go-style `%v` format verbs.
        Expr::FormatStr { template, args } => {
            let go_template = template.replace("{}", "%v");
            let mut call_args = vec![Expr::Str(go_template)];
            call_args.extend(args.iter().map(|a| lower_expr(a, config)));
            Expr::Call {
                func: Box::new(Expr::var("fmt.Sprintf")),
                args: call_args,
                obligation: None,
            }
        }

        // Recurse into subexpressions.
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => Expr::MethodCall {
            receiver: Box::new(lower_expr(receiver, config)),
            method: to_pascal_case(method),
            args: args.iter().map(|a| lower_expr(a, config)).collect(),
        },
        Expr::BinOp { left, op, right } => Expr::BinOp {
            left: Box::new(lower_expr(left, config)),
            op: op.clone(),
            right: Box::new(lower_expr(right, config)),
        },
        Expr::UnaryOp { op, expr: inner } => Expr::UnaryOp {
            op: op.clone(),
            expr: Box::new(lower_expr(inner, config)),
        },
        Expr::If {
            cond,
            then_body,
            else_body,
        } => Expr::If {
            cond: Box::new(lower_expr(cond, config)),
            then_body: lower_body(then_body, false, config),
            else_body: else_body.as_ref().map(|b| lower_body(b, false, config)),
        },
        Expr::Block(stmts) => Expr::Block(lower_body(stmts, false, config)),
        Expr::Field(inner, field) => {
            Expr::Field(Box::new(lower_expr(inner, config)), to_pascal_case(field))
        }
        Expr::Array(elems) => Expr::Array(elems.iter().map(|e| lower_expr(e, config)).collect()),
        Expr::Tuple(elems) => Expr::Tuple(elems.iter().map(|e| lower_expr(e, config)).collect()),
        Expr::Closure { args, body } => Expr::Closure {
            args: args.clone(),
            body: Box::new(lower_expr(body, config)),
        },
        Expr::Struct { name, fields, rest } => Expr::Struct {
            name: to_pascal_case(name),
            fields: fields
                .iter()
                .map(|(k, v)| (to_pascal_case(k), lower_expr(v, config)))
                .collect(),
            rest: rest.as_ref().map(|r| Box::new(lower_expr(r, config))),
        },
        // MacroCall doesn't exist in Go — convert to function call.
        Expr::MacroCall { name, args } => Expr::Call {
            func: Box::new(Expr::var(name.clone())),
            args: args.iter().map(|a| lower_expr(a, config)).collect(),
            obligation: None,
        },
        // Leaf expressions pass through.
        other => other.clone(),
    }
}

// ===========================================================================
// Transport call detection and rewriting
// ===========================================================================

fn rewrite_transport_call_go(name: &str, config: &GoConfig) -> Option<String> {
    if !config.use_exec_runtime {
        return None;
    }

    match name {
        "prepare_file_read" => Some("transport.NewFileReadRequest".to_string()),
        "execute_file_read" => Some("transport.Execute".to_string()),
        "parse_file_read_response" => Some("transport.ParseFileResponse".to_string()),
        "prepare_file_write" => Some("transport.NewFileWriteRequest".to_string()),
        "execute_file_write" => Some("transport.Execute".to_string()),
        "parse_file_write_response" => Some("transport.ParseFileResponse".to_string()),
        "prepare_file_exists" => Some("transport.NewFileExistsRequest".to_string()),
        "execute_file_exists" => Some("transport.Execute".to_string()),
        "prepare_shell_exec" => Some("transport.NewShellRequest".to_string()),
        "execute_shell_exec" => Some("transport.Execute".to_string()),
        "parse_shell_exec_response" => Some("transport.ParseShellResponse".to_string()),
        "prepare_http_request" => Some("transport.NewHTTPRequest".to_string()),
        "execute_http_request" => Some("transport.Execute".to_string()),
        "prepare_directory_list" => Some("transport.NewDirListRequest".to_string()),
        "execute_directory_list" => Some("transport.Execute".to_string()),
        "acquire_resource" => Some("resource.Acquire".to_string()),
        _ => None,
    }
}

// ===========================================================================
// B3.3: Import analysis
// ===========================================================================

fn collect_go_imports(source: &SourceFile, config: &GoConfig) -> Vec<String> {
    let mut imports = Vec::new();
    let has_transport = source
        .items
        .iter()
        .any(|item| matches!(item, Item::Fn(f) if body_has_transport_calls(&f.body)));

    let has_format = source.items.iter().any(|item| {
        if let Item::Fn(f) = item {
            body_has_format(&f.body)
        } else {
            false
        }
    });

    let has_json = source.items.iter().any(|item| {
        if let Item::Fn(f) = item {
            body_uses_json(&f.body)
        } else {
            false
        }
    });

    if has_format {
        imports.push("fmt".to_string());
    }
    if has_json {
        imports.push("encoding/json".to_string());
    }
    if has_transport && config.use_exec_runtime {
        imports.push("github.com/gunb-ai/gunbc/transport".to_string());
    }

    imports.sort();
    imports.dedup();
    imports
}

fn body_has_format(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Let { expr, .. } => expr_has_format(expr),
        Stmt::Bind { expr, .. } => expr_has_format(expr),
        Stmt::Expr(expr) | Stmt::Return(expr) | Stmt::TailExpr(expr) => expr_has_format(expr),
        Stmt::For { body, .. } => body_has_format(body),
        _ => false,
    })
}

fn expr_has_format(expr: &Expr) -> bool {
    match expr {
        Expr::FormatStr { .. } => true,
        Expr::Call { func, args, .. } => expr_has_format(func) || args.iter().any(expr_has_format),
        Expr::MethodCall { receiver, args, .. } => {
            expr_has_format(receiver) || args.iter().any(expr_has_format)
        }
        Expr::BinOp { left, right, .. } => expr_has_format(left) || expr_has_format(right),
        Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_has_format(cond)
                || body_has_format(then_body)
                || else_body.as_ref().is_some_and(|b| body_has_format(b))
        }
        Expr::Block(stmts) => body_has_format(stmts),
        _ => false,
    }
}

fn body_uses_json(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Let { expr, .. } => expr_uses_json(expr),
        Stmt::Bind { expr, .. } => expr_uses_json(expr),
        Stmt::Expr(expr) | Stmt::Return(expr) | Stmt::TailExpr(expr) => expr_uses_json(expr),
        Stmt::For { body, .. } => body_uses_json(body),
        _ => false,
    })
}

fn expr_uses_json(expr: &Expr) -> bool {
    match expr {
        Expr::Value(gunbc_ir::ValueExpr::Json(_)) => true,
        Expr::Call { func, args, .. } => expr_uses_json(func) || args.iter().any(expr_uses_json),
        Expr::MethodCall { receiver, args, .. } => {
            expr_uses_json(receiver) || args.iter().any(expr_uses_json)
        }
        Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_uses_json(cond)
                || body_uses_json(then_body)
                || else_body.as_ref().is_some_and(|b| body_uses_json(b))
        }
        Expr::Block(stmts) => body_uses_json(stmts),
        _ => false,
    }
}

// ===========================================================================
// B3.4: Type mapping
// ===========================================================================

/// Map an abstract type name to its Go equivalent.
///
/// When a `TypeRegistry` is available, delegates to `resolve_and_emit` for
/// structural type resolution. Falls back to the static mapping table.
fn map_to_go_type_with_registry(
    abstract_type: &str,
    registry: Option<&gunbc_ir::TypeRegistry>,
) -> String {
    crate::type_mapping::resolve_and_emit(
        abstract_type,
        registry,
        crate::type_mapping::Backend::Go,
    )
}

/// Map an abstract type name to its Go equivalent (no registry).
fn map_to_go_type(abstract_type: &str) -> String {
    map_to_go_type_with_registry(abstract_type, None)
}

// ===========================================================================
// Naming conventions
// ===========================================================================

/// Convert a snake_case name to camelCase (Go unexported convention).
fn to_camel_case(name: &str) -> String {
    let parts: Vec<&str> = name.split('_').collect();
    if parts.is_empty() {
        return name.to_string();
    }
    let mut result = parts[0].to_string();
    for part in &parts[1..] {
        if let Some(first) = part.chars().next() {
            result.push(first.to_ascii_uppercase());
            result.push_str(&part[first.len_utf8()..]);
        }
    }
    result
}

/// Convert a snake_case name to PascalCase (Go exported convention).
fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => {
                    let mut s = c.to_ascii_uppercase().to_string();
                    s.push_str(chars.as_str());
                    s
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Convert function name to Go convention.
fn to_go_func_name(name: &str, is_pub: bool) -> String {
    if is_pub {
        to_pascal_case(name)
    } else {
        to_camel_case(name)
    }
}

// ===========================================================================
// Tests (B3.5)
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
                params: vec![("file_path".to_string(), "String".to_string())],
                return_type: None,
                body: stmts,
                doc: vec![],
                attributes: vec![],
            })],
        }
    }

    // -- B3.2: Multi-return error handling --

    #[test]
    fn error_return_for_transport_functions() {
        let source = make_abstract_main(vec![
            Stmt::let_bind(
                "request",
                Expr::call_with_obligation(
                    "prepare_file_read",
                    vec![Expr::var("file_path")],
                    CallObligation::ServiceTransportPrepare,
                ),
            ),
            Stmt::let_bind(
                "response",
                Expr::call_with_obligation(
                    "execute_file_read",
                    vec![Expr::var("request")],
                    CallObligation::ServiceTransportExecute,
                ),
            ),
        ]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .expect("should have fn Main");

        // Return type should be error.
        assert_eq!(
            main_fn.return_type,
            Some("error".to_string()),
            "main should return error when it has transport calls"
        );

        // Should end with `return nil`.
        let last_stmt = main_fn.body.last().unwrap();
        assert!(
            matches!(last_stmt, Stmt::Return(Expr::Var(name)) if name == "nil"),
            "should end with return nil, got {last_stmt:?}"
        );
    }

    #[test]
    fn multi_return_error_check_inserted() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "response",
            Expr::call_with_obligation(
                "execute_file_read",
                vec![Expr::var("req")],
                CallObligation::ServiceTransportExecute,
            ),
        )]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .unwrap();

        // Should have: typed multi-target bind + `if err != nil { return err }`.
        let has_typed_bind = main_fn.body.iter().any(|stmt| {
            matches!(
                stmt,
                Stmt::Bind {
                    targets,
                    intent: BindIntent::Declare,
                    ..
                } if matches!(
                    targets.as_slice(),
                    [BindTarget::Name(result), BindTarget::Name(err)]
                        if result == "response" && err == "err"
                )
            )
        });
        assert!(
            has_typed_bind,
            "should have typed multi-target bind for response/err"
        );

        let has_err_check = main_fn.body.iter().any(|stmt| {
            matches!(
                stmt,
                Stmt::Expr(Expr::If { cond, .. })
                    if matches!(
                        cond.as_ref(),
                        Expr::BinOp { left, op, right }
                            if matches!(
                                (&**left, op.as_str(), &**right),
                                (Expr::Var(lhs), "!=", Expr::Var(rhs)) if lhs == "err" && rhs == "nil"
                            )
                    )
            )
        });
        assert!(has_err_check, "should have explicit err != nil check");
    }

    #[test]
    fn repeated_transport_expression_statements_are_block_scoped() {
        let source = make_abstract_main(vec![
            Stmt::Expr(Expr::call_with_obligation(
                "execute_file_read",
                vec![Expr::var("req_a")],
                CallObligation::ServiceTransportExecute,
            )),
            Stmt::Expr(Expr::call_with_obligation(
                "execute_file_read",
                vec![Expr::var("req_b")],
                CallObligation::ServiceTransportExecute,
            )),
        ]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .expect("should have fn Main");

        let scoped_err_blocks = main_fn
            .body
            .iter()
            .filter(|stmt| {
                matches!(
                    stmt,
                    Stmt::BlockScope(inner)
                        if matches!(
                            inner.first(),
                            Some(Stmt::Bind {
                                targets,
                                intent: BindIntent::Declare,
                                ..
                            }) if matches!(
                                targets.as_slice(),
                                [BindTarget::Discard, BindTarget::Name(err)] if err == "err"
                            )
                        )
                )
            })
            .count();
        assert_eq!(
            scoped_err_blocks, 2,
            "each transport expression statement should isolate err declaration in its own block"
        );
    }

    #[test]
    fn no_error_return_for_pure_functions() {
        let source = make_abstract_main(vec![
            Stmt::let_bind("x", Expr::IntLit(42)),
            Stmt::let_bind("y", Expr::BoolLit(true)),
        ]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .unwrap();

        assert_eq!(
            main_fn.return_type, None,
            "pure function should not get error return"
        );
    }

    // -- B3.3: Package + imports --

    #[test]
    fn package_declaration_added() {
        let source = make_abstract_main(vec![Stmt::let_bind("x", Expr::IntLit(42))]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let first_item = &lowered.items[0];
        assert!(
            matches!(first_item, Item::Raw(s) if s == "package main"),
            "first item should be package declaration, got {first_item:?}"
        );
    }

    #[test]
    fn imports_generated_for_transport() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "resp",
            Expr::call_with_obligation(
                "execute_file_read",
                vec![Expr::var("req")],
                CallObligation::ServiceTransportExecute,
            ),
        )]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let imports: Vec<&Import> = lowered
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Use(import) => Some(import),
                _ => None,
            })
            .collect();

        assert!(
            imports
                .iter()
                .any(|i| i.path.iter().any(|p| p.contains("transport"))),
            "should import transport package, got imports: {:?}",
            imports
        );
    }

    #[test]
    fn fmt_import_for_format_strings() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "msg",
            Expr::FormatStr {
                template: "Hello, %s!".to_string(),
                args: vec![Expr::var("name")],
            },
        )]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let imports: Vec<&Import> = lowered
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Use(import) => Some(import),
                _ => None,
            })
            .collect();

        assert!(
            imports.iter().any(|i| i.path.contains(&"fmt".to_string())),
            "should import fmt, got imports: {:?}",
            imports
        );
    }

    // -- B3.4: Type mapping --

    #[test]
    fn map_abstract_types_to_go() {
        assert_eq!(map_to_go_type("String"), "string");
        assert_eq!(map_to_go_type("Bool"), "bool");
        assert_eq!(map_to_go_type("Int"), "int64");
        assert_eq!(map_to_go_type("Float"), "float64");
        assert_eq!(map_to_go_type("Path"), "string");
        assert_eq!(map_to_go_type("FilesystemHandle"), "string");
        assert_eq!(map_to_go_type("ToolRegistry"), "interface{}");
        assert_eq!(map_to_go_type("List<String>"), "[]string");
        assert_eq!(map_to_go_type("Optional<Int>"), "*int64");
        assert_eq!(map_to_go_type("Map<String, Int>"), "map[string]int64");
        assert_eq!(map_to_go_type("UnknownType"), "interface{}");
    }

    #[test]
    fn map_to_go_type_with_registry_structural_emit() {
        use gunbc_ir::type_op::Predicate;
        let mut registry = gunbc_ir::TypeRegistry::with_primitives();
        registry.register(
            "UInt32",
            gunbc_ir::type_lib::refined("Int", vec![
                Predicate::Width(32),
                Predicate::Unsigned,
                Predicate::Arithmetic,
            ]),
        );
        assert_eq!(map_to_go_type_with_registry("UInt32", Some(&registry)), "uint32");
        // Fallback still works
        assert_eq!(map_to_go_type_with_registry("String", Some(&registry)), "string");
    }

    // -- B3.4: FormatStr lowering --

    #[test]
    fn format_str_becomes_sprintf() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "msg",
            Expr::FormatStr {
                template: "Hello, %s!".to_string(),
                args: vec![Expr::var("name")],
            },
        )]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .unwrap();

        let body_debug = format!("{:?}", main_fn.body);
        assert!(
            body_debug.contains("fmt.Sprintf"),
            "FormatStr should lower to fmt.Sprintf, body: {body_debug}"
        );
    }

    // -- B3.4: Transport call rewriting --

    #[test]
    fn transport_calls_rewritten_to_go_runtime() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "req",
            Expr::call_with_obligation(
                "prepare_file_read",
                vec![Expr::var("path")],
                CallObligation::ServiceTransportPrepare,
            ),
        )]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .unwrap();

        let body_debug = format!("{:?}", main_fn.body);
        assert!(
            body_debug.contains("transport.NewFileReadRequest"),
            "prepare_file_read should be rewritten, body: {body_debug}"
        );
    }

    #[test]
    fn transport_named_call_without_obligation_is_not_treated_as_runtime() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "req",
            Expr::call("prepare_file_read", vec![Expr::var("path")]),
        )]);

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let imports: Vec<&Import> = lowered
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Use(import) => Some(import),
                _ => None,
            })
            .collect();
        assert!(
            imports
                .iter()
                .all(|i| !i.path.iter().any(|p| p.contains("transport"))),
            "call names alone should not trigger transport imports"
        );

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .expect("should have fn Main");
        assert_eq!(main_fn.return_type, None);

        let body_debug = format!("{:?}", main_fn.body);
        assert!(body_debug.contains("prepare_file_read"));
        assert!(!body_debug.contains("transport.NewFileReadRequest"));
    }

    #[test]
    fn transport_calls_preserved_in_standalone_mode() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "req",
            Expr::call_with_obligation(
                "prepare_file_read",
                vec![Expr::var("path")],
                CallObligation::ServiceTransportPrepare,
            ),
        )]);

        let config = GoConfig {
            use_exec_runtime: false,
            ..GoConfig::default()
        };
        let lowered = lower_to_go(&source, &config).unwrap();

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .unwrap();

        let body_debug = format!("{:?}", main_fn.body);
        assert!(
            body_debug.contains("prepare_file_read"),
            "standalone mode should keep abstract names, body: {body_debug}"
        );
    }

    // -- Naming conventions --

    #[test]
    fn naming_conversions() {
        assert_eq!(to_camel_case("file_path"), "filePath");
        assert_eq!(to_camel_case("x"), "x");
        assert_eq!(to_pascal_case("file_path"), "FilePath");
        assert_eq!(to_pascal_case("main"), "Main");
        assert_eq!(to_go_func_name("render_makefile", true), "RenderMakefile");
        assert_eq!(to_go_func_name("render_makefile", false), "renderMakefile");
    }

    // -- Struct lowering --

    #[test]
    fn struct_derives_removed_and_types_mapped() {
        let source = SourceFile {
            doc: vec![],
            items: vec![Item::Struct(StructDef {
                name: "config".to_string(),
                is_pub: true,
                derives: vec!["Debug".to_string(), "Clone".to_string()],
                fields: vec![
                    ("file_name".to_string(), "String".to_string(), true),
                    ("count".to_string(), "Int".to_string(), false),
                ],
                doc: vec![],
            })],
        };

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        // Skip package declaration.
        let struct_item = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Struct(s) => Some(s),
                _ => None,
            })
            .expect("should have struct");

        assert!(struct_item.derives.is_empty(), "Go has no derives");
        assert_eq!(struct_item.name, "Config", "name should be PascalCase");
        assert_eq!(struct_item.fields[0].0, "FileName", "pub field PascalCase");
        assert_eq!(struct_item.fields[0].1, "string", "String → string");
        assert_eq!(struct_item.fields[1].1, "int64", "Int → int64");
    }

    // -- Enum lowering --

    #[test]
    fn enum_becomes_const_iota() {
        let source = SourceFile {
            doc: vec![],
            items: vec![Item::Enum(EnumDef {
                name: "op".to_string(),
                is_pub: true,
                derives: vec![],
                variants: vec!["Read".to_string(), "Write".to_string()],
                doc: vec![],
            })],
        };

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        let raw_item = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Raw(code) if code.contains("type") => Some(code),
                _ => None,
            })
            .expect("should have Raw item for enum");

        assert!(raw_item.contains("type Op int"), "should define type");
        assert!(raw_item.contains("iota"), "should use iota");
        assert!(raw_item.contains("OpRead"), "should have prefixed variants");
        assert!(
            raw_item.contains("OpWrite"),
            "should have prefixed variants"
        );
    }

    // -- B3.5: Integration test --

    #[test]
    fn lower_makegen_abstract_ir_to_go_ir() {
        let source = SourceFile {
            doc: vec!["Generated from makegen.dag".to_string()],
            items: vec![Item::Fn(FnDef {
                name: "main".to_string(),
                is_pub: true,
                params: vec![("file_path".to_string(), "String".to_string())],
                return_type: None,
                body: vec![
                    Stmt::comment("step 0: load_registry"),
                    Stmt::let_bind("registry", Expr::Value(ValueExpr::Unit)),
                    Stmt::Blank,
                    Stmt::comment("step 1: render_makefile"),
                    Stmt::let_bind(
                        "content",
                        Expr::FormatStr {
                            template: "render_makefile".to_string(),
                            args: vec![Expr::var("registry")],
                        },
                    ),
                    Stmt::Blank,
                    Stmt::comment("step 2: prepare_read"),
                    Stmt::let_bind(
                        "read_request",
                        Expr::call_with_obligation(
                            "prepare_file_read",
                            vec![Expr::var("file_path")],
                            CallObligation::ServiceTransportPrepare,
                        ),
                    ),
                    Stmt::comment("step 3: execute_read"),
                    Stmt::let_bind(
                        "read_response",
                        Expr::call_with_obligation(
                            "execute_file_read",
                            vec![Expr::var("read_request")],
                            CallObligation::ServiceTransportExecute,
                        ),
                    ),
                    Stmt::Blank,
                    Stmt::comment("step 4: compare"),
                    Stmt::let_bind(
                        "fresh",
                        Expr::BinOp {
                            left: Box::new(Expr::var("content")),
                            op: "==".to_string(),
                            right: Box::new(Expr::var("read_response")),
                        },
                    ),
                ],
                doc: vec!["Generated main from EmitPlan.".to_string()],
                attributes: vec![],
            })],
        };

        let config = GoConfig::default();
        let lowered = lower_to_go(&source, &config).unwrap();

        // Should have package declaration.
        assert!(
            matches!(&lowered.items[0], Item::Raw(s) if s.contains("package main")),
            "first item should be package declaration"
        );

        // Should have imports.
        let has_transport_import = lowered.items.iter().any(|item| {
            matches!(item, Item::Use(import) if import.path.iter().any(|p| p.contains("transport")))
        });
        assert!(has_transport_import, "should import transport");

        // Check main fn.
        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "Main" => Some(f),
                _ => None,
            })
            .expect("should have fn Main");

        // Return type should be error.
        assert_eq!(main_fn.return_type, Some("error".to_string()));

        // Params should use Go types + camelCase.
        assert_eq!(main_fn.params[0].0, "filePath");
        assert_eq!(main_fn.params[0].1, "string");

        // Body should contain rewritten transport calls.
        let body_debug = format!("{:?}", main_fn.body);
        assert!(body_debug.contains("transport.NewFileReadRequest"));
        assert!(body_debug.contains("transport.Execute"));

        // FormatStr should become fmt.Sprintf.
        assert!(body_debug.contains("fmt.Sprintf"));

        // Should end with return nil.
        assert!(
            matches!(main_fn.body.last(), Some(Stmt::Return(Expr::Var(name))) if name == "nil"),
            "should end with return nil"
        );
    }
}
