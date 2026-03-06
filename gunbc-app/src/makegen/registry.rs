//! Build-system command policy for makegen.
//!
//! Tool target discovery now comes from DSL entrypoint inference. This module
//! only carries the command/rendering policy shared by the Makefile, Justfile,
//! and bootstrap projections.

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
}
