//! ANF (A-normal form) normalization for lowered IR.
//!
//! Ensures `LoweredExpr::Call` only appears at statement level:
//! `LoweredStmt::Let(name, Call{..})` or `LoweredStmt::Expr(Call{..})`.
//! Never nested inside another `LoweredExpr`.
//!
//! This is the boundary contract between lowerer and evaluator
//! (DESIGN-eval-redesign.md §"The Lowering Contract").

use daglang_eval::expr::{
    LoweredBinOp, LoweredExpr, LoweredFnBody, LoweredLiteral, LoweredMatchArm, LoweredStmt,
    LoweredStringPart,
};

// ── Public API ──────────────────────────────────────────────────────────────

/// Normalize a fn body to ANF: hoist all nested calls to statement level.
pub fn anf_normalize(body: LoweredFnBody) -> LoweredFnBody {
    let mut state = AnfState { counter: 0 };
    LoweredFnBody {
        stmts: anf_stmts(body.stmts, &mut state),
    }
}

/// Structural verification: returns Err if any `LoweredExpr::Call` is nested
/// inside another `LoweredExpr` (i.e., not at statement level).
pub fn verify_anf(body: &LoweredFnBody) -> Result<(), String> {
    for (i, stmt) in body.stmts.iter().enumerate() {
        verify_stmt(stmt, &format!("stmt[{i}]"))?;
    }
    Ok(())
}

// ── ANF state ───────────────────────────────────────────────────────────────

struct AnfState {
    counter: usize,
}

impl AnfState {
    fn fresh_name(&mut self) -> String {
        let n = self.counter;
        self.counter += 1;
        format!("__anf_{n}")
    }
}

// ── Statement normalization ─────────────────────────────────────────────────

fn anf_stmts(stmts: Vec<LoweredStmt>, state: &mut AnfState) -> Vec<LoweredStmt> {
    let mut result = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        result.extend(anf_stmt(stmt, state));
    }
    result
}

/// Normalize one statement. May produce multiple (hoisted lets + the original).
fn anf_stmt(stmt: LoweredStmt, state: &mut AnfState) -> Vec<LoweredStmt> {
    match stmt {
        LoweredStmt::Let(name, expr) => {
            let (prefix, clean) = anf_expr_in_stmt(expr, state);
            let mut out = prefix;
            out.push(LoweredStmt::Let(name, clean));
            out
        }
        LoweredStmt::Expr(expr) => {
            let (prefix, clean) = anf_expr_in_stmt(expr, state);
            let mut out = prefix;
            out.push(LoweredStmt::Expr(clean));
            out
        }
        LoweredStmt::Return(fields) => {
            let mut prefix = Vec::new();
            let mut clean_fields = Vec::with_capacity(fields.len());
            for (name, expr) in fields {
                let (p, e) = anf_expr_hoist(expr, state);
                prefix.extend(p);
                clean_fields.push((name, e));
            }
            let mut out = prefix;
            out.push(LoweredStmt::Return(clean_fields));
            out
        }
    }
}

// ── Expression normalization ────────────────────────────────────────────────
//
// Two modes:
//  - `anf_expr_in_stmt`: top-level of a Let/Expr statement. A Call here is OK
//    (it IS at statement level), but its args must be call-free.
//  - `anf_expr_hoist`: nested inside another expression. Any Call must be
//    hoisted out as a Let, replaced with a fresh Ident.

/// Process an expression at the top of a statement (Let/Expr).
/// A Call at this level stays in place; nested calls within are hoisted.
fn anf_expr_in_stmt(
    expr: LoweredExpr,
    state: &mut AnfState,
) -> (Vec<LoweredStmt>, LoweredExpr) {
    match expr {
        LoweredExpr::Call { name, args } => {
            // Call at statement level is fine — just hoist its args.
            let mut prefix = Vec::new();
            let mut clean_args = Vec::with_capacity(args.len());
            for (label, arg) in args {
                let (p, a) = anf_expr_hoist(arg, state);
                prefix.extend(p);
                clean_args.push((label, a));
            }
            (prefix, LoweredExpr::Call { name, args: clean_args })
        }
        _ => anf_expr_hoist(expr, state),
    }
}

/// Process an expression in nested position. Calls are hoisted out.
fn anf_expr_hoist(
    expr: LoweredExpr,
    state: &mut AnfState,
) -> (Vec<LoweredStmt>, LoweredExpr) {
    match expr {
        // ── Leaves ──────────────────────────────────────────────────────
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => (vec![], expr),

        // ── Call: hoist to a Let ────────────────────────────────────────
        LoweredExpr::Call { name, args } => {
            let mut prefix = Vec::new();
            let mut clean_args = Vec::with_capacity(args.len());
            for (label, arg) in args {
                let (p, a) = anf_expr_hoist(arg, state);
                prefix.extend(p);
                clean_args.push((label, a));
            }
            let fresh = state.fresh_name();
            prefix.push(LoweredStmt::Let(
                fresh.clone(),
                LoweredExpr::Call { name, args: clean_args },
            ));
            (prefix, LoweredExpr::Ident(fresh))
        }

        // ── Field access ────────────────────────────────────────────────
        LoweredExpr::FieldAccess { expr, field } => {
            let (prefix, clean) = anf_expr_hoist(*expr, state);
            (prefix, LoweredExpr::FieldAccess { expr: Box::new(clean), field })
        }

        // ── String interpolation ────────────────────────────────────────
        LoweredExpr::StringInterp(parts) => {
            let mut prefix = Vec::new();
            let mut clean_parts = Vec::with_capacity(parts.len());
            for part in parts {
                match part {
                    LoweredStringPart::Literal(s) => {
                        clean_parts.push(LoweredStringPart::Literal(s));
                    }
                    LoweredStringPart::Expr(e) => {
                        let (p, c) = anf_expr_hoist(e, state);
                        prefix.extend(p);
                        clean_parts.push(LoweredStringPart::Expr(c));
                    }
                }
            }
            (prefix, LoweredExpr::StringInterp(clean_parts))
        }

        // ── Binary operations ───────────────────────────────────────────
        LoweredExpr::BinOp { left, op, right } => {
            anf_binop(*left, op, *right, state)
        }

        // ── Unary operations ────────────────────────────────────────────
        LoweredExpr::UnaryOp { op, expr } => {
            let (prefix, clean) = anf_expr_hoist(*expr, state);
            (prefix, LoweredExpr::UnaryOp { op, expr: Box::new(clean) })
        }

        // ── If/else ─────────────────────────────────────────────────────
        LoweredExpr::IfElse { cond, then_, else_ } => {
            let (cond_prefix, cond_clean) = anf_expr_hoist(*cond, state);
            let then_clean = anf_branch(*then_, state);
            let else_clean = else_.map(|e| Box::new(anf_branch(*e, state)));
            (
                cond_prefix,
                LoweredExpr::IfElse {
                    cond: Box::new(cond_clean),
                    then_: Box::new(then_clean),
                    else_: else_clean,
                },
            )
        }

        // ── Match ───────────────────────────────────────────────────────
        LoweredExpr::Match { expr, arms } => {
            let (scrutinee_prefix, scrutinee_clean) = anf_expr_hoist(*expr, state);
            let clean_arms: Vec<_> = arms
                .into_iter()
                .map(|arm| LoweredMatchArm {
                    pattern: arm.pattern,
                    guard: arm.guard.map(|g| {
                        let (gp, gc) = anf_expr_hoist(g, state);
                        if !gp.is_empty() {
                            wrap_in_block(gp, gc)
                        } else {
                            gc
                        }
                    }),
                    body: anf_branch(arm.body, state),
                })
                .collect();
            (
                scrutinee_prefix,
                LoweredExpr::Match { expr: Box::new(scrutinee_clean), arms: clean_arms },
            )
        }

        // ── Lambda ──────────────────────────────────────────────────────
        LoweredExpr::Lambda { params, body } => {
            let clean_body = anf_branch(*body, state);
            (vec![], LoweredExpr::Lambda { params, body: Box::new(clean_body) })
        }

        // ── List ────────────────────────────────────────────────────────
        LoweredExpr::List(items) => {
            let mut prefix = Vec::new();
            let mut clean_items = Vec::with_capacity(items.len());
            for item in items {
                let (p, c) = anf_expr_hoist(item, state);
                prefix.extend(p);
                clean_items.push(c);
            }
            (prefix, LoweredExpr::List(clean_items))
        }

        // ── Block ───────────────────────────────────────────────────────
        LoweredExpr::Block(stmts) => {
            let clean = anf_stmts(stmts, state);
            (vec![], LoweredExpr::Block(clean))
        }

        // ── Record ──────────────────────────────────────────────────────
        LoweredExpr::Record { type_name, fields } => {
            let mut prefix = Vec::new();
            let mut clean_fields = Vec::with_capacity(fields.len());
            for (name, expr) in fields {
                let (p, c) = anf_expr_hoist(expr, state);
                prefix.extend(p);
                clean_fields.push((name, c));
            }
            (prefix, LoweredExpr::Record { type_name, fields: clean_fields })
        }

        // ── For loop ────────────────────────────────────────────────────
        LoweredExpr::For { binding, iterable, body } => {
            let (iter_prefix, iter_clean) = anf_expr_hoist(*iterable, state);
            let body_clean = anf_branch(*body, state);
            (
                iter_prefix,
                LoweredExpr::For {
                    binding,
                    iterable: Box::new(iter_clean),
                    body: Box::new(body_clean),
                },
            )
        }

        // ── Return expression ───────────────────────────────────────────
        LoweredExpr::Return(fields) => {
            let mut prefix = Vec::new();
            let mut clean_fields = Vec::with_capacity(fields.len());
            for (name, expr) in fields {
                let (p, c) = anf_expr_hoist(expr, state);
                prefix.extend(p);
                clean_fields.push((name, c));
            }
            (prefix, LoweredExpr::Return(clean_fields))
        }

        // ── Variant construct ───────────────────────────────────────────
        LoweredExpr::VariantConstruct { tag, fields } => {
            let mut prefix = Vec::new();
            let mut clean_fields = Vec::with_capacity(fields.len());
            for (name, expr) in fields {
                let (p, c) = anf_expr_hoist(expr, state);
                prefix.extend(p);
                clean_fields.push((name, c));
            }
            (prefix, LoweredExpr::VariantConstruct { tag, fields: clean_fields })
        }
    }
}

// ── Binary operation normalization ──────────────────────────────────────────

fn anf_binop(
    left: LoweredExpr,
    op: LoweredBinOp,
    right: LoweredExpr,
    state: &mut AnfState,
) -> (Vec<LoweredStmt>, LoweredExpr) {
    match op {
        // Short-circuit: right side is conditionally evaluated.
        // If right contains calls, desugar to IfElse so calls end up at
        // statement level within the branch (not nested in BinOp).
        LoweredBinOp::And if contains_call(&right) => {
            let (left_prefix, left_clean) = anf_expr_hoist(left, state);
            let right_branch = anf_branch(right, state);
            (
                left_prefix,
                LoweredExpr::IfElse {
                    cond: Box::new(left_clean),
                    then_: Box::new(right_branch),
                    else_: Some(Box::new(LoweredExpr::Literal(LoweredLiteral::Bool(false)))),
                },
            )
        }
        LoweredBinOp::Or if contains_call(&right) => {
            let (left_prefix, left_clean) = anf_expr_hoist(left, state);
            let right_branch = anf_branch(right, state);
            (
                left_prefix,
                LoweredExpr::IfElse {
                    cond: Box::new(left_clean),
                    then_: Box::new(LoweredExpr::Literal(LoweredLiteral::Bool(true))),
                    else_: Some(Box::new(right_branch)),
                },
            )
        }
        LoweredBinOp::NullCoalesce if contains_call(&right) => {
            let (left_prefix, left_clean) = anf_expr_hoist(left, state);
            let fresh = state.fresh_name();
            let right_branch = anf_branch(right, state);
            let mut prefix = left_prefix;
            prefix.push(LoweredStmt::Let(fresh.clone(), left_clean));
            (
                prefix,
                LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(LoweredExpr::Ident(fresh.clone())),
                        op: LoweredBinOp::Ne,
                        right: Box::new(LoweredExpr::Literal(LoweredLiteral::None)),
                    }),
                    then_: Box::new(LoweredExpr::Ident(fresh)),
                    else_: Some(Box::new(right_branch)),
                },
            )
        }
        // Non-short-circuit (or short-circuit where right has no calls):
        // hoist both sides normally.
        _ => {
            let (left_prefix, left_clean) = anf_expr_hoist(left, state);
            let (right_prefix, right_clean) = anf_expr_hoist(right, state);
            let mut prefix = left_prefix;
            prefix.extend(right_prefix);
            (
                prefix,
                LoweredExpr::BinOp {
                    left: Box::new(left_clean),
                    op,
                    right: Box::new(right_clean),
                },
            )
        }
    }
}

// ── Branch normalization ────────────────────────────────────────────────────

/// Normalize an expression that forms a branch body (if/else, match arm, etc).
/// If hoisting produces prefix statements OR the result is a Call, wraps
/// everything in a Block so that calls are always at statement level.
fn anf_branch(expr: LoweredExpr, state: &mut AnfState) -> LoweredExpr {
    match expr {
        // Block: normalize its statements directly.
        LoweredExpr::Block(stmts) => {
            LoweredExpr::Block(anf_stmts(stmts, state))
        }
        // Non-block: hoist, and wrap in Block if needed.
        other => {
            let (prefix, clean) = anf_expr_in_stmt(other, state);
            let needs_block = !prefix.is_empty() || matches!(clean, LoweredExpr::Call { .. });
            if needs_block {
                wrap_in_block(prefix, clean)
            } else {
                clean
            }
        }
    }
}

/// Wrap prefix statements + trailing expression into a Block.
fn wrap_in_block(mut prefix: Vec<LoweredStmt>, trailing: LoweredExpr) -> LoweredExpr {
    prefix.push(LoweredStmt::Expr(trailing));
    LoweredExpr::Block(prefix)
}

// ── Call detection ──────────────────────────────────────────────────────────

/// Returns true if the expression tree contains any `Call` node.
fn contains_call(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { .. } => true,
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => false,
        LoweredExpr::FieldAccess { expr, .. } => contains_call(expr),
        LoweredExpr::StringInterp(parts) => parts.iter().any(|p| match p {
            LoweredStringPart::Literal(_) => false,
            LoweredStringPart::Expr(e) => contains_call(e),
        }),
        LoweredExpr::BinOp { left, right, .. } => {
            contains_call(left) || contains_call(right)
        }
        LoweredExpr::UnaryOp { expr, .. } => contains_call(expr),
        LoweredExpr::IfElse { cond, then_, else_ } => {
            contains_call(cond)
                || contains_call(then_)
                || else_.as_ref().is_some_and(|e| contains_call(e))
        }
        LoweredExpr::Match { expr, arms } => {
            contains_call(expr)
                || arms.iter().any(|a| {
                    contains_call(&a.body)
                        || a.guard.as_ref().is_some_and(|g| contains_call(g))
                })
        }
        LoweredExpr::Lambda { body, .. } => contains_call(body),
        LoweredExpr::List(items) => items.iter().any(contains_call),
        LoweredExpr::Block(stmts) => stmts.iter().any(stmt_contains_call),
        LoweredExpr::Record { fields, .. } => fields.iter().any(|(_, e)| contains_call(e)),
        LoweredExpr::For { iterable, body, .. } => {
            contains_call(iterable) || contains_call(body)
        }
        LoweredExpr::Return(fields) => fields.iter().any(|(_, e)| contains_call(e)),
        LoweredExpr::VariantConstruct { fields, .. } => {
            fields.iter().any(|(_, e)| contains_call(e))
        }
    }
}

fn stmt_contains_call(stmt: &LoweredStmt) -> bool {
    match stmt {
        LoweredStmt::Let(_, expr) | LoweredStmt::Expr(expr) => contains_call(expr),
        LoweredStmt::Return(fields) => fields.iter().any(|(_, e)| contains_call(e)),
    }
}

// ── Structural verifier ─────────────────────────────────────────────────────

fn verify_stmt(stmt: &LoweredStmt, path: &str) -> Result<(), String> {
    match stmt {
        LoweredStmt::Let(name, expr) => {
            let p = format!("{path}/Let({name})");
            match expr {
                // Call at statement level: verify args are call-free.
                LoweredExpr::Call { args, .. } => {
                    for (i, (_, arg)) in args.iter().enumerate() {
                        verify_no_call(arg, &format!("{p}/arg[{i}]"))?;
                    }
                    Ok(())
                }
                _ => verify_no_call(expr, &p),
            }
        }
        LoweredStmt::Expr(expr) => {
            let p = format!("{path}/Expr");
            match expr {
                LoweredExpr::Call { args, .. } => {
                    for (i, (_, arg)) in args.iter().enumerate() {
                        verify_no_call(arg, &format!("{p}/arg[{i}]"))?;
                    }
                    Ok(())
                }
                _ => verify_no_call(expr, &p),
            }
        }
        LoweredStmt::Return(fields) => {
            for (i, (_, expr)) in fields.iter().enumerate() {
                verify_no_call(expr, &format!("{path}/Return[{i}]"))?;
            }
            Ok(())
        }
    }
}

/// Verify no Call appears in this expression tree (recursing into scopes).
fn verify_no_call(expr: &LoweredExpr, path: &str) -> Result<(), String> {
    match expr {
        LoweredExpr::Call { name, .. } => {
            Err(format!("ANF violation at {path}: nested Call to '{name}'"))
        }
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => Ok(()),
        LoweredExpr::FieldAccess { expr, .. } => {
            verify_no_call(expr, &format!("{path}/FieldAccess"))
        }
        LoweredExpr::StringInterp(parts) => {
            for (i, part) in parts.iter().enumerate() {
                if let LoweredStringPart::Expr(e) = part {
                    verify_no_call(e, &format!("{path}/Interp[{i}]"))?;
                }
            }
            Ok(())
        }
        LoweredExpr::BinOp { left, right, .. } => {
            verify_no_call(left, &format!("{path}/BinOp.left"))?;
            verify_no_call(right, &format!("{path}/BinOp.right"))
        }
        LoweredExpr::UnaryOp { expr, .. } => {
            verify_no_call(expr, &format!("{path}/UnaryOp"))
        }
        LoweredExpr::IfElse { cond, then_, else_ } => {
            verify_no_call(cond, &format!("{path}/If.cond"))?;
            verify_branch(then_, &format!("{path}/If.then"))?;
            if let Some(e) = else_ {
                verify_branch(e, &format!("{path}/If.else"))?;
            }
            Ok(())
        }
        LoweredExpr::Match { expr, arms } => {
            verify_no_call(expr, &format!("{path}/Match.scrutinee"))?;
            for (i, arm) in arms.iter().enumerate() {
                if let Some(g) = &arm.guard {
                    verify_branch(g, &format!("{path}/Match.arm[{i}].guard"))?;
                }
                verify_branch(&arm.body, &format!("{path}/Match.arm[{i}].body"))?;
            }
            Ok(())
        }
        LoweredExpr::Lambda { body, .. } => {
            verify_branch(body, &format!("{path}/Lambda"))
        }
        LoweredExpr::List(items) => {
            for (i, item) in items.iter().enumerate() {
                verify_no_call(item, &format!("{path}/List[{i}]"))?;
            }
            Ok(())
        }
        LoweredExpr::Block(stmts) => {
            for (i, stmt) in stmts.iter().enumerate() {
                verify_stmt(stmt, &format!("{path}/Block/stmt[{i}]"))?;
            }
            Ok(())
        }
        LoweredExpr::Record { fields, .. } => {
            for (i, (_, e)) in fields.iter().enumerate() {
                verify_no_call(e, &format!("{path}/Record[{i}]"))?;
            }
            Ok(())
        }
        LoweredExpr::For { iterable, body, .. } => {
            verify_no_call(iterable, &format!("{path}/For.iter"))?;
            verify_branch(body, &format!("{path}/For.body"))
        }
        LoweredExpr::Return(fields) => {
            for (i, (_, e)) in fields.iter().enumerate() {
                verify_no_call(e, &format!("{path}/Return[{i}]"))?;
            }
            Ok(())
        }
        LoweredExpr::VariantConstruct { fields, .. } => {
            for (i, (_, e)) in fields.iter().enumerate() {
                verify_no_call(e, &format!("{path}/Variant[{i}]"))?;
            }
            Ok(())
        }
    }
}

/// Verify a branch expression. Calls are allowed at statement level within
/// Blocks, but not nested in bare expressions.
fn verify_branch(expr: &LoweredExpr, path: &str) -> Result<(), String> {
    match expr {
        LoweredExpr::Block(stmts) => {
            for (i, stmt) in stmts.iter().enumerate() {
                verify_stmt(stmt, &format!("{path}/stmt[{i}]"))?;
            }
            Ok(())
        }
        _ => verify_no_call(expr, path),
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn call(name: &str, args: Vec<(&str, LoweredExpr)>) -> LoweredExpr {
        LoweredExpr::Call {
            name: name.to_string(),
            args: args
                .into_iter()
                .map(|(k, v)| (Some(k.to_string()), v))
                .collect(),
        }
    }

    fn ident(name: &str) -> LoweredExpr {
        LoweredExpr::Ident(name.to_string())
    }

    fn int(n: i64) -> LoweredExpr {
        LoweredExpr::Literal(LoweredLiteral::Int(n))
    }

    #[test]
    fn anf_no_change_for_call_free() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "x".to_string(),
                    LoweredExpr::BinOp {
                        left: Box::new(ident("a")),
                        op: LoweredBinOp::Add,
                        right: Box::new(int(1)),
                    },
                ),
                LoweredStmt::Return(vec![("return".to_string(), ident("x"))]),
            ],
        };
        let result = anf_normalize(body.clone());
        assert_eq!(result.stmts.len(), 2);
        verify_anf(&result).unwrap();
    }

    #[test]
    fn anf_hoists_nested_call_in_binop() {
        // let x = f(a: 1) + g(b: 2)
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Let(
                "x".to_string(),
                LoweredExpr::BinOp {
                    left: Box::new(call("f", vec![("a", int(1))])),
                    op: LoweredBinOp::Add,
                    right: Box::new(call("g", vec![("b", int(2))])),
                },
            )],
        };
        let result = anf_normalize(body);
        // Should be: let __anf_0 = f(a: 1); let __anf_1 = g(b: 2); let x = __anf_0 + __anf_1
        assert_eq!(result.stmts.len(), 3);
        verify_anf(&result).unwrap();
    }

    #[test]
    fn anf_hoists_call_in_call_args() {
        // let x = f(a: g(b: 1))
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Let(
                "x".to_string(),
                call("f", vec![("a", call("g", vec![("b", int(1))]))]),
            )],
        };
        let result = anf_normalize(body);
        // Should be: let __anf_0 = g(b: 1); let x = f(a: __anf_0)
        assert_eq!(result.stmts.len(), 2);
        verify_anf(&result).unwrap();
    }

    #[test]
    fn anf_desugars_and_with_calls() {
        // let x = f() && g()
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Let(
                "x".to_string(),
                LoweredExpr::BinOp {
                    left: Box::new(call("f", vec![])),
                    op: LoweredBinOp::And,
                    right: Box::new(call("g", vec![])),
                },
            )],
        };
        let result = anf_normalize(body);
        verify_anf(&result).unwrap();
        // Should produce: let __anf_0 = f(); let x = if __anf_0 { <block with g()> } else { false }
        assert!(result.stmts.len() >= 2);
    }

    #[test]
    fn anf_preserves_and_without_calls_on_right() {
        // let x = f() && true
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Let(
                "x".to_string(),
                LoweredExpr::BinOp {
                    left: Box::new(call("f", vec![])),
                    op: LoweredBinOp::And,
                    right: Box::new(LoweredExpr::Literal(LoweredLiteral::Bool(true))),
                },
            )],
        };
        let result = anf_normalize(body);
        verify_anf(&result).unwrap();
        // Should hoist f() but keep the And operator (no desugaring needed)
        assert_eq!(result.stmts.len(), 2);
    }

    #[test]
    fn anf_hoists_call_in_field_access() {
        // let x = f(a: 1).field
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Let(
                "x".to_string(),
                LoweredExpr::FieldAccess {
                    expr: Box::new(call("f", vec![("a", int(1))])),
                    field: "field".to_string(),
                },
            )],
        };
        let result = anf_normalize(body);
        // Should be: let __anf_0 = f(a: 1); let x = __anf_0.field
        assert_eq!(result.stmts.len(), 2);
        verify_anf(&result).unwrap();
    }

    #[test]
    fn anf_normalizes_if_branches() {
        // if f() { g() } else { h() }
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(LoweredExpr::IfElse {
                cond: Box::new(call("f", vec![])),
                then_: Box::new(call("g", vec![])),
                else_: Some(Box::new(call("h", vec![]))),
            })],
        };
        let result = anf_normalize(body);
        verify_anf(&result).unwrap();
        // f() hoisted before the if; g() and h() stay in their branches
        assert!(result.stmts.len() >= 2);
    }

    #[test]
    fn anf_normalizes_lambda_body() {
        // x => f(x) + g(x)
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(LoweredExpr::Lambda {
                params: vec!["x".to_string()],
                body: Box::new(LoweredExpr::BinOp {
                    left: Box::new(call("f", vec![("x", ident("x"))])),
                    op: LoweredBinOp::Add,
                    right: Box::new(call("g", vec![("x", ident("x"))])),
                }),
            })],
        };
        let result = anf_normalize(body);
        verify_anf(&result).unwrap();
    }

    #[test]
    fn anf_hoists_call_in_return() {
        // return { value: f(x) }
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "value".to_string(),
                call("f", vec![("x", ident("x"))]),
            )])],
        };
        let result = anf_normalize(body);
        verify_anf(&result).unwrap();
        // Should be: let __anf_0 = f(x: x); return { value: __anf_0 }
        assert_eq!(result.stmts.len(), 2);
    }

    #[test]
    fn anf_verifier_catches_nested_call() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Let(
                "x".to_string(),
                LoweredExpr::BinOp {
                    left: Box::new(call("f", vec![])),
                    op: LoweredBinOp::Add,
                    right: Box::new(int(1)),
                },
            )],
        };
        assert!(verify_anf(&body).is_err());
    }

    #[test]
    fn anf_verifier_passes_clean_body() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("x".to_string(), call("f", vec![])),
                LoweredStmt::Return(vec![("return".to_string(), ident("x"))]),
            ],
        };
        verify_anf(&body).unwrap();
    }

    #[test]
    fn anf_no_nested_calls_after_lowering() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "x".to_string(),
                    call("f", vec![("a", call("g", vec![("b", call("h", vec![]))]))]),
                ),
                LoweredStmt::Expr(LoweredExpr::BinOp {
                    left: Box::new(call("i", vec![])),
                    op: LoweredBinOp::Add,
                    right: Box::new(call("j", vec![])),
                }),
                LoweredStmt::Return(vec![(
                    "value".to_string(),
                    LoweredExpr::FieldAccess {
                        expr: Box::new(call("k", vec![])),
                        field: "result".to_string(),
                    },
                )]),
            ],
        };
        let result = anf_normalize(body);
        verify_anf(&result).expect("all calls should be at statement level after ANF");
    }
}
