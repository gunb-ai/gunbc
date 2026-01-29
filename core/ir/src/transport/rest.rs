//! REST API request/response types.

use super::http::HttpMethod;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication method for REST APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// Bearer token (e.g., OAuth, JWT)
    Bearer(String),
    /// Basic authentication (username:password base64 encoded)
    Basic { username: String, password: String },
    /// API key in header
    ApiKey { header: String, key: String },
    /// Environment variable reference (resolved at execution time)
    EnvVar(String),
}

/// REST API request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestRequest {
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// JSON request body
    pub body: Option<serde_json::Value>,
    /// Authentication method
    pub auth: Option<AuthMethod>,
    /// Query parameters
    #[serde(default)]
    pub query: HashMap<String, String>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// REST API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// JSON response body
    pub body: serde_json::Value,
}

impl RestRequest {
    /// Create a new GET request.
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            auth: None,
            query: HashMap::new(),
            timeout_ms: None,
        }
    }

    /// Create a new POST request.
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            body: None,
            auth: None,
            query: HashMap::new(),
            timeout_ms: None,
        }
    }

    /// Create a new PUT request.
    pub fn put(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Put,
            headers: HashMap::new(),
            body: None,
            auth: None,
            query: HashMap::new(),
            timeout_ms: None,
        }
    }

    /// Create a new DELETE request.
    pub fn delete(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Delete,
            headers: HashMap::new(),
            body: None,
            auth: None,
            query: HashMap::new(),
            timeout_ms: None,
        }
    }

    /// Set the JSON body.
    pub fn json(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Add a header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add a query parameter.
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    /// Set bearer token authentication.
    pub fn bearer(mut self, token: impl Into<String>) -> Self {
        self.auth = Some(AuthMethod::Bearer(token.into()));
        self
    }

    /// Set authentication from environment variable.
    pub fn auth_env(mut self, var_name: impl Into<String>) -> Self {
        self.auth = Some(AuthMethod::EnvVar(var_name.into()));
        self
    }

    /// Set the timeout.
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }
}

impl RestResponse {
    /// Create a new response.
    pub fn new(status: u16, body: serde_json::Value) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body,
        }
    }

    /// Create a successful response.
    pub fn ok(body: serde_json::Value) -> Self {
        Self::new(200, body)
    }

    /// Create an error response.
    pub fn error(status: u16, message: &str) -> Self {
        Self::new(status, serde_json::json!({ "error": message }))
    }

    /// Check if the response was successful (2xx status).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Get a field from the JSON body.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.body.get(key)
    }

    /// Get a string field from the JSON body.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.body.get(key).and_then(|v| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_request_builder() {
        let req = RestRequest::post("https://api.github.com/gists")
            .json(serde_json::json!({
                "public": true,
                "files": {
                    "test.md": { "content": "# Test" }
                }
            }))
            .bearer("token123")
            .header("Accept", "application/vnd.github+json")
            .timeout(30000);

        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.url, "https://api.github.com/gists");
        assert!(req.body.is_some());
        assert!(matches!(req.auth, Some(AuthMethod::Bearer(_))));
    }

    #[test]
    fn test_rest_response() {
        let resp = RestResponse::ok(serde_json::json!({
            "html_url": "https://gist.github.com/abc123"
        }));

        assert!(resp.is_success());
        assert_eq!(resp.get_str("html_url"), Some("https://gist.github.com/abc123"));
    }
}
