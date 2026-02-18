//! Mock specification for the bootstrap tool.

use crate::bootstrap::graph::build_bootstrap_graph;
use gunbc_test::MockSpec;

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "bootstrap",
    builder = "crate::build_bootstrap_graph().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "bootstrap",
    output = "gunbc-dag/src/bootstrap/generated_tests.rs",
    module = "bootstrap_generated_tests",
    builder = "crate::build_bootstrap_graph().unwrap()",
    signature = "crate::bootstrap_signature()",
    tool = "bootstrap",
    flow_tests
)]
pub fn bootstrap_mock_spec() -> MockSpec {
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");
    crate::mock_defaults::auto_mock_spec(&dag, "bootstrap")
}

#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn bootstrap_mock_spec_makefile_only() -> MockSpec {
    bootstrap_mock_spec()
}

#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn bootstrap_mock_spec_makefile_fails() -> MockSpec {
    bootstrap_mock_spec()
}

#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn bootstrap_mock_spec_all_fail() -> MockSpec {
    bootstrap_mock_spec()
}
