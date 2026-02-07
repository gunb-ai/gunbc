//! Testgen targets for tool crates that should be tested in isolation.

use gunbc_test::MockSpec;
use gunbc_testgen_registry::{inventory, TestgenTarget, TestgenTargetDef};

fn clippy_testgen_mock_spec() -> MockSpec {
    gunbc_clippy::graph_mock::clippy_mock_spec()
}

fn clippy_generate(config: &TestgenTargetDef) -> String {
    let dag = gunbc_clippy::build_clippy_graph_lint_all();
    let spec = clippy_testgen_mock_spec();
    gunbc_testgen_registry::generate_target(config, dag, spec)
}

inventory::submit! {
    TestgenTarget {
        origin_crate: "gunbc-clippy",
        name: "clippy",
        output_path: "lib/tools/clippy/src/generated_tests.rs",
        module_name: "clippy_generated_tests",
        dag_builder_call: "gunbc_clippy::build_clippy_graph_lint_all()",
        mock_spec_path: "gunbc_clippy::graph_mock::clippy_mock_spec()",
        signature_path: None,
        boundary_tests: true,
        chain_tests: true,
        flow_tests: false,
        window_max_nodes: None,
        test_class: None,
        fermi_cost: None,
        requires: None,
        secrets: None,
        generate: clippy_generate,
    }
}
