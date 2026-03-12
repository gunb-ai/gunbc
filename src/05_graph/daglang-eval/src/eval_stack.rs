//! Explicit-stack evaluator — a pure pipeline from fn bodies to values.
//!
//! # Pipeline stages
//!
//! ```text
//! [1. Build EvalContext]  →  [2. Verify ANF contract]  →  [3. Run machine]
//!
//!   sibling_fns             assert no nested Call         iterative main loop
//!   data_values             in any expression tree        heap continuation stack
//!   → fn_index                                           → Result<outputs, error>
//! ```
//!
//! # Architecture
//!
//! The machine has two evaluation layers with different properties:
//!
//! | Layer | Function | Can suspend? | Stack bound |
//! |-------|----------|-------------|-------------|
//! | `eval_expr` | Arithmetic, field access, match, string interp | Never | AST depth (syntactic) |
//! | `eval_body` | Sequencing, binding, fn calls | On sibling fn calls | Unbounded (heap) |
//!
//! `eval_body` never calls `eval_body`. Only the main loop does.
//! `eval_expr` never sees a `Call` (ANF contract). Native stack = O(AST depth).
//!
//! See DESIGN-eval-redesign.md for the full rationale.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use gunbc_ir::Value;

use crate::eval::{
    eval_binop, eval_match, field_access, value_to_string, value_truthy, EvalError,
};
use crate::expr::{
    LoweredBinOp, LoweredExpr, LoweredFnBody, LoweredLiteral, LoweredMatchArm, LoweredPattern,
    LoweredStmt, LoweredStringPart, LoweredUnaryOp,
};

// ── Limits ──────────────────────────────────────────────────────────────────

/// Maximum suspended continuations on the heap stack.
const MAX_STACK_DEPTH: usize = 100_000;

/// Maximum main-loop transitions (each eval_body invocation counts as one).
/// Catches infinite tail-call loops. Distinct from stack depth because tail
/// calls use O(1) stack but unbounded transitions.
const MAX_TRANSITIONS: usize = 10_000_000;

// ═══════════════════════════════════════════════════════════════════════════
// Stage 1: Build EvalContext
// ═══════════════════════════════════════════════════════════════════════════

pub type FnId = usize;

/// Immutable code store. Built once, shared by the entire evaluation.
pub struct EvalContext<'a> {
    fns: Vec<&'a LoweredFnBody>,
    fn_index: HashMap<&'a str, FnId>,
    data_values: &'a HashMap<String, Value>,
    sibling_fns: &'a HashMap<String, LoweredFnBody>,
}

impl<'a> EvalContext<'a> {
    /// Build the context and register the entry body.
    /// Returns (context, entry_fn_id).
    fn build(
        entry_body: &'a LoweredFnBody,
        sibling_fns: &'a HashMap<String, LoweredFnBody>,
        data_values: &'a HashMap<String, Value>,
    ) -> (Self, FnId) {
        let mut fns = Vec::with_capacity(sibling_fns.len() + 1);
        let mut fn_index = HashMap::with_capacity(sibling_fns.len() + 1);
        for (name, body) in sibling_fns {
            let id = fns.len();
            fns.push(body);
            fn_index.insert(name.as_str(), id);
        }
        let entry_id = fns.len();
        fns.push(entry_body);
        (EvalContext { fns, fn_index, data_values, sibling_fns }, entry_id)
    }

    fn is_sibling(&self, name: &str) -> Option<FnId> {
        self.fn_index.get(name).copied()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage 2: Verify ANF contract
// ═══════════════════════════════════════════════════════════════════════════

/// Assert no `LoweredExpr::Call` is nested inside another expression.
/// Calls must appear only at statement level: `Let(_, Call{..})` or `Expr(Call{..})`.
/// Fails immediately if the contract is violated — fail-closed during migration.
fn verify_anf_contract(ctx: &EvalContext) -> Result<(), String> {
    for (id, body) in ctx.fns.iter().enumerate() {
        for (i, stmt) in body.stmts.iter().enumerate() {
            verify_stmt_anf(stmt, id, i)?;
        }
    }
    Ok(())
}

fn verify_stmt_anf(stmt: &LoweredStmt, fn_id: usize, stmt_idx: usize) -> Result<(), String> {
    let loc = || format!("fn[{fn_id}]/stmt[{stmt_idx}]");
    match stmt {
        LoweredStmt::Let(_, LoweredExpr::Call { args, .. }) => {
            for (_, arg) in args {
                assert_no_nested_call(arg, &loc())?;
            }
            Ok(())
        }
        LoweredStmt::Let(_, expr) => assert_no_nested_call(expr, &loc()),
        LoweredStmt::Expr(LoweredExpr::Call { args, .. }) => {
            for (_, arg) in args {
                assert_no_nested_call(arg, &loc())?;
            }
            Ok(())
        }
        LoweredStmt::Expr(expr) => assert_no_nested_call_in_branch(expr, &loc()),
        LoweredStmt::Return(fields) => {
            for (_, expr) in fields {
                assert_no_nested_call(expr, &loc())?;
            }
            Ok(())
        }
    }
}

fn assert_no_nested_call(expr: &LoweredExpr, loc: &str) -> Result<(), String> {
    match expr {
        LoweredExpr::Call { name, .. } => Err(format!("ANF violation at {loc}: nested Call to '{name}'")),
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => Ok(()),
        LoweredExpr::FieldAccess { expr, .. } => assert_no_nested_call(expr, loc),
        LoweredExpr::BinOp { left, right, .. } => {
            assert_no_nested_call(left, loc)?;
            assert_no_nested_call(right, loc)
        }
        LoweredExpr::UnaryOp { expr, .. } => assert_no_nested_call(expr, loc),
        LoweredExpr::StringInterp(parts) => {
            for p in parts { if let LoweredStringPart::Expr(e) = p { assert_no_nested_call(e, loc)?; } }
            Ok(())
        }
        LoweredExpr::IfElse { cond, then_, else_ } => {
            assert_no_nested_call(cond, loc)?;
            assert_no_nested_call_in_branch(then_, loc)?;
            if let Some(e) = else_ { assert_no_nested_call_in_branch(e, loc)?; }
            Ok(())
        }
        LoweredExpr::Match { expr, arms } => {
            assert_no_nested_call(expr, loc)?;
            for a in arms {
                if let Some(g) = &a.guard { assert_no_nested_call_in_branch(g, loc)?; }
                assert_no_nested_call_in_branch(&a.body, loc)?;
            }
            Ok(())
        }
        LoweredExpr::Lambda { body, .. } => assert_no_nested_call_in_branch(body, loc),
        LoweredExpr::List(items) => { for i in items { assert_no_nested_call(i, loc)?; } Ok(()) }
        LoweredExpr::Block(stmts) => { for s in stmts { verify_stmt_anf(s, 0, 0)?; } Ok(()) }
        LoweredExpr::Record { fields, .. } | LoweredExpr::VariantConstruct { fields, .. } => {
            for (_, e) in fields { assert_no_nested_call(e, loc)?; } Ok(())
        }
        LoweredExpr::For { iterable, body, .. } => {
            assert_no_nested_call(iterable, loc)?;
            assert_no_nested_call_in_branch(body, loc)
        }
        LoweredExpr::Return(fields) => { for (_, e) in fields { assert_no_nested_call(e, loc)?; } Ok(()) }
    }
}

fn assert_no_nested_call_in_branch(expr: &LoweredExpr, loc: &str) -> Result<(), String> {
    match expr {
        LoweredExpr::Block(stmts) => { for s in stmts { verify_stmt_anf(s, 0, 0)?; } Ok(()) }
        _ => assert_no_nested_call(expr, loc),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage 3: Run machine
// ═══════════════════════════════════════════════════════════════════════════

// ── Machine types ───────────────────────────────────────────────────────────

/// How to extract the caller's value from a callee's output map.
/// Decided at call-site construction time, not at runtime.
#[derive(Debug, Clone)]
enum Projection {
    /// Extract the "return" field. Falls back to single "value" field for
    /// compatibility with `return expr` (which lowers to `Return([("value", expr)])`).
    /// This fallback will be removed once the return convention is standardized.
    ReturnField,
    /// Use the entire output map as Value::Map.
    WholeMap,
}

impl Projection {
    fn extract(&self, outputs: &HashMap<String, Value>) -> Value {
        match self {
            Projection::ReturnField => {
                if let Some(v) = outputs.get("return") { return v.clone(); }
                if outputs.len() == 1 {
                    if let Some(v) = outputs.get("value") { return v.clone(); }
                }
                if outputs.is_empty() { return Value::Unit; }
                Value::Map(outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            }
            Projection::WholeMap => {
                Value::Map(outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            }
        }
    }
}

/// Saved state for resuming after a callee returns.
#[derive(Debug, Clone)]
struct Continuation {
    fn_id: FnId,
    /// Absolute index into `ctx.fns[fn_id].stmts` — the next statement
    /// to execute after the call result is bound.
    pc: usize,
    binding: Option<String>,
    projection: Projection,
    env: Env,
}

/// What eval_body decided to do.
enum Step {
    /// Body completed. Contains the output map.
    Return(HashMap<String, Value>),
    /// Needs a sibling fn call. `cont: None` = true tail call (identity
    /// continuation — no binding, no remaining work, no projection).
    Call {
        callee: FnId,
        inputs: HashMap<String, Value>,
        cont: Option<Continuation>,
    },
    /// Unrecoverable error.
    Error(String),
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Evaluate a fn body using the explicit-stack evaluator.
///
/// Pipeline: build context → verify ANF contract → run machine.
pub fn evaluate_stack(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, EvalError> {
    // Stage 1: build context
    let (ctx, entry_fn_id) = EvalContext::build(body, sibling_fns, data_values);

    // Stage 2: verify ANF contract (debug builds only for performance)
    debug_assert!(
        verify_anf_contract(&ctx).is_ok(),
        "ANF contract violated: {}",
        verify_anf_contract(&ctx).unwrap_err()
    );

    // Stage 3: run machine
    run_machine(entry_fn_id, inputs, &ctx)
}

// ── Main loop ───────────────────────────────────────────────────────────────

fn run_machine(
    entry_fn_id: FnId,
    inputs: &HashMap<String, Value>,
    ctx: &EvalContext,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut stack: Vec<Continuation> = Vec::new();
    let mut fn_id = entry_fn_id;
    let mut pc: usize = 0;
    let mut env = Env::from_inputs(inputs);
    seed_data(&mut env, ctx.data_values);
    let mut transitions: usize = 0;

    loop {
        transitions += 1;
        if transitions > MAX_TRANSITIONS {
            return Err(EvalError::new(format!(
                "transition budget ({MAX_TRANSITIONS}) exceeded"
            )));
        }

        match eval_body(fn_id, pc, &mut env, ctx) {
            Step::Return(result) => {
                // Unwind: pop continuations until one has remaining work.
                let mut result = result;
                loop {
                    match stack.pop() {
                        None => return Ok(result),
                        Some(cont) => {
                            let value = cont.projection.extract(&result);
                            let stmts = &ctx.fns[cont.fn_id].stmts;
                            if cont.pc >= stmts.len() && cont.binding.is_none() {
                                result = wrap_as_result(value);
                            } else {
                                env = cont.env;
                                if let Some(ref name) = cont.binding {
                                    bind_let_result(&mut env, name.clone(), &value);
                                }
                                fn_id = cont.fn_id;
                                pc = cont.pc;
                                break;
                            }
                        }
                    }
                }
            }
            Step::Call { callee, inputs, cont } => {
                if let Some(cont) = cont {
                    if stack.len() >= MAX_STACK_DEPTH {
                        return Err(EvalError::new(format!(
                            "max stack depth ({MAX_STACK_DEPTH}) exceeded"
                        )));
                    }
                    stack.push(cont);
                }
                env = Env::from_inputs(&inputs);
                seed_data(&mut env, ctx.data_values);
                fn_id = callee;
                pc = 0;
            }
            Step::Error(msg) => return Err(EvalError::new(msg)),
        }
    }
}

// ── Body evaluation ─────────────────────────────────────────────────────────
//
// Owns the full statement loop over `ctx.fns[fn_id].stmts`.
// `start_pc` is an absolute index into that array.

fn eval_body(
    fn_id: FnId,
    start_pc: usize,
    env: &mut Env,
    ctx: &EvalContext,
) -> Step {
    let stmts = &ctx.fns[fn_id].stmts;

    for i in start_pc..stmts.len() {
        let is_last = i == stmts.len() - 1;

        match &stmts[i] {
            LoweredStmt::Let(name, expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(callee_id) = ctx.is_sibling(callee) {
                        match eval_call_args(args, env, ctx) {
                            Ok(fn_inputs) => {
                                return Step::Call {
                                    callee: callee_id,
                                    inputs: fn_inputs,
                                    cont: Some(Continuation {
                                        fn_id,
                                        pc: i + 1,
                                        binding: Some(name.clone()),
                                        projection: Projection::ReturnField,
                                        env: env.clone(),
                                    }),
                                };
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling(callee, args, env, ctx) {
                            Ok(value) => bind_let_result(env, name.clone(), &value),
                            Err(e) => return step_from_eval_error(e),
                        }
                    }
                } else {
                    match eval_expr(expr, env, ctx) {
                        Ok(value) => bind_let_result(env, name.clone(), &value),
                        Err(e) => return step_from_eval_error(e),
                    }
                }
            }

            LoweredStmt::Expr(expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(callee_id) = ctx.is_sibling(callee) {
                        match eval_call_args(args, env, ctx) {
                            Ok(fn_inputs) => {
                                return Step::Call {
                                    callee: callee_id,
                                    inputs: fn_inputs,
                                    cont: Some(Continuation {
                                        fn_id,
                                        pc: i + 1,
                                        binding: None,
                                        projection: Projection::ReturnField,
                                        env: env.clone(),
                                    }),
                                };
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling(callee, args, env, ctx) {
                            Ok(value) => {
                                if is_last {
                                    return Step::Return(wrap_as_result(value));
                                }
                            }
                            Err(e) => return step_from_eval_error(e),
                        }
                    }
                } else if is_last {
                    return eval_trailing_expr(expr, env, ctx);
                } else {
                    match eval_expr(expr, env, ctx) {
                        Ok(_) => {}
                        Err(e) => return step_from_eval_error(e),
                    }
                }
            }

            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, fexpr) in fields {
                    match eval_expr(fexpr, env, ctx) {
                        Ok(value) => { result.insert(name.clone(), value); }
                        Err(e) => return step_from_eval_error(e),
                    }
                }
                return Step::Return(result);
            }
        }
    }

    Step::Return([("return".to_string(), Value::Unit)].into_iter().collect())
}

// ── Trailing expression ─────────────────────────────────────────────────────
//
// Drills through if/match/block to find the terminal form (Call or value)
// at the tail of the last Expr statement.

fn eval_trailing_expr(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Step {
    match expr {
        LoweredExpr::IfElse { cond, then_, else_ } => {
            match eval_expr(cond, env, ctx) {
                Ok(c) => {
                    if value_truthy(&c) { eval_trailing_expr(then_, env, ctx) }
                    else if let Some(e) = else_ { eval_trailing_expr(e, env, ctx) }
                    else { Step::Return(wrap_as_result(Value::Unit)) }
                }
                Err(e) => step_from_eval_error(e),
            }
        }
        LoweredExpr::Match { expr: scrutinee, arms } => {
            match eval_expr(scrutinee, env, ctx) {
                Ok(val) => eval_trailing_match(&val, arms, env, ctx),
                Err(e) => step_from_eval_error(e),
            }
        }
        LoweredExpr::Block(stmts) => {
            let mut child_env = env.child();
            eval_block_as_body(stmts, &mut child_env, ctx)
        }
        LoweredExpr::Return(fields) => {
            let mut result = HashMap::new();
            for (name, fexpr) in fields {
                match eval_expr(fexpr, env, ctx) {
                    Ok(value) => { result.insert(name.clone(), value); }
                    Err(e) => return step_from_eval_error(e),
                }
            }
            Step::Return(result)
        }
        _ => {
            match eval_expr(expr, env, ctx) {
                Ok(value) => Step::Return(wrap_as_result(value)),
                Err(e) => step_from_eval_error(e),
            }
        }
    }
}

fn eval_trailing_match(
    scrutinee: &Value, arms: &[LoweredMatchArm], env: &Env, ctx: &EvalContext,
) -> Step {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, scrutinee) {
            let mut arm_env = env.child();
            for (name, val) in bindings { arm_env.bind(name, val); }
            if let Some(guard) = &arm.guard {
                match eval_expr(guard, &arm_env, ctx) {
                    Ok(g) if value_truthy(&g) => {}
                    Ok(_) => continue,
                    Err(e) => return step_from_eval_error(e),
                }
            }
            return eval_trailing_expr(&arm.body, &arm_env, ctx);
        }
    }
    Step::Error(format!("no matching arm for: {scrutinee:?}"))
}

/// Evaluate a Block's statements within a trailing expression.
/// Handles calls inside blocks by falling back to the old evaluator.
fn eval_block_as_body(stmts: &[LoweredStmt], env: &mut Env, ctx: &EvalContext) -> Step {
    let last_idx = stmts.len().saturating_sub(1);
    for (i, stmt) in stmts.iter().enumerate() {
        let is_last = i == last_idx && !stmts.is_empty();
        match stmt {
            LoweredStmt::Let(name, expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(callee_id) = ctx.is_sibling(callee) {
                        match eval_call_args(args, env, ctx) {
                            Ok(fn_inputs) => {
                                match eval_sibling_recursive(callee_id, &fn_inputs, ctx) {
                                    Ok(value) => bind_let_result(env, name.clone(), &value),
                                    Err(msg) => return Step::Error(msg),
                                }
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling(callee, args, env, ctx) {
                            Ok(value) => bind_let_result(env, name.clone(), &value),
                            Err(e) => return step_from_eval_error(e),
                        }
                    }
                } else {
                    match eval_expr(expr, env, ctx) {
                        Ok(value) => bind_let_result(env, name.clone(), &value),
                        Err(e) => return step_from_eval_error(e),
                    }
                }
            }
            LoweredStmt::Expr(expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(callee_id) = ctx.is_sibling(callee) {
                        match eval_call_args(args, env, ctx) {
                            Ok(fn_inputs) => {
                                match eval_sibling_recursive(callee_id, &fn_inputs, ctx) {
                                    Ok(value) => {
                                        if is_last {
                                            return Step::Return(wrap_as_result(value));
                                        }
                                    }
                                    Err(msg) => return Step::Error(msg),
                                }
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling(callee, args, env, ctx) {
                            Ok(value) => { if is_last { return Step::Return(wrap_as_result(value)); } }
                            Err(e) => return step_from_eval_error(e),
                        }
                    }
                } else if is_last {
                    return eval_trailing_expr(expr, env, ctx);
                } else {
                    match eval_expr(expr, env, ctx) {
                        Ok(_) => {}
                        Err(e) => return step_from_eval_error(e),
                    }
                }
            }
            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, fexpr) in fields {
                    match eval_expr(fexpr, env, ctx) {
                        Ok(value) => { result.insert(name.clone(), value); }
                        Err(e) => return step_from_eval_error(e),
                    }
                }
                return Step::Return(result);
            }
        }
    }
    Step::Return([("return".to_string(), Value::Unit)].into_iter().collect())
}

// ── Pure expression evaluation ──────────────────────────────────────────────
//
// Total function over call-free expression trees. After ANF normalization,
// eval_expr never encounters a Call node (except via the non-sibling bridge
// for intrinsics/builtins, which are evaluated inline).

fn eval_expr(
    expr: &LoweredExpr, env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    match expr {
        LoweredExpr::Literal(lit) => Ok(eval_literal(lit)),
        LoweredExpr::Ident(name) => {
            if name == "None" || name == "null" { return Ok(Value::Unit); }
            if let Some(val) = env.get(name) { return Ok(val.clone()); }
            if name.chars().next().unwrap_or('a').is_uppercase() { return Ok(Value::Str(name.clone())); }
            Err(EvalError::new(format!("unbound variable: {name}")))
        }
        LoweredExpr::FieldAccess { expr, field } => {
            let base = eval_expr(expr, env, ctx)?;
            field_access(&base, field)
        }
        LoweredExpr::StringInterp(parts) => {
            let mut s = String::new();
            for p in parts {
                match p {
                    LoweredStringPart::Literal(lit) => s.push_str(lit),
                    LoweredStringPart::Expr(e) => s.push_str(&value_to_string(&eval_expr(e, env, ctx)?)),
                }
            }
            Ok(Value::Str(s))
        }
        LoweredExpr::BinOp { left, op, right } => {
            let lhs = eval_expr(left, env, ctx)?;
            match op {
                LoweredBinOp::And => {
                    if !value_truthy(&lhs) { return Ok(Value::Bool(false)); }
                    Ok(Value::Bool(value_truthy(&eval_expr(right, env, ctx)?)))
                }
                LoweredBinOp::Or => {
                    if value_truthy(&lhs) { return Ok(Value::Bool(true)); }
                    Ok(Value::Bool(value_truthy(&eval_expr(right, env, ctx)?)))
                }
                LoweredBinOp::NullCoalesce => {
                    if !matches!(lhs, Value::Unit | Value::Skipped) { Ok(lhs) }
                    else { eval_expr(right, env, ctx) }
                }
                _ => {
                    let rhs = eval_expr(right, env, ctx)?;
                    if *op == LoweredBinOp::Add {
                        match (lhs, rhs) {
                            (Value::List(mut a), Value::List(b)) => { a.extend(b); return Ok(Value::List(a)); }
                            (Value::Str(mut a), Value::Str(b)) => { a.push_str(&b); return Ok(Value::Str(a)); }
                            (Value::Str(mut a), Value::Enum { variant, .. }) => { a.push_str(&variant); return Ok(Value::Str(a)); }
                            (Value::Enum { variant, .. }, Value::Str(b)) => { return Ok(Value::Str(format!("{variant}{b}"))); }
                            (l, r) => return eval_binop(&l, *op, &r),
                        }
                    }
                    eval_binop(&lhs, *op, &rhs)
                }
            }
        }
        LoweredExpr::UnaryOp { op, expr } => {
            let val = eval_expr(expr, env, ctx)?;
            match op {
                LoweredUnaryOp::Not => Ok(Value::Bool(!value_truthy(&val))),
                LoweredUnaryOp::Neg => match val {
                    Value::Int(i) => Ok(Value::Int(-i)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(EvalError::new(format!("cannot negate {:?}", val))),
                },
            }
        }
        LoweredExpr::IfElse { cond, then_, else_ } => {
            let c = eval_expr(cond, env, ctx)?;
            if value_truthy(&c) { eval_expr(then_, env, ctx) }
            else if let Some(e) = else_ { eval_expr(e, env, ctx) }
            else { Ok(Value::Unit) }
        }
        LoweredExpr::Match { expr: scrutinee, arms } => {
            let val = eval_expr(scrutinee, env, ctx)?;
            let bindings: HashMap<String, Value> = env.bindings.as_ref().clone();
            eval_match(&val, arms, &bindings, ctx.sibling_fns)
        }
        LoweredExpr::VariantConstruct { tag, fields } => {
            if fields.is_empty() {
                Ok(Value::Enum { ty: String::new(), variant: tag.clone() })
            } else {
                let mut map = BTreeMap::new();
                map.insert("_variant".to_string(), Value::Str(tag.clone()));
                for (k, v) in fields { map.insert(k.clone(), eval_expr(v, env, ctx)?); }
                Ok(Value::Map(map))
            }
        }
        LoweredExpr::Call { name, args } => {
            eval_non_sibling(name, args, env, ctx)
        }
        LoweredExpr::Lambda { .. } => Err(EvalError::new("lambda cannot be evaluated standalone")),
        LoweredExpr::List(items) => {
            let v: Result<Vec<_>, _> = items.iter().map(|i| eval_expr(i, env, ctx)).collect();
            Ok(Value::List(v?))
        }
        LoweredExpr::Block(stmts) => {
            let mut child = env.child();
            let outputs = eval_block_stmts(stmts, &mut child, ctx)?;
            if outputs.len() == 1 { if let Some(v) = outputs.get("return") { return Ok(v.clone()); } }
            Ok(Value::Map(outputs.into_iter().collect()))
        }
        LoweredExpr::Record { fields, .. } => {
            let mut map = BTreeMap::new();
            for (k, v) in fields { map.insert(k.clone(), eval_expr(v, env, ctx)?); }
            Ok(Value::Map(map))
        }
        LoweredExpr::For { binding, iterable, body } => {
            let items = eval_expr(iterable, env, ctx)?;
            match items {
                Value::List(list) => {
                    let mut results = Vec::with_capacity(list.len());
                    for item in &list {
                        let mut iter_env = env.child();
                        iter_env.bind(binding.clone(), item.clone());
                        results.push(eval_expr(body, &iter_env, ctx)?);
                    }
                    Ok(Value::List(results))
                }
                _ => Err(EvalError::new(format!("for requires list, got {:?}", items))),
            }
        }
        LoweredExpr::Return(fields) => {
            let mut result = HashMap::new();
            for (n, e) in fields { result.insert(n.clone(), eval_expr(e, env, ctx)?); }
            Err(EvalError::early_return(result))
        }
    }
}

fn eval_block_stmts(
    stmts: &[LoweredStmt], env: &mut Env, ctx: &EvalContext,
) -> Result<HashMap<String, Value>, EvalError> {
    let last = stmts.last();
    for stmt in stmts {
        let is_last = last.is_some_and(|l| std::ptr::eq(stmt, l));
        match stmt {
            LoweredStmt::Let(name, expr) => {
                let value = eval_expr(expr, env, ctx)?;
                bind_let_result(env, name.clone(), &value);
            }
            LoweredStmt::Expr(expr) => {
                let value = eval_expr(expr, env, ctx)?;
                if is_last {
                    if let Value::Map(map) = &value {
                        return Ok(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                    }
                    return Ok([("return".to_string(), value)].into_iter().collect());
                }
            }
            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, fexpr) in fields {
                    result.insert(name.clone(), eval_expr(fexpr, env, ctx)?);
                }
                return Err(EvalError::early_return(result));
            }
        }
    }
    Ok([("return".to_string(), Value::Unit)].into_iter().collect())
}

// ── Call helpers ─────────────────────────────────────────────────────────────

fn eval_call_args(
    args: &[(Option<String>, LoweredExpr)], env: &Env, ctx: &EvalContext,
) -> Result<HashMap<String, Value>, String> {
    let mut inputs = HashMap::new();
    for (param, arg_expr) in args {
        let value = eval_expr(arg_expr, env, ctx).map_err(|e| e.message)?;
        if let Some(name) = param { inputs.insert(name.clone(), value); }
    }
    Ok(inputs)
}

fn eval_sibling_recursive(
    callee_id: FnId, inputs: &HashMap<String, Value>, ctx: &EvalContext,
) -> Result<Value, String> {
    let body = ctx.fns[callee_id];
    let outputs = crate::eval::evaluate_fn_body_old(
        body, inputs, ctx.sibling_fns, ctx.data_values,
    ).map_err(|e| e.message)?;
    Ok(Projection::ReturnField.extract(&outputs))
}

fn eval_non_sibling(
    name: &str, args: &[(Option<String>, LoweredExpr)], env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    let env_bindings: HashMap<String, Value> = env.bindings.as_ref().clone();
    crate::eval::eval_non_sibling_call(
        name, args, &env_bindings, ctx.sibling_fns, ctx.data_values,
    )
}

// ── Binding ─────────────────────────────────────────────────────────────────

/// Central let-binding helper. Used for both normal statement processing and
/// continuation resume. Flattens Map/Json fields into `name__field` entries
/// so the `__` convention works for local let bindings.
fn bind_let_result(env: &mut Env, name: String, value: &Value) {
    match value {
        Value::Map(fields) => {
            for (fname, fval) in fields {
                env.bind(format!("{name}__{fname}"), fval.clone());
            }
        }
        Value::Json(serde_json::Value::Object(map)) => {
            for (fname, fval) in map {
                env.bind(format!("{name}__{fname}"), Value::Json(fval.clone()));
            }
        }
        _ => {}
    }
    env.bind(name, value.clone());
}

// ── Pattern matching ────────────────────────────────────────────────────────

fn match_pattern(pattern: &LoweredPattern, value: &Value) -> Option<Vec<(String, Value)>> {
    match pattern {
        LoweredPattern::Wildcard => Some(vec![]),
        LoweredPattern::Ident(name) => Some(vec![(name.clone(), value.clone())]),
        LoweredPattern::Literal(lit) => {
            let lv = eval_literal(lit);
            if values_match(&lv, value) { Some(vec![]) } else { None }
        }
        LoweredPattern::Variant(tag, fields) => {
            match value {
                Value::Enum { variant, .. } if variant == tag && fields.is_empty() => Some(vec![]),
                Value::Str(s) if s == tag && fields.is_empty() => Some(vec![]),
                Value::Map(map) => {
                    if let Some(Value::Str(v)) = map.get("_variant") {
                        if v == tag {
                            let mut bindings = Vec::new();
                            for (fname, fpat) in fields {
                                match map.get(fname).and_then(|fval| match_pattern(fpat, fval)) {
                                    Some(mut fb) => bindings.append(&mut fb),
                                    None => return None,
                                }
                            }
                            return Some(bindings);
                        }
                    }
                    None
                }
                _ => None,
            }
        }
    }
}

fn values_match(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Unit, Value::Unit) | (Value::Unit, Value::Skipped) | (Value::Skipped, Value::Unit) => true,
        _ => a == b,
    }
}

// ── Small helpers ───────────────────────────────────────────────────────────

fn eval_literal(lit: &LoweredLiteral) -> Value {
    match lit {
        LoweredLiteral::Int(i) => Value::Int(*i),
        LoweredLiteral::Bool(b) => Value::Bool(*b),
        LoweredLiteral::String(s) => Value::Str(s.clone()),
        LoweredLiteral::None => Value::Unit,
    }
}

fn wrap_as_result(value: Value) -> HashMap<String, Value> {
    if let Value::Map(map) = &value {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    } else {
        [("return".to_string(), value)].into_iter().collect()
    }
}

fn seed_data(env: &mut Env, data_values: &HashMap<String, Value>) {
    for (name, val) in data_values {
        if env.get(name).is_none() { env.bind(name.clone(), val.clone()); }
    }
}

fn step_from_eval_error(e: EvalError) -> Step {
    if let Some(ret) = e.early_return { Step::Return(ret) }
    else { Step::Error(e.message) }
}

// ── Environment ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Env {
    bindings: Rc<HashMap<String, Value>>,
}

impl Env {
    fn from_inputs(inputs: &HashMap<String, Value>) -> Self {
        Self { bindings: Rc::new(inputs.clone()) }
    }
    fn bind(&mut self, name: String, value: Value) {
        Rc::make_mut(&mut self.bindings).insert(name, value);
    }
    fn get(&self, name: &str) -> Option<&Value> { self.bindings.get(name) }
    fn child(&self) -> Self { Self { bindings: Rc::clone(&self.bindings) } }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{LoweredFnBody, LoweredStmt, LoweredExpr, LoweredLiteral, LoweredBinOp};

    fn call(name: &str, args: Vec<(&str, LoweredExpr)>) -> LoweredExpr {
        LoweredExpr::Call {
            name: name.to_string(),
            args: args.into_iter().map(|(k, v)| (Some(k.to_string()), v)).collect(),
        }
    }
    fn ident(n: &str) -> LoweredExpr { LoweredExpr::Ident(n.to_string()) }
    fn int(n: i64) -> LoweredExpr { LoweredExpr::Literal(LoweredLiteral::Int(n)) }

    #[test]
    fn simple_fn() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![("return".to_string(), int(42))])],
        };
        let r = evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(r["return"], Value::Int(42));
    }

    #[test]
    fn sibling_call_with_projection() {
        let inner = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![("value".to_string(), int(99))])],
        };
        let outer = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("r".to_string(), call("inner", vec![])),
                LoweredStmt::Return(vec![("return".to_string(), ident("r"))]),
            ],
        };
        let mut sibs = HashMap::new();
        sibs.insert("inner".to_string(), inner);
        sibs.insert("outer".to_string(), outer.clone());
        let r = evaluate_stack(&outer, &HashMap::new(), &sibs, &HashMap::new()).unwrap();
        assert_eq!(r["return"], Value::Int(99));
    }

    #[test]
    fn deep_mutual_recursion_40k() {
        let is_even = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")), op: LoweredBinOp::Eq, right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![
                        ("return".to_string(), LoweredExpr::Literal(LoweredLiteral::Bool(true))),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Expr(call("is_odd", vec![
                    ("n", LoweredExpr::BinOp {
                        left: Box::new(ident("n")), op: LoweredBinOp::Sub, right: Box::new(int(1)),
                    }),
                ])),
            ],
        };
        let is_odd = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")), op: LoweredBinOp::Eq, right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![
                        ("return".to_string(), LoweredExpr::Literal(LoweredLiteral::Bool(false))),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Expr(call("is_even", vec![
                    ("n", LoweredExpr::BinOp {
                        left: Box::new(ident("n")), op: LoweredBinOp::Sub, right: Box::new(int(1)),
                    }),
                ])),
            ],
        };
        let mut sibs = HashMap::new();
        sibs.insert("is_even".to_string(), is_even.clone());
        sibs.insert("is_odd".to_string(), is_odd);

        let mut inp = HashMap::new();
        inp.insert("n".to_string(), Value::Int(40_000));
        assert_eq!(evaluate_stack(&is_even, &inp, &sibs, &HashMap::new()).unwrap()["return"], Value::Bool(true));

        inp.insert("n".to_string(), Value::Int(40_001));
        assert_eq!(evaluate_stack(&is_even, &inp, &sibs, &HashMap::new()).unwrap()["return"], Value::Bool(false));
    }

    #[test]
    fn value_normalization() {
        let inner = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![("value".to_string(), int(42))])],
        };
        let wrapper = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(call("inner", vec![]))],
        };
        let mut sibs = HashMap::new();
        sibs.insert("inner".to_string(), inner);
        sibs.insert("wrapper".to_string(), wrapper.clone());
        let r = evaluate_stack(&wrapper, &HashMap::new(), &sibs, &HashMap::new()).unwrap();
        assert_eq!(r.get("return"), Some(&Value::Int(42)));
        assert!(!r.contains_key("value"));
    }

    #[test]
    fn builtin_call() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("result".to_string(), LoweredExpr::Call {
                    name: "skip_horizontal_ws".to_string(),
                    args: vec![
                        (Some("s".to_string()), ident("s")),
                        (Some("start".to_string()), ident("start")),
                    ],
                }),
                LoweredStmt::Return(vec![("return".to_string(), ident("result"))]),
            ],
        };
        let mut inp = HashMap::new();
        inp.insert("s".to_string(), Value::Str("   hello".to_string()));
        inp.insert("start".to_string(), Value::Int(0));
        assert_eq!(evaluate_stack(&body, &inp, &HashMap::new(), &HashMap::new()).unwrap()["return"], Value::Int(3));
    }

    #[test]
    fn sibling_then_builtin() {
        let make_state = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![
                ("source".to_string(), ident("source")),
                ("start".to_string(), int(0)),
            ])],
        };
        let outer = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("state".to_string(), call("make_state", vec![("source", ident("source"))])),
                LoweredStmt::Let("result".to_string(), LoweredExpr::Call {
                    name: "skip_horizontal_ws".to_string(),
                    args: vec![
                        (Some("s".to_string()), LoweredExpr::FieldAccess {
                            expr: Box::new(ident("state")), field: "source".to_string(),
                        }),
                        (Some("start".to_string()), LoweredExpr::FieldAccess {
                            expr: Box::new(ident("state")), field: "start".to_string(),
                        }),
                    ],
                }),
                LoweredStmt::Return(vec![("return".to_string(), ident("result"))]),
            ],
        };
        let mut sibs = HashMap::new();
        sibs.insert("make_state".to_string(), make_state);
        sibs.insert("outer".to_string(), outer.clone());
        let mut inp = HashMap::new();
        inp.insert("source".to_string(), Value::Str("   hello".to_string()));
        assert_eq!(evaluate_stack(&outer, &inp, &sibs, &HashMap::new()).unwrap()["return"], Value::Int(3));
    }

    #[test]
    fn anf_verifier_catches_nested_call() {
        let bad_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Let("x".to_string(), LoweredExpr::BinOp {
                left: Box::new(call("f", vec![])),
                op: LoweredBinOp::Add,
                right: Box::new(int(1)),
            })],
        };
        let sibs = HashMap::new();
        let data = HashMap::new();
        let (ctx, _) = EvalContext::build(&bad_body, &sibs, &data);
        assert!(verify_anf_contract(&ctx).is_err());
    }
}
