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
use gunbc_ir::{Dag, TypeRegistry};
use gunbc_test::{FermiCost, MockSpec, TestClass};

/// Metadata for a DAG spec (output and ownership details).
#[derive(Debug, Clone)]
pub struct DagSpecMeta {
    pub output_path: &'static str,
    pub module_name: &'static str,
    /// Tool name for CLI contract test generation. When set, entrypoints
    /// are looked up from DSL-driven `discover_tool_defs_from_dsl()` and a
    /// CLI contract test is emitted.
    pub tool_name: Option<&'static str>,
}

/// Testgen configuration for a DAG spec.
#[derive(Debug, Clone)]
pub struct DagSpecTestgen {
    pub boundary_tests: bool,
    pub chain_tests: bool,
    pub flow_tests: bool,
    pub live_flow_tests: bool,
    pub window_max_nodes: Option<usize>,
    pub test_class: Option<TestClass>,
    pub fermi_cost: Option<FermiCost>,
    pub requires: Option<&'static [&'static str]>,
    pub secrets: Option<&'static [&'static str]>,
    pub live_test_class: Option<TestClass>,
    pub live_fermi_cost: Option<FermiCost>,
    pub live_requires: Option<&'static [&'static str]>,
    pub live_required: Option<&'static [&'static str]>,
    pub live_required_any_of: Option<&'static [&'static [&'static str]]>,
}

/// A registered DAG spec (auto-discovered via inventory).
///
/// Uses &'static str fields so registration can be const.
#[derive(Debug)]
pub struct DagSpecDef {
    /// Originating crate name (for crate:: path rewriting)
    pub origin_crate: &'static str,
    pub name: &'static str,
    pub dag_builder_call: &'static str,
    pub mock_spec_path: &'static str,
    pub signature_path: Option<&'static str>,
    pub meta: DagSpecMeta,
    pub testgen: DagSpecTestgen,
    pub generate: fn(&TestgenTargetDef) -> String,
}

impl DagSpecDef {
    /// Convert this registration into a TestgenTargetDef.
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

        let mut def =
            TestgenTargetDef::new(self.name, self.meta.output_path, self.meta.module_name);
        def.dag_builder_call = to_crate_path(self.dag_builder_call, self.origin_crate).into();
        def.mock_spec_path = to_crate_path(self.mock_spec_path, self.origin_crate).into();
        def.signature_path = self
            .signature_path
            .map(|s| to_crate_path(s, self.origin_crate).into());
        def.boundary_tests = self.testgen.boundary_tests;
        def.chain_tests = self.testgen.chain_tests;
        def.flow_tests = self.testgen.flow_tests;
        def.live_flow_tests = self.testgen.live_flow_tests;
        def.window_max_nodes = self.testgen.window_max_nodes;
        def.test_class = self.testgen.test_class.unwrap_or(TestClass::Unit);
        def.fermi_cost = self.testgen.fermi_cost.unwrap_or(FermiCost::XS);
        def.requires = self
            .testgen
            .requires
            .map(|items| items.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();
        def.secrets = self
            .testgen
            .secrets
            .map(|items| items.iter().map(|s| s.to_string()).collect());
        def.live_test_class = self.testgen.live_test_class;
        def.live_fermi_cost = self.testgen.live_fermi_cost;
        def.live_requires = self
            .testgen
            .live_requires
            .map(|items| items.iter().map(|s| s.to_string()).collect());
        def.live_required = self
            .testgen
            .live_required
            .map(|items| items.iter().map(|s| s.to_string()).collect());
        def.live_required_any_of = self.testgen.live_required_any_of.map(|groups| {
            groups
                .iter()
                .map(|group| group.iter().map(|s| s.to_string()).collect())
                .collect()
        });
        def.tool_name = self.meta.tool_name.map(Into::into);
        def
    }
}

inventory::collect!(DagSpecDef);

/// Iterate over all registered DAG specs.
pub fn iter_dag_specs() -> impl Iterator<Item = &'static DagSpecDef> {
    inventory::iter::<DagSpecDef>.into_iter()
}

/// Registered DAG builders for resource purity checks.
#[derive(Debug)]
pub struct ResourceTestDef {
    pub origin_crate: &'static str,
    pub name: &'static str,
    pub build: fn() -> Dag<()>,
}

inventory::collect!(ResourceTestDef);

/// Iterate over all registered resource test DAGs.
pub fn iter_resource_tests() -> impl Iterator<Item = &'static ResourceTestDef> {
    use std::collections::HashSet;

    let mut seen: HashSet<(&'static str, &'static str)> = HashSet::new();
    inventory::iter::<ResourceTestDef>
        .into_iter()
        .filter(move |def| seen.insert((def.origin_crate, def.name)))
}

/// Shared test generation helper: builds test code from a DAG + MockSpec + config.
///
/// This is the single codegen path — all targets use this function.
/// Per-target variation is only in which DAG and MockSpec are provided.
pub fn generate_target<T: Executable + Clone + 'static>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
) -> String {
    generate_target_with_types(config, dag, spec, None)
}

/// Like [`generate_target`] but merges a DSL-extracted type registry into the
/// core type registry, making DSL-defined sum/product types visible to testgen
/// for variant coverage obligations.
pub fn generate_target_with_types<T: Executable + Clone + 'static>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
    dsl_type_registry: Option<&TypeRegistry>,
) -> String {
    // Classification is now provided by callers via fidelity::classify_module().
    // Simple defaults for the rare case a caller doesn't provide them.
    let test_class = config.test_class;
    let fermi_cost = config.fermi_cost;
    let requires = config.requires.clone();
    let secrets = config.secrets.clone().unwrap_or_default();
    let live_test_class = config.live_test_class.unwrap_or(TestClass::Integration);
    let live_requires = config.live_requires.clone().unwrap_or_default();
    let live_fermi_cost = config.live_fermi_cost.unwrap_or(FermiCost::M);
    let live_required = config.live_required.clone().unwrap_or_default();
    let live_required_any_of = config.live_required_any_of.clone().unwrap_or_default();

    let test_config = TestConfig {
        boundary_tests: config.boundary_tests,
        chain_tests: config.chain_tests,
        flow_tests: config.flow_tests,
        live_flow_tests: config.live_flow_tests,
        window_max_nodes: config.window_max_nodes,
        test_class,
        fermi_cost,
        requires,
        secrets,
        live_test_class,
        live_fermi_cost,
        live_requires,
        live_required,
        live_required_any_of,
        live_profile_tests: config.live_profile_tests.clone(),
        target_name: config.name.to_string(),
        ..TestConfig::default()
    };
    let mut registry = TypeRegistry::with_core_types();
    if let Some(dsl_types) = dsl_type_registry {
        registry.merge(dsl_types);
    }
    let mut generator = TestGenerator::new(&dag)
        .with_config(test_config)
        .with_type_registry(registry)
        .with_mock_spec(spec)
        .with_mock_spec_fn(config.mock_spec_path.as_ref());
    if let Some(signature_fn) = &config.signature_path {
        generator = generator.with_signature_fn(signature_fn.as_ref());
    }

    // CLI contract test generation: tool_name lookup is now DSL-driven via
    // discover_tool_defs_from_dsl() in gunbc-dag/src/dsl_registry.rs.
    // All current testgen targets set tool_name: None, so this path is unused.

    generator.generate_test_module(&config.module_name, &config.dag_builder_call)
}

// TestgenTargetDef is re-exported above for macro users.

#[cfg(test)]
mod resource_tests {
    use super::iter_resource_tests;
    use gunbc_ir::{
        derive_resource_accesses, detect_resource_conflicts, validate_resource_wiring_recursive,
    };

    #[test]
    fn resource_purity_checks() {
        let defs: Vec<_> = iter_resource_tests().collect();
        // inventory registrations only appear when downstream crates are linked
        // into the same binary (e.g. gunbc-dag). When this crate is tested in
        // isolation via `cargo test -p gunbc-testgen-registry`, no registrations
        // exist — skip gracefully rather than panic.
        if defs.is_empty() {
            return;
        }

        let mut failures = Vec::new();

        for def in defs {
            let dag = (def.build)();

            match derive_resource_accesses(&dag) {
                Ok(_) => {}
                Err(err) => {
                    failures.push(format!(
                        "{} ({}): derive_resource_accesses failed: {:?}",
                        def.name, def.origin_crate, err
                    ));
                    continue;
                }
            }

            match detect_resource_conflicts(&dag) {
                Ok(conflicts) => {
                    if !conflicts.is_empty() {
                        failures.push(format!(
                            "{} ({}): {} resource conflict(s): {:?}",
                            def.name,
                            def.origin_crate,
                            conflicts.len(),
                            conflicts
                        ));
                    }
                }
                Err(err) => failures.push(format!(
                    "{} ({}): detect_resource_conflicts failed: {:?}",
                    def.name, def.origin_crate, err
                )),
            }

            let unwired = validate_resource_wiring_recursive(&dag);
            if !unwired.is_empty() {
                failures.push(format!(
                    "{} ({}): {} unwired resource port(s): {:?}",
                    def.name,
                    def.origin_crate,
                    unwired.len(),
                    unwired
                ));
            }
        }

        assert!(
            failures.is_empty(),
            "resource purity checks failed:\n{}",
            failures.join("\n")
        );
    }
}
