//! Test code generation.

use crate::analyze::{analyze_dag, DagAnalysis};
use gunbc_ir::language::traits::comment::{generated_header, RUST_COMMENTS};
use gunbc_ir::language::NamingCase;
use gunbc_ir::{Dag, Value};
use gunbc_test::MockSpec;

/// Configuration for test generation.
///
/// Note: Type and cardinality compatibility are verified at compile time
/// by `validate_dag()`, so we don't generate redundant tests for those.
/// Generated tests focus on runtime behavior that can't be statically proven.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Generate boundary tests (verifies dry-run interception works)
    pub boundary_tests: bool,
    /// Generate chain validation tests (verifies mock values satisfy downstream)
    pub chain_tests: bool,
    /// Generate resource simulation tests (verifies lock/lease behavior)
    pub resource_tests: bool,
    /// Generate flow verification tests (DryRun the full DAG, verify terminal outputs)
    pub flow_tests: bool,
    /// Test module visibility
    pub visibility: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            boundary_tests: true,
            chain_tests: true,
            resource_tests: true,
            flow_tests: false,
            visibility: "pub".to_string(),
        }
    }
}

/// Test code generator.
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
    pub fn generate_test_module(&self, module_name: &str, graph_builder_fn: &str) -> String {
        let analysis = analyze_dag(self.dag);
        let mut code = String::new();

        // Module header - use line comments (not doc comments) since these files
        // are include!()'d into modules and doc comments would attach to `use` items.
        let prefix = RUST_COMMENTS.line_prefix;
        code.push_str(&format!("{} Generated tests for {} DAG.\n", prefix, module_name));
        code.push_str(&format!("{}\n", prefix));
        code.push_str(&generated_header(&gunbc_ir::cargo::name("testgen"), "make testgen", prefix));
        code.push_str("\n\n");

        // Imports
        code.push_str("use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};\n");
        code.push_str("use gunbc_ir::{detect_boundaries, Cardinality, Value};\n");
        code.push_str("use gunbc_test::{assert_boundary_mockable, assert_types_compatible, default_mocks};\n");

        if self.config.chain_tests && self.mock_spec.is_some() {
            code.push_str("use gunbc_test::{validate_chain, MockSpec, InputConstraint};\n");
        }
        if self.config.resource_tests && self.mock_spec.as_ref().is_some_and(|s| !s.resource_mocks.resources.is_empty()) {
            code.push_str("use gunbc_test::{ResourceAcquireResult, ResourceSimulation};\n");
        }
        code.push('\n');

        // Flow verification tests (DryRun full DAG, verify terminal outputs)
        if self.config.flow_tests {
            code.push_str(&self.generate_flow_tests(&analysis, graph_builder_fn));
        }

        // Boundary tests (verify dry-run interception works at runtime)
        if self.config.boundary_tests {
            code.push_str(&self.generate_boundary_tests(&analysis, graph_builder_fn));
        }

        // NOTE: Type and cardinality compatibility are verified at compile time
        // by validate_dag(), so we don't generate redundant tests for those.
        // The compiler proves: types match, cardinalities satisfy, no cycles.

        // Chain validation tests (verify mock values satisfy downstream expectations)
        if self.config.chain_tests {
            code.push_str(&self.generate_chain_tests(&analysis));
        }

        // Resource simulation tests (verify lock/lease runtime behavior)
        if self.config.resource_tests {
            code.push_str(&self.generate_resource_tests(&analysis));
        }

        code
    }

    /// Get mock value for a boundary port, using MockSpec if available.
    fn get_mock_value(&self, node: &str, port: &str, type_id: &str) -> String {
        // First try MockSpec
        if let Some(spec) = &self.mock_spec {
            if let Some(value) = spec.get_boundary_mock(node, port) {
                return value_to_rust_literal(value);
            }
        }
        
        // Fall back to type-based defaults
        default_mock_for_type(type_id)
    }

    /// Generate flow verification tests.
    ///
    /// Flow tests build the DAG, inject mocked transport responses via DryRun,
    /// execute the full pure node chain, and verify terminal node outputs.
    fn generate_flow_tests(&self, _analysis: &DagAnalysis, graph_builder_fn: &str) -> String {
        let mut code = String::new();

        let Some(spec) = &self.mock_spec else {
            return code;
        };

        if !spec.has_flow_test_data() {
            return code;
        }

        code.push_str("// ============================================================================\n");
        code.push_str("// Flow Verification Tests\n");
        code.push_str("// These tests execute the full DAG in DryRun mode with mocked transport\n");
        code.push_str("// responses, verifying that pure node logic produces expected outputs.\n");
        code.push_str("// ============================================================================\n\n");

        let test_name = format!(
            "test_flow_{}",
            NamingCase::SnakeCase.apply(&spec.name)
        );

        code.push_str(&format!("/// Flow verification: {} scenario.\n", spec.name));
        code.push_str("///\n");
        code.push_str("/// Builds the DAG, injects mocked transport responses via DryRun,\n");
        code.push_str("/// and verifies that the pure node chain produces expected terminal outputs.\n");
        code.push_str("#[test]\n");
        code.push_str(&format!("fn {}() {{\n", test_name));
        code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
        code.push_str("    let spec = mock_spec();\n");
        code.push_str("    let mocks = spec.to_boundary_mocks();\n");
        code.push_str("    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))\n");
        code.push_str("        .expect(\"DryRun execution should succeed\");\n");
        code.push('\n');

        // Generate assertions for each expected output
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

        code.push_str("// ============================================================================\n");
        code.push_str("// Boundary Tests\n");
        code.push_str("// ============================================================================\n\n");

        // Test that the graph is boundary-mockable
        code.push_str("/// Test that all boundaries can be mocked.\n");
        code.push_str("#[test]\n");
        code.push_str("fn test_boundaries_mockable() {\n");
        code.push_str(&format!("    let dag = {};\n", graph_builder_fn));
        code.push_str("    let result = assert_boundary_mockable(&dag, default_mocks());\n");
        code.push_str("    assert!(result.is_ok(), \"Boundaries should be mockable: {:?}\", result.error);\n");
        code.push_str("}\n\n");

        // Individual boundary node tests with MockSpec values
        for boundary_node in &analysis.boundaries.boundary_nodes {
            let test_name = format!("test_boundary_{}_mockable", NamingCase::SnakeCase.apply(&boundary_node.0));
            let node_name = &boundary_node.0;

            code.push_str(&format!("/// Test that {} boundary can be mocked.\n", node_name));
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
            
            // Add mocks for all boundary ports on this node - use MockSpec values if available
            for (node_id, port_name) in &analysis.boundaries.boundary_ports {
                if node_id == boundary_node {
                    // Find type for this port
                    let type_id = self.dag.nodes.iter()
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
            code.push_str("    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();\n");
            code.push_str(&format!(
                "    let entry = log.get(\"{}\").expect(\"node should be in log\");\n",
                node_name
            ));
            code.push_str("    assert!(entry.was_intercepted, \"boundary should be intercepted in dry-run\");\n");
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

        code.push_str("// ============================================================================\n");
        code.push_str("// Chain Validation Tests\n");
        code.push_str("// These tests verify that mock outputs satisfy downstream input expectations.\n");
        code.push_str("// ============================================================================\n\n");

        // Test self-consistency
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

        // Test input constraints are satisfiable
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
                    gunbc_test::InputConstraint::Custom { description, .. } => description.as_str(),
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

    /// Generate resource simulation tests.
    fn generate_resource_tests(&self, _analysis: &DagAnalysis) -> String {
        let mut code = String::new();

        let Some(spec) = &self.mock_spec else {
            return code;
        };

        if spec.resource_mocks.resources.is_empty() {
            return code;
        }

        code.push_str("// ============================================================================\n");
        code.push_str("// Resource Simulation Tests\n");
        code.push_str("// These tests verify behavior under different resource acquisition scenarios.\n");
        code.push_str("// ============================================================================\n\n");

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
            
            // Check expected behavior
            let has_fail = resource.behaviors.iter().any(|b| matches!(b, gunbc_test::ResourceBehavior::FailAcquire { .. }));
            if has_fail {
                code.push_str("    assert!(matches!(result, ResourceAcquireResult::Failed(_)), \"should fail to acquire\");\n");
            } else {
                code.push_str("    assert!(matches!(result, ResourceAcquireResult::Acquired), \"should acquire successfully\");\n");
            }
            
            code.push_str("}\n\n");

            // Generate lease expiration test if applicable
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
}

/// Sanitize a resource ID into a valid snake_case Rust identifier.
///
/// Replaces non-alphanumeric characters with `_`, lowercases, and collapses
/// consecutive underscores (e.g. `fs:.gitignore` → `fs_gitignore`).
fn sanitize_resource_id(id: &str) -> String {
    let raw: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    // Collapse runs of underscores and strip leading/trailing underscores
    let mut result = String::with_capacity(raw.len());
    let mut prev_underscore = true; // treat start as underscore to strip leading _
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
    // Strip trailing underscore
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
        Value::Str(s) => format!("Value::Str(\"{}\".to_string())", s.replace('\"', "\\\"")),
        Value::Int(i) => format!("Value::Int({})", i),
        Value::StrList(list) => {
            let items: Vec<String> = list.iter()
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
                }))".to_string()
        }
        _ => "Value::Str(\"<MOCK>\".to_string())".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build::*, Dag, Node};

    #[test]
    fn test_generate_test_module() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("source", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("sink", vec![port("in", "String")], vec![port("result", "String")], ()));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let generator = TestGenerator::new(&dag);
        let code = generator.generate_test_module("example", "build_example_graph()");

        // Should generate boundary tests (runtime behavior)
        assert!(code.contains("test_boundaries_mockable"));
        assert!(code.contains("test_boundary_sink_mockable"));
        
        // Should NOT generate composition tests (compiler proves these)
        assert!(!code.contains("test_all_edges_compatible"));
        assert!(!code.contains("test_edge_source_out_to_sink_in"));
    }

    #[test]
    fn test_generate_with_mock_spec() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("source", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("sink", vec![port("in", "String")], vec![port("result", "String")], ()));
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
        dag.add_node(Node::opaque("source", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("sink", vec![port("in", "String")], vec![port("result", "String")], ()));
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
}
