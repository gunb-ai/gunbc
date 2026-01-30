//! Gist SubDag builder.
//!
//! Wraps the gist tool as a SubDag node using WorkspaceOp.

use crate::WorkspaceOp;
use gunbc_gist::{build_gist_graph, GistGraphOp, GistOps};
use gunbc_ir::{Dag, Node, Port};

/// Convert a Node<GistGraphOp> to Node<WorkspaceOp>.
fn convert_gist_node(node: Node<GistGraphOp>) -> Node<WorkspaceOp> {
    Node {
        id: node.id,
        inputs: node.inputs,
        outputs: node.outputs,
        body: match node.body {
            gunbc_ir::NodeBody::Opaque(op) => {
                gunbc_ir::NodeBody::Opaque(convert_gist_op(op))
            }
            gunbc_ir::NodeBody::SubDag(dag) => {
                gunbc_ir::NodeBody::SubDag(convert_gist_dag(dag))
            }
        },
        requires_tools: node.requires_tools,
    }
}

/// Convert a GistGraphOp to WorkspaceOp.
fn convert_gist_op(op: GistGraphOp) -> WorkspaceOp {
    match op {
        // Gist-specific ops - wrap in Gist variant with a placeholder
        // Note: GistOps only has PrepareRequest and ParseGistResponse
        // The internal graph ops don't have direct WorkspaceOp equivalents
        GistGraphOp::PrepareListFiles
        | GistGraphOp::ParseListFiles
        | GistGraphOp::PrepareReadFiles
        | GistGraphOp::ParseReadFiles
        | GistGraphOp::PrepareReadFile
        | GistGraphOp::ParseReadFile
        | GistGraphOp::CollectFileContents
        | GistGraphOp::FilterByExtension { .. } => {
            // These are gist-internal ops - use ParseGistResponse as placeholder
            WorkspaceOp::Gist(GistOps::ParseGistResponse)
        }
        GistGraphOp::Markdown(_) => {
            // Markdown ops - use Gist as container
            WorkspaceOp::Gist(GistOps::ParseGistResponse)
        }
        GistGraphOp::Gist(gist_op) => WorkspaceOp::Gist(gist_op),
        GistGraphOp::Transport(t) => WorkspaceOp::Transport(t),
    }
}

/// Convert a Dag<GistGraphOp> to Dag<WorkspaceOp>.
fn convert_gist_dag(dag: Dag<GistGraphOp>) -> Dag<WorkspaceOp> {
    Dag {
        nodes: dag.nodes.into_iter().map(convert_gist_node).collect(),
        edges: dag.edges,
    }
}

/// Build the gist SubDag node.
///
/// This wraps the gist workflow as a `Node<WorkspaceOp>` that can be
/// composed into the Workspace DAG.
///
/// # Arguments
///
/// * `extensions` - File extensions to include (e.g., `vec![".rs", ".md"]`)
/// * `create_gist` - Whether to actually create the gist
///
/// # I/O Interface
///
/// Inputs:
/// - `repo_path`: String (optional) - Path to repository
///
/// Outputs:
/// - `markdown`: String - Generated markdown content
/// - `gist_url`: String (optional) - URL of created gist
pub fn build_gist_subdag(extensions: Vec<String>, create_gist: bool) -> Node<WorkspaceOp> {
    let original = build_gist_graph(extensions, create_gist).expect("Gist graph should build");
    let converted_dag = convert_gist_dag(original);

    Node::subdag(
        "gist",
        vec![Port::optional("repo_path", "String")],
        vec![
            Port::scalar("markdown", "String"),
            Port::optional("gist_url", "String"),
        ],
        converted_dag,
    )
}

/// Build a default gist SubDag for Rust files.
pub fn build_gist_rust_subdag() -> Node<WorkspaceOp> {
    build_gist_subdag(vec![".rs".to_string()], false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_gist_subdag_is_subdag() {
        let node = build_gist_rust_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "gist");
    }

    #[test]
    fn test_gist_subdag_has_nodes() {
        let node = build_gist_rust_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                // Gist should have multiple nodes
                assert!(dag.nodes.len() > 5);
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
