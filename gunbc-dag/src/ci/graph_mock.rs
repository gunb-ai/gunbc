//! Mock specification for the CI tool.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - Resource simulations for CI operations

use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse};
use gunbc_ir::Value;
use gunbc_test::{MockSpec, NodeExample, OutputMatcher};

/// Mock specification for the CI graph.
///
/// # Boundary Mocks
///
/// The `report` node is the boundary (world write).
/// It outputs:
/// - `overall_success`: Whether CI passed
/// - `report`: Human-readable CI report
///
/// # Input Expectations
///
/// No external inputs - CI runs from workspace root.
///
/// # Resource Simulations
///
/// - Build lock: Only one cargo build at a time
/// - Test parallelism: Cargo test uses multiple threads
pub fn ci_mock_spec() -> MockSpec {
    MockSpec::new("ci")
        // Boundary: report outputs
        .boundary("report", "overall_success", Value::Bool(true))
        .boundary("report", "report", Value::Str(mock_ci_report_success()))
        // Resources
        .resource_lock("cargo:build")
        .resource_lock("cargo:test")
        .resource_lock("cargo:clippy")
        // Transport mocks: values returned by intercepted transport executor nodes
        // -- SetupDeps: deps.toml exists
        .transport_mock(
            "execute_deps_exists",
            "response",
            Value::Response(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            }.into()),
        )
        // -- Prep: codegen output already exists
        .transport_mock(
            "execute_codegen_exists",
            "response",
            Value::Response(FileResponse {
                path: "buck-out/gen/bin".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            }.into()),
        )
        // -- Codegen: skipped (already exists)
        .transport_mock("execute_codegen", "response", Value::Skipped)
        .transport_mock("execute_codegen", "skip", Value::Bool(true))
        // -- Build: succeeds
        .transport_mock(
            "execute_build",
            "response",
            Value::Response(ShellResponse::ok("Compiling gunbc v0.1.0\n    Finished dev target(s)").into()),
        )
        .transport_mock("execute_build", "skip", Value::Bool(false))
        .transport_mock("execute_build", "skip_reason", Value::Str(String::new()))
        // -- Test: succeeds
        .transport_mock(
            "execute_test",
            "response",
            Value::Response(ShellResponse::ok("running 42 tests\ntest result: ok. 42 passed").into()),
        )
        .transport_mock("execute_test", "skip", Value::Bool(false))
        .transport_mock("execute_test", "skip_reason", Value::Str(String::new()))
        // -- Clippy lint: succeeds (intercepted because it consumes ToolHandle)
        .transport_mock("clippy_lint", "success", Value::Bool(true))
        .transport_mock("clippy_lint", "stdout", Value::Str("Checking gunbc v0.1.0\n    Finished dev".into()))
        .transport_mock("clippy_lint", "stderr", Value::Str(String::new()))
        .transport_mock("clippy_lint", "skip", Value::Bool(false))
        // Expected outputs: verified after DryRun execution
        .expected_output("report", "overall_success", Value::Bool(true))
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("report")
                .input("build_success", Value::Bool(true))
                .input("test_success", Value::Bool(true))
                .input("lint_success", Value::Bool(true))
                .output("overall_success", OutputMatcher::exact(Value::Bool(true)))
                .output("report", OutputMatcher::contains("SUCCESS"))
                .description("All stages pass → overall success"),
        )
        .node_example(
            NodeExample::new("report")
                .input("build_success", Value::Bool(false))
                .input("test_success", Value::Bool(true))
                .input("lint_success", Value::Bool(true))
                .output("overall_success", OutputMatcher::exact(Value::Bool(false)))
                .output("report", OutputMatcher::contains("FAILURE"))
                .description("Build failure → overall failure"),
        )
        // Node I/O examples: verify pure node behavior
        //
        // Parse nodes now test both real transport responses AND skip propagation.
        .node_example(
            NodeExample::new("parse_deps_exists")
                .input("response", Value::Response(FileResponse {
                    path: "deps.toml".into(),
                    operation: FileOp::Exists,
                    success: true,
                    content: None,
                    exists: Some(true),
                    error: None,
                }.into()))
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
                .input("response", Value::Response(FileResponse {
                    path: "buck-out/gen/bin".into(),
                    operation: FileOp::Exists,
                    success: true,
                    content: None,
                    exists: Some(true),
                    error: None,
                }.into()))
                .output("codegen_needed", OutputMatcher::exact(Value::Bool(false)))
                .output("prep_success", OutputMatcher::exact(Value::Bool(true)))
                .output("codegen_ran", OutputMatcher::exact(Value::Bool(false)))
                .description("File exists: codegen dir found → codegen not needed"),
        )
        .node_example(
            NodeExample::new("parse_codegen_exists")
                .input("response", Value::Skipped)
                .output("codegen_needed", OutputMatcher::Any)
                .description("Handles skipped transport response gracefully"),
        )
        .node_example(
            NodeExample::new("parse_codegen_result")
                .input("response", Value::Response(ShellResponse::ok("Generated 3 files").into()))
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
            NodeExample::new("parse_build")
                .input("response", Value::Response(ShellResponse::ok("Compiling gunbc v0.1.0\n    Finished dev").into()))
                .input("skip", Value::Bool(false))
                .output("build_success", OutputMatcher::exact(Value::Bool(true)))
                .output("build_skipped", OutputMatcher::exact(Value::Bool(false)))
                .output("build_stdout", OutputMatcher::contains("Compiling"))
                .description("Build shell success → build_success true"),
        )
        .node_example(
            NodeExample::new("parse_build")
                .input("skip", Value::Bool(true))
                .output("build_success", OutputMatcher::exact(Value::Bool(false)))
                .output("build_skipped", OutputMatcher::exact(Value::Bool(true)))
                .description("Skip path: build skipped → success false, skipped true"),
        )
        .node_example(
            NodeExample::new("parse_test")
                .input("response", Value::Response(ShellResponse::ok("running 42 tests\ntest result: ok. 42 passed").into()))
                .input("skip", Value::Bool(false))
                .output("test_success", OutputMatcher::exact(Value::Bool(true)))
                .output("test_skipped", OutputMatcher::exact(Value::Bool(false)))
                .output("test_stdout", OutputMatcher::contains("42 tests"))
                .description("Test shell success → test_success true"),
        )
        .node_example(
            NodeExample::new("parse_test")
                .input("skip", Value::Bool(true))
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
            NodeExample::new("prepare_codegen_cmd")
                .output("skip", OutputMatcher::IsBool)
                .description("Codegen command prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("prepare_build")
                .output("skip", OutputMatcher::IsBool)
                .description("Build prepare emits skip flag"),
        )
        .node_example(
            NodeExample::new("prepare_test")
                .output("skip", OutputMatcher::IsBool)
                .description("Test prepare emits skip flag"),
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
                .output("lint_success", OutputMatcher::IsBool)
                .output("lint_skipped", OutputMatcher::IsBool)
                .description("Clippy result parse produces success/skipped flags"),
        )
        // Primitive nodes — tested in their own crates
        .skip_node_example("prepare_deps_exists")
}

/// Mock spec for testing CI failure.
pub fn ci_mock_spec_test_fails() -> MockSpec {
    MockSpec::new("ci")
        .boundary("report", "overall_success", Value::Bool(false))
        .boundary("report", "report", Value::Str(mock_ci_report_test_fail()))
        .resource_lock("cargo:build")
        .resource_lock("cargo:test")
        .resource_lock("cargo:clippy")
}

/// Mock spec for testing build failure.
pub fn ci_mock_spec_build_fails() -> MockSpec {
    MockSpec::new("ci")
        .boundary("report", "overall_success", Value::Bool(false))
        .boundary("report", "report", Value::Str(mock_ci_report_build_fail()))
        .resource_lock("cargo:build")
}

/// Mock spec for testing prep/codegen failure.
pub fn ci_mock_spec_prep_fails() -> MockSpec {
    MockSpec::new("ci")
        .boundary("prep", "prep_success", Value::Bool(false))
        .boundary("prep", "codegen_ran", Value::Bool(true))
        .boundary("prep", "prep_message", Value::Str("Codegen failed".into()))
        .boundary("report", "overall_success", Value::Bool(false))
        .boundary("report", "report", Value::Str(mock_ci_report_prep_fail()))
}

/// Mock spec for testing lint failure.
pub fn ci_mock_spec_lint_fails() -> MockSpec {
    MockSpec::new("ci")
        .boundary("report", "overall_success", Value::Bool(false))
        .boundary("report", "report", Value::Str(mock_ci_report_lint_fail()))
        .resource_lock("cargo:build")
        .resource_lock("cargo:test")
        .resource_lock("cargo:clippy")
}

/// Mock spec with build lock contention.
pub fn ci_mock_spec_build_contended() -> MockSpec {
    MockSpec::new("ci")
        .boundary("report", "overall_success", Value::Bool(false))
        .boundary("report", "report", Value::Str("Build blocked: another build in progress".into()))
        .resource_lock_fails("cargo:build", "Another cargo build is in progress")
}

fn mock_ci_report_success() -> String {
    r#"CI Report
=========
Build:  PASS
Test:   PASS (42 tests)
Lint:   PASS

Overall: SUCCESS"#
        .to_string()
}

fn mock_ci_report_test_fail() -> String {
    r#"CI Report
=========
Build:  PASS
Test:   FAIL (2 failures)
Lint:   PASS

Overall: FAILURE"#
        .to_string()
}

fn mock_ci_report_build_fail() -> String {
    r#"CI Report
=========
Build:  FAIL (compilation error)
Test:   SKIPPED
Lint:   SKIPPED

Overall: FAILURE"#
        .to_string()
}

fn mock_ci_report_prep_fail() -> String {
    r#"CI Report
=========
Prep:   FAIL (codegen error)
Build:  SKIPPED
Test:   SKIPPED
Lint:   SKIPPED

Overall: FAILURE"#
        .to_string()
}

fn mock_ci_report_lint_fail() -> String {
    r#"CI Report
=========
Build:  PASS
Test:   PASS (42 tests)
Lint:   FAIL (3 warnings as errors)

Overall: FAILURE"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_spec_has_boundary() {
        let spec = ci_mock_spec();
        assert!(spec.get_boundary_mock("report", "overall_success").is_some());
        assert!(spec.get_boundary_mock("report", "report").is_some());
    }

    #[test]
    fn test_mock_spec_success() {
        let spec = ci_mock_spec();
        let success = spec.get_boundary_mock("report", "overall_success").unwrap();
        assert!(matches!(success, Value::Bool(true)));
    }

    #[test]
    fn test_mock_spec_failure() {
        let spec = ci_mock_spec_test_fails();
        let success = spec.get_boundary_mock("report", "overall_success").unwrap();
        assert!(matches!(success, Value::Bool(false)));
    }

    #[test]
    fn test_cargo_locks_present() {
        let spec = ci_mock_spec();
        assert!(spec.get_resource("cargo:build").is_some());
        assert!(spec.get_resource("cargo:test").is_some());
        assert!(spec.get_resource("cargo:clippy").is_some());
    }

    #[test]
    fn test_build_contended_spec() {
        let spec = ci_mock_spec_build_contended();
        let build = spec.get_resource("cargo:build").unwrap();
        let result = build.acquire();
        assert!(matches!(result, gunbc_test::ResourceAcquireResult::Failed(_)));
    }

    #[test]
    fn test_report_contains_status() {
        let spec = ci_mock_spec();
        let report = spec.get_boundary_mock("report", "report").unwrap();
        if let Value::Str(s) = report {
            assert!(s.contains("SUCCESS"));
            assert!(s.contains("Build:  PASS"));
        } else {
            panic!("Expected string report");
        }
    }
}
