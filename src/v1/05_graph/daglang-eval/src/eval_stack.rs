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
//! Only `run_machine` does.
//!
//! **ANF contract:** SIBLING fn calls appear only at statement level.
//! Non-sibling calls (builtins, intrinsics) may still appear nested in
//! expressions and are evaluated inline by `eval_expr` → `eval_non_sibling_call_raw`.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use gunbc_ir::Value;

use crate::eval_core::{
    eval_binop, eval_builtin_call, eval_literal, field_access, match_pattern,
    value_to_string, value_truthy, EvalError,
};
use crate::expr::{
    LoweredBinOp, LoweredExpr, LoweredFnBody, LoweredMatchArm, LoweredStmt, LoweredStringPart,
    LoweredUnaryOp,
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

    if let Err(msg) = verify_anf_contract(&ctx) {
        return Err(EvalError::new(format!("ANF contract violated: {msg}")));
    }

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
///
/// If entry_body is pointer-equal to one of the sibling_fns values, its
/// FnId will be that sibling's id (no duplication). Otherwise it gets a
/// dedicated FnId at the end of the table.
///
/// **Note:** If entry_body is NOT in sibling_fns, self-recursive calls by
/// name will not resolve (the entry has a FnId but no name→FnId mapping).
/// Callers must ensure the entry fn is present in sibling_fns for
/// self-recursion to work.
fn build_context<'a>(
    entry_body: &'a LoweredFnBody,
    sibling_fns: &'a HashMap<String, LoweredFnBody>,
    data_values: &'a HashMap<String, Value>,
) -> (EvalContext<'a>, FnId) {
    let mut fns = Vec::with_capacity(sibling_fns.len() + 1);
    let mut fn_index = HashMap::with_capacity(sibling_fns.len() + 1);
    let mut entry_id: Option<FnId> = None;
    for (name, body) in sibling_fns {
        let id = fns.len();
        fns.push(body);
        fn_index.insert(name.as_str(), id);
        if std::ptr::eq(body, entry_body) {
            entry_id = Some(id);
        }
    }
    let entry_id = entry_id.unwrap_or_else(|| {
        let id = fns.len();
        fns.push(entry_body);
        id
    });
    (EvalContext { fns, fn_index, data_values, sibling_fns }, entry_id)
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage 2 — Verify ANF contract
// ═══════════════════════════════════════════════════════════════════════════

/// Pure: EvalContext → Ok(()) or Err(location).
/// Asserts no SIBLING fn Call nested inside another expression.
/// Builtin calls are allowed in nested position (they evaluate inline).
fn verify_anf_contract(ctx: &EvalContext) -> Result<(), String> {
    for (id, body) in ctx.fns.iter().enumerate() {
        for (i, stmt) in body.stmts.iter().enumerate() {
            check_stmt_anf(stmt, &format!("fn[{id}]/stmt[{i}]"), &ctx.fn_index)?;
        }
    }
    Ok(())
}

fn check_stmt_anf(stmt: &LoweredStmt, loc: &str, sibs: &HashMap<&str, FnId>) -> Result<(), String> {
    match stmt {
        LoweredStmt::Let(_, LoweredExpr::Call { args, .. })
        | LoweredStmt::Expr(LoweredExpr::Call { args, .. }) => {
            for (_, a) in args { no_sibling_call(a, loc, sibs)?; }
            Ok(())
        }
        LoweredStmt::Let(_, e) => no_sibling_call(e, loc, sibs),
        LoweredStmt::Expr(e)   => no_sibling_call_in_branch(e, loc, sibs),
        LoweredStmt::Return(fields) => {
            for (_, e) in fields { no_sibling_call(e, loc, sibs)?; }
            Ok(())
        }
    }
}

fn no_sibling_call(expr: &LoweredExpr, loc: &str, sibs: &HashMap<&str, FnId>) -> Result<(), String> {
    match expr {
        LoweredExpr::Call { name, args } => {
            if sibs.contains_key(name.as_str()) {
                return Err(format!("ANF violation at {loc}: nested sibling Call to '{name}'"));
            }
            for (_, a) in args { no_sibling_call(a, loc, sibs)?; }
            Ok(())
        }
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => Ok(()),
        LoweredExpr::FieldAccess { expr, .. }
        | LoweredExpr::UnaryOp { expr, .. } => no_sibling_call(expr, loc, sibs),
        LoweredExpr::BinOp { left, right, .. } => {
            no_sibling_call(left, loc, sibs)?; no_sibling_call(right, loc, sibs)
        }
        LoweredExpr::StringInterp(ps) => {
            for p in ps { if let LoweredStringPart::Expr(e) = p { no_sibling_call(e, loc, sibs)?; } }
            Ok(())
        }
        LoweredExpr::IfElse { cond, then_, else_ } => {
            no_sibling_call(cond, loc, sibs)?;
            no_sibling_call_in_branch(then_, loc, sibs)?;
            if let Some(e) = else_ { no_sibling_call_in_branch(e, loc, sibs)?; }
            Ok(())
        }
        LoweredExpr::Match { expr, arms } => {
            no_sibling_call(expr, loc, sibs)?;
            for a in arms {
                if let Some(g) = &a.guard { no_sibling_call_in_branch(g, loc, sibs)?; }
                no_sibling_call_in_branch(&a.body, loc, sibs)?;
            }
            Ok(())
        }
        LoweredExpr::Lambda { body, .. } => no_sibling_call_in_branch(body, loc, sibs),
        LoweredExpr::List(xs) => { for x in xs { no_sibling_call(x, loc, sibs)?; } Ok(()) }
        LoweredExpr::Block(ss) => { for s in ss { check_stmt_anf(s, loc, sibs)?; } Ok(()) }
        LoweredExpr::Record { fields, .. }
        | LoweredExpr::VariantConstruct { fields, .. } => {
            for (_, e) in fields { no_sibling_call(e, loc, sibs)?; } Ok(())
        }
        LoweredExpr::For { iterable, body, .. } => {
            no_sibling_call(iterable, loc, sibs)?; no_sibling_call_in_branch(body, loc, sibs)
        }
        LoweredExpr::Return(fs) => { for (_, e) in fs { no_sibling_call(e, loc, sibs)?; } Ok(()) }
    }
}

fn no_sibling_call_in_branch(expr: &LoweredExpr, loc: &str, sibs: &HashMap<&str, FnId>) -> Result<(), String> {
    match expr {
        LoweredExpr::Block(ss) => { for s in ss { check_stmt_anf(s, loc, sibs)?; } Ok(()) }
        _ => no_sibling_call(expr, loc, sibs),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage 3 — Run machine (slice-based continuations + stack bubbling)
// ═══════════════════════════════════════════════════════════════════════════
//
// Continuations store &[LoweredStmt] slices — remaining statements to
// process after a call returns. This allows suspension from ANY depth:
// top-level fn bodies, nested blocks, if/match branches.
//
// When eval_expr_s encounters a sibling call inside a Block, it pushes the
// block's remaining stmts and returns Suspend. The caller catches Suspend,
// pushes ITS remaining stmts, and bubbles upward. The native Rust stack
// unwinds instantly; the heap stack captures all saved positions.
//
// Deleted: eval_block_as_body, eval_pure_block_stmts, resolve_call_value,
// eval_sibling_recursive, CallKind::ContainsCalls. One unified path.

// ── 3a. Types ───────────────────────────────────────────────────────────────

enum Step {
    /// Fn body or block completed normally with this output map.
    Return(HashMap<String, Value>),
    /// Early return: unwind past block-resume continuations to the fn boundary.
    EarlyReturn(HashMap<String, Value>),
    /// Need a sibling fn call.
    Call { callee: FnId, inputs: HashMap<String, Value> },
    /// Error.
    Error(String),
}

enum ExprResult {
    Value(Box<Value>),
    EarlyReturn(HashMap<String, Value>),
    Suspend { callee: FnId, inputs: HashMap<String, Value> },
    Error(String),
}

#[derive(Debug, Clone)]
enum Projection {
    ReturnField,
    #[allow(dead_code)]
    WholeMap,
}

#[derive(Clone)]
struct Continuation<'a> {
    remaining: &'a [LoweredStmt],
    binding: Option<String>,
    projection: Projection,
    env: Env,
    is_fn_boundary: bool,
}

// ── 3b. Main loop ───────────────────────────────────────────────────────────

fn run_machine<'a>(
    entry: FnId,
    inputs: &HashMap<String, Value>,
    ctx: &'a EvalContext<'a>,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut stack: Vec<Continuation<'a>> = Vec::new();
    let mut stmts: &'a [LoweredStmt] = &ctx.fns[entry].stmts;
    let mut env = Env::from_inputs(inputs);
    let mut transitions: usize = 0;

    loop {
        transitions += 1;
        if transitions > MAX_TRANSITIONS {
            return Err(EvalError::new(format!(
                "transition budget ({MAX_TRANSITIONS}) exceeded"
            )));
        }

        match eval_stmts(stmts, &mut env, ctx, &mut stack) {
            Step::Return(result) => {
                match pop_stack(&mut stack, result) {
                    PopResult::Done(output) => return Ok(output),
                    PopResult::Resume { stmts: s, env: e, .. } => { stmts = s; env = e; }
                    PopResult::Error(msg) => return Err(EvalError::new(msg)),
                }
            }
            Step::EarlyReturn(result) => {
                // Unwind past block-resume continuations to the fn boundary.
                while let Some(cont) = stack.last() {
                    if cont.is_fn_boundary { break; }
                    stack.pop();
                }
                match pop_stack(&mut stack, result) {
                    PopResult::Done(output) => return Ok(output),
                    PopResult::Resume { stmts: s, env: e, .. } => { stmts = s; env = e; }
                    PopResult::Error(msg) => return Err(EvalError::new(msg)),
                }
            }
            Step::Call { callee, inputs } => {
                if stack.len() >= MAX_STACK_DEPTH {
                    return Err(EvalError::new(format!(
                        "max stack depth ({MAX_STACK_DEPTH}) exceeded"
                    )));
                }
                stmts = &ctx.fns[callee].stmts;
                env = Env::from_inputs(&inputs);
            }
            Step::Error(msg) => return Err(EvalError::new(msg)),
        }
    }
}

enum PopResult<'a> {
    Done(HashMap<String, Value>),
    Resume { stmts: &'a [LoweredStmt], env: Env, #[allow(dead_code)] is_fn_boundary: bool },
    Error(String),
}

fn pop_stack<'a>(
    stack: &mut Vec<Continuation<'a>>,
    mut result: HashMap<String, Value>,
) -> PopResult<'a> {
    loop {
        match stack.pop() {
            None => return PopResult::Done(result),
            Some(cont) => {
                let value = match extract_projection(&cont.projection, &result) {
                    Ok(v) => v,
                    Err(msg) => return PopResult::Error(msg),
                };
                let mut env = cont.env;
                if let Some(ref name) = cont.binding {
                    bind_let_result(&mut env, name.clone(), &value);
                }
                if cont.remaining.is_empty() {
                    if cont.binding.is_none() {
                        result = wrap_value_as_output(value);
                    } else {
                        result = unit_output();
                    }
                } else {
                    return PopResult::Resume { stmts: cont.remaining, env, is_fn_boundary: cont.is_fn_boundary };
                }
            }
        }
    }
}

// ── 3c. Statement processing (single unified implementation) ────────────────

fn eval_stmts<'a>(
    stmts: &'a [LoweredStmt],
    env: &mut Env,
    ctx: &'a EvalContext<'a>,
    stack: &mut Vec<Continuation<'a>>,
) -> Step {
    for (i, stmt) in stmts.iter().enumerate() {
        let is_last = i == stmts.len() - 1;
        let remaining = &stmts[i + 1..];
        // Record stack depth before eval_expr_s calls. If eval_expr_s pushes
        // inner (block/match) continuations, our outer continuation must go
        // BELOW them so pop_stack processes inner continuations first.
        let stack_base = stack.len();

        match stmt {
            LoweredStmt::Let(name, expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(&callee_id) = ctx.fn_index.get(callee.as_str()) {
                        match eval_call_args(args, env, ctx) {
                            Ok(inputs) => {
                                stack.push(Continuation {
                                    remaining, binding: Some(name.clone()),
                                    projection: Projection::ReturnField,
                                    env: env.clone(), is_fn_boundary: true,
                                });
                                return Step::Call { callee: callee_id, inputs };
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, env, ctx) {
                            Ok(value) => bind_let_result(env, name.clone(), &value),
                            Err(e) => return step_from_error(e),
                        }
                    }
                } else {
                    match eval_expr_s(expr, env, ctx, stack) {
                        ExprResult::Value(value) => bind_let_result(env, name.clone(), &value),
                        ExprResult::EarlyReturn(map) => return Step::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(stack_base, Continuation {
                                remaining, binding: Some(name.clone()),
                                projection: Projection::ReturnField,
                                env: env.clone(), is_fn_boundary: false,
                            });
                            return Step::Call { callee, inputs };
                        }
                        ExprResult::Error(msg) => return Step::Error(msg),
                    }
                }
            }

            LoweredStmt::Expr(expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(&callee_id) = ctx.fn_index.get(callee.as_str()) {
                        match eval_call_args(args, env, ctx) {
                            Ok(inputs) => {
                                stack.push(Continuation {
                                    remaining, binding: None,
                                    projection: Projection::ReturnField,
                                    env: env.clone(), is_fn_boundary: true,
                                });
                                return Step::Call { callee: callee_id, inputs };
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, env, ctx) {
                            Ok(value) if is_last => return Step::Return(wrap_value_as_output(value)),
                            Ok(_) => {}
                            Err(e) => return step_from_error(e),
                        }
                    }
                } else if is_last {
                    match eval_expr_s(expr, env, ctx, stack) {
                        ExprResult::Value(value) => return Step::Return(wrap_value_as_output(*value)),
                        ExprResult::EarlyReturn(map) => return Step::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(stack_base, Continuation {
                                remaining: &[], binding: None,
                                projection: Projection::ReturnField,
                                env: env.clone(), is_fn_boundary: false,
                            });
                            return Step::Call { callee, inputs };
                        }
                        ExprResult::Error(msg) => return Step::Error(msg),
                    }
                } else {
                    match eval_expr_s(expr, env, ctx, stack) {
                        ExprResult::Value(_) => {}
                        ExprResult::EarlyReturn(map) => return Step::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(stack_base, Continuation {
                                remaining, binding: None,
                                projection: Projection::ReturnField,
                                env: env.clone(), is_fn_boundary: false,
                            });
                            return Step::Call { callee, inputs };
                        }
                        ExprResult::Error(msg) => return Step::Error(msg),
                    }
                }
            }

            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, fexpr) in fields {
                    match eval_expr(fexpr, env, ctx) {
                        Ok(v) => { result.insert(name.clone(), v); }
                        Err(e) => return step_from_error(e),
                    }
                }
                return Step::EarlyReturn(result);
            }
        }
    }

    Step::Return(unit_output())
}

// ── 3d. Suspendable expression evaluation ───────────────────────────────────
//
// Handles IfElse, Match, Block by drilling into branches. When a sibling
// call is found inside a block, pushes block-resume continuations and
// returns Suspend. The caller catches Suspend and bubbles it upward.
// Pure leaf expressions delegate to eval_expr (which never suspends).

fn eval_expr_s<'a>(
    expr: &'a LoweredExpr,
    env: &Env,
    ctx: &'a EvalContext<'a>,
    stack: &mut Vec<Continuation<'a>>,
) -> ExprResult {
    match expr {
        LoweredExpr::IfElse { cond, then_, else_ } => {
            match eval_expr(cond, env, ctx) {
                Ok(c) => {
                    let branch = if value_truthy(&c) { Some(then_.as_ref()) }
                        else { else_.as_ref().map(|e| e.as_ref()) };
                    match branch {
                        Some(b) => eval_expr_s(b, env, ctx, stack),
                        None => ExprResult::Value(Box::new(Value::Unit)),
                    }
                }
                Err(e) => expr_from_error(e),
            }
        }
        LoweredExpr::Match { expr: scrutinee, arms } => {
            match eval_expr(scrutinee, env, ctx) {
                Ok(val) => eval_match_s(&val, arms, env, ctx, stack),
                Err(e) => expr_from_error(e),
            }
        }
        LoweredExpr::Block(block_stmts) => {
            eval_block_s(block_stmts, env, ctx, stack)
        }
        _ => match eval_expr(expr, env, ctx) {
            Ok(v) => ExprResult::Value(Box::new(v)),
            Err(e) => expr_from_error(e),
        }
    }
}

fn eval_match_s<'a>(
    scrutinee: &Value,
    arms: &'a [LoweredMatchArm],
    env: &Env,
    ctx: &'a EvalContext<'a>,
    stack: &mut Vec<Continuation<'a>>,
) -> ExprResult {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, scrutinee) {
            let mut arm_env = env.child();
            for (name, val) in bindings { arm_env.bind(name, val); }
            if let Some(guard) = &arm.guard {
                match eval_expr(guard, &arm_env, ctx) {
                    Ok(g) if value_truthy(&g) => {}
                    Ok(_) => continue,
                    Err(e) => return expr_from_error(e),
                }
            }
            return eval_expr_s(&arm.body, &arm_env, ctx, stack);
        }
    }
    ExprResult::Error(format!("no matching arm for: {scrutinee:?}"))
}

fn eval_block_s<'a>(
    block_stmts: &'a [LoweredStmt],
    env: &Env,
    ctx: &'a EvalContext<'a>,
    stack: &mut Vec<Continuation<'a>>,
) -> ExprResult {
    let mut child = env.child();
    for (i, stmt) in block_stmts.iter().enumerate() {
        let is_last = i == block_stmts.len() - 1;
        let remaining = &block_stmts[i + 1..];
        let stack_base = stack.len();

        match stmt {
            LoweredStmt::Let(name, expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(&callee_id) = ctx.fn_index.get(callee.as_str()) {
                        match eval_call_args(args, &child, ctx) {
                            Ok(inputs) => {
                                stack.push(Continuation {
                                    remaining, binding: Some(name.clone()),
                                    projection: Projection::ReturnField,
                                    env: child, is_fn_boundary: false,
                                });
                                return ExprResult::Suspend { callee: callee_id, inputs };
                            }
                            Err(msg) => return ExprResult::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, &child, ctx) {
                            Ok(value) => bind_let_result(&mut child, name.clone(), &value),
                            Err(e) => return expr_from_error(e),
                        }
                    }
                } else {
                    match eval_expr_s(expr, &child, ctx, stack) {
                        ExprResult::Value(value) => bind_let_result(&mut child, name.clone(), &value),
                        other @ (ExprResult::EarlyReturn(_) | ExprResult::Error(_)) => return other,
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(stack_base, Continuation {
                                remaining, binding: Some(name.clone()),
                                projection: Projection::ReturnField,
                                env: child, is_fn_boundary: false,
                            });
                            return ExprResult::Suspend { callee, inputs };
                        }
                    }
                }
            }
            LoweredStmt::Expr(expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(&callee_id) = ctx.fn_index.get(callee.as_str()) {
                        match eval_call_args(args, &child, ctx) {
                            Ok(inputs) => {
                                stack.push(Continuation {
                                    remaining, binding: None,
                                    projection: Projection::ReturnField,
                                    env: child, is_fn_boundary: false,
                                });
                                return ExprResult::Suspend { callee: callee_id, inputs };
                            }
                            Err(msg) => return ExprResult::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, &child, ctx) {
                            Ok(value) if is_last => return ExprResult::Value(Box::new(value)),
                            Ok(_) => {}
                            Err(e) => return expr_from_error(e),
                        }
                    }
                } else if is_last {
                    return eval_expr_s(expr, &child, ctx, stack);
                } else {
                    match eval_expr_s(expr, &child, ctx, stack) {
                        ExprResult::Value(_) => {}
                        ExprResult::EarlyReturn(map) => return ExprResult::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(stack_base, Continuation {
                                remaining, binding: None,
                                projection: Projection::ReturnField,
                                env: child, is_fn_boundary: false,
                            });
                            return ExprResult::Suspend { callee, inputs };
                        }
                        ExprResult::Error(msg) => return ExprResult::Error(msg),
                    }
                }
            }
            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, fexpr) in fields {
                    match eval_expr(fexpr, &child, ctx) {
                        Ok(v) => { result.insert(name.clone(), v); }
                        Err(e) => return expr_from_error(e),
                    }
                }
                return ExprResult::EarlyReturn(result);
            }
        }
    }
    ExprResult::Value(Box::new(Value::Unit))
}

// ── 3e. Pure expression evaluation ──────────────────────────────────────────

fn eval_expr(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    match expr {
        LoweredExpr::Literal(lit) => Ok(eval_literal(lit)),
        LoweredExpr::Ident(name) => eval_ident(name, env, ctx),
        LoweredExpr::FieldAccess { expr, field } =>
            field_access(&eval_expr(expr, env, ctx)?, field),
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
                LoweredBinOp::Add => {
                    let rhs = eval_expr(right, env, ctx)?;
                    match (lhs, rhs) {
                        (Value::List(mut a), Value::List(b)) => { a.extend(b); Ok(Value::List(a)) }
                        (Value::Str(mut a), Value::Str(b)) => { a.push_str(&b); Ok(Value::Str(a)) }
                        (Value::Str(mut a), Value::Enum { variant, .. }) => { a.push_str(&variant); Ok(Value::Str(a)) }
                        (Value::Enum { variant, .. }, Value::Str(b)) => Ok(Value::Str(format!("{variant}{b}"))),
                        (l, r) => eval_binop(&l, *op, &r),
                    }
                }
                _ => eval_binop(&lhs, *op, &eval_expr(right, env, ctx)?),
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
            if value_truthy(&eval_expr(cond, env, ctx)?) { eval_expr(then_, env, ctx) }
            else if let Some(e) = else_ { eval_expr(e, env, ctx) }
            else { Ok(Value::Unit) }
        }
        LoweredExpr::Match { expr: scrutinee, arms } => {
            let val = eval_expr(scrutinee, env, ctx)?;
            eval_match_local(&val, arms, env, ctx)
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
        LoweredExpr::Call { name, args } => eval_non_sibling_call_raw(name, args, env, ctx),
        LoweredExpr::Lambda { .. } => Err(EvalError::new("lambda cannot be evaluated standalone")),
        LoweredExpr::List(items) =>
            items.iter().map(|i| eval_expr(i, env, ctx)).collect::<Result<Vec<_>, _>>().map(Value::List),
        LoweredExpr::Block(stmts) => {
            let mut child = env.child();
            let last = stmts.last();
            for stmt in stmts {
                let is_last = last.is_some_and(|l| std::ptr::eq(stmt, l));
                match stmt {
                    LoweredStmt::Let(name, e) => {
                        let val = eval_expr(e, &child, ctx)?;
                        bind_let_result(&mut child, name.clone(), &val);
                    }
                    LoweredStmt::Expr(e) => {
                        let value = eval_expr(e, &child, ctx)?;
                        if is_last { return Ok(value); }
                    }
                    LoweredStmt::Return(fields) => {
                        let mut result = HashMap::new();
                        for (name, e) in fields { result.insert(name.clone(), eval_expr(e, &child, ctx)?); }
                        return Err(EvalError::early_return(result));
                    }
                }
            }
            Ok(Value::Unit)
        }
        LoweredExpr::Record { fields, .. } => {
            let mut map = BTreeMap::new();
            for (k, v) in fields { map.insert(k.clone(), eval_expr(v, env, ctx)?); }
            Ok(Value::Map(map))
        }
        LoweredExpr::For { binding, iterable, body } => {
            match eval_expr(iterable, env, ctx)? {
                Value::List(list) => {
                    let mut results = Vec::with_capacity(list.len());
                    for item in &list {
                        let mut iter_env = env.child();
                        iter_env.bind(binding.clone(), item.clone());
                        results.push(eval_expr(body, &iter_env, ctx)?);
                    }
                    Ok(Value::List(results))
                }
                other => Err(EvalError::new(format!("for requires list, got {:?}", other))),
            }
        }
        LoweredExpr::Return(fields) => {
            let mut result = HashMap::new();
            for (n, e) in fields { result.insert(n.clone(), eval_expr(e, env, ctx)?); }
            Err(EvalError::early_return(result))
        }
    }
}

fn eval_ident(name: &str, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    if name == "None" || name == "null" { return Ok(Value::Unit); }
    if let Some(val) = env.get(name) { return Ok(val.clone()); }
    if let Some(val) = ctx.data_values.get(name) { return Ok(val.clone()); }
    if name.chars().next().unwrap_or('a').is_uppercase() { return Ok(Value::Str(name.to_string())); }
    Err(EvalError::new(format!("unbound variable: {name}")))
}

// ── 3f. Call bridge ─────────────────────────────────────────────────────────

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

fn eval_non_sibling_call_raw(
    name: &str, args: &[(Option<String>, LoweredExpr)], env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    // 1. Sibling fn call (e.g. from synthetic calls inside intrinsics)
    if let Some(fn_body) = ctx.sibling_fns.get(name) {
        let mut fn_inputs = HashMap::new();
        for (param_name, arg_expr) in args {
            let value = eval_expr(arg_expr, env, ctx)?;
            if let Some(pname) = param_name { fn_inputs.insert(pname.clone(), value); }
        }
        let outputs = evaluate_stack(fn_body, &fn_inputs, ctx.sibling_fns, ctx.data_values)?;
        return sibling_fn_value_extract(name, outputs);
    }
    // 2. Intrinsics (need unevaluated args for lambdas)
    if is_intrinsic(name) {
        return eval_intrinsic_call_s(name, args, env, ctx);
    }
    // 3. scan_while (builtin that needs a lambda predicate)
    if name == "scan_while" {
        return eval_scan_while_s(args, env, ctx);
    }
    // 4. Pre-evaluate args, try builtins
    let evaluated: Vec<(Option<String>, Value)> = args.iter()
        .map(|(n, e)| Ok((n.clone(), eval_expr(e, env, ctx)?)))
        .collect::<Result<_, EvalError>>()?;
    if let Some(result) = eval_builtin_call(name, &evaluated) {
        return result;
    }
    Err(EvalError::new(format!("unknown function: {name}")))
}

fn sibling_fn_value_extract(
    _name: &str, outputs: HashMap<String, Value>,
) -> Result<Value, EvalError> {
    if let Some(v) = outputs.get("return") { return Ok(v.clone()); }
    if outputs.len() == 1 {
        if let Some(v) = outputs.get("value") { return Ok(v.clone()); }
    }
    if outputs.is_empty() { return Ok(Value::Unit); }
    Ok(Value::Map(outputs.into_iter().collect()))
}

fn eval_match_local(
    scrutinee: &Value, arms: &[LoweredMatchArm], env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, scrutinee) {
            let mut arm_env = env.child();
            for (name, val) in bindings { arm_env.bind(name, val); }
            if let Some(guard) = &arm.guard {
                let g = eval_expr(guard, &arm_env, ctx)?;
                if !value_truthy(&g) { continue; }
            }
            return eval_expr(&arm.body, &arm_env, ctx);
        }
    }
    Err(EvalError::new(format!("no matching arm for: {:?}", scrutinee)))
}

// ── Intrinsics (self-contained, no bridge to eval.rs) ────────────────────

const INTRINSIC_CALLS: &[&str] = &[
    "map", "filter", "filter_map", "flat_map", "fold", "append",
    "join", "count", "sum", "first", "last", "any", "all", "contains",
    "sort_by", "split", "zip", "skip", "enumerate",
    "starts_with", "ends_with", "repeat", "chars",
];

fn is_intrinsic(name: &str) -> bool { INTRINSIC_CALLS.contains(&name) }

fn eval_intrinsic_call_s(
    name: &str, args: &[(Option<String>, LoweredExpr)], env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    let receiver = if let Some((_, first_arg)) = args.first() {
        eval_expr(first_arg, env, ctx)?
    } else {
        return Err(EvalError::new(format!("{name}: missing receiver argument")));
    };
    let rest = &args[1..];

    match name {
        "join" => {
            let sep = if let Some((_, e)) = rest.first() {
                match eval_expr(e, env, ctx)? { Value::Str(s) => s, _ => ",".into() }
            } else { ",".into() };
            match receiver {
                Value::List(items) => Ok(Value::Str(
                    items.iter().map(value_to_string).collect::<Vec<_>>().join(&sep))),
                _ => Err(EvalError::new("join requires a list")),
            }
        }
        "map" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut out = Vec::new();
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item);
                        out.push(eval_expr(body, &c, ctx)?);
                    }
                    Ok(Value::List(out))
                }
                (Value::List(items), Some(LoweredExpr::Ident(fn_name)))
                    if ctx.sibling_fns.contains_key(fn_name.as_str()) =>
                {
                    let p = "_item".to_string();
                    let call = LoweredExpr::Call {
                        name: fn_name.clone(),
                        args: vec![(None, LoweredExpr::Ident(p.clone()))],
                    };
                    let mut out = Vec::new();
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item);
                        out.push(eval_expr(&call, &c, ctx)?);
                    }
                    Ok(Value::List(out))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                (other, _) => Err(EvalError::new(format!("map requires a list, got {other:?}"))),
            }
        }
        "filter" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut out = Vec::new();
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item.clone());
                        if value_truthy(&eval_expr(body, &c, ctx)?) { out.push(item); }
                    }
                    Ok(Value::List(out))
                }
                (Value::List(items), Some(LoweredExpr::Ident(fn_name)))
                    if ctx.sibling_fns.contains_key(fn_name.as_str()) =>
                {
                    let p = "_item".to_string();
                    let call = LoweredExpr::Call {
                        name: fn_name.clone(),
                        args: vec![(None, LoweredExpr::Ident(p.clone()))],
                    };
                    let mut out = Vec::new();
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item.clone());
                        if value_truthy(&eval_expr(&call, &c, ctx)?) { out.push(item); }
                    }
                    Ok(Value::List(out))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("filter requires a list")),
            }
        }
        "filter_map" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut out = Vec::new();
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item);
                        let val = eval_expr(body, &c, ctx)?;
                        if !matches!(val, Value::Unit | Value::Skipped) { out.push(val); }
                    }
                    Ok(Value::List(out))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("filter_map requires a list")),
            }
        }
        "flat_map" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut out = Vec::new();
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item);
                        match eval_expr(body, &c, ctx)? {
                            Value::List(inner) => out.extend(inner),
                            other => out.push(other),
                        }
                    }
                    Ok(Value::List(out))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("flat_map requires a list")),
            }
        }
        "fold" => {
            let init = rest.iter().find(|(k, _)| k.as_deref() == Some("init"));
            let func = rest.iter().find(|(k, _)| k.as_deref() == Some("f"));
            match (receiver, init, func) {
                (Value::List(items), Some((_, init_e)), Some((_, LoweredExpr::Lambda { params, body }))) => {
                    let mut acc = eval_expr(init_e, env, ctx)?;
                    let ap = params.first().cloned().unwrap_or_else(|| "acc".into());
                    let ip = params.get(1).cloned().unwrap_or_else(|| "item".into());
                    for item in items {
                        let mut c = env.child();
                        c.bind(ap.clone(), acc); c.bind(ip.clone(), item);
                        acc = eval_expr(body, &c, ctx)?;
                    }
                    Ok(acc)
                }
                _ => Err(EvalError::new("fold requires list, init, and f")),
            }
        }
        "append" => {
            let new_items = rest.iter().find(|(k, _)| k.as_deref() == Some("items"));
            match (receiver, new_items) {
                (Value::List(mut base), Some((_, e))) => {
                    match eval_expr(e, env, ctx)? {
                        Value::List(more) => base.extend(more),
                        other => base.push(other),
                    }
                    Ok(Value::List(base))
                }
                (other, _) => Err(EvalError::new(format!("append requires a list, got {other:?}"))),
            }
        }
        "count" => match receiver {
            Value::List(items) => Ok(Value::Int(items.len() as i64)),
            _ => Err(EvalError::new("count requires a list")),
        },
        "sum" => match receiver {
            Value::List(items) => {
                let total: i64 = items.iter().filter_map(|v| if let Value::Int(i) = v { Some(i) } else { None }).sum();
                Ok(Value::Int(total))
            }
            _ => Err(EvalError::new("sum requires a list")),
        },
        "first" => match receiver {
            Value::List(items) => {
                let mut m = BTreeMap::new();
                if let Some(item) = items.into_iter().next() {
                    m.insert("_variant".into(), Value::Str("Some".into()));
                    m.insert("value".into(), item);
                } else {
                    m.insert("_variant".into(), Value::Str("None".into()));
                }
                Ok(Value::Map(m))
            }
            _ => Err(EvalError::new("first requires a list")),
        },
        "last" => match receiver {
            Value::List(items) => {
                let mut m = BTreeMap::new();
                if let Some(item) = items.into_iter().last() {
                    m.insert("_variant".into(), Value::Str("Some".into()));
                    m.insert("value".into(), item);
                } else {
                    m.insert("_variant".into(), Value::Str("None".into()));
                }
                Ok(Value::Map(m))
            }
            _ => Err(EvalError::new("last requires a list")),
        },
        "any" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item);
                        if value_truthy(&eval_expr(body, &c, ctx)?) { return Ok(Value::Bool(true)); }
                    }
                    Ok(Value::Bool(false))
                }
                _ => Err(EvalError::new("any requires list and predicate")),
            }
        }
        "all" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    for item in items {
                        let mut c = env.child(); c.bind(p.clone(), item);
                        if !value_truthy(&eval_expr(body, &c, ctx)?) { return Ok(Value::Bool(false)); }
                    }
                    Ok(Value::Bool(true))
                }
                _ => Err(EvalError::new("all requires list and predicate")),
            }
        }
        "contains" => {
            let needle_expr = rest.first().or_else(|| rest.iter().find(|(k, _)| k.as_deref() == Some("item")));
            match (receiver, needle_expr) {
                (Value::List(items), Some((_, expr))) => {
                    let needle = eval_expr(expr, env, ctx)?;
                    Ok(Value::Bool(items.contains(&needle)))
                }
                _ => Err(EvalError::new("contains requires list and item")),
            }
        }
        "sort_by" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(mut items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut keyed: Vec<(String, Value)> = Vec::with_capacity(items.len());
                    for item in items.drain(..) {
                        let mut c = env.child(); c.bind(p.clone(), item.clone());
                        let key = eval_expr(body, &c, ctx).map(|v| value_to_string(&v))?;
                        keyed.push((key, item));
                    }
                    keyed.sort_by(|a, b| a.0.cmp(&b.0));
                    Ok(Value::List(keyed.into_iter().map(|(_, v)| v).collect()))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("sort_by requires a list")),
            }
        }
        "starts_with" => {
            let prefix = rest.iter().find(|(k, _)| k.as_deref() == Some("prefix")).or_else(|| rest.first());
            match (receiver, prefix) {
                (Value::Str(s), Some((_, e))) =>
                    Ok(Value::Bool(s.starts_with(&value_to_string(&eval_expr(e, env, ctx)?)))),
                _ => Err(EvalError::new("starts_with requires string and prefix")),
            }
        }
        "ends_with" => {
            let suffix = rest.iter().find(|(k, _)| k.as_deref() == Some("suffix")).or_else(|| rest.first());
            match (receiver, suffix) {
                (Value::Str(s), Some((_, e))) =>
                    Ok(Value::Bool(s.ends_with(&value_to_string(&eval_expr(e, env, ctx)?)))),
                _ => Err(EvalError::new("ends_with requires string and suffix")),
            }
        }
        "split" => {
            let delim = rest.iter().find(|(k, _)| k.as_deref() == Some("delimiter")).or_else(|| rest.first());
            match (receiver, delim) {
                (Value::Str(s), Some((_, e))) => {
                    let d = value_to_string(&eval_expr(e, env, ctx)?);
                    Ok(Value::List(s.split(&d).map(|p| Value::Str(p.to_string())).collect()))
                }
                (Value::Str(s), None) =>
                    Ok(Value::List(s.split(',').map(|p| Value::Str(p.to_string())).collect())),
                _ => Err(EvalError::new("split requires a string")),
            }
        }
        "zip" => {
            let other_expr = rest.iter().find(|(k, _)| k.as_deref() == Some("other")).or_else(|| rest.first());
            match (receiver, other_expr) {
                (Value::List(items), Some((_, e))) => {
                    let other = match eval_expr(e, env, ctx)? {
                        Value::List(l) => l,
                        _ => return Err(EvalError::new("zip requires a list for 'other'")),
                    };
                    Ok(Value::List(items.into_iter().zip(other).map(|(a, b)| {
                        let mut m = BTreeMap::new();
                        m.insert("first".into(), a); m.insert("second".into(), b);
                        Value::Map(m)
                    }).collect()))
                }
                _ => Err(EvalError::new("zip requires a list")),
            }
        }
        "skip" => {
            let n_expr = rest.iter().find(|(k, _)| k.as_deref() == Some("n")).or_else(|| rest.first());
            match (receiver, n_expr) {
                (Value::List(items), Some((_, e))) => match eval_expr(e, env, ctx)? {
                    Value::Int(count) => Ok(Value::List(items.into_iter().skip(count.max(0) as usize).collect())),
                    _ => Err(EvalError::new("skip requires integer count")),
                },
                (Value::List(items), None) => Ok(Value::List(items)),
                _ => Err(EvalError::new("skip requires a list")),
            }
        }
        "enumerate" => match receiver {
            Value::List(items) => Ok(Value::List(items.into_iter().enumerate().map(|(i, v)| {
                let mut m = BTreeMap::new();
                m.insert("first".into(), Value::Int(i as i64));
                m.insert("second".into(), v);
                Value::Map(m)
            }).collect())),
            _ => Err(EvalError::new("enumerate requires a list")),
        },
        "repeat" => {
            let n_expr = rest.first();
            match (receiver, n_expr) {
                (Value::Str(s), Some((_, e))) => match eval_expr(e, env, ctx)? {
                    Value::Int(count) => Ok(Value::Str(s.repeat(count.max(0) as usize))),
                    _ => Err(EvalError::new("repeat requires integer count")),
                },
                _ => Err(EvalError::new("repeat requires string and count")),
            }
        }
        "chars" => match &receiver {
            Value::Str(s) => Ok(Value::List(s.chars().map(|c| Value::Str(c.to_string())).collect())),
            _ => Err(EvalError::new(format!("chars: expected String, got {receiver:?}"))),
        },
        _ => Err(EvalError::new(format!("unknown intrinsic call: {name}"))),
    }
}

fn eval_scan_while_s(
    args: &[(Option<String>, LoweredExpr)], env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    let s = eval_positional_or_named_s("s", 0, args, env, ctx)?;
    let start = eval_positional_or_named_s("start", 1, args, env, ctx)?;
    let pred_expr = get_arg_expr_s("pred", 2, args);
    let resolved_pred = match pred_expr {
        Some(LoweredExpr::Lambda { params, body }) =>
            Some((params.clone(), body.as_ref().clone())),
        Some(LoweredExpr::Ident(name)) if ctx.sibling_fns.contains_key(name.as_str()) => {
            let p = "ch".to_string();
            Some((vec![p.clone()], LoweredExpr::Call {
                name: name.clone(),
                args: vec![(Some("ch".into()), LoweredExpr::Ident(p))],
            }))
        }
        _ => None,
    };
    match (s, start, resolved_pred) {
        (Value::Str(s), Value::Int(start), Some((params, body))) => {
            let p = params.first().cloned().unwrap_or_else(|| "_".into());
            let chars: Vec<char> = s.chars().collect();
            let mut pos = start.max(0) as usize;
            while pos < chars.len() {
                let mut c = env.child();
                c.bind(p.clone(), Value::Str(chars[pos].to_string()));
                if !value_truthy(&eval_expr(&body, &c, ctx)?) { break; }
                pos += 1;
            }
            Ok(Value::Int(pos as i64))
        }
        _ => Err(EvalError::new("scan_while requires (String, Int, Lambda)")),
    }
}

fn eval_positional_or_named_s(
    param: &str, index: usize, args: &[(Option<String>, LoweredExpr)],
    env: &Env, ctx: &EvalContext,
) -> Result<Value, EvalError> {
    for (name, expr) in args {
        if name.as_deref() == Some(param) { return eval_expr(expr, env, ctx); }
    }
    if let Some((_, expr)) = args.get(index) { return eval_expr(expr, env, ctx); }
    Err(EvalError::new(format!("missing argument '{param}'")))
}

fn get_arg_expr_s<'a>(
    param: &str, index: usize, args: &'a [(Option<String>, LoweredExpr)],
) -> Option<&'a LoweredExpr> {
    for (name, expr) in args {
        if name.as_deref() == Some(param) { return Some(expr); }
    }
    args.get(index).map(|(_, expr)| expr)
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════════

fn extract_projection(proj: &Projection, outputs: &HashMap<String, Value>) -> Result<Value, String> {
    match proj {
        Projection::ReturnField => {
            if let Some(v) = outputs.get("return") { return Ok(v.clone()); }
            // Legacy compat: wrap_value_as_output flattens Map trailing
            // expressions, which can produce single-field output maps with
            // a "value" key (e.g. from `Some { value: x }` variants).
            if outputs.len() == 1 {
                if let Some(v) = outputs.get("value") { return Ok(v.clone()); }
            }
            if outputs.is_empty() { return Ok(Value::Unit); }
            Ok(Value::Map(outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect()))
        }
        Projection::WholeMap =>
            Ok(Value::Map(outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect())),
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

fn step_from_error(e: EvalError) -> Step {
    if let Some(ret) = e.early_return { Step::EarlyReturn(ret) } else { Step::Error(e.message) }
}

fn expr_from_error(e: EvalError) -> ExprResult {
    if let Some(ret) = e.early_return { ExprResult::EarlyReturn(ret) } else { ExprResult::Error(e.message) }
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
        let mk = |_base_name: &str, base_val: bool, other: &str| LoweredFnBody {
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
        let inner = LoweredFnBody { stmts: vec![LoweredStmt::Return(vec![("return".into(), int(99))])] };
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
        let inner = LoweredFnBody { stmts: vec![LoweredStmt::Return(vec![("return".into(), int(42))])] };
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

    #[test] fn sibling_call_inside_if_branch() {
        // fn process(x: Int) -> Int {
        //   if x > 0 {
        //     let r = double(x: x)   // sibling call inside if-branch Block
        //     return { return: r }
        //   }
        //   return { return: 0 }
        // }
        // fn double(x: Int) -> Int { return { return: x * 2 } }
        let double_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::BinOp {
                    left: Box::new(ident("x")),
                    op: LoweredBinOp::Mul,
                    right: Box::new(int(2)),
                },
            )])],
        };
        let process_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("x")),
                        op: LoweredBinOp::Gt,
                        right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Block(vec![
                        LoweredStmt::Let("r".to_string(), call("double", vec![("x", ident("x"))])),
                        LoweredStmt::Return(vec![("return".to_string(), ident("r"))]),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Return(vec![("return".to_string(), int(0))]),
            ],
        };
        let mut sibs = HashMap::new();
        sibs.insert("double".to_string(), double_body);
        sibs.insert("process".to_string(), process_body.clone());
        let mut inp = HashMap::new();
        inp.insert("x".to_string(), Value::Int(5));
        let result = evaluate_stack(&process_body, &inp, &sibs, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Int(10));
    }

    #[test] fn anf_verifier_catches_nested_call() {
        let bad = LoweredFnBody { stmts: vec![LoweredStmt::Let("x".into(), LoweredExpr::BinOp {
            left: Box::new(call("f", vec![])), op: LoweredBinOp::Add, right: Box::new(int(1)),
        })] };
        let f_body = LoweredFnBody { stmts: vec![LoweredStmt::Return(vec![("return".into(), int(1))])] };
        let mut sibs = HashMap::new();
        sibs.insert("f".to_string(), f_body);
        let data = HashMap::new();
        let (ctx, _) = build_context(&bad, &sibs, &data);
        assert!(verify_anf_contract(&ctx).is_err());
    }

    // ── Ported builtin/intrinsic tests (Phase 3 migration) ──────────────

    #[test] fn builtin_char_at() {
        let body = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("r".into(), LoweredExpr::Call {
                name: "char_at".into(),
                args: vec![
                    (Some("s".into()), LoweredExpr::Literal(LoweredLiteral::String("hello".into()))),
                    (Some("pos".into()), int(1)),
                ],
            }),
            LoweredStmt::Return(vec![("return".into(), ident("r"))]),
        ]};
        assert_eq!(evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()["return"], Value::Str("e".into()));
    }

    #[test] fn builtin_scan_while_with_lambda() {
        // scan_while(s: "123abc", start: 0, pred: c => code_point(c) >= code_point("0") && code_point(c) <= code_point("9"))
        let body = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("r".into(), LoweredExpr::Call {
                name: "scan_while".into(),
                args: vec![
                    (Some("s".into()), LoweredExpr::Literal(LoweredLiteral::String("123abc".into()))),
                    (Some("start".into()), int(0)),
                    (Some("pred".into()), LoweredExpr::Lambda {
                        params: vec!["c".into()],
                        body: Box::new(LoweredExpr::BinOp {
                            left: Box::new(LoweredExpr::BinOp {
                                left: Box::new(LoweredExpr::Call { name: "code_point".into(),
                                    args: vec![(Some("c".into()), ident("c"))] }),
                                op: LoweredBinOp::Ge,
                                right: Box::new(LoweredExpr::Call { name: "code_point".into(),
                                    args: vec![(Some("c".into()), LoweredExpr::Literal(LoweredLiteral::String("0".into())))] }),
                            }),
                            op: LoweredBinOp::And,
                            right: Box::new(LoweredExpr::BinOp {
                                left: Box::new(LoweredExpr::Call { name: "code_point".into(),
                                    args: vec![(Some("c".into()), ident("c"))] }),
                                op: LoweredBinOp::Le,
                                right: Box::new(LoweredExpr::Call { name: "code_point".into(),
                                    args: vec![(Some("c".into()), LoweredExpr::Literal(LoweredLiteral::String("9".into())))] }),
                            }),
                        }),
                    }),
                ],
            }),
            LoweredStmt::Return(vec![("return".into(), ident("r"))]),
        ]};
        assert_eq!(evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()["return"], Value::Int(3));
    }

    #[test] fn intrinsic_map_with_lambda() {
        // map([1,2,3], x => x * 2)
        let body = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("r".into(), LoweredExpr::Call {
                name: "map".into(),
                args: vec![
                    (None, LoweredExpr::List(vec![int(1), int(2), int(3)])),
                    (None, LoweredExpr::Lambda {
                        params: vec!["x".into()],
                        body: Box::new(LoweredExpr::BinOp {
                            left: Box::new(ident("x")), op: LoweredBinOp::Mul, right: Box::new(int(2)),
                        }),
                    }),
                ],
            }),
            LoweredStmt::Return(vec![("return".into(), ident("r"))]),
        ]};
        assert_eq!(evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()["return"],
            Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
    }

    #[test] fn intrinsic_fold() {
        // fold([1,2,3], init: 0, f: (acc, x) => acc + x)
        let body = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("r".into(), LoweredExpr::Call {
                name: "fold".into(),
                args: vec![
                    (None, LoweredExpr::List(vec![int(1), int(2), int(3)])),
                    (Some("init".into()), int(0)),
                    (Some("f".into()), LoweredExpr::Lambda {
                        params: vec!["acc".into(), "x".into()],
                        body: Box::new(LoweredExpr::BinOp {
                            left: Box::new(ident("acc")), op: LoweredBinOp::Add, right: Box::new(ident("x")),
                        }),
                    }),
                ],
            }),
            LoweredStmt::Return(vec![("return".into(), ident("r"))]),
        ]};
        assert_eq!(evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()["return"], Value::Int(6));
    }

    #[test] fn builtin_lookup() {
        let mut inp = HashMap::new();
        let mut m = std::collections::BTreeMap::new();
        m.insert("x".to_string(), Value::Int(42));
        inp.insert("m".into(), Value::Map(m));
        let body = LoweredFnBody { stmts: vec![
            LoweredStmt::Let("r".into(), LoweredExpr::Call {
                name: "lookup".into(),
                args: vec![
                    (Some("map".into()), ident("m")),
                    (Some("key".into()), LoweredExpr::Literal(LoweredLiteral::String("x".into()))),
                ],
            }),
            LoweredStmt::Return(vec![("return".into(), ident("r"))]),
        ]};
        let r = evaluate_stack(&body, &inp, &HashMap::new(), &HashMap::new()).unwrap();
        let result = &r["return"];
        // Should be Some { value: 42 }
        if let Value::Map(map) = result {
            assert_eq!(map.get("_variant"), Some(&Value::Str("Some".into())));
            assert_eq!(map.get("value"), Some(&Value::Int(42)));
        } else { panic!("expected Map, got {result:?}"); }
    }
}
