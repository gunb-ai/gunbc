//! Mock specification for the deps tool.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - What input constraints upstream must satisfy
//! - Resource simulations for package manager operations

use gunbc_ir::Value;
use gunbc_test::{InputConstraint, MockSpec};

/// Mock specification for the deps graph.
///
/// # Boundary Mocks
///
/// The `execute_installs` node is the boundary (world write).
/// It outputs:
/// - `executed`: Whether the install script was run
/// - `script`: The script that was executed
///
/// # Input Expectations
///
/// - `manifest_path`: Optional string, defaults to "deps.toml"
///
/// # Resource Simulations
///
/// - Package manager lock: Ensures only one install runs at a time
/// - Sudo lease: Time-bounded privilege elevation
pub fn deps_mock_spec() -> MockSpec {
    MockSpec::new("deps")
        // Boundary: execute_installs outputs
        .boundary("execute_installs", "executed", Value::Bool(true))
        .boundary(
            "execute_installs",
            "script",
            Value::Str("# Mock install script\necho 'Dependencies installed'".into()),
        )
        // Input expectations
        .expects_input("manifest_path", InputConstraint::Any)
        // Resource: package manager lock (only one apt/brew at a time)
        .resource_lock("pkg:manager")
}

/// Mock spec for testing sudo elevation scenarios.
///
/// Simulates a time-bounded sudo lease (5 minutes).
pub fn deps_mock_spec_with_sudo() -> MockSpec {
    deps_mock_spec()
        // Sudo lease: 5 minutes before re-auth needed
        .resource_lease("sudo:elevation", 300_000)
}

/// Mock spec for testing package manager failure.
pub fn deps_mock_spec_pkg_fails() -> MockSpec {
    MockSpec::new("deps")
        .boundary("execute_installs", "executed", Value::Bool(false))
        .boundary(
            "execute_installs",
            "script",
            Value::Str("# Failed install script".into()),
        )
        .expects_input("manifest_path", InputConstraint::Any)
        .resource_lock_fails("pkg:manager", "Package manager locked by another process")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_spec_has_boundary() {
        let spec = deps_mock_spec();
        assert!(spec.get_boundary_mock("execute_installs", "executed").is_some());
        assert!(spec.get_boundary_mock("execute_installs", "script").is_some());
    }

    #[test]
    fn test_mock_spec_executed_is_bool() {
        let spec = deps_mock_spec();
        let executed = spec.get_boundary_mock("execute_installs", "executed").unwrap();
        assert!(matches!(executed, Value::Bool(true)));
    }

    #[test]
    fn test_sudo_lease_present() {
        let spec = deps_mock_spec_with_sudo();
        let resource = spec.get_resource("sudo:elevation").unwrap();
        assert!(matches!(
            resource.resource_type,
            gunbc_test::ResourceType::Lease { duration_ms: 300_000 }
        ));
    }

    #[test]
    fn test_pkg_fails_spec() {
        let spec = deps_mock_spec_pkg_fails();
        let resource = spec.get_resource("pkg:manager").unwrap();
        let result = resource.acquire();
        assert!(matches!(result, gunbc_test::ResourceAcquireResult::Failed(_)));
    }
}
