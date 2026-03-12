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

use std::collections::{BTreeMap, HashMap};

use gunbc_ir::Value;

use crate::expr::{
    LoweredBinOp, LoweredLiteral, LoweredPattern, LoweredUnaryOp,
};

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EvalError {
    pub message: String,
    pub early_return: Option<HashMap<String, Value>>,
    /// Self-recursive tail-call signal: contains the new inputs for the next
    /// trampoline iteration. Only used for self-recursive calls (A→A).
    pub(crate) tail_call: Option<HashMap<String, Value>>,
}

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), early_return: None, tail_call: None }
    }

    pub fn early_return(values: HashMap<String, Value>) -> Self {
        Self { message: "__early_return__".to_string(), early_return: Some(values), tail_call: None }
    }

    pub(crate) fn tail_call(inputs: HashMap<String, Value>) -> Self {
        Self { message: "__tail_call__".to_string(), early_return: None, tail_call: Some(inputs) }
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
            .ok_or_else(|| EvalError::new(format!("no field '{field}' in map"))),
        Value::Json(json) => match json {
            serde_json::Value::Object(obj) => Ok(obj
                .get(field)
                .map(|v| Value::Json(v.clone()))
                .unwrap_or(Value::Json(serde_json::Value::Null))),
            serde_json::Value::Null => Ok(Value::Json(serde_json::Value::Null)),
            _ => Err(EvalError::new(format!(
                "cannot access field '{field}' on JSON {:?}", json
            ))),
        },
        Value::Unit | Value::Skipped => Ok(Value::Unit),
        _ => Err(EvalError::new(format!(
            "cannot access field '{field}' on {:?}", base
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
                    "list concat requires both sides to be lists: {:?}, {:?}", lhs, rhs
                ))),
            }
        }
        LoweredBinOp::Add
            if matches!(lhs, Value::Str(_) | Value::Enum { .. })
                || matches!(rhs, Value::Str(_) | Value::Enum { .. }) =>
        {
            Ok(Value::Str(format!("{}{}", value_to_string(lhs), value_to_string(rhs))))
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
            if !matches!(lhs, Value::Unit | Value::Skipped) { Ok(lhs.clone()) }
            else { Ok(rhs.clone()) }
        }
    }
}

fn int_op(lhs: &Value, rhs: &Value, f: impl Fn(i64, i64) -> i64) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(f(*a, *b))),
        (Value::Skipped, _) | (_, Value::Skipped) => Ok(Value::Skipped),
        _ => Err(EvalError::new(format!(
            "arithmetic on non-integers: {:?}, {:?}", lhs, rhs
        ))),
    }
}

fn cmp_op(
    lhs: &Value, rhs: &Value, pred: impl Fn(std::cmp::Ordering) -> bool,
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
        _ => return Err(EvalError::new(format!(
            "cannot compare {:?} with {:?}", lhs, rhs
        ))),
    };
    Ok(Value::Bool(pred(ordering)))
}

// ── Unary operations ────────────────────────────────────────────────────────

pub fn eval_unary_op(op: LoweredUnaryOp, value: &Value) -> Result<Value, EvalError> {
    if matches!(value, Value::Skipped) { return Ok(Value::Skipped); }
    match op {
        LoweredUnaryOp::Not => Ok(Value::Bool(!value_truthy(value))),
        LoweredUnaryOp::Neg => match value {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            other => Err(EvalError::new(format!("UnaryOp Neg: cannot negate {other:?}"))),
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
        Value::List(items) => items.iter().map(value_to_string).collect::<Vec<_>>().join(", "),
        Value::Map(map) => format!("map({})", map.len()),
        Value::Set(items) => format!("set({})", items.len()),
        Value::Json(json) => json.to_string(),
        Value::Request(r) => format!("{r:?}"),
        Value::Response(r) => format!("{r:?}"),
        Value::Secret(s) => format!("secret({})", s.len()),
        Value::Enum { ty, variant } => {
            if ty.is_empty() { variant.clone() } else { format!("{ty}.{variant}") }
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
        if matches!(v, Value::Skipped) { return Ok(Value::Skipped); }
    }
    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        result.push_str(part);
        if i < values.len() { result.push_str(&value_to_string(&values[i])); }
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
            if values_equal(&expected, value) { return Some(vec![]); }
            if matches!(lit, LoweredLiteral::None) {
                if let Value::Map(map) = value {
                    if let Some(Value::Str(tag)) = map.get("_variant") {
                        if tag == "None" { return Some(vec![]); }
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
                        if tag == "None" { return None; }
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
                        if let Value::Str(s) = v { Some(s.as_str()) } else { None }
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
                    } else { None }
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
    if matches!(condition, Value::Skipped) { return Value::Skipped; }
    if value_truthy(condition) { then_val.clone() }
    else if let Some(e) = else_val { e.clone() }
    else { Value::Skipped }
}

pub fn eval_record_construct(fields: &[(String, Value)]) -> Result<Value, EvalError> {
    let mut map = BTreeMap::new();
    for (name, value) in fields {
        if matches!(value, Value::Skipped) { return Ok(Value::Skipped); }
        map.insert(name.clone(), value.clone());
    }
    Ok(Value::Map(map))
}

pub fn eval_null_coalesce(value: &Value, default: &Value) -> Value {
    if matches!(value, Value::Unit | Value::Skipped) { default.clone() }
    else { value.clone() }
}

pub fn eval_variant_construct(tag: &str, fields: &[(String, Value)]) -> Result<Value, EvalError> {
    if fields.is_empty() {
        return Ok(Value::Enum { ty: String::new(), variant: tag.to_string() });
    }
    let mut map = BTreeMap::new();
    map.insert("_variant".to_string(), Value::Str(tag.to_string()));
    for (name, value) in fields {
        if matches!(value, Value::Skipped) { return Ok(Value::Skipped); }
        map.insert(name.clone(), value.clone());
    }
    Ok(Value::Map(map))
}

pub fn eval_list_construct(elements: Vec<Value>) -> Result<Value, EvalError> {
    for elem in &elements {
        if matches!(elem, Value::Skipped) { return Ok(Value::Skipped); }
    }
    Ok(Value::List(elements))
}
