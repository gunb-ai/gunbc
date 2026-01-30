//! Graph builder for the deps tool.
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes:
//! - LoadManifest: PrepareLoadManifest -> Execute -> ParseManifest
//! - ExecuteInstalls: PrepareExecuteInstalls -> Execute -> ParseExecuteResult

use crate::ops::DepsOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use std::collections::HashMap;

/// Union type for deps graph operations.
///
/// Following the gist pattern: all I/O through Transport(TransportOps::Execute) nodes.
#[derive(Debug, Clone)]
pub enum DepsGraphOp {
    /// Deps-specific operations (all PURE)
    Deps(DepsOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for DepsGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DepsGraphOp::Deps(op) => op.execute(inputs),
            DepsGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the deps workflow.
pub fn deps_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs
        .with_input("manifest_path", "String", Cardinality::ZeroOrOne)
        // Outputs - boundary outputs from terminal nodes
        .with_output("dep_count", "Int", Cardinality::One)
        .with_output("dep_names", "StrList", Cardinality::ZeroOrMore)
        .with_output("already_installed", "StrList", Cardinality::ZeroOrMore)
        .with_output("needs_install", "StrList", Cardinality::ZeroOrMore)
        .with_output("platform", "String", Cardinality::One)
        .with_output("executed", "Bool", Cardinality::One)
        .with_output("success", "Bool", Cardinality::One)
        .with_output("script", "String", Cardinality::One)
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
                list("dep_names", "StrList"),
                scalar("manifest_path", "String"),
            ],
            DepsGraphOp::Deps(DepsOp::ParseManifest),
        ),
        &execute_load,
    )?;

    // ========================================================================
    // GenerateScripts (PURE domain logic)
    // ========================================================================

    let generate_scripts = builder.add_node_after(
        Node::opaque(
            "generate_scripts",
            vec![scalar("manifest_path", "String")],
            vec![
                scalar("install_script", "String"),
                list("already_installed", "StrList"),
                list("needs_install", "StrList"),
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

    // To GenerateScripts
    builder.add_edge(parse_manifest.out("manifest_path"), generate_scripts.in_port("manifest_path"))?;

    // ExecuteInstalls chain
    builder.add_edge(generate_scripts.out("install_script"), prepare_execute.in_port("install_script"))?;
    builder.add_edge(prepare_execute.out("request"), execute_installs.in_port("request"))?;
    builder.add_edge(execute_installs.out("response"), parse_result.in_port("response"))?;
    builder.add_edge(prepare_execute.out("script"), parse_result.in_port("script"))?;

    Ok(builder.build())
}

// Mockable implementation
use gunbc_test::Mockable;

impl Mockable for DepsGraphOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            DepsGraphOp::Deps(op) => op.mock_outputs(),
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
