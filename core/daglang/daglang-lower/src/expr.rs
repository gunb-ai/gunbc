//! Lowered expression IR — the compiler's representation of fn body computation.
//!
//! Translates `ast::FnBody` → `LoweredFnBody` during lowering. Each pipeline
//! stage produces its own representation: `.dag → parse (AST) → typecheck →
//! lower (LoweredExpr) → eval`. Downstream consumers never see parser types.

use std::collections::HashSet;

use daglang_syntax::ast;
use serde::{Deserialize, Serialize};

// ── IR types ────────────────────────────────────────────────────────────────

/// A lowered function body — the unit of computation for `fn` items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredFnBody {
    pub stmts: Vec<LoweredStmt>,
}

/// Typed reference to an expression leaf source used by lowerer wiring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeafRef {
    Param {
        name: String,
        field: Option<String>,
        ty: String,
    },
    Callable {
        endpoint: String,
        port: String,
    },
    Service {
        endpoint: String,
        port: String,
    },
}

/// A lowered statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredStmt {
    /// `let name = expr`
    Let(String, LoweredExpr),
    /// Expression statement (side effects or trailing return)
    Expr(LoweredExpr),
    /// `return { field: expr, ... }`
    Return(Vec<(String, LoweredExpr)>),
}

/// A lowered expression — fully independent of parser AST types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredExpr {
    /// Literal value
    Literal(LoweredLiteral),
    /// Variable reference
    Ident(String),
    /// Field access: `expr.field`
    FieldAccess {
        expr: Box<LoweredExpr>,
        field: String,
    },
    /// String interpolation: `"hello {name}"`
    StringInterp(Vec<LoweredStringPart>),
    /// Binary operation: `a + b`, `a == b`
    BinOp {
        left: Box<LoweredExpr>,
        op: LoweredBinOp,
        right: Box<LoweredExpr>,
    },
    /// Unary operation: `!x`, `-x`
    UnaryOp {
        op: LoweredUnaryOp,
        expr: Box<LoweredExpr>,
    },
    /// Conditional: `if cond { then } else { else_ }`
    IfElse {
        cond: Box<LoweredExpr>,
        then_: Box<LoweredExpr>,
        else_: Option<Box<LoweredExpr>>,
    },
    /// Pattern match
    Match {
        expr: Box<LoweredExpr>,
        arms: Vec<LoweredMatchArm>,
    },
    /// Function call: `f(a: x, b: y)` — named args preserved
    Call {
        name: String,
        args: Vec<(Option<String>, LoweredExpr)>,
    },
    /// Pipe: `expr |> method(args)`
    Pipe {
        receiver: Box<LoweredExpr>,
        call: Box<LoweredExpr>,
    },
    /// Lambda: `x => body` or `(x, y) => body`
    Lambda {
        params: Vec<String>,
        body: Box<LoweredExpr>,
    },
    /// List literal: `[a, b, c]`
    List(Vec<LoweredExpr>),
    /// Record literal: `Name { a: 1 }` or `{ a: 1 }`
    Record {
        type_name: Option<String>,
        fields: Vec<(String, LoweredExpr)>,
    },
    /// For loop (map sugar): `for x in iterable { body }`
    For {
        binding: String,
        iterable: Box<LoweredExpr>,
        body: Box<LoweredExpr>,
    },
    /// Return: `return { field: value }`
    Return(Vec<(String, LoweredExpr)>),
    /// Sum-type variant construction: `Closed` or `Ok { value: x }`
    VariantConstruct {
        tag: String,
        fields: Vec<(String, LoweredExpr)>,
    },
}

/// Literal value (no Float — LoweredOp requires Eq; add via ordered-float if needed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredLiteral {
    Int(i64),
    Bool(bool),
    String(String),
    None,
}

/// String interpolation part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredStringPart {
    Literal(String),
    Expr(LoweredExpr),
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    NullCoalesce,
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredUnaryOp {
    Not,
    Neg,
}

/// Match arm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredMatchArm {
    pub pattern: LoweredPattern,
    pub guard: Option<LoweredExpr>,
    pub body: LoweredExpr,
}

/// Match pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredPattern {
    /// Bind to name (or unit variant)
    Ident(String),
    /// Variant with destructured fields
    Variant(String, Vec<(String, LoweredPattern)>),
    /// Wildcard `_`
    Wildcard,
    /// Literal value
    Literal(LoweredLiteral),
}

// ── AST → LoweredExpr translation ──────────────────────────────────────────

/// Controls how AST expressions are lowered — standard lowering vs. DAG port
/// remapping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprLowerMode {
    /// Standard lowering for fn/func bodies during the main lowering pass.
    Standard,
    /// Remapping mode: flattens `ident.field` to `ident__field` for DAG port wiring.
    /// Used by synthesize_expr_compute and collect_project_fn_bodies.
    Remap,
}

/// Lower an AST fn body to the compiler's expression IR.
pub fn lower_fn_body(body: &ast::FnBody, variant_names: &HashSet<String>) -> LoweredFnBody {
    lower_fn_body_with_mode(body, variant_names, ExprLowerMode::Standard)
}

/// Lower a single AST expression in Remap mode (flattens `ident.field` to `ident__field`).
pub fn lower_expr_remap(expr: &ast::Expr, variant_names: &HashSet<String>) -> LoweredExpr {
    lower_expr(expr, variant_names, ExprLowerMode::Remap)
}

/// Lower an AST fn body with an explicit lowering mode.
///
/// `Remap` mode flattens `ident.field` to `ident__field` for DAG port wiring
/// and is used by `synthesize_expr_compute` and `collect_project_fn_bodies`.
pub fn lower_fn_body_with_mode(
    body: &ast::FnBody,
    variant_names: &HashSet<String>,
    mode: ExprLowerMode,
) -> LoweredFnBody {
    LoweredFnBody {
        stmts: body
            .stmts
            .iter()
            .map(|s| lower_stmt(s, variant_names, mode))
            .collect(),
    }
}

/// Lower a single AST statement with an explicit lowering mode.
pub fn lower_stmt_with_mode(
    stmt: &ast::Stmt,
    variant_names: &HashSet<String>,
    mode: ExprLowerMode,
) -> LoweredStmt {
    lower_stmt(stmt, variant_names, mode)
}

fn lower_stmt(
    stmt: &ast::Stmt,
    variant_names: &HashSet<String>,
    mode: ExprLowerMode,
) -> LoweredStmt {
    match stmt {
        ast::Stmt::Let(name, expr) => {
            LoweredStmt::Let(name.clone(), lower_expr(expr, variant_names, mode))
        }
        ast::Stmt::Assign(name, expr) => {
            LoweredStmt::Let(name.clone(), lower_expr(expr, variant_names, mode))
        }
        ast::Stmt::Node(ns) => {
            let mut expr = lower_expr(&ns.expr, variant_names, mode);
            if let Some(guard) = &ns.when_guard {
                expr = LoweredExpr::IfElse {
                    cond: Box::new(lower_expr(guard, variant_names, mode)),
                    then_: Box::new(expr),
                    else_: Some(Box::new(LoweredExpr::Literal(LoweredLiteral::None))),
                };
            }
            LoweredStmt::Let(ns.name.clone(), expr)
        }
        ast::Stmt::Expr(expr) => LoweredStmt::Expr(lower_expr(expr, variant_names, mode)),
        ast::Stmt::Return(fields) => LoweredStmt::Return(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v, variant_names, mode)))
                .collect(),
        ),
    }
}

fn lower_expr(
    expr: &ast::Expr,
    variant_names: &HashSet<String>,
    mode: ExprLowerMode,
) -> LoweredExpr {
    match expr {
        ast::Expr::Literal(lit) => LoweredExpr::Literal(lower_literal(lit)),
        // Bare unit variant (e.g., `Closed`)
        ast::Expr::Ident(name) if variant_names.contains(name.as_str()) => {
            LoweredExpr::VariantConstruct {
                tag: name.clone(),
                fields: vec![],
            }
        }
        ast::Expr::Ident(name) => LoweredExpr::Ident(name.clone()),
        // Remap mode: flatten `ident.field` to `ident__field` for DAG port wiring
        ast::Expr::FieldAccess(base, field)
            if mode == ExprLowerMode::Remap && matches!(base.as_ref(), ast::Expr::Ident(_)) =>
        {
            if let ast::Expr::Ident(base_ident) = base.as_ref() {
                LoweredExpr::Ident(format!("{base_ident}__{field}"))
            } else {
                unreachable!()
            }
        }
        ast::Expr::FieldAccess(base, field) => LoweredExpr::FieldAccess {
            expr: Box::new(lower_expr(base, variant_names, mode)),
            field: field.clone(),
        },
        // Variant constructor call (e.g., `Ok(value: "x")`)
        ast::Expr::Call(name, args) if variant_names.contains(name.as_str()) => {
            LoweredExpr::VariantConstruct {
                tag: name.clone(),
                fields: args
                    .iter()
                    .enumerate()
                    .map(|(i, (k, v))| {
                        let field_name = k.clone().unwrap_or_else(|| format!("_{i}"));
                        (field_name, lower_expr(v, variant_names, mode))
                    })
                    .collect(),
            }
        }
        ast::Expr::Call(name, args) => LoweredExpr::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v, variant_names, mode)))
                .collect(),
        },
        ast::Expr::ServiceCall(path, args) => {
            // Service calls in fn bodies are lowered as calls with dotted name
            LoweredExpr::Call {
                name: path.join("."),
                args: args
                    .iter()
                    .map(|(k, v)| (k.clone(), lower_expr(v, variant_names, mode)))
                    .collect(),
            }
        }
        ast::Expr::BinOp(left, op, right) => LoweredExpr::BinOp {
            left: Box::new(lower_expr(left, variant_names, mode)),
            op: lower_binop(op),
            right: Box::new(lower_expr(right, variant_names, mode)),
        },
        ast::Expr::UnaryOp(op, expr) => LoweredExpr::UnaryOp {
            op: lower_unaryop(op),
            expr: Box::new(lower_expr(expr, variant_names, mode)),
        },
        ast::Expr::StringInterp(parts) => LoweredExpr::StringInterp(
            parts
                .iter()
                .map(|p| lower_string_part(p, variant_names, mode))
                .collect(),
        ),
        // Named variant record (e.g., `Ok { value: "x" }`)
        ast::Expr::Record(Some(name), fields) if variant_names.contains(name.as_str()) => {
            LoweredExpr::VariantConstruct {
                tag: name.clone(),
                fields: fields
                    .iter()
                    .map(|(k, v)| (k.clone(), lower_expr(v, variant_names, mode)))
                    .collect(),
            }
        }
        ast::Expr::Record(type_name, fields) => LoweredExpr::Record {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v, variant_names, mode)))
                .collect(),
        },
        ast::Expr::Match(scrutinee, arms) => LoweredExpr::Match {
            expr: Box::new(lower_expr(scrutinee, variant_names, mode)),
            arms: arms
                .iter()
                .map(|a| lower_match_arm(a, variant_names, mode))
                .collect(),
        },
        ast::Expr::If(cond, then_, else_) => LoweredExpr::IfElse {
            cond: Box::new(lower_expr(cond, variant_names, mode)),
            then_: Box::new(lower_expr(then_, variant_names, mode)),
            else_: else_
                .as_ref()
                .map(|e| Box::new(lower_expr(e, variant_names, mode))),
        },
        ast::Expr::For(binding, iterable, _passthrough, body) => LoweredExpr::For {
            binding: binding.clone(),
            iterable: Box::new(lower_expr(iterable, variant_names, mode)),
            body: Box::new(lower_expr(body, variant_names, mode)),
        },
        ast::Expr::Pipe(receiver, call) => LoweredExpr::Pipe {
            receiver: Box::new(lower_expr(receiver, variant_names, mode)),
            call: Box::new(lower_expr(call, variant_names, mode)),
        },
        ast::Expr::PipeCall(receiver, method, args) => LoweredExpr::Pipe {
            receiver: Box::new(lower_expr(receiver, variant_names, mode)),
            call: Box::new(LoweredExpr::Call {
                name: method.as_str().to_string(),
                args: args
                    .iter()
                    .map(|(k, v)| (k.clone(), lower_expr(v, variant_names, mode)))
                    .collect(),
            }),
        },
        ast::Expr::Lambda(params, body) => LoweredExpr::Lambda {
            params: params.clone(),
            body: Box::new(lower_expr(body, variant_names, mode)),
        },
        ast::Expr::List(items) => LoweredExpr::List(
            items
                .iter()
                .map(|i| lower_expr(i, variant_names, mode))
                .collect(),
        ),
        ast::Expr::Map(entries) => {
            // Map literals → Record with string keys
            LoweredExpr::Record {
                type_name: None,
                fields: entries
                    .iter()
                    .filter_map(|(k, v)| {
                        if let ast::Expr::Literal(ast::Literal::String(key)) = k {
                            Some((key.clone(), lower_expr(v, variant_names, mode)))
                        } else {
                            None
                        }
                    })
                    .collect(),
            }
        }
        ast::Expr::Guarded(expr, _guard) => {
            // Guards are DAG scheduling concerns — evaluate the inner expr
            lower_expr(expr, variant_names, mode)
        }
        ast::Expr::After(expr, _deps) => {
            // After deps are DAG scheduling concerns — evaluate the inner expr
            lower_expr(expr, variant_names, mode)
        }
        ast::Expr::Return(fields) => LoweredExpr::Return(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v, variant_names, mode)))
                .collect(),
        ),
    }
}

fn lower_literal(lit: &ast::Literal) -> LoweredLiteral {
    match lit {
        ast::Literal::Int(i) => LoweredLiteral::Int(*i),
        ast::Literal::Bool(b) => LoweredLiteral::Bool(*b),
        ast::Literal::String(s) => LoweredLiteral::String(s.clone()),
        ast::Literal::Float(_) => {
            // Float not representable in Eq-requiring IR; demote to string
            LoweredLiteral::String(format!("{}", lit_float_value(lit)))
        }
        ast::Literal::None => LoweredLiteral::None,
    }
}

/// Extract a float value from a literal. Returns `None` if the literal is
/// not a float — callers must handle the type mismatch explicitly rather
/// than silently falling back to 0.0.
pub(crate) fn lit_float_value(lit: &ast::Literal) -> f64 {
    match lit {
        ast::Literal::Float(f) => *f,
        // Only called on `Literal::Float` variants — the callers guard on
        // `Expr::Literal(Literal::Float(_))` match arms. If this is reached
        // with a non-float literal, it's an internal bug (not user input).
        other => unreachable!("lit_float_value called on non-float literal: {other:?}"),
    }
}

pub(crate) fn lower_binop(op: &ast::BinOp) -> LoweredBinOp {
    match op {
        ast::BinOp::Add => LoweredBinOp::Add,
        ast::BinOp::Sub => LoweredBinOp::Sub,
        ast::BinOp::Mul => LoweredBinOp::Mul,
        ast::BinOp::Div => LoweredBinOp::Div,
        ast::BinOp::Mod => LoweredBinOp::Mod,
        ast::BinOp::Eq => LoweredBinOp::Eq,
        ast::BinOp::Ne => LoweredBinOp::Ne,
        ast::BinOp::Lt => LoweredBinOp::Lt,
        ast::BinOp::Gt => LoweredBinOp::Gt,
        ast::BinOp::Le => LoweredBinOp::Le,
        ast::BinOp::Ge => LoweredBinOp::Ge,
        ast::BinOp::And => LoweredBinOp::And,
        ast::BinOp::Or => LoweredBinOp::Or,
        ast::BinOp::NullCoalesce => LoweredBinOp::NullCoalesce,
    }
}

fn lower_unaryop(op: &ast::UnaryOp) -> LoweredUnaryOp {
    match op {
        ast::UnaryOp::Not => LoweredUnaryOp::Not,
        ast::UnaryOp::Neg => LoweredUnaryOp::Neg,
    }
}

fn lower_string_part(
    part: &ast::StringPart,
    variant_names: &HashSet<String>,
    mode: ExprLowerMode,
) -> LoweredStringPart {
    match part {
        ast::StringPart::Literal(s) => LoweredStringPart::Literal(s.clone()),
        ast::StringPart::Expr(expr) => {
            LoweredStringPart::Expr(lower_expr(expr, variant_names, mode))
        }
    }
}

pub(crate) fn lower_match_arm(
    arm: &ast::MatchArm,
    variant_names: &HashSet<String>,
    mode: ExprLowerMode,
) -> LoweredMatchArm {
    LoweredMatchArm {
        pattern: lower_pattern(&arm.pattern, variant_names),
        guard: arm
            .guard
            .as_ref()
            .map(|g| lower_expr(g, variant_names, mode)),
        body: lower_expr(&arm.body, variant_names, mode),
    }
}

fn lower_pattern(pattern: &ast::Pattern, variant_names: &HashSet<String>) -> LoweredPattern {
    match pattern {
        ast::Pattern::Ident(name) if name == "None" || name == "null" => {
            LoweredPattern::Literal(LoweredLiteral::None)
        }
        // Unit variant in match arm (e.g., `match x { Closed => ... }`)
        ast::Pattern::Ident(name) if variant_names.contains(name.as_str()) => {
            LoweredPattern::Variant(name.clone(), vec![])
        }
        ast::Pattern::Ident(name) => LoweredPattern::Ident(name.clone()),
        ast::Pattern::Variant(name, fields) => LoweredPattern::Variant(
            name.clone(),
            fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_pattern(v, variant_names)))
                .collect(),
        ),
        ast::Pattern::Wildcard => LoweredPattern::Wildcard,
        ast::Pattern::Literal(lit) => LoweredPattern::Literal(lower_literal(lit)),
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_empty_fn_body() {
        let body = ast::FnBody {
            stmts: vec![],
            lossy: false,
        };
        let lowered = lower_fn_body(&body, &HashSet::new());
        assert!(lowered.stmts.is_empty());
    }

    #[test]
    fn lower_let_with_string_interp() {
        let body = ast::FnBody {
            stmts: vec![ast::Stmt::Let(
                "msg".to_string(),
                ast::Expr::StringInterp(vec![
                    ast::StringPart::Literal("hello ".to_string()),
                    ast::StringPart::Expr(ast::Expr::Ident("name".to_string())),
                ]),
            )],
            lossy: false,
        };
        let lowered = lower_fn_body(&body, &HashSet::new());
        assert_eq!(lowered.stmts.len(), 1);
        match &lowered.stmts[0] {
            LoweredStmt::Let(name, LoweredExpr::StringInterp(parts)) => {
                assert_eq!(name, "msg");
                assert_eq!(parts.len(), 2);
                assert_eq!(parts[0], LoweredStringPart::Literal("hello ".to_string()));
            }
            other => panic!("expected Let with StringInterp, got: {other:?}"),
        }
    }

    #[test]
    fn lower_if_else() {
        let body = ast::FnBody {
            stmts: vec![ast::Stmt::Expr(ast::Expr::If(
                Box::new(ast::Expr::Ident("flag".to_string())),
                Box::new(ast::Expr::Literal(ast::Literal::String("yes".to_string()))),
                Some(Box::new(ast::Expr::Literal(ast::Literal::String(
                    "no".to_string(),
                )))),
            ))],
            lossy: false,
        };
        let lowered = lower_fn_body(&body, &HashSet::new());
        match &lowered.stmts[0] {
            LoweredStmt::Expr(LoweredExpr::IfElse { cond, then_, else_ }) => {
                assert!(matches!(cond.as_ref(), LoweredExpr::Ident(n) if n == "flag"));
                assert!(
                    matches!(then_.as_ref(), LoweredExpr::Literal(LoweredLiteral::String(s)) if s == "yes")
                );
                assert!(else_.is_some());
            }
            other => panic!("expected IfElse, got: {other:?}"),
        }
    }

    #[test]
    fn lower_pipe_chain() {
        // items |> join("\n")
        let expr = ast::Expr::Pipe(
            Box::new(ast::Expr::Ident("items".to_string())),
            Box::new(ast::Expr::Call(
                "join".to_string(),
                vec![(
                    None,
                    ast::Expr::Literal(ast::Literal::String("\n".to_string())),
                )],
            )),
        );
        let lowered = lower_expr(&expr, &HashSet::new(), ExprLowerMode::Standard);
        match &lowered {
            LoweredExpr::Pipe { receiver, call } => {
                assert!(matches!(receiver.as_ref(), LoweredExpr::Ident(n) if n == "items"));
                assert!(matches!(call.as_ref(), LoweredExpr::Call { name, .. } if name == "join"));
            }
            other => panic!("expected Pipe, got: {other:?}"),
        }
    }

    #[test]
    fn lower_fn_call_with_named_args() {
        let expr = ast::Expr::Call(
            "render_target".to_string(),
            vec![
                (Some("name".to_string()), ast::Expr::Ident("n".to_string())),
                (Some("deps".to_string()), ast::Expr::List(vec![])),
            ],
        );
        let lowered = lower_expr(&expr, &HashSet::new(), ExprLowerMode::Standard);
        match &lowered {
            LoweredExpr::Call { name, args } => {
                assert_eq!(name, "render_target");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0].0, Some("name".to_string()));
                assert_eq!(args[1].0, Some("deps".to_string()));
            }
            other => panic!("expected Call, got: {other:?}"),
        }
    }

    #[test]
    fn lower_bare_variant_ident() {
        let variant_names: HashSet<String> =
            ["Closed", "Open"].iter().map(|s| s.to_string()).collect();
        let expr = ast::Expr::Ident("Closed".to_string());
        let lowered = lower_expr(&expr, &variant_names, ExprLowerMode::Standard);
        match &lowered {
            LoweredExpr::VariantConstruct { tag, fields } => {
                assert_eq!(tag, "Closed");
                assert!(fields.is_empty());
            }
            other => panic!("expected VariantConstruct, got: {other:?}"),
        }
    }

    #[test]
    fn lower_variant_call() {
        let variant_names: HashSet<String> = ["Ok", "Err"].iter().map(|s| s.to_string()).collect();
        let expr = ast::Expr::Call(
            "Ok".to_string(),
            vec![(
                Some("value".to_string()),
                ast::Expr::Literal(ast::Literal::String("hello".to_string())),
            )],
        );
        let lowered = lower_expr(&expr, &variant_names, ExprLowerMode::Standard);
        match &lowered {
            LoweredExpr::VariantConstruct { tag, fields } => {
                assert_eq!(tag, "Ok");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0, "value");
            }
            other => panic!("expected VariantConstruct, got: {other:?}"),
        }
    }

    #[test]
    fn lower_variant_record() {
        let variant_names: HashSet<String> = ["Ok", "Err"].iter().map(|s| s.to_string()).collect();
        let expr = ast::Expr::Record(
            Some("Ok".to_string()),
            vec![(
                "value".to_string(),
                ast::Expr::Literal(ast::Literal::String("hello".to_string())),
            )],
        );
        let lowered = lower_expr(&expr, &variant_names, ExprLowerMode::Standard);
        match &lowered {
            LoweredExpr::VariantConstruct { tag, fields } => {
                assert_eq!(tag, "Ok");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0, "value");
            }
            other => panic!("expected VariantConstruct, got: {other:?}"),
        }
    }

    #[test]
    fn lower_variant_pattern() {
        let variant_names: HashSet<String> =
            ["Closed", "Open"].iter().map(|s| s.to_string()).collect();
        let pattern = ast::Pattern::Ident("Closed".to_string());
        let lowered = lower_pattern(&pattern, &variant_names);
        match &lowered {
            LoweredPattern::Variant(name, fields) => {
                assert_eq!(name, "Closed");
                assert!(fields.is_empty());
            }
            other => panic!("expected Variant pattern, got: {other:?}"),
        }
    }
}
