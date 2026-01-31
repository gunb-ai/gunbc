//! REST API request/response types.

use super::http::HttpMethod;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Authentication method for REST APIs.
///
/// Debug and Display output redacts credential values to prevent
/// accidental leakage in logs. Only the auth variant and non-sensitive
/// metadata (header names, env var names) are shown.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// Bearer token (e.g., OAuth, JWT)
    Bearer(String),
    /// Basic authentication (username:password base64 encoded)
    Basic { username: String, password: String },
    /// API key in header (literal value)
    ApiKey { header: String, key: String },
    /// Environment variable resolved at execution time → `Authorization: Bearer {value}`.
    EnvVar(String),
    /// Environment variable resolved at execution time → custom header.
    ///
    /// Like `EnvVar` but inserts the value into the specified header instead of
    /// `Authorization: Bearer`. Used for APIs that authenticate via a custom
    /// header (e.g., Anthropic's `x-api-key`).
    EnvVarHeader { header: String, env_var: String },
}

impl fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthMethod::None => write!(f, "AuthMethod::None"),
            AuthMethod::Bearer(_) => write!(f, "AuthMethod::Bearer(***)"),
            AuthMethod::Basic { username, .. } => {
                write!(f, "AuthMethod::Basic {{ username: {username:?}, password: *** }}")
            }
            AuthMethod::ApiKey { header, .. } => {
                write!(f, "AuthMethod::ApiKey {{ header: {header:?}, key: *** }}")
            }
            AuthMethod::EnvVar(var) => write!(f, "AuthMethod::EnvVar({var:?})"),
            AuthMethod::EnvVarHeader { header, env_var } => {
                write!(f, "AuthMethod::EnvVarHeader {{ header: {header:?}, env_var: {env_var:?} }}")
            }
        }
    }
}

impl fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthMethod::None => write!(f, "none"),
            AuthMethod::Bearer(_) => write!(f, "bearer(***)"),
            AuthMethod::Basic { username, .. } => write!(f, "basic({username}, ***)"),
            AuthMethod::ApiKey { header, .. } => write!(f, "api-key({header}, ***)"),
            AuthMethod::EnvVar(var) => write!(f, "env({var})"),
            AuthMethod::EnvVarHeader { header, env_var } => {
                write!(f, "env-header({header}, {env_var})")
            }
        }
    }
}

/// REST API request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
