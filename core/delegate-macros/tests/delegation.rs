use gunbc_delegate_macros::{DelegateExecutable, DelegateMockable};
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_test::{CardinalityTestInput, ErrorTestCase, Mockable};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct TestOp {
    label: &'static str,
}

impl Executable for TestOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        Ok(HashMap::from([(
            "label".to_string(),
            Value::Str(self.label.to_string()),
        )]))
    }
}

impl Mockable for TestOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        HashMap::from([("mock".to_string(), Value::Str(self.label.to_string()))])
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        vec![CardinalityTestInput::succeeds(
            "input",
            1,
            Value::Str(self.label.to_string()),
        )]
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        vec![ErrorTestCase::new(
            "invalid_input",
            HashMap::from([("input".to_string(), Value::Unit)]),
            "bad input",
        )]
    }
}

#[derive(Debug, Clone, DelegateExecutable, DelegateMockable)]
enum WrappedOp {
    Alpha(TestOp),
    Beta(TestOp),
}

#[test]
fn delegate_executable_calls_inner_variant_execute() {
    let op = WrappedOp::Beta(TestOp { label: "beta" });
    let out = op.execute(HashMap::new()).expect("delegated execute");
    assert_eq!(out.get("label").and_then(Value::as_str), Some("beta"));
}

#[test]
fn delegate_mockable_calls_inner_variant_methods() {
    let op = WrappedOp::Alpha(TestOp { label: "alpha" });
    let outputs = op.mock_outputs();
    assert_eq!(outputs.get("mock").and_then(Value::as_str), Some("alpha"));
    assert_eq!(op.cardinality_inputs().len(), 1);
    assert_eq!(op.error_cases().len(), 1);
}
