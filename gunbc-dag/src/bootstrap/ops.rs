//! Bootstrap operations.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes in the DAG.
//! The ops here are PURE (no I/O) - they prepare requests and parse responses.

use gunbc_exec::{
    optional_str_list_strict, propagate_skipped, require_response, ExecError, Executable, OutputMap,
};
use gunbc_ir::transport::{ShellRequest, TransportResponse};
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
fn execute_prepare_scan_workspace(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Use find to list directories in crates/
    let request = ShellRequest::new("find")
        .args(["crates", "-maxdepth", "1", "-mindepth", "1", "-type", "d"])
        .into_transport_request();

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

// ============================================================================
// ParseScanResult - PURE (parses TransportResponse)
// ============================================================================

/// Parse scan result (PURE - no I/O).
fn execute_parse_scan_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "response", &["crate_count", "crate_names"]) {
        return result;
    }

    let response = require_response(&inputs, "response")?;

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

    OutputMap::new()
        .int("crate_count", crate_names.len() as i64)
        .str_list("crate_names", crate_names)
        .ok()
}

/// Generate Makefile content using the makegen renderer.
///
/// Uses `ToolRegistry::default_registry()` and `render_makefile()` from gunbc-makegen
/// to generate a complete Makefile with:
/// - Dev UX convention: `<target>` verifies, `<target>-fix` auto-fixes
/// - All registered tools with their entrypoint parameters
/// - Meta targets (test, check, fmt, clippy) with variants
fn execute_generate_makefile(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let _ = optional_str_list_strict(&inputs, "crate_names")?;
    use crate::makegen::{registry::ToolRegistry, render::render_makefile};

    let registry = ToolRegistry::default_registry();
    let makefile = render_makefile(&registry);

    OutputMap::new()
        .str("makefile_content", makefile.clone())
        .str("return", makefile)
        .ok()
}

/// Generate .gitignore content using the makegen renderer.
///
/// Uses `render_gitignore()` from gunbc-makegen to generate a .gitignore with:
/// - Patterns derived from `BuildConfig.build_system` (Cargo, Buck2, etc.)
/// - Section comments showing provenance (from the-gunbai pattern)
/// - Universal categories (editor, OS, secrets, generators)
fn execute_generate_gitignore(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let _ = optional_str_list_strict(&inputs, "crate_names")?;
    use crate::makegen::{gitignore::render_gitignore, registry::default_build_config};

    let config = default_build_config();
    let gitignore = render_gitignore(&config);

    OutputMap::new()
        .str("gitignore_content", gitignore.clone())
        .str("return", gitignore)
        .ok()
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for BootstrapOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            BootstrapOp::PrepareScanWorkspace => OutputMap::new()
                .request(
                    "request",
                    ShellRequest::new("find")
                        .arg("crates")
                        .into_transport_request(),
                )
                .build(),
            BootstrapOp::ParseScanResult => OutputMap::new()
                .int("crate_count", 5)
                .str_list(
                    "crate_names",
                    vec!["lib-a".to_string(), "lib-b".to_string()],
                )
                .build(),
            BootstrapOp::GenerateMakefile => OutputMap::new()
                .str("makefile_content", "# Mock Makefile")
                .build(),
            BootstrapOp::GenerateGitignore => OutputMap::new()
                .str("gitignore_content", "# Mock .gitignore\n/target/")
                .build(),
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
                assert!(content.contains("Generated by gunbc-bootstrap"));
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
