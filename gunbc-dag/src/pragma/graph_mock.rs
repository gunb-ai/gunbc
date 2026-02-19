//! Mock specification for the pragma tool.

use crate::pragma::graph::build_pragma_graph;
use gunbc_test::MockSpec;

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "pragma",
    builder = "crate::build_pragma_graph().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "pragma",
    output = "gunbc-dag/src/pragma/generated_tests.rs",
    module = "pragma_generated_tests",
    builder = "crate::build_pragma_graph().unwrap()",
    signature = "crate::pragma_signature()",
    flow_tests
)]
pub fn pragma_mock_spec() -> MockSpec {
    let dag = build_pragma_graph().expect("pragma graph should build");
    crate::mock_defaults::auto_mock_spec(&dag, "pragma")
}
