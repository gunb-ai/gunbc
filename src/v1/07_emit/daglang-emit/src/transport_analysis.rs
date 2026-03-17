//! Shared code-IR analysis for emit backends.
//!
//! These functions detect whether CodeIR expressions and statements contain
//! specific node kinds (transport calls, JSON literals, etc.). All backends
//! use them to determine import requirements and classify statements for
//! code generation.

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

// ===========================================================================
// JSON literal detection
// ===========================================================================

/// Check if any statement in a function body contains JSON literal values.
///
/// Used by multiple backends (Rust, Go) to conditionally emit JSON-related
/// imports (e.g. `serde_json::Value`, `encoding/json`).
pub fn body_uses_json(stmts: &[Stmt]) -> bool {
    stmts.iter().any(stmt_uses_json)
}

fn stmt_uses_json(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { expr, .. } | Stmt::Bind { expr, .. } => expr_uses_json(expr),
        Stmt::Expr(expr) | Stmt::Return(expr) | Stmt::TailExpr(expr) => expr_uses_json(expr),
        Stmt::For { body, .. } => body_uses_json(body),
        _ => false,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::code_ir::Expr;

    #[test]
    fn json_literal_detected_in_let() {
        let stmts = vec![Stmt::Let {
            name: "x".into(),
            mutable: false,
            expr: Expr::Value(gunbc_ir::ValueExpr::Json(
                serde_json::json!({"key": "value"}),
            )),
            ir_type: None,
        }];
        assert!(body_uses_json(&stmts));
    }

    #[test]
    fn no_json_in_plain_body() {
        let stmts = vec![Stmt::Let {
            name: "x".into(),
            mutable: false,
            expr: Expr::Value(gunbc_ir::ValueExpr::Str("hello".into())),
            ir_type: None,
        }];
        assert!(!body_uses_json(&stmts));
    }

    #[test]
    fn json_detected_in_nested_call() {
        let json_expr = Expr::Value(gunbc_ir::ValueExpr::Json(serde_json::json!(42)));
        let call_expr = Expr::Call {
            func: Box::new(Expr::Var("f".into())),
            args: vec![json_expr],
            obligation: None,
        };
        let stmts = vec![Stmt::Expr(call_expr)];
        assert!(body_uses_json(&stmts));
    }

    #[test]
    fn json_detected_in_array() {
        let json_expr = Expr::Value(gunbc_ir::ValueExpr::Json(serde_json::json!(null)));
        let stmts = vec![Stmt::Expr(Expr::Array(vec![json_expr]))];
        assert!(body_uses_json(&stmts));
    }

    #[test]
    fn json_detected_in_binop() {
        let json_expr = Expr::Value(gunbc_ir::ValueExpr::Json(serde_json::json!("a")));
        let binop = Expr::BinOp {
            op: "==".into(),
            left: Box::new(json_expr),
            right: Box::new(Expr::Var("y".into())),
        };
        let stmts = vec![Stmt::Expr(binop)];
        assert!(body_uses_json(&stmts));
    }
}
