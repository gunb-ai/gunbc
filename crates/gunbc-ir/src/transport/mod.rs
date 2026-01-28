//! Transport stack as nested SubDAGs.
//!
//! Each transport layer wraps the layer below it as a swappable SubDAG:
//!
//! ```text
//! External (nominal boundary - the real world)
//!     ↑
//! TCP (SubDAG: socket operations)
//!     ↑
//! HTTP (SubDAG: request/response on TCP)
//!     ↑
//! REST (SubDAG: semantic operations on HTTP)
//!     ↑
//! GitHub::Gist (SubDAG: gist operations on REST)
//! ```
//!
//! Mocking at any level is achieved by swapping the SubDAG for that layer:
//! - `--mock-gist` → swap Gist SubDAG for mock (fake URL, no network)
//! - `--mock-rest` → swap REST SubDAG for mock (fake HTTP response, real gist parsing)
//! - `--mock-http` → swap HTTP SubDAG for mock (fake TCP response, real HTTP/REST parsing)
//! - `--mock-tcp` → swap TCP SubDAG for loopback (rarely needed)

pub mod tcp;
pub mod http;
pub mod rest;

pub use tcp::{TcpOp, build_tcp_real, build_tcp_mock};
pub use http::{HttpOp, build_http_real, build_http_mock};
pub use rest::{RestOp, build_rest_real, build_rest_mock};

/// External type ID conventions for transport layer boundaries.
pub mod external_types {
    use crate::TypeId;

    // TCP layer
    pub fn tcp_connection() -> TypeId { TypeId("External::TCP::Connection".into()) }

    // HTTP layer
    pub fn http_request() -> TypeId { TypeId("External::HTTP::Request".into()) }
    pub fn http_response() -> TypeId { TypeId("External::HTTP::Response".into()) }

    // REST layer
    pub fn rest_request() -> TypeId { TypeId("External::REST::Request".into()) }
    pub fn rest_response() -> TypeId { TypeId("External::REST::Response".into()) }

    // GitHub layer
    pub fn github_gist() -> TypeId { TypeId("External::GitHub::Gist".into()) }
    pub fn github_auth() -> TypeId { TypeId("External::GitHub::Auth".into()) }

    // Filesystem layer
    pub fn fs_read() -> TypeId { TypeId("External::FS::Read".into()) }
    pub fn fs_write() -> TypeId { TypeId("External::FS::Write".into()) }

    /// Extract the layer name from an External type ID.
    /// e.g., "External::GitHub::Gist" -> "gist"
    pub fn extract_layer_name(type_id: &TypeId) -> Option<String> {
        let s = &type_id.0;
        if !s.starts_with("External::") {
            return None;
        }
        // Get the last segment and lowercase it
        s.rsplit("::").next().map(|s| s.to_lowercase())
    }
}
