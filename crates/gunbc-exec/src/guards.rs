use super::Value;

/// Evaluate a simple guard expression against a value.
///
/// Supported forms:
/// - `name == value`  → true if the value's string form equals `value`
/// - `name != value`  → true if the value's string form does not equal `value`
///
/// If the value is `Skipped`, the guard always fails (returns false).
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
}
