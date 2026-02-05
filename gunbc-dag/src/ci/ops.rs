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
use gunbc_ir::resource::{ExecMode, HashBuilder, ResourceManifest};
use gunbc_ir::transport::{FileRequest, TransportRequest};
use gunbc_ir::{ResourceId, Value};
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
// Prep Stage - Pure Operations
// ============================================================================

/// Prepare file exists check for codegen directory (pure).
///
/// This is a fallback check - the primary freshness check uses the manifest.
/// The file check runs in parallel and serves as a bootstrap fallback when
/// the manifest doesn't exist yet.
fn execute_prepare_codegen_exists_check(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Check for a representative generated file - deps CLI is always generated
    // This is a fallback for when the manifest doesn't exist (first run)
    let request = TransportRequest::File(FileRequest::exists("target/codegen/bin/deps/main.rs"));

    OutputMap::new().request("request", request).ok()
}

/// Parse the codegen exists check result with manifest-based freshness (pure).
///
/// Uses a two-tier freshness check:
/// 1. **Manifest check** (primary): Compare stored hash to computed input hash
/// 2. **File existence** (fallback): If manifest is missing, use file existence
///
/// In **verify mode** (`--mode=verify`), stale/missing resources cause immediate failure.
/// In **ensure mode** (`--mode=ensure`, default), stale/missing resources trigger codegen.
fn execute_parse_codegen_exists(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &[
            "codegen_needed",
            "prep_success",
            "codegen_ran",
            "prep_message",
        ],
    ) {
        return result;
    }

    let response = require_response(&inputs, "response")?;

    let file_resp = response.require_file()?;
    let file_exists = file_resp
        .exists
        .ok_or_else(|| ExecError::new("codegen exists check missing 'exists' field"))?;

    // Get resource mode from environment (set by CI main.rs)
    let exec_mode = get_exec_mode_from_env();

    // Primary check: manifest-based freshness
    let manifest_result = check_codegen_manifest_freshness();

    // In verify mode, stale/missing resources are errors
    if exec_mode == ExecMode::Verify {
        match &manifest_result {
            ManifestCheckResult::Stale(reason) => {
                return Err(ExecError::new(&format!(
                    "Generated code is stale: {} (run with --mode=ensure to fix)",
                    reason
                )));
            }
            ManifestCheckResult::Missing if !file_exists => {
                return Err(ExecError::new(
                    "Generated code missing and no manifest (run with --mode=ensure to fix)",
                ));
            }
            _ => {} // Fresh or missing-with-file-fallback is ok
        }
    }

    let (codegen_needed, message) = match manifest_result {
        ManifestCheckResult::Fresh => {
            // Manifest says inputs haven't changed - codegen not needed
            (false, "Generated code is fresh (manifest check passed)")
        }
        ManifestCheckResult::Stale(reason) => {
            // Inputs changed - codegen needed even if files exist
            (true, reason)
        }
        ManifestCheckResult::Missing => {
            // No manifest - fall back to file existence check (bootstrap scenario)
            if file_exists {
                (false, "Generated code exists (no manifest, using file fallback)")
            } else {
                (true, "Generated code missing (no manifest)")
            }
        }
        ManifestCheckResult::Error(err) => {
            // Error checking manifest - fall back to file existence
            eprintln!("Warning: Manifest check failed: {}", err);
            if file_exists {
                (false, "Generated code exists (manifest check failed, using file fallback)")
            } else {
                (true, "Generated code missing (manifest check failed)")
            }
        }
    };

    let mut out = OutputMap::new().bool("codegen_needed", codegen_needed);

    if !codegen_needed {
        out = out
            .bool("prep_success", true)
            .bool("codegen_ran", false)
            .str("prep_message", message);
    }

    out.ok()
}

/// Get the execution mode from environment variable.
///
/// Reads `GUNBC_EXEC_MODE` which is set by the CI main.rs based on --mode flag.
fn get_exec_mode_from_env() -> ExecMode {
    match std::env::var("GUNBC_EXEC_MODE").as_deref() {
        Ok("verify") => ExecMode::Verify,
        Ok("ensure") | _ => ExecMode::Ensure,
    }
}

/// Result of checking the codegen manifest for freshness.
enum ManifestCheckResult {
    /// Manifest exists and inputs are unchanged
    Fresh,
    /// Manifest exists but inputs have changed
    Stale(&'static str),
    /// Manifest doesn't exist
    Missing,
    /// Error reading manifest or computing hash
    Error(String),
}

/// Check if codegen output is fresh based on the manifest.
///
/// Computes a hash of codegen inputs and compares to the stored manifest key.
fn check_codegen_manifest_freshness() -> ManifestCheckResult {
    // Load manifest
    let manifest = match ResourceManifest::load_default() {
        Ok(m) => m,
        Err(_) => return ManifestCheckResult::Missing,
    };

    // Check for codegen entry
    let resource_id = ResourceId::build("generated_cli");
    let entry = match manifest.get(&resource_id) {
        Some(e) => e,
        None => return ManifestCheckResult::Missing,
    };

    // Compute current input hash
    let current_hash = match compute_codegen_input_hash() {
        Ok(h) => h,
        Err(e) => return ManifestCheckResult::Error(e.to_string()),
    };

    // Compare
    if entry.key == current_hash {
        ManifestCheckResult::Fresh
    } else {
        ManifestCheckResult::Stale("inputs changed since last codegen")
    }
}

/// Compute hash of codegen inputs (same as in codegen main.rs).
fn compute_codegen_input_hash() -> Result<gunbc_ir::resource::ContentHash, std::io::Error> {
    let builder = HashBuilder::new();

    // Hash codegen source files
    let (builder, _) = builder.update_glob("core/codegen/src/**/*.rs")?;

    // Hash IR source files
    let (builder, _) = builder.update_glob("core/ir/src/**/*.rs")?;

    // Hash relevant Cargo.toml files
    let builder = builder.update_file("core/codegen/Cargo.toml")?;
    let builder = builder.update_file("core/ir/Cargo.toml")?;

    // Include Rust version
    let rust_version = std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let builder = builder.update_str(&rust_version);

    Ok(builder.finalize())
}

/// Prepare the codegen shell command (pure).
fn execute_prepare_codegen_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Use optional_bool to handle Value::Skipped gracefully.
    // If codegen_needed is missing/Skipped, skip codegen.
    let codegen_needed = optional_bool(&inputs, "codegen_needed").unwrap_or(false);

    if !codegen_needed {
        return OutputMap::new().bool("skip", true).ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.codegen.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse the codegen shell response (pure).
fn execute_parse_codegen_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Use optional_bool to handle Value::Skipped gracefully (for skip propagation tests).
    // If skip is missing/Skipped, default to false and let propagate_skipped handle it.
    let skip = optional_bool(&inputs, "skip").unwrap_or(false);

    if skip {
        // Codegen was skipped because it already exists
        return OutputMap::new()
            .bool("prep_success", true)
            .bool("codegen_ran", false)
            .str("prep_message", "Generated code already exists")
            .ok();
    }

    // Propagate skipped if response is Skipped
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["prep_success", "codegen_ran", "prep_message"],
    ) {
        return result;
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;
    let success = shell.success();

    let message = if success {
        "Codegen completed successfully".to_string()
    } else {
        format!("Codegen failed: {}", shell.stderr)
    };

    OutputMap::new()
        .bool("prep_success", success)
        .bool("codegen_ran", true)
        .str("prep_message", message)
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

    if !prep_success {
        return OutputMap::new()
            .bool("skip", true)
            .str("skip_reason", "Skipped due to prep failure")
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
        if overall_success {
            "SUCCESS"
        } else {
            "FAILURE"
        }
    );

    // Append failure details for any failed stage
    if !build_success {
        let stderr = require_str(&inputs, "build_stderr")?;
        if !stderr.is_empty() {
            report.push_str(&format!("\n--- Build stderr ---\n{stderr}\n"));
        }
    }

    if !test_success {
        // For tests, try to extract the "failures:" section from stdout - that's where
        // the actual test names and panic messages are (stderr just says "test failed")
        let stdout = require_str(&inputs, "test_stdout")?;

        // Try to extract just the failures section for cleaner output
        if let Some(failures_section) = extract_test_failures(stdout) {
            report.push_str(&format!("\n--- Test failures ---\n{failures_section}\n"));
        } else if !stdout.is_empty() {
            // Fallback: show full stdout if we couldn't extract failures section
            // (this handles different cargo test output formats)
            report.push_str(&format!("\n--- Test stdout ---\n{stdout}\n"));
        }

        let stderr = require_str(&inputs, "test_stderr")?;
        if !stderr.is_empty() {
            report.push_str(&format!("\n--- Test stderr ---\n{stderr}\n"));
        }
    }

    if !lint_success {
        let stderr = require_str(&inputs, "lint_stderr")?;
        if !stderr.is_empty() {
            report.push_str(&format!("\n--- Lint stderr ---\n{stderr}\n"));
        }
    }

    OutputMap::new()
        .bool("overall_success", overall_success)
        .str("report", report)
        .ok()
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
            CIOp::PrepareCodegenExistsCheck => OutputMap::new()
                .request(
                    "request",
                    TransportRequest::File(FileRequest::exists("target/codegen/bin/deps/main.rs")),
                )
                .build(),
            CIOp::ParseCodegenExists => OutputMap::new()
                .bool("codegen_needed", false)
                .bool("prep_success", true)
                .bool("codegen_ran", false)
                .str("prep_message", "Generated code exists")
                .build(),
            CIOp::PrepareCodegenCommand => OutputMap::new().bool("skip", true).build(),
            CIOp::ParseCodegenResult => OutputMap::new()
                .bool("prep_success", true)
                .bool("codegen_ran", false)
                .str("prep_message", "Generated code exists")
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
            CIOp::PrepareClippyLint => OutputMap::new().bool("skip", false).build(),
            CIOp::ParseClippyLintResult => OutputMap::new()
                .bool("lint_success", true)
                .bool("lint_skipped", false)
                .str("lint_stdout", "No warnings")
                .str("lint_stderr", "")
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
