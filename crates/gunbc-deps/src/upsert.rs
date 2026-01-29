//! Upsert pattern for idempotent tool installation.
//!
//! The upsert pattern has three phases:
//! 1. Check: Is the tool already installed?
//! 2. Create: If not, install it
//! 3. Resolve: Verify installation succeeded

use crate::installer::Installer;
use crate::manifest::Dependency;

/// Phase of the upsert operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertPhase {
    /// Check if resource exists (read-only).
    Check,
    /// Create resource if missing (idempotent).
    Create,
    /// Verify and return resolved handle (read-only).
    Resolve,
}

/// Result of an upsert operation.
#[derive(Debug, Clone)]
pub struct UpsertResult {
    /// Tool name.
    pub name: String,
    /// Whether the tool was already installed.
    pub was_installed: bool,
    /// Whether installation was attempted.
    pub install_attempted: bool,
    /// Whether the tool is now installed (after upsert).
    pub is_installed: bool,
    /// Any error message.
    pub error: Option<String>,
}

impl UpsertResult {
    /// Check if the upsert succeeded.
    pub fn is_ok(&self) -> bool {
        self.is_installed && self.error.is_none()
    }

    /// Create a result for an already-installed tool.
    pub fn already_installed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            was_installed: true,
            install_attempted: false,
            is_installed: true,
            error: None,
        }
    }

    /// Create a result for a newly-installed tool.
    pub fn newly_installed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            was_installed: false,
            install_attempted: true,
            is_installed: true,
            error: None,
        }
    }

    /// Create a result for a failed installation.
    pub fn failed(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            was_installed: false,
            install_attempted: true,
            is_installed: false,
            error: Some(error.into()),
        }
    }

    /// Create a result for a missing platform configuration.
    pub fn no_platform_config(name: impl Into<String>, platform: &str) -> Self {
        Self {
            name: name.into(),
            was_installed: false,
            install_attempted: false,
            is_installed: false,
            error: Some(format!("no install configuration for platform: {}", platform)),
        }
    }
}

/// Execute the upsert pattern for a dependency.
///
/// This is a dry-run version that generates the script without executing.
pub fn upsert_dry_run(
    installer: &Installer,
    dep: &Dependency,
) -> Result<(UpsertResult, String), String> {
    // Check phase
    let is_installed = installer.is_installed(&dep.verify);

    if is_installed {
        return Ok((
            UpsertResult::already_installed(&dep.name),
            format!("# {} is already installed\n", dep.name),
        ));
    }

    // Get platform install config
    let install_config = dep
        .install_for(installer.platform())
        .ok_or_else(|| format!("no install config for platform: {}", installer.platform()))?;

    // Generate install command
    let install_cmd = installer.generate_install_cmd(install_config)?;

    // Generate idempotent script
    let script = installer.generate_idempotent_script(&dep.name, &dep.verify, &install_cmd);

    Ok((
        UpsertResult {
            name: dep.name.clone(),
            was_installed: false,
            install_attempted: false, // Dry run
            is_installed: false,
            error: None,
        },
        script,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::DepsManifest;

    #[test]
    fn test_upsert_already_installed() {
        let installer = Installer::new();

        // Use 'echo' which is always available
        let manifest = DepsManifest::parse(
            r#"
[[dependency]]
name = "echo"
verify = "echo test"

[dependency.install.linux]
method = "script"
script = "echo 'noop'"

[dependency.install.macos]
method = "script"
script = "echo 'noop'"

[dependency.install.windows]
method = "script"
script = "echo 'noop'"
"#,
        )
        .unwrap();

        let dep = manifest.get("echo").unwrap();
        let (result, script) = upsert_dry_run(&installer, dep).unwrap();

        assert!(result.was_installed);
        assert!(script.contains("already installed"));
    }

    #[test]
    fn test_upsert_result_states() {
        let already = UpsertResult::already_installed("test");
        assert!(already.is_ok());
        assert!(already.was_installed);

        let newly = UpsertResult::newly_installed("test");
        assert!(newly.is_ok());
        assert!(!newly.was_installed);
        assert!(newly.install_attempted);

        let failed = UpsertResult::failed("test", "something went wrong");
        assert!(!failed.is_ok());
        assert!(failed.error.is_some());
    }
}
