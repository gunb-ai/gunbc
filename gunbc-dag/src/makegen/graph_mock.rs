//! Mock specification for the makegen tool.

use crate::makegen::graph::build_makegen_graph;
use gunbc_test::MockSpec;

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "makegen",
    builder = "crate::build_makegen_graph().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "makegen",
    output = "gunbc-dag/src/makegen/generated_tests.rs",
    module = "makegen_generated_tests",
    builder = "crate::build_makegen_graph().unwrap()",
    signature = "crate::makegen_signature()",
    tool = "makegen",
    flow_tests
)]
pub fn makegen_mock_spec() -> MockSpec {
    let dag = build_makegen_graph().expect("makegen graph should build");
    crate::mock_defaults::auto_mock_spec(&dag, "makegen")
}
