//! Makegen operations.

use gunbc_codegen::FileWriter;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;

use crate::registry::ToolRegistry;
use crate::render::render_makefile;

/// Operations for the makegen tool.
#[derive(Debug, Clone)]
pub enum MakegenOp {
    /// Load the tool registry
    LoadRegistry,
    /// Render Makefile content
    RenderMakefile,
    /// Write Makefile to disk (boundary)
    WriteMakefile,
}

impl Executable for MakegenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            MakegenOp::LoadRegistry => execute_load_registry(inputs),
            MakegenOp::RenderMakefile => execute_render_makefile(inputs),
            MakegenOp::WriteMakefile => execute_write_makefile(inputs),
        }
    }
}

/// Load the default tool registry.
fn execute_load_registry(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let registry = ToolRegistry::default_registry();

    // Serialize tool names for downstream
    let tool_names: Vec<String> = registry.tools.iter().map(|t| t.short_name.clone()).collect();

    let mut out = HashMap::new();
    out.insert("tool_count".to_string(), Value::Int(registry.tools.len() as i64));
    out.insert("tool_names".to_string(), Value::StrList(tool_names));
    // Store registry as JSON for downstream
    let registry_json = serde_json::json!({
        "tools": registry.tools.iter().map(|t| {
            serde_json::json!({
                "crate_name": t.crate_name,
                "short_name": t.short_name,
                "description": t.description,
                "entrypoints": t.entrypoints.iter().map(|e| {
                    serde_json::json!({
                        "port_name": e.port_name,
                        "make_var": e.make_var,
                        "cli_flag": e.cli_flag,
                        "type_hint": e.type_hint,
                        "default": e.default,
                        "repeatable": e.repeatable,
                    })
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>()
    });
    out.insert("registry".to_string(), Value::Json(registry_json));
    Ok(out)
}

/// Render the Makefile content.
fn execute_render_makefile(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // For now, use the default registry directly
    // In a more sophisticated version, we'd deserialize from inputs
    let registry = ToolRegistry::default_registry();
    let content = render_makefile(&registry);

    let mut out = HashMap::new();
    out.insert("makefile_content".to_string(), Value::Str(content));
    Ok(out)
}

/// Write the Makefile to disk.
fn execute_write_makefile(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let content = match inputs.get("makefile_content") {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(ExecError::new("missing or invalid 'makefile_content' input")),
    };

    let output_path = match inputs.get("output_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "Makefile".to_string(),
    };

    let writer = FileWriter::real();
    let result = writer
        .write(&output_path, &content)
        .map_err(|e| ExecError::new(format!("failed to write Makefile: {}", e)))?;

    let mut out = HashMap::new();
    out.insert("written_path".to_string(), Value::Str(result.path));
    out.insert("content".to_string(), Value::Str(content));
    out.insert("changed".to_string(), Value::Bool(result.changed));
    Ok(out)
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_test::{CardinalityTestInput, ErrorTestCase, Mockable};

impl Mockable for MakegenOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            MakegenOp::LoadRegistry => {
                let mut out = HashMap::new();
                out.insert("tool_count".to_string(), Value::Int(3));
                out.insert(
                    "tool_names".to_string(),
                    Value::StrList(vec![
                        "gist".to_string(),
                        "deps".to_string(),
                        "buck2".to_string(),
                    ]),
                );
                out.insert(
                    "registry".to_string(),
                    Value::Json(serde_json::json!({
                        "tools": [
                            {"crate_name": "gunbc-gist", "short_name": "gist"},
                            {"crate_name": "gunbc-deps", "short_name": "deps"},
                            {"crate_name": "gunbc-buck2", "short_name": "buck2"},
                        ]
                    })),
                );
                out
            }
            MakegenOp::RenderMakefile => {
                let mut out = HashMap::new();
                out.insert(
                    "makefile_content".to_string(),
                    Value::Str(
                        r#"# Generated Makefile
.PHONY: gist deps buck2

gist:
	cargo run -p gunbc-gist

deps:
	cargo run -p gunbc-deps

buck2:
	cargo run -p gunbc-buck2
"#
                        .to_string(),
                    ),
                );
                out
            }
            MakegenOp::WriteMakefile => {
                let mut out = HashMap::new();
                out.insert("written_path".to_string(), Value::Str("Makefile".to_string()));
                out.insert(
                    "content".to_string(),
                    Value::Str("# Mock Makefile content".to_string()),
                );
                out.insert("changed".to_string(), Value::Bool(true));
                out
            }
        }
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        // MakegenOp doesn't have list inputs that need cardinality testing
        vec![]
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        match self {
            MakegenOp::LoadRegistry => vec![
                // LoadRegistry doesn't require inputs, so no error cases
            ],
            MakegenOp::RenderMakefile => vec![
                // RenderMakefile doesn't require inputs currently
            ],
            MakegenOp::WriteMakefile => vec![
                ErrorTestCase::new(
                    "missing_makefile_content",
                    HashMap::new(),
                    "missing or invalid 'makefile_content' input",
                ),
                ErrorTestCase::new(
                    "wrong_type_makefile_content",
                    {
                        let mut m = HashMap::new();
                        m.insert("makefile_content".to_string(), Value::Int(123));
                        m
                    },
                    "missing or invalid 'makefile_content' input",
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_registry() {
        let result = execute_load_registry(HashMap::new()).unwrap();

        match result.get("tool_count") {
            Some(Value::Int(n)) => assert!(*n >= 2),
            _ => panic!("expected tool count"),
        }

        match result.get("tool_names") {
            Some(Value::StrList(names)) => {
                assert!(names.contains(&"gist".to_string()));
                assert!(names.contains(&"buck2".to_string()));
            }
            _ => panic!("expected tool names"),
        }
    }

    #[test]
    fn test_render_makefile() {
        let result = execute_render_makefile(HashMap::new()).unwrap();

        match result.get("makefile_content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("gist:"));
                assert!(content.contains("buck2:"));
            }
            _ => panic!("expected makefile content"),
        }
    }
}
