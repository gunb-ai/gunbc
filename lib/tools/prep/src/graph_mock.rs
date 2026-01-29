//! Mock specification for the prep tool.
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

/// Mock specification for the prep graph.
///
/// # Boundary Mocks
///
/// The `build` node is the only boundary.
/// It outputs:
/// - `build_ran`: Whether build was executed
/// - `build_success`: Whether build succeeded
///
/// # Input Expectations
///
/// The prep tool expects:
/// - `dry_run`: Optional boolean (can be omitted)
pub fn prep_mock_spec() -> MockSpec {
    MockSpec::new("prep")
        // Boundary: build outputs
        .boundary("build", "build_ran", Value::Bool(true))
        .boundary("build", "build_success", Value::Bool(true))
        // Input expectations
        .expects_input("dry_run", InputConstraint::Any)
        // Resource locks - prevent concurrent builds
        .resource_lock("cargo:build")
}

/// Mock spec for testing prep with codegen already done.
pub fn prep_mock_spec_codegen_exists() -> MockSpec {
    prep_mock_spec()
        // Simulate codegen already exists
        .boundary("check_state", "needs_codegen", Value::Bool(false))
        .boundary("check_state", "buck_out_exists", Value::Bool(true))
}

/// Mock spec for testing prep failure scenarios.
pub fn prep_mock_spec_build_fails() -> MockSpec {
    MockSpec::new("prep_failure")
        .boundary("build", "build_ran", Value::Bool(true))
        .boundary("build", "build_success", Value::Bool(false))
        .resource_lock("cargo:build")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_test::validate_chain;
    use std::collections::HashMap;

    #[test]
    fn test_mock_spec_has_boundary() {
        let spec = prep_mock_spec();

        assert!(spec.get_boundary_mock("build", "build_ran").is_some());
        assert!(spec.get_boundary_mock("build", "build_success").is_some());
    }

    #[test]
    fn test_mock_spec_build_success() {
        let spec = prep_mock_spec();
        let success = spec.get_boundary_mock("build", "build_success").unwrap();

        if let Value::Bool(b) = success {
            assert!(b, "Default mock should indicate success");
        } else {
            panic!("Expected boolean");
        }
    }

    #[test]
    fn test_chain_validation_self() {
        let spec = prep_mock_spec();
        let mapping = HashMap::new();
        let result = validate_chain(&spec, &spec, &mapping);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resource_lock_present() {
        let spec = prep_mock_spec();
        assert!(spec.get_resource("cargo:build").is_some());
    }

    #[test]
    fn test_failure_spec() {
        let spec = prep_mock_spec_build_fails();
        let success = spec.get_boundary_mock("build", "build_success").unwrap();

        if let Value::Bool(b) = success {
            assert!(!b, "Failure mock should indicate failure");
        } else {
            panic!("Expected boolean");
        }
    }
}
