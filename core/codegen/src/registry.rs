//! Tool registry for CLI generation and DAG definition.
//!
//! Defines metadata for all tools that need CLI and DAG generation.
//!
//! # Architecture
//!
//! Tools can be defined declaratively using:
//! - `ToolDef`: Overall tool metadata and CLI configuration
//! - `NodeDef`: DAG nodes with operation type
//! - `EdgeDef`: Edges connecting node ports
//! - `PortDef`: Input/output port specifications
//!
//! Eventually, the entire graph.rs can be generated from these definitions.

use crate::cli_gen::{CliEntrypoint, ToolMeta};
use gunbc_ir::cargo;
use gunbc_ir::types::Cardinality;
use gunbc_test::{FermiCost, TestClass};

// ============================================================================
// DAG Definition Structures
// ============================================================================

/// Definition for a port on a node.
#[derive(Debug, Clone)]
pub struct PortDef {
    /// Port name
    pub name: String,
    /// Type identifier (e.g., "String", "Json")
    pub type_id: String,
    /// Cardinality (interval struct, not a string)
    pub cardinality: Cardinality,
}

impl PortDef {
    /// Create a scalar (One cardinality) port.
    pub fn scalar(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: Cardinality::ONE,
        }
    }

    /// Create an optional (ZeroOrOne cardinality) port.
    pub fn optional(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: Cardinality::ZERO_OR_ONE,
        }
    }

    /// Create a list (ZeroOrMore cardinality) port.
    pub fn list(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: Cardinality::ZERO_OR_MORE,
        }
    }

    /// Create a non-empty list (OneOrMore cardinality) port.
    pub fn list_nonempty(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: Cardinality::ONE_OR_MORE,
        }
    }
}

/// Definition for a DAG node.
#[derive(Debug, Clone)]
pub struct NodeDef {
    /// Node identifier
    pub id: String,
    /// Input ports
    pub inputs: Vec<PortDef>,
    /// Output ports
    pub outputs: Vec<PortDef>,
    /// Operation crate (e.g., "gunbc_primitives")
    pub op_crate: String,
    /// Operation type (e.g., "PrimitiveOp")
    pub op_type: String,
    /// Operation variant (e.g., "Parse(ParseOp::Toml)")
    pub op_variant: String,
}

impl NodeDef {
    /// Create a new node definition.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            inputs: vec![],
            outputs: vec![],
            op_crate: String::new(),
            op_type: String::new(),
            op_variant: String::new(),
        }
    }

    /// Add an input port.
    pub fn input(mut self, port: PortDef) -> Self {
        self.inputs.push(port);
        self
    }

    /// Add an output port.
    pub fn output(mut self, port: PortDef) -> Self {
        self.outputs.push(port);
        self
    }

    /// Set the operation (crate, type, variant).
    pub fn op(mut self, crate_name: &str, op_type: &str, variant: &str) -> Self {
        self.op_crate = crate_name.to_string();
        self.op_type = op_type.to_string();
        self.op_variant = variant.to_string();
        self
    }

    /// Shorthand for primitive operation.
    pub fn primitive(mut self, variant: &str) -> Self {
        self.op_crate = "gunbc_primitives".to_string();
        self.op_type = "PrimitiveOp".to_string();
        self.op_variant = variant.to_string();
        self
    }
}

/// Definition for an edge connecting two nodes.
#[derive(Debug, Clone)]
pub struct EdgeDef {
    /// Source node ID
    pub from_node: String,
    /// Source port name
    pub from_port: String,
    /// Destination node ID
    pub to_node: String,
    /// Destination port name
    pub to_port: String,
}

impl EdgeDef {
    /// Create a new edge definition.
    pub fn new(from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Self {
        Self {
            from_node: from_node.to_string(),
            from_port: from_port.to_string(),
            to_node: to_node.to_string(),
            to_port: to_port.to_string(),
        }
    }
}

/// Definition for a complete DAG.
#[derive(Debug, Clone, Default)]
pub struct DagDef {
    /// Nodes in the DAG
    pub nodes: Vec<NodeDef>,
    /// Edges connecting nodes
    pub edges: Vec<EdgeDef>,
}

impl DagDef {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the DAG.
    pub fn node(mut self, node: NodeDef) -> Self {
        self.nodes.push(node);
        self
    }

    /// Add an edge to the DAG.
    pub fn edge(mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Self {
        self.edges
            .push(EdgeDef::new(from_node, from_port, to_node, to_port));
        self
    }
}

// ============================================================================
// Tool Definition
// ============================================================================

/// A tool that needs CLI generation.
pub struct ToolDef {
    pub meta: ToolMeta,
    pub entrypoints: Vec<CliEntrypoint>,
    /// Custom import line (if different from default pattern)
    pub custom_import: Option<String>,
    /// Output artifacts produced by this tool (for clean/rollback)
    pub outputs: Vec<String>,
    /// Declarative DAG definition (optional - for generated graphs)
    pub dag: Option<DagDef>,
    /// Cargo invocation for running this tool.
    /// When set, the tool gets a Makefile target automatically.
    /// When None, the tool has no runnable binary (e.g., library-only or not wired up yet).
    pub invocation: Option<cargo::CargoInvocation>,
}

/// Configuration for test generation.
#[derive(Debug, Clone)]
pub struct TestgenTargetDef {
    /// Short identifier (e.g., "bootstrap", "llm-openai")
    pub name: String,
    /// Output path for generated tests (relative to workspace)
    pub output_path: String,
    /// Module name for the generated test module
    pub module_name: String,
    /// MockSpec function path (e.g., "crate::graph_mock::my_mock_spec")
    pub mock_spec_path: String,
    /// DAG builder call expression (e.g., "crate::build_graph().unwrap()")
    pub dag_builder_call: String,
    /// Signature function path (e.g., "crate::makegen_signature()")
    pub signature_path: Option<String>,
    /// Enable boundary tests
    pub boundary_tests: bool,
    /// Enable chain tests
    pub chain_tests: bool,
    /// Enable flow tests
    pub flow_tests: bool,
    /// Max window size for windowed tests (None = no limit)
    pub window_max_nodes: Option<usize>,
    /// Test class override (unit/hermetic/integration)
    pub test_class: Option<TestClass>,
    /// Fermi cost override
    pub fermi_cost: Option<FermiCost>,
    /// External requirements override
    pub requires: Option<Vec<String>>,
    /// Required secrets override (env vars)
    pub secrets: Option<Vec<String>>,
    /// Tool name for CLI contract test generation. When set, entrypoints
    /// are looked up from `all_tools()` and a CLI contract test is emitted
    /// alongside the DAG tests.
    pub tool_name: Option<String>,
}

impl TestgenTargetDef {
    /// Create a new testgen target definition.
    pub fn new(name: &str, output_path: &str, module_name: &str) -> Self {
        Self {
            name: name.to_string(),
            output_path: output_path.to_string(),
            module_name: module_name.to_string(),
            mock_spec_path: String::new(),
            dag_builder_call: String::new(),
            signature_path: None,
            boundary_tests: true,
            chain_tests: true,
            flow_tests: false,
            window_max_nodes: Some(5),
            test_class: None,
            fermi_cost: None,
            requires: None,
            secrets: None,
            tool_name: None,
        }
    }

    /// Set the MockSpec function path.
    pub fn mock_spec(mut self, path: &str) -> Self {
        self.mock_spec_path = path.to_string();
        self
    }

    /// Set the DAG builder call expression.
    pub fn dag_builder(mut self, call: &str) -> Self {
        self.dag_builder_call = call.to_string();
        self
    }

    /// Set the signature function path.
    pub fn signature(mut self, path: &str) -> Self {
        self.signature_path = Some(path.to_string());
        self
    }

    /// Enable flow tests (and disable boundary/chain tests).
    pub fn flow_tests(mut self) -> Self {
        self.boundary_tests = false;
        self.chain_tests = false;
        self.flow_tests = true;
        self
    }

    /// Set the max window size for windowed tests.
    pub fn window_max_nodes(mut self, max: usize) -> Self {
        self.window_max_nodes = Some(max);
        self
    }

    /// Disable boundary tests.
    pub fn no_boundary_tests(mut self) -> Self {
        self.boundary_tests = false;
        self
    }
}

impl ToolDef {
    pub fn new(
        crate_name: &str,
        tool_name: &str,
        description: &str,
        graph_builder_call: &str,
        graph_builder_args: &str,
    ) -> Self {
        Self {
            meta: ToolMeta {
                crate_name: crate_name.to_string(),
                tool_name: tool_name.to_string(),
                description: description.to_string(),
                graph_builder_call: graph_builder_call.to_string(),
                graph_builder_args: graph_builder_args.to_string(),
                returns_result: false,
                success_port: None,
                enable_step_mode: false,
                mock_spec_call: None,
            },
            entrypoints: vec![],
            custom_import: None,
            outputs: vec![],
            dag: None,
            invocation: None,
        }
    }

    /// Mark that this tool's graph builder returns Result<Dag, BuilderError>.
    pub fn returns_result(mut self) -> Self {
        self.meta.returns_result = true;
        self
    }

    /// Set the output port to check for success.
    /// If this port is false, the CLI exits with code 1.
    pub fn check_success(mut self, port_name: &str) -> Self {
        self.meta.success_port = Some(port_name.to_string());
        self
    }

    /// Enable step mode for this tool.
    /// This generates a CLI with `step <node>` and `list-steps` subcommands
    /// for better CI visibility (each DAG node can be a separate CI step).
    pub fn enable_step_mode(mut self) -> Self {
        self.meta.enable_step_mode = true;
        self
    }

    /// Set the cargo invocation for running this tool.
    ///
    /// When set, the makegen registry will automatically create a Makefile
    /// target for this tool. This is the canonical way to say "this tool
    /// has a runnable binary invoked via cargo".
    pub fn invocation(mut self, inv: cargo::CargoInvocation) -> Self {
        self.invocation = Some(inv);
        self
    }

    /// Set a declarative DAG definition (enables graph generation).
    pub fn dag(mut self, dag: DagDef) -> Self {
        self.dag = Some(dag);
        self
    }

    /// Check if this tool has a declarative DAG definition.
    pub fn has_dag(&self) -> bool {
        self.dag.is_some()
    }

    /// Set a custom import line.
    pub fn import(mut self, import_line: &str) -> Self {
        self.custom_import = Some(import_line.to_string());
        self
    }

    /// Add an output artifact (file or directory produced by this tool).
    pub fn output(mut self, path: &str) -> Self {
        self.outputs.push(path.to_string());
        self
    }

    pub fn entrypoint(mut self, ep: CliEntrypoint) -> Self {
        self.entrypoints.push(ep);
        self
    }

    /// Set entrypoints from a JSON string (matching the format in `#[tool_target]` annotations).
    ///
    /// This is the preferred way to define entrypoints — the JSON is the single
    /// source of truth, shared between `all_tools()` and the `#[tool_target]` annotation.
    pub fn entrypoints_json(mut self, json: &str) -> Self {
        self.entrypoints = CliEntrypoint::from_json(json);
        self
    }

    /// Set the mock_spec_call expression for dry-run boundary mocking.
    ///
    /// When set, generated CLIs call this expression to get a MockSpec
    /// instead of using inline boundary values. This makes MockSpec the
    /// single source of truth for boundary mock values.
    pub fn mock_spec_call(mut self, call: &str) -> Self {
        self.meta.mock_spec_call = Some(call.to_string());
        self
    }

}

// ============================================================================
// Entrypoint JSON constants (shared with #[tool_target] annotations)
// ============================================================================

/// Entrypoints for gist snapshot (repo_path, extensions, public).
const GIST_ENTRYPOINTS: &str = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#;

/// Entrypoints for gist-diff (adds base_ref).
const GIST_DIFF_ENTRYPOINTS: &str = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"base_ref","type_id":"String","short":"b","default":"main","help":"Base branch for diff","make_var":"BASE"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#;

/// Entrypoints for gist-recent (same as gist snapshot).
const GIST_RECENT_ENTRYPOINTS: &str = GIST_ENTRYPOINTS;

/// Entrypoints for makegen (path).
const MAKEGEN_ENTRYPOINTS: &str = r#"[{"port_name":"path","type_id":"String","short":"o","default":"Makefile","help":"Output Makefile path","make_var":"OUTPUT"}]"#;

/// Entrypoints for deps (manifest_path).
const DEPS_ENTRYPOINTS: &str = r#"[{"port_name":"manifest_path","type_id":"String","short":"m","help":"Path to deps.toml manifest","make_var":"MANIFEST"}]"#;

/// Entrypoints for review (repo_path, base_ref).
const REVIEW_ENTRYPOINTS: &str = r#"[{"port_name":"repo_path","type_id":"String","short":"r","help":"Repository path to diff","make_var":"REPO"},{"port_name":"base_ref","type_id":"String","short":"b","default":"main","help":"Base branch for diff (default: main)"}]"#;

/// Get all tool definitions for CLI generation.
///
/// Entrypoints are defined as JSON constants that are shared with `#[tool_target]`
/// annotations. The `tool_registrations_match_all_tools` test validates that
/// these match the annotations at compile time.
pub fn all_tools() -> Vec<ToolDef> {
    let tools = vec![
        // gunbc-gist (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("gist"),
            "gist",
            "Create a GitHub gist from code files",
            "build_gist_graph",
            "GistMode::Snapshot, extensions.clone(), public",
        )
        .returns_result()
        .mock_spec_call("gunbc_gist::graph_mock::gist_snapshot_mock_spec()")
        .invocation(cargo::CargoInvocation::composed("gist", "gist"))
        .import("use gunbc_gist::{build_gist_graph, GistMode};")
        .entrypoints_json(GIST_ENTRYPOINTS),

        // gunbc-gist-diff (diff mode variant - same package, different binary)
        ToolDef::new(
            &cargo::name("gist"),
            "gist-diff",
            "Create a GitHub gist from branch diff",
            "build_gist_graph",
            "GistMode::Diff { base_ref: base_ref.clone() }, extensions.clone(), public",
        )
        .returns_result()
        .mock_spec_call("gunbc_gist::graph_mock::gist_diff_mock_spec()")
        .invocation(cargo::CargoInvocation::composed("gist-diff", "gist"))
        .import("use gunbc_gist::{build_gist_graph, GistMode};")
        .entrypoints_json(GIST_DIFF_ENTRYPOINTS),

        // gunbc-gist-recent (recent mode variant - same package, different binary)
        ToolDef::new(
            &cargo::name("gist"),
            "gist-recent",
            "Create a GitHub gist from recent changes (last 7 days)",
            "build_gist_graph",
            "GistMode::Recent, extensions.clone(), public",
        )
        .returns_result()
        .mock_spec_call("gunbc_gist::graph_mock::gist_recent_mock_spec()")
        .invocation(cargo::CargoInvocation::composed("gist-recent", "gist"))
        .import("use gunbc_gist::{build_gist_graph, GistMode};")
        .entrypoints_json(GIST_RECENT_ENTRYPOINTS),

        // gunbc-makegen (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("makegen"),
            "makegen",
            "Generate Makefile from tool registry",
            "build_makegen_graph",
            "",
        )
        .returns_result()
        .mock_spec_call("gunbc_dag::makegen::graph_mock::makegen_mock_spec()")
        .invocation(cargo::CargoInvocation::composed("makegen", "dag"))
        .import("use gunbc_makegen::build_makegen_graph;")
        // Declarative DAG definition (POC for graph generation)
        .dag(makegen_dag())
        .entrypoints_json(MAKEGEN_ENTRYPOINTS),

        // gunbc-deps (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("deps"),
            "deps",
            "Install tool dependencies",
            "build_deps_graph",
            "",
        )
        .returns_result()
        .mock_spec_call("gunbc_deps::graph_mock::deps_mock_spec()")
        .invocation(cargo::CargoInvocation::standalone("deps"))
        .import("use gunbc_deps::build_deps_graph;")
        .entrypoints_json(DEPS_ENTRYPOINTS),

        // gunbc-review (diff review using LLM)
        //
        // Provider, model, and criteria are pipeline config (baked into DAG via
        // LoadPipelineConfig node), NOT CLI flags.
        ToolDef::new(
            "gunbc-lib-review",
            "review",
            "Review code changes using LLM analysis",
            "build_diff_review_graph",
            "",
        )
        .mock_spec_call("gunbc_lib_review::graph_mock::diff_review_mock_spec()")
        .import("use gunbc_lib_review::graph::build_diff_review_graph;")
        .entrypoints_json(REVIEW_ENTRYPOINTS),

        // NOTE: gunbc-ci is NOT in this registry.
        // It has a handwritten main.rs because it's the bootstrap tool that
        // runs codegen for other tools. It cannot depend on generated code.

        // gunbc-bootstrap (uses DagBuilder - returns Result, no entrypoints)
        ToolDef::new(
            &cargo::name("bootstrap"),
            "bootstrap",
            "Generate Makefile and .gitignore",
            "build_bootstrap_graph",
            "",
        )
        .returns_result()
        .mock_spec_call("gunbc_dag::bootstrap::graph_mock::bootstrap_mock_spec()")
        .invocation(cargo::CargoInvocation::composed("bootstrap", "dag"))
        .import("use gunbc_bootstrap::build_bootstrap_graph;"),

        // gunbc-clippy (sub-DAG, no standalone CLI — used as a component in CI)
        ToolDef::new(
            "gunbc-clippy",
            "clippy",
            "Run clippy via upsert (check → install → run)",
            "build_clippy_graph_lint_all",
            "",
        )
        .returns_result()
        .mock_spec_call("gunbc_clippy::graph_mock::clippy_mock_spec()")
        .import("use gunbc_clippy::build_clippy_graph_lint_all;"),
    ];

    tools
}

/// Core build system artifacts (not tool-specific).
pub fn core_outputs() -> Vec<&'static str> {
    vec![
        "target/", // cargo build output
        "bin",     // symlink to target/release
    ]
}

/// Get all cleanable artifacts from tools and core.
pub fn all_cleanable_outputs() -> Vec<String> {
    let mut outputs: Vec<String> = core_outputs().into_iter().map(|s| s.to_string()).collect();

    for tool in all_tools() {
        outputs.extend(tool.outputs);
    }

    // Deduplicate
    outputs.sort();
    outputs.dedup();
    outputs
}

// ============================================================================
// Declarative DAG Definitions
// ============================================================================

/// Declarative definition of the makegen DAG.
///
/// Pipeline:
/// ```text
/// LoadRegistry -> RenderMakefile -> WriteMakefile
///                                        ↓
///                                   (boundary)
/// ```
fn makegen_dag() -> DagDef {
    DagDef::new()
        // Node: LoadRegistry - no inputs, outputs tool metadata
        .node(
            NodeDef::new("load_registry")
                .output(PortDef::scalar("tool_count", "Int"))
                .output(PortDef::list_nonempty("tool_names", "String"))
                .output(PortDef::scalar("registry", "Json"))
                .op("", "MakegenOp", "MakegenOp::LoadRegistry"),
        )
        // Node: RenderMakefile - registry input, content output
        .node(
            NodeDef::new("render_makefile")
                .input(PortDef::scalar("registry", "Json"))
                .output(PortDef::scalar("makefile_content", "String"))
                .op("", "MakegenOp", "MakegenOp::RenderMakefile"),
        )
        // Node: WriteMakefile - BOUNDARY (world write)
        .node(
            NodeDef::new("write_makefile")
                .input(PortDef::scalar("makefile_content", "String"))
                .input(PortDef::scalar("path", "String"))
                .output(PortDef::scalar("written_path", "String"))
                .output(PortDef::scalar("content", "String"))
                .output(PortDef::scalar("changed", "Bool"))
                .op("", "MakegenOp", "MakegenOp::WriteMakefile"),
        )
        // Wire up the pipeline
        .edge("load_registry", "registry", "render_makefile", "registry")
        .edge(
            "render_makefile",
            "makefile_content",
            "write_makefile",
            "makefile_content",
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gist_repo_path_default_is_dot() {
        let tools = all_tools();
        let tool_names = ["gist", "gist-diff", "gist-recent"];
        for name in tool_names {
            let tool = tools
                .iter()
                .find(|tool| tool.meta.tool_name == name)
                .unwrap_or_else(|| panic!("missing tool definition for {}", name));
            let repo_entry = tool
                .entrypoints
                .iter()
                .find(|entry| entry.port_name == "repo_path")
                .unwrap_or_else(|| panic!("missing repo_path entrypoint for {}", name));
            assert_eq!(
                repo_entry.default_value.as_deref(),
                Some("."),
                "repo_path default should be \".\" for {}",
                name
            );
        }
    }
}
