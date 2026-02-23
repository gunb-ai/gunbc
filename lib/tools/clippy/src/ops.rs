//! Clippy operations.
//!
//! Re-exports the generic `CliToolOp` configured for Clippy.

use gunbc_ir::transport::cli;
pub use gunbc_ir::transport::cli::CliToolOp;

/// Convenience functions for Clippy-specific operations.
pub struct Clippy;

impl Clippy {
    /// Check if clippy is installed.
    pub fn check() -> CliToolOp {
        CliToolOp::check(&cli::CLIPPY)
    }

    /// Install clippy via rustup.
    pub fn install() -> CliToolOp {
        CliToolOp::install(&cli::CLIPPY)
    }

    /// Run clippy with arguments.
    pub fn run(args: &[&str]) -> CliToolOp {
        CliToolOp::run(&cli::CLIPPY, args)
    }

    /// Run clippy with lint-all flags.
    pub fn lint_all() -> CliToolOp {
        Self::run(&["--all-targets", "--", "-D", "warnings"])
    }
}
