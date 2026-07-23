use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

pub type Symbol = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeOccurrenceId {
    SyntheticOccurrence,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Connective {
    Atom { identity: String },
    Conj,
    Disj,
    Arrow,
    Cardinality,
    Instantiation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behavior {
    Value,
    Transform,
    Branch,
    Loop,
    Bind,
    Match,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    TypeNode { connective: Rc<Connective> },
    ComputationNode { behavior: Behavior },
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeLabel {
    Named { name: String },
    Positional,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub label: Rc<EdgeLabel>,
    pub target: Rc<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: Rc<NodeKind>,
    pub children: Rc<Vec<Rc<Edge>>>,
    pub occurrence_id: Rc<NodeOccurrenceId>,
}

pub fn node_rebuild(n: Rc<Node>, children: Rc<Vec<Rc<Edge>>>) -> Rc<Node> {
    Rc::new(Node {
        kind: n.kind.clone(),
        children,
        occurrence_id: n.occurrence_id.clone(),
    })
}

pub fn well_formed(n: Rc<Node>) -> bool {
    match n.kind.as_ref() {
        NodeKind::TypeNode { connective } => match connective.as_ref() {
            Connective::Atom { .. } => true,
            Connective::Conj | Connective::Disj => !n.children.is_empty(),
            _ => true,
        },
        NodeKind::ComputationNode { .. } => true,
    }
}

pub fn node_synthetic(kind: Rc<NodeKind>, children: Rc<Vec<Rc<Edge>>>) -> Rc<Node> {
    Rc::new(Node {
        kind,
        children,
        occurrence_id: Rc::new(NodeOccurrenceId::SyntheticOccurrence),
    })
}
