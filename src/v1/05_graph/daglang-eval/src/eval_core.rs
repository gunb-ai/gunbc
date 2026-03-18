//! Pure value utilities — shared by all evaluator implementations.
//!
//! Every function here is a pure function on Values. No Env, no eval_expr,
//! no sibling_fns, no control flow state. These are the leaf operations that
//! both the old evaluator (eval.rs) and the explicit-stack evaluator
//! (eval_stack.rs) import.
//!
//! This module is the single source of truth for:
//! - Value truthiness, equality, ordering, string conversion
//! - Literal evaluation, field access, binary/unary operations
//! - Pattern matching
//! - Record/variant/list construction
//! - EvalError type

use std::collections::BTreeMap;

use gunbc_ir::Value;

use crate::expr::{
    LoweredBinOp, LoweredExpr, LoweredLiteral, LoweredPattern, LoweredStmt, LoweredStringPart,
    LoweredUnaryOp,
};

// ── Error type ──────────────────────────────────────────────────────────────

/// Evaluation error.
///
/// Previously also carried an `early_return` control-flow signal (S67 item 5),
/// but `Return` is now a statement-level construct handled by `eval_block_s` /
/// `eval_stmts` via `Step::EarlyReturn`, so `EvalError` is purely an error type.
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

// ── Literal evaluation ──────────────────────────────────────────────────────

pub fn eval_literal(lit: &LoweredLiteral) -> Value {
    match lit {
        LoweredLiteral::Int(i) => Value::Int(*i),
        LoweredLiteral::Bool(b) => Value::Bool(*b),
        LoweredLiteral::String(s) => Value::Str(s.clone()),
        LoweredLiteral::None => Value::Unit,
    }
}

// ── Field access ────────────────────────────────────────────────────────────

pub fn field_access(base: &Value, field: &str) -> Result<Value, EvalError> {
    match base {
        Value::Map(map) => map
            .get(field)
            .cloned()
            .ok_or_else(|| {
                let keys: Vec<&String> = map.keys().collect();
                EvalError::new(format!("no field '{field}' in map (keys: {keys:?})"))
            }),
        Value::Json(json) => match json {
            serde_json::Value::Object(obj) => Ok(obj
                .get(field)
                .map(|v| Value::Json(v.clone()))
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

/// Detailed field access for DAG executor diagnostics.
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

// ── Binary operations ───────────────────────────────────────────────────────

pub fn eval_binop(lhs: &Value, op: LoweredBinOp, rhs: &Value) -> Result<Value, EvalError> {
    if !matches!(op, LoweredBinOp::NullCoalesce)
        && (matches!(lhs, Value::Skipped) || matches!(rhs, Value::Skipped))
    {
        return Ok(Value::Skipped);
    }
    match op {
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
        LoweredBinOp::Add => int_op(lhs, rhs, |a, b| a + b),
        LoweredBinOp::Sub => int_op(lhs, rhs, |a, b| a - b),
        LoweredBinOp::Mul => int_op(lhs, rhs, |a, b| a * b),
        LoweredBinOp::Div => int_op(lhs, rhs, |a, b| if b != 0 { a / b } else { 0 }),
        LoweredBinOp::Mod => int_op(lhs, rhs, |a, b| if b != 0 { a % b } else { 0 }),
        LoweredBinOp::Eq => Ok(Value::Bool(values_equal(lhs, rhs))),
        LoweredBinOp::Ne => Ok(Value::Bool(!values_equal(lhs, rhs))),
        LoweredBinOp::Lt => cmp_op(lhs, rhs, |o| o.is_lt()),
        LoweredBinOp::Gt => cmp_op(lhs, rhs, |o| o.is_gt()),
        LoweredBinOp::Le => cmp_op(lhs, rhs, |o| o.is_le()),
        LoweredBinOp::Ge => cmp_op(lhs, rhs, |o| o.is_ge()),
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

fn cmp_op(
    lhs: &Value,
    rhs: &Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, EvalError> {
    if matches!(lhs, Value::Skipped) || matches!(rhs, Value::Skipped) {
        return Ok(Value::Skipped);
    }
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

// ── Unary operations ────────────────────────────────────────────────────────

pub fn eval_unary_op(op: LoweredUnaryOp, value: &Value) -> Result<Value, EvalError> {
    if matches!(value, Value::Skipped) {
        return Ok(Value::Skipped);
    }
    match op {
        LoweredUnaryOp::Not => Ok(Value::Bool(!value_truthy(value))),
        LoweredUnaryOp::Neg => match value {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            other => Err(EvalError::new(format!(
                "UnaryOp Neg: cannot negate {other:?}"
            ))),
        },
    }
}

// ── Value predicates ────────────────────────────────────────────────────────

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

pub fn values_equal(lhs: &Value, rhs: &Value) -> bool {
    match (lhs, rhs) {
        (Value::Unit, Value::Unit) | (Value::Skipped, Value::Skipped) => true,
        (Value::Unit, Value::Skipped) | (Value::Skipped, Value::Unit) => true,
        (Value::Enum { variant, .. }, Value::Str(s))
        | (Value::Str(s), Value::Enum { variant, .. }) => variant == s,
        _ => lhs == rhs,
    }
}

// ── String conversion ───────────────────────────────────────────────────────

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::Str(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Unit | Value::Skipped => String::new(),
        Value::List(items) => items
            .iter()
            .map(value_to_string)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Map(map) => format!("map({})", map.len()),
        Value::Set(items) => format!("set({})", items.len()),
        Value::Json(json) => json.to_string(),
        Value::Request(r) => format!("{r:?}"),
        Value::Response(r) => format!("{r:?}"),
        Value::Secret(s) => format!("secret({})", s.len()),
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

pub fn sort_key(value: &Value) -> String {
    match value {
        Value::Str(s) => format!("s:{s}"),
        Value::Int(i) => format!("i:{i:020}"),
        Value::Bool(b) => format!("b:{b}"),
        Value::List(items) => format!("l:{}", items.len()),
        Value::Map(map) => format!("m:{}", map.len()),
        Value::Set(items) => format!("set:{}", items.len()),
        Value::Json(json) => format!("j:{json}"),
        Value::Request(r) => format!("req:{r:?}"),
        Value::Response(r) => format!("resp:{r:?}"),
        Value::Secret(s) => format!("secret:{}", s.len()),
        Value::Enum { ty, variant } => format!("enum:{ty}:{variant}"),
        Value::Float(f) => format!("f:{f}"),
        Value::Bytes(b) => format!("bytes:{}", b.len()),
        Value::Skipped => "skipped".to_string(),
        Value::Unit => "unit".to_string(),
    }
}

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

// ── Pattern matching ────────────────────────────────────────────────────────

pub fn match_pattern(pattern: &LoweredPattern, value: &Value) -> Option<Vec<(String, Value)>> {
    match pattern {
        LoweredPattern::Wildcard => Some(vec![]),
        LoweredPattern::Ident(name) => Some(vec![(name.clone(), value.clone())]),
        LoweredPattern::Literal(lit) => {
            let expected = eval_literal(lit);
            if values_equal(&expected, value) {
                return Some(vec![]);
            }
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
            if variant_name == "Some" && fields.len() == 1 {
                if let Value::Map(map) = value {
                    if let Some(Value::Str(tag)) = map.get("_variant") {
                        if tag == "Some" {
                            let inner = map.get("value").cloned().unwrap_or(Value::Unit);
                            return match_pattern(&fields[0].1, &inner);
                        }
                        if tag == "None" {
                            return None;
                        }
                    }
                }
                if !matches!(value, Value::Unit | Value::Skipped) {
                    return match_pattern(&fields[0].1, value);
                }
                return None;
            }
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
                            match match_pattern(sub_pattern, &field_value) {
                                Some(sub) => bindings.extend(sub),
                                None => return None,
                            }
                        }
                        Some(bindings)
                    } else {
                        None
                    }
                }
                Value::Enum { variant, .. } if variant == variant_name => Some(vec![]),
                Value::Str(s) if s == variant_name => Some(vec![]),
                _ => None,
            }
        }
    }
}

// ── Construction helpers (used by interp crate) ─────────────────────────────

pub fn eval_conditional(condition: &Value, then_val: &Value, else_val: Option<&Value>) -> Value {
    if matches!(condition, Value::Skipped) {
        return Value::Skipped;
    }
    if value_truthy(condition) {
        then_val.clone()
    } else if let Some(e) = else_val {
        e.clone()
    } else {
        Value::Skipped
    }
}

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

pub fn eval_null_coalesce(value: &Value, default: &Value) -> Value {
    if matches!(value, Value::Unit | Value::Skipped) {
        default.clone()
    } else {
        value.clone()
    }
}

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

pub fn eval_list_construct(elements: Vec<Value>) -> Result<Value, EvalError> {
    for elem in &elements {
        if matches!(elem, Value::Skipped) {
            return Ok(Value::Skipped);
        }
    }
    Ok(Value::List(elements))
}

// ── IR tree walkers ─────────────────────────────────────────────────────────

/// Single authoritative implementation: does this expression tree contain any
/// `LoweredExpr::Call` node? Used by both the ANF normalizer (to decide what
/// to hoist) and the evaluator (to classify expressions).
pub fn expr_contains_call(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { .. } => true,
        LoweredExpr::Literal(_) | LoweredExpr::Ident(_) => false,
        LoweredExpr::FieldAccess { expr, .. } | LoweredExpr::UnaryOp { expr, .. } => {
            expr_contains_call(expr)
        }
        LoweredExpr::BinOp { left, right, .. } => {
            expr_contains_call(left) || expr_contains_call(right)
        }
        LoweredExpr::StringInterp(parts) => parts.iter().any(|p| match p {
            LoweredStringPart::Expr(e) => expr_contains_call(e),
            _ => false,
        }),
        LoweredExpr::IfElse { cond, then_, else_ } => {
            expr_contains_call(cond)
                || expr_contains_call(then_)
                || else_.as_ref().is_some_and(|e| expr_contains_call(e))
        }
        LoweredExpr::Match { expr, arms } => {
            expr_contains_call(expr)
                || arms.iter().any(|a| {
                    expr_contains_call(&a.body) || a.guard.as_ref().is_some_and(expr_contains_call)
                })
        }
        LoweredExpr::Lambda { body, .. } => expr_contains_call(body),
        LoweredExpr::List(items) => items.iter().any(expr_contains_call),
        LoweredExpr::Block(stmts) => stmts.iter().any(stmt_contains_call),
        LoweredExpr::Record { fields, .. } | LoweredExpr::VariantConstruct { fields, .. } => {
            fields.iter().any(|(_, e)| expr_contains_call(e))
        }
        LoweredExpr::For { iterable, body, .. } => {
            expr_contains_call(iterable) || expr_contains_call(body)
        }
    }
}

pub fn stmt_contains_call(stmt: &LoweredStmt) -> bool {
    match stmt {
        LoweredStmt::Let(_, e) | LoweredStmt::Expr(e) => expr_contains_call(e),
        LoweredStmt::Return(fields) => fields.iter().any(|(_, e)| expr_contains_call(e)),
    }
}

// ── Built-in call evaluation (pre-evaluated args) ────────────────────────

fn require_builtin_arg(
    param: &str,
    index: usize,
    args: &[(Option<String>, Value)],
) -> Result<Value, EvalError> {
    for (name, val) in args {
        if name.as_deref() == Some(param) {
            return Ok(val.clone());
        }
    }
    if let Some((_, val)) = args.get(index) {
        return Ok(val.clone());
    }
    Err(EvalError::new(format!("missing argument '{param}'")))
}

/// Evaluate a built-in function call with pre-evaluated arguments.
/// Returns `None` if the name is not a recognized built-in.
/// `scan_while` is NOT handled here (needs lambda evaluation).
pub fn eval_builtin_call(
    name: &str,
    args: &[(Option<String>, Value)],
) -> Option<Result<Value, EvalError>> {
    Some(match name {
        // concat(a, b, ...) — sequence concatenation for strings and lists.
        // Replaces the overloaded `+` operator for non-arithmetic operations.
        "concat" => {
            if args.len() < 2 {
                Err(EvalError::new("concat requires at least 2 arguments"))
            } else {
                let mut result = args[0].1.clone();
                for (_, arg) in &args[1..] {
                    result = match (result, arg) {
                        (Value::Str(a), Value::Str(b)) => Value::Str(format!("{a}{b}")),
                        (Value::List(mut a), Value::List(b)) => {
                            a.extend(b.iter().cloned());
                            Value::List(a)
                        }
                        (a, b) => {
                            // Fallback: convert to strings and concatenate
                            Value::Str(format!("{}{}", value_to_string(&a), value_to_string(b)))
                        }
                    };
                }
                Ok(result)
            }
        }
        "with" => {
            if args.len() >= 2 {
                match (&args[0].1, &args[1].1) {
                    (Value::Map(base), Value::Map(updates)) => {
                        let mut result = base.clone();
                        for (k, v) in updates {
                            result.insert(k.clone(), v.clone());
                        }
                        Ok(Value::Map(result))
                    }
                    _ => Err(EvalError::new("'with' requires record values")),
                }
            } else {
                Err(EvalError::new("'with' requires base and updates"))
            }
        }
        "map_get" => {
            if args.len() >= 2 {
                match (&args[0].1, &args[1].1) {
                    (Value::Map(map), key) => {
                        let key = value_to_string(key);
                        let mut result = BTreeMap::new();
                        if let Some(value) = map.get(&key) {
                            result.insert("_variant".to_string(), Value::Str("Some".to_string()));
                            result.insert("value".to_string(), value.clone());
                        } else {
                            result.insert("_variant".to_string(), Value::Str("None".to_string()));
                        }
                        Ok(Value::Map(result))
                    }
                    _ => Err(EvalError::new("'map_get' requires a map and key")),
                }
            } else {
                Err(EvalError::new("'map_get' requires map and key"))
            }
        }
        "map_values" => match args.first() {
            Some((_, Value::Map(map))) => Ok(Value::List(map.values().cloned().collect())),
            _ => Err(EvalError::new("'map_values' requires a map")),
        },
        "map_keys" => match args.first() {
            Some((_, Value::Map(map))) => Ok(Value::List(
                map.keys().cloned().map(Value::Str).collect(),
            )),
            _ => Err(EvalError::new("'map_keys' requires a map")),
        },
        "empty_map" => Ok(Value::Map(BTreeMap::new())),
        "map_insert" => {
            if args.len() >= 3 {
                match &args[0].1 {
                    Value::Map(map) => {
                        let key = value_to_string(&args[1].1);
                        let value = args[2].1.clone();
                        let mut new_map = map.clone();
                        new_map.insert(key, value);
                        Ok(Value::Map(new_map))
                    }
                    _ => Err(EvalError::new("'map_insert' requires a map, key, value")),
                }
            } else {
                Err(EvalError::new("'map_insert' requires map, key, value"))
            }
        }
        "map_merge" => {
            if args.len() >= 2 {
                match (&args[0].1, &args[1].1) {
                    (Value::Map(base), Value::Map(overlay)) => {
                        let mut merged = base.clone();
                        merged.extend(overlay.iter().map(|(k, v)| (k.clone(), v.clone())));
                        Ok(Value::Map(merged))
                    }
                    _ => Err(EvalError::new("'map_merge' requires two maps")),
                }
            } else {
                Err(EvalError::new("'map_merge' requires two maps"))
            }
        }
        "map_contains_key" => {
            if args.len() >= 2 {
                match &args[0].1 {
                    Value::Map(map) => {
                        let key = value_to_string(&args[1].1);
                        Ok(Value::Bool(map.contains_key(&key)))
                    }
                    _ => Err(EvalError::new("'map_contains_key' requires a map and key")),
                }
            } else {
                Err(EvalError::new("'map_contains_key' requires map and key"))
            }
        }
        "Some" => match args.first() {
            Some((_, value)) => {
                let mut result = BTreeMap::new();
                result.insert("_variant".to_string(), Value::Str("Some".to_string()));
                result.insert("value".to_string(), value.clone());
                Ok(Value::Map(result))
            }
            None => Err(EvalError::new("'Some' requires a value")),
        },
        "code_point" => {
            let val = require_builtin_arg("c", 0, args);
            match val {
                Err(e) => Err(e),
                Ok(val) => match &val {
                    Value::Str(s) => match s.chars().next() {
                        Some(c) => Ok(Value::Int(c as i64)),
                        None => Err(EvalError::new("code_point: empty string")),
                    },
                    Value::Int(n) => Ok(Value::Int(*n)),
                    _ => Err(EvalError::new(format!(
                        "code_point: expected Char, got {:?}",
                        val
                    ))),
                },
            }
        }
        "from_code_point" => match require_builtin_arg("cp", 0, args) {
            Err(e) => Err(e),
            Ok(Value::Int(cp)) => match char::from_u32(cp as u32) {
                Some(c) => Ok(Value::Str(c.to_string())),
                None => Err(EvalError::new(format!(
                    "from_code_point: invalid code point {cp}"
                ))),
            },
            Ok(_) => Err(EvalError::new("from_code_point: expected Int")),
        },
        "to_string" => match require_builtin_arg("value", 0, args) {
            Err(e) => Err(e),
            Ok(val) => Ok(Value::Str(value_to_string(&val))),
        },
        "char_at" => {
            match (
                require_builtin_arg("s", 0, args),
                require_builtin_arg("pos", 1, args),
            ) {
                (Err(e), _) | (_, Err(e)) => Err(e),
                (Ok(Value::Str(s)), Ok(Value::Int(i))) => match s.chars().nth(i as usize) {
                    Some(c) => Ok(Value::Str(c.to_string())),
                    None => Ok(Value::Unit),
                },
                _ => Err(EvalError::new("char_at requires (String, Int)")),
            }
        }
        "substring" => {
            match (
                require_builtin_arg("s", 0, args),
                require_builtin_arg("start", 1, args),
                require_builtin_arg("end", 2, args),
            ) {
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Err(e),
                (Ok(Value::Str(s)), Ok(Value::Int(start)), Ok(Value::Int(end))) => {
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as i64;
                    let s_idx = (start.max(0) as usize).min(len as usize);
                    let e_idx = (end.max(0) as usize).min(len as usize);
                    Ok(Value::Str(chars[s_idx..e_idx].iter().collect()))
                }
                _ => Err(EvalError::new("substring requires (String, Int, Int)")),
            }
        }
        "string_length" => match require_builtin_arg("s", 0, args) {
            Err(e) => Err(e),
            Ok(Value::Str(s)) => Ok(Value::Int(s.chars().count() as i64)),
            _ => Err(EvalError::new("string_length requires a String")),
        },
        "parse_int" => match require_builtin_arg("s", 0, args) {
            Err(e) => Err(e),
            Ok(Value::Str(s)) => s
                .trim()
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|e| EvalError::new(format!("parse_int: cannot parse '{s}': {e}"))),
            _ => Err(EvalError::new("parse_int requires a String")),
        },
        "scan_string_end" => {
            match (
                require_builtin_arg("s", 0, args),
                require_builtin_arg("start", 1, args),
            ) {
                (Err(e), _) | (_, Err(e)) => Err(e),
                (Ok(Value::Str(s)), Ok(Value::Int(start))) => {
                    let chars: Vec<char> = s.chars().collect();
                    let mut pos = start.max(0) as usize;
                    while pos < chars.len() {
                        if chars[pos] == '\\' {
                            pos += 2;
                        } else if chars[pos] == '"' {
                            return Some(Ok(Value::Int((pos + 1) as i64)));
                        } else {
                            pos += 1;
                        }
                    }
                    Ok(Value::Int(chars.len() as i64))
                }
                _ => Err(EvalError::new("scan_string_end requires (String, Int)")),
            }
        }
        "scan_to_eol" => {
            match (
                require_builtin_arg("s", 0, args),
                require_builtin_arg("start", 1, args),
            ) {
                (Err(e), _) | (_, Err(e)) => Err(e),
                (Ok(Value::Str(s)), Ok(Value::Int(start))) => {
                    let chars: Vec<char> = s.chars().collect();
                    let start = start.max(0) as usize;
                    for (i, &ch) in chars.iter().enumerate().skip(start) {
                        if ch == '\n' {
                            return Some(Ok(Value::Int(i as i64)));
                        }
                    }
                    Ok(Value::Int(chars.len() as i64))
                }
                _ => Err(EvalError::new("scan_to_eol requires (String, Int)")),
            }
        }
        "skip_horizontal_ws" => {
            match (
                require_builtin_arg("s", 0, args),
                require_builtin_arg("start", 1, args),
            ) {
                (Err(e), _) | (_, Err(e)) => Err(e),
                (Ok(Value::Str(s)), Ok(Value::Int(start))) => {
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
            match (
                require_builtin_arg("map", 0, args),
                require_builtin_arg("key", 1, args),
            ) {
                (Err(e), _) | (_, Err(e)) => Err(e),
                (Ok(Value::Map(map)), Ok(Value::Str(key))) => {
                    let mut result = BTreeMap::new();
                    if let Some(value) = map.get(&key) {
                        result.insert("_variant".to_string(), Value::Str("Some".to_string()));
                        result.insert("value".to_string(), value.clone());
                    } else {
                        result.insert("_variant".to_string(), Value::Str("None".to_string()));
                    }
                    Ok(Value::Map(result))
                }
                _ => Err(EvalError::new("lookup requires (Map, String)")),
            }
        }
        "reverse" => match require_builtin_arg("list", 0, args) {
            Err(e) => Err(e),
            Ok(Value::List(list)) => {
                let mut reversed = list.clone();
                reversed.reverse();
                Ok(Value::List(reversed))
            }
            _ => Err(EvalError::new("reverse requires a List")),
        },
        // list_push(list, item) — O(1) amortized append to end of list.
        "list_push" => {
            if args.len() >= 2 {
                match &args[0].1 {
                    Value::List(list) => {
                        let mut new_list = list.clone();
                        new_list.push(args[1].1.clone());
                        Ok(Value::List(new_list))
                    }
                    _ => Err(EvalError::new("list_push requires (List, item)")),
                }
            } else {
                Err(EvalError::new("list_push requires list and item"))
            }
        }
        _ if name.chars().next().unwrap_or('a').is_uppercase() => {
            let mut map = BTreeMap::new();
            map.insert("_variant".to_string(), Value::Str(name.to_string()));
            for (idx, (arg_name, arg_val)) in args.iter().enumerate() {
                let field_name = arg_name.clone().unwrap_or_else(|| format!("_{idx}"));
                map.insert(field_name, arg_val.clone());
            }
            Ok(Value::Map(map))
        }
        _ if name.contains('.') => Ok(Value::Unit),
        _ => return None,
    })
}
