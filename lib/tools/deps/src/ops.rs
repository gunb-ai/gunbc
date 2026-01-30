//! Deps operations.
//!
//! Demonstrates decomposition into primitives where possible.
//! Domain-specific logic (manifest parsing, platform detection) remains,
//! but file I/O and command execution delegate to primitives.

use crate::installer::Installer;
use crate::manifest::DepsManifest;
use crate::upsert::upsert_dry_run;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_primitives::io::{ExecuteOp, ReadFileOp};
use gunbc_primitives::data::ParseOp;
use std::collections::HashMap;

/// Operations for the deps tool.
///
/// These operations use primitives internally where possible:
/// - `LoadManifest`: Uses `ReadFileOp` + `ParseOp::Toml` internally
/// - `ExecuteInstalls`: Uses `ExecuteOp` internally
#[derive(Debug, Clone)]
pub enum DepsOp {
    /// Load the deps manifest (uses ReadFile + Parse primitives internally)
    LoadManifest,
    /// Generate install scripts (domain-specific logic)
    GenerateScripts,
    /// Execute installs (boundary - uses Execute primitive internally)
    ExecuteInstalls,
}

impl Executable for DepsOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DepsOp::LoadManifest => execute_load_manifest(inputs),
            DepsOp::GenerateScripts => execute_generate_scripts(inputs),
            DepsOp::ExecuteInstalls => execute_execute_installs(inputs),
        }
    }
}

/// Load the deps manifest using primitives internally.
///
/// Decomposition:
/// 1. ReadFileOp reads the manifest file
/// 2. ParseOp::Toml parses the TOML content
/// 3. Domain-specific extraction of dependency info
fn execute_load_manifest(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let manifest_path = match inputs.get("manifest_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "deps.toml".to_string(),
    };

    // Step 1: Use ReadFileOp primitive to read the file
    let mut read_inputs = HashMap::new();
    read_inputs.insert("path".to_string(), Value::Str(manifest_path.clone()));
    let read_result = ReadFileOp.execute(read_inputs)?;
    
    let content = read_result
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new(format!("failed to read manifest: {}", manifest_path)))?;

    let exists = read_result
        .get("exists")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !exists {
        return Err(ExecError::new(format!("failed to load manifest: file not found: {}", manifest_path)));
    }

    // Step 2: Use ParseOp::Toml primitive to parse (validates TOML structure)
    let mut parse_inputs = HashMap::new();
    parse_inputs.insert("input".to_string(), Value::Str(content.to_string()));
    let _parse_result = ParseOp::Toml.execute(parse_inputs)?;

    // Step 3: Domain-specific: Use DepsManifest for structured extraction
    // (This could be further decomposed with ExtractOp, but DepsManifest
    // handles schema validation and defaults)
    let manifest = DepsManifest::load(&manifest_path)
        .map_err(|e| ExecError::new(format!("failed to load manifest: {}", e)))?;

    let dep_names: Vec<String> = manifest.dependency.iter().map(|d| d.name.clone()).collect();

    let mut out = HashMap::new();
    out.insert("dep_count".to_string(), Value::Int(manifest.dependency.len() as i64));
    out.insert("dep_names".to_string(), Value::StrList(dep_names));
    out.insert("manifest_path".to_string(), Value::Str(manifest_path));
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

/// Execute the install scripts using ExecuteOp primitive.
///
/// Decomposition:
/// 1. ExecuteOp runs the script via sh -c
fn execute_execute_installs(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let script = match inputs.get("install_script") {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(ExecError::new("missing install_script input")),
    };

    // Use ExecuteOp primitive to run the script
    let mut exec_inputs = HashMap::new();
    exec_inputs.insert("command".to_string(), Value::Str("sh".to_string()));
    exec_inputs.insert("args".to_string(), Value::StrList(vec!["-c".to_string(), script.clone()]));
    
    let exec_result = ExecuteOp.execute(exec_inputs)?;

    let success = exec_result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let stdout = exec_result
        .get("stdout")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let stderr = exec_result
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

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
            DepsOp::LoadManifest => {
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
            DepsOp::ExecuteInstalls => {
                let mut out = HashMap::new();
                out.insert("executed".to_string(), Value::Bool(true));
                out.insert(
                    "script".to_string(),
                    Value::Str("echo 'mock install'".to_string()),
                );
                out
            }
        }
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        match self {
            DepsOp::LoadManifest => vec![
                // manifest_path is optional (defaults to deps.toml)
            ],
            DepsOp::GenerateScripts => vec![
                // dep_names could be tested with cardinality
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
            DepsOp::ExecuteInstalls => vec![],
        }
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        match self {
            DepsOp::LoadManifest => vec![
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
            DepsOp::ExecuteInstalls => vec![ErrorTestCase::new(
                "missing_install_script",
                HashMap::new(),
                "missing install_script input",
            )],
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
