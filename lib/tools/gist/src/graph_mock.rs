//! Mock specification for the gist tool.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - What input constraints upstream must satisfy
//!
//! Used by testgen for:
//! - Dry-run testing with realistic mock values
//! - Chain validation with other tools

use gunbc_ir::Value;
use gunbc_test::{InputConstraint, MockSpec};

/// Mock specification for the gist graph.
///
/// # Boundary Mocks
///
/// Transport boundary nodes:
/// - `execute_list_files`: Lists files via git ls-files
/// - `execute_gist`: Creates the gist (world write)
///
/// # Input Expectations
///
/// The gist tool expects:
/// - `repo_path`: Optional string (can be empty)
/// - Files must exist and be readable (checked by transport layer)
pub fn gist_mock_spec() -> MockSpec {
    MockSpec::new("gist")
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
        )
        // Boundary: execute_gist outputs
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
        )
        // Input expectations
        .expects_input("repo_path", InputConstraint::Any) // Optional
}

/// Mock spec for testing gist with file system lock simulation.
///
/// Use this when testing tools that acquire file locks before reading.
pub fn gist_mock_spec_with_fs_lock() -> MockSpec {
    gist_mock_spec()
        // Simulate file system read lock
        .resource_lock("fs:read")
}

/// Mock spec for testing lease expiration scenarios.
pub fn gist_mock_spec_lease_expires() -> MockSpec {
    gist_mock_spec()
        // Simulate a lease that expires after 5 seconds
        .resource_lease_expires("github:api_token", 5000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_test::validate_chain;
    use std::collections::HashMap;

    #[test]
    fn test_mock_spec_has_boundaries() {
        let spec = gist_mock_spec();

        // execute_list_files boundary
        assert!(spec.get_boundary_mock("execute_list_files", "response").is_some());

        // execute_gist boundary
        assert!(spec.get_boundary_mock("execute_gist", "url").is_some());
        assert!(spec.get_boundary_mock("execute_gist", "response").is_some());
    }

    #[test]
    fn test_mock_spec_url_is_valid() {
        let spec = gist_mock_spec();
        let url = spec.get_boundary_mock("execute_gist", "url").unwrap();

        if let Value::Str(s) = url {
            assert!(s.starts_with("https://gist.github.com/"));
        } else {
            panic!("Expected string URL");
        }
    }

    #[test]
    fn test_chain_validation_self() {
        // A tool should be self-consistent
        let spec = gist_mock_spec();
        let mapping = HashMap::new(); // No cross-tool edges for self
        let result = validate_chain(&spec, &spec, &mapping);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resource_lock_present() {
        let spec = gist_mock_spec_with_fs_lock();
        assert!(spec.get_resource("fs:read").is_some());
    }
}
