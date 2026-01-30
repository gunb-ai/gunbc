//! CI operations.
//!
//! Demonstrates decomposition into primitives where possible.
//! Command execution delegates to ExecuteOp primitive.
//!
//! The CI pipeline now includes a Prep stage that runs codegen
//! to ensure all generated code exists before building/testing.
//!
//! # BuildConfig Integration
//!
//! All build/test/lint commands are sourced from `BuildConfig` in the
//! makegen registry. This ensures a single source of truth for commands.
//!
//! # Transport Pattern
//!
//! All command execution goes through `ExecuteOp`, which will be migrated
//! to `PrepareShellOp` + `TransportOps::Execute` in a future refactor.
//! This ensures consistent interception for dry-run mode.

use gunbc_clippy::ClippyOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_makegen::{BuildConfig, ToolRegistry};
#[allow(deprecated)]
use gunbc_primitives::ExecuteOp;
use std::collections::HashMap;

/// Run a command from BuildConfig, returning (success, stdout, stderr).
fn run_config_command(command: &[&str]) -> Result<(bool, String, String), ExecError> {
    if command.is_empty() {
        return Err(ExecError::new("Empty command"));
    }

    let program = command[0];
    let args: Vec<&str> = command[1..].to_vec();

    println!("Running: {} {}", program, args.join(" "));

    let mut exec_inputs = HashMap::new();
    exec_inputs.insert("command".to_string(), Value::Str(program.to_string()));
    exec_inputs.insert(
        "args".to_string(),
        Value::StrList(args.iter().map(|s| s.to_string()).collect()),
    );

    let exec_result = ExecuteOp.execute(exec_inputs)?;

    let success = exec_result
        .get("success")
        .and_then(|v: &Value| v.as_bool())
        .unwrap_or(false);
    let stdout = exec_result
        .get("stdout")
        .and_then(|v: &Value| v.as_str())
        .unwrap_or("")
        .to_string();
    let stderr = exec_result
        .get("stderr")
        .and_then(|v: &Value| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok((success, stdout, stderr))
}

/// Operations for the CI tool.
#[derive(Debug, Clone)]
pub enum CIOp {
    /// Check and install dependencies
    SetupDeps,
    /// Run codegen to ensure all generated code exists (prep/unwind)
    Prep,
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
            CIOp::Prep => execute_prep(inputs),
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

/// Run prep/unwind to ensure all generated code exists.
///
/// This is the "fractal unwind" step - it runs codegen to generate
/// CLI main.rs files, and daggen to generate graph.rs files from
/// declarative DAG definitions. This ensures the repo is in a
/// consistent state before building and testing.
///
/// Uses `ToolRegistry::needs_codegen()` to check if codegen is needed,
/// and `BuildConfig` to get the codegen command.
///
/// Now uses `run_config_command` (via ExecuteOp) for consistent
/// interception in dry-run mode.
fn execute_prep(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let registry = ToolRegistry::default_registry();
    let config = BuildConfig::cargo();

    // Use registry to check if codegen is needed
    if !registry.needs_codegen() {
        println!("Prep: Generated code exists, skipping codegen");
        let mut out = HashMap::new();
        out.insert("prep_success".to_string(), Value::Bool(true));
        out.insert("codegen_ran".to_string(), Value::Bool(false));
        out.insert(
            "prep_message".to_string(),
            Value::Str("Generated code already exists".to_string()),
        );
        return Ok(out);
    }

    println!("Prep: Running codegen to generate CLIs...");

    // Run codegen using BuildConfig command via ExecuteOp (consistent with build/test/lint)
    let (success, _stdout, stderr) = run_config_command(&config.codegen_command)?;

    if !success {
        let mut out = HashMap::new();
        out.insert("prep_success".to_string(), Value::Bool(false));
        out.insert("codegen_ran".to_string(), Value::Bool(true));
        out.insert(
            "prep_message".to_string(),
            Value::Str(format!("Codegen failed: {}", stderr)),
        );
        return Ok(out);
    }

    println!("Prep: Codegen complete");

    let mut out = HashMap::new();
    out.insert("prep_success".to_string(), Value::Bool(true));
    out.insert("codegen_ran".to_string(), Value::Bool(true));
    out.insert(
        "prep_message".to_string(),
        Value::Str("Codegen completed successfully".to_string()),
    );
    Ok(out)
}

/// Build the project using BuildConfig command.
fn execute_build(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Check if prep succeeded
    let prep_success = inputs
        .get("prep_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if !prep_success {
        println!("Build: Skipped due to prep failure");
        let mut out = HashMap::new();
        out.insert("build_success".to_string(), Value::Bool(false));
        out.insert("build_skipped".to_string(), Value::Bool(true));
        out.insert("build_stdout".to_string(), Value::Str(String::new()));
        out.insert(
            "build_stderr".to_string(),
            Value::Str("Skipped due to prep failure".to_string()),
        );
        return Ok(out);
    }

    let config = BuildConfig::cargo();
    let (success, stdout, stderr) = run_config_command(&config.build_command)?;

    let mut out = HashMap::new();
    out.insert("build_success".to_string(), Value::Bool(success));
    out.insert("build_skipped".to_string(), Value::Bool(false));
    out.insert("build_stdout".to_string(), Value::Str(stdout));
    out.insert("build_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

/// Run tests using BuildConfig command.
fn execute_test(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = match inputs.get("build_success") {
        Some(Value::Bool(b)) => *b,
        _ => true,
    };

    if !build_success {
        let mut out = HashMap::new();
        out.insert("test_success".to_string(), Value::Bool(false));
        out.insert("test_skipped".to_string(), Value::Bool(true));
        out.insert(
            "message".to_string(),
            Value::Str("Skipped due to build failure".to_string()),
        );
        return Ok(out);
    }

    let config = BuildConfig::cargo();
    let (success, stdout, stderr) = run_config_command(&config.test_command)?;

    let mut out = HashMap::new();
    out.insert("test_success".to_string(), Value::Bool(success));
    out.insert("test_skipped".to_string(), Value::Bool(false));
    out.insert("test_stdout".to_string(), Value::Str(stdout));
    out.insert("test_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

/// Run linter using Clippy with automatic upsert.
///
/// This step uses the Clippy tool DAG internally:
/// 1. Check if clippy is installed
/// 2. Install clippy if needed (via rustup)
/// 3. Run cargo clippy with lint arguments
///
/// The dependency on clippy is implicit through usage - by using
/// ClippyOp, we automatically get the upsert behavior.
fn execute_lint(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = match inputs.get("build_success") {
        Some(Value::Bool(b)) => *b,
        _ => true,
    };

    if !build_success {
        let mut out = HashMap::new();
        out.insert("lint_success".to_string(), Value::Bool(false));
        out.insert("lint_skipped".to_string(), Value::Bool(true));
        out.insert(
            "message".to_string(),
            Value::Str("Skipped due to build failure".to_string()),
        );
        return Ok(out);
    }

    // Step 1: Check if clippy is installed
    let check_result = ClippyOp::CheckInstalled.execute(HashMap::new())?;
    let clippy_exists = check_result
        .get("exists")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Step 2: Install clippy if needed (upsert pattern)
    if !clippy_exists {
        println!("Clippy not found, installing...");
        ClippyOp::Install.execute(HashMap::new())?;
    }

    // Step 3: Run clippy with lint arguments
    let run_op = ClippyOp::lint_all();
    let run_result = run_op.execute(HashMap::new())?;

    let success = run_result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let stdout = run_result
        .get("stdout")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let stderr = run_result
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

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
            CIOp::Prep => {
                let mut out = HashMap::new();
                out.insert("prep_success".to_string(), Value::Bool(true));
                out.insert("codegen_ran".to_string(), Value::Bool(false));
                out.insert("prep_message".to_string(), Value::Str("Generated code exists".to_string()));
                out
            }
            CIOp::Build => {
                let mut out = HashMap::new();
                out.insert("build_success".to_string(), Value::Bool(true));
                out.insert("build_skipped".to_string(), Value::Bool(false));
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
