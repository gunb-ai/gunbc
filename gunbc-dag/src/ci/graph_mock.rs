//! Mock specification for the CI tool.

use crate::ci::graph::build_ci_graph;
use crate::resources::MAKEFILE_OUTPUT_PATH;
use gunbc_test::MockSpec;
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_testgen_registry::iter_dag_specs;

fn ci_generated_tests_path() -> &'static str {
    iter_dag_specs()
        .find(|spec| spec.name == "ci")
        .map(|spec| spec.meta.output_path)
        .unwrap_or("gunbc-dag/src/ci/generated_tests.rs")
}

fn ci_path_for_node(node_id: &str) -> Option<&'static str> {
    if node_id.contains("Find_ListDirs") {
        Some("crates")
    } else if node_id.contains("makegen") {
        Some(MAKEFILE_OUTPUT_PATH)
    } else if node_id.contains("render_and_upsert")
        || node_id == "std.patterns::content_upsert"
        || node_id == "std.patterns::file_content_matches"
    {
        Some(ci_generated_tests_path())
    } else {
        None
    }
}

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
    let mut spec = crate::mock_defaults::auto_mock_spec(&dag, "ci");
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        let node = node_id.0.clone();
        let port = port_name.0.clone();
        match port.as_str() {
            "check_mode" => {
                spec = spec.input_mock(node, port, Value::Bool(false));
            }
            "path" => {
                if let Some(path) = ci_path_for_node(&node_id.0) {
                    spec = spec.input_mock(node, port, Value::Str(path.to_string()));
                }
            }
            "content" => {
                spec = spec.input_mock(node, port, Value::Str(String::new()));
            }
            "audience" | "project" | "secret_name" => {
                spec = spec.input_mock(node, port, Value::Str("mock".to_string()));
            }
            "max_depth" if node_id.0.contains("Find_ListDirs") => {
                spec = spec.input_mock(node, port, Value::Int(1));
            }
            "min_depth" if node_id.0.contains("Find_ListDirs") => {
                spec = spec.input_mock(node, port, Value::Int(1));
            }
            _ => {}
        }
    }
    spec
}
