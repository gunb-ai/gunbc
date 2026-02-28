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
use crate::CollectionOpKind;

// ── Public API ──────────────────────────────────────────────────────────────

/// Evaluate a lowered fn body with the given inputs.
///
/// `sibling_fns` maps fn names → their lowered bodies for recursive calls
/// within the same module.
pub fn evaluate_fn_body(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<HashMap<String, Value>, EvalError> {
    let mut env = Env::from_inputs(inputs);

    for stmt in &body.stmts {
        match stmt {
            LoweredStmt::Let(name, expr) => {
                let value = eval_expr(expr, &env, sibling_fns)?;
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
        // Split/Zip are handled as pipe methods in eval_pipe_method, not as
        // collection ops on pre-materialized item lists.
        CollectionOpKind::Split => Ok(Value::List(items)),
        CollectionOpKind::Zip => Ok(Value::List(items)),
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
}

impl Env {
    fn from_inputs(inputs: &HashMap<String, Value>) -> Self {
        Self {
            bindings: inputs.clone(),
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
            eval_match(&scrutinee, arms, env, sibling_fns)
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

        LoweredExpr::Pipe { receiver, call } => {
            let recv_val = eval_expr(receiver, env, sibling_fns)?;
            eval_pipe(recv_val, call, env, sibling_fns)
        }

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
        Value::Unit | Value::Skipped => Ok(Value::Unit),
        _ => Err(EvalError::new(format!(
            "cannot access field '{field}' on {:?}",
            base
        ))),
    }
}

fn eval_binop(lhs: &Value, op: LoweredBinOp, rhs: &Value) -> Result<Value, EvalError> {
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
        let outputs = evaluate_fn_body(fn_body, &fn_inputs, sibling_fns)?;
        // Return the primary output (first key or "return")
        return outputs
            .get("return")
            .or_else(|| outputs.values().next())
            .cloned()
            .ok_or_else(|| EvalError::new(format!("fn {name} produced no output")));
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
        _ => Err(EvalError::new(format!("unknown function: {name}"))),
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

fn eval_match(
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

// ── Pipe method evaluation ──────────────────────────────────────────────────

fn eval_pipe(
    receiver: Value,
    call: &LoweredExpr,
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    match call {
        LoweredExpr::Call { name, args } => {
            eval_pipe_method(name, receiver, args, env, sibling_fns)
        }
        _ => Err(EvalError::new("pipe RHS must be a call")),
    }
}

fn eval_pipe_method(
    method: &str,
    receiver: Value,
    args: &[(Option<String>, LoweredExpr)],
    env: &Env,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    match method {
        "join" => {
            let sep = if let Some((_, sep_expr)) = args.first() {
                match eval_expr(sep_expr, env, sibling_fns)? {
                    Value::Str(s) => s,
                    _ => ",".to_string(),
                }
            } else {
                ",".to_string()
            };
            match receiver {
                Value::List(items) => {
                    let joined = items
                        .iter()
                        .map(value_to_string)
                        .collect::<Vec<_>>()
                        .join(&sep);
                    Ok(Value::Str(joined))
                }
                Value::Skipped => Ok(Value::Str(String::new())),
                _ => Err(EvalError::new("join requires a list")),
            }
        }

        "map" => {
            let lambda = args.first().map(|(_, e)| e);
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
                (Value::Skipped, _) => Ok(Value::List(vec![])),
                (other, _) => Err(EvalError::new(format!("map requires a list, got {:?}", other))),
            }
        }

        "filter" => {
            let lambda = args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item.clone());
                        let keep = eval_expr(body, &child_env, sibling_fns)?;
                        if value_truthy(&keep) {
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
            let lambda = args.first().map(|(_, e)| e);
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
            let lambda = args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    let mut results = Vec::new();
                    for item in items {
                        let mut child_env = env.child();
                        child_env.bind(param.clone(), item);
                        let val = eval_expr(body, &child_env, sibling_fns)?;
                        match val {
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
            let init = args.iter().find(|(k, _)| k.as_deref() == Some("init"));
            let func = args.iter().find(|(k, _)| k.as_deref() == Some("f"));
            match (receiver, init, func) {
                (
                    Value::List(items),
                    Some((_, init_expr)),
                    Some((_, LoweredExpr::Lambda { params, body })),
                ) => {
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
            let new_items = args.iter().find(|(k, _)| k.as_deref() == Some("items"));
            match (receiver, new_items) {
                (Value::List(mut base), Some((_, items_expr))) => {
                    let to_append = eval_expr(items_expr, env, sibling_fns)?;
                    match to_append {
                        Value::List(more) => base.extend(more),
                        other => base.push(other),
                    }
                    Ok(Value::List(base))
                }
                (other, _) => Err(EvalError::new(format!(
                    "append requires a list, got {:?}",
                    other
                ))),
            }
        }

        "count" => match receiver {
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
            Value::List(items) => Ok(items.into_iter().next().unwrap_or(Value::Unit)),
            _ => Err(EvalError::new("first requires a list")),
        },

        "last" => match receiver {
            Value::List(items) => Ok(items.into_iter().last().unwrap_or(Value::Unit)),
            _ => Err(EvalError::new("last requires a list")),
        },

        "any" => {
            let lambda = args.first().map(|(_, e)| e);
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
            let lambda = args.first().map(|(_, e)| e);
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
            let needle_expr = args
                .first()
                .or_else(|| args.iter().find(|(k, _)| k.as_deref() == Some("item")));
            match (receiver, needle_expr) {
                (Value::List(items), Some((_, expr))) => {
                    let needle = eval_expr(expr, env, sibling_fns)?;
                    Ok(Value::Bool(items.contains(&needle)))
                }
                _ => Err(EvalError::new("contains requires list and item")),
            }
        }

        "sort_by" => {
            let lambda = args.first().map(|(_, e)| e);
            match (receiver, lambda) {
                (Value::List(mut items), Some(LoweredExpr::Lambda { params, body })) => {
                    let param = params.first().cloned().unwrap_or_else(|| "_".to_string());
                    // Compute sort keys, then sort
                    let mut keyed: Vec<(String, Value)> = items
                        .drain(..)
                        .map(|item| {
                            let mut child_env = env.child();
                            child_env.bind(param.clone(), item.clone());
                            let key = eval_expr(body, &child_env, sibling_fns)
                                .map(|v| value_to_string(&v))
                                .unwrap_or_default();
                            (key, item)
                        })
                        .collect();
                    keyed.sort_by(|a, b| a.0.cmp(&b.0));
                    Ok(Value::List(keyed.into_iter().map(|(_, v)| v).collect()))
                }
                (Value::List(items), _) => Ok(Value::List(items)),
                _ => Err(EvalError::new("sort_by requires a list")),
            }
        }

        // String methods
        "starts_with" => {
            let prefix = args
                .iter()
                .find(|(k, _)| k.as_deref() == Some("prefix"))
                .or_else(|| args.first());
            match (receiver, prefix) {
                (Value::Str(s), Some((_, expr))) => {
                    let p = eval_expr(expr, env, sibling_fns)?;
                    Ok(Value::Bool(s.starts_with(&value_to_string(&p))))
                }
                _ => Err(EvalError::new("starts_with requires string and prefix")),
            }
        }

        "ends_with" => {
            let suffix = args
                .iter()
                .find(|(k, _)| k.as_deref() == Some("suffix"))
                .or_else(|| args.first());
            match (receiver, suffix) {
                (Value::Str(s), Some((_, expr))) => {
                    let p = eval_expr(expr, env, sibling_fns)?;
                    Ok(Value::Bool(s.ends_with(&value_to_string(&p))))
                }
                _ => Err(EvalError::new("ends_with requires string and suffix")),
            }
        }

        "split" => {
            let delim = args
                .iter()
                .find(|(k, _)| k.as_deref() == Some("delimiter"))
                .or_else(|| args.first());
            match (receiver, delim) {
                (Value::Str(s), Some((_, expr))) => {
                    let d = eval_expr(expr, env, sibling_fns)?;
                    let delimiter = value_to_string(&d);
                    let parts: Vec<Value> = s
                        .split(&delimiter)
                        .map(|part| Value::Str(part.to_string()))
                        .collect();
                    Ok(Value::List(parts))
                }
                (Value::Str(s), None) => {
                    // Default delimiter: ","
                    let parts: Vec<Value> = s
                        .split(',')
                        .map(|part| Value::Str(part.to_string()))
                        .collect();
                    Ok(Value::List(parts))
                }
                _ => Err(EvalError::new("split requires a string")),
            }
        }

        "zip" => {
            let other_expr = args
                .iter()
                .find(|(k, _)| k.as_deref() == Some("other"))
                .or_else(|| args.first());
            match (receiver, other_expr) {
                (Value::List(items), Some((_, expr))) => {
                    let other = eval_expr(expr, env, sibling_fns)?;
                    let other_list = match other {
                        Value::List(l) => l,
                        _ => return Err(EvalError::new("zip requires a list for 'other'")),
                    };
                    let pairs: Vec<Value> = items
                        .into_iter()
                        .zip(other_list)
                        .map(|(a, b)| {
                            let mut map = std::collections::BTreeMap::new();
                            map.insert("first".to_string(), a);
                            map.insert("second".to_string(), b);
                            Value::Map(map)
                        })
                        .collect();
                    Ok(Value::List(pairs))
                }
                (Value::List(_), None) => Err(EvalError::new("zip requires 'other' argument")),
                _ => Err(EvalError::new("zip requires a list")),
            }
        }

        "repeat" => {
            let n_expr = args.first();
            match (receiver, n_expr) {
                (Value::Str(s), Some((_, expr))) => {
                    let n = eval_expr(expr, env, sibling_fns)?;
                    match n {
                        Value::Int(count) => Ok(Value::Str(s.repeat(count.max(0) as usize))),
                        _ => Err(EvalError::new("repeat requires integer count")),
                    }
                }
                _ => Err(EvalError::new("repeat requires string and count")),
            }
        }

        // Passthrough for unknown pipe methods — try as sibling fn call
        _ => {
            if let Some(fn_body) = sibling_fns.get(method) {
                // Call sibling fn with receiver as first positional arg
                let mut fn_inputs = HashMap::new();
                // Try to map receiver to first param
                fn_inputs.insert("_receiver".to_string(), receiver);
                for (param_name, arg_expr) in args {
                    let value = eval_expr(arg_expr, env, sibling_fns)?;
                    if let Some(name) = param_name {
                        fn_inputs.insert(name.clone(), value);
                    }
                }
                let outputs = evaluate_fn_body(fn_body, &fn_inputs, sibling_fns)?;
                outputs
                    .get("return")
                    .or_else(|| outputs.values().next())
                    .cloned()
                    .ok_or_else(|| EvalError::new(format!("pipe fn {method} produced no output")))
            } else {
                Err(EvalError::new(format!("unknown pipe method: {method}")))
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
    fn eval_pipe_join() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::Pipe {
                    receiver: Box::new(LoweredExpr::Ident("items".to_string())),
                    call: Box::new(LoweredExpr::Call {
                        name: "join".to_string(),
                        args: vec![(
                            None,
                            LoweredExpr::Literal(LoweredLiteral::String("\n".to_string())),
                        )],
                    }),
                },
            )])],
        };
        let inputs: HashMap<String, Value> = [(
            "items".to_string(),
            Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ]),
        )]
        .into_iter()
        .collect();
        let result = evaluate_fn_body(&body, &inputs, &empty_siblings()).unwrap();
        assert_eq!(result["return"], Value::Str("a\nb".to_string()));
    }

    #[test]
    fn eval_pipe_map_with_lambda() {
        let body = LoweredFnBody {
            stmts: vec![LoweredStmt::Return(vec![(
                "return".to_string(),
                LoweredExpr::Pipe {
                    receiver: Box::new(LoweredExpr::Ident("items".to_string())),
                    call: Box::new(LoweredExpr::Call {
                        name: "map".to_string(),
                        args: vec![(
                            None,
                            LoweredExpr::Lambda {
                                params: vec!["x".to_string()],
                                body: Box::new(LoweredExpr::StringInterp(vec![
                                    LoweredStringPart::Literal("- ".to_string()),
                                    LoweredStringPart::Expr(LoweredExpr::Ident("x".to_string())),
                                ])),
                            },
                        )],
                    }),
                },
            )])],
        };
        let inputs: HashMap<String, Value> = [(
            "items".to_string(),
            Value::List(vec![
                Value::Str("foo".to_string()),
                Value::Str("bar".to_string()),
            ]),
        )]
        .into_iter()
        .collect();
        let result = evaluate_fn_body(&body, &inputs, &empty_siblings()).unwrap();
        assert_eq!(
            result["return"],
            Value::List(vec![
                Value::Str("- foo".to_string()),
                Value::Str("- bar".to_string()),
            ])
        );
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
}
