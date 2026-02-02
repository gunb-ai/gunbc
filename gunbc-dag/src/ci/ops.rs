//! CI operations - all pure.
//!
//! This module follows the "every node is pure" principle:
//! - Pure ops prepare `TransportRequest` values or parse `TransportResponse`
//! - No `execute_transport()` calls - that happens in TransportOps::Execute
//! - No `println!` - logging is done via outputs
//!
//! # Transport Pattern
//!
//! ```text
//! [Prepare*Op] -> [TransportOps::Execute] -> [Parse*Op]
//!    (pure)           (boundary)              (pure)
//! ```

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use crate::makegen::BuildConfig;
use std::collections::HashMap;

// ============================================================================
// CIOp - Pure CI-specific operations
// ============================================================================

/// Pure operations for the CI tool.
///
/// All these operations are deterministic and have no side effects.
/// I/O happens only through `TransportOps::Execute` in the graph.
#[derive(Debug, Clone)]
pub enum CIOp {
    // ========== SetupDeps stage ==========
    /// Parse the deps.toml exists check result (pure)
    ParseDepsExists,

    // ========== Prep stage ==========
    /// Prepare file exists check for codegen directory (pure)
    /// Outputs: request: TransportRequest
    PrepareCodegenExistsCheck,
    /// Parse the codegen exists check result and decide if codegen needed (pure)
    /// Outputs: codegen_needed: Bool, prep_success: Bool (if exists)
    ParseCodegenExists,
    /// Prepare the codegen shell command (pure)
    /// Outputs: request: TransportRequest
    PrepareCodegenCommand,
    /// Parse the codegen shell response (pure)
    ParseCodegenResult,

    // ========== Build stage ==========
    /// Prepare the build shell command (pure)
    /// Inputs: prep_success: Bool
    /// Outputs: request: TransportRequest, skip: Bool
    PrepareBuildCommand,
    /// Parse the build shell response (pure)
    /// Inputs: response: TransportResponse, skip: Bool
    /// Outputs: build_success, build_skipped, build_stdout, build_stderr
    ParseBuildResult,

    // ========== Test stage ==========
    /// Prepare the test shell command (pure)
    PrepareTestCommand,
    /// Parse the test shell response (pure)
    ParseTestResult,

    // ========== Lint stage (receives tool handle from env node) ==========
    /// Prepare clippy lint - check if we should skip based on build_success (pure)
    /// Inputs: build_success: Bool
    /// Outputs: skip: Bool, skip_reason: String (if skipping)
    PrepareClippyLint,
    /// Parse clippy lint result - convert CliToolOp outputs to CI format (pure)
    /// Inputs: success: Bool, stdout: String, stderr: String, skip: Bool
    /// Outputs: lint_success, lint_skipped, lint_stdout, lint_stderr
    ParseClippyLintResult,

    // ========== Report stage (already pure) ==========
    /// Generate CI report (pure)
    Report,
}

impl Executable for CIOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CIOp::ParseDepsExists => execute_parse_deps_exists(inputs),
            CIOp::PrepareCodegenExistsCheck => execute_prepare_codegen_exists_check(inputs),
            CIOp::ParseCodegenExists => execute_parse_codegen_exists(inputs),
            CIOp::PrepareCodegenCommand => execute_prepare_codegen_command(inputs),
            CIOp::ParseCodegenResult => execute_parse_codegen_result(inputs),
            CIOp::PrepareBuildCommand => execute_prepare_build_command(inputs),
            CIOp::ParseBuildResult => execute_parse_build_result(inputs),
            CIOp::PrepareTestCommand => execute_prepare_test_command(inputs),
            CIOp::ParseTestResult => execute_parse_test_result(inputs),
            CIOp::PrepareClippyLint => execute_prepare_clippy_lint(inputs),
            CIOp::ParseClippyLintResult => execute_parse_clippy_lint_result(inputs),
            CIOp::Report => execute_report(inputs),
        }
    }
}

// ============================================================================
// SetupDeps Stage - Pure Operations
// ============================================================================

/// Parse the deps.toml exists check result (pure).
fn execute_parse_deps_exists(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Handle skipped response (upstream transport was skipped)
    if matches!(inputs.get("response"), Some(Value::Skipped)) {
        let mut out = HashMap::new();
        out.insert("deps_exists".to_string(), Value::Skipped);
        out.insert("deps_checked".to_string(), Value::Skipped);
        out.insert("deps_installed".to_string(), Value::Skipped);
        out.insert("message".to_string(), Value::Skipped);
        return Ok(out);
    }

    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing or invalid 'response' input"))?;

    let deps_exists = match response {
        TransportResponse::File(file_resp) => file_resp.exists.unwrap_or(false),
        _ => false,
    };

    let message = if deps_exists {
        "deps.toml found, dependencies will be checked"
    } else {
        "No deps.toml found, skipping dependency check"
    };

    let mut out = HashMap::new();
    out.insert("deps_exists".to_string(), Value::Bool(deps_exists));
    out.insert("deps_checked".to_string(), Value::Bool(true));
    out.insert("deps_installed".to_string(), Value::Int(0));
    out.insert("message".to_string(), Value::Str(message.to_string()));
    Ok(out)
}

// ============================================================================
// Prep Stage - Pure Operations
// ============================================================================

/// Prepare file exists check for codegen directory (pure).
fn execute_prepare_codegen_exists_check(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Check if the codegen output directory exists
    let request = TransportRequest::File(FileRequest::exists("buck-out/gen/bin"));
    
    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    Ok(out)
}

/// Parse the codegen exists check result (pure).
fn execute_parse_codegen_exists(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Handle skipped response (upstream transport was skipped)
    if matches!(inputs.get("response"), Some(Value::Skipped)) {
        let mut out = HashMap::new();
        out.insert("codegen_needed".to_string(), Value::Skipped);
        out.insert("prep_success".to_string(), Value::Skipped);
        out.insert("codegen_ran".to_string(), Value::Skipped);
        out.insert("prep_message".to_string(), Value::Skipped);
        return Ok(out);
    }

    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing response input"))?;

    let codegen_exists = match response {
        TransportResponse::File(file_resp) => file_resp.exists.unwrap_or(false),
        _ => false,
    };

    let mut out = HashMap::new();
    out.insert("codegen_needed".to_string(), Value::Bool(!codegen_exists));

    // If codegen exists, prep is already successful
    if codegen_exists {
        out.insert("prep_success".to_string(), Value::Bool(true));
        out.insert("codegen_ran".to_string(), Value::Bool(false));
        out.insert("prep_message".to_string(), Value::Str("Generated code already exists".to_string()));
    }

    Ok(out)
}

/// Prepare the codegen shell command (pure).
fn execute_prepare_codegen_command(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Check if codegen is needed
    let codegen_needed = inputs
        .get("codegen_needed")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut out = HashMap::new();
    
    if !codegen_needed {
        out.insert("skip".to_string(), Value::Bool(true));
        return Ok(out);
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.codegen.to_shell_request());

    out.insert("request".to_string(), Value::Request(request));
    out.insert("skip".to_string(), Value::Bool(false));
    Ok(out)
}

/// Parse the codegen shell response (pure).
fn execute_parse_codegen_result(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Handle skipped response (upstream transport was skipped)
    if matches!(inputs.get("response"), Some(Value::Skipped)) {
        let mut out = HashMap::new();
        out.insert("prep_success".to_string(), Value::Skipped);
        out.insert("codegen_ran".to_string(), Value::Skipped);
        out.insert("prep_message".to_string(), Value::Skipped);
        return Ok(out);
    }

    // Check if codegen was skipped
    let skip = inputs
        .get("skip")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut out = HashMap::new();

    if skip {
        // Codegen was skipped because it already exists
        out.insert("prep_success".to_string(), Value::Bool(true));
        out.insert("codegen_ran".to_string(), Value::Bool(false));
        out.insert("prep_message".to_string(), Value::Str("Generated code already exists".to_string()));
        return Ok(out);
    }

    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing response input"))?;

    let (success, _stdout, stderr) = match response {
        TransportResponse::Shell(shell) => (shell.success(), shell.stdout.clone(), shell.stderr.clone()),
        _ => return Err(ExecError::new("expected shell response")),
    };

    out.insert("prep_success".to_string(), Value::Bool(success));
    out.insert("codegen_ran".to_string(), Value::Bool(true));

    let message = if success {
        "Codegen completed successfully".to_string()
    } else {
        format!("Codegen failed: {}", stderr)
    };
    out.insert("prep_message".to_string(), Value::Str(message));

    Ok(out)
}

// ============================================================================
// Build Stage - Pure Operations
// ============================================================================

/// Prepare the build shell command (pure).
fn execute_prepare_build_command(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = inputs
        .get("prep_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut out = HashMap::new();

    if !prep_success {
        out.insert("skip".to_string(), Value::Bool(true));
        out.insert("skip_reason".to_string(), Value::Str("Skipped due to prep failure".to_string()));
        return Ok(out);
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.build.to_shell_request());

    out.insert("request".to_string(), Value::Request(request));
    out.insert("skip".to_string(), Value::Bool(false));
    Ok(out)
}

/// Parse the build shell response (pure).
fn execute_parse_build_result(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Handle skipped response (upstream transport was skipped)
    if matches!(inputs.get("response"), Some(Value::Skipped)) {
        let mut out = HashMap::new();
        out.insert("build_success".to_string(), Value::Skipped);
        out.insert("build_skipped".to_string(), Value::Skipped);
        out.insert("build_stdout".to_string(), Value::Skipped);
        out.insert("build_stderr".to_string(), Value::Skipped);
        return Ok(out);
    }

    let skip = inputs
        .get("skip")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut out = HashMap::new();

    if skip {
        let reason = inputs
            .get("skip_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("Skipped");
        out.insert("build_success".to_string(), Value::Bool(false));
        out.insert("build_skipped".to_string(), Value::Bool(true));
        out.insert("build_stdout".to_string(), Value::Str(String::new()));
        out.insert("build_stderr".to_string(), Value::Str(reason.to_string()));
        return Ok(out);
    }

    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing response input"))?;

    let (success, stdout, stderr) = match response {
        TransportResponse::Shell(shell) => (shell.success(), shell.stdout.clone(), shell.stderr.clone()),
        _ => return Err(ExecError::new("expected shell response")),
    };

    out.insert("build_success".to_string(), Value::Bool(success));
    out.insert("build_skipped".to_string(), Value::Bool(false));
    out.insert("build_stdout".to_string(), Value::Str(stdout));
    out.insert("build_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

// ============================================================================
// Test Stage - Pure Operations
// ============================================================================

/// Prepare the test shell command (pure).
fn execute_prepare_test_command(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = inputs
        .get("build_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut out = HashMap::new();

    if !build_success {
        out.insert("skip".to_string(), Value::Bool(true));
        out.insert("skip_reason".to_string(), Value::Str("Skipped due to build failure".to_string()));
        return Ok(out);
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.test.to_shell_request());

    out.insert("request".to_string(), Value::Request(request));
    out.insert("skip".to_string(), Value::Bool(false));
    Ok(out)
}

/// Parse the test shell response (pure).
fn execute_parse_test_result(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Handle skipped response (upstream transport was skipped)
    if matches!(inputs.get("response"), Some(Value::Skipped)) {
        let mut out = HashMap::new();
        out.insert("test_success".to_string(), Value::Skipped);
        out.insert("test_skipped".to_string(), Value::Skipped);
        out.insert("test_stdout".to_string(), Value::Skipped);
        out.insert("test_stderr".to_string(), Value::Skipped);
        return Ok(out);
    }

    let skip = inputs
        .get("skip")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut out = HashMap::new();

    if skip {
        let reason = inputs
            .get("skip_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("Skipped");
        out.insert("test_success".to_string(), Value::Bool(false));
        out.insert("test_skipped".to_string(), Value::Bool(true));
        out.insert("test_stdout".to_string(), Value::Str(String::new()));
        out.insert("test_stderr".to_string(), Value::Str(reason.to_string()));
        return Ok(out);
    }

    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing response input"))?;

    let (success, stdout, stderr) = match response {
        TransportResponse::Shell(shell) => (shell.success(), shell.stdout.clone(), shell.stderr.clone()),
        _ => return Err(ExecError::new("expected shell response")),
    };

    out.insert("test_success".to_string(), Value::Bool(success));
    out.insert("test_skipped".to_string(), Value::Bool(false));
    out.insert("test_stdout".to_string(), Value::Str(stdout));
    out.insert("test_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

// ============================================================================
// Lint Stage - Pure Operations (using Clippy SubDag)
// ============================================================================

/// Prepare clippy lint - check if we should skip based on build_success (pure).
///
/// This is the pre-gate for the Clippy tool execution. It checks if the build succeeded
/// and either allows the lint or signals to skip.
fn execute_prepare_clippy_lint(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = inputs
        .get("build_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut out = HashMap::new();

    if !build_success {
        out.insert("skip".to_string(), Value::Bool(true));
        out.insert("skip_reason".to_string(), Value::Str("Skipped due to build failure".to_string()));
        return Ok(out);
    }

    out.insert("skip".to_string(), Value::Bool(false));
    Ok(out)
}

/// Parse clippy lint result - convert CliToolOp outputs to CI format (pure).
///
/// This is the post-parse for the Clippy SubDag. It converts the clippy run
/// outputs (success, stdout, stderr) to the expected CI format.
fn execute_parse_clippy_lint_result(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let skip = inputs
        .get("skip")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut out = HashMap::new();

    if skip {
        let reason = inputs
            .get("skip_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("Skipped");
        out.insert("lint_success".to_string(), Value::Bool(false));
        out.insert("lint_skipped".to_string(), Value::Bool(true));
        out.insert("lint_stdout".to_string(), Value::Str(String::new()));
        out.insert("lint_stderr".to_string(), Value::Str(reason.to_string()));
        return Ok(out);
    }

    // Get outputs from the clippy SubDag (from the 'resolve' node which runs clippy)
    let success = inputs
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let stdout = inputs
        .get("stdout")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let stderr = inputs
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    out.insert("lint_success".to_string(), Value::Bool(success));
    out.insert("lint_skipped".to_string(), Value::Bool(false));
    out.insert("lint_stdout".to_string(), Value::Str(stdout));
    out.insert("lint_stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

// ============================================================================
// Report Stage - Already Pure
// ============================================================================

/// Generate CI report (pure).
///
/// When a stage fails, its stderr is included below the summary so
/// developers can see what went wrong without expanding individual
/// CI groups. For test failures, the "failures:" section from stdout
/// is also extracted and shown.
fn execute_report(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = inputs
        .get("build_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let test_success = inputs
        .get("test_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let lint_success = inputs
        .get("lint_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let overall_success = build_success && test_success && lint_success;

    let mut report = format!(
        "\nCI Report\n\
         =========\n\
         Build: {}\n\
         Test:  {}\n\
         Lint:  {}\n\
         ---------\n\
         Overall: {}\n",
        if build_success { "PASS" } else { "FAIL" },
        if test_success { "PASS" } else { "FAIL" },
        if lint_success { "PASS" } else { "FAIL" },
        if overall_success { "SUCCESS" } else { "FAILURE" }
    );

    // Append failure details for any failed stage
    if !build_success {
        let stderr = inputs
            .get("build_stderr")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !stderr.is_empty() {
            report.push_str(&format!("\n--- Build stderr ---\n{stderr}\n"));
        }
    }

    if !test_success {
        // For tests, extract the "failures:" section from stdout - that's where
        // the actual test names and panic messages are (stderr just says "test failed")
        let stdout = inputs
            .get("test_stdout")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if let Some(failures_section) = extract_test_failures(stdout) {
            report.push_str(&format!("\n--- Test failures ---\n{failures_section}\n"));
        }

        let stderr = inputs
            .get("test_stderr")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !stderr.is_empty() {
            report.push_str(&format!("\n--- Test stderr ---\n{stderr}\n"));
        }
    }

    if !lint_success {
        let stderr = inputs
            .get("lint_stderr")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !stderr.is_empty() {
            report.push_str(&format!("\n--- Lint stderr ---\n{stderr}\n"));
        }
    }

    let mut out = HashMap::new();
    out.insert("overall_success".to_string(), Value::Bool(overall_success));
    out.insert("report".to_string(), Value::Str(report));
    Ok(out)
}

/// Extract the "failures:" section from cargo test output.
/// This includes the failure list and any panic messages.
fn extract_test_failures(stdout: &str) -> Option<String> {
    // Look for the "failures:" line that lists failed test names
    let failures_start = stdout.find("\nfailures:\n")?;
    let section = &stdout[failures_start + 1..]; // skip the leading newline

    // Find the end - either "test result:" or end of string
    let end = section
        .find("\ntest result:")
        .unwrap_or(section.len());

    let failures_section = section[..end].trim();

    if failures_section.is_empty() {
        None
    } else {
        Some(failures_section.to_string())
    }
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_test::Mockable;

impl Mockable for CIOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            CIOp::ParseDepsExists => {
                let mut out = HashMap::new();
                out.insert("deps_exists".to_string(), Value::Bool(false));
                out.insert("deps_checked".to_string(), Value::Bool(true));
                out.insert("deps_installed".to_string(), Value::Int(0));
                out.insert("message".to_string(), Value::Str("No deps.toml found".to_string()));
                out
            }
            CIOp::PrepareCodegenExistsCheck => {
                let mut out = HashMap::new();
                out.insert("request".to_string(), Value::Request(
                    TransportRequest::File(FileRequest::exists("buck-out/gen/bin"))
                ));
                out
            }
            CIOp::ParseCodegenExists => {
                let mut out = HashMap::new();
                out.insert("codegen_needed".to_string(), Value::Bool(false));
                out.insert("prep_success".to_string(), Value::Bool(true));
                out.insert("codegen_ran".to_string(), Value::Bool(false));
                out.insert("prep_message".to_string(), Value::Str("Generated code exists".to_string()));
                out
            }
            CIOp::PrepareCodegenCommand => {
                let mut out = HashMap::new();
                out.insert("skip".to_string(), Value::Bool(true));
                out
            }
            CIOp::ParseCodegenResult => {
                let mut out = HashMap::new();
                out.insert("prep_success".to_string(), Value::Bool(true));
                out.insert("codegen_ran".to_string(), Value::Bool(false));
                out.insert("prep_message".to_string(), Value::Str("Generated code exists".to_string()));
                out
            }
            CIOp::PrepareBuildCommand => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.build.to_shell_request());
                let mut out = HashMap::new();
                out.insert("request".to_string(), Value::Request(request));
                out.insert("skip".to_string(), Value::Bool(false));
                out
            }
            CIOp::ParseBuildResult => {
                let mut out = HashMap::new();
                out.insert("build_success".to_string(), Value::Bool(true));
                out.insert("build_skipped".to_string(), Value::Bool(false));
                out.insert("build_stdout".to_string(), Value::Str("Build complete".to_string()));
                out.insert("build_stderr".to_string(), Value::Str(String::new()));
                out
            }
            CIOp::PrepareTestCommand => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.test.to_shell_request());
                let mut out = HashMap::new();
                out.insert("request".to_string(), Value::Request(request));
                out.insert("skip".to_string(), Value::Bool(false));
                out
            }
            CIOp::ParseTestResult => {
                let mut out = HashMap::new();
                out.insert("test_success".to_string(), Value::Bool(true));
                out.insert("test_skipped".to_string(), Value::Bool(false));
                out.insert("test_stdout".to_string(), Value::Str("All tests passed".to_string()));
                out.insert("test_stderr".to_string(), Value::Str(String::new()));
                out
            }
            CIOp::PrepareClippyLint => {
                let mut out = HashMap::new();
                out.insert("skip".to_string(), Value::Bool(false));
                out
            }
            CIOp::ParseClippyLintResult => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse};

    #[test]
    fn test_parse_deps_exists_true() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Response(
            TransportResponse::File(FileResponse {
                path: "deps.toml".to_string(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true),
                error: None,
            })
        ));

        let result = execute_parse_deps_exists(inputs).unwrap();
        assert_eq!(result.get("deps_exists").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_parse_deps_exists_false() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Response(
            TransportResponse::File(FileResponse {
                path: "deps.toml".to_string(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(false),
                error: None,
            })
        ));

        let result = execute_parse_deps_exists(inputs).unwrap();
        assert_eq!(result.get("deps_exists").and_then(|v| v.as_bool()), Some(false));
    }

    #[test]
    fn test_prepare_build_command_success() {
        let mut inputs = HashMap::new();
        inputs.insert("prep_success".to_string(), Value::Bool(true));

        let result = execute_prepare_build_command(inputs).unwrap();
        assert_eq!(result.get("skip").and_then(|v| v.as_bool()), Some(false));
        assert!(result.contains_key("request"));
    }

    #[test]
    fn test_prepare_build_command_skip() {
        let mut inputs = HashMap::new();
        inputs.insert("prep_success".to_string(), Value::Bool(false));

        let result = execute_prepare_build_command(inputs).unwrap();
        assert_eq!(result.get("skip").and_then(|v| v.as_bool()), Some(true));
        assert!(!result.contains_key("request"));
    }

    #[test]
    fn test_parse_build_result() {
        let mut inputs = HashMap::new();
        inputs.insert("skip".to_string(), Value::Bool(false));
        inputs.insert("response".to_string(), Value::Response(
            TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: "Build success".to_string(),
                stderr: String::new(),
            })
        ));

        let result = execute_parse_build_result(inputs).unwrap();
        assert_eq!(result.get("build_success").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(result.get("build_skipped").and_then(|v| v.as_bool()), Some(false));
    }

    #[test]
    fn test_report_all_pass() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(true));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("lint_success".to_string(), Value::Bool(true));

        let result = execute_report(inputs).unwrap();
        assert_eq!(result.get("overall_success").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_report_build_fail() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(false));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("lint_success".to_string(), Value::Bool(true));

        let result = execute_report(inputs).unwrap();
        assert_eq!(result.get("overall_success").and_then(|v| v.as_bool()), Some(false));
    }
}
