//! GitHub REST API configuration and request builder.
//!
//! This module provides typed configuration for the GitHub REST API,
//! enabling consistent headers, authentication, and versioning across
//! all GitHub API requests.
//!
//! # Example
//!
//! ```text
//! use gunbc_ir::transport::github::api::*;
//!
//! // Create a request to the gists endpoint
//! let request = github_rest_request("/gists")
//!     .method(HttpMethod::Post)
//!     .json(body);
//! ```

use super::GITHUB_API_VERSION;
use crate::transport::rest::RestRequest;
use crate::transport::HttpMethod;

// ============================================================================
// API Configuration
// ============================================================================

/// GitHub REST API configuration.
///
/// Encapsulates the base URL, API version, and accept header for GitHub API requests.
#[derive(Debug, Clone)]
pub struct GitHubApi {
    /// Base URL for the API (e.g., "https://api.github.com")
    pub base_url: &'static str,
    /// API version header value
    pub api_version: &'static str,
    /// Accept header for JSON responses
    pub accept_header: &'static str,
}

/// Default GitHub API configuration (public github.com).
pub const GITHUB_API: GitHubApi = GitHubApi {
    base_url: "https://api.github.com",
    api_version: GITHUB_API_VERSION,
    accept_header: "application/vnd.github+json",
};

/// GitHub Enterprise API configuration template.
///
/// Use this as a starting point for GitHub Enterprise Server instances.
/// Replace `base_url` with your enterprise instance URL.
pub fn github_enterprise_api(base_url: &'static str) -> GitHubApi {
    GitHubApi {
        base_url,
        api_version: GITHUB_API_VERSION,
        accept_header: "application/vnd.github+json",
    }
}

// ============================================================================
// Request Builder
// ============================================================================

/// Build a REST request with GitHub API headers.
///
/// This creates a GET request with:
/// - Proper Accept header for GitHub API
/// - X-GitHub-Api-Version header
/// - Authentication should be attached via the cloud credential chain at the DAG boundary
///
/// # Arguments
///
/// * `endpoint` - API endpoint path (e.g., "/gists", "/repos/owner/repo")
///
/// # Example
///
/// ```text
/// // GET /user
/// let req = github_rest_request("/user");
///
/// // POST /gists with body
/// let req = github_rest_request("/gists")
///     .method(HttpMethod::Post)
///     .json(body);
/// ```
pub fn github_rest_request(endpoint: &str) -> RestRequest {
    RestRequest::get(format!("{}{}", GITHUB_API.base_url, endpoint))
        .header("Accept", GITHUB_API.accept_header)
        .header("X-GitHub-Api-Version", GITHUB_API.api_version)
}

/// Build a REST request for a specific GitHub API configuration.
///
/// Use this for GitHub Enterprise or custom configurations.
pub fn github_rest_request_with_config(api: &GitHubApi, endpoint: &str) -> RestRequest {
    RestRequest::get(format!("{}{}", api.base_url, endpoint))
        .header("Accept", api.accept_header)
        .header("X-GitHub-Api-Version", api.api_version)
}

/// Build a POST request to the GitHub API.
pub fn github_rest_post(endpoint: &str) -> RestRequest {
    github_rest_request(endpoint).method(HttpMethod::Post)
}

// ============================================================================
// Helper trait for method chaining
// ============================================================================

/// Extension trait to set HTTP method on RestRequest.
pub trait RestRequestExt {
    fn method(self, method: HttpMethod) -> Self;
}

impl RestRequestExt for RestRequest {
    fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_api_config() {
        assert_eq!(GITHUB_API.base_url, "https://api.github.com");
        assert_eq!(GITHUB_API.api_version, GITHUB_API_VERSION);
        assert_eq!(GITHUB_API.accept_header, "application/vnd.github+json");
    }

    #[test]
    fn test_github_rest_request() {
        let req = github_rest_request("/gists");

        assert_eq!(req.url, "https://api.github.com/gists");
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(
            req.headers.get("Accept"),
            Some(&"application/vnd.github+json".to_string())
        );
        assert_eq!(
            req.headers.get("X-GitHub-Api-Version"),
            Some(&GITHUB_API_VERSION.to_string())
        );
        assert!(req.auth.is_none());
    }

    #[test]
    fn test_github_rest_post() {
        let req = github_rest_post("/gists");
        assert_eq!(req.method, HttpMethod::Post);
    }

    #[test]
    fn test_github_enterprise_api() {
        let enterprise = github_enterprise_api("https://github.mycompany.com/api/v3");
        assert_eq!(enterprise.base_url, "https://github.mycompany.com/api/v3");
        assert_eq!(enterprise.api_version, GITHUB_API_VERSION);
    }

    #[test]
    fn test_github_rest_request_with_config() {
        let enterprise = github_enterprise_api("https://github.mycompany.com/api/v3");
        let req = github_rest_request_with_config(&enterprise, "/user");

        assert_eq!(req.url, "https://github.mycompany.com/api/v3/user");
    }
}
