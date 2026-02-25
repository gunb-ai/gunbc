//! Testgen DAG operations.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes in the DAG.
//! The ops here are PURE (no I/O) - they generate test code.

use gunbc_codegen::TestgenTargetDef;
use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::fmt;

/// Operations for the testgen DAG.
///
/// All operations are PURE - no I/O. I/O happens via TransportOps::Execute nodes.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum TestgenOp {
    /// Generate test code for a target (inventory-registered path).
    Generate {
        name: String,
        target_def: TestgenTargetDef,
        generate_fn: fn(&TestgenTargetDef) -> String,
    },
    /// Auto-generate test code for a discovered .dag module.
    ///
    /// Pipeline: compile → auto_mock_spec → generate_target. Zero manual input.
    AutoGenerate {
        dsl_path: String,
        module_name: String,
        output_path: String,
    },
}

impl fmt::Debug for TestgenOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestgenOp::Generate { name, .. } => {
                f.debug_struct("Generate").field("name", name).finish()
            }
            TestgenOp::AutoGenerate { module_name, .. } => f
                .debug_struct("AutoGenerate")
                .field("module", module_name)
                .finish(),
        }
    }
}

impl Executable for TestgenOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            TestgenOp::Generate {
                name,
                target_def,
                generate_fn,
            } => {
                let def = target_def.clone();
                let f = *generate_fn;
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (f)(&def)));
                match result {
                    Ok(content) => OutputMap::new()
                        .str("content", content)
                        .str("path", target_def.output_path.to_string())
                        .ok(),
                    Err(payload) => {
                        let message = if let Some(s) = payload.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = payload.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "unknown panic".to_string()
                        };
                        Err(ExecError::new(format!(
                            "generate '{}' failed:\n{}",
                            name, message
                        )))
                    }
                }
            }
            TestgenOp::AutoGenerate {
                dsl_path,
                module_name,
                output_path,
            } => {
                // 1. Compile .dag → Dag<DynOp>
                let dag = crate::dsl_builder::build_dsl_graph(dsl_path).map_err(|e| {
                    ExecError::new(format!("auto-generate '{module_name}' compile error: {e}"))
                })?;

                // 2. Auto-generate MockSpec from types + DAG structure
                let safe_name = module_name.replace('.', "-");
                let spec = crate::mock_defaults::auto_mock_spec(&dag, &safe_name);

                // 3. Build TestgenTargetDef
                let module_test_name =
                    format!("{}_generated_tests", module_name.replace('.', "_"));
                let dag_builder_call = format!(
                    "crate::dsl_builder::build_dsl_graph(\"{dsl_path}\").expect(\"graph should build\")"
                );
                let mock_spec_path = format!(
                    "crate::mock_defaults::auto_mock_spec(&dag, \"{safe_name}\")"
                );

                let target_def = TestgenTargetDef {
                    name: std::borrow::Cow::Owned(safe_name),
                    output_path: std::borrow::Cow::Owned(output_path.clone()),
                    module_name: std::borrow::Cow::Owned(module_test_name),
                    mock_spec_path: std::borrow::Cow::Owned(mock_spec_path),
                    dag_builder_call: std::borrow::Cow::Owned(dag_builder_call),
                    signature_path: None,
                    boundary_tests: true,
                    chain_tests: true,
                    flow_tests: true,
                    live_flow_tests: false,
                    window_max_nodes: None,
                    test_class: None,
                    fermi_cost: None,
                    requires: None,
                    secrets: None,
                    live_test_class: None,
                    live_fermi_cost: None,
                    live_requires: None,
                    live_required: None,
                    live_required_any_of: None,
                    tool_name: None,
                };

                // 4. Generate test code
                let content =
                    gunbc_testgen_registry::generate_target(&target_def, dag, spec);

                OutputMap::new()
                    .str("content", content)
                    .str("path", output_path.to_string())
                    .ok()
            }
        }
    }
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for TestgenOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            TestgenOp::Generate {
                name, target_def, ..
            } => OutputMap::new()
                .str("content", format!("// Mock generated tests for {}", name))
                .str("path", target_def.output_path.to_string())
                .build(),
            TestgenOp::AutoGenerate {
                module_name,
                output_path,
                ..
            } => OutputMap::new()
                .str(
                    "content",
                    format!("// Mock auto-generated tests for {}", module_name),
                )
                .str("path", output_path.to_string())
                .build(),
        }
    }
}
