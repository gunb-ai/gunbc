//! Cargo/Rust toolchain operations.
//!
//! This module provides Cargo-specific operations built on top of the
//! generic `CliToolOp` abstraction. Cargo operations are convenience
//! wrappers for common cargo commands.
//!
//! Note: Cargo does not support automatic installation (requires rustup.rs),
//! so the Install operation will fail with a helpful error message.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::resource::AccessMode;
use gunbc_ir::transport::cli::{self, CliToolOp};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Operations for the Cargo tool.
///
/// These map to `CliToolOp` variants but provide a Cargo-specific API.
#[derive(Debug, Clone)]
pub enum CargoOp {
    /// Check if cargo is installed.
    /// Outputs: exists (Bool), output (String)
    CheckInstalled,

    /// Check if rustup is available for component management.
    /// Outputs: exists (Bool), output (String)
    CheckRustup,

    /// Run a cargo command with the given subcommand and args.
    /// Outputs: success (Bool), stdout (String), stderr (String), exit_code (Int)
    Run {
        subcommand: String,
        args: Vec<String>,
    },
}

impl CargoOp {
    /// Create a Run operation for cargo build.
    pub fn build(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Run {
            subcommand: "build".to_string(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// Create a Run operation for cargo test.
    pub fn test(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Run {
            subcommand: "test".to_string(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// Create a Run operation for cargo check.
    pub fn check(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Run {
            subcommand: "check".to_string(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// Convert to the underlying generic CliToolOp.
    pub fn to_cli_op(&self) -> CliToolOp {
        match self {
            Self::CheckInstalled => CliToolOp::check(&cli::CARGO),
            Self::CheckRustup => {
                // Rustup check - use a custom tool def inline
                static RUSTUP: cli::CliToolDef = cli::CliToolDef {
                    id: "rustup",
                    check_cmd: &["rustup", "--version"],
                    install_cmd: None,
                    run_cmd: &["rustup"],
                    description: "Rust toolchain manager",
                    access_mode: AccessMode::Read,
                };
                CliToolOp::check(&RUSTUP)
            }
            Self::Run { subcommand, args } => {
                let mut full_args = vec![subcommand.clone()];
                full_args.extend(args.iter().cloned());
                CliToolOp::Run {
                    tool: &cli::CARGO,
                    args: full_args,
                }
            }
        }
    }
}

impl Executable for CargoOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        self.to_cli_op()
            .execute()
            .map_err(|e| ExecError::new(e.to_string()))
    }
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for CargoOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            CargoOp::CheckInstalled => {
                let mut out = HashMap::new();
                out.insert("exists".to_string(), Value::Bool(true));
                out.insert("output".to_string(), Value::Str("cargo 1.75.0".to_string()));
                out
            }
            CargoOp::CheckRustup => {
                let mut out = HashMap::new();
                out.insert("exists".to_string(), Value::Bool(true));
                out.insert("output".to_string(), Value::Str("rustup 1.26.0".to_string()));
                out
            }
            CargoOp::Run { .. } => {
                let mut out = HashMap::new();
                out.insert("success".to_string(), Value::Bool(true));
                out.insert("exit_code".to_string(), Value::Int(0));
                out.insert("stdout".to_string(), Value::Str("Build complete".to_string()));
                out.insert("stderr".to_string(), Value::Str(String::new()));
                out
            }
        }
    }
}
