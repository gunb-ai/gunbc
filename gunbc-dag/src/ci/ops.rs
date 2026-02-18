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

use crate::makegen::registry::BuildCommand;
use crate::makegen::BuildConfig;
use gunbc_exec::{
    optional_response_strict, optional_str_strict, propagate_skipped, require_bool,
    require_response, ExecError, Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::render_ir::{PlainText, StructuredBlock, StructuredRenderer};
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::transport::{ShellRequest, TransportRequest};
use gunbc_ir::PlainStructuredRenderer;
use gunbc_ir::Value;
use gunbc_ir::{CargoCommand, Subcommand, Warnings};
use gunbc_testgen_registry::iter_dag_specs;
use std::collections::HashMap;

// ============================================================================
// CIOp - Pure CI-specific operations
// ============================================================================

fn require_bool_or_skipped(
    inputs: &HashMap<String, Value>,
    key: &str,
    skipped_value: bool,
) -> Result<bool, ExecError> {
    match inputs.get(key) {
        Some(Value::Bool(b)) => Ok(*b),
        Some(Value::Skipped) => Ok(skipped_value),
        Some(_) => Err(ExecError::new(format!(
            "missing or invalid '{}' input",
            key
        ))),
        None => Err(ExecError::new(format!("missing '{}' input", key))),
    }
}

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

    // ========== Bootstrap stage ==========
    /// Prepare the bootstrap shell command (pure)
    PrepareBootstrapCommand,
    /// Parse the bootstrap shell response (pure)
    ParseBootstrapResult,

    // ========== Pragma stage ==========
    /// Prepare the pragma shell command (pure)
    PreparePragmaCommand,
    /// Parse the pragma shell response (pure)
    ParsePragmaResult,

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
    /// Parse clippy lint result - convert TransportResponse to CI format (pure)
    /// Inputs: success: Bool, stdout: String, stderr: String, skip: Bool
    /// Outputs: lint_success, lint_skipped, lint_stdout, lint_stderr
    ParseClippyLintResult,

    // ========== Guardrails stage ==========
    /// Prepare disallowed-methods check (pure)
    PrepareGuardrailCheck,
    /// Parse disallowed-methods check response (pure)
    ParseGuardrailResult,

    // ========== Verify stage ==========
    /// Prepare the makegen verify shell command (pure)
    PrepareVerifyMakegenCheck,
    /// Parse the makegen verify shell response (pure)
    ParseVerifyMakegenResult,
    /// Prepare the deps-config verify shell command (pure)
    PrepareVerifyDepsConfigCheck,
    /// Parse the deps-config verify shell response (pure)
    ParseVerifyDepsConfigResult,
    /// Prepare the bootstrap verify shell command (pure)
    PrepareVerifyBootstrapCheck,
    /// Parse the bootstrap verify shell response (pure)
    ParseVerifyBootstrapResult,
    /// Prepare the testgen verify shell command (pure)
    PrepareVerifyTestgenCheck,
    /// Parse the testgen verify shell response (pure)
    ParseVerifyTestgenResult,
    /// Prepare the pragma verify shell command (pure)
    PrepareVerifyPragmaCheck,
    /// Parse the pragma verify shell response (pure)
    ParseVerifyPragmaResult,
    /// Aggregate per-check verify results into report-friendly outputs (pure)
    AggregateVerifyResults,

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
            CIOp::PrepareBootstrapCommand => execute_prepare_bootstrap_command(inputs),
            CIOp::ParseBootstrapResult => execute_parse_bootstrap_result(inputs),
            CIOp::PreparePragmaCommand => execute_prepare_pragma_command(inputs),
            CIOp::ParsePragmaResult => execute_parse_pragma_result(inputs),
            CIOp::PrepareBuildCommand => execute_prepare_build_command(inputs),
            CIOp::ParseBuildResult => execute_parse_build_result(inputs),
            CIOp::PrepareTestCommand => execute_prepare_test_command(inputs),
            CIOp::ParseTestResult => execute_parse_test_result(inputs),
            CIOp::PrepareClippyLint => execute_prepare_clippy_lint(inputs),
            CIOp::ParseClippyLintResult => execute_parse_clippy_lint_result(inputs),
            CIOp::PrepareGuardrailCheck => execute_prepare_guardrail_check(inputs),
            CIOp::ParseGuardrailResult => execute_parse_guardrail_result(inputs),
            CIOp::PrepareVerifyMakegenCheck => execute_prepare_verify_makegen_check(inputs),
            CIOp::ParseVerifyMakegenResult => execute_parse_verify_makegen_result(inputs),
            CIOp::PrepareVerifyDepsConfigCheck => execute_prepare_verify_deps_config_check(inputs),
            CIOp::ParseVerifyDepsConfigResult => execute_parse_verify_deps_config_result(inputs),
            CIOp::PrepareVerifyBootstrapCheck => execute_prepare_verify_bootstrap_check(inputs),
            CIOp::ParseVerifyBootstrapResult => execute_parse_verify_bootstrap_result(inputs),
            CIOp::PrepareVerifyTestgenCheck => execute_prepare_verify_testgen_check(inputs),
            CIOp::ParseVerifyTestgenResult => execute_parse_verify_testgen_result(inputs),
            CIOp::PrepareVerifyPragmaCheck => execute_prepare_verify_pragma_check(inputs),
            CIOp::ParseVerifyPragmaResult => execute_parse_verify_pragma_result(inputs),
            CIOp::AggregateVerifyResults => execute_aggregate_verify_results(inputs),
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
        &[
            "deps_exists",
            "deps_checked",
            "deps_installed",
            "message",
            "success",
            "error_summary",
            "detail",
        ],
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
        .status(true, "", message)
        .ok()
}

// ============================================================================
// Testgen Stage - Pure Operations
// ============================================================================

/// Prepare the testgen shell command (pure).
fn execute_prepare_testgen_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = require_bool_or_skipped(&inputs, "prep_success", false)?;

    if !prep_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to prep failure")
            .ok();
    }

    if iter_dag_specs().next().is_none() {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "No DagSpec registrations found")
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
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["testgen_success", "testgen_stderr", "testgen_stdout"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;
    let response = optional_response_strict(&inputs, "response")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool("testgen_success", false)
            .str("testgen_stderr", reason)
            .str("testgen_stdout", "")
            .ok();
    }

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool("testgen_success", false)
                .str("testgen_stderr", "missing response")
                .str("testgen_stdout", "")
                .ok();
        }
    };

    OutputMap::new()
        .bool("testgen_success", shell.success())
        .str("testgen_stderr", shell.stderr.clone())
        .str("testgen_stdout", shell.stdout.clone())
        .ok()
}

// ============================================================================
// Bootstrap Stage - Pure Operations
// ============================================================================

/// Prepare the bootstrap shell command (pure).
fn execute_prepare_bootstrap_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = require_bool_or_skipped(&inputs, "prep_success", false)?;

    if !prep_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to prep failure")
            .ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.bootstrap.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the bootstrap shell response (pure).
fn execute_parse_bootstrap_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["bootstrap_success", "bootstrap_stderr", "bootstrap_stdout"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;
    let response = optional_response_strict(&inputs, "response")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool("bootstrap_success", false)
            .str("bootstrap_stderr", reason)
            .str("bootstrap_stdout", "")
            .ok();
    }

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool("bootstrap_success", false)
                .str("bootstrap_stderr", "missing response")
                .str("bootstrap_stdout", "")
                .ok();
        }
    };

    OutputMap::new()
        .bool("bootstrap_success", shell.success())
        .str("bootstrap_stderr", shell.stderr.clone())
        .str("bootstrap_stdout", shell.stdout.clone())
        .ok()
}

// ============================================================================
// Pragma Stage - Pure Operations
// ============================================================================

/// Prepare the pragma shell command (pure).
fn execute_prepare_pragma_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = require_bool_or_skipped(&inputs, "prep_success", false)?;

    if !prep_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to prep failure")
            .ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.pragma.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the pragma shell response (pure).
fn execute_parse_pragma_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["pragma_success", "pragma_stderr", "pragma_stdout"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;
    let response = optional_response_strict(&inputs, "response")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool("pragma_success", false)
            .str("pragma_stderr", reason)
            .str("pragma_stdout", "")
            .ok();
    }

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool("pragma_success", false)
                .str("pragma_stderr", "missing response")
                .str("pragma_stdout", "")
                .ok();
        }
    };

    OutputMap::new()
        .bool("pragma_success", shell.success())
        .str("pragma_stderr", shell.stderr.clone())
        .str("pragma_stdout", shell.stdout.clone())
        .ok()
}

// ============================================================================
// Build Stage - Pure Operations
// ============================================================================

/// Prepare the build shell command (pure).
fn execute_prepare_build_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = require_bool_or_skipped(&inputs, "prep_success", false)?;
    let testgen_success = require_bool_or_skipped(&inputs, "testgen_success", false)?;

    if !prep_success || !testgen_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to prep/testgen failure")
            .ok();
    }

    // Compile test artifacts once up front so the later `cargo test` stage
    // can run without a separate dev-profile build pass.
    let compile_only_test_build = CargoCommand::new(Subcommand::Test)
        .no_run()
        .warnings(Warnings::Deny);
    let request = TransportRequest::Shell(compile_only_test_build.to_shell_request());

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
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;
    let response = optional_response_strict(&inputs, "response")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool("build_success", false)
            .bool("build_skipped", true)
            .str("build_stdout", "")
            .str("build_stderr", reason)
            .ok();
    }

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool("build_success", false)
                .bool("build_skipped", false)
                .str("build_stdout", "")
                .str("build_stderr", "missing response")
                .ok();
        }
    };

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
    let build_success = require_bool_or_skipped(&inputs, "build_success", false)?;

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
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;
    let response = optional_response_strict(&inputs, "response")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool("test_success", false)
            .bool("test_skipped", true)
            .str("test_stdout", "")
            .str("test_stderr", reason)
            .ok();
    }

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool("test_success", false)
                .bool("test_skipped", false)
                .str("test_stdout", "")
                .str("test_stderr", "missing response")
                .ok();
        }
    };

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

/// Prepare clippy lint - check if we should skip, and build the shell command (pure).
///
/// Outputs: request (TransportRequest), skip (Bool), skip_reason (OptionalString)
fn execute_prepare_clippy_lint(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = require_bool_or_skipped(&inputs, "build_success", false)?;
    let pragma_success = require_bool_or_skipped(&inputs, "pragma_success", false)?;

    if !build_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to build failure")
            .ok();
    }

    if !pragma_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to pragma failure")
            .ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.lint.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse clippy lint result - convert TransportResponse to CI format (pure).
///
/// Inputs: response (TransportResponse), skip (Bool), skip_reason (OptionalString)
/// Outputs: lint_success, lint_skipped, lint_stdout, lint_stderr
fn execute_parse_clippy_lint_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["lint_success", "lint_skipped", "lint_stdout", "lint_stderr"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool("lint_success", false)
            .bool("lint_skipped", true)
            .str("lint_stdout", "")
            .str("lint_stderr", reason)
            .ok();
    }

    let response = optional_response_strict(&inputs, "response")?;

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool("lint_success", false)
                .bool("lint_skipped", false)
                .str("lint_stdout", "")
                .str("lint_stderr", "missing response")
                .ok();
        }
    };

    OutputMap::new()
        .bool("lint_success", shell.success())
        .bool("lint_skipped", false)
        .str("lint_stdout", shell.stdout.clone())
        .str("lint_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Guardrails Stage - Pure Operations
// ============================================================================

const GUARDRAIL_CHECK_COMMAND: &str =
    "cargo test -p gunbc-dag --test resource_purity_checks --quiet";

/// Prepare the disallowed-methods check (pure).
fn execute_prepare_guardrail_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let testgen_success = require_bool_or_skipped(&inputs, "testgen_success", false)?;
    let pragma_success = require_bool_or_skipped(&inputs, "pragma_success", false)?;

    if !testgen_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to testgen failure")
            .ok();
    }

    if !pragma_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to pragma failure")
            .ok();
    }

    let request =
        TransportRequest::Shell(ShellRequest::new("bash").args(["-lc", GUARDRAIL_CHECK_COMMAND]));

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
        &["guardrail_success", "guardrail_stderr", "guardrail_stdout"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;
    let response = optional_response_strict(&inputs, "response")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool("guardrail_success", false)
            .str("guardrail_stderr", reason)
            .str("guardrail_stdout", "")
            .ok();
    }

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool("guardrail_success", false)
                .str("guardrail_stderr", "missing response")
                .str("guardrail_stdout", "")
                .ok();
        }
    };

    OutputMap::new()
        .bool("guardrail_success", shell.success())
        .str("guardrail_stderr", shell.stderr.clone())
        .str("guardrail_stdout", shell.stdout.clone())
        .ok()
}

// ============================================================================
// Verify Stage - Check generated artifacts are fresh
// ============================================================================

fn verify_skip_reason(inputs: &HashMap<String, Value>) -> Result<Option<&'static str>, ExecError> {
    let prep_success = require_bool_or_skipped(inputs, "prep_success", false)?;
    let bootstrap_success = require_bool_or_skipped(inputs, "bootstrap_success", false)?;
    let testgen_success = require_bool_or_skipped(inputs, "testgen_success", false)?;
    let pragma_success = require_bool_or_skipped(inputs, "pragma_success", false)?;

    if !prep_success {
        return Ok(Some("Skipped due to codegen failure"));
    }

    if !bootstrap_success {
        return Ok(Some("Skipped due to bootstrap failure"));
    }

    if !testgen_success {
        return Ok(Some("Skipped due to testgen failure"));
    }

    if !pragma_success {
        return Ok(Some("Skipped due to pragma failure"));
    }

    Ok(None)
}

fn execute_prepare_verify_check(
    inputs: HashMap<String, Value>,
    command: &BuildCommand,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(reason) = verify_skip_reason(&inputs)? {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", reason)
            .ok();
    }

    let request = TransportRequest::Shell(command.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Prepare the makegen verify shell command (pure).
fn execute_prepare_verify_makegen_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let config = BuildConfig::cargo();
    execute_prepare_verify_check(inputs, &config.makegen_check)
}

/// Prepare the deps-config verify shell command (pure).
fn execute_prepare_verify_deps_config_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let config = BuildConfig::cargo();
    execute_prepare_verify_check(inputs, &config.deps_config_check)
}

/// Prepare the bootstrap verify shell command (pure).
fn execute_prepare_verify_bootstrap_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let config = BuildConfig::cargo();
    execute_prepare_verify_check(inputs, &config.bootstrap_check)
}

/// Prepare the testgen verify shell command (pure).
fn execute_prepare_verify_testgen_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let config = BuildConfig::cargo();
    execute_prepare_verify_check(inputs, &config.testgen_check)
}

/// Prepare the pragma verify shell command (pure).
fn execute_prepare_verify_pragma_check(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let config = BuildConfig::cargo();
    execute_prepare_verify_check(inputs, &config.pragma_check)
}

/// Parse a verify sub-check shell response (pure).
fn execute_parse_verify_result(
    inputs: HashMap<String, Value>,
    success_key: &str,
    stderr_key: &str,
    stdout_key: &str,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) =
        propagate_skipped(&inputs, "response", &[success_key, stderr_key, stdout_key])
    {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;
    let skip_reason = optional_str_strict(&inputs, "skip_reason")?;
    let response = optional_response_strict(&inputs, "response")?;

    if skip {
        let reason = skip_reason.unwrap_or("Skipped");
        return OutputMap::new()
            .bool(success_key, false)
            .str(stderr_key, reason)
            .str(stdout_key, "")
            .ok();
    }

    let shell = match response {
        Some(response) => response.require_shell()?,
        None => {
            return OutputMap::new()
                .bool(success_key, false)
                .str(stderr_key, "missing response")
                .str(stdout_key, "")
                .ok();
        }
    };

    OutputMap::new()
        .bool(success_key, shell.success())
        .str(stderr_key, shell.stderr.clone())
        .str(stdout_key, shell.stdout.clone())
        .ok()
}

/// Parse the makegen verify shell response (pure).
fn execute_parse_verify_makegen_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    execute_parse_verify_result(
        inputs,
        "verify_makegen_success",
        "verify_makegen_stderr",
        "verify_makegen_stdout",
    )
}

/// Parse the deps-config verify shell response (pure).
fn execute_parse_verify_deps_config_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    execute_parse_verify_result(
        inputs,
        "verify_deps_config_success",
        "verify_deps_config_stderr",
        "verify_deps_config_stdout",
    )
}

/// Parse the bootstrap verify shell response (pure).
fn execute_parse_verify_bootstrap_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    execute_parse_verify_result(
        inputs,
        "verify_bootstrap_success",
        "verify_bootstrap_stderr",
        "verify_bootstrap_stdout",
    )
}

/// Parse the testgen verify shell response (pure).
fn execute_parse_verify_testgen_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    execute_parse_verify_result(
        inputs,
        "verify_testgen_success",
        "verify_testgen_stderr",
        "verify_testgen_stdout",
    )
}

/// Parse the pragma verify shell response (pure).
fn execute_parse_verify_pragma_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    execute_parse_verify_result(
        inputs,
        "verify_pragma_success",
        "verify_pragma_stderr",
        "verify_pragma_stdout",
    )
}

/// Aggregate per-check verify results into report-friendly outputs.
fn execute_aggregate_verify_results(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let checks = [
        (
            "makegen",
            "verify_makegen_success",
            "verify_makegen_stderr",
            "verify_makegen_stdout",
        ),
        (
            "deps-config",
            "verify_deps_config_success",
            "verify_deps_config_stderr",
            "verify_deps_config_stdout",
        ),
        (
            "bootstrap",
            "verify_bootstrap_success",
            "verify_bootstrap_stderr",
            "verify_bootstrap_stdout",
        ),
        (
            "testgen",
            "verify_testgen_success",
            "verify_testgen_stderr",
            "verify_testgen_stdout",
        ),
        (
            "pragma",
            "verify_pragma_success",
            "verify_pragma_stderr",
            "verify_pragma_stdout",
        ),
    ];

    let mut verify_success = true;
    let mut failure_messages: Vec<String> = Vec::new();
    let mut verify_stdout_parts: Vec<String> = Vec::new();

    for (name, success_key, stderr_key, stdout_key) in checks {
        let success = require_bool_or_skipped(&inputs, success_key, true)?;
        if let Some(stdout) = optional_str_strict(&inputs, stdout_key)? {
            if !stdout.trim().is_empty() {
                verify_stdout_parts.push(format!("{name}:\n{stdout}"));
            }
        }
        if success {
            continue;
        }

        verify_success = false;
        let stderr = optional_str_strict(&inputs, stderr_key)?;
        let message = match stderr {
            Some(stderr) if !stderr.trim().is_empty() => format!("{name}: {stderr}"),
            _ => format!("{name}: verify check failed (see stdout output)"),
        };
        failure_messages.push(message);
    }

    let verify_stderr = if verify_success {
        String::new()
    } else {
        failure_messages.join("\n\n")
    };

    let verify_stdout = verify_stdout_parts.join("\n\n");
    let verify_detail = if verify_success {
        "All verify checks passed".to_string()
    } else {
        verify_stderr.clone()
    };

    OutputMap::new()
        .bool("verify_success", verify_success)
        .str("verify_stdout", verify_stdout)
        .str("verify_stderr", verify_stderr)
        .status(
            verify_success,
            if verify_success {
                ""
            } else {
                "One or more verify checks failed"
            },
            verify_detail,
        )
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
    // Skipped stages are treated as passed (true) since they didn't actually fail.
    let build_success = require_bool_or_skipped(&inputs, "build_success", true)?;
    let test_success = require_bool_or_skipped(&inputs, "test_success", true)?;
    let lint_success = require_bool_or_skipped(&inputs, "lint_success", true)?;
    let testgen_success = require_bool_or_skipped(&inputs, "testgen_success", true)?;
    let bootstrap_success = require_bool_or_skipped(&inputs, "bootstrap_success", true)?;
    let pragma_success = require_bool_or_skipped(&inputs, "pragma_success", true)?;
    let guardrail_success = require_bool_or_skipped(&inputs, "guardrail_success", true)?;
    let verify_success = require_bool_or_skipped(&inputs, "verify_success", true)?;

    let overall_success = build_success
        && test_success
        && lint_success
        && testgen_success
        && bootstrap_success
        && pragma_success
        && guardrail_success
        && verify_success;

    let blocks = build_report_blocks(
        build_success,
        test_success,
        lint_success,
        verify_success,
        testgen_success,
        bootstrap_success,
        pragma_success,
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
    let status_summary = if overall_success {
        ""
    } else {
        "One or more CI stages failed"
    };

    OutputMap::new()
        .bool("overall_success", overall_success)
        .str("report", report.clone())
        .status(overall_success, status_summary, report)
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
    bootstrap_success: bool,
    pragma_success: bool,
    guardrail_success: bool,
    overall_success: bool,
    inputs: &HashMap<String, Value>,
) -> Result<Vec<StructuredBlock>, ExecError> {
    let mut blocks = Vec::new();
    let build_stderr = optional_str_strict(inputs, "build_stderr")?.unwrap_or("");
    let test_stdout = optional_str_strict(inputs, "test_stdout")?.unwrap_or("");
    let test_stderr = optional_str_strict(inputs, "test_stderr")?.unwrap_or("");
    let lint_stderr = optional_str_strict(inputs, "lint_stderr")?.unwrap_or("");
    let testgen_stderr = optional_str_strict(inputs, "testgen_stderr")?.unwrap_or("");
    let bootstrap_stderr = optional_str_strict(inputs, "bootstrap_stderr")?.unwrap_or("");
    let pragma_stderr = optional_str_strict(inputs, "pragma_stderr")?.unwrap_or("");
    let guardrail_stderr = optional_str_strict(inputs, "guardrail_stderr")?.unwrap_or("");
    let verify_stderr = optional_str_strict(inputs, "verify_stderr")?.unwrap_or("");
    let cloud_env_status = optional_str_strict(inputs, "cloud_env_status")?.unwrap_or("");

    // Summary section
    blocks.push(StructuredBlock::Raw(format!(
        "\nCI Report\n\
         =========\n\
         Build: {}\n\
         Test:  {}\n\
         Lint:  {}\n\
         Testgen: {}\n\
         Bootstrap: {}\n\
         Pragma: {}\n\
         Verify: {}\n\
         Guardrails: {}\n\
         ---------\n\
         Overall: {}\n",
        if build_success { "PASS" } else { "FAIL" },
        if test_success { "PASS" } else { "FAIL" },
        if lint_success { "PASS" } else { "FAIL" },
        if testgen_success { "PASS" } else { "FAIL" },
        if bootstrap_success { "PASS" } else { "FAIL" },
        if pragma_success { "PASS" } else { "FAIL" },
        if verify_success { "PASS" } else { "FAIL" },
        if guardrail_success { "PASS" } else { "FAIL" },
        if overall_success {
            "SUCCESS"
        } else {
            "FAILURE"
        }
    )));

    // Failure details — each stage gets a section with specialized extraction
    // where available, falling back to tail summary.
    let build_stdout = optional_str_strict(inputs, "build_stdout")?.unwrap_or("");
    let lint_stdout = optional_str_strict(inputs, "lint_stdout")?.unwrap_or("");
    let testgen_stdout = optional_str_strict(inputs, "testgen_stdout")?.unwrap_or("");
    let bootstrap_stdout = optional_str_strict(inputs, "bootstrap_stdout")?.unwrap_or("");
    let pragma_stdout = optional_str_strict(inputs, "pragma_stdout")?.unwrap_or("");
    let guardrail_stdout = optional_str_strict(inputs, "guardrail_stdout")?.unwrap_or("");
    let verify_stdout = optional_str_strict(inputs, "verify_stdout")?.unwrap_or("");

    let stages = [
        StageResult::new("Build", build_success, build_stdout, build_stderr)
            .with_extractor(extract_build_stage),
        StageResult::new("Test", test_success, test_stdout, test_stderr),
        StageResult::new("Lint", lint_success, lint_stdout, lint_stderr)
            .with_extractor(extract_lint_stage),
        StageResult::new("Testgen", testgen_success, testgen_stdout, testgen_stderr),
        StageResult::new(
            "Bootstrap",
            bootstrap_success,
            bootstrap_stdout,
            bootstrap_stderr,
        ),
        StageResult::new("Pragma", pragma_success, pragma_stdout, pragma_stderr),
        StageResult::new(
            "Guardrails",
            guardrail_success,
            guardrail_stdout,
            guardrail_stderr,
        ),
        StageResult::new("Verify", verify_success, verify_stdout, verify_stderr)
            .with_extractor(extract_verify_failures),
    ];

    for stage in &stages {
        if stage.success {
            continue;
        }
        // Special handling for test failures: extract the "failures:" section
        if stage.name == "Test" {
            if let Some(failures_section) = extract_test_failures(stage.stdout) {
                let summary = truncate_for_report(&failures_section);
                blocks.push(StructuredBlock::Raw(format!(
                    "\n--- Test failures ---\n{summary}\n"
                )));
            }
            if !stage.stderr.is_empty() {
                let summary = truncate_for_report(stage.stderr);
                blocks.push(StructuredBlock::Raw(format!(
                    "\n--- Test stderr ---\n{summary}\n"
                )));
            }
            continue;
        }
        if let Some(section) =
            format_stage_failure(stage.name, stage.stdout, stage.stderr, stage.extractor)
        {
            blocks.push(StructuredBlock::Raw(section));
        }
    }

    if !cloud_env_status.is_empty() {
        blocks.push(StructuredBlock::Raw(format!(
            "\n--- Cloud env ---\n{cloud_env_status}\n"
        )));
    }

    Ok(blocks)
}

/// Maximum lines per stderr/stdout section in the CI report.
const MAX_REPORT_SECTION_LINES: usize = 60;
/// Maximum characters per line before truncation.
const MAX_REPORT_LINE_WIDTH: usize = 500;

/// Truncate verbose output for the CI report.
///
/// - Individual lines longer than [`MAX_REPORT_LINE_WIDTH`] are truncated
///   (catches massive linker commands with hundreds of `.rlib` paths).
/// - If the total exceeds [`MAX_REPORT_SECTION_LINES`], the middle is
///   replaced with a marker keeping the first 10 and last 50 lines.
fn truncate_for_report(text: &str) -> String {
    let raw_lines: Vec<&str> = text.lines().collect();

    // Truncate individual long lines
    let lines: Vec<String> = raw_lines
        .iter()
        .map(|line| {
            if line.len() > MAX_REPORT_LINE_WIDTH {
                format!(
                    "{}... ({} more chars)",
                    &line[..MAX_REPORT_LINE_WIDTH],
                    line.len() - MAX_REPORT_LINE_WIDTH
                )
            } else {
                (*line).to_string()
            }
        })
        .collect();

    if lines.len() <= MAX_REPORT_SECTION_LINES {
        return lines.join("\n");
    }

    let head = 10;
    let tail = MAX_REPORT_SECTION_LINES - head;
    let omitted = lines.len() - head - tail;

    let mut result = Vec::with_capacity(head + 1 + tail);
    result.extend_from_slice(&lines[..head]);
    result.push(format!("... ({omitted} lines omitted) ..."));
    result.extend_from_slice(&lines[lines.len() - tail..]);
    result.join("\n")
}

/// Extract `error[E...]` lines + context from build stderr.
///
/// Filters out massive linker `.rlib` paths that bloat the output.
fn extract_build_errors(stderr: &str) -> Option<String> {
    let mut result_lines = Vec::new();
    for line in stderr.lines() {
        // Skip lines that are mostly linker .rlib paths
        if line.contains(".rlib") && line.len() > 200 {
            continue;
        }
        // Include error lines and their context (-->, |, note:)
        if line.starts_with("error")
            || line.contains("error[E")
            || line.starts_with("warning")
            || line.trim_start().starts_with("-->")
            || line.trim_start().starts_with('|')
            || line.trim_start().starts_with("note:")
            || line.trim_start().starts_with("= help:")
            || line.trim_start().starts_with("= note:")
        {
            result_lines.push(line);
        }
    }
    if result_lines.is_empty() {
        None
    } else {
        Some(result_lines.join("\n"))
    }
}

/// Extract `warning:` and `error:` lines + context from clippy output.
fn extract_lint_warnings(stderr: &str) -> Option<String> {
    let mut result_lines = Vec::new();
    for line in stderr.lines() {
        if line.starts_with("warning")
            || line.starts_with("error")
            || line.trim_start().starts_with("-->")
            || line.trim_start().starts_with('|')
            || line.trim_start().starts_with("= help:")
            || line.trim_start().starts_with("= note:")
        {
            result_lines.push(line);
        }
    }
    if result_lines.is_empty() {
        None
    } else {
        Some(result_lines.join("\n"))
    }
}

fn extract_verify_failures(stdout: &str, stderr: &str) -> Option<String> {
    let mut sections = Vec::new();

    let check_summary = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if !check_summary.is_empty() {
        sections.push(format!("failed checks:\n{check_summary}"));
    }

    if !stdout.trim().is_empty() {
        let stdout_tail = extract_tail_summary(stdout, 40).unwrap_or_default();
        if !stdout_tail.trim().is_empty() {
            sections.push(format!("verify output:\n{stdout_tail}"));
        }
    }

    if sections.is_empty() {
        None
    } else {
        Some(sections.join("\n\n"))
    }
}

/// Generic fallback: last N lines of text.
fn extract_tail_summary(text: &str, max_lines: usize) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let start = lines.len().saturating_sub(max_lines);
    Some(lines[start..].join("\n"))
}

/// A single stage's result for report generation.
struct StageResult<'a> {
    name: &'a str,
    success: bool,
    stdout: &'a str,
    stderr: &'a str,
    extractor: Option<fn(&str, &str) -> Option<String>>,
}

impl<'a> StageResult<'a> {
    fn new(name: &'a str, success: bool, stdout: &'a str, stderr: &'a str) -> Self {
        Self {
            name,
            success,
            stdout,
            stderr,
            extractor: None,
        }
    }

    fn with_extractor(mut self, f: fn(&str, &str) -> Option<String>) -> Self {
        self.extractor = Some(f);
        self
    }
}

fn extract_build_stage(_stdout: &str, stderr: &str) -> Option<String> {
    extract_build_errors(stderr)
}

fn extract_lint_stage(_stdout: &str, stderr: &str) -> Option<String> {
    extract_lint_warnings(stderr)
}

/// Unified helper for formatting a stage failure section.
///
/// Tries the extractor on stderr, falls back to `extract_tail_summary`,
/// applies `truncate_for_report` to bound output size.
fn format_stage_failure(
    name: &str,
    stdout: &str,
    stderr: &str,
    extractor: Option<fn(&str, &str) -> Option<String>>,
) -> Option<String> {
    // Try the specialized extractor first (stage-specific).
    if let Some(extract) = extractor {
        if let Some(extracted) = extract(stdout, stderr) {
            let summary = truncate_for_report(&extracted);
            return Some(format!("\n--- {name} errors ---\n{summary}\n"));
        }
    }

    // Fall back to tail of stderr
    if !stderr.is_empty() {
        let tail = extract_tail_summary(stderr, 30).unwrap_or_default();
        let summary = truncate_for_report(&tail);
        return Some(format!("\n--- {name} stderr ---\n{summary}\n"));
    }

    // If stderr empty but stdout has content, show tail of stdout
    if !stdout.is_empty() {
        let tail = extract_tail_summary(stdout, 30).unwrap_or_default();
        let summary = truncate_for_report(&tail);
        return Some(format!("\n--- {name} stdout ---\n{summary}\n"));
    }

    None
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
                .str("testgen_stdout", "")
                .build(),
            CIOp::PrepareBootstrapCommand => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.bootstrap.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseBootstrapResult => OutputMap::new()
                .bool("bootstrap_success", true)
                .str("bootstrap_stderr", "")
                .str("bootstrap_stdout", "")
                .build(),
            CIOp::PreparePragmaCommand => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.pragma.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParsePragmaResult => OutputMap::new()
                .bool("pragma_success", true)
                .str("pragma_stderr", "")
                .str("pragma_stdout", "")
                .build(),
            CIOp::PrepareBuildCommand => {
                let compile_only_test_build = CargoCommand::new(Subcommand::Test)
                    .no_run()
                    .warnings(Warnings::Deny);
                let request = TransportRequest::Shell(compile_only_test_build.to_shell_request());
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
                    ShellRequest::new("bash").args(["-lc", GUARDRAIL_CHECK_COMMAND]),
                );
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseGuardrailResult => OutputMap::new()
                .bool("guardrail_success", true)
                .str("guardrail_stderr", "")
                .str("guardrail_stdout", "")
                .build(),
            CIOp::PrepareClippyLint => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.lint.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseClippyLintResult => OutputMap::new()
                .bool("lint_success", true)
                .bool("lint_skipped", false)
                .str("lint_stdout", "No warnings")
                .str("lint_stderr", "")
                .build(),
            CIOp::PrepareVerifyMakegenCheck => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.makegen_check.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseVerifyMakegenResult => OutputMap::new()
                .bool("verify_makegen_success", true)
                .str("verify_makegen_stderr", "")
                .str("verify_makegen_stdout", "")
                .build(),
            CIOp::PrepareVerifyDepsConfigCheck => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.deps_config_check.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseVerifyDepsConfigResult => OutputMap::new()
                .bool("verify_deps_config_success", true)
                .str("verify_deps_config_stderr", "")
                .str("verify_deps_config_stdout", "")
                .build(),
            CIOp::PrepareVerifyBootstrapCheck => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.bootstrap_check.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseVerifyBootstrapResult => OutputMap::new()
                .bool("verify_bootstrap_success", true)
                .str("verify_bootstrap_stderr", "")
                .str("verify_bootstrap_stdout", "")
                .build(),
            CIOp::PrepareVerifyTestgenCheck => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.testgen_check.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseVerifyTestgenResult => OutputMap::new()
                .bool("verify_testgen_success", true)
                .str("verify_testgen_stderr", "")
                .str("verify_testgen_stdout", "")
                .build(),
            CIOp::PrepareVerifyPragmaCheck => {
                let config = BuildConfig::cargo();
                let request = TransportRequest::Shell(config.pragma_check.to_shell_request());
                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .build()
            }
            CIOp::ParseVerifyPragmaResult => OutputMap::new()
                .bool("verify_pragma_success", true)
                .str("verify_pragma_stderr", "")
                .str("verify_pragma_stdout", "")
                .build(),
            CIOp::AggregateVerifyResults => OutputMap::new()
                .bool("verify_success", true)
                .str("verify_stdout", "")
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
    use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportRequest};

    fn normalize_report(report: &str) -> String {
        report
            .lines()
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n")
    }

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
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            result.get("error_summary").and_then(|v| v.as_str()),
            Some("")
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
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            result.get("detail").and_then(|v| v.as_str()),
            Some("No deps.toml found, skipping dependency check")
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

        let request = result
            .get("request")
            .and_then(|v| v.as_request())
            .expect("request should be a TransportRequest");
        match request {
            TransportRequest::Shell(shell) => {
                assert_eq!(shell.command, "cargo");
                assert_eq!(shell.args, vec!["test".to_string(), "--no-run".to_string()]);
                assert_eq!(shell.env.get("RUSTFLAGS"), Some(&"-D warnings".to_string()));
            }
            other => panic!("expected shell request, got {other:?}"),
        }
    }

    #[test]
    fn test_prepare_build_command_skip() {
        let mut inputs = HashMap::new();
        inputs.insert("prep_success".to_string(), Value::Bool(false));
        inputs.insert("testgen_success".to_string(), Value::Bool(true));

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
    fn test_parse_verify_result_captures_stdout_and_stderr() {
        let mut inputs = HashMap::new();
        inputs.insert("skip".to_string(), Value::Bool(false));
        inputs.insert(
            "response".to_string(),
            Value::Response(
                ShellResponse {
                    exit_code: 1,
                    stdout: "verify details".to_string(),
                    stderr: "verify error".to_string(),
                }
                .into(),
            ),
        );

        let result = execute_parse_verify_makegen_result(inputs).expect("parse should succeed");
        assert_eq!(
            result
                .get("verify_makegen_success")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            result.get("verify_makegen_stdout").and_then(|v| v.as_str()),
            Some("verify details")
        );
        assert_eq!(
            result.get("verify_makegen_stderr").and_then(|v| v.as_str()),
            Some("verify error")
        );
    }

    #[test]
    fn test_aggregate_verify_results_uses_stdout_fallback_for_failure_message() {
        let mut inputs = HashMap::new();
        inputs.insert("verify_makegen_success".to_string(), Value::Bool(false));
        inputs.insert(
            "verify_makegen_stderr".to_string(),
            Value::Str(String::new()),
        );
        inputs.insert(
            "verify_makegen_stdout".to_string(),
            Value::Str("stdout-only failure".to_string()),
        );

        inputs.insert("verify_deps_config_success".to_string(), Value::Bool(true));
        inputs.insert(
            "verify_deps_config_stderr".to_string(),
            Value::Str(String::new()),
        );
        inputs.insert(
            "verify_deps_config_stdout".to_string(),
            Value::Str(String::new()),
        );

        inputs.insert("verify_bootstrap_success".to_string(), Value::Bool(true));
        inputs.insert(
            "verify_bootstrap_stderr".to_string(),
            Value::Str(String::new()),
        );
        inputs.insert(
            "verify_bootstrap_stdout".to_string(),
            Value::Str(String::new()),
        );

        inputs.insert("verify_testgen_success".to_string(), Value::Bool(true));
        inputs.insert(
            "verify_testgen_stderr".to_string(),
            Value::Str(String::new()),
        );
        inputs.insert(
            "verify_testgen_stdout".to_string(),
            Value::Str(String::new()),
        );

        inputs.insert("verify_pragma_success".to_string(), Value::Bool(true));
        inputs.insert(
            "verify_pragma_stderr".to_string(),
            Value::Str(String::new()),
        );
        inputs.insert(
            "verify_pragma_stdout".to_string(),
            Value::Str(String::new()),
        );

        let result = execute_aggregate_verify_results(inputs).expect("aggregation should succeed");
        assert_eq!(
            result.get("verify_success").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            result.get("verify_stderr").and_then(|v| v.as_str()),
            Some("makegen: verify check failed (see stdout output)")
        );
        assert_eq!(
            result.get("verify_stdout").and_then(|v| v.as_str()),
            Some("makegen:\nstdout-only failure")
        );
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(
            result.get("error_summary").and_then(|v| v.as_str()),
            Some("One or more verify checks failed")
        );
    }

    #[test]
    fn test_report_all_pass() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(true));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("lint_success".to_string(), Value::Bool(true));
        inputs.insert("testgen_success".to_string(), Value::Bool(true));
        inputs.insert("bootstrap_success".to_string(), Value::Bool(true));
        inputs.insert("pragma_success".to_string(), Value::Bool(true));
        inputs.insert("guardrail_success".to_string(), Value::Bool(true));
        inputs.insert("verify_success".to_string(), Value::Bool(true));

        let result = execute_report(inputs).unwrap();
        assert_eq!(
            result.get("overall_success").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            result.get("error_summary").and_then(|v| v.as_str()),
            Some("")
        );
        let normalized = normalize_report(
            result
                .get("report")
                .and_then(|v| v.as_str())
                .expect("report text"),
        );
        let expected = "\nCI Report\n=========\nBuild: PASS\nTest:  PASS\nLint:  PASS\nTestgen: PASS\nBootstrap: PASS\nPragma: PASS\nVerify: PASS\nGuardrails: PASS\n---------\nOverall: SUCCESS";
        assert_eq!(normalized.trim_end(), expected);
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
        inputs.insert("testgen_success".to_string(), Value::Bool(true));
        inputs.insert("bootstrap_success".to_string(), Value::Bool(true));
        inputs.insert("pragma_success".to_string(), Value::Bool(true));
        inputs.insert("guardrail_success".to_string(), Value::Bool(true));
        inputs.insert("verify_success".to_string(), Value::Bool(true));

        let result = execute_report(inputs).unwrap();
        assert_eq!(
            result.get("overall_success").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(
            result.get("error_summary").and_then(|v| v.as_str()),
            Some("One or more CI stages failed")
        );
        let normalized = normalize_report(
            result
                .get("report")
                .and_then(|v| v.as_str())
                .expect("report text"),
        );
        assert!(
            normalized.contains("\n--- Build errors ---\nerror: compilation failed"),
            "expected build failure section in report, got:\n{normalized}"
        );
    }

    #[test]
    fn test_report_verify_failure_includes_verify_stdout() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(true));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("lint_success".to_string(), Value::Bool(true));
        inputs.insert("testgen_success".to_string(), Value::Bool(true));
        inputs.insert("bootstrap_success".to_string(), Value::Bool(true));
        inputs.insert("pragma_success".to_string(), Value::Bool(true));
        inputs.insert("guardrail_success".to_string(), Value::Bool(true));
        inputs.insert("verify_success".to_string(), Value::Bool(false));
        inputs.insert(
            "verify_stdout".to_string(),
            Value::Str("verify output details".to_string()),
        );
        inputs.insert("verify_stderr".to_string(), Value::Str(String::new()));

        let result = execute_report(inputs).expect("report should succeed");
        let report = result
            .get("report")
            .and_then(|v| v.as_str())
            .expect("report text");
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(false));
        assert!(report.contains("Verify errors"));
        assert!(report.contains("verify output:"));
        assert!(report.contains("verify output details"));
    }

    #[test]
    fn test_truncate_for_report_short() {
        let text = "line 1\nline 2\nline 3";
        assert_eq!(truncate_for_report(text), text);
    }

    #[test]
    fn test_truncate_for_report_long_lines() {
        let long_line = "x".repeat(600);
        let result = truncate_for_report(&long_line);
        assert!(result.len() < long_line.len());
        assert!(result.contains("more chars)"));
    }

    #[test]
    fn test_truncate_for_report_many_lines() {
        let lines: Vec<String> = (0..200).map(|i| format!("line {i}")).collect();
        let text = lines.join("\n");
        let result = truncate_for_report(&text);
        let result_lines: Vec<&str> = result.lines().collect();
        // Should be capped at MAX_REPORT_SECTION_LINES + 1 (truncation marker)
        assert!(result_lines.len() <= MAX_REPORT_SECTION_LINES + 1);
        assert!(result.contains("lines omitted"));
        // First 10 lines preserved
        assert!(result.contains("line 0"));
        assert!(result.contains("line 9"));
        // Last 50 lines preserved
        assert!(result.contains("line 199"));
    }

    #[test]
    fn test_report_build_fail_truncates_stderr() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(false));
        // Build a massive stderr with 200 lines
        let lines: Vec<String> = (0..200).map(|i| format!("error line {i}")).collect();
        inputs.insert("build_stderr".to_string(), Value::Str(lines.join("\n")));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("test_stdout".to_string(), Value::Str(String::new()));
        inputs.insert("test_stderr".to_string(), Value::Str(String::new()));
        inputs.insert("lint_success".to_string(), Value::Bool(true));
        inputs.insert("lint_stderr".to_string(), Value::Str(String::new()));
        inputs.insert("testgen_success".to_string(), Value::Bool(true));
        inputs.insert("bootstrap_success".to_string(), Value::Bool(true));
        inputs.insert("pragma_success".to_string(), Value::Bool(true));
        inputs.insert("guardrail_success".to_string(), Value::Bool(true));
        inputs.insert("verify_success".to_string(), Value::Bool(true));

        let result = execute_report(inputs).unwrap();
        let report = result.get("report").and_then(|v| v.as_str()).unwrap();
        assert!(report.contains("lines omitted"));
        // Last lines should be preserved (errors tend to be at the end)
        assert!(report.contains("error line 199"));
    }

    #[test]
    fn test_extract_build_errors_filters_rlib_lines() {
        let stderr = format!(
            "error[E0308]: mismatched types\n  --> src/lib.rs:42:5\n  |\n42 |     foo()\n  |     ^^^^^ expected u32\n\n{}\n\nerror: aborting due to previous error",
            "x".repeat(4000) + ".rlib"
        );
        let result = extract_build_errors(&stderr).unwrap();
        assert!(result.contains("error[E0308]"));
        assert!(!result.contains(".rlib"));
        assert!(result.len() < 500);
    }

    #[test]
    fn test_extract_build_errors_empty() {
        assert!(extract_build_errors("Compiling foo v0.1.0\nFinished dev").is_none());
    }

    #[test]
    fn test_extract_lint_warnings() {
        let stderr = "warning: unused variable `x`\n  --> src/main.rs:5:9\n  |\n5 |     let x = 1;\n  |         ^ help: use `_x`\n\nerror: aborting due to previous error";
        let result = extract_lint_warnings(stderr).unwrap();
        assert!(result.contains("warning: unused variable"));
        assert!(result.contains("error: aborting"));
    }

    #[test]
    fn test_extract_lint_warnings_empty() {
        assert!(extract_lint_warnings("Checking foo v0.1.0\nFinished dev").is_none());
    }

    #[test]
    fn test_extract_tail_summary() {
        let text = (0..100)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let result = extract_tail_summary(&text, 5).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 5);
        assert!(result.contains("line 99"));
    }

    #[test]
    fn test_extract_tail_summary_short() {
        let result = extract_tail_summary("just one line", 10).unwrap();
        assert_eq!(result, "just one line");
    }

    #[test]
    fn test_format_stage_failure_with_extractor() {
        let stderr = "error[E0308]: mismatched types\n  --> src/lib.rs:42:5";
        let result = format_stage_failure("Build", "", stderr, Some(extract_build_stage));
        assert!(result.is_some());
        let section = result.unwrap();
        assert!(section.contains("Build errors"));
        assert!(section.contains("error[E0308]"));
    }

    #[test]
    fn test_format_stage_failure_without_extractor() {
        let result = format_stage_failure("Testgen", "", "some error output", None);
        assert!(result.is_some());
        let section = result.unwrap();
        assert!(section.contains("Testgen stderr"));
    }

    #[test]
    fn test_format_stage_failure_empty() {
        let result = format_stage_failure("Build", "", "", Some(extract_build_stage));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_verify_failures_includes_stderr_and_stdout() {
        let stdout = "makegen:\ncheck details\n\npragma:\nmore detail";
        let stderr = "makegen: failed\npragma: failed";
        let extracted = extract_verify_failures(stdout, stderr).expect("expected extraction");
        assert!(extracted.contains("failed checks"));
        assert!(extracted.contains("verify output"));
        assert!(extracted.contains("makegen: failed"));
        assert!(extracted.contains("check details"));
    }

    #[test]
    fn test_regression_linker_explosion() {
        // Regression test: massive linker stderr with 4000-char .rlib paths
        let mut stderr = String::new();
        stderr.push_str("error[E0308]: mismatched types\n");
        stderr.push_str("  --> src/lib.rs:42:5\n");
        // Simulate huge linker line with many .rlib paths
        let rlib_line = (0..100)
            .map(|i| format!("/long/path/to/target/debug/deps/libfoo_{i}.rlib"))
            .collect::<Vec<_>>()
            .join(" ");
        stderr.push_str(&rlib_line);
        stderr.push('\n');
        stderr.push_str("error: aborting due to previous error\n");

        let result = extract_build_errors(&stderr).unwrap();
        // Should not include the giant .rlib line
        assert!(!result.contains(".rlib"));
        // Should include the actual error
        assert!(result.contains("error[E0308]"));
        // Output should be bounded
        assert!(result.len() < 1000);
    }
}
