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

use crate::cli_gen::{CliBoundary, CliEntrypoint, ToolMeta};
use gunbc_ir::cargo;

// ============================================================================
// DAG Definition Structures
// ============================================================================

/// Definition for a port on a node.
#[derive(Debug, Clone)]
pub struct PortDef {
    /// Port name
    pub name: String,
    /// Type identifier (e.g., "String", "StrList", "Json")
    pub type_id: String,
    /// Cardinality ("One", "ZeroOrOne", "ZeroOrMore", "OneOrMore")
    pub cardinality: String,
}

impl PortDef {
    /// Create a scalar (One cardinality) port.
    pub fn scalar(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: "One".to_string(),
        }
    }

    /// Create an optional (ZeroOrOne cardinality) port.
    pub fn optional(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: "ZeroOrOne".to_string(),
        }
    }

    /// Create a list (ZeroOrMore cardinality) port.
    pub fn list(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: "ZeroOrMore".to_string(),
        }
    }

    /// Create a non-empty list (OneOrMore cardinality) port.
    pub fn list_nonempty(name: &str, type_id: &str) -> Self {
        Self {
            name: name.to_string(),
            type_id: type_id.to_string(),
            cardinality: "OneOrMore".to_string(),
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
        self.edges.push(EdgeDef::new(from_node, from_port, to_node, to_port));
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
    pub boundaries: Vec<CliBoundary>,
    /// Custom import line (if different from default pattern)
    pub custom_import: Option<String>,
    /// Output artifacts produced by this tool (for clean/rollback)
    pub outputs: Vec<String>,
    /// Declarative DAG definition (optional - for generated graphs)
    pub dag: Option<DagDef>,
    /// Testgen configuration (if this tool should have generated tests)
    pub testgen: Option<TestgenTargetDef>,
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
    /// Enable boundary tests
    pub boundary_tests: bool,
    /// Enable chain tests
    pub chain_tests: bool,
    /// Enable flow tests
    pub flow_tests: bool,
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
            boundary_tests: true,
            chain_tests: true,
            flow_tests: false,
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

    /// Enable flow tests (and disable boundary/chain tests).
    pub fn flow_tests(mut self) -> Self {
        self.boundary_tests = false;
        self.chain_tests = false;
        self.flow_tests = true;
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
        graph_builder: &str,
        graph_builder_args: &str,
    ) -> Self {
        Self {
            meta: ToolMeta {
                crate_name: crate_name.to_string(),
                tool_name: tool_name.to_string(),
                description: description.to_string(),
                graph_builder: graph_builder.to_string(),
                graph_builder_args: graph_builder_args.to_string(),
                returns_result: false,
                success_port: None,
                enable_step_mode: false,
            },
            entrypoints: vec![],
            boundaries: vec![],
            custom_import: None,
            outputs: vec![],
            dag: None,
            testgen: None,
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

    pub fn boundary(mut self, node_id: &str, mock_outputs: Vec<(&str, &str)>) -> Self {
        self.boundaries.push(CliBoundary {
            node_id: node_id.to_string(),
            mock_outputs: mock_outputs
                .into_iter()
                .map(|(p, v)| (p.to_string(), v.to_string()))
                .collect(),
        });
        self
    }

    /// Configure test generation for this tool.
    pub fn testgen(mut self, target: TestgenTargetDef) -> Self {
        self.testgen = Some(target);
        self
    }

    /// Check if this tool has testgen configuration.
    pub fn has_testgen(&self) -> bool {
        self.testgen.is_some()
    }
}

/// Get all testgen targets from registered tools.
///
/// Returns a list of (tool_name, TestgenTargetDef) for tools that have
/// testgen configuration. This enables auto-discovery of which tools
/// need test generation without maintaining a separate list.
pub fn all_testgen_targets() -> Vec<(&'static str, TestgenTargetDef)> {
    all_tools()
        .into_iter()
        .filter_map(|tool| {
            tool.testgen.map(|t| {
                // Leak the tool name to get a static lifetime
                // This is safe since all_tools() returns static data
                let name: &'static str = Box::leak(tool.meta.tool_name.into_boxed_str());
                (name, t)
            })
        })
        .collect()
}

/// Get all tool definitions for CLI generation.
pub fn all_tools() -> Vec<ToolDef> {
    vec![
        // gunbc-gist (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("gist"),
            "gist",
            "Create a GitHub gist from code files",
            "build_gist_graph",
            "GistMode::Snapshot, extensions.clone(), public",
        )
        .returns_result()
        .import("use gunbc_gist::{build_gist_graph, GistMode};")
        .entrypoint(
            CliEntrypoint::new("repo_path", "String")
                .short('r')
                .default(".")
                .help("Repository path to scan"),
        )
        .entrypoint(
            CliEntrypoint::new("extensions", "StrList")
                .short('e')
                .help("File extensions to include (can be repeated)"),
        )
        .entrypoint(
            CliEntrypoint::new("public", "Bool")
                .short('p')
                .help("Make gist public"),
        )
        .boundary(
            "execute_list_files",
            vec![
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::Shell(gunbc_ir::transport::ShellResponse { exit_code: 0, stdout: \"src/main.rs\\n\".to_string(), stderr: String::new() }))"),
            ],
        )
        .boundary(
            "execute_read_files",
            vec![
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::Shell(gunbc_ir::transport::ShellResponse { exit_code: 0, stdout: \"===GUNBC_FILE:src/main.rs===\\nfn main() {}\\n\".to_string(), stderr: String::new() }))"),
            ],
        )
        .boundary(
            "execute_gist",
            vec![
                ("url", "Value::Str(\"<DRY-RUN: gist URL>\".to_string())"),
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::Shell(gunbc_ir::transport::ShellResponse { exit_code: 0, stdout: String::new(), stderr: String::new() }))"),
            ],
        ), // gist creates a remote gist, no local output

        // gunbc-gist-diff (diff mode variant - same package, different binary)
        ToolDef::new(
            &cargo::name("gist"),
            "gist-diff",
            "Create a GitHub gist from branch diff",
            "build_gist_graph",
            "GistMode::Diff { base_ref: base_ref.clone() }, extensions.clone(), public",
        )
        .returns_result()
        .import("use gunbc_gist::{build_gist_graph, GistMode};")
        .entrypoint(
            CliEntrypoint::new("repo_path", "String")
                .short('r')
                .default(".")
                .help("Repository path to scan"),
        )
        .entrypoint(
            CliEntrypoint::new("base_ref", "String")
                .short('b')
                .default("main")
                .help("Base branch for diff"),
        )
        .entrypoint(
            CliEntrypoint::new("extensions", "StrList")
                .short('e')
                .help("File extensions to include (can be repeated)"),
        )
        .entrypoint(
            CliEntrypoint::new("public", "Bool")
                .short('p')
                .help("Make gist public"),
        )
        .boundary(
            "execute_diff",
            vec![
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::Shell(gunbc_ir::transport::ShellResponse { exit_code: 0, stdout: \"diff --git a/src/main.rs b/src/main.rs\\n--- a/src/main.rs\\n+++ b/src/main.rs\\n@@ -1 +1,2 @@\\n fn main() {}\\n+// changed\\n\".to_string(), stderr: String::new() }))"),
            ],
        )
        .boundary(
            "execute_gist",
            vec![
                ("url", "Value::Str(\"<DRY-RUN: gist URL>\".to_string())"),
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::Shell(gunbc_ir::transport::ShellResponse { exit_code: 0, stdout: String::new(), stderr: String::new() }))"),
            ],
        ), // gist-diff creates a remote gist from branch diff

        // gunbc-buck2 (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("buck2"),
            "buck2",
            "Generate BUCK file from Cargo.toml",
            "build_buck2_graph",
            "",
        )
        .returns_result()
        .entrypoint(
            CliEntrypoint::new("cargo_toml_path", "String")
                .short('i')
                .default("Cargo.toml")
                .help("Path to Cargo.toml"),
        )
        .entrypoint(
            CliEntrypoint::new("output_path", "String")
                .short('o')
                .default("BUCK")
                .help("Output BUCK file path"),
        )
        .output("BUCK")  // default output
        .boundary(
            "execute_transport",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("content", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::File(gunbc_ir::transport::FileResponse::written(\"BUCK\")))"),
            ],
        ),

        // gunbc-makegen (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("makegen"),
            "makegen",
            "Generate Makefile from tool registry",
            "build_makegen_graph",
            "",
        )
        .returns_result()
        .import("use gunbc_makegen::build_makegen_graph;")
        // Declarative DAG definition (POC for graph generation)
        .dag(makegen_dag())
        .entrypoint(
            CliEntrypoint::new("output_path", "String")
                .short('o')
                .default("Makefile")
                .help("Output Makefile path"),
        )
        .boundary(
            "write_makefile",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("content", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("changed", "Value::Bool(true)"),
            ],
        ),

        // gunbc-deps (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("deps"),
            "deps",
            "Install tool dependencies",
            "build_deps_graph",
            "",
        )
        .returns_result()
        .import("use gunbc_deps::build_deps_graph;")
        .entrypoint(
            CliEntrypoint::new("manifest_path", "String")
                .short('m')
                .default("deps.toml")
                .help("Path to deps.toml manifest"),
        )
        .boundary(
            "execute_installs",
            vec![
                ("executed", "Value::Bool(true)"),
                ("script", "Value::Str(\"<DRY-RUN>\".to_string())"),
            ],
        ),

        // NOTE: gunbc-ci is NOT in this registry.
        // It has a handwritten main.rs because it's the bootstrap tool that
        // runs codegen for other tools. It cannot depend on generated code.
        // See lib/tools/ci/src/main.rs

        // gunbc-bootstrap (uses DagBuilder - returns Result)
        ToolDef::new(
            &cargo::name("bootstrap"),
            "bootstrap",
            "Generate Makefile and .gitignore",
            "build_bootstrap_graph",
            "",
        )
        .returns_result()
        .import("use gunbc_bootstrap::build_bootstrap_graph;")
        .boundary(
            "write_makefile",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
            ],
        )
        .boundary(
            "write_gitignore",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
            ],
        ),
        // NOTE: prep tool has been removed - its functionality is now
        // consolidated into CI's Prep stage, using BuildConfig from makegen
    ]
}

/// Core build system artifacts (not tool-specific).
pub fn core_outputs() -> Vec<&'static str> {
    vec![
        "buck-out/",      // codegen output directory
        "target/",        // cargo build output
        "bin",            // symlink to target/release
    ]
}

/// Get all cleanable artifacts from tools and core.
pub fn all_cleanable_outputs() -> Vec<String> {
    let mut outputs: Vec<String> = core_outputs()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    
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
                .output(PortDef::list_nonempty("tool_names", "StrList"))
                .output(PortDef::scalar("registry", "Json"))
                .op("", "MakegenOp", "MakegenOp::LoadRegistry")
        )
        // Node: RenderMakefile - registry input, content output
        .node(
            NodeDef::new("render_makefile")
                .input(PortDef::scalar("registry", "Json"))
                .output(PortDef::scalar("makefile_content", "String"))
                .op("", "MakegenOp", "MakegenOp::RenderMakefile")
        )
        // Node: WriteMakefile - BOUNDARY (world write)
        .node(
            NodeDef::new("write_makefile")
                .input(PortDef::scalar("makefile_content", "String"))
                .input(PortDef::optional("output_path", "String"))
                .output(PortDef::scalar("written_path", "String"))
                .output(PortDef::scalar("content", "String"))
                .output(PortDef::scalar("changed", "Bool"))
                .op("", "MakegenOp", "MakegenOp::WriteMakefile")
        )
        // Wire up the pipeline
        .edge("load_registry", "registry", "render_makefile", "registry")
        .edge("render_makefile", "makefile_content", "write_makefile", "makefile_content")
}

