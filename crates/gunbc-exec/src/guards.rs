use gunbc_ir::algebra::{Predicate, Value as IrValue};

use super::Value;

/// Parse a simple guard expression string into a typed Predicate.
///
/// Supported forms:
/// - `name == value`  → Predicate::Eq(Value::String(value)) or Predicate::Eq(Value::Bool(..))
/// - `name != value`  → Predicate::NotEq(Value::String(value)) or Predicate::NotEq(Value::Bool(..))
///
/// Returns None if the expression cannot be parsed.
pub fn parse_guard(expr: &str) -> Option<Predicate> {
    if let Some(pos) = expr.find("!=") {
        let expected = expr[pos + 2..].trim();
        let value = parse_literal(expected);
        return Some(Predicate::NotEq(value));
    }

    if let Some(pos) = expr.find("==") {
        let expected = expr[pos + 2..].trim();
        let value = parse_literal(expected);
        return Some(Predicate::Eq(value));
    }

    None
}

/// Parse a literal value from a guard expression.
fn parse_literal(s: &str) -> IrValue {
    match s {
        "true" => IrValue::Bool(true),
        "false" => IrValue::Bool(false),
        _ => IrValue::String(s.to_string()),
    }
}

/// Evaluate a simple guard expression against a value (legacy string-based API).
///
/// Supported forms:
/// - `name == value`  → true if the value's string form equals `value`
/// - `name != value`  → true if the value's string form does not equal `value`
///
/// If the value is `Skipped`, the guard always fails (returns false).
///
/// NOTE: Prefer using typed Predicate guards directly. This function exists
/// for backwards compatibility with string-based guard expressions.
pub fn eval_guard(expr: &str, value: &Value) -> bool {
    if matches!(value, Value::Skipped) {
        return false;
    }

    let value_str = match value {
        Value::Str(s) => s.as_str(),
        Value::Bool(b) => if *b { "true" } else { "false" },
        Value::Secret(_) => return false,
        Value::Skipped => unreachable!(),
        Value::Unit => "()",
        Value::StrList(_) | Value::MapStrStr(_) => return false,
    };

    if let Some(pos) = expr.find("!=") {
        let expected = expr[pos + 2..].trim();
        return value_str != expected;
    }

    if let Some(pos) = expr.find("==") {
        let expected = expr[pos + 2..].trim();
        return value_str == expected;
    }

    // Unknown expression format — fail closed
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_match() {
        assert!(eval_guard("needs_create == true", &Value::Str("true".into())));
        assert!(!eval_guard("needs_create == true", &Value::Str("false".into())));
    }

    #[test]
    fn equality_match_bool() {
        assert!(eval_guard("flag == true", &Value::Bool(true)));
        assert!(!eval_guard("flag == true", &Value::Bool(false)));
    }

    #[test]
    fn inequality_match() {
        assert!(eval_guard("x != foo", &Value::Str("bar".into())));
        assert!(!eval_guard("x != foo", &Value::Str("foo".into())));
    }

    #[test]
    fn skipped_fails_guard() {
        assert!(!eval_guard("needs_create == true", &Value::Skipped));
    }

    #[test]
    fn parse_guard_equality() {
        let pred = parse_guard("x == true").unwrap();
        assert_eq!(pred, Predicate::Eq(IrValue::Bool(true)));

        let pred = parse_guard("x == hello").unwrap();
        assert_eq!(pred, Predicate::Eq(IrValue::String("hello".into())));
    }

    #[test]
    fn parse_guard_inequality() {
        let pred = parse_guard("x != false").unwrap();
        assert_eq!(pred, Predicate::NotEq(IrValue::Bool(false)));

        let pred = parse_guard("x != world").unwrap();
        assert_eq!(pred, Predicate::NotEq(IrValue::String("world".into())));
    }

    #[test]
    fn parse_guard_invalid() {
        assert!(parse_guard("invalid").is_none());
        assert!(parse_guard("x > 5").is_none());
    }
}
