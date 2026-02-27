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
        /// PT-6: Per-profile live test configurations for this module.
        live_profile_tests: Vec<gunbc_codegen::registry::LiveProfileTestConfig>,
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
                live_profile_tests,
            } => {
                // 1. Compile .dag → Dag<DynOp> + DSL type registry
                let result = match crate::dsl_builder::build_dsl_graph_with_types(dsl_path) {
                    Ok(r) => r,
                    Err(e) => {
                        let placeholder = format!(
                            "// Auto-testgen skipped for '{module_name}': {e}\n\
                             // This module cannot be compiled without additional context\n\
                             // (e.g., provider binding or --profile flag).\n\
                             // See IS-3 in TODO/tasks.md for the proper fix.\n"
                        );
                        return OutputMap::new()
                            .str("content", placeholder)
                            .str("path", output_path.to_string())
                            .ok();
                    }
                };

                // 2. Auto-generate MockSpec from types + DAG structure
                let safe_name = module_name.replace('.', "-");
                let spec = crate::mock_defaults::auto_mock_spec(&result.dag, &safe_name);

                // 2b. Classify the module via DSL-evaluated fidelity policy
                let classification =
                    crate::fidelity::classify_module(&result.callable_properties);
                let all_transport_classes: Vec<_> = result
                    .callable_properties
                    .values()
                    .flat_map(|p| p.transport_classes.iter().cloned())
                    .collect();
                let requires =
                    crate::fidelity::requires_from_transport_classes(&all_transport_classes);

                // 3. Build TestgenTargetDef
                let module_test_name = format!("{}_generated_tests", module_name.replace('.', "_"));
                let dag_builder_call = format!(
                    "crate::dsl_builder::build_dsl_graph(\"{dsl_path}\").expect(\"graph should build\")"
                );
                let mock_spec_path =
                    format!("crate::mock_defaults::auto_mock_spec(&dag, \"{safe_name}\")");

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
                    test_class: classification.test_class,
                    fermi_cost: classification.fermi_cost,
                    requires,
                    secrets: None,
                    live_test_class: None,
                    live_fermi_cost: None,
                    live_requires: None,
                    live_required: None,
                    live_required_any_of: None,
                    tool_name: None,
                    live_profile_tests: live_profile_tests.clone(),
                };

                // 4. Generate test code with DSL type awareness
                let content = gunbc_testgen_registry::generate_target_with_types(
                    &target_def,
                    result.dag,
                    spec,
                    Some(&result.dsl_type_registry),
                );

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
