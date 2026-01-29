//! Tool registry for makegen.
//!
//! Defines what tools exist and their entrypoint parameters.

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

/// Registry of all gunbc tools.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    pub tools: Vec<ToolInfo>,
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Add a tool to the registry.
    pub fn register(&mut self, tool: ToolInfo) {
        self.tools.push(tool);
    }

    /// Build the default registry with all known gunbc tools.
    pub fn default_registry() -> Self {
        let mut registry = Self::new();

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

        // gunbc-makegen (self!)
        registry.register(
            ToolInfo::new("gunbc-makegen", "makegen", "Generate Makefile from tool registry")
                .with_param(
                    EntrypointParam::new("output_path", "OUTPUT", "--output", "String")
                        .with_default("Makefile"),
                ),
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
                    ExtraTarget::new("serve", "Generate viz data and open in browser")
                        .with_command("@echo \"Starting server at http://localhost:8080/viz.html\"")
                        .with_command("@(sleep 1 && python3 -c \"import webbrowser; webbrowser.open('http://localhost:8080/viz.html')\") &")
                        .with_command("@python3 -m http.server 8080"),
                ),
        );

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
