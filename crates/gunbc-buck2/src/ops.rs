//! Buck2 generation operations.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, FileResponse, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use gunbc_transport::execute_transport;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

/// Operations for the Buck2 tool.
#[derive(Debug, Clone)]
pub enum Buck2Op {
    /// Parse a Cargo.toml file
    ParseCargoToml,
    /// Extract workspace members and dependencies
    ExtractDeps,
    /// Generate Buck2 target definitions
    GenerateBuckTargets,
    /// Prepare a file write request (PURE - no I/O)
    PrepareFileWrite,
    /// Execute a transport request (BOUNDARY - world write)
    ExecuteTransport,
}

impl Executable for Buck2Op {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Buck2Op::ParseCargoToml => execute_parse_cargo_toml(inputs),
            Buck2Op::ExtractDeps => execute_extract_deps(inputs),
            Buck2Op::GenerateBuckTargets => execute_generate_targets(inputs),
            Buck2Op::PrepareFileWrite => execute_prepare_file_write(inputs),
            Buck2Op::ExecuteTransport => execute_transport_op(inputs),
        }
    }
}

/// Parse Cargo.toml file.
fn execute_parse_cargo_toml(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let path = match inputs.get("cargo_toml_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "Cargo.toml".to_string(),
    };

    let content = fs::read_to_string(&path)
        .map_err(|e| ExecError::new(format!("failed to read {}: {}", path, e)))?;

    let parsed: toml::Value = content
        .parse()
        .map_err(|e| ExecError::new(format!("failed to parse TOML: {}", e)))?;

    // Convert to JSON for storage
    let json = serde_json::to_value(&parsed)
        .map_err(|e| ExecError::new(format!("failed to convert TOML to JSON: {}", e)))?;

    let mut out = HashMap::new();
    out.insert("cargo_toml".to_string(), Value::Json(json));
    Ok(out)
}

/// Extract dependencies from parsed Cargo.toml.
fn execute_extract_deps(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let cargo_toml = match inputs.get("cargo_toml") {
        Some(Value::Json(j)) => j.clone(),
        _ => return Err(ExecError::new("missing or invalid 'cargo_toml' input")),
    };

    let mut deps: BTreeMap<String, String> = BTreeMap::new();

    // Extract workspace members
    let mut members: Vec<String> = Vec::new();
    if let Some(workspace) = cargo_toml.get("workspace") {
        if let Some(mems) = workspace.get("members") {
            if let Some(arr) = mems.as_array() {
                for m in arr {
                    if let Some(s) = m.as_str() {
                        members.push(s.to_string());
                    }
                }
            }
        }
    }

    // Extract dependencies from workspace if present
    if let Some(workspace) = cargo_toml.get("workspace") {
        if let Some(wdeps) = workspace.get("dependencies") {
            if let Some(obj) = wdeps.as_object() {
                for (name, spec) in obj {
                    let version = if let Some(v) = spec.as_str() {
                        v.to_string()
                    } else if let Some(obj) = spec.as_object() {
                        obj.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("*")
                            .to_string()
                    } else {
                        "*".to_string()
                    };
                    deps.insert(name.clone(), version);
                }
            }
        }
    }

    // Extract direct dependencies
    if let Some(direct_deps) = cargo_toml.get("dependencies") {
        if let Some(obj) = direct_deps.as_object() {
            for (name, spec) in obj {
                let version = if let Some(v) = spec.as_str() {
                    v.to_string()
                } else if let Some(obj) = spec.as_object() {
                    obj.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string()
                } else {
                    "*".to_string()
                };
                deps.insert(name.clone(), version);
            }
        }
    }

    let mut out = HashMap::new();
    out.insert("members".to_string(), Value::StrList(members));
    out.insert("deps".to_string(), Value::MapStrStr(deps));
    Ok(out)
}

/// Generate Buck2 target definitions.
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

    // Header
    buck_content.push_str("# Generated by gunbc-buck2\n");
    buck_content.push_str("# DO NOT EDIT - regenerate with: gunbc-buck2\n\n");

    // Load statement
    buck_content.push_str("load(\"@prelude//rust:defs.bzl\", \"rust_binary\", \"rust_library\")\n\n");

    // Generate targets for workspace members
    for member in &members {
        let crate_name = member
            .strip_prefix("crates/")
            .unwrap_or(member)
            .replace('-', "_");

        // Determine if it's a binary or library
        let member_path = Path::new(member);
        let has_main = member_path.join("src/main.rs").exists()
            || Path::new(".").join(member).join("src/main.rs").exists();

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

    // If no members, generate a simple library target
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

/// Prepare a file write request (PURE - just builds the request, no I/O).
fn execute_prepare_file_write(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let content = match inputs.get("buck_content") {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(ExecError::new("missing or invalid 'buck_content' input")),
    };

    let output_path = match inputs.get("output_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "BUCK".to_string(),
    };

    // Build the file write request
    let request = TransportRequest::File(FileRequest::write(&output_path, content));

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    Ok(out)
}

/// Execute a transport request (BOUNDARY - world write).
fn execute_transport_op(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let request = match inputs.get("request") {
        Some(Value::Request(r)) => r.clone(),
        _ => return Err(ExecError::new("missing or invalid 'request' input")),
    };

    let response = execute_transport(&request)
        .map_err(|e| ExecError::new(format!("transport error: {}", e)))?;

    // Extract path from response
    let (written_path, content) = match &response {
        TransportResponse::File(FileResponse { path, content, .. }) => {
            (path.clone(), content.clone().unwrap_or_default())
        }
        _ => ("unknown".to_string(), String::new()),
    };

    let mut out = HashMap::new();
    out.insert("response".to_string(), Value::Response(response));
    out.insert("written_path".to_string(), Value::Str(written_path));
    out.insert("content".to_string(), Value::Str(content));
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

    #[test]
    fn test_prepare_file_write() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "buck_content".to_string(),
            Value::Str("# BUCK content".to_string()),
        );
        inputs.insert("output_path".to_string(), Value::Str("BUCK".to_string()));

        let result = execute_prepare_file_write(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::File(req))) => {
                assert_eq!(req.path, "BUCK");
            }
            _ => panic!("expected file request"),
        }
    }
}
