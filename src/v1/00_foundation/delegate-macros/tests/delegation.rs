use gunbc_delegate_macros::{DelegateExecutable, DelegateMockable};
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_test::{CardinalityTestInput, ErrorTestCase, ExpectedBehavior, Mockable};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

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

#[derive(Debug, Clone)]
struct GenericTestOp<T> {
    inner: TestOp,
    _marker: PhantomData<T>,
}

impl<T> GenericTestOp<T> {
    fn new(label: &'static str) -> Self {
        Self {
            inner: TestOp { label },
            _marker: PhantomData,
        }
    }
}

impl<T: Debug> Executable for GenericTestOp<T> {
    fn execute(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        self.inner.execute(inputs)
    }
}

impl<T: Debug> Mockable for GenericTestOp<T> {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        self.inner.mock_outputs()
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        self.inner.cardinality_inputs()
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        self.inner.error_cases()
    }
}

#[derive(Debug, Clone, DelegateExecutable, DelegateMockable)]
enum WrappedOp {
    Alpha(TestOp),
    Beta(TestOp),
}

#[derive(Debug, Clone, DelegateExecutable, DelegateMockable)]
enum GenericWrappedOp<T>
where
    T: Clone + Debug,
{
    Alpha(GenericTestOp<T>),
    Beta(GenericTestOp<T>),
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

    let cardinality_inputs = op.cardinality_inputs();
    assert_eq!(cardinality_inputs.len(), 1);
    let input = &cardinality_inputs[0];
    assert_eq!(input.port, "input");
    assert_eq!(input.count, 1);
    assert_eq!(input.value.as_str(), Some("alpha"));
    assert!(matches!(&input.expected, ExpectedBehavior::Succeeds));

    let error_cases = op.error_cases();
    assert_eq!(error_cases.len(), 1);
    let error_case = &error_cases[0];
    assert_eq!(error_case.name, "invalid_input");
    assert_eq!(error_case.inputs.get("input"), Some(&Value::Unit));
    assert_eq!(error_case.expected_error, "bad input");
}

#[test]
fn delegate_derives_preserve_generics_and_where_clauses() {
    let execute_op = GenericWrappedOp::<u8>::Beta(GenericTestOp::new("generic-beta"));
    let out = execute_op
        .execute(HashMap::new())
        .expect("delegated execute");
    assert_eq!(out.get("label").and_then(Value::as_str), Some("generic-beta"));

    let mock_op = GenericWrappedOp::<u8>::Alpha(GenericTestOp::new("generic-alpha"));
    let outputs = mock_op.mock_outputs();
    assert_eq!(
        outputs.get("mock").and_then(Value::as_str),
        Some("generic-alpha")
    );

    let cardinality_inputs = mock_op.cardinality_inputs();
    assert_eq!(cardinality_inputs.len(), 1);
    let input = &cardinality_inputs[0];
    assert_eq!(input.port, "input");
    assert_eq!(input.count, 1);
    assert_eq!(input.value.as_str(), Some("generic-alpha"));
    assert!(matches!(&input.expected, ExpectedBehavior::Succeeds));

    let error_cases = mock_op.error_cases();
    assert_eq!(error_cases.len(), 1);
    let error_case = &error_cases[0];
    assert_eq!(error_case.name, "invalid_input");
    assert_eq!(error_case.inputs.get("input"), Some(&Value::Unit));
    assert_eq!(error_case.expected_error, "bad input");
}
