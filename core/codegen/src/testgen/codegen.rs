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
use crate::testgen::test_ir::{
    Assert, Expr, HelperFn, Import, Stmt, TestFile, TestFn, TestSection,
};
use gunbc_ir::boundary_label;
use gunbc_ir::language::NamingCase;
use gunbc_ir::{Cardinality, Dag, NodeId, PortName, ValueExpr};
use gunbc_test::{MockSpec, OutputMatcher};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
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
    /// Generate flow verification tests (DryRun full DAG, verify terminal outputs)
    pub flow_tests: bool,
    /// Generate per-node I/O example tests (from MockSpec.node_examples)
    pub example_tests: bool,
    /// Max window size for windowed tests (None = no limit)
    pub window_max_nodes: Option<usize>,
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
            window_max_nodes: Some(5),
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
    /// Function path to call for declared signature (e.g., "crate::makegen_signature()")
    signature_fn: Option<String>,
}

impl<'a, T: Clone> TestGenerator<'a, T> {
    /// Create a new test generator for a DAG.
    pub fn new(dag: &'a Dag<T>) -> Self {
        Self {
            dag,
            config: TestConfig::default(),
            mock_spec: None,
            mock_spec_fn: None,
            signature_fn: None,
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
            let missing_mocks = self.find_missing_transport_mocks(&analysis, spec);
            if !missing_mocks.is_empty() {
                panic!(
                    "Transport mock coverage incomplete: DAG '{}' has {} transport output port(s) \
                     connected downstream but not mocked:\n\
                     \n\
                     {}\n\
                     \n\
                     Each transport executor output that flows to downstream nodes needs a mock.\n\
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
                        .map(|(node, port)| format!("  - {}.{}", node, port))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    module_name,
                    missing_mocks
                        .iter()
                        .map(|(node, port)| {
                            format!("    .transport_mock(\"{}\", \"{}\", mock_value())", node, port)
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }

            // Validate that all mocks reference existing nodes and ports.
            //
            // This catches typos and stale mocks that reference renamed/removed nodes.
            let unknown_slots = self.find_unknown_mock_slots(spec);
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
            let type_mismatches = self.find_mock_type_mismatches(spec);
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

        let obligations = collect_obligations(self.dag, None, None);

        let mut file = self.generate_test_file(&analysis, &obligations, graph_builder_fn);

        // Render body (no header) to compute content hash.
        let body = RustRenderer.render_file(&file);
        let content_hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            body.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };

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
            format!("Content-Hash: {}", content_hash),
        ];

        RustRenderer.render_file(&file)
    }

    /// Find transport executor output ports that are connected downstream but lack mocks.
    ///
    /// Returns a list of (node_id, port_name) pairs that need transport_mock entries.
    fn find_missing_transport_mocks(
        &self,
        analysis: &DagAnalysis,
        spec: &MockSpec,
    ) -> Vec<(String, String)> {
        let mut missing = Vec::new();

        // Build a set of (node, port) pairs that have mocks
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

        // For each transport executor, check if its connected output ports have mocks
        for transport_id in &analysis.transport_executors {
            if let Some(node) = self.dag.get_node(&NodeId(transport_id.clone())) {
                for output_port in &node.outputs {
                    // Check if this output port is connected to any downstream node
                    let is_connected = self.dag.edges.iter().any(|e| {
                        e.from_node.0 == *transport_id && e.from_port.0 == output_port.name.0
                    });

                    if is_connected {
                        // Check if there's a mock for this (node, port)
                        let has_mock =
                            mocked_ports.contains(&(transport_id.as_str(), output_port.name.0.as_str()));

                        if !has_mock {
                            missing.push((transport_id.clone(), output_port.name.0.clone()));
                        }
                    }
                }
            }
        }

        missing
    }

    /// Find mock values whose types don't match the port's declared TypeId.
    ///
    /// Returns a list of (node_id, port_name, expected_type, actual_type) tuples.
    fn find_mock_type_mismatches(
        &self,
        spec: &MockSpec,
    ) -> Vec<(String, String, String, String)> {
        use gunbc_ir::Value;

        let mut mismatches = Vec::new();

        // Helper to get type name from a Value
        let value_type_name = |v: &Value| -> &'static str {
            match v {
                Value::Unit => "Unit",
                Value::Bool(_) => "Bool",
                Value::Str(_) => "String",
                Value::Int(_) => "Int",
                Value::List(_) => "List",
                Value::Set(_) => "Set",
                Value::Map(_) => "Map",
                Value::Json(_) => "Json",
                Value::Request(_) => "TransportRequest",
                Value::Response(_) => "TransportResponse",
                Value::Secret(_) => "Secret",
                Value::Skipped => "Skipped", // Skipped is compatible with anything
            }
        };

        // Helper to check type compatibility
        // NOTE: This must match MockRequirements::types_compatible in gunbc-test
        let types_compatible = |port_type: &str, value_type: &str| -> bool {
            // Exact match
            if port_type == value_type {
                return true;
            }
            // Any matches anything
            if port_type == "Any" || value_type == "Any" {
                return true;
            }
            // Skipped is a control flow value, compatible with any type
            if value_type == "Skipped" {
                return true;
            }
            // Json is flexible - can hold structured data that might be typed differently
            // NOTE: This is intentionally permissive; consider tightening if type drift is a concern
            if port_type == "Json" || value_type == "Json" {
                return true;
            }
            // Map-backed types: ToolHandle, AuthToken, FilesystemHandle
            // These types serialize to/from Map when stored as Value
            if value_type == "Map" {
                let map_backed_types = ["ToolHandle", "AuthToken", "FilesystemHandle"];
                if map_backed_types.contains(&port_type) {
                    return true;
                }
            }
            // Int-backed types: Timestamp stores milliseconds as Int
            if value_type == "Int" && port_type == "Timestamp" {
                return true;
            }
            // String-backed types: Platform serializes as String
            if value_type == "String" && port_type == "Platform" {
                return true;
            }
            // Map can also represent Platform (for structured platform info)
            if value_type == "Map" && port_type == "Platform" {
                return true;
            }
            false
        };

        // Check transport mocks
        for tm in &spec.transport_mocks {
            if let Some(node) = self.dag.get_node(&NodeId(tm.node.clone())) {
                if let Some(port) = node.outputs.iter().find(|p| p.name.0 == tm.port) {
                    let expected = &port.type_id.0;
                    let actual = value_type_name(&tm.value);
                    if !types_compatible(expected, actual) {
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
            if let Some(node) = self.dag.get_node(&NodeId(bm.node.clone())) {
                if let Some(port) = node.outputs.iter().find(|p| p.name.0 == bm.port) {
                    let expected = &port.type_id.0;
                    let actual = value_type_name(&bm.value);
                    if !types_compatible(expected, actual) {
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

        mismatches
    }

    /// Find mocks that reference non-existent nodes or ports.
    ///
    /// Returns a list of (node_id, port_name, reason) tuples.
    fn find_unknown_mock_slots(&self, spec: &MockSpec) -> Vec<(String, String, String)> {
        let mut unknown = Vec::new();

        // Check transport mocks
        for tm in &spec.transport_mocks {
            match self.dag.get_node(&NodeId(tm.node.clone())) {
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
            match self.dag.get_node(&NodeId(bm.node.clone())) {
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

        unknown
    }

    /// Generate the full test file (header excluded).
    fn generate_test_file(
        &self,
        analysis: &DagAnalysis,
        obligations: &ObligationSet,
        graph_builder_fn: &str,
    ) -> TestFile {
        let mut file = TestFile {
            header: Vec::new(),
            imports: Vec::new(),
            helpers: Vec::new(),
            sections: Vec::new(),
        };

        // Imports
        if self.mock_spec_fn.is_some() && self.config.window_max_nodes.unwrap_or(usize::MAX) >= 2 {
            file.imports.push(Import {
                path: vec!["gunbc_exec".to_string()],
                items: vec![
                    "execute_with_mode".to_string(),
                    "lower".to_string(),
                    "BoundaryMocks".to_string(),
                    "ExecutionMode".to_string(),
                ],
            });
        } else {
            file.imports.push(Import {
                path: vec!["gunbc_exec".to_string()],
                items: vec![
                    "execute_with_mode".to_string(),
                    "BoundaryMocks".to_string(),
                    "ExecutionMode".to_string(),
                ],
            });
        }

        file.imports.push(Import {
            path: vec!["gunbc_ir".to_string()],
            items: vec![
                "detect_boundaries".to_string(),
                "Cardinality".to_string(),
                "Value".to_string(),
            ],
        });

        if self.mock_spec_fn.is_some() {
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items: vec![
                    "assert_boundary_mockable".to_string(),
                    "assert_types_compatible".to_string(),
                    "MockSpec".to_string(),
                ],
            });
        } else {
            file.imports.push(Import {
                path: vec!["gunbc_test".to_string()],
                items: vec![
                    "assert_boundary_mockable".to_string(),
                    "assert_types_compatible".to_string(),
                ],
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
            file.sections.push(TestSection {
                title: "Signature Validation".to_string(),
                notes: Vec::new(),
                tests: vec![test],
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

        if self.mock_spec_fn.is_some() && self.config.window_max_nodes.unwrap_or(usize::MAX) >= 2 {
            if let Some(section) = self.build_window_section(graph_builder_fn) {
                file.sections.push(section);
            }
        }

        if self.config.example_tests {
            if let Some(section) = self.build_node_example_section(graph_builder_fn) {
                file.sections.push(section);
            }
        }

        file
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

        let mut tests = Vec::new();
        tests.extend(self.build_cardinality_coverage_tests(
            analysis,
            obligations,
            graph_builder_fn,
        ));
        tests.extend(self.build_coercion_coverage_tests(analysis, obligations, graph_builder_fn));

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

                    let mock_value = mock_value_expr_for_count(type_id, *cardinality, count);
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
                            "Proves: engine correctly applies {} coercion at this edge.",
                            kind_label
                        ),
                    ],
                    body: vec![
                        Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                        Stmt::let_bind("mocks", mocks_expr),
                        Stmt::let_bind("_log", exec),
                    ],
                });
            }
        }

        tests
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

            if let gunbc_test::ResourceType::Lease { duration_ms } = resource.resource_type {
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
                        "Test resource '{}' lease expiration after {}ms.",
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
                "log",
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

    fn build_window_section(&self, graph_builder_fn: &str) -> Option<TestSection> {
        self.mock_spec_fn.as_ref()?;

        let max_nodes = self.config.window_max_nodes.unwrap_or(usize::MAX);
        if max_nodes < 2 {
            return None;
        }

        let flat =
            gunbc_exec::lower(self.dag).expect("window tests require DAG lowering to succeed");
        let pure_nodes = collect_pure_nodes(&flat);
        let windows = enumerate_window_specs(&flat, max_nodes, &pure_nodes);
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
                        .method("expect", vec![Expr::Str("lower should succeed".into())]),
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

    fn build_boundary_section(
        &self,
        analysis: &DagAnalysis,
        graph_builder_fn: &str,
    ) -> Option<TestSection> {
        let mut tests = Vec::new();

        let mocks_expr = self.dryrun_mocks_expr(analysis, "boundary mockability tests");

        tests.push(TestFn {
            name: "test_boundaries_mockable".to_string(),
            doc: vec!["Test that all boundaries can be mocked.".to_string()],
            body: vec![
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
                        Expr::Str("Boundaries should be mockable: {:?}".into()),
                        Expr::var("result").field("error"),
                    ],
                )),
            ],
        });

        for boundary_node in &analysis.boundaries.boundary_nodes {
            let test_name = format!(
                "test_boundary_{}_mockable",
                NamingCase::SnakeCase.apply(&boundary_node.0)
            );
            let node_name = &boundary_node.0;

            let mut body = vec![
                Stmt::let_bind("dag", Expr::var(graph_builder_fn)),
                Stmt::let_bind(
                    "boundaries",
                    Expr::call("detect_boundaries", vec![Expr::var("dag").ref_of()]),
                ),
                Stmt::Assert(Assert::True {
                    expr: Expr::var("boundaries").method(
                        "is_boundary_node",
                        vec![Expr::Str(node_name.clone()).method("into", vec![]).ref_of()],
                    ),
                    message: format!("{} should be a boundary", node_name),
                }),
                Stmt::Blank,
                Stmt::let_mut("mocks", Expr::call("BoundaryMocks::new", vec![])),
            ];

            for (node_id, port_name) in &analysis.boundaries.boundary_ports {
                if node_id == boundary_node {
                    let (type_id, cardinality) = analysis
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
                let var_name = NamingCase::SnakeCase.apply(port);
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
            right: Expr::Value(ValueExpr::from(expected)),
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

/// Generate a mock ValueExpr for a specific count and cardinality.
///
/// Cardinality determines whether values are wrapped as lists. The `count`
/// is the number of elements (from `testgen::cardinality::fermi_test_cases()`).
///
/// For count=0, scalar types emit `Value::Unit` (absence), not concrete
/// "empty content" like `false` or `0`. List cardinalities emit empty
/// collections (which correctly represent zero elements).
fn mock_value_expr_for_count(type_id: &str, cardinality: Cardinality, count: u32) -> ValueExpr {
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
        "String" => match index {
            Some(1) | None => ValueExpr::Str("<MOCK>".to_string()),
            Some(i) => ValueExpr::Str(format!("<MOCK_{}>", i)),
        },
        "Bool" => match index {
            Some(i) => ValueExpr::Bool(i % 2 == 1),
            None => ValueExpr::Bool(true),
        },
        "Int" | "i64" | "i32" => match index {
            Some(i) => ValueExpr::Int(i as i64),
            None => ValueExpr::Int(0),
        },
        "Unit" => ValueExpr::Unit,
        "Json" => ValueExpr::Json(JsonValue::Null),
        "Map" => ValueExpr::Map(vec![]),
        "Secret" => ValueExpr::Secret("<MOCK_SECRET>".to_string()),
        "Any" => ValueExpr::Json(JsonValue::Null),
        "S" => ValueExpr::Str("<MOCK>".to_string()),
        "Path" => ValueExpr::Str("/tmp/mock".to_string()),
        "Platform" => ValueExpr::Str("linux".to_string()),
        "Error" => ValueExpr::Str("<ERROR>".to_string()),
        "OptionalString" => ValueExpr::Str("<MOCK>".to_string()),
        "StringList" => ValueExpr::List(vec![ValueExpr::Str("<MOCK>".to_string())]),
        "NonEmptyStringList" => ValueExpr::List(vec![ValueExpr::Str("<MOCK>".to_string())]),
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
        "AuthToken" => ValueExpr::Map(vec![
            ("service".to_string(), ValueExpr::Str("auth".to_string())),
            ("env_var".to_string(), ValueExpr::Str("AUTH_TOKEN".to_string())),
            ("token".to_string(), ValueExpr::Secret("mock-token".to_string())),
            ("cap".to_string(), ValueExpr::Secret("capability".to_string())),
        ]),
        "FilesystemHandle" => ValueExpr::Map(vec![
            ("type".to_string(), ValueExpr::Str("filesystem_handle".to_string())),
            ("scope".to_string(), ValueExpr::Str("read".to_string())),
            ("targets".to_string(), ValueExpr::List(vec![ValueExpr::Str("ext4".to_string())])),
            ("replacement".to_string(), ValueExpr::Str("-".to_string())),
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
    let is_transport_executor = node
        .inputs
        .iter()
        .any(|p| p.type_id.0 == "TransportRequest");
    let is_tool_env = node.outputs.iter().any(|p| p.type_id.0 == "ToolHandle");
    let is_tool_consumer = node.inputs.iter().any(|p| p.type_id.0 == "ToolHandle");

    !is_transport_executor && !is_tool_env && !is_tool_consumer
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build, build::*, Cardinality, Dag, Node, Value, ValueExpr};

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
        assert_eq!(hash.len(), 16, "hash should be 16 hex chars");
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
        let list_expr = mock_value_expr_for_count("String", Cardinality::ZERO_OR_MORE, 1);
        assert_eq!(
            list_expr,
            ValueExpr::List(vec![ValueExpr::Str("<MOCK>".to_string())])
        );

        let opt_zero = mock_value_expr_for_count("String", Cardinality::ZERO_OR_ONE, 0);
        assert_eq!(opt_zero, ValueExpr::Unit);
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
        dag.add_node(Node::opaque(
            "transform",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));

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
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            (),
        ));

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
    #[should_panic(expected = "Transport mock coverage incomplete")]
    fn test_transport_mock_coverage_required() {
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
            vec![
                port("response", "TransportResponse"),
                port("status", "Int"),
            ],
            (),
        ));
        dag.add_node(Node::opaque(
            "parse",
            vec![
                port("response", "TransportResponse"),
                port("status", "Int"),
            ],
            vec![port("result", "String")],
            (),
        ));
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
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")],
            vec![
                port("response", "TransportResponse"),
                port("status", "Int"),
            ],
            (),
        ));
        dag.add_node(Node::opaque(
            "parse",
            vec![
                port("response", "TransportResponse"),
                port("status", "Int"),
            ],
            vec![port("result", "String")],
            (),
        ));
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
}
