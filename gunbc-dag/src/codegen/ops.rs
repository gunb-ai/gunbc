//! Pure operations for the codegen DAG.

use gunbc_codegen::registry::all_tools;
use gunbc_exec::{
    propagate_skipped, require_bool, require_response, ExecError, Executable, OutputMap,
    TransportResponseExt,
};
use gunbc_ir::cargo::{CargoCommand, CargoInvocation, Subcommand};
use gunbc_ir::transport::{FileRequest, ShellRequest, TransportRequest};
use gunbc_ir::Value;
use gunbc_ir::{CODEGEN_BIN_DIR, CODEGEN_STAMP_PATH};
use std::collections::HashMap;

/// Operations for the codegen DAG.
#[derive(Debug, Clone)]
pub enum CodegenOp {
    /// Prepare a shell request that checks for generated CLI files.
    PrepareCodegenExists,
    /// Parse the exists check response.
    ParseCodegenExists,
    /// Prepare the codegen shell command.
    PrepareCodegenCommand,
    /// Parse the codegen command response.
    ParseCodegenResult,
    /// Prepare a stamp file write on success.
    PrepareStampWrite,
}

impl Executable for CodegenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CodegenOp::PrepareCodegenExists => execute_prepare_codegen_exists(inputs),
            CodegenOp::ParseCodegenExists => execute_parse_codegen_exists(inputs),
            CodegenOp::PrepareCodegenCommand => execute_prepare_codegen_command(inputs),
            CodegenOp::ParseCodegenResult => execute_parse_codegen_result(inputs),
            CodegenOp::PrepareStampWrite => execute_prepare_stamp_write(inputs),
        }
    }
}

fn execute_prepare_codegen_exists(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let paths = expected_codegen_paths();

    // If no tools are registered, treat as "exists".
    if paths.is_empty() {
        let request = TransportRequest::Shell(ShellRequest {
            command: "true".to_string(),
            args: Vec::new(),
            cwd: None,
            env: HashMap::new(),
            stdin: None,
        });
        return OutputMap::new().request("request", request).ok();
    }

    let mut cmd = String::new();
    for path in paths {
        if !cmd.is_empty() {
            cmd.push_str(" && ");
        }
        cmd.push_str("test -f ");
        cmd.push_str(&shell_quote(&path));
    }

    let request = TransportRequest::Shell(ShellRequest {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), cmd],
        cwd: None,
        env: HashMap::new(),
        stdin: None,
    });

    OutputMap::new().request("request", request).ok()
}

fn execute_parse_codegen_exists(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "response", &["codegen_needed"]) {
        return result;
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;
    let exists = shell.exit_code == 0;

    OutputMap::new().bool("codegen_needed", !exists).ok()
}

fn execute_prepare_codegen_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let codegen_needed = require_bool(&inputs, "codegen_needed")?;

    if !codegen_needed {
        return OutputMap::new().bool("skip", true).ok();
    }

    let inv = CargoInvocation::standalone("codegen");
    let cmd = CargoCommand::new(Subcommand::Run(inv))
        .release()
        .trailing_arg("codegen");
    let request = TransportRequest::Shell(cmd.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn execute_parse_codegen_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["prep_success", "codegen_ran", "prep_message"],
    ) {
        return result;
    }

    let skip = require_bool(&inputs, "skip")?;
    if skip {
        return OutputMap::new()
            .bool("prep_success", true)
            .bool("codegen_ran", false)
            .str("prep_message", "Codegen skipped (already present)")
            .ok();
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

fn execute_prepare_stamp_write(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let prep_success = require_bool(&inputs, "prep_success")?;
    if !prep_success {
        return OutputMap::new().bool("skip", true).ok();
    }

    let content = "codegen ok\n";
    let request = TransportRequest::File(FileRequest::write(CODEGEN_STAMP_PATH, content));

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn expected_codegen_paths() -> Vec<String> {
    all_tools()
        .into_iter()
        .map(|tool| format!("{}/{}/main.rs", CODEGEN_BIN_DIR, tool.meta.tool_name))
        .collect()
}

fn shell_quote(value: &str) -> String {
    if value.contains('\'') {
        let escaped = value.replace('\'', "'\"'\"'");
        format!("'{}'", escaped)
    } else {
        format!("'{}'", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_quote_simple() {
        assert_eq!(shell_quote("path/to/file"), "'path/to/file'");
    }

    #[test]
    fn test_shell_quote_with_single_quote() {
        assert_eq!(shell_quote("a'b"), "'a'\"'\"'b'");
    }

    #[test]
    fn test_expected_paths_non_empty() {
        let paths = expected_codegen_paths();
        assert!(!paths.is_empty());
        assert!(paths.iter().all(|p| p.starts_with(CODEGEN_BIN_DIR)));
    }
}
