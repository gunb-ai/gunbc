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
//! - `manifest_path`: Optional string, defaults to "deps.toml"
//!
//! # Resource Simulations
//!
//! - Package manager lock: Ensures only one install runs at a time
//! - Sudo lease: Time-bounded privilege elevation

use crate::graph::build_deps_graph;
use crate::Platform;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_test::{extract_mock_requirements, InputConstraint, MockSpec};

fn mock_manifest() -> &'static str {
    r#"[dependency]
name = "ripgrep"
verify_cmd = "rg --version"
install_cmd = "cargo install ripgrep"
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
#[gunbc_testgen_registry_macros::testgen_target(
    name = "deps",
    output = "lib/tools/deps/src/generated_tests.rs",
    module = "deps_generated_tests",
    builder = "crate::graph::build_deps_graph().unwrap()",
    signature = "crate::deps_signature()"
)]
pub fn deps_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_deps_graph().expect("deps graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "deps")
        // Resource: platform environment
        .boundary("platform_env", "platform", Platform::Linux)
        .expect("platform mock should match type")
        // Transport: load manifest
        .transport_response(
            "execute_load_manifest",
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
            "execute_installs",
            "response",
            TransportResponse::Shell(ShellResponse::ok("Dependencies installed")),
        )
        .expect("execute_installs response should match type")
        // Build spec (pure terminal outputs are computed, not mocked)
        .build_unchecked()
        // Input expectations (via legacy API post-build)
        .expects_input("manifest_path", InputConstraint::Any)
        // Resource: package manager lock
        .resource_lock("pkg:manager")
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
        // Resource: platform environment
        .boundary("platform_env", "platform", Platform::Linux)
        .expect("platform mock should match type")
        // Transport: load manifest (succeeds)
        .transport_response(
            "execute_load_manifest",
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
            "execute_installs",
            "response",
            TransportResponse::Shell(ShellResponse::failed(1, "Package manager locked by another process")),
        )
        .expect("execute_installs response should match type")
        // Build spec
        .build_unchecked()
        .expects_input("manifest_path", InputConstraint::Any)
        .resource_lock_fails("pkg:manager", "Package manager locked by another process")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::Value;

    // ========================================================================
    // Mock spec tests (Pattern B - mock value properties)
    // ========================================================================
    //
    // These tests verify specific mock value properties. Pattern A (presence),
    // C (self-chain), and D (resource presence) tests are auto-generated by
    // testgen and have been removed.

    #[test]
    fn test_mock_spec_platform_is_boundary() {
        let spec = deps_mock_spec();
        let platform = spec.get_boundary_mock("platform_env", "platform").unwrap();
        // Platform converts to Value::Str
        assert!(matches!(platform, Value::Str(_)));
    }

    #[test]
    fn test_sudo_lease_present() {
        let spec = deps_mock_spec_with_sudo();
        let resource = spec.get_resource("sudo:elevation").unwrap();
        assert!(matches!(
            resource.resource_type,
            gunbc_test::ResourceType::Lease {
                duration_ms: 300_000
            }
        ));
    }

    #[test]
    fn test_typed_builder_rejects_wrong_slot() {
        // This test verifies that setting an unknown slot fails
        let dag = build_deps_graph().expect("graph should build");
        let reqs = extract_mock_requirements(&dag, "deps");

        // Try to set a mock for a non-existent node
        let result = reqs.boundary_str("nonexistent_node", "port", "value");

        // This should fail with UnknownSlot
        assert!(result.is_err());
    }
}
