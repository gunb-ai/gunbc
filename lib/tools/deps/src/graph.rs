//! Graph builder for the deps tool.
//!
//! Two graphs are provided:
//!
//! ## Install Graph (`build_deps_graph`)
//! All I/O happens through explicit `TransportOps::Execute` nodes:
//! - LoadManifest: PrepareLoadManifest -> Execute -> ParseManifest
//! - ExecuteInstalls: PrepareExecuteInstalls -> Execute -> ParseExecuteResult
//!
//! ## Generation Graph (`build_deps_generate_graph`)
//! Generates deps.toml from the tool registry:
//! - LoadToolRegistry -> RenderDepsToml -> PrepareFileWrite -> ExecuteTransport

use crate::manifest::DEFAULT_MANIFEST_FILENAME;
use crate::ops::DepsOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;
use std::collections::HashMap;

/// Union type for deps graph operations.
///
/// Following the gist pattern: all I/O through Transport(TransportOps::Execute) nodes.
#[derive(Debug, Clone)]
pub enum DepsGraphOp {
    /// Deps-specific operations (all PURE)
    Deps(DepsOp),
    /// Prepare file write (primitive - PURE)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for DepsGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DepsGraphOp::Deps(op) => op.execute(inputs),
            DepsGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            DepsGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the deps workflow.
pub fn deps_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs
        .with_input("manifest_path", "String", Cardinality::ZERO_OR_ONE)
        .with_input("platform", "String", Cardinality::ZERO_OR_ONE)
        // Outputs - boundary outputs from terminal nodes
        .with_output("dep_count", "Int", Cardinality::ONE)
        .with_output("dep_names", "List", Cardinality::ZERO_OR_MORE)
        .with_output("already_installed", "List", Cardinality::ZERO_OR_MORE)
        .with_output("needs_install", "List", Cardinality::ZERO_OR_MORE)
        .with_output("platform", "String", Cardinality::ONE)
        .with_output("executed", "Bool", Cardinality::ONE)
        .with_output("success", "Bool", Cardinality::ONE)
        .with_output("script", "String", Cardinality::ONE)
}

/// Build the deps graph with explicit transport nodes.
///
/// Pipeline:
/// ```text
/// PrepareLoadManifest -> Execute -> ParseManifest -> GenerateScripts -> PrepareExecuteInstalls -> Execute -> ParseExecuteResult
///                          ↑                                                                        ↑
///                      (boundary)                                                               (boundary)
/// ```
pub fn build_deps_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // LoadManifest chain: PrepareLoadManifest -> Execute -> ParseManifest
    // ========================================================================

    // Node: PrepareLoadManifest (PURE)
    let prepare_load = builder.add_root_node(Node::opaque(
        "prepare_load_manifest",
        vec![optional("manifest_path", "String")],
        vec![
            port("request", "TransportRequest"),
            port("manifest_path", "String"),
        ],
        DepsGraphOp::Deps(DepsOp::PrepareLoadManifest),
    ))?;

    // Node: Execute manifest load (BOUNDARY)
    let execute_load = builder.add_node_after(
        Node::opaque(
            "execute_load_manifest",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            DepsGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_load,
    )?;

    // Node: ParseManifest (PURE)
    let parse_manifest = builder.add_node_after(
        Node::opaque(
            "parse_manifest",
            vec![
                port("response", "TransportResponse"),
                port("manifest_path", "String"),
            ],
            vec![
                scalar("dep_count", "Int"),
                list("dep_names", "List"),
                scalar("manifest_path", "String"),
                scalar("manifest_content", "String"),  // Pass content to GenerateScripts
            ],
            DepsGraphOp::Deps(DepsOp::ParseManifest),
        ),
        &execute_load,
    )?;

    // ========================================================================
    // GenerateScripts (PURE domain logic)
    // Now receives manifest_content instead of loading from file
    // ========================================================================

    let generate_scripts = builder.add_node_after(
        Node::opaque(
            "generate_scripts",
            vec![
                scalar("manifest_content", "String"),   // Receives content, not path
                optional("platform", "String"),          // Platform acquired at boundary
            ],
            vec![
                scalar("install_script", "String"),
                list("already_installed", "List"),
                list("needs_install", "List"),
                scalar("platform", "String"),
            ],
            DepsGraphOp::Deps(DepsOp::GenerateScripts),
        ),
        &parse_manifest,
    )?;

    // ========================================================================
    // ExecuteInstalls chain: PrepareExecuteInstalls -> Execute -> ParseExecuteResult
    // ========================================================================

    // Node: PrepareExecuteInstalls (PURE)
    let prepare_execute = builder.add_node_after(
        Node::opaque(
            "prepare_execute_installs",
            vec![scalar("install_script", "String")],
            vec![
                port("request", "TransportRequest"),
                port("script", "String"),
            ],
            DepsGraphOp::Deps(DepsOp::PrepareExecuteInstalls),
        ),
        &generate_scripts,
    )?;

    // Node: Execute installs (BOUNDARY)
    let execute_installs = builder.add_node_after(
        Node::opaque(
            "execute_installs",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            DepsGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_execute,
    )?;

    // Node: ParseExecuteResult (PURE)
    let parse_result = builder.add_node_after(
        Node::opaque(
            "parse_execute_result",
            vec![
                port("response", "TransportResponse"),
                port("script", "String"),
            ],
            vec![
                scalar("executed", "Bool"),
                scalar("success", "Bool"),
                scalar("script", "String"),
                scalar("stdout", "String"),
                scalar("stderr", "String"),
            ],
            DepsGraphOp::Deps(DepsOp::ParseExecuteResult),
        ),
        &execute_installs,
    )?;

    // ========================================================================
    // Wire up the pipeline
    // ========================================================================

    // LoadManifest chain
    builder.add_edge(prepare_load.out("request"), execute_load.in_port("request"))?;
    builder.add_edge(execute_load.out("response"), parse_manifest.in_port("response"))?;
    builder.add_edge(prepare_load.out("manifest_path"), parse_manifest.in_port("manifest_path"))?;

    // To GenerateScripts (now receives manifest_content instead of path)
    builder.add_edge(parse_manifest.out("manifest_content"), generate_scripts.in_port("manifest_content"))?;

    // ExecuteInstalls chain
    builder.add_edge(generate_scripts.out("install_script"), prepare_execute.in_port("install_script"))?;
    builder.add_edge(prepare_execute.out("request"), execute_installs.in_port("request"))?;
    builder.add_edge(execute_installs.out("response"), parse_result.in_port("response"))?;
    builder.add_edge(prepare_execute.out("script"), parse_result.in_port("script"))?;

    Ok(builder.build())
}

// ============================================================================
// deps.toml Generation Graph
// ============================================================================

/// Get the declared signature for the deps generate workflow.
pub fn deps_generate_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("output_path", "String", Cardinality::ZERO_OR_ONE)
        // Outputs from execute_transport (boundary)
        .with_output("response", "TransportResponse", Cardinality::ONE)
        .with_output("written_path", "String", Cardinality::ONE)
        .with_output("content", "String", Cardinality::ONE)
        // Informational outputs from load_tool_registry
        .with_output("tool_count", "Int", Cardinality::ONE)
        .with_output("tool_names", "List", Cardinality::ONE_OR_MORE)
}

/// Build the deps generate graph.
///
/// Pipeline (follows makegen pattern):
/// ```text
/// LoadToolRegistry -> RenderDepsToml -> PrepareFileWrite -> ExecuteTransport
///      (deps)           (deps)           (primitive)         (transport)
///                                           PURE              BOUNDARY
/// ```
///
/// This graph generates deps.toml from the tool registry, owning the file's
/// generation in the same way makegen owns Makefile generation.
#[allow(clippy::result_large_err)]
pub fn build_deps_generate_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: LoadToolRegistry (deps-specific) - generation 0
    // No inputs (uses default registry)
    // Outputs: tool metadata (informational)
    let load_registry = builder.add_root_node(Node::opaque(
        "load_tool_registry",
        vec![],
        vec![
            scalar("tool_count", "Int"),
            non_empty_list("tool_names", "List"),
        ],
        DepsGraphOp::Deps(DepsOp::LoadToolRegistry),
    ))?;

    // Node: RenderDepsToml (deps-specific) - generation 1
    // No inputs (uses registry directly internally)
    // Output: generated deps.toml content
    let render_deps_toml = builder.add_node_after(
        Node::opaque(
            "render_deps_toml",
            vec![],
            vec![scalar("deps_toml_content", "String")],
            DepsGraphOp::Deps(DepsOp::RenderDepsToml),
        ),
        &load_registry,
    )?;

    // Node: PrepareFileWrite (primitive) - generation 2
    // Prepares the TransportRequest for file write
    // Note: output_path defaults to DEFAULT_MANIFEST_FILENAME in execution
    let prepare_write = builder.add_node_after(
        Node::opaque(
            "prepare_file_write",
            vec![
                scalar("content", "String"),
                optional("output_path", "String"),
            ],
            vec![port("request", "TransportRequest")],
            DepsGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &render_deps_toml,
    )?;

    // Node: ExecuteTransport (transport boundary) - generation 3
    // Actually writes the file
    let execute_transport = builder.add_node_after(
        Node::opaque(
            "execute_transport",
            vec![port("request", "TransportRequest")],
            vec![
                port("response", "TransportResponse"),
                port("written_path", "String"),
                port("content", "String"),
            ],
            DepsGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_write,
    )?;

    // ========================================================================
    // Wire up the pipeline
    // ========================================================================

    // RenderDepsToml -> PrepareFileWrite
    builder.add_edge(render_deps_toml.out("deps_toml_content"), prepare_write.in_port("content"))?;

    // PrepareFileWrite -> ExecuteTransport
    builder.add_edge(prepare_write.out("request"), execute_transport.in_port("request"))?;

    Ok(builder.build())
}

// Mockable implementation
use gunbc_test::Mockable;

impl Mockable for DepsGraphOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            DepsGraphOp::Deps(op) => op.mock_outputs(),
            DepsGraphOp::PrepareFileWrite(_) => {
                // Manual mock for PrepareFileWriteOp
                let mut out = HashMap::new();
                out.insert(
                    "request".to_string(),
                    Value::Request(gunbc_ir::transport::TransportRequest::File(
                        gunbc_ir::transport::FileRequest::write(
                            DEFAULT_MANIFEST_FILENAME,
                            "# mock deps.toml",
                        ),
                    )),
                );
                out
            }
            DepsGraphOp::Transport(_) => {
                let mut out = HashMap::new();
                out.insert(
                    "response".to_string(),
                    Value::Response(gunbc_ir::transport::TransportResponse::File(
                        gunbc_ir::transport::FileResponse {
                            path: "deps.toml".to_string(),
                            operation: gunbc_ir::transport::FileOp::Read,
                            success: true,
                            content: Some("[[dependency]]\nname = \"mock\"".to_string()),
                            exists: Some(true),
                            error: None,
                        },
                    )),
                );
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_deps_graph().expect("graph should build");
        // 7 nodes: prepare_load, execute_load, parse_manifest, generate_scripts,
        //          prepare_execute, execute_installs, parse_result
        assert_eq!(dag.nodes.len(), 7);
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_deps_graph().expect("graph should build");

        // Verify transport nodes exist
        assert!(dag.get_node(&"execute_load_manifest".into()).is_some());
        assert!(dag.get_node(&"execute_installs".into()).is_some());
    }

    #[test]
    fn test_graph_has_entrypoint() {
        let dag = build_deps_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // manifest_path is an entrypoint
        assert!(entrypoints.is_entrypoint_port(&"prepare_load_manifest".into(), &"manifest_path".into()));
    }

    #[test]
    fn test_pure_nodes_not_boundaries() {
        let dag = build_deps_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Pure nodes should not be boundaries (except terminal ones)
        assert!(!boundaries.is_boundary_node(&"prepare_load_manifest".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_execute_installs".into()));
    }
}
