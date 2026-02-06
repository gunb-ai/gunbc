//! Auto-discovered testgen target registry.
//!
//! This crate provides:
//! - A registry for testgen targets (via `inventory`)
//! - A shared helper to generate test modules from DAGs + MockSpecs

#![deny(dead_code)]
// Re-export inventory so macros can submit without depending on it directly.
pub use inventory;

use gunbc_codegen::testgen::{TestConfig, TestGenerator};
pub use gunbc_codegen::TestgenTargetDef;
use gunbc_exec::Executable;
use gunbc_ir::Dag;
use gunbc_test::MockSpec;

/// A registered testgen target (auto-discovered via inventory).
///
/// Uses &'static str fields so registration can be const.
#[derive(Debug)]
pub struct TestgenTarget {
    /// Originating crate name (for crate:: path rewriting)
    pub origin_crate: &'static str,
    pub name: &'static str,
    pub output_path: &'static str,
    pub module_name: &'static str,
    pub dag_builder_call: &'static str,
    pub mock_spec_path: &'static str,
    pub signature_path: Option<&'static str>,
    pub boundary_tests: bool,
    pub chain_tests: bool,
    pub flow_tests: bool,
    pub window_max_nodes: Option<usize>,
    pub generate: fn(&TestgenTargetDef) -> String,
}

impl TestgenTarget {
    /// Convert this registration into a TestgenTargetDef (owned strings).
    pub fn to_def(&self) -> TestgenTargetDef {
        fn to_crate_path(path: &str, origin: &str) -> String {
            // module_path! uses the crate identifier form (hyphens -> underscores).
            let origin_ident = origin.replace('-', "_");
            let prefix = format!("{}::", origin_ident);
            if let Some(stripped) = path.strip_prefix(&prefix) {
                format!("crate::{}", stripped)
            } else {
                path.to_string()
            }
        }

        let mut def = TestgenTargetDef::new(self.name, self.output_path, self.module_name);
        def.dag_builder_call = to_crate_path(self.dag_builder_call, self.origin_crate);
        def.mock_spec_path = to_crate_path(self.mock_spec_path, self.origin_crate);
        def.signature_path =
            self.signature_path.map(|s| to_crate_path(s, self.origin_crate));
        def.boundary_tests = self.boundary_tests;
        def.chain_tests = self.chain_tests;
        def.flow_tests = self.flow_tests;
        def.window_max_nodes = self.window_max_nodes;
        def
    }
}

inventory::collect!(TestgenTarget);

/// Iterate over all registered testgen targets.
pub fn iter_targets() -> impl Iterator<Item = &'static TestgenTarget> {
    inventory::iter::<TestgenTarget>.into_iter()
}

/// Shared test generation helper: builds test code from a DAG + MockSpec + config.
///
/// This is the single codegen path — all targets use this function.
/// Per-target variation is only in which DAG and MockSpec are provided.
pub fn generate_target<T: Executable + Clone>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
) -> String {
    let test_config = TestConfig {
        boundary_tests: config.boundary_tests,
        chain_tests: config.chain_tests,
        flow_tests: config.flow_tests,
        window_max_nodes: config.window_max_nodes,
        ..TestConfig::default()
    };
    let mut generator = TestGenerator::new(&dag)
        .with_config(test_config)
        .with_mock_spec(spec)
        .with_mock_spec_fn(&config.mock_spec_path);
    if let Some(signature_fn) = &config.signature_path {
        generator = generator.with_signature_fn(signature_fn);
    }
    generator.generate_test_module(&config.module_name, &config.dag_builder_call)
}

// TestgenTargetDef is re-exported above for macro users.
