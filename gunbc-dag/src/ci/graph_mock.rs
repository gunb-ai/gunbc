//! Mock specification for the CI tool.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - Resource simulations for CI operations

use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_test::MockSpec;

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
            Value::Response(TransportResponse::File(FileResponse {
                path: "deps.toml".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            })),
        )
        // -- Prep: codegen output already exists
        .transport_mock(
            "execute_codegen_exists",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: "buck-out/gen/bin".into(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            })),
        )
        // -- Codegen: skipped (already exists)
        .transport_mock("execute_codegen", "response", Value::Skipped)
        .transport_mock("execute_codegen", "skip", Value::Bool(true))
        // -- Build: succeeds
        .transport_mock(
            "execute_build",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: "Compiling gunbc v0.1.0\n    Finished dev target(s)".into(),
                stderr: String::new(),
            })),
        )
        .transport_mock("execute_build", "skip", Value::Bool(false))
        .transport_mock("execute_build", "skip_reason", Value::Str(String::new()))
        // -- Test: succeeds
        .transport_mock(
            "execute_test",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: "running 42 tests\ntest result: ok. 42 passed".into(),
                stderr: String::new(),
            })),
        )
        .transport_mock("execute_test", "skip", Value::Bool(false))
        .transport_mock("execute_test", "skip_reason", Value::Str(String::new()))
        // -- Lint: clippy succeeds
        .transport_mock("clippy_lint", "success", Value::Bool(true))
        .transport_mock("clippy_lint", "stdout", Value::Str(String::new()))
        .transport_mock("clippy_lint", "stderr", Value::Str(String::new()))
        .transport_mock("clippy_lint", "skip", Value::Bool(false))
        // Expected outputs: verified after DryRun execution
        .expected_output("report", "overall_success", Value::Bool(true))
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
