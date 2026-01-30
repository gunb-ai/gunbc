//! Clippy operations.
//!
//! Each operation in the Clippy DAG. The DAG is self-ensuring:
//! it checks if clippy is installed, installs if needed, then runs.
//!
//! This crate uses gunbc-cargo internally to verify the Rust toolchain
//! is available before attempting to install or run clippy.

use gunbc_cargo::CargoOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::process::Command;

/// Operations for the Clippy tool DAG.
#[derive(Debug, Clone)]
pub enum ClippyOp {
    /// Check if clippy is installed (verify command).
    /// Outputs: exists (Bool)
    CheckInstalled,
    
    /// Install clippy via rustup component.
    /// Outputs: created (Bool)
    Install,
    
    /// Run cargo clippy with the configured arguments.
    /// Outputs: success (Bool), stdout (String), stderr (String)
    Run {
        /// Additional arguments to pass to cargo clippy
        args: Vec<String>,
    },
}

impl ClippyOp {
    /// Create a Run operation with the given arguments.
    pub fn run(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Run {
            args: args.into_iter().map(Into::into).collect(),
        }
    }
    
    /// Create a Run operation with default lint arguments.
    pub fn lint_all() -> Self {
        Self::run(["--all-targets", "--", "-D", "warnings"])
    }
}

impl Executable for ClippyOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            ClippyOp::CheckInstalled => execute_check_installed(),
            ClippyOp::Install => execute_install(),
            ClippyOp::Run { args } => execute_run(args),
        }
    }
}

/// Check if clippy is installed by running `cargo clippy --version`.
fn execute_check_installed() -> Result<HashMap<String, Value>, ExecError> {
    let output = Command::new("cargo")
        .args(["clippy", "--version"])
        .output()
        .map_err(|e| ExecError::new(format!("Failed to check clippy: {}", e)))?;
    
    let exists = output.status.success();
    
    let mut out = HashMap::new();
    out.insert("exists".to_string(), Value::Bool(exists));
    Ok(out)
}

/// Install clippy via rustup component add.
///
/// This first verifies that rustup is available (via CargoOp::CheckRustup),
/// ensuring the transitive dependency on the Rust toolchain is satisfied.
fn execute_install() -> Result<HashMap<String, Value>, ExecError> {
    // First, verify rustup is available (transitive dependency via gunbc-cargo)
    let rustup_check = CargoOp::CheckRustup.execute(HashMap::new())?;
    let rustup_exists = rustup_check
        .get("exists")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    if !rustup_exists {
        return Err(ExecError::new(
            "rustup not found. Clippy requires rustup for installation. \
             Visit https://rustup.rs to install the Rust toolchain."
        ));
    }
    
    println!("Installing clippy via rustup...");
    
    let output = Command::new("rustup")
        .args(["component", "add", "clippy"])
        .output()
        .map_err(|e| ExecError::new(format!("Failed to install clippy: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ExecError::new(format!("Failed to install clippy: {}", stderr)));
    }
    
    println!("Clippy installed successfully");
    
    let mut out = HashMap::new();
    out.insert("created".to_string(), Value::Bool(true));
    Ok(out)
}

/// Run cargo clippy with the given arguments.
fn execute_run(args: &[String]) -> Result<HashMap<String, Value>, ExecError> {
    let mut cmd_args = vec!["clippy".to_string()];
    cmd_args.extend(args.iter().cloned());
    
    println!("Running: cargo {}", cmd_args.join(" "));
    
    let output = Command::new("cargo")
        .args(&cmd_args)
        .output()
        .map_err(|e| ExecError::new(format!("Failed to run clippy: {}", e)))?;
    
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

impl Mockable for ClippyOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            ClippyOp::CheckInstalled => {
                let mut out = HashMap::new();
                out.insert("exists".to_string(), Value::Bool(true));
                out
            }
            ClippyOp::Install => {
                let mut out = HashMap::new();
                out.insert("created".to_string(), Value::Bool(true));
                out
            }
            ClippyOp::Run { .. } => {
                let mut out = HashMap::new();
                out.insert("success".to_string(), Value::Bool(true));
                out.insert("stdout".to_string(), Value::Str("No warnings".to_string()));
                out.insert("stderr".to_string(), Value::Str(String::new()));
                out
            }
        }
    }
}
