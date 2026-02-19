//! Mock specification for the CI tool.

use crate::ci::graph::build_ci_graph;
use gunbc_test::MockSpec;

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "ci",
    builder = "crate::build_ci_graph().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "ci",
    output = "gunbc-dag/src/ci/generated_tests.rs",
    module = "ci_generated_tests",
    builder = "crate::build_ci_graph().unwrap()",
    signature = "crate::ci_signature()",
    flow_tests
)]
pub fn ci_mock_spec() -> MockSpec {
    let dag = build_ci_graph().expect("ci graph should build");
    crate::mock_defaults::auto_mock_spec(&dag, "ci")
}
