//! Buck2-specific operations.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes in the DAG.
//! The ops here are PURE (no I/O) - they prepare requests and parse responses.

use gunbc_exec::{
    optional_map_str_str, optional_str, optional_str_list, require_json, require_response,
    ExecError, Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::language::traits::comment::generated_header;
use gunbc_ir::transport::{FileRequest, TransportRequest};
use gunbc_ir::Value;
use gunbc_primitives::data::{ExtractOp, ParseOp};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Buck2-specific operations for use in DAG nodes.
///
/// All operations are PURE - no I/O. I/O happens via TransportOps::Execute nodes.
#[derive(Debug, Clone)]
pub enum Buck2Op {
    // ========================================================================
    // ParseCargoToml chain: PrepareParseCargoToml -> Execute -> ParseCargoTomlResult
    // ========================================================================
    /// Prepare file read request for Cargo.toml (PURE)
    PrepareParseCargoToml,
    /// Parse Cargo.toml response (PURE)
    ParseCargoTomlResult,

    // ========================================================================
    // Pure domain logic
    // ========================================================================
    /// Extract workspace members and dependencies (PURE)
    ExtractDeps,
    /// Generate Buck2 target definitions (PURE)
    GenerateBuckTargets,
}

impl Executable for Buck2Op {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Buck2Op::PrepareParseCargoToml => execute_prepare_parse_cargo_toml(inputs),
            Buck2Op::ParseCargoTomlResult => execute_parse_cargo_toml_result(inputs),
            Buck2Op::ExtractDeps => execute_extract_deps(inputs),
            Buck2Op::GenerateBuckTargets => execute_generate_targets(inputs),
        }
    }
}

// ============================================================================
// PrepareParseCargoToml - PURE (builds TransportRequest)
// ============================================================================

/// Prepare file read request for Cargo.toml (PURE - no I/O).
fn execute_prepare_parse_cargo_toml(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let path = optional_str(&inputs, "cargo_toml_path")
        .unwrap_or("Cargo.toml")
        .to_string();

    let request = TransportRequest::File(FileRequest::read(&path));

    OutputMap::new()
        .request("request", request)
        .str("cargo_toml_path", path)
        .ok()
}

// ============================================================================
// ParseCargoTomlResult - PURE (parses TransportResponse)
// ============================================================================

/// Parse Cargo.toml response (PURE - no I/O).
fn execute_parse_cargo_toml_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = require_response(&inputs, "response")?;

    let path = optional_str(&inputs, "cargo_toml_path").unwrap_or("Cargo.toml");

    let file_resp = response.require_file()?;
    let content = file_resp.content.clone().ok_or_else(|| {
        ExecError::new(format!("file not found or empty: {}", path))
    })?;

    // Use ParseOp::Toml primitive
    let mut parse_inputs = HashMap::new();
    parse_inputs.insert("input".to_string(), Value::Str(content));
    let parse_result = ParseOp::Toml.execute(parse_inputs)?;

    let json = parse_result
        .get("output")
        .cloned()
        .ok_or_else(|| ExecError::new("TOML parse failed"))?;

    OutputMap::new().value("cargo_toml", json).ok()
}

/// Extract dependencies using primitives.
///
/// Decomposition:
/// 1. ExtractOp for "workspace.members"
/// 2. ExtractOp for "workspace.dependencies"
/// 3. ExtractOp for "dependencies"
fn execute_extract_deps(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let cargo_toml = require_json(&inputs, "cargo_toml")?.clone();

    let mut deps: BTreeMap<String, String> = BTreeMap::new();

    // Step 1: Extract workspace.members using ExtractOp
    let mut extract_inputs = HashMap::new();
    extract_inputs.insert("input".to_string(), Value::Json(cargo_toml.clone()));
    extract_inputs.insert("path".to_string(), Value::Str("workspace.members".to_string()));
    let members_result = ExtractOp.execute(extract_inputs)?;

    let members: Vec<String> = match members_result.get("output") {
        Some(val) if val.as_str_list().is_some() => val.as_str_list().unwrap(),
        Some(Value::Json(arr)) if arr.is_array() => arr
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    };

    // Step 2: Extract workspace.dependencies
    let mut extract_inputs = HashMap::new();
    extract_inputs.insert("input".to_string(), Value::Json(cargo_toml.clone()));
    extract_inputs.insert("path".to_string(), Value::Str("workspace.dependencies".to_string()));
    let wdeps_result = ExtractOp.execute(extract_inputs)?;

    if let Some(Value::Json(wdeps)) = wdeps_result.get("output") {
        if let Some(obj) = wdeps.as_object() {
            for (name, spec) in obj {
                let version = extract_version(spec);
                deps.insert(name.clone(), version);
            }
        }
    }

    // Step 3: Extract direct dependencies
    let mut extract_inputs = HashMap::new();
    extract_inputs.insert("input".to_string(), Value::Json(cargo_toml.clone()));
    extract_inputs.insert("path".to_string(), Value::Str("dependencies".to_string()));
    let direct_result = ExtractOp.execute(extract_inputs)?;

    if let Some(Value::Json(direct)) = direct_result.get("output") {
        if let Some(obj) = direct.as_object() {
            for (name, spec) in obj {
                let version = extract_version(spec);
                deps.insert(name.clone(), version);
            }
        }
    }

    OutputMap::new()
        .str_list("members", members)
        .map_str_str("deps", deps)
        .ok()
}

/// Helper to extract version from dependency spec.
fn extract_version(spec: &serde_json::Value) -> String {
    if let Some(v) = spec.as_str() {
        v.to_string()
    } else if let Some(obj) = spec.as_object() {
        obj.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string()
    } else {
        "*".to_string()
    }
}

/// Check if a workspace member is a binary crate.
///
/// Currently assumes all members are library crates. To detect binaries,
/// add an upstream transport node using `FileRequest::exists("src/main.rs")`
/// per member and pass the results as an input port (e.g., `binary_members: StrList`).
fn is_binary_crate(_member_path: &Path) -> bool {
    false
}

/// Generate Buck2 target definitions.
///
/// This operation is more complex and would require:
/// - LoopBuilder pattern to iterate over members
/// - Format primitive for each target
/// - Concat primitive to join results
///
/// For now, this remains a monolithic operation, but demonstrates
/// where the decomposition would happen.
fn execute_generate_targets(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let members = optional_str_list(&inputs, "members").unwrap_or_default();

    let _deps = optional_map_str_str(&inputs, "deps").unwrap_or_default();

    let mut buck_content = String::new();

    // Header using language module's generated_header for consistency
    let buck2_name = gunbc_ir::cargo::name("buck2");
    buck_content.push_str(&generated_header(&buck2_name, &buck2_name, "#"));
    buck_content.push_str("\n\n");
    buck_content.push_str("load(\"@prelude//rust:defs.bzl\", \"rust_binary\", \"rust_library\")\n\n");

    // Generate targets for each member (would be: Loop + Format + Branch)
    for member in &members {
        let crate_name = member
            .strip_prefix("crates/")
            .or_else(|| member.strip_prefix("core/"))
            .or_else(|| member.strip_prefix("lib/"))
            .unwrap_or(member)
            .replace(['-', '/'], "_");

        // Determine if it's a binary or library.
        // See is_binary_crate doc for how to extend with transport-based detection.
        let member_path = Path::new(member);
        let has_main = is_binary_crate(member_path);

        // Generate target (would be: Format with template)
        if has_main {
            buck_content.push_str(&format!(
                r#"rust_binary(
    name = "{}",
    srcs = glob(["{}/**/*.rs"]),
    edition = "2021",
)

"#,
                crate_name, member
            ));
        } else {
            buck_content.push_str(&format!(
                r#"rust_library(
    name = "{}",
    srcs = glob(["{}/**/*.rs"]),
    edition = "2021",
)

"#,
                crate_name, member
            ));
        }
    }

    // Default library if no members
    if members.is_empty() {
        buck_content.push_str(
            r#"rust_library(
    name = "lib",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
)
"#,
        );
    }

    OutputMap::new().str("buck_content", buck_content).ok()
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for Buck2Op {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            Buck2Op::PrepareParseCargoToml => {
                OutputMap::new()
                    .request("request", TransportRequest::File(FileRequest::read("Cargo.toml")))
                    .str("cargo_toml_path", "Cargo.toml")
                    .build()
            }
            Buck2Op::ParseCargoTomlResult => {
                OutputMap::new()
                    .json("cargo_toml", serde_json::json!({
                        "package": { "name": "test-crate" },
                        "workspace": { "members": ["crates/foo"] }
                    }))
                    .build()
            }
            Buck2Op::ExtractDeps => {
                OutputMap::new()
                    .str_list("members", vec!["foo".to_string()])
                    .map_str_str("deps", BTreeMap::new())
                    .build()
            }
            Buck2Op::GenerateBuckTargets => {
                OutputMap::new()
                    .str("buck_content", "# Mock BUCK content\nrust_library(name = \"foo\")")
                    .build()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_deps() {
        let cargo_toml = serde_json::json!({
            "workspace": {
                "members": ["crates/foo", "crates/bar"],
                "dependencies": {
                    "serde": "1.0"
                }
            }
        });

        let mut inputs = HashMap::new();
        inputs.insert("cargo_toml".to_string(), Value::Json(cargo_toml));

        let result = execute_extract_deps(inputs).unwrap();

        match result.get("members").and_then(|v| v.as_str_list()) {
            Some(members) => {
                assert_eq!(members.len(), 2);
                assert!(members.contains(&"crates/foo".to_string()));
            }
            _ => panic!("expected members list"),
        }

        match result.get("deps").and_then(|v| v.as_map_str_str()) {
            Some(deps) => {
                assert_eq!(deps.get("serde"), Some(&"1.0".to_string()));
            }
            _ => panic!("expected deps map"),
        }
    }

    #[test]
    fn test_generate_targets() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "members".to_string(),
            Value::str_list(vec!["crates/foo".to_string()]),
        );
        inputs.insert("deps".to_string(), Value::Map(BTreeMap::new()));

        let result = execute_generate_targets(inputs).unwrap();

        match result.get("buck_content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("rust_library"), "should contain rust_library");
                assert!(content.contains("foo"), "should contain crate name");
                assert!(content.contains("crates/foo"), "should contain path");
            }
            _ => panic!("expected buck content"),
        }
    }
}
