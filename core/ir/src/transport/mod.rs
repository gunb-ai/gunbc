//! Transport layer types for I/O abstraction.
//!
//! This module provides request/response types for different transport mechanisms:
//! - REST/HTTP for web APIs
//! - File operations for filesystem I/O
//! - TCP for raw network connections
//! - Shell for command execution
//!
//! The key insight is that all world I/O can be modeled as request/response pairs,
//! allowing business logic to remain pure while transport execution happens at
//! well-defined boundaries.

pub mod file;
pub mod gist;
pub mod http;
pub mod rest;
pub mod tcp;

pub use file::{FileOp, FileRequest, FileResponse};
pub use gist::GistRequest;
pub use http::{HttpMethod, HttpRequest, HttpResponse};
pub use rest::{AuthMethod, RestRequest, RestResponse};
pub use tcp::{TcpRequest, TcpResponse};

use serde::{Deserialize, Serialize};

/// Unified transport request enum.
///
/// All I/O operations are represented as one of these request types,
/// allowing uniform handling at transport boundaries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportRequest {
    /// REST API request
    Rest(RestRequest),
    /// Raw HTTP request
    Http(HttpRequest),
    /// File operation request
    File(FileRequest),
    /// TCP connection request
    Tcp(TcpRequest),
    /// Shell command request
    Shell(ShellRequest),
}

/// Unified transport response enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportResponse {
    /// REST API response
    Rest(RestResponse),
    /// Raw HTTP response
    Http(HttpResponse),
    /// File operation response
    File(FileResponse),
    /// TCP connection response
    Tcp(TcpResponse),
    /// Shell command response
    Shell(ShellResponse),
}

/// Shell command request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellRequest {
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory (optional)
    pub cwd: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Standard input to pipe to the command
    pub stdin: Option<String>,
}

/// Shell command response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellResponse {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}

impl ShellRequest {
    /// Create a new shell request.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            cwd: None,
            env: std::collections::HashMap::new(),
            stdin: None,
        }
    }

    /// Add an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set standard input.
    pub fn stdin(mut self, stdin: impl Into<String>) -> Self {
        self.stdin = Some(stdin.into());
        self
    }

    /// Set an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

impl ShellResponse {
    /// Check if the command succeeded (exit code 0).
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_request_builder() {
        let req = ShellRequest::new("gh")
            .args(["gist", "create"])
            .arg("-f")
            .arg("test.md")
            .cwd("/tmp")
            .stdin("# Test");

        assert_eq!(req.command, "gh");
        assert_eq!(req.args, vec!["gist", "create", "-f", "test.md"]);
        assert_eq!(req.cwd, Some("/tmp".to_string()));
        assert_eq!(req.stdin, Some("# Test".to_string()));
    }
}
