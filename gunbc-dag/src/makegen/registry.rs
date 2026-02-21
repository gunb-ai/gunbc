//! Tool registry for makegen.
//!
//! Tool targets are derived from the codegen registry (`derive_tool_defs()`) — adding
//! a tool with `.invocation()` there automatically gives it a Makefile target.
//! Only tools that can't be in codegen (like `ci`, which is the bootstrap tool)
//! are registered manually here.
//!
//! Meta targets (test, check, fmt, clippy) declare resource needs.
//!
//! # BuildConfig
//!
//! The `BuildConfig` struct is the single source of truth for all build/test/lint
//! commands. This eliminates duplicate hardcoded commands across the codebase.

use crate::resources::{
    compiled_code_resource_id, deps_config_resource_id, generated_cli_resource_id,
    generated_tests_resource_id, gitignore_resource_id, makefile_resource_id,
    pragma_config_resource_id, verified_artifacts_resource_id,
};
use crate::WorkspaceBinary;
use gunbc_infra::ResourceId;
use gunbc_ir::cargo::{BinaryArgs, CargoCommand, CodegenSubcommand, Subcommand, Warnings};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::ShellRequest;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

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

/// A build command that can be either a structured cargo command or a raw
/// shell command (for non-cargo build tools like buck2).
///
/// `Cargo` variants carry full semantic information (subcommand, flags,
/// warning policy) and render themselves via [`CargoCommand`]. `Shell`
/// variants are opaque command vectors for tools outside the cargo model.
#[derive(Debug, Clone)]
pub enum BuildCommand {
    /// A structured cargo command with full semantic rendering.
    Cargo(CargoCommand),
    /// A raw shell command (for non-cargo tools like buck2).
    Shell(Vec<String>),
}

impl BuildCommand {
    /// Render as a shell command string.
    pub fn to_shell(&self) -> String {
        match self {
            BuildCommand::Cargo(cmd) => cmd.to_shell_with_env(),
            BuildCommand::Shell(parts) => parts.join(" "),
        }
    }

    /// Convert to a `ShellRequest` for transport execution.
    pub fn to_shell_request(&self) -> ShellRequest {
        match self {
            BuildCommand::Cargo(cmd) => cmd.to_shell_request(),
            BuildCommand::Shell(parts) => {
                let (command, args) = parts.split_first().expect("empty command");
                ShellRequest::new(command).args(args.iter().cloned())
            }
        }
    }
}

/// Unified build system configuration.
/// Single source of truth for build/test/lint operations.
///
/// Commands are modeled as [`BuildCommand`] values — either structured
/// [`CargoCommand`] values (for cargo-based builds) or raw shell commands
/// (for non-cargo tools like buck2). The warning policy is applied to all
/// cargo compilation subcommands.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Build system (cargo, buck2)
    pub build_system: BuildSystem,
    /// Whether Makefile targets should use DAG entrypoints (e.g., gunbc-build)
    /// instead of raw cargo/buck2 commands.
    pub use_dag_entrypoints: bool,
    /// Repo-level warning policy (applied to all compilation commands).
    pub warnings: Warnings,
    /// Command to ensure codegen outputs exist (bootstrap-safe)
    pub ensure_codegen: BuildCommand,
    /// Command to run codegen
    pub codegen: BuildCommand,
    /// Command to run daggen
    pub daggen: BuildCommand,
    /// Command to build all targets
    pub build: BuildCommand,
    /// Command to run tests
    pub test: BuildCommand,
    /// Command to run linter
    pub lint: BuildCommand,
    /// Command to auto-fix lint issues (cargo clippy --fix)
    pub lint_fix: BuildCommand,
    /// Command to format code
    pub fmt: BuildCommand,
    /// Command to check formatting
    pub fmt_check: BuildCommand,
    /// Command to type-check without full build
    pub check: BuildCommand,
    /// Command to generate CI YAML
    pub ci_yaml: BuildCommand,
    /// Command to regenerate tests from DAGs
    pub testgen: BuildCommand,
    /// Command to ensure deps.toml is up to date (`--mode=ensure`)
    pub deps_config_ensure: BuildCommand,
    /// Command to check if deps.toml is stale (`--mode=verify`)
    pub deps_config_check: BuildCommand,
    /// Command to generate bootstrap artifacts (Makefile + .gitignore)
    pub bootstrap: BuildCommand,
    /// Command to generate pragma artifacts (clippy.toml + allowlists)
    pub pragma: BuildCommand,
    /// Command to check if generated tests are stale (`--mode=verify`)
    pub testgen_check: BuildCommand,
    /// Command to check if generated Makefile is stale (`--mode=verify`)
    pub makegen_check: BuildCommand,
    /// Command to check if generated bootstrap files are stale (`--mode=verify`)
    pub bootstrap_check: BuildCommand,
    /// Command to check if generated pragma/clippy config is stale (`--mode=verify`)
    pub pragma_check: BuildCommand,
    /// Command to ensure generated tests are up to date (`--mode=ensure`)
    pub testgen_ensure: BuildCommand,
    /// Command to ensure generated Makefile is up to date (`--mode=ensure`)
    pub makegen_ensure: BuildCommand,
    /// Command to ensure generated bootstrap files are up to date (`--mode=ensure`)
    pub bootstrap_ensure: BuildCommand,
    /// Command to ensure generated pragma/clippy config is up to date (`--mode=ensure`)
    pub pragma_ensure: BuildCommand,
}

impl BuildConfig {
    /// Default cargo-based build config.
    ///
    /// Warning policy is `Deny` — all warnings are promoted to errors.
    /// This is the repo's standard policy for both CI and local builds.
    pub fn cargo() -> Self {
        let w = Warnings::Deny;
        let codegen_inv = WorkspaceBinary::Codegen.invocation();
        let codegen_dag_inv = WorkspaceBinary::CodegenDag.invocation();
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
            testgen: c(
                CargoCommand::new(Subcommand::Run(WorkspaceBinary::Testgen.invocation()))
                    .warnings(w),
            ),
            deps_config_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::DepsConfig.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            deps_config_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::DepsConfig.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            bootstrap: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Bootstrap.invocation(),
            ))
            .warnings(w)),
            pragma: c(
                CargoCommand::new(Subcommand::Run(WorkspaceBinary::Pragma.invocation()))
                    .warnings(w),
            ),
            testgen_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Testgen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            makegen_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Makegen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            bootstrap_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Bootstrap.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            pragma_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Pragma.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            testgen_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Testgen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            makegen_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Makegen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            bootstrap_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Bootstrap.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            pragma_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Pragma.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
        }
    }

    /// Cargo build config that routes build/test/lint through DAG entrypoints.
    ///
    /// Used for Makefile generation so `make build/test/clippy` are graph-driven,
    /// while internal ops (BuildOp/CI) still use raw cargo commands via `cargo()`.
    pub fn cargo_entrypoints() -> Self {
        let mut config = Self::cargo();
        let build_inv = WorkspaceBinary::Build.invocation();
        let entry = BuildCommand::Cargo(
            CargoCommand::new(Subcommand::Run(build_inv)).warnings(config.warnings),
        );

        config.build = entry.clone();
        config.test = entry.clone();
        config.lint = entry;
        config.use_dag_entrypoints = true;
        config
    }

    /// Buck2-based build config (for future use).
    ///
    /// Uses `BuildCommand::Shell` for buck2-native commands (build, test, lint, check)
    /// and `BuildCommand::Cargo` for commands that still use cargo (codegen, fmt).
    pub fn buck2() -> Self {
        let w = Warnings::Deny;
        let codegen_inv = WorkspaceBinary::Codegen.invocation();
        let codegen_dag_inv = WorkspaceBinary::CodegenDag.invocation();
        let c = |cmd: CargoCommand| BuildCommand::Cargo(cmd);
        let sh =
            |parts: &[&str]| BuildCommand::Shell(parts.iter().map(|s| s.to_string()).collect());
        Self {
            build_system: BuildSystem::Buck2,
            use_dag_entrypoints: false,
            warnings: w,
            ensure_codegen: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .args(BinaryArgs::codegen(CodegenSubcommand::Codegen))
                .warnings(w)),
            codegen: c(CargoCommand::new(Subcommand::Run(codegen_dag_inv.clone())).warnings(w)),
            daggen: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .args(BinaryArgs::codegen(CodegenSubcommand::Daggen))
                .warnings(w)),
            // Buck2-native commands
            build: sh(&["buck2", "build", "//..."]),
            test: sh(&["buck2", "test", "//..."]),
            lint: sh(&["buck2", "run", "//tools:clippy"]),
            // lint-fix still uses cargo (buck2 doesn't have an equivalent)
            lint_fix: c(CargoCommand::new(Subcommand::Clippy)
                .fix()
                .workspace()
                .allow_dirty()
                .allow_staged()
                .warnings(w)),
            // fmt stays cargo (buck2 delegates to cargo fmt)
            fmt: c(CargoCommand::new(Subcommand::Fmt)),
            fmt_check: c(CargoCommand::new(Subcommand::Fmt).check()),
            check: sh(&["buck2", "build", "//..."]),
            ci_yaml: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .args(BinaryArgs::codegen(CodegenSubcommand::Cigen))
                .warnings(w)),
            // testgen uses cargo (no buck2 equivalent yet)
            testgen: c(
                CargoCommand::new(Subcommand::Run(WorkspaceBinary::Testgen.invocation()))
                    .warnings(w),
            ),
            deps_config_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::DepsConfig.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            deps_config_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::DepsConfig.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            bootstrap: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Bootstrap.invocation(),
            ))
            .warnings(w)),
            pragma: c(
                CargoCommand::new(Subcommand::Run(WorkspaceBinary::Pragma.invocation()))
                    .warnings(w),
            ),
            testgen_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Testgen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            makegen_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Makegen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            bootstrap_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Bootstrap.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            pragma_check: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Pragma.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Verify))
            .warnings(w)),
            testgen_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Testgen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            makegen_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Makegen.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            bootstrap_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Bootstrap.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
            pragma_ensure: c(CargoCommand::new(Subcommand::Run(
                WorkspaceBinary::Pragma.invocation(),
            ))
            .args(BinaryArgs::with_mode(ExecMode::Ensure))
            .warnings(w)),
        }
    }

    /// Get the codegen command as a shell string (for Makefile generation).
    pub fn codegen_shell(&self) -> String {
        format!("@{}", self.codegen.to_shell())
    }

    /// Get the ensure-codegen command as a shell string.
    ///
    /// TODO(WF15): Replace `cargo run` dispatch with direct pre-built binary
    /// invocation once binary freshness is planner-managed. Codegen freshness
    /// will be a ledger-backed keyed unit, not a Make subprocess.
    pub fn ensure_codegen_shell(&self) -> String {
        format!("@{}", self.ensure_codegen.to_shell())
    }

    /// Get the build command as a shell string.
    pub fn build_shell(&self) -> String {
        format!("@{}", self.build.to_shell())
    }

    /// Get the test command as a shell string.
    pub fn test_shell(&self) -> String {
        format!("@{}", self.test.to_shell())
    }

    /// Get the lint command as a shell string.
    pub fn lint_shell(&self) -> String {
        format!("@{}", self.lint.to_shell())
    }

    /// Get the lint-fix command as a shell string.
    pub fn lint_fix_shell(&self) -> String {
        format!("@{}", self.lint_fix.to_shell())
    }

    /// Get the fmt command as a shell string.
    pub fn fmt_shell(&self) -> String {
        format!("@{}", self.fmt.to_shell())
    }

    /// Get the fmt-check command as a shell string.
    pub fn fmt_check_shell(&self) -> String {
        format!("@{}", self.fmt_check.to_shell())
    }

    /// Get the check command as a shell string.
    pub fn check_shell(&self) -> String {
        format!("@{}", self.check.to_shell())
    }

    /// Get the CI YAML generation command as a shell string.
    pub fn ci_yaml_shell(&self) -> String {
        format!("@{}", self.ci_yaml.to_shell())
    }

    /// Get the testgen command as a shell string.
    pub fn testgen_shell(&self) -> String {
        format!("@{}", self.testgen.to_shell())
    }

    /// Get the pragma command as a shell string.
    pub fn pragma_shell(&self) -> String {
        format!("@{}", self.pragma.to_shell())
    }

    /// Get the deps-config-check command as a shell string.
    pub fn deps_config_check_shell(&self) -> String {
        format!("@{}", self.deps_config_check.to_shell())
    }

    /// Get the deps-config-ensure command as a shell string.
    pub fn deps_config_ensure_shell(&self) -> String {
        format!("@{}", self.deps_config_ensure.to_shell())
    }

    /// Get the testgen-check command as a shell string.
    pub fn testgen_check_shell(&self) -> String {
        format!("@{}", self.testgen_check.to_shell())
    }

    /// Get the makegen-check command as a shell string.
    pub fn makegen_check_shell(&self) -> String {
        format!("@{}", self.makegen_check.to_shell())
    }

    /// Get the bootstrap-check command as a shell string.
    pub fn bootstrap_check_shell(&self) -> String {
        format!("@{}", self.bootstrap_check.to_shell())
    }

    /// Get the pragma-check command as a shell string.
    pub fn pragma_check_shell(&self) -> String {
        format!("@{}", self.pragma_check.to_shell())
    }

    /// Get the testgen-ensure command as a shell string.
    pub fn testgen_ensure_shell(&self) -> String {
        format!("@{}", self.testgen_ensure.to_shell())
    }

    /// Get the makegen-ensure command as a shell string.
    pub fn makegen_ensure_shell(&self) -> String {
        format!("@{}", self.makegen_ensure.to_shell())
    }

    /// Get the bootstrap-ensure command as a shell string.
    pub fn bootstrap_ensure_shell(&self) -> String {
        format!("@{}", self.bootstrap_ensure.to_shell())
    }

    /// Get the pragma-ensure command as a shell string.
    pub fn pragma_ensure_shell(&self) -> String {
        format!("@{}", self.pragma_ensure.to_shell())
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
    /// How to invoke this tool via cargo.
    pub invocation: gunbc_ir::CargoInvocation,
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
    /// Whether this tool needs a generated CLI entrypoint (codegen dependency).
    /// False for hand-written binaries (ci, pragma, build-all).
    pub needs_generated_cli: bool,
    /// Secret environment variables required for live execution.
    /// Derived from `DagSpecTestgen` registrations. Empty if no secrets needed.
    pub live_secrets: Vec<String>,
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
    /// Create a tool in its own package (e.g., `cargo run -p gunbc-gist`).
    ///
    /// Prefer [`Self::standalone`] which composes the binary name from a component.
    pub fn new(
        binary: impl Into<String>,
        short_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            invocation: gunbc_ir::CargoInvocation::new(binary),
            short_name: short_name.into(),
            description: description.into(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        }
    }

    /// Create a tool that's a binary inside another package
    /// (e.g., `cargo run -p gunbc-dag --bin gunbc-ci`).
    ///
    /// Prefer [`Self::composed`] which composes both names from components.
    pub fn in_package(
        binary: impl Into<String>,
        package: impl Into<String>,
        short_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            invocation: gunbc_ir::CargoInvocation::in_package(binary, package),
            short_name: short_name.into(),
            description: description.into(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        }
    }

    /// Create a standalone tool from its component name.
    ///
    /// The binary name is composed as `{PREFIX}-{component}`, and the
    /// component is used as the short name for make targets.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ToolInfo::standalone("gist", "Create a GitHub gist")
    /// // binary: "gunbc-gist", short_name: "gist"
    /// ```
    pub fn standalone(component: &str, description: impl Into<String>) -> Self {
        Self {
            invocation: gunbc_ir::CargoInvocation::standalone(component),
            short_name: component.to_string(),
            description: description.into(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        }
    }

    /// Create a tool that is a repo-local workspace binary.
    pub fn workspace(binary: WorkspaceBinary, description: impl Into<String>) -> Self {
        Self {
            invocation: binary.invocation(),
            short_name: binary.component().to_string(),
            description: description.into(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        }
    }

    /// Create a tool that lives inside another package, both composed
    /// from component names.
    ///
    /// The binary is `{PREFIX}-{component}`, the package is
    /// `{PREFIX}-{package_component}`, and the component is used as
    /// the short name.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ToolInfo::composed("ci", "dag", "Run CI pipeline")
    /// // binary: "gunbc-ci", package: "gunbc-dag", short_name: "ci"
    /// ```
    pub fn composed(
        component: &str,
        package_component: &str,
        description: impl Into<String>,
    ) -> Self {
        Self {
            invocation: gunbc_ir::CargoInvocation::composed(component, package_component),
            short_name: component.to_string(),
            description: description.into(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        }
    }

    /// Create a ToolInfo from a codegen ToolDef.
    ///
    /// Returns `None` if the tool has no invocation (no runnable binary).
    /// Entrypoints with `make_var` set are converted to `EntrypointParam`s;
    /// CLI-only entrypoints (no make_var) are omitted from the Makefile.
    pub fn from_tool_def(def: &gunbc_codegen::registry::ToolDef) -> Option<Self> {
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

        // Convert entrypoints that have make_var set
        for ep in &def.entrypoints {
            if let Some(ref make_var) = ep.make_var {
                info.entrypoints.push(EntrypointParam {
                    port_name: ep.port_name.clone(),
                    make_var: make_var.clone(),
                    // Use the actual CLI flag name (matches generated CLI)
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

    /// Mark this tool as having a hand-written main.rs (no generated CLI).
    pub fn manual(mut self) -> Self {
        self.needs_generated_cli = false;
        self
    }

    /// Build a normalized workflow specification for this tool target.
    pub fn workflow_spec(&self, config: &BuildConfig) -> WorkflowSpec {
        WorkflowSpec {
            name: self.short_name.clone(),
            description: self.description.clone(),
            kind: WorkflowKind::Tool,
            entrypoints: self.entrypoints.clone(),
            deps: tool_dependency_targets(self, config),
            resources: Vec::new(),
            live_secrets: self.live_secrets.clone(),
        }
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

/// Workflow category in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowKind {
    Core,
    Tool,
    Meta,
}

/// Normalized workflow descriptor used by workflow-level registries/renderers.
///
/// Captures the three pieces needed for orchestration:
/// - `entrypoints`: externally configurable inputs
/// - `deps`: target-level build dependencies
/// - `resources`: logical resource requirements (for meta workflows)
#[derive(Debug, Clone)]
pub struct WorkflowSpec {
    pub name: String,
    pub description: String,
    pub kind: WorkflowKind,
    pub entrypoints: Vec<EntrypointParam>,
    pub deps: Vec<String>,
    pub resources: Vec<ResourceNeed>,
    /// Secret environment variables required for live execution.
    pub live_secrets: Vec<String>,
}

impl WorkflowSpec {
    #[allow(dead_code)]
    fn core(name: &str, description: &str, deps: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            kind: WorkflowKind::Core,
            entrypoints: Vec::new(),
            deps: deps.iter().map(|dep| (*dep).to_string()).collect(),
            resources: Vec::new(),
            live_secrets: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn with_resource(mut self, id: ResourceId, base_mode: ExecMode) -> Self {
        self.resources.push(ResourceNeed { id, base_mode });
        self
    }
}

fn tool_dependency_targets(tool: &ToolInfo, config: &BuildConfig) -> Vec<String> {
    let _ = (tool, config);
    Vec::new()
}

// ============================================================================
// Meta Targets - Resource-based dependency model
// ============================================================================

/// What resource a meta target needs, and in what mode for the base variant.
///
/// Base mode determines the dependency for the verify-only (CI) variant:
/// - `Ensure` → use the ensure target (e.g., "testgen")
/// - `Verify` → use the verify target (e.g., "pragma-check")
///
/// Fix variants always use the ensure target (fix mode = ensure everything).
#[derive(Debug, Clone)]
pub struct ResourceNeed {
    pub id: ResourceId,
    /// Base target mode: Verify = check-only dep, Ensure = regenerate dep
    pub base_mode: ExecMode,
}

/// Maps ResourceId → Make target names for verify and ensure modes.
///
/// This decouples MetaTarget declarations (what resources they need) from
/// the concrete Make target names (how those resources are provided).
pub struct ResourceTargetMap {
    entries: Vec<ResourceTargetEntry>,
}

struct ResourceTargetEntry {
    id: ResourceId,
    ensure_target: String,
    /// Verify target. Always explicit (no implicit fallback behavior).
    verify_target: String,
}

impl ResourceTargetMap {
    /// Resolve a ResourceId + mode to a Make target name.
    ///
    /// - `Ensure` → ensure_target
    /// - `Verify` → verify_target
    pub fn resolve(&self, id: &ResourceId, mode: ExecMode) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.id == *id)
            .map(|e| match mode {
                ExecMode::Ensure => e.ensure_target.as_str(),
                ExecMode::Verify => e.verify_target.as_str(),
            })
    }

    /// Build the default resource target map.
    ///
    /// This maps ResourceIds used by meta targets to their concrete Make
    /// target names. The mapping accounts for `use_dag_entrypoints` which
    /// changes how `compiled_code` is provided.
    pub fn default_map(config: &BuildConfig) -> Self {
        let compiled_code_target = if config.use_dag_entrypoints {
            "codegen"
        } else {
            "build"
        };

        Self {
            entries: vec![
                ResourceTargetEntry {
                    id: generated_cli_resource_id(),
                    ensure_target: "ensure-codegen".to_string(),
                    // Explicitly ensure-only in verify workflows (no check target exists yet).
                    verify_target: "ensure-codegen".to_string(),
                },
                ResourceTargetEntry {
                    id: generated_tests_resource_id(),
                    ensure_target: "testgen".to_string(),
                    verify_target: "testgen-check".to_string(),
                },
                ResourceTargetEntry {
                    id: pragma_config_resource_id(),
                    ensure_target: "pragma".to_string(),
                    verify_target: "pragma-check".to_string(),
                },
                ResourceTargetEntry {
                    id: compiled_code_resource_id(),
                    ensure_target: compiled_code_target.to_string(),
                    // Explicitly ensure-only in verify workflows (no check target exists yet).
                    verify_target: compiled_code_target.to_string(),
                },
                ResourceTargetEntry {
                    id: verified_artifacts_resource_id(),
                    ensure_target: "verify-fix".to_string(),
                    verify_target: "verify".to_string(),
                },
                ResourceTargetEntry {
                    id: deps_config_resource_id(),
                    ensure_target: "deps-config".to_string(),
                    verify_target: "deps-config-check".to_string(),
                },
                ResourceTargetEntry {
                    id: makefile_resource_id(),
                    ensure_target: "makegen".to_string(),
                    verify_target: "makegen-check".to_string(),
                },
                ResourceTargetEntry {
                    id: gitignore_resource_id(),
                    ensure_target: "bootstrap".to_string(),
                    verify_target: "bootstrap-check".to_string(),
                },
            ],
        }
    }
}

/// Typed reference to a fix alias target.
///
/// Fix aliases are meta-target variants used as prerequisites for `-fix` targets.
/// Using an enum instead of raw strings ensures that renaming a fix alias
/// causes a compile error rather than a silent runtime mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixAlias {
    /// `fmt-fix` — apply formatting
    FmtFix,
    /// `lint-fix` — auto-fix lint issues
    LintFix,
}

impl FixAlias {
    /// Resolve to the Make target name.
    pub fn target_name(self) -> &'static str {
        match self {
            FixAlias::FmtFix => "fmt-fix",
            FixAlias::LintFix => "lint-fix",
        }
    }
}

/// Which BuildConfig field to use for a meta target.
///
/// This allows MetaTarget to reference commands from BuildConfig
/// rather than storing them directly, ensuring a single source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    /// Use test_command
    Test,
    /// Use test_command filtered to integration-oriented tests.
    TestIntegration,
    /// Use test_command filtered to external/live-flow tests.
    TestExternal,
    /// Use lint_command
    Lint,
    /// Use fmt_command
    Fmt,
    /// Use check_command
    Check,
    /// Use build_command
    Build,
    /// Use ci_yaml_command
    CiYaml,
}

impl ConfigField {
    fn with_test_filter(cmd: String, filter: &str) -> String {
        let base = cmd.strip_prefix('@').unwrap_or(&cmd);
        format!("@{} {}", base, filter)
    }

    /// Get the command from BuildConfig for this field.
    pub fn get_command(&self, config: &BuildConfig) -> String {
        match self {
            ConfigField::Test => config.test_shell(),
            ConfigField::TestIntegration => {
                Self::with_test_filter(config.test_shell(), "integration")
            }
            ConfigField::TestExternal => Self::with_test_filter(config.test_shell(), "live_flow"),
            ConfigField::Lint => config.lint_shell(),
            ConfigField::Fmt => config.fmt_shell(),
            ConfigField::Check => config.check_shell(),
            ConfigField::Build => config.build_shell(),
            ConfigField::CiYaml => config.ci_yaml_shell(),
        }
    }

    /// Get the check variant command if applicable.
    pub fn get_check_command(&self, config: &BuildConfig) -> Option<String> {
        match self {
            ConfigField::Fmt => Some(config.fmt_check_shell()),
            _ => None,
        }
    }

    /// Get the fix variant command if applicable.
    /// Currently only Lint has a dedicated fix command (clippy --fix).
    pub fn get_fix_command(&self, config: &BuildConfig) -> Option<String> {
        match self {
            ConfigField::Lint => Some(config.lint_fix_shell()),
            // For Fmt, the "fix" is just the regular fmt command
            ConfigField::Fmt => Some(config.fmt_shell()),
            _ => None,
        }
    }
}

/// A meta target that composes resource needs + a specific operation.
///
/// Meta targets are holistic targets like `test`, `check`, `fmt`, `clippy`
/// that developers use frequently. They declare *what resources they need*
/// via `ResourceNeed`, and the renderer resolves those to Make target names
/// via `ResourceTargetMap`.
///
/// Commands are referenced via `ConfigField` to ensure BuildConfig
/// remains the single source of truth.
///
/// # Dev UX Convention (from the-gunbai)
///
/// - `make <target>` - verify by default (CI-safe, fails on issues)
/// - `make <target>-fix` - auto-fix then verify (for dev)
///
/// Note: some resources can map Ensure to a fix target (e.g. generated artifacts),
/// so `make test` may auto-repair drift before running tests.
#[derive(Debug, Clone)]
pub struct MetaTarget {
    /// Target name (e.g., "test")
    pub name: String,
    /// Description for help text
    pub description: String,
    /// Which BuildConfig field to use for the command
    pub config_field: ConfigField,
    /// Optional command prefix (e.g., env vars)
    pub command_prefix: Option<String>,
    /// Whether this target has a check variant (e.g., fmt-check)
    pub has_check_variant: bool,
    /// Whether this target has a fix variant (e.g., test-fix, clippy-fix)
    /// Following the-gunbai convention: <target>-fix auto-fixes before running
    pub has_fix_variant: bool,
    /// Resources this target needs, with base-mode for each.
    pub resources: Vec<ResourceNeed>,
    /// Prerequisites for the fix variant (e.g., [FmtFix, LintFix] for test-fix).
    /// These targets are run before the main command in the -fix variant.
    pub fix_prerequisites: Vec<FixAlias>,
}

impl MetaTarget {
    /// Create a new meta target using ConfigField.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        config_field: ConfigField,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            config_field,
            command_prefix: None,
            has_check_variant: false,
            has_fix_variant: false,
            resources: Vec::new(),
            fix_prerequisites: Vec::new(),
        }
    }

    /// Declare a resource need with a base-mode.
    ///
    /// The base mode determines what Make target is used for the verify variant:
    /// - `Ensure` → always use the ensure target (e.g., "build")
    /// - `Verify` → use the verify target (e.g., "pragma-check")
    ///
    /// Fix variants always resolve all resources in Ensure mode.
    pub fn needs(mut self, id: ResourceId, base_mode: ExecMode) -> Self {
        self.resources.push(ResourceNeed { id, base_mode });
        self
    }

    /// Mark this target as having a check variant (e.g., fmt-check).
    pub fn with_check_variant(mut self) -> Self {
        self.has_check_variant = true;
        self
    }

    /// Mark this target as having a fix variant (e.g., test-fix, clippy-fix).
    ///
    /// The fix variant runs the specified prerequisites before the main command.
    /// Following the-gunbai convention:
    /// - `make test` - verify by default (CI-safe)
    /// - `make test-fix` - auto-fix (fmt + lint) then verify
    ///
    /// Some resources can map Ensure to a fix target (e.g. verify-fix for
    /// generated artifacts), so `make test` may auto-repair drift.
    pub fn with_fix_variant(mut self, prerequisites: Vec<FixAlias>) -> Self {
        self.has_fix_variant = true;
        self.fix_prerequisites = prerequisites;
        self
    }

    /// Set a command prefix (e.g., env vars) for this meta target.
    pub fn with_command_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.command_prefix = Some(prefix.into());
        self
    }

    /// Get the command for this meta target from BuildConfig.
    pub fn get_command(&self, config: &BuildConfig) -> String {
        self.apply_prefix(self.config_field.get_command(config))
    }

    /// Get the check command for this meta target from BuildConfig.
    pub fn get_check_command(&self, config: &BuildConfig) -> Option<String> {
        if self.has_check_variant {
            self.config_field
                .get_check_command(config)
                .map(|cmd| self.apply_prefix(cmd))
        } else {
            None
        }
    }

    /// Get the fix command for this meta target from BuildConfig.
    /// Returns the dedicated fix command if available, otherwise the regular command.
    pub fn get_fix_command(&self, config: &BuildConfig) -> Option<String> {
        if self.has_fix_variant {
            // Try to get dedicated fix command, fall back to regular command
            self.config_field
                .get_fix_command(config)
                .or_else(|| Some(self.config_field.get_command(config)))
                .map(|cmd| self.apply_prefix(cmd))
        } else {
            None
        }
    }

    fn apply_prefix(&self, cmd: String) -> String {
        match &self.command_prefix {
            Some(prefix) => {
                let base = cmd.strip_prefix('@').unwrap_or(&cmd);
                let prefix = prefix.trim();
                if prefix.is_empty() {
                    cmd
                } else {
                    format!("@{} {}", prefix, base)
                }
            }
            None => cmd,
        }
    }

    /// Build a normalized workflow specification for this meta target.
    pub fn workflow_spec(&self, res_map: &ResourceTargetMap) -> WorkflowSpec {
        let deps = self
            .resources
            .iter()
            .map(|need| {
                res_map
                    .resolve(&need.id, need.base_mode)
                    .unwrap_or_else(|| {
                        panic!(
                            "missing resource target mapping for {:?} ({:?}) in meta target '{}'",
                            need.id, need.base_mode, self.name
                        )
                    })
                    .to_string()
            })
            .collect();

        WorkflowSpec {
            name: self.name.clone(),
            description: self.description.clone(),
            kind: WorkflowKind::Meta,
            entrypoints: Vec::new(),
            deps,
            resources: self.resources.clone(),
            live_secrets: Vec::new(),
        }
    }
}

/// Get the default meta targets.
///
/// # Dev UX Convention (from the-gunbai)
///
/// - `make <target>` - verify by default (CI-safe, fails on issues)
/// - `make <target>-fix` - auto-fix then verify (for dev)
///
/// Examples:
/// - `make test` runs tests (auto-repairs generated artifacts)
/// - `make test-fix` runs fmt-fix + lint-fix, then tests
pub fn default_meta_targets() -> Vec<MetaTarget> {
    vec![
        // test - run all tests (requires full build + verify)
        // build already includes testgen, so no separate generated_tests dependency needed
        // test-fix: fmt-fix + lint-fix first, then test
        MetaTarget::new("test", "Run tests (<=S)", ConfigField::Test)
            .needs(compiled_code_resource_id(), ExecMode::Ensure)
            .needs(verified_artifacts_resource_id(), ExecMode::Ensure)
            .with_fix_variant(vec![FixAlias::FmtFix, FixAlias::LintFix]),
        // test-integration - run integration-oriented test subset.
        MetaTarget::new(
            "test-integration",
            "Run integration-focused tests",
            ConfigField::TestIntegration,
        )
        .with_command_prefix("GUNBC_TEST_MAX_COST=XL")
        .needs(compiled_code_resource_id(), ExecMode::Ensure)
        .needs(verified_artifacts_resource_id(), ExecMode::Ensure),
        // test-external - run external/live-flow test subset.
        MetaTarget::new(
            "test-external",
            "Run external/live-flow tests",
            ConfigField::TestExternal,
        )
        .with_command_prefix("GUNBC_TEST_MAX_COST=XL")
        .needs(compiled_code_resource_id(), ExecMode::Ensure)
        .needs(verified_artifacts_resource_id(), ExecMode::Ensure),
        // check - type check without building (requires codegen + pragma)
        // check-fix: fmt-fix first, then check
        MetaTarget::new("check", "Type check all targets", ConfigField::Check)
            .needs(generated_cli_resource_id(), ExecMode::Ensure)
            .needs(pragma_config_resource_id(), ExecMode::Verify)
            .with_fix_variant(vec![FixAlias::FmtFix]),
        // clippy - run linter (requires codegen + pragma)
        // clippy-fix: uses cargo clippy --fix (auto-fix where possible)
        MetaTarget::new("clippy", "Run clippy linter", ConfigField::Lint)
            .needs(generated_cli_resource_id(), ExecMode::Ensure)
            .needs(pragma_config_resource_id(), ExecMode::Verify)
            .with_fix_variant(vec![]),
        // fmt - format code (no resources needed)
        // fmt has check variant (fmt-check) but not fix variant (fmt IS the fix)
        MetaTarget::new("fmt", "Format all code", ConfigField::Fmt).with_check_variant(),
        // ci-yaml - generate CI workflow files (no resources needed)
        MetaTarget::new(
            "ci-yaml",
            "Generate CI workflow YAML (GitHub Actions & GitLab CI)",
            ConfigField::CiYaml,
        ),
    ]
}

/// Get core workflow targets that are not tool entrypoints or meta targets.
///
/// These mirror the non-tool, non-meta targets currently rendered in
/// `makegen::render` (build orchestration, verification, and fix aliases).
pub fn default_core_workflows() -> Vec<WorkflowSpec> {
    vec![
        WorkflowSpec::core(
            "preflight-fix",
            "Preflight: auto-fix rustc warnings before running generators",
            &[],
        ),
        WorkflowSpec::core(
            "ensure-codegen",
            "Ensure CLI entrypoints exist (bootstrap-safe)",
            &[],
        )
        .with_resource(generated_cli_resource_id(), ExecMode::Ensure),
        WorkflowSpec::core(
            "build-release-bins",
            "Build workspace binaries once for direct tool execution",
            &["ensure-codegen"],
        )
        .with_resource(compiled_code_resource_id(), ExecMode::Ensure),
        WorkflowSpec::core(
            "lint-upsert",
            "Lint upsert: fix if needed, then verify",
            &["ensure-codegen", "preflight-fix"],
        ),
        WorkflowSpec::core(
            "codegen",
            "Generate CLI entrypoints (DAG upsert)",
            &["lint-upsert"],
        )
        .with_resource(generated_cli_resource_id(), ExecMode::Ensure),
        WorkflowSpec::core("build", "Full build transaction", &["codegen", "testgen"])
            .with_resource(compiled_code_resource_id(), ExecMode::Ensure),
        WorkflowSpec::core("clean", "Clean build artifacts", &[]),
        WorkflowSpec::core(
            "testgen",
            "Regenerate tests from DAG structures and MockSpecs",
            &["lint-upsert"],
        )
        .with_resource(generated_tests_resource_id(), ExecMode::Ensure),
        WorkflowSpec::core(
            "testgen-check",
            "Check if generated tests are stale",
            &["lint-upsert"],
        )
        .with_resource(generated_tests_resource_id(), ExecMode::Verify),
        WorkflowSpec::core(
            "deps-config",
            "Ensure deps.toml matches canonical generated configuration",
            &["build-release-bins"],
        )
        .with_resource(deps_config_resource_id(), ExecMode::Ensure),
        WorkflowSpec::core(
            "deps-config-check",
            "Check if deps.toml is stale",
            &["build-release-bins"],
        )
        .with_resource(deps_config_resource_id(), ExecMode::Verify),
        WorkflowSpec::core(
            "makegen-check",
            "Check if generated Makefile is stale",
            &["lint-upsert"],
        )
        .with_resource(makefile_resource_id(), ExecMode::Verify),
        WorkflowSpec::core(
            "bootstrap-check",
            "Check if generated bootstrap artifacts are stale",
            &["lint-upsert"],
        )
        .with_resource(gitignore_resource_id(), ExecMode::Verify),
        WorkflowSpec::core(
            "pragma-check",
            "Check if pragma artifacts are stale",
            &["lint-upsert"],
        )
        .with_resource(pragma_config_resource_id(), ExecMode::Verify),
        WorkflowSpec::core(
            "verify",
            "Verify generated artifacts match their generators",
            &["lint-upsert"],
        )
        .with_resource(verified_artifacts_resource_id(), ExecMode::Verify),
        WorkflowSpec::core(
            "verify-fix",
            "Ensure generated artifacts are up to date",
            &["lint-upsert"],
        )
        .with_resource(verified_artifacts_resource_id(), ExecMode::Ensure),
        WorkflowSpec::core("fmt-fix", "fmt-fix: apply formatting (alias for fmt)", &[]),
        WorkflowSpec::core(
            "lint-fix",
            "lint-fix: auto-fix lint issues where possible",
            &["pragma"],
        ),
        // WF8: CI and test-all are thin wrappers over gunbc-workflow planner.
        WorkflowSpec::core(
            "ci",
            "Run CI via workflow planner (typed units, deterministic keying)",
            &[],
        ),
        WorkflowSpec::core(
            "test-all",
            "Run all tests via workflow planner (warm-path optimized)",
            &[],
        ),
    ]
}

/// Registry of all gunbc tools and meta targets.
#[derive(Debug)]
pub struct ToolRegistry {
    /// Core orchestration targets (non-tool, non-meta).
    pub core_workflows: Vec<WorkflowSpec>,
    /// Individual tool targets (gist, deps, etc.)
    pub tools: Vec<ToolInfo>,
    /// Meta targets (test, check, fmt, clippy)
    pub meta_targets: Vec<MetaTarget>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            core_workflows: default_core_workflows(),
            tools: Vec::new(),
            meta_targets: default_meta_targets(),
        }
    }
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            core_workflows: Vec::new(),
            tools: Vec::new(),
            meta_targets: Vec::new(),
        }
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

    /// Add a meta target to the registry.
    pub fn register_meta(&mut self, target: MetaTarget) {
        self.meta_targets.push(target);
    }

    /// Add a core workflow to the registry.
    pub fn register_core_workflow(&mut self, workflow: WorkflowSpec) {
        self.core_workflows.push(workflow);
    }

    /// Build normalized workflow specifications for all registered targets.
    ///
    /// Output order is stable: meta targets first (dev workflow surface), then
    /// concrete tool targets.
    pub fn workflow_specs(&self, config: &BuildConfig) -> Vec<WorkflowSpec> {
        let res_map = ResourceTargetMap::default_map(config);
        let mut specs = Vec::with_capacity(
            self.core_workflows.len() + self.meta_targets.len() + self.tools.len(),
        );
        specs.extend(self.core_workflows.iter().cloned());
        specs.extend(
            self.meta_targets
                .iter()
                .map(|meta| meta.workflow_spec(&res_map)),
        );
        specs.extend(self.tools.iter().map(|tool| tool.workflow_spec(config)));
        propagate_workflow_live_secrets(&mut specs);
        specs
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

    /// Check if any codegen is needed.
    ///
    /// Returns true if the registry contains tools that require code generation.
    /// This is a conservative check — it does not verify whether generated files
    /// are stale, only whether the registry has anything to generate.
    ///
    /// For staleness checking at runtime, the CI graph uses transport-based file
    /// existence checks (see `CIOp::PrepareCodegenExistsCheck`).
    pub fn needs_codegen(&self) -> bool {
        !self.tools.is_empty()
    }

    /// Check if any daggen is needed.
    ///
    /// DEFERRED: Daggen (generating lowered DAG workflow definitions from DSL into
    /// compiled Rust) is explicitly out of scope for the current planner phase.
    /// Workflow DAGs remain hand-authored in Rust via `build_*_graph()` functions.
    /// See `docs/design/workflow-minimal-execution-model.md` Section 17.5 and the
    /// "Daggen status: Deferred" design decision in `TODO/tasks.md`.
    pub fn needs_daggen(&self) -> bool {
        false
    }

    /// Build the default registry with all known gunbc tools.
    ///
    /// Tool targets are derived from the codegen registry's `derive_tool_defs()`.
    /// Tools with a `CargoInvocation` set automatically get Makefile targets.
    /// Entrypoints with `make_var` set become Make variables.
    ///
    /// This eliminates manual dual-registration: adding a tool to the codegen
    /// registry with `.invocation()` is sufficient for it to appear in the
    /// Makefile. Only tools with handwritten binaries that are intentionally
    /// outside codegen discovery are added manually here.
    pub fn default_registry() -> Self {
        let mut registry = Self {
            core_workflows: default_core_workflows(),
            tools: Vec::new(),
            meta_targets: default_meta_targets(),
        };

        // Derive tool targets from the codegen registry (single source of truth).
        for tool_def in gunbc_codegen::registry::derive_tool_defs() {
            if let Some(tool_info) = ToolInfo::from_tool_def(&tool_def) {
                registry.register(tool_info);
            }
        }

        let tool_modules = discover_dsl_tool_modules();
        let pipeline_modules = discover_dsl_pipeline_modules();
        validate_required_manual_tool_modules(&tool_modules, &pipeline_modules);
        for tool in manual_workspace_tools_from_dsl_modules(&tool_modules, &pipeline_modules) {
            registry.register_if_missing(tool);
        }

        // Enrich tools with live-secret requirements from DagSpec registrations.
        enrich_live_secrets(&mut registry.tools);

        registry
    }
}

fn propagate_workflow_live_secrets(specs: &mut [WorkflowSpec]) {
    let index_by_name: BTreeMap<String, usize> = specs
        .iter()
        .enumerate()
        .map(|(index, workflow)| (workflow.name.clone(), index))
        .collect();
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for index in 0..specs.len() {
        let mut stack = BTreeSet::new();
        let secrets =
            resolve_workflow_live_secrets(index, specs, &index_by_name, &mut cache, &mut stack);
        specs[index].live_secrets = secrets;
    }
}

fn resolve_workflow_live_secrets(
    index: usize,
    specs: &[WorkflowSpec],
    index_by_name: &BTreeMap<String, usize>,
    cache: &mut BTreeMap<String, Vec<String>>,
    stack: &mut BTreeSet<String>,
) -> Vec<String> {
    let workflow_name = specs[index].name.clone();
    if let Some(cached) = cache.get(&workflow_name) {
        return cached.clone();
    }

    if !stack.insert(workflow_name.clone()) {
        return specs[index].live_secrets.clone();
    }

    let mut secrets = dedupe_live_secrets(specs[index].live_secrets.iter().cloned());
    for dep_name in &specs[index].deps {
        let Some(dep_index) = index_by_name.get(dep_name) else {
            continue;
        };
        let dep_secrets =
            resolve_workflow_live_secrets(*dep_index, specs, index_by_name, cache, stack);
        extend_live_secrets_unique(&mut secrets, dep_secrets);
    }

    stack.remove(&workflow_name);
    cache.insert(workflow_name, secrets.clone());
    secrets
}

fn dedupe_live_secrets(secrets: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut deduped = Vec::new();
    extend_live_secrets_unique(&mut deduped, secrets);
    deduped
}

fn extend_live_secrets_unique(target: &mut Vec<String>, secrets: impl IntoIterator<Item = String>) {
    for secret in secrets {
        if !target.contains(&secret) {
            target.push(secret);
        }
    }
}

/// Enrich tool entries with live-secret requirements from `DagSpecDef` registrations.
///
/// Looks up each tool by name in the testgen registry. If a matching `DagSpecDef`
/// has `live_required` secrets, those are attached to the tool's `live_secrets` field.
fn enrich_live_secrets(tools: &mut [ToolInfo]) {
    use std::collections::BTreeMap;

    // Build tool_name → live_secrets lookup from DagSpec registrations.
    let mut secrets_by_tool: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for spec in gunbc_testgen_registry::iter_dag_specs() {
        if let Some(tool_name) = spec.meta.tool_name {
            let entry = secrets_by_tool.entry(tool_name).or_default();
            if let Some(required) = spec.testgen.live_required {
                for secret in required {
                    let s = secret.to_string();
                    if !entry.contains(&s) {
                        entry.push(s);
                    }
                }
            }
            // Also include any-of groups (flattened for display purposes).
            if let Some(groups) = spec.testgen.live_required_any_of {
                for group in groups {
                    for secret in *group {
                        let s = secret.to_string();
                        if !entry.contains(&s) {
                            entry.push(s);
                        }
                    }
                }
            }
        }
    }

    // Apply to tools.
    for tool in tools.iter_mut() {
        if let Some(secrets) = secrets_by_tool.remove(tool.short_name.as_str()) {
            tool.live_secrets = secrets;
        }
    }
}

fn dsl_tools_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl/tools")
}

fn dsl_pipelines_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl/pipelines")
}

#[allow(clippy::disallowed_methods)] // Build-time DSL module discovery (not runtime I/O)
fn discover_dsl_modules(root: &Path, module_kind: &str) -> BTreeSet<String> {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        panic!(
            "failed to read DSL {module_kind} discovery root for makegen registry ({}): {error}",
            root.display()
        )
    });
    let mut modules = BTreeSet::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!(
                "failed to read DSL {module_kind} discovery entry for makegen registry ({}): {error}",
                root.display()
            )
        });
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("dag") {
            continue;
        }
        let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_else(|| {
            panic!(
                "failed to parse UTF-8 {module_kind} module stem for makegen registry discovery: {}",
                path.display()
            )
        });
        modules.insert(stem.to_string());
    }
    modules
}

fn discover_dsl_tool_modules() -> BTreeSet<String> {
    discover_dsl_modules(&dsl_tools_root(), "tool")
}

fn discover_dsl_pipeline_modules() -> BTreeSet<String> {
    discover_dsl_modules(&dsl_pipelines_root(), "pipeline")
}

/// Manual tool definitions: tools that need Makefile targets but aren't in the
/// tool registry (no `#[tool_target]` registration). Each entry declares its
/// required DSL module — validation and registration are co-located.
struct ManualToolDef {
    /// DSL module name (file stem in `dsl/tools/` or `dsl/pipelines/`).
    module: &'static str,
    /// Whether this is a pipeline module (dsl/pipelines/) vs tool module (dsl/tools/).
    is_pipeline: bool,
}

impl ManualToolDef {
    const fn tool(module: &'static str) -> Self {
        Self {
            module,
            is_pipeline: false,
        }
    }
}

/// All manual tool definitions. Adding a new manual tool here automatically
/// validates its DSL module exists and registers its Makefile target.
// WF8: ci is now a core workflow (thin wrapper over gunbc-workflow), not a manual tool.
const MANUAL_TOOL_DEFS: &[ManualToolDef] =
    &[ManualToolDef::tool("pragma"), ManualToolDef::tool("build")];

fn validate_required_manual_tool_modules(
    tool_modules: &BTreeSet<String>,
    pipeline_modules: &BTreeSet<String>,
) {
    let missing: Vec<&str> = MANUAL_TOOL_DEFS
        .iter()
        .filter(|def| {
            let modules = if def.is_pipeline {
                pipeline_modules
            } else {
                tool_modules
            };
            !modules.contains(def.module)
        })
        .map(|def| def.module)
        .collect();

    if missing.is_empty() {
        return;
    }
    panic!(
        "missing required DSL modules for makegen manual targets: {}",
        missing.join(", ")
    );
}

fn manual_workspace_tools_from_dsl_modules(
    tool_modules: &BTreeSet<String>,
    pipeline_modules: &BTreeSet<String>,
) -> Vec<ToolInfo> {
    let _ = pipeline_modules;
    let mut tools = Vec::new();
    if tool_modules.contains("pragma") {
        tools.push(
            ToolInfo::workspace(
                WorkspaceBinary::Pragma,
                "Generate clippy.toml and pragma allowlists",
            )
            .manual(),
        );
    }
    if tool_modules.contains("build") {
        // Keep the historical Makefile target name (`build-all`) to avoid
        // churn for existing local workflows while sourcing availability
        // from DSL module discovery.
        tools.push(ToolInfo {
            invocation: WorkspaceBinary::Build.invocation(),
            short_name: "build-all".to_string(),
            description: "Build, test, and lint with progress display".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: false,
            live_secrets: Vec::new(),
        });
    }
    tools
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
        assert_eq!(config.warnings, Warnings::Deny);
        assert!(config.build.to_shell().contains("cargo build"));
        assert!(config.test.to_shell().contains("cargo test"));
    }

    #[test]
    fn test_build_config_buck2() {
        let config = BuildConfig::buck2();
        assert_eq!(config.build_system, BuildSystem::Buck2);
        assert!(config.build.to_shell().contains("buck2 build"));
    }

    #[test]
    fn test_build_config_shell_methods() {
        let config = BuildConfig::cargo();
        assert!(config.build_shell().starts_with("@"));
        assert!(config.test_shell().contains("cargo test"));
        assert!(config.lint_shell().contains("clippy"));
    }

    #[test]
    fn test_build_config_cargo_entrypoints() {
        let config = BuildConfig::cargo_entrypoints();
        assert!(config.use_dag_entrypoints);
        assert!(config.build_shell().contains("gunbc-build"));
        assert!(config.test_shell().contains("gunbc-build"));
        assert!(config.lint_shell().contains("gunbc-build"));
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

        let deps = registry.tools.iter().find(|t| t.short_name == "deps");
        assert!(deps.is_some());
    }

    #[test]
    fn test_default_registry_manual_tools_follow_dsl_discovery() {
        let registry = ToolRegistry::default_registry();
        assert!(!registry.tools.iter().any(|t| t.short_name == "ci"));
        assert!(registry.tools.iter().any(|t| t.short_name == "pragma"));
        assert!(registry.tools.iter().any(|t| t.short_name == "build-all"));
    }

    #[test]
    fn test_default_registry_has_unique_short_names() {
        let registry = ToolRegistry::default_registry();
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
    fn test_tool_has_entrypoints() {
        let registry = ToolRegistry::default_registry();
        let gist = registry
            .tools
            .iter()
            .find(|t| t.short_name == "gist")
            .unwrap();

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

    // ========================================================================
    // MetaTarget Tests
    // ========================================================================

    #[test]
    fn test_default_meta_targets() {
        let targets = default_meta_targets();

        // Should have test, integration/external slices, check, clippy, fmt.
        assert!(targets.iter().any(|t| t.name == "test"));
        assert!(targets.iter().any(|t| t.name == "test-integration"));
        assert!(targets.iter().any(|t| t.name == "test-external"));
        assert!(targets.iter().any(|t| t.name == "check"));
        assert!(targets.iter().any(|t| t.name == "clippy"));
        assert!(targets.iter().any(|t| t.name == "fmt"));
    }

    #[test]
    fn test_default_core_workflows_contains_key_targets() {
        let workflows = default_core_workflows();
        assert!(workflows.iter().any(|w| w.name == "build"));
        assert!(workflows.iter().any(|w| w.name == "codegen"));
        assert!(workflows.iter().any(|w| w.name == "testgen"));
        assert!(workflows.iter().any(|w| w.name == "verify"));
        assert!(workflows.iter().any(|w| w.name == "pragma-check"));
        assert!(workflows.iter().all(|w| w.kind == WorkflowKind::Core));
    }

    #[test]
    fn test_meta_target_resources() {
        let targets = default_meta_targets();

        let test = targets.iter().find(|t| t.name == "test").unwrap();
        assert_eq!(test.resources.len(), 2);
        assert_eq!(test.resources[0].id, compiled_code_resource_id());
        assert_eq!(test.resources[0].base_mode, ExecMode::Ensure);

        let fmt = targets.iter().find(|t| t.name == "fmt").unwrap();
        assert!(fmt.resources.is_empty());

        let integration = targets
            .iter()
            .find(|t| t.name == "test-integration")
            .unwrap();
        assert_eq!(integration.resources.len(), 2);
        assert_eq!(integration.resources[0].id, compiled_code_resource_id());
        assert_eq!(
            integration.resources[1].id,
            verified_artifacts_resource_id()
        );

        let external = targets.iter().find(|t| t.name == "test-external").unwrap();
        assert_eq!(external.resources.len(), 2);
        assert_eq!(external.resources[0].id, compiled_code_resource_id());
        assert_eq!(external.resources[1].id, verified_artifacts_resource_id());

        let clippy = targets.iter().find(|t| t.name == "clippy").unwrap();
        assert_eq!(clippy.resources.len(), 2);
        assert_eq!(clippy.resources[0].id, generated_cli_resource_id());
        assert_eq!(clippy.resources[1].base_mode, ExecMode::Verify);
    }

    #[test]
    fn test_filtered_test_targets_use_expected_filters() {
        let targets = default_meta_targets();
        let config = BuildConfig::cargo();

        let integration = targets
            .iter()
            .find(|t| t.name == "test-integration")
            .unwrap();
        let integration_cmd = integration.get_command(&config);
        assert!(integration_cmd.contains("GUNBC_TEST_MAX_COST=XL"));
        assert!(integration_cmd.contains("cargo test integration"));

        let external = targets.iter().find(|t| t.name == "test-external").unwrap();
        let external_cmd = external.get_command(&config);
        assert!(external_cmd.contains("GUNBC_TEST_MAX_COST=XL"));
        assert!(external_cmd.contains("cargo test live_flow"));
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

    #[test]
    fn test_tool_workflow_spec_contains_entrypoints_and_deps() {
        let config = BuildConfig::cargo();
        let mut tool = ToolInfo::workspace(WorkspaceBinary::Ci, "Run CI pipeline")
            .manual()
            .with_param(EntrypointParam::new("mode", "MODE", "--mode", "String"));
        tool.live_secrets = vec!["CI_TOKEN".to_string()];

        let spec = tool.workflow_spec(&config);
        assert_eq!(spec.name, "ci");
        assert_eq!(spec.kind, WorkflowKind::Tool);
        assert_eq!(spec.entrypoints.len(), 1);
        assert_eq!(spec.resources.len(), 0);
        assert!(spec.deps.is_empty());
        assert_eq!(spec.live_secrets, vec!["CI_TOKEN"]);
    }

    #[test]
    fn test_meta_workflow_spec_contains_resources_and_deps() {
        let config = BuildConfig::cargo();
        let res_map = ResourceTargetMap::default_map(&config);
        let meta = MetaTarget::new("clippy", "Run clippy", ConfigField::Lint)
            .needs(generated_cli_resource_id(), ExecMode::Ensure)
            .needs(pragma_config_resource_id(), ExecMode::Verify);

        let spec = meta.workflow_spec(&res_map);
        assert_eq!(spec.name, "clippy");
        assert_eq!(spec.kind, WorkflowKind::Meta);
        assert!(spec.entrypoints.is_empty());
        assert_eq!(spec.resources.len(), 2);
        assert_eq!(spec.deps, vec!["ensure-codegen", "pragma-check"]);
    }

    #[test]
    fn test_registry_workflow_specs_covers_core_tools_and_meta_targets() {
        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::cargo();
        let specs = registry.workflow_specs(&config);
        assert_eq!(
            specs.len(),
            registry.core_workflows.len() + registry.tools.len() + registry.meta_targets.len()
        );
        assert!(specs.iter().any(|spec| spec.kind == WorkflowKind::Core));
        assert!(specs.iter().any(|spec| spec.kind == WorkflowKind::Tool));
        assert!(specs.iter().any(|spec| spec.kind == WorkflowKind::Meta));
    }

    #[test]
    fn test_registry_workflow_specs_propagates_live_secrets_through_dependencies() {
        let config = BuildConfig::cargo();
        let mut registry = ToolRegistry::new();
        registry.register_core_workflow(WorkflowSpec::core("root", "Root workflow", &["middle"]));
        registry.register_core_workflow(WorkflowSpec::core(
            "middle",
            "Middle workflow",
            &["alpha", "beta"],
        ));

        let mut alpha = ToolInfo::new("gunbc-alpha", "alpha", "Alpha tool");
        alpha.live_secrets = vec!["ALPHA_TOKEN".to_string(), "SHARED_TOKEN".to_string()];
        registry.register(alpha);

        let mut beta = ToolInfo::new("gunbc-beta", "beta", "Beta tool");
        beta.live_secrets = vec!["BETA_TOKEN".to_string(), "SHARED_TOKEN".to_string()];
        registry.register(beta);

        let specs = registry.workflow_specs(&config);
        let middle = specs
            .iter()
            .find(|spec| spec.name == "middle")
            .expect("middle workflow should exist");
        assert_eq!(
            middle.live_secrets,
            vec!["ALPHA_TOKEN", "SHARED_TOKEN", "BETA_TOKEN"]
        );

        let root = specs
            .iter()
            .find(|spec| spec.name == "root")
            .expect("root workflow should exist");
        assert_eq!(
            root.live_secrets,
            vec!["ALPHA_TOKEN", "SHARED_TOKEN", "BETA_TOKEN"]
        );
    }

    // ========================================================================
    // Fix Variant Tests (the-gunbai dev UX convention)
    // ========================================================================

    #[test]
    fn test_test_has_fix_variant() {
        let targets = default_meta_targets();
        let test = targets.iter().find(|t| t.name == "test").unwrap();

        assert!(test.has_fix_variant);
        assert_eq!(
            test.fix_prerequisites,
            vec![FixAlias::FmtFix, FixAlias::LintFix]
        );
    }

    #[test]
    fn test_check_has_fix_variant() {
        let targets = default_meta_targets();
        let check = targets.iter().find(|t| t.name == "check").unwrap();

        assert!(check.has_fix_variant);
        assert_eq!(check.fix_prerequisites, vec![FixAlias::FmtFix]);
    }

    #[test]
    fn test_clippy_has_fix_variant() {
        let targets = default_meta_targets();
        let clippy = targets.iter().find(|t| t.name == "clippy").unwrap();
        let config = BuildConfig::cargo();

        assert!(clippy.has_fix_variant);
        // clippy-fix uses dedicated fix command
        let fix_cmd = clippy.get_fix_command(&config).unwrap();
        assert!(fix_cmd.contains("--fix"));
    }

    #[test]
    fn test_fmt_has_no_fix_variant() {
        // fmt has check variant, but not fix variant (fmt IS the fix)
        let targets = default_meta_targets();
        let fmt = targets.iter().find(|t| t.name == "fmt").unwrap();

        assert!(fmt.has_check_variant);
        assert!(!fmt.has_fix_variant);
    }

    #[test]
    fn test_lint_fix_command() {
        let config = BuildConfig::cargo();
        assert!(config.lint_fix_shell().contains("--fix"));
        assert!(config.lint_fix_shell().contains("--allow-dirty"));
    }

    // ========================================================================
    // ResourceTargetMap Tests
    // ========================================================================

    #[test]
    fn test_resource_target_map_resolve_ensure() {
        let config = BuildConfig::cargo();
        let map = ResourceTargetMap::default_map(&config);

        assert_eq!(
            map.resolve(&generated_tests_resource_id(), ExecMode::Ensure),
            Some("testgen")
        );
        assert_eq!(
            map.resolve(&verified_artifacts_resource_id(), ExecMode::Ensure),
            Some("verify-fix")
        );
        assert_eq!(
            map.resolve(&compiled_code_resource_id(), ExecMode::Ensure),
            Some("build")
        );
        assert_eq!(
            map.resolve(&generated_cli_resource_id(), ExecMode::Ensure),
            Some("ensure-codegen")
        );
        assert_eq!(
            map.resolve(&deps_config_resource_id(), ExecMode::Ensure),
            Some("deps-config")
        );
        assert_eq!(
            map.resolve(&makefile_resource_id(), ExecMode::Ensure),
            Some("makegen")
        );
        assert_eq!(
            map.resolve(&gitignore_resource_id(), ExecMode::Ensure),
            Some("bootstrap")
        );
    }

    #[test]
    fn test_resource_target_map_resolve_verify() {
        let config = BuildConfig::cargo();
        let map = ResourceTargetMap::default_map(&config);

        assert_eq!(
            map.resolve(&generated_tests_resource_id(), ExecMode::Verify),
            Some("testgen-check")
        );
        assert_eq!(
            map.resolve(&pragma_config_resource_id(), ExecMode::Verify),
            Some("pragma-check")
        );
        assert_eq!(
            map.resolve(&generated_cli_resource_id(), ExecMode::Verify),
            Some("ensure-codegen")
        );
        assert_eq!(
            map.resolve(&verified_artifacts_resource_id(), ExecMode::Verify),
            Some("verify")
        );
        assert_eq!(
            map.resolve(&deps_config_resource_id(), ExecMode::Verify),
            Some("deps-config-check")
        );
        assert_eq!(
            map.resolve(&makefile_resource_id(), ExecMode::Verify),
            Some("makegen-check")
        );
        assert_eq!(
            map.resolve(&gitignore_resource_id(), ExecMode::Verify),
            Some("bootstrap-check")
        );
    }

    #[test]
    fn test_resource_target_map_dag_entrypoints() {
        let config = BuildConfig::cargo_entrypoints();
        let map = ResourceTargetMap::default_map(&config);

        // compiled_code maps to "codegen" when DAG entrypoints are used
        assert_eq!(
            map.resolve(&compiled_code_resource_id(), ExecMode::Ensure),
            Some("codegen")
        );
    }

    #[test]
    fn test_resource_target_map_unknown_resource() {
        let config = BuildConfig::cargo();
        let map = ResourceTargetMap::default_map(&config);

        assert_eq!(
            map.resolve(&ResourceId::build("nonexistent"), ExecMode::Ensure),
            None
        );
    }

    // ========================================================================
    // Registry Derivation Tests (single source of truth)
    // ========================================================================

    #[test]
    fn test_registry_derived_from_codegen() {
        let registry = ToolRegistry::default_registry();
        let codegen_tools = gunbc_codegen::registry::derive_tool_defs();

        // Every codegen tool with an invocation must appear in the makegen registry
        for tool_def in &codegen_tools {
            if tool_def.invocation.is_some() {
                let found = registry
                    .tools
                    .iter()
                    .any(|t| t.short_name == tool_def.meta.tool_name);
                assert!(
                    found,
                    "Tool '{}' has invocation in codegen but missing from makegen registry",
                    tool_def.meta.tool_name
                );
            }
        }
    }

    #[test]
    fn test_registry_entrypoints_match_codegen() {
        let registry = ToolRegistry::default_registry();
        let codegen_tools = gunbc_codegen::registry::derive_tool_defs();

        for tool_def in &codegen_tools {
            if tool_def.invocation.is_none() {
                continue;
            }
            let tool_info = registry
                .tools
                .iter()
                .find(|t| t.short_name == tool_def.meta.tool_name)
                .unwrap();

            // Count entrypoints with make_var in codegen
            let codegen_make_params: Vec<_> = tool_def
                .entrypoints
                .iter()
                .filter(|ep| ep.make_var.is_some())
                .collect();

            assert_eq!(
                tool_info.entrypoints.len(),
                codegen_make_params.len(),
                "Tool '{}': makegen has {} params, codegen has {} with make_var",
                tool_def.meta.tool_name,
                tool_info.entrypoints.len(),
                codegen_make_params.len()
            );

            // Verify CLI flags match generated CLI flag names
            for (info_param, codegen_ep) in
                tool_info.entrypoints.iter().zip(codegen_make_params.iter())
            {
                assert_eq!(
                    info_param.cli_flag,
                    format!("--{}", codegen_ep.flag_name()),
                    "Tool '{}' param '{}': Makefile flag doesn't match generated CLI flag",
                    tool_def.meta.tool_name,
                    info_param.port_name
                );
            }
        }
    }

    #[test]
    fn test_tools_without_invocation_excluded() {
        let registry = ToolRegistry::default_registry();
        let codegen_tools = gunbc_codegen::registry::derive_tool_defs();

        // Tools without invocation should NOT appear (unless manually added)
        for tool_def in &codegen_tools {
            if tool_def.invocation.is_none() {
                let found = registry
                    .tools
                    .iter()
                    .any(|t| t.short_name == tool_def.meta.tool_name);
                assert!(
                    !found,
                    "Tool '{}' has no invocation but appeared in makegen registry",
                    tool_def.meta.tool_name
                );
            }
        }
    }

    #[test]
    fn test_all_tools_have_no_make_deps() {
        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::cargo();
        for tool in &registry.tools {
            let deps = tool_dependency_targets(tool, &config);
            assert!(
                deps.is_empty(),
                "tool '{}' should have no Make prerequisites, found: {:?}",
                tool.short_name,
                deps
            );
        }
    }

    #[test]
    fn test_gist_has_no_make_prerequisites() {
        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::cargo();
        let gist = registry
            .tools
            .iter()
            .find(|t| t.short_name == "gist")
            .expect("gist should be in registry");
        let deps = tool_dependency_targets(gist, &config);
        assert!(
            deps.is_empty(),
            "gist should be dispatched by workflow without make prerequisites"
        );
    }
}
