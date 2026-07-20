// Seed realization for v2.compiler.target_carriers (Wave 2 Band A).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.target_carriers
// is emitted-only and the behavioral harness is modeled (stc_scaffold_dissolution_trigger).

use crate::usv_pilot_v2_std_node::{
    named_edge_target_lookup, node_synthetic, Connective, Edge, EdgeLabel, NamedEdgeTargetLookup,
    Node, NodeKind,
};
use im::{vector as vec, Vector as Vec};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum DecodeFidelity {
    Lossless,
    Lossy,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Medium<T> {
    pub carried: T,
    pub fidelity: DecodeFidelity,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TargetModel {
    pub bundle: Arc<Node>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum OptionalNode {
    Absent,
    Present { value: Arc<Node> },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Outcome<T> {
    Accepted { value: T },
    Rejected { reason: String },
}

pub fn source_medium(text: String, fidelity: DecodeFidelity) -> Medium<String> {
    Medium {
        carried: text,
        fidelity,
    }
}

pub fn lossless_source(text: String) -> Medium<String> {
    source_medium(text, DecodeFidelity::Lossless)
}

fn decode_fidelity_merge(left: DecodeFidelity, right: DecodeFidelity) -> DecodeFidelity {
    match (left, right) {
        (DecodeFidelity::Lossy, _) | (_, DecodeFidelity::Lossy) => DecodeFidelity::Lossy,
        (DecodeFidelity::Lossless, DecodeFidelity::Lossless) => DecodeFidelity::Lossless,
    }
}

fn fidelity_disposition_node_has_named_kind(root: Arc<Node>, name: String) -> bool {
    match &*named_edge_target_lookup(root.children.clone(), name) {
        NamedEdgeTargetLookup::Found { .. } => true,
        _ => false,
    }
}

fn fidelity_disposition_node_decode_fidelity(node: Arc<Node>) -> Outcome<DecodeFidelity> {
    match &*node.kind {
        NodeKind::TypeNode { connective } => match &**connective {
            Connective::Atom { identity }
                if identity == "dag_fidelity_disposition_kind_modeled" =>
            {
                Outcome::Accepted {
                    value: DecodeFidelity::Lossless,
                }
            }
            Connective::Conj => {
                if fidelity_disposition_node_has_named_kind(
                    node.clone(),
                    "dag_fidelity_disposition_kind_declared_normalized".to_string(),
                ) || fidelity_disposition_node_has_named_kind(
                    node.clone(),
                    "dag_fidelity_disposition_kind_fail_closed".to_string(),
                ) {
                    Outcome::Accepted {
                        value: DecodeFidelity::Lossy,
                    }
                } else {
                    Outcome::Rejected {
                        reason: "target_carriers_fidelity_disposition_malformed".to_string(),
                    }
                }
            }
            _ => Outcome::Rejected {
                reason: "target_carriers_fidelity_disposition_malformed".to_string(),
            },
        },
        _ => Outcome::Rejected {
            reason: "target_carriers_fidelity_disposition_malformed".to_string(),
        },
    }
}

fn fidelity_quotient_decode_fidelity(quotient: Arc<Node>) -> Outcome<DecodeFidelity> {
    match &*quotient.kind {
        NodeKind::TypeNode { connective } => match &**connective {
            Connective::Conj => quotient.children.iter().fold(
                Outcome::Accepted {
                    value: DecodeFidelity::Lossless,
                },
                |acc, edge| match acc {
                    Outcome::Rejected { reason } => Outcome::Rejected { reason },
                    Outcome::Accepted {
                        value: current_fidelity,
                    } => match fidelity_disposition_node_decode_fidelity(edge.target.clone()) {
                        Outcome::Rejected { reason } => Outcome::Rejected { reason },
                        Outcome::Accepted {
                            value: next_fidelity,
                        } => Outcome::Accepted {
                            value: decode_fidelity_merge(current_fidelity, next_fidelity),
                        },
                    },
                },
            ),
            _ => Outcome::Rejected {
                reason: "target_carriers_fidelity_quotient_malformed".to_string(),
            },
        },
        _ => Outcome::Rejected {
            reason: "target_carriers_fidelity_quotient_malformed".to_string(),
        },
    }
}

fn target_fidelity_quotient_optional(target: Arc<TargetModel>) -> Outcome<OptionalNode> {
    match &*named_edge_target_lookup(
        target.bundle.children.clone(),
        "target_model_edge_fidelity_quotient".to_string(),
    ) {
        NamedEdgeTargetLookup::Found { target: child } => Outcome::Accepted {
            value: OptionalNode::Present {
                value: child.clone(),
            },
        },
        _ => Outcome::Accepted {
            value: OptionalNode::Absent,
        },
    }
}

pub fn decode_fidelity_from_target(target: Arc<TargetModel>) -> Outcome<DecodeFidelity> {
    match target_fidelity_quotient_optional(target) {
        Outcome::Rejected { reason } => Outcome::Rejected { reason },
        Outcome::Accepted { value: opt } => match opt {
            OptionalNode::Absent => Outcome::Accepted {
                value: DecodeFidelity::Lossless,
            },
            OptionalNode::Present { value: quotient } => {
                fidelity_quotient_decode_fidelity(quotient)
            }
        },
    }
}

pub fn target_source_medium(text: String, target: Arc<TargetModel>) -> Outcome<Medium<String>> {
    match decode_fidelity_from_target(target) {
        Outcome::Rejected { reason } => Outcome::Rejected { reason },
        Outcome::Accepted { value: fidelity } => Outcome::Accepted {
            value: source_medium(text, fidelity),
        },
    }
}

pub fn fidelity_disposition_modeled_node() -> Arc<Node> {
    node_synthetic(
        Arc::new(NodeKind::TypeNode {
            connective: Arc::new(Connective::Atom {
                identity: "dag_fidelity_disposition_kind_modeled".to_string(),
            }),
        }),
        Arc::new(vec![]),
    )
}

pub fn fidelity_disposition_declared_loss_node() -> Arc<Node> {
    let child = node_synthetic(
        Arc::new(NodeKind::TypeNode {
            connective: Arc::new(Connective::Atom {
                identity: "dag_fidelity_disposition_kind_declared_normalized".to_string(),
            }),
        }),
        Arc::new(vec![]),
    );
    node_synthetic(
        Arc::new(NodeKind::TypeNode {
            connective: Arc::new(Connective::Conj),
        }),
        Arc::new(vec![Arc::new(Edge {
            label: Arc::new(EdgeLabel::Named {
                name: "child".to_string(),
            }),
            target: child,
        })]),
    )
}
