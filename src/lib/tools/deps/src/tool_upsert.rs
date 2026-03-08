//! Integration between ToolDef and the deps tool's upsert pattern.
//!
//! This module bridges the gap between:
//! - `ToolDef` from `gunbc_ir::transport::tool` (declarative tool definitions)
//! - `Installer` from this crate (platform-specific install commands)
//! - `UpsertBuilder` from `gunbc_ir::patterns` (Check → Create → Resolve)
//!
//! # Example
//!
//! ```text
//! use gunbc_deps::tool_upsert::*;
//! use gunbc_ir::transport::{GH_TOOL, default_platform_registry};
//!
//! let platform_registry = default_platform_registry();
//! let available_pms = platform_registry.available_pms("ubuntu");
//!
//! // Convert ToolDef to PlatformInstall for the Installer (strict parse path)
//! if let Ok(Some(platform_install)) = tool_to_platform_install(&GH_TOOL, &available_pms) {
//!     let installer = Installer::for_platform(Platform::detect());
//!     let cmd = installer.generate_install_cmd(&platform_install);
//! }
//! ```

use crate::installer::Installer;
use crate::manifest::PlatformInstall;
use crate::package_manager::PackageManagerId;
use gunbc_ir::transport::tool::{InstallInputs, InstallOption, ToolDef};
use gunbc_ir::Os;
use std::collections::HashSet;
use std::fmt::Write;

/// Explicit install-option selection policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallSelectionPolicy {
    priority: Vec<PackageManagerId>,
}

impl InstallSelectionPolicy {
    /// Deterministic manager-priority policy.
    pub fn deterministic_default() -> Self {
        Self {
            priority: vec![
                PackageManagerId::Apt,
                PackageManagerId::Apk,
                PackageManagerId::Brew,
                PackageManagerId::Cargo,
                PackageManagerId::Script,
                PackageManagerId::GithubRelease,
            ],
        }
    }

    fn rank(&self, pm: PackageManagerId) -> usize {
        self.priority
            .iter()
            .position(|candidate| *candidate == pm)
            .unwrap_or(usize::MAX)
    }
}

impl Default for InstallSelectionPolicy {
    fn default() -> Self {
        Self::deterministic_default()
    }
}

/// Convert a ToolDef's InstallInputs to a PlatformInstall for the Installer.
///
/// This bridges the gap between the declarative `InstallInputs` and the
/// legacy `PlatformInstall` that the `Installer` expects.
pub fn install_inputs_to_platform_install(
    pm_id: PackageManagerId,
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
        script: None, // ToolDef install inputs do not currently model script bodies.
        url: inputs.git_url.map(|value| value.to_string()),
    }
}

/// Find the best install option for a tool given available package managers.
///
/// Policy is explicit + deterministic (not declaration-order dependent).
pub fn find_install_option<'a>(
    tool: &'a ToolDef,
    available_pms: &HashSet<&str>,
) -> Result<Option<&'a InstallOption>, String> {
    find_install_option_with_policy(tool, available_pms, &InstallSelectionPolicy::default())
}

/// Find best install option under explicit policy.
pub fn find_install_option_with_policy<'a>(
    tool: &'a ToolDef,
    available_pms: &HashSet<&str>,
    policy: &InstallSelectionPolicy,
) -> Result<Option<&'a InstallOption>, String> {
    let typed_available = available_pms
        .iter()
        .map(|raw| PackageManagerId::parse_strict(raw))
        .collect::<Result<HashSet<_>, _>>()?;

    let mut candidates = tool
        .install_options
        .iter()
        .enumerate()
        .map(|(idx, opt)| PackageManagerId::parse_strict(opt.via).map(|pm| (idx, pm, opt)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(_, pm, _)| typed_available.contains(pm))
        .collect::<Vec<_>>();

    candidates.sort_by(|(left_idx, left_pm, _), (right_idx, right_pm, _)| {
        policy
            .rank(*left_pm)
            .cmp(&policy.rank(*right_pm))
            .then_with(|| left_pm.cmp(right_pm))
            .then(left_idx.cmp(right_idx))
    });
    Ok(candidates.first().map(|(_, _, opt)| *opt))
}

/// Convert a ToolDef to a PlatformInstall for the given available PMs.
///
/// Returns None if the tool cannot be installed via any available PM.
pub fn tool_to_platform_install(
    tool: &ToolDef,
    available_pms: &HashSet<&str>,
) -> Result<Option<PlatformInstall>, String> {
    let option = find_install_option(tool, available_pms)?;
    let Some(option) = option else {
        return Ok(None);
    };
    let pm = PackageManagerId::parse_strict(option.via)?;
    Ok(Some(install_inputs_to_platform_install(pm, &option.inputs)))
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
    let platform_install = tool_to_platform_install(tool, available_pms)?.ok_or_else(|| {
        format!(
            "no install option for {} with available PMs: {:?}",
            tool.id, available_pms
        )
    })?;

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
        let deps: Vec<_> = tool
            .depends_on
            .iter()
            .map(|d| format!("\"{}\"", d))
            .collect();
        writeln!(entry, "depends_on = [{}]", deps.join(", ")).unwrap();
    }

    // Map PM -> platform for install sections
    // This is a simplified mapping; a more complete implementation would use PlatformRegistry
    let pm_to_platform = |pm: &str| -> Option<String> {
        match pm {
            "apt" => Some(Os::Linux.as_token().to_string()),
            "brew" => Some(Os::Macos.as_token().to_string()),
            "apk" => Some(Os::Other("alpine".to_string()).as_token().to_string()),
            "cargo" => Some("any".to_string()), // cargo works on all platforms with rust
            _ => None,
        }
    };

    for opt in tool.install_options {
        if let Some(platform) = pm_to_platform(opt.via) {
            write!(
                entry,
                r#"
[dependency.install.{}]
method = "{}"
"#,
                platform.as_str(),
                opt.via
            )
            .unwrap();

            // Add packages or crate_name
            if let Some(packages) = opt.inputs.packages {
                let pkg_strs: Vec<_> = packages.iter().map(|p| format!("\"{}\"", p)).collect();
                writeln!(entry, "packages = [{}]", pkg_strs.join(", ")).unwrap();
            } else if let Some(crate_name) = opt.inputs.crate_name {
                writeln!(entry, "packages = [\"{}\"]", crate_name).unwrap();
                if let Some(git_url) = opt.inputs.git_url {
                    writeln!(entry, "git = \"{}\"", git_url).unwrap();
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
    use crate::package_manager::PackageManagerId;
    use crate::platform::Platform;
    use gunbc_ir::transport::{GH_TOOL, GIT};

    #[test]
    fn test_install_inputs_to_platform_install_packages() {
        let inputs = InstallInputs::packages(&["gh"]);
        let platform_install = install_inputs_to_platform_install(PackageManagerId::Apt, &inputs);

        assert_eq!(platform_install.method, "apt");
        assert_eq!(platform_install.packages, vec!["gh"]);
    }

    #[test]
    fn test_install_inputs_to_platform_install_cargo() {
        let inputs = InstallInputs::crate_install("cargo-nextest");
        let platform_install = install_inputs_to_platform_install(PackageManagerId::Cargo, &inputs);

        assert_eq!(platform_install.method, "cargo");
        assert_eq!(platform_install.packages, vec!["cargo-nextest"]);
    }

    #[test]
    fn test_find_install_option() {
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        let opt = find_install_option(&GIT, &available);
        assert!(opt.is_ok());
        assert_eq!(opt.unwrap().unwrap().via, "apt");
    }

    #[test]
    fn test_find_install_option_no_match() {
        let available: HashSet<&str> = ["pacman"].into_iter().collect();

        let opt = find_install_option(&GIT, &available);
        assert!(opt.is_err());
    }

    #[test]
    fn test_tool_to_platform_install() {
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        let platform_install = tool_to_platform_install(&GIT, &available);
        assert!(platform_install.is_ok());

        let pi = platform_install.unwrap().unwrap();
        assert_eq!(pi.method, "apt");
        assert_eq!(pi.packages, vec!["git"]);
    }

    #[test]
    fn test_selection_policy_is_deterministic_not_registry_order_dependent() {
        let available: HashSet<&str> = ["apt", "brew"].into_iter().collect();
        let policy = InstallSelectionPolicy::deterministic_default();
        let selected = find_install_option_with_policy(&GIT, &available, &policy)
            .expect("selection should succeed")
            .expect("expected option");
        assert_eq!(selected.via, "apt");
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
    fn test_generate_tool_install_cmd_fails_closed_for_unknown_pm_id() {
        static UNKNOWN_PM_TOOL: ToolDef = ToolDef {
            id: "unknown-pm-tool",
            command: "unknown",
            verify: "unknown --version",
            install_options: &[InstallOption {
                via: "nix",
                inputs: InstallInputs::packages(&["unknown"]),
            }],
            depends_on: &[],
        };
        let available: HashSet<&str> = ["nix"].into_iter().collect();
        let installer = Installer::for_platform(Platform::Linux);

        let result = generate_tool_install_cmd(&UNKNOWN_PM_TOOL, &available, &installer);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown package manager id"));
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
