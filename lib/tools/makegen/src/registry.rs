//! Tool registry for makegen.
//!
//! Defines what tools exist and their entrypoint parameters.
//! Also defines meta targets (test, check, fmt, clippy) that compose with prep.
//! 
//! # BuildConfig
//! 
//! The `BuildConfig` struct is the single source of truth for all build/test/lint
//! commands. This eliminates duplicate hardcoded commands across the codebase.

// ============================================================================
// Build Configuration - Single source of truth for build commands
// ============================================================================

/// Build system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSystem {
    /// Standard Cargo build
    Cargo,
    /// Buck2 build system
    Buck2,
}

/// Unified build system configuration.
/// Single source of truth for build/test/lint operations.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Build system (cargo, buck2)
    pub build_system: BuildSystem,
    /// Command to run codegen
    pub codegen_command: Vec<&'static str>,
    /// Command to run daggen
    pub daggen_command: Vec<&'static str>,
    /// Command to build all targets
    pub build_command: Vec<&'static str>,
    /// Command to run tests
    pub test_command: Vec<&'static str>,
    /// Command to run linter
    pub lint_command: Vec<&'static str>,
    /// Command to format code
    pub fmt_command: Vec<&'static str>,
    /// Command to check formatting
    pub fmt_check_command: Vec<&'static str>,
    /// Command to type-check without full build
    pub check_command: Vec<&'static str>,
}

impl BuildConfig {
    /// Default cargo-based build config.
    pub fn cargo() -> Self {
        Self {
            build_system: BuildSystem::Cargo,
            codegen_command: vec!["cargo", "run", "-p", "gunbc-codegen", "--release", "--", "codegen"],
            daggen_command: vec!["cargo", "run", "-p", "gunbc-codegen", "--release", "--", "daggen"],
            build_command: vec!["cargo", "build", "--all-targets"],
            test_command: vec!["cargo", "test"],
            lint_command: vec!["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
            fmt_command: vec!["cargo", "fmt"],
            fmt_check_command: vec!["cargo", "fmt", "--", "--check"],
            check_command: vec!["cargo", "check", "--all-targets"],
        }
    }

    /// Buck2-based build config (for future use).
    pub fn buck2() -> Self {
        Self {
            build_system: BuildSystem::Buck2,
            codegen_command: vec!["cargo", "run", "-p", "gunbc-codegen", "--release", "--", "codegen"],
            daggen_command: vec!["cargo", "run", "-p", "gunbc-codegen", "--release", "--", "daggen"],
            build_command: vec!["buck2", "build", "//..."],
            test_command: vec!["buck2", "test", "//..."],
            lint_command: vec!["buck2", "run", "//tools:clippy"],
            fmt_command: vec!["cargo", "fmt"], // fmt stays cargo
            fmt_check_command: vec!["cargo", "fmt", "--", "--check"],
            check_command: vec!["buck2", "build", "//..."], // buck2 check is same as build
        }
    }

    /// Get the command as a shell string (for Makefile generation).
    pub fn codegen_shell(&self) -> String {
        format!("@{}", self.codegen_command.join(" "))
    }

    /// Get the build command as a shell string.
    pub fn build_shell(&self) -> String {
        format!("@{}", self.build_command.join(" "))
    }

    /// Get the test command as a shell string.
    pub fn test_shell(&self) -> String {
        format!("@{}", self.test_command.join(" "))
    }

    /// Get the lint command as a shell string.
    pub fn lint_shell(&self) -> String {
        format!("@{}", self.lint_command.join(" "))
    }

    /// Get the fmt command as a shell string.
    pub fn fmt_shell(&self) -> String {
        format!("@{}", self.fmt_command.join(" "))
    }

    /// Get the fmt-check command as a shell string.
    pub fn fmt_check_shell(&self) -> String {
        format!("@{}", self.fmt_check_command.join(" "))
    }

    /// Get the check command as a shell string.
    pub fn check_shell(&self) -> String {
        format!("@{}", self.check_command.join(" "))
    }
}

/// Get the default build config (cargo-based).
pub fn default_build_config() -> BuildConfig {
    BuildConfig::cargo()
}

// ============================================================================
// Tool Information
// ============================================================================

/// Information about a gunbc tool.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// Crate name (e.g., "gunbc-gist")
    pub crate_name: String,
    /// Short name for make target (e.g., "gist")
    pub short_name: String,
    /// Description for help text
    pub description: String,
    /// Entrypoint parameters (from DAG entrypoints)
    pub entrypoints: Vec<EntrypointParam>,
    /// Extra composite targets (e.g., "viz-serve" that runs viz + starts server)
    pub extra_targets: Vec<ExtraTarget>,
    /// Whether this tool has a declarative DAG definition (for daggen detection)
    pub has_declarative_dag: bool,
}

/// An extra target that combines the main tool with additional commands.
#[derive(Debug, Clone)]
pub struct ExtraTarget {
    /// Target name suffix (e.g., "serve" becomes "viz-serve")
    pub suffix: String,
    /// Description for help text
    pub description: String,
    /// Shell commands to run after the main tool
    pub post_commands: Vec<String>,
}

impl ToolInfo {
    /// Create a new tool info.
    pub fn new(
        crate_name: impl Into<String>,
        short_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            crate_name: crate_name.into(),
            short_name: short_name.into(),
            description: description.into(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
        }
    }

    /// Add an entrypoint parameter.
    pub fn with_param(mut self, param: EntrypointParam) -> Self {
        self.entrypoints.push(param);
        self
    }

    /// Add an extra composite target.
    pub fn with_extra_target(mut self, target: ExtraTarget) -> Self {
        self.extra_targets.push(target);
        self
    }

    /// Mark this tool as having a declarative DAG definition.
    pub fn with_declarative_dag(mut self) -> Self {
        self.has_declarative_dag = true;
        self
    }
}

impl ExtraTarget {
    /// Create a new extra target.
    pub fn new(suffix: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            suffix: suffix.into(),
            description: description.into(),
            post_commands: Vec::new(),
        }
    }

    /// Add a post command.
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.post_commands.push(cmd.into());
        self
    }
}

/// An entrypoint parameter that becomes a Make variable.
#[derive(Debug, Clone)]
pub struct EntrypointParam {
    /// DAG port name (e.g., "repo_path")
    pub port_name: String,
    /// Make variable name (e.g., "REPO")
    pub make_var: String,
    /// CLI flag (e.g., "--repo")
    pub cli_flag: String,
    /// Type hint for help text
    pub type_hint: String,
    /// Default value if any
    pub default: Option<String>,
    /// Whether this param can be repeated (for list types)
    pub repeatable: bool,
}

impl EntrypointParam {
    /// Create a new entrypoint parameter.
    pub fn new(
        port_name: impl Into<String>,
        make_var: impl Into<String>,
        cli_flag: impl Into<String>,
        type_hint: impl Into<String>,
    ) -> Self {
        Self {
            port_name: port_name.into(),
            make_var: make_var.into(),
            cli_flag: cli_flag.into(),
            type_hint: type_hint.into(),
            default: None,
            repeatable: false,
        }
    }

    /// Set a default value.
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Mark as repeatable (for list parameters).
    pub fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }
}

// ============================================================================
// Meta Targets - Holistic targets that compose with prep
// ============================================================================

/// How much preparation is needed before running a meta target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrepLevel {
    /// No prep needed (e.g., fmt just formats existing code)
    None,
    /// Just ensure codegen has run (light prep)
    Codegen,
    /// Full prep including build (heavy prep)
    Full,
}

/// Which BuildConfig field to use for a meta target.
///
/// This allows MetaTarget to reference commands from BuildConfig
/// rather than storing them directly, ensuring a single source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    /// Use test_command
    Test,
    /// Use lint_command
    Lint,
    /// Use fmt_command
    Fmt,
    /// Use check_command
    Check,
    /// Use build_command
    Build,
}

impl ConfigField {
    /// Get the command from BuildConfig for this field.
    pub fn get_command(&self, config: &BuildConfig) -> String {
        match self {
            ConfigField::Test => config.test_shell(),
            ConfigField::Lint => config.lint_shell(),
            ConfigField::Fmt => config.fmt_shell(),
            ConfigField::Check => config.check_shell(),
            ConfigField::Build => config.build_shell(),
        }
    }

    /// Get the check variant command if applicable.
    pub fn get_check_command(&self, config: &BuildConfig) -> Option<String> {
        match self {
            ConfigField::Fmt => Some(config.fmt_check_shell()),
            _ => None,
        }
    }
}

/// A meta target that composes prep + a specific operation.
///
/// Meta targets are holistic targets like `test`, `check`, `fmt`, `clippy`
/// that developers use frequently. They depend on the appropriate prep level
/// to ensure the repository is in a consistent state.
///
/// Commands are now referenced via `ConfigField` to ensure BuildConfig
/// remains the single source of truth.
#[derive(Debug, Clone)]
pub struct MetaTarget {
    /// Target name (e.g., "test")
    pub name: String,
    /// Description for help text
    pub description: String,
    /// How much prep is needed
    pub prep_level: PrepLevel,
    /// Which BuildConfig field to use for the command
    pub config_field: ConfigField,
    /// Whether this target has a check variant (e.g., fmt-check)
    pub has_check_variant: bool,
    // Legacy fields kept for backward compatibility during transition
    /// Shell command to run (deprecated: use config_field instead)
    pub command: String,
    /// Command for check variant (deprecated: use config_field instead)
    pub check_command: Option<String>,
}

impl MetaTarget {
    /// Create a new meta target using ConfigField.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        prep_level: PrepLevel,
        config_field: ConfigField,
    ) -> Self {
        let name_str = name.into();
        Self {
            name: name_str,
            description: description.into(),
            prep_level,
            config_field,
            has_check_variant: false,
            // Legacy: empty strings as placeholders
            command: String::new(),
            check_command: None,
        }
    }

    /// Mark this target as having a check variant (e.g., fmt-check).
    pub fn with_check_variant(mut self) -> Self {
        self.has_check_variant = true;
        self
    }

    /// Get the command for this meta target from BuildConfig.
    pub fn get_command(&self, config: &BuildConfig) -> String {
        self.config_field.get_command(config)
    }

    /// Get the check command for this meta target from BuildConfig.
    pub fn get_check_command(&self, config: &BuildConfig) -> Option<String> {
        if self.has_check_variant {
            self.config_field.get_check_command(config)
        } else {
            None
        }
    }
}

/// Get the default meta targets.
pub fn default_meta_targets() -> Vec<MetaTarget> {
    vec![
        // test - run all tests (requires full prep)
        MetaTarget::new("test", "Run all tests", PrepLevel::Full, ConfigField::Test),
        // check - type check without building (requires codegen)
        MetaTarget::new(
            "check",
            "Type check all targets",
            PrepLevel::Codegen,
            ConfigField::Check,
        ),
        // clippy - run linter (requires codegen)
        MetaTarget::new(
            "clippy",
            "Run clippy linter",
            PrepLevel::Codegen,
            ConfigField::Lint,
        ),
        // fmt - format code (no prep needed)
        MetaTarget::new("fmt", "Format all code", PrepLevel::None, ConfigField::Fmt)
            .with_check_variant(),
    ]
}

/// Registry of all gunbc tools and meta targets.
#[derive(Debug)]
pub struct ToolRegistry {
    /// Individual tool targets (gist, buck2, etc.)
    pub tools: Vec<ToolInfo>,
    /// Meta targets (test, check, fmt, clippy)
    pub meta_targets: Vec<MetaTarget>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            tools: Vec::new(),
            meta_targets: default_meta_targets(),
        }
    }
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            meta_targets: Vec::new(),
        }
    }

    /// Add a tool to the registry.
    pub fn register(&mut self, tool: ToolInfo) {
        self.tools.push(tool);
    }

    /// Add a meta target to the registry.
    pub fn register_meta(&mut self, target: MetaTarget) {
        self.meta_targets.push(target);
    }

    // ========================================================================
    // Derived Properties - Computed from registry state
    // ========================================================================

    /// Get tools that need CLI codegen (all tools with binaries).
    pub fn tools_needing_codegen(&self) -> Vec<&ToolInfo> {
        // All tools need codegen since they all have generated CLIs
        self.tools.iter().collect()
    }

    /// Get tools that need daggen (have declarative DAG definitions).
    pub fn tools_needing_daggen(&self) -> Vec<&ToolInfo> {
        self.tools
            .iter()
            .filter(|t| t.has_declarative_dag)
            .collect()
    }

    /// Check if any codegen is needed by examining the filesystem.
    /// Returns true if any tool's main.rs is missing.
    pub fn needs_codegen(&self) -> bool {
        let buck_out = std::path::Path::new("buck-out/gen/bin");
        if !buck_out.exists() {
            return true;
        }
        self.tools.iter().any(|t| {
            !buck_out.join(&t.short_name).join("main.rs").exists()
        })
    }

    /// Check if any daggen is needed.
    pub fn needs_daggen(&self) -> bool {
        // For now, daggen is optional - only needed if we want to regenerate
        // declarative DAG definitions into code
        false
    }

    /// Build the default registry with all known gunbc tools.
    pub fn default_registry() -> Self {
        let mut registry = Self {
            tools: Vec::new(),
            meta_targets: default_meta_targets(),
        };

        // gunbc-gist
        registry.register(
            ToolInfo::new("gunbc-gist", "gist", "Create a GitHub gist from code files")
                .with_param(
                    EntrypointParam::new("repo_path", "REPO", "--repo", "String")
                        .with_default("."),
                )
                .with_param(
                    EntrypointParam::new("extensions", "EXT", "-e", "String")
                        .repeatable(),
                ),
        );

        // gunbc-buck2
        registry.register(
            ToolInfo::new("gunbc-buck2", "buck2", "Generate BUCK file from Cargo.toml")
                .with_param(
                    EntrypointParam::new("cargo_toml_path", "INPUT", "--input", "String")
                        .with_default("Cargo.toml"),
                )
                .with_param(
                    EntrypointParam::new("output_path", "OUTPUT", "--output", "String")
                        .with_default("BUCK"),
                ),
        );

        // gunbc-makegen (self!) - has declarative DAG
        registry.register(
            ToolInfo::new("gunbc-makegen", "makegen", "Generate Makefile from tool registry")
                .with_param(
                    EntrypointParam::new("output_path", "OUTPUT", "--output", "String")
                        .with_default("Makefile"),
                )
                .with_declarative_dag(),
        );

        // gunbc-deps
        registry.register(
            ToolInfo::new("gunbc-deps", "deps", "Install tool dependencies")
                .with_param(
                    EntrypointParam::new("manifest_path", "MANIFEST", "--manifest", "String")
                        .with_default("deps.toml"),
                ),
        );

        // gunbc-ci
        registry.register(
            ToolInfo::new("gunbc-ci", "ci", "Run CI pipeline"),
        );

        // gunbc-bootstrap
        registry.register(
            ToolInfo::new("gunbc-bootstrap", "bootstrap", "Generate Makefile and .gitignore"),
        );

        // gunbc-viz
        registry.register(
            ToolInfo::new("gunbc-viz", "viz", "Generate DAG visualization data")
                .with_param(
                    EntrypointParam::new("output_path", "OUTPUT", "--output", "String")
                        .with_default("viz-data.json"),
                )
                .with_extra_target(
                    // Simple HTTP server - no Python escape hatches for browser opening
                    // Users can open http://localhost:8080/viz.html manually
                    ExtraTarget::new("serve", "Start HTTP server for viz (open http://localhost:8080/viz.html)")
                        .with_command("@echo \"Serving at http://localhost:8080/viz.html\"")
                        .with_command("@echo \"Press Ctrl+C to stop\"")
                        .with_command("@python3 -m http.server 8080"),
                ),
        );

        // NOTE: prep tool has been removed - CI now handles all preparation
        // The prep functionality is consolidated into CI's Prep stage

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // BuildConfig Tests
    // ========================================================================

    #[test]
    fn test_build_config_cargo() {
        let config = BuildConfig::cargo();
        assert_eq!(config.build_system, BuildSystem::Cargo);
        assert!(config.build_command.contains(&"cargo"));
        assert!(config.test_command.contains(&"test"));
    }

    #[test]
    fn test_build_config_buck2() {
        let config = BuildConfig::buck2();
        assert_eq!(config.build_system, BuildSystem::Buck2);
        assert!(config.build_command.contains(&"buck2"));
    }

    #[test]
    fn test_build_config_shell_methods() {
        let config = BuildConfig::cargo();
        assert!(config.build_shell().starts_with("@"));
        assert!(config.test_shell().contains("cargo test"));
        assert!(config.lint_shell().contains("clippy"));
    }

    // ========================================================================
    // ToolRegistry Tests
    // ========================================================================

    #[test]
    fn test_default_registry_has_tools() {
        let registry = ToolRegistry::default_registry();
        assert!(registry.tools.len() >= 2);
        
        let gist = registry.tools.iter().find(|t| t.short_name == "gist");
        assert!(gist.is_some());
        
        let buck2 = registry.tools.iter().find(|t| t.short_name == "buck2");
        assert!(buck2.is_some());
    }

    #[test]
    fn test_tool_has_entrypoints() {
        let registry = ToolRegistry::default_registry();
        let gist = registry.tools.iter().find(|t| t.short_name == "gist").unwrap();
        
        assert!(!gist.entrypoints.is_empty());
        
        let repo_param = gist.entrypoints.iter().find(|p| p.port_name == "repo_path");
        assert!(repo_param.is_some());
        assert_eq!(repo_param.unwrap().make_var, "REPO");
    }

    #[test]
    fn test_tools_needing_codegen() {
        let registry = ToolRegistry::default_registry();
        let tools = registry.tools_needing_codegen();
        // All tools need codegen
        assert_eq!(tools.len(), registry.tools.len());
    }

    #[test]
    fn test_tools_needing_daggen() {
        let registry = ToolRegistry::default_registry();
        let tools = registry.tools_needing_daggen();
        // Only makegen has declarative DAG currently
        assert!(tools.iter().any(|t| t.short_name == "makegen"));
    }

    #[test]
    fn test_makegen_has_declarative_dag() {
        let registry = ToolRegistry::default_registry();
        let makegen = registry.tools.iter().find(|t| t.short_name == "makegen").unwrap();
        assert!(makegen.has_declarative_dag);
    }

    // ========================================================================
    // MetaTarget Tests
    // ========================================================================

    #[test]
    fn test_default_meta_targets() {
        let targets = default_meta_targets();
        
        // Should have test, check, clippy, fmt
        assert!(targets.iter().any(|t| t.name == "test"));
        assert!(targets.iter().any(|t| t.name == "check"));
        assert!(targets.iter().any(|t| t.name == "clippy"));
        assert!(targets.iter().any(|t| t.name == "fmt"));
    }

    #[test]
    fn test_meta_target_prep_levels() {
        let targets = default_meta_targets();
        
        let test = targets.iter().find(|t| t.name == "test").unwrap();
        assert_eq!(test.prep_level, PrepLevel::Full);
        
        let fmt = targets.iter().find(|t| t.name == "fmt").unwrap();
        assert_eq!(fmt.prep_level, PrepLevel::None);
        
        let clippy = targets.iter().find(|t| t.name == "clippy").unwrap();
        assert_eq!(clippy.prep_level, PrepLevel::Codegen);
    }

    #[test]
    fn test_fmt_has_check_variant() {
        let targets = default_meta_targets();
        let fmt = targets.iter().find(|t| t.name == "fmt").unwrap();
        let config = BuildConfig::cargo();

        assert!(fmt.has_check_variant);
        // Now uses ConfigField to get check command from BuildConfig
        assert!(fmt.get_check_command(&config).is_some());
        assert!(fmt.get_check_command(&config).unwrap().contains("--check"));
    }

    #[test]
    fn test_registry_has_meta_targets() {
        let registry = ToolRegistry::default_registry();
        assert!(!registry.meta_targets.is_empty());
        assert!(registry.meta_targets.iter().any(|t| t.name == "test"));
    }
}
