//! Tool registry for makegen.
//!
//! Tool targets are derived from DSL entrypoint inference
//! (`discover_tool_defs_from_dsl()`). Adding a new `.dag` entrypoint should not
//! require Rust-side registry edits.

use gunbc_ir::cargo::{
    BinaryArgs, CargoCommand, CargoInvocation, CodegenSubcommand, Subcommand, Warnings,
};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::ShellRequest;

// ============================================================================
// Build configuration
// ============================================================================

/// Build system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSystem {
    /// Standard Cargo build.
    Cargo,
    /// Buck2 build system.
    Buck2,
}

/// A build command that can be either a structured cargo command or a raw
/// shell command (for non-cargo tools like buck2).
#[derive(Debug, Clone)]
pub enum BuildCommand {
    /// A structured cargo command with semantic rendering.
    Cargo(CargoCommand),
    /// A raw shell command.
    Shell(Vec<String>),
}

impl BuildCommand {
    /// Render as a prefixed `@` shell command string.
    pub fn shell(&self) -> String {
        format!("@{}", self.to_shell())
    }

    /// Derive a new command with a `--mode` argument appended.
    pub fn with_mode(&self, mode: ExecMode) -> Self {
        match self {
            Self::Cargo(cmd) => Self::Cargo(cmd.clone().args(BinaryArgs::with_mode(mode))),
            Self::Shell(parts) => {
                let mut p = parts.clone();
                p.push(format!("--mode={mode}"));
                Self::Shell(p)
            }
        }
    }

    /// Render as a shell command string.
    pub fn to_shell(&self) -> String {
        match self {
            Self::Cargo(cmd) => cmd.to_shell_with_env(),
            Self::Shell(parts) => parts.join(" "),
        }
    }

    /// Convert to a `ShellRequest` for transport execution.
    pub fn to_shell_request(&self) -> ShellRequest {
        match self {
            Self::Cargo(cmd) => cmd.to_shell_request(),
            Self::Shell(parts) => {
                let (command, args) = parts.split_first().expect("empty command");
                ShellRequest::new(command).args(args.iter().cloned())
            }
        }
    }
}

/// Unified build system configuration.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Build system (cargo, buck2).
    pub build_system: BuildSystem,
    /// Whether Makefile targets should use DAG entrypoints instead of raw
    /// cargo/buck2 commands.
    pub use_dag_entrypoints: bool,
    /// Repo-level warning policy.
    pub warnings: Warnings,
    /// Command to ensure codegen outputs exist (bootstrap-safe).
    pub ensure_codegen: BuildCommand,
    /// Command to run codegen.
    pub codegen: BuildCommand,
    /// Command to run daggen.
    pub daggen: BuildCommand,
    /// Command to build all targets.
    pub build: BuildCommand,
    /// Command to run tests.
    pub test: BuildCommand,
    /// Command to run linter.
    pub lint: BuildCommand,
    /// Command to auto-fix lint issues.
    pub lint_fix: BuildCommand,
    /// Command to format code.
    pub fmt: BuildCommand,
    /// Command to check formatting.
    pub fmt_check: BuildCommand,
    /// Command to type-check without full build.
    pub check: BuildCommand,
    /// Command to generate CI YAML.
    pub ci_yaml: BuildCommand,
    /// Command to regenerate tests from DAGs.
    pub testgen: BuildCommand,
    /// Command to generate bootstrap artifacts.
    pub bootstrap: BuildCommand,
    /// Command to generate pragma artifacts.
    pub pragma: BuildCommand,
    /// Command to generate Makefile.
    pub makegen: BuildCommand,
}

impl BuildConfig {
    /// Default cargo-based build config.
    pub fn cargo() -> Self {
        let w = Warnings::Deny;
        let codegen_inv = CargoInvocation::composed("codegen", "dag");
        let codegen_dag_inv = CargoInvocation::composed("codegen-dag", "dag");
        let c = |cmd: CargoCommand| BuildCommand::Cargo(cmd);
        Self {
            build_system: BuildSystem::Cargo,
            use_dag_entrypoints: false,
            warnings: w,
            ensure_codegen: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .args(BinaryArgs::codegen(CodegenSubcommand::Codegen))
                .warnings(w)),
            codegen: c(CargoCommand::new(Subcommand::Run(codegen_dag_inv.clone())).warnings(w)),
            daggen: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .args(BinaryArgs::codegen(CodegenSubcommand::Daggen))
                .warnings(w)),
            build: c(CargoCommand::new(Subcommand::Build)
                .all_targets()
                .warnings(w)),
            test: c(CargoCommand::new(Subcommand::Test).warnings(w)),
            lint: c(CargoCommand::new(Subcommand::Clippy)
                .all_targets()
                .warnings(w)),
            lint_fix: c(CargoCommand::new(Subcommand::Clippy)
                .fix()
                .workspace()
                .allow_dirty()
                .allow_staged()
                .warnings(w)),
            fmt: c(CargoCommand::new(Subcommand::Fmt)),
            fmt_check: c(CargoCommand::new(Subcommand::Fmt).check()),
            check: c(CargoCommand::new(Subcommand::Check)
                .all_targets()
                .warnings(w)),
            ci_yaml: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .args(BinaryArgs::codegen(CodegenSubcommand::Cigen))
                .warnings(w)),
            testgen: c(CargoCommand::new(Subcommand::Run(CargoInvocation::composed(
                "testgen", "dag",
            )))
            .warnings(w)),
            bootstrap: c(CargoCommand::new(Subcommand::Run(CargoInvocation::composed(
                "bootstrap",
                "dag",
            )))
            .warnings(w)),
            pragma: c(CargoCommand::new(Subcommand::Run(CargoInvocation::composed(
                "pragma", "dag",
            )))
            .warnings(w)),
            makegen: c(CargoCommand::new(Subcommand::Run(CargoInvocation::composed(
                "makegen", "dag",
            )))
            .warnings(w)),
        }
    }

    /// Buck2-based build config (delta from `cargo()`).
    pub fn buck2() -> Self {
        let sh =
            |parts: &[&str]| BuildCommand::Shell(parts.iter().map(|s| s.to_string()).collect());
        let mut config = Self::cargo();
        config.build_system = BuildSystem::Buck2;
        config.build = sh(&["buck2", "build", "//..."]);
        config.test = sh(&["buck2", "test", "//..."]);
        config.lint = sh(&["buck2", "run", "//tools:clippy"]);
        config.check = sh(&["buck2", "build", "//..."]);
        config
    }
}

/// Get the default build config (cargo-based).
pub fn default_build_config() -> BuildConfig {
    BuildConfig::cargo()
}

// ============================================================================
// Tool information
// ============================================================================

/// An extra target that combines the main tool with additional commands.
#[derive(Debug, Clone)]
pub struct ExtraTarget {
    /// Target name suffix (e.g., "serve" becomes "viz-serve").
    pub suffix: String,
    /// Description for help text.
    pub description: String,
    /// Shell commands to run after the main tool.
    pub post_commands: Vec<String>,
}

/// An entrypoint parameter that becomes a Make variable.
#[derive(Debug, Clone)]
pub struct EntrypointParam {
    /// DAG port name (e.g., "repo_path").
    pub port_name: String,
    /// Make variable name (e.g., "REPO").
    pub make_var: String,
    /// CLI flag (e.g., "--repo").
    pub cli_flag: String,
    /// Type hint for help text.
    pub type_hint: String,
    /// Default value if any.
    pub default: Option<String>,
    /// Whether this param can be repeated (for list types).
    pub repeatable: bool,
}

/// Minimal workflow descriptor for tool targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowKind {
    Tool,
}

/// Workflow descriptor for tool targets.
#[derive(Debug, Clone)]
pub struct WorkflowSpec {
    pub name: String,
    pub description: String,
    pub kind: WorkflowKind,
    pub entrypoints: Vec<EntrypointParam>,
    pub deps: Vec<String>,
    pub live_secrets: Vec<String>,
}

/// Information about a gunbc tool.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// How to invoke this tool via cargo.
    pub invocation: CargoInvocation,
    /// Short name for make target (e.g., "gist").
    pub short_name: String,
    /// Description for help text.
    pub description: String,
    /// Entrypoint parameters (from DAG entrypoints).
    pub entrypoints: Vec<EntrypointParam>,
    /// Extra composite targets.
    pub extra_targets: Vec<ExtraTarget>,
    /// Whether this tool has a declarative DAG definition.
    pub has_declarative_dag: bool,
    /// Whether this tool needs a generated CLI entrypoint.
    pub needs_generated_cli: bool,
    /// Secret environment variables required for live execution.
    pub live_secrets: Vec<String>,
}

impl ToolInfo {
    /// Create a ToolInfo from a codegen ToolDef.
    pub fn from_tool_def(def: &crate::registry::ToolDef) -> Option<Self> {
        let invocation = def.invocation.as_ref()?;
        let mut info = Self {
            invocation: invocation.clone(),
            short_name: def.meta.tool_name.to_string(),
            description: def.meta.description.to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: def.meta.tool_name == "makegen",
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        };

        for ep in &def.entrypoints {
            if let Some(ref make_var) = ep.make_var {
                info.entrypoints.push(EntrypointParam {
                    port_name: ep.port_name.clone(),
                    make_var: make_var.clone(),
                    cli_flag: format!("--{}", ep.flag_name()),
                    type_hint: ep.type_id.to_string(),
                    default: ep.default_value.clone(),
                    repeatable: ep.cardinality.allows_many(),
                });
            }
        }

        Some(info)
    }

    /// The binary name (e.g., "gunbc-ci").
    pub fn binary_name(&self) -> &str {
        &self.invocation.binary
    }

    /// Build a normalized workflow specification for this tool target.
    pub fn workflow_spec(&self) -> WorkflowSpec {
        WorkflowSpec {
            name: self.short_name.clone(),
            description: self.description.clone(),
            kind: WorkflowKind::Tool,
            entrypoints: self.entrypoints.clone(),
            deps: tool_dependency_targets(self),
            live_secrets: self.live_secrets.clone(),
        }
    }
}

fn tool_dependency_targets(tool: &ToolInfo) -> Vec<String> {
    if tool.needs_generated_cli {
        vec!["ensure-codegen".to_string()]
    } else {
        Vec::new()
    }
}

// ============================================================================
// Tool registry
// ============================================================================

/// Registry of all discovered tool targets.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    /// Individual tool targets (gist, deps, etc.).
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

    /// Add a tool only when its short name is not already present.
    pub fn register_if_missing(&mut self, tool: ToolInfo) {
        if self
            .tools
            .iter()
            .any(|existing| existing.short_name == tool.short_name)
        {
            return;
        }
        self.tools.push(tool);
    }

    /// Return a new registry excluding tools whose short_name is in the reserved set.
    ///
    /// Used by `render_makefile` to filter out DSL-discovered tools whose name
    /// collides with a core or meta make target.
    pub fn without_reserved(&self, reserved: &std::collections::BTreeSet<String>) -> Self {
        Self {
            tools: self
                .tools
                .iter()
                .filter(|tool| !reserved.contains(&tool.short_name))
                .cloned()
                .collect(),
        }
    }

    /// Build the default registry with all tools discovered from DSL.
    ///
    /// NOTE: This returns an un-enriched registry (no live_secrets).
    /// Callers that need live_secrets should use the enriched wrapper
    /// in `gunbc_dag::tool_graphs::default_registry_enriched()`.
    pub fn default_registry() -> Result<Self, String> {
        let mut registry = Self::new();

        for tool_def in crate::tool_discovery::discover_tool_defs_from_dsl()? {
            if let Some(tool_info) = ToolInfo::from_tool_def(&tool_def) {
                registry.register_if_missing(tool_info);
            }
        }

        registry
            .tools
            .sort_by(|a, b| a.short_name.cmp(&b.short_name));
        Ok(registry)
    }

    /// Get tools that need CLI codegen.
    pub fn tools_needing_codegen(&self) -> Vec<&ToolInfo> {
        self.tools
            .iter()
            .filter(|tool| tool.needs_generated_cli)
            .collect()
    }

    /// Get tools that have declarative DAG definitions.
    pub fn tools_needing_daggen(&self) -> Vec<&ToolInfo> {
        self.tools
            .iter()
            .filter(|tool| tool.has_declarative_dag)
            .collect()
    }

    /// Whether any codegen is needed.
    pub fn needs_codegen(&self) -> bool {
        !self.tools_needing_codegen().is_empty()
    }

    /// Daggen remains deferred.
    pub fn needs_daggen(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_config_cargo_uses_warning_policy() {
        let config = BuildConfig::cargo();
        assert_eq!(config.build_system, BuildSystem::Cargo);
        assert_eq!(config.warnings, Warnings::Deny);
        assert!(config.build.to_shell().contains("cargo build"));
        assert!(config.test.to_shell().contains("cargo test"));
    }

    #[test]
    fn build_config_buck2_switches_build_commands() {
        let config = BuildConfig::buck2();
        assert_eq!(config.build_system, BuildSystem::Buck2);
        assert!(config.build.to_shell().contains("buck2 build"));
        assert!(config.test.to_shell().contains("buck2 test"));
    }

    #[test]
    fn default_registry_derives_tools_from_dsl() {
        let registry = ToolRegistry::default_registry().expect("registry discovery should succeed");
        assert!(registry.tools.iter().any(|tool| tool.short_name == "deps"));
        assert!(registry
            .tools
            .iter()
            .any(|tool| tool.short_name == "makegen"));
        assert!(registry
            .tools
            .iter()
            .any(|tool| tool.short_name == "pragma"));
    }

    #[test]
    fn default_registry_has_unique_short_names() {
        let registry = ToolRegistry::default_registry().expect("registry discovery should succeed");
        let mut seen = std::collections::BTreeSet::new();
        for tool in &registry.tools {
            assert!(
                seen.insert(tool.short_name.clone()),
                "duplicate tool short_name in makegen registry: {}",
                tool.short_name
            );
        }
    }

    #[test]
    fn tools_needing_codegen_reflects_flag() {
        let mut registry = ToolRegistry::new();
        registry.register(ToolInfo {
            invocation: CargoInvocation::composed("generated", "dag"),
            short_name: "generated".to_string(),
            description: "Generated tool".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        });
        registry.register(ToolInfo {
            invocation: CargoInvocation::composed("manual", "dag"),
            short_name: "manual".to_string(),
            description: "Manual tool".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: false,
            live_secrets: Vec::new(),
        });
        let codegen = registry.tools_needing_codegen();
        assert_eq!(codegen.len(), 1);
        assert_eq!(codegen[0].short_name, "generated");
    }

    #[test]
    fn workflow_spec_for_tool_uses_codegen_dependency_when_needed() {
        let generated = ToolInfo {
            invocation: CargoInvocation::composed("alpha", "dag"),
            short_name: "alpha".to_string(),
            description: "Alpha".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        };
        let manual = ToolInfo {
            invocation: CargoInvocation::composed("beta", "dag"),
            short_name: "beta".to_string(),
            description: "Beta".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: false,
            live_secrets: Vec::new(),
        };
        let generated_spec = generated.workflow_spec();
        let manual_spec = manual.workflow_spec();
        assert_eq!(generated_spec.deps, vec!["ensure-codegen"]);
        assert!(manual_spec.deps.is_empty());
    }
}
