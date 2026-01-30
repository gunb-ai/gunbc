//! GitHub CLI (gh) - source of truth for gh usage in gunbc.
//!
//! This module is the **authoritative source** for gh CLI in gunbc. It encodes:
//! - Tool metadata (version, docs, verify command)
//! - Platform-specific installation instructions
//! - Commands we depend on (contract documentation)
//! - Upsert-compatible interface
//!
//! Files like `deps.toml` can be **generated** from this data, but this module
//! is the source of truth.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::github::cli::*;
//!
//! // Check if gh is installed
//! if !is_gh_installed() {
//!     // Get install instructions
//!     for (platform, method) in gh_install_methods() {
//!         println!("{}: {:?}", platform, method);
//!     }
//! }
//!
//! // Build a shell request
//! let req = gh_cli_request(&["gist", "create", "-f", "test.md"]);
//! ```

use super::GH_CLI_MIN_VERSION;
use crate::transport::tool::{InstallInputs, InstallOption, ToolDef};
use crate::transport::ShellRequest;

// ============================================================================
// Tool Definition (New - uses tool module)
// ============================================================================

/// GitHub CLI tool definition using the unified ToolDef type.
///
/// This is the source of truth for gh CLI installation. It integrates with
/// the tool registry for satisfiability checks and deps.toml generation.
pub static GH_TOOL: ToolDef = ToolDef {
    id: "gh",
    command: "gh",
    verify: "gh --version",
    install_options: &[
        InstallOption {
            via: "apt",
            inputs: InstallInputs::packages(&["gh"]),
        },
        InstallOption {
            via: "brew",
            inputs: InstallInputs::packages(&["gh"]),
        },
    ],
    depends_on: &[],
};

// ============================================================================
// Legacy Tool Definition (kept for backward compatibility)
// ============================================================================

/// GitHub CLI tool definition (legacy).
///
/// Contains all metadata needed to identify, verify, and document the gh CLI.
/// For new code, prefer using `GH_TOOL` which integrates with the tool registry.
#[derive(Debug, Clone)]
pub struct GitHubCLI {
    /// Unique identifier for this tool (used in deps.toml, etc.)
    pub id: &'static str,
    /// Command to invoke (usually "gh")
    pub command: &'static str,
    /// Minimum version required for our usage
    pub min_version: &'static str,
    /// Command to verify installation
    pub verify_command: &'static str,
    /// Documentation URL
    pub docs_url: &'static str,
}

/// The gh CLI tool definition (legacy).
///
/// For new code, prefer using `GH_TOOL` which integrates with the tool registry.
pub const GH_CLI: GitHubCLI = GitHubCLI {
    id: "gh",
    command: "gh",
    min_version: GH_CLI_MIN_VERSION,
    verify_command: "gh --version",
    docs_url: "https://cli.github.com/manual/",
};

// ============================================================================
// Platform Installation (DAG source of truth)
// ============================================================================

/// Platform-specific installation method.
///
/// Encodes how to install a CLI tool on different platforms.
/// This is the source of truth; deps.toml can be generated from this.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallMethod {
    /// Install via apt package manager (Debian/Ubuntu)
    Apt { packages: &'static [&'static str] },
    /// Install via Homebrew (macOS/Linux)
    Brew { packages: &'static [&'static str] },
    /// Install via script (curl | sh pattern)
    Script { script: &'static str },
    /// Install via cargo (Rust tools)
    Cargo { packages: &'static [&'static str] },
    /// Install via GitHub release download
    GithubRelease { url_template: &'static str },
}

/// Installation instructions for gh CLI per platform.
///
/// This is the authoritative source; deps.toml entries are generated from this.
pub fn gh_install_methods() -> Vec<(&'static str, InstallMethod)> {
    vec![
        ("linux", InstallMethod::Apt { packages: &["gh"] }),
        ("macos", InstallMethod::Brew { packages: &["gh"] }),
        // Windows could use: winget install GitHub.cli
        // or: scoop install gh
    ]
}

// ============================================================================
// Commands We Depend On (contract documentation)
// ============================================================================

/// A gh CLI command that our integration depends on.
///
/// This documents which gh subcommands we use and whether they're required
/// for our integration to function.
#[derive(Debug, Clone)]
pub struct GHCommand {
    /// Subcommand path (e.g., &["gist", "create"])
    pub subcommand: &'static [&'static str],
    /// Human-readable description of what we use this for
    pub description: &'static str,
    /// Whether this command is required (true) or optional (false)
    pub required: bool,
}

/// Commands our integration uses from gh CLI.
///
/// This serves as contract documentation - if gh CLI changes these commands,
/// our integration may break.
pub fn gh_cli_commands() -> Vec<GHCommand> {
    vec![
        GHCommand {
            subcommand: &["gist", "create"],
            description: "Create a new gist",
            required: true,
        },
        GHCommand {
            subcommand: &["gist", "list"],
            description: "List user's gists",
            required: false,
        },
        GHCommand {
            subcommand: &["auth", "status"],
            description: "Check authentication status",
            required: false,
        },
        GHCommand {
            subcommand: &["api"],
            description: "Make raw API requests",
            required: false,
        },
    ]
}

// ============================================================================
// Request Builder
// ============================================================================

/// Build a shell request using gh CLI.
///
/// Creates a ShellRequest configured to invoke the gh CLI with the given subcommand.
///
/// # Arguments
///
/// * `subcommand` - The gh subcommand and arguments (e.g., &["gist", "create", "-f", "file.md"])
///
/// # Example
///
/// ```ignore
/// let req = gh_cli_request(&["gist", "create", "-f", "test.md", "-"])
///     .stdin("# My Gist Content");
/// ```
pub fn gh_cli_request(subcommand: &[&str]) -> ShellRequest {
    ShellRequest::new(GH_CLI.command).args(subcommand.iter().map(|s| s.to_string()))
}

/// Build a shell request for gh auth status.
pub fn gh_auth_status_request() -> ShellRequest {
    gh_cli_request(&["auth", "status"])
}

/// Build a shell request for gh api (raw API call).
pub fn gh_api_request(endpoint: &str) -> ShellRequest {
    gh_cli_request(&["api", endpoint])
}

// ============================================================================
// Upsert Interface
// ============================================================================

// Note: These functions use Command::new directly because they ARE the gh CLI
// abstraction. For new code, consider using cli::GH with node.requires().
// Once migration is complete, these should delegate to CliToolOp.

/// Check if gh CLI is installed.
///
/// This is the "Check" phase of the upsert pattern.
/// Returns true if `gh --version` exits successfully.
#[allow(clippy::disallowed_methods)]
pub fn is_gh_installed() -> bool {
    std::process::Command::new("gh")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the installed gh CLI version, if available.
///
/// Returns None if gh is not installed or version cannot be parsed.
#[allow(clippy::disallowed_methods)]
pub fn gh_installed_version() -> Option<String> {
    let output = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output format: "gh version 2.40.0 (2024-01-01)"
    stdout
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(2))
        .map(|v| v.to_string())
}

/// Check if gh CLI is authenticated.
///
/// Runs `gh auth status` and checks for success.
#[allow(clippy::disallowed_methods)]
pub fn is_gh_authenticated() -> bool {
    std::process::Command::new("gh")
        .args(["auth", "status"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ============================================================================
// deps.toml Generation
// ============================================================================

/// Generate a deps.toml entry from this source of truth.
///
/// This allows generating the manifest file from DAG data.
/// The code is authoritative; deps.toml is a derived artifact.
pub fn to_deps_toml_entry() -> String {
    let mut entry = format!(
        r#"[[dependency]]
name = "{}"
verify = "{}"
"#,
        GH_CLI.id, GH_CLI.verify_command
    );

    for (platform, method) in gh_install_methods() {
        match method {
            InstallMethod::Apt { packages } => {
                let packages_str: Vec<_> = packages.iter().map(|p| format!("\"{}\"", p)).collect();
                entry.push_str(&format!(
                    r#"
[dependency.install.{}]
method = "apt"
packages = [{}]
"#,
                    platform,
                    packages_str.join(", ")
                ));
            }
            InstallMethod::Brew { packages } => {
                let packages_str: Vec<_> = packages.iter().map(|p| format!("\"{}\"", p)).collect();
                entry.push_str(&format!(
                    r#"
[dependency.install.{}]
method = "brew"
packages = [{}]
"#,
                    platform,
                    packages_str.join(", ")
                ));
            }
            InstallMethod::Script { script } => {
                entry.push_str(&format!(
                    r#"
[dependency.install.{}]
method = "script"
script = {:?}
"#,
                    platform, script
                ));
            }
            InstallMethod::Cargo { packages } => {
                let packages_str: Vec<_> = packages.iter().map(|p| format!("\"{}\"", p)).collect();
                entry.push_str(&format!(
                    r#"
[dependency.install.{}]
method = "cargo"
packages = [{}]
"#,
                    platform,
                    packages_str.join(", ")
                ));
            }
            InstallMethod::GithubRelease { url_template } => {
                entry.push_str(&format!(
                    r#"
[dependency.install.{}]
method = "github_release"
url = {:?}
"#,
                    platform, url_template
                ));
            }
        }
    }
    entry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gh_cli_definition() {
        assert_eq!(GH_CLI.id, "gh");
        assert_eq!(GH_CLI.command, "gh");
        assert_eq!(GH_CLI.verify_command, "gh --version");
        assert!(!GH_CLI.min_version.is_empty());
    }

    #[test]
    fn test_gh_install_methods() {
        let methods = gh_install_methods();

        // Should have at least linux and macos
        assert!(methods.len() >= 2);

        // Check linux uses apt
        let linux = methods.iter().find(|(p, _)| *p == "linux");
        assert!(linux.is_some());
        assert!(matches!(linux.unwrap().1, InstallMethod::Apt { .. }));

        // Check macos uses brew
        let macos = methods.iter().find(|(p, _)| *p == "macos");
        assert!(macos.is_some());
        assert!(matches!(macos.unwrap().1, InstallMethod::Brew { .. }));
    }

    #[test]
    fn test_gh_cli_commands() {
        let commands = gh_cli_commands();

        // Should have at least gist create
        assert!(!commands.is_empty());

        // Find gist create and verify it's required
        let gist_create = commands
            .iter()
            .find(|c| c.subcommand == ["gist", "create"]);
        assert!(gist_create.is_some());
        assert!(gist_create.unwrap().required);
    }

    #[test]
    fn test_gh_cli_request() {
        let req = gh_cli_request(&["gist", "create", "-f", "test.md"]);

        assert_eq!(req.command, "gh");
        assert_eq!(req.args, vec!["gist", "create", "-f", "test.md"]);
    }

    #[test]
    fn test_gh_cli_request_with_stdin() {
        let req = gh_cli_request(&["gist", "create", "-f", "test.md", "-"]).stdin("# Content");

        assert_eq!(req.stdin, Some("# Content".to_string()));
    }

    #[test]
    fn test_to_deps_toml_entry() {
        let entry = to_deps_toml_entry();

        // Should contain the tool name and verify command
        assert!(entry.contains("name = \"gh\""));
        assert!(entry.contains("verify = \"gh --version\""));

        // Should have linux and macos install sections
        assert!(entry.contains("[dependency.install.linux]"));
        assert!(entry.contains("[dependency.install.macos]"));
        assert!(entry.contains("method = \"apt\""));
        assert!(entry.contains("method = \"brew\""));
    }

    #[test]
    fn test_install_method_variants() {
        // Just ensure all variants are constructable
        let _apt = InstallMethod::Apt {
            packages: &["gh"],
        };
        let _brew = InstallMethod::Brew {
            packages: &["gh"],
        };
        let _script = InstallMethod::Script {
            script: "curl | sh",
        };
        let _cargo = InstallMethod::Cargo {
            packages: &["ripgrep"],
        };
        let _release = InstallMethod::GithubRelease {
            url_template: "https://github.com/...",
        };
    }

    #[test]
    fn test_gh_tool_definition() {
        // Test the new unified ToolDef
        assert_eq!(GH_TOOL.id, "gh");
        assert_eq!(GH_TOOL.command, "gh");
        assert_eq!(GH_TOOL.verify, "gh --version");
        assert!(GH_TOOL.depends_on.is_empty());

        // Should have apt and brew install options
        let apt_opt = GH_TOOL.install_options.iter().find(|o| o.via == "apt");
        assert!(apt_opt.is_some());
        assert_eq!(apt_opt.unwrap().inputs.packages, Some(&["gh"][..]));

        let brew_opt = GH_TOOL.install_options.iter().find(|o| o.via == "brew");
        assert!(brew_opt.is_some());
        assert_eq!(brew_opt.unwrap().inputs.packages, Some(&["gh"][..]));
    }

    #[test]
    fn test_gh_tool_satisfiable() {
        use crate::transport::tool::{is_satisfiable, default_tool_registry};
        use std::collections::HashSet;

        let registry = default_tool_registry();
        
        // gh should be satisfiable via apt (ubuntu)
        let apt_available: HashSet<&str> = ["apt"].into_iter().collect();
        assert!(is_satisfiable(&GH_TOOL, &apt_available, &registry).is_ok());

        // gh should be satisfiable via brew (macos)
        let brew_available: HashSet<&str> = ["brew"].into_iter().collect();
        assert!(is_satisfiable(&GH_TOOL, &brew_available, &registry).is_ok());
    }
}
