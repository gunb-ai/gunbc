//! Pure operations for the codegen DAG.

use gunbc_codegen::registry::all_tools;
use gunbc_exec::{
    propagate_skipped, require_bool, require_response, ExecError, Executable, OutputMap,
    TransportResponseExt,
};
use gunbc_infra::freshness::{check_freshness_mtime, MtimeResult};
use gunbc_ir::cargo::{CargoCommand, CargoInvocation, Subcommand};
use gunbc_ir::resource::{
    codegen_resource_def, compute_key_from_def, mtime_inputs_from_def, ExecMode,
    ResourceManifest,
};
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
    mode: ExecMode,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "response", &["codegen_needed"]) {
        return result;
    }

    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;
    let output_exists = shell.exit_code == 0;

    let manifest_result = check_codegen_manifest_freshness(output_exists);

    if mode == ExecMode::Verify {
        match &manifest_result {
            ManifestCheckResult::Fresh => {}
            ManifestCheckResult::Stale(reason) => {
                return Err(ExecError::new(format!(
                    "Generated code is stale: {} (run with --mode=ensure to fix)",
                    reason
                )));
            }
            ManifestCheckResult::Missing => {
                return Err(ExecError::new(
                    "Cannot verify freshness: no manifest entry for codegen \
                     (run with --mode=ensure to generate manifest)",
                ));
            }
            ManifestCheckResult::Error(err) => {
                return Err(ExecError::new(format!(
                    "Cannot verify freshness: {} (run with --mode=ensure to fix)",
                    err
                )));
            }
        }
    }

    let codegen_needed = match manifest_result {
        ManifestCheckResult::Fresh => false,
        ManifestCheckResult::Stale(_) => true,
        ManifestCheckResult::Missing => !output_exists,
        ManifestCheckResult::Error(_) => !output_exists,
    };

    OutputMap::new().bool("codegen_needed", codegen_needed).ok()
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
/// Also verifies that representative output files exist (manifest might be
/// restored from cache without the actual generated files).
fn check_codegen_manifest_freshness(output_exists: bool) -> ManifestCheckResult {
    let manifest = match ResourceManifest::load_default() {
        Ok(m) if m.is_empty() => return ManifestCheckResult::Missing,
        Ok(m) => m,
        Err(e) => {
            let kind = e.kind();
            if kind == std::io::ErrorKind::NotFound {
                return ManifestCheckResult::Missing;
            }
            return ManifestCheckResult::Error(format!("manifest load failed: {}", e));
        }
    };

    let def = codegen_resource_def();
    let entry = match manifest.get(&def.id) {
        Some(e) => e,
        None => return ManifestCheckResult::Missing,
    };

    if !output_exists {
        return ManifestCheckResult::Stale("manifest present but output files missing");
    }

    let mtime_inputs = mtime_inputs_from_def(&def);
    if !mtime_inputs.has_non_file_inputs {
        let glob_refs: Vec<&str> = mtime_inputs
            .glob_patterns
            .iter()
            .map(|s| s.as_str())
            .collect();
        let file_refs: Vec<&str> = mtime_inputs
            .files
            .iter()
            .map(|s| s.as_str())
            .collect();
        if let MtimeResult::Fresh = check_freshness_mtime(entry, &glob_refs, &file_refs) {
            return ManifestCheckResult::Fresh;
        }
    }

    let (current_hash, _count) = match compute_key_from_def(&def, &manifest) {
        Ok(h) => h,
        Err(e) => return ManifestCheckResult::Error(e.to_string()),
    };

    if entry.key == current_hash {
        ManifestCheckResult::Fresh
    } else {
        ManifestCheckResult::Stale("inputs changed since last codegen")
    }
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
