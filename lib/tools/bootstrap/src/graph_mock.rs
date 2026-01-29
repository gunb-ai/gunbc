//! Mock specification for the bootstrap tool.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - Resource simulations for file system operations

use gunbc_ir::Value;
use gunbc_test::MockSpec;

/// Mock specification for the bootstrap graph.
///
/// # Boundary Mocks
///
/// The `write_files` node is the boundary (world write).
/// It outputs:
/// - `files_written`: List of files that were written
/// - `write_count`: Number of files written
///
/// # Input Expectations
///
/// No inputs - bootstrap scans the workspace automatically.
///
/// # Resource Simulations
///
/// - File locks for Makefile and .gitignore
pub fn bootstrap_mock_spec() -> MockSpec {
    MockSpec::new("bootstrap")
        // Boundary: write_files outputs
        .boundary(
            "write_files",
            "files_written",
            Value::StrList(vec![
                "Makefile".into(),
                ".gitignore".into(),
            ]),
        )
        .boundary("write_files", "write_count", Value::Int(2))
        // Resources: file locks for both outputs
        .resource_lock("fs:Makefile")
        .resource_lock("fs:.gitignore")
}

/// Mock spec for testing single file write.
pub fn bootstrap_mock_spec_makefile_only() -> MockSpec {
    MockSpec::new("bootstrap")
        .boundary(
            "write_files",
            "files_written",
            Value::StrList(vec!["Makefile".into()]),
        )
        .boundary("write_files", "write_count", Value::Int(1))
        .resource_lock("fs:Makefile")
}

/// Mock spec for testing file system failure on Makefile.
pub fn bootstrap_mock_spec_makefile_fails() -> MockSpec {
    MockSpec::new("bootstrap")
        .boundary(
            "write_files",
            "files_written",
            Value::StrList(vec![".gitignore".into()]),
        )
        .boundary("write_files", "write_count", Value::Int(1))
        .resource_lock_fails("fs:Makefile", "Permission denied: Makefile is read-only")
        .resource_lock("fs:.gitignore")
}

/// Mock spec for testing complete write failure.
pub fn bootstrap_mock_spec_all_fail() -> MockSpec {
    MockSpec::new("bootstrap")
        .boundary("write_files", "files_written", Value::StrList(vec![]))
        .boundary("write_files", "write_count", Value::Int(0))
        .resource_lock_fails("fs:Makefile", "Permission denied")
        .resource_lock_fails("fs:.gitignore", "Permission denied")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_spec_has_boundary() {
        let spec = bootstrap_mock_spec();
        assert!(spec.get_boundary_mock("write_files", "files_written").is_some());
        assert!(spec.get_boundary_mock("write_files", "write_count").is_some());
    }

    #[test]
    fn test_mock_spec_write_count() {
        let spec = bootstrap_mock_spec();
        let count = spec.get_boundary_mock("write_files", "write_count").unwrap();
        assert!(matches!(count, Value::Int(2)));
    }

    #[test]
    fn test_mock_spec_files_written() {
        let spec = bootstrap_mock_spec();
        let files = spec.get_boundary_mock("write_files", "files_written").unwrap();
        if let Value::StrList(list) = files {
            assert!(list.contains(&"Makefile".to_string()));
            assert!(list.contains(&".gitignore".to_string()));
        } else {
            panic!("Expected StrList");
        }
    }

    #[test]
    fn test_both_file_locks_present() {
        let spec = bootstrap_mock_spec();
        assert!(spec.get_resource("fs:Makefile").is_some());
        assert!(spec.get_resource("fs:.gitignore").is_some());
    }

    #[test]
    fn test_makefile_fails_spec() {
        let spec = bootstrap_mock_spec_makefile_fails();
        let makefile = spec.get_resource("fs:Makefile").unwrap();
        let gitignore = spec.get_resource("fs:.gitignore").unwrap();
        
        assert!(matches!(makefile.acquire(), gunbc_test::ResourceAcquireResult::Failed(_)));
        assert!(matches!(gitignore.acquire(), gunbc_test::ResourceAcquireResult::Acquired));
    }
}
