//! Explicit-stack evaluator for lowered fn bodies.
//!
//! Replaces native recursion with a heap-based continuation stack.
//! Fn-to-fn call chains go through an iterative main loop — O(1) native
//! stack per call. Expression evaluation within a single fn body uses
//! bounded native recursion (bounded by AST depth, not call chain depth).
//!
//! See DESIGN-eval-redesign.md for the design rationale.

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

// ── Configuration ───────────────────────────────────────────────────────────

const MAX_STACK_DEPTH: usize = 100_000;
const MAX_STEP_BUDGET: usize = 10_000_000;

// ── Types ───────────────────────────────────────────────────────────────────

pub type FnId = usize;

/// How to extract the return value from a callee's output map.
#[derive(Debug, Clone)]
enum Projection {
    PrimaryReturn,
}

impl Projection {
    fn extract(&self, outputs: &HashMap<String, Value>) -> Value {
        match self {
            Projection::PrimaryReturn => {
                if let Some(v) = outputs.get("return") {
                    return v.clone();
                }
                if outputs.len() == 1 {
                    if let Some(v) = outputs.get("value") {
                        return v.clone();
                    }
                }
                if outputs.is_empty() {
                    return Value::Unit;
                }
                Value::Map(outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Continuation {
    fn_id: FnId,
    pc: usize,
    binding: Option<String>,
    projection: Projection,
    env: Env,
}

enum Action {
    Done(HashMap<String, Value>),
    Suspend {
        callee: FnId,
        inputs: HashMap<String, Value>,
        cont: Continuation,
    },
    TailCall {
        callee: FnId,
        inputs: HashMap<String, Value>,
    },
    Error(String),
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Evaluate a fn body using the explicit-stack evaluator.
///
/// Drop-in replacement for `evaluate_fn_body_with_data` that uses O(1)
/// native stack per fn call instead of O(N).
pub fn evaluate_stack(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, EvalError> {
    // Build fn index: name → (FnId, body ref).
    // The entry body gets a dedicated FnId (the last one) so the main loop
    // can create continuations for it.
    let mut fn_bodies: Vec<&LoweredFnBody> = Vec::with_capacity(sibling_fns.len() + 1);
    let mut fn_index: HashMap<&str, FnId> = HashMap::with_capacity(sibling_fns.len() + 1);
    for (name, fn_body) in sibling_fns {
        let id = fn_bodies.len();
        fn_bodies.push(fn_body);
        fn_index.insert(name.as_str(), id);
    }
    let entry_fn_id: FnId = fn_bodies.len();
    fn_bodies.push(body);

    let bridge = Bridge {
        fn_bodies: &fn_bodies,
        fn_index: &fn_index,
        sibling_fns,
        data_values,
    };

    let mut stack: Vec<Continuation> = Vec::new();
    let mut current_body: &LoweredFnBody = body;
    let mut current_fn_id: FnId = entry_fn_id;
    let mut env = Env::from_inputs(inputs);
    seed_data(&mut env, data_values);
    let mut pc: usize = 0;
    let mut steps: usize = 0;

    loop {
        steps += 1;
        if steps > MAX_STEP_BUDGET {
            return Err(EvalError::new("step budget exceeded"));
        }

        match eval_body(&current_body.stmts, pc, &mut env, &bridge, Some(current_fn_id)) {
            Action::Done(mut result) => {
                // Unwind the continuation stack until we find a
                // continuation that resumes execution (has remaining stmts).
                loop {
                    match stack.pop() {
                        None => return Ok(result),
                        Some(cont) => {
                            let value = cont.projection.extract(&result);
                            let body_stmts = &bridge.fn_bodies[cont.fn_id].stmts;
                            if cont.pc >= body_stmts.len() && cont.binding.is_none() {
                                // Past the end with no binding — this is
                                // an Expr(Call) at tail position. The
                                // projected value is the fn's return.
                                result = wrap_as_result(value);
                            } else {
                                env = cont.env;
                                if let Some(ref name) = cont.binding {
                                    bind_with_flattening(&mut env, name.clone(), &value);
                                }
                                current_body = bridge.fn_bodies[cont.fn_id];
                                current_fn_id = cont.fn_id;
                                pc = cont.pc;
                                break;
                            }
                        }
                    }
                }
            }
            Action::Suspend { callee, inputs, cont } => {
                if stack.len() >= MAX_STACK_DEPTH {
                    return Err(EvalError::new(format!(
                        "max call depth ({MAX_STACK_DEPTH}) exceeded"
                    )));
                }
                stack.push(cont);
                env = Env::from_inputs(&inputs);
                seed_data(&mut env, data_values);
                current_body = bridge.fn_bodies[callee];
                current_fn_id = callee;
                pc = 0;
            }
            Action::TailCall { callee, inputs } => {
                env = Env::from_inputs(&inputs);
                seed_data(&mut env, data_values);
                current_body = bridge.fn_bodies[callee];
                current_fn_id = callee;
                pc = 0;
            }
            Action::Error(msg) => return Err(EvalError::new(msg)),
        }
    }
}

// ── Bridge to old evaluator ─────────────────────────────────────────────────

struct Bridge<'a> {
    fn_bodies: &'a [&'a LoweredFnBody],
    fn_index: &'a HashMap<&'a str, FnId>,
    sibling_fns: &'a HashMap<String, LoweredFnBody>,
    data_values: &'a HashMap<String, Value>,
}

impl<'a> Bridge<'a> {
    fn is_sibling(&self, name: &str) -> Option<FnId> {
        self.fn_index.get(name).copied()
    }
}

// ── Body evaluation ─────────────────────────────────────────────────────────

fn eval_body(
    stmts: &[LoweredStmt],
    start_pc: usize,
    env: &mut Env,
    bridge: &Bridge,
    fn_id: Option<FnId>,
) -> Action {
    for i in start_pc..stmts.len() {
        let is_last = i == stmts.len() - 1;

        match &stmts[i] {
            LoweredStmt::Let(name, expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(callee_id) = bridge.is_sibling(callee) {
                        match eval_call_args(args, env, bridge) {
                            Ok(fn_inputs) => {
                                if let Some(fn_id) = fn_id {
                                    return Action::Suspend {
                                        callee: callee_id,
                                        inputs: fn_inputs,
                                        cont: Continuation {
                                            fn_id,
                                            pc: i + 1,
                                            binding: Some(name.clone()),
                                            projection: Projection::PrimaryReturn,
                                            env: env.clone(),
                                        },
                                    };
                                }
                                // Entry fn — use old evaluator for recursive call
                                match eval_sibling_recursive(callee_id, &fn_inputs, bridge) {
                                    Ok(value) => bind_with_flattening(env, name.clone(), &value),
                                    Err(msg) => return Action::Error(msg),
                                }
                            }
                            Err(msg) => return Action::Error(msg),
                        }
                    } else {
                        match eval_non_sibling(callee, args, env, bridge) {
                            Ok(value) => bind_with_flattening(env, name.clone(), &value),
                            Err(e) => return err_action(e),
                        }
                    }
                } else {
                    match eval_expr_pure(expr, env, bridge) {
                        Ok(value) => bind_with_flattening(env, name.clone(), &value),
                        Err(e) => {
                            if let Some(ret) = e.early_return {
                                return Action::Done(ret);
                            }
                            return Action::Error(e.message);
                        }
                    }
                }
            }

            LoweredStmt::Expr(expr) => {
                if let LoweredExpr::Call { name: callee, args } = expr {
                    if let Some(callee_id) = bridge.is_sibling(callee) {
                        match eval_call_args(args, env, bridge) {
                            Ok(fn_inputs) => {
                                // Both tail and non-tail positions use Suspend
                                // because the result needs PrimaryReturn projection
                                // + Expr wrapping ({"return": value}).
                                // True tail calls (identity continuation) are only
                                // possible when the fn body's Return fields reference
                                // the call directly with no transformation.
                                if let Some(fn_id) = fn_id {
                                    return Action::Suspend {
                                        callee: callee_id,
                                        inputs: fn_inputs,
                                        cont: Continuation {
                                            fn_id,
                                            pc: i + 1,
                                            binding: None,
                                            projection: Projection::PrimaryReturn,
                                            env: env.clone(),
                                        },
                                    };
                                }
                                match eval_sibling_recursive(callee_id, &fn_inputs, bridge) {
                                    Ok(_) => {}
                                    Err(msg) => return Action::Error(msg),
                                }
                            }
                            Err(msg) => return Action::Error(msg),
                        }
                    } else {
                        match eval_non_sibling(callee, args, env, bridge) {
                            Ok(value) => {
                                if is_last {
                                    return Action::Done(wrap_as_result(value));
                                }
                            }
                            Err(e) => return err_action(e),
                        }
                    }
                } else if is_last {
                    return eval_trailing_expr(expr, env, bridge);
                } else {
                    match eval_expr_pure(expr, env, bridge) {
                        Ok(_) => {}
                        Err(e) => {
                            if let Some(ret) = e.early_return {
                                return Action::Done(ret);
                            }
                            return Action::Error(e.message);
                        }
                    }
                }
            }

            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, fexpr) in fields {
                    match eval_expr_pure(fexpr, env, bridge) {
                        Ok(value) => { result.insert(name.clone(), value); }
                        Err(e) => {
                            if let Some(ret) = e.early_return {
                                return Action::Done(ret);
                            }
                            return Action::Error(e.message);
                        }
                    }
                }
                return Action::Done(result);
            }
        }
    }

    Action::Done([("return".to_string(), Value::Unit)].into_iter().collect())
}

// ── Trailing expression ─────────────────────────────────────────────────────

fn eval_trailing_expr(
    expr: &LoweredExpr,
    env: &Env,
    bridge: &Bridge,
) -> Action {
    match expr {
        LoweredExpr::IfElse { cond, then_, else_ } => {
            match eval_expr_pure(cond, env, bridge) {
                Ok(condition) => {
                    if value_truthy(&condition) {
                        eval_trailing_expr(then_, env, bridge)
                    } else if let Some(e) = else_ {
                        eval_trailing_expr(e, env, bridge)
                    } else {
                        Action::Done(wrap_as_result(Value::Unit))
                    }
                }
                Err(e) => err_action(e),
            }
        }
        LoweredExpr::Match { expr: scrutinee, arms } => {
            match eval_expr_pure(scrutinee, env, bridge) {
                Ok(val) => eval_trailing_match(&val, arms, env, bridge),
                Err(e) => err_action(e),
            }
        }
        LoweredExpr::Block(stmts) => {
            let mut child_env = env.child();
            eval_body(stmts, 0, &mut child_env, bridge, None)
        }
        LoweredExpr::Return(fields) => {
            let mut result = HashMap::new();
            for (name, fexpr) in fields {
                match eval_expr_pure(fexpr, env, bridge) {
                    Ok(value) => { result.insert(name.clone(), value); }
                    Err(e) => return err_action(e),
                }
            }
            Action::Done(result)
        }
        _ => {
            match eval_expr_pure(expr, env, bridge) {
                Ok(value) => Action::Done(wrap_as_result(value)),
                Err(e) => err_action(e),
            }
        }
    }
}

fn eval_trailing_match(
    scrutinee: &Value,
    arms: &[LoweredMatchArm],
    env: &Env,
    bridge: &Bridge,
) -> Action {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, scrutinee) {
            let mut arm_env = env.child();
            for (name, val) in bindings {
                arm_env.bind(name, val);
            }
            if let Some(guard) = &arm.guard {
                match eval_expr_pure(guard, &arm_env, bridge) {
                    Ok(g) if value_truthy(&g) => {}
                    Ok(_) => continue,
                    Err(e) => return err_action(e),
                }
            }
            return eval_trailing_expr(&arm.body, &arm_env, bridge);
        }
    }
    Action::Error(format!("no matching arm for: {scrutinee:?}"))
}

// ── Pure expression evaluation ──────────────────────────────────────────────

fn eval_expr_pure(
    expr: &LoweredExpr,
    env: &Env,
    bridge: &Bridge,
) -> Result<Value, EvalError> {
    match expr {
        LoweredExpr::Literal(lit) => Ok(eval_literal(lit)),
        LoweredExpr::Ident(name) => {
            if name == "None" || name == "null" {
                return Ok(Value::Unit);
            }
            if let Some(val) = env.get(name) {
                return Ok(val.clone());
            }
            if name.chars().next().unwrap_or('a').is_uppercase() {
                return Ok(Value::Str(name.clone()));
            }
            Err(EvalError::new(format!("unbound variable: {name}")))
        }
        LoweredExpr::FieldAccess { expr, field } => {
            let base = eval_expr_pure(expr, env, bridge)?;
            field_access(&base, field)
        }
        LoweredExpr::StringInterp(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    LoweredStringPart::Literal(s) => result.push_str(s),
                    LoweredStringPart::Expr(e) => {
                        let val = eval_expr_pure(e, env, bridge)?;
                        result.push_str(&value_to_string(&val));
                    }
                }
            }
            Ok(Value::Str(result))
        }
        LoweredExpr::BinOp { left, op, right } => {
            let lhs = eval_expr_pure(left, env, bridge)?;
            match op {
                LoweredBinOp::And => {
                    if !value_truthy(&lhs) { return Ok(Value::Bool(false)); }
                    let rhs = eval_expr_pure(right, env, bridge)?;
                    Ok(Value::Bool(value_truthy(&rhs)))
                }
                LoweredBinOp::Or => {
                    if value_truthy(&lhs) { return Ok(Value::Bool(true)); }
                    let rhs = eval_expr_pure(right, env, bridge)?;
                    Ok(Value::Bool(value_truthy(&rhs)))
                }
                LoweredBinOp::NullCoalesce => {
                    if !matches!(lhs, Value::Unit | Value::Skipped) { Ok(lhs) }
                    else { eval_expr_pure(right, env, bridge) }
                }
                _ => {
                    let rhs = eval_expr_pure(right, env, bridge)?;
                    if *op == LoweredBinOp::Add {
                        match (lhs, rhs) {
                            (Value::List(mut a), Value::List(b)) => { a.extend(b); return Ok(Value::List(a)); }
                            (Value::Str(mut a), Value::Str(b)) => { a.push_str(&b); return Ok(Value::Str(a)); }
                            (Value::Str(mut a), Value::Enum { variant, .. }) => { a.push_str(&variant); return Ok(Value::Str(a)); }
                            (Value::Enum { variant, .. }, Value::Str(b)) => { return Ok(Value::Str(format!("{variant}{b}"))); }
                            (lhs, rhs) => return eval_binop(&lhs, *op, &rhs),
                        }
                    }
                    eval_binop(&lhs, *op, &rhs)
                }
            }
        }
        LoweredExpr::UnaryOp { op, expr } => {
            let val = eval_expr_pure(expr, env, bridge)?;
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
            let c = eval_expr_pure(cond, env, bridge)?;
            if value_truthy(&c) { eval_expr_pure(then_, env, bridge) }
            else if let Some(e) = else_ { eval_expr_pure(e, env, bridge) }
            else { Ok(Value::Unit) }
        }
        LoweredExpr::Match { expr: scrutinee, arms } => {
            let val = eval_expr_pure(scrutinee, env, bridge)?;
            let bindings: HashMap<String, Value> = env.bindings.as_ref().clone();
            eval_match(&val, arms, &bindings, bridge.sibling_fns)
        }
        LoweredExpr::VariantConstruct { tag, fields } => {
            if fields.is_empty() {
                Ok(Value::Enum { ty: String::new(), variant: tag.clone() })
            } else {
                let mut map = BTreeMap::new();
                map.insert("_variant".to_string(), Value::Str(tag.clone()));
                for (k, v) in fields { map.insert(k.clone(), eval_expr_pure(v, env, bridge)?); }
                Ok(Value::Map(map))
            }
        }
        LoweredExpr::Call { name, args } => {
            eval_non_sibling(name, args, env, bridge)
        }
        LoweredExpr::Lambda { .. } => {
            Err(EvalError::new("lambda cannot be evaluated standalone"))
        }
        LoweredExpr::List(items) => {
            let vals: Result<Vec<_>, _> = items.iter().map(|i| eval_expr_pure(i, env, bridge)).collect();
            Ok(Value::List(vals?))
        }
        LoweredExpr::Block(stmts) => {
            let mut child = env.child();
            let outputs = eval_block_stmts(stmts, &mut child, bridge)?;
            if outputs.len() == 1 { if let Some(v) = outputs.get("return") { return Ok(v.clone()); } }
            Ok(Value::Map(outputs.into_iter().collect()))
        }
        LoweredExpr::Record { fields, .. } => {
            let mut map = BTreeMap::new();
            for (k, v) in fields { map.insert(k.clone(), eval_expr_pure(v, env, bridge)?); }
            Ok(Value::Map(map))
        }
        LoweredExpr::For { binding, iterable, body } => {
            let items = eval_expr_pure(iterable, env, bridge)?;
            match items {
                Value::List(list) => {
                    let mut results = Vec::with_capacity(list.len());
                    for item in &list {
                        let mut iter_env = env.child();
                        iter_env.bind(binding.clone(), item.clone());
                        results.push(eval_expr_pure(body, &iter_env, bridge)?);
                    }
                    Ok(Value::List(results))
                }
                _ => Err(EvalError::new(format!("for requires list, got {:?}", items))),
            }
        }
        LoweredExpr::Return(fields) => {
            let mut result = HashMap::new();
            for (name, fexpr) in fields {
                result.insert(name.clone(), eval_expr_pure(fexpr, env, bridge)?);
            }
            Err(EvalError::early_return(result))
        }
    }
}

fn eval_block_stmts(
    stmts: &[LoweredStmt],
    env: &mut Env,
    bridge: &Bridge,
) -> Result<HashMap<String, Value>, EvalError> {
    let last = stmts.last();
    for stmt in stmts {
        let is_last = last.is_some_and(|l| std::ptr::eq(stmt, l));
        match stmt {
            LoweredStmt::Let(name, expr) => {
                let value = eval_expr_pure(expr, env, bridge)?;
                bind_with_flattening(env, name.clone(), &value);
            }
            LoweredStmt::Expr(expr) => {
                let value = eval_expr_pure(expr, env, bridge)?;
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
                    result.insert(name.clone(), eval_expr_pure(fexpr, env, bridge)?);
                }
                return Err(EvalError::early_return(result));
            }
        }
    }
    Ok([("return".to_string(), Value::Unit)].into_iter().collect())
}

// ── Call helpers ─────────────────────────────────────────────────────────────

fn eval_call_args(
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    bridge: &Bridge,
) -> Result<HashMap<String, Value>, String> {
    let mut inputs = HashMap::new();
    for (param, arg_expr) in args {
        let value = eval_expr_pure(arg_expr, env, bridge).map_err(|e| e.message)?;
        if let Some(name) = param {
            inputs.insert(name.clone(), value);
        }
    }
    Ok(inputs)
}

fn eval_sibling_recursive(
    callee_id: FnId,
    inputs: &HashMap<String, Value>,
    bridge: &Bridge,
) -> Result<Value, String> {
    let body = bridge.fn_bodies[callee_id];
    // Use the old recursive evaluator directly (not evaluate_fn_body_with_data
    // which may route back to evaluate_stack, creating mutual recursion).
    let outputs = crate::eval::evaluate_fn_body_old(
        body, inputs, bridge.sibling_fns, bridge.data_values,
    ).map_err(|e| e.message)?;
    Ok(Projection::PrimaryReturn.extract(&outputs))
}

fn eval_non_sibling(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    bridge: &Bridge,
) -> Result<Value, EvalError> {
    let env_bindings: HashMap<String, Value> = env.bindings.as_ref().clone();
    crate::eval::eval_non_sibling_call(
        name, args, &env_bindings, bridge.sibling_fns, bridge.data_values,
    )
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
                                if let Some(fval) = map.get(fname) {
                                    match match_pattern(fpat, fval) {
                                        Some(mut fb) => bindings.append(&mut fb),
                                        None => return None,
                                    }
                                } else { return None; }
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

// ── Helpers ─────────────────────────────────────────────────────────────────

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

fn bind_with_flattening(env: &mut Env, name: String, value: &Value) {
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

fn seed_data(env: &mut Env, data_values: &HashMap<String, Value>) {
    for (name, val) in data_values {
        if env.get(name).is_none() {
            env.bind(name.clone(), val.clone());
        }
    }
}

fn err_action(e: EvalError) -> Action {
    if let Some(ret) = e.early_return {
        Action::Done(ret)
    } else {
        Action::Error(e.message)
    }
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

    fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    fn child(&self) -> Self {
        Self { bindings: Rc::clone(&self.bindings) }
    }
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
    fn stack_eval_simple_fn() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![
                ("return".to_string(), int(42)),
            ])],
        };
        let result = evaluate_stack(&body, &HashMap::new(), &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Int(42));
    }

    #[test]
    fn stack_eval_sibling_call() {
        let inner_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![
                ("value".to_string(), int(99)),
            ])],
        };
        let outer_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("r".to_string(), call("inner", vec![])),
                LoweredStmt::Return(vec![("return".to_string(), ident("r"))]),
            ],
        };
        let mut siblings = HashMap::new();
        siblings.insert("inner".to_string(), inner_body);
        siblings.insert("outer".to_string(), outer_body.clone());

        let result = evaluate_stack(&outer_body, &HashMap::new(), &siblings, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Int(99));
    }

    #[test]
    fn stack_eval_mutual_recursion() {
        let is_even_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Eq,
                        right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![
                        ("return".to_string(), LoweredExpr::Literal(LoweredLiteral::Bool(true))),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Expr(call("is_odd", vec![
                    ("n", LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Sub,
                        right: Box::new(int(1)),
                    }),
                ])),
            ],
        };
        let is_odd_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Eq,
                        right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![
                        ("return".to_string(), LoweredExpr::Literal(LoweredLiteral::Bool(false))),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Expr(call("is_even", vec![
                    ("n", LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Sub,
                        right: Box::new(int(1)),
                    }),
                ])),
            ],
        };

        let mut siblings = HashMap::new();
        siblings.insert("is_even".to_string(), is_even_body.clone());
        siblings.insert("is_odd".to_string(), is_odd_body);

        let mut inputs = HashMap::new();
        inputs.insert("n".to_string(), Value::Int(1000));
        let result = evaluate_stack(&is_even_body, &inputs, &siblings, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Bool(true));
    }

    #[test]
    fn stack_eval_deep_mutual_recursion() {
        // N=40000 — would overflow native stack without explicit stack eval
        let is_even_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Eq,
                        right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![
                        ("return".to_string(), LoweredExpr::Literal(LoweredLiteral::Bool(true))),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Expr(call("is_odd", vec![
                    ("n", LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Sub,
                        right: Box::new(int(1)),
                    }),
                ])),
            ],
        };
        let is_odd_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Eq,
                        right: Box::new(int(0)),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![
                        ("return".to_string(), LoweredExpr::Literal(LoweredLiteral::Bool(false))),
                    ])),
                    else_: None,
                }),
                LoweredStmt::Expr(call("is_even", vec![
                    ("n", LoweredExpr::BinOp {
                        left: Box::new(ident("n")),
                        op: LoweredBinOp::Sub,
                        right: Box::new(int(1)),
                    }),
                ])),
            ],
        };

        let mut siblings = HashMap::new();
        siblings.insert("is_even".to_string(), is_even_body.clone());
        siblings.insert("is_odd".to_string(), is_odd_body);

        let mut inputs = HashMap::new();
        inputs.insert("n".to_string(), Value::Int(40_000));
        let result = evaluate_stack(&is_even_body, &inputs, &siblings, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Bool(true));

        inputs.insert("n".to_string(), Value::Int(40_001));
        let result = evaluate_stack(&is_even_body, &inputs, &siblings, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Bool(false));
    }

    #[test]
    #[test]
    fn stack_eval_sibling_then_builtin() {
        // fn outer(source: String) {
        //   let state = make_state(source: source)
        //   let result = skip_horizontal_ws(s: state.source, start: state.start)
        //   return { return: result }
        // }
        // fn make_state(source: String) {
        //   return { source: source, start: 0 }
        // }
        let make_state = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![
                ("source".to_string(), ident("source")),
                ("start".to_string(), int(0)),
            ])],
        };
        let outer = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let("state".to_string(), call("make_state", vec![("source", ident("source"))])),
                LoweredStmt::Let(
                    "result".to_string(),
                    LoweredExpr::Call {
                        name: "skip_horizontal_ws".to_string(),
                        args: vec![
                            (Some("s".to_string()), LoweredExpr::FieldAccess {
                                expr: Box::new(ident("state")),
                                field: "source".to_string(),
                            }),
                            (Some("start".to_string()), LoweredExpr::FieldAccess {
                                expr: Box::new(ident("state")),
                                field: "start".to_string(),
                            }),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".to_string(), ident("result"))]),
            ],
        };

        let mut siblings = HashMap::new();
        siblings.insert("make_state".to_string(), make_state);
        siblings.insert("outer".to_string(), outer.clone());

        let mut inputs = HashMap::new();
        inputs.insert("source".to_string(), Value::Str("   hello".to_string()));
        let result = evaluate_stack(&outer, &inputs, &siblings, &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Int(3));
    }

    #[test]
    fn stack_eval_builtin_call() {
        // Test that built-in calls (not in sibling_fns) work correctly
        // through the stack evaluator's bridge to the old evaluator.
        //
        // fn test(s: String, start: Int) {
        //   let result = skip_horizontal_ws(s: s, start: start)
        //   return { return: result }
        // }
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "result".to_string(),
                    LoweredExpr::Call {
                        name: "skip_horizontal_ws".to_string(),
                        args: vec![
                            (Some("s".to_string()), ident("s")),
                            (Some("start".to_string()), ident("start")),
                        ],
                    },
                ),
                LoweredStmt::Return(vec![("return".to_string(), ident("result"))]),
            ],
        };

        let mut inputs = HashMap::new();
        inputs.insert("s".to_string(), Value::Str("   hello".to_string()));
        inputs.insert("start".to_string(), Value::Int(0));

        let result = evaluate_stack(&body, &inputs, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(result["return"], Value::Int(3));
    }

    #[test]
    fn stack_eval_value_normalization() {
        let inner = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![
                ("value".to_string(), int(42)),
            ])],
        };
        let wrapper = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(call("inner", vec![]))],
        };
        let mut siblings = HashMap::new();
        siblings.insert("inner".to_string(), inner);
        siblings.insert("wrapper".to_string(), wrapper.clone());

        let result = evaluate_stack(&wrapper, &HashMap::new(), &siblings, &HashMap::new()).unwrap();
        assert_eq!(result.get("return"), Some(&Value::Int(42)));
        assert!(!result.contains_key("value"));
    }
}
