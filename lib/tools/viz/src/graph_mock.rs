//! Mock specification for the viz tool.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - What input constraints upstream must satisfy
//! - Resource simulations for file system operations

use gunbc_ir::Value;
use gunbc_test::{InputConstraint, MockSpec};

/// Mock specification for the viz graph.
///
/// # Boundary Mocks
///
/// The `execute_transport` node is the boundary (world write).
/// It outputs:
/// - `response`: Transport response
/// - `written_path`: Path where viz data was written
///
/// # Input Expectations
///
/// - `output_path`: Optional string, defaults to "viz-data.json"
pub fn viz_mock_spec() -> MockSpec {
    MockSpec::new("viz")
        // Boundary: execute_transport outputs
        .boundary(
            "execute_transport",
            "written_path",
            Value::Str("viz-data.json".into()),
        )
        .boundary(
            "execute_transport",
            "response",
            Value::Json(serde_json::json!({
                "status": "ok",
                "bytes_written": 1024
            })),
        )
        // Input expectations
        .expects_input("output_path", InputConstraint::Any)
        // Resource: file write lock
        .resource_lock("fs:viz-data.json")
}

/// Mock spec for testing custom output path.
pub fn viz_mock_spec_custom_path(path: &str) -> MockSpec {
    MockSpec::new("viz")
        .boundary("execute_transport", "written_path", Value::Str(path.into()))
        .boundary(
            "execute_transport",
            "response",
            Value::Json(serde_json::json!({
                "status": "ok",
                "bytes_written": 1024
            })),
        )
        .expects_input("output_path", InputConstraint::Any)
        .resource_lock(&format!("fs:{}", path))
}

/// Mock spec for testing file system failure.
pub fn viz_mock_spec_fs_fails() -> MockSpec {
    MockSpec::new("viz")
        .boundary("execute_transport", "written_path", Value::Str("".into()))
        .boundary(
            "execute_transport",
            "response",
            Value::Json(serde_json::json!({
                "status": "error",
                "error": "Permission denied"
            })),
        )
        .expects_input("output_path", InputConstraint::Any)
        .resource_lock_fails("fs:viz-data.json", "Permission denied: viz-data.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_spec_has_boundary() {
        let spec = viz_mock_spec();
        assert!(spec.get_boundary_mock("execute_transport", "written_path").is_some());
        assert!(spec.get_boundary_mock("execute_transport", "response").is_some());
    }

    #[test]
    fn test_mock_spec_path_default() {
        let spec = viz_mock_spec();
        let path = spec.get_boundary_mock("execute_transport", "written_path").unwrap();
        assert!(matches!(path, Value::Str(s) if s == "viz-data.json"));
    }

    #[test]
    fn test_custom_path_spec() {
        let spec = viz_mock_spec_custom_path("output/graphs.json");
        let path = spec.get_boundary_mock("execute_transport", "written_path").unwrap();
        assert!(matches!(path, Value::Str(s) if s == "output/graphs.json"));
    }

    #[test]
    fn test_fs_lock_present() {
        let spec = viz_mock_spec();
        assert!(spec.get_resource("fs:viz-data.json").is_some());
    }

    #[test]
    fn test_fs_fails_spec() {
        let spec = viz_mock_spec_fs_fails();
        let resource = spec.get_resource("fs:viz-data.json").unwrap();
        let result = resource.acquire();
        assert!(matches!(result, gunbc_test::ResourceAcquireResult::Failed(_)));
    }
}
