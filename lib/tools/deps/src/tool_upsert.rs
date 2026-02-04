//! Integration between ToolDef and the deps tool's upsert pattern.
//!
//! This module bridges the gap between:
//! - `ToolDef` from `gunbc_ir::transport::tool` (declarative tool definitions)
//! - `Installer` from this crate (platform-specific install commands)
//! - `UpsertBuilder` from `gunbc_ir::patterns` (Check → Create → Resolve)
//!
//! # Example
//!
//! ```ignore
//! use gunbc_deps::tool_upsert::*;
//! use gunbc_ir::transport::{GH_TOOL, default_platform_registry};
//!
//! let platform_registry = default_platform_registry();
//! let available_pms = platform_registry.available_pms("ubuntu");
//!
//! // Convert ToolDef to PlatformInstall for the Installer
//! if let Some(platform_install) = tool_to_platform_install(&GH_TOOL, &available_pms) {
//!     let installer = Installer::for_platform(Platform::detect());
//!     let cmd = installer.generate_install_cmd(&platform_install);
//! }
//! ```

use crate::installer::Installer;
use crate::manifest::PlatformInstall;
use gunbc_ir::transport::tool::{InstallInputs, InstallOption, ToolDef};
use std::collections::HashSet;

/// Convert a ToolDef's InstallInputs to a PlatformInstall for the Installer.
///
/// This bridges the gap between the declarative `InstallInputs` and the
/// legacy `PlatformInstall` that the `Installer` expects.
pub fn install_inputs_to_platform_install(
    pm_id: &str,
    inputs: &InstallInputs,
) -> PlatformInstall {
    let packages = inputs
        .packages
        .map(|pkgs| pkgs.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    // For cargo installs, the crate_name goes in packages
    let packages = if let Some(crate_name) = inputs.crate_name {
        vec![crate_name.to_string()]
    } else {
        packages
    };

    PlatformInstall {
        method: pm_id.to_string(),
        packages,
        script: None, // No script support in new model
        url: None,
    }
}

/// Find the best install option for a tool given available package managers.
///
/// Returns the first matching install option, or None if no PM matches.
pub fn find_install_option<'a>(
    tool: &'a ToolDef,
    available_pms: &HashSet<&str>,
) -> Option<&'a InstallOption> {
    tool.install_options
        .iter()
        .find(|opt| available_pms.contains(opt.via))
}

/// Convert a ToolDef to a PlatformInstall for the given available PMs.
///
/// Returns None if the tool cannot be installed via any available PM.
pub fn tool_to_platform_install(
    tool: &ToolDef,
    available_pms: &HashSet<&str>,
) -> Option<PlatformInstall> {
    find_install_option(tool, available_pms)
        .map(|opt| install_inputs_to_platform_install(opt.via, &opt.inputs))
}

/// Generate an install command for a tool.
///
/// This is a convenience function that combines:
/// 1. Finding the right install option
/// 2. Converting to PlatformInstall
/// 3. Using the Installer to generate the command
pub fn generate_tool_install_cmd(
    tool: &ToolDef,
    available_pms: &HashSet<&str>,
    installer: &Installer,
) -> Result<String, String> {
    let platform_install = tool_to_platform_install(tool, available_pms)
        .ok_or_else(|| format!("no install option for {} with available PMs: {:?}", tool.id, available_pms))?;

    installer.generate_install_cmd(&platform_install)
}

/// Generate an idempotent install script for a tool.
///
/// The script checks if the tool is installed before attempting installation.
pub fn generate_tool_idempotent_script(
    tool: &ToolDef,
    available_pms: &HashSet<&str>,
    installer: &Installer,
) -> Result<String, String> {
    let install_cmd = generate_tool_install_cmd(tool, available_pms, installer)?;
    Ok(installer.generate_idempotent_script(tool.id, tool.verify, &install_cmd))
}

// ============================================================================
// deps.toml Generation
// ============================================================================

/// Generate a deps.toml entry for a single tool.
///
/// The output format matches the existing deps.toml schema:
/// ```toml
/// [[dependency]]
/// name = "gh"
/// verify = "gh --version"
///
/// [dependency.install.linux]
/// method = "apt"
/// packages = ["gh"]
///
/// [dependency.install.macos]
/// method = "brew"
/// packages = ["gh"]
/// ```
pub fn generate_tool_deps_entry(tool: &ToolDef) -> String {
    let mut entry = format!(
        r#"[[dependency]]
name = "{}"
verify = "{}"
"#,
        tool.id, tool.verify
    );

    // Add depends_on if present
    if !tool.depends_on.is_empty() {
        let deps: Vec<_> = tool.depends_on.iter().map(|d| format!("\"{}\"", d)).collect();
        entry.push_str(&format!("depends_on = [{}]\n", deps.join(", ")));
    }

    // Map PM -> platform for install sections
    // This is a simplified mapping; a more complete implementation would use PlatformRegistry
    let pm_to_platform = |pm: &str| -> Option<&str> {
        match pm {
            "apt" => Some("linux"),
            "brew" => Some("macos"),
            "apk" => Some("alpine"),
            "cargo" => Some("any"), // cargo works on all platforms with rust
            _ => None,
        }
    };

    for opt in tool.install_options {
        if let Some(platform) = pm_to_platform(opt.via) {
            entry.push_str(&format!(
                r#"
[dependency.install.{}]
method = "{}"
"#,
                platform, opt.via
            ));

            // Add packages or crate_name
            if let Some(packages) = opt.inputs.packages {
                let pkg_strs: Vec<_> = packages.iter().map(|p| format!("\"{}\"", p)).collect();
                entry.push_str(&format!("packages = [{}]\n", pkg_strs.join(", ")));
            } else if let Some(crate_name) = opt.inputs.crate_name {
                entry.push_str(&format!("packages = [\"{}\"]\n", crate_name));
                if let Some(git_url) = opt.inputs.git_url {
                    entry.push_str(&format!("git = \"{}\"\n", git_url));
                }
            }
        }
    }

    entry
}

/// Generate a complete deps.toml from a tool registry.
///
/// Tools are sorted alphabetically by ID for deterministic output.
pub fn generate_deps_toml<'a>(
    tools: impl IntoIterator<Item = &'a ToolDef>,
    header_comment: Option<&str>,
) -> String {
    let mut output = String::new();

    // Add header comment
    if let Some(comment) = header_comment {
        output.push_str(comment);
        output.push_str("\n\n");
    } else {
        output.push_str("# Generated from tool registry - do not edit manually\n");
        output.push_str("# Regenerate with: cargo run -p gunbc-deps --bin gen-deps-toml\n\n");
    }

    // Sort tools by ID for deterministic output
    let mut tools: Vec<_> = tools.into_iter().collect();
    tools.sort_by_key(|t| t.id);

    // Filter out package managers (they have empty install_options and are base deps)
    let tools: Vec<_> = tools
        .into_iter()
        .filter(|t| !t.install_options.is_empty())
        .collect();

    for (i, tool) in tools.iter().enumerate() {
        output.push_str(&generate_tool_deps_entry(tool));
        if i < tools.len() - 1 {
            output.push('\n');
        }
    }

    output
}

/// Generate deps.toml from the default tool registry.
pub fn generate_deps_toml_from_registry() -> String {
    use gunbc_ir::transport::tool::default_tool_registry;

    let registry = default_tool_registry();
    generate_deps_toml(registry.all(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;
    use gunbc_ir::transport::{GIT, GH_TOOL};

    #[test]
    fn test_install_inputs_to_platform_install_packages() {
        let inputs = InstallInputs::packages(&["gh"]);
        let platform_install = install_inputs_to_platform_install("apt", &inputs);

        assert_eq!(platform_install.method, "apt");
        assert_eq!(platform_install.packages, vec!["gh"]);
    }

    #[test]
    fn test_install_inputs_to_platform_install_cargo() {
        let inputs = InstallInputs::crate_install("cargo-nextest");
        let platform_install = install_inputs_to_platform_install("cargo", &inputs);

        assert_eq!(platform_install.method, "cargo");
        assert_eq!(platform_install.packages, vec!["cargo-nextest"]);
    }

    #[test]
    fn test_find_install_option() {
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        let opt = find_install_option(&GIT, &available);
        assert!(opt.is_some());
        assert_eq!(opt.unwrap().via, "apt");
    }

    #[test]
    fn test_find_install_option_no_match() {
        let available: HashSet<&str> = ["pacman"].into_iter().collect();

        let opt = find_install_option(&GIT, &available);
        assert!(opt.is_none());
    }

    #[test]
    fn test_tool_to_platform_install() {
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        let platform_install = tool_to_platform_install(&GIT, &available);
        assert!(platform_install.is_some());

        let pi = platform_install.unwrap();
        assert_eq!(pi.method, "apt");
        assert_eq!(pi.packages, vec!["git"]);
    }

    #[test]
    fn test_generate_tool_install_cmd_git_apt() {
        let available: HashSet<&str> = ["apt"].into_iter().collect();
        let installer = Installer::for_platform(Platform::Linux);

        let cmd = generate_tool_install_cmd(&GIT, &available, &installer);
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        assert!(cmd.contains("apt-get"));
        assert!(cmd.contains("git"));
    }

    #[test]
    fn test_generate_tool_install_cmd_git_brew() {
        let available: HashSet<&str> = ["brew"].into_iter().collect();
        let installer = Installer::for_platform(Platform::Macos);

        let cmd = generate_tool_install_cmd(&GIT, &available, &installer);
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        assert!(cmd.contains("brew install"));
        assert!(cmd.contains("git"));
    }

    #[test]
    fn test_generate_tool_install_cmd_gh() {
        let available: HashSet<&str> = ["apt"].into_iter().collect();
        let installer = Installer::for_platform(Platform::Linux);

        let cmd = generate_tool_install_cmd(&GH_TOOL, &available, &installer);
        assert!(cmd.is_ok());

        let cmd = cmd.unwrap();
        assert!(cmd.contains("apt-get"));
        assert!(cmd.contains("gh"));
    }

    #[test]
    fn test_generate_tool_install_cmd_no_pm() {
        let available: HashSet<&str> = HashSet::new();
        let installer = Installer::for_platform(Platform::Linux);

        let cmd = generate_tool_install_cmd(&GIT, &available, &installer);
        assert!(cmd.is_err());
    }

    #[test]
    fn test_generate_tool_idempotent_script() {
        let available: HashSet<&str> = ["apt"].into_iter().collect();
        let installer = Installer::for_platform(Platform::Linux);

        let script = generate_tool_idempotent_script(&GIT, &available, &installer);
        assert!(script.is_ok());

        let script = script.unwrap();
        assert!(script.contains("git --version")); // verify command
        assert!(script.contains("apt-get")); // install command
        assert!(script.contains("already installed")); // idempotency message
    }

    #[test]
    fn test_generate_tool_deps_entry_git() {
        let entry = super::generate_tool_deps_entry(&GIT);

        assert!(entry.contains("name = \"git\""));
        assert!(entry.contains("verify = \"git --version\""));
        assert!(entry.contains("[dependency.install.linux]"));
        assert!(entry.contains("method = \"apt\""));
        assert!(entry.contains("packages = [\"git\"]"));
        assert!(entry.contains("[dependency.install.macos]"));
        assert!(entry.contains("method = \"brew\""));
    }

    #[test]
    fn test_generate_tool_deps_entry_gh() {
        let entry = super::generate_tool_deps_entry(&GH_TOOL);

        assert!(entry.contains("name = \"gh\""));
        assert!(entry.contains("verify = \"gh --version\""));
        assert!(entry.contains("[dependency.install.linux]"));
        assert!(entry.contains("[dependency.install.macos]"));
    }

    #[test]
    fn test_generate_deps_toml() {
        let tools = [&GIT, &GH_TOOL];
        let toml = super::generate_deps_toml(tools, Some("# Test deps.toml"));

        assert!(toml.starts_with("# Test deps.toml"));
        // gh comes before git alphabetically
        let gh_pos = toml.find("name = \"gh\"");
        let git_pos = toml.find("name = \"git\"");
        assert!(gh_pos.is_some());
        assert!(git_pos.is_some());
        assert!(gh_pos.unwrap() < git_pos.unwrap());
    }

    #[test]
    fn test_generate_deps_toml_from_registry() {
        let toml = super::generate_deps_toml_from_registry();

        // Should contain gh and git (they have install options)
        assert!(toml.contains("name = \"gh\""));
        assert!(toml.contains("name = \"git\""));

        // Should NOT contain apt or brew (they are base PMs with empty install_options)
        assert!(!toml.contains("name = \"apt\""));
        assert!(!toml.contains("name = \"brew\""));
    }
}
