//! DSL FnBody → abstract IR compiler.
//!
//! Translates DSL expressions and statements (`daglang_syntax::ast::Expr` /
//! `Stmt`) into the shared abstract IR (`gunbc_ir::code_ir::Expr` / `Stmt`).
//! Because the output is target-agnostic IR, the generated functions flow
//! through all backends (Rust, Go, C, MIPS) via the existing lowering and
//! rendering pipeline.
//!
//! ## Pipe chain strategy
//!
//! DSL pipe chains (`|>`) are compiled to `For` loops inside `Expr::Block`
//! to ensure cross-target compatibility. This avoids closures (unsupported
//! in C) and method-chain idioms that not all backends can render.

use daglang_syntax::ast;
use gunbc_ir::code_ir;
use std::collections::HashSet;

use crate::type_codegen::to_snake_case;

/// Context for compiling DSL function bodies.
///
/// Carries the set of data table names defined in the module so that
/// identifier references can be mapped to their SCREAMING_SNAKE_CASE
/// static names in the generated output, and struct field optionality
/// information for automatic `Some()` wrapping.
pub struct CompileContext {
    /// Names of `data` definitions visible in this module.
    pub data_names: HashSet<String>,
    /// Map from struct name → set of field names that are `Option<T>`.
    pub optional_fields: std::collections::HashMap<String, HashSet<String>>,
    /// Map from bare variant name → parent enum name (e.g. "ZeroWidth" → "DisplayWidth").
    /// Ambiguous variants (present in multiple enums) are excluded.
    pub variant_to_enum: std::collections::HashMap<String, String>,
    /// Map from struct name → (field name → field type name) for contextual resolution.
    pub struct_field_types:
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// Map from enum name → set of variant names, for field-type-based disambiguation.
    pub enum_variants: std::collections::HashMap<String, HashSet<String>>,
}

impl Default for CompileContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            data_names: HashSet::new(),
            optional_fields: std::collections::HashMap::new(),
            variant_to_enum: std::collections::HashMap::new(),
            struct_field_types: std::collections::HashMap::new(),
            enum_variants: std::collections::HashMap::new(),
        }
    }
}

/// Compile a DSL `FnBody` into a list of abstract IR statements.
pub fn compile_fn_body(body: &ast::FnBody, ctx: &CompileContext) -> Vec<code_ir::Stmt> {
    let len = body.stmts.len();
    body.stmts
        .iter()
        .enumerate()
        .map(|(i, s)| compile_stmt(s, i == len - 1, ctx))
        .collect()
}

/// Counter for generating unique temporary variable names within a
/// compilation unit. Scoped per top-level function compilation.
struct TmpCounter(std::cell::Cell<usize>);

impl TmpCounter {
    fn new() -> Self {
        Self(std::cell::Cell::new(0))
    }
    fn next(&self, prefix: &str) -> String {
        let n = self.0.get();
        self.0.set(n + 1);
        format!("__{prefix}_{n}")
    }
}

thread_local! {
    static TMP: TmpCounter = TmpCounter::new();
}

fn fresh(prefix: &str) -> String {
    TMP.with(|t| t.next(prefix))
}

pub fn reset_tmp_counter() {
    TMP.with(|t| t.0.set(0));
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

fn compile_stmt(stmt: &ast::Stmt, is_last: bool, ctx: &CompileContext) -> code_ir::Stmt {
    match stmt {
        ast::Stmt::Let(name, expr) => code_ir::Stmt::Let {
            name: name.clone(),
            mutable: false,
            expr: compile_expr(expr, ctx),
        },
        ast::Stmt::Assign(name, expr) => code_ir::Stmt::Assign {
            dest: code_ir::Expr::Var(name.clone()),
            value: compile_expr(expr, ctx),
        },
        ast::Stmt::Node(ns) => code_ir::Stmt::Let {
            name: ns.name.clone(),
            mutable: false,
            expr: compile_expr(&ns.expr, ctx),
        },
        ast::Stmt::Expr(expr) => {
            if is_last {
                code_ir::Stmt::TailExpr(compile_expr(expr, ctx))
            } else {
                code_ir::Stmt::Expr(compile_expr(expr, ctx))
            }
        }
        ast::Stmt::Return(fields) => {
            let ir_expr = compile_return_fields(fields, ctx);
            if is_last {
                code_ir::Stmt::TailExpr(ir_expr)
            } else {
                code_ir::Stmt::Return(ir_expr)
            }
        }
    }
}

fn compile_return_fields(fields: &[(String, ast::Expr)], ctx: &CompileContext) -> code_ir::Expr {
    if fields.is_empty() {
        code_ir::Expr::Tuple(vec![])
    } else if fields.len() == 1 && fields[0].0 == "value" {
        compile_expr(&fields[0].1, ctx)
    } else {
        code_ir::Expr::Struct {
            name: String::new(),
            fields: fields
                .iter()
                .map(|(name, expr)| (name.clone(), compile_expr(expr, ctx)))
                .collect(),
            rest: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

fn compile_expr(expr: &ast::Expr, ctx: &CompileContext) -> code_ir::Expr {
    match expr {
        ast::Expr::Literal(lit) => compile_literal(lit),
        ast::Expr::Ident(name) => compile_ident(name, ctx),
        ast::Expr::FieldAccess(receiver, field) => {
            code_ir::Expr::Field(Box::new(compile_expr(receiver, ctx)), field.clone())
        }
        ast::Expr::Call(name, args) => {
            if let Some(intrinsic) = compile_intrinsic_call(name, args, ctx) {
                intrinsic
            } else {
                compile_call(name, args, ctx)
            }
        }
        ast::Expr::BinOp(left, op, right) => {
            if matches!(op, ast::BinOp::Add) && contains_string_literal(expr) {
                compile_string_concat(expr, ctx)
            } else {
                code_ir::Expr::BinOp {
                    left: Box::new(compile_expr(left, ctx)),
                    op: compile_binop(op),
                    right: Box::new(compile_expr(right, ctx)),
                }
            }
        }
        ast::Expr::UnaryOp(op, expr) => code_ir::Expr::UnaryOp {
            op: compile_unaryop(op),
            expr: Box::new(compile_expr(expr, ctx)),
        },
        ast::Expr::Record(name, fields) => {
            let struct_name = name.clone().unwrap_or_default();
            let opt_set = ctx.optional_fields.get(&struct_name);
            let field_types = ctx.struct_field_types.get(&struct_name);
            let ir_fields: Vec<(String, code_ir::Expr)> = fields
                .iter()
                .map(|(n, e)| {
                    let compiled = compile_expr_in_field_context(e, n, field_types, ctx);
                    let is_opt = opt_set.is_some_and(|s| s.contains(n.as_str()));
                    if is_opt && !is_none_expr(&compiled) {
                        (n.clone(), code_ir::Expr::Call {
                            func: Box::new(code_ir::Expr::Var("Some".to_string())),
                            args: vec![compiled],
                            obligation: None,
                        })
                    } else {
                        (n.clone(), compiled)
                    }
                })
                .collect();
            code_ir::Expr::Struct {
                name: struct_name,
                fields: ir_fields,
                rest: None,
            }
        }
        ast::Expr::Match(scrutinee, arms) => compile_match(scrutinee, arms, ctx),
        ast::Expr::If(cond, then_expr, else_expr) => compile_if(cond, then_expr, else_expr, ctx),
        ast::Expr::Lambda(params, body) => code_ir::Expr::Closure {
            args: params.clone(),
            body: Box::new(compile_expr(body, ctx)),
        },
        ast::Expr::List(elements) => {
            // DSL List<T> maps to Rust Vec<T>, so use vec![] not [].
            code_ir::Expr::MacroCall {
                name: "vec".to_string(),
                args: elements.iter().map(|e| compile_expr(e, ctx)).collect(),
            }
        }
        ast::Expr::StringInterp(parts) => compile_string_interp(parts, ctx),
        ast::Expr::For(binding, iter_expr, _passthrough, body) => {
            let result_var = fresh("for_result");
            let iter = make_owned_iter(compile_expr(iter_expr, ctx));
            let push_expr = match body {
                ast::ForBody::Expr(expr) => compile_expr(expr, ctx),
                ast::ForBody::Block(stmts) => code_ir::Expr::Block(
                    stmts
                        .iter()
                        .enumerate()
                        .map(|(index, stmt)| compile_stmt(stmt, index + 1 == stmts.len(), ctx))
                        .collect(),
                ),
            };
            code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result_var, code_ir::Expr::MacroCall {
                    name: "vec".to_string(),
                    args: vec![],
                }),
                code_ir::Stmt::For {
                    binding: binding.clone(),
                    iter,
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result_var.clone())),
                        method: "push".to_string(),
                        args: vec![push_expr],
                    })],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result_var)),
            ])
        }
        ast::Expr::Return(fields) => compile_return_fields(fields, ctx),
        ast::Expr::Guarded(inner, _) | ast::Expr::After(inner, _) => compile_expr(inner, ctx),
        ast::Expr::Block(stmts) => code_ir::Expr::Block(
            stmts
                .iter()
                .enumerate()
                .map(|(index, stmt)| compile_stmt(stmt, index + 1 == stmts.len(), ctx))
                .collect(),
        ),
        ast::Expr::ServiceCall(_, _) | ast::Expr::Map(_) => {
            code_ir::Expr::RawCode(
                "compile_error!(\"unsupported DSL construct: ServiceCall/Map not yet supported in fn codegen\");"
                    .to_string(),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

fn compile_literal(lit: &ast::Literal) -> code_ir::Expr {
    match lit {
        ast::Literal::Int(n) => code_ir::Expr::IntLit(*n),
        ast::Literal::Float(f) => code_ir::Expr::RawCode(format!("{f:?}_f64")),
        ast::Literal::String(s) => code_ir::Expr::Str(s.clone()),
        ast::Literal::Bool(b) => code_ir::Expr::BoolLit(*b),
        ast::Literal::None => code_ir::Expr::Var("None".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Identifiers — bare names resolve to enum-variant paths
// ---------------------------------------------------------------------------

fn compile_ident(name: &str, ctx: &CompileContext) -> code_ir::Expr {
    if name == "null" {
        return code_ir::Expr::Var("None".to_string());
    }
    if ctx.data_names.contains(name) {
        code_ir::Expr::MethodCall {
            receiver: Box::new(code_ir::Expr::Var(to_screaming_snake(name))),
            method: "clone".to_string(),
            args: vec![],
        }
    } else if let Some(enum_name) = ctx.variant_to_enum.get(name) {
        code_ir::Expr::Path(vec![enum_name.clone(), name.to_string()])
    } else {
        code_ir::Expr::Var(name.to_string())
    }
}

fn to_screaming_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}

// ---------------------------------------------------------------------------
// Function calls
// ---------------------------------------------------------------------------

/// Intercept collection intrinsic calls that were formerly pipe methods.
///
/// These are compiled to For loops for cross-target compatibility — the
/// code_ir `For` / `Block` / `If` constructs are understood by all backend
/// renderers (Rust, Go, C), unlike language-specific iterator chains.
///
/// Returns `None` for non-intrinsic calls.
fn compile_intrinsic_call(
    name: &str,
    args: &[(Option<String>, ast::Expr)],
    ctx: &CompileContext,
) -> Option<code_ir::Expr> {
    // First arg is always the collection/receiver
    let collection = || compile_expr(&args[0].1, ctx);

    match name {
        "map" if args.len() == 2 => {
            Some(compile_map_intrinsic(&collection(), args.get(1).map(|(_, e)| e), ctx))
        }
        "filter" if args.len() == 2 => {
            Some(compile_filter_intrinsic(&collection(), args.get(1).map(|(_, e)| e), ctx))
        }
        "fold" if args.len() >= 2 => {
            let init = args.iter().find(|(n, _)| n.as_deref() == Some("init"))
                .or_else(|| args.get(1)).map(|(_, e)| e);
            let func = args.iter().find(|(n, _)| n.as_deref() == Some("f"))
                .or_else(|| args.get(2)).map(|(_, e)| e);
            Some(compile_fold_intrinsic(&collection(), init, func, ctx))
        }
        "any" if args.len() == 2 => {
            let pred = args.iter().find(|(n, _)| n.as_deref() == Some("predicate"))
                .or_else(|| args.get(1)).map(|(_, e)| e);
            Some(compile_any_intrinsic(&collection(), pred, ctx))
        }
        "contains" if args.len() == 2 => {
            let target = compile_expr(&args[1].1, ctx);
            let result = fresh("contains");
            let elem = fresh("elem");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result, code_ir::Expr::BoolLit(false)),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: make_owned_iter(collection()),
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                        cond: Box::new(code_ir::Expr::BinOp {
                            left: Box::new(code_ir::Expr::Var(elem)),
                            op: "==".to_string(),
                            right: Box::new(target),
                        }),
                        then_body: vec![
                            code_ir::Stmt::Assign {
                                dest: code_ir::Expr::Var(result.clone()),
                                value: code_ir::Expr::BoolLit(true),
                            },
                            code_ir::Stmt::Expr(code_ir::Expr::RawCode("break".to_string())),
                        ],
                        else_body: None,
                    })],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "sum" if args.len() == 1 => {
            let result = fresh("sum");
            let elem = fresh("elem");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result, code_ir::Expr::IntLit(0)),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: make_owned_iter(collection()),
                    body: vec![code_ir::Stmt::Assign {
                        dest: code_ir::Expr::Var(result.clone()),
                        value: code_ir::Expr::BinOp {
                            left: Box::new(code_ir::Expr::Var(result.clone())),
                            op: "+".to_string(),
                            right: Box::new(code_ir::Expr::Var(elem)),
                        },
                    }],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "join" if args.len() == 2 => {
            let sep = compile_expr(&args[1].1, ctx);
            let result = fresh("joined");
            let elem = fresh("elem");
            let first = fresh("first");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result, code_ir::Expr::Str(String::new())),
                code_ir::Stmt::let_mut(&first, code_ir::Expr::BoolLit(true)),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: make_owned_iter(collection()),
                    body: vec![
                        code_ir::Stmt::Expr(code_ir::Expr::If {
                            cond: Box::new(code_ir::Expr::UnaryOp {
                                op: "!".to_string(),
                                expr: Box::new(code_ir::Expr::Var(first.clone())),
                            }),
                            then_body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                                receiver: Box::new(code_ir::Expr::Var(result.clone())),
                                method: "push_str".to_string(),
                                args: vec![code_ir::Expr::MethodCall {
                                    receiver: Box::new(sep.clone()),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }],
                            })],
                            else_body: None,
                        }),
                        code_ir::Stmt::Assign {
                            dest: code_ir::Expr::Var(first.clone()),
                            value: code_ir::Expr::BoolLit(false),
                        },
                        code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                            receiver: Box::new(code_ir::Expr::Var(result.clone())),
                            method: "push_str".to_string(),
                            args: vec![code_ir::Expr::MethodCall {
                                receiver: Box::new(code_ir::Expr::Var(elem)),
                                method: "as_str".to_string(),
                                args: vec![],
                            }],
                        }),
                    ],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "last" if args.len() == 1 => {
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::MethodCall {
                    receiver: Box::new(collection()),
                    method: "last".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            })
        }
        "split" if args.len() == 2 => {
            let delim = compile_expr(&args[1].1, ctx);
            let result = fresh("split_parts");
            let elem = fresh("part");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result, code_ir::Expr::Array(vec![])),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::MethodCall {
                            receiver: Box::new(collection()),
                            method: "split".to_string(),
                            args: vec![code_ir::Expr::MethodCall {
                                receiver: Box::new(delim),
                                method: "as_str".to_string(),
                                args: vec![],
                            }],
                        }),
                        method: "map".to_string(),
                        args: vec![code_ir::Expr::Path(vec![
                            "String".to_string(),
                            "from".to_string(),
                        ])],
                    },
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result.clone())),
                        method: "push".to_string(),
                        args: vec![code_ir::Expr::Var(elem)],
                    })],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "chars" if args.len() == 1 => {
            let result = fresh("chars");
            let ch = fresh("ch");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result, code_ir::Expr::Array(vec![])),
                code_ir::Stmt::For {
                    binding: ch.clone(),
                    iter: code_ir::Expr::MethodCall {
                        receiver: Box::new(collection()),
                        method: "chars".to_string(),
                        args: vec![],
                    },
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result.clone())),
                        method: "push".to_string(),
                        args: vec![code_ir::Expr::MethodCall {
                            receiver: Box::new(code_ir::Expr::Var(ch)),
                            method: "to_string".to_string(),
                            args: vec![],
                        }],
                    })],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Intrinsic helpers — For-loop unrollings for cross-target compatibility
// ---------------------------------------------------------------------------

fn compile_map_intrinsic(
    collection: &code_ir::Expr,
    mapper: Option<&ast::Expr>,
    ctx: &CompileContext,
) -> code_ir::Expr {
    let result = fresh("mapped");
    let elem = fresh("elem");
    let mapped_value = match mapper {
        Some(ast::Expr::Lambda(params, body)) => {
            let compiled = compile_expr(body, ctx);
            params.first()
                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                .unwrap_or(compiled)
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx)),
            args: vec![code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::Var(elem.clone()),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(&result, code_ir::Expr::MacroCall { name: "vec".to_string(), args: vec![] }),
        code_ir::Stmt::For {
            binding: elem,
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Var(result.clone())),
                method: "push".to_string(),
                args: vec![mapped_value],
            })],
        },
        code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
    ])
}

fn compile_filter_intrinsic(
    collection: &code_ir::Expr,
    predicate: Option<&ast::Expr>,
    ctx: &CompileContext,
) -> code_ir::Expr {
    let result = fresh("filtered");
    let elem = fresh("elem");
    let cond = match predicate {
        Some(ast::Expr::Lambda(params, body)) => {
            let compiled = compile_expr(body, ctx);
            params.first()
                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                .unwrap_or(compiled)
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx)),
            args: vec![code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::BoolLit(true),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(&result, code_ir::Expr::MacroCall { name: "vec".to_string(), args: vec![] }),
        code_ir::Stmt::For {
            binding: elem.clone(),
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                cond: Box::new(cond),
                then_body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                    receiver: Box::new(code_ir::Expr::Var(result.clone())),
                    method: "push".to_string(),
                    args: vec![code_ir::Expr::Var(elem)],
                })],
                else_body: None,
            })],
        },
        code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
    ])
}

fn compile_fold_intrinsic(
    collection: &code_ir::Expr,
    init: Option<&ast::Expr>,
    func: Option<&ast::Expr>,
    ctx: &CompileContext,
) -> code_ir::Expr {
    let acc = fresh("acc");
    let elem = fresh("elem");
    let init_expr = init.map(|e| compile_expr(e, ctx)).unwrap_or(code_ir::Expr::IntLit(0));
    let body_expr = match func {
        Some(ast::Expr::Lambda(params, body)) => {
            let mut compiled = compile_expr(body, ctx);
            if let Some(p) = params.first() {
                compiled = substitute_var(&compiled, p, &code_ir::Expr::Var(acc.clone()));
            }
            if let Some(p) = params.get(1) {
                compiled = substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone()));
            }
            compiled
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx)),
            args: vec![code_ir::Expr::Var(acc.clone()), code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::Var(acc.clone()),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(&acc, init_expr),
        code_ir::Stmt::For {
            binding: elem,
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Assign {
                dest: code_ir::Expr::Var(acc.clone()),
                value: body_expr,
            }],
        },
        code_ir::Stmt::TailExpr(code_ir::Expr::Var(acc)),
    ])
}

fn compile_any_intrinsic(
    collection: &code_ir::Expr,
    predicate: Option<&ast::Expr>,
    ctx: &CompileContext,
) -> code_ir::Expr {
    let result = fresh("any");
    let elem = fresh("elem");
    let cond = match predicate {
        Some(ast::Expr::Lambda(params, body)) => {
            let compiled = compile_expr(body, ctx);
            params.first()
                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                .unwrap_or(compiled)
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx)),
            args: vec![code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::BoolLit(false),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(&result, code_ir::Expr::BoolLit(false)),
        code_ir::Stmt::For {
            binding: elem,
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                cond: Box::new(cond),
                then_body: vec![
                    code_ir::Stmt::Assign {
                        dest: code_ir::Expr::Var(result.clone()),
                        value: code_ir::Expr::BoolLit(true),
                    },
                    code_ir::Stmt::Expr(code_ir::Expr::RawCode("break".to_string())),
                ],
                else_body: None,
            })],
        },
        code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
    ])
}

/// Substitute a variable name in a code_ir expression tree.
fn substitute_var(expr: &code_ir::Expr, from: &str, to: &code_ir::Expr) -> code_ir::Expr {
    match expr {
        code_ir::Expr::Var(name) if name == from => to.clone(),
        code_ir::Expr::Var(_)
        | code_ir::Expr::Str(_)
        | code_ir::Expr::IntLit(_)
        | code_ir::Expr::BoolLit(_)
        | code_ir::Expr::RawCode(_)
        | code_ir::Expr::Value(_)
        | code_ir::Expr::Path(_) => expr.clone(),
        code_ir::Expr::Field(receiver, field) => {
            code_ir::Expr::Field(Box::new(substitute_var(receiver, from, to)), field.clone())
        }
        code_ir::Expr::Call { func, args, obligation } => code_ir::Expr::Call {
            func: Box::new(substitute_var(func, from, to)),
            args: args.iter().map(|a| substitute_var(a, from, to)).collect(),
            obligation: *obligation,
        },
        code_ir::Expr::MethodCall { receiver, method, args } => code_ir::Expr::MethodCall {
            receiver: Box::new(substitute_var(receiver, from, to)),
            method: method.clone(),
            args: args.iter().map(|a| substitute_var(a, from, to)).collect(),
        },
        code_ir::Expr::BinOp { left, op, right } => code_ir::Expr::BinOp {
            left: Box::new(substitute_var(left, from, to)),
            op: op.clone(),
            right: Box::new(substitute_var(right, from, to)),
        },
        code_ir::Expr::UnaryOp { op, expr: inner } => code_ir::Expr::UnaryOp {
            op: op.clone(),
            expr: Box::new(substitute_var(inner, from, to)),
        },
        code_ir::Expr::If { cond, then_body, else_body } => code_ir::Expr::If {
            cond: Box::new(substitute_var(cond, from, to)),
            then_body: then_body.iter().map(|s| substitute_var_in_stmt(s, from, to)).collect(),
            else_body: else_body.as_ref().map(|stmts| stmts.iter().map(|s| substitute_var_in_stmt(s, from, to)).collect()),
        },
        code_ir::Expr::Block(stmts) => {
            code_ir::Expr::Block(stmts.iter().map(|s| substitute_var_in_stmt(s, from, to)).collect())
        }
        code_ir::Expr::Ref(inner) => code_ir::Expr::Ref(Box::new(substitute_var(inner, from, to))),
        // For other variants, return as-is (they don't contain variable references)
        other => other.clone(),
    }
}

fn substitute_var_in_stmt(stmt: &code_ir::Stmt, from: &str, to: &code_ir::Expr) -> code_ir::Stmt {
    match stmt {
        code_ir::Stmt::Let { name, expr, mutable } => code_ir::Stmt::Let {
            name: name.clone(),
            expr: substitute_var(expr, from, to),
            mutable: *mutable,
        },
        code_ir::Stmt::Assign { dest, value } => code_ir::Stmt::Assign {
            dest: substitute_var(dest, from, to),
            value: substitute_var(value, from, to),
        },
        code_ir::Stmt::Expr(expr) => code_ir::Stmt::Expr(substitute_var(expr, from, to)),
        code_ir::Stmt::TailExpr(expr) => code_ir::Stmt::TailExpr(substitute_var(expr, from, to)),
        code_ir::Stmt::For { binding, iter, body } => code_ir::Stmt::For {
            binding: binding.clone(),
            iter: substitute_var(iter, from, to),
            body: body.iter().map(|s| substitute_var_in_stmt(s, from, to)).collect(),
        },
        other => other.clone(),
    }
}

fn compile_call(
    name: &str,
    args: &[(Option<String>, ast::Expr)],
    ctx: &CompileContext,
) -> code_ir::Expr {
    let ir_args: Vec<code_ir::Expr> = args.iter().map(|(_, e)| compile_expr(e, ctx)).collect();
    let rust_name = to_snake_case(name);

    code_ir::Expr::Call {
        func: Box::new(code_ir::Expr::Var(rust_name)),
        args: ir_args,
        obligation: None,
    }
}

// ---------------------------------------------------------------------------
// Binary / unary operators
// ---------------------------------------------------------------------------

fn compile_binop(op: &ast::BinOp) -> String {
    match op {
        ast::BinOp::Add => "+".to_string(),
        ast::BinOp::Sub => "-".to_string(),
        ast::BinOp::Mul => "*".to_string(),
        ast::BinOp::Div => "/".to_string(),
        ast::BinOp::Mod => "%".to_string(),
        ast::BinOp::Eq => "==".to_string(),
        ast::BinOp::Ne => "!=".to_string(),
        ast::BinOp::Lt => "<".to_string(),
        ast::BinOp::Gt => ">".to_string(),
        ast::BinOp::Le => "<=".to_string(),
        ast::BinOp::Ge => ">=".to_string(),
        ast::BinOp::And => "&&".to_string(),
        ast::BinOp::Or => "||".to_string(),
        ast::BinOp::NullCoalesce => "??".to_string(),
    }
}

fn compile_unaryop(op: &ast::UnaryOp) -> String {
    match op {
        ast::UnaryOp::Not => "!".to_string(),
        ast::UnaryOp::Neg => "-".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Match
// ---------------------------------------------------------------------------

fn compile_match(
    scrutinee: &ast::Expr,
    arms: &[ast::MatchArm],
    ctx: &CompileContext,
) -> code_ir::Expr {
    let has_none_arm = arms.iter().any(|a| is_null_pattern(&a.pattern));
    code_ir::Expr::Match {
        expr: Box::new(compile_expr(scrutinee, ctx)),
        arms: arms
            .iter()
            .map(|a| compile_match_arm(a, has_none_arm, ctx))
            .collect(),
    }
}

fn compile_match_arm(
    arm: &ast::MatchArm,
    option_context: bool,
    ctx: &CompileContext,
) -> code_ir::MatchArm {
    let mut pattern = compile_pattern(&arm.pattern, ctx);
    if option_context
        && !is_null_pattern(&arm.pattern)
        && !matches!(arm.pattern, ast::Pattern::Wildcard)
    {
        pattern = format!("Some({pattern})");
    }
    code_ir::MatchArm {
        pattern,
        body: vec![code_ir::Stmt::TailExpr(compile_expr(&arm.body, ctx))],
    }
}

fn is_null_pattern(pat: &ast::Pattern) -> bool {
    matches!(pat, ast::Pattern::Ident(name) if name == "null")
        || matches!(pat, ast::Pattern::Literal(ast::Literal::None))
}

fn compile_pattern(pat: &ast::Pattern, ctx: &CompileContext) -> String {
    match pat {
        ast::Pattern::Ident(name) => {
            if name == "null" {
                "None".to_string()
            } else if let Some(enum_name) = ctx.variant_to_enum.get(name.as_str()) {
                format!("{enum_name}::{name}")
            } else {
                name.clone()
            }
        }
        ast::Pattern::Variant(name, fields) => {
            let qualified = ctx
                .variant_to_enum
                .get(name.as_str())
                .map(|e| format!("{e}::{name}"))
                .unwrap_or_else(|| name.clone());
            if fields.is_empty() {
                qualified
            } else {
                let field_pats: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| format!("{}: {}", n, compile_pattern(p, ctx)))
                    .collect();
                format!("{} {{ {} }}", qualified, field_pats.join(", "))
            }
        }
        ast::Pattern::Wildcard => "_".to_string(),
        ast::Pattern::Literal(lit) => match lit {
            ast::Literal::Int(n) => n.to_string(),
            ast::Literal::Float(f) => format!("{f:?}"),
            ast::Literal::String(s) => format!("\"{s}\""),
            ast::Literal::Bool(b) => b.to_string(),
            ast::Literal::None => "None".to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// If / else
// ---------------------------------------------------------------------------

fn compile_if(
    cond: &ast::Expr,
    then_expr: &ast::Expr,
    else_expr: &Option<Box<ast::Expr>>,
    ctx: &CompileContext,
) -> code_ir::Expr {
    let then_stmts = expr_to_stmts(then_expr, ctx);
    let else_stmts = else_expr.as_ref().map(|e| expr_to_stmts(e, ctx));

    code_ir::Expr::If {
        cond: Box::new(compile_expr(cond, ctx)),
        then_body: then_stmts,
        else_body: else_stmts,
    }
}

fn expr_to_stmts(expr: &ast::Expr, ctx: &CompileContext) -> Vec<code_ir::Stmt> {
    match expr {
        ast::Expr::Return(fields) => {
            vec![code_ir::Stmt::Return(compile_return_fields(fields, ctx))]
        }
        _ => {
            vec![code_ir::Stmt::TailExpr(compile_expr(expr, ctx))]
        }
    }
}

// ---------------------------------------------------------------------------
// String interpolation
// ---------------------------------------------------------------------------

fn compile_string_interp(parts: &[ast::StringPart], ctx: &CompileContext) -> code_ir::Expr {
    let mut template = String::new();
    let mut args = Vec::new();
    for part in parts {
        match part {
            ast::StringPart::Literal(s) => template.push_str(s),
            ast::StringPart::Expr(e) => {
                template.push_str("{}");
                args.push(compile_expr(e, ctx));
            }
        }
    }
    code_ir::Expr::FormatStr { template, args }
}

// ---------------------------------------------------------------------------
// Helpers — static iteration, string concat, Option wrapping
// ---------------------------------------------------------------------------

/// When a compiled collection expression is a `.clone()` call on a static
/// data table, replace `STATIC.clone()` with `STATIC.iter().cloned()` so
/// the for-loop iterates by value.  For non-static collections, returns
/// the expression unchanged.
fn make_owned_iter(collection: code_ir::Expr) -> code_ir::Expr {
    match &collection {
        code_ir::Expr::MethodCall {
            receiver,
            method,
            args,
        } if method == "clone" && args.is_empty() => code_ir::Expr::MethodCall {
            receiver: Box::new(code_ir::Expr::MethodCall {
                receiver: receiver.clone(),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        },
        _ => collection,
    }
}

fn is_none_expr(expr: &code_ir::Expr) -> bool {
    matches!(expr, code_ir::Expr::Var(name) if name == "None")
}

/// Compile a field value expression with type-aware variant resolution.
///
/// When the field's declared type is an enum that contains the bare identifier
/// as a variant, use `EnumType::Variant` instead of the global `variant_to_enum`
/// mapping (which may be wrong for ambiguous variants like `Info`).
fn compile_expr_in_field_context(
    expr: &ast::Expr,
    field_name: &str,
    field_types: Option<&std::collections::HashMap<String, String>>,
    ctx: &CompileContext,
) -> code_ir::Expr {
    if let ast::Expr::Ident(name) = expr {
        if name == "null" {
            return code_ir::Expr::Var("None".to_string());
        }
        if let Some(ft_map) = field_types {
            if let Some(type_name) = ft_map.get(field_name) {
                if let Some(variants) = ctx.enum_variants.get(type_name) {
                    if variants.contains(name.as_str()) {
                        return code_ir::Expr::Path(vec![type_name.clone(), name.to_string()]);
                    }
                }
            }
        }
    }
    compile_expr(expr, ctx)
}

/// Check if compiled IR contains empty anonymous records in match arms,
/// which indicates the DSL parser failed to capture complex block bodies.
pub fn body_has_empty_construct(stmts: &[code_ir::Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        code_ir::Stmt::TailExpr(e) | code_ir::Stmt::Expr(e) => expr_has_empty(e),
        code_ir::Stmt::Let { expr, .. } => expr_has_empty(expr),
        _ => false,
    })
}

fn expr_has_empty(e: &code_ir::Expr) -> bool {
    match e {
        code_ir::Expr::Struct { name, fields, .. } if name.is_empty() && fields.is_empty() => true,
        code_ir::Expr::Match { arms, .. } => arms.iter().any(|a| body_has_empty_construct(&a.body)),
        code_ir::Expr::If {
            then_body,
            else_body,
            ..
        } => {
            body_has_empty_construct(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|b| body_has_empty_construct(b))
        }
        code_ir::Expr::Block(stmts) => body_has_empty_construct(stmts),
        _ => false,
    }
}

/// Check if any leaf of a `+` chain is a string literal.
fn contains_string_literal(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Literal(ast::Literal::String(_)) => true,
        ast::Expr::BinOp(left, ast::BinOp::Add, right) => {
            contains_string_literal(left) || contains_string_literal(right)
        }
        _ => false,
    }
}

enum ConcatPart {
    Lit(String),
    Dyn(code_ir::Expr),
}

/// Flatten a chain of `a + " " + b + " "` into FormatStr parts.
fn flatten_concat_parts(expr: &ast::Expr, parts: &mut Vec<ConcatPart>, ctx: &CompileContext) {
    match expr {
        ast::Expr::BinOp(left, ast::BinOp::Add, right) if contains_string_literal(expr) => {
            flatten_concat_parts(left, parts, ctx);
            flatten_concat_parts(right, parts, ctx);
        }
        ast::Expr::Literal(ast::Literal::String(s)) => {
            parts.push(ConcatPart::Lit(s.clone()));
        }
        other => {
            parts.push(ConcatPart::Dyn(compile_expr(other, ctx)));
        }
    }
}

fn compile_string_concat(expr: &ast::Expr, ctx: &CompileContext) -> code_ir::Expr {
    let mut parts = Vec::new();
    flatten_concat_parts(expr, &mut parts, ctx);

    let mut template = String::new();
    let mut args = Vec::new();
    for part in parts {
        match part {
            ConcatPart::Lit(s) => template.push_str(&s),
            ConcatPart::Dyn(e) => {
                template.push_str("{}");
                args.push(e);
            }
        }
    }
    code_ir::Expr::FormatStr { template, args }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_syntax::ast::{BinOp, Expr, FnBody, Literal, MatchArm, Pattern, Stmt};

    fn empty_ctx() -> CompileContext {
        CompileContext::new()
    }

    fn ctx_with_data(names: &[&str]) -> CompileContext {
        let mut ctx = CompileContext::new();
        for n in names {
            ctx.data_names.insert(n.to_string());
        }
        ctx
    }

    #[test]
    fn compile_literal_int() {
        reset_tmp_counter();
        let ir = compile_expr(&Expr::Literal(Literal::Int(42)), &empty_ctx());
        assert!(matches!(ir, code_ir::Expr::IntLit(42)));
    }

    #[test]
    fn compile_literal_bool() {
        reset_tmp_counter();
        let ir = compile_expr(&Expr::Literal(Literal::Bool(true)), &empty_ctx());
        assert!(matches!(ir, code_ir::Expr::BoolLit(true)));
    }

    #[test]
    fn compile_literal_string() {
        reset_tmp_counter();
        let ir = compile_expr(
            &Expr::Literal(Literal::String("hello".into())),
            &empty_ctx(),
        );
        assert!(matches!(ir, code_ir::Expr::Str(s) if s == "hello"));
    }

    #[test]
    fn compile_field_access() {
        reset_tmp_counter();
        let expr = Expr::FieldAccess(Box::new(Expr::Ident("block".into())), "start".into());
        let ir = compile_expr(&expr, &empty_ctx());
        match ir {
            code_ir::Expr::Field(receiver, field) => {
                assert!(matches!(*receiver, code_ir::Expr::Var(ref n) if n == "block"));
                assert_eq!(field, "start");
            }
            other => panic!("expected Field, got: {other:?}"),
        }
    }

    #[test]
    fn compile_binop() {
        reset_tmp_counter();
        let expr = Expr::BinOp(
            Box::new(Expr::Ident("a".into())),
            BinOp::Ge,
            Box::new(Expr::Literal(Literal::Int(10))),
        );
        let ir = compile_expr(&expr, &empty_ctx());
        match ir {
            code_ir::Expr::BinOp { op, .. } => assert_eq!(op, ">="),
            other => panic!("expected BinOp, got: {other:?}"),
        }
    }

    #[test]
    fn compile_match_expression() {
        reset_tmp_counter();
        let expr = Expr::Match(
            Box::new(Expr::Ident("w".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Ident("ZeroWidth".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(0)),
                },
                MatchArm {
                    pattern: Pattern::Ident("Narrow".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(1)),
                },
            ],
        );
        let ir = compile_expr(&expr, &empty_ctx());
        match ir {
            code_ir::Expr::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].pattern, "ZeroWidth");
                assert_eq!(arms[1].pattern, "Narrow");
            }
            other => panic!("expected Match, got: {other:?}"),
        }
    }

    #[test]
    fn compile_if_else() {
        reset_tmp_counter();
        let expr = Expr::If(
            Box::new(Expr::Ident("flag".into())),
            Box::new(Expr::Literal(Literal::Int(1))),
            Some(Box::new(Expr::Literal(Literal::Int(0)))),
        );
        let ir = compile_expr(&expr, &empty_ctx());
        match ir {
            code_ir::Expr::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_some());
            }
            other => panic!("expected If, got: {other:?}"),
        }
    }

    #[test]
    fn compile_fn_body_let_and_return() {
        reset_tmp_counter();
        let body = FnBody {
            stmts: vec![
                Stmt::Let("x".into(), Expr::Literal(Literal::Int(1))),
                Stmt::Expr(Expr::Ident("x".into())),
            ],
        };
        let ir = compile_fn_body(&body, &empty_ctx());
        assert_eq!(ir.len(), 2);
        assert!(matches!(ir[0], code_ir::Stmt::Let { .. }));
        assert!(matches!(ir[1], code_ir::Stmt::TailExpr(_)));
    }

    #[test]
    fn compile_record_construction() {
        reset_tmp_counter();
        let expr = Expr::Record(
            Some("Point".into()),
            vec![
                ("x".into(), Expr::Literal(Literal::Int(1))),
                ("y".into(), Expr::Literal(Literal::Int(2))),
            ],
        );
        let ir = compile_expr(&expr, &empty_ctx());
        match ir {
            code_ir::Expr::Struct { name, fields, .. } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
            }
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn compile_data_table_ident_uses_screaming_snake_with_clone() {
        reset_tmp_counter();
        let ctx = ctx_with_data(&["zero_width_blocks"]);
        let ir = compile_expr(&Expr::Ident("zero_width_blocks".into()), &ctx);
        match &ir {
            code_ir::Expr::MethodCall {
                receiver, method, ..
            } => {
                assert_eq!(method, "clone");
                assert!(
                    matches!(receiver.as_ref(), code_ir::Expr::Var(ref n) if n == "ZERO_WIDTH_BLOCKS")
                );
            }
            other => panic!("expected MethodCall(clone), got: {other:?}"),
        }
    }

    #[test]
    fn compile_null_ident_becomes_none() {
        reset_tmp_counter();
        let ir = compile_expr(&Expr::Ident("null".into()), &empty_ctx());
        assert!(matches!(ir, code_ir::Expr::Var(ref n) if n == "None"));
    }

    #[test]
    fn option_match_wraps_non_null_patterns_in_some() {
        reset_tmp_counter();
        let expr = Expr::Match(
            Box::new(Expr::Ident("color".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Ident("null".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(0)),
                },
                MatchArm {
                    pattern: Pattern::Ident("c".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(1)),
                },
            ],
        );
        let ir = compile_expr(&expr, &empty_ctx());
        match ir {
            code_ir::Expr::Match { arms, .. } => {
                assert_eq!(arms[0].pattern, "None");
                assert_eq!(arms[1].pattern, "Some(c)");
            }
            other => panic!("expected Match, got: {other:?}"),
        }
    }

    #[test]
    fn field_context_resolves_ambiguous_variant() {
        reset_tmp_counter();
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "BoxConfig".to_string(),
            [("color".to_string(), "SemanticColor".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "SemanticColor".to_string(),
            ["Info".to_string(), "Error".to_string()]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "SymbolId".to_string(),
            ["Info".to_string(), "Error".to_string()]
                .into_iter()
                .collect(),
        );
        let expr = Expr::Record(
            Some("BoxConfig".into()),
            vec![("color".into(), Expr::Ident("Info".into()))],
        );
        let ir = compile_expr(&expr, &ctx);
        match ir {
            code_ir::Expr::Struct { fields, .. } => {
                assert!(
                    matches!(&fields[0].1, code_ir::Expr::Path(parts) if parts == &["SemanticColor", "Info"]),
                    "expected SemanticColor::Info, got: {:?}",
                    fields[0].1
                );
            }
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn unsupported_service_call_map_produces_compile_error() {
        reset_tmp_counter();
        let body = FnBody {
            stmts: vec![Stmt::Expr(Expr::ServiceCall(vec!["svc".into()], vec![]))],
        };
        let ir = compile_fn_body(&body, &empty_ctx());
        match &ir[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(s))
            | code_ir::Stmt::Expr(code_ir::Expr::RawCode(s)) => {
                assert!(
                    s.contains("compile_error!"),
                    "expected compile_error! marker, got: {s}"
                );
                assert!(
                    !s.contains("/* unsupported"),
                    "should not produce silent comment"
                );
            }
            other => panic!("expected RawCode in stmt, got: {other:?}"),
        }

        reset_tmp_counter();
        let body_map = FnBody {
            stmts: vec![Stmt::Expr(Expr::Map(vec![]))],
        };
        let ir_map = compile_fn_body(&body_map, &empty_ctx());
        match &ir_map[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(s))
            | code_ir::Stmt::Expr(code_ir::Expr::RawCode(s)) => {
                assert!(
                    s.contains("compile_error!"),
                    "Map: expected compile_error! marker, got: {s}"
                );
            }
            other => panic!("Map: expected RawCode, got: {other:?}"),
        }
    }

    #[test]
    fn empty_construct_detected_in_match_arms() {
        let stmts = vec![code_ir::Stmt::TailExpr(code_ir::Expr::Match {
            expr: Box::new(code_ir::Expr::Var("x".into())),
            arms: vec![code_ir::MatchArm {
                pattern: "A".into(),
                body: vec![code_ir::Stmt::TailExpr(code_ir::Expr::Struct {
                    name: String::new(),
                    fields: vec![],
                    rest: None,
                })],
            }],
        })];
        assert!(body_has_empty_construct(&stmts));
    }
}
