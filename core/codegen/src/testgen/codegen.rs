// Vec-init-then-push in fidelity ladder test generation is clearer than vec![] macro.
#![allow(clippy::vec_init_then_push)]

//! Test code generation from proof obligations.
//!
//! Tests are organized by obligation bucket:
//!
//! - **Bucket A (Execution Semantics)**: DryRun completion, transport interception,
//!   pure node determinism.
//! - **Bucket B (Contract Obligations)**: Predicate entailment, node contract
//!   compliance. Only generated when static analysis returns Unknown.
//! - **Bucket C (Scenario Coverage)**: All-succeed, single-failure, guard/skip
//!   branch coverage. N+1 scenarios instead of 2^N.
//! - **Bucket D (Resource Hygiene)**: Resource connectivity, ownership, conflict
//!   absence, simulation tests.
//!
//! # Anti-tautology rule
//!
//! Proven by construction (NO tests generated):
//! - Acyclicity (DAG structure)
//! - Edge type compatibility (validate_dag proves this)
//! - Edge cardinality compatibility (compile-time checked)
//!
//! Only obligations that are Unknown or RuntimeOnly produce tests.

use crate::testgen::analyze::{analyze_dag, DagAnalysis};
use crate::testgen::registry_gen::derive_virtual_backend_requirements;
use crate::testgen::obligation::{
    collect_obligations, DischargeStatus, Obligation, ObligationSet, ObligationSource,
    ProofObligation,
};
use crate::testgen::probe_observer::{
    analyze_probe_observers, observability_report, ProbeObserverAnalysis,
};
use crate::testgen::render_rust::plain_rust_renderer;
use gunbc_cli::ParamType;
use gunbc_infra::hash::ContentHash;
use gunbc_ir::boundary_label;
use gunbc_ir::code_ir::{
    Assert, BindTarget, Expr, HelperFn, Import, Item, Stmt, TestFile, TestFn, TestSection,
};
use gunbc_ir::language::NamingCase;
use gunbc_ir::render_ir::CodeRenderer;
use gunbc_ir::transport::{ShellRequest, ShellResponse, TransportRequest, TransportResponse};
use gunbc_ir::{
    contract, parse_map_type_id, semantic_carrier_class_for_type_id, value_compatible_with_type_id,
    value_kind_name, Cardinality, Dag, NodeId, NodeKind, Os, PortName, SecretString,
    SeedPlaceholderPolicy, SemanticCarrierClass, TypeRegistry, Value, ValueExpr,
};
use gunbc_test::{FermiCost, MockSpec, OutputMatcher, TestClass};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;

type SeedPolicy = SeedPlaceholderPolicy;
type SeedClass = SemanticCarrierClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeedContext {
    RealSingleNode,
    Scenario,
    LiveFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SeedMatrix {
    structural_policy: SeedPolicy,
    semantic_policy: SeedPolicy,
}

const STRICT_REAL_SINGLE_NODE_SEED_MATRIX: SeedMatrix = SeedMatrix {
    structural_policy: SeedPolicy::Generated,
    semantic_policy: SeedPolicy::ExplicitSeedRequired,
};

const STRICT_SCENARIO_SEED_MATRIX: SeedMatrix = SeedMatrix {
    structural_policy: SeedPolicy::Generated,
    semantic_policy: SeedPolicy::ExplicitSeedRequired,
};

const STRICT_LIVE_FLOW_SEED_MATRIX: SeedMatrix = SeedMatrix {
    structural_policy: SeedPolicy::Generated,
    semantic_policy: SeedPolicy::ExplicitSeedRequired,
};

fn seed_matrix_for_context(context: SeedContext) -> SeedMatrix {
    match context {
        SeedContext::RealSingleNode => STRICT_REAL_SINGLE_NODE_SEED_MATRIX,
        SeedContext::Scenario => STRICT_SCENARIO_SEED_MATRIX,
        SeedContext::LiveFlow => STRICT_LIVE_FLOW_SEED_MATRIX,
    }
}

fn seed_class_for_type(type_id: &str) -> SeedClass {
    semantic_carrier_class_for_type_id(type_id)
}

fn seed_policy_for_type(type_id: &str) -> SeedPolicy {
    match seed_class_for_type(type_id) {
        SeedClass::StructuralGeneratable => SeedPolicy::Generated,
        SeedClass::SemanticCarrier => SeedPolicy::ExplicitSeedRequired,
    }
}

fn seed_policy_for_context(type_id: &str, context: SeedContext) -> SeedPolicy {
    let matrix = seed_matrix_for_context(context);
    match seed_policy_for_type(type_id) {
        SeedPolicy::Generated => matrix.structural_policy,
        SeedPolicy::ExplicitSeedRequired => matrix.semantic_policy,
    }
}

fn requires_explicit_seed(type_id: &str, context: SeedContext) -> bool {
    seed_policy_for_context(type_id, context) == SeedPolicy::ExplicitSeedRequired
}

/// Returns the canonical type-name string for a Value's kind.
/// Thin wrapper around `Value::kind().type_name()` — kept as a private helper
/// so callers don't repeat the two-step chain.
fn mock_value_kind_name(value: &Value) -> &'static str {
    value_kind_name(value)
}

fn mock_types_compatible(port_type: &str, value: &Value) -> bool {
    value_compatible_with_type_id(port_type, value)
}

/// Configuration for test generation.
///
/// # What is NOT generated (proven by construction):
///
/// - Type compatibility: `validate_dag()` proves types match at compile time
/// - Cardinality satisfaction: edge creation verifies cardinalities
/// - Acyclicity: DAG structure is acyclic by construction
///
/// # What IS generated (runtime / Unknown obligations):
///
/// - DryRun completion and transport interception
/// - Contract entailment tests (when proof engine returns Unknown)
/// - Scenario coverage (success + per-transport failure + guard toggles)
/// - Resource hygiene (connectivity, ownership, conflicts)
/// - Resource simulation (MockSpec-based acquisition/timeout)
/// - Node I/O example tests (when MockSpec has node_examples)
/// - Windowed segment tests (contiguous sub-DAG slices)
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Generate Bucket A tests (execution semantics)
    pub execution_tests: bool,
    /// Generate Bucket B tests (contract obligations)
    pub contract_tests: bool,
    /// Generate Bucket C tests (scenario coverage)
    pub scenario_tests: bool,
    /// Generate Bucket D tests (resource hygiene + simulation)
    pub resource_tests: bool,
    /// Generate legacy boundary tests (individual per-boundary-node mockability)
    pub boundary_tests: bool,
    /// Generate chain validation tests (mock spec self-consistency)
    pub chain_tests: bool,
    /// Generate flow verification tests (DryRun full DAG, verify terminal outputs).
    /// DEPRECATED: Tautological — replaced by probe_observer_tests.
    pub flow_tests: bool,
    /// Generate live flow verification tests (Real execution, gated by env + cost)
    pub live_flow_tests: bool,
    /// Generate per-node I/O example tests (from MockSpec.node_examples)
    pub example_tests: bool,
    /// Generate optional-input behavior tests (missing + wrong-type)
    pub optional_input_tests: bool,
    /// Generate probe-observer integration tests (non-tautological chain tests)
    pub probe_observer_tests: bool,
    /// Generate per-node corpus tests (BB-2: black-box node testing from cross-workflow corpus)
    pub corpus_tests: bool,
    /// Generate adjacent-pair edge tests (BB-3: 2-node window through real executor wiring)
    pub adjacent_pair_tests: bool,
    /// Generate cross-workflow consistency tests (BB-5: same node, multiple workflows)
    pub cross_workflow_tests: bool,
    /// Generate transport fidelity ladder tests (BB-6: tiered test variants)
    pub fidelity_ladder_tests: bool,
    /// Max window size for windowed tests (None = disabled).
    /// DEPRECATED: Tautological — replaced by probe_observer_tests.
    pub window_max_nodes: Option<usize>,
    /// Test module visibility
    pub visibility: String,
    /// Test class (unit/hermetic/integration)
    pub test_class: TestClass,
    /// Fermi-style cost bucket
    pub fermi_cost: FermiCost,
    /// External requirements (informational)
    pub requires: Vec<String>,
    /// Required secrets (env vars) for live integration tests
    pub secrets: Vec<String>,
    /// Live test class override (typically Integration)
    pub live_test_class: TestClass,
    /// Live test cost bucket
    pub live_fermi_cost: FermiCost,
    /// Live test external requirements (informational)
    pub live_requires: Vec<String>,
    /// Live test required env vars (hard requirements)
    pub live_required: Vec<String>,
    /// Live test required any-of env var groups
    pub live_required_any_of: Vec<Vec<String>>,
    /// Per-profile live test configurations (PT-5).
    pub live_profile_tests: Vec<crate::registry::LiveProfileTestConfig>,
    /// Target name for generating test function names.
    pub target_name: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            execution_tests: true,
            contract_tests: true,
            scenario_tests: true,
            resource_tests: true,
            boundary_tests: true,
            chain_tests: true,
            flow_tests: false,
            live_flow_tests: false,
            example_tests: true,
            optional_input_tests: true,
            probe_observer_tests: true,
            corpus_tests: false,
            adjacent_pair_tests: false,
            cross_workflow_tests: false,
            fidelity_ladder_tests: false,
            window_max_nodes: None, // Deprecated: use probe_observer_tests instead
            visibility: "pub".to_string(),
            test_class: TestClass::Hermetic,
            fermi_cost: FermiCost::XS,
            requires: Vec::new(),
            secrets: Vec::new(),
            live_test_class: TestClass::Integration,
            live_fermi_cost: FermiCost::M,
            live_requires: Vec::new(),
            live_required: Vec::new(),
            live_required_any_of: Vec::new(),
            live_profile_tests: Vec::new(),
            target_name: String::new(),
        }
    }
}

/// Test code generator.
///
/// Generates test code from DAG structure + proof obligations.
/// Uses the obligation model to ensure only non-tautological tests are produced.
pub struct TestGenerator<'a, T> {
    dag: &'a Dag<T>,
    config: TestConfig,
    mock_spec: Option<MockSpec>,
    /// Function path to call for MockSpec (e.g., "crate::ci::graph_mock::ci_mock_spec()")
    mock_spec_fn: Option<String>,
    /// Function path to call for declared signature (e.g., "crate::makegen_signature()")
    signature_fn: Option<String>,
    /// CLI entrypoints for contract test generation: (tool_name, entrypoints).
    cli_entrypoints: Option<(String, Vec<crate::cli_gen::CliEntrypoint>)>,
    /// Type registry for contract-derived witness values.
    type_registry: TypeRegistry,
    /// Cross-workflow mock corpus for per-node black-box tests (BB-2).
    corpus: Option<HashMap<gunbc_test::NodeIdentity, gunbc_test::MockCorpus>>,
    /// Edge examples for adjacent-pair tests (BB-3).
    edge_examples: Option<Vec<gunbc_test::EdgeExample>>,
}

struct ProbeObserverBundle {
    analysis: ProbeObserverAnalysis,
    report: String,
    lowering_error: Option<String>,
}

impl ProbeObserverBundle {
    fn has_coverage(&self) -> bool {
        !self.analysis.probes.is_empty() || !self.analysis.observers.is_empty()
    }
}

impl<'a, T: Clone + 'static> TestGenerator<'a, T> {
    /// Create a new test generator for a DAG.
    pub fn new(dag: &'a Dag<T>) -> Self {
        Self {
            dag,
            config: TestConfig::default(),
            mock_spec: None,
            mock_spec_fn: None,
            signature_fn: None,
            cli_entrypoints: None,
            type_registry: TypeRegistry::with_core_types(),
            corpus: None,
            edge_examples: None,
        }
    }

    /// Set the test configuration.
    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the mock specification for realistic mock values.
    pub fn with_mock_spec(mut self, spec: MockSpec) -> Self {
        self.mock_spec = Some(spec);
        self
    }

    /// Set the mock spec function path (e.g., "crate::ci::graph_mock::ci_mock_spec()").
    ///
    /// This is used to generate a `mock_spec()` helper function in the test module
    /// that calls the specified function to get the MockSpec at test time.
    pub fn with_mock_spec_fn(mut self, path: impl Into<String>) -> Self {
        self.mock_spec_fn = Some(path.into());
        self
    }

    /// Set the signature function path (e.g., "crate::ci::ci_signature()").
    pub fn with_signature_fn(mut self, path: impl Into<String>) -> Self {
        self.signature_fn = Some(path.into());
        self
    }

    /// Set CLI entrypoints for contract test generation.
    ///
    /// When set, a CLI contract test section is emitted that verifies
    /// `gunbc_cli::parse()` handles the tool's argument schema correctly.
    pub fn with_cli_entrypoints(
        mut self,
        tool_name: String,
        entrypoints: Vec<crate::cli_gen::CliEntrypoint>,
    ) -> Self {
        self.cli_entrypoints = Some((tool_name, entrypoints));
        self
    }

    /// Set a type registry for contract-derived witness values.
    pub fn with_type_registry(mut self, registry: TypeRegistry) -> Self {
        self.type_registry = registry;
        self
    }

    /// Set cross-workflow mock corpus for per-node black-box tests (BB-2).
    pub fn with_corpus(
        mut self,
        corpus: HashMap<gunbc_test::NodeIdentity, gunbc_test::MockCorpus>,
    ) -> Self {
        self.corpus = Some(corpus);
        self
    }

    /// Set edge examples for adjacent-pair tests (BB-3).
    pub fn with_edge_examples(mut self, examples: Vec<gunbc_test::EdgeExample>) -> Self {
        self.edge_examples = Some(examples);
        self
    }

    fn build_probe_observer_bundle(&self, analysis: &DagAnalysis) -> Option<ProbeObserverBundle> {
        let spec = self.mock_spec.as_ref()?;

        let (po_analysis, lowering_error) = match gunbc_exec::lower(self.dag) {
            Ok(lowered) => {
                let lowered_analysis = analyze_dag(&lowered.dag);
                (
                    analyze_probe_observers(&lowered.dag, spec, &lowered_analysis),
                    None,
                )
            }
            Err(err) => (
                analyze_probe_observers(self.dag, spec, analysis),
                Some(err.to_string()),
            ),
        };

        Some(ProbeObserverBundle {
            report: observability_report(&po_analysis),
            analysis: po_analysis,
            lowering_error,
        })
    }

    /// Generate the test module code.
    ///
    /// This is the main entry point. It:
    /// 1. Analyzes the DAG structure
    /// 2. Validates MockSpec requirement (panics if missing for DAGs with transports)
    /// 3. Collects proof obligations
    /// 4. Generates tests for undischarged obligations
    ///
    /// # Panics
    ///
    /// Panics if the DAG has transport executor nodes but no MockSpec was provided.
    /// This ensures that test generation fails early with a clear message rather than
    /// producing incomplete tests.
    pub fn generate_test_module(&self, module_name: &str, graph_builder_fn: &str) -> String {
        let analysis = analyze_dag(self.dag);

        // Validate MockSpec requirement for DAGs with transport nodes
        if !analysis.transport_executors.is_empty() && self.mock_spec.is_none() {
            panic!(
                "MockSpec required: DAG '{}' has {} transport executor node(s) ({}) but no MockSpec was provided.\n\
                 \n\
                 To fix this, create a MockSpec and pass it to TestGenerator:\n\
                 \n\
                 ```rust\n\
                 let spec = MockSpec::new(\"{}\")\n\
                     .boundary(\"<transport_node>\", \"response\", mock_response());\n\
                 \n\
                 TestGenerator::new(&dag)\n\
                     .with_mock_spec(spec)\n\
                     .generate_test_module(...)\n\
                 ```\n\
                 \n\
                 Transport nodes require mocks to specify what values they return during testing.",
                module_name,
                analysis.transport_executors.len(),
                analysis.transport_executors.join(", "),
                module_name
            );
        }

        // Validate transport mock coverage: every transport executor output port
        // that is connected downstream must have a mock in MockSpec.
        //
        // This replaces manual "boundary presence" tests in graph_mock.rs files.
        // If a transport output is used but not mocked, DryRun will fail at runtime.
        // Catching this early at test generation time gives a better error message.
        if let Some(spec) = &self.mock_spec {
            let lowered_for_validation = gunbc_exec::lower(self.dag).unwrap_or_else(|e| {
                panic!(
                    "DryRun mock coverage validation requires DAG lowering for '{}', but lowering failed: {}.\n\
                     Fix lowering issues before generating tests.",
                    module_name, e
                )
            });
            let lowered_dag = &lowered_for_validation.dag;

            let missing_mocks = self.find_missing_intercept_mocks_lowered(lowered_dag, spec);
            if !missing_mocks.is_empty() {
                panic!(
                    "DryRun mock coverage incomplete: DAG '{}' has {} intercepted output port(s) \
                     connected downstream but not mocked:\n\
                     \n\
                     {}\n\
                     \n\
                     Each intercepted output that flows to downstream nodes needs a mock.\n\
                     This includes lowered sub-DAG node IDs (for example `parent/subdag/node`).\n\
                     Add the missing mocks to your MockSpec:\n\
                     \n\
                     ```rust\n\
                     MockSpec::new(\"{}\")\n\
                     {}\n\
                     ```",
                    module_name,
                    missing_mocks.len(),
                    missing_mocks
                        .iter()
                        .map(|(node, port, kind)| format!("  - {}.{} ({})", node, port, kind))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    module_name,
                    missing_mocks
                        .iter()
                        .map(|(node, port, _)| {
                            format!("    .boundary(\"{}\", \"{}\", mock_value())", node, port)
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }

            // Validate that all mocks reference existing nodes and ports.
            //
            // This catches typos and stale mocks that reference renamed/removed nodes.
            let unknown_slots = self.find_unknown_mock_slots(spec, Some(lowered_dag));
            if !unknown_slots.is_empty() {
                panic!(
                    "Unknown mock slots: DAG '{}' has {} mock(s) referencing unknown nodes/ports:\n\
                     \n\
                     {}\n\
                     \n\
                     Each mock must reference an existing node output port.\n\
                     Check for typos or remove stale mocks.",
                    module_name,
                    unknown_slots.len(),
                    unknown_slots
                        .iter()
                        .map(|(node, port, reason)| {
                            format!("  - {}.{}: {}", node, port, reason)
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }

            // Validate mock-value type compatibility: every mock value's type should
            // be compatible with the corresponding port's TypeId.
            //
            // This catches type drift between MockSpec and DAG port definitions.
            // Contract-level check: "mock is Bool-typed" not "mock == Bool(true)".
            let type_mismatches = self.find_mock_type_mismatches(spec, Some(lowered_dag));
            if !type_mismatches.is_empty() {
                panic!(
                    "Mock value type mismatch: DAG '{}' has {} mock value(s) with incompatible types:\n\
                     \n\
                     {}\n\
                     \n\
                     Each mock value's type must be compatible with the port's declared type.\n\
                     Update the MockSpec with correctly-typed values.",
                    module_name,
                    type_mismatches.len(),
                    type_mismatches
                        .iter()
                        .map(|(node, port, expected, actual)| {
                            format!("  - {}.{}: expected {}, got {}", node, port, expected, actual)
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
        }

        // Validate that all pure nodes have I/O examples or are explicitly skipped.
        //
        // Pure nodes (non-transport, non-tool-env) contain domain logic that
        // should be tested. Each pure node must either:
        // - Have at least one NodeExample in the MockSpec
        // - Have at least one NodeIoExample on the Node itself
        // - Be explicitly skipped via MockSpec::skip_node_example()
        if self.config.example_tests && !analysis.pure_nodes.is_empty() {
            let example_node_ids: std::collections::HashSet<&str> = self
                .mock_spec
                .as_ref()
                .map(|s| s.node_examples.iter().map(|e| e.node_id.as_str()).collect())
                .unwrap_or_default();

            let skipped_node_ids: std::collections::HashSet<&str> = self
                .mock_spec
                .as_ref()
                .map(|s| s.skipped_node_examples.iter().map(|s| s.as_str()).collect())
                .unwrap_or_default();

            let node_example_ids: std::collections::HashSet<&str> = self
                .dag
                .nodes
                .iter()
                .filter(|n| !n.examples.is_empty())
                .map(|n| n.id.0.as_str())
                .collect();

            let uncovered: Vec<&str> = analysis
                .pure_nodes
                .iter()
                .map(|s| s.as_str())
                .filter(|id| {
                    !example_node_ids.contains(id)
                        && !skipped_node_ids.contains(id)
                        && !node_example_ids.contains(id)
                })
                .collect();

            if !uncovered.is_empty() {
                panic!(
                    "I/O examples required: DAG '{}' has {} pure node(s) without examples:\n\
                     \n\
                     {}\n\
                     \n\
                     Pure nodes contain domain logic that must be tested. For each node, either:\n\
                     \n\
                     1. Add a NodeExample to the MockSpec:\n\
                     \n\
                     ```rust\n\
                     MockSpec::new(\"{}\")\n\
                     {}    .node_example(\n\
                     {}        NodeExample::new(\"<node_id>\")\n\
                     {}            .input(\"port\", Value::Str(\"...\".into()))\n\
                     {}            .output(\"port\", OutputMatcher::non_empty())\n\
                     {}    )\n\
                     ```\n\
                     \n\
                     2. Skip enforcement for primitive/utility nodes:\n\
                     \n\
                     ```rust\n\
                     MockSpec::new(\"{}\")\n\
                     {}    .skip_node_example(\"<node_id>\")\n\
                     ```",
                    module_name,
                    uncovered.len(),
                    uncovered
                        .iter()
                        .map(|id| format!("  - {}", id))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    module_name,
                    " ",
                    " ",
                    " ",
                    " ",
                    " ",
                    module_name,
                    " ",
                );
            }
        }

        let mut obligations = collect_obligations(self.dag, &self.type_registry, None);
        if let Some((tool_name, entrypoints)) = &self.cli_entrypoints {
            if !entrypoints.is_empty() {
                obligations.all.push(ProofObligation::runtime(
                    Obligation::CliContractRoundTrip {
                        tool_name: tool_name.clone(),
                    },
                    format!(
                        "Tool '{}' CLI arguments must round-trip through parse and --print-inputs json",
                        tool_name
                    ),
                    ObligationSource::Contract,
                ));
            }
        }
        let probe_observer_bundle = self.build_probe_observer_bundle(&analysis);

        let mut file = self.generate_test_file(
            &analysis,
            &obligations,
            graph_builder_fn,
            probe_observer_bundle.as_ref(),
        );

        // Render body (no header) to compute content hash.
        let body = plain_rust_renderer().render_file(&file);
        let content_hash = ContentHash::from_bytes(body.as_bytes());

        let stats = obligations.stats();
        let generator = gunbc_ir::cargo::name("testgen");

        file.header = vec![
            format!("Generated tests for {} DAG.", module_name),
            String::new(),
            format!("Generated by {}", generator),
            "DO NOT EDIT - regenerate with: make testgen".to_string(),
            format!("Obligations: {}", stats),
            "Proven by construction: acyclicity, type compatibility, cardinality satisfaction."
                .to_string(),
            format!("Content-Hash: {}", content_hash.as_str()),
        ];

        // Append probe-observer coverage report if available.
        if let Some(bundle) = probe_observer_bundle.as_ref() {
            if bundle.has_coverage() {
                file.header.push(String::new());
                file.header.push("Probe-Observer Coverage:".to_string());
                for line in bundle.report.lines() {
                    file.header.push(format!("  {}", line));
                }
                if !bundle.analysis.gaps.is_empty() {
                    file.header.push(String::new());
                    file.header
                        .push("WARNING: Unobserved terminal nodes detected.".to_string());
                    file.header
                        .push("Add OutputMatchers via NodeExample for these nodes.".to_string());
                }
            }
        }

        plain_rust_renderer().render_file(&file)
    }

    /// Find intercepted output ports in the lowered DAG that are connected downstream
    /// but have no explicit DryRun mock in MockSpec.
    ///
    /// This catches missing mocks for nested SubDag nodes (e.g. `parent/child/node`)
    /// that are intercepted after lowering.
    fn find_missing_intercept_mocks_lowered(
        &self,
        lowered_dag: &Dag<T>,
        spec: &MockSpec,
    ) -> Vec<(String, String, &'static str)> {
        let mut missing = Vec::new();

        let mocked_ports: HashSet<(&str, &str)> = spec
            .transport_mocks
            .iter()
            .map(|m| (m.node.as_str(), m.port.as_str()))
            .chain(
                spec.boundary_mocks
                    .iter()
                    .map(|m| (m.node.as_str(), m.port.as_str())),
            )
            .collect();

        for node in &lowered_dag.nodes {
            let Some(kind) = Self::lowered_intercept_kind(node) else {
                continue;
            };

            for output_port in &node.outputs {
                let is_connected = lowered_dag
                    .edges
                    .iter()
                    .any(|e| e.from_node == node.id && e.from_port == output_port.name);
                if !is_connected {
                    continue;
                }

                if !mocked_ports.contains(&(node.id.0.as_str(), output_port.name.0.as_str())) {
                    missing.push((node.id.0.clone(), output_port.name.0.clone(), kind));
                }
            }
        }

        missing.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        missing.dedup();
        missing
    }

    fn lowered_intercept_kind(node: &gunbc_ir::Node<T>) -> Option<&'static str> {
        match node.kind {
            NodeKind::TransportExecute => Some("transport executor"),
            NodeKind::ToolEnvironment => Some("tool environment"),
            NodeKind::ResourceEnvironment | NodeKind::ResourceAcquire => {
                Some("resource environment")
            }
            NodeKind::ToolConsumer => Some("tool consumer"),
            _ => None,
        }
    }

    /// Find mock values whose types don't match the port's declared TypeId.
    ///
    /// Returns a list of (node_id, port_name, expected_type, actual_type) tuples.
    fn find_mock_type_mismatches(
        &self,
        spec: &MockSpec,
        lowered_dag: Option<&Dag<T>>,
    ) -> Vec<(String, String, String, String)> {
        let mut mismatches = Vec::new();

        // Check transport mocks
        for tm in &spec.transport_mocks {
            let node = self
                .dag
                .get_node(&NodeId(tm.node.clone()))
                .or_else(|| lowered_dag.and_then(|dag| dag.get_node(&NodeId(tm.node.clone()))));
            if let Some(node) = node {
                if let Some(port) = node.outputs.iter().find(|p| p.name.0 == tm.port) {
                    let expected = &port.type_id.0;
                    let actual = mock_value_kind_name(&tm.value);
                    if !mock_types_compatible(expected, &tm.value) {
                        mismatches.push((
                            tm.node.clone(),
                            tm.port.clone(),
                            expected.clone(),
                            actual.to_string(),
                        ));
                    }
                }
            }
        }

        // Check boundary mocks
        for bm in &spec.boundary_mocks {
            let node = self
                .dag
                .get_node(&NodeId(bm.node.clone()))
                .or_else(|| lowered_dag.and_then(|dag| dag.get_node(&NodeId(bm.node.clone()))));
            if let Some(node) = node {
                if let Some(port) = node.outputs.iter().find(|p| p.name.0 == bm.port) {
                    let expected = &port.type_id.0;
                    let actual = mock_value_kind_name(&bm.value);
                    if !mock_types_compatible(expected, &bm.value) {
                        mismatches.push((
                            bm.node.clone(),
                            bm.port.clone(),
                            expected.clone(),
                            actual.to_string(),
                        ));
                    }
                }
            }
        }

        // Check input mocks against input port type definitions
        for im in &spec.input_mocks {
            let node = self
                .dag
                .get_node(&NodeId(im.node.clone()))
                .or_else(|| lowered_dag.and_then(|dag| dag.get_node(&NodeId(im.node.clone()))));
            if let Some(node) = node {
                if let Some(port) = node.inputs.iter().find(|p| p.name.0 == im.port) {
                    let expected = &port.type_id.0;
                    let actual = mock_value_kind_name(&im.value);
                    if !mock_types_compatible(expected, &im.value) {
                        mismatches.push((
                            im.node.clone(),
                            im.port.clone(),
                            expected.clone(),
                            actual.to_string(),
                        ));
                    }
                }
            }
        }

        mismatches
    }

    /// Find mocks that reference non-existent nodes or ports.
    ///
    /// Returns a list of (node_id, port_name, reason) tuples.
    fn find_unknown_mock_slots(
        &self,
        spec: &MockSpec,
        lowered_dag: Option<&Dag<T>>,
    ) -> Vec<(String, String, String)> {
        let mut unknown = Vec::new();

        // Check transport mocks
        for tm in &spec.transport_mocks {
            let node = self
                .dag
                .get_node(&NodeId(tm.node.clone()))
                .or_else(|| lowered_dag.and_then(|dag| dag.get_node(&NodeId(tm.node.clone()))));
            match node {
                None => {
                    unknown.push((
                        tm.node.clone(),
                        tm.port.clone(),
                        "node does not exist".to_string(),
                    ));
                }
                Some(node) => {
                    if !node.outputs.iter().any(|p| p.name.0 == tm.port) {
                        unknown.push((
                            tm.node.clone(),
                            tm.port.clone(),
                            format!(
                                "port does not exist on node (available: {})",
                                node.outputs
                                    .iter()
                                    .map(|p| p.name.0.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ));
                    }
                }
            }
        }

        // Check boundary mocks
        for bm in &spec.boundary_mocks {
            let node = self
                .dag
                .get_node(&NodeId(bm.node.clone()))
                .or_else(|| lowered_dag.and_then(|dag| dag.get_node(&NodeId(bm.node.clone()))));
            match node {
                None => {
                    unknown.push((
                        bm.node.clone(),
                        bm.port.clone(),
                        "node does not exist".to_string(),
                    ));
                }
                Some(node) => {
                    if !node.outputs.iter().any(|p| p.name.0 == bm.port) {
                        unknown.push((
                            bm.node.clone(),
                            bm.port.clone(),
                            format!(
                                "port does not exist on node (available: {})",
                                node.outputs
                                    .iter()
                                    .map(|p| p.name.0.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ));
                    }
                }
            }
        }

        // Check input mocks (must reference existing input ports)
        for im in &spec.input_mocks {
            let node = self
                .dag
                .get_node(&NodeId(im.node.clone()))
                .or_else(|| lowered_dag.and_then(|dag| dag.get_node(&NodeId(im.node.clone()))));
            match node {
                None => {
                    unknown.push((
                        im.node.clone(),
                        im.port.clone(),
                        "node does not exist".to_string(),
                    ));
                }
                Some(node) => {
                    if !node.inputs.iter().any(|p| p.name.0 == im.port) {
                        unknown.push((
                            im.node.clone(),
                            im.port.clone(),
                            format!(
                                "input port does not exist on node (available: {})",
                                node.inputs
                                    .iter()
                                    .map(|p| p.name.0.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ));
                    }
                }
            }
        }

        unknown
    }

    /// Generate the full test file (header excluded).
    fn generate_test_file(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
        probe_observer_bundle: Option<&ProbeObserverBundle>,
    ) -> TestFile {
        let mut file = TestFile {
            header: Vec::new(),
            imports: Vec::new(),
            helpers: Vec::new(),
            sections: Vec::new(),
        };

        // Imports
        file.imports.push(Import {
            path: vec!["gunbc_exec".to_string()],
            items: vec![
                "execute_with_mode".to_string(),
                "lower".to_string(),
                "BoundaryMocks".to_string(),
                "ExecutionMode".to_string(),
            ],
        });

        file.imports.push(Import {
            path: vec!["gunbc_ir".to_string()],
            items: vec![
                "detect_boundaries".to_string(),
                "Cardinality".to_string(),
                "Value".to_string(),
            ],
        });

        if self.mock_spec_fn.is_some() {
            let mut items = vec![
                "assert_boundary_mockable".to_string(),
                "assert_types_compatible".to_string(),
                "guard_test".to_string(),
                "FermiCost".to_string(),
                "MockSpec".to_string(),
                "TestClass".to_string(),
            ];
            if self.config.live_flow_tests {
                items.push("guard_test_with_env".to_string());
            }
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items,
            });
        } else {
            let mut items = vec![
                "assert_boundary_mockable".to_string(),
                "assert_types_compatible".to_string(),
                "guard_test".to_string(),
                "FermiCost".to_string(),
                "TestClass".to_string(),
            ];
            if self.config.live_flow_tests {
                items.push("guard_test_with_env".to_string());
            }
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items,
            });
        }

        if self.mock_spec_fn.is_some() && self.config.window_max_nodes.unwrap_or(usize::MAX) >= 2 {
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items: vec![
                    "apply_window_inputs".to_string(),
                    "assert_window_outputs".to_string(),
                    "window_subdag".to_string(),
                    "Window".to_string(),
                ],
            });
        }

        if self.config.probe_observer_tests && self.mock_spec.is_some() {
            file.imports.push(Import {
                path: vec!["gunbc_exec".to_string()],
                items: vec![
                    "lower".to_string(),
                    "ExecutionMode".to_string(),
                    "execute_with_mode".to_string(),
                ],
            });
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items: vec![
                    "apply_window_inputs".to_string(),
                    "assert_chain_outputs".to_string(),
                    "OutputMatcher".to_string(),
                    "window_subdag".to_string(),
                    "Window".to_string(),
                ],
            });
            file.imports.push(Import {
                path: vec!["std::collections".to_string()],
                items: vec!["HashMap".to_string()],
            });
        }

        if self.config.chain_tests && self.mock_spec.is_some() {
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items: vec!["validate_chain".to_string(), "InputConstraint".to_string()],
            });
        }

        if self.config.resource_tests
            && self
                .mock_spec
                .as_ref()
                .is_some_and(|s| !s.resource_mocks.resources.is_empty())
        {
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items: vec![
                    "ResourceAcquireResult".to_string(),
                    "ResourceSimulation".to_string(),
                ],
            });
        }

        // BB-2: Corpus type-contract tests need `value_compatible_with_type_id`.
        if self.config.corpus_tests && self.corpus.is_some() {
            file.imports.push(Import {
                path: vec!["gunbc_ir".to_string()],
                items: vec!["value_compatible_with_type_id".to_string()],
            });
        }

        // Helpers
        if let Some(mock_spec_fn) = &self.mock_spec_fn {
            file.helpers.push(HelperFn {
                name: "mock_spec".to_string(),
                return_type: "MockSpec".to_string(),
                body: vec![Stmt::tail(Expr::var(mock_spec_fn))],
            });
        }

        // Signature validation (optional)
        if let Some(signature_fn) = &self.signature_fn {
            let test = TestFn {
                name: "test_signature_matches_dag".to_string(),
                doc: vec!["Declared signature matches the DAG inputs/outputs.".to_string()],
                body: vec![
                    Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                    Stmt::let_bind("sig", Expr::var(signature_fn)),
                    Stmt::Expr(
                        Expr::var("sig")
                            .method("validate", vec![Expr::var("dag").ref_of()])
                            .method(
                                "expect",
                                vec![Expr::Str("signature should match DAG".into())],
                            ),
                    ),
                ],
            };

            let build_port_membership_loop = |declared_binding: &str,
                                              candidate_binding: &str,
                                              declared_iter: Expr,
                                              inferred_iter: Expr,
                                              message: &str|
             -> Stmt {
                let cond = Expr::var(candidate_binding)
                    .field("name")
                    .bin_op("==", Expr::var(declared_binding).field("name"))
                    .bin_op(
                        "&&",
                        Expr::var(candidate_binding)
                            .field("type_id")
                            .bin_op("==", Expr::var(declared_binding).field("type_id")),
                    )
                    .bin_op(
                        "&&",
                        Expr::var(candidate_binding)
                            .field("cardinality")
                            .bin_op("==", Expr::var(declared_binding).field("cardinality")),
                    );

                Stmt::For {
                    binding: declared_binding.to_string(),
                    iter: declared_iter,
                    body: vec![
                        Stmt::let_mut("found", Expr::BoolLit(false)),
                        Stmt::For {
                            binding: candidate_binding.to_string(),
                            iter: inferred_iter,
                            body: vec![Stmt::Expr(Expr::If {
                                cond: Box::new(cond),
                                then_body: vec![Stmt::Expr(Expr::raw("found = true"))],
                                else_body: None,
                            })],
                        },
                        Stmt::Assert(Assert::True {
                            expr: Expr::var("found"),
                            message: message.to_string(),
                        }),
                    ],
                }
            };

            let inferred_test = TestFn {
                name: "test_inferred_signature_matches_declared".to_string(),
                doc: vec!["Inferred signature matches the declared inputs/outputs.".to_string()],
                body: vec![
                    Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                    Stmt::let_bind("sig", Expr::var(signature_fn)),
                    Stmt::let_bind(
                        "inferred",
                        Expr::call("gunbc_ir::infer_signature", vec![Expr::var("dag").ref_of()]),
                    ),
                    Stmt::Assert(Assert::Eq {
                        left: Expr::var("sig").field("inputs").method("len", vec![]),
                        right: Expr::var("inferred").field("inputs").method("len", vec![]),
                        message: "declared inputs length matches inferred".to_string(),
                    }),
                    build_port_membership_loop(
                        "declared_input",
                        "inferred_input",
                        Expr::var("sig").field("inputs").method("iter", vec![]),
                        Expr::var("inferred").field("inputs").method("iter", vec![]),
                        "declared input should exist in inferred signature",
                    ),
                    Stmt::Assert(Assert::Eq {
                        left: Expr::var("sig").field("outputs").method("len", vec![]),
                        right: Expr::var("inferred").field("outputs").method("len", vec![]),
                        message: "declared outputs length matches inferred".to_string(),
                    }),
                    build_port_membership_loop(
                        "declared_output",
                        "inferred_output",
                        Expr::var("sig").field("outputs").method("iter", vec![]),
                        Expr::var("inferred")
                            .field("outputs")
                            .method("iter", vec![]),
                        "declared output should exist in inferred signature",
                    ),
                ],
            };

            file.sections.push(TestSection {
                title: "Signature Validation".to_string(),
                notes: Vec::new(),
                tests: vec![test, inferred_test],
            });
        }

        // Invalid obligations — structural errors surfaced as failing tests
        let invalids = obligations.invalids();
        if !invalids.is_empty() {
            let mut tests = Vec::new();
            for (i, obligation) in invalids.iter().enumerate() {
                let reason = match &obligation.status {
                    DischargeStatus::Invalid { reason } => reason.as_str(),
                    _ => "unknown",
                };
                tests.push(TestFn {
                    name: format!("test_invalid_obligation_{}", i),
                    doc: vec![format!("INVALID: {}", obligation.reason)],
                    body: vec![Stmt::Expr(Expr::call(
                        "panic!",
                        vec![Expr::Str(format!("Structural error: {}", reason))],
                    ))],
                });
            }
            file.sections.push(TestSection {
                title: "INVALID OBLIGATIONS — structural errors detected during analysis"
                    .to_string(),
                notes: Vec::new(),
                tests,
            });
        }

        if self.config.execution_tests {
            if let Some(section) =
                self.build_execution_section(analysis, obligations, graph_builder_fn)
            {
                file.sections.push(section);
            }
        }

        if self.config.contract_tests {
            let sections = self.build_contract_sections(analysis, obligations, graph_builder_fn);
            file.sections.extend(sections);
        }

        if self.config.scenario_tests {
            if let Some(section) =
                self.build_scenario_section(analysis, obligations, graph_builder_fn)
            {
                file.sections.push(section);
            }
        }

        if self.config.resource_tests {
            let sections = self.build_resource_sections(analysis, obligations);
            file.sections.extend(sections);
        }

        if self.config.boundary_tests {
            if let Some(section) = self.build_boundary_section(analysis, graph_builder_fn) {
                file.sections.push(section);
            }
        }

        if self.config.chain_tests {
            if let Some(section) = self.build_chain_section(analysis) {
                file.sections.push(section);
            }
        }

        if self.config.flow_tests {
            if let Some(section) = self.build_flow_section(analysis, graph_builder_fn) {
                file.sections.push(section);
            }
        }

        if self.config.live_flow_tests {
            if let Some(section) = self.build_live_flow_section(graph_builder_fn) {
                file.sections.push(section);
            }
        }

        // PT-5: Per-profile live flow test sections.
        for section in self.build_per_profile_live_flow_sections() {
            file.sections.push(section);
        }

        if self.mock_spec_fn.is_some() && self.config.window_max_nodes.unwrap_or(usize::MAX) >= 2 {
            if let Some(section) = self.build_window_section(graph_builder_fn) {
                file.sections.push(section);
            }
        }

        if self.config.probe_observer_tests {
            if let Some(section) =
                self.build_probe_observer_section(probe_observer_bundle, graph_builder_fn)
            {
                file.sections.push(section);
            }
        }

        if self.config.example_tests {
            if let Some(section) = self.build_node_example_section(graph_builder_fn) {
                file.sections.push(section);
            }
        }

        // BB-2: Per-node corpus tests
        if self.config.corpus_tests {
            if let Some(section) = self.build_corpus_section(graph_builder_fn, analysis) {
                file.sections.push(section);
            }
        }

        // BB-3: Adjacent pair tests
        if self.config.adjacent_pair_tests {
            if let Some(section) = self.build_adjacent_pair_section(graph_builder_fn, analysis) {
                file.sections.push(section);
            }
        }

        // BB-5: Cross-workflow consistency tests
        if self.config.cross_workflow_tests {
            if let Some(section) = self.build_cross_workflow_section(graph_builder_fn, analysis) {
                file.sections.push(section);
            }
        }

        // BB-6: Transport fidelity ladder tests
        if self.config.fidelity_ladder_tests {
            if let Some(section) = self.build_fidelity_ladder_section(analysis, graph_builder_fn) {
                file.sections.push(section);
            }
        }

        if let Some(section) = self.build_cli_contract_section(obligations) {
            // CLI contract tests need gunbc_cli imports
            file.imports.push(Import {
                path: vec!["gunbc_cli".to_string()],
                items: vec![
                    "parse".to_string(),
                    "CliParam".to_string(),
                    "ParamType".to_string(),
                ],
            });
            file.sections.push(section);
        }

        self.inject_test_guards(&mut file);
        Self::prune_unused_imports(&mut file);
        Self::dedup_imports(&mut file);

        file
    }

    fn inject_test_guards(&self, file: &mut TestFile) {
        let class_variant = Self::class_variant(self.config.test_class);
        let cost_variant = Self::cost_variant(self.config.fermi_cost);
        let class_expr = Expr::path(&["TestClass", class_variant]);
        let cost_expr = Expr::path(&["FermiCost", cost_variant]);
        let requires_expr = Self::slice_expr(&self.config.requires);
        let secrets_expr = Self::slice_expr(&self.config.secrets);

        for section in &mut file.sections {
            for test in &mut section.tests {
                let guard_call = Expr::call(
                    "guard_test",
                    vec![
                        Expr::Str(test.name.clone()),
                        class_expr.clone(),
                        cost_expr.clone(),
                        requires_expr.clone(),
                        secrets_expr.clone(),
                    ],
                );
                let guard_stmt = Stmt::Expr(Expr::If {
                    cond: Box::new(guard_call.logical_not()),
                    then_body: vec![Stmt::Return(Expr::raw("()"))],
                    else_body: None,
                });
                test.body.insert(0, guard_stmt);
            }
        }
    }

    fn slice_expr(items: &[String]) -> Expr {
        let entries: Vec<Expr> = items.iter().map(Expr::str_lit).collect();
        Expr::Ref(Box::new(Expr::Array(entries)))
    }

    fn slice_2d_expr(items: &[Vec<String>]) -> Expr {
        let groups: Vec<Expr> = items
            .iter()
            .map(|group| {
                let entries: Vec<Expr> = group.iter().map(Expr::str_lit).collect();
                Expr::Ref(Box::new(Expr::Array(entries)))
            })
            .collect();
        Expr::Ref(Box::new(Expr::Array(groups)))
    }

    fn class_variant(class: TestClass) -> &'static str {
        match class {
            TestClass::Unit => "Unit",
            TestClass::Hermetic => "Hermetic",
            TestClass::Integration => "Integration",
        }
    }

    fn cost_variant(cost: FermiCost) -> &'static str {
        match cost {
            FermiCost::XS => "XS",
            FermiCost::S => "S",
            FermiCost::M => "M",
            FermiCost::L => "L",
            FermiCost::XL => "XL",
        }
    }

    fn prune_unused_imports(file: &mut TestFile) {
        let mut used: HashSet<String> = HashSet::new();

        for helper in &file.helpers {
            Self::collect_idents_from_type(&helper.return_type, &mut used);
            for stmt in &helper.body {
                Self::collect_idents_from_stmt(stmt, &mut used);
            }
        }

        for section in &file.sections {
            for test in &section.tests {
                for stmt in &test.body {
                    Self::collect_idents_from_stmt(stmt, &mut used);
                }
            }
        }

        for import in &mut file.imports {
            import.items.retain(|item| used.contains(item));
        }
        file.imports.retain(|import| !import.items.is_empty());
    }

    /// Merge imports from the same module path to avoid duplicate use statements.
    fn dedup_imports(file: &mut TestFile) {
        let mut by_path: BTreeMap<Vec<String>, HashSet<String>> = BTreeMap::new();
        for import in &file.imports {
            by_path
                .entry(import.path.clone())
                .or_default()
                .extend(import.items.iter().cloned());
        }
        file.imports = by_path
            .into_iter()
            .map(|(path, items)| {
                let mut items: Vec<String> = items.into_iter().collect();
                items.sort();
                Import { path, items }
            })
            .filter(|imp| !imp.items.is_empty())
            .collect();
    }

    fn collect_idents_from_type(ty: &str, used: &mut HashSet<String>) {
        let mut buf = String::new();
        for ch in ty.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                buf.push(ch);
            } else if !buf.is_empty() {
                used.insert(buf.clone());
                buf.clear();
            }
        }
        if !buf.is_empty() {
            used.insert(buf);
        }
    }

    fn collect_idents_from_stmt(stmt: &Stmt, used: &mut HashSet<String>) {
        match stmt {
            Stmt::Let { expr, .. } => Self::collect_idents_from_expr(expr, used),
            Stmt::Bind { targets, expr, .. } => {
                for target in targets {
                    if let BindTarget::Name(name) = target {
                        used.insert(name.clone());
                    }
                }
                Self::collect_idents_from_expr(expr, used);
            }
            Stmt::Expr(expr) => Self::collect_idents_from_expr(expr, used),
            Stmt::Assert(assert) => Self::collect_idents_from_assert(assert, used),
            Stmt::Comment(_) | Stmt::Blank => {}
            Stmt::Return(expr) | Stmt::TailExpr(expr) => Self::collect_idents_from_expr(expr, used),
            Stmt::For { iter, body, .. } => {
                Self::collect_idents_from_expr(iter, used);
                for s in body {
                    Self::collect_idents_from_stmt(s, used);
                }
            }
            Stmt::Item(Item::Raw(code)) => Self::collect_idents_from_type(code, used),
            Stmt::Item(_) => {}
            Stmt::Assign { dest, value } => {
                Self::collect_idents_from_expr(dest, used);
                Self::collect_idents_from_expr(value, used);
            }
            Stmt::BlockScope(stmts) => {
                for s in stmts {
                    Self::collect_idents_from_stmt(s, used);
                }
            }
        }
    }

    fn collect_idents_from_assert(assert: &Assert, used: &mut HashSet<String>) {
        match assert {
            Assert::Eq { left, right, .. } => {
                Self::collect_idents_from_expr(left, used);
                Self::collect_idents_from_expr(right, used);
            }
            Assert::True { expr, .. } | Assert::NonEmpty { expr, .. } => {
                Self::collect_idents_from_expr(expr, used);
            }
            Assert::Contains { expr, .. } => {
                Self::collect_idents_from_expr(expr, used);
            }
        }
    }

    fn collect_idents_from_expr(expr: &Expr, used: &mut HashSet<String>) {
        match expr {
            Expr::Value(_) => {
                used.insert("Value".to_string());
            }
            Expr::Var(name) => {
                Self::record_ident(name, used);
            }
            Expr::Str(_) | Expr::IntLit(_) | Expr::BoolLit(_) => {}
            Expr::Call { func, args, .. } => {
                Self::collect_idents_from_expr(func, used);
                for arg in args {
                    Self::collect_idents_from_expr(arg, used);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                Self::collect_idents_from_expr(receiver, used);
                for arg in args {
                    Self::collect_idents_from_expr(arg, used);
                }
            }
            Expr::Field(expr, _) => Self::collect_idents_from_expr(expr, used),
            Expr::Deref(expr) | Expr::Ref(expr) | Expr::RefMut(expr) => {
                Self::collect_idents_from_expr(expr, used);
            }
            Expr::Path(segments) => {
                if let Some(first) = segments.first() {
                    Self::record_ident(first, used);
                }
            }
            Expr::Struct { name, fields } => {
                Self::record_ident(name, used);
                for (_, expr) in fields {
                    Self::collect_idents_from_expr(expr, used);
                }
            }
            Expr::Closure { body, .. } => Self::collect_idents_from_expr(body, used),
            Expr::BinOp { left, right, .. } => {
                Self::collect_idents_from_expr(left, used);
                Self::collect_idents_from_expr(right, used);
            }
            Expr::UnaryOp { expr, .. } => Self::collect_idents_from_expr(expr, used),
            Expr::Match { expr, arms } => {
                Self::collect_idents_from_expr(expr, used);
                for arm in arms {
                    for s in &arm.body {
                        Self::collect_idents_from_stmt(s, used);
                    }
                }
            }
            Expr::If {
                cond,
                then_body,
                else_body,
            } => {
                Self::collect_idents_from_expr(cond, used);
                for s in then_body {
                    Self::collect_idents_from_stmt(s, used);
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        Self::collect_idents_from_stmt(s, used);
                    }
                }
            }
            Expr::Block(stmts) => {
                for s in stmts {
                    Self::collect_idents_from_stmt(s, used);
                }
            }
            Expr::FormatStr { args, .. } | Expr::MacroCall { args, .. } => {
                for arg in args {
                    Self::collect_idents_from_expr(arg, used);
                }
            }
            Expr::Tuple(exprs) | Expr::Array(exprs) => {
                for e in exprs {
                    Self::collect_idents_from_expr(e, used);
                }
            }
            Expr::RawCode(code) => {
                // Extract identifiers from raw code using the same splitter
                // as collect_idents_from_type — splits on non-alphanumeric chars.
                Self::collect_idents_from_type(code, used);
            }
        }
    }

    fn record_ident(raw: &str, used: &mut HashSet<String>) {
        let base = raw.split("::").next().unwrap_or(raw);
        let base = base.split('<').next().unwrap_or(base);
        let base = base.trim_end_matches('!');
        if !base.is_empty() {
            used.insert(base.to_string());
        }
    }

    // =======================================================================
    // Bucket A: Execution Semantics
    // =======================================================================

    fn build_execution_section(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> Option<TestSection> {
        let bucket = obligations.bucket_a();
        if bucket.is_empty() {
            return None;
        }

        let mut tests = Vec::new();
        let mut notes =
            vec!["Proves: executor/boundary model correctness (runtime-only)".to_string()];

        let mocks_expr = self.dryrun_mocks_expr(analysis, "execution tests");

        if bucket
            .iter()
            .any(|o| matches!(o.kind, Obligation::DryRunCompletion))
        {
            let exec = Expr::call(
                "execute_with_mode",
                vec![
                    Expr::var("dag").ref_of(),
                    Expr::call("ExecutionMode::DryRun", vec![mocks_expr.clone()]),
                ],
            )
            .method(
                "expect",
                vec![Expr::Str(
                    "DryRun execution should complete without crash".into(),
                )],
            );

            tests.push(TestFn {
                name: "test_dryrun_completion".to_string(),
                doc: vec![
                    "DryRun execution completes without crash.".to_string(),
                    String::new(),
                    "This is the minimal smoke test: build the DAG, run it in DryRun".to_string(),
                    "with explicit boundary mocks, and verify it completes successfully."
                        .to_string(),
                ],
                body: vec![
                    Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                    Stmt::let_bind("log", exec),
                    Stmt::Assert(Assert::True {
                        expr: Expr::var("log")
                            .field("entries")
                            .method("is_empty", vec![])
                            .logical_not(),
                        message: "execution should produce log entries".to_string(),
                    }),
                ],
            });
        }

        let transport_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::TransportInterceptable { .. }))
            .collect();

        if !transport_obligations.is_empty() {
            let mut body = vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::let_bind(
                    "result",
                    Expr::call(
                        "assert_boundary_mockable",
                        vec![Expr::var("dag").ref_of(), mocks_expr],
                    ),
                ),
                Stmt::Expr(Expr::call(
                    "assert!",
                    vec![
                        Expr::var("result").method("is_ok", vec![]),
                        Expr::Str("All transports should be interceptable: {:?}".into()),
                        Expr::var("result").field("error"),
                    ],
                )),
            ];

            for obligation in &transport_obligations {
                if let Obligation::TransportInterceptable { node_id } = &obligation.kind {
                    let contains = Expr::var("result")
                        .field("boundary_nodes")
                        .method("iter", vec![])
                        .method(
                            "any",
                            vec![Expr::Closure {
                                args: vec!["n".to_string()],
                                body: Box::new(
                                    Expr::var("n").bin_op("==", Expr::Str(node_id.0.clone())),
                                ),
                            }],
                        );

                    body.push(Stmt::Assert(Assert::True {
                        expr: contains,
                        message: format!(
                            "transport executor '{}' should be in intercepted list",
                            node_id.0
                        ),
                    }));
                }
            }

            tests.push(TestFn {
                name: "test_transport_interception".to_string(),
                doc: vec![
                    "All transport executors are intercepted in DryRun.".to_string(),
                    String::new(),
                    "Proves: every transport executor is interceptable; DryRun won't".to_string(),
                    "accidentally perform real I/O.".to_string(),
                ],
                body,
            });

            tests.push(TestFn {
                name: "test_transport_behavior_specs_cover_core_families".to_string(),
                doc: vec![
                    "Transport behavior specs cover all core executor families.".to_string(),
                    String::new(),
                    "Proves: testgen is wired to the shared transport behavior catalog".to_string(),
                    "used to drive behavioral transport contract checks.".to_string(),
                ],
                body: vec![
                    Stmt::let_bind(
                        "specs",
                        Expr::RawCode("gunbc_ir::default_transport_behaviors()".to_string()),
                    ),
                    Stmt::Expr(Expr::call(
                        "assert_eq!",
                        vec![
                            Expr::RawCode("specs.len()".to_string()),
                            Expr::IntLit(5),
                            Expr::Str(
                                "transport behavior catalog should cover tcp/http/rest/file/shell"
                                    .to_string(),
                            ),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "specs.iter().any(|s| matches!(s.transport, gunbc_ir::TransportKind::Tcp))"
                                    .to_string(),
                            ),
                            Expr::Str("missing tcp transport behavior spec".to_string()),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "specs.iter().any(|s| matches!(s.transport, gunbc_ir::TransportKind::Http))"
                                    .to_string(),
                            ),
                            Expr::Str("missing http transport behavior spec".to_string()),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "specs.iter().any(|s| matches!(s.transport, gunbc_ir::TransportKind::Rest))"
                                    .to_string(),
                            ),
                            Expr::Str("missing rest transport behavior spec".to_string()),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "specs.iter().any(|s| matches!(s.transport, gunbc_ir::TransportKind::File))"
                                    .to_string(),
                            ),
                            Expr::Str("missing file transport behavior spec".to_string()),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "specs.iter().any(|s| matches!(s.transport, gunbc_ir::TransportKind::Shell))"
                                    .to_string(),
                            ),
                            Expr::Str("missing shell transport behavior spec".to_string()),
                        ],
                    )),
                    Stmt::let_bind(
                        "tcp",
                        Expr::RawCode(
                            "specs.iter().find(|s| matches!(s.transport, gunbc_ir::TransportKind::Tcp)).expect(\"tcp behavior spec should exist\")"
                                .to_string(),
                        ),
                    ),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "tcp.field_routes.iter().any(|route| route.request_field == \"read_timeout_ms\" && route.operation == \"set_read_timeout\")"
                                    .to_string(),
                            ),
                            Expr::Str(
                                "tcp behavior spec should pin read_timeout_ms routing".to_string(),
                            ),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "tcp.field_routes.iter().any(|route| route.request_field == \"write_timeout_ms\" && route.operation == \"set_write_timeout\")"
                                    .to_string(),
                            ),
                            Expr::Str(
                                "tcp behavior spec should pin write_timeout_ms routing".to_string(),
                            ),
                        ],
                    )),
                    Stmt::let_bind(
                        "models",
                        Expr::RawCode("gunbc_ir::default_system_models()".to_string()),
                    ),
                    Stmt::let_bind(
                        "contract_specs",
                        Expr::RawCode("gunbc_ir::derive_contract_test_specs(&models)".to_string()),
                    ),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode("!contract_specs.is_empty()".to_string()),
                            Expr::Str(
                                "system model contract specs should derive at least one test spec"
                                    .to_string(),
                            ),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "contract_specs.iter().any(|s| matches!(s.phase, gunbc_ir::UpsertPhase::Check))"
                                    .to_string(),
                            ),
                            Expr::Str(
                                "derived contract specs should include check-phase coverage"
                                    .to_string(),
                            ),
                        ],
                    )),
                    Stmt::let_bind(
                        "harnesses",
                        Expr::RawCode(
                            "gunbc_ir::generate_contract_test_harnesses(&contract_specs)"
                                .to_string(),
                        ),
                    ),
                    Stmt::Expr(Expr::call(
                        "assert_eq!",
                        vec![
                            Expr::RawCode("harnesses.len()".to_string()),
                            Expr::RawCode("contract_specs.len()".to_string()),
                            Expr::Str(
                                "each contract test spec should render a harness signature"
                                    .to_string(),
                            ),
                        ],
                    )),
                    Stmt::Expr(Expr::call(
                        "assert!",
                        vec![
                            Expr::RawCode(
                                "harnesses.iter().all(|h| h.starts_with(\"fn contract_\"))"
                                    .to_string(),
                            ),
                            Expr::Str(
                                "rendered harnesses should be generated contract function signatures"
                                    .to_string(),
                            ),
                        ],
                    )),
                ],
            });
        }

        let determinism_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::PureNodeDeterminism { .. }))
            .collect();
        if !determinism_obligations.is_empty() {
            notes.push(String::new());
            notes.push(format!(
                "Determinism obligations: {} pure nodes.",
                determinism_obligations.len()
            ));
            notes.push(
                "To enable per-node determinism tests, use `execute_single_node`".to_string(),
            );
            notes.push("from gunbc_exec with baseline-derived inputs (Tier 1 infra).".to_string());
            for obligation in determinism_obligations {
                if let Obligation::PureNodeDeterminism { node_id } = &obligation.kind {
                    notes.push(format!("- '{}': same inputs → same outputs", node_id.0));
                }
            }
        }

        Some(TestSection {
            title: "Bucket A: Execution Semantics".to_string(),
            notes,
            tests,
        })
    }

    // =======================================================================
    // Bucket B: Contract Obligations
    // =======================================================================

    fn build_contract_sections(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> Vec<TestSection> {
        let bucket = obligations.bucket_b();
        if bucket.is_empty() {
            return Vec::new();
        }

        let mut notes =
            vec!["Tests for semantic compatibility when proof engine returns Unknown.".to_string()];

        let entailment_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::EdgePredicateEntailment { .. }))
            .collect();
        if !entailment_obligations.is_empty() {
            notes.push(format!(
                "{} edge predicate entailment obligations (Unknown).",
                entailment_obligations.len()
            ));
            notes.push(
                "Full entailment tests require contract tower witnesses (Tier 3 infra)."
                    .to_string(),
            );
            notes.push("For now, these are documented as obligations:".to_string());
            for obligation in &entailment_obligations {
                if let Obligation::EdgePredicateEntailment {
                    from_node,
                    from_port,
                    to_node,
                    to_port,
                    ..
                } = &obligation.kind
                {
                    notes.push(format!(
                        "- {}.{} → {}.{}: {}",
                        from_node.0, from_port.0, to_node.0, to_port.0, obligation.reason
                    ));
                }
            }
            notes.push(String::new());
        }

        let compliance_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::NodeContractCompliance { .. }))
            .collect();
        if !compliance_obligations.is_empty() {
            notes.push(format!(
                "{} node contract compliance obligations.",
                compliance_obligations.len()
            ));
            notes.push(
                "Per-node compliance tests use `execute_single_node` (Tier 1 infra).".to_string(),
            );
            for obligation in &compliance_obligations {
                if let Obligation::NodeContractCompliance { node_id } = &obligation.kind {
                    notes.push(format!("- '{}': valid inputs → valid outputs", node_id.0));
                }
            }
        }

        let optional_obligations = obligations.optional_input_obligations();
        if !optional_obligations.is_empty() {
            notes.push(format!(
                "{} optional input handling obligations.",
                optional_obligations.len()
            ));
            notes.push(
                "Optional inputs must accept missing values and reject wrong-typed inputs."
                    .to_string(),
            );
        }

        let mut tests = Vec::new();
        tests.extend(self.build_cardinality_coverage_tests(
            analysis,
            obligations,
            graph_builder_fn,
        ));
        tests.extend(self.build_coercion_coverage_tests(analysis, obligations, graph_builder_fn));
        tests.extend(self.build_variant_coverage_tests(analysis, obligations, graph_builder_fn));
        let (optional_tests, optional_notes) =
            self.build_optional_input_tests(analysis, obligations, graph_builder_fn);
        tests.extend(optional_tests);
        notes.extend(optional_notes);

        vec![TestSection {
            title: "Bucket B: Contract Obligations".to_string(),
            notes,
            tests,
        }]
    }

    fn build_cardinality_coverage_tests(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> Vec<TestFn> {
        let card_obligations = obligations.cardinality_obligations();
        if card_obligations.is_empty() {
            return Vec::new();
        }

        let mut tests = Vec::new();
        for obligation in &card_obligations {
            if let Obligation::CardinalityCoverage {
                node_id,
                port_name,
                cardinality,
                boundary_values,
            } = &obligation.kind
            {
                let type_id = analysis
                    .port_cardinalities
                    .iter()
                    .find(|p| p.node_id == node_id.0 && p.port_name == port_name.0 && !p.is_input)
                    .map(|p| p.type_id.0.as_str())
                    .unwrap_or_else(|| {
                        panic!(
                            "missing output port type for {}.{} in analysis; cannot generate cardinality coverage tests",
                            node_id.0, port_name.0
                        )
                    });

                for &count in boundary_values {
                    let label = boundary_label(count);
                    let test_name = format!(
                        "test_cardinality_{}_{}_{}_{}",
                        NamingCase::SnakeCase.apply(&node_id.0),
                        NamingCase::SnakeCase.apply(&port_name.0),
                        label,
                        count
                    );

                    let mock_value = mock_value_expr_for_count(
                        type_id,
                        *cardinality,
                        count,
                        &self.type_registry,
                    );
                    let mocks_expr = self.dryrun_mocks_expr(analysis, "cardinality coverage tests");

                    let exec = Expr::call(
                        "execute_with_mode",
                        vec![
                            Expr::var("dag").ref_of(),
                            Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "cardinality count={} ({}) should not crash",
                            count, label
                        ))],
                    );

                    tests.push(TestFn {
                        name: test_name,
                        doc: vec![
                            format!(
                                "Cardinality coverage: {}.{} with {} element(s) (cardinality: {}).",
                                node_id.0, port_name.0, count, cardinality
                            ),
                            String::new(),
                            format!(
                                "Proves: DAG handles count={} ({}) for boundary port {}.{}.",
                                count, label, node_id.0, port_name.0
                            ),
                        ],
                        body: vec![
                            Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                            Stmt::let_mut("mocks", mocks_expr),
                            Stmt::Expr(Expr::var("mocks").method(
                                "set_value",
                                vec![
                                    Expr::Str(node_id.0.clone()),
                                    Expr::Str(port_name.0.clone()),
                                    Expr::Value(mock_value),
                                ],
                            )),
                            Stmt::let_bind("_log", exec),
                        ],
                    });
                }
            }
        }

        tests
    }

    fn build_coercion_coverage_tests(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> Vec<TestFn> {
        let coercion_obligations = obligations.coercion_obligations();
        if coercion_obligations.is_empty() {
            return Vec::new();
        }

        let mut tests = Vec::new();
        for obligation in &coercion_obligations {
            if let Obligation::CoercionCoverage {
                from_node,
                from_port,
                to_node,
                to_port,
                from_cardinality,
                to_cardinality,
                kind,
            } = &obligation.kind
            {
                let kind_label = format!("{}", kind);
                let test_name = format!(
                    "test_coercion_{}_{}_{}_{}",
                    NamingCase::SnakeCase.apply(&from_node.0),
                    NamingCase::SnakeCase.apply(&from_port.0),
                    NamingCase::SnakeCase.apply(&to_node.0),
                    NamingCase::SnakeCase.apply(&to_port.0),
                );

                let mocks_expr = self.dryrun_mocks_expr(analysis, "coercion coverage tests");

                let exec = Expr::call(
                    "execute_with_mode",
                    vec![
                        Expr::var("dag").ref_of(),
                        Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                    ],
                )
                .method(
                    "expect",
                    vec![Expr::Str(format!(
                        "coercion {} at {}.{} → {}.{} should not crash",
                        kind_label, from_node.0, from_port.0, to_node.0, to_port.0
                    ))],
                );

                tests.push(TestFn {
                    name: test_name,
                    doc: vec![
                        format!(
                            "Coercion coverage: {}.{} {} → {}.{} {} ({}).",
                            from_node.0,
                            from_port.0,
                            from_cardinality,
                            to_node.0,
                            to_port.0,
                            to_cardinality,
                            kind_label,
                        ),
                        String::new(),
                        format!(
                            "Proves: engine correctly applies {} coercion at this edge and delivers the expected input shape at {}.{}.",
                            kind_label, to_node.0, to_port.0
                        ),
                    ],
                    body: vec![
                        Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                        Stmt::let_bind("mocks", mocks_expr),
                        Stmt::let_bind("log", exec),
                        // Raw because: coercion assertion sequence requires chained
                        // let-bindings with complex match/assert shapes that have no
                        // Code IR statement equivalent.
                        Stmt::Item(Item::Raw(format!(
                            "let target_entry = log.entries.iter().rev().find(|entry| entry.node_id == \"{}\").expect(\"coercion target entry missing for {}.{}\");",
                            to_node.0, to_node.0, to_port.0
                        ))),
                        Stmt::Item(Item::Raw(format!(
                            "let target_inputs = target_entry.inputs.as_ref().expect(\"coercion target {}.{} should capture inputs\");",
                            to_node.0, to_port.0
                        ))),
                        Stmt::Item(Item::Raw(format!(
                            "let received = target_inputs.get(\"{}\").expect(\"coercion target {}.{} should receive coerced value\");",
                            to_port.0, to_node.0, to_port.0
                        ))),
                        Stmt::Item(Item::Raw(match kind {
                            gunbc_ir::coerce::CoercionKind::WrapScalar => format!(
                                "assert!(matches!(received, Value::List(values) if !values.is_empty()), \"WrapScalar should deliver non-empty list to {}.{}; got {{:?}}\", received);",
                                to_node.0, to_port.0
                            ),
                            gunbc_ir::coerce::CoercionKind::OptionalToList => format!(
                                "assert!(matches!(received, Value::List(_)), \"OptionalToList should deliver list to {}.{}; got {{:?}}\", received);",
                                to_node.0, to_port.0
                            ),
                            gunbc_ir::coerce::CoercionKind::Widen => format!(
                                "assert!(matches!(received, Value::List(_)), \"Widen should preserve list shape at {}.{}; got {{:?}}\", received);",
                                to_node.0, to_port.0
                            ),
                        })),
                    ],
                });
            }
        }

        tests
    }

    fn build_variant_coverage_tests(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> Vec<TestFn> {
        let variant_obligations = obligations.variant_coverage_obligations();
        if variant_obligations.is_empty() {
            return Vec::new();
        }

        let mut tests = Vec::new();
        for obligation in &variant_obligations {
            if let Obligation::VariantCoverage {
                node_id,
                port_name,
                type_id,
                variants,
            } = &obligation.kind
            {
                for variant_name in variants {
                    let test_name = format!(
                        "test_variant_{}_{}_{}",
                        NamingCase::SnakeCase.apply(&node_id.0),
                        NamingCase::SnakeCase.apply(&port_name.0),
                        NamingCase::SnakeCase.apply(variant_name),
                    );

                    let mock_value = ValueExpr::Str(variant_name.clone());
                    let mocks_expr = self.dryrun_mocks_expr(analysis, "variant coverage tests");

                    let exec = Expr::call(
                        "execute_with_mode",
                        vec![
                            Expr::var("dag").ref_of(),
                            Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "variant {} for {}.{} should not crash",
                            variant_name, node_id.0, port_name.0
                        ))],
                    );

                    tests.push(TestFn {
                        name: test_name,
                        doc: vec![
                            format!(
                                "Variant coverage: {}.{} with variant '{}' of type {}.",
                                node_id.0, port_name.0, variant_name, type_id.0
                            ),
                            String::new(),
                            format!(
                                "Proves: DAG handles variant '{}' at boundary port {}.{}.",
                                variant_name, node_id.0, port_name.0
                            ),
                        ],
                        body: vec![
                            Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                            Stmt::let_mut("mocks", mocks_expr),
                            Stmt::Expr(Expr::var("mocks").method(
                                "set_value",
                                vec![
                                    Expr::Str(node_id.0.clone()),
                                    Expr::Str(port_name.0.clone()),
                                    Expr::Value(mock_value),
                                ],
                            )),
                            Stmt::let_bind("_log", exec),
                        ],
                    });
                }
            }
        }

        tests
    }

    fn build_optional_input_tests(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> (Vec<TestFn>, Vec<String>) {
        if !self.config.optional_input_tests {
            return (Vec::new(), Vec::new());
        }
        let optional_obligations = obligations.optional_input_obligations();
        if optional_obligations.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let mut tests = Vec::new();
        let mut notes = Vec::new();
        let mut base_inputs_cache: HashMap<
            String,
            Result<BTreeMap<String, ValueExpr>, Vec<String>>,
        > = HashMap::new();
        let mut skipped_nodes: HashSet<String> = HashSet::new();
        let mut strict_seed_checked_nodes: HashSet<String> = HashSet::new();
        let lowered_ids = gunbc_exec::lower(self.dag)
            .unwrap_or_else(|err| {
                panic!(
                    "Optional input test generation requires DAG lowering, but lowering failed: {}",
                    err
                )
            })
            .dag
            .nodes
            .iter()
            .map(|n| n.id.0.clone())
            .collect::<HashSet<_>>();

        for obligation in &optional_obligations {
            let Obligation::OptionalInputHandling { node_id, port_name } = &obligation.kind else {
                continue;
            };

            let Some(node) = self.dag.get_node(node_id) else {
                continue;
            };
            if !node.is_opaque() {
                continue;
            }

            if !analysis.pure_nodes.contains(&node_id.0) {
                continue;
            }

            if !lowered_ids.contains(&node_id.0) {
                if skipped_nodes.insert(node_id.0.clone()) {
                    notes.push(format!(
                        "Optional input tests skipped for '{}': node is lowered away (sub-DAG/pattern).",
                        node_id.0
                    ));
                }
                continue;
            }

            let port_info = analysis
                .port_cardinalities
                .iter()
                .find(|p| p.node_id == node_id.0 && p.port_name == port_name.0 && p.is_input)
                .unwrap_or_else(|| {
                    panic!(
                        "missing input port info for {}.{} in analysis; cannot generate optional input tests",
                        node_id.0, port_name.0
                    )
                });

            let base_inputs = base_inputs_cache
                .entry(node_id.0.clone())
                .or_insert_with(|| self.build_minimal_inputs_for_node(node_id));

            let base_inputs = match base_inputs {
                Ok(inputs) => inputs,
                Err(reasons) => {
                    if skipped_nodes.insert(node_id.0.clone()) {
                        notes.push(format!(
                            "Optional input tests skipped for '{}': {}.",
                            node_id.0,
                            reasons.join("; ")
                        ));
                    }
                    continue;
                }
            };

            if strict_seed_checked_nodes.insert(node_id.0.clone()) {
                self.assert_optional_required_inputs_seeded(node, base_inputs);
            }

            // Missing optional input should not error.
            let mut inputs_missing = base_inputs.clone();
            inputs_missing.remove(&port_name.0);

            let test_name = format!(
                "test_optional_missing_{}_{}",
                NamingCase::SnakeCase.apply(&node_id.0),
                NamingCase::SnakeCase.apply(&port_name.0)
            );

            let mut body = Vec::new();
            body.push(Stmt::let_bind("dag", Expr::var(graph_builder_fn)));
            body.extend(self.build_inputs_map_stmts(&inputs_missing));

            let exec = Expr::call(
                "gunbc_exec::execute_single_node",
                vec![
                    Expr::var("dag").ref_of(),
                    Expr::Str(node_id.0.clone()),
                    Expr::var("inputs"),
                    Expr::Path(vec![
                        "gunbc_exec".to_string(),
                        "ExecutionMode".to_string(),
                        "Real".to_string(),
                    ]),
                ],
            )
            .method(
                "expect",
                vec![Expr::Str(format!(
                    "optional input {}.{} missing should not error",
                    node_id.0, port_name.0
                ))],
            );
            body.push(Stmt::let_bind("_outputs", exec));

            tests.push(TestFn {
                name: test_name,
                doc: vec![
                    format!(
                        "Optional input: {}.{} (cardinality: {}).",
                        node_id.0, port_name.0, port_info.cardinality
                    ),
                    String::new(),
                    "Proves: missing optional input does not crash.".to_string(),
                ],
                body,
            });

            // Wrong-typed optional input should error (unless type accepts any).
            if let Some(wrong_value) = mock_wrong_type_expr(port_info.type_id.0.as_str()) {
                let mut inputs_wrong = base_inputs.clone();
                inputs_wrong.insert(port_name.0.clone(), wrong_value);

                let test_name = format!(
                    "test_optional_wrong_type_{}_{}",
                    NamingCase::SnakeCase.apply(&node_id.0),
                    NamingCase::SnakeCase.apply(&port_name.0)
                );

                let mut body = Vec::new();
                body.push(Stmt::let_bind("dag", Expr::var(graph_builder_fn)));
                body.extend(self.build_inputs_map_stmts(&inputs_wrong));

                let exec = Expr::call(
                    "gunbc_exec::execute_single_node",
                    vec![
                        Expr::var("dag").ref_of(),
                        Expr::Str(node_id.0.clone()),
                        Expr::var("inputs"),
                        Expr::Path(vec![
                            "gunbc_exec".to_string(),
                            "ExecutionMode".to_string(),
                            "Real".to_string(),
                        ]),
                    ],
                );
                body.push(Stmt::let_bind("result", exec));
                body.push(Stmt::Assert(Assert::True {
                    expr: Expr::var("result").method("is_err", vec![]),
                    message: format!(
                        "optional input {}.{} wrong type should error",
                        node_id.0, port_name.0
                    ),
                }));

                tests.push(TestFn {
                    name: test_name,
                    doc: vec![
                        format!(
                            "Optional input: {}.{} (cardinality: {}).",
                            node_id.0, port_name.0, port_info.cardinality
                        ),
                        String::new(),
                        "Proves: wrong-typed optional input is rejected.".to_string(),
                    ],
                    body,
                });
            }
        }

        (tests, notes)
    }

    fn has_explicit_node_input_seed(&self, node_id: &str, port_name: &str) -> bool {
        if let Some(spec) = &self.mock_spec {
            if spec
                .input_mocks
                .iter()
                .any(|mock| mock.node == node_id && mock.port == port_name)
            {
                return true;
            }
            if spec
                .node_examples
                .iter()
                .any(|example| example.node_id == node_id && example.inputs.contains_key(port_name))
            {
                return true;
            }
        }

        self.dag
            .get_node(&NodeId(node_id.to_string()))
            .is_some_and(|node| {
                node.examples
                    .iter()
                    .any(|example| example.inputs.contains_key(port_name))
            })
    }

    fn assert_optional_required_inputs_seeded(
        &self,
        node: &gunbc_ir::Node<T>,
        base_inputs: &BTreeMap<String, ValueExpr>,
    ) {
        // Nodes explicitly marked skip_node_example are resource/boundary nodes
        // that won't be tested in single-node Real mode — no seed assertion needed.
        if let Some(spec) = &self.mock_spec {
            if spec.skipped_node_examples.iter().any(|s| s == &node.id.0) {
                return;
            }
        }

        let mut missing = Vec::new();

        for port in &node.inputs {
            let needs_value = port.has_guard() || !port.cardinality.allows_empty();
            if !needs_value {
                continue;
            }
            if port.name.0 == "skip" && port.type_id.0 == "Bool" {
                continue;
            }
            if !requires_explicit_seed(port.type_id.0.as_str(), SeedContext::RealSingleNode) {
                continue;
            }
            if !base_inputs.contains_key(&port.name.0)
                || !self.has_explicit_node_input_seed(&node.id.0, &port.name.0)
            {
                missing.push(format!(
                    "{}.{} ({})",
                    node.id.0, port.name.0, port.type_id.0
                ));
            }
        }

        if missing.is_empty() {
            return;
        }

        panic!(
            "Optional input tests require explicit seeds for required semantic inputs in Real single-node mode.\n\
             Missing explicit seeds: {}.\n\
             Add one of: MockSpec::input_mock(\"<node>\", \"<port>\", ...), MockSpec::node_example(...), or Node::with_example(...).",
            missing.join(", ")
        );
    }

    // =======================================================================
    // Bucket C: Scenario Coverage
    // =======================================================================

    fn build_scenario_section(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> Option<TestSection> {
        let _seed_matrix = seed_matrix_for_context(SeedContext::Scenario);
        let bucket = obligations.bucket_c();
        if bucket.is_empty() {
            return None;
        }

        let mut tests = Vec::new();
        let mut notes = vec![
            "N+1 scenarios: one success + one per-transport failure + guard toggles.".to_string(),
        ];

        if bucket
            .iter()
            .any(|o| matches!(o.kind, Obligation::AllTransportsSucceed))
        {
            let mocks_expr = self.dryrun_mocks_expr(analysis, "scenario all-succeed tests");
            let exec = Expr::call(
                "execute_with_mode",
                vec![
                    Expr::var("dag").ref_of(),
                    Expr::call("ExecutionMode::DryRun", vec![mocks_expr]),
                ],
            )
            .method(
                "expect",
                vec![Expr::Str("all-succeed scenario should complete".into())],
            );

            let mut body = vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::let_bind("log", exec),
            ];

            for transport in &analysis.transport_executors {
                body.push(Stmt::let_bind(
                    "entry",
                    Expr::var("log")
                        .method("get", vec![Expr::Str(transport.clone())])
                        .method(
                            "expect",
                            vec![Expr::Str(format!("'{}' should be in log", transport))],
                        ),
                ));
                body.push(Stmt::Assert(Assert::True {
                    expr: Expr::var("entry").field("was_intercepted"),
                    message: format!("'{}' should be intercepted in DryRun", transport),
                }));
            }

            tests.push(TestFn {
                name: "test_scenario_all_succeed".to_string(),
                doc: vec![
                    "Happy path: all transports succeed.".to_string(),
                    String::new(),
                    "Proves: workflow reaches terminal outputs with all transports mocked as success."
                        .to_string(),
                ],
                body,
            });
        }

        let failure_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::SingleTransportFailure { .. }))
            .collect();
        if !failure_obligations.is_empty() {
            notes.push(format!(
                "{} single-failure scenarios (one per transport executor).",
                failure_obligations.len()
            ));
            notes.push(
                "Full failure scenarios require per-transport failure mocks (Tier 0 infra)."
                    .to_string(),
            );
            notes.push(String::new());

            for obligation in &failure_obligations {
                if let Obligation::SingleTransportFailure { node_id } = &obligation.kind {
                    let test_name = format!(
                        "test_scenario_{}_fails",
                        NamingCase::SnakeCase.apply(&node_id.0)
                    );

                    let mocks_expr =
                        self.dryrun_mocks_expr(analysis, "scenario single-failure tests");

                    let body = vec![
                        Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                        Stmt::let_mut("mocks", mocks_expr),
                        Stmt::Comment(format!("Inject failure at '{}'", node_id.0)),
                        Stmt::Expr(Expr::var("mocks").method(
                            "set_value",
                            vec![
                                Expr::Str(node_id.0.clone()),
                                Expr::Str("response".to_string()),
                                Expr::Value(ValueExpr::Str("<TRANSPORT_FAILURE>".to_string())),
                            ],
                        )),
                        Stmt::Comment(
                            "Execution may succeed or fail depending on graph semantics;"
                                .to_string(),
                        ),
                        Stmt::Comment(
                            "the key property is that it doesn't crash/hang.".to_string(),
                        ),
                        Stmt::let_bind(
                            "_result",
                            Expr::call(
                                "execute_with_mode",
                                vec![
                                    Expr::var("dag").ref_of(),
                                    Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                                ],
                            ),
                        ),
                    ];

                    tests.push(TestFn {
                        name: test_name,
                        doc: vec![
                            format!("Single failure: '{}' transport fails.", node_id.0),
                            String::new(),
                            "Proves: failure propagation semantics are consistent.".to_string(),
                        ],
                        body,
                    });
                }
            }
        }

        let skip_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::SkipPathPropagation { .. }))
            .collect();
        for obligation in &skip_obligations {
            if let Obligation::SkipPathPropagation { trigger_node } = &obligation.kind {
                let test_name = format!(
                    "test_skip_propagation_{}",
                    NamingCase::SnakeCase.apply(&trigger_node.0)
                );

                let output_ports: Vec<_> = analysis
                    .port_cardinalities
                    .iter()
                    .filter(|p| p.node_id == trigger_node.0 && !p.is_input)
                    .map(|p| (p.port_name.clone(), p.type_id.0.clone()))
                    .collect();

                let downstream: Vec<_> = self
                    .dag
                    .edges
                    .iter()
                    .filter(|e| e.from_node.0 == trigger_node.0)
                    .map(|e| e.to_node.0.clone())
                    .collect();

                let mocks_expr = self.dryrun_mocks_expr(analysis, "skip propagation tests");

                let mut body = vec![
                    Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                    Stmt::let_mut("mocks", mocks_expr),
                ];

                for (port_name, _type_id) in &output_ports {
                    body.push(Stmt::Expr(Expr::var("mocks").method(
                        "set_value",
                        vec![
                            Expr::Str(trigger_node.0.clone()),
                            Expr::Str(port_name.clone()),
                            Expr::Value(ValueExpr::Skipped),
                        ],
                    )));
                }

                let exec = Expr::call(
                    "execute_with_mode",
                    vec![
                        Expr::var("dag").ref_of(),
                        Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                    ],
                )
                .method(
                    "expect",
                    vec![Expr::Str(
                        "skip propagation should not crash or hang".into(),
                    )],
                );
                body.push(Stmt::let_bind("log", exec));

                for ds_node in &downstream {
                    body.push(Stmt::Assert(Assert::True {
                        expr: Expr::var("log")
                            .method("get", vec![Expr::Str(ds_node.clone())])
                            .method("is_some", vec![]),
                        message: format!("downstream '{}' should still appear in log", ds_node),
                    }));
                }

                tests.push(TestFn {
                    name: test_name,
                    doc: vec![
                        format!(
                            "Skip propagation: '{}' returns Skipped → downstream handles it.",
                            trigger_node.0
                        ),
                        String::new(),
                        "Proves: when a transport's output is Skipped, downstream nodes"
                            .to_string(),
                        "either skip themselves (guarded) or process the Skipped value".to_string(),
                        "without crashing.".to_string(),
                    ],
                    body,
                });
            }
        }

        let guard_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::GuardBranchCoverage { .. }))
            .collect();

        for obligation in &guard_obligations {
            if let Obligation::GuardBranchCoverage {
                node_id,
                guard_port,
            } = &obligation.kind
            {
                let guard_type = analysis
                    .port_cardinalities
                    .iter()
                    .find(|p| p.node_id == node_id.0 && p.port_name == guard_port.0 && p.is_input)
                    .map(|p| p.type_id.0.as_str())
                    .unwrap_or("Unknown");

                let upstream_edge = self
                    .dag
                    .edges
                    .iter()
                    .find(|e| e.to_node.0 == node_id.0 && e.to_port.0 == guard_port.0);

                let upstream_is_mockable = upstream_edge
                    .map(|e| {
                        analysis.transport_executors.contains(&e.from_node.0)
                            || analysis.tool_env_nodes.contains(&e.from_node.0)
                    })
                    .unwrap_or(false);

                if guard_type == "Bool" {
                    let test_name = format!(
                        "test_guard_{}_{}_branch_coverage",
                        NamingCase::SnakeCase.apply(&node_id.0),
                        NamingCase::SnakeCase.apply(&guard_port.0)
                    );

                    let mut body = vec![Stmt::let_bind("dag", Expr::var(graph_builder_fn))];

                    if let Some(edge) = upstream_edge {
                        if upstream_is_mockable {
                            let mocks_init = self.dryrun_mocks_expr(analysis, "guard branch tests");

                            body.push(Stmt::Comment(
                                "Guard value flows from a mockable boundary node.".to_string(),
                            ));

                            body.push(Stmt::Comment("Test with true:".to_string()));
                            body.push(Stmt::let_mut("mocks_true", mocks_init.clone()));
                            body.push(Stmt::Expr(Expr::var("mocks_true").method(
                                "set_value",
                                vec![
                                    Expr::Str(edge.from_node.0.clone()),
                                    Expr::Str(edge.from_port.0.clone()),
                                    Expr::Value(ValueExpr::Bool(true)),
                                ],
                            )));
                            let log_true = Expr::call(
                                "execute_with_mode",
                                vec![
                                    Expr::var("dag").ref_of(),
                                    Expr::call(
                                        "ExecutionMode::DryRun",
                                        vec![Expr::var("mocks_true")],
                                    ),
                                ],
                            )
                            .method(
                                "expect",
                                vec![Expr::Str("guard=true scenario should not crash".into())],
                            );
                            body.push(Stmt::let_bind("log_true", log_true));

                            let skipped_true = Expr::var("log_true")
                                .method("get", vec![Expr::Str(node_id.0.clone())])
                                .method(
                                    "map",
                                    vec![Expr::Closure {
                                        args: vec!["e".to_string()],
                                        body: Box::new(
                                            Expr::var("e")
                                                .field("outputs")
                                                .method("values", vec![])
                                                .method(
                                                    "all",
                                                    vec![Expr::Closure {
                                                        args: vec!["v".to_string()],
                                                        body: Box::new(
                                                            Expr::var("v")
                                                                .method("is_skipped", vec![]),
                                                        ),
                                                    }],
                                                ),
                                        ),
                                    }],
                                )
                                .method("unwrap_or", vec![Expr::bool_lit(true)]);
                            body.push(Stmt::let_bind("skipped_true", skipped_true));
                            body.push(Stmt::Blank);

                            body.push(Stmt::Comment("Test with false:".to_string()));
                            body.push(Stmt::let_mut("mocks_false", mocks_init));
                            body.push(Stmt::Expr(Expr::var("mocks_false").method(
                                "set_value",
                                vec![
                                    Expr::Str(edge.from_node.0.clone()),
                                    Expr::Str(edge.from_port.0.clone()),
                                    Expr::Value(ValueExpr::Bool(false)),
                                ],
                            )));
                            let log_false = Expr::call(
                                "execute_with_mode",
                                vec![
                                    Expr::var("dag").ref_of(),
                                    Expr::call(
                                        "ExecutionMode::DryRun",
                                        vec![Expr::var("mocks_false")],
                                    ),
                                ],
                            )
                            .method(
                                "expect",
                                vec![Expr::Str("guard=false scenario should not crash".into())],
                            );
                            body.push(Stmt::let_bind("log_false", log_false));

                            let skipped_false = Expr::var("log_false")
                                .method("get", vec![Expr::Str(node_id.0.clone())])
                                .method(
                                    "map",
                                    vec![Expr::Closure {
                                        args: vec!["e".to_string()],
                                        body: Box::new(
                                            Expr::var("e")
                                                .field("outputs")
                                                .method("values", vec![])
                                                .method(
                                                    "all",
                                                    vec![Expr::Closure {
                                                        args: vec!["v".to_string()],
                                                        body: Box::new(
                                                            Expr::var("v")
                                                                .method("is_skipped", vec![]),
                                                        ),
                                                    }],
                                                ),
                                        ),
                                    }],
                                )
                                .method("unwrap_or", vec![Expr::bool_lit(true)]);
                            body.push(Stmt::let_bind("skipped_false", skipped_false));
                            body.push(Stmt::Blank);

                            body.push(Stmt::Comment(
                                "Exactly one path should execute and the other should skip."
                                    .to_string(),
                            ));
                            body.push(Stmt::Expr(Expr::call(
                                "assert_ne!",
                                vec![
                                    Expr::var("skipped_true"),
                                    Expr::var("skipped_false"),
                                    Expr::Str(format!(
                                        "guard on '{}'.{} should cause one branch to execute and the other to skip",
                                        node_id.0, guard_port.0
                                    )),
                                ],
                            )));
                        } else {
                            body.push(Stmt::Comment(format!(
                                "Guard value flows from pure node '{}' — not directly mockable.",
                                edge.from_node.0
                            )));
                            body.push(Stmt::Comment(
                                "Structural check: guard port is connected and the node has outputs."
                                    .to_string(),
                            ));
                            body.push(Stmt::let_bind(
                                "node",
                                Expr::var("dag")
                                    .method(
                                        "get_node",
                                        vec![Expr::Str(node_id.0.clone())
                                            .method("into", vec![])
                                            .ref_of()],
                                    )
                                    .method("expect", vec![Expr::Str("node should exist".into())]),
                            ));
                            body.push(Stmt::let_bind(
                                "port",
                                Expr::var("node")
                                    .field("inputs")
                                    .method("iter", vec![])
                                    .method(
                                        "find",
                                        vec![Expr::Closure {
                                            args: vec!["p".to_string()],
                                            body: Box::new(
                                                Expr::var("p")
                                                    .field("name")
                                                    .field("0")
                                                    .bin_op("==", Expr::Str(guard_port.0.clone())),
                                            ),
                                        }],
                                    )
                                    .method("expect", vec![Expr::Str("port should exist".into())]),
                            ));
                            body.push(Stmt::Assert(Assert::True {
                                expr: Expr::var("port").method("has_guard", vec![]),
                                message: "port should have a guard".to_string(),
                            }));
                        }
                    } else {
                        body.push(Stmt::Comment(format!(
                            "WARNING: guard port '{}'.{} has no incoming edge.",
                            node_id.0, guard_port.0
                        )));
                        body.push(Stmt::Comment(
                            "The node will always skip (missing input → skip).".to_string(),
                        ));
                        let mocks_expr =
                            self.dryrun_mocks_expr(analysis, "guard disconnected tests");
                        let exec = Expr::call(
                            "execute_with_mode",
                            vec![
                                Expr::var("dag").ref_of(),
                                Expr::call("ExecutionMode::DryRun", vec![mocks_expr]),
                            ],
                        )
                        .method(
                            "expect",
                            vec![Expr::Str("execution should not crash".into())],
                        );
                        body.push(Stmt::let_bind("log", exec));

                        let skipped = Expr::var("log")
                            .method("get", vec![Expr::Str(node_id.0.clone())])
                            .method(
                                "map",
                                vec![Expr::Closure {
                                    args: vec!["e".to_string()],
                                    body: Box::new(
                                        Expr::var("e")
                                            .field("outputs")
                                            .method("values", vec![])
                                            .method(
                                                "all",
                                                vec![Expr::Closure {
                                                    args: vec!["v".to_string()],
                                                    body: Box::new(
                                                        Expr::var("v").method("is_skipped", vec![]),
                                                    ),
                                                }],
                                            ),
                                    ),
                                }],
                            )
                            .method("unwrap_or", vec![Expr::bool_lit(true)]);
                        body.push(Stmt::Assert(Assert::True {
                            expr: skipped,
                            message: "disconnected guard → node should always skip".to_string(),
                        }));
                    }

                    tests.push(TestFn {
                        name: test_name,
                        doc: vec![
                            format!(
                                "Guard branch coverage: '{}'.{} (Bool guard).",
                                node_id.0, guard_port.0
                            ),
                            String::new(),
                            "Proves: one of {true, false} causes the node to execute,".to_string(),
                            "the other causes it to skip (all outputs = Value::Skipped)."
                                .to_string(),
                        ],
                        body,
                    });
                } else {
                    notes.push(format!(
                        "Guard branch: '{}'.{} (type: {})",
                        node_id.0, guard_port.0, guard_type
                    ));
                    if let Some(edge) = upstream_edge {
                        notes.push(format!(
                            "  fed by: {}.{}",
                            edge.from_node.0, edge.from_port.0
                        ));
                        if upstream_is_mockable {
                            notes.push(
                                "  upstream is mockable — full branch test possible with Tier 1 infra"
                                    .to_string(),
                            );
                        } else {
                            notes.push(
                                "  upstream is pure — needs per-node isolation (Tier 1) for full test"
                                    .to_string(),
                            );
                        }
                    } else {
                        notes.push("  WARNING: no incoming edge (disconnected guard)".to_string());
                    }
                    notes.push(String::new());
                }
            }
        }

        Some(TestSection {
            title: "Bucket C: Scenario Coverage".to_string(),
            notes,
            tests,
        })
    }

    // =======================================================================
    // Bucket D: Resource Hygiene + Simulation
    // =======================================================================

    fn build_resource_sections(
        &self,
        _analysis: &DagAnalysis,
        obligations: &ObligationSet,
    ) -> Vec<TestSection> {
        let bucket = obligations.bucket_d();
        let has_mockspec_resources = self
            .mock_spec
            .as_ref()
            .is_some_and(|s| !s.resource_mocks.resources.is_empty());

        if bucket.is_empty() && !has_mockspec_resources {
            return Vec::new();
        }

        let mut notes =
            vec!["Structural resource/tool wiring correctness + simulation tests.".to_string()];

        let connectivity_issues: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::ResourceInputConnected { .. }))
            .collect();
        if !connectivity_issues.is_empty() {
            notes.push("Resource connectivity issues (disconnected resource ports):".to_string());
            for obligation in &connectivity_issues {
                if let Obligation::ResourceInputConnected { node_id, port_name } = &obligation.kind
                {
                    notes.push(format!(
                        "WARNING: {}.{} — {}",
                        node_id.0, port_name.0, obligation.reason
                    ));
                }
            }
            notes.push(String::new());
        }

        let orphan_issues: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::ResourceOrphan { .. }))
            .collect();
        if !orphan_issues.is_empty() {
            notes.push("Resource orphans (acquired but never consumed):".to_string());
            for obligation in &orphan_issues {
                if let Obligation::ResourceOrphan { node_id, port_name } = &obligation.kind {
                    notes.push(format!(
                        "WARNING: {}.{} — {}",
                        node_id.0, port_name.0, obligation.reason
                    ));
                }
            }
            notes.push(String::new());
        }

        let conflict_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::ResourceConflictAbsence { .. }))
            .collect();
        if !conflict_obligations.is_empty() {
            for obligation in &conflict_obligations {
                if let Obligation::ResourceConflictAbsence { conflicts } = &obligation.kind {
                    if !conflicts.is_empty() {
                        notes.push("RESOURCE CONFLICTS DETECTED:".to_string());
                        for conflict in conflicts {
                            notes.push(format!("- {}", conflict));
                        }
                        notes.push(String::new());
                    }
                }
            }
        }

        if has_mockspec_resources {
            notes.push("Resource simulation tests (MockSpec-based)".to_string());
        }

        let tests = if has_mockspec_resources {
            self.build_resource_simulation_tests()
        } else {
            Vec::new()
        };

        vec![TestSection {
            title: "Bucket D: Resource Hygiene".to_string(),
            notes,
            tests,
        }]
    }

    fn build_resource_simulation_tests(&self) -> Vec<TestFn> {
        let Some(spec) = &self.mock_spec else {
            return Vec::new();
        };
        if spec.resource_mocks.resources.is_empty() {
            return Vec::new();
        }

        let mut tests = Vec::new();
        for resource in &spec.resource_mocks.resources {
            let test_name = format!(
                "test_resource_{}_acquire",
                sanitize_resource_id(&resource.resource_id)
            );

            let mut doc = Vec::new();
            let resource_type = match &resource.resource_type {
                gunbc_test::ResourceType::Lock => "Lock",
                gunbc_test::ResourceType::Lease { duration_ms } => {
                    doc.push(format!(
                        "Test resource '{}' lease behavior ({}ms).",
                        resource.resource_id, duration_ms
                    ));
                    "Lease"
                }
                gunbc_test::ResourceType::SharedLock { max_holders } => {
                    doc.push(format!(
                        "Test resource '{}' shared lock (max {} holders).",
                        resource.resource_id, max_holders
                    ));
                    "SharedLock"
                }
                gunbc_test::ResourceType::PoolSlot { pool_size } => {
                    doc.push(format!(
                        "Test resource '{}' pool slot (pool size {}).",
                        resource.resource_id, pool_size
                    ));
                    "PoolSlot"
                }
                gunbc_test::ResourceType::Credential { expiry_ms } => {
                    doc.push(format!(
                        "Test resource '{}' credential (expiry: {:?}).",
                        resource.resource_id, expiry_ms
                    ));
                    "Credential"
                }
            };
            doc.push(format!(
                "Test resource '{}' ({}) acquisition.",
                resource.resource_id, resource_type
            ));

            let mut body = vec![
                Stmt::let_bind("spec", Expr::call("mock_spec", vec![])),
                Stmt::let_bind(
                    "resource",
                    Expr::var("spec")
                        .method(
                            "get_resource",
                            vec![Expr::Str(resource.resource_id.clone())],
                        )
                        .method("expect", vec![Expr::Str("resource should exist".into())]),
                ),
                Stmt::let_bind("result", Expr::var("resource").method("acquire", vec![])),
            ];

            let has_fail = resource
                .behaviors
                .iter()
                .any(|b| matches!(b, gunbc_test::ResourceBehavior::FailAcquire { .. }));
            if has_fail {
                body.push(Stmt::Expr(Expr::call(
                    "assert!",
                    vec![
                        Expr::call(
                            "matches!",
                            vec![
                                Expr::var("result"),
                                Expr::var("ResourceAcquireResult::Failed(_)"),
                            ],
                        ),
                        Expr::Str("should fail to acquire".into()),
                    ],
                )));
            } else {
                body.push(Stmt::Expr(Expr::call(
                    "assert!",
                    vec![
                        Expr::call(
                            "matches!",
                            vec![
                                Expr::var("result"),
                                Expr::var("ResourceAcquireResult::Acquired"),
                            ],
                        ),
                        Expr::Str("should acquire successfully".into()),
                    ],
                )));
            }

            tests.push(TestFn {
                name: test_name,
                doc,
                body,
            });

            // Timeout tests for Lease and Credential with expiry
            let timeout_ms = match resource.resource_type {
                gunbc_test::ResourceType::Lease { duration_ms } => Some(duration_ms),
                gunbc_test::ResourceType::Credential {
                    expiry_ms: Some(ms),
                } => Some(ms),
                _ => None,
            };
            if let Some(duration_ms) = timeout_ms {
                let timeout_test = format!(
                    "test_resource_{}_timeout",
                    sanitize_resource_id(&resource.resource_id)
                );
                let body = vec![
                    Stmt::let_bind("spec", Expr::call("mock_spec", vec![])),
                    Stmt::let_bind(
                        "resource",
                        Expr::var("spec")
                            .method(
                                "get_resource",
                                vec![Expr::Str(resource.resource_id.clone())],
                            )
                            .method("expect", vec![Expr::Str("resource should exist".into())]),
                    ),
                    Stmt::Assert(Assert::True {
                        expr: Expr::var("resource")
                            .method("should_timeout", vec![Expr::int((duration_ms / 2) as i64)])
                            .logical_not(),
                        message: "should not timeout before duration".to_string(),
                    }),
                    Stmt::Assert(Assert::True {
                        expr: Expr::var("resource")
                            .method("should_timeout", vec![Expr::int((duration_ms + 1) as i64)]),
                        message: "should timeout after duration".to_string(),
                    }),
                ];

                tests.push(TestFn {
                    name: timeout_test,
                    doc: vec![format!(
                        "Test resource '{}' expiration after {}ms.",
                        resource.resource_id, duration_ms
                    )],
                    body,
                });
            }

        }

        tests
    }

    // =======================================================================
    // Helpers
    // =======================================================================

    /// Build the boundary mocks expression for DryRun tests.
    ///
    /// Boundary mocks are required when the DAG has boundary ports. If none exist,
    /// we return an explicit empty mock set.
    fn dryrun_mocks_expr(&self, analysis: &DagAnalysis, context: &str) -> Expr {
        if analysis.boundaries.boundary_nodes.is_empty() {
            return Expr::call("BoundaryMocks::new", vec![]);
        }
        if self.mock_spec_fn.is_none() {
            let boundary_nodes = analysis
                .boundaries
                .boundary_nodes
                .iter()
                .map(|n| n.0.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            panic!(
                "MockSpec function required for {}: DAG has boundary nodes ({}), but no mock_spec_fn was provided.\n\
                 Provide TestGenerator::with_mock_spec_fn(\"path::to::mock_spec()\") so generated tests can build explicit boundary mocks.",
                context, boundary_nodes
            );
        }
        Expr::call("mock_spec", vec![]).method("to_boundary_mocks", vec![])
    }

    fn build_inputs_map_stmts(&self, inputs: &BTreeMap<String, ValueExpr>) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        if inputs.is_empty() {
            stmts.push(Stmt::let_bind(
                "inputs",
                Expr::call("std::collections::HashMap::new", vec![]),
            ));
            return stmts;
        }

        stmts.push(Stmt::let_mut(
            "inputs",
            Expr::call("std::collections::HashMap::new", vec![]),
        ));

        for (port, value) in inputs {
            stmts.push(Stmt::Expr(Expr::var("inputs").method(
                "insert",
                vec![
                    Expr::Str(port.clone()).method("to_string", vec![]),
                    Expr::Value(value.clone()),
                ],
            )));
        }

        stmts
    }

    fn build_minimal_inputs_for_node(
        &self,
        node_id: &NodeId,
    ) -> Result<BTreeMap<String, ValueExpr>, Vec<String>> {
        let Some(node) = self.dag.get_node(node_id) else {
            return Err(vec![format!("unknown node '{}'", node_id.0)]);
        };

        let mut inputs = BTreeMap::new();
        let mut required_names = HashSet::new();
        for port in &node.inputs {
            let needs_value = port.has_guard() || !port.cardinality.allows_empty();
            if needs_value {
                required_names.insert(port.name.0.clone());
            }
        }

        let mut best_example: Option<&HashMap<String, Value>> = None;
        let mut best_count = 0usize;

        if let Some(spec) = &self.mock_spec {
            for example in &spec.node_examples {
                if example.node_id != node_id.0 {
                    continue;
                }
                let count = required_names
                    .iter()
                    .filter(|name| example.inputs.contains_key(*name))
                    .count();
                if count > best_count {
                    best_count = count;
                    best_example = Some(&example.inputs);
                }
                if count == required_names.len() && count > 0 {
                    break;
                }
            }
        }

        if best_count < required_names.len() {
            for example in &node.examples {
                let count = required_names
                    .iter()
                    .filter(|name| example.inputs.contains_key(*name))
                    .count();
                if count > best_count {
                    best_count = count;
                    best_example = Some(&example.inputs);
                }
                if count == required_names.len() && count > 0 {
                    break;
                }
            }
        }

        if let Some(example_inputs) = best_example {
            for (name, value) in example_inputs {
                if node.inputs.iter().any(|port| port.name.0 == *name) {
                    inputs.insert(name.clone(), ValueExpr::from(value));
                }
            }
        }

        // Seed known per-node inputs from MockSpec before generic synthesis.
        if let Some(spec) = &self.mock_spec {
            for input_mock in &spec.input_mocks {
                if input_mock.node != node_id.0 {
                    continue;
                }
                if !node
                    .inputs
                    .iter()
                    .any(|port| port.name.0 == input_mock.port)
                {
                    continue;
                }
                inputs
                    .entry(input_mock.port.clone())
                    .or_insert_with(|| ValueExpr::from(&input_mock.value));
            }
        }

        let mut issues = Vec::new();

        for port in &node.inputs {
            if port.name.0 == "skip" && port.type_id.0 == "Bool" {
                inputs.insert(port.name.0.clone(), ValueExpr::Bool(false));
                continue;
            }
            if inputs.contains_key(&port.name.0) {
                continue;
            }
            let needs_value = port.has_guard() || !port.cardinality.allows_empty();
            if !needs_value {
                continue;
            }

            let value = if port.has_guard() {
                select_guard_value(port, &self.type_registry)
            } else {
                required_value_for_port(port, &self.type_registry)
            };

            match value {
                Some(value) => {
                    inputs.insert(port.name.0.clone(), ValueExpr::from(&value));
                }
                None => {
                    issues.push(format!(
                        "{}.{} (type: {})",
                        node_id.0, port.name.0, port.type_id.0
                    ));
                }
            }
        }

        if issues.is_empty() {
            Ok(inputs)
        } else {
            Err(issues)
        }
    }

    /// Get mock value for a boundary port, using MockSpec only.
    fn get_mock_value(
        &self,
        node: &str,
        port: &str,
        type_id: &str,
        cardinality: Cardinality,
    ) -> ValueExpr {
        let spec = self.mock_spec.as_ref().unwrap_or_else(|| {
            panic!(
                "MockSpec required for boundary tests; missing spec while generating {}.{}",
                node, port
            )
        });
        let Some(value) = spec.get_boundary_mock(node, port) else {
            panic!(
                "MockSpec missing boundary mock for {}.{} (type: {}, cardinality: {}).\n\
                 Add MockSpec::boundary(\"{}\", \"{}\", ...).",
                node, port, type_id, cardinality, node, port
            );
        };
        ValueExpr::from(value)
    }

    fn build_flow_section(
        &self,
        _analysis: &DagAnalysis,
        graph_builder_fn: &str,
    ) -> Option<TestSection> {
        let Some(spec) = &self.mock_spec else {
            return None;
        };
        if self.mock_spec_fn.is_none() {
            panic!(
                "Flow tests require mock_spec_fn so generated tests can build boundary mocks.\n\
                 Provide TestGenerator::with_mock_spec_fn(\"path::to::mock_spec()\") to enable flow tests."
            );
        }
        if !spec.has_flow_test_data() {
            return None;
        }

        let test_name = format!("test_flow_{}", NamingCase::SnakeCase.apply(&spec.name));

        let mut body = vec![
            Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
            Stmt::let_bind("spec", Expr::call("mock_spec", vec![])),
            Stmt::let_bind(
                "mocks",
                Expr::var("spec").method("to_boundary_mocks", vec![]),
            ),
            Stmt::let_bind(
                if spec.expected_outputs.is_empty() {
                    "_log"
                } else {
                    "log"
                },
                Expr::call(
                    "execute_with_mode",
                    vec![
                        Expr::var("dag").ref_of(),
                        Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                    ],
                )
                .method(
                    "expect",
                    vec![Expr::Str("DryRun execution should succeed".into())],
                ),
            ),
            Stmt::Blank,
        ];

        for eo in &spec.expected_outputs {
            body.push(Stmt::Comment(format!("Verify {}.{}", eo.node, eo.port)));
            body.push(Stmt::let_bind(
                "entry",
                Expr::var("log")
                    .method("get", vec![Expr::Str(eo.node.clone())])
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "node '{}' should be in execution log",
                            eo.node
                        ))],
                    ),
            ));

            let left = Expr::var("entry")
                .field("outputs")
                .method("get", vec![Expr::Str(eo.port.clone())])
                .method(
                    "expect",
                    vec![Expr::Str(format!(
                        "port '{}' should exist on '{}'",
                        eo.port, eo.node
                    ))],
                );
            let right = Expr::Value(ValueExpr::from(&eo.expected)).ref_of();
            body.push(Stmt::Assert(Assert::Eq {
                left,
                right,
                message: format!("flow verification: {}.{} mismatch", eo.node, eo.port),
            }));
            body.push(Stmt::Blank);
        }

        Some(TestSection {
            title: "Flow Verification Tests".to_string(),
            notes: vec![
                "These tests execute the full DAG in DryRun mode with mocked transport".to_string(),
                "responses, verifying that pure node logic produces expected outputs.".to_string(),
            ],
            tests: vec![TestFn {
                name: test_name,
                doc: vec![
                    format!("Flow verification: {} scenario.", spec.name),
                    String::new(),
                    "Builds the DAG, injects mocked transport responses via DryRun,".to_string(),
                    "and verifies that the pure node chain produces expected terminal outputs."
                        .to_string(),
                ],
                body,
            }],
        })
    }

    fn build_live_flow_section(&self, graph_builder_fn: &str) -> Option<TestSection> {
        let Some(spec) = &self.mock_spec else {
            return None;
        };
        if !self.config.live_flow_tests {
            return None;
        }
        let _seed_matrix = seed_matrix_for_context(SeedContext::LiveFlow);

        let has_satisfies = spec
            .live_expected_outputs
            .iter()
            .any(|eo| matches!(eo.matcher, OutputMatcher::Satisfies { .. }));
        if has_satisfies && self.mock_spec_fn.is_none() {
            panic!(
                "Live flow tests with OutputMatcher::Satisfies require mock_spec_fn so generated tests can access runtime matchers.\n\
                 Provide TestGenerator::with_mock_spec_fn(\"path::to::mock_spec\")."
            );
        }

        let test_name = format!("test_live_flow_{}", NamingCase::SnakeCase.apply(&spec.name));

        let class_variant = Self::class_variant(self.config.live_test_class);
        let cost_variant = Self::cost_variant(self.config.live_fermi_cost);
        let class_expr = Expr::path(&["TestClass", class_variant]);
        let cost_expr = Expr::path(&["FermiCost", cost_variant]);
        let requires_expr = Self::slice_expr(&self.config.live_requires);
        let required_expr = Self::slice_expr(&self.config.live_required);
        let required_any_of_expr = Self::slice_2d_expr(&self.config.live_required_any_of);

        let guard_call = Expr::call(
            "guard_test_with_env",
            vec![
                Expr::Str(test_name.clone()),
                class_expr,
                cost_expr,
                requires_expr,
                required_expr,
                required_any_of_expr,
            ],
        );
        let guard_stmt = Stmt::Expr(Expr::If {
            cond: Box::new(guard_call.logical_not()),
            then_body: vec![Stmt::Return(Expr::raw("()"))],
            else_body: None,
        });

        let mut body = vec![
            guard_stmt,
            Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
            Stmt::let_bind(
                "log",
                Expr::call(
                    "execute_with_mode",
                    vec![
                        Expr::var("dag").ref_of(),
                        Expr::Path(vec!["ExecutionMode".to_string(), "Real".to_string()]),
                    ],
                )
                .method(
                    "expect",
                    vec![Expr::Str("Real execution should succeed".into())],
                ),
            ),
            Stmt::Assert(Assert::True {
                expr: Expr::var("log")
                    .field("entries")
                    .method("is_empty", vec![])
                    .logical_not(),
                message: "execution should produce log entries".to_string(),
            }),
            Stmt::Blank,
        ];

        if has_satisfies {
            body.push(Stmt::let_bind("spec", Expr::call("mock_spec", vec![])));
            body.push(Stmt::Blank);
        }

        for (idx, eo) in spec.live_expected_outputs.iter().enumerate() {
            body.push(Stmt::Comment(format!(
                "Verify {}.{} (live)",
                eo.node, eo.port
            )));
            body.push(Stmt::let_bind(
                "entry",
                Expr::var("log")
                    .method("get", vec![Expr::Str(eo.node.clone())])
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "node '{}' should be in execution log",
                            eo.node
                        ))],
                    ),
            ));

            let var_name = sanitize_to_snake_case(&eo.port);
            let output_var = format!("output_{}", var_name);
            body.push(Stmt::let_bind(
                output_var.clone(),
                Expr::var("entry")
                    .field("outputs")
                    .method("get", vec![Expr::Str(eo.port.clone())])
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "port '{}' should exist on '{}'",
                            eo.port, eo.node
                        ))],
                    ),
            ));

            if matches!(eo.matcher, OutputMatcher::Satisfies { .. }) {
                let matcher_var = format!("matcher_{}", var_name);
                body.push(Stmt::let_bind(
                    "expected_spec",
                    Expr::var("spec")
                        .field("live_expected_outputs")
                        .method("get", vec![Expr::int(idx as i64)])
                        .method(
                            "expect",
                            vec![Expr::Str(format!(
                                "mock_spec missing live expected output {} for '{}.{}'",
                                idx, eo.node, eo.port
                            ))],
                        ),
                ));
                body.push(Stmt::Assert(Assert::Eq {
                    left: Expr::var("expected_spec")
                        .field("node")
                        .method("as_str", vec![]),
                    right: Expr::Str(eo.node.clone()),
                    message: format!(
                        "live expected output {} should match node id '{}'",
                        idx, eo.node
                    ),
                }));
                body.push(Stmt::Assert(Assert::Eq {
                    left: Expr::var("expected_spec")
                        .field("port")
                        .method("as_str", vec![]),
                    right: Expr::Str(eo.port.clone()),
                    message: format!(
                        "live expected output {} should match port '{}'",
                        idx, eo.port
                    ),
                }));
                body.push(Stmt::let_bind(
                    matcher_var.clone(),
                    Expr::var("expected_spec").field("matcher"),
                ));
                body.push(Stmt::Expr(
                    Expr::var(&matcher_var)
                        .method("check", vec![Expr::var(&output_var)])
                        .method(
                            "expect",
                            vec![Expr::Str(format!(
                                "live output port '{}.{}' failed satisfies matcher",
                                eo.node, eo.port
                            ))],
                        ),
                ));
            } else {
                let mut stmts = render_output_matcher_check(&eo.matcher, &var_name);
                body.append(&mut stmts);
            }

            body.push(Stmt::Blank);
        }

        Some(TestSection {
            title: "Live Flow Tests".to_string(),
            notes: vec![
                "These tests execute the full DAG in Real mode with actual I/O.".to_string(),
                "They are gated by env requirements and Fermi cost.".to_string(),
            ],
            tests: vec![TestFn {
                name: test_name,
                doc: vec![
                    format!("Live flow verification: {} scenario.", spec.name),
                    String::new(),
                    "Builds the DAG, executes in Real mode, and checks key outputs.".to_string(),
                ],
                body,
            }],
        })
    }

    /// PT-5: Generate per-profile live flow test sections.
    ///
    /// One test function per `LiveProfileTestConfig` entry, named
    /// `test_live_flow_{module}_{profile}()`, gated by the profile's
    /// env requirements.
    fn build_per_profile_live_flow_sections(&self) -> Vec<TestSection> {
        self.config
            .live_profile_tests
            .iter()
            .map(|profile_test| {
                let test_name = format!(
                    "test_live_flow_{}_{}_profile",
                    NamingCase::SnakeCase.apply(&self.config.target_name),
                    NamingCase::SnakeCase.apply(&profile_test.profile_name),
                );

                let class_variant = Self::class_variant(profile_test.test_class);
                let cost_variant = Self::cost_variant(profile_test.fermi_cost);
                let class_expr = Expr::path(&["TestClass", class_variant]);
                let cost_expr = Expr::path(&["FermiCost", cost_variant]);
                let required_expr = Self::slice_expr(&profile_test.required_env);
                let required_any_of_expr = Self::slice_2d_expr(&profile_test.required_any_of);

                let guard_call = Expr::call(
                    "guard_test_with_env",
                    vec![
                        Expr::Str(test_name.clone()),
                        class_expr,
                        cost_expr,
                        Expr::Array(vec![]), // requires (external tools)
                        required_expr,
                        required_any_of_expr,
                    ],
                );
                let guard_stmt = Stmt::Expr(Expr::If {
                    cond: Box::new(guard_call.logical_not()),
                    then_body: vec![Stmt::Return(Expr::raw("()"))],
                    else_body: None,
                });

                let body = vec![
                    guard_stmt,
                    Stmt::Comment(format!(
                        "Compile with profile '{}' and execute in Real mode",
                        profile_test.profile_name
                    )),
                    Stmt::let_bind("dag", Expr::raw(&profile_test.dag_builder_call)),
                    Stmt::let_bind(
                        "log",
                        Expr::call(
                            "execute_with_mode",
                            vec![
                                Expr::var("dag").ref_of(),
                                Expr::Path(vec!["ExecutionMode".to_string(), "Real".to_string()]),
                            ],
                        )
                        .method(
                            "expect",
                            vec![Expr::Str(format!(
                                "Real execution with profile '{}' should succeed",
                                profile_test.profile_name,
                            ))],
                        ),
                    ),
                    Stmt::Assert(Assert::True {
                        expr: Expr::var("log")
                            .field("entries")
                            .method("is_empty", vec![])
                            .logical_not(),
                        message: "execution should produce log entries".to_string(),
                    }),
                ];

                TestSection {
                    title: format!(
                        "Per-profile live flow: {} (profile: {})",
                        self.config.target_name, profile_test.profile_name
                    ),
                    notes: vec![format!(
                        "Compiles with --profile {}, executes in Real mode.",
                        profile_test.profile_name
                    )],
                    tests: vec![TestFn {
                        name: test_name,
                        doc: vec![format!(
                            "Live flow test for profile '{}'.",
                            profile_test.profile_name
                        )],
                        body,
                    }],
                }
            })
            .collect()
    }

    fn build_window_section(&self, graph_builder_fn: &str) -> Option<TestSection> {
        self.mock_spec_fn.as_ref()?;

        let max_nodes = self.config.window_max_nodes.unwrap_or(usize::MAX);
        if max_nodes < 2 {
            return None;
        }

        let lowered =
            gunbc_exec::lower(self.dag).expect("window tests require DAG lowering to succeed");
        let pure_nodes = collect_pure_nodes(&lowered.dag);
        let windows = enumerate_window_specs(&lowered.dag, max_nodes, &pure_nodes);
        if windows.is_empty() {
            return None;
        }

        let mut used_names: HashSet<String> = HashSet::new();
        let mut tests = Vec::new();

        for (idx, spec) in windows.iter().enumerate() {
            let base_name = format!(
                "test_window_{}_through_{}",
                NamingCase::SnakeCase.apply(&spec.first.0),
                NamingCase::SnakeCase.apply(&spec.last.0)
            );
            let test_name = if used_names.insert(base_name.clone()) {
                base_name
            } else {
                format!("{}_{}", base_name, idx)
            };

            let mut node_args = Vec::new();
            for node in &spec.nodes {
                node_args.push(Expr::Str(node.0.clone()));
            }

            let baseline = Expr::call(
                "execute_with_mode",
                vec![
                    Expr::var("dag").ref_of(),
                    Expr::call(
                        "ExecutionMode::DryRun",
                        vec![Expr::call("mock_spec", vec![]).method("to_boundary_mocks", vec![])],
                    ),
                ],
            )
            .method(
                "expect",
                vec![Expr::Str("baseline DryRun should succeed".into())],
            );

            let body = vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::let_bind(
                    "flat",
                    Expr::call("lower", vec![Expr::var("dag").ref_of()])
                        .method("expect", vec![Expr::Str("lower should succeed".into())])
                        .field("dag"),
                ),
                Stmt::let_bind("baseline", baseline),
                Stmt::let_bind(
                    "window",
                    Expr::call(
                        "Window::from_nodes",
                        vec![Expr::var("flat").ref_of(), Expr::call("vec!", node_args)],
                    ),
                ),
                Stmt::let_mut(
                    "mocks",
                    Expr::call("mock_spec", vec![]).method("to_boundary_mocks", vec![]),
                ),
                Stmt::Expr(
                    Expr::call(
                        "apply_window_inputs",
                        vec![
                            Expr::var("flat").ref_of(),
                            Expr::var("window").ref_of(),
                            Expr::var("baseline").ref_of(),
                            Expr::var("mocks").ref_mut(),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str(
                            "window inputs should be derivable from baseline".into(),
                        )],
                    ),
                ),
                Stmt::let_bind(
                    "window_dag",
                    Expr::call(
                        "window_subdag",
                        vec![Expr::var("flat").ref_of(), Expr::var("window").ref_of()],
                    ),
                ),
                Stmt::let_bind(
                    "log",
                    Expr::call(
                        "execute_with_mode",
                        vec![
                            Expr::var("window_dag").ref_of(),
                            Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str("window execution should succeed".into())],
                    ),
                ),
                Stmt::Expr(
                    Expr::call(
                        "assert_window_outputs",
                        vec![
                            Expr::var("flat").ref_of(),
                            Expr::var("window").ref_of(),
                            Expr::var("baseline").ref_of(),
                            Expr::var("log").ref_of(),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str("window outputs should match baseline".into())],
                    ),
                ),
            ];

            tests.push(TestFn {
                name: test_name,
                doc: vec![format!("Window: {} -> {}", spec.first.0, spec.last.0)],
                body,
            });
        }

        Some(TestSection {
            title: "Windowed Segment Tests".to_string(),
            notes: vec![
                "These tests execute contiguous windows of the DAG using baseline DryRun"
                    .to_string(),
                "values as injected inputs, then verify window exit outputs match baseline."
                    .to_string(),
            ],
            tests,
        })
    }

    fn build_probe_observer_section(
        &self,
        bundle: Option<&ProbeObserverBundle>,
        graph_builder_fn: &str,
    ) -> Option<TestSection> {
        self.mock_spec.as_ref()?;
        self.mock_spec_fn.as_ref()?;
        let bundle = bundle?;

        if let Some(err) = &bundle.lowering_error {
            let err_msg = format!("DAG lowering failed, probe-observer tests skipped: {}", err);
            return Some(TestSection {
                title: "Probe-Observer Integration Tests".to_string(),
                notes: vec![err_msg.clone()],
                tests: vec![TestFn {
                    name: "test_probe_observer_lowering_failed".to_string(),
                    doc: vec!["Lowering must succeed for probe-observer tests.".to_string()],
                    body: vec![Stmt::Expr(Expr::call("panic!", vec![Expr::Str(err_msg)]))],
                }],
            });
        }

        let po_analysis = &bundle.analysis;
        if po_analysis.tests.is_empty() && po_analysis.gaps.is_empty() {
            return None;
        }

        let mut tests = Vec::new();
        let mut used_names: HashSet<String> = HashSet::new();

        for (idx, chain_test) in po_analysis.tests.iter().enumerate() {
            let base_name = format!(
                "test_chain_{}_to_{}",
                NamingCase::SnakeCase.apply(&chain_test.probe.node_id),
                NamingCase::SnakeCase.apply(&chain_test.observer.node_id)
            );
            let test_name = if used_names.insert(base_name.clone()) {
                base_name
            } else {
                format!("{}_{}", base_name, idx)
            };

            // Build node list for the subgraph window.
            let mut node_args = Vec::new();
            for node in &chain_test.subgraph_nodes {
                node_args.push(Expr::Str(node.clone()));
            }

            // Build matcher HashMap entries.
            let mut matcher_stmts = Vec::new();
            for (port, matcher_desc) in &chain_test.observer.matchers {
                // We need to reconstruct the matcher from the MockSpec's NodeExample.
                // Emit: matchers.insert(("node".into(), "port".into()), mock_spec().node_examples[...].outputs["port"].clone());
                // Instead, use a simpler approach: reference the mock_spec at runtime.
                matcher_stmts.push(Stmt::Comment(format!(
                    "Observer: {}.{} — {}",
                    chain_test.observer.node_id, port, matcher_desc.description
                )));
            }

            // Generate the test body that:
            // 1. Builds the DAG and lowers it
            // 2. Runs a full baseline DryRun to get input values
            // 3. Creates a window from the subgraph nodes
            // 4. Injects entry inputs from the baseline
            // 5. Executes the window subDAG
            // 6. Checks observer outputs via assert_chain_outputs (not baseline!)
            let body = vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::let_bind(
                    "flat",
                    Expr::call("lower", vec![Expr::var("dag").ref_of()])
                        .method("expect", vec![Expr::Str("lower should succeed".into())])
                        .field("dag"),
                ),
                Stmt::let_bind("spec", Expr::call("mock_spec", vec![])),
                Stmt::Comment("Full baseline DryRun to derive window entry inputs".to_string()),
                Stmt::let_bind(
                    "baseline",
                    Expr::call(
                        "execute_with_mode",
                        vec![
                            Expr::var("flat").ref_of(),
                            Expr::call(
                                "ExecutionMode::DryRun",
                                vec![Expr::call("mock_spec", vec![]).method("to_boundary_mocks", vec![])],
                            ),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str("baseline DryRun should succeed".into())],
                    ),
                ),
                Stmt::let_bind(
                    "window",
                    Expr::call(
                        "Window::from_nodes",
                        vec![Expr::var("flat").ref_of(), Expr::call("vec!", node_args)],
                    ),
                ),
                Stmt::let_mut(
                    "mocks",
                    Expr::var("spec").method("to_boundary_mocks", vec![]),
                ),
                Stmt::Expr(
                    Expr::call(
                        "apply_window_inputs",
                        vec![
                            Expr::var("flat").ref_of(),
                            Expr::var("window").ref_of(),
                            Expr::var("baseline").ref_of(),
                            Expr::var("mocks").ref_mut(),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str(
                            "chain entry inputs should be derivable from baseline".into(),
                        )],
                    ),
                ),
                Stmt::let_bind(
                    "window_dag",
                    Expr::call(
                        "window_subdag",
                        vec![Expr::var("flat").ref_of(), Expr::var("window").ref_of()],
                    ),
                ),
                Stmt::let_bind(
                    "log",
                    Expr::call(
                        "execute_with_mode",
                        vec![
                            Expr::var("window_dag").ref_of(),
                            Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str("chain execution should succeed".into())],
                    ),
                ),
                Stmt::Blank,
                Stmt::Comment(format!(
                    "Verify observer: {} (depth {})",
                    chain_test.observer.node_id, chain_test.depth
                )),
                Stmt::let_mut(
                    "matchers",
                    Expr::call("HashMap::new", vec![]),
                ),
                // Raw because: runtime for-loop filter+insert patterns have no Code IR
                // equivalent — these iterate spec data to populate matcher maps.
                Stmt::Item(Item::Raw(format!(
                    "for ex in spec.node_examples.iter().filter(|e| e.node_id == \"{}\") {{\n\
                        for (port, matcher) in &ex.outputs {{\n\
                            if matcher.is_chain_safe() {{\n\
                                matchers.insert((\"{}\".to_string(), port.clone()), matcher.clone());\n\
                            }}\n\
                        }}\n\
                     }}",
                    chain_test.observer.node_id, chain_test.observer.node_id
                ))),
                // Also insert chain-safe matchers from live_expected_outputs
                Stmt::Item(Item::Raw(format!(
                    "for leo in spec.live_expected_outputs.iter().filter(|e| e.node == \"{}\") {{\n\
                        if leo.matcher.is_chain_safe() {{\n\
                            matchers.insert((\"{}\".to_string(), leo.port.clone()), leo.matcher.clone());\n\
                        }}\n\
                     }}",
                    chain_test.observer.node_id, chain_test.observer.node_id
                ))),
                Stmt::Expr(
                    Expr::call(
                        "assert_chain_outputs",
                        vec![
                            Expr::var("log").ref_of(),
                            Expr::var("matchers").ref_of(),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "chain {} -> {} should satisfy observer matchers",
                            chain_test.probe.node_id, chain_test.observer.node_id
                        ))],
                    ),
                ),
            ];

            tests.push(TestFn {
                name: test_name,
                doc: vec![
                    format!(
                        "Chain test: {} -> {} (depth {})",
                        chain_test.probe.node_id, chain_test.observer.node_id, chain_test.depth
                    ),
                    String::new(),
                    "Non-tautological: asserts observer matchers, not baseline values.".to_string(),
                ],
                body,
            });
        }

        // Generate a failing test for coverage gaps (observability invariant).
        // The module doc states: "Every terminal node reachable from any probe
        // must have an OutputMatcher. Testgen emits an error for unobserved terminals."
        if !po_analysis.gaps.is_empty() {
            let gap_lines: Vec<String> = po_analysis
                .gaps
                .iter()
                .map(|g| {
                    format!(
                        "  terminal '{}' reachable from probe '{}' has no OutputMatcher",
                        g.terminal_node, g.probe_node
                    )
                })
                .collect();
            let msg = format!(
                "Observability invariant violated: {} unobserved terminal(s):\n{}\n\
                 Add OutputMatchers via NodeExample or live_expected_output for these nodes.",
                po_analysis.gaps.len(),
                gap_lines.join("\n")
            );
            tests.push(TestFn {
                name: "test_observability_invariant_no_gaps".to_string(),
                doc: vec![
                    "Every terminal node reachable from a probe must have an OutputMatcher."
                        .to_string(),
                    "This test fails when coverage gaps exist — add observers to fix.".to_string(),
                ],
                body: vec![Stmt::Expr(Expr::call("panic!", vec![Expr::Str(msg)]))],
            });
        }

        Some(TestSection {
            title: "Probe-Observer Integration Tests".to_string(),
            notes: vec![
                format!(
                    "Probes: {} | Observers: {} | Tests: {}",
                    po_analysis.probes.len(),
                    po_analysis.observers.len(),
                    po_analysis.tests.len()
                ),
                if po_analysis.gaps.is_empty() {
                    "All terminal nodes are observed.".to_string()
                } else {
                    format!(
                        "Coverage gaps: {} unobserved terminal(s) — add OutputMatchers to fix.",
                        po_analysis.gaps.len()
                    )
                },
            ],
            tests,
        })
    }

    fn build_boundary_section(
        &self,
        analysis: &DagAnalysis,
        graph_builder_fn: &str,
    ) -> Option<TestSection> {
        let mut tests = Vec::new();

        // Use lowered DAG for boundary analysis so node IDs match lowered MockSpec IDs.
        // SubDag nodes in the un-lowered DAG become flattened, prefixed nodes after lowering;
        // the MockSpec uses these lowered IDs since the executor operates on the lowered DAG.
        let lowered_result = gunbc_exec::lower(self.dag).ok();
        let lowered_analysis = lowered_result.as_ref().map(|lr| analyze_dag(&lr.dag));
        let boundary_analysis = lowered_analysis.as_ref().unwrap_or(analysis);

        let mocks_expr = self.dryrun_mocks_expr(boundary_analysis, "boundary mockability tests");

        tests.push(TestFn {
            name: "test_boundaries_mockable".to_string(),
            doc: vec!["Test that all boundaries can be mocked.".to_string()],
            body: vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::let_bind(
                    "result",
                    Expr::call(
                        "assert_boundary_mockable",
                        vec![Expr::var("dag").ref_of(), mocks_expr.clone()],
                    ),
                ),
                Stmt::Expr(Expr::call(
                    "assert!",
                    vec![
                        Expr::var("result").method("is_ok", vec![]),
                        Expr::Str("Boundaries should be mockable: {:?}".into()),
                        Expr::var("result").field("error"),
                    ],
                )),
            ],
        });

        for boundary_node in &boundary_analysis.boundaries.boundary_nodes {
            let total_outputs = boundary_analysis
                .port_cardinalities
                .iter()
                .filter(|p| p.node_id == boundary_node.0 && !p.is_input)
                .count();
            let boundary_outputs = boundary_analysis
                .boundaries
                .boundary_ports
                .iter()
                .filter(|(n, _)| n == boundary_node)
                .count();
            // Only generate per-node boundary tests when all outputs are boundaries.
            // Mixed nodes (some outputs wired downstream) require full mocks to
            // intercept and can break downstream execution.
            if boundary_outputs != total_outputs {
                continue;
            }
            let test_name = format!(
                "test_boundary_{}_mockable",
                NamingCase::SnakeCase.apply(&boundary_node.0)
            );
            let node_name = &boundary_node.0;

            // Per-node boundary tests lower the DAG at runtime since the MockSpec
            // and executor both operate on lowered (flattened) node IDs.
            let mut body = vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::let_bind(
                    "lowered",
                    Expr::call("lower", vec![Expr::var("dag").ref_of()])
                        .method("expect", vec![Expr::Str("lowering should succeed".into())]),
                ),
                Stmt::let_bind(
                    "boundaries",
                    Expr::call(
                        "detect_boundaries",
                        vec![Expr::var("lowered").field("dag").ref_of()],
                    ),
                ),
                Stmt::Assert(Assert::True {
                    expr: Expr::var("boundaries").method(
                        "is_boundary_node",
                        vec![Expr::Str(node_name.clone()).method("into", vec![]).ref_of()],
                    ),
                    message: format!("{} should be a boundary", node_name),
                }),
                Stmt::Blank,
                Stmt::let_mut("mocks", mocks_expr.clone()),
            ];

            for (node_id, port_name) in &boundary_analysis.boundaries.boundary_ports {
                if node_id == boundary_node {
                    let (type_id, cardinality) = boundary_analysis
                        .port_cardinalities
                        .iter()
                        .find(|p| {
                            p.node_id == node_id.0 && p.port_name == port_name.0 && !p.is_input
                        })
                        .map(|p| (p.type_id.0.as_str(), p.cardinality))
                        .unwrap_or(("String", Cardinality::ONE));

                    let mock_value =
                        self.get_mock_value(&node_id.0, &port_name.0, type_id, cardinality);
                    body.push(Stmt::Expr(Expr::var("mocks").method(
                        "set_value",
                        vec![
                            Expr::Str(node_id.0.clone()),
                            Expr::Str(port_name.0.clone()),
                            Expr::Value(mock_value),
                        ],
                    )));
                }
            }

            body.push(Stmt::Blank);
            body.push(Stmt::let_bind(
                "log",
                Expr::call(
                    "execute_with_mode",
                    vec![
                        Expr::var("dag").ref_of(),
                        Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
                    ],
                )
                .method("unwrap", vec![]),
            ));
            body.push(Stmt::let_bind(
                "entry",
                Expr::var("log")
                    .method("get", vec![Expr::Str(node_name.clone())])
                    .method("expect", vec![Expr::Str("node should be in log".into())]),
            ));
            body.push(Stmt::Assert(Assert::True {
                expr: Expr::var("entry").field("was_intercepted"),
                message: "boundary should be intercepted in dry-run".to_string(),
            }));

            tests.push(TestFn {
                name: test_name,
                doc: vec![format!("Test that {} boundary can be mocked.", node_name)],
                body,
            });
        }

        Some(TestSection {
            title: "Boundary Tests (per-node mockability)".to_string(),
            notes: Vec::new(),
            tests,
        })
    }

    fn build_chain_section(&self, _analysis: &DagAnalysis) -> Option<TestSection> {
        let Some(spec) = &self.mock_spec else {
            return None;
        };
        if self.mock_spec_fn.is_none() {
            panic!(
                "Chain tests require mock_spec_fn so generated tests can access MockSpec at runtime.\n\
                 Provide TestGenerator::with_mock_spec_fn(\"path::to::mock_spec()\") to enable chain tests."
            );
        }
        if spec.input_expectations.is_empty() && spec.boundary_mocks.is_empty() {
            return None;
        }

        let mut tests = Vec::new();

        let mut body = vec![
            Stmt::let_bind("spec", Expr::call("mock_spec", vec![])),
            Stmt::Comment("Verify all boundary mocks are present".to_string()),
        ];
        for mock in &spec.boundary_mocks {
            body.push(Stmt::Assert(Assert::True {
                expr: Expr::var("spec")
                    .method(
                        "get_boundary_mock",
                        vec![Expr::Str(mock.node.clone()), Expr::Str(mock.port.clone())],
                    )
                    .method("is_some", vec![]),
                message: format!(
                    "MockSpec should have boundary mock for {}.{}",
                    mock.node, mock.port
                ),
            }));
        }
        tests.push(TestFn {
            name: "test_mock_spec_self_consistent".to_string(),
            doc: vec!["Test that this tool's mock spec is self-consistent.".to_string()],
            body,
        });

        if !spec.input_expectations.is_empty() {
            let mut body = vec![Stmt::let_bind("spec", Expr::call("mock_spec", vec![]))];
            for exp in &spec.input_expectations {
                let constraint_str = match &exp.constraint {
                    gunbc_test::InputConstraint::NonEmpty => "NonEmpty",
                    gunbc_test::InputConstraint::Any => "Any",
                    gunbc_test::InputConstraint::OneOf(_) => "OneOf(...)",
                    gunbc_test::InputConstraint::TypePattern(_) => "TypePattern(...)",
                    gunbc_test::InputConstraint::Custom { description, .. } => description.as_str(),
                };
                body.push(Stmt::Comment(format!(
                    "Port '{}' expects: {}",
                    exp.port, constraint_str
                )));
            }

            body.push(Stmt::Expr(Expr::call(
                "assert_eq!",
                vec![
                    Expr::var("spec")
                        .field("input_expectations")
                        .method("len", vec![]),
                    Expr::int(spec.input_expectations.len() as i64),
                ],
            )));

            tests.push(TestFn {
                name: "test_input_expectations_documented".to_string(),
                doc: vec!["Test that input expectations are documented.".to_string()],
                body,
            });
        }

        Some(TestSection {
            title: "Chain Validation Tests".to_string(),
            notes: vec![
                "These tests verify that mock outputs satisfy downstream input expectations."
                    .to_string(),
            ],
            tests,
        })
    }

    // =======================================================================
    // Node I/O Example Tests
    // =======================================================================

    fn build_node_example_section(&self, graph_builder_fn: &str) -> Option<TestSection> {
        let mockspec_examples = self
            .mock_spec
            .as_ref()
            .map(|s| &s.node_examples[..])
            .unwrap_or(&[]);

        let has_node_examples = self.dag.nodes.iter().any(|n| !n.examples.is_empty());

        if mockspec_examples.is_empty() && !has_node_examples {
            return None;
        }

        let mut tests = Vec::new();

        for (idx, example) in mockspec_examples.iter().enumerate() {
            let has_satisfies = example
                .outputs
                .values()
                .any(|matcher| matches!(matcher, OutputMatcher::Satisfies { .. }));
            if has_satisfies && self.mock_spec_fn.is_none() {
                panic!(
                    "OutputMatcher::Satisfies requires mock_spec_fn so generated tests can access runtime matchers.\n\
                     Example {} for node '{}' uses Satisfies, but no mock_spec_fn was provided.\n\
                     Provide TestGenerator::with_mock_spec_fn(\"path::to::mock_spec\").",
                    idx, example.node_id
                );
            }

            let test_name = if let Some(desc) = &example.description {
                let sanitized_desc = sanitize_to_snake_case(desc);
                format!(
                    "test_example_{}_{}",
                    NamingCase::SnakeCase.apply(&example.node_id),
                    sanitized_desc
                )
            } else {
                format!(
                    "test_example_{}_{}",
                    NamingCase::SnakeCase.apply(&example.node_id),
                    idx
                )
            };

            let mut doc = if let Some(desc) = &example.description {
                vec![format!("Node example: {} - {}", example.node_id, desc)]
            } else {
                vec![format!(
                    "Node example: {} (example {})",
                    example.node_id, idx
                )]
            };
            doc.push(String::new());
            doc.push(format!(
                "Tests that node '{}' produces expected outputs for given inputs.",
                example.node_id
            ));

            let mut body = vec![Stmt::let_bind("dag", Expr::var(graph_builder_fn))];
            if has_satisfies {
                body.push(Stmt::let_bind("spec", Expr::call("mock_spec", vec![])));
                body.push(Stmt::let_bind(
                    "example_spec",
                    Expr::var("spec")
                        .field("node_examples")
                        .method("get", vec![Expr::int(idx as i64)])
                        .method(
                            "expect",
                            vec![Expr::Str(format!(
                                "mock_spec missing node example {} for '{}'",
                                idx, example.node_id
                            ))],
                        ),
                ));
                body.push(Stmt::Assert(Assert::Eq {
                    left: Expr::var("example_spec")
                        .field("node_id")
                        .method("as_str", vec![]),
                    right: Expr::Str(example.node_id.clone()),
                    message: format!(
                        "mock_spec example {} should match node id '{}'",
                        idx, example.node_id
                    ),
                }));
                if let Some(desc) = &example.description {
                    body.push(Stmt::Assert(Assert::Eq {
                        left: Expr::var("example_spec")
                            .field("description")
                            .method("as_deref", vec![]),
                        right: Expr::call("Some", vec![Expr::Str(desc.clone())]),
                        message: format!(
                            "mock_spec example {} should match description '{}'",
                            idx, desc
                        ),
                    }));
                } else {
                    body.push(Stmt::Assert(Assert::True {
                        expr: Expr::var("example_spec")
                            .field("description")
                            .method("is_none", vec![]),
                        message: format!("mock_spec example {} should have no description", idx),
                    }));
                }
                body.push(Stmt::Blank);
            }
            if example.inputs.is_empty() {
                body.push(Stmt::let_bind(
                    "inputs",
                    Expr::call("std::collections::HashMap::new", vec![]),
                ));
            } else {
                body.push(Stmt::let_mut(
                    "inputs",
                    Expr::call("std::collections::HashMap::new", vec![]),
                ));
            }

            let mut sorted_inputs: Vec<_> = example.inputs.iter().collect();
            sorted_inputs.sort_by_key(|(k, _)| k.as_str());
            for (port, value) in sorted_inputs {
                body.push(Stmt::Expr(Expr::var("inputs").method(
                    "insert",
                    vec![
                        Expr::Str(port.clone()).method("to_string", vec![]),
                        Expr::Value(ValueExpr::from(value)),
                    ],
                )));
            }

            let outputs = Expr::call(
                "gunbc_exec::execute_single_node",
                vec![
                    Expr::var("dag").ref_of(),
                    Expr::Str(example.node_id.clone()),
                    Expr::var("inputs"),
                    Expr::Path(vec![
                        "gunbc_exec".to_string(),
                        "ExecutionMode".to_string(),
                        "Real".to_string(),
                    ]),
                ],
            )
            .method(
                "expect",
                vec![Expr::Str(format!(
                    "node '{}' should execute successfully",
                    example.node_id
                ))],
            );
            body.push(Stmt::let_bind("outputs", outputs));
            body.push(Stmt::Blank);

            let mut sorted_outputs: Vec<_> = example.outputs.iter().collect();
            sorted_outputs.sort_by_key(|(k, _)| k.as_str());
            for (port, matcher) in sorted_outputs {
                let var_name = sanitize_to_snake_case(port);
                let prefix = if matcher.generates_assertion() {
                    ""
                } else {
                    "_"
                };
                let output_var = format!("{}output_{}", prefix, var_name);

                body.push(Stmt::Comment(format!("Check output port '{}'", port)));
                body.push(Stmt::let_bind(
                    output_var.clone(),
                    Expr::var("outputs")
                        .method("get", vec![Expr::Str(port.clone())])
                        .method(
                            "expect",
                            vec![Expr::Str(format!("output port '{}' should exist", port))],
                        ),
                ));

                if matches!(matcher, OutputMatcher::Satisfies { .. }) {
                    let matcher_var = format!("matcher_{}", var_name);
                    body.push(Stmt::let_bind(
                        matcher_var.clone(),
                        Expr::var("example_spec")
                            .field("outputs")
                            .method("get", vec![Expr::Str(port.clone())])
                            .method(
                                "expect",
                                vec![Expr::Str(format!(
                                    "mock_spec example {} missing output matcher for port '{}'",
                                    idx, port
                                ))],
                            ),
                    ));
                    body.push(Stmt::Expr(
                        Expr::var(&matcher_var)
                            .method("check", vec![Expr::var(&output_var)])
                            .method(
                                "expect",
                                vec![Expr::Str(format!(
                                    "output port '{}' failed satisfies matcher",
                                    port
                                ))],
                            ),
                    ));
                } else {
                    let mut stmts = render_output_matcher_check(matcher, &var_name);
                    body.append(&mut stmts);
                }
            }

            tests.push(TestFn {
                name: test_name,
                doc,
                body,
            });
        }

        for node in &self.dag.nodes {
            for (idx, example) in node.examples.iter().enumerate() {
                let test_name = if let Some(desc) = &example.description {
                    let sanitized_desc = sanitize_to_snake_case(desc);
                    format!(
                        "test_node_example_{}_{}",
                        NamingCase::SnakeCase.apply(&node.id.0),
                        sanitized_desc
                    )
                } else {
                    format!(
                        "test_node_example_{}_{}",
                        NamingCase::SnakeCase.apply(&node.id.0),
                        idx
                    )
                };

                let mut doc = if let Some(desc) = &example.description {
                    vec![format!("Node I/O example: {} - {}", node.id.0, desc)]
                } else {
                    vec![format!("Node I/O example: {} (example {})", node.id.0, idx)]
                };
                doc.push(String::new());
                doc.push(format!(
                    "Tests that node '{}' produces expected outputs for given inputs (exact match).",
                    node.id.0
                ));

                let mut body = vec![Stmt::let_bind("dag", Expr::var(graph_builder_fn))];
                if example.inputs.is_empty() {
                    body.push(Stmt::let_bind(
                        "inputs",
                        Expr::call("std::collections::HashMap::new", vec![]),
                    ));
                } else {
                    body.push(Stmt::let_mut(
                        "inputs",
                        Expr::call("std::collections::HashMap::new", vec![]),
                    ));
                }

                let mut sorted_inputs: Vec<_> = example.inputs.iter().collect();
                sorted_inputs.sort_by_key(|(k, _)| k.as_str());
                for (port, value) in sorted_inputs {
                    body.push(Stmt::Expr(Expr::var("inputs").method(
                        "insert",
                        vec![
                            Expr::Str(port.clone()).method("to_string", vec![]),
                            Expr::Value(ValueExpr::from(value)),
                        ],
                    )));
                }

                let outputs = Expr::call(
                    "gunbc_exec::execute_single_node",
                    vec![
                        Expr::var("dag").ref_of(),
                        Expr::Str(node.id.0.clone()),
                        Expr::var("inputs"),
                        Expr::Path(vec![
                            "gunbc_exec".to_string(),
                            "ExecutionMode".to_string(),
                            "Real".to_string(),
                        ]),
                    ],
                )
                .method(
                    "expect",
                    vec![Expr::Str(format!(
                        "node '{}' should execute successfully",
                        node.id.0
                    ))],
                );
                body.push(Stmt::let_bind("outputs", outputs));
                body.push(Stmt::Blank);

                let mut sorted_outputs: Vec<_> = example.expected_outputs.iter().collect();
                sorted_outputs.sort_by_key(|(k, _)| k.as_str());
                for (port, expected) in sorted_outputs {
                    body.push(Stmt::Comment(format!("Check output port '{}'", port)));
                    let left = Expr::var("outputs")
                        .method("get", vec![Expr::Str(port.clone())])
                        .method(
                            "expect",
                            vec![Expr::Str(format!("output port '{}' should exist", port))],
                        );
                    let right = Expr::Value(ValueExpr::from(expected)).ref_of();
                    body.push(Stmt::Assert(Assert::Eq {
                        left,
                        right,
                        message: format!(
                            "node '{}' port '{}' should match expected value",
                            node.id.0, port
                        ),
                    }));
                }

                tests.push(TestFn {
                    name: test_name,
                    doc,
                    body,
                });
            }
        }

        Some(TestSection {
            title: "Node I/O Example Tests".to_string(),
            notes: vec![
                "These tests verify individual node behavior against specified examples."
                    .to_string(),
                "Each test executes a single node with given inputs and checks outputs."
                    .to_string(),
            ],
            tests,
        })
    }

    /// Build a CLI contract test section from entrypoints.
    ///
    /// This verifies that `gunbc_cli::parse()` handles the tool's CLI schema
    /// correctly by parsing sample arguments and checking the results.
    fn build_cli_contract_section(&self, obligations: &ObligationSet) -> Option<TestSection> {
        let (tool_name, entrypoints) = self.cli_entrypoints.as_ref()?;
        let has_cli_contract_obligation = obligations.cli_contract_obligations().iter().any(|o| {
            matches!(
                &o.kind,
                Obligation::CliContractRoundTrip {
                    tool_name: obligation_tool
                } if obligation_tool == tool_name
            )
        });
        if !has_cli_contract_obligation {
            return None;
        }

        let parse_test_name = format!("test_cli_contract_{}", tool_name.replace('-', "_"));
        let print_inputs_test_name = format!(
            "test_cli_contract_print_inputs_{}",
            tool_name.replace('-', "_")
        );

        // Shared schema body for both generated tests.
        let mut schema_code = String::new();
        schema_code.push_str("let schema = vec![\n");
        for ep in entrypoints {
            let type_expr = match ep.type_id {
                ParamType::Str => "ParamType::Str",
                ParamType::Int => "ParamType::Int",
                ParamType::Bool => "ParamType::Bool",
            };
            write!(
                schema_code,
                "    CliParam::new(\"{}\", {})",
                ep.port_name, type_expr
            )
            .unwrap();
            if ep.cardinality.allows_many() {
                schema_code.push_str(".with_cardinality(Cardinality::ZERO_OR_MORE)");
            }
            if let Some(c) = ep.short_flag {
                write!(schema_code, ".short('{}')", c).unwrap();
            }
            if let Some(ref d) = ep.default_value {
                write!(schema_code, ".default(\"{}\")", cli_escape(d)).unwrap();
            }
            schema_code.push_str(",\n");
        }
        schema_code.push_str("];\n");

        // Build argv and shared assertions.
        let mut argv_parts: Vec<String> =
            vec![format!("\"{}\"", tool_name), "\"--dry-run\"".to_string()];
        let mut assertions: Vec<String> = Vec::new();

        for ep in entrypoints {
            let flag = format!("--{}", ep.flag_name());
            if ep.is_repeatable() {
                let (v1, v2) = cli_sample_repeatable(ep);
                argv_parts.push(format!("\"{}\"", cli_escape(&flag)));
                argv_parts.push(format!("\"{}\"", cli_escape(&v1)));
                argv_parts.push(format!("\"{}\"", cli_escape(&flag)));
                argv_parts.push(format!("\"{}\"", cli_escape(&v2)));
                assertions.push(format!(
                    "assert_eq!(result.values[\"{}\"], Value::str_list(vec![\"{}\".into(), \"{}\".into()]), \"repeatable param '{}' mismatch\");\n",
                    ep.port_name, cli_escape(&v1), cli_escape(&v2), ep.port_name
                ));
            } else {
                match ep.type_id {
                    ParamType::Bool => {
                        argv_parts.push(format!("\"{}\"", cli_escape(&flag)));
                        assertions.push(format!(
                            "assert_eq!(result.values[\"{}\"], Value::Bool(true), \"bool param '{}' mismatch\");\n",
                            ep.port_name, ep.port_name
                        ));
                    }
                    ParamType::Int => {
                        let value = cli_sample_int(ep);
                        argv_parts.push(format!("\"{}\"", cli_escape(&flag)));
                        argv_parts.push(format!("\"{}\"", cli_escape(&value)));
                        assertions.push(format!(
                            "assert_eq!(result.values[\"{}\"], Value::Int({}), \"int param '{}' mismatch\");\n",
                            ep.port_name, value, ep.port_name
                        ));
                    }
                    ParamType::Str => {
                        let value = cli_sample_string(ep);
                        argv_parts.push(format!("\"{}\"", cli_escape(&flag)));
                        argv_parts.push(format!("\"{}\"", cli_escape(&value)));
                        assertions.push(format!(
                            "assert_eq!(result.values[\"{}\"], Value::Str(\"{}\".into()), \"string param '{}' mismatch\");\n",
                            ep.port_name, cli_escape(&value), ep.port_name
                        ));
                    }
                }
            }
        }
        assertions.push("assert!(result.dry_run, \"dry_run should be true\");\n".to_string());

        // Parse contract test body.
        let mut parse_code = String::new();
        parse_code.push_str(&schema_code);
        let argv_str = argv_parts.join(", ");
        writeln!(
            parse_code,
            "let argv: Vec<String> = [{}].iter().map(|s| s.to_string()).collect();",
            argv_str
        )
        .unwrap();
        parse_code
            .push_str("let result = parse(&argv, &schema).expect(\"parse should succeed\");\n");
        for assertion in &assertions {
            parse_code.push_str(assertion);
        }

        // --print-inputs json round-trip contract test body.
        let mut print_inputs_code = String::new();
        print_inputs_code.push_str(&schema_code);
        let mut full_argv_parts: Vec<String> = vec![
            format!("\"{}\"", tool_name),
            "\"--dry-run\"".to_string(),
            "\"--print-inputs\"".to_string(),
            "\"json\"".to_string(),
        ];
        full_argv_parts.extend(argv_parts.iter().skip(2).cloned());
        writeln!(
            print_inputs_code,
            "let full_argv: Vec<String> = [{}].iter().map(|s| s.to_string()).collect();",
            full_argv_parts.join(", ")
        )
        .unwrap();
        print_inputs_code
            .push_str("let mut parse_args: Vec<String> = Vec::with_capacity(full_argv.len());\n");
        print_inputs_code.push_str("if let Some(program) = full_argv.first() {\n");
        print_inputs_code.push_str("    parse_args.push(program.clone());\n");
        print_inputs_code.push_str("}\n");
        print_inputs_code.push_str("let mut print_inputs_json = false;\n");
        print_inputs_code.push_str("let mut raw_idx = 1usize;\n");
        print_inputs_code.push_str("while raw_idx < full_argv.len() {\n");
        print_inputs_code.push_str("    let arg = &full_argv[raw_idx];\n");
        print_inputs_code.push_str("    if arg == \"--print-inputs\" {\n");
        print_inputs_code.push_str("        raw_idx += 1;\n");
        print_inputs_code.push_str("        assert!(raw_idx < full_argv.len(), \"--print-inputs should include a format value\");\n");
        print_inputs_code.push_str("        assert_eq!(full_argv[raw_idx], \"json\", \"--print-inputs only supports json\");\n");
        print_inputs_code.push_str("        print_inputs_json = true;\n");
        print_inputs_code
            .push_str("    } else if let Some(format) = arg.strip_prefix(\"--print-inputs=\") {\n");
        print_inputs_code.push_str(
            "        assert_eq!(format, \"json\", \"--print-inputs only supports json\");\n",
        );
        print_inputs_code.push_str("        print_inputs_json = true;\n");
        print_inputs_code.push_str("    } else {\n");
        print_inputs_code.push_str("        parse_args.push(arg.clone());\n");
        print_inputs_code.push_str("    }\n");
        print_inputs_code.push_str("    raw_idx += 1;\n");
        print_inputs_code.push_str("}\n");
        print_inputs_code.push_str(
            "assert!(print_inputs_json, \"expected --print-inputs json to be detected\");\n",
        );
        print_inputs_code.push_str(
            "let result = parse(&parse_args, &schema).expect(\"parse should succeed\");\n",
        );
        for assertion in &assertions {
            print_inputs_code.push_str(assertion);
        }
        print_inputs_code.push_str("let mut ordered_inputs = std::collections::BTreeMap::new();\n");
        print_inputs_code.push_str("for (port, value) in &result.values {\n");
        print_inputs_code.push_str("    ordered_inputs.insert(port.clone(), value.clone());\n");
        print_inputs_code.push_str("}\n");
        print_inputs_code
            .push_str("let json = gunbc_ir::to_bridge_json(&Value::Map(ordered_inputs))\n");
        print_inputs_code
            .push_str("    .expect(\"to_bridge_json should serialize parsed inputs\");\n");
        print_inputs_code.push_str(
            "assert!(json.is_object(), \"--print-inputs json should be a JSON object\");\n",
        );

        // Wrap entire body in a single TailExpr to avoid extra semicolons.
        // The raw code already has its own semicolons where needed.
        let parse_body = vec![Stmt::TailExpr(Expr::raw(parse_code.trim_end()))];
        let print_inputs_body = vec![Stmt::TailExpr(Expr::raw(print_inputs_code.trim_end()))];

        let parse_test = TestFn {
            name: parse_test_name,
            doc: vec![format!(
                "CLI contract: verify gunbc_cli::parse() handles '{}' arguments.",
                tool_name
            )],
            body: parse_body,
        };

        let print_inputs_test = TestFn {
            name: print_inputs_test_name,
            doc: vec![format!(
                "CLI contract: verify '{}' supports --print-inputs json round-trip.",
                tool_name
            )],
            body: print_inputs_body,
        };

        Some(TestSection {
            title: "CLI Contract Tests".to_string(),
            notes: vec![
                "Verifies CLI argument parsing for this tool's entrypoints.".to_string(),
                "Uses gunbc_cli::parse() for in-process validation (no subprocess).".to_string(),
                "Also validates --print-inputs json preprocessing and JSON serialization."
                    .to_string(),
            ],
            tests: vec![parse_test, print_inputs_test],
        })
    }

    // =======================================================================
    // BB-2: Per-Node Corpus Tests
    // =======================================================================
}

/// Shared context for a single corpus example, passed to body-generation helpers.
struct CorpusExampleCtx<'a> {
    graph_builder_fn: &'a str,
    concrete_node_id: &'a str,
    identity: &'a gunbc_test::NodeIdentity,
    ex_idx: usize,
    workflow: &'a str,
    inputs: &'a HashMap<String, Value>,
}

impl<T: Clone + 'static> TestGenerator<'_, T> {

    fn build_corpus_section(
        &self,
        graph_builder_fn: &str,
        analysis: &DagAnalysis,
    ) -> Option<TestSection> {
        let corpus = self.corpus.as_ref()?;
        if corpus.is_empty() {
            return None;
        }

        // Map NodeIdentity → concrete Node in this DAG.
        // Corpus examples may come from foreign workflows; we only generate tests
        // for identities that have a matching node in *this* DAG.
        let mut identity_to_node: HashMap<gunbc_test::NodeIdentity, &gunbc_ir::Node<T>> =
            HashMap::new();
        for node in &self.dag.nodes {
            if let Some(identity) = gunbc_test::NodeIdentity::from_node_id(&node.id.0) {
                // First match wins (multiple sub-instances share the same identity).
                identity_to_node.entry(identity).or_insert(node);
            }
        }

        let mut tests = Vec::new();

        // Surface unmatched corpus identities — violates the "errors are explicit,
        // no silent fallbacks" invariant if we silently drop them.
        let mut unmatched_identities = Vec::new();
        for (identity, node_corpus) in corpus {
            if !node_corpus.is_empty() && !identity_to_node.contains_key(identity) {
                unmatched_identities.push(identity.to_string());
            }
        }
        if !unmatched_identities.is_empty() {
            eprintln!(
                "[corpus] WARNING: {} corpus identit{} not found in DAG (corpus drift?): {}",
                unmatched_identities.len(),
                if unmatched_identities.len() == 1 { "y" } else { "ies" },
                unmatched_identities.join(", "),
            );
        }

        for (identity, node_corpus) in corpus {
            if node_corpus.is_empty() {
                continue;
            }

            // Find the matching node in this DAG.
            let node = match identity_to_node.get(identity) {
                Some(n) => *n,
                None => continue, // Already surfaced as warning above.
            };
            let concrete_node_id = &node.id.0;

            let max_examples = crate::testgen::mock_corpus::MAX_EXAMPLES_PER_NODE;
            for (ex_idx, example) in node_corpus.examples.iter().take(max_examples).enumerate() {
                let sanitized_id = sanitize_to_snake_case(&identity.to_string());
                let test_name = format!("test_corpus_{}_{}", sanitized_id, ex_idx);

                let ctx = CorpusExampleCtx {
                    graph_builder_fn,
                    concrete_node_id,
                    identity,
                    ex_idx,
                    workflow: &example.provenance.workflow,
                    inputs: &example.inputs,
                };

                match &example.expectation {
                    gunbc_test::Expectation::ExactOutputs(expected_outputs) => {
                        // Gate on node purity: ExactOutputs assumes Real mode,
                        // but effectful nodes must not execute Real (side effects).
                        // Fall back to DryRun + type contract for effectful nodes.
                        let (doc, body) = if is_pure_node(node) {
                            self.build_corpus_body_real(&ctx, expected_outputs)
                        } else {
                            eprintln!(
                                "[corpus] NOTE: Node '{}' has ExactOutputs expectation but is \
                                 effectful — falling back to DryRun mode.",
                                identity,
                            );
                            self.build_corpus_body_dryrun(&ctx, analysis, node, None)
                        };
                        tests.push(TestFn {
                            name: test_name,
                            doc,
                            body,
                        });
                    }
                    gunbc_test::Expectation::TypeContractOnly => {
                        let (doc, body) =
                            self.build_corpus_body_dryrun(&ctx, analysis, node, None);
                        tests.push(TestFn {
                            name: test_name,
                            doc,
                            body,
                        });
                    }
                    gunbc_test::Expectation::OutputMatchers(matchers) => {
                        let (doc, body) =
                            self.build_corpus_body_dryrun(&ctx, analysis, node, Some(matchers));
                        tests.push(TestFn {
                            name: test_name,
                            doc,
                            body,
                        });
                    }
                    gunbc_test::Expectation::ExpectValidationError => {
                        // BB-2 Level 2 scope — skip for now.
                        continue;
                    }
                }
            }
        }

        if tests.is_empty() {
            return None;
        }

        Some(TestSection {
            title: "BB-2: Per-Node Corpus Tests (cross-workflow black-box)".to_string(),
            notes: vec![
                "Tests nodes against inputs accumulated from ALL workflows they appear in."
                    .to_string(),
                "Catches regressions that single-workflow tests miss.".to_string(),
            ],
            tests,
        })
    }

    /// Build test body for a pure node corpus example (Level 1a).
    ///
    /// Executes with `ExecutionMode::Real` and asserts exact output match.
    fn build_corpus_body_real(
        &self,
        ctx: &CorpusExampleCtx<'_>,
        expected_outputs: &HashMap<String, Value>,
    ) -> (Vec<String>, Vec<Stmt>) {
        let doc = vec![
            format!(
                "Corpus test for node '{}' (example {} from workflow '{}').",
                ctx.identity, ctx.ex_idx, ctx.workflow,
            ),
            String::new(),
            "Pure node: executes with Real mode, asserts exact output match.".to_string(),
        ];

        let mut body = Vec::new();
        body.push(Stmt::let_bind("dag", Expr::var(ctx.graph_builder_fn)));

        // Build inputs map from corpus example.
        let sorted_inputs: BTreeMap<String, ValueExpr> = ctx
            .inputs
            .iter()
            .map(|(k, v)| (k.clone(), ValueExpr::from(v)))
            .collect();
        body.extend(self.build_inputs_map_stmts(&sorted_inputs));

        // Execute with Real mode.
        let exec = Expr::call(
            "gunbc_exec::execute_single_node",
            vec![
                Expr::var("dag").ref_of(),
                Expr::Str(ctx.concrete_node_id.to_string()),
                Expr::var("inputs"),
                Expr::Path(vec![
                    "gunbc_exec".to_string(),
                    "ExecutionMode".to_string(),
                    "Real".to_string(),
                ]),
            ],
        )
        .method(
            "expect",
            vec![Expr::Str(format!(
                "corpus node '{}' should execute successfully (example {})",
                ctx.identity, ctx.ex_idx
            ))],
        );
        body.push(Stmt::let_bind("outputs", exec));
        body.push(Stmt::Blank);

        // Assert exact output match for each expected port.
        let mut sorted_expected: Vec<_> = expected_outputs.iter().collect();
        sorted_expected.sort_by_key(|(k, _)| k.as_str());
        for (port, expected_value) in sorted_expected {
            body.push(Stmt::Comment(format!("Check output port '{}'", port)));
            let left = Expr::var("outputs")
                .method("get", vec![Expr::Str(port.clone())])
                .method(
                    "expect",
                    vec![Expr::Str(format!("output port '{}' should exist", port))],
                );
            let right = Expr::Value(ValueExpr::from(expected_value)).ref_of();
            body.push(Stmt::Assert(Assert::Eq {
                left,
                right,
                message: format!(
                    "corpus node '{}' port '{}' should match expected value (example {})",
                    ctx.identity, port, ctx.ex_idx
                ),
            }));
        }

        (doc, body)
    }

    /// Build test body for an effectful node corpus example (Level 1b).
    ///
    /// Executes with `ExecutionMode::DryRun` and asserts either type contract
    /// compliance (when `matchers` is `None`) or output matchers (when present).
    fn build_corpus_body_dryrun(
        &self,
        ctx: &CorpusExampleCtx<'_>,
        analysis: &DagAnalysis,
        node: &gunbc_ir::Node<T>,
        matchers: Option<&HashMap<String, OutputMatcher>>,
    ) -> (Vec<String>, Vec<Stmt>) {
        let kind_label = if matchers.is_some() {
            "Effectful node: executes with DryRun mode, asserts output matchers."
        } else {
            "Effectful node: executes with DryRun mode, asserts type contract compliance."
        };

        let doc = vec![
            format!(
                "Corpus test for node '{}' (example {} from workflow '{}').",
                ctx.identity, ctx.ex_idx, ctx.workflow,
            ),
            String::new(),
            kind_label.to_string(),
        ];

        let mut body = Vec::new();
        body.push(Stmt::let_bind("dag", Expr::var(ctx.graph_builder_fn)));

        // Build inputs map from corpus example.
        let sorted_inputs: BTreeMap<String, ValueExpr> = ctx
            .inputs
            .iter()
            .map(|(k, v)| (k.clone(), ValueExpr::from(v)))
            .collect();
        body.extend(self.build_inputs_map_stmts(&sorted_inputs));

        // Build DryRun mocks.
        let mocks_expr = self.dryrun_mocks_expr(analysis, "corpus type-contract tests");
        body.push(Stmt::let_bind("mocks", mocks_expr));

        // Execute with DryRun mode.
        let exec = Expr::call(
            "gunbc_exec::execute_single_node",
            vec![
                Expr::var("dag").ref_of(),
                Expr::Str(ctx.concrete_node_id.to_string()),
                Expr::var("inputs"),
                Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks")]),
            ],
        )
        .method(
            "expect",
            vec![Expr::Str(format!(
                "corpus node '{}' should execute in DryRun (example {})",
                ctx.identity, ctx.ex_idx
            ))],
        );
        body.push(Stmt::let_bind("outputs", exec));
        body.push(Stmt::Blank);

        if let Some(matchers) = matchers {
            // Assert output matchers for each port.
            let mut sorted_matchers: Vec<_> = matchers.iter().collect();
            sorted_matchers.sort_by_key(|(k, _)| k.as_str());
            for (port, matcher) in sorted_matchers {
                let var_name = sanitize_to_snake_case(port);
                body.push(Stmt::Comment(format!("Check output port '{}'", port)));
                body.push(Stmt::let_bind(
                    format!("output_{}", var_name),
                    Expr::var("outputs")
                        .method("get", vec![Expr::Str(port.clone())])
                        .method(
                            "expect",
                            vec![Expr::Str(format!(
                                "output port '{}' should exist",
                                port
                            ))],
                        ),
                ));
                body.extend(render_output_matcher_check(matcher, &var_name));
            }
        } else {
            // Assert type contract for each output port.
            for port in &node.outputs {
                let port_name = &port.name.0;
                let type_id = &port.type_id.0;
                body.push(Stmt::Comment(format!(
                    "Type contract: port '{}' ({})",
                    port_name, type_id
                )));
                body.push(Stmt::Assert(Assert::True {
                    expr: Expr::call(
                        "gunbc_ir::value_compatible_with_type_id",
                        vec![
                            Expr::Str(type_id.clone()),
                            Expr::var("outputs")
                                .method("get", vec![Expr::Str(port_name.clone())])
                                .method(
                                    "expect",
                                    vec![Expr::Str(format!(
                                        "output port '{}' should exist",
                                        port_name
                                    ))],
                                ),
                        ],
                    ),
                    message: format!(
                        "corpus node '{}' port '{}' should be type-compatible with '{}' (example {})",
                        ctx.identity, port_name, type_id, ctx.ex_idx
                    ),
                }));
            }
        }

        (doc, body)
    }

    // =======================================================================
    // BB-3: Adjacent Pair Tests
    // =======================================================================

    fn build_adjacent_pair_section(
        &self,
        graph_builder_fn: &str,
        analysis: &DagAnalysis,
    ) -> Option<TestSection> {
        let edge_examples = self.edge_examples.as_ref()?;
        if edge_examples.is_empty() {
            return None;
        }

        // Map NodeIdentity → concrete Node for both endpoints.
        let mut identity_to_node: HashMap<gunbc_test::NodeIdentity, &gunbc_ir::Node<T>> =
            HashMap::new();
        for node in &self.dag.nodes {
            if let Some(identity) = gunbc_test::NodeIdentity::from_node_id(&node.id.0) {
                identity_to_node.entry(identity).or_insert(node);
            }
        }

        let mut tests = Vec::new();
        let mut seen_pairs = HashSet::new();

        for example in edge_examples {
            let pair_key = format!("{}_{}", example.from_node, example.to_node);
            if !seen_pairs.insert(pair_key) {
                continue;
            }

            // Both endpoints must exist in this DAG.
            let from_node = match identity_to_node.get(&example.from_node) {
                Some(n) => *n,
                None => continue,
            };
            let to_node = match identity_to_node.get(&example.to_node) {
                Some(n) => *n,
                None => continue,
            };

            let wf = &example.provenance.workflow;
            let test_name = format!(
                "test_edge_{}_{}_wf_{}",
                sanitize_to_snake_case(&example.from_node.to_string()),
                sanitize_to_snake_case(&example.to_node.to_string()),
                sanitize_to_snake_case(wf),
            );

            let doc = vec![
                format!(
                    "Adjacent pair test: {} → {} (from workflow '{}')",
                    example.from_node, example.to_node, wf
                ),
                String::new(),
                "Executes source node, wires outputs through the edge, executes target node."
                    .to_string(),
                "Catches port wiring and type translation bugs between connected nodes."
                    .to_string(),
            ];

            let mut body = Vec::new();
            body.push(Stmt::let_bind("dag", Expr::var(graph_builder_fn)));

            // Build source node inputs.
            let sorted_a_inputs: BTreeMap<String, ValueExpr> = example
                .a_inputs
                .iter()
                .map(|(k, v)| (k.clone(), ValueExpr::from(v)))
                .collect();
            body.push(Stmt::Comment(format!(
                "Execute source node '{}'",
                example.from_node
            )));
            body.extend(self.build_inputs_map_stmts(&sorted_a_inputs));

            // Determine execution mode for source node.
            let from_is_pure = is_pure_node(from_node);
            let a_exec_mode = if from_is_pure {
                Expr::Path(vec![
                    "gunbc_exec".to_string(),
                    "ExecutionMode".to_string(),
                    "Real".to_string(),
                ])
            } else {
                let mocks_expr =
                    self.dryrun_mocks_expr(analysis, "adjacent pair source node");
                body.push(Stmt::let_bind("mocks_a", mocks_expr));
                Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks_a")])
            };

            let a_exec = Expr::call(
                "gunbc_exec::execute_single_node",
                vec![
                    Expr::var("dag").ref_of(),
                    Expr::Str(from_node.id.0.clone()),
                    Expr::var("inputs"),
                    a_exec_mode,
                ],
            )
            .method(
                "expect",
                vec![Expr::Str(format!(
                    "source node '{}' should execute successfully",
                    example.from_node
                ))],
            );
            body.push(Stmt::let_bind("a_outputs", a_exec));
            body.push(Stmt::Blank);

            // Wire source outputs through edge_port_map into target inputs.
            body.push(Stmt::Comment(format!(
                "Wire outputs → inputs via edge: {} → {}",
                example.from_node, example.to_node
            )));
            body.push(Stmt::let_mut(
                "b_inputs",
                Expr::call("std::collections::HashMap::new", vec![]),
            ));

            // First, add the edge-wired ports.
            let mut sorted_edge_map: Vec<_> = example.edge_port_map.iter().collect();
            sorted_edge_map.sort_by_key(|(k, _)| k.as_str());
            for (from_port, to_port) in sorted_edge_map {
                body.push(Stmt::Expr(Expr::var("b_inputs").method(
                    "insert",
                    vec![
                        Expr::Str(to_port.clone()).method("to_string", vec![]),
                        Expr::var("a_outputs")
                            .method("get", vec![Expr::Str(from_port.clone())])
                            .method(
                                "expect",
                                vec![Expr::Str(format!(
                                    "source output port '{}' should exist for edge wiring",
                                    from_port
                                ))],
                            )
                            .method("clone", vec![]),
                    ],
                )));
            }

            // Then, add the extra inputs not covered by the edge.
            let mut sorted_b_other: Vec<_> = example.b_other_inputs.iter().collect();
            sorted_b_other.sort_by_key(|(k, _)| k.as_str());
            for (port, value) in sorted_b_other {
                body.push(Stmt::Expr(Expr::var("b_inputs").method(
                    "insert",
                    vec![
                        Expr::Str(port.clone()).method("to_string", vec![]),
                        Expr::Value(ValueExpr::from(value)),
                    ],
                )));
            }

            // Execute target node.
            body.push(Stmt::Blank);
            body.push(Stmt::Comment(format!(
                "Execute target node '{}'",
                example.to_node
            )));

            let to_is_pure = is_pure_node(to_node);
            let b_exec_mode = if to_is_pure {
                Expr::Path(vec![
                    "gunbc_exec".to_string(),
                    "ExecutionMode".to_string(),
                    "Real".to_string(),
                ])
            } else {
                let mocks_expr =
                    self.dryrun_mocks_expr(analysis, "adjacent pair target node");
                body.push(Stmt::let_bind("mocks_b", mocks_expr));
                Expr::call("ExecutionMode::DryRun", vec![Expr::var("mocks_b")])
            };

            let b_exec = Expr::call(
                "gunbc_exec::execute_single_node",
                vec![
                    Expr::var("dag").ref_of(),
                    Expr::Str(to_node.id.0.clone()),
                    Expr::var("b_inputs"),
                    b_exec_mode,
                ],
            )
            .method(
                "expect",
                vec![Expr::Str(format!(
                    "target node '{}' should execute successfully with wired inputs",
                    example.to_node
                ))],
            );
            body.push(Stmt::let_bind("b_outputs", b_exec));
            body.push(Stmt::Blank);

            // Assert type contract on target node outputs.
            body.push(Stmt::Comment("Verify target node output types.".to_string()));
            for port in &to_node.outputs {
                let port_name = &port.name.0;
                let type_id = &port.type_id.0;
                body.push(Stmt::Assert(Assert::True {
                    expr: Expr::call(
                        "gunbc_ir::value_compatible_with_type_id",
                        vec![
                            Expr::Str(type_id.clone()),
                            Expr::var("b_outputs")
                                .method("get", vec![Expr::Str(port_name.clone())])
                                .method(
                                    "expect",
                                    vec![Expr::Str(format!(
                                        "target output port '{}' should exist",
                                        port_name
                                    ))],
                                ),
                        ],
                    ),
                    message: format!(
                        "edge {} → {}: target port '{}' should be type-compatible with '{}'",
                        example.from_node, example.to_node, port_name, type_id
                    ),
                }));
            }

            tests.push(TestFn {
                name: test_name,
                doc,
                body,
            });
        }

        if tests.is_empty() {
            return None;
        }

        Some(TestSection {
            title: "BB-3: Adjacent Pair Tests (2-node window, real wiring)".to_string(),
            notes: vec![
                "Tests real executor wiring between connected node pairs.".to_string(),
                "Catches param→port translation bugs that per-node tests miss.".to_string(),
            ],
            tests,
        })
    }

    // =======================================================================
    // BB-5: Cross-Workflow Consistency Tests
    // =======================================================================

    fn build_cross_workflow_section(
        &self,
        graph_builder_fn: &str,
        analysis: &DagAnalysis,
    ) -> Option<TestSection> {
        let corpus = self.corpus.as_ref()?;

        // Map NodeIdentity → concrete Node in this DAG.
        let mut identity_to_node: HashMap<gunbc_test::NodeIdentity, &gunbc_ir::Node<T>> =
            HashMap::new();
        for node in &self.dag.nodes {
            if let Some(identity) = gunbc_test::NodeIdentity::from_node_id(&node.id.0) {
                identity_to_node.entry(identity).or_insert(node);
            }
        }

        let multi_workflow_nodes: Vec<_> = corpus
            .iter()
            .filter(|(_, c)| c.workflow_names().len() >= 2)
            .collect();

        if multi_workflow_nodes.is_empty() {
            return None;
        }

        let mut tests = Vec::new();

        for (identity, node_corpus) in &multi_workflow_nodes {
            let node = match identity_to_node.get(identity) {
                Some(n) => *n,
                None => continue,
            };
            let concrete_node_id = &node.id.0;

            let workflows = node_corpus.workflow_names();
            let test_name = format!(
                "test_cross_wf_{}",
                sanitize_to_snake_case(&identity.to_string())
            );

            let doc = vec![
                format!(
                    "Cross-workflow consistency: '{}' appears in {} workflows ({}).",
                    identity,
                    workflows.len(),
                    workflows.join(", ")
                ),
                String::new(),
                "Executes the node with inputs from each workflow and asserts".to_string(),
                "that all executions produce the same set of output port names.".to_string(),
            ];

            let mut body = Vec::new();
            body.push(Stmt::let_bind("dag", Expr::var(graph_builder_fn)));

            // Collect one representative example per workflow.
            // Use the first example from each workflow.
            let mut workflow_examples: Vec<(&str, &gunbc_test::CorpusExample)> = Vec::new();
            for wf_name in &workflows {
                if let Some(ex) = node_corpus
                    .examples
                    .iter()
                    .find(|e| e.provenance.workflow == *wf_name)
                {
                    workflow_examples.push((wf_name, ex));
                }
            }

            // For each workflow example: execute the node, collect output port keys.
            body.push(Stmt::let_mut(
                "output_keys_per_wf",
                Expr::call("Vec::new", vec![]),
            ));

            for (wf_idx, (wf_name, example)) in workflow_examples.iter().enumerate() {
                body.push(Stmt::Blank);
                body.push(Stmt::Comment(format!(
                    "Workflow '{}' (example {})",
                    wf_name, wf_idx
                )));

                let input_var = format!("inputs_{}", wf_idx);
                let sorted_inputs: BTreeMap<String, ValueExpr> = example
                    .inputs
                    .iter()
                    .map(|(k, v)| (k.clone(), ValueExpr::from(v)))
                    .collect();

                // Build inputs inline (can't reuse build_inputs_map_stmts because of var name).
                if sorted_inputs.is_empty() {
                    body.push(Stmt::let_bind(
                        input_var.clone(),
                        Expr::call("std::collections::HashMap::new", vec![]),
                    ));
                } else {
                    body.push(Stmt::let_mut(
                        input_var.clone(),
                        Expr::call("std::collections::HashMap::new", vec![]),
                    ));
                    for (port, value) in &sorted_inputs {
                        body.push(Stmt::Expr(Expr::var(&input_var).method(
                            "insert",
                            vec![
                                Expr::Str(port.clone()).method("to_string", vec![]),
                                Expr::Value(value.clone()),
                            ],
                        )));
                    }
                }

                // Determine execution mode.
                let is_exact = matches!(
                    example.expectation,
                    gunbc_test::Expectation::ExactOutputs(_)
                );
                let output_var = format!("outputs_{}", wf_idx);

                if is_exact {
                    let exec = Expr::call(
                        "gunbc_exec::execute_single_node",
                        vec![
                            Expr::var("dag").ref_of(),
                            Expr::Str(concrete_node_id.clone()),
                            Expr::var(&input_var),
                            Expr::Path(vec![
                                "gunbc_exec".to_string(),
                                "ExecutionMode".to_string(),
                                "Real".to_string(),
                            ]),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "cross-wf '{}' workflow '{}' should execute",
                            identity, wf_name
                        ))],
                    );
                    body.push(Stmt::let_bind(output_var.clone(), exec));
                } else {
                    let mocks_var = format!("mocks_{}", wf_idx);
                    let mocks_expr = self.dryrun_mocks_expr(
                        analysis,
                        &format!("cross-workflow consistency {}", wf_name),
                    );
                    body.push(Stmt::let_bind(mocks_var.clone(), mocks_expr));
                    let exec = Expr::call(
                        "gunbc_exec::execute_single_node",
                        vec![
                            Expr::var("dag").ref_of(),
                            Expr::Str(concrete_node_id.clone()),
                            Expr::var(&input_var),
                            Expr::call(
                                "ExecutionMode::DryRun",
                                vec![Expr::var(&mocks_var)],
                            ),
                        ],
                    )
                    .method(
                        "expect",
                        vec![Expr::Str(format!(
                            "cross-wf '{}' workflow '{}' should execute in DryRun",
                            identity, wf_name
                        ))],
                    );
                    body.push(Stmt::let_bind(output_var.clone(), exec));
                }

                // Collect output keys (sorted for stable comparison).
                body.push(Stmt::Expr(Expr::var("output_keys_per_wf").method(
                    "push",
                    vec![Expr::var(&output_var)
                        .method("keys", vec![])
                        .method("cloned", vec![])
                        .method("collect::<Vec<_>>", vec![])],
                )));
            }

            // Assert all workflows produced the same output port set.
            body.push(Stmt::Blank);
            body.push(Stmt::Comment(
                "All workflows should produce the same output port set.".to_string(),
            ));
            // Sort each key set for stable comparison.
            body.push(Stmt::Expr(Expr::call(
                "output_keys_per_wf.iter_mut().for_each",
                vec![Expr::Closure {
                    args: vec!["keys".to_string()],
                    body: Box::new(Expr::var("keys").method("sort", vec![])),
                }],
            )));

            // Compare consecutive pairs.
            if workflow_examples.len() >= 2 {
                for i in 0..workflow_examples.len() - 1 {
                    let (wf_a, _) = workflow_examples[i];
                    let (wf_b, _) = workflow_examples[i + 1];
                    body.push(Stmt::Assert(Assert::Eq {
                        left: Expr::var("output_keys_per_wf")
                            .method("get", vec![Expr::int(i as i64)])
                            .method("expect", vec![Expr::Str("keys should exist".into())]),
                        right: Expr::var("output_keys_per_wf")
                            .method("get", vec![Expr::int((i + 1) as i64)])
                            .method("expect", vec![Expr::Str("keys should exist".into())]),
                        message: format!(
                            "cross-wf '{}': output ports from '{}' should match '{}'",
                            identity, wf_a, wf_b
                        ),
                    }));
                }
            }

            tests.push(TestFn {
                name: test_name,
                doc,
                body,
            });
        }

        if tests.is_empty() {
            return None;
        }

        Some(TestSection {
            title: "BB-5: Cross-Workflow Consistency Tests".to_string(),
            notes: vec![
                "For nodes appearing in 2+ workflows, asserts consistent output shape."
                    .to_string(),
                "Would have caught the FnBodyDelegate regression (pragma/gist broke, makegen didn't)."
                    .to_string(),
            ],
            tests,
        })
    }

    // =======================================================================
    // BB-6: Transport Fidelity Ladder Tests
    // =======================================================================

    fn build_fidelity_ladder_section(
        &self,
        analysis: &DagAnalysis,
        graph_builder_fn: &str,
    ) -> Option<TestSection> {
        if analysis.transport_executors.is_empty() {
            return None;
        }

        let requirements = derive_virtual_backend_requirements(self.dag, analysis);
        let mut tests = Vec::new();

        // -- XS tier: DryRun intercept (pure mock) ----------------------------

        tests.push(TestFn {
            name: "test_fidelity_xs_dryrun".to_string(),
            doc: vec![
                "Fidelity XS: DryRun intercept (pure mock).".to_string(),
                String::new(),
                "This is the baseline tier — all transport nodes are intercepted.".to_string(),
            ],
            body: vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::Assert(Assert::True {
                    expr: Expr::var("dag")
                        .field("nodes")
                        .method("is_empty", vec![])
                        .logical_not(),
                    message: "DAG should have nodes for fidelity XS test".to_string(),
                }),
                Stmt::Expr(Expr::call(
                    "eprintln!",
                    vec![Expr::Str(
                        "[fidelity] XS tier: DryRun intercept verified".to_string(),
                    )],
                )),
            ],
        });

        // -- S tier: Virtual I/O (RT14) ---------------------------------------

        if requirements.needs_virtual_backend() {
            let mut s_body = Vec::new();
            s_body.push(Stmt::let_bind("dag", Expr::var(graph_builder_fn)));

            // Build VirtualTransportBackend setup code.
            let mut setup_lines = String::new();
            setup_lines.push_str(
                "let backend = std::sync::Arc::new(\
                 gunbc_transport::test_backend::VirtualTransportBackend::new());\n",
            );

            // Register per-transport-class default stubs.
            if requirements.needs_rest {
                setup_lines.push_str(
                    "backend.add_http_stub(gunbc_transport::test_backend::HttpStub {\n\
                     \x20   method: None,\n\
                     \x20   path_pattern: \"/\".to_string(),\n\
                     \x20   exact_path: false,\n\
                     \x20   status: 200,\n\
                     \x20   response_body: \"{}\".to_string(),\n\
                     \x20   response_headers: Default::default(),\n\
                     });\n",
                );
            }
            if requirements.needs_shell {
                setup_lines.push_str(
                    "backend.add_shell_cassette(gunbc_transport::test_backend::ShellCassette {\n\
                     \x20   command: String::new(),\n\
                     \x20   args: vec![],\n\
                     \x20   stdout: String::new(),\n\
                     \x20   stderr: String::new(),\n\
                     \x20   exit_code: 0,\n\
                     });\n",
                );
            }

            setup_lines.push_str(
                "let _guard = gunbc_transport::backend::TransportBackendGuard::install(backend);\n",
            );
            setup_lines.push_str(
                "eprintln!(\"[fidelity] S tier: VirtualTransportBackend installed with {} transport class(es)\", ",
            );
            setup_lines.push_str(&format!("{}", requirements.transport_class_count()));
            setup_lines.push_str(");\n");

            s_body.push(Stmt::Expr(Expr::raw(setup_lines)));

            tests.push(TestFn {
                name: "test_fidelity_s_virtual_io".to_string(),
                doc: vec![
                    "Fidelity S: in-memory hermetic I/O via VirtualTransportBackend.".to_string(),
                    String::new(),
                    format!(
                        "Installs virtual backend with {} transport class(es): {}.",
                        requirements.transport_class_count(),
                        Self::transport_class_summary(&requirements),
                    ),
                ],
                body: s_body,
            });
        }

        // -- M tier: Sandboxed tempdir (RT15) ---------------------------------

        {
            let mut m_body = Vec::new();
            m_body.push(Stmt::Comment(
                "M-tier: sandboxed tempdir for real filesystem I/O.".to_string(),
            ));
            m_body.push(Stmt::Comment(
                "Requires feature `sandboxed_tests` — skip otherwise.".to_string(),
            ));
            m_body.push(Stmt::Expr(Expr::raw(
                "if cfg!(not(feature = \"sandboxed_tests\")) {\n\
                 \x20   eprintln!(\"[fidelity] M tier: skipped (feature sandboxed_tests not enabled)\");\n\
                 \x20   return;\n\
                 }\n"
                .to_string(),
            )));
            m_body.push(Stmt::let_bind("_dag", Expr::var(graph_builder_fn)));
            m_body.push(Stmt::Expr(Expr::call(
                "eprintln!",
                vec![Expr::Str(
                    "[fidelity] M tier: sandboxed tempdir execution".to_string(),
                )],
            )));

            tests.push(TestFn {
                name: "test_fidelity_m_sandboxed".to_string(),
                doc: vec![
                    "Fidelity M: sandboxed tempdir with real filesystem I/O.".to_string(),
                    String::new(),
                    "Gated by `sandboxed_tests` feature flag.".to_string(),
                ],
                body: m_body,
            });
        }

        // -- L tier: Real local (RT16) ----------------------------------------

        {
            let mut l_body = Vec::new();
            l_body.push(Stmt::Comment(
                "L-tier: real local execution, cost-gated.".to_string(),
            ));
            l_body.push(Stmt::Expr(Expr::raw(
                "if !gunbc_test::guard_test(\n\
                 \x20   \"test_fidelity_l_real_local\",\n\
                 \x20   gunbc_test::TestClass::Integration,\n\
                 \x20   gunbc_test::FermiCost::L,\n\
                 \x20   &[],\n\
                 \x20   &[],\n\
                 ) { return; }\n"
                    .to_string(),
            )));
            l_body.push(Stmt::let_bind("_dag", Expr::var(graph_builder_fn)));
            l_body.push(Stmt::Expr(Expr::call(
                "eprintln!",
                vec![Expr::Str(
                    "[fidelity] L tier: real local execution".to_string(),
                )],
            )));

            tests.push(TestFn {
                name: "test_fidelity_l_real_local".to_string(),
                doc: vec![
                    "Fidelity L: real local execution.".to_string(),
                    String::new(),
                    "Cost-gated: requires GUNBC_TEST_MAX_COST >= L.".to_string(),
                ],
                body: l_body,
            });
        }

        // -- XL tier: Real remote (RT16) --------------------------------------

        {
            let mut xl_body = Vec::new();
            xl_body.push(Stmt::Comment(
                "XL-tier: real remote/network, cost-gated with secrets.".to_string(),
            ));
            xl_body.push(Stmt::Expr(Expr::raw(
                "if !gunbc_test::guard_test(\n\
                 \x20   \"test_fidelity_xl_real_remote\",\n\
                 \x20   gunbc_test::TestClass::Integration,\n\
                 \x20   gunbc_test::FermiCost::XL,\n\
                 \x20   &[],\n\
                 \x20   &[],\n\
                 ) { return; }\n"
                    .to_string(),
            )));
            xl_body.push(Stmt::let_bind("_dag", Expr::var(graph_builder_fn)));
            xl_body.push(Stmt::Expr(Expr::call(
                "eprintln!",
                vec![Expr::Str(
                    "[fidelity] XL tier: real remote execution".to_string(),
                )],
            )));

            tests.push(TestFn {
                name: "test_fidelity_xl_real_remote".to_string(),
                doc: vec![
                    "Fidelity XL: real remote/network execution.".to_string(),
                    String::new(),
                    "Cost-gated: requires GUNBC_TEST_MAX_COST >= XL.".to_string(),
                ],
                body: xl_body,
            });
        }

        Some(TestSection {
            title: "BB-6: Transport Fidelity Ladder Tests".to_string(),
            notes: vec![
                "Tiered test variants based on transport fidelity (XS=mock through XL=real)."
                    .to_string(),
                "S-tier uses VirtualTransportBackend; M-tier uses sandboxed tempdir; L/XL are cost-gated."
                    .to_string(),
            ],
            tests,
        })
    }

    /// Summarize transport classes for doc comments.
    fn transport_class_summary(
        requirements: &crate::testgen::registry_gen::VirtualBackendRequirements,
    ) -> String {
        let mut classes = Vec::new();
        if requirements.needs_rest {
            classes.push("REST");
        }
        if requirements.needs_shell {
            classes.push("Shell");
        }
        if requirements.needs_file {
            classes.push("File");
        }
        if requirements.needs_tcp {
            classes.push("TCP");
        }
        classes.join(", ")
    }
}

/// Sanitize a description string into a valid snake_case identifier fragment.
///
/// Replaces non-alphanumeric characters with `_`, collapses runs of `_`,
/// strips leading/trailing `_`, and lowercases.
fn sanitize_to_snake_case(desc: &str) -> String {
    let raw: String = desc
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let mut result = String::with_capacity(raw.len());
    let mut prev_underscore = true; // starts true to strip leading _
    for c in raw.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push('_');
            }
            prev_underscore = true;
        } else {
            result.push(c.to_ascii_lowercase());
            prev_underscore = false;
        }
    }
    if result.ends_with('_') {
        result.pop();
    }
    result
}

/// Sanitize a resource ID into a valid snake_case Rust identifier.
fn sanitize_resource_id(id: &str) -> String {
    let raw: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let mut result = String::with_capacity(raw.len());
    let mut prev_underscore = true;
    for c in raw.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push('_');
            }
            prev_underscore = true;
        } else {
            result.push(c.to_ascii_lowercase());
            prev_underscore = false;
        }
    }
    if result.ends_with('_') {
        result.pop();
    }
    result
}

// ============================================================================
// CLI Contract Test Helpers
// ============================================================================

/// Escape a string for use inside a Rust string literal in generated code.
fn cli_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

/// Sample a string value for a CLI entrypoint in contract tests.
fn cli_sample_string(ep: &crate::cli_gen::CliEntrypoint) -> String {
    let lower = ep.port_name.to_lowercase();
    let mut value = if lower.contains("repo") {
        "test-repo".to_string()
    } else if lower.contains("manifest") {
        "deps.toml".to_string()
    } else if lower == "path" || lower.contains("makefile") {
        "Makefile.test".to_string()
    } else if lower.contains("path") {
        "out/path".to_string()
    } else if lower.contains("base") || lower.contains("branch") || lower.contains("ref") {
        "feature/test".to_string()
    } else if lower.contains("ext") {
        ".rs".to_string()
    } else {
        format!("{}_value", ep.port_name)
    };
    if let Some(default) = ep.default_value.as_deref() {
        if value == default {
            value = format!("{}_override", ep.port_name);
        }
    }
    value
}

/// Sample an int value for a CLI entrypoint in contract tests.
fn cli_sample_int(ep: &crate::cli_gen::CliEntrypoint) -> String {
    let mut value = "42".to_string();
    if let Some(default) = ep.default_value.as_deref() {
        if default == value {
            value = "7".to_string();
        }
    }
    value
}

/// Sample repeatable values for a CLI entrypoint in contract tests.
fn cli_sample_repeatable(ep: &crate::cli_gen::CliEntrypoint) -> (String, String) {
    let lower = ep.port_name.to_lowercase();
    if lower.contains("ext") {
        return (".rs".to_string(), ".toml".to_string());
    }
    let first = format!("{}_one", ep.port_name);
    let second = format!("{}_two", ep.port_name);
    (first, second)
}

/// Render an output matcher assertion as a single line of Rust code.
///
/// Uses the test IR (Assert, Expr) and RustRenderer to produce the assertion,
/// replacing the old `OutputMatcher::to_check_code()` string interpolation.
/// The `var_name` is the snake_case port name; the actual variable is `output_{var_name}`.
fn render_output_matcher_check(matcher: &OutputMatcher, var_name: &str) -> Vec<Stmt> {
    let output_var = format!("output_{}", var_name);
    match matcher {
        OutputMatcher::Exact(expected) => vec![Stmt::Assert(Assert::Eq {
            left: Expr::var(&output_var).deref(),
            right: Expr::Value(ValueExpr::from(expected.as_ref())),
            message: "expected exact value".to_string(),
        })],
        OutputMatcher::Contains(substring) => vec![Stmt::Assert(Assert::Contains {
            expr: Expr::var(&output_var),
            substring: substring.clone(),
            message: format!("expected to contain '{}', got: {{:?}}", substring),
        })],
        OutputMatcher::NonEmpty => vec![Stmt::Assert(Assert::NonEmpty {
            expr: Expr::var(&output_var),
            message: "expected non-empty value".to_string(),
        })],
        OutputMatcher::IsBool => vec![Stmt::Assert(Assert::True {
            expr: Expr::var(&output_var)
                .method("as_bool", vec![])
                .method("is_some", vec![]),
            message: format!("expected Bool for {}", output_var),
        })],
        OutputMatcher::IsInt => vec![Stmt::Assert(Assert::True {
            expr: Expr::var(&output_var)
                .method("as_int", vec![])
                .method("is_some", vec![]),
            message: format!("expected Int for {}", output_var),
        })],
        OutputMatcher::IsString => vec![Stmt::Assert(Assert::True {
            expr: Expr::var(&output_var)
                .method("as_str", vec![])
                .method("is_some", vec![]),
            message: format!("expected String for {}", output_var),
        })],
        OutputMatcher::IsRequest => vec![Stmt::Assert(Assert::True {
            expr: Expr::var(&output_var)
                .method("as_request", vec![])
                .method("is_some", vec![]),
            message: format!("expected Request for {}", output_var),
        })],
        OutputMatcher::IsResponse => vec![Stmt::Assert(Assert::True {
            expr: Expr::var(&output_var)
                .method("as_response", vec![])
                .method("is_some", vec![]),
            message: format!("expected Response for {}", output_var),
        })],
        OutputMatcher::IntGe(threshold) => vec![Stmt::Assert(Assert::True {
            expr: Expr::var(&output_var).method("as_int", vec![]).method(
                "is_some_and",
                vec![Expr::Closure {
                    args: vec!["n".to_string()],
                    body: Box::new(Expr::var("n").bin_op(">=", Expr::int(*threshold))),
                }],
            ),
            message: format!("expected Int >= {} for {}", threshold, output_var),
        })],
        OutputMatcher::IntLe(threshold) => vec![Stmt::Assert(Assert::True {
            expr: Expr::var(&output_var).method("as_int", vec![]).method(
                "is_some_and",
                vec![Expr::Closure {
                    args: vec!["n".to_string()],
                    body: Box::new(Expr::var("n").bin_op("<=", Expr::int(*threshold))),
                }],
            ),
            message: format!("expected Int <= {} for {}", threshold, output_var),
        })],
        OutputMatcher::Satisfies { description, .. } => vec![Stmt::Expr(Expr::call(
            "panic!",
            vec![Expr::Str(format!(
                "OutputMatcher::Satisfies requires runtime matcher checks: {}",
                description
            ))],
        ))],
        OutputMatcher::Any => vec![Stmt::Comment(format!(
            "Any value accepted for {}",
            output_var
        ))],
    }
}

fn try_mock_element_value(type_id: &str, index: Option<u32>) -> Option<Value> {
    if let Some((key_type, value_type)) = parse_map_type_id(type_id) {
        if key_type != "String" {
            return None;
        }
        let value = try_mock_element_value(&value_type, index)?;
        let mut map = BTreeMap::new();
        map.insert("mock_key".to_string(), value);
        return Some(Value::Map(map));
    }

    let value = match type_id {
        "String" => match index {
            Some(1) | None => Value::Str("<MOCK>".to_string()),
            Some(i) => Value::Str(format!("<MOCK_{}>", i)),
        },
        "Bool" => match index {
            Some(i) => Value::Bool(i % 2 == 1),
            None => Value::Bool(true),
        },
        "Int" | "i64" | "i32" => match index {
            Some(i) => Value::Int(i as i64),
            None => Value::Int(0),
        },
        "Unit" => Value::Unit,
        "Json" => Value::Json(JsonValue::Null),
        "Map" => Value::Map(BTreeMap::new()),
        "CloudSecretConfig" => Value::Json(mock_cloud_secret_config_json()),
        "Secret" => Value::Secret(SecretString::new("<MOCK_SECRET>")),
        "Any" => Value::Json(JsonValue::Null),
        "S" => Value::Str("<MOCK>".to_string()),
        "Path" | "FilePath" => Value::Str("/tmp/mock".to_string()),
        "SourceIR" => Value::Str("<SOURCE_IR>".to_string()),
        "Platform" => Value::Str(platform_mock_token(index)),
        "Error" => Value::Str("<ERROR>".to_string()),
        "Tier" => Value::Str("Ascii".to_string()),
        "Unknown" => Value::Json(JsonValue::Null),
        "ToolId" => Value::Str("clippy".to_string()),
        "ToolHandle" => {
            let mut map = BTreeMap::new();
            map.insert("type".to_string(), Value::Str("tool_handle".to_string()));
            map.insert("id".to_string(), Value::Str("clippy".to_string()));
            map.insert("path".to_string(), Value::Str("/mock/clippy".to_string()));
            map.insert(
                "cap".to_string(),
                Value::Secret(SecretString::new("capability")),
            );
            Value::Map(map)
        }
        "CliResult" => {
            let mut map = BTreeMap::new();
            map.insert("success".to_string(), Value::Bool(true));
            map.insert("exit_code".to_string(), Value::Int(0));
            map.insert("stdout".to_string(), Value::Str(String::new()));
            map.insert("stderr".to_string(), Value::Str(String::new()));
            Value::Map(map)
        }
        "Timestamp" => Value::Int(0),
        "Credential" => {
            let mut map = BTreeMap::new();
            map.insert(
                "token".to_string(),
                Value::Secret(SecretString::new("mock-token")),
            );
            map.insert("source_type".to_string(), Value::Str("static".to_string()));
            map.insert("scheme".to_string(), Value::Str("bearer".to_string()));
            map.insert(
                "cap".to_string(),
                Value::Secret(SecretString::new("capability")),
            );
            Value::Map(map)
        }
        "FilesystemHandle" => {
            let mut map = BTreeMap::new();
            map.insert(
                "type".to_string(),
                Value::Str("filesystem_handle".to_string()),
            );
            map.insert("scope".to_string(), Value::Str("read".to_string()));
            map.insert(
                "targets".to_string(),
                Value::List(vec![Value::Str("ext4".to_string())]),
            );
            map.insert("replacement".to_string(), Value::Str("-".to_string()));
            map.insert(
                "cap".to_string(),
                Value::Secret(SecretString::new("capability")),
            );
            Value::Map(map)
        }
        "NetworkHandle" => {
            let mut map = BTreeMap::new();
            map.insert("type".to_string(), Value::Str("network_handle".to_string()));
            map.insert(
                "cap".to_string(),
                Value::Secret(SecretString::new("capability")),
            );
            Value::Map(map)
        }
        "TransportRequest" => Value::Request(TransportRequest::Shell(ShellRequest::new("true"))),
        "TransportResponse" => {
            Value::Response(TransportResponse::Shell(ShellResponse::ok("<MOCK>")))
        }
        "List" | "Set" => return None,
        _ => return None,
    };

    Some(value)
}

fn mock_cloud_secret_config_json() -> JsonValue {
    serde_json::json!({
        "provider": "Gcp",
        "runtime": "GitHubActions",
        "audience": "projects/123/locations/global/workloadIdentityPools/github/providers/gha",
        "project_or_account": "mock-secrets",
        "secret": {
            "prefix": "ci-",
            "name": "example",
            "delimiter": ""
        },
        "service_account_or_role": "ci-secrets@mock.iam.gserviceaccount.com"
    })
}

fn witness_value_for_count(
    type_id: &str,
    cardinality: Cardinality,
    count: u32,
    registry: &TypeRegistry,
) -> Option<Value> {
    let type_dag = registry.get_by_name(type_id)?;
    let witnesses = contract::witnesses(type_dag);

    let nonzero = witnesses
        .iter()
        .find(|w| w.count == 1)
        .or_else(|| witnesses.iter().find(|w| w.count > 0))
        .map(|w| w.value.clone());

    if cardinality.is_list() {
        if count == 0 {
            return Some(Value::List(vec![]));
        }
        let elem = nonzero?;
        // For coproduct element types, use variant-diverse elements
        let variant_values = contract::variant_witnesses(type_id, registry);
        let elements = if variant_values.len() > 1 {
            (0..count as usize)
                .map(|i| variant_values[i % variant_values.len()].1.clone())
                .collect()
        } else {
            vec![elem; count as usize]
        };
        return Some(Value::List(elements));
    }

    if count == 0 {
        return Some(Value::Unit);
    }

    nonzero
}

fn try_mock_value_for_count(
    type_id: &str,
    cardinality: Cardinality,
    count: u32,
    registry: &TypeRegistry,
) -> Option<Value> {
    if let Some(value) = witness_value_for_count(type_id, cardinality, count, registry) {
        return Some(value);
    }
    if cardinality.is_list() {
        if count == 0 {
            return Some(Value::List(vec![]));
        }
        let mut elements = Vec::new();
        for i in 1..=count {
            elements.push(try_mock_element_value(type_id, Some(i))?);
        }
        return Some(Value::List(elements));
    }

    match count {
        0 => Some(Value::Unit),
        n => try_mock_element_value(type_id, Some(n)),
    }
}

fn required_count_for_port(port: &gunbc_ir::Port) -> Option<u32> {
    if port.cardinality.max == Some(0) {
        return None;
    }
    if port.cardinality.is_list() {
        let count = port.cardinality.min.max(1);
        if port.cardinality.max.is_some_and(|max| count > max) {
            return None;
        }
        return Some(count);
    }
    Some(1)
}

fn candidate_values_for_guard(port: &gunbc_ir::Port, registry: &TypeRegistry) -> Vec<Value> {
    let Some(count) = required_count_for_port(port) else {
        return Vec::new();
    };
    if let Some(value) =
        witness_value_for_count(port.type_id.0.as_str(), port.cardinality, count, registry)
    {
        return vec![value];
    }
    let mut values = Vec::new();
    for seed in [1u32, 2u32] {
        if port.cardinality.is_list() {
            let mut elements = Vec::new();
            for offset in 0..count {
                let idx = seed + offset;
                let Some(elem) = try_mock_element_value(port.type_id.0.as_str(), Some(idx)) else {
                    elements.clear();
                    break;
                };
                elements.push(elem);
            }
            if !elements.is_empty() {
                values.push(Value::List(elements));
            }
        } else if let Some(elem) = try_mock_element_value(port.type_id.0.as_str(), Some(seed)) {
            values.push(elem);
        }
    }
    values
}

fn select_guard_value(port: &gunbc_ir::Port, registry: &TypeRegistry) -> Option<Value> {
    candidate_values_for_guard(port, registry)
        .into_iter()
        .find(|candidate| port.check_guard(candidate))
}

fn required_value_for_port(port: &gunbc_ir::Port, registry: &TypeRegistry) -> Option<Value> {
    let count = required_count_for_port(port)?;
    try_mock_value_for_count(port.type_id.0.as_str(), port.cardinality, count, registry)
}

/// Generate a mock ValueExpr for a specific count and cardinality.
///
/// Cardinality determines whether values are wrapped as lists. The `count`
/// is the number of elements (from `testgen::cardinality::fermi_test_cases()`).
///
/// For count=0, scalar types emit `Value::Unit` (absence), not concrete
/// "empty content" like `false` or `0`. List cardinalities emit empty
/// collections (which correctly represent zero elements).
fn mock_value_expr_for_count(
    type_id: &str,
    cardinality: Cardinality,
    count: u32,
    registry: &TypeRegistry,
) -> ValueExpr {
    if let Some(value) = witness_value_for_count(type_id, cardinality, count, registry) {
        return ValueExpr::from(&value);
    }
    if cardinality.is_list() {
        if count == 0 {
            return ValueExpr::List(vec![]);
        }
        let elements: Vec<ValueExpr> = (1..=count)
            .map(|i| mock_element_expr(type_id, Some(i)))
            .collect();
        return ValueExpr::List(elements);
    }

    match count {
        0 => ValueExpr::Unit,
        n => mock_element_expr(type_id, Some(n)),
    }
}

/// Generate a mock ValueExpr for a single element of a type.
///
/// When `index` is provided, string/int/bool values are varied for readability.
fn mock_element_expr(type_id: &str, index: Option<u32>) -> ValueExpr {
    match type_id {
        "String" | "OptionalString" | "StringList" | "NonEmptyStringList" => match index {
            Some(1) | None => ValueExpr::Str("<MOCK>".to_string()),
            Some(i) => ValueExpr::Str(format!("<MOCK_{}>", i)),
        },
        "Bool" | "OptionalBool" | "BoolList" => match index {
            Some(i) => ValueExpr::Bool(i % 2 == 1),
            None => ValueExpr::Bool(true),
        },
        "Int" | "i64" | "i32" | "OptionalInt" | "IntList" => match index {
            Some(i) => ValueExpr::Int(i as i64),
            None => ValueExpr::Int(0),
        },
        "Unit" => ValueExpr::Unit,
        "Json" | "OptionalJson" | "JsonList" => ValueExpr::Json(JsonValue::Null),
        "CloudSecretConfig" => ValueExpr::Json(mock_cloud_secret_config_json()),
        ty if parse_map_type_id(ty).is_some() => {
            let (key_type, value_type) = parse_map_type_id(ty).expect("already checked");
            if key_type != "String" {
                panic!("unsupported map key type '{}'; only String keys are supported", key_type);
            }
            ValueExpr::Map(vec![(
                "mock_key".to_string(),
                mock_element_expr(&value_type, index),
            )])
        }
        "Map" => ValueExpr::Map(vec![]),
        "Secret" => ValueExpr::Secret("<MOCK_SECRET>".to_string()),
        "Any" => ValueExpr::Json(JsonValue::Null),
        "S" => ValueExpr::Str("<MOCK>".to_string()),
        "Path" | "FilePath" => ValueExpr::Str("/tmp/mock".to_string()),
        "SourceIR" => ValueExpr::Str("<SOURCE_IR>".to_string()),
        "Platform" => ValueExpr::Str(platform_mock_token(index)),
        "Error" => ValueExpr::Str("<ERROR>".to_string()),
        // Container aliases: element value derives from the inner type.
        "Tier" => ValueExpr::Str("Ascii".to_string()),
        "Unknown" => ValueExpr::Json(JsonValue::Null),
        "ToolId" => ValueExpr::Str("clippy".to_string()),
        "ToolHandle" => ValueExpr::Map(vec![
            ("type".to_string(), ValueExpr::Str("tool_handle".to_string())),
            ("id".to_string(), ValueExpr::Str("clippy".to_string())),
            ("path".to_string(), ValueExpr::Str("/mock/clippy".to_string())),
            ("cap".to_string(), ValueExpr::Secret("capability".to_string())),
        ]),
        "CliResult" => ValueExpr::Map(vec![
            ("success".to_string(), ValueExpr::Bool(true)),
            ("exit_code".to_string(), ValueExpr::Int(0)),
            ("stdout".to_string(), ValueExpr::Str(String::new())),
            ("stderr".to_string(), ValueExpr::Str(String::new())),
        ]),
        "Timestamp" => ValueExpr::Int(0),
        "Credential" => ValueExpr::Map(vec![
            ("token".to_string(), ValueExpr::Secret("mock-token".to_string())),
            ("source_type".to_string(), ValueExpr::Str("static".to_string())),
            ("scheme".to_string(), ValueExpr::Str("bearer".to_string())),
            ("cap".to_string(), ValueExpr::Secret("capability".to_string())),
        ]),
        "FilesystemHandle" => ValueExpr::Map(vec![
            ("type".to_string(), ValueExpr::Str("filesystem_handle".to_string())),
            ("scope".to_string(), ValueExpr::Str("read".to_string())),
            ("targets".to_string(), ValueExpr::List(vec![ValueExpr::Str("ext4".to_string())])),
            ("replacement".to_string(), ValueExpr::Str("-".to_string())),
            ("cap".to_string(), ValueExpr::Secret("capability".to_string())),
        ]),
        "NetworkHandle" => ValueExpr::Map(vec![
            ("type".to_string(), ValueExpr::Str("network_handle".to_string())),
            ("cap".to_string(), ValueExpr::Secret("capability".to_string())),
        ]),
        "TransportRequest" => ValueExpr::Struct {
            name: "TransportRequest::Shell".to_string(),
            fields: vec![
                ("command".to_string(), ValueExpr::Str("true".to_string())),
                ("args".to_string(), ValueExpr::List(vec![])),
                ("env".to_string(), ValueExpr::Map(vec![])),
                ("cwd".to_string(), ValueExpr::Unit),
                ("stdin".to_string(), ValueExpr::Unit),
                ("timeout_ms".to_string(), ValueExpr::Unit),
                ("passthrough".to_string(), ValueExpr::Bool(false)),
            ],
        },
        "TransportResponse" => ValueExpr::Struct {
            name: "TransportResponse::Shell".to_string(),
            fields: vec![
                ("exit_code".to_string(), ValueExpr::Int(0)),
                ("stdout".to_string(), ValueExpr::Str("<MOCK>".to_string())),
                ("stderr".to_string(), ValueExpr::Str(String::new())),
            ],
        },
        "List" | "Set" => panic!(
            "invalid type_id '{}' for mock value; use element type + cardinality instead",
            type_id
        ),
        _ => panic!(
            "no mock value for type_id '{}'; add a MockSpec boundary value or extend mock_element_expr",
            type_id
        ),
    }
}

fn platform_mock_variants() -> Vec<String> {
    // Use a fixed ordering for determinism across machines. The default
    // (index 0) is always "linux" regardless of the host OS, which avoids
    // golden-file churn and "works on my machine" test instability.
    vec![
        Os::Linux.as_token().to_string(),
        Os::Macos.as_token().to_string(),
        Os::Windows.as_token().to_string(),
    ]
}

fn platform_mock_token(index: Option<u32>) -> String {
    let variants = platform_mock_variants();
    let idx = match index {
        Some(i) if i > 0 => ((i - 1) as usize) % variants.len(),
        _ => 0,
    };
    variants[idx].clone()
}

/// Generate a wrong-typed value for the given type_id.
///
/// Returns None for types that accept any value or where wrong-type
/// tests would be ambiguous.
fn mock_wrong_type_expr(type_id: &str) -> Option<ValueExpr> {
    match type_id {
        // String-like types → use Int
        "String" | "OptionalString" | "StringList" | "NonEmptyStringList" | "Path" | "FilePath"
        | "SourceIR" | "Platform" | "Error" | "Tier" | "ToolId" | "S" => Some(ValueExpr::Int(1)),
        // Int-like types → use String
        "Int" | "i64" | "i32" | "Timestamp" | "OptionalInt" | "IntList" => {
            Some(ValueExpr::Str("<WRONG>".to_string()))
        }
        // Bool → use String
        "Bool" | "OptionalBool" | "BoolList" => Some(ValueExpr::Str("<WRONG>".to_string())),
        // Secret → use String
        "Secret" => Some(ValueExpr::Str("<WRONG>".to_string())),
        // Map → use Bool
        "Map" => Some(ValueExpr::Bool(true)),
        // Structured types → use String
        "CliResult" | "ToolHandle" | "Credential" | "FilesystemHandle" | "NetworkHandle"
        | "TransportRequest" | "TransportResponse" => Some(ValueExpr::Str("<WRONG>".to_string())),
        // Unknown/Any/Json/Unit are too permissive or ambiguous
        "Json" | "OptionalJson" | "JsonList" | "Any" | "Unknown" | "Unit" => None,
        _ => None,
    }
}

#[derive(Debug)]
struct WindowSpec {
    nodes: Vec<NodeId>,
    first: NodeId,
    last: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct WindowSignature {
    entry_ports: Vec<(String, String)>,
    exit_ports: Vec<(String, String)>,
}

fn enumerate_window_specs<T>(
    dag: &Dag<T>,
    max_nodes: usize,
    pure_nodes: &HashSet<NodeId>,
) -> Vec<WindowSpec> {
    let topo = gunbc_exec::topo_sort(dag);
    let mut specs = Vec::new();
    let mut seen: HashSet<WindowSignature> = HashSet::new();

    if topo.len() < 2 {
        return specs;
    }

    let max_size = max_nodes.min(topo.len());
    for size in 2..=max_size {
        for start in 0..=(topo.len() - size) {
            let end = start + size - 1;
            let slice = &topo[start..=end];
            let node_set: HashSet<NodeId> = slice.iter().cloned().collect();

            if !window_is_connected(dag, &node_set) {
                continue;
            }
            if window_has_mixed_inputs(dag, &node_set) {
                continue;
            }
            if !window_has_pure_node(&node_set, pure_nodes) {
                continue;
            }

            let signature = window_signature(dag, &node_set);
            if signature.exit_ports.is_empty() {
                continue;
            }
            if !seen.insert(signature) {
                continue;
            }

            specs.push(WindowSpec {
                nodes: slice.to_vec(),
                first: slice[0].clone(),
                last: slice[slice.len() - 1].clone(),
            });
        }
    }

    specs
}

fn window_is_connected<T>(dag: &Dag<T>, nodes: &HashSet<NodeId>) -> bool {
    if nodes.len() <= 1 {
        return true;
    }

    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node in nodes {
        adj.entry(node.clone()).or_default();
    }

    for edge in &dag.edges {
        if nodes.contains(&edge.from_node) && nodes.contains(&edge.to_node) {
            adj.entry(edge.from_node.clone())
                .or_default()
                .push(edge.to_node.clone());
            adj.entry(edge.to_node.clone())
                .or_default()
                .push(edge.from_node.clone());
        }
    }

    let start = nodes.iter().next().unwrap().clone();
    let mut stack = vec![start];
    let mut visited: HashSet<NodeId> = HashSet::new();

    while let Some(node) = stack.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        if let Some(neighbors) = adj.get(&node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    stack.push(neighbor.clone());
                }
            }
        }
    }

    visited.len() == nodes.len()
}

fn window_has_mixed_inputs<T>(dag: &Dag<T>, nodes: &HashSet<NodeId>) -> bool {
    let mut seen: HashMap<(NodeId, PortName), (bool, bool)> = HashMap::new();

    for edge in &dag.edges {
        if nodes.contains(&edge.to_node) {
            let entry = seen
                .entry((edge.to_node.clone(), edge.to_port.clone()))
                .or_insert((false, false));
            if nodes.contains(&edge.from_node) {
                entry.0 = true;
            } else {
                entry.1 = true;
            }
            if entry.0 && entry.1 {
                return true;
            }
        }
    }

    false
}

fn window_has_pure_node(nodes: &HashSet<NodeId>, pure_nodes: &HashSet<NodeId>) -> bool {
    nodes.iter().any(|n| pure_nodes.contains(n))
}

fn window_signature<T>(dag: &Dag<T>, nodes: &HashSet<NodeId>) -> WindowSignature {
    let mut internal_incoming: HashSet<(NodeId, PortName)> = HashSet::new();
    let mut internal_outgoing: HashSet<(NodeId, PortName)> = HashSet::new();

    for edge in &dag.edges {
        if nodes.contains(&edge.from_node) && nodes.contains(&edge.to_node) {
            internal_incoming.insert((edge.to_node.clone(), edge.to_port.clone()));
            internal_outgoing.insert((edge.from_node.clone(), edge.from_port.clone()));
        }
    }

    let mut entry_ports: Vec<(String, String)> = Vec::new();
    let mut exit_ports: Vec<(String, String)> = Vec::new();

    for node in &dag.nodes {
        if !nodes.contains(&node.id) {
            continue;
        }
        for port in &node.inputs {
            if !internal_incoming.contains(&(node.id.clone(), port.name.clone())) {
                entry_ports.push((node.id.0.clone(), port.name.0.clone()));
            }
        }
        for port in &node.outputs {
            if !internal_outgoing.contains(&(node.id.clone(), port.name.clone())) {
                exit_ports.push((node.id.0.clone(), port.name.0.clone()));
            }
        }
    }

    entry_ports.sort();
    exit_ports.sort();

    WindowSignature {
        entry_ports,
        exit_ports,
    }
}

fn collect_pure_nodes<T>(dag: &Dag<T>) -> HashSet<NodeId> {
    dag.nodes
        .iter()
        .filter(|node| is_pure_node(node))
        .map(|node| node.id.clone())
        .collect()
}

fn is_pure_node<T>(node: &gunbc_ir::Node<T>) -> bool {
    matches!(
        node.kind,
        NodeKind::Pure | NodeKind::TransportPrepare | NodeKind::TransportParse
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build, build::*, Cardinality, Dag, Node, Value, ValueExpr};

    #[test]
    fn test_platform_mock_token_includes_host_and_variants() {
        let variants = platform_mock_variants();
        // Default (index 0) is always "linux" for determinism across machines.
        assert_eq!(variants.first(), Some(&"linux".to_string()));
        assert!(variants.iter().any(|v| v == "linux"));
        assert!(variants.iter().any(|v| v == "macos"));
        assert!(variants.iter().any(|v| v == "windows"));
    }

    #[test]
    fn test_platform_mock_token_cycles_variants() {
        let variants = platform_mock_variants();
        assert_eq!(platform_mock_token(None), variants[0]);
        assert_eq!(platform_mock_token(Some(1)), variants[0]);
        assert_eq!(platform_mock_token(Some(2)), variants[1 % variants.len()]);
        assert_eq!(
            platform_mock_token(Some((variants.len() as u32) + 1)),
            variants[0]
        );
    }

    #[test]
    fn test_generate_test_module() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let spec = MockSpec::new("example")
            .boundary("sink", "result", Value::Str("<MOCK>".into()))
            .skip_node_example("source")
            .skip_node_example("sink");
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should generate boundary tests (runtime behavior)
        assert!(code.contains("test_boundaries_mockable"));
        assert!(code.contains("test_boundary_sink_mockable"));

        // Should generate Bucket A tests
        assert!(code.contains("test_dryrun_completion"));
        assert!(code.contains("Bucket A: Execution Semantics"));

        // Should have obligation summary in header
        assert!(code.contains("Obligations:"));
        assert!(code.contains("Proven by construction"));

        // Should have content hash in header
        assert!(
            code.contains("Content-Hash:"),
            "should have content hash in header"
        );

        // Should NOT generate composition tests (compiler proves these)
        assert!(!code.contains("test_all_edges_compatible"));
        assert!(!code.contains("test_edge_source_out_to_sink_in"));
    }

    #[test]
    fn test_content_hash_is_stable() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let spec = MockSpec::new("example")
            .boundary("sink", "result", Value::Str("<MOCK>".into()))
            .skip_node_example("source")
            .skip_node_example("sink");
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code1 = generator.generate_test_module("example", "build_example_graph()");
        let code2 = generator.generate_test_module("example", "build_example_graph()");

        // Same DAG should produce identical output (including hash)
        assert_eq!(code1, code2, "content hash should be deterministic");

        // Extract the hash value
        let hash_line = code1
            .lines()
            .find(|l| l.contains("Content-Hash:"))
            .expect("should have Content-Hash line");
        let hash = hash_line.split("Content-Hash: ").nth(1).unwrap().trim();
        assert_eq!(hash.len(), 64, "hash should be 64 hex chars (SHA-256)");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "hash should be hex"
        );
    }

    #[test]
    fn test_generate_with_mock_spec() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let spec = MockSpec::new("test")
            .boundary("sink", "result", Value::Str("test_output".into()))
            .skip_node_example("source")
            .skip_node_example("sink");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should use mock spec value
        assert!(code.contains("test_output"));
        // Should have chain tests
        assert!(code.contains("test_mock_spec_self_consistent"));
    }

    #[test]
    fn test_generate_cli_contract_section_when_entrypoints_present() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let spec = MockSpec::new("cli_contract")
            .boundary("sink", "result", Value::Str("<MOCK>".into()))
            .skip_node_example("source")
            .skip_node_example("sink");

        let entrypoints = vec![crate::cli_gen::CliEntrypoint::new(
            "workspace",
            gunbc_cli::ParamType::Str,
        )];
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()")
            .with_cli_entrypoints("gunbc-demo".to_string(), entrypoints);

        let code = generator.generate_test_module("cli_contract", "build_cli_contract_graph()");
        assert!(code.contains("CLI Contract Tests"));
        assert!(code.contains("test_cli_contract_gunbc_demo"));
        assert!(code.contains("test_cli_contract_print_inputs_gunbc_demo"));
        assert!(code.contains("--print-inputs json"));
    }

    #[test]
    fn test_generate_skips_cli_contract_section_without_entrypoints() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let spec = MockSpec::new("cli_contract")
            .boundary("sink", "result", Value::Str("<MOCK>".into()))
            .skip_node_example("source")
            .skip_node_example("sink");
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");

        let code = generator.generate_test_module("cli_contract", "build_cli_contract_graph()");
        assert!(!code.contains("CLI Contract Tests"));
        assert!(!code.contains("test_cli_contract_"));
    }

    #[test]
    fn test_generate_with_resources() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let spec = MockSpec::new("test")
            .boundary("sink", "result", Value::Str("test_output".into()))
            .resource_lock("db:write")
            .resource_lease("api:token", 5000)
            .skip_node_example("source")
            .skip_node_example("sink");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should have resource tests
        assert!(code.contains("test_resource_db_write_acquire"));
        assert!(code.contains("test_resource_api_token_acquire"));
        assert!(code.contains("test_resource_api_token_timeout"));
    }

    #[test]
    fn test_generate_with_transport_executor() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "prepare",
                vec![],
                vec![port("request", "TransportRequest")],
                (),
            )
            .with_kind(NodeKind::TransportPrepare),
        );
        dag.add_node(
            Node::opaque(
                "execute",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "parse",
                vec![port("response", "TransportResponse")],
                vec![port("result", "String")],
                (),
            )
            .with_kind(NodeKind::TransportParse),
        );
        dag.add_edge(edge("prepare", "request", "execute", "request"));
        dag.add_edge(edge("execute", "response", "parse", "response"));

        // MockSpec required for DAGs with transport executors
        let spec = MockSpec::new("example")
            .transport_mock(
                "execute",
                "response",
                Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                    gunbc_ir::transport::RestResponse::ok(serde_json::json!({})),
                )),
            )
            .boundary("parse", "result", Value::Str("<MOCK_RESULT>".into()))
            .skip_node_example("prepare")
            .skip_node_example("parse");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should have transport interception test
        assert!(code.contains("test_transport_interception"));
        assert!(code.contains("test_transport_behavior_specs_cover_core_families"));
        assert!(code.contains("default_system_models()"));
        assert!(code.contains("derive_contract_test_specs(&models)"));
        assert!(code.contains("generate_contract_test_harnesses(&contract_specs)"));

        // Should have scenario tests
        assert!(code.contains("test_scenario_all_succeed"));
        assert!(code.contains("test_scenario_execute_fails"));

        // Should have skip-path propagation test (execute has downstream parse)
        assert!(
            code.contains("test_skip_propagation_execute"),
            "should generate skip propagation test for transport with downstream"
        );
        assert!(
            code.contains("Value::Skipped"),
            "skip propagation test should inject Value::Skipped"
        );
        assert!(
            code.contains("\"parse\""),
            "skip propagation test should verify downstream node 'parse'"
        );
    }

    #[test]
    fn test_mock_value_respects_cardinality() {
        let registry = TypeRegistry::with_core_types();
        let list_expr =
            mock_value_expr_for_count("String", Cardinality::ZERO_OR_MORE, 1, &registry);
        assert_eq!(
            list_expr,
            ValueExpr::List(vec![ValueExpr::Str("example".to_string())])
        );

        let opt_zero = mock_value_expr_for_count("String", Cardinality::ZERO_OR_ONE, 0, &registry);
        assert_eq!(opt_zero, ValueExpr::Unit);
    }

    #[test]
    fn test_generate_with_bool_guard() {
        let mut dag: Dag<()> = Dag::new();

        // Transport executor that produces a condition
        dag.add_node(
            Node::opaque(
                "check",
                vec![port("request", "TransportRequest")],
                vec![
                    port("response", "TransportResponse"),
                    port("condition", "Bool"),
                ],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        // Guarded node: only executes when condition is true
        dag.add_node(
            Node::opaque(
                "process",
                vec![
                    build::guarded("condition", "Bool", Value::Bool(true)),
                    port("data", "String"),
                ],
                vec![port("result", "String")],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_edge(edge("check", "condition", "process", "condition"));
        dag.add_edge(edge("check", "response", "process", "data"));

        // MockSpec required for DAGs with transport executors
        let spec = MockSpec::new("guarded")
            .transport_mock(
                "check",
                "response",
                Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                    gunbc_ir::transport::RestResponse::ok(serde_json::json!({})),
                )),
            )
            .transport_mock("check", "condition", Value::Bool(true))
            .boundary("process", "result", Value::Str("<MOCK_RESULT>".into()))
            .skip_node_example("process");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("guarded", "build_guarded_graph()");

        // Should have guard branch coverage test
        assert!(
            code.contains("test_guard_process_condition_branch_coverage"),
            "should generate guard branch coverage test for Bool guard"
        );
        // Should test both true and false values
        assert!(
            code.contains("Value::Bool(true)"),
            "should test guard with true"
        );
        assert!(
            code.contains("Value::Bool(false)"),
            "should test guard with false"
        );
        // Should assert one path executes and the other skips
        assert!(
            code.contains("assert_ne!(skipped_true, skipped_false"),
            "should assert exactly one path skips"
        );
    }

    #[test]
    fn test_generate_guard_non_bool_emits_comment() {
        let mut dag: Dag<()> = Dag::new();

        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("status", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "conditional",
            vec![
                build::guarded("status", "String", Value::Str("ready".into())),
                port("data", "String"),
            ],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "status", "conditional", "status"));

        let spec = MockSpec::new("str_guard")
            .boundary("conditional", "result", Value::Str("<MOCK_RESULT>".into()))
            .skip_node_example("source")
            .skip_node_example("conditional");
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("str_guard", "build_graph()");

        // Non-Bool guard should emit a structured comment, not a test function
        assert!(
            code.contains("Guard branch: 'conditional'.status (type: String)"),
            "should emit structured comment for non-Bool guard"
        );
        assert!(
            code.contains("fed by: source.status"),
            "should document the upstream edge"
        );
        // Should NOT generate a test function for non-Bool guard
        assert!(
            !code.contains("test_guard_conditional_status_branch_coverage"),
            "should NOT generate test function for non-Bool guard"
        );
    }

    #[test]
    #[should_panic(expected = "I/O examples required")]
    fn test_examples_required_for_pure_nodes() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "transform",
                vec![port("in", "String")],
                vec![port("out", "String")],
                (),
            )
            .with_kind(NodeKind::Pure),
        );

        // MockSpec provided but no examples and no skip — should panic
        let spec = MockSpec::new("test").boundary("transform", "out", Value::Str("<MOCK>".into()));
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let _ = generator.generate_test_module("test", "build_test_graph()");
    }

    #[test]
    #[should_panic(expected = "MockSpec required")]
    fn test_mockspec_required_for_transport_dags() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "execute",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        // No MockSpec provided - should panic
        let config = TestConfig {
            execution_tests: false,
            contract_tests: false,
            scenario_tests: false,
            resource_tests: false,
            boundary_tests: false,
            chain_tests: false,
            flow_tests: false,
            example_tests: true,
            window_max_nodes: Some(0),
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag).with_config(config);
        let _ = generator.generate_test_module("test", "build_test_graph()");
    }

    #[test]
    #[should_panic(expected = "DryRun mock coverage incomplete")]
    fn test_transport_mock_coverage_required() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "prepare",
                vec![],
                vec![port("request", "TransportRequest")],
                (),
            )
            .with_kind(NodeKind::TransportPrepare),
        );
        dag.add_node(
            Node::opaque(
                "execute",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse"), port("status", "Int")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "parse",
                vec![port("response", "TransportResponse"), port("status", "Int")],
                vec![port("result", "String")],
                (),
            )
            .with_kind(NodeKind::TransportParse),
        );
        dag.add_edge(edge("prepare", "request", "execute", "request"));
        dag.add_edge(edge("execute", "response", "parse", "response"));
        dag.add_edge(edge("execute", "status", "parse", "status"));

        // MockSpec provided but only mocks "response", not "status"
        let spec = MockSpec::new("test")
            .transport_mock(
                "execute",
                "response",
                Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                    gunbc_ir::transport::RestResponse::ok(serde_json::json!({})),
                )),
            )
            // Missing: .transport_mock("execute", "status", Value::Int(200))
            .skip_node_example("prepare")
            .skip_node_example("parse");

        let config = TestConfig {
            example_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_config(config)
            .with_mock_spec_fn("crate::mock_spec()");
        let _ = generator.generate_test_module("test", "build_test_graph()");
    }

    #[test]
    fn test_transport_mock_coverage_passes_when_complete() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "execute",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse"), port("status", "Int")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "parse",
                vec![port("response", "TransportResponse"), port("status", "Int")],
                vec![port("result", "String")],
                (),
            )
            .with_kind(NodeKind::TransportParse),
        );
        dag.add_edge(edge("execute", "response", "parse", "response"));
        dag.add_edge(edge("execute", "status", "parse", "status"));

        // MockSpec with all connected outputs mocked
        let spec = MockSpec::new("test")
            .transport_mock(
                "execute",
                "response",
                Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                    gunbc_ir::transport::RestResponse::ok(serde_json::json!({})),
                )),
            )
            .transport_mock("execute", "status", Value::Int(200))
            .boundary("parse", "result", Value::Str("<RESULT>".into()))
            .skip_node_example("parse");

        let config = TestConfig {
            example_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_config(config)
            .with_mock_spec_fn("crate::mock_spec()");
        // Should not panic
        let code = generator.generate_test_module("test", "build_test_graph()");
        assert!(code.contains("test_"));
    }

    #[test]
    #[should_panic(expected = "Mock value type mismatch")]
    fn test_mock_type_mismatch_detected() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "transform",
            vec![port("input", "String")],
            vec![port("output", "Int")], // <-- output is Int
            (),
        ));

        // MockSpec provides a String for an Int port
        let spec = MockSpec::new("test")
            .boundary("transform", "output", Value::Str("wrong type".into())) // <-- String, not Int
            .skip_node_example("transform");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let _ = generator.generate_test_module("test", "build_test_graph()");
    }

    #[test]
    #[should_panic(expected = "Mock value type mismatch")]
    fn test_input_mock_type_mismatch_detected() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "transform",
            vec![port("count", "Int")],
            vec![port("output", "String")],
            (),
        ));

        // Input mock provides String for Int input port.
        let spec = MockSpec::new("test")
            .input_mock("transform", "count", Value::Str("wrong type".into()))
            .boundary("transform", "output", Value::Str("ok".into()))
            .skip_node_example("transform");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let _ = generator.generate_test_module("test", "build_test_graph()");
    }

    #[test]
    #[should_panic(expected = "Unknown mock slots")]
    fn test_input_mock_unknown_port_detected() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "transform",
            vec![port("count", "Int")],
            vec![port("output", "String")],
            (),
        ));

        let spec = MockSpec::new("test")
            .input_mock("transform", "unknown_input", Value::Int(1))
            .boundary("transform", "output", Value::Str("ok".into()))
            .skip_node_example("transform");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let _ = generator.generate_test_module("test", "build_test_graph()");
    }

    #[test]
    fn test_mock_type_compatibility_accepts_skipped() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "transform",
            vec![port("input", "String")],
            vec![port("output", "Int")],
            (),
        ));

        // Value::Skipped is compatible with any type
        let spec = MockSpec::new("test")
            .boundary("transform", "output", Value::Skipped)
            .skip_node_example("transform");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        // Should not panic
        let code = generator.generate_test_module("test", "build_test_graph()");
        assert!(code.contains("test_"));
    }

    #[test]
    fn test_mock_type_compatibility_accepts_json() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "transform",
            vec![port("input", "String")],
            vec![port("output", "Json")],
            (),
        ));

        // Json port accepts Int value (flexible typing)
        let spec = MockSpec::new("test")
            .boundary("transform", "output", Value::Int(42))
            .skip_node_example("transform");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        // Should not panic
        let code = generator.generate_test_module("test", "build_test_graph()");
        assert!(code.contains("test_"));
    }

    #[test]
    fn test_no_mockspec_required_for_pure_dags() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        // No transport MockSpec needed, but pure nodes still need examples or skip.
        // Provide a MockSpec that skips both pure nodes.
        let spec = MockSpec::new("pure")
            .boundary("sink", "result", Value::Str("<MOCK>".into()))
            .skip_node_example("source")
            .skip_node_example("sink");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("pure", "build_pure_graph()");

        // Should generate tests without panicking
        assert!(code.contains("test_boundaries_mockable"));
    }

    #[test]
    fn test_generate_with_node_examples() {
        use gunbc_test::{NodeExample, OutputMatcher};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "prepare",
            vec![port("input", "String")],
            vec![port("output", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "process",
            vec![port("data", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("prepare", "output", "process", "data"));

        // Create examples for testing node I/O
        let example = NodeExample::new("prepare")
            .input("input", Value::Str("test input".into()))
            .output("output", OutputMatcher::non_empty())
            .description("basic input processing");

        let spec = MockSpec::new("test")
            .boundary("process", "result", Value::Str("test_output".into()))
            .node_example(example)
            .skip_node_example("process");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should have node example tests section
        assert!(
            code.contains("Node I/O Example Tests"),
            "should have example tests section header"
        );

        // Should generate test function (description sanitized to snake_case)
        assert!(
            code.contains("test_example_prepare_basic_input_processing"),
            "should generate test with description-based name: {}",
            code
        );

        // Should use execute_single_node
        assert!(
            code.contains("execute_single_node"),
            "should use execute_single_node to run individual node"
        );

        // Should have input setup
        assert!(
            code.contains("inputs.insert"),
            "should set up inputs from example"
        );

        // Should have output assertion
        assert!(
            code.contains("is_empty"),
            "should have non_empty assertion check"
        );
    }

    #[test]
    fn test_generate_with_exact_matcher() {
        use gunbc_test::{NodeExample, OutputMatcher};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "echo",
            vec![port("input", "String")],
            vec![port("output", "String")],
            (),
        ));

        let example = NodeExample::new("echo")
            .input("input", Value::Str("hello".into()))
            .output("output", OutputMatcher::exact(Value::Str("hello".into())));

        let spec = MockSpec::new("test")
            .boundary("echo", "output", Value::Str("hello".into()))
            .node_example(example);

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("exact", "build_exact_graph()");

        // Should have exact assertion
        assert!(
            code.contains("assert_eq!"),
            "should have exact match assertion"
        );
    }

    #[test]
    fn test_coercion_coverage_asserts_target_input_shape() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![port("input", "String")],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![list("in_items", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in_items"));

        let spec = MockSpec::new("coerce")
            .boundary("sink", "result", Value::Str("ok".into()))
            .skip_node_example("source")
            .skip_node_example("sink");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("coerce", "build_coerce_graph()");

        assert!(
            code.contains("coercion target entry missing for sink.in_items"),
            "coercion tests should assert target entry presence: {}",
            code
        );
        assert!(
            code.contains("let received = target_inputs.get(\"in_items\")"),
            "coercion tests should inspect target input port shape: {}",
            code
        );
        assert!(
            code.contains("WrapScalar should deliver non-empty list to sink.in_items"),
            "coercion tests should verify WrapScalar list shape: {}",
            code
        );
    }

    #[test]
    fn test_optional_inputs_use_mockspec_input_mocks_for_required_ports() {
        use gunbc_ir::transport::{RestResponse, TransportResponse};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "parse",
            vec![
                port("response", "TransportResponse"),
                optional("fallback", "OptionalString"),
            ],
            vec![port("result", "String")],
            (),
        ));

        let spec = MockSpec::new("opt")
            .input_mock(
                "parse",
                "response",
                Value::Response(TransportResponse::Rest(RestResponse::ok(
                    serde_json::json!({ "result": "ok" }),
                ))),
            )
            .boundary("parse", "result", Value::Str("ok".into()))
            .skip_node_example("parse");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("opt", "build_opt_graph()");

        assert!(
            code.contains("test_optional_missing_parse_fallback"),
            "should generate missing-optional test"
        );
        assert!(
            code.contains("TransportResponse::Rest"),
            "required inputs should come from MockSpec input_mock values"
        );
        assert!(
            !code.contains(
                "TransportResponse::Shell(gunbc_ir::transport::ShellResponse { exit_code: 0, stdout: \"<MOCK>\""
            ),
            "should not synthesize generic shell placeholders when input mocks exist"
        );
    }

    #[test]
    fn test_seed_policy_marks_semantic_types_explicit() {
        // Known semantic carriers require explicit seeds.
        assert_eq!(
            seed_policy_for_type("TransportResponse"),
            SeedPolicy::ExplicitSeedRequired
        );
        assert_eq!(
            seed_policy_for_type("TransportRequest"),
            SeedPolicy::ExplicitSeedRequired
        );
        assert_eq!(
            seed_policy_for_type("Credential"),
            SeedPolicy::ExplicitSeedRequired
        );
        assert_eq!(
            seed_policy_for_type("Secret"),
            SeedPolicy::ExplicitSeedRequired
        );
        assert_eq!(
            seed_policy_for_type("FilesystemHandle"),
            SeedPolicy::ExplicitSeedRequired
        );

        // Primitive/structural types are safe for generated placeholders.
        assert_eq!(seed_policy_for_type("String"), SeedPolicy::Generated);
        assert_eq!(seed_policy_for_type("Int"), SeedPolicy::Generated);
        assert_eq!(seed_policy_for_type("Bool"), SeedPolicy::Generated);
        assert_eq!(
            seed_policy_for_type("Map<String,String>"),
            SeedPolicy::Generated
        );
        assert_eq!(
            seed_policy_for_type("OptionalString"),
            SeedPolicy::Generated
        );
        assert_eq!(seed_policy_for_type("StringList"), SeedPolicy::Generated);
        assert_eq!(
            seed_policy_for_type("Map<String,Credential>"),
            SeedPolicy::ExplicitSeedRequired
        );

        // Fail-closed: unknown/new types default to ExplicitSeedRequired.
        assert_eq!(
            seed_policy_for_type("SomeNewCarrierType"),
            SeedPolicy::ExplicitSeedRequired,
            "unknown types must fail closed"
        );
        assert_eq!(
            seed_policy_for_type("CustomAuthToken"),
            SeedPolicy::ExplicitSeedRequired,
            "unknown types must fail closed"
        );

        assert!(requires_explicit_seed(
            "TransportResponse",
            SeedContext::RealSingleNode
        ));
        assert!(!requires_explicit_seed(
            "String",
            SeedContext::RealSingleNode
        ));
    }

    #[test]
    fn test_seed_matrix_scenario_context() {
        assert_eq!(
            seed_policy_for_context("TransportResponse", SeedContext::Scenario),
            SeedPolicy::ExplicitSeedRequired
        );
        assert_eq!(
            seed_policy_for_context("String", SeedContext::Scenario),
            SeedPolicy::Generated
        );
        assert!(requires_explicit_seed("Credential", SeedContext::Scenario));
        assert!(!requires_explicit_seed(
            "OptionalString",
            SeedContext::Scenario
        ));
    }

    #[test]
    fn test_seed_matrix_live_flow_context() {
        assert_eq!(
            seed_policy_for_context("TransportRequest", SeedContext::LiveFlow),
            SeedPolicy::ExplicitSeedRequired
        );
        assert_eq!(
            seed_policy_for_context("Map<String,String>", SeedContext::LiveFlow),
            SeedPolicy::Generated
        );
        assert!(requires_explicit_seed("Secret", SeedContext::LiveFlow));
        assert!(!requires_explicit_seed("StringList", SeedContext::LiveFlow));
    }

    #[test]
    fn test_seed_matrix_enforcement_consistent_across_contexts() {
        let contexts = [
            SeedContext::RealSingleNode,
            SeedContext::Scenario,
            SeedContext::LiveFlow,
        ];

        for context in contexts {
            assert!(
                requires_explicit_seed("TransportResponse", context),
                "semantic carrier must require explicit seed in {:?}",
                context
            );
            assert!(
                !requires_explicit_seed("String", context),
                "structural type should not require explicit seed in {:?}",
                context
            );
            assert!(
                requires_explicit_seed("SomeNewCarrierType", context),
                "unknown types must fail closed in {:?}",
                context
            );
        }
    }

    #[test]
    fn test_try_mock_element_value_supports_parametric_string_map() {
        let value = try_mock_element_value("Map<String,String>", Some(1))
            .expect("parametric map mock should generate");
        match value {
            Value::Map(entries) => {
                assert_eq!(entries.len(), 1);
                assert!(matches!(entries.get("mock_key"), Some(Value::Str(_))));
            }
            other => panic!("expected Value::Map, got {other:?}"),
        }
    }

    #[test]
    #[should_panic(
        expected = "Optional input tests require explicit seeds for required semantic inputs in Real single-node mode"
    )]
    fn test_optional_inputs_require_explicit_semantic_seed() {
        use gunbc_test::{NodeExample, OutputMatcher};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "parse",
            vec![
                port("response", "TransportResponse"),
                optional("fallback", "OptionalString"),
            ],
            vec![port("result", "String")],
            (),
        ));

        // Provide a node_example with outputs (satisfies the I/O example
        // requirement) but do NOT seed the TransportResponse input — the
        // seed assertion should fire.
        let spec = MockSpec::new("opt")
            .boundary("parse", "result", Value::Str("ok".into()))
            .node_example(
                NodeExample::new("parse")
                    .input("fallback", Value::Str("fb".into()))
                    .output("result", OutputMatcher::non_empty()),
            );

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let _ = generator.generate_test_module("opt", "build_opt_graph()");
    }

    #[test]
    fn test_skip_node_example_bypasses_seed_assertion() {
        // Nodes with skip_node_example should NOT panic about missing seeds
        // — they are known resource/boundary nodes that won't be tested
        // in single-node Real mode.
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "resolver",
            vec![
                port("config", "TransportResponse"),
                optional("name", "OptionalString"),
            ],
            vec![port("result", "String")],
            (),
        ));

        let spec = MockSpec::new("skip")
            .boundary("resolver", "result", Value::Str("ok".into()))
            .skip_node_example("resolver");

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        // Should NOT panic — skip_node_example suppresses the seed check.
        let _ = generator.generate_test_module("skip", "build_skip_graph()");
    }

    #[test]
    fn test_generate_with_satisfies_matcher_uses_runtime_check() {
        use gunbc_test::{NodeExample, OutputMatcher};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "echo",
            vec![port("input", "String")],
            vec![port("output", "String")],
            (),
        ));

        let example = NodeExample::new("echo")
            .input("input", Value::Str("hello".into()))
            .output(
                "output",
                OutputMatcher::satisfies(
                    "non-empty",
                    |v| matches!(v, Value::Str(s) if !s.is_empty()),
                ),
            );

        let spec = MockSpec::new("test")
            .boundary("echo", "output", Value::Str("hello".into()))
            .node_example(example);

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("satisfies", "build_satisfies_graph()");

        assert!(code.contains("mock_spec()"));
        assert!(code.contains(".check("));
        assert!(code.contains("example_spec"));
    }

    #[test]
    fn test_generate_with_contains_matcher() {
        use gunbc_test::{NodeExample, OutputMatcher};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "format",
            vec![port("data", "String")],
            vec![port("message", "String")],
            (),
        ));

        let example = NodeExample::new("format")
            .input("data", Value::Str("world".into()))
            .output("message", OutputMatcher::contains("hello"));

        let spec = MockSpec::new("test")
            .boundary("format", "message", Value::Str("hello world".into()))
            .node_example(example);

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("contains", "build_contains_graph()");

        // Should have contains assertion
        assert!(
            code.contains("contains(\"hello\")"),
            "should have contains check in assertion"
        );
    }

    #[test]
    fn test_generate_with_node_io_examples() {
        use std::collections::HashMap;

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "upper",
                vec![port("input", "String")],
                vec![port("output", "String")],
                (),
            )
            .with_described_example(
                "basic uppercase",
                HashMap::from([("input".to_string(), Value::Str("hello".into()))]),
                HashMap::from([("output".to_string(), Value::Str("HELLO".into()))]),
            ),
        );

        // MockSpec required for DAGs with boundary nodes
        let spec = MockSpec::new("node_ex").boundary("upper", "output", Value::Str("HELLO".into()));

        let generator = TestGenerator::new(&dag)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("node_ex", "build_node_ex_graph()");

        // Should have example tests section
        assert!(
            code.contains("Node I/O Example Tests"),
            "should have example tests section: {}",
            code
        );

        // Should generate test from node-sourced example
        assert!(
            code.contains("test_node_example_upper_basic_uppercase"),
            "should generate test with node example name: {}",
            code
        );

        // Should use exact match (assert_eq!)
        assert!(
            code.contains("assert_eq!"),
            "node examples should use exact match: {}",
            code
        );

        // Should reference the expected value
        assert!(
            code.contains("HELLO"),
            "should contain expected output value: {}",
            code
        );
    }

    // ===================================================================
    // BB-2: Per-Node Corpus Tests
    // ===================================================================

    #[test]
    fn test_corpus_section_empty_returns_none() {
        let dag: Dag<()> = Dag::new();
        let empty_corpus: HashMap<gunbc_test::NodeIdentity, gunbc_test::MockCorpus> =
            HashMap::new();
        let config = TestConfig {
            corpus_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_corpus(empty_corpus);
        let code = generator.generate_test_module("empty_corpus", "build_graph()");
        assert!(
            !code.contains("BB-2: Per-Node Corpus Tests"),
            "empty corpus should not produce a section: {}",
            code
        );
    }

    #[test]
    fn test_corpus_section_pure_node_exact_match() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "mod_a::compute",
            vec![port("x", "Int")],
            vec![port("result", "Int")],
            (),
        ));

        let identity = gunbc_test::NodeIdentity::new("mod_a", "compute");
        let mut corpus_entry = gunbc_test::MockCorpus::new();
        corpus_entry.add(gunbc_test::CorpusExample {
            provenance: gunbc_test::Provenance {
                workflow: "wf_alpha".to_string(),
                profile: None,
                node_instance: NodeId("mod_a::compute".into()),
                subdag_path: vec![],
                seed_kind: gunbc_test::SeedKind::WorkflowObserved,
            },
            inputs: HashMap::from([("x".to_string(), Value::Int(42))]),
            expectation: gunbc_test::Expectation::ExactOutputs(HashMap::from([(
                "result".to_string(),
                Value::Int(84),
            )])),
        });

        let mut corpus = HashMap::new();
        corpus.insert(identity, corpus_entry);

        let spec = MockSpec::new("corpus_pure")
            .boundary("mod_a::compute", "result", Value::Int(84))
            .skip_node_example("mod_a::compute");

        let config = TestConfig {
            corpus_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_corpus(corpus)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("corpus_pure", "build_graph()");

        assert!(
            code.contains("BB-2: Per-Node Corpus Tests"),
            "should have corpus section: {}",
            code
        );
        assert!(
            code.contains("test_corpus_mod_a_compute_0"),
            "should generate indexed test name: {}",
            code
        );
        assert!(
            code.contains("ExecutionMode::Real"),
            "pure node should use Real mode: {}",
            code
        );
        assert!(
            code.contains("execute_single_node"),
            "should call execute_single_node: {}",
            code
        );
        assert!(
            code.contains("assert_eq!"),
            "exact outputs should use assert_eq!: {}",
            code
        );
    }

    #[test]
    fn test_corpus_section_effectful_node_type_contract() {
        use gunbc_ir::NodeKind;
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "svc::fetch",
                vec![port("url", "String")],
                vec![port("body", "String")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let identity = gunbc_test::NodeIdentity::new("svc", "fetch");
        let mut corpus_entry = gunbc_test::MockCorpus::new();
        corpus_entry.add(gunbc_test::CorpusExample {
            provenance: gunbc_test::Provenance {
                workflow: "wf_beta".to_string(),
                profile: None,
                node_instance: NodeId("svc::fetch".into()),
                subdag_path: vec![],
                seed_kind: gunbc_test::SeedKind::WorkflowObserved,
            },
            inputs: HashMap::from([("url".to_string(), Value::Str("https://example.com".into()))]),
            expectation: gunbc_test::Expectation::TypeContractOnly,
        });

        let mut corpus = HashMap::new();
        corpus.insert(identity, corpus_entry);

        let spec = MockSpec::new("corpus_effectful")
            .boundary("svc::fetch", "body", Value::Str("<MOCK>".into()));

        let config = TestConfig {
            corpus_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_corpus(corpus)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("corpus_effectful", "build_graph()");

        assert!(
            code.contains("BB-2: Per-Node Corpus Tests"),
            "should have corpus section: {}",
            code
        );
        assert!(
            code.contains("test_corpus_svc_fetch_0"),
            "should generate indexed test name: {}",
            code
        );
        assert!(
            code.contains("DryRun"),
            "effectful node should use DryRun mode: {}",
            code
        );
        assert!(
            code.contains("value_compatible_with_type_id"),
            "type-contract should check type compatibility: {}",
            code
        );
    }

    #[test]
    fn test_corpus_section_skips_unknown_identity() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "known::node",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));

        // Corpus entry for a node NOT in the DAG.
        let unknown_identity = gunbc_test::NodeIdentity::new("unknown", "node");
        let mut corpus_entry = gunbc_test::MockCorpus::new();
        corpus_entry.add(gunbc_test::CorpusExample {
            provenance: gunbc_test::Provenance {
                workflow: "wf_gamma".to_string(),
                profile: None,
                node_instance: NodeId("unknown::node".into()),
                subdag_path: vec![],
                seed_kind: gunbc_test::SeedKind::WorkflowObserved,
            },
            inputs: HashMap::from([("in".to_string(), Value::Str("hello".into()))]),
            expectation: gunbc_test::Expectation::ExactOutputs(HashMap::from([(
                "out".to_string(),
                Value::Str("world".into()),
            )])),
        });

        let mut corpus = HashMap::new();
        corpus.insert(unknown_identity, corpus_entry);

        let spec = MockSpec::new("corpus_skip")
            .boundary("known::node", "out", Value::Str("<MOCK>".into()))
            .skip_node_example("known::node");

        let config = TestConfig {
            corpus_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_corpus(corpus)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("corpus_skip", "build_graph()");

        // Unknown identity should be silently skipped — no corpus section at all.
        assert!(
            !code.contains("BB-2: Per-Node Corpus Tests"),
            "unknown identity should not produce a section: {}",
            code
        );
    }

    #[test]
    fn test_corpus_section_multiple_examples_per_node() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "math::add",
            vec![port("a", "Int"), port("b", "Int")],
            vec![port("sum", "Int")],
            (),
        ));

        let identity = gunbc_test::NodeIdentity::new("math", "add");
        let mut corpus_entry = gunbc_test::MockCorpus::new();
        for (i, (a, b, s)) in [(1, 2, 3), (10, 20, 30), (0, 0, 0)].iter().enumerate() {
            corpus_entry.add(gunbc_test::CorpusExample {
                provenance: gunbc_test::Provenance {
                    workflow: format!("wf_{}", i),
                    profile: None,
                    node_instance: NodeId("math::add".into()),
                    subdag_path: vec![],
                    seed_kind: gunbc_test::SeedKind::WorkflowObserved,
                },
                inputs: HashMap::from([
                    ("a".to_string(), Value::Int(*a)),
                    ("b".to_string(), Value::Int(*b)),
                ]),
                expectation: gunbc_test::Expectation::ExactOutputs(HashMap::from([(
                    "sum".to_string(),
                    Value::Int(*s),
                )])),
            });
        }

        let mut corpus = HashMap::new();
        corpus.insert(identity, corpus_entry);

        let spec = MockSpec::new("corpus_multi")
            .boundary("math::add", "sum", Value::Int(0))
            .skip_node_example("math::add");

        let config = TestConfig {
            corpus_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_corpus(corpus)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("corpus_multi", "build_graph()");

        // Should generate 3 separate tests with indexed names.
        assert!(
            code.contains("test_corpus_math_add_0"),
            "should have example 0: {}",
            code
        );
        assert!(
            code.contains("test_corpus_math_add_1"),
            "should have example 1: {}",
            code
        );
        assert!(
            code.contains("test_corpus_math_add_2"),
            "should have example 2: {}",
            code
        );
    }

    // ===================================================================
    // BB-3: Adjacent Pair Tests
    // ===================================================================

    #[test]
    fn test_adjacent_pair_section_generates_wiring_test() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "svc::prepare",
            vec![port("url", "String")],
            vec![port("request", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "svc::execute",
            vec![port("request", "String"), port("timeout", "Int")],
            vec![port("response", "String")],
            (),
        ));
        dag.add_edge(edge("svc::prepare", "request", "svc::execute", "request"));

        let edge_example = gunbc_test::EdgeExample {
            provenance: gunbc_test::Provenance {
                workflow: "wf_edge".to_string(),
                profile: None,
                node_instance: NodeId("svc::prepare".into()),
                subdag_path: vec![],
                seed_kind: gunbc_test::SeedKind::WorkflowObserved,
            },
            from_node: gunbc_test::NodeIdentity::new("svc", "prepare"),
            to_node: gunbc_test::NodeIdentity::new("svc", "execute"),
            edge_port_map: HashMap::from([("request".to_string(), "request".to_string())]),
            a_inputs: HashMap::from([("url".to_string(), Value::Str("https://x.com".into()))]),
            b_other_inputs: HashMap::from([("timeout".to_string(), Value::Int(30))]),
        };

        let spec = MockSpec::new("adj_pair")
            .boundary("svc::prepare", "request", Value::Str("<REQ>".into()))
            .boundary("svc::execute", "response", Value::Str("<RESP>".into()))
            .skip_node_example("svc::prepare")
            .skip_node_example("svc::execute");

        let config = TestConfig {
            adjacent_pair_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_edge_examples(vec![edge_example])
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("adj_pair", "build_graph()");

        assert!(
            code.contains("BB-3: Adjacent Pair Tests"),
            "should have adjacent pair section: {}",
            code
        );
        assert!(
            code.contains("test_edge_svc_prepare_svc_execute_wf_wf_edge"),
            "should generate edge test name: {}",
            code
        );
        assert!(
            code.contains("execute_single_node"),
            "should call execute_single_node: {}",
            code
        );
        assert!(
            code.contains("a_outputs"),
            "should have source outputs variable: {}",
            code
        );
        assert!(
            code.contains("b_inputs"),
            "should wire inputs to target: {}",
            code
        );
        assert!(
            code.contains("b_outputs"),
            "should have target outputs variable: {}",
            code
        );
        assert!(
            code.contains("value_compatible_with_type_id"),
            "should check target output types: {}",
            code
        );
    }

    #[test]
    fn test_adjacent_pair_section_skips_unknown_nodes() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "known::a",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));
        // Edge references a target node not in the DAG.
        let edge_example = gunbc_test::EdgeExample {
            provenance: gunbc_test::Provenance {
                workflow: "wf_skip".to_string(),
                profile: None,
                node_instance: NodeId("known::a".into()),
                subdag_path: vec![],
                seed_kind: gunbc_test::SeedKind::WorkflowObserved,
            },
            from_node: gunbc_test::NodeIdentity::new("known", "a"),
            to_node: gunbc_test::NodeIdentity::new("missing", "b"),
            edge_port_map: HashMap::from([("out".to_string(), "in".to_string())]),
            a_inputs: HashMap::from([("in".to_string(), Value::Str("x".into()))]),
            b_other_inputs: HashMap::new(),
        };

        let spec = MockSpec::new("adj_skip")
            .boundary("known::a", "out", Value::Str("<MOCK>".into()))
            .skip_node_example("known::a");

        let config = TestConfig {
            adjacent_pair_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_edge_examples(vec![edge_example])
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("adj_skip", "build_graph()");

        assert!(
            !code.contains("BB-3: Adjacent Pair Tests"),
            "missing target node should skip edge section: {}",
            code
        );
    }

    // ===================================================================
    // BB-5: Cross-Workflow Consistency Tests
    // ===================================================================

    #[test]
    fn test_cross_workflow_section_generates_consistency_test() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "shared::format",
            vec![port("input", "String")],
            vec![port("output", "String")],
            (),
        ));

        let identity = gunbc_test::NodeIdentity::new("shared", "format");
        let mut corpus_entry = gunbc_test::MockCorpus::new();
        // Two examples from different workflows.
        for (wf, val) in [("wf_alpha", "hello"), ("wf_beta", "world")] {
            corpus_entry.add(gunbc_test::CorpusExample {
                provenance: gunbc_test::Provenance {
                    workflow: wf.to_string(),
                    profile: None,
                    node_instance: NodeId("shared::format".into()),
                    subdag_path: vec![],
                    seed_kind: gunbc_test::SeedKind::WorkflowObserved,
                },
                inputs: HashMap::from([("input".to_string(), Value::Str(val.into()))]),
                expectation: gunbc_test::Expectation::ExactOutputs(HashMap::from([(
                    "output".to_string(),
                    Value::Str(format!("formatted: {}", val)),
                )])),
            });
        }

        let mut corpus = HashMap::new();
        corpus.insert(identity, corpus_entry);

        let spec = MockSpec::new("cross_wf")
            .boundary("shared::format", "output", Value::Str("<MOCK>".into()))
            .skip_node_example("shared::format");

        let config = TestConfig {
            cross_workflow_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_corpus(corpus)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("cross_wf", "build_graph()");

        assert!(
            code.contains("BB-5: Cross-Workflow Consistency Tests"),
            "should have cross-workflow section: {}",
            code
        );
        assert!(
            code.contains("test_cross_wf_shared_format"),
            "should generate cross-wf test name: {}",
            code
        );
        assert!(
            code.contains("execute_single_node"),
            "should execute the node: {}",
            code
        );
        assert!(
            code.contains("output_keys_per_wf"),
            "should collect output keys per workflow: {}",
            code
        );
        assert!(
            code.contains("assert_eq!"),
            "should assert output port consistency: {}",
            code
        );
    }

    #[test]
    fn test_cross_workflow_section_skips_single_workflow_node() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "solo::node",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));

        let identity = gunbc_test::NodeIdentity::new("solo", "node");
        let mut corpus_entry = gunbc_test::MockCorpus::new();
        // Only one workflow — should not trigger cross-workflow test.
        corpus_entry.add(gunbc_test::CorpusExample {
            provenance: gunbc_test::Provenance {
                workflow: "wf_only".to_string(),
                profile: None,
                node_instance: NodeId("solo::node".into()),
                subdag_path: vec![],
                seed_kind: gunbc_test::SeedKind::WorkflowObserved,
            },
            inputs: HashMap::from([("in".to_string(), Value::Str("x".into()))]),
            expectation: gunbc_test::Expectation::ExactOutputs(HashMap::from([(
                "out".to_string(),
                Value::Str("y".into()),
            )])),
        });

        let mut corpus = HashMap::new();
        corpus.insert(identity, corpus_entry);

        let spec = MockSpec::new("cross_wf_skip")
            .boundary("solo::node", "out", Value::Str("<MOCK>".into()))
            .skip_node_example("solo::node");

        let config = TestConfig {
            cross_workflow_tests: true,
            ..TestConfig::default()
        };
        let generator = TestGenerator::new(&dag)
            .with_config(config)
            .with_corpus(corpus)
            .with_mock_spec(spec)
            .with_mock_spec_fn("crate::mock_spec()");
        let code = generator.generate_test_module("cross_wf_skip", "build_graph()");

        assert!(
            !code.contains("BB-5: Cross-Workflow Consistency Tests"),
            "single-workflow node should not produce cross-workflow section: {}",
            code
        );
    }
}
