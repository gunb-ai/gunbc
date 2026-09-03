// Seed-retained dep surface for v2.compiler.use_site_verdict pilot (Wave 2 Band A).
// Dissolve-on: v2.std.node self-emits; seed-linked extern imports replace this scaffold.
// CLIPPY ROSTER -- 1 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    unused_imports,  // 1
)]

use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

pub type Symbol = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NodeOccurrenceIdentity {
    OccurrenceSynthetic,
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
    pub occurrence_id: Rc<NodeOccurrenceIdentity>,
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
        occurrence_id: Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
    })
}

pub fn node_rebuild(n: Rc<Node>, children: Rc<Vec<Rc<Edge>>>) -> Rc<Node> {
    Rc::new(Node {
        kind: n.kind.clone(),
        children,
        occurrence_id: n.occurrence_id.clone(),
    })
}

pub fn named_edge_target_lookup(
    children: Rc<Vec<Rc<Edge>>>,
    name: Symbol,
) -> Rc<NamedEdgeTargetLookup> {
    children
        .iter()
        .cloned()
        .fold(Rc::new(NamedEdgeTargetLookup::Absent), |acc, e| {
            match (&*e.label, &*acc) {
                (EdgeLabel::Named { name: sym }, NamedEdgeTargetLookup::Absent) if sym == &name => {
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
