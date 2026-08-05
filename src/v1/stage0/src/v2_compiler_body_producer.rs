// Seed realization for v2.compiler.body_producer (Wave 2 easy-flip lane).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.body_producer
// is emitted-only and the behavioral harness is modeled (sbp_scaffold_dissolution_trigger).

use crate::usv_pilot_v2_std_algebra::list_snoc_item;
use crate::usv_pilot_v2_std_node::{
    node_rebuild, node_synthetic, Behavior, Connective, Edge, EdgeLabel, Node, NodeKind, Symbol,
};
use im::{vector as vec, Vector as Vec};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Locus {
    pub reason: Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NoCorrectionReason {
    ExternalContractUnknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Correction {
    Unavailable { reason: NoCorrectionReason },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub reason: Symbol,
    pub at: Arc<Locus>,
    pub correction: Arc<Correction>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Outcome<T> {
    Accepted {
        value: T,
        diagnostics: Arc<Vec<Arc<Diagnostic>>>,
    },
    Rejected {
        diagnostics: Arc<Vec<Arc<Diagnostic>>>,
    },
}

pub fn node_locus(node: Arc<Node>) -> Arc<Locus> {
    let _ = node;
    Arc::new(Locus {
        reason: "synthetic".to_string(),
    })
}

pub fn outcome_accepted<T>(value: T) -> Arc<Outcome<T>> {
    Arc::new(Outcome::Accepted {
        value,
        diagnostics: Arc::new(Vec::new()),
    })
}

pub fn outcome_rejected<T>(d: Arc<Diagnostic>) -> Arc<Outcome<T>> {
    let mut diags = Vec::new();
    diags.push_back(d);
    Arc::new(Outcome::Rejected {
        diagnostics: Arc::new(diags),
    })
}

pub fn bind_outcome<T: Clone, U, F>(o: Arc<Outcome<T>>, f: F) -> Arc<Outcome<U>>
where
    F: Fn(T) -> Arc<Outcome<U>> + Clone,
{
    match &*o {
        Outcome::Accepted { value, .. } => f(value.clone()),
        Outcome::Rejected { diagnostics } => Arc::new(Outcome::Rejected {
            diagnostics: diagnostics.clone(),
        }),
    }
}

pub fn well_formed(n: Arc<Node>) -> bool {
    let _ = n;
    true
}

pub fn body_producer_diagnostic(reason: Symbol, n: Arc<Node>) -> Arc<Diagnostic> {
    Arc::new(Diagnostic {
        reason,
        at: node_locus(n),
        correction: Arc::new(Correction::Unavailable {
            reason: NoCorrectionReason::ExternalContractUnknown,
        }),
    })
}

pub fn attach_arrow_body(arrow: Arc<Node>, body: Arc<Node>) -> Arc<Outcome<Arc<Node>>> {
    match &*arrow.kind {
        NodeKind::TypeNode { connective } => match &**connective {
            Connective::Arrow => {
                let children = arrow.children.clone();
                outcome_accepted(node_rebuild(
                    arrow,
                    list_snoc_item(
                        children,
                        Arc::new(Edge {
                            label: Arc::new(EdgeLabel::Named {
                                name: "arrow_body_edge".to_string(),
                            }),
                            target: body,
                        }),
                    ),
                ))
            }
            _ => outcome_rejected(body_producer_diagnostic(
                "body_producer_reason_malformed_attachment".to_string(),
                arrow,
            )),
        },
        _ => outcome_rejected(body_producer_diagnostic(
            "body_producer_reason_malformed_attachment".to_string(),
            arrow,
        )),
    }
}

pub fn body_producer_validated_behavior(body: Arc<Node>) -> Arc<Outcome<Arc<Node>>> {
    if well_formed(body.clone()) {
        outcome_accepted(body)
    } else {
        outcome_rejected(body_producer_diagnostic(
            "body_producer_reason_post_produce_not_well_formed".to_string(),
            body,
        ))
    }
}

pub fn body_producer_dispatch_structured_body(body: Arc<Node>) -> Arc<Outcome<Arc<Node>>> {
    match &*body.kind {
        NodeKind::ComputationNode { .. } => body_producer_validated_behavior(body),
        _ => outcome_rejected(body_producer_diagnostic(
            "body_producer_reason_resolved_shape".to_string(),
            body,
        )),
    }
}

pub fn produce_arrow_with_structured_body(
    signature: Arc<Node>,
    structured_body: Arc<Node>,
) -> Arc<Outcome<Arc<Node>>> {
    match &*body_producer_dispatch_structured_body(structured_body) {
        Outcome::Accepted { value, .. } => attach_arrow_body(signature, value.clone()),
        Outcome::Rejected { diagnostics } => Arc::new(Outcome::Rejected {
            diagnostics: diagnostics.clone(),
        }),
    }
}
