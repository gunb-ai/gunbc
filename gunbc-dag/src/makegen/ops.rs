//! Makegen operations.
//!
//! This module contains pure makegen-specific operations.
//! File writing is handled via the transport pattern:
//! - `PrepareFileWriteOp` (primitive) creates the TransportRequest
//! - `TransportOps::Execute` (boundary) performs actual I/O

use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::transport::{FileOp, TransportResponse};
use gunbc_ir::Value;
use gunbc_testgen_registry::iter_dag_specs;
use std::collections::HashMap;

use crate::makegen::registry::ToolRegistry;
use crate::makegen::render::render_makefile;

/// Operations for the makegen tool.
///
/// These are all pure operations - no direct I/O.
/// I/O is handled via `PrepareFileWriteOp` + `TransportOps::Execute`.
#[derive(Debug, Clone)]
pub enum MakegenOp {
    /// Load the tool registry (pure - reads static configuration)
    LoadRegistry,
    /// Render Makefile content (pure - string generation)
    RenderMakefile,
    /// Entrypoint: check if makegen wrote output (inspects __deps)
    Entrypoint,
}

impl Executable for MakegenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            MakegenOp::LoadRegistry => execute_load_registry(inputs),
            MakegenOp::RenderMakefile => execute_render_makefile(inputs),
            MakegenOp::Entrypoint => execute_entrypoint(inputs),
        }
    }
}

/// Load the default tool registry.
fn execute_load_registry(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let registry = ToolRegistry::default_registry();

    // Serialize tool names for downstream
    let tool_names: Vec<String> = registry
        .tools
        .iter()
        .map(|t| t.short_name.clone())
        .collect();

    let testgen_targets: Vec<serde_json::Value> = iter_dag_specs()
        .map(|spec| {
            serde_json::json!({
                "name": spec.name,
                "origin_crate": spec.origin_crate,
                "output_path": spec.meta.output_path,
                "module_name": spec.meta.module_name,
                "tool_name": spec.meta.tool_name,
            })
        })
        .collect();

    // Store registry as JSON for downstream
    let registry_json = serde_json::json!({
        "tools": registry.tools.iter().map(|t| {
            serde_json::json!({
                "binary_name": t.binary_name(),
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
        }).collect::<Vec<_>>(),
        "testgen_targets": testgen_targets,
        "testgen_target_count": testgen_targets.len(),
    });

    OutputMap::new()
        .int("tool_count", registry.tools.len() as i64)
        .str_list("tool_names", tool_names)
        .json("registry", registry_json)
        .ok()
}

/// Render the Makefile content.
fn execute_render_makefile(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // For now, use the default registry directly
    // In a more sophisticated version, we'd deserialize from inputs
    let registry = ToolRegistry::default_registry();
    let content = render_makefile(&registry);

    OutputMap::new().str("return", content).ok()
}

/// Check if the makegen transport wrote successfully.
fn execute_entrypoint(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let written = inputs
        .get("__deps")
        .and_then(Value::as_list)
        .map(|deps| {
            deps.iter().any(|value| {
                matches!(
                    value,
                    Value::Response(TransportResponse::File(response))
                        if response.operation == FileOp::Write && response.success
                )
            })
        })
        .unwrap_or(false);
    OutputMap::new().bool("written", written).ok()
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_ir::{cargo, CargoInvocation};
use gunbc_test::{CardinalityTestInput, ErrorTestCase, Mockable};

impl Mockable for MakegenOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            MakegenOp::Entrypoint => OutputMap::new().bool("written", true).build(),
            MakegenOp::LoadRegistry => {
                let testgen_targets: Vec<serde_json::Value> = iter_dag_specs()
                    .map(|spec| {
                        serde_json::json!({
                            "name": spec.name,
                            "origin_crate": spec.origin_crate,
                            "output_path": spec.meta.output_path,
                            "module_name": spec.meta.module_name,
                            "tool_name": spec.meta.tool_name,
                        })
                    })
                    .collect();

                OutputMap::new()
                    .int("tool_count", 3)
                    .str_list(
                        "tool_names",
                        vec!["gist".to_string(), "deps".to_string(), "buck2".to_string()],
                    )
                    .json(
                        "registry",
                        serde_json::json!({
                            "tools": [
                                {"binary_name": cargo::name("gist"), "short_name": "gist"},
                                {"binary_name": cargo::name("deps"), "short_name": "deps"},
                                {"binary_name": cargo::name("buck2"), "short_name": "buck2"},
                            ],
                            "testgen_targets": testgen_targets,
                            "testgen_target_count": testgen_targets.len(),
                        }),
                    )
                    .build()
            }
            MakegenOp::RenderMakefile => {
                let gist = CargoInvocation::standalone("gist").command();
                let deps = CargoInvocation::standalone("deps").command();
                let buck2 = CargoInvocation::standalone("buck2").command();
                OutputMap::new()
                    .str(
                        "return",
                        format!(
                            "# Generated Makefile\n\
                            .PHONY: gist deps buck2\n\
                            \n\
                            gist:\n\
                            \t{gist}\n\
                            \n\
                            deps:\n\
                            \t{deps}\n\
                            \n\
                            buck2:\n\
                            \t{buck2}\n"
                        ),
                    )
                    .build()
            }
        }
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        // MakegenOp doesn't have list inputs that need cardinality testing
        vec![]
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        match self {
            MakegenOp::LoadRegistry => vec![],
            MakegenOp::RenderMakefile => vec![],
            MakegenOp::Entrypoint => vec![],
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

        match result.get("tool_names").and_then(|v| v.as_str_list()) {
            Some(names) => {
                assert!(names.contains(&"gist".to_string()));
                assert!(names.contains(&"deps".to_string()));
            }
            _ => panic!("expected tool names"),
        }
    }

    #[test]
    fn test_render_makefile() {
        let result = execute_render_makefile(HashMap::new()).unwrap();

        match result.get("return") {
            Some(Value::Str(content)) => {
                assert!(content.contains("gist:"));
                assert!(content.contains("deps:"));
            }
            _ => panic!("expected makefile content"),
        }
    }
}
