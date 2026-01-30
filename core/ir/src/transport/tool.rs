//! CLI Tool definitions and registry.
//!
//! This module provides a unified way to define CLI tools and their installation
//! options across different package managers. Tools are responsible for defining
//! how they can be installed - package managers provide the upsert mechanics.
//!
//! # Architecture
//!
//! ```text
//! Platform (ubuntu) → Available PMs [apt] → Tool satisfiability → InstallPlan
//! ```
//!
//! # Key Design Decisions
//!
//! - **No enums for install methods**: Package manager IDs are strings
//! - **Tools reference package managers**: `install_options` lists which PMs can install the tool
//! - **Dependencies are explicit**: `depends_on` creates edges in the dependency graph
//! - **No fallbacks**: Each install method must be a properly modeled package manager
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::tool::*;
//!
//! // Define a tool
//! pub const GH: ToolDef = ToolDef {
//!     id: "gh",
//!     command: "gh",
//!     verify: "gh --version",
//!     install_options: &[
//!         InstallOption { via: "apt", inputs: InstallInputs::packages(&["gh"]) },
//!         InstallOption { via: "brew", inputs: InstallInputs::packages(&["gh"]) },
//!     ],
//!     depends_on: &[],
//! };
//!
//! // Check satisfiability
//! let mut registry = ToolRegistry::new();
//! registry.register(&GH);
//! let available_pms: HashSet<&str> = ["apt"].into_iter().collect();
//! assert!(is_satisfiable(&GH, &available_pms, &registry).is_ok());
//! ```

use std::collections::{HashMap, HashSet};

// ============================================================================
// Core Types
// ============================================================================

/// A CLI tool definition (CLI tool or package manager).
///
/// Tools are responsible for defining how they can be installed. Package managers
/// are also tools, but with empty `install_options` (they are base dependencies).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDef {
    /// Unique identifier for this tool (e.g., "gh", "git", "apt")
    pub id: &'static str,
    /// Command to invoke (e.g., "gh", "git", "apt-get")
    pub command: &'static str,
    /// Command to verify installation (e.g., "gh --version")
    pub verify: &'static str,
    /// How this tool can be installed, keyed by package manager id
    pub install_options: &'static [InstallOption],
    /// Other tools this depends on (must be installed first)
    pub depends_on: &'static [&'static str],
}

/// An install option via a specific package manager.
#[derive(Debug, Clone, PartialEq)]
pub struct InstallOption {
    /// Package manager id (e.g., "apt", "brew", "cargo")
    pub via: &'static str,
    /// Inputs for that package manager
    pub inputs: InstallInputs,
}

/// Inputs to a package manager.
///
/// This is flexible data, not an enum. Different package managers use different
/// fields. The package manager's installer knows how to interpret these.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InstallInputs {
    /// Package names for apt, brew, etc.
    pub packages: Option<&'static [&'static str]>,
    /// Crate name for cargo install
    pub crate_name: Option<&'static str>,
    /// Git URL for cargo install from git
    pub git_url: Option<&'static str>,
}

impl InstallInputs {
    /// Create inputs for package-based installation (apt, brew).
    pub const fn packages(packages: &'static [&'static str]) -> Self {
        Self {
            packages: Some(packages),
            crate_name: None,
            git_url: None,
        }
    }

    /// Create inputs for cargo crate installation.
    pub const fn crate_install(crate_name: &'static str) -> Self {
        Self {
            packages: None,
            crate_name: Some(crate_name),
            git_url: None,
        }
    }

    /// Create inputs for cargo install from git.
    pub const fn cargo_git(crate_name: &'static str, git_url: &'static str) -> Self {
        Self {
            packages: None,
            crate_name: Some(crate_name),
            git_url: Some(git_url),
        }
    }
}

// ============================================================================
// Platform Definition
// ============================================================================

/// Platform definition with available package managers.
///
/// Platforms can inherit from a parent (e.g., ubuntu → linux).
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformDef {
    /// Platform identifier (e.g., "ubuntu", "macos", "alpine")
    pub id: &'static str,
    /// Parent platform for inheritance (e.g., "linux" for "ubuntu")
    pub parent: Option<&'static str>,
    /// Package managers available on this platform
    pub available_pms: &'static [&'static str],
}

// ============================================================================
// Tool Registry
// ============================================================================

/// Registry of all known tools.
///
/// Tools register themselves here. The registry is used for:
/// - Looking up tools by ID
/// - Satisfiability checks
/// - deps.toml generation
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<&'static str, &'static ToolDef>,
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool.
    pub fn register(&mut self, tool: &'static ToolDef) {
        self.tools.insert(tool.id, tool);
    }

    /// Get a tool by ID.
    pub fn get(&self, id: &str) -> Option<&'static ToolDef> {
        self.tools.get(id).copied()
    }

    /// Get all registered tools.
    pub fn all(&self) -> impl Iterator<Item = &'static ToolDef> + '_ {
        self.tools.values().copied()
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

// ============================================================================
// Platform Registry
// ============================================================================

/// Registry of all known platforms.
#[derive(Debug, Default)]
pub struct PlatformRegistry {
    platforms: HashMap<&'static str, &'static PlatformDef>,
}

impl PlatformRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            platforms: HashMap::new(),
        }
    }

    /// Register a platform.
    pub fn register(&mut self, platform: &'static PlatformDef) {
        self.platforms.insert(platform.id, platform);
    }

    /// Get a platform by ID.
    pub fn get(&self, id: &str) -> Option<&'static PlatformDef> {
        self.platforms.get(id).copied()
    }

    /// Get all available package managers for a platform, including inherited ones.
    pub fn available_pms(&self, platform_id: &str) -> HashSet<&'static str> {
        let mut pms = HashSet::new();
        let mut current = platform_id;

        loop {
            if let Some(platform) = self.get(current) {
                pms.extend(platform.available_pms.iter().copied());
                if let Some(parent) = platform.parent {
                    current = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        pms
    }
}

// ============================================================================
// Satisfiability Check
// ============================================================================

/// Error when a tool cannot be satisfied.
#[derive(Debug, Clone, PartialEq)]
pub struct UnsatisfiableError {
    /// Tool that cannot be satisfied
    pub tool_id: String,
    /// Reason for unsatisfiability
    pub reason: String,
}

impl std::fmt::Display for UnsatisfiableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.tool_id, self.reason)
    }
}

impl std::error::Error for UnsatisfiableError {}

/// Check if a tool is satisfiable given available package managers.
///
/// A tool is satisfiable if:
/// 1. At least one of its install options uses an available package manager
/// 2. All its dependencies are also satisfiable
///
/// # Arguments
///
/// * `tool` - The tool to check
/// * `available_pms` - Set of available package manager IDs
/// * `registry` - Tool registry for looking up dependencies
///
/// # Returns
///
/// `Ok(())` if satisfiable, `Err(Vec<UnsatisfiableError>)` with all errors otherwise.
pub fn is_satisfiable(
    tool: &ToolDef,
    available_pms: &HashSet<&str>,
    registry: &ToolRegistry,
) -> Result<(), Vec<UnsatisfiableError>> {
    is_satisfiable_impl(tool, available_pms, registry, &mut HashSet::new())
}

fn is_satisfiable_impl(
    tool: &ToolDef,
    available_pms: &HashSet<&str>,
    registry: &ToolRegistry,
    visited: &mut HashSet<&'static str>,
) -> Result<(), Vec<UnsatisfiableError>> {
    // Prevent infinite recursion on circular dependencies
    if visited.contains(tool.id) {
        return Ok(());
    }
    visited.insert(tool.id);

    let mut errors = Vec::new();

    // Check if this tool can be installed via any available PM
    // Package managers themselves have empty install_options (they are base dependencies)
    let is_base_pm = tool.install_options.is_empty();
    let can_install = is_base_pm
        || tool
            .install_options
            .iter()
            .any(|opt| available_pms.contains(opt.via));

    if !can_install {
        let available: Vec<_> = available_pms.iter().collect();
        let needs: Vec<_> = tool.install_options.iter().map(|o| o.via).collect();
        errors.push(UnsatisfiableError {
            tool_id: tool.id.to_string(),
            reason: format!(
                "no install option available (have: {:?}, need one of: {:?})",
                available, needs
            ),
        });
    }

    // Check all dependencies
    for dep_id in tool.depends_on {
        if let Some(dep) = registry.get(dep_id) {
            if let Err(dep_errors) = is_satisfiable_impl(dep, available_pms, registry, visited) {
                errors.extend(dep_errors);
            }
        } else {
            errors.push(UnsatisfiableError {
                tool_id: tool.id.to_string(),
                reason: format!("dependency '{}' not found in registry", dep_id),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check if all tools are satisfiable given available package managers.
pub fn check_all_satisfiable<'a>(
    tools: impl IntoIterator<Item = &'a ToolDef>,
    available_pms: &HashSet<&str>,
    registry: &ToolRegistry,
) -> Result<(), Vec<UnsatisfiableError>> {
    let mut all_errors = Vec::new();

    for tool in tools {
        if let Err(errors) = is_satisfiable(tool, available_pms, registry) {
            all_errors.extend(errors);
        }
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}

/// Result of a satisfiability check with install plan.
#[derive(Debug, Clone)]
pub struct InstallPlan {
    /// Tools in dependency order (dependencies first)
    pub install_order: Vec<&'static str>,
    /// For each tool, which PM to use
    pub install_via: HashMap<&'static str, &'static str>,
}

/// Check satisfiability and produce an install plan.
///
/// The install plan lists tools in dependency order with the selected
/// package manager for each.
pub fn plan_installation(
    tools: &[&'static ToolDef],
    available_pms: &HashSet<&str>,
    registry: &ToolRegistry,
) -> Result<InstallPlan, Vec<UnsatisfiableError>> {
    // First check all are satisfiable
    check_all_satisfiable(tools.iter().copied(), available_pms, registry)?;

    let mut install_order = Vec::new();
    let mut install_via = HashMap::new();
    let mut visited = HashSet::new();

    fn visit(
        tool: &'static ToolDef,
        available_pms: &HashSet<&str>,
        registry: &ToolRegistry,
        install_order: &mut Vec<&'static str>,
        install_via: &mut HashMap<&'static str, &'static str>,
        visited: &mut HashSet<&'static str>,
    ) {
        if visited.contains(tool.id) {
            return;
        }
        visited.insert(tool.id);

        // Visit dependencies first
        for dep_id in tool.depends_on {
            if let Some(dep) = registry.get(dep_id) {
                visit(dep, available_pms, registry, install_order, install_via, visited);
            }
        }

        // Select first available PM
        let selected_pm = tool
            .install_options
            .iter()
            .find(|opt| available_pms.contains(opt.via))
            .map(|opt| opt.via);

        if let Some(pm) = selected_pm {
            install_via.insert(tool.id, pm);
        }

        install_order.push(tool.id);
    }

    for tool in tools {
        visit(
            *tool,
            available_pms,
            registry,
            &mut install_order,
            &mut install_via,
            &mut visited,
        );
    }

    Ok(InstallPlan {
        install_order,
        install_via,
    })
}

// ============================================================================
// Package Manager Definitions
// ============================================================================

/// apt package manager (Debian/Ubuntu).
pub static APT: ToolDef = ToolDef {
    id: "apt",
    command: "apt-get",
    verify: "apt-get --version",
    install_options: &[], // Base PM - no install options
    depends_on: &[],
};

/// Homebrew package manager (macOS/Linux).
pub static BREW: ToolDef = ToolDef {
    id: "brew",
    command: "brew",
    verify: "brew --version",
    install_options: &[], // Base PM - no install options
    depends_on: &[],
};

/// apk package manager (Alpine Linux).
pub static APK: ToolDef = ToolDef {
    id: "apk",
    command: "apk",
    verify: "apk --version",
    install_options: &[], // Base PM - no install options
    depends_on: &[],
};

/// cargo package manager (Rust).
///
/// Note: cargo depends on rust being installed.
pub static CARGO: ToolDef = ToolDef {
    id: "cargo",
    command: "cargo",
    verify: "cargo --version",
    install_options: &[], // Base PM once rust is installed
    depends_on: &["rust"],
};

// ============================================================================
// Tool Definitions
// ============================================================================

/// Git version control.
pub static GIT: ToolDef = ToolDef {
    id: "git",
    command: "git",
    verify: "git --version",
    install_options: &[
        InstallOption {
            via: "apt",
            inputs: InstallInputs::packages(&["git"]),
        },
        InstallOption {
            via: "brew",
            inputs: InstallInputs::packages(&["git"]),
        },
        InstallOption {
            via: "apk",
            inputs: InstallInputs::packages(&["git"]),
        },
    ],
    depends_on: &[],
};

/// Rust toolchain (rustc, cargo).
///
/// Note: On macOS, rust can be installed via brew (rustup).
/// On other platforms, proper modeling of shell environments would be needed.
pub static RUST: ToolDef = ToolDef {
    id: "rust",
    command: "rustc",
    verify: "rustc --version",
    install_options: &[
        InstallOption {
            via: "brew",
            inputs: InstallInputs::packages(&["rustup"]),
        },
        // TODO: apt has rustc package but rustup is preferred
    ],
    depends_on: &[],
};

// ============================================================================
// Platform Definitions
// ============================================================================

/// Linux base platform.
pub static LINUX: PlatformDef = PlatformDef {
    id: "linux",
    parent: None,
    available_pms: &[],
};

/// Ubuntu platform (Debian-based Linux).
pub static UBUNTU: PlatformDef = PlatformDef {
    id: "ubuntu",
    parent: Some("linux"),
    available_pms: &["apt"],
};

/// Debian platform.
pub static DEBIAN: PlatformDef = PlatformDef {
    id: "debian",
    parent: Some("linux"),
    available_pms: &["apt"],
};

/// Alpine Linux platform.
pub static ALPINE: PlatformDef = PlatformDef {
    id: "alpine",
    parent: Some("linux"),
    available_pms: &["apk"],
};

/// macOS platform.
pub static MACOS: PlatformDef = PlatformDef {
    id: "macos",
    parent: None,
    available_pms: &["brew"],
};

// ============================================================================
// Registry Initialization
// ============================================================================

/// Create a tool registry with all built-in package managers and tools.
pub fn default_tool_registry() -> ToolRegistry {
    use crate::transport::github::cli::GH_TOOL;
    
    let mut registry = ToolRegistry::new();
    // Package managers
    registry.register(&APT);
    registry.register(&BREW);
    registry.register(&APK);
    registry.register(&CARGO);
    // Tools
    registry.register(&GIT);
    registry.register(&RUST);
    registry.register(&GH_TOOL);
    registry
}

/// Create a platform registry with all built-in platforms.
pub fn default_platform_registry() -> PlatformRegistry {
    let mut registry = PlatformRegistry::new();
    registry.register(&LINUX);
    registry.register(&UBUNTU);
    registry.register(&DEBIAN);
    registry.register(&ALPINE);
    registry.register(&MACOS);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test tool definitions
    static TEST_APT: ToolDef = ToolDef {
        id: "apt",
        command: "apt-get",
        verify: "apt-get --version",
        install_options: &[],
        depends_on: &[],
    };

    static TEST_BREW: ToolDef = ToolDef {
        id: "brew",
        command: "brew",
        verify: "brew --version",
        install_options: &[],
        depends_on: &[],
    };

    static TEST_GH: ToolDef = ToolDef {
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

    static TEST_GIT: ToolDef = ToolDef {
        id: "git",
        command: "git",
        verify: "git --version",
        install_options: &[
            InstallOption {
                via: "apt",
                inputs: InstallInputs::packages(&["git"]),
            },
            InstallOption {
                via: "brew",
                inputs: InstallInputs::packages(&["git"]),
            },
        ],
        depends_on: &[],
    };

    static TEST_TOOL_WITH_DEP: ToolDef = ToolDef {
        id: "tool_with_dep",
        command: "tool",
        verify: "tool --version",
        install_options: &[InstallOption {
            via: "apt",
            inputs: InstallInputs::packages(&["tool"]),
        }],
        depends_on: &["git"],
    };

    fn test_registry() -> ToolRegistry {
        let mut registry = ToolRegistry::new();
        registry.register(&TEST_APT);
        registry.register(&TEST_BREW);
        registry.register(&TEST_GH);
        registry.register(&TEST_GIT);
        registry.register(&TEST_TOOL_WITH_DEP);
        registry
    }

    #[test]
    fn test_install_inputs_packages() {
        let inputs = InstallInputs::packages(&["gh"]);
        assert_eq!(inputs.packages, Some(&["gh"][..]));
        assert_eq!(inputs.crate_name, None);
        assert_eq!(inputs.git_url, None);
    }

    #[test]
    fn test_install_inputs_crate() {
        let inputs = InstallInputs::crate_install("cargo-nextest");
        assert_eq!(inputs.packages, None);
        assert_eq!(inputs.crate_name, Some("cargo-nextest"));
        assert_eq!(inputs.git_url, None);
    }

    #[test]
    fn test_install_inputs_cargo_git() {
        let inputs = InstallInputs::cargo_git("reindeer", "https://github.com/example/reindeer");
        assert_eq!(inputs.packages, None);
        assert_eq!(inputs.crate_name, Some("reindeer"));
        assert_eq!(inputs.git_url, Some("https://github.com/example/reindeer"));
    }

    #[test]
    fn test_tool_registry() {
        let registry = test_registry();
        assert_eq!(registry.len(), 5);
        assert!(registry.get("gh").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_satisfiable_with_apt() {
        let registry = test_registry();
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        assert!(is_satisfiable(&TEST_GH, &available, &registry).is_ok());
        assert!(is_satisfiable(&TEST_GIT, &available, &registry).is_ok());
    }

    #[test]
    fn test_satisfiable_with_brew() {
        let registry = test_registry();
        let available: HashSet<&str> = ["brew"].into_iter().collect();

        assert!(is_satisfiable(&TEST_GH, &available, &registry).is_ok());
        assert!(is_satisfiable(&TEST_GIT, &available, &registry).is_ok());
    }

    #[test]
    fn test_unsatisfiable_no_pm() {
        let registry = test_registry();
        let available: HashSet<&str> = HashSet::new();

        let result = is_satisfiable(&TEST_GH, &available, &registry);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].tool_id, "gh");
    }

    #[test]
    fn test_satisfiable_with_dependencies() {
        let registry = test_registry();
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        // tool_with_dep depends on git, both should be satisfiable via apt
        assert!(is_satisfiable(&TEST_TOOL_WITH_DEP, &available, &registry).is_ok());
    }

    static TEST_TOOL_MISSING_DEP: ToolDef = ToolDef {
        id: "tool_missing_dep",
        command: "tool",
        verify: "tool --version",
        install_options: &[InstallOption {
            via: "apt",
            inputs: InstallInputs::packages(&["tool"]),
        }],
        depends_on: &["nonexistent"], // dependency not in registry
    };

    #[test]
    fn test_unsatisfiable_missing_dependency() {
        let mut registry = ToolRegistry::new();
        registry.register(&TEST_APT);
        // Don't register the dependency

        let available: HashSet<&str> = ["apt"].into_iter().collect();
        let result = is_satisfiable(&TEST_TOOL_MISSING_DEP, &available, &registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_base_pm_always_satisfiable() {
        let registry = test_registry();
        let available: HashSet<&str> = HashSet::new();

        // Base PMs (with empty install_options) are always satisfiable
        // because they represent the platform's built-in capabilities
        assert!(is_satisfiable(&TEST_APT, &available, &registry).is_ok());
        assert!(is_satisfiable(&TEST_BREW, &available, &registry).is_ok());
    }

    #[test]
    fn test_check_all_satisfiable() {
        let registry = test_registry();
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        let tools = [&TEST_GH, &TEST_GIT];
        assert!(check_all_satisfiable(tools, &available, &registry).is_ok());
    }

    #[test]
    fn test_plan_installation() {
        let registry = test_registry();
        let available: HashSet<&str> = ["apt"].into_iter().collect();

        let tools = [&TEST_TOOL_WITH_DEP];
        let plan = plan_installation(&tools, &available, &registry).unwrap();

        // git should come before tool_with_dep (dependency order)
        let git_pos = plan.install_order.iter().position(|&t| t == "git");
        let tool_pos = plan.install_order.iter().position(|&t| t == "tool_with_dep");
        assert!(git_pos.is_some());
        assert!(tool_pos.is_some());
        assert!(git_pos.unwrap() < tool_pos.unwrap());

        // Both should use apt
        assert_eq!(plan.install_via.get("git"), Some(&"apt"));
        assert_eq!(plan.install_via.get("tool_with_dep"), Some(&"apt"));
    }

    #[test]
    fn test_platform_registry() {
        static TEST_LINUX: PlatformDef = PlatformDef {
            id: "linux",
            parent: None,
            available_pms: &[],
        };

        static TEST_UBUNTU: PlatformDef = PlatformDef {
            id: "ubuntu",
            parent: Some("linux"),
            available_pms: &["apt"],
        };

        let mut registry = PlatformRegistry::new();
        registry.register(&TEST_LINUX);
        registry.register(&TEST_UBUNTU);

        let ubuntu_pms = registry.available_pms("ubuntu");
        assert!(ubuntu_pms.contains("apt"));
    }

    #[test]
    fn test_builtin_package_managers() {
        // Verify built-in PMs are defined correctly
        assert_eq!(super::APT.id, "apt");
        assert_eq!(super::APT.command, "apt-get");
        assert!(super::APT.install_options.is_empty()); // Base PM

        assert_eq!(super::BREW.id, "brew");
        assert!(super::BREW.install_options.is_empty()); // Base PM

        assert_eq!(super::APK.id, "apk");
        assert!(super::APK.install_options.is_empty()); // Base PM

        assert_eq!(super::CARGO.id, "cargo");
        assert!(super::CARGO.depends_on.contains(&"rust")); // cargo needs rust
    }

    #[test]
    fn test_builtin_platforms() {
        // Verify built-in platforms are defined correctly
        assert_eq!(super::UBUNTU.id, "ubuntu");
        assert_eq!(super::UBUNTU.parent, Some("linux"));
        assert!(super::UBUNTU.available_pms.contains(&"apt"));

        assert_eq!(super::MACOS.id, "macos");
        assert_eq!(super::MACOS.parent, None);
        assert!(super::MACOS.available_pms.contains(&"brew"));

        assert_eq!(super::ALPINE.id, "alpine");
        assert!(super::ALPINE.available_pms.contains(&"apk"));
    }

    #[test]
    fn test_default_registries() {
        let tool_registry = super::default_tool_registry();
        assert!(tool_registry.get("apt").is_some());
        assert!(tool_registry.get("brew").is_some());
        assert!(tool_registry.get("apk").is_some());
        assert!(tool_registry.get("cargo").is_some());
        assert!(tool_registry.get("git").is_some());
        assert!(tool_registry.get("rust").is_some());
        assert!(tool_registry.get("gh").is_some());

        let platform_registry = super::default_platform_registry();
        assert!(platform_registry.get("ubuntu").is_some());
        assert!(platform_registry.get("macos").is_some());
        assert!(platform_registry.get("alpine").is_some());

        // Test ubuntu has apt via inheritance
        let ubuntu_pms = platform_registry.available_pms("ubuntu");
        assert!(ubuntu_pms.contains("apt"));
    }

    #[test]
    fn test_git_tool_definition() {
        assert_eq!(super::GIT.id, "git");
        assert_eq!(super::GIT.command, "git");
        assert_eq!(super::GIT.verify, "git --version");
        
        // git should be installable via apt, brew, and apk
        let apt_opt = super::GIT.install_options.iter().find(|o| o.via == "apt");
        assert!(apt_opt.is_some());
        assert_eq!(apt_opt.unwrap().inputs.packages, Some(&["git"][..]));

        let brew_opt = super::GIT.install_options.iter().find(|o| o.via == "brew");
        assert!(brew_opt.is_some());

        let apk_opt = super::GIT.install_options.iter().find(|o| o.via == "apk");
        assert!(apk_opt.is_some());
    }

    #[test]
    fn test_rust_tool_definition() {
        assert_eq!(super::RUST.id, "rust");
        assert_eq!(super::RUST.command, "rustc");
        assert_eq!(super::RUST.verify, "rustc --version");
        
        // rust can be installed via brew (rustup)
        let brew_opt = super::RUST.install_options.iter().find(|o| o.via == "brew");
        assert!(brew_opt.is_some());
        assert_eq!(brew_opt.unwrap().inputs.packages, Some(&["rustup"][..]));
    }

    #[test]
    fn test_cargo_depends_on_rust() {
        assert!(super::CARGO.depends_on.contains(&"rust"));
    }

    #[test]
    fn test_git_satisfiable_on_ubuntu() {
        let registry = super::default_tool_registry();
        let available: HashSet<&str> = ["apt"].into_iter().collect();
        
        assert!(super::is_satisfiable(&super::GIT, &available, &registry).is_ok());
    }

    #[test]
    fn test_git_satisfiable_on_macos() {
        let registry = super::default_tool_registry();
        let available: HashSet<&str> = ["brew"].into_iter().collect();
        
        assert!(super::is_satisfiable(&super::GIT, &available, &registry).is_ok());
    }

    #[test]
    fn test_rust_satisfiable_on_macos() {
        let registry = super::default_tool_registry();
        let available: HashSet<&str> = ["brew"].into_iter().collect();
        
        assert!(super::is_satisfiable(&super::RUST, &available, &registry).is_ok());
    }

    #[test]
    fn test_rust_not_satisfiable_on_ubuntu_only_apt() {
        let registry = super::default_tool_registry();
        let available: HashSet<&str> = ["apt"].into_iter().collect();
        
        // rust only has brew install option, so not satisfiable with just apt
        let result = super::is_satisfiable(&super::RUST, &available, &registry);
        assert!(result.is_err());
    }
}
