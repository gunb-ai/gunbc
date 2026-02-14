//! Cargo workspace binary invocation.
//!
//! Models how to invoke a binary from a cargo workspace. Handles the
//! distinction between standalone packages and binaries that live inside
//! another package:
//!
//! - Standalone: `cargo run -p gunbc-gist`
//! - In-package: `cargo run -p gunbc-dag --bin gunbc-ci`
//!
//! This is the single source of truth for cargo invocation rendering,
//! used by both Makefile generation and CI YAML generation.
//!
//! # Name Composition
//!
//! All binary names follow the `{PREFIX}-{component}` pattern (e.g., `gunbc-ci`).
//! Use [`CargoInvocation::standalone`] and [`CargoInvocation::composed`] to
//! construct invocations from component names, avoiding hardcoded full names.

use crate::resource::ExecMode;

/// Workspace binary name prefix. All gunbc binaries follow `{PREFIX}-{component}`.
pub const PREFIX: &str = "gunbc";

/// Compose a full binary name from a component: `{PREFIX}-{component}`.
///
/// # Examples
///
/// ```
/// use gunbc_ir::cargo::name;
/// assert_eq!(name("ci"), "gunbc-ci");
/// assert_eq!(name("gist"), "gunbc-gist");
/// ```
pub fn name(component: &str) -> String {
    format!("{PREFIX}-{component}")
}

/// Describes how to invoke a cargo workspace binary.
///
/// This type is the canonical way to represent "run this binary from
/// the workspace" across all renderers (Makefile, GitHub Actions, GitLab CI).
///
/// Prefer [`CargoInvocation::standalone`] and [`CargoInvocation::composed`]
/// over [`CargoInvocation::new`] and [`CargoInvocation::in_package`] to
/// ensure names are composed from components rather than hardcoded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CargoInvocation {
    /// The binary name (e.g., "gunbc-ci").
    pub binary: String,
    /// The package name, if different from the binary (e.g., "gunbc-dag").
    /// Set when the binary is a `[[bin]]` entry in another crate's Cargo.toml.
    pub package: Option<String>,
}

impl CargoInvocation {
    /// Create an invocation for a binary in its own package.
    ///
    /// Produces: `cargo run -p <binary>`
    ///
    /// Prefer [`Self::standalone`] which composes the name from a component.
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            package: None,
        }
    }

    /// Create an invocation for a binary inside another package.
    ///
    /// Produces: `cargo run -p <package> --bin <binary>`
    ///
    /// Prefer [`Self::composed`] which composes both names from components.
    pub fn in_package(binary: impl Into<String>, package: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            package: Some(package.into()),
        }
    }

    /// Create an invocation for a standalone binary from its component name.
    ///
    /// The binary name is composed as `{PREFIX}-{component}`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gunbc_ir::CargoInvocation;
    /// let inv = CargoInvocation::standalone("gist");
    /// assert_eq!(inv.binary, "gunbc-gist");
    /// assert_eq!(inv.command(), "cargo run -p gunbc-gist");
    /// ```
    pub fn standalone(component: &str) -> Self {
        Self::new(name(component))
    }

    /// Create an invocation for a binary inside another package, both
    /// composed from component names.
    ///
    /// The binary is `{PREFIX}-{binary_component}`, the package is
    /// `{PREFIX}-{package_component}`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gunbc_ir::CargoInvocation;
    /// let inv = CargoInvocation::composed("ci", "dag");
    /// assert_eq!(inv.binary, "gunbc-ci");
    /// assert_eq!(inv.package, Some("gunbc-dag".to_string()));
    /// assert_eq!(inv.command(), "cargo run -p gunbc-dag --bin gunbc-ci");
    /// ```
    pub fn composed(binary_component: &str, package_component: &str) -> Self {
        Self::in_package(name(binary_component), name(package_component))
    }

    /// Get the cargo run arguments (without the `cargo run` prefix).
    ///
    /// Returns `-p <package> --bin <binary>` or `-p <binary>`.
    pub fn args(&self) -> String {
        match &self.package {
            Some(pkg) => format!("-p {} --bin {}", pkg, self.binary),
            None => format!("-p {}", self.binary),
        }
    }

    /// Get the full `cargo run` command string.
    ///
    /// Returns `cargo run -p <package> --bin <binary>` or `cargo run -p <binary>`.
    pub fn command(&self) -> String {
        format!("cargo run {}", self.args())
    }

    /// Get the cargo run command as individual string parts.
    ///
    /// Returns `["cargo", "run", "-p", package, "--bin", binary]` or
    /// `["cargo", "run", "-p", binary]`.
    pub fn command_parts(&self) -> Vec<String> {
        let mut parts = vec!["cargo".to_string(), "run".to_string()];
        match &self.package {
            Some(pkg) => {
                parts.push("-p".to_string());
                parts.push(pkg.clone());
                parts.push("--bin".to_string());
                parts.push(self.binary.clone());
            }
            None => {
                parts.push("-p".to_string());
                parts.push(self.binary.clone());
            }
        }
        parts
    }

    /// Get the cargo run command parts followed by additional arguments.
    ///
    /// Convenience for building command vectors like
    /// `["cargo", "run", "-p", "gunbc-codegen", "--release", "--", "codegen"]`.
    pub fn run_with_args(&self, extra: &[&str]) -> Vec<String> {
        let mut parts = self.command_parts();
        parts.extend(extra.iter().map(|s| s.to_string()));
        parts
    }
}

// ============================================================================
// Cargo Command Model
// ============================================================================

/// Cargo subcommand.
///
/// Models the `cargo <subcommand>` process tree. Each variant corresponds
/// to a top-level cargo subcommand with its own set of applicable flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subcommand {
    Build,
    Test,
    Check,
    Clippy,
    Fmt,
    /// `cargo run` with a specific binary invocation.
    Run(CargoInvocation),
}

impl Subcommand {
    /// The subcommand string as used on the command line.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Build => "build",
            Self::Test => "test",
            Self::Check => "check",
            Self::Clippy => "clippy",
            Self::Fmt => "fmt",
            Self::Run(_) => "run",
        }
    }

    /// Whether this subcommand invokes the Rust compiler (affected by RUSTFLAGS).
    pub fn compiles(&self) -> bool {
        matches!(self, Self::Build | Self::Test | Self::Check | Self::Run(_))
    }
}

// ============================================================================
// Binary Arguments
// ============================================================================

/// Subcommand for the codegen binary (gunbc-codegen).
///
/// The codegen binary has multiple modes of operation, selected via
/// a positional subcommand argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodegenSubcommand {
    /// Default: full commit (generate CLIs, build binaries, create bin directory).
    #[default]
    Commit,
    /// Just generate CLIs (partial commit).
    Codegen,
    /// Generate DAG files.
    Daggen,
    /// Generate CI workflow YAML (GitHub Actions and GitLab CI).
    Cigen,
    /// Remove all generated artifacts.
    Rollback,
}

impl CodegenSubcommand {
    /// The subcommand string as used on the command line.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Codegen => "codegen",
            Self::Daggen => "daggen",
            Self::Cigen => "cigen",
            Self::Rollback => "rollback",
        }
    }
}

/// Binary-specific arguments.
///
/// Models the arguments passed after `--` to cargo run, in a typed way.
/// Each variant corresponds to a specific binary's CLI interface. This
/// eliminates stringly-typed escape hatches by providing typed constructors.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BinaryArgs {
    /// No binary arguments.
    #[default]
    None,
    /// Subcommand for gunbc-codegen binary.
    Codegen(CodegenSubcommand),
    /// Mode argument for binaries that support ExecMode (testgen, makegen, etc.).
    WithMode(ExecMode),
}

impl BinaryArgs {
    /// Create args for codegen subcommand.
    pub fn codegen(sub: CodegenSubcommand) -> Self {
        Self::Codegen(sub)
    }

    /// Create args for binaries that take --mode flag.
    pub fn with_mode(mode: ExecMode) -> Self {
        Self::WithMode(mode)
    }

    /// Convert to trailing args for cargo run.
    pub fn to_args(&self) -> Vec<String> {
        match self {
            Self::None => vec![],
            Self::Codegen(sub) => vec![sub.as_str().to_string()],
            Self::WithMode(mode) => vec![format!("--mode={mode}")],
        }
    }

    /// Returns true if there are no arguments.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// How compiler warnings should be treated.
///
/// The rendering layer owns how this is expressed per subcommand:
/// - `Clippy`: `-- -D warnings` (clippy driver flag)
/// - `Build`/`Test`/`Check`: `RUSTFLAGS="-D warnings"` (env var)
/// - `Run`: `RUSTFLAGS="-D warnings"` (env var)
/// - `Fmt`: no effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Warnings {
    /// Default compiler behavior (warnings are warnings).
    #[default]
    Default,
    /// Promote all warnings to compile errors (`-D warnings`).
    Deny,
}

/// Cargo terminal color output mode.
///
/// Maps to the `CARGO_TERM_COLOR` environment variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TermColor {
    /// Automatic detection (default).
    #[default]
    Auto,
    /// Always emit color.
    Always,
    /// Never emit color.
    Never,
}

impl TermColor {
    /// The value for `CARGO_TERM_COLOR`.
    pub fn as_env_value(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "always",
            Self::Never => "never",
        }
    }
}

/// Cargo environment configuration.
///
/// Repo-level cargo settings that affect all commands. These are expressed
/// as environment variables by the rendering layer.
#[derive(Debug, Clone, Default)]
pub struct CargoEnv {
    /// Terminal color output mode (`CARGO_TERM_COLOR`).
    pub term_color: TermColor,
    /// Warning policy applied to compilation subcommands.
    pub warnings: Warnings,
}

impl CargoEnv {
    /// Standard CI configuration: colored output + warnings-as-errors.
    pub fn ci() -> Self {
        Self {
            term_color: TermColor::Always,
            warnings: Warnings::Deny,
        }
    }

    /// Build the environment variable map for CI/shell contexts.
    ///
    /// Returns all cargo-related env vars that should be set. The rendering
    /// layer decides where to place them (workflow-level env, Makefile export,
    /// or inline shell prefix).
    pub fn to_env_map(&self) -> Vec<(String, String)> {
        let mut env = Vec::new();
        // Always emit CARGO_TERM_COLOR unless it's the default (auto)
        if self.term_color != TermColor::Auto {
            env.push((
                "CARGO_TERM_COLOR".to_string(),
                self.term_color.as_env_value().to_string(),
            ));
        }
        // RUSTFLAGS for warning denial (applies to build/test/check)
        if self.warnings == Warnings::Deny {
            env.push(("RUSTFLAGS".to_string(), "-D warnings".to_string()));
        }
        env
    }
}

/// A fully-specified cargo command invocation.
///
/// Combines a subcommand with its flags and the repo's warning policy.
/// The rendering methods produce correct output per subcommand:
///
/// ```text
/// CargoCommand::new(Subcommand::Clippy)
///     .all_targets()
///     .warnings(Warnings::Deny)
///
/// → args:  ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
/// → env:   []  (clippy handles it via trailing args)
///
/// CargoCommand::new(Subcommand::Test)
///     .warnings(Warnings::Deny)
///
/// → args:  ["cargo", "test"]
/// → env:   [("RUSTFLAGS", "-D warnings")]
/// ```
#[derive(Debug, Clone)]
pub struct CargoCommand {
    pub subcommand: Subcommand,
    pub with_all_targets: bool,
    pub with_release: bool,
    pub with_warnings: Warnings,
    /// Enable `--fix` (clippy auto-fix mode).
    pub with_fix: bool,
    /// Enable `--workspace` (operate on all workspace members).
    pub with_workspace: bool,
    /// Enable `--allow-dirty` (allow uncommitted changes).
    pub with_allow_dirty: bool,
    /// Enable `--allow-staged` (allow staged changes).
    pub with_allow_staged: bool,
    /// Enable `--check` (fmt verification mode, no modifications).
    pub with_check: bool,
    /// Enable `--lib` (test only library unit tests).
    pub with_lib: bool,
    /// Enable `--no-run` (compile tests without executing them).
    pub with_no_run: bool,
    /// Typed binary arguments (only for `Subcommand::Run`).
    ///
    /// These are passed after `--` to the compiled binary. Use typed
    /// constructors like `BinaryArgs::codegen()` or `BinaryArgs::with_mode()`
    /// rather than raw strings.
    pub binary_args: BinaryArgs,
}

impl CargoCommand {
    /// Create a new cargo command for the given subcommand.
    pub fn new(subcommand: Subcommand) -> Self {
        Self {
            subcommand,
            with_all_targets: false,
            with_release: false,
            with_warnings: Warnings::Default,
            with_fix: false,
            with_workspace: false,
            with_allow_dirty: false,
            with_allow_staged: false,
            with_check: false,
            with_lib: false,
            with_no_run: false,
            binary_args: BinaryArgs::None,
        }
    }

    /// Enable `--all-targets`.
    pub fn all_targets(mut self) -> Self {
        self.with_all_targets = true;
        self
    }

    /// Enable `--release`.
    pub fn release(mut self) -> Self {
        self.with_release = true;
        self
    }

    /// Set the warning policy.
    pub fn warnings(mut self, w: Warnings) -> Self {
        self.with_warnings = w;
        self
    }

    /// Enable `--fix` (clippy auto-fix mode).
    pub fn fix(mut self) -> Self {
        self.with_fix = true;
        self
    }

    /// Enable `--workspace` (operate on all workspace members).
    pub fn workspace(mut self) -> Self {
        self.with_workspace = true;
        self
    }

    /// Enable `--allow-dirty` (allow uncommitted changes).
    pub fn allow_dirty(mut self) -> Self {
        self.with_allow_dirty = true;
        self
    }

    /// Enable `--allow-staged` (allow staged changes).
    pub fn allow_staged(mut self) -> Self {
        self.with_allow_staged = true;
        self
    }

    /// Enable `--check` (fmt verification mode).
    pub fn check(mut self) -> Self {
        self.with_check = true;
        self
    }

    /// Enable `--lib` (test only library unit tests).
    pub fn lib_only(mut self) -> Self {
        self.with_lib = true;
        self
    }

    /// Enable `--no-run` (for `cargo test` compile-only mode).
    pub fn no_run(mut self) -> Self {
        self.with_no_run = true;
        self
    }

    /// Set typed binary arguments (only for `Subcommand::Run`).
    ///
    /// Use `BinaryArgs::codegen()` for codegen subcommands or
    /// `BinaryArgs::with_mode()` for binaries that take --mode.
    pub fn args(mut self, args: BinaryArgs) -> Self {
        self.binary_args = args;
        self
    }

    /// Build the full command-line argument vector.
    ///
    /// For `Clippy` with `Warnings::Deny`, the `-D warnings` flags are
    /// appended to the trailing args (after `--`). For compilation subcommands,
    /// the warning policy is expressed via `env()` instead.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec!["cargo".to_string()];

        match &self.subcommand {
            Subcommand::Run(inv) => {
                args.push("run".to_string());
                match &inv.package {
                    Some(pkg) => {
                        args.extend([
                            "-p".to_string(),
                            pkg.clone(),
                            "--bin".to_string(),
                            inv.binary.clone(),
                        ]);
                    }
                    None => {
                        args.extend(["-p".to_string(), inv.binary.clone()]);
                    }
                }
            }
            sub => args.push(sub.as_str().to_string()),
        }

        if self.with_all_targets {
            args.push("--all-targets".to_string());
        }
        if self.with_release {
            args.push("--release".to_string());
        }
        if self.with_fix {
            args.push("--fix".to_string());
        }
        if self.with_workspace {
            args.push("--workspace".to_string());
        }
        if self.with_allow_dirty {
            args.push("--allow-dirty".to_string());
        }
        if self.with_allow_staged {
            args.push("--allow-staged".to_string());
        }
        if self.with_check {
            args.push("--check".to_string());
        }
        if self.with_lib {
            args.push("--lib".to_string());
        }
        if self.with_no_run {
            args.push("--no-run".to_string());
        }

        // Build trailing args:
        // - For Run: binary_args.to_args()
        // - For Clippy with Deny: append -D warnings
        let mut trailing = self.binary_args.to_args();
        if self.with_warnings == Warnings::Deny && self.subcommand == Subcommand::Clippy {
            trailing.extend(["-D".to_string(), "warnings".to_string()]);
        }

        if !trailing.is_empty() {
            args.push("--".to_string());
            args.extend(trailing);
        }

        args
    }

    /// Environment variables required by this command's configuration.
    ///
    /// For compilation subcommands (`build`, `test`, `check`) with
    /// `Warnings::Deny`, returns `RUSTFLAGS="-D warnings"`.
    /// Clippy handles warnings via trailing args instead.
    pub fn env(&self) -> Vec<(String, String)> {
        let mut env = Vec::new();
        if self.with_warnings == Warnings::Deny && self.subcommand.compiles() {
            env.push(("RUSTFLAGS".to_string(), "-D warnings".to_string()));
        }
        env
    }

    /// Render as a shell command string (without env prefix).
    pub fn to_shell(&self) -> String {
        self.to_args().join(" ")
    }

    /// Render as a shell command string with env prefix.
    ///
    /// Produces `RUSTFLAGS="-D warnings" cargo test` when env vars are needed.
    pub fn to_shell_with_env(&self) -> String {
        let env = self.env();
        if env.is_empty() {
            return self.to_shell();
        }
        let prefix: Vec<String> = env.iter().map(|(k, v)| format!("{k}=\"{v}\"")).collect();
        format!("{} {}", prefix.join(" "), self.to_shell())
    }

    /// Convert to a `ShellRequest` for transport execution.
    pub fn to_shell_request(&self) -> crate::transport::ShellRequest {
        let args = self.to_args();
        let mut req = crate::transport::ShellRequest::new(&args[0]).args(args[1..].iter().cloned());
        for (k, v) in self.env() {
            req = req.env(k, v);
        }
        req
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_composition() {
        assert_eq!(name("ci"), "gunbc-ci");
        assert_eq!(name("gist"), "gunbc-gist");
        assert_eq!(name("dag"), "gunbc-dag");
    }

    #[test]
    fn test_standalone() {
        let inv = CargoInvocation::standalone("gist");
        assert_eq!(inv.binary, "gunbc-gist");
        assert_eq!(inv.package, None);
        assert_eq!(inv.args(), "-p gunbc-gist");
        assert_eq!(inv.command(), "cargo run -p gunbc-gist");
    }

    #[test]
    fn test_composed() {
        let inv = CargoInvocation::composed("ci", "dag");
        assert_eq!(inv.binary, "gunbc-ci");
        assert_eq!(inv.package, Some("gunbc-dag".to_string()));
        assert_eq!(inv.args(), "-p gunbc-dag --bin gunbc-ci");
        assert_eq!(inv.command(), "cargo run -p gunbc-dag --bin gunbc-ci");
    }

    #[test]
    fn test_standalone_package() {
        let inv = CargoInvocation::new("gunbc-gist");
        assert_eq!(inv.args(), "-p gunbc-gist");
        assert_eq!(inv.command(), "cargo run -p gunbc-gist");
    }

    #[test]
    fn test_binary_in_package() {
        let inv = CargoInvocation::in_package("gunbc-ci", "gunbc-dag");
        assert_eq!(inv.args(), "-p gunbc-dag --bin gunbc-ci");
        assert_eq!(inv.command(), "cargo run -p gunbc-dag --bin gunbc-ci");
    }

    #[test]
    fn test_command_parts_standalone() {
        let inv = CargoInvocation::standalone("gist");
        assert_eq!(
            inv.command_parts(),
            vec!["cargo", "run", "-p", "gunbc-gist"]
        );
    }

    #[test]
    fn test_command_parts_composed() {
        let inv = CargoInvocation::composed("ci", "dag");
        assert_eq!(
            inv.command_parts(),
            vec!["cargo", "run", "-p", "gunbc-dag", "--bin", "gunbc-ci"]
        );
    }

    #[test]
    fn test_run_with_args() {
        let inv = CargoInvocation::standalone("codegen");
        assert_eq!(
            inv.run_with_args(&["--release", "--", "codegen"]),
            vec![
                "cargo",
                "run",
                "-p",
                "gunbc-codegen",
                "--release",
                "--",
                "codegen"
            ]
        );
    }

    // ========================================================================
    // CargoCommand Tests
    // ========================================================================

    #[test]
    fn test_cargo_build_basic() {
        let cmd = CargoCommand::new(Subcommand::Build);
        assert_eq!(cmd.to_args(), vec!["cargo", "build"]);
        assert!(cmd.env().is_empty());
    }

    #[test]
    fn test_cargo_build_all_targets() {
        let cmd = CargoCommand::new(Subcommand::Build).all_targets();
        assert_eq!(cmd.to_args(), vec!["cargo", "build", "--all-targets"]);
    }

    #[test]
    fn test_cargo_test_deny_warnings() {
        let cmd = CargoCommand::new(Subcommand::Test).warnings(Warnings::Deny);
        // Test doesn't add trailing args for warnings - uses env instead
        assert_eq!(cmd.to_args(), vec!["cargo", "test"]);
        assert_eq!(
            cmd.env(),
            vec![("RUSTFLAGS".to_string(), "-D warnings".to_string())]
        );
    }

    #[test]
    fn test_cargo_test_no_run_deny_warnings() {
        let cmd = CargoCommand::new(Subcommand::Test)
            .no_run()
            .warnings(Warnings::Deny);
        assert_eq!(cmd.to_args(), vec!["cargo", "test", "--no-run"]);
        assert_eq!(
            cmd.env(),
            vec![("RUSTFLAGS".to_string(), "-D warnings".to_string())]
        );
    }

    #[test]
    fn test_cargo_clippy_deny_warnings() {
        let cmd = CargoCommand::new(Subcommand::Clippy)
            .all_targets()
            .warnings(Warnings::Deny);
        // Clippy uses trailing args for -D warnings, not env
        assert_eq!(
            cmd.to_args(),
            vec!["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
        );
        assert!(cmd.env().is_empty());
    }

    #[test]
    fn test_cargo_clippy_fix() {
        let cmd = CargoCommand::new(Subcommand::Clippy)
            .fix()
            .workspace()
            .allow_dirty()
            .allow_staged()
            .warnings(Warnings::Deny);
        assert_eq!(
            cmd.to_args(),
            vec![
                "cargo",
                "clippy",
                "--fix",
                "--workspace",
                "--allow-dirty",
                "--allow-staged",
                "--",
                "-D",
                "warnings"
            ]
        );
    }

    #[test]
    fn test_cargo_fmt_check() {
        let cmd = CargoCommand::new(Subcommand::Fmt).check();
        assert_eq!(cmd.to_args(), vec!["cargo", "fmt", "--check"]);
    }

    #[test]
    fn test_cargo_run_release() {
        let inv = CargoInvocation::standalone("codegen");
        let cmd = CargoCommand::new(Subcommand::Run(inv))
            .release()
            .args(BinaryArgs::codegen(CodegenSubcommand::Codegen));
        assert_eq!(
            cmd.to_args(),
            vec![
                "cargo",
                "run",
                "-p",
                "gunbc-codegen",
                "--release",
                "--",
                "codegen"
            ]
        );
    }

    #[test]
    fn test_cargo_run_deny_warnings() {
        let inv = CargoInvocation::standalone("codegen");
        let cmd = CargoCommand::new(Subcommand::Run(inv)).warnings(Warnings::Deny);
        assert_eq!(
            cmd.env(),
            vec![("RUSTFLAGS".to_string(), "-D warnings".to_string())]
        );
        assert_eq!(
            cmd.to_shell_with_env(),
            "RUSTFLAGS=\"-D warnings\" cargo run -p gunbc-codegen"
        );
    }

    #[test]
    fn test_to_shell() {
        let cmd = CargoCommand::new(Subcommand::Build).all_targets();
        assert_eq!(cmd.to_shell(), "cargo build --all-targets");
    }

    #[test]
    fn test_to_shell_with_env() {
        let cmd = CargoCommand::new(Subcommand::Test).warnings(Warnings::Deny);
        assert_eq!(
            cmd.to_shell_with_env(),
            "RUSTFLAGS=\"-D warnings\" cargo test"
        );
    }

    #[test]
    fn test_to_shell_with_env_no_env() {
        let cmd = CargoCommand::new(Subcommand::Clippy)
            .all_targets()
            .warnings(Warnings::Deny);
        // Clippy has no env (uses trailing args), so to_shell_with_env == to_shell
        assert_eq!(
            cmd.to_shell_with_env(),
            "cargo clippy --all-targets -- -D warnings"
        );
    }

    #[test]
    fn test_cargo_env_ci() {
        let env = CargoEnv::ci();
        let map = env.to_env_map();
        assert!(map.contains(&("CARGO_TERM_COLOR".to_string(), "always".to_string())));
        assert!(map.contains(&("RUSTFLAGS".to_string(), "-D warnings".to_string())));
    }

    #[test]
    fn test_cargo_env_default() {
        let env = CargoEnv::default();
        // Default produces no env vars (auto color, no warning denial)
        assert!(env.to_env_map().is_empty());
    }

    #[test]
    fn test_subcommand_compiles() {
        assert!(Subcommand::Build.compiles());
        assert!(Subcommand::Test.compiles());
        assert!(Subcommand::Check.compiles());
        assert!(Subcommand::Run(CargoInvocation::standalone("codegen")).compiles());
        assert!(!Subcommand::Clippy.compiles());
        assert!(!Subcommand::Fmt.compiles());
    }

    #[test]
    fn test_to_shell_request() {
        let cmd = CargoCommand::new(Subcommand::Build)
            .all_targets()
            .warnings(Warnings::Deny);
        let req = cmd.to_shell_request();
        assert_eq!(req.command, "cargo");
        assert_eq!(req.args, vec!["build", "--all-targets"]);
        assert_eq!(req.env.get("RUSTFLAGS"), Some(&"-D warnings".to_string()));
    }
}
