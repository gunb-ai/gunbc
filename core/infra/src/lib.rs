//! Infrastructure layer for gunbc: hashing, manifests, resource IDs.
//!
//! This crate provides the low-level primitives that other gunbc crates build on.
//! It has no internal dependencies — only external crates (serde, sha2, etc.).

#![deny(dead_code)]
// Infra is a low-level hub (fs helpers, test utilities).
#![allow(clippy::disallowed_methods)]
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a resource.
///
/// Resources can be files, locks, connections, or any other external state.
/// Two accesses conflict if they reference the same ResourceId.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);

impl ResourceId {
    /// Create a new resource ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create a file resource ID.
    pub fn file(path: impl Into<String>) -> Self {
        Self(format!("file:{}", path.into()))
    }

    /// Create a lock resource ID.
    pub fn lock(name: impl Into<String>) -> Self {
        Self(format!("lock:{}", name.into()))
    }

    /// Create a connection resource ID.
    pub fn connection(name: impl Into<String>) -> Self {
        Self(format!("conn:{}", name.into()))
    }

    /// Create a tool resource ID.
    ///
    /// Used for CLI tool capability tracking. When a node requires a tool,
    /// it creates a resource access with this ID.
    pub fn tool(name: impl Into<String>) -> Self {
        Self(format!("tool:{}", name.into()))
    }

    /// Create a build resource ID.
    ///
    /// Used for build artifact tracking (codegen, testgen, etc.).
    pub fn build(name: impl Into<String>) -> Self {
        Self(format!("build:{}", name.into()))
    }
}

impl From<&str> for ResourceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub mod emit_manifest;
pub mod freshness;
pub mod hash;
pub mod manifest;
pub mod test_utils;
