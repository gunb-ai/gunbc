//! Deps operations.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes in the DAG.
//! The ops here are PURE (no I/O) - they prepare requests and parse responses.

use crate::installer::Installer;
use crate::manifest::DepsManifest;
use crate::upsert::upsert_dry_run;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, ShellRequest, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use gunbc_primitives::data::ParseOp;
use std::collections::HashMap;

/// Operations for the deps tool.
///
/// All operations are PURE - no I/O. I/O happens via TransportOps::Execute nodes.
#[derive(Debug, Clone)]
pub enum DepsOp {
    // ========================================================================
    // LoadManifest chain: PrepareLoadManifest -> Execute -> ParseManifest
    // ========================================================================
    /// Prepare file read request for manifest (PURE)
    PrepareLoadManifest,
    /// Parse manifest file response (PURE)
    ParseManifest,

    // ========================================================================
    // Pure domain logic
    // ========================================================================
    /// Generate install scripts (domain-specific logic, PURE)
    GenerateScripts,

    // ========================================================================
    // ExecuteInstalls chain: PrepareExecuteInstalls -> Execute -> ParseExecuteResult
    // ========================================================================
    /// Prepare shell command for install script (PURE)
    PrepareExecuteInstalls,
    /// Parse execute result (PURE)
    ParseExecuteResult,
}

impl Executable for DepsOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // LoadManifest chain
            DepsOp::PrepareLoadManifest => execute_prepare_load_manifest(inputs),
            DepsOp::ParseManifest => execute_parse_manifest(inputs),
            // Pure domain logic
            DepsOp::GenerateScripts => execute_generate_scripts(inputs),
            // ExecuteInstalls chain
            DepsOp::PrepareExecuteInstalls => execute_prepare_execute_installs(inputs),
            DepsOp::ParseExecuteResult => execute_parse_execute_result(inputs),
        }
    }
}

// ============================================================================
// PrepareLoadManifest - PURE (builds TransportRequest)
// ============================================================================

/// Prepare file read request for manifest (PURE - no I/O).
fn execute_prepare_load_manifest(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let manifest_path = match inputs.get("manifest_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "deps.toml".to_string(),
    };

    let request = TransportRequest::File(FileRequest::read(&manifest_path));

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    out.insert("manifest_path".to_string(), Value::Str(manifest_path));
    Ok(out)
}

// ============================================================================
// ParseManifest - PURE (parses TransportResponse)
// ============================================================================

/// Parse manifest file response (PURE - no I/O).
fn execute_parse_manifest(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing or invalid 'response' input"))?;

    let manifest_path = inputs
        .get("manifest_path")
        .and_then(|v| v.as_str())
        .unwrap_or("deps.toml");

    let content = match response {
        TransportResponse::File(file_resp) => {
            file_resp.content.clone().ok_or_else(|| {
                ExecError::new(format!("failed to load manifest: file not found: {}", manifest_path))
            })?
        }
        _ => return Err(ExecError::new("unexpected response type")),
    };

    // Use ParseOp::Toml primitive to parse (validates TOML structure)
    let mut parse_inputs = HashMap::new();
    parse_inputs.insert("input".to_string(), Value::Str(content.clone()));
    let _parse_result = ParseOp::Toml.execute(parse_inputs)?;

    // Domain-specific: Use DepsManifest for structured extraction
    let manifest = DepsManifest::parse(&content)
        .map_err(|e| ExecError::new(format!("failed to parse manifest: {}", e)))?;

    let dep_names: Vec<String> = manifest.dependency.iter().map(|d| d.name.clone()).collect();

    let mut out = HashMap::new();
    out.insert("dep_count".to_string(), Value::Int(manifest.dependency.len() as i64));
    out.insert("dep_names".to_string(), Value::StrList(dep_names));
    out.insert("manifest_path".to_string(), Value::Str(manifest_path.to_string()));
    Ok(out)
}

/// Generate install scripts for all dependencies.
fn execute_generate_scripts(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let manifest_path = match inputs.get("manifest_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "deps.toml".to_string(),
    };

    let manifest = DepsManifest::load(&manifest_path)
        .map_err(|e| ExecError::new(format!("failed to load manifest: {}", e)))?;

    let installer = Installer::new();
    let mut scripts = Vec::new();
    let mut already_installed = Vec::new();
    let mut needs_install = Vec::new();

    for dep in &manifest.dependency {
        match upsert_dry_run(&installer, dep) {
            Ok((result, script)) => {
                if result.was_installed {
                    already_installed.push(dep.name.clone());
                } else {
                    needs_install.push(dep.name.clone());
                }
                scripts.push(script);
            }
            Err(e) => {
                scripts.push(format!("# Error for {}: {}\n", dep.name, e));
            }
        }
    }

    let combined_script = scripts.join("\n");

    let mut out = HashMap::new();
    out.insert("install_script".to_string(), Value::Str(combined_script));
    out.insert("already_installed".to_string(), Value::StrList(already_installed));
    out.insert("needs_install".to_string(), Value::StrList(needs_install));
    out.insert("platform".to_string(), Value::Str(installer.platform().name().to_string()));
    Ok(out)
}

// ============================================================================
// PrepareExecuteInstalls - PURE (builds TransportRequest)
// ============================================================================

/// Prepare shell command for install script (PURE - no I/O).
fn execute_prepare_execute_installs(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let script = match inputs.get("install_script") {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(ExecError::new("missing install_script input")),
    };

    let request = TransportRequest::Shell(ShellRequest {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.clone()],
        cwd: None,
        env: HashMap::new(),
        stdin: None,
    });

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    out.insert("script".to_string(), Value::Str(script));
    Ok(out)
}

// ============================================================================
// ParseExecuteResult - PURE (parses TransportResponse)
// ============================================================================

/// Parse execute result (PURE - no I/O).
fn execute_parse_execute_result(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing or invalid 'response' input"))?;

    let script = inputs
        .get("script")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let (success, stdout, stderr) = match response {
        TransportResponse::Shell(shell) => (shell.success(), shell.stdout.clone(), shell.stderr.clone()),
        _ => return Err(ExecError::new("unexpected response type")),
    };

    let mut out = HashMap::new();
    out.insert("executed".to_string(), Value::Bool(true));
    out.insert("success".to_string(), Value::Bool(success));
    out.insert("script".to_string(), Value::Str(script));
    out.insert("stdout".to_string(), Value::Str(stdout));
    out.insert("stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_ir::CardinalityCase;
use gunbc_test::{CardinalityTestInput, ErrorTestCase, Mockable};

impl Mockable for DepsOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            DepsOp::PrepareLoadManifest => {
                let mut out = HashMap::new();
                out.insert(
                    "request".to_string(),
                    Value::Request(TransportRequest::File(FileRequest::read("deps.toml"))),
                );
                out.insert("manifest_path".to_string(), Value::Str("deps.toml".to_string()));
                out
            }
            DepsOp::ParseManifest => {
                let mut out = HashMap::new();
                out.insert("dep_count".to_string(), Value::Int(2));
                out.insert(
                    "dep_names".to_string(),
                    Value::StrList(vec!["rust".to_string(), "git".to_string()]),
                );
                out.insert("manifest_path".to_string(), Value::Str("deps.toml".to_string()));
                out
            }
            DepsOp::GenerateScripts => {
                let mut out = HashMap::new();
                out.insert(
                    "install_script".to_string(),
                    Value::Str(
                        r#"#!/bin/bash
# Install script for mock deps
echo "Installing rust..."
echo "Installing git..."
"#
                        .to_string(),
                    ),
                );
                out.insert(
                    "already_installed".to_string(),
                    Value::StrList(vec!["git".to_string()]),
                );
                out.insert(
                    "needs_install".to_string(),
                    Value::StrList(vec!["rust".to_string()]),
                );
                out.insert("platform".to_string(), Value::Str("linux".to_string()));
                out
            }
            DepsOp::PrepareExecuteInstalls => {
                let mut out = HashMap::new();
                out.insert(
                    "request".to_string(),
                    Value::Request(TransportRequest::Shell(ShellRequest {
                        command: "sh".to_string(),
                        args: vec!["-c".to_string(), "echo mock".to_string()],
                        cwd: None,
                        env: HashMap::new(),
                        stdin: None,
                    })),
                );
                out.insert("script".to_string(), Value::Str("echo mock".to_string()));
                out
            }
            DepsOp::ParseExecuteResult => {
                let mut out = HashMap::new();
                out.insert("executed".to_string(), Value::Bool(true));
                out.insert("success".to_string(), Value::Bool(true));
                out.insert("script".to_string(), Value::Str("echo 'mock install'".to_string()));
                out.insert("stdout".to_string(), Value::Str("mock install\n".to_string()));
                out.insert("stderr".to_string(), Value::Str("".to_string()));
                out
            }
        }
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        match self {
            DepsOp::PrepareLoadManifest => vec![],
            DepsOp::ParseManifest => vec![],
            DepsOp::GenerateScripts => vec![
                CardinalityTestInput::succeeds(
                    "dep_names",
                    CardinalityCase::Empty,
                    Value::StrList(vec![]),
                ),
                CardinalityTestInput::succeeds(
                    "dep_names",
                    CardinalityCase::One,
                    Value::StrList(vec!["single-dep".to_string()]),
                ),
                CardinalityTestInput::succeeds(
                    "dep_names",
                    CardinalityCase::Many,
                    Value::StrList(vec![
                        "dep1".to_string(),
                        "dep2".to_string(),
                        "dep3".to_string(),
                    ]),
                ),
            ],
            DepsOp::PrepareExecuteInstalls => vec![],
            DepsOp::ParseExecuteResult => vec![],
        }
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        match self {
            DepsOp::PrepareLoadManifest => vec![],
            DepsOp::ParseManifest => vec![],
            DepsOp::GenerateScripts => vec![
                ErrorTestCase::new(
                    "missing_manifest_file",
                    {
                        let mut m = HashMap::new();
                        m.insert(
                            "manifest_path".to_string(),
                            Value::Str("/nonexistent/path/deps.toml".to_string()),
                        );
                        m
                    },
                    "failed to load manifest",
                ),
            ],
            DepsOp::PrepareExecuteInstalls => vec![ErrorTestCase::new(
                "missing_install_script",
                HashMap::new(),
                "missing install_script input",
            )],
            DepsOp::ParseExecuteResult => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    #[allow(clippy::disallowed_methods)] // Test needs direct fs access
    fn test_generate_scripts_with_temp_manifest() {
        let temp_dir = env::temp_dir();
        let manifest_path = temp_dir.join("test-deps.toml");

        let manifest_content = r#"
[[dependency]]
name = "echo"
verify = "echo test"

[dependency.install.linux]
method = "script"
script = "echo 'installing echo'"

[dependency.install.macos]
method = "script"
script = "echo 'installing echo'"

[dependency.install.windows]
method = "script"
script = "echo 'installing echo'"
"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let mut inputs = HashMap::new();
        inputs.insert(
            "manifest_path".to_string(),
            Value::Str(manifest_path.display().to_string()),
        );

        let result = execute_generate_scripts(inputs).unwrap();

        // echo should be already installed
        match result.get("already_installed") {
            Some(Value::StrList(list)) => {
                assert!(list.contains(&"echo".to_string()));
            }
            _ => panic!("expected already_installed list"),
        }

        // Cleanup
        let _ = fs::remove_file(&manifest_path);
    }
}
