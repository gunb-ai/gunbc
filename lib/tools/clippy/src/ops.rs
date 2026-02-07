//! Clippy operations.
//!
//! Re-exports the generic `CliToolOp` configured for Clippy.
//! Most code should use `build_clippy_upsert()` from the graph module
//! to get a fractal sub-DAG, but direct operations are available here.

pub use gunbc_ir::transport::cli::CliToolOp;
use gunbc_ir::transport::cli::{self, CliToolError};
use gunbc_ir::Value;
use gunbc_lib_transport::cli::execute_cli_tool_op;
use std::collections::HashMap;

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

    /// Execute the full upsert imperatively (for simple cases).
    ///
    /// Prefer `build_clippy_upsert()` for the fractal DAG approach.
    pub fn upsert_and_run(args: &[&str]) -> Result<HashMap<String, Value>, CliToolError> {
        // Check
        let check_result = execute_cli_tool_op(&Self::check())?;
        let exists = check_result
            .get("exists")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Install if needed
        if !exists {
            execute_cli_tool_op(&Self::install())?;
        }

        // Run
        execute_cli_tool_op(&Self::run(args))
    }
}
