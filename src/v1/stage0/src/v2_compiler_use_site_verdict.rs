// Seed realization for v2.compiler.use_site_verdict (Wave 2 Band A pilot).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.use_site_verdict
// is emitted-only and the behavioral harness is modeled (ssuv_scaffold_dissolution_trigger).

use crate::usv_pilot_v2_std_algebra::list_snoc_item;
use crate::usv_pilot_v2_std_node::{
    named_edge_target_lookup, node_rebuild, node_synthetic, Connective, Edge, EdgeLabel,
    NamedEdgeTargetLookup, Node, NodeKind, Symbol,
};
use im::{vector as vec, Vector as Vec};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum UseSiteVerdict {
    MoveWhole,
    MoveField { field: Symbol },
    Borrow,
    CloneShared,
    Unclassified,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum UseSiteVerdictLookup {
    VerdictFound { verdict: Arc<UseSiteVerdict> },
    VerdictAbsent,
    VerdictAmbiguous,
}

pub fn use_site_verdict_edge() -> Symbol {
    "use_site_verdict".to_string()
}

pub fn use_site_verdict_field_edge() -> Symbol {
    "use_site_verdict_field".to_string()
}

pub fn use_site_verdict_move_whole_tag() -> Symbol {
    "use_site_verdict_move_whole".to_string()
}

pub fn use_site_verdict_move_field_tag() -> Symbol {
    "use_site_verdict_move_field".to_string()
}

pub fn use_site_verdict_borrow_tag() -> Symbol {
    "use_site_verdict_borrow".to_string()
}

pub fn use_site_verdict_clone_shared_tag() -> Symbol {
    "use_site_verdict_clone_shared".to_string()
}

pub fn use_site_verdict_unclassified_tag() -> Symbol {
    "use_site_verdict_unclassified".to_string()
}

pub fn use_site_verdict_atom(tag: Symbol) -> Arc<Node> {
    node_synthetic(
        Arc::new(NodeKind::TypeNode {
            connective: Arc::new(Connective::Atom { identity: tag }),
        }),
        Arc::new(vec![]),
    )
}

pub fn use_site_verdict_to_node(verdict: Arc<UseSiteVerdict>) -> Arc<Node> {
    match &*verdict {
        UseSiteVerdict::MoveWhole => use_site_verdict_atom(use_site_verdict_move_whole_tag()),
        UseSiteVerdict::MoveField { field } => node_synthetic(
            Arc::new(NodeKind::TypeNode {
                connective: Arc::new(Connective::Atom {
                    identity: use_site_verdict_move_field_tag(),
                }),
            }),
            Arc::new(vec![Arc::new(Edge {
                label: Arc::new(EdgeLabel::Named {
                    name: use_site_verdict_field_edge(),
                }),
                target: use_site_verdict_atom(field.clone()),
            })]),
        ),
        UseSiteVerdict::Borrow => use_site_verdict_atom(use_site_verdict_borrow_tag()),
        UseSiteVerdict::CloneShared => use_site_verdict_atom(use_site_verdict_clone_shared_tag()),
        UseSiteVerdict::Unclassified => use_site_verdict_atom(use_site_verdict_unclassified_tag()),
    }
}

pub fn use_site_verdict_move_field_of(children: Arc<Vec<Arc<Edge>>>) -> Arc<UseSiteVerdict> {
    match &*named_edge_target_lookup(children, use_site_verdict_field_edge()) {
        NamedEdgeTargetLookup::Found { target: field_node } => match &*field_node.kind {
            NodeKind::TypeNode { connective } => match &**connective {
                Connective::Atom { identity: f } => {
                    Arc::new(UseSiteVerdict::MoveField { field: f.clone() })
                }
                _ => Arc::new(UseSiteVerdict::Unclassified),
            },
            _ => Arc::new(UseSiteVerdict::Unclassified),
        },
        NamedEdgeTargetLookup::Absent | NamedEdgeTargetLookup::Ambiguous => {
            Arc::new(UseSiteVerdict::Unclassified)
        }
    }
}

pub fn use_site_verdict_of_node(target: Arc<Node>) -> Arc<UseSiteVerdict> {
    match &*target.kind {
        NodeKind::TypeNode { connective } => match &**connective {
            Connective::Atom { identity: sym } => {
                if sym == &use_site_verdict_move_whole_tag() {
                    Arc::new(UseSiteVerdict::MoveWhole)
                } else if sym == &use_site_verdict_borrow_tag() {
                    Arc::new(UseSiteVerdict::Borrow)
                } else if sym == &use_site_verdict_clone_shared_tag() {
                    Arc::new(UseSiteVerdict::CloneShared)
                } else if sym == &use_site_verdict_move_field_tag() {
                    use_site_verdict_move_field_of(target.children.clone())
                } else {
                    Arc::new(UseSiteVerdict::Unclassified)
                }
            }
            _ => Arc::new(UseSiteVerdict::Unclassified),
        },
        _ => Arc::new(UseSiteVerdict::Unclassified),
    }
}

pub fn use_site_verdict_lookup(node: Arc<Node>) -> Arc<UseSiteVerdictLookup> {
    match &*named_edge_target_lookup(node.children.clone(), use_site_verdict_edge()) {
        NamedEdgeTargetLookup::Found { target } => Arc::new(UseSiteVerdictLookup::VerdictFound {
            verdict: use_site_verdict_of_node(target.clone()),
        }),
        NamedEdgeTargetLookup::Absent => Arc::new(UseSiteVerdictLookup::VerdictAbsent),
        NamedEdgeTargetLookup::Ambiguous => Arc::new(UseSiteVerdictLookup::VerdictAmbiguous),
    }
}

pub fn attach_use_site_verdict(node: Arc<Node>, verdict: Arc<UseSiteVerdict>) -> Arc<Node> {
    let children = node.children.clone();
    node_rebuild(
        node,
        list_snoc_item(
            children,
            Arc::new(Edge {
                label: Arc::new(EdgeLabel::Named {
                    name: use_site_verdict_edge(),
                }),
                target: use_site_verdict_to_node(verdict),
            }),
        ),
    )
}
