// Shared std-bridge shim — curated minimal v2.std.node surface.
//
// AUTHORITY: src/v2/std/node.dag. Hand-authored scaffold; see the header of
// v2_std_diagnostic.rs in this directory for why the bridge exists and what dissolves it.
// Drift is caught by tools.self_host_shim_surface_wall, not by a wet cargo build days later.
use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

pub type Symbol = String;

// src/v2/std/node.dag:7  `type Hash = ContentHash`
// dag/std/types.dag:164  `type ContentHash = NonEmptyStr where brand("ContentHash")`
pub type Hash = String;

// src/v2/std/node.dag:9  `type OccurrenceId = std.occurrence_identity.OccurrenceId`
// dag/std/occurrence_identity.dag:22  `type OccurrenceId { value: Int }`
//
// Modeled as the real record, NOT as the emitted form. The emitter currently renders this
// alias as `pub struct OccurrenceId(pub std::marker::PhantomData<()>)` — the hollow-alias /
// checkpoint_scalar_phantom class DESIGN.md tracks as a defect — so copying the emitted shape
// here would enshrine it in a second representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OccurrenceId {
    pub value: i64,
}

// src/v2/std/node.dag:15  `type NodeOccurrenceId = SyntheticOccurrence | MintedOccurrence { id: OccurrenceId }`
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NodeOccurrenceId {
    SyntheticOccurrence,
    MintedOccurrence { id: OccurrenceId },
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

pub fn node_rebuild(n: Rc<Node>, children: Rc<Vec<Rc<Edge>>>) -> Rc<Node> {
    Rc::new(Node {
        kind: n.kind.clone(),
        children,
        occurrence_id: n.occurrence_id.clone(),
    })
}

pub fn well_formed(n: Rc<Node>) -> bool {
    let _ = n;
    true
}

// src/v2/std/node.dag:355  `type NamedEdgeTargetLookup = Found { target: Node } | Ambiguous | Absent`
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NamedEdgeTargetLookup {
    Found { target: Rc<Node> },
    Ambiguous,
    Absent,
}

// src/v2/std/node.dag:360  `fn named_edge_target_lookup(children: List<Edge>, name: Symbol)`
pub fn named_edge_target_lookup(
    children: Rc<Vec<Rc<Edge>>>,
    name: Symbol,
) -> Rc<NamedEdgeTargetLookup> {
    children
        .iter()
        .cloned()
        .fold(Rc::new(NamedEdgeTargetLookup::Absent), |acc, e| {
            match (&*e.label, &*acc) {
                (EdgeLabel::Named { name: sym }, NamedEdgeTargetLookup::Absent)
                    if sym == &name =>
                {
                    Rc::new(NamedEdgeTargetLookup::Found {
                        target: e.target.clone(),
                    })
                }
                (EdgeLabel::Named { name: sym }, _) if sym == &name => {
                    Rc::new(NamedEdgeTargetLookup::Ambiguous)
                }
                _ => acc,
            }
        })
}
