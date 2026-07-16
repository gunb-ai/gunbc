// seed-linked dep shim — minimal v2.std.node surface for fold_lowering self-emit receipt
use im_rc::{vector as vec, Vector as Vec};
use std::rc::Rc;

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

pub fn node_synthetic(kind: Rc<NodeKind>, children: Rc<Vec<Rc<Edge>>>) -> Rc<Node> {
    Rc::new(Node {
        kind,
        children,
        occurrence_id: Rc::new(NodeOccurrenceId::SyntheticOccurrence),
    })
}

pub fn name_occurrences(name: Symbol, children: Rc<Vec<Rc<Edge>>>) -> i64 {
    children
        .iter()
        .filter(|e| matches!(&*e.label, EdgeLabel::Named { name: n } if n == &name))
        .count() as i64
}
