//! Gist SubDag builder.
//!
//! Wraps the gist tool as a SubDag node using WorkspaceOp.

use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_gist::{build_gist_graph_with_config, GistGraphOp, GistMode};
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::Node;
use gunbc_lib_gist_ops::GistOps;

/// Convert a GistGraphOp to WorkspaceOp.
///
/// Internal gist ops (Git, FsEnv, Cloud, Pattern, Markdown, etc.) don't have
/// direct WorkspaceOp equivalents. They live inside SubDag nodes and are never
/// dispatched directly by the workspace executor — the SubDag executor handles
/// them using the original GistGraphOp type. The placeholder here is only used
/// for structural traversal (e.g., Mermaid rendering, node counting).
fn convert_gist_op(op: GistGraphOp) -> WorkspaceOp {
    match op {
        GistGraphOp::GistUpload(gunbc_lib_gist_ops::GistUploadOp::Gist(gist_op)) => {
            WorkspaceOp::Gist(gist_op)
        }
        GistGraphOp::Transport(t) => WorkspaceOp::Transport(t),
        // Internal ops — structurally present but never directly executed
        // in workspace context (SubDag executor dispatches them).
        _ => WorkspaceOp::Gist(GistOps::ParseGistResponse),
    }
}

/// Build the gist SubDag node.
///
/// This wraps the gist workflow as a `Node<WorkspaceOp>` that can be
/// composed into the Workspace DAG.
///
/// # Arguments
///
/// * `mode` - Content acquisition mode (snapshot or diff)
/// * `extensions` - File extensions to include (e.g., `vec![".rs", ".md"]`)
/// * `create_gist` - Whether to actually create the gist
///
/// # I/O Interface
///
/// Inputs:
/// - `repo_path`: String (required) - Path to repository
/// - `base_ref`: String (optional, diff mode only) - Base branch for diff
///
/// Outputs:
/// - `markdown`: String - Generated markdown content
/// - `gist_url`: String (optional) - URL of created gist
pub fn build_gist_subdag(
    mode: GistMode,
    extensions: Vec<String>,
    create_gist: bool,
) -> Node<WorkspaceOp> {
    build_gist_subdag_with_config(mode, extensions, create_gist, None)
}

/// Build a gist SubDag with explicit cloud config.
///
/// When `cloud_config` is `Some`, it is used directly; when `None`,
/// `graph_cloud_config()` provides centralized profile-aware resolution.
pub fn build_gist_subdag_with_config(
    mode: GistMode,
    extensions: Vec<String>,
    create_gist: bool,
    cloud_config: Option<CloudSecretConfig>,
) -> Node<WorkspaceOp> {
    let config = cloud_config.unwrap_or_else(gunbc_lib_cloud_ops::graph_cloud_config);
    let original = build_gist_graph_with_config(mode, extensions, create_gist, config)
        .expect("Gist graph should build");
    let converted_dag = convert_dag(original, &convert_gist_op);

    Node::subdag("gist", converted_dag)
}

/// Build a default gist SubDag for Rust files (snapshot mode).
pub fn build_gist_rust_subdag() -> Node<WorkspaceOp> {
    build_gist_subdag(GistMode::Snapshot, vec![".rs".to_string()], false)
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
                // list_files is still top-level; gist_upload is a SubDag wrapping execute_gist
                for node_id in ["list_files", "gist_upload"] {
                    assert!(
                        dag.get_node(&node_id.into()).is_some(),
                        "missing node: {}",
                        node_id
                    );
                }
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
