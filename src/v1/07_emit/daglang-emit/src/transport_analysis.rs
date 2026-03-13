//! Shared transport call analysis for emit backends.
//!
//! These functions detect whether CodeIR expressions and statements contain
//! transport (runtime I/O) calls. All backends use them to determine import
//! requirements and classify statements for code generation.

use gunbc_ir::code_ir::{CallObligation, Expr, Stmt};

/// Check if an expression is a transport or resource runtime call.
pub fn expr_is_transport_call(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call {
            obligation: Some(obligation),
            ..
        } if obligation.is_runtime_call()
    )
}

/// Check if any statement in a function body contains transport calls.
pub fn body_has_transport_calls(stmts: &[Stmt]) -> bool {
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
        Expr::Call {
            func,
            args,
            obligation,
        } => {
            obligation.is_some_and(CallObligation::is_runtime_call)
                || expr_has_transport(func)
                || args.iter().any(expr_has_transport)
        }
        Expr::MethodCall { receiver, args, .. } => {
            expr_has_transport(receiver) || args.iter().any(expr_has_transport)
        }
        Expr::BinOp { left, right, .. } => expr_has_transport(left) || expr_has_transport(right),
        Expr::UnaryOp { expr, .. } => expr_has_transport(expr),
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
        Expr::Field(inner, _) | Expr::Deref(inner) | Expr::Ref(inner) | Expr::RefMut(inner) => {
            expr_has_transport(inner)
        }
        _ => false,
    }
}
