//! Pure evaluation engine for lowered expression IR.
//!
//! Evaluates `LoweredFnBody` and collection operations using only `Value`
//! types from `gunbc-ir`. No dependency on `gunbc-exec` — pure functions only.
//! Thin DynOp wrappers in `resolve.rs` call these functions.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use gunbc_ir::Value;

use crate::expr::{
    LoweredBinOp, LoweredExpr, LoweredFnBody, LoweredLiteral, LoweredMatchArm, LoweredPattern,
    LoweredStmt, LoweredStringPart, LoweredUnaryOp,
};
use gunbc_ir::patterns::CollectionKind as CollectionOpKind;

// ── Public API ──────────────────────────────────────────────────────────────

/// Evaluate a lowered fn body with the given inputs.
///
/// `sibling_fns` maps fn names → their lowered bodies for recursive calls
/// within the same module.
///
/// `data_values` provides compile-time `data` declaration bindings. These are
/// seeded into the evaluator's environment so fn bodies can reference them.
pub fn evaluate_fn_body(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<HashMap<String, Value>, EvalError> {
    evaluate_fn_body_with_data(body, inputs, sibling_fns, &HashMap::new())
}

/// Like `evaluate_fn_body` but with explicit data declaration bindings.
pub fn evaluate_fn_body_with_data(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, EvalError> {
    eval_fn_body_rc(body, inputs, sibling_fns, Rc::new(data_values.clone()), 0, None)
}

/// Internal evaluation with shared data_values via Rc (avoids re-cloning on
/// every recursive sibling fn call).
///
/// When `fn_name` is provided, self-recursive tail calls are trampolined
/// instead of consuming native stack frames.
fn eval_fn_body_rc(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: Rc<HashMap<String, Value>>,
    call_depth: usize,
    fn_name: Option<&str>,
) -> Result<HashMap<String, Value>, EvalError> {
    if call_depth > MAX_CALL_DEPTH {
        return Err(EvalError::new(format!(
            "maximum call depth ({MAX_CALL_DEPTH}) exceeded — possible infinite recursion"
        )));
    }

    // Pre-compute data seeds once (shared across trampoline iterations).
    let seeds: Vec<_> = data_values
        .iter()
        .map(|(name, val)| (name.clone(), val.clone()))
        .collect();

    let mut current_body = body;
    let mut current_name: Option<String> = fn_name.map(String::from);
    let mut current_inputs = inputs.clone();
    let mut trampoline_iters: usize = 0;

    // Trampoline loop: handles both self-recursive and mutual tail calls.
    // Self-recursive tail calls rebind inputs and loop. Mutual tail calls
    // switch to the callee's body and loop — no native stack growth.
    loop {
        let name_ref = current_name.as_deref();
        match eval_fn_body_once(current_body, &current_inputs, sibling_fns, &data_values, &seeds, call_depth, name_ref) {
            Ok(result) => return Ok(result),
            Err(e) if e.tail_call.is_some() => {
                trampoline_iters += 1;
                if trampoline_iters > MAX_TRAMPOLINE_ITERS {
                    return Err(EvalError::new(format!(
                        "maximum tail-call iterations ({MAX_TRAMPOLINE_ITERS}) exceeded in '{}' — possible infinite loop",
                        name_ref.unwrap_or("<anonymous>")
                    )));
                }
                current_inputs = e.tail_call.unwrap();

                // Mutual tail call: switch to the callee's fn body.
                if let Some(callee_name) = e.tail_call_name {
                    if let Some(callee_body) = sibling_fns.get(&callee_name) {
                        current_body = callee_body;
                        current_name = Some(callee_name);
                    } else {
                        return Err(EvalError::new(format!(
                            "mutual tail call to unknown function: {callee_name}"
                        )));
                    }
                }
                // Self-recursive tail call (tail_call_name is None): same body, new inputs.
            }
            Err(e) => return Err(e),
        }
    }
}

/// Single pass of a fn body (no trampoline — that's in eval_fn_body_rc).
fn eval_fn_body_once(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &Rc<HashMap<String, Value>>,
    seeds: &[(String, Value)],
    call_depth: usize,
    fn_name: Option<&str>,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut env = Env::from_inputs(inputs);
    env.call_depth = call_depth;
    env.self_name = fn_name.map(String::from);
    // Share the Rc — no deep clone.
    env.data_values = Rc::clone(data_values);
    // Seed data declarations into the environment (lower priority than inputs).
    for (name, value) in seeds {
        if !env.bindings.contains_key(name.as_str()) {
            env.bind(name.clone(), value.clone());
        }
    }

    eval_stmts(&body.stmts, &mut env, sibling_fns, true, true)
}

/// Shared statement evaluation loop used by both fn body execution and
/// block expression evaluation. Single implementation — no parallel copies.
///
/// `allow_tco`: when true, the last Expr stmt and Return stmts evaluate
/// their expressions in tail context (enabling self-recursive trampolining).
///
/// `is_fn_body`: when true, `return` stmts produce `Ok(result)` (we are
/// the function, and this is our return value). When false, `return` stmts
/// produce `Err(early_return)` to propagate up through block/if scopes
/// to the enclosing function body.
fn eval_stmts(
    stmts: &[LoweredStmt],
    env: &mut Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    allow_tco: bool,
    is_fn_body: bool,
) -> Result<HashMap<String, Value>, EvalError> {
    let last_stmt = stmts.last();

    for stmt in stmts {
        let is_last = last_stmt.is_some_and(|l| std::ptr::eq(stmt, l));
        let tail_ctx = allow_tco && is_last;
        match stmt {
            LoweredStmt::Let(name, expr) => {
                let value = match eval_expr(expr, env, sibling_fns) {
                    Ok(v) => v,
                    Err(e) if e.early_return.is_some() => {
                        return Ok(e.early_return.unwrap());
                    }
                    // Propagate tail_call signals from return expressions
                    // inside if/match branches within let bindings.
                    Err(e) if e.tail_call.is_some() => return Err(e),
                    Err(e) => return Err(e),
                };
                // Flatten Map/JSON fields into `parent__field` entries so that
                // the `__` convention works for local let bindings.
                match &value {
                    Value::Map(fields) => {
                        for (field_name, field_value) in fields {
                            env.bind(format!("{name}__{field_name}"), field_value.clone());
                        }
                    }
                    Value::Json(serde_json::Value::Object(map)) => {
                        for (field_name, field_value) in map {
                            env.bind(
                                format!("{name}__{field_name}"),
                                Value::Json(field_value.clone()),
                            );
                        }
                    }
                    _ => {}
                }
                env.bind(name.clone(), value);
            }
            LoweredStmt::Expr(expr) => {
                // Last expression stmt is in tail position.
                match eval_expr_tc(expr, env, sibling_fns, tail_ctx) {
                    Ok(value) => {
                        // If this is the last statement and produces a value, capture as "return"
                        if is_last {
                            if let Value::Map(map) = &value {
                                let mut result = HashMap::new();
                                for (k, v) in map {
                                    result.insert(k.clone(), v.clone());
                                }
                                return Ok(result);
                            }
                            return Ok([("return".to_string(), value)].into_iter().collect());
                        }
                    }
                    Err(e) if e.early_return.is_some() => {
                        return Ok(e.early_return.unwrap());
                    }
                    // Propagate tail_call signal
                    Err(e) if e.tail_call.is_some() => return Err(e),
                    Err(e) => return Err(e),
                }
            }
            LoweredStmt::Return(fields) => {
                // Return is ALWAYS in tail position for the enclosing function,
                // regardless of block nesting depth. A `return self_call()`
                // inside a non-tail `if` must still trampoline — the tail_call
                // signal propagates through early_return/block scopes to the
                // fn body's trampoline loop.
                if fields.len() == 1 {
                    let (name, expr) = &fields[0];
                    // Evaluate in tail context for self-recursive TCO. If a
                    // mutual tail call fires, catch it and evaluate normally —
                    // mutual TCO from Return would lose the field-name wrapping.
                    let value = match eval_expr_tc(expr, env, sibling_fns, true) {
                        Ok(v) => v,
                        Err(e) if e.tail_call.is_some() && e.tail_call_name.is_some() => {
                            // Mutual tail call from Return — evaluate the callee
                            // normally to preserve the Return wrapping.
                            let callee_name = e.tail_call_name.unwrap();
                            let callee_inputs = e.tail_call.unwrap();
                            if let Some(callee_body) = sibling_fns.get(&callee_name) {
                                let outputs = eval_fn_body_rc(
                                    callee_body, &callee_inputs, sibling_fns,
                                    Rc::clone(&env.data_values), env.call_depth + 1,
                                    Some(&callee_name),
                                )?;
                                sibling_fn_value(&callee_name, outputs)?
                            } else {
                                return Err(EvalError::new(format!(
                                    "mutual tail call to unknown function: {callee_name}"
                                )));
                            }
                        }
                        Err(e) => return Err(e),
                    };
                    if is_fn_body {
                        return Ok([(name.clone(), value)].into_iter().collect());
                    }
                    return Err(EvalError::early_return(
                        [(name.clone(), value)].into_iter().collect(),
                    ));
                }
                let mut result = HashMap::new();
                for (name, expr) in fields {
                    let value = eval_expr(expr, env, sibling_fns)?;
                    result.insert(name.clone(), value);
                }
                if is_fn_body {
                    return Ok(result);
                }
                return Err(EvalError::early_return(result));
            }
        }
    }

    // No explicit return — return unit
    Ok([("return".to_string(), Value::Unit)].into_iter().collect())
}

/// Evaluate a collection operation.
pub fn evaluate_collection(
    kind: &CollectionOpKind,
    items: Vec<Value>,
    inputs: &HashMap<String, Value>,
) -> Result<Value, EvalError> {
    match kind {
        CollectionOpKind::Map | CollectionOpKind::Filter | CollectionOpKind::FlatMap => {
            Ok(Value::List(items))
        }
        CollectionOpKind::Sort => {
            let mut sorted = items;
            sorted.sort_by_key(sort_key);
            Ok(Value::List(sorted))
        }
        CollectionOpKind::Dedup => {
            let mut out = Vec::new();
            for item in items {
                if !out.contains(&item) {
                    out.push(item);
                }
            }
            Ok(Value::List(out))
        }
        CollectionOpKind::Join => {
            let joined = items
                .iter()
                .map(value_to_string)
                .collect::<Vec<_>>()
                .join(",");
            Ok(Value::Str(joined))
        }
        CollectionOpKind::Fold | CollectionOpKind::Len => Ok(Value::Int(items.len() as i64)),
        CollectionOpKind::Any => Ok(Value::Bool(items.iter().any(value_truthy))),
        CollectionOpKind::All => Ok(Value::Bool(items.iter().all(value_truthy))),
        CollectionOpKind::Contains => {
            let needle = inputs
                .get("needle")
                .or_else(|| inputs.get("item"))
                .or_else(|| inputs.get("contains"));
            let found = needle
                .map(|needle| items.iter().any(|v| v == needle))
                .unwrap_or(false);
            Ok(Value::Bool(found))
        }
        // Split/Zip are passthrough for collection ops on pre-materialized item lists.
        CollectionOpKind::Split => Ok(Value::List(items)),
        CollectionOpKind::Zip => Ok(Value::List(items)),
        CollectionOpKind::Skip => {
            let n = inputs
                .get("n")
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i as usize),
                    _ => None,
                })
                .unwrap_or(0);
            Ok(Value::List(items.into_iter().skip(n).collect()))
        }
        CollectionOpKind::Enumerate => {
            let enumerated = items
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert("first".to_string(), Value::Int(i as i64));
                    map.insert("second".to_string(), v);
                    Value::Map(map)
                })
                .collect();
            Ok(Value::List(enumerated))
        }
    }
}

/// Collection/intrinsic function names handled by the evaluator.
///
/// These functions take a collection (or string) as the first positional
/// argument and operate on it. They were formerly pipe methods.
const INTRINSIC_CALLS: &[&str] = &[
    "map", "filter", "filter_map", "flat_map", "fold", "append",
    "join", "count", "sum", "first", "last", "any", "all", "contains",
    "sort_by", "split", "zip", "skip", "enumerate",
    "starts_with", "ends_with", "repeat", "chars",
];

/// Check if a function name is an evaluator-handled intrinsic.
pub fn is_intrinsic_call(name: &str) -> bool {
    INTRINSIC_CALLS.contains(&name)
}

/// Evaluate an intrinsic call: `name(receiver, args...)`.
///
/// The first positional argument is the receiver (collection or string).
/// Remaining arguments are forwarded as-is.
fn eval_intrinsic_call(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    // The first positional arg is the receiver.
    let receiver = if let Some((_, first_arg)) = args.first() {
        eval_expr(first_arg, env, sibling_fns)?
    } else {
        return Err(EvalError::new(format!("{name}: missing receiver argument")));
    };
    // Remaining args (skip the first positional one).
    let rest_args: Vec<(Option<String>, LoweredExpr)> = args[1..].to_vec();

    match name {
        "join" => {
            let sep = if let Some((_, sep_expr)) = rest_args.first() {
                match eval_expr(sep_expr, env, sibling_fns)? {
                    Value::Str(s) => s,
                    _ => ",".to_string(),
                }
            } else {
                ",".to_string()
            };
            match receiver {
                Value::List(items) => Ok(Value::Str(
                    items.iter().map(value_to_string).collect::<Vec<_>>().join(&sep),
                )),
                _ => Err(EvalError::new("join requires a list")),
            }
        }
        "map" => {
            let lambda = rest_args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item);
                        results.push(eval_expr(body, &child_env, sibling_fns)?);
                    }
                    Ok(Value::List(results))
                }
                (Value::List(items), Some(LoweredExpr::Ident(fn_name))) if sibling_fns.contains_key(fn_name.as_str()) => {
                    let fn_name = fn_name.clone();
                    let param = "_item".to_string();
                    let body = LoweredExpr::Call {
                        name: fn_name,
                        args: vec![(None, LoweredExpr::Ident(param.clone()))],
                    };
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item);
                        results.push(eval_expr(&body, &child_env, sibling_fns)?);
                    }
                    Ok(Value::List(results))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                (other, _) => Err(EvalError::new(format!("map requires a list, got {other:?}"))),
            }
        }
        "filter" => {
            let lambda = rest_args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item.clone());
                        if value_truthy(&eval_expr(body, &child_env, sibling_fns)?) {
                            results.push(item);
                        }
                    }
                    Ok(Value::List(results))
                }
                (Value::List(items), Some(LoweredExpr::Ident(fn_name))) if sibling_fns.contains_key(fn_name.as_str()) => {
                    let fn_name = fn_name.clone();
                    let param = "_item".to_string();
                    let body = LoweredExpr::Call {
                        name: fn_name,
                        args: vec![(None, LoweredExpr::Ident(param.clone()))],
                    };
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item.clone());
                        if value_truthy(&eval_expr(&body, &child_env, sibling_fns)?) {
                            results.push(item);
                        }
                    }
                    Ok(Value::List(results))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("filter requires a list")),
            }
        }
        "filter_map" => {
            let lambda = rest_args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item);
                        let val = eval_expr(body, &child_env, sibling_fns)?;
                        if !matches!(val, Value::Unit | Value::Skipped) {
                            results.push(val);
                        }
                    }
                    Ok(Value::List(results))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("filter_map requires a list")),
            }
        }
        "flat_map" => {
            let lambda = rest_args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item);
                        match eval_expr(body, &child_env, sibling_fns)? {
                            Value::List(inner) => results.extend(inner),
                            other => results.push(other),
                        }
                    }
                    Ok(Value::List(results))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("flat_map requires a list")),
            }
        }
        "fold" => {
            let init = rest_args.iter().find(|(k, _)| k.as_deref() == Some("init"));
            let func = rest_args.iter().find(|(k, _)| k.as_deref() == Some("f"));
            match (receiver, init, func) {
                (Value::List(items), Some((_, init_expr)), Some((_, LoweredExpr::Lambda { params, body }))) => {
                    let mut acc = eval_expr(init_expr, env, sibling_fns)?;
                    let acc_param = params.first().cloned().unwrap_or_else(|| "acc".to_string());
                    let item_param = params.get(1).cloned().unwrap_or_else(|| "item".to_string());
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(acc_param.clone(), acc);
                        child_env.bind(item_param.clone(), item);
                        acc = eval_expr(body, &child_env, sibling_fns)?;
                    }
                    Ok(acc)
                }
                _ => Err(EvalError::new("fold requires list, init, and f")),
            }
        }
        "append" => {
            let new_items = rest_args.iter().find(|(k, _)| k.as_deref() == Some("items"));
            match (receiver, new_items) {
                (Value::List(mut base), Some((_, items_expr))) => {
                    match eval_expr(items_expr, env, sibling_fns)? {
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
                if let Some(item) = items.into_iter().next() {
                    let mut map = BTreeMap::new();
                    map.insert("_variant".to_string(), Value::Str("Some".to_string()));
                    map.insert("value".to_string(), item);
                    Ok(Value::Map(map))
                } else {
                    let mut map = BTreeMap::new();
                    map.insert("_variant".to_string(), Value::Str("None".to_string()));
                    Ok(Value::Map(map))
                }
            }
            _ => Err(EvalError::new("first requires a list")),
        },
        "last" => match receiver {
            Value::List(items) => {
                if let Some(item) = items.into_iter().last() {
                    let mut map = BTreeMap::new();
                    map.insert("_variant".to_string(), Value::Str("Some".to_string()));
                    map.insert("value".to_string(), item);
                    Ok(Value::Map(map))
                } else {
                    let mut map = BTreeMap::new();
                    map.insert("_variant".to_string(), Value::Str("None".to_string()));
                    Ok(Value::Map(map))
                }
            }
            _ => Err(EvalError::new("last requires a list")),
        },
        "any" => {
            let lambda = rest_args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item);
                        if value_truthy(&eval_expr(body, &child_env, sibling_fns)?) {
                            return Ok(Value::Bool(true));
                        }
                    }
                    Ok(Value::Bool(false))
                }
                _ => Err(EvalError::new("any requires list and predicate")),
            }
        }
        "all" => {
            let lambda = rest_args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item);
                        if !value_truthy(&eval_expr(body, &child_env, sibling_fns)?) {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                }
                _ => Err(EvalError::new("all requires list and predicate")),
            }
        }
        "contains" => {
            let needle_expr = rest_args.first().or_else(|| rest_args.iter().find(|(k, _)| k.as_deref() == Some("item")));
            match (receiver, needle_expr) {
                (Value::List(items), Some((_, expr))) => {
                    let needle = eval_expr(expr, env, sibling_fns)?;
                    Ok(Value::Bool(items.contains(&needle)))
                }
                _ => Err(EvalError::new("contains requires list and item")),
            }
        }
        "sort_by" => {
            let lambda = rest_args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(mut items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let mut keyed: Vec<(String, Value)> = Vec::with_capacity(items.len());
                    for item in items.drain(..) {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item.clone());
                        let key = eval_expr(body, &child_env, sibling_fns).map(|v| value_to_string(&v))?;
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
            let prefix = rest_args.iter().find(|(k, _)| k.as_deref() == Some("prefix")).or_else(|| rest_args.first());
            match (receiver, prefix) {
                (Value::Str(s), Some((_, expr))) => Ok(Value::Bool(s.starts_with(&value_to_string(&eval_expr(expr, env, sibling_fns)?)))),
                _ => Err(EvalError::new("starts_with requires string and prefix")),
            }
        }
        "ends_with" => {
            let suffix = rest_args.iter().find(|(k, _)| k.as_deref() == Some("suffix")).or_else(|| rest_args.first());
            match (receiver, suffix) {
                (Value::Str(s), Some((_, expr))) => Ok(Value::Bool(s.ends_with(&value_to_string(&eval_expr(expr, env, sibling_fns)?)))),
                _ => Err(EvalError::new("ends_with requires string and suffix")),
            }
        }
        "split" => {
            let delim = rest_args.iter().find(|(k, _)| k.as_deref() == Some("delimiter")).or_else(|| rest_args.first());
            match (receiver, delim) {
                (Value::Str(s), Some((_, expr))) => {
                    let d = value_to_string(&eval_expr(expr, env, sibling_fns)?);
                    Ok(Value::List(s.split(&d).map(|p| Value::Str(p.to_string())).collect()))
                }
                (Value::Str(s), None) => Ok(Value::List(s.split(',').map(|p| Value::Str(p.to_string())).collect())),
                _ => Err(EvalError::new("split requires a string")),
            }
        }
        "zip" => {
            let other_expr = rest_args.iter().find(|(k, _)| k.as_deref() == Some("other")).or_else(|| rest_args.first());
            match (receiver, other_expr) {
                (Value::List(items), Some((_, expr))) => {
                    let other = match eval_expr(expr, env, sibling_fns)? {
                        Value::List(l) => l,
                        _ => return Err(EvalError::new("zip requires a list for 'other'")),
                    };
                    Ok(Value::List(items.into_iter().zip(other).map(|(a, b)| {
                        let mut map = BTreeMap::new();
                        map.insert("first".to_string(), a);
                        map.insert("second".to_string(), b);
                        Value::Map(map)
                    }).collect()))
                }
                _ => Err(EvalError::new("zip requires a list")),
            }
        }
        "skip" => {
            let n_expr = rest_args.iter().find(|(k, _)| k.as_deref() == Some("n")).or_else(|| rest_args.first());
            match (receiver, n_expr) {
                (Value::List(items), Some((_, expr))) => {
                    match eval_expr(expr, env, sibling_fns)? {
                        Value::Int(count) => Ok(Value::List(items.into_iter().skip(count.max(0) as usize).collect())),
                        _ => Err(EvalError::new("skip requires integer count")),
                    }
                }
                (Value::List(items), None) => Ok(Value::List(items)),
                _ => Err(EvalError::new("skip requires a list")),
            }
        }
        "enumerate" => match receiver {
            Value::List(items) => Ok(Value::List(items.into_iter().enumerate().map(|(i, v)| {
                let mut map = BTreeMap::new();
                map.insert("first".to_string(), Value::Int(i as i64));
                map.insert("second".to_string(), v);
                Value::Map(map)
            }).collect())),
            _ => Err(EvalError::new("enumerate requires a list")),
        },
        "repeat" => {
            let n_expr = rest_args.first();
            match (receiver, n_expr) {
                (Value::Str(s), Some((_, expr))) => {
                    match eval_expr(expr, env, sibling_fns)? {
                        Value::Int(count) => Ok(Value::Str(s.repeat(count.max(0) as usize))),
                        _ => Err(EvalError::new("repeat requires integer count")),
                    }
                }
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

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EvalError {
    pub message: String,
    /// Early return from a fn body — contains the return values.
    pub early_return: Option<HashMap<String, Value>>,
    /// Tail-call signal: a (possibly mutual) recursive call in tail position.
    /// Contains the new inputs for the next iteration.
    tail_call: Option<HashMap<String, Value>>,
    /// Name of the callee for mutual tail calls. `None` means self-recursive.
    tail_call_name: Option<String>,
}

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            early_return: None,
            tail_call: None,
            tail_call_name: None,
        }
    }

    pub fn early_return(values: HashMap<String, Value>) -> Self {
        Self {
            message: "__early_return__".to_string(),
            early_return: Some(values),
            tail_call: None,
            tail_call_name: None,
        }
    }

    fn tail_call(inputs: HashMap<String, Value>) -> Self {
        Self {
            message: "__tail_call__".to_string(),
            early_return: None,
            tail_call: Some(inputs),
            tail_call_name: None,
        }
    }

    fn mutual_tail_call(name: String, inputs: HashMap<String, Value>) -> Self {
        Self {
            message: "__tail_call__".to_string(),
            early_return: None,
            tail_call: Some(inputs),
            tail_call_name: Some(name),
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "eval error: {}", self.message)
    }
}

// ── Environment ─────────────────────────────────────────────────────────────

/// Maximum sibling-fn call depth before the evaluator bails out with a clear
/// error instead of blowing the native stack.
const MAX_CALL_DEPTH: usize = 10_000;

/// Maximum trampoline iterations for a single self-recursive function.
/// Higher than MAX_CALL_DEPTH because trampoline iterations are O(1) stack
/// (just a loop), not O(N) stack. The risk is infinite loops, not stack
/// overflow. 1M iterations is ~seconds of CPU, not a memory problem.
const MAX_TRAMPOLINE_ITERS: usize = 1_000_000;

struct Env {
    /// Variable bindings. Wrapped in Rc for copy-on-write: child scopes
    /// share the parent's map until they bind a new variable, at which
    /// point `Rc::make_mut` clones only if the refcount > 1.
    bindings: Rc<HashMap<String, Value>>,
    /// Data declaration values carried through so sibling fn calls can
    /// reference module-level `data` items without re-threading them.
    /// Wrapped in Rc to avoid deep-cloning on every recursive call.
    data_values: Rc<HashMap<String, Value>>,
    /// Current sibling-fn call depth (incremented in eval_fn_body_rc).
    call_depth: usize,
    /// Name of the currently-executing function (for tail-call detection).
    self_name: Option<String>,
}

impl Env {
    fn from_inputs(inputs: &HashMap<String, Value>) -> Self {
        Self {
            bindings: Rc::new(inputs.clone()),
            data_values: Rc::new(HashMap::new()),
            call_depth: 0,
            self_name: None,
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
            data_values: Rc::clone(&self.data_values),
            call_depth: self.call_depth,
            self_name: self.self_name.clone(),
        }
    }
}

// ── Expression evaluation ───────────────────────────────────────────────────

/// Evaluate an expression. Not in tail position (self-calls will recurse).
fn eval_expr(
    expr: &LoweredExpr,
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    eval_expr_tc(expr, env, sibling_fns, false)
}

/// Evaluate an expression with tail-call context.
/// When `tail_ctx` is true and a self-recursive call is encountered, a
/// `tail_call` signal is returned instead of recursing.
fn eval_expr_tc(
    expr: &LoweredExpr,
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    tail_ctx: bool,
) -> Result<Value, EvalError> {
    match expr {
        LoweredExpr::Literal(lit) => Ok(eval_literal(lit)),

        LoweredExpr::Ident(name) => {
            if name == "None" || name == "null" {
                return Ok(Value::Unit);
            }
            // Check env first — if bound, use the bound value
            if let Some(val) = env.get(name) {
                return Ok(val.clone());
            }
            // Capitalized identifiers without arguments are unit variants (e.g. `Closed`)
            if name.chars().next().unwrap_or('a').is_uppercase() {
                return Ok(Value::Str(name.clone()));
            }
            Err(EvalError::new(format!("unbound variable: {name}")))
        }

        LoweredExpr::FieldAccess { expr, field } => {
            let base = eval_expr(expr, env, sibling_fns)?;
            field_access(&base, field)
        }

        LoweredExpr::StringInterp(parts) => eval_string_interp(parts, env, sibling_fns),

        LoweredExpr::BinOp { left, op, right } => {
            let lhs = eval_expr(left, env, sibling_fns)?;
            // Short-circuit for logical operators
            match op {
                LoweredBinOp::And => {
                    if !value_truthy(&lhs) {
                        return Ok(Value::Bool(false));
                    }
                    let rhs = eval_expr(right, env, sibling_fns)?;
                    Ok(Value::Bool(value_truthy(&rhs)))
                }
                LoweredBinOp::Or => {
                    if value_truthy(&lhs) {
                        return Ok(Value::Bool(true));
                    }
                    let rhs = eval_expr(right, env, sibling_fns)?;
                    Ok(Value::Bool(value_truthy(&rhs)))
                }
                LoweredBinOp::NullCoalesce => {
                    if !matches!(lhs, Value::Unit | Value::Skipped) {
                        Ok(lhs)
                    } else {
                        eval_expr(right, env, sibling_fns)
                    }
                }
                _ => {
                    let rhs = eval_expr(right, env, sibling_fns)?;
                    // Fast path: list/string concat with owned values avoids
                    // cloning the left-hand side (O(1) amortized append instead
                    // of O(N) clone + extend).
                    if *op == LoweredBinOp::Add {
                        match (lhs, rhs) {
                            (Value::List(mut a), Value::List(b)) => {
                                a.extend(b);
                                return Ok(Value::List(a));
                            }
                            (Value::Str(mut a), Value::Str(b)) => {
                                a.push_str(&b);
                                return Ok(Value::Str(a));
                            }
                            (Value::Str(mut a), Value::Enum { variant, .. }) => {
                                a.push_str(&variant);
                                return Ok(Value::Str(a));
                            }
                            (Value::Enum { variant, .. }, Value::Str(b)) => {
                                return Ok(Value::Str(format!("{variant}{b}")));
                            }
                            (lhs, rhs) => {
                                return eval_binop(&lhs, *op, &rhs);
                            }
                        }
                    }
                    eval_binop(&lhs, *op, &rhs)
                }
            }
        }

        LoweredExpr::UnaryOp { op, expr } => {
            let val = eval_expr(expr, env, sibling_fns)?;
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
            let condition = eval_expr(cond, env, sibling_fns)?;
            if value_truthy(&condition) {
                eval_expr_tc(then_, env, sibling_fns, tail_ctx)
            } else if let Some(else_branch) = else_ {
                eval_expr_tc(else_branch, env, sibling_fns, tail_ctx)
            } else {
                Ok(Value::Unit)
            }
        }

        LoweredExpr::Match { expr, arms } => {
            let scrutinee = eval_expr(expr, env, sibling_fns)?;
            eval_match_inner_tc(&scrutinee, arms, env, sibling_fns, tail_ctx)
        }

        LoweredExpr::VariantConstruct { tag, fields } => {
            if fields.is_empty() {
                // Unit variant: `Closed` → Value::Enum { ty: "", variant: "Closed" }
                Ok(Value::Enum {
                    ty: String::new(),
                    variant: tag.clone(),
                })
            } else {
                // Payload variant: `Ok { value: x }` → Map with _variant tag
                let mut map = BTreeMap::new();
                map.insert("_variant".to_string(), Value::Str(tag.clone()));
                for (key, value_expr) in fields {
                    map.insert(key.clone(), eval_expr(value_expr, env, sibling_fns)?);
                }
                Ok(Value::Map(map))
            }
        }

        LoweredExpr::Call { name, args } => eval_call_tc(name, args, env, sibling_fns, tail_ctx),

        LoweredExpr::Lambda { .. } => {
            // Lambdas are evaluated inline when used in pipe methods
            Err(EvalError::new("lambda cannot be evaluated standalone"))
        }

        LoweredExpr::List(items) => {
            let values: Result<Vec<_>, _> = items
                .iter()
                .map(|item| eval_expr(item, env, sibling_fns))
                .collect();
            Ok(Value::List(values?))
        }

        LoweredExpr::Block(stmts) => eval_block_expr(stmts, env, sibling_fns, tail_ctx),

        LoweredExpr::Record { fields, .. } => eval_record_expr(fields, env, sibling_fns),

        LoweredExpr::For {
            binding,
            iterable,
            body,
        } => eval_for_expr(binding, iterable, body, env, sibling_fns),

        LoweredExpr::Return(fields) => eval_return_expr(fields, env, sibling_fns),
    }
}

/// Extracted from eval_expr_tc to reduce stack frame size.
#[inline(never)]
fn eval_block_expr(
    stmts: &[LoweredStmt],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    tail_ctx: bool,
) -> Result<Value, EvalError> {
    let mut child_env = env.child();
    let outputs = eval_stmts(stmts, &mut child_env, sibling_fns, tail_ctx, false)?;
    if outputs.len() == 1 {
        if let Some(value) = outputs.get("return") {
            return Ok(value.clone());
        }
    }
    Ok(Value::Map(outputs.into_iter().collect()))
}

/// Extracted from eval_expr_tc to reduce stack frame size.
#[inline(never)]
fn eval_record_expr(
    fields: &[(String, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    let mut map = BTreeMap::new();
    for (key, value_expr) in fields {
        let value = eval_expr(value_expr, env, sibling_fns)?;
        map.insert(key.clone(), value);
    }
    Ok(Value::Map(map))
}

/// Extracted from eval_expr_tc to reduce stack frame size.
#[inline(never)]
fn eval_for_expr(
    binding: &str,
    iterable: &LoweredExpr,
    body: &LoweredExpr,
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    let items = eval_expr(iterable, env, sibling_fns)?;
    let list = match items {
        Value::List(items) => items,
        other => vec![other],
    };
    let mut results = Vec::new();
    for item in list {
        let mut child_env = env.child();
        child_env.bind(binding.to_string(), item);
        results.push(eval_expr(body, &child_env, sibling_fns)?);
    }
    Ok(Value::List(results))
}

/// Extracted from eval_expr_tc to reduce stack frame size.
#[inline(never)]
fn eval_return_expr(
    fields: &[(String, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    if fields.len() == 1 {
        let (key, value_expr) = &fields[0];
        let value = eval_expr_tc(value_expr, env, sibling_fns, true)?;
        let mut map = HashMap::new();
        map.insert(key.clone(), value);
        return Err(EvalError::early_return(map));
    }
    let mut map = HashMap::new();
    for (key, value_expr) in fields {
        let value = eval_expr(value_expr, env, sibling_fns)?;
        map.insert(key.clone(), value);
    }
    Err(EvalError::early_return(map))
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

fn eval_string_interp(
    parts: &[LoweredStringPart],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    let mut result = String::new();
    for part in parts {
        match part {
            LoweredStringPart::Literal(s) => result.push_str(s),
            LoweredStringPart::Expr(expr) => {
                let value = eval_expr(expr, env, sibling_fns)?;
                result.push_str(&value_to_string(&value));
            }
        }
    }
    Ok(Value::Str(result))
}

fn field_access(base: &Value, field: &str) -> Result<Value, EvalError> {
    match base {
        Value::Map(map) => map
            .get(field)
            .cloned()
            .ok_or_else(|| EvalError::new(format!("no field '{field}' in map"))),
        Value::Json(json) => match json {
            serde_json::Value::Object(obj) => Ok(obj
                .get(field)
                .map(|v| Value::Json(v.clone()))
                // Missing JSON fields → JSON null (not Unit) to preserve
                // JSON semantics and avoid masking typos like `resp.mesage`.
                .unwrap_or(Value::Json(serde_json::Value::Null))),
            serde_json::Value::Null => Ok(Value::Json(serde_json::Value::Null)),
            _ => Err(EvalError::new(format!(
                "cannot access field '{field}' on JSON {:?}",
                json
            ))),
        },
        Value::Unit | Value::Skipped => Ok(Value::Unit),
        _ => Err(EvalError::new(format!(
            "cannot access field '{field}' on {:?}",
            base
        ))),
    }
}

pub fn eval_binop(lhs: &Value, op: LoweredBinOp, rhs: &Value) -> Result<Value, EvalError> {
    // Propagate Skipped through all binary ops except NullCoalesce (which handles it specially).
    if !matches!(op, LoweredBinOp::NullCoalesce)
        && (matches!(lhs, Value::Skipped) || matches!(rhs, Value::Skipped))
    {
        return Ok(Value::Skipped);
    }
    match op {
        // List concatenation — must be checked before string concat
        // because Value::List should never fall through to string coercion.
        LoweredBinOp::Add if matches!(lhs, Value::List(_)) || matches!(rhs, Value::List(_)) => {
            match (lhs, rhs) {
                (Value::List(a), Value::List(b)) => {
                    let mut result = a.clone();
                    result.extend(b.iter().cloned());
                    Ok(Value::List(result))
                }
                _ => Err(EvalError::new(format!(
                    "list concat requires both sides to be lists: {:?}, {:?}",
                    lhs, rhs
                ))),
            }
        }
        LoweredBinOp::Add
            if matches!(lhs, Value::Str(_) | Value::Enum { .. })
                || matches!(rhs, Value::Str(_) | Value::Enum { .. }) =>
        {
            Ok(Value::Str(format!(
                "{}{}",
                value_to_string(lhs),
                value_to_string(rhs)
            )))
        }
        // Arithmetic
        LoweredBinOp::Add => int_op(lhs, rhs, |a, b| a + b),
        LoweredBinOp::Sub => int_op(lhs, rhs, |a, b| a - b),
        LoweredBinOp::Mul => int_op(lhs, rhs, |a, b| a * b),
        LoweredBinOp::Div => int_op(lhs, rhs, |a, b| if b != 0 { a / b } else { 0 }),
        LoweredBinOp::Mod => int_op(lhs, rhs, |a, b| if b != 0 { a % b } else { 0 }),
        // Comparison
        LoweredBinOp::Eq => Ok(Value::Bool(values_equal(lhs, rhs))),
        LoweredBinOp::Ne => Ok(Value::Bool(!values_equal(lhs, rhs))),
        LoweredBinOp::Lt => cmp_op(lhs, rhs, |o| o.is_lt()),
        LoweredBinOp::Gt => cmp_op(lhs, rhs, |o| o.is_gt()),
        LoweredBinOp::Le => cmp_op(lhs, rhs, |o| o.is_le()),
        LoweredBinOp::Ge => cmp_op(lhs, rhs, |o| o.is_ge()),
        // Logical (non-short-circuit handled in caller)
        LoweredBinOp::And => Ok(Value::Bool(value_truthy(lhs) && value_truthy(rhs))),
        LoweredBinOp::Or => Ok(Value::Bool(value_truthy(lhs) || value_truthy(rhs))),
        LoweredBinOp::NullCoalesce => {
            if !matches!(lhs, Value::Unit | Value::Skipped) {
                Ok(lhs.clone())
            } else {
                Ok(rhs.clone())
            }
        }
    }
}

fn int_op(lhs: &Value, rhs: &Value, f: impl Fn(i64, i64) -> i64) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(f(*a, *b))),
        (Value::Skipped, _) | (_, Value::Skipped) => Ok(Value::Skipped),
        _ => Err(EvalError::new(format!(
            "arithmetic on non-integers: {:?}, {:?}",
            lhs, rhs
        ))),
    }
}

fn values_equal(lhs: &Value, rhs: &Value) -> bool {
    match (lhs, rhs) {
        (Value::Unit, Value::Unit) | (Value::Skipped, Value::Skipped) => true,
        (Value::Unit, Value::Skipped) | (Value::Skipped, Value::Unit) => true,
        (Value::Enum { variant, .. }, Value::Str(s))
        | (Value::Str(s), Value::Enum { variant, .. }) => variant == s,
        _ => lhs == rhs,
    }
}

fn cmp_op(
    lhs: &Value,
    rhs: &Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, EvalError> {
    if matches!(lhs, Value::Skipped) || matches!(rhs, Value::Skipped) {
        return Ok(Value::Skipped);
    }
    // Unit compared with anything is false (not an error). This handles
    // cases like `char_at` returning Unit for out-of-bounds positions,
    // which then flows into `is_digit(ch)` → `ch >= "0"`.
    if matches!(lhs, Value::Unit) || matches!(rhs, Value::Unit) {
        return Ok(Value::Bool(false));
    }
    let ordering = match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Str(a), Value::Str(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        _ => {
            return Err(EvalError::new(format!(
                "cannot compare {:?} with {:?}",
                lhs, rhs
            )))
        }
    };
    Ok(Value::Bool(pred(ordering)))
}

fn eval_call_tc(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    tail_ctx: bool,
) -> Result<Value, EvalError> {
    // Check sibling fn first
    if let Some(fn_body) = sibling_fns.get(name) {
        let mut fn_inputs = HashMap::new();
        for (param_name, arg_expr) in args {
            let value = eval_expr(arg_expr, env, sibling_fns)?;
            if let Some(name) = param_name {
                fn_inputs.insert(name.clone(), value);
            }
        }
        // Tail-call optimization: if we're in tail position, signal a
        // trampoline instead of recursing on the native stack.
        // Handles both self-recursive (A→A) and mutual (A→B) tail calls.
        if tail_ctx {
            if env.self_name.as_deref() == Some(name) {
                // Self-recursive tail call
                return Err(EvalError::tail_call(fn_inputs));
            }
            // Mutual tail call — signal with callee name
            return Err(EvalError::mutual_tail_call(name.to_string(), fn_inputs));
        }
        let outputs = eval_fn_body_rc(fn_body, &fn_inputs, sibling_fns, Rc::clone(&env.data_values), env.call_depth + 1, Some(name))?;
        return sibling_fn_value(name, outputs);
    }

    // Collection / intrinsic functions: map, filter, fold, join, etc.
    // These were formerly pipe methods (items |> map(f)) and are now
    // called as regular functions (map(items, f)).
    if is_intrinsic_call(name) {
        return eval_intrinsic_call(name, args, env, sibling_fns);
    }

    // Built-in functions
    match name {
        // Record update: `expr with { field: value }`
        "with" => {
            if args.len() >= 2 {
                let base = eval_expr(&args[0].1, env, sibling_fns)?;
                let updates = eval_expr(&args[1].1, env, sibling_fns)?;
                record_update(&base, &updates)
            } else {
                Err(EvalError::new("'with' requires base and updates"))
            }
        }
        // Option transparent wrapper: return the first argument directly
        "Some" => {
            if let Some((_, arg_expr)) = args.first() {
                eval_expr(arg_expr, env, sibling_fns)
            } else {
                Ok(Value::Unit)
            }
        }
        // code_point(c: Char) -> Int: Unicode scalar value of a character
        "code_point" => {
            let val = eval_named_arg("c", args, env, sibling_fns)?;
            match &val {
                Value::Str(s) => {
                    let c = s.chars().next().ok_or_else(|| EvalError::new("code_point: empty string"))?;
                    Ok(Value::Int(c as i64))
                }
                Value::Int(n) => Ok(Value::Int(*n)),
                _ => Err(EvalError::new(format!("code_point: expected Char, got {:?}", val))),
            }
        }
        // from_code_point(cp: Int) -> String: Unicode scalar value to character
        "from_code_point" => {
            let val = eval_positional_or_named("cp", 0, args, env, sibling_fns)?;
            match val {
                Value::Int(cp) => {
                    if let Some(c) = char::from_u32(cp as u32) {
                        Ok(Value::Str(c.to_string()))
                    } else {
                        Err(EvalError::new(format!("from_code_point: invalid code point {cp}")))
                    }
                }
                _ => Err(EvalError::new("from_code_point: expected Int")),
            }
        }
        // to_string(value: Int) -> String: integer to string conversion
        "to_string" => {
            let val = eval_positional_or_named("value", 0, args, env, sibling_fns)?;
            Ok(Value::Str(value_to_string(&val)))
        }
        // ── v2 kernel intrinsics ───────────────────────────────────────
        "char_at" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            let pos = eval_positional_or_named("pos", 1, args, env, sibling_fns)?;
            match (s, pos) {
                (Value::Str(s), Value::Int(i)) => {
                    match s.chars().nth(i as usize) {
                        Some(c) => Ok(Value::Str(c.to_string())),
                        None => Ok(Value::Unit),
                    }
                }
                _ => Err(EvalError::new("char_at requires (String, Int)")),
            }
        }
        "substring" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            let start = eval_positional_or_named("start", 1, args, env, sibling_fns)?;
            let end = eval_positional_or_named("end", 2, args, env, sibling_fns)?;
            match (s, start, end) {
                (Value::Str(s), Value::Int(start), Value::Int(end)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as i64;
                    let start = (start.max(0) as usize).min(len as usize);
                    let end = (end.max(0) as usize).min(len as usize);
                    let slice: String = chars[start..end].iter().collect();
                    Ok(Value::Str(slice))
                }
                _ => Err(EvalError::new("substring requires (String, Int, Int)")),
            }
        }
        "string_length" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            match s {
                Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
                _ => Err(EvalError::new("string_length requires a String")),
            }
        }
        "parse_int" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            match s {
                Value::Str(s) => {
                    let n = s.trim().parse::<i64>().map_err(|e| {
                        EvalError::new(format!("parse_int: cannot parse '{s}': {e}"))
                    })?;
                    Ok(Value::Int(n))
                }
                _ => Err(EvalError::new("parse_int requires a String")),
            }
        }
        "scan_while" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            let start = eval_positional_or_named("start", 1, args, env, sibling_fns)?;
            // The pred argument is a lambda or function reference — get it unevaluated.
            let pred_expr = get_arg_expr("pred", 2, args);
            // Handle function references: `pred: is_ident_char` → wrap as Call
            let resolved_pred = match pred_expr {
                Some(LoweredExpr::Lambda { params, body }) => {
                    Some((params.clone(), body.as_ref().clone()))
                }
                Some(LoweredExpr::Ident(name)) if sibling_fns.contains_key(name.as_str()) => {
                    // Function reference → synthetic lambda: ch => fn_name(ch: ch)
                    let param = "ch".to_string();
                    let body = LoweredExpr::Call {
                        name: name.clone(),
                        args: vec![(Some("ch".to_string()), LoweredExpr::Ident(param.clone()))],
                    };
                    Some((vec![param], body))
                }
                _ => None,
            };
            match (s, start, resolved_pred) {
                (Value::Str(s), Value::Int(start), Some((params, body))) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let chars: Vec<char> = s.chars().collect();
                    let mut pos = start.max(0) as usize;
                    while pos < chars.len() {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), Value::Str(chars[pos].to_string()));
                        let result = eval_expr(&body, &child_env, sibling_fns)?;
                        if !value_truthy(&result) {
                            break;
                        }
                        pos += 1;
                    }
                    Ok(Value::Int(pos as i64))
                }
                _ => Err(EvalError::new("scan_while requires (String, Int, Lambda)")),
            }
        }
        "scan_string_end" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            let start = eval_positional_or_named("start", 1, args, env, sibling_fns)?;
            match (s, start) {
                (Value::Str(s), Value::Int(start)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let mut pos = start.max(0) as usize;
                    while pos < chars.len() {
                        if chars[pos] == '\\' {
                            pos += 2; // skip escaped char
                        } else if chars[pos] == '"' {
                            return Ok(Value::Int((pos + 1) as i64));
                        } else {
                            pos += 1;
                        }
                    }
                    // No closing quote found — return end of string
                    Ok(Value::Int(chars.len() as i64))
                }
                _ => Err(EvalError::new("scan_string_end requires (String, Int)")),
            }
        }
        "scan_to_eol" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            let start = eval_positional_or_named("start", 1, args, env, sibling_fns)?;
            match (s, start) {
                (Value::Str(s), Value::Int(start)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let start = start.max(0) as usize;
                    for (i, &ch) in chars.iter().enumerate().skip(start) {
                        if ch == '\n' {
                            return Ok(Value::Int(i as i64));
                        }
                    }
                    Ok(Value::Int(chars.len() as i64))
                }
                _ => Err(EvalError::new("scan_to_eol requires (String, Int)")),
            }
        }
        "skip_horizontal_ws" => {
            let s = eval_positional_or_named("s", 0, args, env, sibling_fns)?;
            let start = eval_positional_or_named("start", 1, args, env, sibling_fns)?;
            match (s, start) {
                (Value::Str(s), Value::Int(start)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let mut pos = start.max(0) as usize;
                    while pos < chars.len() && (chars[pos] == ' ' || chars[pos] == '\t') {
                        pos += 1;
                    }
                    Ok(Value::Int(pos as i64))
                }
                _ => Err(EvalError::new("skip_horizontal_ws requires (String, Int)")),
            }
        }
        "lookup" => {
            let map_val = eval_positional_or_named("map", 0, args, env, sibling_fns)?;
            let key = eval_positional_or_named("key", 1, args, env, sibling_fns)?;
            match (map_val, key) {
                (Value::Map(map), Value::Str(key)) => {
                    if let Some(value) = map.get(&key) {
                        let mut result = BTreeMap::new();
                        result.insert("_variant".to_string(), Value::Str("Some".to_string()));
                        result.insert("value".to_string(), value.clone());
                        Ok(Value::Map(result))
                    } else {
                        let mut result = BTreeMap::new();
                        result.insert("_variant".to_string(), Value::Str("None".to_string()));
                        Ok(Value::Map(result))
                    }
                }
                _ => Err(EvalError::new("lookup requires (Map, String)")),
            }
        }
        // chars is handled by eval_intrinsic_call via INTRINSIC_CALLS.
        _ if name.chars().next().unwrap_or('a').is_uppercase() => {
            // Generic variant constructor (e.g. `Ok { value: "x" }`, `Closed`)
            let mut map = BTreeMap::new();
            map.insert("_variant".to_string(), Value::Str(name.to_string()));
            for (idx, (arg_name, arg_expr)) in args.iter().enumerate() {
                let field_name = arg_name.clone().unwrap_or_else(|| format!("_{idx}"));
                map.insert(field_name, eval_expr(arg_expr, env, sibling_fns)?);
            }
            Ok(Value::Map(map))
        }
        // Dotted names are service calls (e.g., llm.Anthropic.Messages) handled
        // by DAG transport nodes, not the expression evaluator. Return a neutral
        // placeholder so fn body evaluation doesn't fail during mock probing.
        _ if name.contains('.') => Ok(Value::Unit),
        _ => Err(EvalError::new(format!("unknown function: {name}"))),
    }
}

/// Evaluate a named argument from a call's argument list.
fn eval_named_arg(
    param: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    for (name, expr) in args {
        if name.as_deref() == Some(param) {
            return eval_expr(expr, env, sibling_fns);
        }
    }
    // Fall back to positional first arg
    if let Some((_, expr)) = args.first() {
        return eval_expr(expr, env, sibling_fns);
    }
    Err(EvalError::new(format!("missing argument '{param}'")))
}

/// Evaluate an argument by name or positional index.
///
/// Tries named lookup first, then falls back to positional index.
fn eval_positional_or_named(
    param: &str,
    index: usize,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    // Try named first
    for (name, expr) in args {
        if name.as_deref() == Some(param) {
            return eval_expr(expr, env, sibling_fns);
        }
    }
    // Fall back to positional
    if let Some((_, expr)) = args.get(index) {
        return eval_expr(expr, env, sibling_fns);
    }
    Err(EvalError::new(format!("missing argument '{param}'")))
}

/// Get an argument expression by name or positional index without evaluating it.
///
/// Used for lambda arguments that must remain as `LoweredExpr::Lambda`.
fn get_arg_expr<'a>(
    param: &str,
    index: usize,
    args: &'a [(Option<String>, LoweredExpr)],
) -> Option<&'a LoweredExpr> {
    // Try named first
    for (name, expr) in args {
        if name.as_deref() == Some(param) {
            return Some(expr);
        }
    }
    // Fall back to positional
    args.get(index).map(|(_, expr)| expr)
}

fn record_update(base: &Value, updates: &Value) -> Result<Value, EvalError> {
    match (base, updates) {
        (Value::Map(base_map), Value::Map(update_map)) => {
            let mut result = base_map.clone();
            for (k, v) in update_map {
                result.insert(k.clone(), v.clone());
            }
            Ok(Value::Map(result))
        }
        _ => Err(EvalError::new("'with' requires record values")),
    }
}

fn sibling_fn_value(name: &str, outputs: HashMap<String, Value>) -> Result<Value, EvalError> {
    if let Some(value) = outputs.get("return") {
        return Ok(value.clone());
    }
    // Single `value` key from `return expr` (which lowers to Return([("value", expr)]))
    if outputs.len() == 1 {
        if let Some(value) = outputs.get("value") {
            return Ok(value.clone());
        }
    }
    if outputs.is_empty() {
        return Err(EvalError::new(format!("fn {name} produced no output")));
    }
    Ok(Value::Map(outputs.into_iter().collect()))
}

pub fn eval_match(
    scrutinee: &Value,
    arms: &[LoweredMatchArm],
    env_bindings: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    let env = Env::from_inputs(env_bindings);
    eval_match_inner(scrutinee, arms, &env, sibling_fns)
}

fn eval_match_inner(
    scrutinee: &Value,
    arms: &[LoweredMatchArm],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    eval_match_inner_tc(scrutinee, arms, env, sibling_fns, false)
}

fn eval_match_inner_tc(
    scrutinee: &Value,
    arms: &[LoweredMatchArm],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    tail_ctx: bool,
) -> Result<Value, EvalError> {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, scrutinee) {
            let mut child_env = env.child();
            for (name, value) in bindings {
                child_env.bind(name, value);
            }
            // Check guard
            if let Some(guard) = &arm.guard {
                let guard_val = eval_expr(guard, &child_env, sibling_fns)?;
                if !value_truthy(&guard_val) {
                    continue;
                }
            }
            return eval_expr_tc(&arm.body, &child_env, sibling_fns, tail_ctx);
        }
    }
    Err(EvalError::new(format!(
        "no match arm matched value: {:?}",
        scrutinee
    )))
}

fn match_pattern(pattern: &LoweredPattern, value: &Value) -> Option<Vec<(String, Value)>> {
    match pattern {
        LoweredPattern::Wildcard => Some(vec![]),
        LoweredPattern::Ident(name) => Some(vec![(name.clone(), value.clone())]),
        LoweredPattern::Literal(lit) => {
            let expected = eval_literal(lit);
            if values_equal(&expected, value) {
                return Some(vec![]);
            }
            // Also match structural None: Map with _variant: "None"
            if matches!(lit, LoweredLiteral::None) {
                if let Value::Map(map) = value {
                    if let Some(Value::Str(tag)) = map.get("_variant") {
                        if tag == "None" {
                            return Some(vec![]);
                        }
                    }
                }
            }
            None
        }
        LoweredPattern::Variant(variant_name, fields) => {
            // Option matching: `Some { value: x }` / `None`
            if variant_name == "Some" && fields.len() == 1 {
                // Structural Option: Map with _variant field matching Some/None
                if let Value::Map(map) = value {
                    if let Some(Value::Str(tag)) = map.get("_variant") {
                        if tag == "Some" {
                            // Extract inner value and match sub-pattern against it
                            let inner = map.get("value").cloned().unwrap_or(Value::Unit);
                            return match_pattern(&fields[0].1, &inner);
                        }
                        if tag == "None" {
                            // Structural None → doesn't match Some
                            return None;
                        }
                        // Other variant tags (e.g., "Ident") → fall through to transparent
                    }
                }
                // Transparent matching for non-structural values
                if !matches!(value, Value::Unit | Value::Skipped) {
                    return match_pattern(&fields[0].1, value);
                }
                return None;
            }
            // Sum type variant matching: check if value is a Map with _variant field
            match value {
                Value::Map(map) => {
                    let variant = map.get("_variant").and_then(|v| {
                        if let Value::Str(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    });
                    if variant == Some(variant_name.as_str()) {
                        let mut bindings = vec![];
                        for (field_name, sub_pattern) in fields {
                            let field_value = map.get(field_name).cloned().unwrap_or(Value::Unit);
                            if let Some(sub_bindings) = match_pattern(sub_pattern, &field_value) {
                                bindings.extend(sub_bindings);
                            } else {
                                return None;
                            }
                        }
                        Some(bindings)
                    } else {
                        None
                    }
                }
                // Unit variants match by enum-variant equality.
                Value::Enum { variant, .. } if variant == variant_name => Some(vec![]),
                // Backward-compat path for older snapshots.
                Value::Str(s) if s == variant_name => Some(vec![]),
                _ => None,
            }
        }
    }
}


// ── Value utilities (moved from resolve.rs) ─────────────────────────────────

/// Sort key for collection ordering.
pub fn sort_key(value: &Value) -> String {
    match value {
        Value::Str(s) => format!("s:{s}"),
        Value::Int(i) => format!("i:{i:020}"),
        Value::Bool(b) => format!("b:{b}"),
        Value::List(items) => format!("l:{}", items.len()),
        Value::Map(map) => format!("m:{}", map.len()),
        Value::Set(items) => format!("set:{}", items.len()),
        Value::Json(json) => format!("j:{json}"),
        Value::Request(request) => format!("req:{request:?}"),
        Value::Response(response) => format!("resp:{response:?}"),
        Value::Secret(secret) => format!("secret:{}", secret.len()),
        Value::Enum { ty, variant } => format!("enum:{ty}:{variant}"),
        Value::Float(f) => format!("f:{f}"),
        Value::Bytes(b) => format!("bytes:{}", b.len()),
        Value::Skipped => "skipped".to_string(),
        Value::Unit => "unit".to_string(),
    }
}

/// Convert Value to display string.
pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::Str(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Unit => "".to_string(),
        Value::Skipped => "".to_string(),
        Value::List(items) => {
            let inner: Vec<String> = items.iter().map(value_to_string).collect();
            inner.join(", ")
        }
        Value::Map(map) => format!("map({})", map.len()),
        Value::Set(items) => format!("set({})", items.len()),
        Value::Json(json) => json.to_string(),
        Value::Request(request) => format!("{request:?}"),
        Value::Response(response) => format!("{response:?}"),
        Value::Secret(secret) => format!("secret({})", secret.len()),
        Value::Enum { ty, variant } => {
            if ty.is_empty() {
                variant.clone()
            } else {
                format!("{ty}.{variant}")
            }
        }
        Value::Float(f) => f.to_string(),
        Value::Bytes(b) => format!("<{} bytes>", b.len()),
    }
}

/// Truthiness of a Value.
pub fn value_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Str(s) => !s.is_empty(),
        Value::List(items) | Value::Set(items) => !items.is_empty(),
        Value::Map(map) => !map.is_empty(),
        Value::Json(json) => !json.is_null(),
        Value::Secret(secret) => !secret.is_empty(),
        Value::Bytes(b) => !b.is_empty(),
        Value::Enum { .. } => true,
        Value::Skipped | Value::Unit => false,
        Value::Request(_) | Value::Response(_) => true,
    }
}

// ── Primitive op helpers (called from resolve.rs PurePrimitiveOp) ────────────

/// Extract a field from a Map or Json object.
///
/// Unlike the private `field_access`, this returns detailed error messages
/// listing available fields — suitable for DAG executor diagnostics.
pub fn eval_get_field(value: &Value, field: &str) -> Result<Value, EvalError> {
    match value {
        Value::Map(fields) => fields.get(field).cloned().ok_or_else(|| {
            let mut available: Vec<&String> = fields.keys().collect();
            available.sort();
            EvalError::new(format!(
                "GetField `{field}`: field not found in Map. Available fields: {available:?}"
            ))
        }),
        Value::Json(serde_json::Value::Object(map)) => {
            map.get(field).map(|v| Value::Json(v.clone())).ok_or_else(|| {
                let mut available: Vec<&String> = map.keys().collect();
                available.sort();
                EvalError::new(format!(
                    "GetField `{field}`: field not found in Json object. Available fields: {available:?}"
                ))
            })
        }
        Value::Skipped => Err(EvalError::new(format!(
            "GetField `{field}`: input is Skipped (unwired or missing upstream)"
        ))),
        other => Err(EvalError::new(format!(
            "GetField `{field}`: expected Map or Json object, got {other:?}"
        ))),
    }
}

/// Interpolate a string from static parts and dynamic values.
///
/// `parts[i]` is static text, `values[i]` is the interpolated expression value
/// between `parts[i]` and `parts[i+1]`. If any value is `Skipped`, returns `Skipped`.
pub fn eval_string_interpolate(parts: &[String], values: &[Value]) -> Result<Value, EvalError> {
    for v in values {
        if matches!(v, Value::Skipped) {
            return Ok(Value::Skipped);
        }
    }
    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        result.push_str(part);
        if i < values.len() {
            result.push_str(&value_to_string(&values[i]));
        }
    }
    Ok(Value::Str(result))
}

/// Evaluate a unary operation (Not, Neg).
pub fn eval_unary_op(op: LoweredUnaryOp, value: &Value) -> Result<Value, EvalError> {
    if matches!(value, Value::Skipped) {
        return Ok(Value::Skipped);
    }
    match op {
        LoweredUnaryOp::Not => Ok(Value::Bool(!value_truthy(value))),
        LoweredUnaryOp::Neg => match value {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            other => Err(EvalError::new(format!("UnaryOp Neg: cannot negate {other:?}"))),
        },
    }
}

/// Evaluate a conditional (if/else).
///
/// Returns `then_val` if condition is truthy, `else_val` if present and condition
/// is falsy, or `Skipped` if no else branch.
pub fn eval_conditional(
    condition: &Value,
    then_val: &Value,
    else_val: Option<&Value>,
) -> Value {
    if matches!(condition, Value::Skipped) {
        return Value::Skipped;
    }
    if value_truthy(condition) {
        then_val.clone()
    } else if let Some(else_v) = else_val {
        else_v.clone()
    } else {
        Value::Skipped
    }
}

/// Construct a record from named fields.
///
/// Returns `Skipped` if any field value is `Skipped`.
pub fn eval_record_construct(fields: &[(String, Value)]) -> Result<Value, EvalError> {
    let mut map = BTreeMap::new();
    for (name, value) in fields {
        if matches!(value, Value::Skipped) {
            return Ok(Value::Skipped);
        }
        map.insert(name.clone(), value.clone());
    }
    Ok(Value::Map(map))
}

/// Null-coalesce: return `value` if non-Unit/non-Skipped, else `default`.
pub fn eval_null_coalesce(value: &Value, default: &Value) -> Value {
    if matches!(value, Value::Unit | Value::Skipped) {
        default.clone()
    } else {
        value.clone()
    }
}

/// Construct a variant value.
///
/// Unit variants (no fields) → `Value::Enum`. Payload variants → `Value::Map`
/// with `_variant` tag. Returns `Skipped` if any payload field is `Skipped`.
pub fn eval_variant_construct(tag: &str, fields: &[(String, Value)]) -> Result<Value, EvalError> {
    if fields.is_empty() {
        return Ok(Value::Enum {
            ty: String::new(),
            variant: tag.to_string(),
        });
    }
    let mut map = BTreeMap::new();
    map.insert("_variant".to_string(), Value::Str(tag.to_string()));
    for (name, value) in fields {
        if matches!(value, Value::Skipped) {
            return Ok(Value::Skipped);
        }
        map.insert(name.clone(), value.clone());
    }
    Ok(Value::Map(map))
}

/// Construct a list from elements.
///
/// Returns `Skipped` if any element is `Skipped`.
pub fn eval_list_construct(elements: Vec<Value>) -> Result<Value, EvalError> {
    for elem in &elements {
        if matches!(elem, Value::Skipped) {
            return Ok(Value::Skipped);
        }
    }
    Ok(Value::List(elements))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::*;

    fn empty_siblings() -> HashMap<String, LoweredFnBody> {
        HashMap::new()
    }

    #[test]
    fn eval_string_interpolation() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::StringInterp(vec![
                    LoweredStringPart::Literal("# ".to_string()),
                    LoweredStringPart::Expr(LoweredExpr::Ident("title".to_string())),
                ]),
            )])],
        };
        let inputs: HashMap<String, Value> =
            [("title".to_string(), Value::Str("Hello".to_string()))]
                .into_iter()
                .collect();
        let result = evaluate_fn_body(&body, &inputs, &empty_siblings()).unwrap();
        assert_eq!(result["return"], Value::Str("# Hello".to_string()));
    }

    #[test]
    fn eval_let_binding_and_return() {
        let body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Let(
                    "x".to_string(),
                    LoweredExpr::BinOp {
                        left: Box::new(LoweredExpr::Ident("a".to_string())),
                        op: LoweredBinOp::Add,
                        right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(1))),
                    },
                ),
                LoweredStmt::Return(vec![(
                    "return".to_string(),
                    LoweredExpr::Ident("x".to_string()),
                )]),
            ],
        };
        let inputs: HashMap<String, Value> =
            [("a".to_string(), Value::Int(41))].into_iter().collect();
        let result = evaluate_fn_body(&body, &inputs, &empty_siblings()).unwrap();
        assert_eq!(result["return"], Value::Int(42));
    }

    #[test]
    fn eval_sibling_fn_call() {
        // fn greet(name: String) -> String { return { return: "hi {name}" } }
        let greet_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::StringInterp(vec![
                    LoweredStringPart::Literal("hi ".to_string()),
                    LoweredStringPart::Expr(LoweredExpr::Ident("name".to_string())),
                ]),
            )])],
        };

        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::Call {
                    name: "greet".to_string(),
                    args: vec![(
                        Some("name".to_string()),
                        LoweredExpr::Literal(LoweredLiteral::String("world".to_string())),
                    )],
                },
            )])],
        };

        let siblings: HashMap<String, LoweredFnBody> =
            [("greet".to_string(), greet_body)].into_iter().collect();
        let result = evaluate_fn_body(&body, &HashMap::new(), &siblings).unwrap();
        assert_eq!(result["return"], Value::Str("hi world".to_string()));
    }

    #[test]
    fn eval_sibling_fn_preserves_named_record_outputs() {
        let auth_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "token".to_string(),
                LoweredExpr::Literal(LoweredLiteral::String("secret".to_string())),
            )])],
        };

        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::Call {
                    name: "auth".to_string(),
                    args: vec![],
                },
            )])],
        };

        let siblings: HashMap<String, LoweredFnBody> =
            [("auth".to_string(), auth_body)].into_iter().collect();
        let result = evaluate_fn_body(&body, &HashMap::new(), &siblings).unwrap();

        let expected = Value::Map(
            [("token".to_string(), Value::Str("secret".to_string()))]
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
        );
        assert_eq!(result["return"], expected);
    }

    #[test]
    fn eval_if_else_with_none_comparison() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(LoweredExpr::Ident("x".to_string())),
                        op: LoweredBinOp::Ne,
                        right: Box::new(LoweredExpr::Literal(LoweredLiteral::None)),
                    }),
                    then_: Box::new(LoweredExpr::Ident("x".to_string())),
                    else_: Some(Box::new(LoweredExpr::Literal(LoweredLiteral::String(
                        "default".to_string(),
                    )))),
                },
            )])],
        };

        // x = "hello" → returns "hello"
        let inputs: HashMap<String, Value> = [("x".to_string(), Value::Str("hello".to_string()))]
            .into_iter()
            .collect();
        let result = evaluate_fn_body(&body, &inputs, &empty_siblings()).unwrap();
        assert_eq!(result["return"], Value::Str("hello".to_string()));

        // x = Unit (none) → returns "default"
        let inputs: HashMap<String, Value> = [("x".to_string(), Value::Unit)].into_iter().collect();
        let result = evaluate_fn_body(&body, &inputs, &empty_siblings()).unwrap();
        assert_eq!(result["return"], Value::Str("default".to_string()));
    }

    #[test]
    fn eval_collection_join() {
        let items = vec![Value::Str("a".to_string()), Value::Str("b".to_string())];
        let result = evaluate_collection(&CollectionOpKind::Join, items, &HashMap::new()).unwrap();
        assert_eq!(result, Value::Str("a,b".to_string()));
    }

    #[test]
    fn eval_collection_sort() {
        let items = vec![
            Value::Str("c".to_string()),
            Value::Str("a".to_string()),
            Value::Str("b".to_string()),
        ];
        let result = evaluate_collection(&CollectionOpKind::Sort, items, &HashMap::new()).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
                Value::Str("c".to_string()),
            ])
        );
    }

    #[test]
    fn eval_field_access_on_map() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::FieldAccess {
                    expr: Box::new(LoweredExpr::Ident("rec".to_string())),
                    field: "name".to_string(),
                },
            )])],
        };
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), Value::Str("test".to_string()));
        let inputs: HashMap<String, Value> =
            [("rec".to_string(), Value::Map(map))].into_iter().collect();
        let result = evaluate_fn_body(&body, &inputs, &empty_siblings()).unwrap();
        assert_eq!(result["return"], Value::Str("test".to_string()));
    }

    #[test]
    fn eval_collection_skip() {
        let items = vec![
            Value::Str("a".to_string()),
            Value::Str("b".to_string()),
            Value::Str("c".to_string()),
        ];
        let mut inputs = HashMap::new();
        inputs.insert("n".to_string(), Value::Int(1));
        let result = evaluate_collection(&CollectionOpKind::Skip, items, &inputs).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Str("b".to_string()),
                Value::Str("c".to_string()),
            ])
        );
    }

    #[test]
    fn eval_collection_enumerate() {
        let items = vec![Value::Str("x".to_string()), Value::Str("y".to_string())];
        let result =
            evaluate_collection(&CollectionOpKind::Enumerate, items, &HashMap::new()).unwrap();
        let expected = Value::List(vec![
            Value::Map({
                let mut m = BTreeMap::new();
                m.insert("first".to_string(), Value::Int(0));
                m.insert("second".to_string(), Value::Str("x".to_string()));
                m
            }),
            Value::Map({
                let mut m = BTreeMap::new();
                m.insert("first".to_string(), Value::Int(1));
                m.insert("second".to_string(), Value::Str("y".to_string()));
                m
            }),
        ]);
        assert_eq!(result, expected);
    }

    // ── v2 kernel intrinsic tests ─────────────────────────────────────

    /// Helper: evaluate a call expression and return the result.
    fn eval_call_expr(
        name: &str,
        args: Vec<(Option<String>, LoweredExpr)>,
        inputs: &HashMap<String, Value>,
    ) -> Result<Value, EvalError> {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::Call {
                    name: name.to_string(),
                    args,
                },
            )])],
        };
        let result = evaluate_fn_body(&body, inputs, &empty_siblings())?;
        Ok(result.get("return").cloned().unwrap_or(Value::Unit))
    }

    #[test]
    fn test_char_at() {
        let result = eval_call_expr(
            "char_at",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello".to_string()))),
                (Some("pos".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(1))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Str("e".to_string()));
    }

    #[test]
    fn test_char_at_out_of_bounds() {
        let result = eval_call_expr(
            "char_at",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hi".to_string()))),
                (Some("pos".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(5))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_char_at_unicode() {
        // Multi-byte characters: char_at uses chars() so it works on codepoints
        let result = eval_call_expr(
            "char_at",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("\u{00e9}bc".to_string()))),
                (Some("pos".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Str("\u{00e9}".to_string()));
    }

    #[test]
    fn test_substring() {
        let result = eval_call_expr(
            "substring",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello world".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
                (Some("end".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(5))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_substring_clamped() {
        // End beyond string length should clamp
        let result = eval_call_expr(
            "substring",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hi".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
                (Some("end".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(100))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Str("hi".to_string()));
    }

    #[test]
    fn test_substring_empty() {
        let result = eval_call_expr(
            "substring",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(3))),
                (Some("end".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(3))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Str(String::new()));
    }

    #[test]
    fn test_string_length() {
        let result = eval_call_expr(
            "string_length",
            vec![(Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())))],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_string_length_empty() {
        let result = eval_call_expr(
            "string_length",
            vec![(Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String(String::new())))],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_parse_int() {
        let result = eval_call_expr(
            "parse_int",
            vec![(Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("42".to_string())))],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_parse_int_with_whitespace() {
        let result = eval_call_expr(
            "parse_int",
            vec![(Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("  -7  ".to_string())))],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(-7));
    }

    #[test]
    fn test_parse_int_invalid() {
        let result = eval_call_expr(
            "parse_int",
            vec![(Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("abc".to_string())))],
            &HashMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_while() {
        // Scan digits from position 0 in "123abc"
        let result = eval_call_expr(
            "scan_while",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("123abc".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
                (Some("pred".to_string()), LoweredExpr::Lambda {
                    params: vec!["c".to_string()],
                    body: Box::new(LoweredExpr::BinOp {
                        left: Box::new(LoweredExpr::BinOp {
                            left: Box::new(LoweredExpr::Call {
                                name: "code_point".to_string(),
                                args: vec![(Some("c".to_string()), LoweredExpr::Ident("c".to_string()))],
                            }),
                            op: LoweredBinOp::Ge,
                            right: Box::new(LoweredExpr::Call {
                                name: "code_point".to_string(),
                                args: vec![(Some("c".to_string()), LoweredExpr::Literal(LoweredLiteral::String("0".to_string())))],
                            }),
                        }),
                        op: LoweredBinOp::And,
                        right: Box::new(LoweredExpr::BinOp {
                            left: Box::new(LoweredExpr::Call {
                                name: "code_point".to_string(),
                                args: vec![(Some("c".to_string()), LoweredExpr::Ident("c".to_string()))],
                            }),
                            op: LoweredBinOp::Le,
                            right: Box::new(LoweredExpr::Call {
                                name: "code_point".to_string(),
                                args: vec![(Some("c".to_string()), LoweredExpr::Literal(LoweredLiteral::String("9".to_string())))],
                            }),
                        }),
                    }),
                }),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_scan_while_all_match() {
        // All characters match predicate
        let result = eval_call_expr(
            "scan_while",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("aaa".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
                (Some("pred".to_string()), LoweredExpr::Lambda {
                    params: vec!["c".to_string()],
                    body: Box::new(LoweredExpr::Literal(LoweredLiteral::Bool(true))),
                }),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_scan_string_end() {
        // Input: hello" (start right after the opening quote)
        let result = eval_call_expr(
            "scan_string_end",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello\"rest".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(6)); // position after the closing "
    }

    #[test]
    fn test_scan_string_end_with_escape() {
        // Input: he\"llo" — escaped quote should be skipped
        let result = eval_call_expr(
            "scan_string_end",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("he\\\"llo\"rest".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(8)); // skip he\", then llo", position after closing "
    }

    #[test]
    fn test_scan_string_end_no_closing() {
        // No closing quote — return string length
        let result = eval_call_expr(
            "scan_string_end",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_scan_to_eol() {
        let result = eval_call_expr(
            "scan_to_eol",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello\nworld".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_scan_to_eol_no_newline() {
        let result = eval_call_expr(
            "scan_to_eol",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_scan_to_eol_from_offset() {
        let result = eval_call_expr(
            "scan_to_eol",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("ab\ncd\nef".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(3))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_skip_horizontal_ws() {
        let result = eval_call_expr(
            "skip_horizontal_ws",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("   \thello".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(4));
    }

    #[test]
    fn test_skip_horizontal_ws_no_ws() {
        let result = eval_call_expr(
            "skip_horizontal_ws",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("hello".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_skip_horizontal_ws_ignores_newline() {
        // Newlines are NOT horizontal whitespace
        let result = eval_call_expr(
            "skip_horizontal_ws",
            vec![
                (Some("s".to_string()), LoweredExpr::Literal(LoweredLiteral::String("  \nhello".to_string()))),
                (Some("start".to_string()), LoweredExpr::Literal(LoweredLiteral::Int(0))),
            ],
            &HashMap::new(),
        ).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_lookup_found() {
        let mut map = BTreeMap::new();
        map.insert("x".to_string(), Value::Int(42));
        let inputs: HashMap<String, Value> = [("m".to_string(), Value::Map(map))].into_iter().collect();
        let result = eval_call_expr(
            "lookup",
            vec![
                (Some("map".to_string()), LoweredExpr::Ident("m".to_string())),
                (Some("key".to_string()), LoweredExpr::Literal(LoweredLiteral::String("x".to_string()))),
            ],
            &inputs,
        ).unwrap();
        let expected = Value::Map({
            let mut m = BTreeMap::new();
            m.insert("_variant".to_string(), Value::Str("Some".to_string()));
            m.insert("value".to_string(), Value::Int(42));
            m
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lookup_not_found() {
        let map = BTreeMap::new();
        let inputs: HashMap<String, Value> = [("m".to_string(), Value::Map(map))].into_iter().collect();
        let result = eval_call_expr(
            "lookup",
            vec![
                (Some("map".to_string()), LoweredExpr::Ident("m".to_string())),
                (Some("key".to_string()), LoweredExpr::Literal(LoweredLiteral::String("missing".to_string()))),
            ],
            &inputs,
        ).unwrap();
        let expected = Value::Map({
            let mut m = BTreeMap::new();
            m.insert("_variant".to_string(), Value::Str("None".to_string()));
            m
        });
        assert_eq!(result, expected);
    }

    /// Self-recursive tail-call function: count_down(n) calls count_down(n-1)
    /// until n <= 0. Without TCO this would overflow the stack at large N.
    #[test]
    fn tco_self_recursive_tail_call() {
        // Build: fn count_down(n: Int) -> Int {
        //   if n <= 0 { return 0 }
        //   count_down(n: n - 1)
        // }
        let body = LoweredFnBody {
            stmts: vec![
                // if n <= 0 { return 0 }
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(LoweredExpr::Ident("n".to_string())),
                        op: LoweredBinOp::Le,
                        right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(0))),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![(
                        "return".to_string(),
                        LoweredExpr::Literal(LoweredLiteral::Int(0)),
                    )])),
                    else_: None,
                }),
                // count_down(n: n - 1)  — last expression, tail position
                LoweredStmt::Expr(LoweredExpr::Call {
                    name: "count_down".to_string(),
                    args: vec![(
                        Some("n".to_string()),
                        LoweredExpr::BinOp {
                            left: Box::new(LoweredExpr::Ident("n".to_string())),
                            op: LoweredBinOp::Sub,
                            right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(1))),
                        },
                    )],
                }),
            ],
        };

        let mut sibling_fns = HashMap::new();
        sibling_fns.insert("count_down".to_string(), body.clone());

        // N=20000 would blow a default 8MB stack without TCO.
        let mut inputs = HashMap::new();
        inputs.insert("n".to_string(), Value::Int(20_000));

        let result = evaluate_fn_body(&body, &inputs, &sibling_fns).unwrap();
        assert_eq!(result["return"], Value::Int(0));
    }

    /// Self-recursive tail call via explicit `return fn(...)` inside an
    /// if-else branch (the pattern used by tokenize_loop, scan_string_body).
    #[test]
    fn tco_self_recursive_return_in_if_branch() {
        // Build: fn sum_down(n: Int, acc: Int) -> Int {
        //   if n <= 0 { return acc }
        //   return sum_down(n: n - 1, acc: acc + n)
        // }
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(LoweredExpr::IfElse {
                cond: Box::new(LoweredExpr::BinOp {
                    left: Box::new(LoweredExpr::Ident("n".to_string())),
                    op: LoweredBinOp::Le,
                    right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(0))),
                }),
                then_: Box::new(LoweredExpr::Return(vec![(
                    "return".to_string(),
                    LoweredExpr::Ident("acc".to_string()),
                )])),
                else_: Some(Box::new(LoweredExpr::Return(vec![(
                    "return".to_string(),
                    LoweredExpr::Call {
                        name: "sum_down".to_string(),
                        args: vec![
                            (
                                Some("n".to_string()),
                                LoweredExpr::BinOp {
                                    left: Box::new(LoweredExpr::Ident("n".to_string())),
                                    op: LoweredBinOp::Sub,
                                    right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(1))),
                                },
                            ),
                            (
                                Some("acc".to_string()),
                                LoweredExpr::BinOp {
                                    left: Box::new(LoweredExpr::Ident("acc".to_string())),
                                    op: LoweredBinOp::Add,
                                    right: Box::new(LoweredExpr::Ident("n".to_string())),
                                },
                            ),
                        ],
                    },
                )]))),
            })],
        };

        let mut sibling_fns = HashMap::new();
        sibling_fns.insert("sum_down".to_string(), body.clone());

        let mut inputs = HashMap::new();
        inputs.insert("n".to_string(), Value::Int(20_000));
        inputs.insert("acc".to_string(), Value::Int(0));

        let result = evaluate_fn_body(&body, &inputs, &sibling_fns).unwrap();
        // sum 1..20000 = 20000 * 20001 / 2 = 200_010_000
        assert_eq!(result["return"], Value::Int(200_010_000));
    }

    /// Mutual tail-call recursion: A calls B calls A, trampolined on the heap.
    /// This would overflow an 8MB stack without mutual TCO.
    #[test]
    fn tco_mutual_recursive_tail_call() {
        // fn is_even(n: Int) -> Bool {
        //   if n == 0 { return { return: true } }
        //   is_odd(n: n - 1)          // last expr stmt → mutual tail call
        // }
        // fn is_odd(n: Int) -> Bool {
        //   if n == 0 { return { return: false } }
        //   is_even(n: n - 1)         // last expr stmt → mutual tail call
        // }
        let is_even_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(LoweredExpr::Ident("n".to_string())),
                        op: LoweredBinOp::Eq,
                        right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(0))),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![(
                        "return".to_string(),
                        LoweredExpr::Literal(LoweredLiteral::Bool(true)),
                    )])),
                    else_: None,
                }),
                LoweredStmt::Expr(LoweredExpr::Call {
                    name: "is_odd".to_string(),
                    args: vec![(
                        Some("n".to_string()),
                        LoweredExpr::BinOp {
                            left: Box::new(LoweredExpr::Ident("n".to_string())),
                            op: LoweredBinOp::Sub,
                            right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(1))),
                        },
                    )],
                }),
            ],
        };

        let is_odd_body = LoweredFnBody {
            stmts: vec![
                LoweredStmt::Expr(LoweredExpr::IfElse {
                    cond: Box::new(LoweredExpr::BinOp {
                        left: Box::new(LoweredExpr::Ident("n".to_string())),
                        op: LoweredBinOp::Eq,
                        right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(0))),
                    }),
                    then_: Box::new(LoweredExpr::Return(vec![(
                        "return".to_string(),
                        LoweredExpr::Literal(LoweredLiteral::Bool(false)),
                    )])),
                    else_: None,
                }),
                LoweredStmt::Expr(LoweredExpr::Call {
                    name: "is_even".to_string(),
                    args: vec![(
                        Some("n".to_string()),
                        LoweredExpr::BinOp {
                            left: Box::new(LoweredExpr::Ident("n".to_string())),
                            op: LoweredBinOp::Sub,
                            right: Box::new(LoweredExpr::Literal(LoweredLiteral::Int(1))),
                        },
                    )],
                }),
            ],
        };

        let mut sibling_fns = HashMap::new();
        sibling_fns.insert("is_even".to_string(), is_even_body.clone());
        sibling_fns.insert("is_odd".to_string(), is_odd_body);

        // N=40000 would overflow a default 8MB stack without mutual TCO.
        let mut inputs = HashMap::new();
        inputs.insert("n".to_string(), Value::Int(40_000));
        let result = evaluate_fn_body(&is_even_body, &inputs, &sibling_fns).unwrap();
        assert_eq!(result["return"], Value::Bool(true)); // 40000 is even

        inputs.insert("n".to_string(), Value::Int(40_001));
        let result = evaluate_fn_body(&is_even_body, &inputs, &sibling_fns).unwrap();
        assert_eq!(result["return"], Value::Bool(false)); // 40001 is odd
    }

    /// Infinite mutual tail recursion (A→B→A) must produce a clean error.
    #[test]
    fn tco_infinite_mutual_tail_recursion_is_caught() {
        // fn ping() { pong() }
        // fn pong() { ping() }
        let ping_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(LoweredExpr::Call {
                name: "pong".to_string(),
                args: vec![],
            })],
        };
        let pong_body = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(LoweredExpr::Call {
                name: "ping".to_string(),
                args: vec![],
            })],
        };

        let mut sibling_fns = HashMap::new();
        sibling_fns.insert("ping".to_string(), ping_body.clone());
        sibling_fns.insert("pong".to_string(), pong_body);

        let result = evaluate_fn_body(&ping_body, &HashMap::new(), &sibling_fns);
        assert!(result.is_err(), "infinite mutual recursion should error");
        let msg = result.unwrap_err().message;
        assert!(
            msg.contains("tail-call iterations") || msg.contains("call depth"),
            "error should mention iteration limit, got: {}",
            msg
        );
    }

    /// Infinite self-tail-recursion must produce a clean error, not hang.
    #[test]
    fn tco_infinite_tail_recursion_is_caught() {
        // fn spin() { spin() }
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Expr(LoweredExpr::Call {
                name: "spin".to_string(),
                args: vec![],
            })],
        };

        let mut sibling_fns = HashMap::new();
        sibling_fns.insert("spin".to_string(), body.clone());

        let result = evaluate_fn_body(&body, &HashMap::new(), &sibling_fns);
        assert!(result.is_err(), "infinite tail recursion should error");
        let msg = result.unwrap_err().message;
        assert!(
            msg.contains("tail-call iterations") || msg.contains("call depth"),
            "error should mention iteration limit, got: {}",
            msg
        );
    }
}
