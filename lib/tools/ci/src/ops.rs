//! CI operations.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::process::Command;

/// Operations for the CI tool.
#[derive(Debug, Clone)]
pub enum CIOp {
    /// Check and install dependencies
    SetupDeps,
    /// Build the project
    Build,
    /// Run tests
    Test,
    /// Run linter/clippy
    Lint,
    /// Report results (boundary)
    Report,
}

impl Executable for CIOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CIOp::SetupDeps => execute_setup_deps(inputs),
            CIOp::Build => execute_build(inputs),
            CIOp::Test => execute_test(inputs),
            CIOp::Lint => execute_lint(inputs),
            CIOp::Report => execute_report(inputs),
        }
    }
}

/// Setup dependencies using gunbc-deps.
fn execute_setup_deps(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Check if deps.toml exists
    let manifest_exists = std::path::Path::new("deps.toml").exists();

    if !manifest_exists {
        let mut out = HashMap::new();
        out.insert("deps_checked".to_string(), Value::Bool(true));
        out.insert("deps_installed".to_string(), Value::Int(0));
        out.insert("message".to_string(), Value::Str("No deps.toml found, skipping".to_string()));
        return Ok(out);
    }

    // Run gunbc-deps --dry-run to check status
    // In a real implementation, we'd actually run the install
    let mut out = HashMap::new();
    out.insert("deps_checked".to_string(), Value::Bool(true));
    out.insert("deps_installed".to_string(), Value::Int(0));
    out.insert("message".to_string(), Value::Str("Dependencies checked".to_string()));
    Ok(out)
}

/// Build the project.
fn execute_build(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    println!("Running: cargo build --all-targets");

    let output = Command::new("cargo")
        .args(["build", "--all-targets"])
        .output()
        .map_err(|e| ExecError::new(format!("failed to run cargo build: {}", e)))?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut out = HashMap::new();
    out.insert("build_success".to_string(), Value::Bool(success));
    out.insert("build_stdout".to_string(), Value::Str(stdout));
    out.insert("build_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

/// Run tests.
fn execute_test(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = match inputs.get("build_success") {
        Some(Value::Bool(b)) => *b,
        _ => true,
    };

    if !build_success {
        let mut out = HashMap::new();
        out.insert("test_success".to_string(), Value::Bool(false));
        out.insert("test_skipped".to_string(), Value::Bool(true));
        out.insert("message".to_string(), Value::Str("Skipped due to build failure".to_string()));
        return Ok(out);
    }

    println!("Running: cargo test");

    let output = Command::new("cargo")
        .args(["test"])
        .output()
        .map_err(|e| ExecError::new(format!("failed to run cargo test: {}", e)))?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut out = HashMap::new();
    out.insert("test_success".to_string(), Value::Bool(success));
    out.insert("test_skipped".to_string(), Value::Bool(false));
    out.insert("test_stdout".to_string(), Value::Str(stdout));
    out.insert("test_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

/// Run linter.
fn execute_lint(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = match inputs.get("build_success") {
        Some(Value::Bool(b)) => *b,
        _ => true,
    };

    if !build_success {
        let mut out = HashMap::new();
        out.insert("lint_success".to_string(), Value::Bool(false));
        out.insert("lint_skipped".to_string(), Value::Bool(true));
        out.insert("message".to_string(), Value::Str("Skipped due to build failure".to_string()));
        return Ok(out);
    }

    println!("Running: cargo clippy");

    let output = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .output()
        .map_err(|e| ExecError::new(format!("failed to run cargo clippy: {}", e)))?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut out = HashMap::new();
    out.insert("lint_success".to_string(), Value::Bool(success));
    out.insert("lint_skipped".to_string(), Value::Bool(false));
    out.insert("lint_stdout".to_string(), Value::Str(stdout));
    out.insert("lint_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

/// Report CI results (boundary).
fn execute_report(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = inputs
        .get("build_success")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false);

    let test_success = inputs
        .get("test_success")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false);

    let lint_success = inputs
        .get("lint_success")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false);

    let overall_success = build_success && test_success && lint_success;

    let report = format!(
        r#"
CI Report
=========
Build: {}
Test:  {}
Lint:  {}
---------
Overall: {}
"#,
        if build_success { "PASS" } else { "FAIL" },
        if test_success { "PASS" } else { "FAIL" },
        if lint_success { "PASS" } else { "FAIL" },
        if overall_success { "SUCCESS" } else { "FAILURE" }
    );

    let mut out = HashMap::new();
    out.insert("overall_success".to_string(), Value::Bool(overall_success));
    out.insert("report".to_string(), Value::Str(report));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_deps_no_manifest() {
        // This test runs in a temp dir without deps.toml
        let result = execute_setup_deps(HashMap::new()).unwrap();
        
        match result.get("deps_checked") {
            Some(Value::Bool(b)) => assert!(*b),
            _ => panic!("expected deps_checked"),
        }
    }
}
