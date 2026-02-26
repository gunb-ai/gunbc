//! Lowered expression IR — the compiler's representation of fn body computation.
//!
//! Translates `ast::FnBody` → `LoweredFnBody` during lowering. Each pipeline
//! stage produces its own representation: `.dag → parse (AST) → typecheck →
//! lower (LoweredExpr) → eval`. Downstream consumers never see parser types.

use daglang_syntax::ast;

// ── IR types ────────────────────────────────────────────────────────────────

/// A lowered function body — the unit of computation for `fn` items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFnBody {
    pub stmts: Vec<LoweredStmt>,
}

/// A lowered statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredStmt {
    /// `let name = expr`
    Let(String, LoweredExpr),
    /// Expression statement (side effects or trailing return)
    Expr(LoweredExpr),
    /// `return { field: expr, ... }`
    Return(Vec<(String, LoweredExpr)>),
}

/// A lowered expression — fully independent of parser AST types.
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// Literal value (no Float — LoweredOp requires Eq; add via ordered-float if needed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredLiteral {
    Int(i64),
    Bool(bool),
    String(String),
    None,
}

/// String interpolation part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredStringPart {
    Literal(String),
    Expr(LoweredExpr),
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoweredUnaryOp {
    Not,
    Neg,
}

/// Match arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredMatchArm {
    pub pattern: LoweredPattern,
    pub guard: Option<LoweredExpr>,
    pub body: LoweredExpr,
}

/// Match pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Lower an AST fn body to the compiler's expression IR.
pub fn lower_fn_body(body: &ast::FnBody) -> LoweredFnBody {
    LoweredFnBody {
        stmts: body.stmts.iter().map(lower_stmt).collect(),
    }
}

fn lower_stmt(stmt: &ast::Stmt) -> LoweredStmt {
    match stmt {
        ast::Stmt::Let(name, expr) => LoweredStmt::Let(name.clone(), lower_expr(expr)),
        ast::Stmt::Assign(name, expr) => LoweredStmt::Let(name.clone(), lower_expr(expr)),
        ast::Stmt::Node(ns) => {
            let mut expr = lower_expr(&ns.expr);
            if let Some(guard) = &ns.when_guard {
                expr = LoweredExpr::IfElse {
                    cond: Box::new(lower_expr(guard)),
                    then_: Box::new(expr),
                    else_: Some(Box::new(LoweredExpr::Literal(LoweredLiteral::None))),
                };
            }
            LoweredStmt::Let(ns.name.clone(), expr)
        }
        ast::Stmt::Expr(expr) => LoweredStmt::Expr(lower_expr(expr)),
        ast::Stmt::Return(fields) => LoweredStmt::Return(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v)))
                .collect(),
        ),
    }
}

fn lower_expr(expr: &ast::Expr) -> LoweredExpr {
    match expr {
        ast::Expr::Literal(lit) => LoweredExpr::Literal(lower_literal(lit)),
        ast::Expr::Ident(name) => LoweredExpr::Ident(name.clone()),
        ast::Expr::FieldAccess(base, field) => LoweredExpr::FieldAccess {
            expr: Box::new(lower_expr(base)),
            field: field.clone(),
        },
        ast::Expr::Call(name, args) => LoweredExpr::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v)))
                .collect(),
        },
        ast::Expr::ServiceCall(path, args) => {
            // Service calls in fn bodies are lowered as calls with dotted name
            LoweredExpr::Call {
                name: path.join("."),
                args: args
                    .iter()
                    .map(|(k, v)| (k.clone(), lower_expr(v)))
                    .collect(),
            }
        }
        ast::Expr::BinOp(left, op, right) => LoweredExpr::BinOp {
            left: Box::new(lower_expr(left)),
            op: lower_binop(op),
            right: Box::new(lower_expr(right)),
        },
        ast::Expr::UnaryOp(op, expr) => LoweredExpr::UnaryOp {
            op: lower_unaryop(op),
            expr: Box::new(lower_expr(expr)),
        },
        ast::Expr::StringInterp(parts) => {
            LoweredExpr::StringInterp(parts.iter().map(lower_string_part).collect())
        }
        ast::Expr::Record(type_name, fields) => LoweredExpr::Record {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v)))
                .collect(),
        },
        ast::Expr::Match(scrutinee, arms) => LoweredExpr::Match {
            expr: Box::new(lower_expr(scrutinee)),
            arms: arms.iter().map(lower_match_arm).collect(),
        },
        ast::Expr::If(cond, then_, else_) => LoweredExpr::IfElse {
            cond: Box::new(lower_expr(cond)),
            then_: Box::new(lower_expr(then_)),
            else_: else_.as_ref().map(|e| Box::new(lower_expr(e))),
        },
        ast::Expr::For(binding, iterable, _passthrough, body) => LoweredExpr::For {
            binding: binding.clone(),
            iterable: Box::new(lower_expr(iterable)),
            body: Box::new(lower_expr(body)),
        },
        ast::Expr::Pipe(receiver, call) => LoweredExpr::Pipe {
            receiver: Box::new(lower_expr(receiver)),
            call: Box::new(lower_expr(call)),
        },
        ast::Expr::Lambda(params, body) => LoweredExpr::Lambda {
            params: params.clone(),
            body: Box::new(lower_expr(body)),
        },
        ast::Expr::List(items) => LoweredExpr::List(items.iter().map(lower_expr).collect()),
        ast::Expr::Map(entries) => {
            // Map literals → Record with string keys
            LoweredExpr::Record {
                type_name: None,
                fields: entries
                    .iter()
                    .filter_map(|(k, v)| {
                        if let ast::Expr::Literal(ast::Literal::String(key)) = k {
                            Some((key.clone(), lower_expr(v)))
                        } else {
                            None
                        }
                    })
                    .collect(),
            }
        }
        ast::Expr::Guarded(expr, _guard) => {
            // Guards are DAG scheduling concerns — evaluate the inner expr
            lower_expr(expr)
        }
        ast::Expr::After(expr, _deps) => {
            // After deps are DAG scheduling concerns — evaluate the inner expr
            lower_expr(expr)
        }
        ast::Expr::Return(fields) => LoweredExpr::Return(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_expr(v)))
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

fn lit_float_value(lit: &ast::Literal) -> f64 {
    match lit {
        ast::Literal::Float(f) => *f,
        _ => 0.0,
    }
}

fn lower_binop(op: &ast::BinOp) -> LoweredBinOp {
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

fn lower_string_part(part: &ast::StringPart) -> LoweredStringPart {
    match part {
        ast::StringPart::Literal(s) => LoweredStringPart::Literal(s.clone()),
        ast::StringPart::Expr(expr) => LoweredStringPart::Expr(lower_expr(expr)),
    }
}

fn lower_match_arm(arm: &ast::MatchArm) -> LoweredMatchArm {
    LoweredMatchArm {
        pattern: lower_pattern(&arm.pattern),
        guard: arm.guard.as_ref().map(lower_expr),
        body: lower_expr(&arm.body),
    }
}

fn lower_pattern(pattern: &ast::Pattern) -> LoweredPattern {
    match pattern {
        ast::Pattern::Ident(name) if name == "None" || name == "null" => {
            LoweredPattern::Literal(LoweredLiteral::None)
        }
        ast::Pattern::Ident(name) => LoweredPattern::Ident(name.clone()),
        ast::Pattern::Variant(name, fields) => LoweredPattern::Variant(
            name.clone(),
            fields
                .iter()
                .map(|(k, v)| (k.clone(), lower_pattern(v)))
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
        let lowered = lower_fn_body(&body);
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
        let lowered = lower_fn_body(&body);
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
        let lowered = lower_fn_body(&body);
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
        let lowered = lower_expr(&expr);
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
        let lowered = lower_expr(&expr);
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
}
