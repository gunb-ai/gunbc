//! Cargo/Rust toolchain operations.
//!
//! Each operation in the Cargo DAG. The DAG is self-ensuring:
//! it checks if cargo/rustc is installed, provides guidance if not,
//! then runs cargo commands.
//!
//! Note: Unlike other tools, Rust/Cargo installation typically requires
//! user interaction (rustup.rs) or is pre-installed by the system/CI.
//! This DAG focuses on verification and providing clear error messages.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::process::Command;

/// Operations for the Cargo tool DAG.
#[derive(Debug, Clone)]
pub enum CargoOp {
    /// Check if cargo is installed (verify command).
    /// Outputs: exists (Bool), version (String)
    CheckInstalled,
    
    /// Check if rustup is available for component management.
    /// Outputs: exists (Bool)
    CheckRustup,
    
    /// Run a cargo command with the given subcommand and args.
    /// Outputs: success (Bool), stdout (String), stderr (String)
    Run {
        /// Subcommand (e.g., "build", "test", "check")
        subcommand: String,
        /// Additional arguments
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
}

impl Executable for CargoOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CargoOp::CheckInstalled => execute_check_installed(),
            CargoOp::CheckRustup => execute_check_rustup(),
            CargoOp::Run { subcommand, args } => execute_run(subcommand, args),
        }
    }
}

/// Check if cargo is installed by running `cargo --version`.
fn execute_check_installed() -> Result<HashMap<String, Value>, ExecError> {
    let output = Command::new("cargo")
        .args(["--version"])
        .output()
        .map_err(|e| ExecError::new(format!("Failed to check cargo: {}", e)))?;
    
    let exists = output.status.success();
    let version = if exists {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        String::new()
    };
    
    let mut out = HashMap::new();
    out.insert("exists".to_string(), Value::Bool(exists));
    out.insert("version".to_string(), Value::Str(version));
    Ok(out)
}

/// Check if rustup is available.
fn execute_check_rustup() -> Result<HashMap<String, Value>, ExecError> {
    let output = Command::new("rustup")
        .args(["--version"])
        .output()
        .map_err(|e| ExecError::new(format!("Failed to check rustup: {}", e)))?;
    
    let exists = output.status.success();
    
    let mut out = HashMap::new();
    out.insert("exists".to_string(), Value::Bool(exists));
    Ok(out)
}

/// Run a cargo command.
fn execute_run(subcommand: &str, args: &[String]) -> Result<HashMap<String, Value>, ExecError> {
    let mut cmd_args = vec![subcommand.to_string()];
    cmd_args.extend(args.iter().cloned());
    
    println!("Running: cargo {}", cmd_args.join(" "));
    
    let output = Command::new("cargo")
        .args(&cmd_args)
        .output()
        .map_err(|e| ExecError::new(format!("Failed to run cargo {}: {}", subcommand, e)))?;
    
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    let mut out = HashMap::new();
    out.insert("success".to_string(), Value::Bool(success));
    out.insert("stdout".to_string(), Value::Str(stdout));
    out.insert("stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for CargoOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            CargoOp::CheckInstalled => {
                let mut out = HashMap::new();
                out.insert("exists".to_string(), Value::Bool(true));
                out.insert("version".to_string(), Value::Str("cargo 1.75.0".to_string()));
                out
            }
            CargoOp::CheckRustup => {
                let mut out = HashMap::new();
                out.insert("exists".to_string(), Value::Bool(true));
                out
            }
            CargoOp::Run { .. } => {
                let mut out = HashMap::new();
                out.insert("success".to_string(), Value::Bool(true));
                out.insert("stdout".to_string(), Value::Str("Build complete".to_string()));
                out.insert("stderr".to_string(), Value::Str(String::new()));
                out
            }
        }
    }
}
