//! Mock specification for the testgen DAG.
//!
//! Uses hardcoded mock targets for deterministic testing.

use crate::testgen_dag::graph::build_testgen_graph_for_test;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_primitives::filename;
use gunbc_test::{extract_mock_requirements, MockSpec, NodeExample, OutputMatcher};

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

/// Mock specification for the testgen DAG graph (using test fixtures).
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

    let mut reqs = extract_mock_requirements(&dag, "testgen-dag")
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs_env should match type");

    for name in &["mock-alpha", "mock-beta"] {
        let read_node = format!("execute_read_{}", name);
        let write_node = format!("execute_{}_transport", name);
        let response_name = format!("{}_response", name);
        let path_name = format!("{}_written_path", name);
        let content_name = format!("{}_content", name);
        let mock_path = format!("{}/generated_tests.rs", name.replace('-', "_"));

        // Read transport mock
        reqs = reqs
            .transport_response(
                &read_node,
                "response",
                TransportResponse::File(FileResponse {
                    path: mock_path.clone(),
                    operation: FileOp::Read,
                    success: true,
                    content: Some(format!("<mock-{}>", name)),
                    exists: None,
                    error: None,
                }),
            )
            .unwrap_or_else(|_| panic!("{} response should match type", read_node));

        // Write transport mock
        reqs = reqs
            .transport_response(
                &write_node,
                &response_name,
                TransportResponse::File(FileResponse {
                    path: mock_path.clone(),
                    operation: FileOp::Write,
                    success: true,
                    content: Some(format!("<mock-{}>", name)),
                    exists: Some(true),
                    error: None,
                }),
            )
            .unwrap_or_else(|_| panic!("{} response should match type", write_node));

        reqs = reqs
            .boundary_str(&write_node, &path_name, &mock_path)
            .unwrap_or_else(|_| panic!("{} path should match type", write_node))
            .boundary_str(&write_node, &content_name, &format!("<mock-{}>", name))
            .unwrap_or_else(|_| panic!("{} content should match type", write_node))
            .boundary_bool(&write_node, "skip", false)
            .unwrap_or_else(|_| panic!("{} skip should match type", write_node))
            .boundary_str(&write_node, "skip_reason", "")
            .unwrap_or_else(|_| panic!("{} skip_reason should match type", write_node));
    }

    let mut spec = reqs.build_unchecked();

    for name in &["mock-alpha", "mock-beta"] {
        let mock_path = format!("{}/generated_tests.rs", name.replace('-', "_"));
        spec = spec
            .input_mock(
                format!("prepare_read_{}", name),
                "path",
                Value::Str(mock_path.clone()),
            )
            .input_mock(
                format!("prepare_write_{}", name),
                "path",
                Value::Str(mock_path),
            )
            .input_mock(
                format!("compare_{}_content", name),
                "check_mode",
                Value::Bool(false),
            )
            .input_mock(
                format!("compare_{}_content", name),
                "response",
                Value::Response(TransportResponse::File(FileResponse {
                    path: format!("{}/generated_tests.rs", name.replace('-', "_")),
                    operation: FileOp::Read,
                    success: true,
                    content: Some(format!("<mock-{}>", name)),
                    exists: None,
                    error: None,
                })),
            )
            .resource_lock(format!("fs:{}/generated_tests.rs", name.replace('-', "_")));
    }

    // Probe-observer: transport terminals need chain-safe observers
    for name in &["mock-alpha", "mock-beta"] {
        spec = spec.live_expected_output(
            format!("execute_{}_transport", name),
            "skip",
            OutputMatcher::IsBool,
        );
    }

    spec = spec
        .node_example(
            NodeExample::new("fs_env")
                .output("fs:write", OutputMatcher::Any)
                .description("Provides filesystem handle for generated test writes"),
        )
        .node_example(
            NodeExample::new("generate_mock-alpha")
                .output(
                    "content",
                    OutputMatcher::contains("mock_alpha_generated_tests"),
                )
                .description("Generates mock test code for alpha target"),
        )
        .node_example(
            NodeExample::new("generate_mock-beta")
                .output(
                    "content",
                    OutputMatcher::contains("mock_beta_generated_tests"),
                )
                .description("Generates mock test code for beta target"),
        )
        .skip_node_example("prepare_read_mock-alpha")
        .skip_node_example("prepare_write_mock-alpha")
        .skip_node_example("compare_mock-alpha_content")
        .skip_node_example("prepare_read_mock-beta")
        .skip_node_example("prepare_write_mock-beta")
        .skip_node_example("compare_mock-beta_content");

    spec
}
