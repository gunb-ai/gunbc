//! Low-level HTTP request/response types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP method.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, gunbc_delegate_macros::StringEnum,
)]
#[string_enum(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

/// Raw HTTP request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Request body (raw bytes as base64 or string)
    pub body: Option<String>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// Raw HTTP response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: String,
}

impl HttpRequest {
    /// Create a new GET request.
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
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
            timeout_ms: None,
        }
    }

    /// Add a header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the timeout.
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }
}

impl HttpResponse {
    /// Check if the response was successful (2xx status).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response was a client error (4xx status).
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if the response was a server error (5xx status).
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_parse() {
        assert_eq!(HttpMethod::parse("GET"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::parse("post"), Some(HttpMethod::Post));
        assert_eq!(HttpMethod::parse("INVALID"), None);
    }

    #[test]
    fn test_http_request_builder() {
        let req = HttpRequest::post("https://api.example.com")
            .header("Content-Type", "application/json")
            .body(r#"{"test": true}"#)
            .timeout(5000);

        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.url, "https://api.example.com");
        assert_eq!(
            req.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(req.body, Some(r#"{"test": true}"#.to_string()));
        assert_eq!(req.timeout_ms, Some(5000));
    }
}
