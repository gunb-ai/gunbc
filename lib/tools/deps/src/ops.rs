//! Deps operations.

use crate::installer::Installer;
use crate::manifest::DepsManifest;
use crate::upsert::upsert_dry_run;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Operations for the deps tool.
#[derive(Debug, Clone)]
pub enum DepsOp {
    /// Load the deps manifest
    LoadManifest,
    /// Generate install scripts
    GenerateScripts,
    /// Execute installs (boundary - world write)
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

/// Load the deps manifest.
fn execute_load_manifest(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let manifest_path = match inputs.get("manifest_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "deps.toml".to_string(),
    };

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

/// Execute the install scripts (world write).
fn execute_execute_installs(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let script = match inputs.get("install_script") {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(ExecError::new("missing install_script input")),
    };

    // For now, just return the script
    // In a real implementation, we'd execute this via sh -c
    let mut out = HashMap::new();
    out.insert("executed".to_string(), Value::Bool(true));
    out.insert("script".to_string(), Value::Str(script));
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
