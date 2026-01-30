//! Bootstrap operations.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes in the DAG.
//! The ops here are PURE (no I/O) - they prepare requests and parse responses.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{ShellRequest, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Operations for the bootstrap tool.
///
/// All operations are PURE - no I/O. I/O happens via TransportOps::Execute nodes.
#[derive(Debug, Clone)]
pub enum BootstrapOp {
    // ========================================================================
    // ScanWorkspace chain: PrepareScan -> Execute -> ParseScanResult
    // ========================================================================
    /// Prepare workspace scan request (PURE)
    PrepareScanWorkspace,
    /// Parse scan result (PURE)
    ParseScanResult,

    // ========================================================================
    // Pure domain logic
    // ========================================================================
    /// Generate Makefile content (PURE)
    GenerateMakefile,
    /// Generate .gitignore content (PURE)
    GenerateGitignore,
}

impl Executable for BootstrapOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BootstrapOp::PrepareScanWorkspace => execute_prepare_scan_workspace(inputs),
            BootstrapOp::ParseScanResult => execute_parse_scan_result(inputs),
            BootstrapOp::GenerateMakefile => execute_generate_makefile(inputs),
            BootstrapOp::GenerateGitignore => execute_generate_gitignore(inputs),
        }
    }
}

// ============================================================================
// PrepareScanWorkspace - PURE (builds TransportRequest)
// ============================================================================

/// Prepare workspace scan request (PURE - no I/O).
fn execute_prepare_scan_workspace(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Use find to list directories in crates/
    let request = TransportRequest::Shell(ShellRequest {
        command: "find".to_string(),
        args: vec![
            "crates".to_string(),
            "-maxdepth".to_string(),
            "1".to_string(),
            "-mindepth".to_string(),
            "1".to_string(),
            "-type".to_string(),
            "d".to_string(),
        ],
        cwd: None,
        env: HashMap::new(),
        stdin: None,
    });

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    Ok(out)
}

// ============================================================================
// ParseScanResult - PURE (parses TransportResponse)
// ============================================================================

/// Parse scan result (PURE - no I/O).
fn execute_parse_scan_result(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing or invalid 'response' input"))?;

    let mut crate_names = Vec::new();

    if let TransportResponse::Shell(shell) = response {
        if shell.success() {
            for line in shell.stdout.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    // Extract crate name from path like "crates/foo"
                    if let Some(name) = line.strip_prefix("crates/") {
                        if !name.is_empty() && !name.contains('/') {
                            crate_names.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    crate_names.sort();

    let mut out = HashMap::new();
    out.insert("crate_count".to_string(), Value::Int(crate_names.len() as i64));
    out.insert("crate_names".to_string(), Value::StrList(crate_names));
    Ok(out)
}

/// Generate Makefile content using the makegen renderer.
///
/// Uses `ToolRegistry::default_registry()` and `render_makefile()` from gunbc-makegen
/// to generate a complete Makefile with:
/// - Dev UX convention: `<target>` verifies, `<target>-fix` auto-fixes
/// - All registered tools with their entrypoint parameters
/// - Meta targets (test, check, fmt, clippy) with variants
fn execute_generate_makefile(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    use gunbc_makegen::{render_makefile, ToolRegistry};

    let registry = ToolRegistry::default_registry();
    let makefile = render_makefile(&registry);

    let mut out = HashMap::new();
    out.insert("makefile_content".to_string(), Value::Str(makefile));
    Ok(out)
}

/// Generate .gitignore content using the makegen renderer.
///
/// Uses `render_gitignore()` from gunbc-makegen to generate a .gitignore with:
/// - Patterns derived from `BuildConfig.build_system` (Cargo, Buck2, etc.)
/// - Section comments showing provenance (from the-gunbai pattern)
/// - Universal categories (editor, OS, secrets, generators)
fn execute_generate_gitignore(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    use gunbc_makegen::{default_build_config, render_gitignore};

    let config = default_build_config();
    let gitignore = render_gitignore(&config);

    let mut out = HashMap::new();
    out.insert("gitignore_content".to_string(), Value::Str(gitignore));
    Ok(out)
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for BootstrapOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            BootstrapOp::PrepareScanWorkspace => {
                let mut out = HashMap::new();
                out.insert("request".to_string(), Value::Request(TransportRequest::Shell(ShellRequest {
                    command: "find".to_string(),
                    args: vec!["crates".to_string()],
                    cwd: None,
                    env: HashMap::new(),
                    stdin: None,
                })));
                out
            }
            BootstrapOp::ParseScanResult => {
                let mut out = HashMap::new();
                out.insert("crate_count".to_string(), Value::Int(5));
                out.insert("crate_names".to_string(), Value::StrList(vec!["lib-a".to_string(), "lib-b".to_string()]));
                out
            }
            BootstrapOp::GenerateMakefile => {
                let mut out = HashMap::new();
                out.insert("makefile_content".to_string(), Value::Str("# Mock Makefile".to_string()));
                out
            }
            BootstrapOp::GenerateGitignore => {
                let mut out = HashMap::new();
                out.insert("gitignore_content".to_string(), Value::Str("# Mock .gitignore\n/target/".to_string()));
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_makefile() {
        let result = execute_generate_makefile(HashMap::new()).unwrap();

        match result.get("makefile_content") {
            Some(Value::Str(content)) => {
                // Should have generated header
                assert!(content.contains("Generated by gunbc-makegen"));
                // Should have dev UX convention
                assert!(content.contains("Naming convention"));
                assert!(content.contains("<target>-fix"));
                // Should have build targets
                assert!(content.contains("build:"));
                assert!(content.contains("test:"));
                // Should have tool targets
                assert!(content.contains("gist:"));
                // Should have fix variants
                assert!(content.contains("test-fix:"));
                assert!(content.contains("clippy-fix:"));
            }
            _ => panic!("expected makefile content"),
        }
    }

    #[test]
    fn test_generate_gitignore() {
        let result = execute_generate_gitignore(HashMap::new()).unwrap();

        match result.get("gitignore_content") {
            Some(Value::Str(content)) => {
                // Should have generated header
                assert!(content.contains("Generated by gunbc-makegen"));
                // Should have provenance comments (from the-gunbai pattern)
                assert!(content.contains("(from cargo)"));
                assert!(content.contains("(from editor)"));
                // Should have patterns
                assert!(content.contains("/target/"));
                assert!(content.contains(".DS_Store"));
                assert!(content.contains(".env"));
            }
            _ => panic!("expected gitignore content"),
        }
    }
}
