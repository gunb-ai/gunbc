//! Cloud resource SubDag builders.
//!
//! Provides SubDag nodes for cloud resource upserts (GCP, AWS) and
//! secret federation setup. Uses the UpsertBuilder pattern for
//! idempotent provisioning (Check → Create → Resolve).
//!
//! # Architecture
//!
//! The cloud upsert SubDag uses the UpsertBuilder pattern internally,
//! which creates a guarded SubDag:
//!
//! ```text
//! Cloud Upsert SubDag
//! ├── check    (check if cloud resource exists via CLI)
//! ├── create   (create resource if missing, guarded by check)
//! └── resolve  (verify resource exists and return handle)
//! ```
//!
//! Each phase (check/create/resolve) is an opaque CloudOp node.
//! The actual CLI execution is delegated to the transport boundary
//! at execution time.

use crate::workspace::ops::CloudOp;
use crate::workspace::WorkspaceOp;
use gunbc_ir::patterns::UpsertBuilder;
use gunbc_ir::Node;

/// Build a cloud resource upsert SubDag node.
///
/// This creates a SubDag that performs the Check → Create → Resolve
/// pattern for a single cloud resource, using the UpsertBuilder to
/// handle guard logic internally.
///
/// # I/O Interface
///
/// Inputs:
/// - `resource_spec`: String (JSON cloud resource description)
///
/// Outputs:
/// - `handle`: String (resource identifier/handle after upsert)
/// - `was_created`: Bool (true if resource was newly created)
pub fn build_cloud_upsert_subdag() -> Node<WorkspaceOp> {
    UpsertBuilder::new("cloud_upsert")
        .with_check(WorkspaceOp::Cloud(CloudOp::PrepareCheckResource))
        .with_create(WorkspaceOp::Cloud(CloudOp::PrepareCreateResource))
        .with_resolve(WorkspaceOp::Cloud(CloudOp::PrepareResolveResource))
        .with_input_port("resource_spec", "String")
        .with_output_port("handle", "String")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_cloud_upsert_subdag_is_subdag() {
        let node = build_cloud_upsert_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "cloud_upsert");
    }

    #[test]
    fn test_cloud_upsert_subdag_io() {
        let node = build_cloud_upsert_subdag();

        // One input: resource_spec
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.inputs[0].name.0, "resource_spec");

        // Two outputs: handle, was_created (from UpsertBuilder)
        assert_eq!(node.outputs.len(), 2);
        let output_names: Vec<_> = node.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"handle"));
        assert!(output_names.contains(&"was_created"));
    }

    #[test]
    fn test_cloud_upsert_subdag_structure() {
        let node = build_cloud_upsert_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                // UpsertBuilder creates 3 internal nodes: check, create, resolve
                assert_eq!(dag.nodes.len(), 3);

                // Verify node names
                let names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(names.contains(&"check"));
                assert!(names.contains(&"create"));
                assert!(names.contains(&"resolve"));

                // Verify the edge from check → create (for guard)
                assert_eq!(dag.edges.len(), 1);
                assert_eq!(dag.edges[0].from_node.0, "check");
                assert_eq!(dag.edges[0].from_port.0, "exists");
                assert_eq!(dag.edges[0].to_node.0, "create");
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
