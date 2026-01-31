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
//!
//! # Name Composition
//!
//! All binary names follow the `{PREFIX}-{component}` pattern (e.g., `gunbc-ci`).
//! Use [`CargoInvocation::standalone`] and [`CargoInvocation::composed`] to
//! construct invocations from component names, avoiding hardcoded full names.

/// Workspace binary name prefix. All gunbc binaries follow `{PREFIX}-{component}`.
pub const PREFIX: &str = "gunbc";

/// Compose a full binary name from a component: `{PREFIX}-{component}`.
///
/// # Examples
///
/// ```
/// use gunbc_ir::cargo::name;
/// assert_eq!(name("ci"), "gunbc-ci");
/// assert_eq!(name("gist"), "gunbc-gist");
/// ```
pub fn name(component: &str) -> String {
    format!("{PREFIX}-{component}")
}

/// Describes how to invoke a cargo workspace binary.
///
/// This type is the canonical way to represent "run this binary from
/// the workspace" across all renderers (Makefile, GitHub Actions, GitLab CI).
///
/// Prefer [`CargoInvocation::standalone`] and [`CargoInvocation::composed`]
/// over [`CargoInvocation::new`] and [`CargoInvocation::in_package`] to
/// ensure names are composed from components rather than hardcoded.
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
    ///
    /// Prefer [`Self::standalone`] which composes the name from a component.
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            package: None,
        }
    }

    /// Create an invocation for a binary inside another package.
    ///
    /// Produces: `cargo run -p <package> --bin <binary>`
    ///
    /// Prefer [`Self::composed`] which composes both names from components.
    pub fn in_package(binary: impl Into<String>, package: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            package: Some(package.into()),
        }
    }

    /// Create an invocation for a standalone binary from its component name.
    ///
    /// The binary name is composed as `{PREFIX}-{component}`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gunbc_ir::CargoInvocation;
    /// let inv = CargoInvocation::standalone("gist");
    /// assert_eq!(inv.binary, "gunbc-gist");
    /// assert_eq!(inv.command(), "cargo run -p gunbc-gist");
    /// ```
    pub fn standalone(component: &str) -> Self {
        Self::new(name(component))
    }

    /// Create an invocation for a binary inside another package, both
    /// composed from component names.
    ///
    /// The binary is `{PREFIX}-{binary_component}`, the package is
    /// `{PREFIX}-{package_component}`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gunbc_ir::CargoInvocation;
    /// let inv = CargoInvocation::composed("ci", "dag");
    /// assert_eq!(inv.binary, "gunbc-ci");
    /// assert_eq!(inv.package, Some("gunbc-dag".to_string()));
    /// assert_eq!(inv.command(), "cargo run -p gunbc-dag --bin gunbc-ci");
    /// ```
    pub fn composed(binary_component: &str, package_component: &str) -> Self {
        Self::in_package(name(binary_component), name(package_component))
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

    /// Get the cargo run command as individual string parts.
    ///
    /// Returns `["cargo", "run", "-p", package, "--bin", binary]` or
    /// `["cargo", "run", "-p", binary]`.
    pub fn command_parts(&self) -> Vec<String> {
        let mut parts = vec!["cargo".to_string(), "run".to_string()];
        match &self.package {
            Some(pkg) => {
                parts.push("-p".to_string());
                parts.push(pkg.clone());
                parts.push("--bin".to_string());
                parts.push(self.binary.clone());
            }
            None => {
                parts.push("-p".to_string());
                parts.push(self.binary.clone());
            }
        }
        parts
    }

    /// Get the cargo run command parts followed by additional arguments.
    ///
    /// Convenience for building command vectors like
    /// `["cargo", "run", "-p", "gunbc-codegen", "--release", "--", "codegen"]`.
    pub fn run_with_args(&self, extra: &[&str]) -> Vec<String> {
        let mut parts = self.command_parts();
        parts.extend(extra.iter().map(|s| s.to_string()));
        parts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_composition() {
        assert_eq!(name("ci"), "gunbc-ci");
        assert_eq!(name("gist"), "gunbc-gist");
        assert_eq!(name("dag"), "gunbc-dag");
    }

    #[test]
    fn test_standalone() {
        let inv = CargoInvocation::standalone("gist");
        assert_eq!(inv.binary, "gunbc-gist");
        assert_eq!(inv.package, None);
        assert_eq!(inv.args(), "-p gunbc-gist");
        assert_eq!(inv.command(), "cargo run -p gunbc-gist");
    }

    #[test]
    fn test_composed() {
        let inv = CargoInvocation::composed("ci", "dag");
        assert_eq!(inv.binary, "gunbc-ci");
        assert_eq!(inv.package, Some("gunbc-dag".to_string()));
        assert_eq!(inv.args(), "-p gunbc-dag --bin gunbc-ci");
        assert_eq!(inv.command(), "cargo run -p gunbc-dag --bin gunbc-ci");
    }

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

    #[test]
    fn test_command_parts_standalone() {
        let inv = CargoInvocation::standalone("gist");
        assert_eq!(
            inv.command_parts(),
            vec!["cargo", "run", "-p", "gunbc-gist"]
        );
    }

    #[test]
    fn test_command_parts_composed() {
        let inv = CargoInvocation::composed("ci", "dag");
        assert_eq!(
            inv.command_parts(),
            vec!["cargo", "run", "-p", "gunbc-dag", "--bin", "gunbc-ci"]
        );
    }

    #[test]
    fn test_run_with_args() {
        let inv = CargoInvocation::standalone("codegen");
        assert_eq!(
            inv.run_with_args(&["--release", "--", "codegen"]),
            vec!["cargo", "run", "-p", "gunbc-codegen", "--release", "--", "codegen"]
        );
    }
}
