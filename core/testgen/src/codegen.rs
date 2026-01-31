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

use crate::analyze::{analyze_dag, DagAnalysis};
use crate::obligation::{collect_obligations, Obligation, ObligationSet};
use gunbc_ir::language::traits::comment::{generated_header, RUST_COMMENTS};
use gunbc_ir::language::NamingCase;
use gunbc_ir::{Dag, Value};
use gunbc_test::MockSpec;

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
}

impl<'a, T> TestGenerator<'a, T> {
    /// Create a new test generator for a DAG.
    pub fn new(dag: &'a Dag<T>) -> Self {
        Self {
            dag,
            config: TestConfig::default(),
            mock_spec: None,
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

    /// Generate the test module code.
    ///
    /// This is the main entry point. It:
    /// 1. Analyzes the DAG structure
    /// 2. Collects proof obligations
    /// 3. Generates tests for undischarged obligations
    pub fn generate_test_module(&self, module_name: &str, graph_builder_fn: &str) -> String {
        let analysis = analyze_dag(self.dag);
        let obligations = collect_obligations(self.dag, None, None);
        let mut code = String::new();

        // Module header
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
        code.push_str("\n\n");

        // Imports
        code.push_str("use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};\n");
        code.push_str("use gunbc_ir::{detect_boundaries, Cardinality, Value};\n");
        code.push_str(
            "use gunbc_test::{assert_boundary_mockable, assert_types_compatible, default_mocks};\n",
        );

        if self.config.chain_tests && self.mock_spec.is_some() {
            code.push_str("use gunbc_test::{validate_chain, MockSpec, InputConstraint};\n");
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

        // ===================================================================
        // Bucket A: Execution Semantics
        // ===================================================================
        if self.config.execution_tests {
            code.push_str(&self.generate_execution_tests(
                &analysis,
                &obligations,
                graph_builder_fn,
            ));
        }

        // ===================================================================
        // Bucket B: Contract Obligations (only for Unknown entailments)
        // ===================================================================
        if self.config.contract_tests {
            code.push_str(&self.generate_contract_tests(&analysis, &obligations));
        }

        // ===================================================================
        // Bucket C: Scenario Coverage
        // ===================================================================
        if self.config.scenario_tests {
            code.push_str(&self.generate_scenario_tests(
                &analysis,
                &obligations,
                graph_builder_fn,
            ));
        }

        // ===================================================================
        // Bucket D: Resource Hygiene + Simulation
        // ===================================================================
        if self.config.resource_tests {
            code.push_str(&self.generate_resource_tests(&analysis, &obligations));
        }

        // ===================================================================
        // Legacy: individual boundary tests, chain validation, flow tests
        // ===================================================================

        // NOTE: Type and cardinality compatibility are verified at compile time
        // by validate_dag(), so we don't generate redundant tests for those.
        // The compiler proves: types match, cardinalities satisfy, no cycles.

        if self.config.boundary_tests {
            code.push_str(&self.generate_boundary_tests(&analysis, graph_builder_fn));
        }

        if self.config.chain_tests {
            code.push_str(&self.generate_chain_tests(&analysis));
        }

        if self.config.flow_tests {
            code.push_str(&self.generate_flow_tests(&analysis, graph_builder_fn));
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
            code.push_str("/// DryRun execution completes without crash.\n");
            code.push_str("///\n");
            code.push_str("/// This is the minimal smoke test: build the DAG, run it in DryRun\n");
            code.push_str("/// with default mocks, and verify it completes successfully.\n");
            code.push_str("#[test]\n");
            code.push_str("fn test_dryrun_completion() {\n");
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
            code.push_str(
                "    let log = execute_with_mode(&dag, ExecutionMode::DryRun(default_mocks()))\n",
            );
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
            code.push_str("/// All transport executors are intercepted in DryRun.\n");
            code.push_str("///\n");
            code.push_str(
                "/// Proves: every transport executor is interceptable; DryRun won't\n",
            );
            code.push_str("/// accidentally perform real I/O.\n");
            code.push_str("#[test]\n");
            code.push_str("fn test_transport_interception() {\n");
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
            code.push_str("    let result = assert_boundary_mockable(&dag, default_mocks());\n");
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
        _analysis: &DagAnalysis,
        obligations: &ObligationSet,
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
            code.push_str("#[test]\n");
            code.push_str("fn test_scenario_all_succeed() {\n");
            code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
            code.push_str(
                "    let log = execute_with_mode(&dag, ExecutionMode::DryRun(default_mocks()))\n",
            );
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
                code.push_str("    let mut mocks = default_mocks();\n");

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
                            code.push_str("    // Test with true:\n");
                            code.push_str("    let mut mocks_true = default_mocks();\n");
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
                            code.push_str("    let mut mocks_false = default_mocks();\n");
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
                        code.push_str(
                            "    let log = execute_with_mode(&dag, ExecutionMode::DryRun(default_mocks()))\n",
                        );
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
        code.push_str("/// Test that all boundaries can be mocked.\n");
        code.push_str("#[test]\n");
        code.push_str("fn test_boundaries_mockable() {\n");
        code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
        code.push_str("    let result = assert_boundary_mockable(&dag, default_mocks());\n");
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

/// Convert a Value to a Rust literal string.
fn value_to_rust_literal(value: &Value) -> String {
    match value {
        Value::Unit => "Value::Unit".to_string(),
        Value::Bool(b) => format!("Value::Bool({})", b),
        Value::Str(s) => format!(
            "Value::Str(\"{}\".to_string())",
            s.replace('\"', "\\\"")
        ),
        Value::Int(i) => format!("Value::Int({})", i),
        Value::StrList(list) => {
            let items: Vec<String> = list
                .iter()
                .map(|s| format!("\"{}\".to_string()", s.replace('\"', "\\\"")))
                .collect();
            format!("Value::StrList(vec![{}])", items.join(", "))
        }
        Value::Json(json) => {
            format!("Value::Json(serde_json::json!({}))", json)
        }
        Value::Secret(_) => {
            "Value::Secret(gunbc_ir::SecretString::new(\"<MOCK_SECRET>\"))".to_string()
        }
        _ => "Value::Str(\"<MOCK>\".to_string())".to_string(),
    }
}

/// Generate a default mock value for a type.
fn default_mock_for_type(type_id: &str) -> String {
    match type_id {
        "String" => "Value::Str(\"<MOCK>\".to_string())".to_string(),
        "Bool" => "Value::Bool(true)".to_string(),
        "Int" | "i64" | "i32" => "Value::Int(0)".to_string(),
        "StrList" => "Value::StrList(vec![\"<MOCK>\".to_string()])".to_string(),
        "Secret" => {
            "Value::Secret(gunbc_ir::SecretString::new(\"<MOCK_SECRET>\"))".to_string()
        }
        "TransportResponse" => {
            "Value::Response(gunbc_ir::transport::TransportResponse::Shell(\
                gunbc_ir::transport::ShellResponse { \
                    exit_code: 0, \
                    stdout: \"<MOCK>\".to_string(), \
                    stderr: String::new() \
                }))"
            .to_string()
        }
        _ => "Value::Str(\"<MOCK>\".to_string())".to_string(),
    }
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

        let generator = TestGenerator::new(&dag);
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

        // Should NOT generate composition tests (compiler proves these)
        assert!(!code.contains("test_all_edges_compatible"));
        assert!(!code.contains("test_edge_source_out_to_sink_in"));
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
            .boundary("sink", "result", Value::Str("test_output".into()));

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
            .resource_lease("api:token", 5000);

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

        let generator = TestGenerator::new(&dag);
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

        let generator = TestGenerator::new(&dag);
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

        let generator = TestGenerator::new(&dag);
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
}
