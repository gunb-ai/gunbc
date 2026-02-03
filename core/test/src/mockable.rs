//! Mockable trait for operations that can provide test fixtures.
//!
//! The `Mockable` trait allows operations to provide:
//! - Default mock outputs for dry-run testing
//! - Cardinality test inputs for edge case testing
//! - Error cases for failure testing
//!
//! This enables automatic test generation based on the operation's declaration.

use gunbc_ir::{CardinalityCase, Value};
use std::collections::HashMap;

/// A trait for operations that can provide test fixtures.
///
/// Operations that implement this trait can be used with the test generator
/// to automatically produce meaningful tests based on their declared behaviors.
///
/// # Example
///
/// ```ignore
/// impl Mockable for GistOp {
///     fn mock_outputs(&self) -> HashMap<String, Value> {
///         match self {
///             GistOp::FilterFiles { .. } => hashmap! {
///                 "files" => Value::str_list(vec!["test.rs".into()])
///             },
///             // ... other variants
///         }
///     }
/// }
/// ```
pub trait Mockable {
    /// Provide default mock outputs for this operation.
    ///
    /// These outputs are used when testing the DAG in dry-run mode.
    /// The mock outputs should be realistic values that represent
    /// typical successful execution.
    fn mock_outputs(&self) -> HashMap<String, Value>;

    /// Provide cardinality test inputs for this operation.
    ///
    /// Each entry describes an input port, a cardinality case to test,
    /// and the test value to use. This enables automatic generation of
    /// edge case tests.
    ///
    /// For example, a `FilterFiles` operation might provide:
    /// - ("files", Empty, []) - test with empty input
    /// - ("files", One, ["single.rs"]) - test with one file
    /// - ("files", Many, ["a.rs", "b.rs", "c.rs"]) - test with multiple files
    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        vec![] // Default: no special cardinality tests
    }

    /// Provide error cases this operation can produce.
    ///
    /// Each entry describes inputs that should cause an error,
    /// along with the expected error message pattern.
    fn error_cases(&self) -> Vec<ErrorTestCase> {
        vec![] // Default: no documented error cases
    }

    /// Check if this operation has any cardinality test inputs.
    fn has_cardinality_tests(&self) -> bool {
        !self.cardinality_inputs().is_empty()
    }

    /// Check if this operation has any error cases.
    fn has_error_tests(&self) -> bool {
        !self.error_cases().is_empty()
    }
}

/// A cardinality test input for a specific port.
#[derive(Debug, Clone)]
pub struct CardinalityTestInput {
    /// The port name to provide input for
    pub port: String,
    /// The cardinality case being tested
    pub case: CardinalityCase,
    /// The test value to use
    pub value: Value,
    /// Expected behavior when this input is used
    pub expected: ExpectedBehavior,
}

impl CardinalityTestInput {
    /// Create a new cardinality test input that should succeed.
    pub fn succeeds(port: impl Into<String>, case: CardinalityCase, value: Value) -> Self {
        Self {
            port: port.into(),
            case,
            value,
            expected: ExpectedBehavior::Succeeds,
        }
    }

    /// Create a new cardinality test input that should fail.
    pub fn fails(
        port: impl Into<String>,
        case: CardinalityCase,
        value: Value,
        error_pattern: impl Into<String>,
    ) -> Self {
        Self {
            port: port.into(),
            case,
            value,
            expected: ExpectedBehavior::FailsWith(error_pattern.into()),
        }
    }
}

/// Expected behavior for a test case.
#[derive(Debug, Clone)]
pub enum ExpectedBehavior {
    /// The operation should succeed
    Succeeds,
    /// The operation should fail with an error containing this substring
    FailsWith(String),
}

impl ExpectedBehavior {
    /// Check if this expectation matches a result.
    pub fn matches(&self, result: &Result<(), String>) -> bool {
        match (self, result) {
            (ExpectedBehavior::Succeeds, Ok(())) => true,
            (ExpectedBehavior::FailsWith(pattern), Err(msg)) => msg.contains(pattern),
            _ => false,
        }
    }
}

/// An error test case.
#[derive(Debug, Clone)]
pub struct ErrorTestCase {
    /// A name for this test case
    pub name: String,
    /// The inputs to provide
    pub inputs: HashMap<String, Value>,
    /// The expected error message pattern
    pub expected_error: String,
}

impl ErrorTestCase {
    /// Create a new error test case.
    pub fn new(
        name: impl Into<String>,
        inputs: HashMap<String, Value>,
        expected_error: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            inputs,
            expected_error: expected_error.into(),
        }
    }
}

/// Helper macro for creating HashMap<String, Value> literals.
///
/// # Example
///
/// ```ignore
/// let outputs = mock_hashmap! {
///     "files" => Value::str_list(vec!["test.rs".into()]),
///     "count" => Value::Int(1)
/// };
/// ```
#[macro_export]
macro_rules! mock_hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value);
        )*
        map
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestOp;

    impl Mockable for TestOp {
        fn mock_outputs(&self) -> HashMap<String, Value> {
            mock_hashmap! {
                "result" => Value::Str("mock".into())
            }
        }

        fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
            vec![
                CardinalityTestInput::succeeds(
                    "input",
                    CardinalityCase::Empty,
                    Value::str_list(vec![]),
                ),
                CardinalityTestInput::succeeds(
                    "input",
                    CardinalityCase::One,
                    Value::str_list(vec!["one".into()]),
                ),
            ]
        }

        fn error_cases(&self) -> Vec<ErrorTestCase> {
            vec![ErrorTestCase::new(
                "missing_input",
                HashMap::new(),
                "missing input",
            )]
        }
    }

    #[test]
    fn test_mockable_outputs() {
        let op = TestOp;
        let outputs = op.mock_outputs();
        assert!(outputs.contains_key("result"));
    }

    #[test]
    fn test_mockable_cardinality_inputs() {
        let op = TestOp;
        let inputs = op.cardinality_inputs();
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].case, CardinalityCase::Empty);
        assert_eq!(inputs[1].case, CardinalityCase::One);
    }

    #[test]
    fn test_mockable_error_cases() {
        let op = TestOp;
        let cases = op.error_cases();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "missing_input");
    }

    #[test]
    fn test_expected_behavior_matches() {
        assert!(ExpectedBehavior::Succeeds.matches(&Ok(())));
        assert!(!ExpectedBehavior::Succeeds.matches(&Err("error".into())));
        
        assert!(ExpectedBehavior::FailsWith("missing".into())
            .matches(&Err("missing input".into())));
        assert!(!ExpectedBehavior::FailsWith("missing".into())
            .matches(&Ok(())));
    }
}
