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
use std::sync::Arc;

use gunbc_ir::value_compatible_with_type_id;
use gunbc_ir::Value;

use crate::eval_core::{
    eval_binop, eval_builtin_call, eval_get_field, eval_literal, match_pattern, sort_key,
    value_to_string, value_truthy, EvalError,
};
use crate::expr::{
    LoweredBinOp, LoweredExpr, LoweredFnBody, LoweredMatchArm, LoweredStmt, LoweredStringPart,
    LoweredUnaryOp,
};

// ── Limits ──────────────────────────────────────────────────────────────────

const MAX_STACK_DEPTH: usize = 100_000;
const MAX_TRANSITIONS: usize = 10_000_000;

// ── Runtime type boundary checks (S57) ───────────────────────────────────
//
// Soft diagnostics: mismatches are collected via an explicit `&mut Vec<String>`
// threaded through the evaluation functions and returned as part of `EvalOutcome`.

/// The result of a successful evaluation, including any type-boundary warnings.
#[derive(Debug, Clone)]
pub struct EvalOutcome {
    /// The output values from the evaluated function.
    pub outputs: HashMap<String, Value>,
    /// S57 type-boundary warnings collected during evaluation.
    /// Empty when all runtime types match declared types.
    pub warnings: Vec<String>,
}

/// Reconstruct the value that callers observe from an output map.
///
/// This must stay in sync with sibling-call projection so runtime boundary
/// checks validate the same shape that downstream code actually receives.
fn output_value(outputs: &HashMap<String, Value>) -> Value {
    if let Some(value) = outputs.get("return") {
        return value.clone();
    }
    if outputs.len() == 1 {
        if let Some(value) = outputs.get("value") {
            // Legacy "value" fallback — all functions should return through
            // the "return" key. This path exists for backward compatibility
            // and should be eliminated once all callers are migrated.
            return value.clone();
        }
    }
    assert!(
        !outputs.is_empty(),
        "BUG: output_value called with empty outputs map — \
         all functions must produce at least one output",
    );
    Value::Map(
        outputs
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}

/// Check that each argument value is compatible with the callee's declared
/// parameter type. Collects a warning for each mismatch.
fn check_call_inputs(
    fn_body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    fn_name: &str,
    warnings: &mut Vec<String>,
) {
    for (param_name, expected_type) in &fn_body.param_types {
        if let Some(value) = inputs.get(param_name) {
            if !value_compatible_with_type_id(expected_type, value) {
                warnings.push(format!(
                    "[S57] type mismatch at call to `{fn_name}`: \
                     param `{param_name}` expects `{expected_type}`, \
                     got `{}`",
                    value.kind().type_name(),
                ));
            }
        }
    }
}

/// Check that the return value is compatible with the callee's declared
/// return type. Collects a warning on mismatch.
fn check_return_value(
    fn_body: &LoweredFnBody,
    result: &HashMap<String, Value>,
    fn_name: &str,
    warnings: &mut Vec<String>,
) {
    let Some(expected_type) = &fn_body.return_type else {
        return;
    };
    let value = output_value(result);
    if !value_compatible_with_type_id(expected_type, &value) {
        warnings.push(format!(
            "[S57] type mismatch at return from `{fn_name}`: \
             expects `{expected_type}`, got `{}`",
            value.kind().type_name(),
        ));
    }
}

/// Look up the human-readable name for a FnId from the context.
fn fn_name_for_id(fn_id: FnId, ctx: &EvalContext) -> String {
    for (name, &id) in &ctx.fn_index {
        if id == fn_id {
            return (*name).to_string();
        }
    }
    format!("<fn#{fn_id}>")
}

// ═══════════════════════════════════════════════════════════════════════════
// Public entry point
// ═══════════════════════════════════════════════════════════════════════════

/// Evaluate a fn body, returning outputs and type-boundary warnings.
///
/// This is the preferred entry point — warnings are part of the return type.
pub fn evaluate_stack_with_diagnostics(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<EvalOutcome, EvalError> {
    let (ctx, entry) = build_context(body, sibling_fns, data_values);

    if let Err(msg) = verify_anf_contract(&ctx) {
        return Err(EvalError::new(format!("ANF contract violated: {msg}")));
    }

    let mut warnings = Vec::new();
    let outputs = run_machine(entry, inputs, &ctx, &mut warnings)?;
    Ok(EvalOutcome { outputs, warnings })
}

/// Evaluate a fn body. Pipeline: build context → verify contract → run machine.
///
/// Returns only the output map. Use `evaluate_stack_with_diagnostics` to also
/// receive S57 type-boundary warnings.
pub fn evaluate_stack(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, EvalError> {
    let outcome = evaluate_stack_with_diagnostics(body, inputs, sibling_fns, data_values)?;
    Ok(outcome.outputs)
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
    (
        EvalContext {
            fns,
            fn_index,
            data_values,
            sibling_fns,
        },
        entry_id,
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Stage 2 — Verify ANF contract
// ═══════════════════════════════════════════════════════════════════════════

/// Pure: EvalContext → Ok(()) or Err(location).
/// Asserts no SIBLING fn Call nested inside another expression.
///
/// Note: This is intentionally weaker than the lowerer's ANF verifier
/// (daglang-lower::anf) which rejects ALL nested calls. The runtime
/// verifier only checks the suspension-critical invariant: sibling calls
/// must be at statement level for the continuation-based call protocol.
/// Non-sibling calls (builtins, intrinsics) are evaluated inline by
/// eval_expr and don't need hoisting.
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
            for (_, a) in args {
                no_sibling_call(a, loc, sibs)?;
            }
            Ok(())
        }
        LoweredStmt::Let(_, e) => no_sibling_call(e, loc, sibs),
        LoweredStmt::Expr(e) => no_sibling_call_in_branch(e, loc, sibs),
        LoweredStmt::Return(fields) => {
            for (_, e) in fields {
                no_sibling_call(e, loc, sibs)?;
            }
            Ok(())
        }
    }
}

fn no_sibling_call(
    expr: &LoweredExpr,
    loc: &str,
    sibs: &HashMap<&str, FnId>,
) -> Result<(), String> {
    match expr {
        LoweredExpr::Call { name, .. } => {
            // Reject ALL nested calls, matching the lowerer's ANF verifier
            // (anf.rs:verify_no_call). Previously only sibling calls were
            // rejected here, but the lowerer rejects all calls in non-statement
            // position, so the runtime check should be equally strict.
            Err(format!("ANF violation at {loc}: nested Call to '{name}'"))
        }
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => Ok(()),
        LoweredExpr::FieldAccess { expr, .. } | LoweredExpr::UnaryOp { expr, .. } => {
            no_sibling_call(expr, loc, sibs)
        }
        LoweredExpr::BinOp { left, right, .. } => {
            no_sibling_call(left, loc, sibs)?;
            no_sibling_call(right, loc, sibs)
        }
        LoweredExpr::StringInterp(ps) => {
            for p in ps {
                if let LoweredStringPart::Expr(e) = p {
                    no_sibling_call(e, loc, sibs)?;
                }
            }
            Ok(())
        }
        LoweredExpr::IfElse { cond, then_, else_ } => {
            no_sibling_call(cond, loc, sibs)?;
            no_sibling_call_in_branch(then_, loc, sibs)?;
            if let Some(e) = else_ {
                no_sibling_call_in_branch(e, loc, sibs)?;
            }
            Ok(())
        }
        LoweredExpr::Match { expr, arms } => {
            no_sibling_call(expr, loc, sibs)?;
            for a in arms {
                if let Some(g) = &a.guard {
                    no_sibling_call_in_branch(g, loc, sibs)?;
                }
                no_sibling_call_in_branch(&a.body, loc, sibs)?;
            }
            Ok(())
        }
        // Lambda bodies are evaluated on the pure eval_expr path by intrinsic
        // handlers (map, filter, scan_while, etc.). The ANF normalizer hoists
        // calls to statement level within blocks inside lambdas. Sibling calls
        // in lambda bodies would be evaluated re-entrantly via evaluate_stack,
        // which is correct but breaks the contract that the pure path should
        // not need the continuation stack. Tighten: reject sibling calls.
        LoweredExpr::Lambda { body, .. } => {
            no_sibling_call_in_branch(body, &format!("{loc}/Lambda"), sibs)
        }
        LoweredExpr::List(xs) => {
            for x in xs {
                no_sibling_call(x, loc, sibs)?;
            }
            Ok(())
        }
        LoweredExpr::Block(ss) => {
            for s in ss {
                check_stmt_anf(s, loc, sibs)?;
            }
            Ok(())
        }
        LoweredExpr::Record { fields, .. } | LoweredExpr::VariantConstruct { fields, .. } => {
            for (_, e) in fields {
                no_sibling_call(e, loc, sibs)?;
            }
            Ok(())
        }
        LoweredExpr::For { iterable, body, .. } => {
            no_sibling_call(iterable, loc, sibs)?;
            no_sibling_call_in_branch(body, loc, sibs)
        }
    }
}

fn no_sibling_call_in_branch(
    expr: &LoweredExpr,
    loc: &str,
    sibs: &HashMap<&str, FnId>,
) -> Result<(), String> {
    match expr {
        LoweredExpr::Block(ss) => {
            for s in ss {
                check_stmt_anf(s, loc, sibs)?;
            }
            Ok(())
        }
        _ => no_sibling_call(expr, loc, sibs),
    }
}

/// Debug-only check: verify that a block's statements contain no sibling calls.
///
/// Blocks that reach `eval_expr` (the pure, non-suspendable path) come from
/// only two sources:
///   1. Lambda bodies — evaluated inside intrinsics (map, filter, etc.)
///   2. Match arm bodies — evaluated by `eval_match_local`
///
/// In both cases, sibling calls are handled re-entrantly via
/// `eval_non_sibling_call_raw` → `evaluate_stack`, not via the continuation
/// stack. The ANF contract guarantees that sibling calls don't appear nested
/// inside these blocks. This function makes that invariant explicit as a
/// debug assertion.
#[cfg(debug_assertions)]
fn debug_assert_no_sibling_calls_in_block(stmts: &[LoweredStmt], sibs: &HashMap<&str, FnId>) {
    for (i, stmt) in stmts.iter().enumerate() {
        let loc = format!("eval_expr/Block/stmt[{i}]");
        if let Err(msg) = check_stmt_anf(stmt, &loc, sibs) {
            panic!(
                "BUG: sibling call found in block evaluated by eval_expr (pure path). \
                 This block should only contain non-sibling calls. {msg}"
            );
        }
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
    Call {
        callee: FnId,
        inputs: HashMap<String, Value>,
    },
    /// Error.
    Error(String),
}

#[allow(clippy::large_enum_variant)]
enum ExprResult {
    Value(Value),
    EarlyReturn(HashMap<String, Value>),
    Suspend {
        callee: FnId,
        inputs: HashMap<String, Value>,
    },
    Error(String),
}

#[derive(Clone)]
struct Continuation<'a> {
    remaining: &'a [LoweredStmt],
    binding: Option<String>,
    env: Env,
    is_fn_boundary: bool,
    /// The fn_id of the caller that pushed this continuation. Bound directly
    /// to the frame so `pop_stack` can restore it automatically — preventing
    /// desync between the continuation stack and a parallel fn_id tracker.
    caller_fn: FnId,
}

// ── 3b. Main loop ───────────────────────────────────────────────────────────

fn run_machine<'a>(
    entry: FnId,
    inputs: &HashMap<String, Value>,
    ctx: &'a EvalContext<'a>,
    warnings: &mut Vec<String>,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut stack: Vec<Continuation<'a>> = Vec::new();
    let mut stmts: &'a [LoweredStmt] = &ctx.fns[entry].stmts;
    let mut env = Env::from_inputs(inputs);
    let mut transitions: usize = 0;
    // S57: track the current executing function. The caller's fn_id is bound
    // directly to each Continuation frame (caller_fn field), so pop_stack
    // restores it automatically — no parallel fn_stack needed.
    let mut current_fn: FnId = entry;

    // S57: check entry function inputs
    check_call_inputs(
        ctx.fns[entry],
        inputs,
        &fn_name_for_id(entry, ctx),
        warnings,
    );

    loop {
        transitions += 1;
        if transitions > MAX_TRANSITIONS {
            return Err(EvalError::new(format!(
                "transition budget ({MAX_TRANSITIONS}) exceeded"
            )));
        }

        match eval_stmts(stmts, &mut env, ctx, &mut stack, current_fn) {
            Step::Return(result) => {
                // S57: check return type of the completing function
                check_return_value(
                    ctx.fns[current_fn],
                    &result,
                    &fn_name_for_id(current_fn, ctx),
                    warnings,
                );
                match pop_stack(&mut stack, result, ctx, warnings) {
                    PopResult::Done(output) => return Ok(output),
                    PopResult::Resume {
                        stmts: s,
                        env: e,
                        fn_id,
                    } => {
                        stmts = s;
                        env = e;
                        current_fn = fn_id;
                    }
                }
            }
            Step::EarlyReturn(result) => {
                // S57: check return type of the completing function
                check_return_value(
                    ctx.fns[current_fn],
                    &result,
                    &fn_name_for_id(current_fn, ctx),
                    warnings,
                );
                // Unwind past block-resume continuations to the fn boundary.
                while let Some(cont) = stack.last() {
                    if cont.is_fn_boundary {
                        break;
                    }
                    stack.pop();
                }
                match pop_stack(&mut stack, result, ctx, warnings) {
                    PopResult::Done(output) => return Ok(output),
                    PopResult::Resume {
                        stmts: s,
                        env: e,
                        fn_id,
                    } => {
                        stmts = s;
                        env = e;
                        current_fn = fn_id;
                    }
                }
            }
            Step::Call { callee, inputs } => {
                if stack.len() >= MAX_STACK_DEPTH {
                    return Err(EvalError::new(format!(
                        "max stack depth ({MAX_STACK_DEPTH}) exceeded"
                    )));
                }
                // S57: check callee input types before transitioning
                check_call_inputs(
                    ctx.fns[callee],
                    &inputs,
                    &fn_name_for_id(callee, ctx),
                    warnings,
                );
                current_fn = callee;
                stmts = &ctx.fns[callee].stmts;
                env = Env::from_inputs(&inputs);
            }
            Step::Error(msg) => return Err(EvalError::new(msg)),
        }
    }
}

enum PopResult<'a> {
    Done(HashMap<String, Value>),
    Resume {
        stmts: &'a [LoweredStmt],
        env: Env,
        fn_id: FnId,
    },
}

fn pop_stack<'a>(
    stack: &mut Vec<Continuation<'a>>,
    mut result: HashMap<String, Value>,
    ctx: &'a EvalContext<'a>,
    warnings: &mut Vec<String>,
) -> PopResult<'a> {
    loop {
        match stack.pop() {
            None => return PopResult::Done(result),
            Some(cont) => {
                // S57: return type checks happen in run_machine before
                // pop_stack is called, so no per-frame check needed here.
                let value = output_value(&result);
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
                    // If this was a fn boundary that collapsed (remaining empty),
                    // check the return value for the function we're collapsing
                    // through. The caller's fn_id is on cont.caller_fn.
                    if cont.is_fn_boundary {
                        let caller_name = fn_name_for_id(cont.caller_fn, ctx);
                        check_return_value(
                            ctx.fns[cont.caller_fn],
                            &result,
                            &caller_name,
                            warnings,
                        );
                    }
                } else {
                    return PopResult::Resume {
                        stmts: cont.remaining,
                        env,
                        fn_id: cont.caller_fn,
                    };
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
    current_fn: FnId,
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
                                    remaining,
                                    binding: Some(name.clone()),
                                    env: env.clone(),
                                    is_fn_boundary: true,
                                    caller_fn: current_fn,
                                });
                                return Step::Call {
                                    callee: callee_id,
                                    inputs,
                                };
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, env, ctx) {
                            Ok(value) => bind_let_result(env, name.clone(), &value),
                            Err(e) => return Step::Error(e.message),
                        }
                    }
                } else {
                    match eval_expr_s(expr, env, ctx, stack, current_fn, false) {
                        ExprResult::Value(value) => bind_let_result(env, name.clone(), &value),
                        ExprResult::EarlyReturn(map) => return Step::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(
                                stack_base,
                                Continuation {
                                    remaining,
                                    binding: Some(name.clone()),
                                    env: env.clone(),
                                    is_fn_boundary: false,
                                    caller_fn: current_fn,
                                },
                            );
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
                                // Tail call elimination: when this is the last
                                // statement (remaining is empty) and we have no
                                // binding, the continuation would be an identity
                                // frame that just collapses in pop_stack. Skip
                                // it to avoid O(call_depth) heap growth for tail
                                // recursive patterns.
                                if !remaining.is_empty() {
                                    stack.push(Continuation {
                                        remaining,
                                        binding: None,
                                        env: env.clone(),
                                        is_fn_boundary: true,
                                        caller_fn: current_fn,
                                    });
                                }
                                return Step::Call {
                                    callee: callee_id,
                                    inputs,
                                };
                            }
                            Err(msg) => return Step::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, env, ctx) {
                            Ok(value) if is_last => {
                                return Step::Return(wrap_value_as_output(value))
                            }
                            Ok(_) => {}
                            Err(e) => return Step::Error(e.message),
                        }
                    }
                } else if is_last {
                    // Tail position: pass true so inner if/match/block
                    // suspends skip their identity continuations too.
                    match eval_expr_s(expr, env, ctx, stack, current_fn, true) {
                        ExprResult::Value(value) => {
                            return Step::Return(wrap_value_as_output(value))
                        }
                        ExprResult::EarlyReturn(map) => return Step::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            // No identity continuation needed — this is
                            // the tail position of the function body.
                            return Step::Call { callee, inputs };
                        }
                        ExprResult::Error(msg) => return Step::Error(msg),
                    }
                } else {
                    match eval_expr_s(expr, env, ctx, stack, current_fn, false) {
                        ExprResult::Value(_) => {}
                        ExprResult::EarlyReturn(map) => return Step::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(
                                stack_base,
                                Continuation {
                                    remaining,
                                    binding: None,
                                    env: env.clone(),
                                    is_fn_boundary: false,
                                    caller_fn: current_fn,
                                },
                            );
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
                        Ok(v) => {
                            result.insert(name.clone(), v);
                        }
                        Err(e) => return Step::Error(e.message),
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
    current_fn: FnId,
    is_tail_position: bool,
) -> ExprResult {
    stacker::maybe_grow(32 * 1024, 2 * 1024 * 1024, || {
        eval_expr_s_inner(expr, env, ctx, stack, current_fn, is_tail_position)
    })
}

fn eval_expr_s_inner<'a>(
    expr: &'a LoweredExpr,
    env: &Env,
    ctx: &'a EvalContext<'a>,
    stack: &mut Vec<Continuation<'a>>,
    current_fn: FnId,
    is_tail_position: bool,
) -> ExprResult {
    match expr {
        LoweredExpr::IfElse { cond, then_, else_ } => match eval_expr(cond, env, ctx) {
            Ok(c) => {
                let branch = if value_truthy(&c) {
                    Some(then_.as_ref())
                } else {
                    else_.as_ref().map(|e| e.as_ref())
                };
                match branch {
                    Some(b) => eval_expr_s(b, env, ctx, stack, current_fn, is_tail_position),
                    None => ExprResult::Value(Value::Unit),
                }
            }
            Err(e) => ExprResult::Error(e.message),
        },
        LoweredExpr::Match {
            expr: scrutinee,
            arms,
        } => match eval_expr(scrutinee, env, ctx) {
            Ok(val) => eval_match_s(&val, arms, env, ctx, stack, current_fn, is_tail_position),
            Err(e) => ExprResult::Error(e.message),
        },
        LoweredExpr::Block(block_stmts) => {
            eval_block_s(block_stmts, env, ctx, stack, current_fn, is_tail_position)
        }
        _ => match eval_expr(expr, env, ctx) {
            Ok(v) => ExprResult::Value(v),
            Err(e) => ExprResult::Error(e.message),
        },
    }
}

fn eval_match_s<'a>(
    scrutinee: &Value,
    arms: &'a [LoweredMatchArm],
    env: &Env,
    ctx: &'a EvalContext<'a>,
    stack: &mut Vec<Continuation<'a>>,
    current_fn: FnId,
    is_tail_position: bool,
) -> ExprResult {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, scrutinee) {
            let mut arm_env = env.child();
            for (name, val) in bindings {
                arm_env.bind(name, val);
            }
            if let Some(guard) = &arm.guard {
                // Guards use eval_expr (pure path), not eval_expr_s. A guard
                // that contains a sibling call will evaluate it via
                // eval_non_sibling_call_raw → evaluate_stack (re-entrant).
                // This is correct but uses native recursion for the guard.
                //
                // Using eval_expr_s here would be wrong: if the guard suspends,
                // the continuation model can't represent "check truthiness of
                // the returned value, then maybe try the next arm." The guard
                // result would be misinterpreted as the match result.
                match eval_expr(guard, &arm_env, ctx) {
                    Ok(g) if value_truthy(&g) => {}
                    Ok(_) => continue,
                    Err(e) => return ExprResult::Error(e.message),
                }
            }
            return eval_expr_s(
                &arm.body,
                &arm_env,
                ctx,
                stack,
                current_fn,
                is_tail_position,
            );
        }
    }
    ExprResult::Error(format!("no matching arm for: {scrutinee:?}"))
}

fn eval_block_s<'a>(
    block_stmts: &'a [LoweredStmt],
    env: &Env,
    ctx: &'a EvalContext<'a>,
    stack: &mut Vec<Continuation<'a>>,
    current_fn: FnId,
    is_tail_position: bool,
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
                                stack.insert(
                                    stack_base,
                                    Continuation {
                                        remaining,
                                        binding: Some(name.clone()),
                                        env: child,
                                        is_fn_boundary: true,
                                        caller_fn: current_fn,
                                    },
                                );
                                return ExprResult::Suspend {
                                    callee: callee_id,
                                    inputs,
                                };
                            }
                            Err(msg) => return ExprResult::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, &child, ctx) {
                            Ok(value) => bind_let_result(&mut child, name.clone(), &value),
                            Err(e) => return ExprResult::Error(e.message),
                        }
                    }
                } else {
                    match eval_expr_s(expr, &child, ctx, stack, current_fn, false) {
                        ExprResult::Value(value) => {
                            bind_let_result(&mut child, name.clone(), &value)
                        }
                        other @ (ExprResult::EarlyReturn(_) | ExprResult::Error(_)) => {
                            return other
                        }
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(
                                stack_base,
                                Continuation {
                                    remaining,
                                    binding: Some(name.clone()),
                                    env: child,
                                    is_fn_boundary: false,
                                    caller_fn: current_fn,
                                },
                            );
                            return ExprResult::Suspend { callee, inputs };
                        }
                    }
                }
            }
            LoweredStmt::Expr(expr) => {
                let tail_ctx = is_last && is_tail_position;
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(&callee_id) = ctx.fn_index.get(callee.as_str()) {
                        match eval_call_args(args, &child, ctx) {
                            Ok(inputs) => {
                                if !tail_ctx {
                                    stack.insert(
                                        stack_base,
                                        Continuation {
                                            remaining,
                                            binding: None,
                                            env: child,
                                            is_fn_boundary: true,
                                            caller_fn: current_fn,
                                        },
                                    );
                                }
                                return ExprResult::Suspend {
                                    callee: callee_id,
                                    inputs,
                                };
                            }
                            Err(msg) => return ExprResult::Error(msg),
                        }
                    } else {
                        match eval_non_sibling_call_raw(callee, args, &child, ctx) {
                            Ok(value) if is_last => return ExprResult::Value(value),
                            Ok(_) => {}
                            Err(e) => return ExprResult::Error(e.message),
                        }
                    }
                } else if is_last {
                    return eval_expr_s(expr, &child, ctx, stack, current_fn, tail_ctx);
                } else {
                    match eval_expr_s(expr, &child, ctx, stack, current_fn, false) {
                        ExprResult::Value(_) => {}
                        ExprResult::EarlyReturn(map) => return ExprResult::EarlyReturn(map),
                        ExprResult::Suspend { callee, inputs } => {
                            stack.insert(
                                stack_base,
                                Continuation {
                                    remaining,
                                    binding: None,
                                    env: child,
                                    is_fn_boundary: false,
                                    caller_fn: current_fn,
                                },
                            );
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
                        Ok(v) => {
                            result.insert(name.clone(), v);
                        }
                        Err(e) => return ExprResult::Error(e.message),
                    }
                }
                return ExprResult::EarlyReturn(result);
            }
        }
    }
    ExprResult::Value(Value::Unit)
}

// ── 3e. Pure expression evaluation ──────────────────────────────────────────
//
// `eval_expr` is the non-suspendable expression evaluator. It runs on the
// native Rust call stack and CANNOT push continuations or suspend for sibling
// calls. Any calls encountered are evaluated re-entrantly via
// `eval_non_sibling_call_raw` → `evaluate_stack`.
//
// Block and For arms in this function are reached ONLY from:
//   - Lambda bodies (via intrinsic evaluation: map, filter, etc.)
//   - Match arm bodies in `eval_match_local`
//   - Nested expressions that `eval_expr_s` delegates here
//
// The ANF contract guarantees these blocks contain no sibling calls. Debug
// assertions enforce this invariant explicitly (see `debug_assert_no_sibling_calls_in_block`).
//
// The suspendable counterpart is `eval_expr_s`, which intercepts Block,
// IfElse, and Match before they reach this function, routing them through
// `eval_block_s` / `eval_match_s` which can push continuations.

fn eval_expr(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    stacker::maybe_grow(32 * 1024, 2 * 1024 * 1024, || {
        eval_expr_inner(expr, env, ctx)
    })
}

fn eval_expr_inner(expr: &LoweredExpr, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    match expr {
        LoweredExpr::Literal(lit) => Ok(eval_literal(lit)),
        LoweredExpr::Ident(name) => eval_ident(name, env, ctx),
        LoweredExpr::FieldAccess { expr, field } => {
            eval_get_field(&eval_expr(expr, env, ctx)?, field)
        }
        LoweredExpr::StringInterp(parts) => {
            let mut s = String::new();
            for p in parts {
                match p {
                    LoweredStringPart::Literal(lit) => s.push_str(lit),
                    LoweredStringPart::Expr(e) => {
                        s.push_str(&value_to_string(&eval_expr(e, env, ctx)?))
                    }
                }
            }
            Ok(Value::Str(s))
        }
        LoweredExpr::BinOp { left, op, right } => {
            let lhs = eval_expr(left, env, ctx)?;
            match op {
                LoweredBinOp::And => {
                    if !value_truthy(&lhs) {
                        return Ok(Value::Bool(false));
                    }
                    Ok(Value::Bool(value_truthy(&eval_expr(right, env, ctx)?)))
                }
                LoweredBinOp::Or => {
                    if value_truthy(&lhs) {
                        return Ok(Value::Bool(true));
                    }
                    Ok(Value::Bool(value_truthy(&eval_expr(right, env, ctx)?)))
                }
                LoweredBinOp::NullCoalesce => {
                    if !matches!(lhs, Value::Unit | Value::Skipped) {
                        Ok(lhs)
                    } else {
                        eval_expr(right, env, ctx)
                    }
                }
                LoweredBinOp::Add => {
                    let rhs = eval_expr(right, env, ctx)?;
                    match (lhs, rhs) {
                        (Value::List(a), Value::List(b)) => {
                            let mut v = Arc::try_unwrap(a).unwrap_or_else(|rc| (*rc).clone());
                            v.extend(b.iter().cloned());
                            Ok(Value::List(Arc::new(v)))
                        }
                        (Value::Str(mut a), Value::Str(b)) => {
                            a.push_str(&b);
                            Ok(Value::Str(a))
                        }
                        (Value::Str(mut a), Value::Enum { variant, .. }) => {
                            a.push_str(&variant);
                            Ok(Value::Str(a))
                        }
                        (Value::Enum { variant, .. }, Value::Str(b)) => {
                            Ok(Value::Str(format!("{variant}{b}")))
                        }
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
            if value_truthy(&eval_expr(cond, env, ctx)?) {
                eval_expr(then_, env, ctx)
            } else if let Some(e) = else_ {
                eval_expr(e, env, ctx)
            } else {
                Ok(Value::Unit)
            }
        }
        // Non-suspendable match. See eval_block_pure for rationale on why
        // this is not a parallel implementation of eval_match_s.
        LoweredExpr::Match {
            expr: scrutinee,
            arms,
        } => {
            let val = eval_expr(scrutinee, env, ctx)?;
            eval_match_local(&val, arms, env, ctx)
        }
        LoweredExpr::VariantConstruct { tag, fields } => {
            if fields.is_empty() {
                Ok(Value::Enum {
                    ty: String::new(),
                    variant: tag.clone(),
                })
            } else {
                let mut map = BTreeMap::new();
                map.insert("_variant".to_string(), Value::Str(tag.clone()));
                for (k, v) in fields {
                    map.insert(k.clone(), eval_expr(v, env, ctx)?);
                }
                Ok(Value::Map(map))
            }
        }
        LoweredExpr::Call { name, args } => eval_non_sibling_call_raw(name, args, env, ctx),
        LoweredExpr::Lambda { .. } => Err(EvalError::new("lambda cannot be evaluated standalone")),
        LoweredExpr::List(items) => items
            .iter()
            .map(|i| eval_expr(i, env, ctx))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| Value::List(Arc::new(v))),
        // Non-suspendable block evaluation. Used for intrinsic lambda bodies
        // and standalone match arms where no continuation stack exists.
        // The suspendable path (eval_block_s) handles blocks at statement
        // level in the main loop via the continuation stack.
        //
        // S67: These are not parallel implementations — they serve different
        // contexts. This path evaluates blocks where sibling calls are
        // resolved re-entrantly (via eval_non_sibling_call_raw → evaluate_stack).
        // The suspendable path resolves them via Step::Call + continuations.
        LoweredExpr::Block(stmts) => eval_block_pure(stmts, env, ctx),
        LoweredExpr::Record { fields, .. } => {
            let mut map = BTreeMap::new();
            for (k, v) in fields {
                map.insert(k.clone(), eval_expr(v, env, ctx)?);
            }
            Ok(Value::Map(map))
        }
        // Pure for-loop evaluation — same constraints as Block above.
        // The body must not contain sibling calls; they would need the
        // continuation stack which is unavailable on this path.
        LoweredExpr::For {
            binding,
            iterable,
            body,
        } => {
            #[cfg(debug_assertions)]
            {
                let loc = "eval_expr/For/body";
                if let Err(msg) = no_sibling_call(body, loc, &ctx.fn_index) {
                    panic!(
                        "BUG: sibling call found in For body evaluated by eval_expr (pure path). \
                         For bodies with sibling calls must go through eval_expr_s. {msg}"
                    );
                }
            }
            match eval_expr(iterable, env, ctx)? {
                Value::List(list) => {
                    let mut results = Vec::with_capacity(list.len());
                    for item in list.iter() {
                        let mut iter_env = env.child();
                        iter_env.bind(binding.clone(), item.clone());
                        results.push(eval_expr(body, &iter_env, ctx)?);
                    }
                    Ok(Value::List(Arc::new(results)))
                }
                other => Err(EvalError::new(format!(
                    "for requires list, got {:?}",
                    other
                ))),
            }
        }
    }
}

fn eval_ident(name: &str, env: &Env, ctx: &EvalContext) -> Result<Value, EvalError> {
    if name == "None" || name == "null" {
        return Ok(Value::Unit);
    }
    if let Some(val) = env.get(name) {
        return Ok(val.clone());
    }
    if let Some(val) = ctx.data_values.get(name) {
        return Ok(val.clone());
    }
    if name.chars().next().unwrap_or('a').is_uppercase() {
        return Ok(Value::Str(name.to_string()));
    }
    Err(EvalError::new(format!("unbound variable: {name}")))
}

// ── 3f. Call bridge ─────────────────────────────────────────────────────────

fn eval_call_args(
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
) -> Result<HashMap<String, Value>, String> {
    let mut inputs = HashMap::new();
    let mut pos_idx = 0usize;
    for (param, arg_expr) in args {
        let value = eval_expr(arg_expr, env, ctx).map_err(|e| e.message)?;
        let key = match param {
            Some(name) => name.clone(),
            None => {
                let k = format!("__pos_{pos_idx}");
                pos_idx += 1;
                k
            }
        };
        inputs.insert(key, value);
    }
    Ok(inputs)
}

fn eval_non_sibling_call_raw(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
) -> Result<Value, EvalError> {
    stacker::maybe_grow(32 * 1024, 2 * 1024 * 1024, || {
        eval_non_sibling_call_inner(name, args, env, ctx)
    })
}

fn eval_non_sibling_call_inner(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
) -> Result<Value, EvalError> {
    // 1. Sibling fn call (e.g. from synthetic calls inside intrinsics)
    if let Some(fn_body) = ctx.sibling_fns.get(name) {
        let mut fn_inputs = HashMap::new();
        let mut pos_idx = 0usize;
        for (param_name, arg_expr) in args {
            let value = eval_expr(arg_expr, env, ctx)?;
            let key = match param_name {
                Some(pname) => pname.clone(),
                None => {
                    let k = format!("__pos_{pos_idx}");
                    pos_idx += 1;
                    k
                }
            };
            fn_inputs.insert(key, value);
        }
        let outputs = evaluate_stack(fn_body, &fn_inputs, ctx.sibling_fns, ctx.data_values)?;
        return sibling_fn_value_extract(name, outputs);
    }
    // 2. Intrinsics (need unevaluated args for lambdas)
    if let Some(result) = eval_intrinsic_call_s(name, args, env, ctx) {
        return result;
    }
    // 3. scan_while (builtin that needs a lambda predicate)
    if name == "scan_while" {
        return eval_scan_while_s(args, env, ctx);
    }
    // 4. Pre-evaluate args, try builtins
    let evaluated: Vec<(Option<String>, Value)> = args
        .iter()
        .map(|(n, e)| Ok((n.clone(), eval_expr(e, env, ctx)?)))
        .collect::<Result<_, EvalError>>()?;
    if let Some(result) = eval_builtin_call(name, &evaluated) {
        return result;
    }
    Err(EvalError::new(format!("unknown function: {name}")))
}

fn sibling_fn_value_extract(
    _name: &str,
    outputs: HashMap<String, Value>,
) -> Result<Value, EvalError> {
    Ok(output_value(&outputs))
}

/// Non-suspendable block evaluation for intrinsic lambda bodies and
/// standalone match arms. Sibling calls inside the block are resolved
/// re-entrantly via `eval_non_sibling_call_raw` → `evaluate_stack`.
///
/// This is NOT a parallel implementation of `eval_block_s`. The two paths
/// serve different contexts:
/// - `eval_block_pure`: no continuation stack available (lambda bodies)
/// - `eval_block_s`: continuation stack available (main-loop statement level)
fn eval_block_pure(
    stmts: &[LoweredStmt],
    env: &Env,
    ctx: &EvalContext,
) -> Result<Value, EvalError> {
    #[cfg(debug_assertions)]
    debug_assert_no_sibling_calls_in_block(stmts, &ctx.fn_index);

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
                if is_last {
                    return Ok(value);
                }
            }
            LoweredStmt::Return(_) => {
                // Return inside a pure-path block is a lowerer bug.
                // The lowerer should not produce Return in lambda/match-arm
                // blocks — only in blocks processed by eval_block_s.
                return Err(EvalError::new(
                    "BUG: LoweredStmt::Return in pure-path block (eval_block_pure). \
                     Return should only appear in blocks processed by eval_block_s."
                        .to_string(),
                ));
            }
        }
    }
    Ok(Value::Unit)
}

/// Pure (non-suspendable) match evaluation — the lambda/standalone-only path.
///
/// This function is reached from two call sites:
///   1. `eval_expr` → `LoweredExpr::Match` — for match expressions inside
///      lambda bodies or other pure-path blocks that cannot suspend.
///   2. `eval_match_standalone` — public API for standalone match evaluation.
///
/// Because this path runs on the native Rust call stack (not the continuation
/// stack), it CANNOT suspend for sibling calls. Any calls in arm bodies are
/// evaluated re-entrantly via `eval_non_sibling_call_raw` → `evaluate_stack`.
/// This is correct for lambda bodies (which can't suspend) and standalone
/// evaluation (which has no continuation stack).
///
/// The suspendable counterpart is `eval_match_s`, which handles match
/// expressions at statement level where sibling calls need continuation-based
/// suspension.
fn eval_match_local(
    scrutinee: &Value,
    arms: &[LoweredMatchArm],
    env: &Env,
    ctx: &EvalContext,
) -> Result<Value, EvalError> {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, scrutinee) {
            let mut arm_env = env.child();
            for (name, val) in bindings {
                arm_env.bind(name, val);
            }
            if let Some(guard) = &arm.guard {
                let g = eval_expr(guard, &arm_env, ctx)?;
                if !value_truthy(&g) {
                    continue;
                }
            }
            return eval_expr(&arm.body, &arm_env, ctx);
        }
    }
    Err(EvalError::new(format!(
        "no matching arm for: {:?}",
        scrutinee
    )))
}

/// Evaluate a match expression without synthetic-fn-body overhead.
/// Used by the public `eval_match` wrapper in eval.rs.
pub fn eval_match_standalone(
    scrutinee: &Value,
    arms: &[LoweredMatchArm],
    env_bindings: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    let data_values = HashMap::new();
    let dummy = LoweredFnBody::from_stmts(vec![]);
    let (ctx, _) = build_context(&dummy, sibling_fns, &data_values);
    let env = Env::from_inputs(env_bindings);
    eval_match_local(scrutinee, arms, &env, &ctx)
}

// ── Intrinsics (self-contained, no bridge to eval.rs) ────────────────────

fn eval_intrinsic_call_s(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
) -> Option<Result<Value, EvalError>> {
    if !gunbc_ir::patterns::is_eval_intrinsic(name) {
        return None;
    }
    Some(eval_intrinsic_inner(name, args, env, ctx))
}

fn eval_intrinsic_inner(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
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
                match eval_expr(e, env, ctx)? {
                    Value::Str(s) => s,
                    _ => ",".into(),
                }
            } else {
                ",".into()
            };
            match receiver {
                Value::List(items) => Ok(Value::Str(
                    items
                        .iter()
                        .map(value_to_string)
                        .collect::<Vec<_>>()
                        .join(&sep),
                )),
                _ => Err(EvalError::new("join requires a list")),
            }
        }
        "map" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut out = Vec::new();
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        out.push(eval_expr(body, &c, ctx)?);
                    }
                    Ok(Value::List(Arc::new(out)))
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
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        out.push(eval_expr(&call, &c, ctx)?);
                    }
                    Ok(Value::List(Arc::new(out)))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                (other, _) => Err(EvalError::new(format!(
                    "map requires a list, got {other:?}"
                ))),
            }
        }
        "filter" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut out = Vec::new();
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        if value_truthy(&eval_expr(body, &c, ctx)?) {
                            out.push(item.clone());
                        }
                    }
                    Ok(Value::List(Arc::new(out)))
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
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        if value_truthy(&eval_expr(&call, &c, ctx)?) {
                            out.push(item.clone());
                        }
                    }
                    Ok(Value::List(Arc::new(out)))
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
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        let val = eval_expr(body, &c, ctx)?;
                        if !matches!(val, Value::Unit | Value::Skipped) {
                            out.push(val);
                        }
                    }
                    Ok(Value::List(Arc::new(out)))
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
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        match eval_expr(body, &c, ctx)? {
                            Value::List(inner) => out.extend(inner.iter().cloned()),
                            other => out.push(other),
                        }
                    }
                    Ok(Value::List(Arc::new(out)))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("flat_map requires a list")),
            }
        }
        "fold" => {
            let init = rest.iter().find(|(k, _)| k.as_deref() == Some("init"));
            let func = rest.iter().find(|(k, _)| k.as_deref() == Some("f"));
            match (receiver, init, func) {
                (
                    Value::List(items),
                    Some((_, init_e)),
                    Some((_, LoweredExpr::Lambda { params, body })),
                ) => {
                    let mut acc = eval_expr(init_e, env, ctx)?;
                    let ap = params.first().cloned().unwrap_or_else(|| "acc".into());
                    let ip = params.get(1).cloned().unwrap_or_else(|| "item".into());
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(ap.clone(), acc);
                        c.bind(ip.clone(), item.clone());
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
                (Value::List(base), Some((_, e))) => {
                    let mut v = Arc::try_unwrap(base).unwrap_or_else(|rc| (*rc).clone());
                    match eval_expr(e, env, ctx)? {
                        Value::List(more) => v.extend(more.iter().cloned()),
                        other => v.push(other),
                    }
                    Ok(Value::List(Arc::new(v)))
                }
                (other, _) => Err(EvalError::new(format!(
                    "append requires a list, got {other:?}"
                ))),
            }
        }
        "len" | "count" => match receiver {
            Value::List(items) => Ok(Value::Int(items.len() as i64)),
            _ => Err(EvalError::new("count requires a list")),
        },
        "sum" => match receiver {
            Value::List(items) => {
                let total: i64 = items
                    .iter()
                    .filter_map(|v| if let Value::Int(i) = v { Some(i) } else { None })
                    .sum();
                Ok(Value::Int(total))
            }
            _ => Err(EvalError::new("sum requires a list")),
        },
        "first" => match receiver {
            Value::List(items) => {
                let mut m = BTreeMap::new();
                if let Some(item) = items.first() {
                    m.insert("_variant".into(), Value::Str("Some".into()));
                    m.insert("value".into(), item.clone());
                } else {
                    m.insert("_variant".into(), Value::Str("None".into()));
                }
                Ok(Value::Map(m))
            }
            _ => Err(EvalError::new("first requires a list")),
        },
        "get" => {
            let idx_expr = rest.first().map(|(_, e)| e);
            match (receiver, idx_expr) {
                (Value::List(items), Some(e)) => {
                    let idx = match eval_expr(e, env, ctx)? {
                        Value::Int(i) => i as usize,
                        other => {
                            return Err(EvalError::new(format!(
                                "get index must be Int, got {other:?}"
                            )))
                        }
                    };
                    let mut m = BTreeMap::new();
                    if let Some(item) = items.get(idx) {
                        m.insert("_variant".into(), Value::Str("Some".into()));
                        m.insert("value".into(), item.clone());
                    } else {
                        m.insert("_variant".into(), Value::Str("None".into()));
                    }
                    Ok(Value::Map(m))
                }
                _ => Err(EvalError::new("get requires a list and an index")),
            }
        }
        "last" => match receiver {
            Value::List(items) => {
                let mut m = BTreeMap::new();
                if let Some(item) = items.last() {
                    m.insert("_variant".into(), Value::Str("Some".into()));
                    m.insert("value".into(), item.clone());
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
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        if value_truthy(&eval_expr(body, &c, ctx)?) {
                            return Ok(Value::Bool(true));
                        }
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
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        if !value_truthy(&eval_expr(body, &c, ctx)?) {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                }
                _ => Err(EvalError::new("all requires list and predicate")),
            }
        }
        "contains" => {
            let needle_expr = rest
                .first()
                .or_else(|| rest.iter().find(|(k, _)| k.as_deref() == Some("item")));
            match (receiver, needle_expr) {
                (Value::List(items), Some((_, expr))) => {
                    let needle = eval_expr(expr, env, ctx)?;
                    Ok(Value::Bool(items.contains(&needle)))
                }
                _ => Err(EvalError::new("contains requires list and item")),
            }
        }
        "sort" => match receiver {
            Value::List(items) => {
                let mut v = Arc::try_unwrap(items).unwrap_or_else(|rc| (*rc).clone());
                v.sort_by_key(sort_key);
                Ok(Value::List(Arc::new(v)))
            }
            _ => Err(EvalError::new("sort requires a list")),
        },
        "dedup" => match receiver {
            Value::List(items) => {
                let mut out = Vec::new();
                for item in items.iter() {
                    if !out.contains(item) {
                        out.push(item.clone());
                    }
                }
                Ok(Value::List(Arc::new(out)))
            }
            _ => Err(EvalError::new("dedup requires a list")),
        },
        "sort_by" => {
            let lambda = rest.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let p = params.first().cloned().unwrap_or_else(|| "_".into());
                    let mut keyed: Vec<(String, Value)> = Vec::with_capacity(items.len());
                    for item in items.iter() {
                        let mut c = env.child();
                        c.bind(p.clone(), item.clone());
                        let key = eval_expr(body, &c, ctx).map(|v| value_to_string(&v))?;
                        keyed.push((key, item.clone()));
                    }
                    keyed.sort_by(|a, b| a.0.cmp(&b.0));
                    Ok(Value::List(Arc::new(keyed.into_iter().map(|(_, v)| v).collect())))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("sort_by requires a list")),
            }
        }
        "starts_with" => {
            let prefix = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("prefix"))
                .or_else(|| rest.first());
            match (receiver, prefix) {
                (Value::Str(s), Some((_, e))) => Ok(Value::Bool(
                    s.starts_with(&value_to_string(&eval_expr(e, env, ctx)?)),
                )),
                _ => Err(EvalError::new("starts_with requires string and prefix")),
            }
        }
        "ends_with" => {
            let suffix = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("suffix"))
                .or_else(|| rest.first());
            match (receiver, suffix) {
                (Value::Str(s), Some((_, e))) => Ok(Value::Bool(
                    s.ends_with(&value_to_string(&eval_expr(e, env, ctx)?)),
                )),
                _ => Err(EvalError::new("ends_with requires string and suffix")),
            }
        }
        "string_contains" => {
            let needle = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("substring"))
                .or_else(|| rest.first());
            match (receiver, needle) {
                (Value::Str(s), Some((_, e))) => Ok(Value::Bool(
                    s.contains(&value_to_string(&eval_expr(e, env, ctx)?)),
                )),
                _ => Err(EvalError::new(
                    "string_contains requires string and substring",
                )),
            }
        }
        "split" => {
            let delim = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("delimiter"))
                .or_else(|| rest.first());
            match (receiver, delim) {
                (Value::Str(s), Some((_, e))) => {
                    let d = value_to_string(&eval_expr(e, env, ctx)?);
                    Ok(Value::List(Arc::new(
                        s.split(&d).map(|p| Value::Str(p.to_string())).collect(),
                    )))
                }
                (Value::Str(s), None) => Ok(Value::List(Arc::new(
                    s.split(',').map(|p| Value::Str(p.to_string())).collect(),
                ))),
                _ => Err(EvalError::new("split requires a string")),
            }
        }
        "zip" => {
            let other_expr = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("other"))
                .or_else(|| rest.first());
            match (receiver, other_expr) {
                (Value::List(items), Some((_, e))) => {
                    let other = match eval_expr(e, env, ctx)? {
                        Value::List(l) => l,
                        _ => return Err(EvalError::new("zip requires a list for 'other'")),
                    };
                    Ok(Value::List(Arc::new(
                        items
                            .iter()
                            .cloned()
                            .zip(other.iter().cloned())
                            .map(|(a, b)| {
                                let mut m = BTreeMap::new();
                                m.insert("first".into(), a);
                                m.insert("second".into(), b);
                                Value::Map(m)
                            })
                            .collect(),
                    )))
                }
                _ => Err(EvalError::new("zip requires a list")),
            }
        }
        "skip" => {
            let n_expr = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("n"))
                .or_else(|| rest.first());
            match (receiver, n_expr) {
                (Value::List(items), Some((_, e))) => match eval_expr(e, env, ctx)? {
                    Value::Int(count) => Ok(Value::List(Arc::new(
                        items.iter().skip(count.max(0) as usize).cloned().collect(),
                    ))),
                    _ => Err(EvalError::new("skip requires integer count")),
                },
                (Value::List(items), None) => Ok(Value::List(items)),
                _ => Err(EvalError::new("skip requires a list")),
            }
        }
        "enumerate" => match receiver {
            Value::List(items) => Ok(Value::List(Arc::new(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let mut m = BTreeMap::new();
                        m.insert("first".into(), Value::Int(i as i64));
                        m.insert("second".into(), v.clone());
                        Value::Map(m)
                    })
                    .collect(),
            ))),
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
            Value::Str(s) => Ok(Value::List(Arc::new(
                s.chars().map(|c| Value::Str(c.to_string())).collect(),
            ))),
            _ => Err(EvalError::new(format!(
                "chars: expected String, got {receiver:?}"
            ))),
        },
        // v2 compiler builtins
        "char_at" => {
            let pos_expr = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("pos"))
                .or_else(|| rest.first());
            match (&receiver, pos_expr) {
                (Value::Str(s), Some((_, e))) => match eval_expr(e, env, ctx)? {
                    Value::Int(pos) => match s.chars().nth(pos as usize) {
                        Some(c) => Ok(Value::Str(c.to_string())),
                        None => Ok(Value::Unit),
                    },
                    _ => Err(EvalError::new("char_at: pos must be Int")),
                },
                _ => Err(EvalError::new("char_at requires (String, Int)")),
            }
        }
        "string_length" => match &receiver {
            Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
            _ => Err(EvalError::new("string_length requires String")),
        },
        "substring" => {
            let start_expr = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("start"))
                .or_else(|| rest.first());
            let end_expr = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("end"))
                .or_else(|| rest.get(1));
            match (&receiver, start_expr, end_expr) {
                (Value::Str(s), Some((_, se)), Some((_, ee))) => {
                    let start = match eval_expr(se, env, ctx)? {
                        Value::Int(n) => n as usize,
                        _ => return Err(EvalError::new("substring: start must be Int")),
                    };
                    let end = match eval_expr(ee, env, ctx)? {
                        Value::Int(n) => n as usize,
                        _ => return Err(EvalError::new("substring: end must be Int")),
                    };
                    Ok(Value::Str(
                        s.chars()
                            .skip(start)
                            .take(end.saturating_sub(start))
                            .collect(),
                    ))
                }
                _ => Err(EvalError::new("substring requires (String, Int, Int)")),
            }
        }
        "lookup" => {
            let key_expr = rest
                .iter()
                .find(|(k, _)| k.as_deref() == Some("key"))
                .or_else(|| rest.first());
            match (&receiver, key_expr) {
                (Value::Map(map), Some((_, e))) => {
                    let key = match eval_expr(e, env, ctx)? {
                        Value::Str(s) => s,
                        other => value_to_string(&other),
                    };
                    match map.get(&key).cloned() {
                        Some(val) => {
                            let mut m = BTreeMap::new();
                            m.insert("_variant".into(), Value::Str("Some".into()));
                            m.insert("value".into(), val);
                            Ok(Value::Map(m))
                        }
                        None => {
                            let mut m = BTreeMap::new();
                            m.insert("_variant".into(), Value::Str("None".into()));
                            Ok(Value::Map(m))
                        }
                    }
                }
                _ => Err(EvalError::new("lookup requires (Map, key)")),
            }
        }
        "with" => {
            // with(record, { field: value, ... }) → record updated with new fields
            match (receiver, rest.first()) {
                (Value::Map(mut map), Some((_, e))) => {
                    let updates = eval_expr(e, env, ctx)?;
                    if let Value::Map(update_map) = updates {
                        for (k, v) in update_map {
                            map.insert(k, v);
                        }
                    }
                    Ok(Value::Map(map))
                }
                (other, _) => Err(EvalError::new(format!(
                    "with: expected record/map, got {other:?}"
                ))),
            }
        }
        _ => {
            // Pure intrinsics share the eval_core builtin table. Only the
            // lambda/sibling-fn cases stay in this explicit-stack layer.
            let mut evaluated = Vec::with_capacity(args.len());
            evaluated.push((args[0].0.clone(), receiver));
            for (arg_name, arg_expr) in rest {
                evaluated.push((arg_name.clone(), eval_expr(arg_expr, env, ctx)?));
            }
            match eval_builtin_call(name, &evaluated) {
                Some(result) => result,
                None => Err(EvalError::new(format!("unknown intrinsic call: {name}"))),
            }
        }
    }
}

fn eval_scan_while_s(
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
) -> Result<Value, EvalError> {
    let s = eval_positional_or_named_s("s", 0, args, env, ctx)?;
    let start = eval_positional_or_named_s("start", 1, args, env, ctx)?;
    let pred_expr = get_arg_expr_s("pred", 2, args);
    let resolved_pred = match pred_expr {
        Some(LoweredExpr::Lambda { params, body }) => Some((params.clone(), body.as_ref().clone())),
        Some(LoweredExpr::Ident(name)) if ctx.sibling_fns.contains_key(name.as_str()) => {
            let p = "ch".to_string();
            Some((
                vec![p.clone()],
                LoweredExpr::Call {
                    name: name.clone(),
                    args: vec![(Some("ch".into()), LoweredExpr::Ident(p))],
                },
            ))
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
                if !value_truthy(&eval_expr(&body, &c, ctx)?) {
                    break;
                }
                pos += 1;
            }
            Ok(Value::Int(pos as i64))
        }
        _ => Err(EvalError::new("scan_while requires (String, Int, Lambda)")),
    }
}

fn eval_positional_or_named_s(
    param: &str,
    index: usize,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    ctx: &EvalContext,
) -> Result<Value, EvalError> {
    for (name, expr) in args {
        if name.as_deref() == Some(param) {
            return eval_expr(expr, env, ctx);
        }
    }
    if let Some((_, expr)) = args.get(index) {
        return eval_expr(expr, env, ctx);
    }
    Err(EvalError::new(format!("missing argument '{param}'")))
}

fn get_arg_expr_s<'a>(
    param: &str,
    index: usize,
    args: &'a [(Option<String>, LoweredExpr)],
) -> Option<&'a LoweredExpr> {
    for (name, expr) in args {
        if name.as_deref() == Some(param) {
            return Some(expr);
        }
    }
    args.get(index).map(|(_, expr)| expr)
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════════

fn bind_let_result(env: &mut Env, name: String, value: &Value) {
    match value {
        Value::Map(fields) => {
            for (f, v) in fields {
                env.bind(format!("{name}__{f}"), v.clone());
            }
        }
        Value::Json(serde_json::Value::Object(map)) => {
            for (f, v) in map {
                env.bind(format!("{name}__{f}"), Value::Json(v.clone()));
            }
        }
        _ => {}
    }
    env.bind(name, value.clone());
}

fn wrap_value_as_output(value: Value) -> HashMap<String, Value> {
    if let Value::Map(map) = &value {
        // Map flattening: a Map result is destructured into the output map.
        // This is documented as "structurally necessary" (Limitation 3 in
        // the design doc) because the v2 evaluation model returns multi-field
        // records this way. Target for removal once all callers wrap
        // explicitly as {"return": map_value}.
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    } else {
        [("return".to_string(), value)].into_iter().collect()
    }
}

fn unit_output() -> HashMap<String, Value> {
    [("return".to_string(), Value::Unit)].into_iter().collect()
}

// ── Environment ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Env {
    bindings: Rc<HashMap<String, Value>>,
}

impl Env {
    fn from_inputs(inputs: &HashMap<String, Value>) -> Self {
        Self {
            bindings: Rc::new(inputs.clone()),
        }
    }
    fn bind(&mut self, name: String, value: Value) {
        Rc::make_mut(&mut self.bindings).insert(name, value);
    }
    fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }
    fn child(&self) -> Self {
        Self {
            bindings: Rc::clone(&self.bindings),
        }
    }
}
// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{LoweredBinOp, LoweredExpr, LoweredFnBody, LoweredLiteral, LoweredStmt};

    fn call(name: &str, args: Vec<(&str, LoweredExpr)>) -> LoweredExpr {
        LoweredExpr::Call {
            name: name.to_string(),
            args: args
                .into_iter()
                .map(|(k, v)| (Some(k.to_string()), v))
                .collect(),
        }
    }
    fn call_positional(name: &str, args: Vec<LoweredExpr>) -> LoweredExpr {
        LoweredExpr::Call {
            name: name.to_string(),
            args: args.into_iter().map(|v| (None, v)).collect(),
        }
    }
    fn ident(n: &str) -> LoweredExpr {
        LoweredExpr::Ident(n.to_string())
    }
    fn int(n: i64) -> LoweredExpr {
        LoweredExpr::Literal(LoweredLiteral::Int(n))
    }
    fn string(s: &str) -> LoweredExpr {
        LoweredExpr::Literal(LoweredLiteral::String(s.to_string()))
    }

    fn is_even_odd_pair() -> (LoweredFnBody, LoweredFnBody) {
        let mk = |_base_name: &str, base_val: bool, other: &str| LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Eq,
                        right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Block(vec![LoweredStmt::Return(vec![(
                        "return".to_string(),
                        LoweredExpr::Literal(LoweredLiteral::Bool(base_val)),
                    )])])),
                    else_: None,
                }),
                LoweredStmt::Expr(call(
                    other,
                    vec![(
                        "n",
                        LoweredExpr::BinOp {
                            left: Box::new(ident("n")),
                            op: LoweredBinOp::Sub,
                            right: Box::new(int(1)),
                        },
                    )],
                )),
            ],
            ..Default::default()
        };
        (
            mk("is_even", true, "is_odd"),
            mk("is_odd", false, "is_even"),
        )
    }

    #[test]
    fn simple_fn() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![("return".into(), int(42))])],
            ..Default::default()
        };
        assert_eq!(
            evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()
                ["return"],
            Value::Int(42)
        );
    }

    #[test]
    fn sibling_call_with_projection() {
        let inner = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![("return".into(), int(99))])],
            ..Default::default()
        };
        let outer = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("r".into(), call("inner", vec![])),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        let mut s = HashMap::new();
        s.insert("inner".into(), inner);
        s.insert("outer".into(), outer.clone());
        assert_eq!(
            evaluate_stack(&outer, &HashMap::new(), &s, &HashMap::new()).unwrap()["return"],
            Value::Int(99)
        );
    }

    #[test]
    fn deep_mutual_recursion_40k() {
        let (even, odd) = is_even_odd_pair();
        let mut s = HashMap::new();
        s.insert("is_even".into(), even.clone());
        s.insert("is_odd".into(), odd);
        let mut i = HashMap::new();
        i.insert("n".into(), Value::Int(40_000));
        assert_eq!(
            evaluate_stack(&even, &i, &s, &HashMap::new()).unwrap()["return"],
            Value::Bool(true)
        );
        i.insert("n".into(), Value::Int(40_001));
        assert_eq!(
            evaluate_stack(&even, &i, &s, &HashMap::new()).unwrap()["return"],
            Value::Bool(false)
        );
    }

    #[test]
    fn value_normalization() {
        let inner = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![("return".into(), int(42))])],
            ..Default::default()
        };
        let wrapper = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(call("inner", vec![]))],
            ..Default::default()
        };
        let mut s = HashMap::new();
        s.insert("inner".into(), inner);
        s.insert("wrapper".into(), wrapper.clone());
        let r = evaluate_stack(&wrapper, &HashMap::new(), &s, &HashMap::new()).unwrap();
        assert_eq!(r.get("return"), Some(&Value::Int(42)));
        assert!(!r.contains_key("value"));
    }

    #[test]
    fn builtin_call() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "result".into(),
                    LoweredExpr::Call {
                        name: "skip_horizontal_ws".into(),
                        args: vec![
                            (Some("s".into()), ident("s")),
                            (Some("start".into()), ident("start")),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("result"))]),
            ],
            ..Default::default()
        };
        let mut i = HashMap::new();
        i.insert("s".into(), Value::Str("   hello".into()));
        i.insert("start".into(), Value::Int(0));
        assert_eq!(
            evaluate_stack(&body, &i, &HashMap::new(), &HashMap::new()).unwrap()["return"],
            Value::Int(3)
        );
    }

    #[test]
    fn sibling_then_builtin() {
        let mk = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![
                ("source".into(), ident("source")),
                ("start".into(), int(0)),
            ])],
            ..Default::default()
        };
        let outer = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "state".into(),
                    call("make_state", vec![("source", ident("source"))]),
                ),
                LoweredStmt::Let(
                    "result".into(),
                    LoweredExpr::Call {
                        name: "skip_horizontal_ws".into(),
                        args: vec![
                            (
                                Some("s".into()),
                                LoweredExpr::FieldAccess {
                                    expr: Box::new(ident("state")),
                                    field: "source".into(),
                                },
                            ),
                            (
                                Some("start".into()),
                                LoweredExpr::FieldAccess {
                                    expr: Box::new(ident("state")),
                                    field: "start".into(),
                                },
                            ),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("result"))]),
            ],
            ..Default::default()
        };
        let mut s = HashMap::new();
        s.insert("make_state".into(), mk);
        s.insert("outer".into(), outer.clone());
        let mut i = HashMap::new();
        i.insert("source".into(), Value::Str("   hello".into()));
        assert_eq!(
            evaluate_stack(&outer, &i, &s, &HashMap::new()).unwrap()["return"],
            Value::Int(3)
        );
    }

    #[test]
    fn sibling_call_inside_if_branch() {
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
            ..Default::default()
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
            ..Default::default()
        };
        let mut sibs = HashMap::new();
        sibs.insert("double".to_string(), double_body);
        sibs.insert("process".to_string(), process_body.clone());
        let mut inp = HashMap::new();
        inp.insert("x".to_string(), Value::Int(5));
        let result = evaluate_stack(&process_body, &inp, &sibs, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Int(10));
    }

    #[test]
    fn anf_verifier_catches_nested_call() {
        let bad = LoweredFnBody {
            stmts: vec![LoweredStmt::Let(
                "x".into(),
                LoweredExpr::BinOp {
                    left: Box::new(call("f", vec![])),
                    op: LoweredBinOp::Add,
                    right: Box::new(int(1)),
                },
            )],
            ..Default::default()
        };
        let f_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![("return".into(), int(1))])],
            ..Default::default()
        };
        let mut sibs = HashMap::new();
        sibs.insert("f".to_string(), f_body);
        let data = HashMap::new();
        let (ctx, _) = build_context(&bad, &sibs, &data);
        assert!(verify_anf_contract(&ctx).is_err());
    }

    // ── Ported builtin/intrinsic tests (Phase 3 migration) ──────────────

    #[test]
    fn builtin_char_at() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "r".into(),
                    LoweredExpr::Call {
                        name: "char_at".into(),
                        args: vec![
                            (
                                Some("s".into()),
                                LoweredExpr::Literal(LoweredLiteral::String("hello".into())),
                            ),
                            (Some("pos".into()), int(1)),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        assert_eq!(
            evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()
                ["return"],
            Value::Str("e".into())
        );
    }

    #[test]
    fn builtin_scan_while_with_lambda() {
        // scan_while(s: "123abc", start: 0, pred: c => {
        //   let __t0 = code_point(c); let __t1 = code_point("0");
        //   let __t2 = code_point(c); let __t3 = code_point("9");
        //   __t0 >= __t1 && __t2 <= __t3
        // })
        // ANF-normalized: calls hoisted to let-bindings inside the lambda block.
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "r".into(),
                    LoweredExpr::Call {
                        name: "scan_while".into(),
                        args: vec![
                            (
                                Some("s".into()),
                                LoweredExpr::Literal(LoweredLiteral::String("123abc".into())),
                            ),
                            (Some("start".into()), int(0)),
                            (
                                Some("pred".into()),
                                LoweredExpr::Lambda {
                                    params: vec!["c".into()],
                                    body: Box::new(LoweredExpr::Block(vec![
                                        LoweredStmt::Let(
                                            "__t0".into(),
                                            LoweredExpr::Call {
                                                name: "code_point".into(),
                                                args: vec![(Some("c".into()), ident("c"))],
                                            },
                                        ),
                                        LoweredStmt::Let(
                                            "__t1".into(),
                                            LoweredExpr::Call {
                                                name: "code_point".into(),
                                                args: vec![(
                                                    Some("c".into()),
                                                    LoweredExpr::Literal(LoweredLiteral::String(
                                                        "0".into(),
                                                    )),
                                                )],
                                            },
                                        ),
                                        LoweredStmt::Let(
                                            "__t2".into(),
                                            LoweredExpr::Call {
                                                name: "code_point".into(),
                                                args: vec![(Some("c".into()), ident("c"))],
                                            },
                                        ),
                                        LoweredStmt::Let(
                                            "__t3".into(),
                                            LoweredExpr::Call {
                                                name: "code_point".into(),
                                                args: vec![(
                                                    Some("c".into()),
                                                    LoweredExpr::Literal(LoweredLiteral::String(
                                                        "9".into(),
                                                    )),
                                                )],
                                            },
                                        ),
                                        LoweredStmt::Expr(LoweredExpr::BinOp {
                                            left: Box::new(LoweredExpr::BinOp {
                                                left: Box::new(ident("__t0")),
                                                op: LoweredBinOp::Ge,
                                                right: Box::new(ident("__t1")),
                                            }),
                                            op: LoweredBinOp::And,
                                            right: Box::new(LoweredExpr::BinOp {
                                                left: Box::new(ident("__t2")),
                                                op: LoweredBinOp::Le,
                                                right: Box::new(ident("__t3")),
                                            }),
                                        }),
                                    ])),
                                },
                            ),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        assert_eq!(
            evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()
                ["return"],
            Value::Int(3)
        );
    }

    #[test]
    fn intrinsic_map_with_lambda() {
        // map([1,2,3], x => x * 2)
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "r".into(),
                    LoweredExpr::Call {
                        name: "map".into(),
                        args: vec![
                            (None, LoweredExpr::List(vec![int(1), int(2), int(3)])),
                            (
                                None,
                                LoweredExpr::Lambda {
                                    params: vec!["x".into()],
                                    body: Box::new(LoweredExpr::BinOp {
                                        left: Box::new(ident("x")),
                                        op: LoweredBinOp::Mul,
                                        right: Box::new(int(2)),
                                    }),
                                },
                            ),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        assert_eq!(
            evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()
                ["return"],
            Value::List(Arc::new(vec![Value::Int(2), Value::Int(4), Value::Int(6)]))
        );
    }

    #[test]
    fn intrinsic_fold() {
        // fold([1,2,3], init: 0, f: (acc, x) => acc + x)
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "r".into(),
                    LoweredExpr::Call {
                        name: "fold".into(),
                        args: vec![
                            (None, LoweredExpr::List(vec![int(1), int(2), int(3)])),
                            (Some("init".into()), int(0)),
                            (
                                Some("f".into()),
                                LoweredExpr::Lambda {
                                    params: vec!["acc".into(), "x".into()],
                                    body: Box::new(LoweredExpr::BinOp {
                                        left: Box::new(ident("acc")),
                                        op: LoweredBinOp::Add,
                                        right: Box::new(ident("x")),
                                    }),
                                },
                            ),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        assert_eq!(
            evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap()
                ["return"],
            Value::Int(6)
        );
    }

    #[test]
    fn canonical_collection_intrinsics_dispatch() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "sorted".into(),
                    call_positional(
                        "sort",
                        vec![LoweredExpr::List(vec![int(3), int(1), int(2), int(1)])],
                    ),
                ),
                LoweredStmt::Let(
                    "deduped".into(),
                    call_positional("dedup", vec![ident("sorted")]),
                ),
                LoweredStmt::Let(
                    "count".into(),
                    call_positional("len", vec![ident("deduped")]),
                ),
                LoweredStmt::Return(vec![
                    ("sorted".into(), ident("sorted")),
                    ("deduped".into(), ident("deduped")),
                    ("return".into(), ident("count")),
                ]),
            ],
            ..Default::default()
        };

        let result =
            evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(
            result["sorted"],
            Value::List(Arc::new(vec![
                Value::Int(1),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3)
            ]))
        );
        assert_eq!(
            result["deduped"],
            Value::List(Arc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]))
        );
        assert_eq!(result["return"], Value::Int(3));
    }

    #[test]
    fn s57_return_check_uses_structured_return_value() {
        let body = LoweredFnBody::with_types(
            vec![LoweredStmt::Expr(LoweredExpr::VariantConstruct {
                tag: "Some".into(),
                fields: vec![("value".into(), int(42))],
            })],
            vec![],
            Some("Map<String,Any>".into()),
        );

        let outcome = evaluate_stack_with_diagnostics(
            &body,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(
            outcome.outputs.get("_variant"),
            Some(&Value::Str("Some".into()))
        );
        assert_eq!(outcome.outputs.get("value"), Some(&Value::Int(42)));
        assert!(outcome.warnings.is_empty());
    }

    #[test]
    fn s57_return_check_validates_multi_field_outputs() {
        let body = LoweredFnBody::with_types(
            vec![LoweredStmt::Return(vec![
                ("a".into(), int(1)),
                ("b".into(), string("oops")),
            ])],
            vec![],
            Some("Map<String,Int>".into()),
        );

        let outcome = evaluate_stack_with_diagnostics(
            &body,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(outcome.outputs["a"], Value::Int(1));
        assert_eq!(outcome.outputs["b"], Value::Str("oops".into()));

        assert_eq!(outcome.warnings.len(), 1);
        assert!(outcome.warnings[0].contains("type mismatch at return"));
    }

    #[test]
    fn s57_warnings_are_scoped_per_top_level_evaluation() {
        let bad = LoweredFnBody::with_types(
            vec![LoweredStmt::Return(vec![(
                "return".into(),
                string("wrong"),
            )])],
            vec![],
            Some("Int".into()),
        );
        let bad_outcome = evaluate_stack_with_diagnostics(
            &bad,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        assert!(!bad_outcome.warnings.is_empty());

        let good = LoweredFnBody::with_types(
            vec![LoweredStmt::Return(vec![("return".into(), int(7))])],
            vec![],
            Some("Int".into()),
        );
        let good_outcome = evaluate_stack_with_diagnostics(
            &good,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();

        assert!(good_outcome.warnings.is_empty());
    }

    #[test]
    fn builtin_lookup() {
        let mut inp = HashMap::new();
        let mut m = std::collections::BTreeMap::new();
        m.insert("x".to_string(), Value::Int(42));
        inp.insert("m".into(), Value::Map(m));
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "r".into(),
                    LoweredExpr::Call {
                        name: "lookup".into(),
                        args: vec![
                            (Some("map".into()), ident("m")),
                            (
                                Some("key".into()),
                                LoweredExpr::Literal(LoweredLiteral::String("x".into())),
                            ),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        let r = evaluate_stack(&body, &inp, &HashMap::new(), &HashMap::new()).unwrap();
        let result = &r["return"];
        // Should be Some { value: 42 }
        if let Value::Map(map) = result {
            assert_eq!(map.get("_variant"), Some(&Value::Str("Some".into())));
            assert_eq!(map.get("value"), Some(&Value::Int(42)));
        } else {
            panic!("expected Map, got {result:?}");
        }
    }

    #[test]
    fn builtin_map_keys() {
        let mut inp = HashMap::new();
        let mut m = std::collections::BTreeMap::new();
        m.insert("a".to_string(), Value::Int(1));
        m.insert("b".to_string(), Value::Int(2));
        inp.insert("m".into(), Value::Map(m));
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "r".into(),
                    LoweredExpr::Call {
                        name: "map_keys".into(),
                        args: vec![(Some("map".into()), ident("m"))],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        let r = evaluate_stack(&body, &inp, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(
            r["return"],
            Value::List(Arc::new(vec![Value::Str("a".into()), Value::Str("b".into())]))
        );
    }

    #[test]
    fn builtin_some_returns_tagged_option() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "r".into(),
                    LoweredExpr::Call {
                        name: "Some".into(),
                        args: vec![(Some("value".into()), int(42))],
                    },
                ),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        let r = evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap();
        let result = &r["return"];
        if let Value::Map(map) = result {
            assert_eq!(map.get("_variant"), Some(&Value::Str("Some".into())));
            assert_eq!(map.get("value"), Some(&Value::Int(42)));
        } else {
            panic!("expected Map, got {result:?}");
        }
    }

    #[test]
    fn positional_args_preserved() {
        // fn adder(__pos_0, __pos_1) -> { return: __pos_0 + __pos_1 }
        let adder = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::BinOp {
                    left: Box::new(ident("__pos_0")),
                    op: LoweredBinOp::Add,
                    right: Box::new(ident("__pos_1")),
                },
            )])],
            ..Default::default()
        };
        // outer calls adder with positional args: adder(10, 32)
        let outer = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("r".into(), call_positional("adder", vec![int(10), int(32)])),
                LoweredStmt::Return(vec![("return".into(), ident("r"))]),
            ],
            ..Default::default()
        };
        let mut sibs = HashMap::new();
        sibs.insert("adder".to_string(), adder);
        sibs.insert("outer".to_string(), outer.clone());
        let result = evaluate_stack(&outer, &HashMap::new(), &sibs, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Int(42));
    }
}
