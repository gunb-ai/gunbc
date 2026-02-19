//! Downstream coordination/readiness analysis (WF4).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use gunbc_ir::{NodeId, PortName, RESOURCE_PORT_PREFIX};

use super::schema::{WorkflowSpec, PORT_AFTER};

/// Why a node is blocked from execution readiness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockedReason {
    UncommittedPrerequisite { node_id: NodeId },
    MissingRequiredDataInput { port: PortName },
}

/// Readiness snapshot for workflow execution coordination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinationStatus {
    pub ready: Vec<NodeId>,
    pub blocked: BTreeMap<NodeId, Vec<BlockedReason>>,
}

/// Compute readiness state from committed nodes and externally provided inputs.
///
/// This is completion-gated:
/// - control prerequisites must be committed,
/// - required data inputs must be available.
pub fn coordination_status(
    spec: &WorkflowSpec,
    committed: &HashSet<NodeId>,
    provided_inputs: &BTreeMap<NodeId, BTreeSet<PortName>>,
) -> CoordinationStatus {
    let mut ready = Vec::new();
    let mut blocked: BTreeMap<NodeId, Vec<BlockedReason>> = BTreeMap::new();

    for node in &spec.dag.nodes {
        let mut reasons = Vec::new();

        let incoming_edges = spec
            .dag
            .edges
            .iter()
            .filter(|edge| edge.to_node == node.id)
            .collect::<Vec<_>>();

        for edge in &incoming_edges {
            if !committed.contains(&edge.from_node) {
                reasons.push(BlockedReason::UncommittedPrerequisite {
                    node_id: edge.from_node.clone(),
                });
            }
        }

        for input in &node.inputs {
            if input.name.0 == PORT_AFTER || input.name.0.starts_with(RESOURCE_PORT_PREFIX) {
                continue;
            }
            if !input.cardinality.requires_one() {
                continue;
            }
            let provided = provided_inputs
                .get(&node.id)
                .is_some_and(|ports| ports.contains(&input.name));
            let has_data_edge = incoming_edges
                .iter()
                .any(|edge| edge.to_port == input.name && edge.kind.carries_data());
            if !provided && !has_data_edge {
                reasons.push(BlockedReason::MissingRequiredDataInput {
                    port: input.name.clone(),
                });
            }
        }

        if reasons.is_empty() {
            ready.push(node.id.clone());
        } else {
            blocked.insert(node.id.clone(), reasons);
        }
    }

    ready.sort();
    CoordinationStatus { ready, blocked }
}
