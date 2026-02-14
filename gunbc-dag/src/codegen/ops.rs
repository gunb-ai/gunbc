//! Pure operations for the codegen DAG.

use crate::WorkspaceBinary;
use gunbc_codegen::registry::derive_tool_defs;
use gunbc_exec::{
    optional_response_strict, propagate_skipped, require_bool, require_response, ExecError,
    Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::cargo::{BinaryArgs, CargoCommand, CodegenSubcommand, Subcommand};
use gunbc_ir::resource::{
    check_manifest_freshness, codegen_resource_def, load_manifest_default, ExecMode,
    FreshnessOptions, ManagedResource, ManifestEntry, ManifestFreshness, ResourceDef,
    ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::transport::{FileOp, FileRequest, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use gunbc_ir::{CODEGEN_BIN_DIR, CODEGEN_STAMP_PATH};
use gunbc_lib_transport::TransportIo;
use std::collections::{HashMap, HashSet};

/// Operations for the codegen DAG.
#[derive(Debug, Clone)]
pub enum CodegenOp {
    /// Prepare a file glob request that checks for generated CLI files.
    PrepareCodegenExists,
    /// Parse the exists check response (file-glob response, with shell fallback).
    ParseCodegenExists(ExecMode),
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
            CodegenOp::ParseCodegenExists(mode) => execute_parse_codegen_exists(*mode, inputs),
            CodegenOp::PrepareCodegenCommand => execute_prepare_codegen_command(inputs),
            CodegenOp::ParseCodegenResult => execute_parse_codegen_result(inputs),
            CodegenOp::PrepareStampWrite => execute_prepare_stamp_write(inputs),
        }
    }
}

fn execute_prepare_codegen_exists(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let pattern = format!("{}/**/main.rs", CODEGEN_BIN_DIR);
    let request = TransportRequest::File(FileRequest::glob(pattern));

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn execute_parse_codegen_exists(
    mode: ExecMode,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "response", &["codegen_needed"]) {
        return result;
    }

    let response = require_response(&inputs, "response")?;
    let output_exists = codegen_outputs_exist(response)?;

    let manifest_result = check_codegen_manifest_freshness(output_exists);

    if mode == ExecMode::Verify {
        match &manifest_result {
            ManifestFreshness::Fresh => {}
            ManifestFreshness::Stale(reason) => {
                return Err(ExecError::new(format!(
                    "Generated code is stale: {} (run with --mode=ensure to fix)",
                    reason
                )));
            }
            ManifestFreshness::Missing => {
                return Err(ExecError::new(
                    "Cannot verify freshness: no manifest entry for codegen \
                     (run with --mode=ensure to generate manifest)",
                ));
            }
            ManifestFreshness::Error(err) => {
                return Err(ExecError::new(format!(
                    "Cannot verify freshness: {} (run with --mode=ensure to fix)",
                    err
                )));
            }
        }
    }

    let codegen_needed = match manifest_result {
        ManifestFreshness::Fresh => false,
        ManifestFreshness::Stale(_) => true,
        ManifestFreshness::Missing => !output_exists,
        ManifestFreshness::Error(_) => !output_exists,
    };

    OutputMap::new().bool("codegen_needed", codegen_needed).ok()
}

fn codegen_outputs_exist(response: &TransportResponse) -> Result<bool, ExecError> {
    let expected_paths = expected_codegen_paths();
    if expected_paths.is_empty() {
        return Ok(true);
    }

    match response {
        TransportResponse::File(file) if file.operation == FileOp::Glob => {
            if !file.success {
                return Ok(false);
            }
            let listed_paths: HashSet<&str> = file
                .content
                .as_deref()
                .unwrap_or_default()
                .lines()
                .filter(|line| !line.is_empty())
                .collect();
            Ok(expected_paths
                .iter()
                .all(|path| listed_paths.contains(path.as_str())))
        }
        // Backward compatibility for older tests/mocks still using shell responses.
        TransportResponse::Shell(shell) => Ok(shell.exit_code == 0),
        TransportResponse::File(file) => Err(ExecError::new(format!(
            "expected file glob response for codegen exists check, got {:?}",
            file.operation
        ))),
        other => Err(ExecError::new(format!(
            "unexpected response for codegen exists check: {:?}",
            other
        ))),
    }
}

#[derive(Clone)]
struct CodegenResourceCheck {
    def: ResourceDef,
}

impl CodegenResourceCheck {
    fn new() -> Self {
        Self {
            def: codegen_resource_def(),
        }
    }
}

impl ManagedResource for CodegenResourceCheck {
    fn definition(&self) -> &ResourceDef {
        &self.def
    }

    fn create(
        &self,
        _manifest: &ResourceManifest,
        _io: &dyn ResourceIo,
    ) -> Result<ManifestEntry, ResourceError> {
        Err(ResourceError::CreateFailed(
            self.def.id.clone(),
            "not supported".into(),
        ))
    }
}

/// Check if codegen output is fresh based on the manifest.
///
/// Computes a hash of codegen inputs and compares to the stored manifest key.
/// Also verifies that representative output files exist (manifest might be
/// restored from cache without the actual generated files).
fn check_codegen_manifest_freshness(output_exists: bool) -> ManifestFreshness {
    let io = TransportIo::new();
    let manifest = match load_manifest_default(&io) {
        Ok(m) if m.is_empty() => return ManifestFreshness::Missing,
        Ok(m) => m,
        Err(e) => return ManifestFreshness::Error(format!("manifest load failed: {}", e)),
    };

    let resource = CodegenResourceCheck::new();
    check_manifest_freshness(
        &resource,
        &manifest,
        FreshnessOptions {
            output_exists: Some(output_exists),
            use_mtime: true,
        },
        &io,
    )
}

fn execute_prepare_codegen_command(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "codegen_needed", &["skip"]) {
        return result;
    }

    let codegen_needed = require_bool(&inputs, "codegen_needed")?;

    if !codegen_needed {
        return OutputMap::new().bool("skip", true).ok();
    }

    let inv = WorkspaceBinary::Codegen.invocation();
    let cmd = CargoCommand::new(Subcommand::Run(inv))
        .release()
        .args(BinaryArgs::codegen(CodegenSubcommand::Codegen));
    let request = TransportRequest::Shell(cmd.to_shell_request());

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn execute_parse_codegen_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Check skip flag first: when codegen was skipped (already present),
    // response is Value::Skipped but we still know the outcome.
    // If skip itself is Skipped (skip propagation), propagate to all outputs.
    if let Some(result) = propagate_skipped(
        &inputs,
        "skip",
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

    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["prep_success", "codegen_ran", "prep_message"],
    ) {
        return result;
    }

    let response = optional_response_strict(&inputs, "response")?;

    let (success, message) = if let Some(response) = response {
        let shell = response.require_shell()?;
        let success = shell.success();
        let message = if success {
            "Codegen completed successfully".to_string()
        } else {
            format!("Codegen failed: {}", shell.stderr)
        };
        (success, message)
    } else {
        (false, "Codegen failed: missing response".to_string())
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
    if let Some(result) = propagate_skipped(&inputs, "prep_success", &["skip"]) {
        return result;
    }

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
    derive_tool_defs()
        .into_iter()
        .filter(|tool| tool.invocation.is_some())
        .map(|tool| format!("{}/{}/main.rs", CODEGEN_BIN_DIR, tool.meta.tool_name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{FileResponse, ShellResponse};

    #[test]
    fn test_codegen_outputs_exist_from_glob_response() {
        let expected = expected_codegen_paths();
        let response = TransportResponse::File(FileResponse::glob_result(
            format!("{}/**/main.rs", CODEGEN_BIN_DIR),
            expected,
        ));
        assert!(codegen_outputs_exist(&response).expect("glob response should parse"));
    }

    #[test]
    fn test_codegen_outputs_exist_accepts_legacy_shell_response() {
        let response = TransportResponse::Shell(ShellResponse::ok(""));
        assert!(codegen_outputs_exist(&response).expect("shell response should parse"));
    }

    #[test]
    fn test_expected_paths_non_empty() {
        let paths = expected_codegen_paths();
        assert!(!paths.is_empty());
        assert!(paths.iter().all(|p| p.starts_with(CODEGEN_BIN_DIR)));
    }
}
