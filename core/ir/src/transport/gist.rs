//! GitHub Gist-specific request types.
//!
//! This module provides convenience builders for GitHub Gist API requests,
//! which can be converted to either REST or Shell transport requests.
//!
//! Built on the [`super::github`] platform layer for consistent GitHub interaction.

use super::github::{api::github_rest_request, cli::gh_cli_request};
use super::TransportRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GitHub Gist request.
///
/// This is a high-level representation of a gist operation that can be
/// converted to either a REST API request or a shell command (gh CLI).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GistRequest {
    /// Files to include in the gist
    pub files: HashMap<String, GistFile>,
    /// Whether the gist should be public
    pub public: bool,
    /// Gist description (optional)
    pub description: Option<String>,
}

/// A file in a gist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GistFile {
    /// File content
    pub content: String,
}

impl GistRequest {
    /// Create a new gist request.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            public: false,
            description: None,
        }
    }

    /// Add a file to the gist.
    pub fn file(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.files.insert(
            name.into(),
            GistFile {
                content: content.into(),
            },
        );
        self
    }

    /// Set whether the gist is public.
    pub fn public(mut self, public: bool) -> Self {
        self.public = public;
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Convert to a REST API request.
    ///
    /// Uses the GitHub Gist API: POST https://api.github.com/gists
    /// Configuration (headers, auth, API version) comes from [`super::github::api`].
    pub fn to_rest_request(&self) -> TransportRequest {
        use super::github::api::RestRequestExt;
        use super::http::HttpMethod;

        let files_json: serde_json::Map<String, serde_json::Value> = self
            .files
            .iter()
            .map(|(name, file)| {
                (
                    name.clone(),
                    serde_json::json!({ "content": file.content }),
                )
            })
            .collect();

        let mut body = serde_json::json!({
            "public": self.public,
            "files": files_json,
        });

        if let Some(ref desc) = self.description {
            body["description"] = serde_json::Value::String(desc.clone());
        }

        // Use the shared GitHub REST request builder
        let request = github_rest_request("/gists")
            .method(HttpMethod::Post)
            .json(body);

        TransportRequest::Rest(request)
    }

    /// Convert to a shell request using the gh CLI.
    ///
    /// This is useful when a GitHub CLI is available and authenticated.
    /// Configuration comes from [`super::github::cli`].
    pub fn to_shell_request(&self) -> TransportRequest {
        // Get the first file (gh CLI creates gist from a single file or stdin)
        let (filename, content) = self
            .files
            .iter()
            .next()
            .map(|(n, f)| (n.clone(), f.content.clone()))
            .unwrap_or_else(|| ("gist.txt".to_string(), String::new()));

        // Use the shared gh CLI request builder
        let mut req = gh_cli_request(&["gist", "create", "-f", &filename, "-"]).stdin(content);

        if self.public {
            req = req.arg("--public");
        }

        if let Some(ref desc) = self.description {
            req = req.args(["--desc", desc]);
        }

        TransportRequest::Shell(req)
    }
}

impl Default for GistRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a gist URL from a shell response.
pub fn parse_gist_url_from_shell(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .find(|line| line.starts_with("https://gist.github.com/"))
        .map(|s| s.trim().to_string())
}

/// Parse a gist URL from a REST response.
pub fn parse_gist_url_from_rest(body: &serde_json::Value) -> Option<String> {
    body.get("html_url").and_then(|v| v.as_str()).map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gist_request_builder() {
        let req = GistRequest::new()
            .file("test.md", "# Test content")
            .file("code.rs", "fn main() {}")
            .public(true)
            .description("A test gist");

        assert_eq!(req.files.len(), 2);
        assert!(req.public);
        assert_eq!(req.description, Some("A test gist".to_string()));
    }

    #[test]
    fn test_to_rest_request() {
        let gist = GistRequest::new()
            .file("test.md", "# Test")
            .public(true);

        let transport = gist.to_rest_request();
        
        match transport {
            TransportRequest::Rest(req) => {
                assert_eq!(req.url, "https://api.github.com/gists");
                assert!(req.body.is_some());
            }
            _ => panic!("expected REST request"),
        }
    }

    #[test]
    fn test_to_shell_request() {
        let gist = GistRequest::new()
            .file("test.md", "# Test")
            .public(true);

        let transport = gist.to_shell_request();

        match transport {
            TransportRequest::Shell(req) => {
                assert_eq!(req.command, "gh");
                assert!(req.args.contains(&"gist".to_string()));
                assert!(req.args.contains(&"--public".to_string()));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_parse_gist_url() {
        let stdout = "https://gist.github.com/abc123\n";
        assert_eq!(
            parse_gist_url_from_shell(stdout),
            Some("https://gist.github.com/abc123".to_string())
        );

        let body = serde_json::json!({
            "html_url": "https://gist.github.com/xyz789"
        });
        assert_eq!(
            parse_gist_url_from_rest(&body),
            Some("https://gist.github.com/xyz789".to_string())
        );
    }
}
