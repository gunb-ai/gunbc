//! Dependency manifest parsing and configuration.

use crate::platform::Platform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ============================================================================
// Manifest Configuration (centralized constants)
// ============================================================================

/// Default manifest filename.
pub const DEFAULT_MANIFEST_FILENAME: &str = "deps.toml";

/// Manifest configuration - centralized location for manifest-related constants.
///
/// Use this instead of hardcoding "deps.toml" throughout the codebase.
#[derive(Debug, Clone)]
pub struct ManifestConfig {
    /// The manifest filename (default: "deps.toml")
    pub filename: &'static str,
}

impl Default for ManifestConfig {
    fn default() -> Self {
        Self {
            filename: DEFAULT_MANIFEST_FILENAME,
        }
    }
}

impl ManifestConfig {
    /// Get the default manifest configuration.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get the manifest filename.
    pub fn filename(&self) -> &str {
        self.filename
    }
}

/// Global manifest config instance.
pub static MANIFEST_CONFIG: ManifestConfig = ManifestConfig {
    filename: DEFAULT_MANIFEST_FILENAME,
};

// ============================================================================
// Manifest Data Structures
// ============================================================================

/// The deps.toml manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepsManifest {
    #[serde(default)]
    pub dependency: Vec<Dependency>,
}

impl DepsManifest {
    /// Load a manifest from a file.
    #[allow(clippy::disallowed_methods)] // Manifest loading needs direct fs access
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("failed to read manifest: {}", e))?;
        Self::parse(&content)
    }

    /// Parse a manifest from a string.
    pub fn parse(content: &str) -> Result<Self, String> {
        toml::from_str(content).map_err(|e| format!("failed to parse manifest: {}", e))
    }

    /// Get a dependency by name.
    pub fn get(&self, name: &str) -> Option<&Dependency> {
        self.dependency.iter().find(|d| d.name == name)
    }

    /// Get all dependencies.
    pub fn all(&self) -> &[Dependency] {
        &self.dependency
    }
}

/// A tool dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Tool name.
    pub name: String,
    /// Command to verify the tool is installed.
    pub verify: String,
    /// Platform-specific installation methods.
    #[serde(default)]
    pub install: HashMap<String, PlatformInstall>,
}

impl Dependency {
    /// Get installation method for the current platform.
    pub fn install_for(&self, platform: Platform) -> Option<&PlatformInstall> {
        self.install.get(platform.name())
    }
}

/// Platform-specific installation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInstall {
    /// Installation method (apt, brew, cargo, script, etc.)
    pub method: String,
    /// Packages to install (for package managers).
    #[serde(default)]
    pub packages: Vec<String>,
    /// Script to run (for script method).
    #[serde(default)]
    pub script: Option<String>,
    /// URL template for download (for github_release method).
    #[serde(default)]
    pub url: Option<String>,
}


#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_MANIFEST: &str = r#"
[[dependency]]
name = "gh"
verify = "gh --version"

[dependency.install.linux]
method = "apt"
packages = ["gh"]

[dependency.install.macos]
method = "brew"
packages = ["gh"]

[[dependency]]
name = "cargo"
verify = "cargo --version"

[dependency.install.linux]
method = "script"
script = "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

[dependency.install.macos]
method = "brew"
packages = ["rustup"]
"#;

    #[test]
    fn test_parse_manifest() {
        let manifest = DepsManifest::parse(EXAMPLE_MANIFEST).unwrap();

        assert_eq!(manifest.dependency.len(), 2);

        let gh = manifest.get("gh").unwrap();
        assert_eq!(gh.verify, "gh --version");

        let linux_install = gh.install_for(Platform::Linux).unwrap();
        assert_eq!(linux_install.method, "apt");
        assert_eq!(linux_install.packages, vec!["gh"]);
    }

    #[test]
    fn test_script_install() {
        let manifest = DepsManifest::parse(EXAMPLE_MANIFEST).unwrap();

        let cargo = manifest.get("cargo").unwrap();
        let linux_install = cargo.install_for(Platform::Linux).unwrap();

        assert_eq!(linux_install.method, "script");
        assert!(linux_install.script.is_some());
        assert!(linux_install.script.as_ref().unwrap().contains("rustup.rs"));
    }
}
