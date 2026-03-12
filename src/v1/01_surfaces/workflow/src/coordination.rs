//! Downstream coordination/readiness analysis (WF4).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use gunbc_ir::{NodeId, PortName};

use crate::schema::{WorkflowSpec, PORT_AFTER};

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
            if input.name.0 == PORT_AFTER || input.name.is_resource() {
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    use super::*;
    use gunbc_ir::{Dag, Edge, Node, Port};

    use crate::schema::{
        required_input_contract, required_output_contract, AggregateSpec, WorkflowId, WorkflowOp,
        WorkflowSpec, WorkflowUnit,
    };

    #[test]
    fn uncommitted_prerequisite_blocks_downstream_node() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "wf.a",
            required_input_contract(),
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::Aggregate(AggregateSpec::new("a"))),
        ));
        dag.add_node(Node::opaque(
            "wf.b",
            required_input_contract(),
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::Aggregate(AggregateSpec::new("b"))),
        ));
        dag.add_edge(Edge::control("wf.a", "commit", "wf.b", "after"));
        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);

        let status = coordination_status(&spec, &HashSet::new(), &BTreeMap::new());
        let reasons = status
            .blocked
            .get(&NodeId::from("wf.b"))
            .expect("wf.b should be blocked");
        assert!(reasons.iter().any(|reason| matches!(
            reason,
            BlockedReason::UncommittedPrerequisite { node_id } if node_id == &NodeId::from("wf.a")
        )));
    }

    #[test]
    fn missing_required_data_input_blocks_node() {
        let mut dag = Dag::new();
        let mut inputs = required_input_contract();
        inputs.push(Port::scalar("payload", "String"));
        dag.add_node(Node::opaque(
            "wf.only",
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::Aggregate(AggregateSpec::new("only"))),
        ));
        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);

        let status = coordination_status(&spec, &HashSet::new(), &BTreeMap::new());
        let reasons = status
            .blocked
            .get(&NodeId::from("wf.only"))
            .expect("wf.only should be blocked");
        assert!(reasons.iter().any(|reason| matches!(
            reason,
            BlockedReason::MissingRequiredDataInput { port } if port == &PortName::from("payload")
        )));
    }

    #[test]
    fn provided_required_data_input_marks_node_ready() {
        let mut dag = Dag::new();
        let mut inputs = required_input_contract();
        inputs.push(Port::scalar("payload", "String"));
        dag.add_node(Node::opaque(
            "wf.only",
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::Aggregate(AggregateSpec::new("only"))),
        ));
        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);

        let provided = BTreeMap::from([(
            NodeId::from("wf.only"),
            BTreeSet::from([PortName::from("payload")]),
        )]);
        let status = coordination_status(&spec, &HashSet::new(), &provided);
        assert!(
            status.ready.contains(&NodeId::from("wf.only")),
            "wf.only should be ready when required payload is provided"
        );
        assert!(
            !status.blocked.contains_key(&NodeId::from("wf.only")),
            "wf.only should not remain blocked once required payload is provided"
        );
    }
}
