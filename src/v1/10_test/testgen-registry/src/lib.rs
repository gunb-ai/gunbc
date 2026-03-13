//! Auto-discovered testgen target registry.
//!
//! This crate provides:
//! - A registry for testgen targets (via `inventory`)
//! - A shared helper to generate test modules from DAGs + MockSpecs

#![deny(dead_code)]
// Re-export inventory so macros can submit without depending on it directly.
pub use inventory;

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
/// Delegates to [`gunbc_codegen::testgen::generate_target`].
pub fn generate_target<T: Executable + Clone + 'static>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
) -> String {
    gunbc_codegen::testgen::generate_target(config, dag, spec)
}

/// Like [`generate_target`] but merges a DSL-extracted type registry.
///
/// Delegates to [`gunbc_codegen::testgen::generate_target_with_types`].
pub fn generate_target_with_types<T: Executable + Clone + 'static>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
    dsl_type_registry: Option<&TypeRegistry>,
) -> String {
    gunbc_codegen::testgen::generate_target_with_types(config, dag, spec, dsl_type_registry)
}

/// Like [`generate_target_with_types`] but also accepts failure-variant MockSpecs.
///
/// Delegates to [`gunbc_codegen::testgen::generate_target_full`].
pub fn generate_target_full<T: Executable + Clone + 'static>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
    dsl_type_registry: Option<&TypeRegistry>,
    failure_variants: Vec<gunbc_test::FailureVariant>,
) -> String {
    gunbc_codegen::testgen::generate_target_full(
        config,
        dag,
        spec,
        dsl_type_registry,
        failure_variants,
    )
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
        // into the same test harness crate (e.g. gunbc-tests). When this crate is tested in
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
