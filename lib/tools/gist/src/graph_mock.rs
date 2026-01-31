//! Mock specification for the gist tool.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - What input constraints upstream must satisfy
//!
//! Used by testgen for:
//! - Dry-run testing with realistic mock values
//! - Chain validation with other tools

use crate::graph::GistMode;
use gunbc_ir::Value;
use gunbc_test::{InputConstraint, MockSpec};

/// Build a mock specification for the gist graph.
///
/// The spec adapts to the mode — snapshot mode mocks `execute_list_files` and
/// `execute_read_files` boundaries, while diff mode mocks `execute_diff`.
/// Both share the `execute_gist` boundary.
///
/// # Boundary Mocks
///
/// **Snapshot mode:**
/// - `execute_list_files`: Lists files via git ls-files
/// - `execute_read_files`: Reads file contents via batch read
/// - `execute_gist`: Creates the gist (world write)
///
/// **Diff mode:**
/// - `execute_diff`: Runs `git diff base...HEAD`
/// - `execute_gist`: Creates the gist (world write)
///
/// # Input Expectations
///
/// - `repo_path`: Optional string (both modes)
/// - `base_ref`: Optional string (diff mode only)
pub fn gist_mock_spec(mode: &GistMode) -> MockSpec {
    let mut spec = MockSpec::new("gist");

    match mode {
        GistMode::Snapshot => {
            spec = spec
                // Boundary: execute_list_files outputs
                .boundary(
                    "execute_list_files",
                    "response",
                    Value::Json(serde_json::json!({
                        "exit_code": 0,
                        "stdout": "src/main.rs\nREADME.md\n",
                        "stderr": ""
                    })),
                )
                // Boundary: execute_read_files outputs
                .boundary(
                    "execute_read_files",
                    "response",
                    Value::Json(serde_json::json!({
                        "exit_code": 0,
                        "stdout": "===GUNBC_FILE:src/main.rs===\nfn main() {}\n===GUNBC_FILE:README.md===\n# README\n",
                        "stderr": ""
                    })),
                );
        }
        GistMode::Diff { .. } => {
            spec = spec
                // Boundary: execute_diff outputs
                .boundary(
                    "execute_diff",
                    "response",
                    Value::Json(serde_json::json!({
                        "exit_code": 0,
                        "stdout": "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }\n",
                        "stderr": ""
                    })),
                );
        }
    }

    // Shared gist boundary (both modes)
    spec = spec
        .boundary(
            "execute_gist",
            "url",
            Value::Str("https://gist.github.com/mock/abc123def456".into()),
        )
        .boundary(
            "execute_gist",
            "response",
            Value::Json(serde_json::json!({
                "id": "abc123def456",
                "html_url": "https://gist.github.com/mock/abc123def456",
                "files": {},
                "public": false
            })),
        );

    // Input expectations
    spec = spec.expects_input("repo_path", InputConstraint::Any);
    if matches!(mode, GistMode::Diff { .. }) {
        spec = spec.expects_input("base_ref", InputConstraint::Any);
    }

    spec
}

/// Mock spec for testing gist with file system lock simulation.
///
/// Use this when testing tools that acquire file locks before reading.
pub fn gist_mock_spec_with_fs_lock() -> MockSpec {
    gist_mock_spec(&GistMode::Snapshot)
        .resource_lock("fs:read")
}

/// Mock spec for testing lease expiration scenarios.
pub fn gist_mock_spec_lease_expires() -> MockSpec {
    gist_mock_spec(&GistMode::Snapshot)
        .resource_lease_expires("github:api_token", 5000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_test::validate_chain;
    use std::collections::HashMap;

    // ========================================================================
    // Snapshot mode mock spec tests
    // ========================================================================

    #[test]
    fn test_snapshot_mock_spec_has_boundaries() {
        let spec = gist_mock_spec(&GistMode::Snapshot);

        assert!(spec.get_boundary_mock("execute_list_files", "response").is_some());
        assert!(spec.get_boundary_mock("execute_read_files", "response").is_some());
        assert!(spec.get_boundary_mock("execute_gist", "url").is_some());
        assert!(spec.get_boundary_mock("execute_gist", "response").is_some());
    }

    #[test]
    fn test_snapshot_mock_spec_no_diff_boundaries() {
        let spec = gist_mock_spec(&GistMode::Snapshot);

        assert!(spec.get_boundary_mock("execute_diff", "response").is_none());
    }

    #[test]
    fn test_snapshot_mock_spec_url_is_valid() {
        let spec = gist_mock_spec(&GistMode::Snapshot);
        let url = spec.get_boundary_mock("execute_gist", "url").unwrap();

        if let Value::Str(s) = url {
            assert!(s.starts_with("https://gist.github.com/"));
        } else {
            panic!("Expected string URL");
        }
    }

    #[test]
    fn test_snapshot_chain_validation_self() {
        let spec = gist_mock_spec(&GistMode::Snapshot);
        let mapping = HashMap::new();
        let result = validate_chain(&spec, &spec, &mapping);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Diff mode mock spec tests
    // ========================================================================

    #[test]
    fn test_diff_mock_spec_has_boundaries() {
        let mode = GistMode::Diff { base_ref: "main".to_string() };
        let spec = gist_mock_spec(&mode);

        assert!(spec.get_boundary_mock("execute_diff", "response").is_some());
        assert!(spec.get_boundary_mock("execute_gist", "url").is_some());
        assert!(spec.get_boundary_mock("execute_gist", "response").is_some());
    }

    #[test]
    fn test_diff_mock_spec_no_snapshot_boundaries() {
        let mode = GistMode::Diff { base_ref: "main".to_string() };
        let spec = gist_mock_spec(&mode);

        assert!(spec.get_boundary_mock("execute_list_files", "response").is_none());
        assert!(spec.get_boundary_mock("execute_read_files", "response").is_none());
    }

    #[test]
    fn test_diff_mock_spec_url_is_valid() {
        let mode = GistMode::Diff { base_ref: "main".to_string() };
        let spec = gist_mock_spec(&mode);
        let url = spec.get_boundary_mock("execute_gist", "url").unwrap();

        if let Value::Str(s) = url {
            assert!(s.starts_with("https://gist.github.com/"));
        } else {
            panic!("Expected string URL");
        }
    }

    #[test]
    fn test_diff_chain_validation_self() {
        let mode = GistMode::Diff { base_ref: "main".to_string() };
        let spec = gist_mock_spec(&mode);
        let mapping = HashMap::new();
        let result = validate_chain(&spec, &spec, &mapping);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Utility mock spec tests
    // ========================================================================

    #[test]
    fn test_resource_lock_present() {
        let spec = gist_mock_spec_with_fs_lock();
        assert!(spec.get_resource("fs:read").is_some());
    }
}
