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

use crate::WorkspaceBinary;
use gunbc_infra::ResourceId;
use gunbc_ir::cargo::{CargoCommand, Subcommand, Warnings};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::ShellRequest;
use gunbc_ir::CargoInvocation;

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
    /// Command to generate bootstrap artifacts (Makefile + .gitignore)
    pub bootstrap: BuildCommand,
    /// Command to generate pragma artifacts (clippy.toml + allowlists)
    pub pragma: BuildCommand,
    /// Command to check if generated tests are stale
    pub testgen_check: BuildCommand,
    /// Command to check if generated Makefile is stale
    pub makegen_check: BuildCommand,
    /// Command to check if generated bootstrap files are stale
    pub bootstrap_check: BuildCommand,
    /// Command to check if generated pragma/clippy config is stale
    pub pragma_check: BuildCommand,
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
                .release()
                .trailing_arg("codegen")
                .warnings(w)),
            codegen: c(CargoCommand::new(Subcommand::Run(codegen_dag_inv.clone()))
                .release()
                .warnings(w)),
            daggen: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .release()
                .trailing_arg("daggen")
                .warnings(w)),
            build: c(CargoCommand::new(Subcommand::Build)
                .all_targets()
                .warnings(w)),
            test: c(CargoCommand::new(Subcommand::Test).warnings(w)),
            lint: c(CargoCommand::new(Subcommand::Clippy)
                .all_targets()
                .warnings(w)),
            lint_fix: c(CargoCommand::new(Subcommand::Clippy)
                .flag("--fix")
                .flag("--workspace")
                .flag("--allow-dirty")
                .flag("--allow-staged")
                .warnings(w)),
            fmt: c(CargoCommand::new(Subcommand::Fmt)),
            fmt_check: c(CargoCommand::new(Subcommand::Fmt).trailing_arg("--check")),
            check: c(CargoCommand::new(Subcommand::Check)
                .all_targets()
                .warnings(w)),
            ci_yaml: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .release()
                .trailing_arg("cigen")
                .warnings(w)),
            testgen: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Testgen.invocation(),
                ))
                .release()
                .warnings(w),
            ),
            bootstrap: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Bootstrap.invocation(),
                ))
                .release()
                .warnings(w),
            ),
            pragma: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Pragma.invocation(),
                ))
                .release()
                .warnings(w),
            ),
            testgen_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Testgen.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
            makegen_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Makegen.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
            bootstrap_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Bootstrap.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
            pragma_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Pragma.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
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
            CargoCommand::new(Subcommand::Run(build_inv))
                .release()
                .warnings(config.warnings),
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
                .release()
                .trailing_arg("codegen")
                .warnings(w)),
            codegen: c(CargoCommand::new(Subcommand::Run(codegen_dag_inv.clone()))
                .release()
                .warnings(w)),
            daggen: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .release()
                .trailing_arg("daggen")
                .warnings(w)),
            // Buck2-native commands
            build: sh(&["buck2", "build", "//..."]),
            test: sh(&["buck2", "test", "//..."]),
            lint: sh(&["buck2", "run", "//tools:clippy"]),
            // lint-fix still uses cargo (buck2 doesn't have an equivalent)
            lint_fix: c(CargoCommand::new(Subcommand::Clippy)
                .flag("--fix")
                .flag("--workspace")
                .flag("--allow-dirty")
                .flag("--allow-staged")
                .warnings(w)),
            // fmt stays cargo (buck2 delegates to cargo fmt)
            fmt: c(CargoCommand::new(Subcommand::Fmt)),
            fmt_check: c(CargoCommand::new(Subcommand::Fmt).trailing_arg("--check")),
            check: sh(&["buck2", "build", "//..."]),
            ci_yaml: c(CargoCommand::new(Subcommand::Run(codegen_inv.clone()))
                .release()
                .trailing_arg("cigen")
                .warnings(w)),
            // testgen uses cargo (no buck2 equivalent yet)
            testgen: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Testgen.invocation(),
                ))
                .release()
                .warnings(w),
            ),
            bootstrap: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Bootstrap.invocation(),
                ))
                .release()
                .warnings(w),
            ),
            pragma: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Pragma.invocation(),
                ))
                .release()
                .warnings(w),
            ),
            testgen_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Testgen.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
            makegen_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Makegen.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
            bootstrap_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Bootstrap.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
            pragma_check: c(
                CargoCommand::new(Subcommand::Run(
                    WorkspaceBinary::Pragma.invocation(),
                ))
                .release()
                .trailing_arg("--mode=verify")
                .warnings(w),
            ),
        }
    }

    /// Get the codegen command as a shell string (for Makefile generation).
    pub fn codegen_shell(&self) -> String {
        format!("@{}", self.codegen.to_shell())
    }

    /// Get the ensure-codegen command as a shell string.
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

    /// Get the testgen-check command as a shell string.
    pub fn testgen_check_shell(&self) -> String {
        format!("@{}", self.testgen_check.to_shell())
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
            short_name: def.meta.tool_name.clone(),
            description: def.meta.description.clone(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: def.meta.tool_name == "makegen",
            needs_generated_cli: true,
        };

        // Convert entrypoints that have make_var set
        for ep in &def.entrypoints {
            if let Some(ref make_var) = ep.make_var {
                info.entrypoints.push(EntrypointParam {
                    port_name: ep.port_name.clone(),
                    make_var: make_var.clone(),
                    // Use the actual CLI flag name (matches generated CLI)
                    cli_flag: format!("--{}", ep.flag_name()),
                    type_hint: ep.type_id.clone(),
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
    /// Verify target. None means same as ensure (always ensure).
    verify_target: Option<String>,
}

impl ResourceTargetMap {
    /// Resolve a ResourceId + mode to a Make target name.
    ///
    /// - `Ensure` → ensure_target
    /// - `Verify` → verify_target (fallback to ensure_target if None)
    pub fn resolve(&self, id: &ResourceId, mode: ExecMode) -> Option<&str> {
        self.entries.iter().find(|e| e.id == *id).map(|e| match mode {
            ExecMode::Ensure => e.ensure_target.as_str(),
            ExecMode::Verify => e
                .verify_target
                .as_deref()
                .unwrap_or(e.ensure_target.as_str()),
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
                    id: ResourceId::build("generated_cli"),
                    ensure_target: "ensure-codegen".to_string(),
                    verify_target: None, // always ensure
                },
                ResourceTargetEntry {
                    id: ResourceId::build("generated_tests"),
                    ensure_target: "testgen".to_string(),
                    verify_target: Some("testgen-check".to_string()),
                },
                ResourceTargetEntry {
                    id: ResourceId::build("pragma_config"),
                    ensure_target: "pragma".to_string(),
                    verify_target: Some("pragma-check".to_string()),
                },
                ResourceTargetEntry {
                    id: ResourceId::build("compiled_code"),
                    ensure_target: compiled_code_target.to_string(),
                    verify_target: None, // always ensure
                },
                ResourceTargetEntry {
                    id: ResourceId::build("verified_artifacts"),
                    ensure_target: "verify-fix".to_string(),
                    verify_target: Some("verify".to_string()),
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
    /// Get the command from BuildConfig for this field.
    pub fn get_command(&self, config: &BuildConfig) -> String {
        match self {
            ConfigField::Test => config.test_shell(),
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
            .needs(ResourceId::build("compiled_code"), ExecMode::Ensure)
            .needs(ResourceId::build("verified_artifacts"), ExecMode::Ensure)
            .with_fix_variant(vec![FixAlias::FmtFix, FixAlias::LintFix]),
        // test-all - run all tests regardless of cost (includes XL)
        MetaTarget::new("test-all", "Run all tests (<=XL)", ConfigField::Test)
            .with_command_prefix("GUNBC_TEST_MAX_COST=XL RUN_LIVE_INTEGRATION=1")
            .needs(ResourceId::build("compiled_code"), ExecMode::Ensure)
            .needs(ResourceId::build("verified_artifacts"), ExecMode::Ensure),
        // check - type check without building (requires codegen + pragma)
        // check-fix: fmt-fix first, then check
        MetaTarget::new("check", "Type check all targets", ConfigField::Check)
            .needs(ResourceId::build("generated_cli"), ExecMode::Ensure)
            .needs(ResourceId::build("pragma_config"), ExecMode::Verify)
            .with_fix_variant(vec![FixAlias::FmtFix]),
        // clippy - run linter (requires codegen + pragma)
        // clippy-fix: uses cargo clippy --fix (auto-fix where possible)
        MetaTarget::new("clippy", "Run clippy linter", ConfigField::Lint)
            .needs(ResourceId::build("generated_cli"), ExecMode::Ensure)
            .needs(ResourceId::build("pragma_config"), ExecMode::Verify)
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

/// Registry of all gunbc tools and meta targets.
#[derive(Debug)]
pub struct ToolRegistry {
    /// Individual tool targets (gist, deps, etc.)
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
    pub fn needs_daggen(&self) -> bool {
        // For now, daggen is optional - only needed if we want to regenerate
        // declarative DAG definitions into code
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
    /// Makefile. Only tools that can't be in the codegen registry (like `ci`,
    /// which has a handwritten main.rs) are added manually here.
    pub fn default_registry() -> Self {
        let mut registry = Self {
            tools: Vec::new(),
            meta_targets: default_meta_targets(),
        };

        // Derive tool targets from the codegen registry (single source of truth).
        for tool_def in gunbc_codegen::registry::derive_tool_defs() {
            if let Some(tool_info) = ToolInfo::from_tool_def(&tool_def) {
                registry.register(tool_info);
            }
        }

        // Manual additions: tools not in the codegen registry.
        // ci has a handwritten main.rs — it's the bootstrap tool that runs
        // codegen for other tools, so it can't depend on generated code.
        registry.register(ToolInfo::workspace(WorkspaceBinary::Ci, "Run CI pipeline").manual());
        registry.register(
            ToolInfo::workspace(
                WorkspaceBinary::Pragma,
                "Generate clippy.toml and pragma allowlists",
            )
            .manual(),
        );

        // build-all has a handwritten main.rs with DAG progress display.
        // This is the explicit pipeline entrypoint; core build/test/clippy
        // targets use BuildConfig commands (cargo by default).
        registry.register(ToolInfo {
            invocation: WorkspaceBinary::Build.invocation(),
            short_name: "build-all".to_string(),
            description: "Build, test, and lint with progress display".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: false,
        });

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

        // Should have test, test-all, check, clippy, fmt
        assert!(targets.iter().any(|t| t.name == "test"));
        assert!(targets.iter().any(|t| t.name == "test-all"));
        assert!(targets.iter().any(|t| t.name == "check"));
        assert!(targets.iter().any(|t| t.name == "clippy"));
        assert!(targets.iter().any(|t| t.name == "fmt"));
    }

    #[test]
    fn test_meta_target_resources() {
        let targets = default_meta_targets();

        let test = targets.iter().find(|t| t.name == "test").unwrap();
        assert_eq!(test.resources.len(), 2);
        assert_eq!(test.resources[0].id, ResourceId::build("compiled_code"));
        assert_eq!(test.resources[0].base_mode, ExecMode::Ensure);

        let fmt = targets.iter().find(|t| t.name == "fmt").unwrap();
        assert!(fmt.resources.is_empty());

        let clippy = targets.iter().find(|t| t.name == "clippy").unwrap();
        assert_eq!(clippy.resources.len(), 2);
        assert_eq!(clippy.resources[0].id, ResourceId::build("generated_cli"));
        assert_eq!(clippy.resources[1].base_mode, ExecMode::Verify);
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

    // ========================================================================
    // Fix Variant Tests (the-gunbai dev UX convention)
    // ========================================================================

    #[test]
    fn test_test_has_fix_variant() {
        let targets = default_meta_targets();
        let test = targets.iter().find(|t| t.name == "test").unwrap();

        assert!(test.has_fix_variant);
        assert_eq!(test.fix_prerequisites, vec![FixAlias::FmtFix, FixAlias::LintFix]);
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
            map.resolve(&ResourceId::build("generated_tests"), ExecMode::Ensure),
            Some("testgen")
        );
        assert_eq!(
            map.resolve(&ResourceId::build("verified_artifacts"), ExecMode::Ensure),
            Some("verify-fix")
        );
        assert_eq!(
            map.resolve(&ResourceId::build("compiled_code"), ExecMode::Ensure),
            Some("build")
        );
    }

    #[test]
    fn test_resource_target_map_resolve_verify() {
        let config = BuildConfig::cargo();
        let map = ResourceTargetMap::default_map(&config);

        // Has verify target → use it
        assert_eq!(
            map.resolve(&ResourceId::build("generated_tests"), ExecMode::Verify),
            Some("testgen-check")
        );
        assert_eq!(
            map.resolve(&ResourceId::build("pragma_config"), ExecMode::Verify),
            Some("pragma-check")
        );
        // No verify target → fallback to ensure
        assert_eq!(
            map.resolve(&ResourceId::build("generated_cli"), ExecMode::Verify),
            Some("ensure-codegen")
        );
        assert_eq!(
            map.resolve(&ResourceId::build("verified_artifacts"), ExecMode::Verify),
            Some("verify")
        );
    }

    #[test]
    fn test_resource_target_map_dag_entrypoints() {
        let config = BuildConfig::cargo_entrypoints();
        let map = ResourceTargetMap::default_map(&config);

        // compiled_code maps to "codegen" when DAG entrypoints are used
        assert_eq!(
            map.resolve(&ResourceId::build("compiled_code"), ExecMode::Ensure),
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
}
