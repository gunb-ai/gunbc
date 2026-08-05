// Seed-retained dep surface for v2.compiler.use_site_verdict pilot (Wave 2 Band A).
// Dissolve-on: v2.std.node self-emits; seed-linked extern imports replace this scaffold.
use im::{vector as vec, Vector as Vec};
use std::sync::Arc;

pub type Symbol = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NodeOccurrenceId {
    SyntheticOccurrence,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Connective {
    Atom { identity: String },
    Conj,
    Disj,
    Arrow,
    Cardinality,
    Instantiation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Behavior {
    Value,
    Transform,
    Branch,
    Loop,
    Bind,
    Match,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NodeKind {
    TypeNode { connective: Arc<Connective> },
    ComputationNode { behavior: Behavior },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum EdgeLabel {
    Named { name: String },
    Positional,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Edge {
    pub label: Arc<EdgeLabel>,
    pub target: Arc<Node>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub kind: Arc<NodeKind>,
    pub children: Arc<Vec<Arc<Edge>>>,
    pub occurrence_id: Arc<NodeOccurrenceId>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NamedEdgeTargetLookup {
    Found { target: Arc<Node> },
    Ambiguous,
    Absent,
}

pub fn node_synthetic(kind: Arc<NodeKind>, children: Arc<Vec<Arc<Edge>>>) -> Arc<Node> {
    Arc::new(Node {
        kind,
        children,
        occurrence_id: Arc::new(NodeOccurrenceId::SyntheticOccurrence),
    })
}

pub fn node_rebuild(n: Arc<Node>, children: Arc<Vec<Arc<Edge>>>) -> Arc<Node> {
    Arc::new(Node {
        kind: n.kind.clone(),
        children,
        occurrence_id: n.occurrence_id.clone(),
    })
}

pub fn named_edge_target_lookup(
    children: Arc<Vec<Arc<Edge>>>,
    name: Symbol,
) -> Arc<NamedEdgeTargetLookup> {
    children
        .iter()
        .cloned()
        .fold(Arc::new(NamedEdgeTargetLookup::Absent), |acc, e| {
            match (&*e.label, &*acc) {
                (EdgeLabel::Named { name: sym }, NamedEdgeTargetLookup::Absent) if sym == &name => {
                    Arc::new(NamedEdgeTargetLookup::Found {
                        target: e.target.clone(),
                    })
                }
                (EdgeLabel::Named { name: sym }, _) if sym == &name => {
                    Arc::new(NamedEdgeTargetLookup::Ambiguous)
                }
                _ => acc,
            }
        })
}
