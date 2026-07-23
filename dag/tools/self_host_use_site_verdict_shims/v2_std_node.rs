// seed-linked dep shim — minimal v2.std.node surface for use_site_verdict pilot
use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

pub type Symbol = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NodeOccurrenceId { SyntheticOccurrence }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Connective {
    Atom { identity: String },
    Conj, Disj, Arrow, Cardinality, Instantiation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Behavior { Value, Transform, Branch, Loop, Bind, Match }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NodeKind {
    TypeNode { connective: Rc<Connective> },
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
    pub label: Rc<EdgeLabel>,
    pub target: Rc<Node>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub kind: Rc<NodeKind>,
    pub children: Rc<Vec<Rc<Edge>>>,
    pub occurrence_id: Rc<NodeOccurrenceId>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NamedEdgeTargetLookup {
    Found { target: Rc<Node> },
    Ambiguous,
    Absent,
}

pub fn node_synthetic(kind: Rc<NodeKind>, children: Rc<Vec<Rc<Edge>>>) -> Rc<Node> {
    Rc::new(Node {
        kind,
        children,
        occurrence_id: Rc::new(NodeOccurrenceId::SyntheticOccurrence),
    })
}

pub fn node_rebuild(n: Rc<Node>, children: Rc<Vec<Rc<Edge>>>) -> Rc<Node> {
    Rc::new(Node { kind: n.kind.clone(), children, occurrence_id: n.occurrence_id.clone() })
}

pub fn named_edge_target_lookup(children: Rc<Vec<Rc<Edge>>>, name: Symbol) -> Rc<NamedEdgeTargetLookup> {
    children.iter().cloned().fold(Rc::new(NamedEdgeTargetLookup::Absent), |acc, e| {
        match (&*e.label, &*acc) {
            (EdgeLabel::Named { name: sym }, NamedEdgeTargetLookup::Absent) if sym == &name => {
                Rc::new(NamedEdgeTargetLookup::Found { target: e.target.clone() })
            }
            (EdgeLabel::Named { name: sym }, _) if sym == &name => {
                Rc::new(NamedEdgeTargetLookup::Ambiguous)
            }
            _ => acc,
        }
    })
}
