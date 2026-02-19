//! AbstractIR → Rust (SystemsIR) lowering.
//!
//! Adds Rust-specific constructs: ownership, Result types, derive macros,
//! use statements, Cargo.toml generation.
//!
//! Provides [`lower_to_rust`] which transforms a target-agnostic `SourceFile`
//! (from `lower_to_ir`) into Rust-flavored IR:
//!
//! - Transport calls get `?` operator (Result propagation)
//! - `main()` becomes `fn main() -> Result<(), ExecError>`
//! - Structs/enums get `#[derive(Debug, Clone)]`
//! - Import analysis produces `use` statements
//! - `FormatStr` → `format!()` macro calls
//!
//! **Owned by**: Task 9 (dsl-codegen-tasks.md)

use gunbc_ir::code_ir::lower::LowerError;
use crate::transport_analysis::{body_has_transport_calls, expr_is_transport_call};
use gunbc_ir::code_ir::{CallObligation, Expr, FnDef, Import, Item, SourceFile, Stmt};

/// Configuration for Rust lowering.
#[derive(Debug, Clone)]
pub struct RustConfig {
    /// Whether to use the gunbc-exec runtime for transport calls.
    /// If false, emit standalone transport functions.
    pub use_exec_runtime: bool,
    /// Error type name for Result wrapping (default: "ExecError").
    pub error_type: String,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            use_exec_runtime: true,
            error_type: "ExecError".to_string(),
        }
    }
}

/// Lower an AbstractIR `SourceFile` to a Rust-specific `SourceFile`.
pub fn lower_to_rust(source: &SourceFile, config: &RustConfig) -> Result<SourceFile, LowerError> {
    let imports = collect_imports(source, config);
    let mut items: Vec<Item> = Vec::new();

    // B2.3: Emit use statements first.
    for import in &imports {
        items.push(Item::Use(import.clone()));
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

fn lower_item(item: &Item, config: &RustConfig) -> Result<Item, LowerError> {
    match item {
        Item::Fn(f) => Ok(Item::Fn(lower_fn_def(f, config)?)),
        Item::Struct(s) => {
            // B2.2: Add derive macros.
            let mut lowered = s.clone();
            if lowered.derives.is_empty() {
                lowered.derives = vec!["Debug".to_string(), "Clone".to_string()];
            }
            Ok(Item::Struct(lowered))
        }
        Item::Enum(e) => {
            // B2.2: Add derive macros.
            let mut lowered = e.clone();
            if lowered.derives.is_empty() {
                lowered.derives = vec!["Debug".to_string(), "Clone".to_string()];
            }
            Ok(Item::Enum(lowered))
        }
        // Pass through other items unchanged.
        other => Ok(other.clone()),
    }
}

// ===========================================================================
// B2.1: Function lowering with Result wrapping
// ===========================================================================

fn lower_fn_def(f: &FnDef, config: &RustConfig) -> Result<FnDef, LowerError> {
    let has_transport = body_has_transport_calls(&f.body);

    // B2.1: If the function contains transport calls, wrap return type in Result.
    let return_type = if has_transport {
        Some(format!("Result<(), {}>", config.error_type))
    } else {
        f.return_type.clone()
    };

    // B2.4 + B2.5: Lower body statements.
    let body = lower_body(&f.body, has_transport, config);

    // If the function is fallible, add `Ok(())` at the end.
    let mut final_body = body;
    if has_transport {
        final_body.push(Stmt::TailExpr(Expr::call("Ok", vec![Expr::Tuple(vec![])])));
    }

    Ok(FnDef {
        name: f.name.clone(),
        is_pub: f.is_pub,
        params: lower_params(&f.params, config),
        return_type,
        body: final_body,
        doc: f.doc.clone(),
        attributes: f.attributes.clone(),
    })
}

/// Lower function parameters — map abstract types to Rust types.
fn lower_params(params: &[(String, String)], _config: &RustConfig) -> Vec<(String, String)> {
    params
        .iter()
        .map(|(name, ty)| (name.clone(), map_to_rust_type(ty)))
        .collect()
}

// ===========================================================================
// B2.4 + B2.5: Body statement lowering
// ===========================================================================

fn lower_body(stmts: &[Stmt], in_fallible_fn: bool, config: &RustConfig) -> Vec<Stmt> {
    stmts
        .iter()
        .map(|stmt| lower_stmt(stmt, in_fallible_fn, config))
        .collect()
}

fn lower_stmt(stmt: &Stmt, in_fallible_fn: bool, config: &RustConfig) -> Stmt {
    match stmt {
        Stmt::Let {
            name,
            mutable,
            expr,
        } => Stmt::Let {
            name: name.clone(),
            mutable: *mutable,
            expr: lower_expr(expr, in_fallible_fn, config),
        },
        Stmt::Expr(expr) => Stmt::Expr(lower_expr(expr, in_fallible_fn, config)),
        Stmt::Return(expr) => Stmt::Return(lower_expr(expr, in_fallible_fn, config)),
        Stmt::For {
            binding,
            iter,
            body,
        } => Stmt::For {
            binding: binding.clone(),
            iter: lower_expr(iter, in_fallible_fn, config),
            body: lower_body(body, in_fallible_fn, config),
        },
        Stmt::Item(item) => match lower_item(item, config) {
            Ok(lowered) => Stmt::Item(lowered),
            Err(_) => stmt.clone(),
        },
        // Pass through Comment, Blank, TailExpr, Assert.
        other => other.clone(),
    }
}

fn lower_expr(expr: &Expr, in_fallible_fn: bool, config: &RustConfig) -> Expr {
    match expr {
        // B2.5: Rewrite abstract transport calls to concrete Rust runtime calls.
        Expr::Call {
            func,
            args,
            obligation,
        } => {
            let lowered_func = lower_expr(func, in_fallible_fn, config);
            let lowered_args: Vec<Expr> = args
                .iter()
                .map(|a| lower_expr(a, in_fallible_fn, config))
                .collect();

            let call = if obligation.is_some_and(CallObligation::is_runtime_call) {
                if let Expr::Var(name) = &lowered_func {
                    if let Some(rust_fn) = rewrite_transport_call(name, config) {
                        Expr::Call {
                            func: Box::new(Expr::Var(rust_fn)),
                            args: lower_transport_args(&lowered_args, name, config),
                            obligation: *obligation,
                        }
                    } else {
                        Expr::Call {
                            func: Box::new(lowered_func),
                            args: lowered_args,
                            obligation: *obligation,
                        }
                    }
                } else {
                    Expr::Call {
                        func: Box::new(lowered_func),
                        args: lowered_args,
                        obligation: *obligation,
                    }
                }
            } else {
                Expr::Call {
                    func: Box::new(lowered_func),
                    args: lowered_args,
                    obligation: *obligation,
                }
            };

            // B2.1: Add ? operator for transport calls in fallible functions.
            if in_fallible_fn && expr_is_transport_call(&call) {
                // Wrap with .expect() → in real codegen this would be `?`.
                // Since code_ir doesn't have a Try/QuestionMark expression,
                // we use a MacroCall or a method call pattern.
                Expr::MethodCall {
                    receiver: Box::new(call),
                    method: "?".to_string(),
                    args: vec![],
                }
            } else {
                call
            }
        }

        // B2.4: String literals in certain positions → .to_string().
        Expr::Str(s) => Expr::Str(s.clone()),

        // Recurse into subexpressions.
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => Expr::MethodCall {
            receiver: Box::new(lower_expr(receiver, in_fallible_fn, config)),
            method: method.clone(),
            args: args
                .iter()
                .map(|a| lower_expr(a, in_fallible_fn, config))
                .collect(),
        },
        Expr::BinOp { left, op, right } => Expr::BinOp {
            left: Box::new(lower_expr(left, in_fallible_fn, config)),
            op: op.clone(),
            right: Box::new(lower_expr(right, in_fallible_fn, config)),
        },
        Expr::UnaryOp { op, expr: inner } => Expr::UnaryOp {
            op: op.clone(),
            expr: Box::new(lower_expr(inner, in_fallible_fn, config)),
        },
        Expr::If {
            cond,
            then_body,
            else_body,
        } => Expr::If {
            cond: Box::new(lower_expr(cond, in_fallible_fn, config)),
            then_body: lower_body(then_body, in_fallible_fn, config),
            else_body: else_body
                .as_ref()
                .map(|b| lower_body(b, in_fallible_fn, config)),
        },
        Expr::Block(stmts) => Expr::Block(lower_body(stmts, in_fallible_fn, config)),
        Expr::FormatStr { template, args } => Expr::MacroCall {
            name: "format".to_string(),
            args: {
                let mut macro_args = vec![Expr::Str(template.clone())];
                macro_args.extend(args.iter().map(|a| lower_expr(a, in_fallible_fn, config)));
                macro_args
            },
        },
        Expr::Field(inner, field) => Expr::Field(
            Box::new(lower_expr(inner, in_fallible_fn, config)),
            field.clone(),
        ),
        Expr::Array(elems) => Expr::Array(
            elems
                .iter()
                .map(|e| lower_expr(e, in_fallible_fn, config))
                .collect(),
        ),
        Expr::Tuple(elems) => Expr::Tuple(
            elems
                .iter()
                .map(|e| lower_expr(e, in_fallible_fn, config))
                .collect(),
        ),
        Expr::Closure { args, body } => Expr::Closure {
            args: args.clone(),
            body: Box::new(lower_expr(body, in_fallible_fn, config)),
        },
        Expr::Struct { name, fields } => Expr::Struct {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v, in_fallible_fn, config)))
                .collect(),
        },
        // Leaf expressions pass through.
        other => other.clone(),
    }
}

// ===========================================================================
// B2.5: Transport call rewriting
// ===========================================================================

/// Rewrite abstract transport function names to concrete Rust runtime functions.
fn rewrite_transport_call(name: &str, config: &RustConfig) -> Option<String> {
    if !config.use_exec_runtime {
        return None; // Standalone mode: keep abstract names.
    }

    match name {
        "prepare_file_read" => Some("FileRequest::read".to_string()),
        "execute_file_read" => Some("execute_transport".to_string()),
        "parse_file_read_response" => Some("parse_file_response".to_string()),
        "prepare_file_write" => Some("FileRequest::write".to_string()),
        "execute_file_write" => Some("execute_transport".to_string()),
        "parse_file_write_response" => Some("parse_file_response".to_string()),
        "prepare_file_exists" => Some("FileRequest::exists".to_string()),
        "execute_file_exists" => Some("execute_transport".to_string()),
        "prepare_shell_exec" => Some("ShellRequest::new".to_string()),
        "execute_shell_exec" => Some("execute_transport".to_string()),
        "parse_shell_exec_response" => Some("parse_shell_response".to_string()),
        "prepare_http_request" => Some("RestRequest::new".to_string()),
        "execute_http_request" => Some("execute_transport".to_string()),
        "prepare_directory_list" => Some("FileRequest::list_dir".to_string()),
        "execute_directory_list" => Some("execute_transport".to_string()),
        "acquire_resource" => Some("acquire_resource_handle".to_string()),
        _ => None,
    }
}

/// Lower transport call arguments for Rust runtime conventions.
fn lower_transport_args(args: &[Expr], _fn_name: &str, _config: &RustConfig) -> Vec<Expr> {
    // For now, pass args through. Future: convert string args to &str references.
    args.to_vec()
}

// ===========================================================================
// B2.3: Import analysis
// ===========================================================================

/// Analyze the source file and determine needed `use` statements.
fn collect_imports(source: &SourceFile, config: &RustConfig) -> Vec<Import> {
    let mut imports = Vec::new();
    let has_transport = source
        .items
        .iter()
        .any(|item| matches!(item, Item::Fn(f) if body_has_transport_calls(&f.body)));

    if has_transport && config.use_exec_runtime {
        imports.push(Import {
            path: vec!["gunbc_exec".to_string()],
            items: vec!["execute_transport".to_string(), "ExecError".to_string()],
        });
        imports.push(Import {
            path: vec!["gunbc_ir".to_string(), "transport".to_string()],
            items: vec![
                "FileRequest".to_string(),
                "ShellRequest".to_string(),
                "RestRequest".to_string(),
            ],
        });
    }

    // Check for serde_json usage.
    let has_json = source.items.iter().any(|item| {
        if let Item::Fn(f) = item {
            body_uses_json(&f.body)
        } else {
            false
        }
    });
    if has_json {
        imports.push(Import {
            path: vec!["serde_json".to_string()],
            items: vec!["Value".to_string()],
        });
    }

    imports
}

fn body_uses_json(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Let { expr, .. } => expr_uses_json(expr),
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
        Expr::BinOp { left, right, .. } => expr_uses_json(left) || expr_uses_json(right),
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
        Expr::Array(elems) | Expr::Tuple(elems) => elems.iter().any(expr_uses_json),
        _ => false,
    }
}

// ===========================================================================
// Type mapping
// ===========================================================================

/// Map an abstract type name to its Rust equivalent.
fn map_to_rust_type(abstract_type: &str) -> String {
    match abstract_type {
        "String" | "Path" => "String".to_string(),
        "Bool" | "bool" => "bool".to_string(),
        "Int" | "i64" | "I64" => "i64".to_string(),
        "ToolRegistry" => "serde_json::Value".to_string(),
        "TransportRequest" => "TransportRequest".to_string(),
        "TransportResponse" => "TransportResponse".to_string(),
        "FilesystemHandle" => "PathBuf".to_string(),
        other => {
            // Check for List<T> pattern.
            if let Some(inner) = other
                .strip_prefix("List<")
                .and_then(|rest| rest.strip_suffix('>'))
            {
                return format!("Vec<{}>", map_to_rust_type(inner));
            }
            // Default: treat as opaque serde_json::Value.
            "serde_json::Value".to_string()
        }
    }
}

// ===========================================================================
// Tests (B2.6)
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

    // -- B2.1: Result wrapping --

    #[test]
    fn result_wrapping_for_transport_functions() {
        let source = make_abstract_main(vec![
            Stmt::comment("step 0: load"),
            Stmt::let_bind("registry", Expr::Value(ValueExpr::Unit)),
            Stmt::comment("step 1: transport"),
            Stmt::let_bind(
                "request",
                Expr::call_with_obligation(
                    "prepare_file_read",
                    vec![Expr::var("path")],
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

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        let main_fn = match &lowered.items.last().unwrap() {
            Item::Fn(f) => f,
            other => panic!("expected Fn, got {other:?}"),
        };

        // Return type should be Result.
        assert_eq!(
            main_fn.return_type,
            Some("Result<(), ExecError>".to_string()),
            "main should return Result when it has transport calls"
        );

        // Should end with Ok(()).
        let last_stmt = main_fn.body.last().unwrap();
        assert!(
            matches!(last_stmt, Stmt::TailExpr(Expr::Call { func, .. })
                if matches!(func.as_ref(), Expr::Var(name) if name == "Ok")),
            "should end with Ok(()) tail expression, got {last_stmt:?}"
        );
    }

    #[test]
    fn no_result_wrapping_for_pure_functions() {
        let source = make_abstract_main(vec![
            Stmt::let_bind("x", Expr::IntLit(42)),
            Stmt::let_bind("y", Expr::BoolLit(true)),
        ]);

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        let main_fn = match &lowered.items.last().unwrap() {
            Item::Fn(f) => f,
            other => panic!("expected Fn, got {other:?}"),
        };

        assert_eq!(
            main_fn.return_type, None,
            "pure function should not get Result wrapping"
        );
    }

    // -- B2.2: Derive macros --

    #[test]
    fn derives_added_to_structs() {
        let source = SourceFile {
            doc: vec![],
            items: vec![Item::Struct(StructDef {
                name: "Config".to_string(),
                is_pub: true,
                derives: vec![],
                fields: vec![("name".to_string(), "String".to_string(), true)],
                doc: vec![],
            })],
        };

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        match &lowered.items.last().unwrap() {
            Item::Struct(s) => {
                assert_eq!(
                    s.derives,
                    vec!["Debug".to_string(), "Clone".to_string()],
                    "struct should get Debug, Clone derives"
                );
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn derives_added_to_enums() {
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

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        match &lowered.items.last().unwrap() {
            Item::Enum(e) => {
                assert_eq!(e.derives, vec!["Debug".to_string(), "Clone".to_string()]);
            }
            other => panic!("expected Enum, got {other:?}"),
        }
    }

    #[test]
    fn existing_derives_preserved() {
        let source = SourceFile {
            doc: vec![],
            items: vec![Item::Struct(StructDef {
                name: "Config".to_string(),
                is_pub: true,
                derives: vec!["Serialize".to_string()],
                fields: vec![],
                doc: vec![],
            })],
        };

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        match &lowered.items.last().unwrap() {
            Item::Struct(s) => {
                assert_eq!(
                    s.derives,
                    vec!["Serialize".to_string()],
                    "existing derives should not be overwritten"
                );
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    // -- B2.3: Import analysis --

    #[test]
    fn imports_generated_for_transport_calls() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "resp",
            Expr::call_with_obligation(
                "execute_file_read",
                vec![Expr::var("req")],
                CallObligation::ServiceTransportExecute,
            ),
        )]);

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        let use_items: Vec<&Import> = lowered
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Use(import) => Some(import),
                _ => None,
            })
            .collect();

        assert!(
            use_items
                .iter()
                .any(|i| i.path == vec!["gunbc_exec".to_string()]),
            "should import gunbc_exec, got imports: {:?}",
            use_items
        );
    }

    #[test]
    fn no_imports_for_pure_functions() {
        let source = make_abstract_main(vec![Stmt::let_bind("x", Expr::IntLit(42))]);

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        let use_items: Vec<&Import> = lowered
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Use(import) => Some(import),
                _ => None,
            })
            .collect();

        assert!(
            use_items.is_empty(),
            "pure functions should not generate imports"
        );
    }

    // -- B2.4: String ownership --

    #[test]
    fn format_str_becomes_format_macro() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "msg",
            Expr::FormatStr {
                template: "Hello, {name}!".to_string(),
                args: vec![Expr::var("name")],
            },
        )]);

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        let main_fn = match &lowered.items.last().unwrap() {
            Item::Fn(f) => f,
            other => panic!("expected Fn, got {other:?}"),
        };

        // The FormatStr should be lowered to a MacroCall("format", ...).
        let let_stmt = &main_fn.body[0];
        match let_stmt {
            Stmt::Let { expr, .. } => {
                assert!(
                    matches!(expr, Expr::MacroCall { name, .. } if name == "format"),
                    "FormatStr should lower to format! macro, got {expr:?}"
                );
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // -- B2.5: Transport call rewriting --

    #[test]
    fn transport_calls_rewritten_to_exec_runtime() {
        let source = make_abstract_main(vec![
            Stmt::let_bind(
                "req",
                Expr::call_with_obligation(
                    "prepare_file_read",
                    vec![Expr::var("path")],
                    CallObligation::ServiceTransportPrepare,
                ),
            ),
            Stmt::let_bind(
                "resp",
                Expr::call_with_obligation(
                    "execute_file_read",
                    vec![Expr::var("req")],
                    CallObligation::ServiceTransportExecute,
                ),
            ),
        ]);

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        let main_fn = match &lowered.items.last().unwrap() {
            Item::Fn(f) => f,
            other => panic!("expected Fn, got {other:?}"),
        };

        let body_debug = format!("{:?}", main_fn.body);
        assert!(
            body_debug.contains("FileRequest::read"),
            "prepare_file_read should be rewritten to FileRequest::read, body: {body_debug}"
        );
        assert!(
            body_debug.contains("execute_transport"),
            "execute_file_read should be rewritten to execute_transport, body: {body_debug}"
        );
    }

    #[test]
    fn transport_named_call_without_obligation_is_not_treated_as_runtime() {
        let source = make_abstract_main(vec![Stmt::let_bind(
            "req",
            Expr::call("prepare_file_read", vec![Expr::var("path")]),
        )]);

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        let use_items: Vec<&Import> = lowered
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Use(import) => Some(import),
                _ => None,
            })
            .collect();
        assert!(
            use_items.is_empty(),
            "call names alone should not trigger runtime imports"
        );

        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "main" => Some(f),
                _ => None,
            })
            .expect("should have fn main");
        assert_eq!(main_fn.return_type, None);

        let body_debug = format!("{:?}", main_fn.body);
        assert!(body_debug.contains("prepare_file_read"));
        assert!(!body_debug.contains("FileRequest::read"));
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

        let config = RustConfig {
            use_exec_runtime: false,
            error_type: "Error".to_string(),
        };
        let lowered = lower_to_rust(&source, &config).unwrap();

        let main_fn = match &lowered.items.last().unwrap() {
            Item::Fn(f) => f,
            other => panic!("expected Fn, got {other:?}"),
        };

        let body_debug = format!("{:?}", main_fn.body);
        assert!(
            body_debug.contains("prepare_file_read"),
            "standalone mode should keep abstract function names"
        );
    }

    // -- B2.6: Integration test --

    #[test]
    fn lower_makegen_abstract_ir_to_rust_ir() {
        // Build a realistic makegen AbstractIR SourceFile.
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
                            vec![Expr::var("path")],
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
                    Stmt::Blank,
                    Stmt::comment("step 5: conditional write"),
                    Stmt::Expr(Expr::If {
                        cond: Box::new(Expr::var("fresh").logical_not()),
                        then_body: vec![
                            Stmt::let_bind(
                                "write_req",
                                Expr::call_with_obligation(
                                    "prepare_file_write",
                                    vec![Expr::var("path"), Expr::var("content")],
                                    CallObligation::ServiceTransportPrepare,
                                ),
                            ),
                            Stmt::Expr(Expr::call_with_obligation(
                                "execute_file_write",
                                vec![Expr::var("write_req")],
                                CallObligation::ServiceTransportExecute,
                            )),
                        ],
                        else_body: None,
                    }),
                ],
                doc: vec!["Generated main from EmitPlan.".to_string()],
                attributes: vec![],
            })],
        };

        let config = RustConfig::default();
        let lowered = lower_to_rust(&source, &config).unwrap();

        // Should have use statements + fn main.
        assert!(
            lowered.items.len() >= 2,
            "should have at least 1 use + 1 fn, got {} items",
            lowered.items.len()
        );

        // Check imports.
        let has_exec_import = lowered.items.iter().any(|item| {
            matches!(item, Item::Use(import) if import.path == vec!["gunbc_exec".to_string()])
        });
        assert!(has_exec_import, "should import gunbc_exec");

        // Check main fn.
        let main_fn = lowered
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name == "main" => Some(f),
                _ => None,
            })
            .expect("should have fn main");

        // Return type should be Result.
        assert_eq!(
            main_fn.return_type,
            Some("Result<(), ExecError>".to_string())
        );

        // Body should contain rewritten transport calls.
        let body_debug = format!("{:?}", main_fn.body);
        assert!(body_debug.contains("FileRequest::read"));
        assert!(body_debug.contains("execute_transport"));
        assert!(body_debug.contains("FileRequest::write"));

        // FormatStr should become format! macro.
        assert!(body_debug.contains("format"));

        // Should end with Ok(()).
        assert!(
            matches!(main_fn.body.last(), Some(Stmt::TailExpr(Expr::Call { func, .. }))
                if matches!(func.as_ref(), Expr::Var(name) if name == "Ok")),
            "should end with Ok(())"
        );
    }

    // -- Type mapping --

    #[test]
    fn map_abstract_types_to_rust() {
        assert_eq!(map_to_rust_type("String"), "String");
        assert_eq!(map_to_rust_type("Bool"), "bool");
        assert_eq!(map_to_rust_type("Int"), "i64");
        assert_eq!(map_to_rust_type("Path"), "String");
        assert_eq!(map_to_rust_type("FilesystemHandle"), "PathBuf");
        assert_eq!(map_to_rust_type("ToolRegistry"), "serde_json::Value");
        assert_eq!(map_to_rust_type("List<String>"), "Vec<String>");
        assert_eq!(map_to_rust_type("UnknownType"), "serde_json::Value");
    }
}
