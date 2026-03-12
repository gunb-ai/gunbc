//! Explicit-stack evaluator — a pure pipeline from fn bodies to values.
//!
//! # Pipeline (read top-down)
//!
//! ```text
//! evaluate_stack(body, inputs, sibling_fns, data_values)
//!   │
//!   ├─ build_context        (body, sibs, data)  →  (EvalContext, entry_fn_id)
//!   ├─ verify_anf_contract  (ctx)                →  Ok(()) or Err
//!   └─ run_machine          (entry, inputs, ctx) →  Result<outputs, error>
//!        │
//!        │  loop {
//!        ├─── eval_body      (fn_id, pc, env, ctx) →  Step
//!        │      │
//!        │      │  for stmt in stmts[pc..] {
//!        │      ├─── eval_stmt  (stmt, position, env, ctx, fn_id) →  StmtOutcome
//!        │      │      ├─ classify_call  — is this a sibling call, builtin, or not a call?
//!        │      │      ├─ eval_expr      — pure expression → Value (never suspends)
//!        │      │      └─ bind_let_result — centralized let-binding with map flattening
//!        │      │
//!        │      └─── eval_trailing  (expr, env, ctx) →  Step
//!        │             drills through if/match/block at tail position
//!        │
//!        └─── handle Step: Return pops stack, Call pushes/replaces
//!   }
//! ```
//!
//! Every box is a pure function. `eval_body` never calls `eval_body`.
//! Only `run_machine` does. `eval_expr` never sees a `Call` (ANF contract).

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use gunbc_ir::Value;

use crate::eval::{
    eval_binop, eval_literal, eval_match, field_access, match_pattern, value_to_string,
    value_truthy, EvalError,
};
use crate::expr::{
    LoweredBinOp, LoweredExpr, LoweredFnBody, LoweredLiteral, LoweredMatchArm, LoweredPattern,
    LoweredStmt, LoweredStringPart, LoweredUnaryOp,
};

// ── Limits ──────────────────────────────────────────────────────────────────

const MAX_STACK_DEPTH: usize = 100_000;
const MAX_TRANSITIONS: usize = 10_000_000;

// ═══════════════════════════════════════════════════════════════════════════
// Public entry point
// ═══════════════════════════════════════════════════════════════════════════

/// Evaluate a fn body. Pipeline: build context → verify contract → run machine.
pub fn evaluate_stack(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, EvalError> {
    let (ctx, entry) = build_context(body, sibling_fns, data_values);

    debug_assert!(
        verify_anf_contract(&ctx).is_ok(),
        "ANF violation: {}", verify_anf_contract(&ctx).unwrap_err()
    );

    run_machine(entry, inputs, &ctx)
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage 1 — Build context
// ═══════════════════════════════════════════════════════════════════════════

pub type FnId = usize;

/// Immutable code store built once, referenced by the entire evaluation.
pub struct EvalContext<'a> {
    fns: Vec<&'a LoweredFnBody>,
    fn_index: HashMap<&'a str, FnId>,
    data_values: &'a HashMap<String, Value>,
    sibling_fns: &'a HashMap<String, LoweredFnBody>,
}

/// Pure: (entry_body, sibling_fns, data_values) → (EvalContext, entry_fn_id)
fn build_context<'a>(
    entry_body: &'a LoweredFnBody,
    sibling_fns: &'a HashMap<String, LoweredFnBody>,
    data_values: &'a HashMap<String, Value>,
) -> (EvalContext<'a>, FnId) {
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

// ═══════════════════════════════════════════════════════════════════════════
// Stage 2 — Verify ANF contract
// ═══════════════════════════════════════════════════════════════════════════

/// Pure: EvalContext → Ok(()) or Err(location).
/// Asserts no Call nested inside another expression — only at stmt level.
fn verify_anf_contract(ctx: &EvalContext) -> Result<(), String> {
    for (id, body) in ctx.fns.iter().enumerate() {
        for (i, stmt) in body.stmts.iter().enumerate() {
            check_stmt_anf(stmt, &format!("fn[{id}]/stmt[{i}]"))?;
        }
    }
    Ok(())
}

fn check_stmt_anf(stmt: &LoweredStmt, loc: &str) -> Result<(), String> {
    match stmt {
        LoweredStmt::Let(_, LoweredExpr::Call { args, .. })
        | LoweredStmt::Expr(LoweredExpr::Call { args, .. }) => {
            for (_, a) in args { no_call(a, loc)?; }
            Ok(())
        }
        LoweredStmt::Let(_, e) => no_call(e, loc),
        LoweredStmt::Expr(e)   => no_call_in_branch(e, loc),
        LoweredStmt::Return(fields) => {
            for (_, e) in fields { no_call(e, loc)?; }
            Ok(())
        }
    }
}

fn no_call(expr: &LoweredExpr, loc: &str) -> Result<(), String> {
    match expr {
        LoweredExpr::Call { name, .. } =>
            Err(format!("ANF violation at {loc}: nested Call to '{name}'")),
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => Ok(()),
        LoweredExpr::FieldAccess { expr, .. }
        | LoweredExpr::UnaryOp { expr, .. } => no_call(expr, loc),
        LoweredExpr::BinOp { left, right, .. } => { no_call(left, loc)?; no_call(right, loc) }
        LoweredExpr::StringInterp(ps) => {
            for p in ps { if let LoweredStringPart::Expr(e) = p { no_call(e, loc)?; } }
            Ok(())
        }
        LoweredExpr::IfElse { cond, then_, else_ } => {
            no_call(cond, loc)?;
            no_call_in_branch(then_, loc)?;
            if let Some(e) = else_ { no_call_in_branch(e, loc)?; }
            Ok(())
        }
        LoweredExpr::Match { expr, arms } => {
            no_call(expr, loc)?;
            for a in arms {
                if let Some(g) = &a.guard { no_call_in_branch(g, loc)?; }
                no_call_in_branch(&a.body, loc)?;
            }
            Ok(())
        }
        LoweredExpr::Lambda { body, .. } => no_call_in_branch(body, loc),
        LoweredExpr::List(xs) => { for x in xs { no_call(x, loc)?; } Ok(()) }
        LoweredExpr::Block(ss) => { for s in ss { check_stmt_anf(s, loc)?; } Ok(()) }
        LoweredExpr::Record { fields, .. }
        | LoweredExpr::VariantConstruct { fields, .. } => {
            for (_, e) in fields { no_call(e, loc)?; } Ok(())
        }
        LoweredExpr::For { iterable, body, .. } => {
            no_call(iterable, loc)?; no_call_in_branch(body, loc)
        }
        LoweredExpr::Return(fs) => { for (_, e) in fs { no_call(e, loc)?; } Ok(()) }
    }
}

fn no_call_in_branch(expr: &LoweredExpr, loc: &str) -> Result<(), String> {
    match expr {
        LoweredExpr::Block(ss) => { for s in ss { check_stmt_anf(s, loc)?; } Ok(()) }
        _ => no_call(expr, loc),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage 3 — Run machine
// ═══════════════════════════════════════════════════════════════════════════

// ── 3a. Types ───────────────────────────────────────────────────────────────

/// What eval_body decided to do.
enum Step {
    Return(HashMap<String, Value>),
    Call {
        callee: FnId,
        inputs: HashMap<String, Value>,
        cont: Option<Continuation>,
    },
    Error(String),
}

/// What eval_stmt decided to do.
enum StmtOutcome {
    /// Statement processed normally; continue to next statement.
    Continue,
    /// Function body is complete; return this output map.
    Done(HashMap<String, Value>),
    /// Need a sibling fn call; return this to the main loop.
    NeedCall {
        callee: FnId,
        inputs: HashMap<String, Value>,
        cont: Option<Continuation>,
    },
    /// Unrecoverable error.
    Err(String),
}

/// How to extract the caller's value from a callee's output map.
#[derive(Debug, Clone)]
enum Projection {
    /// Extract "return" field, falling back to single "value" field.
    ReturnField,
    /// Use entire output map as Value::Map.
    #[allow(dead_code)]
    WholeMap,
}

/// Saved state for resuming after a callee returns.
#[derive(Debug, Clone)]
struct Continuation {
    fn_id: FnId,
    pc: usize,
    binding: Option<String>,
    projection: Projection,
    env: Env,
}

// ── 3b. Main loop ───────────────────────────────────────────────────────────

/// Pure: (entry_fn_id, inputs, ctx) → Result<output_map, error>
///
/// The only function that calls eval_body. All fn-to-fn call chains go
/// through this loop — O(1) native stack per call.
fn run_machine(
    entry_fn_id: FnId,
    inputs: &HashMap<String, Value>,
    ctx: &EvalContext,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut stack: Vec<Continuation> = Vec::new();
    let mut fn_id = entry_fn_id;
    let mut pc: usize = 0;
    let mut env = make_initial_env(inputs, ctx.data_values);
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
                match pop_to_resume(&mut stack, result, ctx) {
                    PopResult::Finished(output) => return Ok(output),
                    PopResult::Resume { fn_id: fid, pc: p, env: e } => {
                        fn_id = fid; pc = p; env = e;
                    }
                }
            }
            Step::Call { callee, inputs, cont } => {
                if let Some(c) = cont {
                    if stack.len() >= MAX_STACK_DEPTH {
                        return Err(EvalError::new(format!(
                            "max stack depth ({MAX_STACK_DEPTH}) exceeded"
                        )));
                    }
                    stack.push(c);
                }
                fn_id = callee;
                pc = 0;
                env = make_initial_env(&inputs, ctx.data_values);
            }
            Step::Error(msg) => return Err(EvalError::new(msg)),
        }
    }
}

enum PopResult {
    Finished(HashMap<String, Value>),
    Resume { fn_id: FnId, pc: usize, env: Env },
}

/// Pop continuations until one has remaining work, or the stack is empty.
fn pop_to_resume(
    stack: &mut Vec<Continuation>,
    mut result: HashMap<String, Value>,
    ctx: &EvalContext,
) -> PopResult {
    loop {
        match stack.pop() {
            None => return PopResult::Finished(result),
            Some(cont) => {
                let value = extract_projection(&cont.projection, &result);
                let past_end = cont.pc >= ctx.fns[cont.fn_id].stmts.len();
                if past_end && cont.binding.is_none() {
                    result = wrap_value_as_output(value);
                } else {
                    let mut env = cont.env;
                    if let Some(ref name) = cont.binding {
                        bind_let_result(&mut env, name.clone(), &value);
                    }
                    return PopResult::Resume { fn_id: cont.fn_id, pc: cont.pc, env };
                }
            }
        }
    }
}

// ── 3c. Body evaluation ─────────────────────────────────────────────────────

/// Pure: (fn_id, start_pc, env, ctx) → Step
///
/// Iterates stmts[start_pc..], calling eval_stmt for each one.
/// Returns as soon as any stmt produces a Done or NeedCall.
fn eval_body(fn_id: FnId, start_pc: usize, env: &mut Env, ctx: &EvalContext) -> Step {
    let stmts = &ctx.fns[fn_id].stmts;

    for i in start_pc..stmts.len() {
        let is_last = i == stmts.len() - 1;
        match eval_stmt(&stmts[i], is_last, fn_id, i, env, ctx) {
            StmtOutcome::Continue => {}
            StmtOutcome::Done(result) => return Step::Return(result),
            StmtOutcome::NeedCall { callee, inputs, cont } =>
                return Step::Call { callee, inputs, cont },
            StmtOutcome::Err(msg) => return Step::Error(msg),
        }
    }

    Step::Return(unit_output())
}

// ── 3d. Statement evaluation ────────────────────────────────────────────────

/// Pure: (stmt, is_last, fn_id, stmt_index, env, ctx) → StmtOutcome
///
/// Handles one statement. Three cases: Let, Expr, Return.
fn eval_stmt(
    stmt: &LoweredStmt,
    is_last: bool,
    fn_id: FnId,
    stmt_idx: usize,
    env: &mut Env,
    ctx: &EvalContext,
) -> StmtOutcome {
    match stmt {
        LoweredStmt::Let(name, expr) =>
            eval_stmt_let(name, expr, fn_id, stmt_idx, env, ctx),
        LoweredStmt::Expr(expr) =>
            eval_stmt_expr(expr, is_last, fn_id, stmt_idx, env, ctx),
        LoweredStmt::Return(fields) =>
            eval_stmt_return(fields, env, ctx),
    }
}

/// `let name = expr` — bind the result. If expr is a sibling call, suspend.
fn eval_stmt_let(
    name: &str,
    expr: &LoweredExpr,
    fn_id: FnId,
    stmt_idx: usize,
    env: &mut Env,
    ctx: &EvalContext,
) -> StmtOutcome {
    match classify_call(expr, ctx) {
        CallKind::SiblingFn(callee_id) => {
            let args = call_args(expr);
            match eval_call_args(args, env, ctx) {
                Ok(inputs) => StmtOutcome::NeedCall {
                    callee: callee_id,
                    inputs,
                    cont: Some(Continuation {
                        fn_id, pc: stmt_idx + 1,
                        binding: Some(name.to_string()),
                        projection: Projection::ReturnField,
                        env: env.clone(),
                    }),
                },
                Err(msg) => StmtOutcome::Err(msg),
            }
        }
        CallKind::Builtin => {
            match eval_non_sibling_call(expr, env, ctx) {
                Ok(value)  => { bind_let_result(env, name.to_string(), &value); StmtOutcome::Continue }
                Err(e)     => outcome_from_eval_error(e),
            }
        }
        CallKind::ContainsCalls => {
            // Expression contains calls in blocks/branches. Using eval_expr
            // would leak to native recursion. Route through resolve_call_value
            // which handles sibling calls via the old evaluator (bounded depth).
            match resolve_call_value(expr, env, ctx) {
                Ok(value)  => { bind_let_result(env, name.to_string(), &value); StmtOutcome::Continue }
                Err(e)     => outcome_from_eval_error(e),
            }
        }
        CallKind::Pure => {
            match eval_expr(expr, env, ctx) {
                Ok(value)  => { bind_let_result(env, name.to_string(), &value); StmtOutcome::Continue }
                Err(e)     => outcome_from_eval_error(e),
            }
        }
    }
}

/// `expr;` — evaluate for side effects. Last-position handles trailing exprs.
fn eval_stmt_expr(
    expr: &LoweredExpr,
    is_last: bool,
    fn_id: FnId,
    stmt_idx: usize,
    env: &mut Env,
    ctx: &EvalContext,
) -> StmtOutcome {
    match classify_call(expr, ctx) {
        CallKind::SiblingFn(callee_id) => {
            let args = call_args(expr);
            match eval_call_args(args, env, ctx) {
                Ok(inputs) => StmtOutcome::NeedCall {
                    callee: callee_id,
                    inputs,
                    cont: Some(Continuation {
                        fn_id, pc: stmt_idx + 1,
                        binding: None,
                        projection: Projection::ReturnField,
                        env: env.clone(),
                    }),
                },
                Err(msg) => StmtOutcome::Err(msg),
            }
        }
        CallKind::Builtin => {
            match eval_non_sibling_call(expr, env, ctx) {
                Ok(value) if is_last => StmtOutcome::Done(wrap_value_as_output(value)),
                Ok(_)                => StmtOutcome::Continue,
                Err(e)               => outcome_from_eval_error(e),
            }
        }
        CallKind::ContainsCalls if is_last => {
            match eval_trailing(expr, env, ctx) {
                Step::Return(r) => StmtOutcome::Done(r),
                Step::Call { callee, inputs, cont } =>
                    StmtOutcome::NeedCall { callee, inputs, cont },
                Step::Error(msg) => StmtOutcome::Err(msg),
            }
        }
        CallKind::ContainsCalls => {
            match resolve_call_value(expr, env, ctx) {
                Ok(_)  => StmtOutcome::Continue,
                Err(e) => outcome_from_eval_error(e),
            }
        }
        CallKind::Pure if is_last => {
            match eval_trailing(expr, env, ctx) {
                Step::Return(r) => StmtOutcome::Done(r),
                Step::Call { callee, inputs, cont } =>
                    StmtOutcome::NeedCall { callee, inputs, cont },
                Step::Error(msg) => StmtOutcome::Err(msg),
            }
        }
        CallKind::Pure => {
            match eval_expr(expr, env, ctx) {
                Ok(_)  => StmtOutcome::Continue,
                Err(e) => outcome_from_eval_error(e),
            }
        }
    }
}

/// `return { k: v, ... }` — evaluate fields, produce output map.
fn eval_stmt_return(
    fields: &[(String, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
) -> StmtOutcome {
    let mut result = HashMap::new();
    for (name, expr) in fields {
        match eval_expr(expr, env, ctx) {
            Ok(value) => { result.insert(name.clone(), value); }
            Err(e) => return outcome_from_eval_error(e),
        }
    }
    StmtOutcome::Done(result)
}

// ── 3e. Call classification ─────────────────────────────────────────────────

enum CallKind {
    /// Top-level Call to a sibling fn — suspend to the heap stack.
    SiblingFn(FnId),
    /// Top-level Call to a builtin/intrinsic — evaluate inline.
    Builtin,
    /// Not a Call, but contains calls in descendant blocks/branches.
    /// Must NOT go to eval_expr (which would leak to native recursion).
    /// Routed through resolve_call_value which handles sibling calls
    /// via the old evaluator as a bounded-depth fallback.
    ContainsCalls,
    /// Pure expression — safe for eval_expr (no calls anywhere).
    Pure,
}

/// Pure: (expr, ctx) → classify how this expression should be evaluated.
///
/// The critical distinction: `Pure` means eval_expr will never encounter
/// a Call node anywhere in the tree. `ContainsCalls` means there are calls
/// hidden inside blocks/branches that eval_expr would silently route to
/// native recursion — these must go through resolve_call_value instead.
fn classify_call(expr: &LoweredExpr, ctx: &EvalContext) -> CallKind {
    if let LoweredExpr::Call { name, .. } = expr {
        match ctx.fn_index.get(name.as_str()) {
            Some(&id) => CallKind::SiblingFn(id),
            None      => CallKind::Builtin,
        }
    } else if expr_contains_call(expr) {
        CallKind::ContainsCalls
    } else {
        CallKind::Pure
    }
}

fn expr_contains_call(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { .. } => true,
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) | LoweredExpr::Lambda { .. } => false,
        LoweredExpr::FieldAccess { expr, .. } | LoweredExpr::UnaryOp { expr, .. } =>
            expr_contains_call(expr),
        LoweredExpr::BinOp { left, right, .. } =>
            expr_contains_call(left) || expr_contains_call(right),
        LoweredExpr::StringInterp(parts) => parts.iter().any(|p| match p {
            LoweredStringPart::Expr(e) => expr_contains_call(e),
            _ => false,
        }),
        LoweredExpr::IfElse { cond, then_, else_ } =>
            expr_contains_call(cond) || expr_contains_call(then_)
                || else_.as_ref().is_some_and(|e| expr_contains_call(e)),
        LoweredExpr::Match { expr, arms } =>
            expr_contains_call(expr)
                || arms.iter().any(|a| expr_contains_call(&a.body)
                    || a.guard.as_ref().is_some_and(|g| expr_contains_call(g))),
        LoweredExpr::List(items) => items.iter().any(expr_contains_call),
        LoweredExpr::Block(stmts) => stmts.iter().any(stmt_contains_call),
        LoweredExpr::Record { fields, .. } | LoweredExpr::VariantConstruct { fields, .. } =>
            fields.iter().any(|(_, e)| expr_contains_call(e)),
        LoweredExpr::For { iterable, body, .. } =>
            expr_contains_call(iterable) || expr_contains_call(body),
        LoweredExpr::Return(fields) => fields.iter().any(|(_, e)| expr_contains_call(e)),
    }
}

fn stmt_contains_call(stmt: &LoweredStmt) -> bool {
    match stmt {
        LoweredStmt::Let(_, e) | LoweredStmt::Expr(e) => expr_contains_call(e),
        LoweredStmt::Return(fields) => fields.iter().any(|(_, e)| expr_contains_call(e)),
    }
}

fn call_args(expr: &LoweredExpr) -> &[(Option<String>, LoweredExpr)] {
    match expr {
        LoweredExpr::Call { args, .. } => args,
        _ => &[],
    }
}

// ── 3f. Trailing expression ─────────────────────────────────────────────────

/// Pure: (expr, env, ctx) → Step
///
/// Drills through if/match/block at the tail of the last Expr statement
/// to find either a call (→ Step::Call) or a value (→ Step::Return).
fn eval_trailing(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Step {
    match expr {
        LoweredExpr::IfElse { cond, then_, else_ } => {
            match eval_expr(cond, env, ctx) {
                Ok(c) if value_truthy(&c) => eval_trailing(then_, env, ctx),
                Ok(_) => match else_ {
                    Some(e) => eval_trailing(e, env, ctx),
                    None    => Step::Return(wrap_value_as_output(Value::Unit)),
                },
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
            let mut child = env.child();
            eval_block_as_body(stmts, &mut child, ctx)
        }
        LoweredExpr::Return(fields) => {
            let mut result = HashMap::new();
            for (name, e) in fields {
                match eval_expr(e, env, ctx) {
                    Ok(v) => { result.insert(name.clone(), v); }
                    Err(e) => return step_from_eval_error(e),
                }
            }
            Step::Return(result)
        }
        _ => match eval_expr(expr, env, ctx) {
            Ok(v) => Step::Return(wrap_value_as_output(v)),
            Err(e) => step_from_eval_error(e),
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
            return eval_trailing(&arm.body, &arm_env, ctx);
        }
    }
    Step::Error(format!("no matching arm for: {scrutinee:?}"))
}

/// Block inside a trailing expression — handles calls via old evaluator fallback.
fn eval_block_as_body(stmts: &[LoweredStmt], env: &mut Env, ctx: &EvalContext) -> Step {
    let last_idx = stmts.len().saturating_sub(1);
    for (i, stmt) in stmts.iter().enumerate() {
        let is_last = i == last_idx && !stmts.is_empty();
        match stmt {
            LoweredStmt::Let(name, expr) => {
                match resolve_call_value(expr, env, ctx) {
                    Ok(value) => bind_let_result(env, name.clone(), &value),
                    Err(e) => return step_from_eval_error(e),
                }
            }
            LoweredStmt::Expr(expr) if is_last => {
                if let LoweredExpr::Call { .. } = expr {
                    match resolve_call_value(expr, env, ctx) {
                        Ok(v) => return Step::Return(wrap_value_as_output(v)),
                        Err(e) => return step_from_eval_error(e),
                    }
                }
                return eval_trailing(expr, env, ctx);
            }
            LoweredStmt::Expr(expr) => {
                match resolve_call_value(expr, env, ctx) {
                    Ok(_) => {}
                    Err(e) => return step_from_eval_error(e),
                }
            }
            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, e) in fields {
                    match eval_expr(e, env, ctx) {
                        Ok(v) => { result.insert(name.clone(), v); }
                        Err(e) => return step_from_eval_error(e),
                    }
                }
                return Step::Return(result);
            }
        }
    }
    Step::Return(unit_output())
}

/// Evaluate an expression that might be a call — sibling or builtin.
/// For sibling calls inside blocks, falls back to old recursive evaluator.
fn resolve_call_value(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    match classify_call(expr, ctx) {
        CallKind::SiblingFn(callee_id) => {
            let inputs = eval_call_args(call_args(expr), env, ctx)
                .map_err(|msg| EvalError::new(msg))?;
            eval_sibling_recursive(callee_id, &inputs, ctx)
                .map_err(|msg| EvalError::new(msg))
        }
        CallKind::Builtin => eval_non_sibling_call(expr, env, ctx),
        CallKind::ContainsCalls | CallKind::Pure => eval_expr(expr, env, ctx),
    }
}

// ── 3g. Pure expression evaluation ──────────────────────────────────────────
//
// Total function. After ANF, never encounters a Call in a nested position.
// (Calls at statement level are handled by eval_stmt before reaching here.)

fn eval_expr(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    match expr {
        LoweredExpr::Literal(lit) => Ok(eval_literal(lit)),
        LoweredExpr::Ident(name) => eval_ident(name, env),
        LoweredExpr::FieldAccess { expr, field } =>
            field_access(&eval_expr(expr, env, ctx)?, field),
        LoweredExpr::StringInterp(parts) => eval_string_interp(parts, env, ctx),
        LoweredExpr::BinOp { left, op, right } => eval_binop_expr(left, *op, right, env, ctx),
        LoweredExpr::UnaryOp { op, expr } => eval_unary(*op, expr, env, ctx),
        LoweredExpr::IfElse { cond, then_, else_ } => {
            if value_truthy(&eval_expr(cond, env, ctx)?) { eval_expr(then_, env, ctx) }
            else if let Some(e) = else_ { eval_expr(e, env, ctx) }
            else { Ok(Value::Unit) }
        }
        LoweredExpr::Match { expr: scrutinee, arms } => {
            let val = eval_expr(scrutinee, env, ctx)?;
            eval_match(&val, arms, env.bindings.as_ref(), ctx.sibling_fns)
        }
        LoweredExpr::VariantConstruct { tag, fields } => eval_variant(tag, fields, env, ctx),
        LoweredExpr::Call { name, args } => eval_non_sibling_call_raw(name, args, env, ctx),
        LoweredExpr::Lambda { .. } => Err(EvalError::new("lambda cannot be evaluated standalone")),
        LoweredExpr::List(items) => {
            items.iter().map(|i| eval_expr(i, env, ctx)).collect::<Result<Vec<_>, _>>()
                .map(Value::List)
        }
        LoweredExpr::Block(stmts) => eval_block(stmts, env, ctx),
        LoweredExpr::Record { fields, .. } => eval_record(fields, env, ctx),
        LoweredExpr::For { binding, iterable, body } => eval_for(binding, iterable, body, env, ctx),
        LoweredExpr::Return(fields) => eval_return_expr(fields, env, ctx),
    }
}

// ── 3h. Expression helpers (each a small pure function) ─────────────────────

fn eval_ident(name: &str, env: &Env) -> Result<Value, EvalError> {
    if name == "None" || name == "null" { return Ok(Value::Unit); }
    if let Some(val) = env.get(name) { return Ok(val.clone()); }
    if name.chars().next().unwrap_or('a').is_uppercase() { return Ok(Value::Str(name.to_string())); }
    Err(EvalError::new(format!("unbound variable: {name}")))
}

fn eval_string_interp(parts: &[LoweredStringPart], env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    let mut s = String::new();
    for p in parts {
        match p {
            LoweredStringPart::Literal(lit) => s.push_str(lit),
            LoweredStringPart::Expr(e) => s.push_str(&value_to_string(&eval_expr(e, env, ctx)?)),
        }
    }
    Ok(Value::Str(s))
}

fn eval_binop_expr(left: &LoweredExpr, op: LoweredBinOp, right: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
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
        LoweredBinOp::Add => {
            let rhs = eval_expr(right, env, ctx)?;
            match (lhs, rhs) {
                (Value::List(mut a), Value::List(b)) => { a.extend(b); Ok(Value::List(a)) }
                (Value::Str(mut a), Value::Str(b)) => { a.push_str(&b); Ok(Value::Str(a)) }
                (Value::Str(mut a), Value::Enum { variant, .. }) => { a.push_str(&variant); Ok(Value::Str(a)) }
                (Value::Enum { variant, .. }, Value::Str(b)) => Ok(Value::Str(format!("{variant}{b}"))),
                (l, r) => eval_binop(&l, op, &r),
            }
        }
        _ => eval_binop(&lhs, op, &eval_expr(right, env, ctx)?),
    }
}

fn eval_unary(op: LoweredUnaryOp, expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
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

fn eval_variant(tag: &str, fields: &[(String, LoweredExpr)], env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    if fields.is_empty() {
        Ok(Value::Enum { ty: String::new(), variant: tag.to_string() })
    } else {
        let mut map = BTreeMap::new();
        map.insert("_variant".to_string(), Value::Str(tag.to_string()));
        for (k, v) in fields { map.insert(k.clone(), eval_expr(v, env, ctx)?); }
        Ok(Value::Map(map))
    }
}

fn eval_block(stmts: &[LoweredStmt], env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    let mut child = env.child();
    let outputs = eval_pure_block_stmts(stmts, &mut child, ctx)?;
    if outputs.len() == 1 { if let Some(v) = outputs.get("return") { return Ok(v.clone()); } }
    Ok(Value::Map(outputs.into_iter().collect()))
}

fn eval_record(fields: &[(String, LoweredExpr)], env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    let mut map = BTreeMap::new();
    for (k, v) in fields { map.insert(k.clone(), eval_expr(v, env, ctx)?); }
    Ok(Value::Map(map))
}

fn eval_for(binding: &str, iterable: &LoweredExpr, body: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    match eval_expr(iterable, env, ctx)? {
        Value::List(list) => {
            let mut results = Vec::with_capacity(list.len());
            for item in &list {
                let mut iter_env = env.child();
                iter_env.bind(binding.to_string(), item.clone());
                results.push(eval_expr(body, &iter_env, ctx)?);
            }
            Ok(Value::List(results))
        }
        other => Err(EvalError::new(format!("for requires list, got {:?}", other))),
    }
}

fn eval_return_expr(fields: &[(String, LoweredExpr)], env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    let mut result = HashMap::new();
    for (n, e) in fields { result.insert(n.clone(), eval_expr(e, env, ctx)?); }
    Err(EvalError::early_return(result))
}

/// Evaluate a block's stmts purely (no calls expected at stmt level in pure context).
fn eval_pure_block_stmts(stmts: &[LoweredStmt], env: &mut Env, ctx: &EvalContext) -> Result<HashMap<String, Value>, EvalError> {
    let last = stmts.last();
    for stmt in stmts {
        let is_last = last.is_some_and(|l| std::ptr::eq(stmt, l));
        match stmt {
            LoweredStmt::Let(name, expr) => {
                bind_let_result(env, name.clone(), &eval_expr(expr, env, ctx)?);
            }
            LoweredStmt::Expr(expr) => {
                let value = eval_expr(expr, env, ctx)?;
                if is_last { return Ok(wrap_value_as_output(value)); }
            }
            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, e) in fields { result.insert(name.clone(), eval_expr(e, env, ctx)?); }
                return Err(EvalError::early_return(result));
            }
        }
    }
    Ok(unit_output())
}

// ── 3i. Call bridges ────────────────────────────────────────────────────────

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
    let outputs = crate::eval::evaluate_fn_body_old(body, inputs, ctx.sibling_fns, ctx.data_values)
        .map_err(|e| e.message)?;
    Ok(extract_projection(&Projection::ReturnField, &outputs))
}

fn eval_non_sibling_call(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    if let LoweredExpr::Call { name, args } = expr {
        eval_non_sibling_call_raw(name, args, env, ctx)
    } else {
        eval_expr(expr, env, ctx)
    }
}

fn eval_non_sibling_call_raw(
    name: &str, args: &[(Option<String>, LoweredExpr)], env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    let env_bindings: HashMap<String, Value> = env.bindings.as_ref().clone();
    crate::eval::eval_non_sibling_call(name, args, &env_bindings, ctx.sibling_fns, ctx.data_values)
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared helpers (used across stages)
// ═══════════════════════════════════════════════════════════════════════════

fn extract_projection(proj: &Projection, outputs: &HashMap<String, Value>) -> Value {
    match proj {
        Projection::ReturnField => {
            if let Some(v) = outputs.get("return") { return v.clone(); }
            if outputs.len() == 1 { if let Some(v) = outputs.get("value") { return v.clone(); } }
            if outputs.is_empty() { return Value::Unit; }
            Value::Map(outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        }
        Projection::WholeMap =>
            Value::Map(outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
    }
}

fn bind_let_result(env: &mut Env, name: String, value: &Value) {
    match value {
        Value::Map(fields) => {
            for (f, v) in fields { env.bind(format!("{name}__{f}"), v.clone()); }
        }
        Value::Json(serde_json::Value::Object(map)) => {
            for (f, v) in map { env.bind(format!("{name}__{f}"), Value::Json(v.clone())); }
        }
        _ => {}
    }
    env.bind(name, value.clone());
}

// match_pattern, eval_literal, values_equal: imported from crate::eval
// (single implementation — no parallel copies)

fn wrap_value_as_output(value: Value) -> HashMap<String, Value> {
    if let Value::Map(map) = &value {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    } else {
        [("return".to_string(), value)].into_iter().collect()
    }
}

fn unit_output() -> HashMap<String, Value> {
    [("return".to_string(), Value::Unit)].into_iter().collect()
}

fn make_initial_env(inputs: &HashMap<String, Value>, data_values: &HashMap<String, Value>) -> Env {
    let mut env = Env::from_inputs(inputs);
    for (name, val) in data_values {
        if env.get(name).is_none() { env.bind(name.clone(), val.clone()); }
    }
    env
}

fn step_from_eval_error(e: EvalError) -> Step {
    if let Some(ret) = e.early_return { Step::Return(ret) } else { Step::Error(e.message) }
}

fn outcome_from_eval_error(e: EvalError) -> StmtOutcome {
    if let Some(ret) = e.early_return { StmtOutcome::Done(ret) } else { StmtOutcome::Err(e.message) }
}

// ── Environment ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Env {
    bindings: Rc<HashMap<String, Value>>,
}

impl Env {
    fn from_inputs(inputs: &HashMap<String, Value>) -> Self { Self { bindings: Rc::new(inputs.clone()) } }
    fn bind(&mut self, name: String, value: Value) { Rc::make_mut(&mut self.bindings).insert(name, value); }
    fn get(&self, name: &str) -> Option<&Value> { self.bindings.get(name) }
    fn child(&self) -> Self { Self { bindings: Rc::clone(&self.bindings) } }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

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

    fn is_even_odd_pair() -> (LoweredFnBody, LoweredFnBody) {
        let mk = |base_name: &str, base_val: bool, other: &str| LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")), op: LoweredBinOp::Eq, right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![
                        ("return".to_string(), LoweredExpr::Literal(LoweredLiteral::Bool(base_val))),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Expr(call(other, vec![
                    ("n", LoweredExpr::BinOp {
                        left: Box::new(ident("n")), op: LoweredBinOp::Sub, right: Box::new(int(1)),
                    }),
                ])),
            ],
        };
        (mk("is_even", true, "is_odd"), mk("is_odd", false, "is_even"))
    }

    #[test] fn simple_fn() {
        let body = LoweredFnBody { stmts: vec![LoweredStmt::Return(vec![("return".into(), int(42))])] };
        assert_eq!(evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()["return"], Value::Int(42));
    }

    #[test] fn sibling_call_with_projection() {
        let inner = LoweredFnBody { stmts: vec![LoweredStmt::Return(vec![("value".into(), int(99))])] };
        let outer = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("r".into(), call("inner", vec![])),
            LoweredStmt::Return(vec![("return".into(), ident("r"))]),
        ]};
        let mut s = HashMap::new(); s.insert("inner".into(), inner); s.insert("outer".into(), outer.clone());
        assert_eq!(evaluate_stack(&outer, &HashMap::new(), &s, &HashMap::new()).unwrap()["return"], Value::Int(99));
    }

    #[test] fn deep_mutual_recursion_40k() {
        let (even, odd) = is_even_odd_pair();
        let mut s = HashMap::new(); s.insert("is_even".into(), even.clone()); s.insert("is_odd".into(), odd);
        let mut i = HashMap::new();
        i.insert("n".into(), Value::Int(40_000));
        assert_eq!(evaluate_stack(&even, &i, &s, &HashMap::new()).unwrap()["return"], Value::Bool(true));
        i.insert("n".into(), Value::Int(40_001));
        assert_eq!(evaluate_stack(&even, &i, &s, &HashMap::new()).unwrap()["return"], Value::Bool(false));
    }

    #[test] fn value_normalization() {
        let inner = LoweredFnBody { stmts: vec![LoweredStmt::Return(vec![("value".into(), int(42))])] };
        let wrapper = LoweredFnBody { stmts: vec![LoweredStmt::Expr(call("inner", vec![]))] };
        let mut s = HashMap::new(); s.insert("inner".into(), inner); s.insert("wrapper".into(), wrapper.clone());
        let r = evaluate_stack(&wrapper, &HashMap::new(), &s, &HashMap::new()).unwrap();
        assert_eq!(r.get("return"), Some(&Value::Int(42)));
        assert!(!r.contains_key("value"));
    }

    #[test] fn builtin_call() {
        let body = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("result".into(), LoweredExpr::Call {
                name: "skip_horizontal_ws".into(),
                args: vec![(Some("s".into()), ident("s")), (Some("start".into()), ident("start"))],
            }),
            LoweredStmt::Return(vec![("return".into(), ident("result"))]),
        ]};
        let mut i = HashMap::new(); i.insert("s".into(), Value::Str("   hello".into())); i.insert("start".into(), Value::Int(0));
        assert_eq!(evaluate_stack(&body, &i, &HashMap::new(), &HashMap::new()).unwrap()["return"], Value::Int(3));
    }

    #[test] fn sibling_then_builtin() {
        let mk = LoweredFnBody { stmts: vec![LoweredStmt::Return(vec![("source".into(), ident("source")), ("start".into(), int(0))])] };
        let outer = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("state".into(), call("make_state", vec![("source", ident("source"))])),
            LoweredStmt::Let("result".into(), LoweredExpr::Call {
                name: "skip_horizontal_ws".into(),
                args: vec![
                    (Some("s".into()), LoweredExpr::FieldAccess { expr: Box::new(ident("state")), field: "source".into() }),
                    (Some("start".into()), LoweredExpr::FieldAccess { expr: Box::new(ident("state")), field: "start".into() }),
                ],
            }),
            LoweredStmt::Return(vec![("return".into(), ident("result"))]),
        ]};
        let mut s = HashMap::new(); s.insert("make_state".into(), mk); s.insert("outer".into(), outer.clone());
        let mut i = HashMap::new(); i.insert("source".into(), Value::Str("   hello".into()));
        assert_eq!(evaluate_stack(&outer, &i, &s, &HashMap::new()).unwrap()["return"], Value::Int(3));
    }

    #[test] fn anf_verifier_catches_nested_call() {
        let bad = LoweredFnBody { stmts: vec![LoweredStmt::Let("x".into(), LoweredExpr::BinOp {
            left: Box::new(call("f", vec![])), op: LoweredBinOp::Add, right: Box::new(int(1)),
        })] };
        let (sibs, data) = (HashMap::new(), HashMap::new());
        let (ctx, _) = build_context(&bad, &sibs, &data);
        assert!(verify_anf_contract(&ctx).is_err());
    }
}
