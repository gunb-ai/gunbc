//! TCP connection request/response types.

use serde::{Deserialize, Serialize};

/// TCP connection request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcpRequest {
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Data to send
    pub data: Option<String>,
    /// Write timeout in milliseconds.
    ///
    /// Note: prior versions exposed this as `connect_timeout_ms`.
    #[serde(alias = "connect_timeout_ms")]
    pub write_timeout_ms: Option<u64>,
    /// Read timeout in milliseconds
    pub read_timeout_ms: Option<u64>,
}

/// TCP connection response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcpResponse {
    /// Whether the connection was successful
    pub connected: bool,
    /// Data received
    pub data: Option<String>,
    /// Bytes sent
    pub bytes_sent: usize,
    /// Bytes received
    pub bytes_received: usize,
    /// Error message if connection failed
    pub error: Option<String>,
}

impl TcpRequest {
    /// Create a new TCP request.
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            data: None,
            write_timeout_ms: Some(30000),
            read_timeout_ms: Some(30000),
        }
    }

    /// Set the data to send.
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Set the write timeout.
    pub fn write_timeout(mut self, ms: u64) -> Self {
        self.write_timeout_ms = Some(ms);
        self
    }

    /// Backward-compatible alias for [`Self::write_timeout`].
    ///
    /// Kept to preserve existing callsites while transport decomposition
    /// migrates to explicit `write_timeout_ms` naming.
    pub fn connect_timeout(self, ms: u64) -> Self {
        self.write_timeout(ms)
    }

    /// Set the read timeout.
    pub fn read_timeout(mut self, ms: u64) -> Self {
        self.read_timeout_ms = Some(ms);
        self
    }
}

impl TcpResponse {
    /// Create a successful response.
    pub fn ok(data: Option<String>, bytes_sent: usize, bytes_received: usize) -> Self {
        Self {
            connected: true,
            data,
            bytes_sent,
            bytes_received,
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            connected: false,
            data: None,
            bytes_sent: 0,
            bytes_received: 0,
            error: Some(error.into()),
        }
    }

    /// Check if the connection was successful.
    pub fn is_ok(&self) -> bool {
        self.connected && self.error.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_request_builder() {
        let req = TcpRequest::new("localhost", 8080)
            .data("PING\n")
            .write_timeout(5000)
            .read_timeout(10000);

        assert_eq!(req.host, "localhost");
        assert_eq!(req.port, 8080);
        assert_eq!(req.data, Some("PING\n".to_string()));
        assert_eq!(req.write_timeout_ms, Some(5000));
    }
}
