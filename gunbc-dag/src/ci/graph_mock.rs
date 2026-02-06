//! Mock specification for the CI tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! The `report` node is the boundary (world write):
//! - `overall_success`: Whether CI passed
//! - `report`: Human-readable CI report
//!
//! # Transport Mocks
//!
//! Multiple transport nodes for CI stages:
//! - `execute_deps_exists`: Check deps.toml exists
//! - `execute_codegen_exists`: Check codegen output exists
//! - `execute_codegen`: Run codegen if needed
//! - `execute_stamp_write`: Write codegen stamp file
//! - `execute_testgen`: Run testgen after codegen
//! - `execute_build`: Run cargo build
//! - `execute_test`: Run cargo test
//! - `execute_guardrail_check`: Check disallowed-methods allowlist
//! - `execute_verify_check`: Run make verify checks (makegen, bootstrap, testgen, pragma)
//!
//! # CLI Tool Mocks
//!
//! - `clippy_lint`: Clippy linting (uses ToolHandle from env)
//!
//! # Resource Simulations
//!
//! - Build lock: Only one cargo build at a time
//! - Test parallelism: Cargo test uses multiple threads

use crate::ci::graph::build_ci_graph;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_test::{extract_mock_requirements, MockSpec, NodeExample, OutputMatcher};

/// Mock specification for the CI graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "ci",
    output = "gunbc-dag/src/ci/generated_tests.rs",
    module = "ci_generated_tests",
    builder = "crate::build_ci_graph().unwrap()",
    signature = "crate::ci_signature()",
    flow_tests
)]
pub fn ci_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_ci_graph().expect("ci graph should build");

    // Extract typed requirements and fill transport mocks
    // All mocks (including clippy_lint) are now handled by with_ci_typed_mocks
    with_ci_typed_mocks(extract_mock_requirements(&dag, "ci"))
        .build_unchecked()
        // Resources
        .resource_lock("cargo:build")
        .resource_lock("cargo:test")
        .resource_lock("cargo:clippy")
        // Expected outputs: verified after DryRun execution
        .expected_output("report", "overall_success", Value::Bool(true))
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("report")
                .input("build_success", Value::Bool(true))
                .input("test_success", Value::Bool(true))
                .input("lint_success", Value::Bool(true))
                .input("testgen_success", Value::Bool(true))
                .input("guardrail_success", Value::Bool(true))
                .input("verify_success", Value::Bool(true))
                .output("overall_success", OutputMatcher::exact(Value::Bool(true)))
                .output("report", OutputMatcher::contains("SUCCESS"))
                .description("All stages pass → overall success"),
        )
        .node_example(
            NodeExample::new("report")
                .input("build_success", Value::Bool(false))
                .input(
                    "build_stderr",
                    Value::Str("error: compilation failed".into()),
                )
                .input("test_success", Value::Bool(true))
                .input("test_stdout", Value::Str(String::new()))
                .input("test_stderr", Value::Str(String::new()))
                .input("lint_success", Value::Bool(true))
                .input("lint_stdout", Value::Str(String::new()))
                .input("lint_stderr", Value::Str(String::new()))
                .input("testgen_success", Value::Bool(true))
                .input("guardrail_success", Value::Bool(true))
                .input("verify_success", Value::Bool(true))
                .output("overall_success", OutputMatcher::exact(Value::Bool(false)))
                .output("report", OutputMatcher::contains("FAILURE"))
                .description("Build failure → overall failure"),
        )
        // Node I/O examples: verify pure node behavior
        //
        // Parse nodes now test both real transport responses AND skip propagation.
        .node_example(
            NodeExample::new("parse_deps_exists")
                .input(
                    "response",
                    Value::Response(
                        FileResponse {
                            path: "deps.toml".into(),
                            operation: FileOp::Exists,
                            success: true,
                            content: None,
                            exists: Some(true),
                            error: None,
                        }
                        .into(),
                    ),
                )
                .output("deps_exists", OutputMatcher::exact(Value::Bool(true)))
                .output("deps_checked", OutputMatcher::exact(Value::Bool(true)))
                .output("deps_installed", OutputMatcher::exact(Value::Int(0)))
                .output("message", OutputMatcher::contains("deps.toml found"))
                .description("File exists: deps.toml found → deps_exists true"),
        )
        .node_example(
            NodeExample::new("parse_deps_exists")
                .input("response", Value::Skipped)
                .output("deps_checked", OutputMatcher::Any)
                .description("Handles skipped transport response gracefully"),
        )
        .node_example(
            NodeExample::new("parse_codegen_exists")
                .input(
                    "response",
                    Value::Response(ShellResponse::ok("").into()),
                )
                .output("codegen_needed", OutputMatcher::exact(Value::Bool(false)))
                .description("Shell exists check success → codegen not needed"),
        )
        .node_example(
            NodeExample::new("parse_codegen_exists")
                .input("response", Value::Skipped)
                .output("codegen_needed", OutputMatcher::Any)
                .description("Handles skipped transport response gracefully"),
        )
        .node_example(
            NodeExample::new("parse_codegen_result")
                .input(
                    "response",
                    Value::Response(ShellResponse::ok("Generated 3 files").into()),
                )
                .input("skip", Value::Bool(false))
                .output("prep_success", OutputMatcher::exact(Value::Bool(true)))
                .output("codegen_ran", OutputMatcher::exact(Value::Bool(true)))
                .output("prep_message", OutputMatcher::contains("successfully"))
                .description("Codegen shell success → prep_success true, codegen_ran true"),
        )
        .node_example(
            NodeExample::new("parse_codegen_result")
                .input("skip", Value::Bool(true))
                .output("prep_success", OutputMatcher::exact(Value::Bool(true)))
                .output("codegen_ran", OutputMatcher::exact(Value::Bool(false)))
                .description("Skip path: codegen exists → prep_success, not ran"),
        )
        .node_example(
            NodeExample::new("prepare_stamp_write")
                .input("prep_success", Value::Bool(true))
                .output("request", OutputMatcher::non_empty())
                .output("skip", OutputMatcher::exact(Value::Bool(false)))
                .description("Prep success → stamp write request"),
        )
        .node_example(
            NodeExample::new("parse_testgen")
                .input(
                    "response",
                    Value::Response(ShellResponse::ok("Generated tests").into()),
                )
                .input("skip", Value::Bool(false))
                .output("testgen_success", OutputMatcher::exact(Value::Bool(true)))
                .description("Testgen shell success → testgen_success true"),
        )
        .node_example(
            NodeExample::new("parse_testgen")
                .input("skip", Value::Bool(true))
                .input(
                    "skip_reason",
                    Value::Str("Skipped due to prep failure".into()),
                )
                .output("testgen_success", OutputMatcher::exact(Value::Bool(false)))
                .description("Skip path: testgen skipped → success false"),
        )
        .node_example(
            NodeExample::new("parse_build")
                .input(
                    "response",
                    Value::Response(
                        ShellResponse::ok("Compiling gunbc v0.1.0\n    Finished dev").into(),
                    ),
                )
                .input("skip", Value::Bool(false))
                .output("build_success", OutputMatcher::exact(Value::Bool(true)))
                .output("build_skipped", OutputMatcher::exact(Value::Bool(false)))
                .output("build_stdout", OutputMatcher::contains("Compiling"))
                .description("Build shell success → build_success true"),
        )
        .node_example(
            NodeExample::new("parse_build")
                .input("skip", Value::Bool(true))
                .input(
                    "skip_reason",
                    Value::Str("Skipped due to prep failure".into()),
                )
                .output("build_success", OutputMatcher::exact(Value::Bool(false)))
                .output("build_skipped", OutputMatcher::exact(Value::Bool(true)))
                .description("Skip path: build skipped → success false, skipped true"),
        )
        .node_example(
            NodeExample::new("parse_test")
                .input(
                    "response",
                    Value::Response(
                        ShellResponse::ok("running 42 tests\ntest result: ok. 42 passed").into(),
                    ),
                )
                .input("skip", Value::Bool(false))
                .output("test_success", OutputMatcher::exact(Value::Bool(true)))
                .output("test_skipped", OutputMatcher::exact(Value::Bool(false)))
                .output("test_stdout", OutputMatcher::contains("42 tests"))
                .description("Test shell success → test_success true"),
        )
        .node_example(
            NodeExample::new("parse_test")
                .input("skip", Value::Bool(true))
                .input(
                    "skip_reason",
                    Value::Str("Skipped due to build failure".into()),
                )
                .output("test_success", OutputMatcher::exact(Value::Bool(false)))
                .output("test_skipped", OutputMatcher::exact(Value::Bool(true)))
                .description("Skip path: test skipped → success false, skipped true"),
        )
        .node_example(
            NodeExample::new("prepare_codegen_exists")
                .output("request", OutputMatcher::non_empty())
                .description("Prepares file-exists check for codegen dir"),
        )
        .node_example(
            NodeExample::new("prepare_codegen_command")
                .input("codegen_needed", Value::Bool(false))
                .output("skip", OutputMatcher::IsBool)
                .description("Codegen command prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("prepare_testgen")
                .input("prep_success", Value::Bool(true))
                .output("skip", OutputMatcher::IsBool)
                .description("Testgen prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("prepare_build")
                .input("prep_success", Value::Bool(true))
                .input("testgen_success", Value::Bool(true))
                .output("skip", OutputMatcher::IsBool)
                .description("Build prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("prepare_test")
                .input("build_success", Value::Bool(true))
                .output("skip", OutputMatcher::IsBool)
                .description("Test prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("prepare_guardrail_check")
                .input("testgen_success", Value::Bool(true))
                .output("skip", OutputMatcher::IsBool)
                .description("Guardrail prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("prepare_clippy_lint")
                .input("build_success", Value::Bool(true))
                .output("skip", OutputMatcher::exact(Value::Bool(false)))
                .description("Build success → clippy not skipped"),
        )
        .node_example(
            NodeExample::new("prepare_clippy_lint")
                .input("build_success", Value::Bool(false))
                .output("skip", OutputMatcher::exact(Value::Bool(true)))
                .description("Build failure → clippy skipped"),
        )
        .node_example(
            NodeExample::new("parse_clippy_lint")
                .input("skip", Value::Bool(false))
                .input("success", Value::Bool(true))
                .input("stdout", Value::Str("Checking gunbc".into()))
                .input("stderr", Value::Str(String::new()))
                .output("lint_success", OutputMatcher::IsBool)
                .output("lint_skipped", OutputMatcher::IsBool)
                .description("Clippy result parse produces success/skipped flags"),
        )
        .node_example(
            NodeExample::new("parse_guardrail_check")
                .input("skip", Value::Bool(false))
                .input("response", Value::Response(ShellResponse::ok("OK").into()))
                .output("guardrail_success", OutputMatcher::exact(Value::Bool(true)))
                .description("Guardrail check success → guardrail_success true"),
        )
        .node_example(
            NodeExample::new("prepare_verify_check")
                .input("prep_success", Value::Bool(true))
                .output("skip", OutputMatcher::IsBool)
                .description("Verify prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("parse_verify_check")
                .input("skip", Value::Bool(false))
                .input("response", Value::Response(ShellResponse::ok("All checks passed").into()))
                .output("verify_success", OutputMatcher::exact(Value::Bool(true)))
                .description("Verify check success → verify_success true"),
        )
        // Primitive nodes — tested in their own crates
        .skip_node_example("prepare_deps_exists")
}

/// Mock spec for testing test failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
///
/// Uses explicit mocks to model execute_test returning a failed response.
pub fn ci_mock_spec_test_fails() -> MockSpec {
    let dag = build_ci_graph().expect("ci graph should build");

    extract_mock_requirements(&dag, "ci")
        // Resource: runner_env provides tool handles
        .boundary("runner_env", "tool:clippy", ToolHandle::mock(&CLIPPY))
        .expect("runner_env tool:clippy should match type")
        // Transport: execute_deps_exists (success)
        .transport_response(
            "execute_deps_exists",
            "response",
            TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_deps_exists response should match type")
        // Transport: execute_codegen_exists (success)
        .transport_response(
            "execute_codegen_exists",
            "response",
            TransportResponse::Shell(ShellResponse::ok("")),
        )
        .expect("execute_codegen_exists response should match type")
        // Transport: execute_codegen (skipped)
        .boundary("execute_codegen", "response", Value::Skipped)
        .expect("execute_codegen response should match type")
        .boundary_bool("execute_codegen", "skip", true)
        .expect("execute_codegen skip should match type")
        // Transport: execute_stamp_write (succeeds)
        .transport_response(
            "execute_stamp_write",
            "response",
            TransportResponse::File(FileResponse {
                path: "target/.codegen-stamp".into(),
                operation: FileOp::Write,
                success: true,
                content: None,
                exists: None,
                error: None,
            }),
        )
        .expect("execute_stamp_write response should match type")
        .boundary_bool("execute_stamp_write", "skip", false)
        .expect("execute_stamp_write skip should match type")
        // Transport: execute_testgen (success)
        .transport_response(
            "execute_testgen",
            "response",
            TransportResponse::Shell(ShellResponse::ok("Generated tests")),
        )
        .expect("execute_testgen response should match type")
        .boundary_bool("execute_testgen", "skip", false)
        .expect("execute_testgen skip should match type")
        .boundary_str("execute_testgen", "skip_reason", "")
        .expect("execute_testgen skip_reason should match type")
        // Transport: execute_build (success)
        .transport_response(
            "execute_build",
            "response",
            TransportResponse::Shell(ShellResponse::ok(
                "Compiling gunbc v0.1.0\n    Finished dev target(s)",
            )),
        )
        .expect("execute_build response should match type")
        .boundary_bool("execute_build", "skip", false)
        .expect("execute_build skip should match type")
        .boundary_str("execute_build", "skip_reason", "")
        .expect("execute_build skip_reason should match type")
        // Transport: execute_test (FAILS)
        .transport_response(
            "execute_test",
            "response",
            TransportResponse::Shell(ShellResponse::failed(
                1,
                "running 42 tests\ntest tests::test_something ... FAILED\n\nfailures:\n    tests::test_something\n\ntest failed",
            )),
        )
        .expect("execute_test response should match type")
        .boundary_bool("execute_test", "skip", false)
        .expect("execute_test skip should match type")
        .boundary_str("execute_test", "skip_reason", "")
        .expect("execute_test skip_reason should match type")
        // Transport: execute_guardrail_check (success)
        .transport_response(
            "execute_guardrail_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("OK")),
        )
        .expect("execute_guardrail_check response should match type")
        .boundary_bool("execute_guardrail_check", "skip", false)
        .expect("execute_guardrail_check skip should match type")
        .boundary_str("execute_guardrail_check", "skip_reason", "")
        .expect("execute_guardrail_check skip_reason should match type")
        // Transport: execute_verify_check (success)
        .transport_response(
            "execute_verify_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("All checks passed")),
        )
        .expect("execute_verify_check response should match type")
        .boundary_bool("execute_verify_check", "skip", false)
        .expect("execute_verify_check skip should match type")
        .boundary_str("execute_verify_check", "skip_reason", "")
        .expect("execute_verify_check skip_reason should match type")
        // CliTool: clippy_lint (success)
        .boundary_bool("clippy_lint", "success", true)
        .expect("clippy_lint success should match type")
        .boundary_str("clippy_lint", "stdout", "Checking gunbc v0.1.0\n    Finished dev")
        .expect("clippy_lint stdout should match type")
        .boundary_str("clippy_lint", "stderr", "")
        .expect("clippy_lint stderr should match type")
        .boundary_bool("clippy_lint", "skip", false)
        .expect("clippy_lint skip should match type")
        .build_unchecked()
        .resource_lock("cargo:build")
        .resource_lock("cargo:test")
        .resource_lock("cargo:clippy")
}

/// Mock spec for testing build failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
///
/// Uses explicit mocks to model execute_build returning a failed response.
pub fn ci_mock_spec_build_fails() -> MockSpec {
    let dag = build_ci_graph().expect("ci graph should build");

    extract_mock_requirements(&dag, "ci")
        // Resource: runner_env provides tool handles
        .boundary("runner_env", "tool:clippy", ToolHandle::mock(&CLIPPY))
        .expect("runner_env tool:clippy should match type")
        // Transport: execute_deps_exists (success)
        .transport_response(
            "execute_deps_exists",
            "response",
            TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_deps_exists response should match type")
        // Transport: execute_codegen_exists (success)
        .transport_response(
            "execute_codegen_exists",
            "response",
            TransportResponse::Shell(ShellResponse::ok("")),
        )
        .expect("execute_codegen_exists response should match type")
        // Transport: execute_codegen (skipped)
        .boundary("execute_codegen", "response", Value::Skipped)
        .expect("execute_codegen response should match type")
        .boundary_bool("execute_codegen", "skip", true)
        .expect("execute_codegen skip should match type")
        // Transport: execute_stamp_write (succeeds)
        .transport_response(
            "execute_stamp_write",
            "response",
            TransportResponse::File(FileResponse {
                path: "target/.codegen-stamp".into(),
                operation: FileOp::Write,
                success: true,
                content: None,
                exists: None,
                error: None,
            }),
        )
        .expect("execute_stamp_write response should match type")
        .boundary_bool("execute_stamp_write", "skip", false)
        .expect("execute_stamp_write skip should match type")
        // Transport: execute_testgen (success)
        .transport_response(
            "execute_testgen",
            "response",
            TransportResponse::Shell(ShellResponse::ok("Generated tests")),
        )
        .expect("execute_testgen response should match type")
        .boundary_bool("execute_testgen", "skip", false)
        .expect("execute_testgen skip should match type")
        .boundary_str("execute_testgen", "skip_reason", "")
        .expect("execute_testgen skip_reason should match type")
        // Transport: execute_build (FAILS)
        .transport_response(
            "execute_build",
            "response",
            TransportResponse::Shell(ShellResponse::failed(
                1,
                "error[E0382]: borrow of moved value: `x`\n  --> src/main.rs:5:13",
            )),
        )
        .expect("execute_build response should match type")
        .boundary_bool("execute_build", "skip", false)
        .expect("execute_build skip should match type")
        .boundary_str("execute_build", "skip_reason", "")
        .expect("execute_build skip_reason should match type")
        // Transport: execute_test (skipped due to build failure)
        .boundary("execute_test", "response", Value::Skipped)
        .expect("execute_test response should match type")
        .boundary_bool("execute_test", "skip", true)
        .expect("execute_test skip should match type")
        .boundary_str("execute_test", "skip_reason", "Build failed")
        .expect("execute_test skip_reason should match type")
        // Transport: execute_guardrail_check (success)
        .transport_response(
            "execute_guardrail_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("OK")),
        )
        .expect("execute_guardrail_check response should match type")
        .boundary_bool("execute_guardrail_check", "skip", false)
        .expect("execute_guardrail_check skip should match type")
        .boundary_str("execute_guardrail_check", "skip_reason", "")
        .expect("execute_guardrail_check skip_reason should match type")
        // Transport: execute_verify_check (success - parallel with build, depends on codegen)
        .transport_response(
            "execute_verify_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("All checks passed")),
        )
        .expect("execute_verify_check response should match type")
        .boundary_bool("execute_verify_check", "skip", false)
        .expect("execute_verify_check skip should match type")
        .boundary_str("execute_verify_check", "skip_reason", "")
        .expect("execute_verify_check skip_reason should match type")
        // CliTool: clippy_lint (skipped due to build failure)
        .boundary_bool("clippy_lint", "success", false)
        .expect("clippy_lint success should match type")
        .boundary_str("clippy_lint", "stdout", "")
        .expect("clippy_lint stdout should match type")
        .boundary_str("clippy_lint", "stderr", "")
        .expect("clippy_lint stderr should match type")
        .boundary_bool("clippy_lint", "skip", true)
        .expect("clippy_lint skip should match type")
        .build_unchecked()
        .resource_lock("cargo:build")
}

/// Mock spec for testing prep/codegen failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
///
/// Uses explicit mocks to model execute_codegen returning a failed response.
pub fn ci_mock_spec_prep_fails() -> MockSpec {
    let dag = build_ci_graph().expect("ci graph should build");

    extract_mock_requirements(&dag, "ci")
        // Resource: runner_env provides tool handles
        .boundary("runner_env", "tool:clippy", ToolHandle::mock(&CLIPPY))
        .expect("runner_env tool:clippy should match type")
        // Transport: execute_deps_exists (success)
        .transport_response(
            "execute_deps_exists",
            "response",
            TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_deps_exists response should match type")
        // Transport: execute_codegen_exists (codegen needed)
        .transport_response(
            "execute_codegen_exists",
            "response",
            TransportResponse::Shell(ShellResponse::failed(1, "missing")),
        )
        .expect("execute_codegen_exists response should match type")
        // Transport: execute_codegen (FAILS)
        .transport_response(
            "execute_codegen",
            "response",
            TransportResponse::Shell(ShellResponse::failed(
                1,
                "error: codegen failed: template not found",
            )),
        )
        .expect("execute_codegen response should match type")
        .boundary_bool("execute_codegen", "skip", false)
        .expect("execute_codegen skip should match type")
        // Transport: execute_stamp_write (skipped due to prep failure)
        .boundary("execute_stamp_write", "response", Value::Skipped)
        .expect("execute_stamp_write response should match type")
        .boundary_bool("execute_stamp_write", "skip", true)
        .expect("execute_stamp_write skip should match type")
        // Transport: execute_testgen (skipped due to prep failure)
        .boundary("execute_testgen", "response", Value::Skipped)
        .expect("execute_testgen response should match type")
        .boundary_bool("execute_testgen", "skip", true)
        .expect("execute_testgen skip should match type")
        .boundary_str("execute_testgen", "skip_reason", "Prep failed")
        .expect("execute_testgen skip_reason should match type")
        // Transport: execute_build (skipped)
        .boundary("execute_build", "response", Value::Skipped)
        .expect("execute_build response should match type")
        .boundary_bool("execute_build", "skip", true)
        .expect("execute_build skip should match type")
        .boundary_str("execute_build", "skip_reason", "Prep failed")
        .expect("execute_build skip_reason should match type")
        // Transport: execute_test (skipped)
        .boundary("execute_test", "response", Value::Skipped)
        .expect("execute_test response should match type")
        .boundary_bool("execute_test", "skip", true)
        .expect("execute_test skip should match type")
        .boundary_str("execute_test", "skip_reason", "Prep failed")
        .expect("execute_test skip_reason should match type")
        // Transport: execute_guardrail_check (skipped due to testgen failure)
        .boundary("execute_guardrail_check", "response", Value::Skipped)
        .expect("execute_guardrail_check response should match type")
        .boundary_bool("execute_guardrail_check", "skip", true)
        .expect("execute_guardrail_check skip should match type")
        .boundary_str(
            "execute_guardrail_check",
            "skip_reason",
            "Skipped due to testgen failure",
        )
        .expect("execute_guardrail_check skip_reason should match type")
        // Transport: execute_verify_check (skipped due to prep failure)
        .boundary("execute_verify_check", "response", Value::Skipped)
        .expect("execute_verify_check response should match type")
        .boundary_bool("execute_verify_check", "skip", true)
        .expect("execute_verify_check skip should match type")
        .boundary_str("execute_verify_check", "skip_reason", "Prep failed")
        .expect("execute_verify_check skip_reason should match type")
        // CliTool: clippy_lint (skipped)
        .boundary_bool("clippy_lint", "success", false)
        .expect("clippy_lint success should match type")
        .boundary_str("clippy_lint", "stdout", "")
        .expect("clippy_lint stdout should match type")
        .boundary_str("clippy_lint", "stderr", "")
        .expect("clippy_lint stderr should match type")
        .boundary_bool("clippy_lint", "skip", true)
        .expect("clippy_lint skip should match type")
        .build_unchecked()
}

/// Mock spec for testing lint failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
///
/// Uses explicit mocks to model clippy_lint returning a failure.
pub fn ci_mock_spec_lint_fails() -> MockSpec {
    let dag = build_ci_graph().expect("ci graph should build");

    extract_mock_requirements(&dag, "ci")
        // Resource: runner_env provides tool handles
        .boundary("runner_env", "tool:clippy", ToolHandle::mock(&CLIPPY))
        .expect("runner_env tool:clippy should match type")
        // Transport: execute_deps_exists (success)
        .transport_response(
            "execute_deps_exists",
            "response",
            TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_deps_exists response should match type")
        // Transport: execute_codegen_exists (success)
        .transport_response(
            "execute_codegen_exists",
            "response",
            TransportResponse::Shell(ShellResponse::ok("")),
        )
        .expect("execute_codegen_exists response should match type")
        // Transport: execute_codegen (skipped)
        .boundary("execute_codegen", "response", Value::Skipped)
        .expect("execute_codegen response should match type")
        .boundary_bool("execute_codegen", "skip", true)
        .expect("execute_codegen skip should match type")
        // Transport: execute_stamp_write (succeeds)
        .transport_response(
            "execute_stamp_write",
            "response",
            TransportResponse::File(FileResponse {
                path: "target/.codegen-stamp".into(),
                operation: FileOp::Write,
                success: true,
                content: None,
                exists: None,
                error: None,
            }),
        )
        .expect("execute_stamp_write response should match type")
        .boundary_bool("execute_stamp_write", "skip", false)
        .expect("execute_stamp_write skip should match type")
        // Transport: execute_testgen (success)
        .transport_response(
            "execute_testgen",
            "response",
            TransportResponse::Shell(ShellResponse::ok("Generated tests")),
        )
        .expect("execute_testgen response should match type")
        .boundary_bool("execute_testgen", "skip", false)
        .expect("execute_testgen skip should match type")
        .boundary_str("execute_testgen", "skip_reason", "")
        .expect("execute_testgen skip_reason should match type")
        // Transport: execute_build (success)
        .transport_response(
            "execute_build",
            "response",
            TransportResponse::Shell(ShellResponse::ok(
                "Compiling gunbc v0.1.0\n    Finished dev target(s)",
            )),
        )
        .expect("execute_build response should match type")
        .boundary_bool("execute_build", "skip", false)
        .expect("execute_build skip should match type")
        .boundary_str("execute_build", "skip_reason", "")
        .expect("execute_build skip_reason should match type")
        // Transport: execute_test (success)
        .transport_response(
            "execute_test",
            "response",
            TransportResponse::Shell(ShellResponse::ok(
                "running 42 tests\ntest result: ok. 42 passed",
            )),
        )
        .expect("execute_test response should match type")
        .boundary_bool("execute_test", "skip", false)
        .expect("execute_test skip should match type")
        .boundary_str("execute_test", "skip_reason", "")
        .expect("execute_test skip_reason should match type")
        // Transport: execute_guardrail_check (success)
        .transport_response(
            "execute_guardrail_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("OK")),
        )
        .expect("execute_guardrail_check response should match type")
        .boundary_bool("execute_guardrail_check", "skip", false)
        .expect("execute_guardrail_check skip should match type")
        .boundary_str("execute_guardrail_check", "skip_reason", "")
        .expect("execute_guardrail_check skip_reason should match type")
        // Transport: execute_verify_check (success)
        .transport_response(
            "execute_verify_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("All checks passed")),
        )
        .expect("execute_verify_check response should match type")
        .boundary_bool("execute_verify_check", "skip", false)
        .expect("execute_verify_check skip should match type")
        .boundary_str("execute_verify_check", "skip_reason", "")
        .expect("execute_verify_check skip_reason should match type")
        // CliTool: clippy_lint (FAILS)
        .boundary_bool("clippy_lint", "success", false)
        .expect("clippy_lint success should match type")
        .boundary_str("clippy_lint", "stdout", "")
        .expect("clippy_lint stdout should match type")
        .boundary_str("clippy_lint", "stderr", "error: unused variable `x`\n  --> src/main.rs:3:9\n   |\n3  |     let x = 1;\n   |         ^ help: if this is intentional, prefix it with an underscore: `_x`\n   |\n   = note: `-D unused-variables` implied by `-D warnings`")
        .expect("clippy_lint stderr should match type")
        .boundary_bool("clippy_lint", "skip", false)
        .expect("clippy_lint skip should match type")
        .build_unchecked()
        .resource_lock("cargo:build")
        .resource_lock("cargo:test")
        .resource_lock("cargo:clippy")
}

/// Mock spec with build lock contention.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn ci_mock_spec_build_contended() -> MockSpec {
    let dag = build_ci_graph().expect("ci graph should build");

    // All mocks (including clippy_lint) are handled by with_ci_typed_mocks
    with_ci_typed_mocks(extract_mock_requirements(&dag, "ci"))
        .build_unchecked()
        .resource_lock_fails("cargo:build", "Another cargo build is in progress")
}

/// Helper to fill common CI transport mocks using typed builder.
///
/// This fills all the required slots for transport, resource, and CLI tool nodes
/// in the CI graph. Includes clippy_lint which is now detected as a CliTool node
/// (has ToolHandle input).
fn with_ci_typed_mocks(
    reqs: gunbc_test::MockRequirements,
) -> gunbc_test::MockRequirements {
    reqs
        // Resource: runner_env provides tool handles
        .boundary("runner_env", "tool:clippy", ToolHandle::mock(&CLIPPY))
        .expect("runner_env tool:clippy should match type")
        // Transport: execute_deps_exists (check deps.toml)
        .transport_response(
            "execute_deps_exists",
            "response",
            TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_deps_exists response should match type")
        // Transport: execute_codegen_exists (check codegen output)
        .transport_response(
            "execute_codegen_exists",
            "response",
            TransportResponse::Shell(ShellResponse::ok("")),
        )
        .expect("execute_codegen_exists response should match type")
        // Transport: execute_codegen (skipped - already exists)
        .boundary("execute_codegen", "response", Value::Skipped)
        .expect("execute_codegen response should match type")
        .boundary_bool("execute_codegen", "skip", true)
        .expect("execute_codegen skip should match type")
        // Transport: execute_stamp_write (succeeds)
        .transport_response(
            "execute_stamp_write",
            "response",
            TransportResponse::File(FileResponse {
                path: "target/.codegen-stamp".into(),
                operation: FileOp::Write,
                success: true,
                content: None,
                exists: None,
                error: None,
            }),
        )
        .expect("execute_stamp_write response should match type")
        .boundary_bool("execute_stamp_write", "skip", false)
        .expect("execute_stamp_write skip should match type")
        // Transport: execute_testgen (succeeds)
        .transport_response(
            "execute_testgen",
            "response",
            TransportResponse::Shell(ShellResponse::ok("Generated tests")),
        )
        .expect("execute_testgen response should match type")
        .boundary_bool("execute_testgen", "skip", false)
        .expect("execute_testgen skip should match type")
        .boundary_str("execute_testgen", "skip_reason", "")
        .expect("execute_testgen skip_reason should match type")
        // Transport: execute_build (succeeds)
        .transport_response(
            "execute_build",
            "response",
            TransportResponse::Shell(ShellResponse::ok(
                "Compiling gunbc v0.1.0\n    Finished dev target(s)",
            )),
        )
        .expect("execute_build response should match type")
        .boundary_bool("execute_build", "skip", false)
        .expect("execute_build skip should match type")
        .boundary_str("execute_build", "skip_reason", "")
        .expect("execute_build skip_reason should match type")
        // Transport: execute_test (succeeds)
        .transport_response(
            "execute_test",
            "response",
            TransportResponse::Shell(ShellResponse::ok(
                "running 42 tests\ntest result: ok. 42 passed",
            )),
        )
        .expect("execute_test response should match type")
        .boundary_bool("execute_test", "skip", false)
        .expect("execute_test skip should match type")
        .boundary_str("execute_test", "skip_reason", "")
        .expect("execute_test skip_reason should match type")
        // Transport: execute_guardrail_check (succeeds)
        .transport_response(
            "execute_guardrail_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("OK")),
        )
        .expect("execute_guardrail_check response should match type")
        .boundary_bool("execute_guardrail_check", "skip", false)
        .expect("execute_guardrail_check skip should match type")
        .boundary_str("execute_guardrail_check", "skip_reason", "")
        .expect("execute_guardrail_check skip_reason should match type")
        // Transport: execute_verify_check (succeeds)
        .transport_response(
            "execute_verify_check",
            "response",
            TransportResponse::Shell(ShellResponse::ok("All checks passed")),
        )
        .expect("execute_verify_check response should match type")
        .boundary_bool("execute_verify_check", "skip", false)
        .expect("execute_verify_check skip should match type")
        .boundary_str("execute_verify_check", "skip_reason", "")
        .expect("execute_verify_check skip_reason should match type")
        // CliTool: clippy_lint (succeeds) - now detected by extract_mock_requirements
        .boundary_bool("clippy_lint", "success", true)
        .expect("clippy_lint success should match type")
        .boundary_str("clippy_lint", "stdout", "Checking gunbc v0.1.0\n    Finished dev")
        .expect("clippy_lint stdout should match type")
        .boundary_str("clippy_lint", "stderr", "")
        .expect("clippy_lint stderr should match type")
        .boundary_bool("clippy_lint", "skip", false)
        .expect("clippy_lint skip should match type")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_builder_rejects_wrong_slot() {
        let dag = build_ci_graph().expect("graph should build");
        gunbc_test::assert_typed_builder_rejects_invalid_slot(&dag, "ci");
    }
}
