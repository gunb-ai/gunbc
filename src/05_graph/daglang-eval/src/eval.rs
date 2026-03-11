//! Pure evaluation engine for lowered expression IR.
//!
//! Evaluates `LoweredFnBody` and collection operations using only `Value`
//! types from `gunbc-ir`. No dependency on `gunbc-exec` — pure functions only.
//! Thin DynOp wrappers in `resolve.rs` call these functions.

use std::collections::{BTreeMap, HashMap};

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
    data_values: &HashMap<String, serde_json::Value>,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut env = Env::from_inputs(inputs);
    // Store data_values so sibling fn calls can access them.
    env.data_values = data_values.clone();
    // Seed data declarations into the environment (lower priority than inputs).
    for (name, json_val) in data_values {
        if !env.bindings.contains_key(name) {
            env.bind(name.clone(), json_to_eval_value(json_val));
        }
    }

    for stmt in &body.stmts {
        match stmt {
            LoweredStmt::Let(name, expr) => {
                let value = eval_expr(expr, &env, sibling_fns)?;
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
                let value = eval_expr(expr, &env, sibling_fns)?;
                // If this is the last statement and produces a value, capture as "return"
                if std::ptr::eq(stmt, body.stmts.last().unwrap()) {
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
            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, expr) in fields {
                    let value = eval_expr(expr, &env, sibling_fns)?;
                    result.insert(name.clone(), value);
                }
                return Ok(result);
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
                    map.insert("index".to_string(), Value::Int(i as i64));
                    map.insert("value".to_string(), v);
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
            Value::List(items) => Ok(items.into_iter().next().unwrap_or(Value::Unit)),
            _ => Err(EvalError::new("first requires a list")),
        },
        "last" => match receiver {
            Value::List(items) => Ok(items.into_iter().last().unwrap_or(Value::Unit)),
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
                map.insert("index".to_string(), Value::Int(i as i64));
                map.insert("value".to_string(), v);
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
}

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "eval error: {}", self.message)
    }
}

// ── Environment ─────────────────────────────────────────────────────────────

struct Env {
    bindings: HashMap<String, Value>,
    /// Data declaration values carried through so sibling fn calls can
    /// reference module-level `data` items without re-threading them.
    data_values: HashMap<String, serde_json::Value>,
}

impl Env {
    fn from_inputs(inputs: &HashMap<String, Value>) -> Self {
        Self {
            bindings: inputs.clone(),
            data_values: HashMap::new(),
        }
    }

    fn bind(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    fn child(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
            data_values: self.data_values.clone(),
        }
    }
}

// ── Expression evaluation ───────────────────────────────────────────────────

fn eval_expr(
    expr: &LoweredExpr,
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
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
                eval_expr(then_, env, sibling_fns)
            } else if let Some(else_branch) = else_ {
                eval_expr(else_branch, env, sibling_fns)
            } else {
                Ok(Value::Unit)
            }
        }

        LoweredExpr::Match { expr, arms } => {
            let scrutinee = eval_expr(expr, env, sibling_fns)?;
            eval_match_inner(&scrutinee, arms, env, sibling_fns)
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

        LoweredExpr::Call { name, args } => eval_call(name, args, env, sibling_fns),

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

        LoweredExpr::Block(stmts) => {
            let body = LoweredFnBody {
                stmts: stmts.clone(),
            };
            let outputs = evaluate_block_body(&body, env, sibling_fns)?;
            if outputs.len() == 1 {
                if let Some(value) = outputs.get("return") {
                    return Ok(value.clone());
                }
            }
            Ok(Value::Map(outputs.into_iter().collect()))
        }

        LoweredExpr::Record { fields, .. } => {
            let mut map = BTreeMap::new();
            for (key, value_expr) in fields {
                let value = eval_expr(value_expr, env, sibling_fns)?;
                map.insert(key.clone(), value);
            }
            Ok(Value::Map(map))
        }

        LoweredExpr::For {
            binding,
            iterable,
            body,
        } => {
            let items = eval_expr(iterable, env, sibling_fns)?;
            let list = match items {
                Value::List(items) => items,
                other => vec![other],
            };
            let mut results = Vec::new();
            for item in list {
                let mut child_env = env.child();
                child_env.bind(binding.clone(), item);
                results.push(eval_expr(body, &child_env, sibling_fns)?);
            }
            Ok(Value::List(results))
        }

        LoweredExpr::Return(fields) => {
            let mut map = BTreeMap::new();
            for (key, value_expr) in fields {
                let value = eval_expr(value_expr, env, sibling_fns)?;
                map.insert(key.clone(), value);
            }
            Ok(Value::Map(map))
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn evaluate_block_body(
    body: &LoweredFnBody,
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut child_env = env.child();

    for stmt in &body.stmts {
        match stmt {
            LoweredStmt::Let(name, expr) => {
                let value = eval_expr(expr, &child_env, sibling_fns)?;
                match &value {
                    Value::Map(fields) => {
                        for (field_name, field_value) in fields {
                            child_env.bind(format!("{name}__{field_name}"), field_value.clone());
                        }
                    }
                    Value::Json(serde_json::Value::Object(map)) => {
                        for (field_name, field_value) in map {
                            child_env.bind(
                                format!("{name}__{field_name}"),
                                Value::Json(field_value.clone()),
                            );
                        }
                    }
                    _ => {}
                }
                child_env.bind(name.clone(), value);
            }
            LoweredStmt::Expr(expr) => {
                let value = eval_expr(expr, &child_env, sibling_fns)?;
                if std::ptr::eq(stmt, body.stmts.last().unwrap()) {
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
            LoweredStmt::Return(fields) => {
                let mut result = HashMap::new();
                for (name, expr) in fields {
                    let value = eval_expr(expr, &child_env, sibling_fns)?;
                    result.insert(name.clone(), value);
                }
                return Ok(result);
            }
        }
    }

    Ok([("return".to_string(), Value::Unit)].into_iter().collect())
}

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

fn eval_call(
    name: &str,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
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
        let outputs = evaluate_fn_body_with_data(fn_body, &fn_inputs, sibling_fns, &env.data_values)?;
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

/// Convert a `serde_json::Value` to a `Value` for the evaluator environment.
/// Recursively converts objects to `Value::Map` and arrays to `Value::List`
/// so that field access and collection operations work on data declarations.
fn json_to_eval_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Unit,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Str(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            Value::List(arr.iter().map(json_to_eval_value).collect())
        }
        serde_json::Value::Object(map) => {
            let btree: BTreeMap<String, Value> =
                map.iter().map(|(k, v)| (k.clone(), json_to_eval_value(v))).collect();
            Value::Map(btree)
        }
    }
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
            return eval_expr(&arm.body, &child_env, sibling_fns);
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
                Some(vec![])
            } else {
                None
            }
        }
        LoweredPattern::Variant(variant_name, fields) => {
            // Option transparent matching: `Some(v)` matches anything except Unit/Skipped
            if variant_name == "Some" && fields.len() == 1 {
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
                m.insert("index".to_string(), Value::Int(0));
                m.insert("value".to_string(), Value::Str("x".to_string()));
                m
            }),
            Value::Map({
                let mut m = BTreeMap::new();
                m.insert("index".to_string(), Value::Int(1));
                m.insert("value".to_string(), Value::Str("y".to_string()));
                m
            }),
        ]);
        assert_eq!(result, expected);
    }
}
