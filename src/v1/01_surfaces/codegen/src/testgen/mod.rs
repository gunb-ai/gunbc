//! Test generation from proof obligations.
//!
//! Generates test code from DAG structures using the
//! **proof obligation** model:
//!
//! 1. Analyze the DAG to collect proof obligations
//! 2. Discharge obligations that are statically proven
//! 3. Generate tests only for undischarged obligations
//!
//! This ensures tests are **non-tautological**: we never test what
//! the compiler already proves (type compatibility, cardinality
//! satisfaction, acyclicity).
//!
//! # Obligation Buckets
//!
//! | Bucket | What It Proves |
//! |--------|---------------|
//! | **A** | Execution semantics (DryRun, interception, determinism) |
//! | **B** | Contract obligations (L3 entailment, node compliance) |
//! | **C** | Scenario coverage (success/failure paths, guard branches) |
//! | **D** | Resource hygiene (connectivity, ownership, conflicts) |

pub mod analyze;
pub mod cardinality;
pub mod codegen;
pub mod mock_corpus;
pub mod obligation;
pub mod probe_observer;
pub mod registry_gen;
pub mod render_rust;

pub use analyze::{analyze_dag, analyze_dag_with_obligations, DagAnalysis};
pub use codegen::{TestConfig, TestGenerator};
pub use obligation::{
    collect_obligations, DischargeStatus, EntailmentStatus, Obligation, ObligationSet,
    ObligationSource, ObligationStats, ProofObligation,
};
pub use probe_observer::{
    analyze_probe_observers, observability_report, CoverageGap, Observer, Probe,
    ProbeObserverAnalysis, ProbeObserverTest, ProbeSource,
};

use crate::registry::TestgenTargetDef;
use gunbc_exec::Executable;
use gunbc_ir::{Dag, TypeRegistry};
use gunbc_test::{FailureVariant, MockSpec, TestClass};

/// Generate test code from a DAG + MockSpec + config.
pub fn generate_target<T: Executable + Clone + 'static>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
) -> String {
    generate_target_full(config, dag, spec, None, Vec::new())
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
    generate_target_full(config, dag, spec, dsl_type_registry, Vec::new())
}

/// Like [`generate_target_with_types`] but also accepts failure-variant MockSpecs
/// for error-path test generation (RV-4).
pub fn generate_target_full<T: Executable + Clone + 'static>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
    dsl_type_registry: Option<&TypeRegistry>,
    failure_variants: Vec<FailureVariant>,
) -> String {
    let test_class = config.test_class;
    let fermi_cost = config.fermi_cost;
    let requires = config.requires.clone();
    let secrets = config.secrets.clone().unwrap_or_default();
    let live_test_class = config.live_test_class.unwrap_or(TestClass::Integration);
    let live_requires = config.live_requires.clone().unwrap_or_default();
    let live_fermi_cost = config.live_fermi_cost.unwrap_or(gunbc_test::FermiCost::M);
    let live_required = config.live_required.clone().unwrap_or_default();
    let live_required_any_of = config.live_required_any_of.clone().unwrap_or_default();

    let test_config = TestConfig {
        boundary_tests: config.boundary_tests,
        chain_tests: config.chain_tests,
        live_flow_tests: config.live_flow_tests,
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
        .with_mock_spec_fn(config.mock_spec_path.as_ref())
        .with_failure_variants(failure_variants);
    if let Some(signature_fn) = &config.signature_path {
        generator = generator.with_signature_fn(signature_fn.as_ref());
    }

    generator.generate_test_module(&config.module_name, &config.dag_builder_call)
}
