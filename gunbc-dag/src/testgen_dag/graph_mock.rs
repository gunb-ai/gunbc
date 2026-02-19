//! Mock specification for the testgen DAG.

use crate::testgen_dag::graph::build_testgen_graph_for_test;
use gunbc_test::MockSpec;

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "testgen-dag",
    builder = "crate::testgen_dag::graph::build_testgen_graph_for_test().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "testgen-dag",
    output = "gunbc-dag/src/testgen_dag/generated_tests.rs",
    module = "testgen_dag_generated_tests",
    builder = "crate::testgen_dag::graph::build_testgen_graph_for_test().unwrap()",
    flow_tests
)]
pub fn testgen_dag_mock_spec() -> MockSpec {
    let dag = build_testgen_graph_for_test().expect("testgen graph should build");
    crate::mock_defaults::auto_mock_spec(&dag, "testgen-dag")
}
