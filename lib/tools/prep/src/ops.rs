//! Prep operations.
//!
//! Operations for the repository preparation DAG.
//! These operations run code generation steps to "unwind" the repo.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::process::Command;

/// Operations for the prep tool.
#[derive(Debug, Clone)]
pub enum PrepOp {
    /// Check current state of generated files
    CheckState,
    /// Run codegen (CLI main.rs generation)
    RunCodegen,
    /// Run daggen (graph.rs generation)
    RunDaggen,
    /// Build all targets
    Build,
}

impl Executable for PrepOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            PrepOp::CheckState => execute_check_state(inputs),
            PrepOp::RunCodegen => execute_run_codegen(inputs),
            PrepOp::RunDaggen => execute_run_daggen(inputs),
            PrepOp::Build => execute_build(inputs),
        }
    }
}

/// Check current state of generated files.
fn execute_check_state(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let buck_out = std::path::Path::new("buck-out/gen/bin");
    let needs_codegen = !buck_out.exists() || !buck_out.join("gist/main.rs").exists();
    
    let mut out = HashMap::new();
    out.insert("needs_codegen".to_string(), Value::Bool(needs_codegen));
    out.insert("buck_out_exists".to_string(), Value::Bool(buck_out.exists()));
    Ok(out)
}

/// Run codegen (CLI main.rs generation).
fn execute_run_codegen(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let needs_codegen = inputs
        .get("needs_codegen")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let dry_run = inputs
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    if !needs_codegen {
        let mut out = HashMap::new();
        out.insert("codegen_ran".to_string(), Value::Bool(false));
        out.insert("codegen_skipped".to_string(), Value::Bool(true));
        return Ok(out);
    }
    
    if dry_run {
        let mut out = HashMap::new();
        out.insert("codegen_ran".to_string(), Value::Bool(false));
        out.insert("codegen_skipped".to_string(), Value::Bool(false));
        out.insert("dry_run".to_string(), Value::Bool(true));
        return Ok(out);
    }
    
    let status = Command::new("cargo")
        .args(["run", "-p", "gunbc-codegen", "--release", "--", "codegen"])
        .status()
        .map_err(|e| ExecError::new(format!("Failed to run codegen: {}", e)))?;
    
    if !status.success() {
        return Err(ExecError::new("Codegen failed"));
    }
    
    let mut out = HashMap::new();
    out.insert("codegen_ran".to_string(), Value::Bool(true));
    out.insert("codegen_skipped".to_string(), Value::Bool(false));
    Ok(out)
}

/// Run daggen (graph.rs generation from declarative DAGs).
fn execute_run_daggen(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let dry_run = inputs
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    if dry_run {
        let mut out = HashMap::new();
        out.insert("daggen_ran".to_string(), Value::Bool(false));
        out.insert("dry_run".to_string(), Value::Bool(true));
        return Ok(out);
    }
    
    let status = Command::new("cargo")
        .args(["run", "-p", "gunbc-codegen", "--release", "--", "daggen"])
        .status()
        .map_err(|e| ExecError::new(format!("Failed to run daggen: {}", e)))?;
    
    if !status.success() {
        return Err(ExecError::new("Daggen failed"));
    }
    
    let mut out = HashMap::new();
    out.insert("daggen_ran".to_string(), Value::Bool(true));
    Ok(out)
}

/// Build all targets.
fn execute_build(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let dry_run = inputs
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    if dry_run {
        let mut out = HashMap::new();
        out.insert("build_ran".to_string(), Value::Bool(false));
        out.insert("build_success".to_string(), Value::Bool(true));
        out.insert("dry_run".to_string(), Value::Bool(true));
        return Ok(out);
    }
    
    let status = Command::new("cargo")
        .args(["build", "--all-targets"])
        .status()
        .map_err(|e| ExecError::new(format!("Failed to run build: {}", e)))?;
    
    let mut out = HashMap::new();
    out.insert("build_ran".to_string(), Value::Bool(true));
    out.insert("build_success".to_string(), Value::Bool(status.success()));
    
    if !status.success() {
        return Err(ExecError::new("Build failed"));
    }
    
    Ok(out)
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_test::{CardinalityTestInput, ErrorTestCase, Mockable};

impl Mockable for PrepOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            PrepOp::CheckState => {
                let mut out = HashMap::new();
                out.insert("needs_codegen".to_string(), Value::Bool(false));
                out.insert("buck_out_exists".to_string(), Value::Bool(true));
                out
            }
            PrepOp::RunCodegen => {
                let mut out = HashMap::new();
                out.insert("codegen_ran".to_string(), Value::Bool(true));
                out.insert("codegen_skipped".to_string(), Value::Bool(false));
                out
            }
            PrepOp::RunDaggen => {
                let mut out = HashMap::new();
                out.insert("daggen_ran".to_string(), Value::Bool(true));
                out
            }
            PrepOp::Build => {
                let mut out = HashMap::new();
                out.insert("build_ran".to_string(), Value::Bool(true));
                out.insert("build_success".to_string(), Value::Bool(true));
                out
            }
        }
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        vec![]
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        match self {
            PrepOp::Build => vec![
                ErrorTestCase::new(
                    "build_failure",
                    HashMap::new(),
                    "Build failed",
                ),
            ],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_state() {
        let result = execute_check_state(HashMap::new()).unwrap();
        assert!(result.contains_key("needs_codegen"));
        assert!(result.contains_key("buck_out_exists"));
    }
}
