//! Mock specification for the deps tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! - `execute_load_manifest`: Reads the deps.toml manifest
//! - `execute_installs`: Runs the install script (world write)
//!
//! # Input Expectations
//!
//! - `manifest_path`: String (required)
//!
//! # Resource Simulations
//!
//! - Package manager lock: Ensures only one install runs at a time
//! - Sudo lease: Time-bounded privilege elevation

use crate::graph::build_deps_graph;
use crate::Platform;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_primitives::filename;
use gunbc_test::{
    extract_mock_requirements, InputConstraint, MockSpec, NodeExample, OutputMatcher,
};

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

fn mock_manifest() -> &'static str {
    r#"[[dependency]]
name = "ripgrep"
verify = "rg --version"

[dependency.install.linux]
method = "cargo"
packages = ["ripgrep"]

[dependency.install.macos]
method = "brew"
packages = ["ripgrep"]
"#
}

/// Mock specification for the deps graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
///
/// Only transport and resource mocks are required. Pure terminal outputs
/// (parse_manifest.*, generate_scripts.*, parse_execute_result.*) are
/// computed during DryRun execution, not mocked.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "deps",
    builder = "crate::graph::build_deps_graph().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "deps",
    output = "lib/tools/deps/src/generated_tests.rs",
    module = "deps_generated_tests",
    builder = "crate::graph::build_deps_graph().unwrap()",
    signature = "crate::deps_signature()",
    tool = "deps"
)]
pub fn deps_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_deps_graph().expect("deps graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "deps")
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type")
        // Resource: platform environment
        .boundary("platform_env", "platform", Platform::Linux)
        .expect("platform mock should match type")
        // Transport: load manifest
        .transport_response(
            "load_manifest/execute_load_manifest",
            "response",
            TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Read,
                success: true,
                content: Some(mock_manifest().to_string()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_load_manifest response should match type")
        // Transport: execute installs
        .transport_response(
            "execute_installs/execute_installs",
            "response",
            TransportResponse::Shell(ShellResponse::ok("Dependencies installed")),
        )
        .expect("execute_installs response should match type")
        // Boundary: parse_manifest outputs (terminal)
        .boundary_int("load_manifest/parse_manifest", "dep_count", 1)
        .expect("parse_manifest dep_count should match type")
        .boundary(
            "load_manifest/parse_manifest",
            "dep_names",
            Value::str_list(vec!["ripgrep".into()]),
        )
        .expect("parse_manifest dep_names should match type")
        .boundary_str("load_manifest/parse_manifest", "manifest_path", "deps.toml")
        .expect("parse_manifest manifest_path should match type")
        // Boundary: generate_scripts outputs (terminal)
        .boundary(
            "generate_scripts",
            "already_installed",
            Value::str_list(vec!["ripgrep".into()]),
        )
        .expect("generate_scripts already_installed should match type")
        .boundary(
            "generate_scripts",
            "needs_install",
            Value::str_list(vec!["ripgrep".into()]),
        )
        .expect("generate_scripts needs_install should match type")
        .boundary_str("generate_scripts", "platform", "linux")
        .expect("generate_scripts platform should match type")
        // Boundary: parse_execute_result outputs (terminal)
        .boundary_bool("execute_installs/parse_execute_result", "executed", true)
        .expect("parse_execute_result executed should match type")
        .boundary_bool("execute_installs/parse_execute_result", "success", true)
        .expect("parse_execute_result success should match type")
        .boundary_str("execute_installs/parse_execute_result", "script", "echo install")
        .expect("parse_execute_result script should match type")
        .boundary_str("execute_installs/parse_execute_result", "stdout", "installed\n")
        .expect("parse_execute_result stdout should match type")
        .boundary_str("execute_installs/parse_execute_result", "stderr", "")
        .expect("parse_execute_result stderr should match type")
        // Build spec (pure terminal outputs are computed; boundary mocks provided for tests)
        .build_unchecked()
        // Input expectations (via legacy API post-build)
        .expects_input("manifest_path", InputConstraint::Any)
        .input_mock(
            "load_manifest/prepare_load_manifest",
            "manifest_path",
            Value::Str("deps.toml".into()),
        )
        // Resource: package manager lock
        .resource_lock("target:manager")
        // Node I/O examples
        .node_example(
            NodeExample::new("fs_env")
                .output("file:write", OutputMatcher::Any)
                .description("Provides filesystem handle for deps operations"),
        )
        .node_example(
            NodeExample::new("platform_env")
                .output("platform", OutputMatcher::IsString)
                .description("Detects host platform as a string"),
        )
        .node_example(
            NodeExample::new("load_manifest/prepare_load_manifest")
                .input("manifest_path", Value::Str("deps.toml".into()))
                .output("request", OutputMatcher::IsRequest)
                .output(
                    "manifest_path",
                    OutputMatcher::exact(Value::Str("deps.toml".into())),
                )
                .description("Prepares file read request for deps.toml"),
        )
        .node_example(
            NodeExample::new("load_manifest/parse_manifest")
                .input(
                    "response",
                    Value::Response(
                        FileResponse {
                            path: "deps.toml".into(),
                            operation: FileOp::Read,
                            success: true,
                            content: Some(mock_manifest().to_string()),
                            exists: Some(true),
                            error: None,
                        }
                        .into(),
                    ),
                )
                .input("manifest_path", Value::Str("deps.toml".into()))
                .output("dep_count", OutputMatcher::exact(Value::Int(1)))
                .output(
                    "dep_names",
                    OutputMatcher::exact(Value::str_list(vec!["ripgrep".into()])),
                )
                .output(
                    "manifest_path",
                    OutputMatcher::exact(Value::Str("deps.toml".into())),
                )
                .output("manifest_content", OutputMatcher::contains("ripgrep"))
                .description("Parses deps.toml into dependency list and content"),
        )
        .node_example(
            NodeExample::new("generate_scripts")
                .input("manifest_content", Value::Str(mock_manifest().to_string()))
                .input("res:platform", Value::Str("linux".into()))
                .output(
                    "install_script",
                    OutputMatcher::contains("cargo install ripgrep"),
                )
                .output("needs_install", OutputMatcher::NonEmpty)
                .output("platform", OutputMatcher::exact(Value::Str("linux".into())))
                .description("Generates install script and plan for linux"),
        )
        .node_example(
            NodeExample::new("execute_installs/prepare_execute_installs")
                .input("install_script", Value::Str("echo install".into()))
                .output("request", OutputMatcher::IsRequest)
                .output(
                    "script",
                    OutputMatcher::exact(Value::Str("echo install".into())),
                )
                .description("Prepares shell request for install script"),
        )
        .node_example(
            NodeExample::new("execute_installs/parse_execute_result")
                .input(
                    "response",
                    Value::Response(ShellResponse::ok("installed\n").into()),
                )
                .input("script", Value::Str("echo install".into()))
                .output("executed", OutputMatcher::exact(Value::Bool(true)))
                .output("success", OutputMatcher::exact(Value::Bool(true)))
                .output(
                    "script",
                    OutputMatcher::exact(Value::Str("echo install".into())),
                )
                .output(
                    "stdout",
                    OutputMatcher::exact(Value::Str("installed\n".into())),
                )
                .output("stderr", OutputMatcher::exact(Value::Str("".into())))
                .description("Parses install execution result"),
        )
        // Probe-observer: terminal needs chain-safe observer
        .live_expected_output("execute_installs/parse_execute_result", "success", OutputMatcher::IsBool)
}

/// Mock spec for testing sudo elevation scenarios.
///
/// Simulates a time-bounded sudo lease (5 minutes).
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn deps_mock_spec_with_sudo() -> MockSpec {
    deps_mock_spec()
        // Sudo lease: 5 minutes before re-auth needed
        .resource_lease("sudo:elevation", 300_000)
}

/// Mock spec for testing package manager failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn deps_mock_spec_pkg_fails() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_deps_graph().expect("deps graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "deps")
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type")
        // Resource: platform environment
        .boundary("platform_env", "platform", Platform::Linux)
        .expect("platform mock should match type")
        // Transport: load manifest (succeeds)
        .transport_response(
            "load_manifest/execute_load_manifest",
            "response",
            TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Read,
                success: true,
                content: Some(mock_manifest().to_string()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_load_manifest response should match type")
        // Transport: execute installs (fails)
        .transport_response(
            "execute_installs/execute_installs",
            "response",
            TransportResponse::Shell(ShellResponse::failed(
                1,
                "Package manager locked by another process",
            )),
        )
        .expect("execute_installs response should match type")
        // Build spec
        .build_unchecked()
        .expects_input("manifest_path", InputConstraint::Any)
        .input_mock(
            "load_manifest/prepare_load_manifest",
            "manifest_path",
            Value::Str("deps.toml".into()),
        )
        .resource_lock_fails("target:manager", "Package manager locked by another process")
}
