//! Deps operations.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes in the DAG.
//! The ops here are PURE (no I/O) - they prepare requests and parse responses.

use crate::installer::Installer;
use crate::manifest::DepsManifest;
use crate::upsert::upsert_dry_run;
use crate::{strict_dry_run_enabled, Platform};
use gunbc_exec::{
    optional_str_strict, propagate_skipped, require_response, require_str, ExecError, Executable,
    IntoExecResult, OutputMap, TransportResponseExt,
};
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
    // deps.toml Generation (ownership of deps.toml)
    // ========================================================================
    /// Load tool registry (PURE - returns tool definitions as JSON)
    /// Outputs: tool_count, tool_names, tools_json
    LoadToolRegistry,
    /// Render deps.toml content from tool registry (PURE)
    /// Inputs: tools_json: Json
    /// Outputs: deps_toml_content: String
    RenderDepsToml,

    // ========================================================================
    // LoadManifest chain: PrepareLoadManifest -> Execute -> ParseManifest
    // ========================================================================
    /// Prepare file read request for manifest (PURE)
    PrepareLoadManifest,
    /// Parse manifest file response (PURE)
    /// Outputs: dep_count, dep_names, manifest_content (for GenerateScripts)
    ParseManifest,

    // ========================================================================
    // Pure domain logic
    // ========================================================================
    /// Generate install scripts (domain-specific logic, PURE)
    /// Note: This now receives manifest_content as input instead of loading from file
    GenerateScripts,

    // ========================================================================
    // ExecuteInstalls chain (batch): PrepareExecuteInstalls -> Execute -> ParseExecuteResult
    // Note: This batches all installs into one script. For per-dependency
    // observability, use the single-dependency ops below with LoopBuilder.
    // ========================================================================
    /// Prepare shell command for install script (PURE)
    PrepareExecuteInstalls,
    /// Parse execute result (PURE)
    ParseExecuteResult,

    // ========================================================================
    // Single-dependency operations (for LoopBuilder + UpsertBuilder integration)
    // These can be composed into an Upsert pattern for each dependency.
    // ========================================================================
    /// Prepare check if dependency is installed (PURE)
    /// Inputs: dep_info: DependencyInfo
    /// Outputs: request: TransportRequest (verify command), dep_name: String
    PrepareCheckInstalled,
    /// Parse check result (PURE)
    /// Inputs: response: TransportResponse, dep_name: String
    /// Outputs: exists: Bool, dep_name: String
    ParseCheckInstalled,
    /// Prepare install command for dependency (PURE)
    /// Inputs: dep_info: DependencyInfo, exists: Bool
    /// Outputs: request: TransportRequest (install command), dep_name: String
    PrepareInstall,
    /// Parse install result (PURE)
    /// Inputs: response: TransportResponse, dep_name: String
    /// Outputs: installed: Bool, dep_name: String
    ParseInstall,
}

impl Executable for DepsOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // deps.toml generation
            DepsOp::LoadToolRegistry => execute_load_tool_registry(inputs),
            DepsOp::RenderDepsToml => execute_render_deps_toml(inputs),
            // LoadManifest chain
            DepsOp::PrepareLoadManifest => execute_prepare_load_manifest(inputs),
            DepsOp::ParseManifest => execute_parse_manifest(inputs),
            // Pure domain logic
            DepsOp::GenerateScripts => execute_generate_scripts(inputs),
            // ExecuteInstalls chain (batch)
            DepsOp::PrepareExecuteInstalls => execute_prepare_execute_installs(inputs),
            DepsOp::ParseExecuteResult => execute_parse_execute_result(inputs),
            // Single-dependency operations
            DepsOp::PrepareCheckInstalled => execute_prepare_check_installed(inputs),
            DepsOp::ParseCheckInstalled => execute_parse_check_installed(inputs),
            DepsOp::PrepareInstall => execute_prepare_install(inputs),
            DepsOp::ParseInstall => execute_parse_install(inputs),
        }
    }
}

// ============================================================================
// deps.toml Generation - LoadToolRegistry and RenderDepsToml
// ============================================================================

use crate::tool_upsert::generate_deps_toml_from_registry;

/// Load tool registry and render deps.toml (PURE - no I/O).
///
/// Combines loading and rendering into one step since ToolDef isn't serializable.
/// Returns tool metadata and rendered deps.toml content.
fn execute_load_tool_registry(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    use gunbc_ir::transport::tool::default_tool_registry;

    let registry = default_tool_registry();
    let tools: Vec<_> = registry.all().collect();

    let tool_names: Vec<String> = tools.iter().map(|t| t.id.to_string()).collect();
    let tool_count = tools.len() as i64;

    OutputMap::new()
        .int("tool_count", tool_count)
        .str_list("tool_names", tool_names)
        .ok()
}

/// Render deps.toml content from tool registry (PURE - no I/O).
///
/// Uses `generate_deps_toml_from_registry()` to render deps.toml from the
/// default tool registry.
///
/// Outputs: deps_toml_content: String
fn execute_render_deps_toml(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Generate deps.toml content directly from registry
    let content = generate_deps_toml_from_registry();

    OutputMap::new().str("deps_toml_content", content).ok()
}

// ============================================================================
// PrepareLoadManifest - PURE (builds TransportRequest)
// ============================================================================

/// Prepare file read request for manifest (PURE - no I/O).
fn execute_prepare_load_manifest(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let manifest_path = require_str(&inputs, "manifest_path")?;

    let request = TransportRequest::File(FileRequest::read(manifest_path));

    OutputMap::new()
        .request("request", request)
        .str("manifest_path", manifest_path)
        .bool("skip", false)
        .ok()
}

// ============================================================================
// ParseManifest - PURE (parses TransportResponse)
// ============================================================================

/// Parse manifest file response (PURE - no I/O).
///
/// Outputs manifest_content for downstream GenerateScripts (avoiding re-load).
fn execute_parse_manifest(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &[
            "dep_count",
            "dep_names",
            "manifest_path",
            "manifest_content",
        ],
    ) {
        return result;
    }

    let response = require_response(&inputs, "response")?;
    let manifest_path = require_str(&inputs, "manifest_path")?;

    let file_resp = response.require_file()?;
    let content = file_resp.content.clone().ok_or_else(|| {
        ExecError::new(format!(
            "failed to load manifest: file not found: {}",
            manifest_path
        ))
    })?;

    // Use ParseOp::Toml primitive to parse (validates TOML structure)
    let mut parse_inputs = HashMap::new();
    parse_inputs.insert("input".to_string(), Value::Str(content.clone()));
    let _parse_result = ParseOp::Toml.execute(parse_inputs)?;

    // Domain-specific: Use DepsManifest for structured extraction
    let manifest = DepsManifest::parse(&content).exec_context("failed to parse manifest")?;

    let dep_names: Vec<String> = manifest.dependency.iter().map(|d| d.name.clone()).collect();

    OutputMap::new()
        .int("dep_count", manifest.dependency.len() as i64)
        .str_list("dep_names", dep_names)
        .str("manifest_path", manifest_path)
        // Pass manifest content to downstream (avoiding file reload in GenerateScripts)
        .str("manifest_content", content)
        .ok()
}

/// Generate install scripts for all dependencies (PURE - no I/O).
///
/// This function is now truly pure - it receives manifest_content as input
/// instead of loading from file. The manifest was already loaded via the
/// PrepareLoadManifest -> Execute -> ParseManifest chain.
fn execute_generate_scripts(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "manifest_content",
        &[
            "install_script",
            "already_installed",
            "needs_install",
            "platform",
        ],
    ) {
        return result;
    }
    if let Some(result) = propagate_skipped(
        &inputs,
        "res:platform",
        &[
            "install_script",
            "already_installed",
            "needs_install",
            "platform",
        ],
    ) {
        return result;
    }

    // Get manifest content from upstream (passed through graph, not file I/O)
    let manifest_content = require_str(&inputs, "manifest_content")?;

    // Parse the manifest content (no file I/O)
    let manifest =
        DepsManifest::parse(manifest_content).exec_context("failed to parse manifest")?;

    // Use platform from DAG input (acquired at boundary)
    let platform_str = require_str(&inputs, "res:platform")?;
    let platform = Platform::parse(platform_str).map_err(ExecError::new)?;
    if strict_dry_run_enabled() && platform == Platform::Unknown {
        return Err(ExecError::new(
            "strict dry-run requires explicit platform wiring/mocks; refusing Platform::Unknown",
        ));
    }
    let installer = Installer::for_platform(platform);
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

    OutputMap::new()
        .str("install_script", combined_script)
        .str_list("already_installed", already_installed)
        .str_list("needs_install", needs_install)
        .str("platform", installer.platform().name().to_string())
        .ok()
}

// ============================================================================
// PrepareExecuteInstalls - PURE (builds TransportRequest)
// ============================================================================

/// Prepare shell command for install script (PURE - no I/O).
fn execute_prepare_execute_installs(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if propagate_skipped(&inputs, "install_script", &["request", "script"]).is_some() {
        return OutputMap::new().bool("skip", true).ok();
    }

    let script = require_str(&inputs, "install_script")?;

    let request = ShellRequest::new("sh")
        .args(["-c", script])
        .into_transport_request();

    OutputMap::new()
        .request("request", request)
        .str("script", script)
        .bool("skip", false)
        .ok()
}

// ============================================================================
// ParseExecuteResult - PURE (parses TransportResponse)
// ============================================================================

/// Parse execute result (PURE - no I/O).
fn execute_parse_execute_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &["executed", "success", "script", "stdout", "stderr"],
    ) {
        return result;
    }

    let response = require_response(&inputs, "response")?;
    let script = optional_str_strict(&inputs, "script")?.unwrap_or("");

    let shell = response.require_shell()?;
    let (success, stdout, stderr) = (shell.success(), shell.stdout.clone(), shell.stderr.clone());

    OutputMap::new()
        .bool("executed", true)
        .bool("success", success)
        .str("script", script)
        .str("stdout", stdout)
        .str("stderr", stderr)
        .ok()
}

// ============================================================================
// Single-dependency operations (for LoopBuilder + UpsertBuilder integration)
// ============================================================================

/// Prepare check if dependency is installed (PURE - no I/O).
///
/// This is part of the per-dependency Upsert pattern: Check -> Install -> Verify.
/// Use with UpsertBuilder for idempotent dependency installation.
///
/// Inputs:
/// - dep_name: String (the dependency name)
/// - verify_cmd: String (command to verify installation)
///
/// Outputs:
/// - request: TransportRequest (shell command for verify)
/// - dep_name: String (pass through for correlation)
fn execute_prepare_check_installed(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let dep_name = require_str(&inputs, "dep_name")?;
    let verify_cmd = require_str(&inputs, "verify_cmd")?;

    let request = ShellRequest::new("sh")
        .args(["-c", verify_cmd])
        .into_transport_request();

    OutputMap::new()
        .request("request", request)
        .str("dep_name", dep_name)
        .ok()
}

/// Parse check result (PURE - no I/O).
///
/// Determines if a dependency is installed based on verify command exit code.
///
/// Inputs:
/// - response: TransportResponse (from verify command)
/// - dep_name: String (for correlation)
///
/// Outputs:
/// - exists: Bool (true if installed)
/// - dep_name: String (pass through)
fn execute_parse_check_installed(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = require_response(&inputs, "response")?;
    let dep_name = optional_str_strict(&inputs, "dep_name")?.unwrap_or("unknown");

    let exists = match response {
        TransportResponse::Shell(shell) => shell.success(),
        _ => false,
    };

    OutputMap::new()
        .bool("exists", exists)
        .str("dep_name", dep_name)
        .ok()
}

/// Prepare install command for dependency (PURE - no I/O).
///
/// Part of the Upsert pattern. Guarded by exists == false.
///
/// Inputs:
/// - dep_name: String
/// - install_cmd: String (command to install)
/// - exists: Bool (guard - only install if false)
///
/// Outputs:
/// - request: TransportRequest (shell command for install)
/// - dep_name: String (pass through)
fn execute_prepare_install(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let dep_name = require_str(&inputs, "dep_name")?;
    let install_cmd = require_str(&inputs, "install_cmd")?;

    let request = ShellRequest::new("sh")
        .args(["-c", install_cmd])
        .into_transport_request();

    OutputMap::new()
        .request("request", request)
        .str("dep_name", dep_name)
        .ok()
}

/// Parse install result (PURE - no I/O).
///
/// Parses the result of an install command.
///
/// Inputs:
/// - response: TransportResponse (from install command)
/// - dep_name: String (for correlation)
///
/// Outputs:
/// - installed: Bool (true if install succeeded)
/// - dep_name: String (pass through)
fn execute_parse_install(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = require_response(&inputs, "response")?;
    let dep_name = optional_str_strict(&inputs, "dep_name")?.unwrap_or("unknown");

    let installed = match response {
        TransportResponse::Shell(shell) => shell.success(),
        _ => false,
    };

    OutputMap::new()
        .bool("installed", installed)
        .str("dep_name", dep_name)
        .ok()
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_test::{CardinalityTestInput, ErrorTestCase, Mockable};

impl Mockable for DepsOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            // deps.toml generation ops
            DepsOp::LoadToolRegistry => {
                OutputMap::new()
                    .int("tool_count", 3)
                    .str_list(
                        "tool_names",
                        vec![
                            "cargo".to_string(),
                            "gh".to_string(),
                            "git".to_string(),
                        ],
                    )
                    .build()
            }
            DepsOp::RenderDepsToml => {
                OutputMap::new()
                    .str(
                        "deps_toml_content",
                        "# Generated deps.toml\n[[dependency]]\nname = \"mock\"\nverify = \"mock --version\"\n",
                    )
                    .build()
            }
            DepsOp::PrepareLoadManifest => {
                OutputMap::new()
                    .request(
                        "request",
                        TransportRequest::File(FileRequest::read("deps.toml")),
                    )
                    .str("manifest_path", "deps.toml")
                    .build()
            }
            DepsOp::ParseManifest => {
                OutputMap::new()
                    .int("dep_count", 2)
                    .str_list(
                        "dep_names",
                        vec!["rust".to_string(), "git".to_string()],
                    )
                    .str("manifest_path", "deps.toml")
                    .str("manifest_content", "[[dependency]]\nname = \"mock\"")
                    .build()
            }
            DepsOp::GenerateScripts => {
                OutputMap::new()
                    .str(
                        "install_script",
                        r#"#!/bin/bash
# Install script for mock deps
echo "Installing rust..."
echo "Installing git..."
"#,
                    )
                    .str_list(
                        "already_installed",
                        vec!["git".to_string()],
                    )
                    .str_list(
                        "needs_install",
                        vec!["rust".to_string()],
                    )
                    .str("platform", "linux")
                    .build()
            }
            DepsOp::PrepareExecuteInstalls => {
                OutputMap::new()
                    .request(
                        "request",
                        ShellRequest::new("sh")
                            .args(["-c", "echo mock"])
                            .into_transport_request(),
                    )
                    .str("script", "echo mock")
                    .build()
            }
            DepsOp::ParseExecuteResult => {
                OutputMap::new()
                    .bool("executed", true)
                    .bool("success", true)
                    .str("script", "echo 'mock install'")
                    .str("stdout", "mock install\n")
                    .str("stderr", "")
                    .build()
            }
            // Single-dependency operations
            DepsOp::PrepareCheckInstalled => {
                OutputMap::new()
                    .request(
                        "request",
                        ShellRequest::new("sh")
                            .args(["-c", "which mock-dep"])
                            .into_transport_request(),
                    )
                    .str("dep_name", "mock-dep")
                    .build()
            }
            DepsOp::ParseCheckInstalled => {
                OutputMap::new()
                    .bool("exists", true)
                    .str("dep_name", "mock-dep")
                    .build()
            }
            DepsOp::PrepareInstall => {
                OutputMap::new()
                    .request(
                        "request",
                        ShellRequest::new("sh")
                            .args(["-c", "echo 'installing mock-dep'"])
                            .into_transport_request(),
                    )
                    .str("dep_name", "mock-dep")
                    .build()
            }
            DepsOp::ParseInstall => {
                OutputMap::new()
                    .bool("installed", true)
                    .str("dep_name", "mock-dep")
                    .build()
            }
        }
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        match self {
            // deps.toml generation ops
            DepsOp::LoadToolRegistry => vec![],
            DepsOp::RenderDepsToml => vec![],
            DepsOp::PrepareLoadManifest => vec![],
            DepsOp::ParseManifest => vec![],
            DepsOp::GenerateScripts => vec![
                CardinalityTestInput::succeeds("dep_names", 0, Value::str_list(vec![])),
                CardinalityTestInput::succeeds(
                    "dep_names",
                    1,
                    Value::str_list(vec!["single-dep".to_string()]),
                ),
                CardinalityTestInput::succeeds(
                    "dep_names",
                    3,
                    Value::str_list(vec![
                        "dep1".to_string(),
                        "dep2".to_string(),
                        "dep3".to_string(),
                    ]),
                ),
            ],
            DepsOp::PrepareExecuteInstalls => vec![],
            DepsOp::ParseExecuteResult => vec![],
            // Single-dependency ops - no special cardinality tests
            DepsOp::PrepareCheckInstalled => vec![],
            DepsOp::ParseCheckInstalled => vec![],
            DepsOp::PrepareInstall => vec![],
            DepsOp::ParseInstall => vec![],
        }
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        match self {
            // deps.toml generation ops
            DepsOp::LoadToolRegistry => vec![],
            DepsOp::RenderDepsToml => vec![],
            DepsOp::PrepareLoadManifest => vec![],
            DepsOp::ParseManifest => vec![],
            DepsOp::GenerateScripts => vec![
                ErrorTestCase::new(
                    "missing_manifest_content",
                    HashMap::new(), // No manifest_content provided
                    "missing or invalid 'manifest_content' input",
                ),
                ErrorTestCase::new(
                    "missing_platform",
                    HashMap::from([(
                        "manifest_content".to_string(),
                        Value::Str("[[dependency]]\nname = \"mock\"".to_string()),
                    )]),
                    "missing 'res:platform' input",
                ),
            ],
            DepsOp::PrepareExecuteInstalls => vec![ErrorTestCase::new(
                "missing_install_script",
                HashMap::new(),
                "missing or invalid 'install_script' input",
            )],
            DepsOp::ParseExecuteResult => vec![],
            // Single-dependency ops
            DepsOp::PrepareCheckInstalled => vec![ErrorTestCase::new(
                "missing_dep_name",
                HashMap::new(),
                "missing or invalid 'dep_name' input",
            )],
            DepsOp::ParseCheckInstalled => vec![],
            DepsOp::PrepareInstall => vec![ErrorTestCase::new(
                "missing_dep_name",
                HashMap::new(),
                "missing or invalid 'dep_name' input",
            )],
            DepsOp::ParseInstall => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::STRICT_DRY_RUN_ENV;
    use crate::test_support::with_env_lock;

    #[test]
    fn test_generate_scripts_with_manifest_content() {
        // Now tests the pure function that receives content, not a path
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

        let mut inputs = HashMap::new();
        inputs.insert(
            "manifest_content".to_string(),
            Value::Str(manifest_content.to_string()),
        );
        inputs.insert("res:platform".to_string(), Value::Str("linux".to_string()));

        let result = execute_generate_scripts(inputs).unwrap();

        // Dry-run does not check install state; expect echo in needs_install
        match result.get("needs_install").and_then(|v| v.as_str_list()) {
            Some(list) => {
                assert!(list.contains(&"echo".to_string()));
            }
            _ => panic!("expected needs_install list"),
        }
    }

    #[test]
    fn test_generate_scripts_missing_content() {
        // Test that missing manifest_content fails with appropriate error
        let mut inputs = HashMap::new();
        inputs.insert("res:platform".to_string(), Value::Str("linux".to_string()));
        let result = execute_generate_scripts(inputs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("manifest_content"));
    }

    #[test]
    fn test_generate_scripts_strict_dry_run_rejects_unknown_platform() {
        with_env_lock(|| {
            std::env::set_var(STRICT_DRY_RUN_ENV, "true");

            let manifest_content = r#"
[[dependency]]
name = "echo"
verify = "echo test"
"#;

            let mut inputs = HashMap::new();
            inputs.insert(
                "manifest_content".to_string(),
                Value::Str(manifest_content.to_string()),
            );
            inputs.insert(
                "res:platform".to_string(),
                Value::Str("unknown".to_string()),
            );

            let err = execute_generate_scripts(inputs).expect_err("strict mode should fail");
            assert!(err.to_string().contains("unknown os: unknown"));
        });
    }
}
