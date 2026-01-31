//! Cargo workspace binary invocation.
//!
//! Models how to invoke a binary from a cargo workspace. Handles the
//! distinction between standalone packages and binaries that live inside
//! another package:
//!
//! - Standalone: `cargo run -p gunbc-gist`
//! - In-package: `cargo run -p gunbc-dag --bin gunbc-ci`
//!
//! This is the single source of truth for cargo invocation rendering,
//! used by both Makefile generation and CI YAML generation.

/// Describes how to invoke a cargo workspace binary.
///
/// This type is the canonical way to represent "run this binary from
/// the workspace" across all renderers (Makefile, GitHub Actions, GitLab CI).
#[derive(Debug, Clone, Default)]
pub struct CargoInvocation {
    /// The binary name (e.g., "gunbc-ci").
    pub binary: String,
    /// The package name, if different from the binary (e.g., "gunbc-dag").
    /// Set when the binary is a `[[bin]]` entry in another crate's Cargo.toml.
    pub package: Option<String>,
}

impl CargoInvocation {
    /// Create an invocation for a binary in its own package.
    ///
    /// Produces: `cargo run -p <binary>`
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            package: None,
        }
    }

    /// Create an invocation for a binary inside another package.
    ///
    /// Produces: `cargo run -p <package> --bin <binary>`
    pub fn in_package(binary: impl Into<String>, package: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            package: Some(package.into()),
        }
    }

    /// Get the cargo run arguments (without the `cargo run` prefix).
    ///
    /// Returns `-p <package> --bin <binary>` or `-p <binary>`.
    pub fn args(&self) -> String {
        match &self.package {
            Some(pkg) => format!("-p {} --bin {}", pkg, self.binary),
            None => format!("-p {}", self.binary),
        }
    }

    /// Get the full `cargo run` command string.
    ///
    /// Returns `cargo run -p <package> --bin <binary>` or `cargo run -p <binary>`.
    pub fn command(&self) -> String {
        format!("cargo run {}", self.args())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standalone_package() {
        let inv = CargoInvocation::new("gunbc-gist");
        assert_eq!(inv.args(), "-p gunbc-gist");
        assert_eq!(inv.command(), "cargo run -p gunbc-gist");
    }

    #[test]
    fn test_binary_in_package() {
        let inv = CargoInvocation::in_package("gunbc-ci", "gunbc-dag");
        assert_eq!(inv.args(), "-p gunbc-dag --bin gunbc-ci");
        assert_eq!(inv.command(), "cargo run -p gunbc-dag --bin gunbc-ci");
    }
}
