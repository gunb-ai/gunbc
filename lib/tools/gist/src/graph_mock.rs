//! Mock specification for the gist tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! Used by testgen for:
//! - Dry-run testing with realistic mock values
//! - Chain validation with other tools

use crate::graph::{build_gist_graph, GistMode};
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{Timestamp, Value};
use gunbc_primitives::filename;
use gunbc_test::{extract_mock_requirements, InputConstraint, MockSpec};
use std::time::SystemTime;

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

fn mock_clock() -> Value {
    Timestamp::from_system_time(SystemTime::UNIX_EPOCH).into()
}

/// Build a mock specification for the gist graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
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
    // Build the actual DAG to extract requirements
    let dag = build_gist_graph(mode.clone(), vec![], false)
        .expect("gist graph should build");

    // Extract typed requirements from DAG structure
    let mut reqs = extract_mock_requirements(&dag, "gist")
        // Environment: filesystem + clock
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs:write mock should match type")
        .boundary("clock_env", "clock", mock_clock())
        .expect("clock mock should match type");

    // Mode-specific transport mocks
    match mode {
        GistMode::Snapshot => {
            reqs = reqs
                // execute_list_files transport response
                .transport_response(
                    "execute_list_files",
                    "response",
                    TransportResponse::Shell(ShellResponse {
                        exit_code: 0,
                        stdout: "src/main.rs\nREADME.md\n".to_string(),
                        stderr: String::new(),
                    }),
                )
                .expect("execute_list_files response should match type")
                // execute_read_files transport response
                .transport_response(
                    "execute_read_files",
                    "response",
                    TransportResponse::Shell(ShellResponse {
                        exit_code: 0,
                        stdout: "===GUNBC_FILE:src/main.rs===\nfn main() {}\n===GUNBC_FILE:README.md===\n# README\n".to_string(),
                        stderr: String::new(),
                    }),
                )
                .expect("execute_read_files response should match type");
        }
        GistMode::Diff { .. } => {
            reqs = reqs
                // execute_diff transport response
                .transport_response(
                    "execute_diff",
                    "response",
                    TransportResponse::Shell(ShellResponse {
                        exit_code: 0,
                        stdout: "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }\n".to_string(),
                        stderr: String::new(),
                    }),
                )
                .expect("execute_diff response should match type");
        }
    }

    // Shared: current branch acquisition
    reqs = reqs
        .transport_response(
            "execute_current_branch",
            "response",
            TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: "main\n".to_string(),
                stderr: String::new(),
            }),
        )
        .expect("execute_current_branch response should match type");

    // Shared: gist creation
    reqs = reqs
        .transport_response(
            "execute_gist",
            "response",
            TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: serde_json::json!({
                    "id": "abc123def456",
                    "html_url": "https://gist.github.com/mock/abc123def456",
                    "files": {},
                    "public": false
                })
                .to_string(),
                stderr: String::new(),
            }),
        )
        .expect("execute_gist response should match type");

    // Terminal boundary: parse_gist_response.url
    reqs = reqs
        .boundary_str(
            "parse_gist_response",
            "url",
            "https://gist.github.com/mock/abc123def456",
        )
        .expect("url mock should match type");

    // Build spec (with input expectations added via legacy API)
    let mut spec = reqs.build_unchecked();

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
    gist_mock_spec(&GistMode::Snapshot).resource_lock("fs:read")
}

/// Mock spec for testing lease expiration scenarios.
pub fn gist_mock_spec_lease_expires() -> MockSpec {
    gist_mock_spec(&GistMode::Snapshot).resource_lease_expires("github:api_token", 5000)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Mock spec mode behavior tests
    // ========================================================================
    //
    // These tests verify mode-specific boundary setup (Pattern B - mock value
    // properties). They ensure snapshot mode doesn't have diff boundaries and
    // vice versa.
    //
    // Note: Boundary PRESENCE tests (Pattern A), self-chain validation (Pattern C),
    // and resource presence tests (Pattern D) are auto-generated by testgen and
    // have been removed from this file.

    #[test]
    fn test_snapshot_mock_spec_no_diff_boundaries() {
        let spec = gist_mock_spec(&GistMode::Snapshot);

        // execute_diff doesn't exist in snapshot mode, so no mock for it
        assert!(spec.get_transport_mock("execute_diff", "response").is_none());
    }

    #[test]
    fn test_snapshot_mock_spec_url_is_valid() {
        let spec = gist_mock_spec(&GistMode::Snapshot);
        let url = spec.get_boundary_mock("parse_gist_response", "url").unwrap();

        if let Value::Str(s) = url {
            assert!(s.starts_with("https://gist.github.com/"));
        } else {
            panic!("Expected string URL");
        }
    }

    #[test]
    fn test_diff_mock_spec_no_snapshot_boundaries() {
        let mode = GistMode::Diff {
            base_ref: "main".to_string(),
        };
        let spec = gist_mock_spec(&mode);

        // execute_list_files and execute_read_files don't exist in diff mode
        assert!(spec
            .get_transport_mock("execute_list_files", "response")
            .is_none());
        assert!(spec
            .get_transport_mock("execute_read_files", "response")
            .is_none());
    }

    #[test]
    fn test_diff_mock_spec_url_is_valid() {
        let mode = GistMode::Diff {
            base_ref: "main".to_string(),
        };
        let spec = gist_mock_spec(&mode);
        let url = spec.get_boundary_mock("parse_gist_response", "url").unwrap();

        if let Value::Str(s) = url {
            assert!(s.starts_with("https://gist.github.com/"));
        } else {
            panic!("Expected string URL");
        }
    }

    #[test]
    fn test_typed_builder_catches_type_errors() {
        // This test verifies that the typed builder pattern works
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");

        let reqs = extract_mock_requirements(&dag, "gist");

        // Try to set a string where we expect a FilesystemHandle
        let result = reqs.boundary_str("fs_env", "fs:write", "wrong type");

        // This should fail with a type mismatch
        assert!(result.is_err());
    }
}
