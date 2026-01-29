//! Buck2-specific operations.
//!
//! This module demonstrates how tool-specific operations can be decomposed
//! into primitive operations. The Buck2Op enum provides high-level operations
//! that internally delegate to primitives from gunbc-primitives.
//!
//! # Decomposition Example
//!
//! `ParseCargoToml` is decomposed into:
//! ```text
//! ReadFileOp(path) → ParseOp::Toml(content) → json
//! ```
//!
//! This shows the migration path: tool ops become thin wrappers around primitives.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_primitives::data::{ExtractOp, ParseOp};
use gunbc_primitives::io::ReadFileOp;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Buck2-specific operations for use in DAG nodes.
///
/// These operations are implemented by composing primitives from gunbc-primitives.
/// Eventually, even these wrappers will be generated from DAG definitions.
#[derive(Debug, Clone)]
pub enum Buck2Op {
    /// Parse a Cargo.toml file
    /// Internally: ReadFile → Parse(Toml)
    ParseCargoToml,
    /// Extract workspace members and dependencies
    /// Internally: Multiple Extract operations
    ExtractDeps,
    /// Generate Buck2 target definitions
    /// Internally: Format → Concat for each member
    GenerateBuckTargets,
}

impl Executable for Buck2Op {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Buck2Op::ParseCargoToml => execute_parse_cargo_toml(inputs),
            Buck2Op::ExtractDeps => execute_extract_deps(inputs),
            Buck2Op::GenerateBuckTargets => execute_generate_targets(inputs),
        }
    }
}

/// Parse Cargo.toml file using primitives.
///
/// Decomposition:
/// 1. ReadFileOp reads the file
/// 2. ParseOp::Toml parses the content
fn execute_parse_cargo_toml(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let path = match inputs.get("cargo_toml_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "Cargo.toml".to_string(),
    };

    // Step 1: Use ReadFileOp primitive
    let mut read_inputs = HashMap::new();
    read_inputs.insert("path".to_string(), Value::Str(path.clone()));
    let read_result = ReadFileOp.execute(read_inputs)?;

    let content = read_result
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new(format!("failed to read file: {}", path)))?;

    // Check if file exists
    let exists = read_result
        .get("exists")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !exists {
        return Err(ExecError::new(format!("file not found: {}", path)));
    }

    // Step 2: Use ParseOp::Toml primitive
    let mut parse_inputs = HashMap::new();
    parse_inputs.insert("input".to_string(), Value::Str(content.to_string()));
    let parse_result = ParseOp::Toml.execute(parse_inputs)?;

    let json = parse_result
        .get("output")
        .cloned()
        .ok_or_else(|| ExecError::new("TOML parse failed"))?;

    let mut out = HashMap::new();
    out.insert("cargo_toml".to_string(), json);
    Ok(out)
}

/// Extract dependencies using primitives.
///
/// Decomposition:
/// 1. ExtractOp for "workspace.members"
/// 2. ExtractOp for "workspace.dependencies"
/// 3. ExtractOp for "dependencies"
fn execute_extract_deps(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let cargo_toml = match inputs.get("cargo_toml") {
        Some(Value::Json(j)) => j.clone(),
        _ => return Err(ExecError::new("missing or invalid 'cargo_toml' input")),
    };

    let mut deps: BTreeMap<String, String> = BTreeMap::new();

    // Step 1: Extract workspace.members using ExtractOp
    let mut extract_inputs = HashMap::new();
    extract_inputs.insert("input".to_string(), Value::Json(cargo_toml.clone()));
    extract_inputs.insert("path".to_string(), Value::Str("workspace.members".to_string()));
    let members_result = ExtractOp.execute(extract_inputs)?;

    let members: Vec<String> = match members_result.get("output") {
        Some(Value::StrList(list)) => list.clone(),
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

    let mut out = HashMap::new();
    out.insert("members".to_string(), Value::StrList(members));
    out.insert("deps".to_string(), Value::MapStrStr(deps));
    Ok(out)
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
    let members = match inputs.get("members") {
        Some(Value::StrList(list)) => list.clone(),
        _ => vec![],
    };

    let _deps = match inputs.get("deps") {
        Some(Value::MapStrStr(map)) => map.clone(),
        _ => BTreeMap::new(),
    };

    let mut buck_content = String::new();

    // Header (would be: Format with header template)
    buck_content.push_str("# Generated by gunbc-buck2\n");
    buck_content.push_str("# DO NOT EDIT - regenerate with: gunbc-buck2\n\n");
    buck_content.push_str("load(\"@prelude//rust:defs.bzl\", \"rust_binary\", \"rust_library\")\n\n");

    // Generate targets for each member (would be: Loop + Format + Branch)
    for member in &members {
        let crate_name = member
            .strip_prefix("crates/")
            .or_else(|| member.strip_prefix("core/"))
            .or_else(|| member.strip_prefix("lib/"))
            .unwrap_or(member)
            .replace('-', "_")
            .replace('/', "_");

        // Determine if it's a binary or library (would be: Branch based on file exists)
        let member_path = Path::new(member);
        let has_main = member_path.join("src/main.rs").exists()
            || Path::new(".").join(member).join("src/main.rs").exists();

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

    let mut out = HashMap::new();
    out.insert("buck_content".to_string(), Value::Str(buck_content));
    Ok(out)
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

        match result.get("members") {
            Some(Value::StrList(members)) => {
                assert_eq!(members.len(), 2);
                assert!(members.contains(&"crates/foo".to_string()));
            }
            _ => panic!("expected members list"),
        }

        match result.get("deps") {
            Some(Value::MapStrStr(deps)) => {
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
            Value::StrList(vec!["crates/foo".to_string()]),
        );
        inputs.insert("deps".to_string(), Value::MapStrStr(BTreeMap::new()));

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

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for Buck2Op {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            Buck2Op::ParseCargoToml => {
                let mut out = HashMap::new();
                out.insert("cargo_toml".to_string(), Value::Json(serde_json::json!({
                    "package": { "name": "test-crate" },
                    "workspace": { "members": ["crates/foo"] }
                })));
                out
            }
            Buck2Op::ExtractDeps => {
                let mut out = HashMap::new();
                out.insert("members".to_string(), Value::StrList(vec!["foo".to_string()]));
                out.insert("deps".to_string(), Value::MapStrStr(std::collections::BTreeMap::new()));
                out
            }
            Buck2Op::GenerateBuckTargets => {
                let mut out = HashMap::new();
                out.insert("buck_content".to_string(), Value::Str("# Mock BUCK content\nrust_library(name = \"foo\")".to_string()));
                out
            }
        }
    }
}
