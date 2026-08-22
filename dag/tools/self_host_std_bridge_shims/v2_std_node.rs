// Shared std-bridge shim — curated minimal v2.std.node surface.
//
// AUTHORITY: src/v2/std/node.dag. Hand-authored scaffold; see the header of
// v2_std_diagnostic.rs in this directory for why the bridge exists, what dissolves it, and
// why drift against the authority is caught only by the wet receipt's cargo build today.
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

// src/v2/std/node.dag:568  `fn well_formed(n: Node) -> Bool`
//
// UNION OF BODIES, not just of names. The two per-transport copies this bridge replaces
// DISAGREED here: body_producer's was `let _ = n; true` and 03_normalize's carried the real
// root check below. Taking the weaker one would have been a §5 fail-open on a fail-closed
// path — emitted 03_normalize gates graft output on `well_formed` and raises
// `post_normalize_not_well_formed_diagnostic` on false, so an always-true stub lets a
// malformed empty Conj/Disj through the normalize gate in the seed-linked Rust path.
// The name-superset check that cleared the other eleven replacements does NOT catch this
// class: both copies define `well_formed`, and only the bodies differ. Bodies are compared
// too now (review 44739 found this one).
//
// RESIDUAL GAP vs the authority, stated rather than implied: the authority folds
// `locally_well_formed` over the whole tree (`connective_edges_conform` /
// `behavior_edges_conform` per node). This carries the strongest surviving SHIM behavior —
// a root-level non-emptiness check — not that fold. It is strictly stronger than
// either copy it replaces and strictly weaker than the authority.
// dissolve-on: emitted v2 std closure compiles; then the authority's fold is used directly.
//
// THE CATCH-ALL IS GONE, AND IT WAS NOT COSMETIC. `_ => true` answered well-formed for
// `Cardinality` and `Instantiation`, and the authority `connective_edges_conform` requires
// `count(children) >= 1` for BOTH — so an empty node of either form passed a check the
// authority fails. That is the same §5 fail-open this shim's own header says it exists to
// prevent, one connective away from the case review 44739 caught, and a wildcard is exactly
// what hid it: the two forms were never named here, so no reader could see they were
// unanswered. Every form is now named, which also means a new `Connective` member in
// `v2.std.node` stops this compile instead of being silently admitted as well-formed.
//
// `Arrow` remains `true` and that is a DECLARED residue, not an answer: the authority routes
// it to `arrow_body_edges_conform`, which this bridge does not carry, and inventing a weaker
// arrow rule here would be a second authority for the same fact. It is named rather than
// absorbed into a wildcard so the gap is countable.
pub fn well_formed(n: Rc<Node>) -> bool {
    match n.kind.as_ref() {
        NodeKind::TypeNode { connective } => match connective.as_ref() {
            Connective::Atom { .. } => true,
            Connective::Conj | Connective::Disj => !n.children.is_empty(),
            Connective::Cardinality | Connective::Instantiation => !n.children.is_empty(),
            Connective::Arrow => true,
        },
        NodeKind::ComputationNode { .. } => true,
    }
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
