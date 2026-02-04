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
use crate::testgen::obligation::{collect_obligations, DischargeStatus, Obligation, ObligationSet};
use crate::testgen::render::TestRenderer;
use crate::testgen::render_rust::RustRenderer;
use crate::testgen::test_ir::{Assert, Expr};
use gunbc_ir::language::traits::comment::{generated_header, RUST_COMMENTS};
use gunbc_ir::language::NamingCase;
use gunbc_ir::types::CardinalityCase;
use gunbc_ir::{Dag, Value, ValueExpr};
use gunbc_test::{MockSpec, OutputMatcher};
use std::hash::{Hash, Hasher};

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
    /// Generate flow verification tests (DryRun full DAG, verify terminal outputs)
    pub flow_tests: bool,
    /// Generate per-node I/O example tests (from MockSpec.node_examples)
    pub example_tests: bool,
    /// Test module visibility
    pub visibility: String,
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
            example_tests: true,
            visibility: "pub".to_string(),
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
}

impl<'a, T> TestGenerator<'a, T> {
    /// Create a new test generator for a DAG.
    pub fn new(dag: &'a Dag<T>) -> Self {
        Self {
            dag,
            config: TestConfig::default(),
            mock_spec: None,
            mock_spec_fn: None,
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
                    uncovered.iter().map(|id| format!("  - {}", id)).collect::<Vec<_>>().join("\n"),
                    module_name,
                    " ", " ", " ", " ", " ",
                    module_name,
                    " ",
                );
            }
        }

        let obligations = collect_obligations(self.dag, None, None);

        // Generate the test body first so we can hash it for staleness detection.
        let body = self.generate_test_body(&analysis, &obligations, graph_builder_fn);

        // Compute content hash of the body for staleness detection.
        let content_hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            body.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };

        // Build the header with the content hash embedded.
        let mut code = String::new();
        let prefix = RUST_COMMENTS.line_prefix;
        code.push_str(&format!(
            "{} Generated tests for {} DAG.\n",
            prefix, module_name
        ));
        code.push_str(&format!("{}\n", prefix));
        code.push_str(&generated_header(
            &gunbc_ir::cargo::name("testgen"),
            "make testgen",
            prefix,
        ));

        // Obligation summary as header comment
        let stats = obligations.stats();
        code.push_str(&format!(
            "{} Obligations: {}\n",
            prefix, stats
        ));
        code.push_str(&format!(
            "{} Proven by construction: acyclicity, type compatibility, cardinality satisfaction.\n",
            prefix
        ));
        code.push_str(&format!(
            "{} Content-Hash: {}\n",
            prefix, content_hash
        ));
        code.push_str("\n\n");

        code.push_str(&body);
        code
    }

    /// Generate the test body (everything after the header).
    ///
    /// Separated from `generate_test_module` so we can hash the body
    /// before emitting the header (which contains the hash).
    fn generate_test_body(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> String {
        let mut code = String::new();

        // Imports
        code.push_str("use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};\n");
        code.push_str("use gunbc_ir::{detect_boundaries, Cardinality, Value};\n");

        // Import MockSpec if we have a mock_spec_fn (need it for helper function)
        if self.mock_spec_fn.is_some() {
            code.push_str(
                "use gunbc_test::{assert_boundary_mockable, assert_types_compatible, MockSpec};\n",
            );
        } else {
            code.push_str(
                "use gunbc_test::{assert_boundary_mockable, assert_types_compatible, default_mocks};\n",
            );
        }

        if self.config.chain_tests && self.mock_spec.is_some() {
            code.push_str("use gunbc_test::{validate_chain, InputConstraint};\n");
        }
        if self.config.resource_tests
            && self
                .mock_spec
                .as_ref()
                .is_some_and(|s| !s.resource_mocks.resources.is_empty())
        {
            code.push_str("use gunbc_test::{ResourceAcquireResult, ResourceSimulation};\n");
        }
        code.push('\n');

        // Generate mock_spec() helper function if we have a mock_spec_fn path
        if let Some(mock_spec_fn) = &self.mock_spec_fn {
            code.push_str("/// Get the MockSpec for this DAG.\n");
            code.push_str("fn mock_spec() -> MockSpec {\n");
            code.push_str(&format!("    {}\n", mock_spec_fn));
            code.push_str("}\n\n");
        }

        // ===================================================================
        // Invalid obligations — structural errors surfaced as failing tests
        // ===================================================================
        let invalids = obligations.invalids();
        if !invalids.is_empty() {
            code.push_str("// =========================================================================\n");
            code.push_str("// INVALID OBLIGATIONS — structural errors detected during analysis\n");
            code.push_str("//\n");
            code.push_str("// These are NOT runtime tests. They surface provably wrong graph structure.\n");
            code.push_str("// Fix the underlying issue rather than deleting these tests.\n");
            code.push_str("// =========================================================================\n\n");

            for (i, obligation) in invalids.iter().enumerate() {
                let reason = match &obligation.status {
                    DischargeStatus::Invalid { reason } => reason.as_str(),
                    _ => "unknown",
                };
                code.push_str(&format!(
                    "/// INVALID: {}\n",
                    obligation.reason
                ));
                code.push_str("#[test]\n");
                code.push_str(&format!(
                    "fn test_invalid_obligation_{}() {{\n",
                    i
                ));
                code.push_str(&format!(
                    "    panic!(\"Structural error: {}\");\n",
                    reason.replace('\"', "\\\"")
                ));
                code.push_str("}\n\n");
            }
        }

        // ===================================================================
        // Bucket A: Execution Semantics
        // ===================================================================
        if self.config.execution_tests {
            code.push_str(&self.generate_execution_tests(
                analysis,
                obligations,
                graph_builder_fn,
            ));
        }

        // ===================================================================
        // Bucket B: Contract Obligations (only for Unknown entailments)
        // ===================================================================
        if self.config.contract_tests {
            code.push_str(&self.generate_contract_tests(analysis, obligations, graph_builder_fn));
        }

        // ===================================================================
        // Bucket C: Scenario Coverage
        // ===================================================================
        if self.config.scenario_tests {
            code.push_str(&self.generate_scenario_tests(
                analysis,
                obligations,
                graph_builder_fn,
            ));
        }

        // ===================================================================
        // Bucket D: Resource Hygiene + Simulation
        // ===================================================================
        if self.config.resource_tests {
            code.push_str(&self.generate_resource_tests(analysis, obligations));
        }

        // ===================================================================
        // Legacy: individual boundary tests, chain validation, flow tests
        // ===================================================================

        // NOTE: Type and cardinality compatibility are verified at compile time
        // by validate_dag(), so we don't generate redundant tests for those.
        // The compiler proves: types match, cardinalities satisfy, no cycles.

        if self.config.boundary_tests {
            code.push_str(&self.generate_boundary_tests(analysis, graph_builder_fn));
        }

        if self.config.chain_tests {
            code.push_str(&self.generate_chain_tests(analysis));
        }

        if self.config.flow_tests {
            code.push_str(&self.generate_flow_tests(analysis, graph_builder_fn));
        }

        // ===================================================================
        // Node I/O Example Tests (from MockSpec.node_examples)
        // ===================================================================
        if self.config.example_tests {
            code.push_str(&self.generate_node_example_tests(graph_builder_fn));
        }

        code
    }

    // =======================================================================
    // Bucket A: Execution Semantics
    // =======================================================================

    fn generate_execution_tests(
        &self,
        _analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> String {
        let bucket = obligations.bucket_a();
        if bucket.is_empty() {
            return String::new();
        }

        let mut code = String::new();
        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Bucket A: Execution Semantics\n");
        code.push_str("// Proves: executor/boundary model correctness (runtime-only)\n");
        code.push_str(
            "// ============================================================================\n\n",
        );

        // A.1: DryRun completion
        if bucket
            .iter()
            .any(|o| matches!(o.kind, Obligation::DryRunCompletion))
        {
            let mocks_expr = if self.mock_spec_fn.is_some() {
                "mock_spec().to_boundary_mocks()"
            } else {
                "default_mocks()"
            };
            code.push_str("/// DryRun execution completes without crash.\n");
            code.push_str("///\n");
            code.push_str("/// This is the minimal smoke test: build the DAG, run it in DryRun\n");
            code.push_str("/// with default mocks, and verify it completes successfully.\n");
            code.push_str("#[test]\n");
            code.push_str("fn test_dryrun_completion() {\n");
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
            code.push_str(&format!(
                "    let log = execute_with_mode(&dag, ExecutionMode::DryRun({}))\n",
                mocks_expr
            ));
            code.push_str("        .expect(\"DryRun execution should complete without crash\");\n");
            code.push_str("    assert!(!log.entries.is_empty(), \"execution should produce log entries\");\n");
            code.push_str("}\n\n");
        }

        // A.2: Transport interception
        let transport_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::TransportInterceptable { .. }))
            .collect();

        if !transport_obligations.is_empty() {
            let mocks_expr = if self.mock_spec_fn.is_some() {
                "mock_spec().to_boundary_mocks()"
            } else {
                "default_mocks()"
            };
            code.push_str("/// All transport executors are intercepted in DryRun.\n");
            code.push_str("///\n");
            code.push_str(
                "/// Proves: every transport executor is interceptable; DryRun won't\n",
            );
            code.push_str("/// accidentally perform real I/O.\n");
            code.push_str("#[test]\n");
            code.push_str("fn test_transport_interception() {\n");
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
            code.push_str(&format!(
                "    let result = assert_boundary_mockable(&dag, {});\n",
                mocks_expr
            ));
            code.push_str(
                "    assert!(result.is_ok(), \"All transports should be interceptable: {:?}\", result.error);\n",
            );

            for obligation in &transport_obligations {
                if let Obligation::TransportInterceptable { node_id } = &obligation.kind {
                    code.push_str(&format!(
                        "    assert!(result.boundary_nodes.iter().any(|n| n == \"{}\"),\n",
                        node_id.0
                    ));
                    code.push_str(&format!(
                        "        \"transport executor '{}' should be in intercepted list\");\n",
                        node_id.0
                    ));
                }
            }

            code.push_str("}\n\n");
        }

        // A.3: Determinism — emit as a structural comment + future test placeholder
        let determinism_count = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::PureNodeDeterminism { .. }))
            .count();

        if determinism_count > 0 {
            code.push_str(&format!(
                "// Determinism obligations: {} pure nodes.\n",
                determinism_count
            ));
            code.push_str("// To enable per-node determinism tests, use `execute_single_node`\n");
            code.push_str("// from gunbc_exec with baseline-derived inputs (Tier 1 infra).\n");
            for obligation in &bucket {
                if let Obligation::PureNodeDeterminism { node_id } = &obligation.kind {
                    code.push_str(&format!(
                        "// - '{}': same inputs → same outputs\n",
                        node_id.0
                    ));
                }
            }
            code.push('\n');
        }

        code
    }

    // =======================================================================
    // Bucket B: Contract Obligations
    // =======================================================================

    fn generate_contract_tests(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> String {
        let bucket = obligations.bucket_b();
        if bucket.is_empty() {
            return String::new();
        }

        let mut code = String::new();
        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Bucket B: Contract Obligations\n");
        code.push_str("// Tests for semantic compatibility when proof engine returns Unknown.\n");
        code.push_str(
            "// ============================================================================\n\n",
        );

        // B.1: Edge predicate entailment (Unknown only)
        let entailment_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::EdgePredicateEntailment { .. }))
            .collect();

        if !entailment_obligations.is_empty() {
            code.push_str(&format!(
                "// {} edge predicate entailment obligations (Unknown).\n",
                entailment_obligations.len()
            ));
            code.push_str(
                "// Full entailment tests require contract tower witnesses (Tier 3 infra).\n",
            );
            code.push_str("// For now, these are documented as obligations:\n");
            for obligation in &entailment_obligations {
                if let Obligation::EdgePredicateEntailment {
                    from_node,
                    from_port,
                    to_node,
                    to_port,
                    ..
                } = &obligation.kind
                {
                    code.push_str(&format!(
                        "// - {}.{} → {}.{}: {}\n",
                        from_node.0, from_port.0, to_node.0, to_port.0, obligation.reason
                    ));
                }
            }
            code.push('\n');
        }

        // B.2: Node contract compliance
        let compliance_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::NodeContractCompliance { .. }))
            .collect();

        if !compliance_obligations.is_empty() {
            code.push_str(&format!(
                "// {} node contract compliance obligations.\n",
                compliance_obligations.len()
            ));
            code.push_str(
                "// Per-node compliance tests use `execute_single_node` (Tier 1 infra).\n",
            );
            for obligation in &compliance_obligations {
                if let Obligation::NodeContractCompliance { node_id } = &obligation.kind {
                    code.push_str(&format!(
                        "// - '{}': valid inputs → valid outputs\n",
                        node_id.0
                    ));
                }
            }
            code.push('\n');
        }

        // B.3: Cardinality boundary coverage
        code.push_str(&self.generate_cardinality_coverage_tests(analysis, obligations, graph_builder_fn));

        code
    }

    /// Generate cardinality boundary coverage tests.
    ///
    /// For each boundary port with non-trivial cardinality (more than one test case),
    /// generates a test per cardinality case (Empty, One, Many) that exercises the
    /// DAG with mock values at that cardinality boundary.
    fn generate_cardinality_coverage_tests(
        &self,
        _analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> String {
        let card_obligations: Vec<_> = obligations
            .cardinality_obligations();

        if card_obligations.is_empty() {
            return String::new();
        }

        let mut code = String::new();
        code.push_str("// --- B.3: Cardinality Boundary Coverage ---\n");
        code.push_str("//\n");
        code.push_str("// These tests exercise boundary ports at different cardinality levels\n");
        code.push_str("// (empty, one, many) to verify runtime behavior across the interval.\n\n");

        for obligation in &card_obligations {
            if let Obligation::CardinalityCoverage {
                node_id,
                port_name,
                cardinality,
                cases,
            } = &obligation.kind
            {
                // Find the port's type
                let type_id = self
                    .dag
                    .nodes
                    .iter()
                    .find(|n| n.id == *node_id)
                    .and_then(|n| n.outputs.iter().find(|p| p.name == *port_name))
                    .map(|p| p.type_id.0.as_str())
                    .unwrap_or("String");

                for case in cases {
                    let case_name = match case {
                        CardinalityCase::Empty => "empty",
                        CardinalityCase::One => "one",
                        CardinalityCase::Many => "many",
                    };

                    let test_name = format!(
                        "test_cardinality_{}_{}_{}",
                        NamingCase::SnakeCase.apply(&node_id.0),
                        NamingCase::SnakeCase.apply(&port_name.0),
                        case_name
                    );

                    let mock_value = cardinality_case_mock_value(*case, type_id);

                    code.push_str(&format!(
                        "/// Cardinality coverage: {}.{} with {} element(s) (cardinality: {}).\n",
                        node_id.0, port_name.0, case_name, cardinality
                    ));
                    code.push_str("///\n");
                    code.push_str(&format!(
                        "/// Proves: DAG handles {} case for boundary port {}.{}.\n",
                        case_name, node_id.0, port_name.0
                    ));
                    code.push_str("#[test]\n");
                    code.push_str(&format!("fn {}() {{\n", test_name));
                    code.push_str(&format!("    let dag = {};\n", graph_builder_fn));

                    let mocks_init = if self.mock_spec_fn.is_some() {
                        "mock_spec().to_boundary_mocks()"
                    } else {
                        "default_mocks()"
                    };
                    code.push_str(&format!("    let mut mocks = {};\n", mocks_init));
                    code.push_str(&format!(
                        "    mocks.set_value(\"{}\", \"{}\", {});\n",
                        node_id.0, port_name.0, mock_value
                    ));
                    code.push_str(
                        "    let _log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))\n",
                    );
                    code.push_str(&format!(
                        "        .expect(\"cardinality {} case should not crash\");\n",
                        case_name
                    ));
                    code.push_str("}\n\n");
                }
            }
        }

        code
    }

    // =======================================================================
    // Bucket C: Scenario Coverage
    // =======================================================================

    fn generate_scenario_tests(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> String {
        let bucket = obligations.bucket_c();
        if bucket.is_empty() {
            return String::new();
        }

        let mut code = String::new();
        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Bucket C: Scenario Coverage\n");
        code.push_str(
            "// N+1 scenarios: one success + one per-transport failure + guard toggles.\n",
        );
        code.push_str(
            "// ============================================================================\n\n",
        );

        // C.1: All transports succeed
        if bucket
            .iter()
            .any(|o| matches!(o.kind, Obligation::AllTransportsSucceed))
        {
            code.push_str("/// Happy path: all transports succeed.\n");
            code.push_str("///\n");
            code.push_str(
                "/// Proves: workflow reaches terminal outputs with all transports mocked as success.\n",
            );
            let mocks_expr = if self.mock_spec_fn.is_some() {
                "mock_spec().to_boundary_mocks()"
            } else {
                "default_mocks()"
            };
            code.push_str("#[test]\n");
            code.push_str("fn test_scenario_all_succeed() {\n");
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
            code.push_str(&format!(
                "    let log = execute_with_mode(&dag, ExecutionMode::DryRun({}))\n",
                mocks_expr
            ));
            code.push_str("        .expect(\"all-succeed scenario should complete\");\n");

            // Verify all transport executors were intercepted
            for transport in &analysis.transport_executors {
                code.push_str(&format!(
                    "    let entry = log.get(\"{}\").expect(\"'{}' should be in log\");\n",
                    transport, transport
                ));
                code.push_str(&format!(
                    "    assert!(entry.was_intercepted, \"'{}' should be intercepted in DryRun\");\n",
                    transport
                ));
            }

            code.push_str("}\n\n");
        }

        // C.2: Single transport failure scenarios
        let failure_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::SingleTransportFailure { .. }))
            .collect();

        if !failure_obligations.is_empty() {
            code.push_str(&format!(
                "// {} single-failure scenarios (one per transport executor).\n",
                failure_obligations.len()
            ));
            code.push_str(
                "// Full failure scenarios require per-transport failure mocks (Tier 0 infra).\n",
            );

            for obligation in &failure_obligations {
                if let Obligation::SingleTransportFailure { node_id } = &obligation.kind {
                    let test_name = format!(
                        "test_scenario_{}_fails",
                        NamingCase::SnakeCase.apply(&node_id.0)
                    );
                    code.push_str(&format!(
                        "/// Single failure: '{}' transport fails.\n",
                        node_id.0
                    ));
                    code.push_str("///\n");
                    code.push_str(
                        "/// Proves: failure propagation semantics are consistent.\n",
                    );
                    code.push_str("#[test]\n");
                    code.push_str(&format!("fn {}() {{\n", test_name));
                    code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
                    code.push_str(
                        "    let mut mocks = BoundaryMocks::with_default(Value::Str(\"<FAIL>\".to_string()));\n",
                    );
                    code.push_str(&format!(
                        "    // Inject failure at '{}'\n",
                        node_id.0
                    ));
                    code.push_str(&format!(
                        "    mocks.set_value(\"{}\", \"response\",\n",
                        node_id.0
                    ));
                    code.push_str(
                        "        Value::Str(\"<TRANSPORT_FAILURE>\".to_string()));\n",
                    );
                    code.push_str(
                        "    // Execution may succeed or fail depending on graph semantics;\n",
                    );
                    code.push_str(
                        "    // the key property is that it doesn't crash/hang.\n",
                    );
                    code.push_str(
                        "    let _result = execute_with_mode(&dag, ExecutionMode::DryRun(mocks));\n",
                    );
                    code.push_str("}\n\n");
                }
            }
        }

        // C.3: Skip-path propagation — real tests
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

                // Find the trigger node's output ports
                let output_ports: Vec<_> = self
                    .dag
                    .nodes
                    .iter()
                    .find(|n| n.id.0 == trigger_node.0)
                    .map(|n| {
                        n.outputs
                            .iter()
                            .map(|p| (p.name.0.clone(), p.type_id.0.clone()))
                            .collect()
                    })
                    .unwrap_or_default();

                // Find downstream node IDs (nodes connected by edges from trigger)
                let downstream: Vec<_> = self
                    .dag
                    .edges
                    .iter()
                    .filter(|e| e.from_node.0 == trigger_node.0)
                    .map(|e| e.to_node.0.clone())
                    .collect();

                code.push_str(&format!(
                    "/// Skip propagation: '{}' returns Skipped → downstream handles it.\n",
                    trigger_node.0
                ));
                code.push_str("///\n");
                code.push_str(
                    "/// Proves: when a transport's output is Skipped, downstream nodes\n",
                );
                code.push_str(
                    "/// either skip themselves (guarded) or process the Skipped value\n",
                );
                code.push_str("/// without crashing.\n");
                code.push_str("#[test]\n");
                code.push_str(&format!("fn {}() {{\n", test_name));
                code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
                if self.mock_spec_fn.is_some() {
                    code.push_str("    let mut mocks = mock_spec().to_boundary_mocks();\n");
                } else {
                    code.push_str("    let mut mocks = default_mocks();\n");
                }

                // Mock all output ports of the trigger node as Skipped
                for (port_name, _type_id) in &output_ports {
                    code.push_str(&format!(
                        "    mocks.set_value(\"{}\", \"{}\", Value::Skipped);\n",
                        trigger_node.0, port_name
                    ));
                }

                code.push_str(
                    "    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))\n",
                );
                code.push_str(
                    "        .expect(\"skip propagation should not crash or hang\");\n",
                );

                // Verify downstream nodes exist in the log (they ran, even if skipped)
                for ds_node in &downstream {
                    code.push_str(&format!(
                        "    assert!(log.get(\"{}\").is_some(), \"downstream '{}' should still appear in log\");\n",
                        ds_node, ds_node
                    ));
                }

                code.push_str("}\n\n");
            }
        }

        // C.4: Guard/skip branch coverage — real tests for Bool guards,
        //       structured comments for other guard types.
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
                // Find the guarded port's type
                let guard_type = self
                    .dag
                    .nodes
                    .iter()
                    .find(|n| n.id.0 == node_id.0)
                    .and_then(|n| n.inputs.iter().find(|p| p.name.0 == guard_port.0))
                    .map(|p| p.type_id.0.as_str())
                    .unwrap_or("Unknown");

                // Find the upstream edge that feeds this guarded port
                let upstream_edge = self
                    .dag
                    .edges
                    .iter()
                    .find(|e| e.to_node.0 == node_id.0 && e.to_port.0 == guard_port.0);

                // Check if upstream is a transport executor (mockable in DryRun)
                let upstream_is_mockable = upstream_edge
                    .map(|e| {
                        analysis
                            .transport_executors
                            .contains(&e.from_node.0)
                            || analysis.tool_env_nodes.contains(&e.from_node.0)
                    })
                    .unwrap_or(false);

                if guard_type == "Bool" {
                    // Bool guards: generate real two-scenario test
                    let test_name = format!(
                        "test_guard_{}_{}_branch_coverage",
                        NamingCase::SnakeCase.apply(&node_id.0),
                        NamingCase::SnakeCase.apply(&guard_port.0)
                    );

                    code.push_str(&format!(
                        "/// Guard branch coverage: '{}'.{} (Bool guard).\n",
                        node_id.0, guard_port.0
                    ));
                    code.push_str("///\n");
                    code.push_str(
                        "/// Proves: one of {true, false} causes the node to execute,\n",
                    );
                    code.push_str(
                        "/// the other causes it to skip (all outputs = Value::Skipped).\n",
                    );
                    code.push_str("#[test]\n");
                    code.push_str(&format!("fn {}() {{\n", test_name));
                    code.push_str(&format!("    let dag = {};\n", graph_builder_fn));

                    if let Some(edge) = upstream_edge {
                        if upstream_is_mockable {
                            // Upstream is a boundary node — we can mock its output
                            code.push_str(
                                "    // Guard value flows from a mockable boundary node.\n",
                            );
                            let mocks_init = if self.mock_spec_fn.is_some() {
                                "mock_spec().to_boundary_mocks()"
                            } else {
                                "default_mocks()"
                            };
                            code.push_str("    // Test with true:\n");
                            code.push_str(&format!("    let mut mocks_true = {};\n", mocks_init));
                            code.push_str(&format!(
                                "    mocks_true.set_value(\"{}\", \"{}\", Value::Bool(true));\n",
                                edge.from_node.0, edge.from_port.0
                            ));
                            code.push_str(
                                "    let log_true = execute_with_mode(&dag, ExecutionMode::DryRun(mocks_true))\n",
                            );
                            code.push_str(
                                "        .expect(\"guard=true scenario should not crash\");\n",
                            );
                            code.push_str(&format!(
                                "    let skipped_true = log_true.get(\"{}\")\n",
                                node_id.0
                            ));
                            code.push_str(
                                "        .map(|e| e.outputs.values().all(|v| v.is_skipped()))\n",
                            );
                            code.push_str("        .unwrap_or(true);\n\n");

                            code.push_str("    // Test with false:\n");
                            code.push_str(&format!("    let mut mocks_false = {};\n", mocks_init));
                            code.push_str(&format!(
                                "    mocks_false.set_value(\"{}\", \"{}\", Value::Bool(false));\n",
                                edge.from_node.0, edge.from_port.0
                            ));
                            code.push_str(
                                "    let log_false = execute_with_mode(&dag, ExecutionMode::DryRun(mocks_false))\n",
                            );
                            code.push_str(
                                "        .expect(\"guard=false scenario should not crash\");\n",
                            );
                            code.push_str(&format!(
                                "    let skipped_false = log_false.get(\"{}\")\n",
                                node_id.0
                            ));
                            code.push_str(
                                "        .map(|e| e.outputs.values().all(|v| v.is_skipped()))\n",
                            );
                            code.push_str("        .unwrap_or(true);\n\n");

                            code.push_str(
                                "    // Exactly one path should execute and the other should skip.\n",
                            );
                            code.push_str(&format!(
                                "    assert_ne!(skipped_true, skipped_false,\n        \"guard on '{}'.{} should cause one branch to execute and the other to skip\");\n",
                                node_id.0, guard_port.0
                            ));
                        } else {
                            // Upstream is a pure node — can't directly mock, use structural assertion
                            code.push_str(&format!(
                                "    // Guard value flows from pure node '{}' — not directly mockable.\n",
                                edge.from_node.0
                            ));
                            code.push_str(
                                "    // Structural check: guard port is connected and the node has outputs.\n",
                            );
                            code.push_str(&format!(
                                "    let node = dag.get_node(&\"{}\".into()).expect(\"node should exist\");\n",
                                node_id.0
                            ));
                            code.push_str(&format!(
                                "    let port = node.inputs.iter().find(|p| p.name.0 == \"{}\").expect(\"port should exist\");\n",
                                guard_port.0
                            ));
                            code.push_str(
                                "    assert!(port.has_guard(), \"port should have a guard\");\n",
                            );
                        }
                    } else {
                        // No upstream edge — guard port is disconnected
                        code.push_str(&format!(
                            "    // WARNING: guard port '{}'.{} has no incoming edge.\n",
                            node_id.0, guard_port.0
                        ));
                        code.push_str(
                            "    // The node will always skip (missing input → skip).\n",
                        );
                        let mocks_expr = if self.mock_spec_fn.is_some() {
                            "mock_spec().to_boundary_mocks()"
                        } else {
                            "default_mocks()"
                        };
                        code.push_str(&format!(
                            "    let log = execute_with_mode(&dag, ExecutionMode::DryRun({}))\n",
                            mocks_expr
                        ));
                        code.push_str(
                            "        .expect(\"execution should not crash\");\n",
                        );
                        code.push_str(&format!(
                            "    if let Some(entry) = log.get(\"{}\") {{\n",
                            node_id.0
                        ));
                        code.push_str(
                            "        assert!(entry.outputs.values().all(|v| v.is_skipped()),\n",
                        );
                        code.push_str(
                            "            \"disconnected guard → node should always skip\");\n",
                        );
                        code.push_str("    }\n");
                    }

                    code.push_str("}\n\n");
                } else {
                    // Non-Bool guard: emit structured comment with details
                    code.push_str(&format!(
                        "// Guard branch: '{}'.{} (type: {})\n",
                        node_id.0, guard_port.0, guard_type
                    ));
                    if let Some(edge) = upstream_edge {
                        code.push_str(&format!(
                            "//   fed by: {}.{}\n",
                            edge.from_node.0, edge.from_port.0
                        ));
                        if upstream_is_mockable {
                            code.push_str(
                                "//   upstream is mockable — full branch test possible with Tier 1 infra\n",
                            );
                        } else {
                            code.push_str(
                                "//   upstream is pure — needs per-node isolation (Tier 1) for full test\n",
                            );
                        }
                    } else {
                        code.push_str("//   WARNING: no incoming edge (disconnected guard)\n");
                    }
                    code.push('\n');
                }
            }
        }

        code
    }

    // =======================================================================
    // Bucket D: Resource Hygiene + Simulation
    // =======================================================================

    fn generate_resource_tests(
        &self,
        _analysis: &DagAnalysis,
        obligations: &ObligationSet,
    ) -> String {
        let bucket = obligations.bucket_d();

        // Also check for MockSpec resource simulation
        let has_mockspec_resources = self
            .mock_spec
            .as_ref()
            .is_some_and(|s| !s.resource_mocks.resources.is_empty());

        if bucket.is_empty() && !has_mockspec_resources {
            return String::new();
        }

        let mut code = String::new();
        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Bucket D: Resource Hygiene\n");
        code.push_str("// Structural resource/tool wiring correctness + simulation tests.\n");
        code.push_str(
            "// ============================================================================\n\n",
        );

        // D.1: Resource connectivity issues (undischarged = disconnected resource ports)
        let connectivity_issues: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::ResourceInputConnected { .. }))
            .collect();

        if !connectivity_issues.is_empty() {
            code.push_str("// Resource connectivity issues (disconnected resource ports):\n");
            for obligation in &connectivity_issues {
                if let Obligation::ResourceInputConnected {
                    node_id,
                    port_name,
                } = &obligation.kind
                {
                    code.push_str(&format!(
                        "// WARNING: {}.{} — {}\n",
                        node_id.0, port_name.0, obligation.reason
                    ));
                }
            }
            code.push('\n');
        }

        // D.2: Resource orphans (acquired but never consumed)
        let orphan_issues: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::ResourceOrphan { .. }))
            .collect();

        if !orphan_issues.is_empty() {
            code.push_str("// Resource orphans (acquired but never consumed):\n");
            for obligation in &orphan_issues {
                if let Obligation::ResourceOrphan {
                    node_id,
                    port_name,
                } = &obligation.kind
                {
                    code.push_str(&format!(
                        "// WARNING: {}.{} — {}\n",
                        node_id.0, port_name.0, obligation.reason
                    ));
                }
            }
            code.push('\n');
        }

        // D.3: Resource conflict absence
        let conflict_obligations: Vec<_> = bucket
            .iter()
            .filter(|o| matches!(o.kind, Obligation::ResourceConflictAbsence { .. }))
            .collect();

        if !conflict_obligations.is_empty() {
            for obligation in &conflict_obligations {
                if let Obligation::ResourceConflictAbsence { conflicts } = &obligation.kind {
                    if !conflicts.is_empty() {
                        code.push_str("// RESOURCE CONFLICTS DETECTED:\n");
                        for conflict in conflicts {
                            code.push_str(&format!("// - {}\n", conflict));
                        }
                        code.push('\n');
                    }
                }
            }
        }

        // D.4: MockSpec-based resource simulation tests
        if has_mockspec_resources {
            code.push_str(&self.generate_resource_simulation_tests());
        }

        code
    }

    /// Generate MockSpec-based resource simulation tests.
    fn generate_resource_simulation_tests(&self) -> String {
        let mut code = String::new();

        let Some(spec) = &self.mock_spec else {
            return code;
        };

        if spec.resource_mocks.resources.is_empty() {
            return code;
        }

        code.push_str("// Resource simulation tests (MockSpec-based)\n\n");

        for resource in &spec.resource_mocks.resources {
            let test_name = format!(
                "test_resource_{}_acquire",
                sanitize_resource_id(&resource.resource_id)
            );

            let resource_type = match &resource.resource_type {
                gunbc_test::ResourceType::Lock => "Lock",
                gunbc_test::ResourceType::Lease { duration_ms } => {
                    code.push_str(&format!(
                        "/// Test resource '{}' lease behavior ({}ms).\n",
                        resource.resource_id, duration_ms
                    ));
                    "Lease"
                }
                gunbc_test::ResourceType::SharedLock { max_holders } => {
                    code.push_str(&format!(
                        "/// Test resource '{}' shared lock (max {} holders).\n",
                        resource.resource_id, max_holders
                    ));
                    "SharedLock"
                }
                gunbc_test::ResourceType::PoolSlot { pool_size } => {
                    code.push_str(&format!(
                        "/// Test resource '{}' pool slot (pool size {}).\n",
                        resource.resource_id, pool_size
                    ));
                    "PoolSlot"
                }
            };

            if !code.ends_with("///") {
                code.push_str(&format!(
                    "/// Test resource '{}' ({}) acquisition.\n",
                    resource.resource_id, resource_type
                ));
            }
            code.push_str("#[test]\n");
            code.push_str(&format!("fn {}() {{\n", test_name));
            code.push_str("    let spec = mock_spec();\n");
            code.push_str(&format!(
                "    let resource = spec.get_resource(\"{}\").expect(\"resource should exist\");\n",
                resource.resource_id
            ));
            code.push_str("    let result = resource.acquire();\n");

            let has_fail = resource
                .behaviors
                .iter()
                .any(|b| matches!(b, gunbc_test::ResourceBehavior::FailAcquire { .. }));
            if has_fail {
                code.push_str(
                    "    assert!(matches!(result, ResourceAcquireResult::Failed(_)), \"should fail to acquire\");\n",
                );
            } else {
                code.push_str(
                    "    assert!(matches!(result, ResourceAcquireResult::Acquired), \"should acquire successfully\");\n",
                );
            }

            code.push_str("}\n\n");

            // Lease expiration test
            if let gunbc_test::ResourceType::Lease { duration_ms } = resource.resource_type {
                let timeout_test = format!(
                    "test_resource_{}_timeout",
                    sanitize_resource_id(&resource.resource_id)
                );
                code.push_str(&format!(
                    "/// Test resource '{}' lease expiration after {}ms.\n",
                    resource.resource_id, duration_ms
                ));
                code.push_str("#[test]\n");
                code.push_str(&format!("fn {}() {{\n", timeout_test));
                code.push_str("    let spec = mock_spec();\n");
                code.push_str(&format!(
                    "    let resource = spec.get_resource(\"{}\").expect(\"resource should exist\");\n",
                    resource.resource_id
                ));
                code.push_str(&format!(
                    "    assert!(!resource.should_timeout({}), \"should not timeout before duration\");\n",
                    duration_ms / 2
                ));
                code.push_str(&format!(
                    "    assert!(resource.should_timeout({}), \"should timeout after duration\");\n",
                    duration_ms + 1
                ));
                code.push_str("}\n\n");
            }
        }

        code
    }

    // =======================================================================
    // Legacy test generators (preserved for backward compatibility)
    // =======================================================================

    /// Get mock value for a boundary port, using MockSpec if available.
    fn get_mock_value(&self, node: &str, port: &str, type_id: &str) -> String {
        if let Some(spec) = &self.mock_spec {
            if let Some(value) = spec.get_boundary_mock(node, port) {
                return value_to_rust_literal(value);
            }
        }
        default_mock_for_type(type_id)
    }

    /// Generate flow verification tests.
    fn generate_flow_tests(&self, _analysis: &DagAnalysis, graph_builder_fn: &str) -> String {
        let mut code = String::new();

        let Some(spec) = &self.mock_spec else {
            return code;
        };

        if !spec.has_flow_test_data() {
            return code;
        }

        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Flow Verification Tests\n");
        code.push_str(
            "// These tests execute the full DAG in DryRun mode with mocked transport\n",
        );
        code.push_str(
            "// responses, verifying that pure node logic produces expected outputs.\n",
        );
        code.push_str(
            "// ============================================================================\n\n",
        );

        let test_name = format!(
            "test_flow_{}",
            NamingCase::SnakeCase.apply(&spec.name)
        );

        code.push_str(&format!(
            "/// Flow verification: {} scenario.\n",
            spec.name
        ));
        code.push_str("///\n");
        code.push_str("/// Builds the DAG, injects mocked transport responses via DryRun,\n");
        code.push_str(
            "/// and verifies that the pure node chain produces expected terminal outputs.\n",
        );
        code.push_str("#[test]\n");
        code.push_str(&format!("fn {}() {{\n", test_name));
        code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
        code.push_str("    let spec = mock_spec();\n");
        code.push_str("    let mocks = spec.to_boundary_mocks();\n");
        code.push_str(
            "    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))\n",
        );
        code.push_str("        .expect(\"DryRun execution should succeed\");\n");
        code.push('\n');

        for eo in &spec.expected_outputs {
            code.push_str(&format!(
                "    // Verify {}.{}\n",
                eo.node, eo.port
            ));
            code.push_str(&format!(
                "    let entry = log.get(\"{}\").expect(\"node '{}' should be in execution log\");\n",
                eo.node, eo.node
            ));
            code.push_str(&format!(
                "    assert_eq!(\n        entry.outputs.get(\"{}\").expect(\"port '{}' should exist on '{}'\"),\n        &{},\n        \"flow verification: {}.{} mismatch\"\n    );\n",
                eo.port, eo.port, eo.node,
                value_to_rust_literal(&eo.expected),
                eo.node, eo.port
            ));
            code.push('\n');
        }

        code.push_str("}\n\n");
        code
    }

    /// Generate boundary tests.
    fn generate_boundary_tests(&self, analysis: &DagAnalysis, graph_builder_fn: &str) -> String {
        let mut code = String::new();

        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Boundary Tests (per-node mockability)\n");
        code.push_str(
            "// ============================================================================\n\n",
        );

        // Overall boundary test
        let mocks_expr = if self.mock_spec_fn.is_some() {
            "mock_spec().to_boundary_mocks()"
        } else {
            "default_mocks()"
        };
        code.push_str("/// Test that all boundaries can be mocked.\n");
        code.push_str("#[test]\n");
        code.push_str("fn test_boundaries_mockable() {\n");
        code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
        code.push_str(&format!(
            "    let result = assert_boundary_mockable(&dag, {});\n",
            mocks_expr
        ));
        code.push_str(
            "    assert!(result.is_ok(), \"Boundaries should be mockable: {:?}\", result.error);\n",
        );
        code.push_str("}\n\n");

        // Per-boundary-node tests
        for boundary_node in &analysis.boundaries.boundary_nodes {
            let test_name = format!(
                "test_boundary_{}_mockable",
                NamingCase::SnakeCase.apply(&boundary_node.0)
            );
            let node_name = &boundary_node.0;

            code.push_str(&format!(
                "/// Test that {} boundary can be mocked.\n",
                node_name
            ));
            code.push_str("#[test]\n");
            code.push_str(&format!("fn {}() {{\n", test_name));
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
            code.push_str("    let boundaries = detect_boundaries(&dag);\n");
            code.push_str(&format!(
                "    assert!(boundaries.is_boundary_node(&\"{}\".into()), \"{} should be a boundary\");\n",
                node_name, node_name
            ));
            code.push_str("    \n");
            code.push_str("    let mut mocks = BoundaryMocks::new();\n");

            for (node_id, port_name) in &analysis.boundaries.boundary_ports {
                if node_id == boundary_node {
                    let type_id = self
                        .dag
                        .nodes
                        .iter()
                        .find(|n| n.id == *node_id)
                        .and_then(|n| n.outputs.iter().find(|p| p.name == *port_name))
                        .map(|p| p.type_id.0.as_str())
                        .unwrap_or("String");

                    let mock_value = self.get_mock_value(&node_id.0, &port_name.0, type_id);
                    code.push_str(&format!(
                        "    mocks.set_value(\"{}\", \"{}\", {});\n",
                        node_id.0, port_name.0, mock_value
                    ));
                }
            }

            code.push_str("    \n");
            code.push_str(
                "    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();\n",
            );
            code.push_str(&format!(
                "    let entry = log.get(\"{}\").expect(\"node should be in log\");\n",
                node_name
            ));
            code.push_str(
                "    assert!(entry.was_intercepted, \"boundary should be intercepted in dry-run\");\n",
            );
            code.push_str("}\n\n");
        }

        code
    }

    /// Generate chain validation tests.
    fn generate_chain_tests(&self, _analysis: &DagAnalysis) -> String {
        let mut code = String::new();

        let Some(spec) = &self.mock_spec else {
            return code;
        };

        if spec.input_expectations.is_empty() && spec.boundary_mocks.is_empty() {
            return code;
        }

        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Chain Validation Tests\n");
        code.push_str(
            "// These tests verify that mock outputs satisfy downstream input expectations.\n",
        );
        code.push_str(
            "// ============================================================================\n\n",
        );

        // Self-consistency
        code.push_str("/// Test that this tool's mock spec is self-consistent.\n");
        code.push_str("#[test]\n");
        code.push_str("fn test_mock_spec_self_consistent() {\n");
        code.push_str("    let spec = mock_spec();\n");
        code.push_str("    // Verify all boundary mocks are present\n");

        for mock in &spec.boundary_mocks {
            code.push_str(&format!(
                "    assert!(spec.get_boundary_mock(\"{}\", \"{}\").is_some(), \n",
                mock.node, mock.port
            ));
            code.push_str(&format!(
                "        \"MockSpec should have boundary mock for {}.{}\");\n",
                mock.node, mock.port
            ));
        }

        code.push_str("}\n\n");

        // Input expectations
        if !spec.input_expectations.is_empty() {
            code.push_str("/// Test that input expectations are documented.\n");
            code.push_str("#[test]\n");
            code.push_str("fn test_input_expectations_documented() {\n");
            code.push_str("    let spec = mock_spec();\n");

            for exp in &spec.input_expectations {
                let constraint_str = match &exp.constraint {
                    gunbc_test::InputConstraint::NonEmpty => "NonEmpty",
                    gunbc_test::InputConstraint::Any => "Any",
                    gunbc_test::InputConstraint::OneOf(_) => "OneOf(...)",
                    gunbc_test::InputConstraint::TypePattern(_) => "TypePattern(...)",
                    gunbc_test::InputConstraint::Custom { description, .. } => {
                        description.as_str()
                    }
                };
                code.push_str(&format!(
                    "    // Port '{}' expects: {}\n",
                    exp.port, constraint_str
                ));
            }

            code.push_str(&format!(
                "    assert_eq!(spec.input_expectations.len(), {});\n",
                spec.input_expectations.len()
            ));
            code.push_str("}\n\n");
        }

        code
    }

    // =======================================================================
    // Node I/O Example Tests
    // =======================================================================

    /// Generate tests from MockSpec.node_examples and Node.examples.
    ///
    /// Examples come from two sources:
    /// 1. `MockSpec.node_examples` — rich matchers (Contains, NonEmpty, Satisfies)
    /// 2. `Node.examples` — exact Value matching (defined on the node itself)
    ///
    /// Each example produces a test that:
    /// 1. Executes the single node with the given inputs
    /// 2. Asserts outputs match the expected values/matchers
    fn generate_node_example_tests(&self, graph_builder_fn: &str) -> String {
        let mut code = String::new();

        let mockspec_examples = self
            .mock_spec
            .as_ref()
            .map(|s| &s.node_examples[..])
            .unwrap_or(&[]);

        let has_node_examples = self.dag.nodes.iter().any(|n| !n.examples.is_empty());

        if mockspec_examples.is_empty() && !has_node_examples {
            return code;
        }

        code.push_str(
            "// ============================================================================\n",
        );
        code.push_str("// Node I/O Example Tests\n");
        code.push_str(
            "// These tests verify individual node behavior against specified examples.\n",
        );
        code.push_str(
            "// Each test executes a single node with given inputs and checks outputs.\n",
        );
        code.push_str(
            "// ============================================================================\n\n",
        );

        // 1. MockSpec-sourced examples (rich matchers)
        for (idx, example) in mockspec_examples.iter().enumerate() {
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

            if let Some(desc) = &example.description {
                code.push_str(&format!("/// Node example: {} - {}\n", example.node_id, desc));
            } else {
                code.push_str(&format!(
                    "/// Node example: {} (example {})\n",
                    example.node_id, idx
                ));
            }
            code.push_str("///\n");
            code.push_str(&format!(
                "/// Tests that node '{}' produces expected outputs for given inputs.\n",
                example.node_id
            ));
            code.push_str("#[test]\n");
            code.push_str(&format!("fn {}() {{\n", test_name));
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));

            if example.inputs.is_empty() {
                code.push_str("    let inputs = std::collections::HashMap::new();\n");
            } else {
                code.push_str("    let mut inputs = std::collections::HashMap::new();\n");
            }

            let mut sorted_inputs: Vec<_> = example.inputs.iter().collect();
            sorted_inputs.sort_by_key(|(k, _)| k.as_str());
            for (port, value) in sorted_inputs {
                code.push_str(&format!(
                    "    inputs.insert(\"{}\".to_string(), {});\n",
                    port,
                    value_to_rust_literal(value)
                ));
            }

            code.push_str(&format!(
                "    let outputs = gunbc_exec::execute_single_node(&dag, \"{}\", inputs, gunbc_exec::ExecutionMode::Real)\n",
                example.node_id
            ));
            code.push_str(&format!(
                "        .expect(\"node '{}' should execute successfully\");\n\n",
                example.node_id
            ));

            let mut sorted_outputs: Vec<_> = example.outputs.iter().collect();
            sorted_outputs.sort_by_key(|(k, _)| k.as_str());
            for (port, matcher) in sorted_outputs {
                let var_name = NamingCase::SnakeCase.apply(port);
                let prefix = if matcher.generates_assertion() { "" } else { "_" };
                code.push_str(&format!("    // Check output port '{}'\n", port));
                code.push_str(&format!(
                    "    let {}output_{} = outputs.get(\"{}\").expect(\"output port '{}' should exist\");\n",
                    prefix, var_name, port, port
                ));

                let check_code = render_output_matcher_check(matcher, &var_name);
                code.push_str(&format!("    {}\n", check_code));
            }

            code.push_str("}\n\n");
        }

        // 2. Node-sourced examples (exact Value matching)
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

                if let Some(desc) = &example.description {
                    code.push_str(&format!(
                        "/// Node I/O example: {} - {}\n",
                        node.id.0, desc
                    ));
                } else {
                    code.push_str(&format!(
                        "/// Node I/O example: {} (example {})\n",
                        node.id.0, idx
                    ));
                }
                code.push_str("///\n");
                code.push_str(&format!(
                    "/// Tests that node '{}' produces expected outputs for given inputs (exact match).\n",
                    node.id.0
                ));
                code.push_str("#[test]\n");
                code.push_str(&format!("fn {}() {{\n", test_name));
                code.push_str(&format!("    let dag = {};\n", graph_builder_fn));

                if example.inputs.is_empty() {
                    code.push_str("    let inputs = std::collections::HashMap::new();\n");
                } else {
                    code.push_str("    let mut inputs = std::collections::HashMap::new();\n");
                }

                let mut sorted_inputs: Vec<_> = example.inputs.iter().collect();
                sorted_inputs.sort_by_key(|(k, _)| k.as_str());
                for (port, value) in sorted_inputs {
                    code.push_str(&format!(
                        "    inputs.insert(\"{}\".to_string(), {});\n",
                        port,
                        value_to_rust_literal(value)
                    ));
                }

                code.push_str(&format!(
                    "    let outputs = gunbc_exec::execute_single_node(&dag, \"{}\", inputs, gunbc_exec::ExecutionMode::Real)\n",
                    node.id.0
                ));
                code.push_str(&format!(
                    "        .expect(\"node '{}' should execute successfully\");\n\n",
                    node.id.0
                ));

                let mut sorted_outputs: Vec<_> = example.expected_outputs.iter().collect();
                sorted_outputs.sort_by_key(|(k, _)| k.as_str());
                for (port, expected) in sorted_outputs {
                    code.push_str(&format!("    // Check output port '{}'\n", port));
                    code.push_str(&format!(
                        "    assert_eq!(\n        outputs.get(\"{}\").expect(\"output port '{}' should exist\"),\n        &{},\n        \"node '{}' port '{}' should match expected value\"\n    );\n",
                        port, port,
                        value_to_rust_literal(expected),
                        node.id.0, port
                    ));
                }

                code.push_str("}\n\n");
            }
        }

        code
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

/// Convert a Value to a Rust literal string via ValueExpr.
///
/// Uses the ValueExpr intermediate representation — every Value variant
/// is handled exhaustively in `Value → ValueExpr`, and every ValueExpr
/// variant is handled exhaustively in `RustRenderer::render_value`.
/// No catch-all at either stage.
fn value_to_rust_literal(value: &Value) -> String {
    RustRenderer.render_value(&ValueExpr::from(value))
}

/// Render an output matcher assertion as a single line of Rust code.
///
/// Uses the test IR (Assert, Expr) and RustRenderer to produce the assertion,
/// replacing the old `OutputMatcher::to_check_code()` string interpolation.
/// The `var_name` is the snake_case port name; the actual variable is `output_{var_name}`.
fn render_output_matcher_check(matcher: &OutputMatcher, var_name: &str) -> String {
    let output_var = format!("output_{}", var_name);
    let result = match matcher {
        OutputMatcher::Exact(expected) => {
            let assert = Assert::Eq {
                left: Expr::var(&output_var).deref(),
                right: Expr::Value(ValueExpr::from(expected)),
                message: "expected exact value".to_string(),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::Contains(substring) => {
            let assert = Assert::Contains {
                expr: Expr::var(&output_var),
                substring: substring.clone(),
                message: format!("expected to contain '{}', got: {{:?}}", substring),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::NonEmpty => {
            let assert = Assert::NonEmpty {
                expr: Expr::var(&output_var),
                message: "expected non-empty value".to_string(),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::IsBool => {
            let assert = Assert::True {
                expr: Expr::var(&output_var).method("as_bool", vec![]).method("is_some", vec![]),
                message: format!("expected Bool for {}", output_var),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::IsInt => {
            let assert = Assert::True {
                expr: Expr::var(&output_var).method("as_int", vec![]).method("is_some", vec![]),
                message: format!("expected Int for {}", output_var),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::IsString => {
            let assert = Assert::True {
                expr: Expr::var(&output_var).method("as_str", vec![]).method("is_some", vec![]),
                message: format!("expected String for {}", output_var),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::IsRequest => {
            let assert = Assert::True {
                expr: Expr::var(&output_var).method("as_request", vec![]).method("is_some", vec![]),
                message: format!("expected Request for {}", output_var),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::IsResponse => {
            let assert = Assert::True {
                expr: Expr::var(&output_var).method("as_response", vec![]).method("is_some", vec![]),
                message: format!("expected Response for {}", output_var),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::IntGe(threshold) => {
            // assert!(var.as_int().unwrap() >= threshold)
            let assert = Assert::True {
                expr: Expr::var(&output_var)
                    .method("as_int", vec![])
                    .method("is_some_and", vec![
                        Expr::Closure {
                            args: vec!["n".to_string()],
                            body: Box::new(Expr::Str(format!("n >= {}", threshold))),
                        },
                    ]),
                message: format!("expected Int >= {} for {}", threshold, output_var),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::IntLe(threshold) => {
            let assert = Assert::True {
                expr: Expr::var(&output_var)
                    .method("as_int", vec![])
                    .method("is_some_and", vec![
                        Expr::Closure {
                            args: vec!["n".to_string()],
                            body: Box::new(Expr::Str(format!("n <= {}", threshold))),
                        },
                    ]),
                message: format!("expected Int <= {} for {}", threshold, output_var),
            };
            RustRenderer.render_assert(&assert, 0)
        }
        OutputMatcher::Satisfies { description, .. } => {
            format!("// Custom assertion: {}\n", description)
        }
        OutputMatcher::Any => {
            format!("// Any value accepted for {}\n", output_var)
        }
    };
    // render_assert appends \n; strip it since the call site adds its own \n
    result.trim_end_matches('\n').to_string()
}

/// Generate a mock value for a specific cardinality case and type.
///
/// Builds a ValueExpr and renders it via RustRenderer. The type_id string
/// matching is a known limitation — ideally this should consult DagAnalysis
/// cardinality data instead (see TODO_codegen_dag.md, "severed analysis").
fn cardinality_case_mock_value(case: CardinalityCase, type_id: &str) -> String {
    let expr = match case {
        CardinalityCase::Empty => match type_id {
            "String" => ValueExpr::Str(String::new()),
            "Bool" => ValueExpr::Bool(false),
            "Int" | "i64" | "i32" => ValueExpr::Int(0),
            _ => ValueExpr::List(vec![]),
        },
        CardinalityCase::One => match type_id {
            "String" => ValueExpr::Str("<MOCK>".to_string()),
            "Bool" => ValueExpr::Bool(true),
            "Int" | "i64" | "i32" => ValueExpr::Int(1),
            _ => ValueExpr::List(vec![ValueExpr::Str("<MOCK>".to_string())]),
        },
        CardinalityCase::Many => match type_id {
            "String" => ValueExpr::List(vec![
                ValueExpr::Str("<MOCK_1>".to_string()),
                ValueExpr::Str("<MOCK_2>".to_string()),
                ValueExpr::Str("<MOCK_3>".to_string()),
            ]),
            "Bool" => ValueExpr::List(vec![
                ValueExpr::Bool(true),
                ValueExpr::Bool(false),
                ValueExpr::Bool(true),
            ]),
            "Int" | "i64" | "i32" => ValueExpr::List(vec![
                ValueExpr::Int(1),
                ValueExpr::Int(2),
                ValueExpr::Int(3),
            ]),
            _ => ValueExpr::List(vec![
                ValueExpr::Str("<MOCK_1>".to_string()),
                ValueExpr::Str("<MOCK_2>".to_string()),
                ValueExpr::Str("<MOCK_3>".to_string()),
            ]),
        },
    };
    RustRenderer.render_value(&expr)
}

/// Generate a default mock value for a type.
///
/// Builds a ValueExpr and renders it via RustRenderer. Transport types
/// are now modeled as Struct ValueExprs rather than hardcoded format strings.
fn default_mock_for_type(type_id: &str) -> String {
    let expr = match type_id {
        "String" => ValueExpr::Str("<MOCK>".to_string()),
        "Bool" => ValueExpr::Bool(true),
        "Int" | "i64" | "i32" => ValueExpr::Int(0),
        "List" => ValueExpr::List(vec![ValueExpr::Str("<MOCK>".to_string())]),
        "Secret" => ValueExpr::Secret("<MOCK_SECRET>".to_string()),
        "TransportResponse" => ValueExpr::Struct {
            name: "TransportResponse::Shell".to_string(),
            fields: vec![
                ("exit_code".to_string(), ValueExpr::Int(0)),
                ("stdout".to_string(), ValueExpr::Str("<MOCK>".to_string())),
                ("stderr".to_string(), ValueExpr::Str(String::new())),
            ],
        },
        _ => ValueExpr::Str("<MOCK>".to_string()),
    };
    RustRenderer.render_value(&expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build, build::*, Dag, Node, Value};

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
            .skip_node_example("source")
            .skip_node_example("sink");
        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
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
        assert!(code.contains("Content-Hash:"), "should have content hash in header");

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
            .skip_node_example("source")
            .skip_node_example("sink");
        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
        let code1 = generator.generate_test_module("example", "build_example_graph()");
        let code2 = generator.generate_test_module("example", "build_example_graph()");

        // Same DAG should produce identical output (including hash)
        assert_eq!(code1, code2, "content hash should be deterministic");

        // Extract the hash value
        let hash_line = code1.lines()
            .find(|l| l.contains("Content-Hash:"))
            .expect("should have Content-Hash line");
        let hash = hash_line.split("Content-Hash: ").nth(1).unwrap().trim();
        assert_eq!(hash.len(), 16, "hash should be 16 hex chars");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "hash should be hex");
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

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should use mock spec value
        assert!(code.contains("test_output"));
        // Should have chain tests
        assert!(code.contains("test_mock_spec_self_consistent"));
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

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should have resource tests
        assert!(code.contains("test_resource_db_write_acquire"));
        assert!(code.contains("test_resource_api_token_acquire"));
        assert!(code.contains("test_resource_api_token_timeout"));
    }

    #[test]
    fn test_generate_with_transport_executor() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "prepare",
            vec![],
            vec![port("request", "TransportRequest")],
            (),
        ));
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            (),
        ));
        dag.add_node(Node::opaque(
            "parse",
            vec![port("response", "TransportResponse")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("prepare", "request", "execute", "request"));
        dag.add_edge(edge("execute", "response", "parse", "response"));

        // MockSpec required for DAGs with transport executors
        let spec = MockSpec::new("example")
            .boundary("execute", "response", Value::Str("<MOCK_RESPONSE>".into()))
            .skip_node_example("prepare")
            .skip_node_example("parse");

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should have transport interception test
        assert!(code.contains("test_transport_interception"));

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
    fn test_generate_with_bool_guard() {
        let mut dag: Dag<()> = Dag::new();

        // Transport executor that produces a condition
        dag.add_node(Node::opaque(
            "check",
            vec![port("request", "TransportRequest")],
            vec![
                port("response", "TransportResponse"),
                port("condition", "Bool"),
            ],
            (),
        ));
        // Guarded node: only executes when condition is true
        dag.add_node(Node::opaque(
            "process",
            vec![
                build::guarded("condition", "Bool", Value::Bool(true)),
                port("data", "String"),
            ],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("check", "condition", "process", "condition"));
        dag.add_edge(edge("check", "response", "process", "data"));

        // MockSpec required for DAGs with transport executors
        let spec = MockSpec::new("guarded")
            .boundary("check", "response", Value::Str("<MOCK>".into()))
            .boundary("check", "condition", Value::Bool(true))
            .skip_node_example("process");

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
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
            .skip_node_example("source")
            .skip_node_example("conditional");
        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
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
        dag.add_node(Node::opaque(
            "transform",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));

        // MockSpec provided but no examples and no skip — should panic
        let spec = MockSpec::new("test");
        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
        let _ = generator.generate_test_module("test", "build_test_graph()");
    }

    #[test]
    #[should_panic(expected = "MockSpec required")]
    fn test_mockspec_required_for_transport_dags() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            (),
        ));

        // No MockSpec provided - should panic
        let generator = TestGenerator::new(&dag);
        let _ = generator.generate_test_module("test", "build_test_graph()");
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
            .skip_node_example("source")
            .skip_node_example("sink");

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
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

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should have node example tests section
        assert!(
            code.contains("Node I/O Example Tests"),
            "should have example tests section header"
        );

        // Should generate test function (description sanitized to snake_case)
        assert!(
            code.contains("test_example_prepare_basic_input_processing"),
            "should generate test with description-based name: {}", code
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

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
        let code = generator.generate_test_module("exact", "build_exact_graph()");

        // Should have exact assertion
        assert!(
            code.contains("assert_eq!"),
            "should have exact match assertion"
        );
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

        let generator = TestGenerator::new(&dag).with_mock_spec(spec);
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

        let generator = TestGenerator::new(&dag);
        let code = generator.generate_test_module("node_ex", "build_node_ex_graph()");

        // Should have example tests section
        assert!(
            code.contains("Node I/O Example Tests"),
            "should have example tests section: {}", code
        );

        // Should generate test from node-sourced example
        assert!(
            code.contains("test_node_example_upper_basic_uppercase"),
            "should generate test with node example name: {}", code
        );

        // Should use exact match (assert_eq!)
        assert!(
            code.contains("assert_eq!"),
            "node examples should use exact match: {}", code
        );

        // Should reference the expected value
        assert!(
            code.contains("HELLO"),
            "should contain expected output value: {}", code
        );
    }
}
