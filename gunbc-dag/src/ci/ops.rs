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

use crate::makegen::BuildConfig;
use gunbc_exec::{
    optional_bool, propagate_skipped, require_bool, require_response, require_str, ExecError,
    Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::render_ir::{PlainText, StructuredBlock, StructuredRenderer};
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::transport::{ShellRequest, TransportRequest};
use gunbc_ir::PlainStructuredRenderer;
use gunbc_ir::Value;
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

    // ========== Testgen stage ==========
    /// Prepare the testgen shell command (pure)
    PrepareTestgenCommand,
    /// Parse the testgen shell response (pure)
    ParseTestgenResult,

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

    // ========== Guardrails stage ==========
    /// Prepare disallowed-methods check (pure)
    PrepareGuardrailCheck,
    /// Parse disallowed-methods check response (pure)
    ParseGuardrailResult,

    // ========== Verify stage ==========
    /// Prepare the verify shell command (pure)
    /// Runs all four `--check` commands to verify generated artifacts are fresh.
    PrepareVerifyCheck,
    /// Parse the verify shell response (pure)
    ParseVerifyResult,

    // ========== Report stage (already pure) ==========
    /// Generate CI report (pure)
    Report,
}

impl Executable for CIOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CIOp::ParseDepsExists => execute_parse_deps_exists(inputs),
            CIOp::PrepareTestgenCommand => execute_prepare_testgen_command(inputs),
            CIOp::ParseTestgenResult => execute_parse_testgen_result(inputs),
            CIOp::PrepareBuildCommand => execute_prepare_build_command(inputs),
            CIOp::ParseBuildResult => execute_parse_build_result(inputs),
            CIOp::PrepareTestCommand => execute_prepare_test_command(inputs),
            CIOp::ParseTestResult => execute_parse_test_result(inputs),
            CIOp::PrepareClippyLint => execute_prepare_clippy_lint(inputs),
            CIOp::ParseClippyLintResult => execute_parse_clippy_lint_result(inputs),
            CIOp::PrepareGuardrailCheck => execute_prepare_guardrail_check(inputs),
            CIOp::ParseGuardrailResult => execute_parse_guardrail_result(inputs),
            CIOp::PrepareVerifyCheck => execute_prepare_verify_check(inputs),
            CIOp::ParseVerifyResult => execute_parse_verify_result(inputs),
            CIOp::Report => execute_report(inputs),
        }
    }
}

// ============================================================================
// SetupDeps Stage - Pure Operations
// ============================================================================

/// Parse the deps.toml exists check result (pure).
fn execute_parse_deps_exists(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["deps_exists", "deps_checked", "deps_installed", "message"],
    ) {
        return result;
    }

    let response = require_response(&inputs, "response")?;

    let file_resp = response.require_file()?;
    let deps_exists = file_resp
        .exists
        .ok_or_else(|| ExecError::new("deps exists check missing 'exists' field"))?;

    let message = if deps_exists {
        "deps.toml found, dependencies will be checked"
    } else {
        "No deps.toml found, skipping dependency check"
    };

    OutputMap::new()
        .bool("deps_exists", deps_exists)
        .bool("deps_checked", true)
        .int("deps_installed", 0)
        .str("message", message)
        .ok()
}

// ============================================================================
// Testgen Stage - Pure Operations
// ============================================================================

/// Prepare the testgen shell command (pure).
fn execute_prepare_testgen_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = optional_bool(&inputs, "prep_success").unwrap_or(false);

    if !prep_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to prep failure")
            .ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.testgen.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the testgen shell response (pure).
fn execute_parse_testgen_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "response", &["testgen_success", "testgen_stderr"]) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;

    if skip {
        let reason = require_str(&inputs, "skip_reason")?;
        return OutputMap::new()
            .bool("testgen_success", false)
            .str("testgen_stderr", reason)
            .ok();
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;

    OutputMap::new()
        .bool("testgen_success", shell.success())
        .str("testgen_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Build Stage - Pure Operations
// ============================================================================

/// Prepare the build shell command (pure).
fn execute_prepare_build_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Use optional_bool to handle Value::Skipped gracefully.
    // If prep_success is missing/Skipped, skip the build.
    let prep_success = optional_bool(&inputs, "prep_success").unwrap_or(false);
    let testgen_success = optional_bool(&inputs, "testgen_success").unwrap_or(false);

    if !prep_success || !testgen_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to prep/testgen failure")
            .ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.build.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the build shell response (pure).
fn execute_parse_build_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &[
            "build_success",
            "build_skipped",
            "build_stdout",
            "build_stderr",
        ],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;

    if skip {
        let reason = require_str(&inputs, "skip_reason")?;
        return OutputMap::new()
            .bool("build_success", false)
            .bool("build_skipped", true)
            .str("build_stdout", "")
            .str("build_stderr", reason)
            .ok();
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;

    OutputMap::new()
        .bool("build_success", shell.success())
        .bool("build_skipped", false)
        .str("build_stdout", shell.stdout.clone())
        .str("build_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Test Stage - Pure Operations
// ============================================================================

/// Prepare the test shell command (pure).
fn execute_prepare_test_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Use optional_bool to handle Value::Skipped gracefully.
    // If build_success is missing/Skipped, skip the test.
    let build_success = optional_bool(&inputs, "build_success").unwrap_or(false);

    if !build_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to build failure")
            .ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.test.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the test shell response (pure).
fn execute_parse_test_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["test_success", "test_skipped", "test_stdout", "test_stderr"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;

    if skip {
        let reason = require_str(&inputs, "skip_reason")?;
        return OutputMap::new()
            .bool("test_success", false)
            .bool("test_skipped", true)
            .str("test_stdout", "")
            .str("test_stderr", reason)
            .ok();
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;

    OutputMap::new()
        .bool("test_success", shell.success())
        .bool("test_skipped", false)
        .str("test_stdout", shell.stdout.clone())
        .str("test_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Lint Stage - Pure Operations (using Clippy SubDag)
// ============================================================================

/// Prepare clippy lint - check if we should skip based on build_success (pure).
///
/// This is the pre-gate for the Clippy tool execution. It checks if the build succeeded
/// and either allows the lint or signals to skip.
fn execute_prepare_clippy_lint(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Use optional_bool to handle Value::Skipped gracefully.
    // If build_success is missing/Skipped, skip clippy.
    let build_success = optional_bool(&inputs, "build_success").unwrap_or(false);

    if !build_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to build failure")
            .ok();
    }

    OutputMap::new().bool("skip", false).ok()
}

/// Parse clippy lint result - convert CliToolOp outputs to CI format (pure).
///
/// This is the post-parse for the Clippy SubDag. It converts the clippy run
/// outputs (success, stdout, stderr) to the expected CI format.
fn execute_parse_clippy_lint_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let skip = require_bool(&inputs, "skip")?;

    if skip {
        let reason = require_str(&inputs, "skip_reason")?;
        return OutputMap::new()
            .bool("lint_success", false)
            .bool("lint_skipped", true)
            .str("lint_stdout", "")
            .str("lint_stderr", reason)
            .ok();
    }

    // Get outputs from the clippy SubDag (from the 'resolve' node which runs clippy)
    let success = require_bool(&inputs, "success")?;
    let stdout = require_str(&inputs, "stdout")?.to_string();
    let stderr = require_str(&inputs, "stderr")?.to_string();

    OutputMap::new()
        .bool("lint_success", success)
        .bool("lint_skipped", false)
        .str("lint_stdout", stdout)
        .str("lint_stderr", stderr)
        .ok()
}

// ============================================================================
// Guardrails Stage - Pure Operations
// ============================================================================

/// Prepare the disallowed-methods check (pure).
fn execute_prepare_guardrail_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let testgen_success = optional_bool(&inputs, "testgen_success").unwrap_or(false);

    if !testgen_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to testgen failure")
            .ok();
    }

    let request = TransportRequest::Shell(
        ShellRequest::new("bash").args(["-lc", "tools/check-disallowed-methods.sh"]),
    );

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the disallowed-methods check response (pure).
fn execute_parse_guardrail_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["guardrail_success", "guardrail_stderr"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;

    if skip {
        let reason = require_str(&inputs, "skip_reason")?;
        return OutputMap::new()
            .bool("guardrail_success", false)
            .str("guardrail_stderr", reason)
            .ok();
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;

    OutputMap::new()
        .bool("guardrail_success", shell.success())
        .str("guardrail_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Verify Stage - Check generated artifacts are fresh
// ============================================================================

/// Prepare the verify shell command (pure).
///
/// Runs all four `--check` commands sequentially. If any fails, the combined
/// command fails and stderr reports which generator drifted.
fn execute_prepare_verify_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = optional_bool(&inputs, "prep_success").unwrap_or(false);

    if !prep_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to codegen failure")
            .ok();
    }

    let config = BuildConfig::cargo();
    // Build a combined shell command that runs all four --check verifiers.
    // Each uses the same binary commands as `make verify`.
    let makegen_check = config.makegen_check.to_shell();
    let bootstrap_check = config.bootstrap_check.to_shell();
    let testgen_check = config.testgen_check.to_shell();
    let pragma_check = config.pragma_check.to_shell();

    let combined = format!(
        "{} && {} && {} && {}",
        makegen_check, bootstrap_check, testgen_check, pragma_check
    );

    let request = TransportRequest::Shell(
        ShellRequest::new("bash").args(["-lc", &combined]),
    );

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the verify shell response (pure).
fn execute_parse_verify_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["verify_success", "verify_stderr"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;

    if skip {
        let reason = require_str(&inputs, "skip_reason")?;
        return OutputMap::new()
            .bool("verify_success", false)
            .str("verify_stderr", reason)
            .ok();
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;

    OutputMap::new()
        .bool("verify_success", shell.success())
        .str("verify_stderr", shell.stderr.clone())
        .ok()
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
    // Use optional_bool to handle Value::Skipped gracefully during skip propagation.
    // Skipped stages are treated as passed (true) since they didn't actually fail.
    let build_success = optional_bool(&inputs, "build_success").unwrap_or(true);
    let test_success = optional_bool(&inputs, "test_success").unwrap_or(true);
    let lint_success = optional_bool(&inputs, "lint_success").unwrap_or(true);
    let testgen_success = optional_bool(&inputs, "testgen_success").unwrap_or(true);
    let guardrail_success = optional_bool(&inputs, "guardrail_success").unwrap_or(true);
    let verify_success = optional_bool(&inputs, "verify_success").unwrap_or(true);

    let overall_success =
        build_success && test_success && lint_success && testgen_success && guardrail_success && verify_success;

    let blocks = build_report_blocks(
        build_success,
        test_success,
        lint_success,
        verify_success,
        testgen_success,
        guardrail_success,
        overall_success,
        &inputs,
    )?;

    let renderer = PlainStructuredRenderer::new(PlainText {
        tier: Tier::Ascii,
        symbol_set: &STANDARD,
    });

    let mut report = String::new();
    for block in &blocks {
        report.push_str(&renderer.render_block(block));
    }

    OutputMap::new()
        .bool("overall_success", overall_success)
        .str("report", report)
        .ok()
}

/// Build CI report as structured blocks.
#[allow(clippy::too_many_arguments)]
fn build_report_blocks(
    build_success: bool,
    test_success: bool,
    lint_success: bool,
    verify_success: bool,
    testgen_success: bool,
    guardrail_success: bool,
    overall_success: bool,
    inputs: &HashMap<String, Value>,
) -> Result<Vec<StructuredBlock>, ExecError> {
    let mut blocks = Vec::new();

    // Summary section
    blocks.push(StructuredBlock::Raw(format!(
        "\nCI Report\n\
         =========\n\
         Build: {}\n\
         Test:  {}\n\
         Lint:  {}\n\
         Verify: {}\n\
         Guardrails: {}\n\
         ---------\n\
         Overall: {}\n",
        if build_success { "PASS" } else { "FAIL" },
        if test_success { "PASS" } else { "FAIL" },
        if lint_success { "PASS" } else { "FAIL" },
        if verify_success { "PASS" } else { "FAIL" },
        if testgen_success && guardrail_success {
            "PASS"
        } else {
            "FAIL"
        },
        if overall_success {
            "SUCCESS"
        } else {
            "FAILURE"
        }
    )));

    // Failure details
    if !build_success {
        let stderr = require_str(inputs, "build_stderr")?;
        if !stderr.is_empty() {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Build stderr ---\n{stderr}\n"
            )));
        }
    }

    if !test_success {
        let stdout = require_str(inputs, "test_stdout")?;

        if let Some(failures_section) = extract_test_failures(stdout) {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Test failures ---\n{failures_section}\n"
            )));
        } else if !stdout.is_empty() {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Test stdout ---\n{stdout}\n"
            )));
        }

        let stderr = require_str(inputs, "test_stderr")?;
        if !stderr.is_empty() {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Test stderr ---\n{stderr}\n"
            )));
        }
    }

    if !lint_success {
        let stderr = require_str(inputs, "lint_stderr")?;
        if !stderr.is_empty() {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Lint stderr ---\n{stderr}\n"
            )));
        }
    }

    if !testgen_success {
        let stderr = require_str(inputs, "testgen_stderr")?;
        if !stderr.is_empty() {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Testgen stderr ---\n{stderr}\n"
            )));
        }
    }

    if !guardrail_success {
        let stderr = require_str(inputs, "guardrail_stderr")?;
        if !stderr.is_empty() {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Guardrails stderr ---\n{stderr}\n"
            )));
        }
    }

    if !verify_success {
        let stderr = require_str(inputs, "verify_stderr")?;
        if !stderr.is_empty() {
            blocks.push(StructuredBlock::Raw(format!(
                "\n--- Verify stderr ---\n{stderr}\n"
            )));
        }
    }

    Ok(blocks)
}

/// Extract the "failures:" section from cargo test output.
/// This includes the failure list and any panic messages.
fn extract_test_failures(stdout: &str) -> Option<String> {
    // Look for "failures:" line - can appear with or without leading newline
    let failures_marker = "failures:\n";
    let failures_pos = stdout.find(failures_marker)?;

    // Get the section after "failures:\n"
    let section_start = failures_pos + failures_marker.len();
    let section = &stdout[section_start..];

    // Find the end - either "test result:" or end of string
    let end = section
        .find("\ntest result:")
        .or_else(|| section.find("test result:"))
        .unwrap_or(section.len());

    let failures_section = section[..end].trim();

    if failures_section.is_empty() {
        None
    } else {
        // Include the "failures:" header for context
        Some(format!("failures:\n{failures_section}"))
    }
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_test::Mockable;

impl Mockable for CIOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            CIOp::ParseDepsExists => OutputMap::new()
                .bool("deps_exists", false)
                .bool("deps_checked", true)
                .int("deps_installed", 0)
                .str("message", "No deps.toml found")
                .build(),
            CIOp::PrepareTestgenCommand => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.testgen.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseTestgenResult => OutputMap::new()
                .bool("testgen_success", true)
                .str("testgen_stderr", "")
                .build(),
            CIOp::PrepareBuildCommand => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.build.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseBuildResult => OutputMap::new()
                .bool("build_success", true)
                .bool("build_skipped", false)
                .str("build_stdout", "Build complete")
                .str("build_stderr", "")
                .build(),
            CIOp::PrepareTestCommand => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.test.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseTestResult => OutputMap::new()
                .bool("test_success", true)
                .bool("test_skipped", false)
                .str("test_stdout", "All tests passed")
                .str("test_stderr", "")
                .build(),
            CIOp::PrepareGuardrailCheck => {
                let request = TransportRequest::Shell(
                    ShellRequest::new("bash").args(["-lc", "tools/check-disallowed-methods.sh"]),
                );
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseGuardrailResult => OutputMap::new()
                .bool("guardrail_success", true)
                .str("guardrail_stderr", "")
                .build(),
            CIOp::PrepareClippyLint => OutputMap::new().bool("skip", false).build(),
            CIOp::ParseClippyLintResult => OutputMap::new()
                .bool("lint_success", true)
                .bool("lint_skipped", false)
                .str("lint_stdout", "No warnings")
                .str("lint_stderr", "")
                .build(),
            CIOp::PrepareVerifyCheck => {
                let config = BuildConfig::cargo();
                let cmd = format!(
                    "{} && {} && {} && {}",
                    config.makegen_check.to_shell(),
                    config.bootstrap_check.to_shell(),
                    config.testgen_check.to_shell(),
                    config.pragma_check.to_shell(),
                );
                let request = TransportRequest::Shell(
                    ShellRequest::new("bash").args(["-lc", &cmd]),
                );
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseVerifyResult => OutputMap::new()
                .bool("verify_success", true)
                .str("verify_stderr", "")
                .build(),
            CIOp::Report => OutputMap::new()
                .bool("overall_success", true)
                .str("report", "CI passed")
                .build(),
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
        inputs.insert(
            "response".to_string(),
            Value::Response(
                FileResponse {
                    path: "deps.toml".to_string(),
                    operation: FileOp::Exists,
                    success: true,
                    content: None,
                    exists: Some(true),
                    error: None,
                }
                .into(),
            ),
        );

        let result = execute_parse_deps_exists(inputs).unwrap();
        assert_eq!(
            result.get("deps_exists").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_parse_deps_exists_false() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(
                FileResponse {
                    path: "deps.toml".to_string(),
                    operation: FileOp::Exists,
                    success: true,
                    content: None,
                    exists: Some(false),
                    error: None,
                }
                .into(),
            ),
        );

        let result = execute_parse_deps_exists(inputs).unwrap();
        assert_eq!(
            result.get("deps_exists").and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn test_prepare_build_command_success() {
        let mut inputs = HashMap::new();
        inputs.insert("prep_success".to_string(), Value::Bool(true));
        inputs.insert("testgen_success".to_string(), Value::Bool(true));

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
        inputs.insert(
            "response".to_string(),
            Value::Response(ShellResponse::ok("Build success").into()),
        );

        let result = execute_parse_build_result(inputs).unwrap();
        assert_eq!(
            result.get("build_success").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            result.get("build_skipped").and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn test_report_all_pass() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(true));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("lint_success".to_string(), Value::Bool(true));

        let result = execute_report(inputs).unwrap();
        assert_eq!(
            result.get("overall_success").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_report_build_fail() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(false));
        inputs.insert(
            "build_stderr".to_string(),
            Value::Str("error: compilation failed".into()),
        );
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("test_stdout".to_string(), Value::Str(String::new()));
        inputs.insert("test_stderr".to_string(), Value::Str(String::new()));
        inputs.insert("lint_success".to_string(), Value::Bool(true));
        inputs.insert("lint_stdout".to_string(), Value::Str(String::new()));
        inputs.insert("lint_stderr".to_string(), Value::Str(String::new()));

        let result = execute_report(inputs).unwrap();
        assert_eq!(
            result.get("overall_success").and_then(|v| v.as_bool()),
            Some(false)
        );
    }
}
