//! File operation request/response types.

use serde::{Deserialize, Serialize};

/// File operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOp {
    /// Read file contents
    Read,
    /// Write file (create or overwrite)
    Write,
    /// Append to file
    Append,
    /// Delete file
    Delete,
    /// Check if file exists
    Exists,
    /// Create directory
    CreateDir,
    /// Expand a glob pattern into file paths (content = newline-separated list)
    Glob,
    /// Fetch file metadata (content = modified millis since epoch)
    Metadata,
}

/// File operation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileRequest {
    /// File path
    pub path: String,
    /// Operation to perform
    pub operation: FileOp,
    /// Content for write/append operations
    pub content: Option<String>,
    /// Create parent directories if needed (for write operations)
    pub create_parents: bool,
}

/// File operation response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileResponse {
    /// File path
    pub path: String,
    /// Operation that was performed
    pub operation: FileOp,
    /// Whether the operation succeeded
    pub success: bool,
    /// Content (for read operations)
    pub content: Option<String>,
    /// Whether the file exists (for exists operations)
    pub exists: Option<bool>,
    /// Error message if operation failed
    pub error: Option<String>,
}

impl FileRequest {
    /// Create a read request.
    pub fn read(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Read,
            content: None,
            create_parents: false,
        }
    }

    /// Create a write request.
    pub fn write(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Write,
            content: Some(content.into()),
            create_parents: true,
        }
    }

    /// Create an append request.
    pub fn append(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Append,
            content: Some(content.into()),
            create_parents: true,
        }
    }

    /// Create a delete request.
    pub fn delete(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Delete,
            content: None,
            create_parents: false,
        }
    }

    /// Create an exists check request.
    pub fn exists(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Exists,
            content: None,
            create_parents: false,
        }
    }

    /// Create a directory creation request.
    pub fn create_dir(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::CreateDir,
            content: None,
            create_parents: true,
        }
    }

    /// Create a glob expansion request.
    pub fn glob(pattern: impl Into<String>) -> Self {
        Self {
            path: pattern.into(),
            operation: FileOp::Glob,
            content: None,
            create_parents: false,
        }
    }

    /// Create a metadata request.
    pub fn metadata(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Metadata,
            content: None,
            create_parents: false,
        }
    }

    /// Set whether to create parent directories.
    pub fn with_create_parents(mut self, create: bool) -> Self {
        self.create_parents = create;
        self
    }
}

impl FileResponse {
    /// Create a successful response for a write operation.
    pub fn written(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Write,
            success: true,
            content: None,
            exists: None,
            error: None,
        }
    }

    /// Create a successful response for a read operation.
    pub fn read_ok(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Read,
            success: true,
            content: Some(content.into()),
            exists: None,
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(path: impl Into<String>, operation: FileOp, error: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            operation,
            success: false,
            content: None,
            exists: None,
            error: Some(error.into()),
        }
    }

    /// Create an exists check response.
    pub fn exists_result(path: impl Into<String>, exists: bool) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Exists,
            success: true,
            content: None,
            exists: Some(exists),
            error: None,
        }
    }

    /// Create a glob response (newline-separated list in content).
    pub fn glob_result(path: impl Into<String>, paths: Vec<String>) -> Self {
        let mut content = paths.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }
        Self {
            path: path.into(),
            operation: FileOp::Glob,
            success: true,
            content: Some(content),
            exists: None,
            error: None,
        }
    }

    /// Create a metadata response (modified millis in content).
    pub fn metadata_result(path: impl Into<String>, modified_millis: i64) -> Self {
        Self {
            path: path.into(),
            operation: FileOp::Metadata,
            success: true,
            content: Some(modified_millis.to_string()),
            exists: None,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_request_builders() {
        let read = FileRequest::read("/tmp/test.txt");
        assert_eq!(read.operation, FileOp::Read);
        assert_eq!(read.path, "/tmp/test.txt");

        let write = FileRequest::write("/tmp/test.txt", "content");
        assert_eq!(write.operation, FileOp::Write);
        assert_eq!(write.content, Some("content".to_string()));
        assert!(write.create_parents);
    }

    #[test]
    fn test_file_response() {
        let resp = FileResponse::read_ok("/tmp/test.txt", "file content");
        assert!(resp.success);
        assert_eq!(resp.content, Some("file content".to_string()));
    }

    #[test]
    fn test_file_request_glob() {
        let req = FileRequest::glob("src/**/*.rs");
        assert_eq!(req.operation, FileOp::Glob);
        assert_eq!(req.path, "src/**/*.rs");
    }

    #[test]
    fn test_file_request_metadata() {
        let req = FileRequest::metadata("Cargo.toml");
        assert_eq!(req.operation, FileOp::Metadata);
        assert_eq!(req.path, "Cargo.toml");
    }
}
