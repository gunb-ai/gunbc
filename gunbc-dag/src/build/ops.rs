//! Build operations - all pure.
//!
//! Follows the transport pattern:
//! ```text
//! [Prepare*Op] -> [TransportOps::Execute] -> [Parse*Op]
//!    (pure)           (boundary)              (pure)
//! ```

use crate::makegen::BuildConfig;
use gunbc_exec::{
    require_bool, require_response, require_str, ExecError, Executable, OutputMap,
    TransportResponseExt,
};
use gunbc_ir::transport::TransportRequest;
use gunbc_ir::Value;
use std::collections::HashMap;

/// Pure operations for the build pipeline.
#[derive(Debug, Clone)]
pub enum BuildOp {
    /// Prepare `cargo build --all-targets` command (pure).
    PrepareBuild,
    /// Parse build response: extract success/stdout/stderr (pure).
    ParseBuild,
    /// Prepare `cargo test` command, skip if build failed (pure).
    PrepareTest,
    /// Parse test response (pure).
    ParseTest,
    /// Prepare `cargo clippy` command, skip if build failed (pure).
    PrepareClippy,
    /// Parse clippy response (pure).
    ParseClippy,
    /// Summarize results from test and clippy (pure).
    Summary,
}

impl Executable for BuildOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BuildOp::PrepareBuild => exec_prepare_build(inputs),
            BuildOp::ParseBuild => exec_parse_build(inputs),
            BuildOp::PrepareTest => exec_prepare_test(inputs),
            BuildOp::ParseTest => exec_parse_test(inputs),
            BuildOp::PrepareClippy => exec_prepare_clippy(inputs),
            BuildOp::ParseClippy => exec_parse_clippy(inputs),
            BuildOp::Summary => exec_summary(inputs),
        }
    }
}

// ============================================================================
// Build Stage
// ============================================================================

fn exec_prepare_build(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.build.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn exec_parse_build(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;
    let success = shell.exit_code == 0;

    OutputMap::new()
        .bool("build_success", success)
        .str("build_stdout", shell.stdout.clone())
        .str("build_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Test Stage
// ============================================================================

fn exec_prepare_test(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = require_bool(&inputs, "build_success")?;

    if !build_success {
        return OutputMap::new().bool("skip", true).ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.test.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn exec_parse_test(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let skip = require_bool(&inputs, "skip")?;

    if skip {
        return OutputMap::new()
            .bool("test_success", false)
            .bool("test_skipped", true)
            .str("test_stdout", "")
            .str("test_stderr", "skipped: build failed")
            .ok();
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;
    let success = shell.exit_code == 0;

    OutputMap::new()
        .bool("test_success", success)
        .bool("test_skipped", false)
        .str("test_stdout", shell.stdout.clone())
        .str("test_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Clippy Stage
// ============================================================================

fn exec_prepare_clippy(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = require_bool(&inputs, "build_success")?;

    if !build_success {
        return OutputMap::new().bool("skip", true).ok();
    }

    let config = BuildConfig::cargo();
    let request = TransportRequest::Shell(config.lint.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn exec_parse_clippy(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let skip = require_bool(&inputs, "skip")?;

    if skip {
        return OutputMap::new()
            .bool("clippy_success", false)
            .bool("clippy_skipped", true)
            .str("clippy_stdout", "")
            .str("clippy_stderr", "skipped: build failed")
            .ok();
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;
    let success = shell.exit_code == 0;

    OutputMap::new()
        .bool("clippy_success", success)
        .bool("clippy_skipped", false)
        .str("clippy_stdout", shell.stdout.clone())
        .str("clippy_stderr", shell.stderr.clone())
        .ok()
}

// ============================================================================
// Summary Stage
// ============================================================================

fn exec_summary(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let build_success = require_bool(&inputs, "build_success")?;
    let test_success = require_bool(&inputs, "test_success")?;
    let clippy_success = require_bool(&inputs, "clippy_success")?;

    let test_stderr = require_str(&inputs, "test_stderr")?.to_string();
    let clippy_stderr = require_str(&inputs, "clippy_stderr")?.to_string();
    let build_stderr = require_str(&inputs, "build_stderr")?.to_string();

    let overall = build_success && test_success && clippy_success;

    let mut report = String::new();
    if !build_success {
        report.push_str(&format!("Build FAILED\n{}\n", build_stderr));
    }
    if !test_success {
        report.push_str(&format!("Test FAILED\n{}\n", test_stderr));
    }
    if !clippy_success {
        report.push_str(&format!("Clippy FAILED\n{}\n", clippy_stderr));
    }
    if overall {
        report.push_str("All checks passed.");
    }

    OutputMap::new()
        .bool("overall_success", overall)
        .str("report", report)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{ShellResponse, TransportResponse};

    #[test]
    fn test_prepare_build() {
        let out = exec_prepare_build(HashMap::new()).unwrap();
        assert!(out.contains_key("request"));
        assert!(matches!(out["request"], Value::Request(_)));
    }

    #[test]
    fn test_parse_build_success() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(ShellResponse::ok("ok"))),
        );
        let out = exec_parse_build(inputs).unwrap();
        assert_eq!(out["build_success"], Value::Bool(true));
    }

    #[test]
    fn test_parse_build_failure() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(ShellResponse::failed(1, "error[E0308]"))),
        );
        let out = exec_parse_build(inputs).unwrap();
        assert_eq!(out["build_success"], Value::Bool(false));
    }

    #[test]
    fn test_prepare_test_skips_on_build_failure() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(false));
        let out = exec_prepare_test(inputs).unwrap();
        assert_eq!(out["skip"], Value::Bool(true));
    }

    #[test]
    fn test_summary_all_pass() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(true));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("clippy_success".to_string(), Value::Bool(true));
        inputs.insert("build_stderr".to_string(), Value::Str(String::new()));
        inputs.insert("test_stderr".to_string(), Value::Str(String::new()));
        inputs.insert("clippy_stderr".to_string(), Value::Str(String::new()));
        let out = exec_summary(inputs).unwrap();
        assert_eq!(out["overall_success"], Value::Bool(true));
    }

    #[test]
    fn test_summary_build_fails() {
        let mut inputs = HashMap::new();
        inputs.insert("build_success".to_string(), Value::Bool(false));
        inputs.insert("test_success".to_string(), Value::Bool(true));
        inputs.insert("clippy_success".to_string(), Value::Bool(true));
        inputs.insert("build_stderr".to_string(), Value::Str("err".to_string()));
        inputs.insert("test_stderr".to_string(), Value::Str(String::new()));
        inputs.insert("clippy_stderr".to_string(), Value::Str(String::new()));
        let out = exec_summary(inputs).unwrap();
        assert_eq!(out["overall_success"], Value::Bool(false));
        assert!(out["report"].as_str().unwrap().contains("Build FAILED"));
    }
}
