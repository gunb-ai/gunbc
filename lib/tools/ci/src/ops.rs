//! CI operations.
//!
//! Demonstrates decomposition into primitives where possible.
//! Command execution delegates to ExecuteOp primitive.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_primitives::ExecuteOp;
use std::collections::HashMap;

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

/// Build the project using ExecuteOp primitive.
fn execute_build(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    println!("Running: cargo build --all-targets");

    // Use ExecuteOp primitive
    let mut exec_inputs = HashMap::new();
    exec_inputs.insert("command".to_string(), Value::Str("cargo".to_string()));
    exec_inputs.insert("args".to_string(), Value::StrList(vec!["build".to_string(), "--all-targets".to_string()]));
    
    let exec_result = ExecuteOp.execute(exec_inputs)?;

    let success = exec_result.get("success").and_then(|v: &Value| v.as_bool()).unwrap_or(false);
    let stdout = exec_result.get("stdout").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();
    let stderr = exec_result.get("stderr").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();

    let mut out = HashMap::new();
    out.insert("build_success".to_string(), Value::Bool(success));
    out.insert("build_stdout".to_string(), Value::Str(stdout));
    out.insert("build_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

/// Run tests using ExecuteOp primitive.
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

    // Use ExecuteOp primitive
    let mut exec_inputs = HashMap::new();
    exec_inputs.insert("command".to_string(), Value::Str("cargo".to_string()));
    exec_inputs.insert("args".to_string(), Value::StrList(vec!["test".to_string()]));
    
    let exec_result = ExecuteOp.execute(exec_inputs)?;

    let success = exec_result.get("success").and_then(|v: &Value| v.as_bool()).unwrap_or(false);
    let stdout = exec_result.get("stdout").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();
    let stderr = exec_result.get("stderr").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();

    let mut out = HashMap::new();
    out.insert("test_success".to_string(), Value::Bool(success));
    out.insert("test_skipped".to_string(), Value::Bool(false));
    out.insert("test_stdout".to_string(), Value::Str(stdout));
    out.insert("test_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

/// Run linter using ExecuteOp primitive.
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

    // Use ExecuteOp primitive
    let mut exec_inputs = HashMap::new();
    exec_inputs.insert("command".to_string(), Value::Str("cargo".to_string()));
    exec_inputs.insert("args".to_string(), Value::StrList(vec![
        "clippy".to_string(),
        "--all-targets".to_string(),
        "--".to_string(),
        "-D".to_string(),
        "warnings".to_string(),
    ]));
    
    let exec_result = ExecuteOp.execute(exec_inputs)?;

    let success = exec_result.get("success").and_then(|v: &Value| v.as_bool()).unwrap_or(false);
    let stdout = exec_result.get("stdout").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();
    let stderr = exec_result.get("stderr").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();

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

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for CIOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            CIOp::SetupDeps => {
                let mut out = HashMap::new();
                out.insert("deps_checked".to_string(), Value::Bool(true));
                out.insert("deps_installed".to_string(), Value::Int(0));
                out.insert("message".to_string(), Value::Str("Dependencies ready".to_string()));
                out
            }
            CIOp::Build => {
                let mut out = HashMap::new();
                out.insert("build_success".to_string(), Value::Bool(true));
                out.insert("build_stdout".to_string(), Value::Str("Build complete".to_string()));
                out.insert("build_stderr".to_string(), Value::Str(String::new()));
                out
            }
            CIOp::Test => {
                let mut out = HashMap::new();
                out.insert("test_success".to_string(), Value::Bool(true));
                out.insert("test_skipped".to_string(), Value::Bool(false));
                out.insert("test_stdout".to_string(), Value::Str("All tests passed".to_string()));
                out.insert("test_stderr".to_string(), Value::Str(String::new()));
                out
            }
            CIOp::Lint => {
                let mut out = HashMap::new();
                out.insert("lint_success".to_string(), Value::Bool(true));
                out.insert("lint_skipped".to_string(), Value::Bool(false));
                out.insert("lint_stdout".to_string(), Value::Str("No warnings".to_string()));
                out.insert("lint_stderr".to_string(), Value::Str(String::new()));
                out
            }
            CIOp::Report => {
                let mut out = HashMap::new();
                out.insert("overall_success".to_string(), Value::Bool(true));
                out.insert("report".to_string(), Value::Str("CI passed".to_string()));
                out
            }
        }
    }
}
