// Shared std-bridge shim — curated minimal v2.std.diagnostic surface.
//
// AUTHORITY: src/v2/std/diagnostic.dag. This file is a hand-authored scaffold standing in
// for the emitted module while the emitted v2 std closure still fails to compile (Clone-bound
// derivation on emitted generics; see cssl_std_bridge_scaffold_dissolution_trigger). It is a
// parallel representation of a canonical fact and is therefore drift-prone BY CONSTRUCTION —
// the public type surface here must stay a superset of what emitted entry modules import.
//
// Drift is no longer silent: tools.self_host_shim_surface_wall reads the emitted
// `use crate::<shim_module>::{..}` names and refuses when this file does not provide them.
// Before this wall existed, a widened v2.std.diagnostic (Locus struct -> coproduct, plus
// Extent / LocusAnchor / Diagnostics / NonEmptyDiagnostics) rotted here unnoticed for days,
// visible only on the wet falsifier cadence.
//
// dissolve-on: emitted v2 std closure compiles; then this bridge is deleted, not re-synced.
use im::Vector as Vec;
use std::rc::Rc;

use crate::v2_std_node::{Node, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NoCorrectionReason {
    UserInputBoundary,
    AmbiguousIntent,
    ExternalContractUnknown,
    CorrectionNotModeled,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Correction {
    Suggested { node: Rc<Node> },
    Unavailable { reason: NoCorrectionReason },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Extent {
    WholeFile,
    ByteRange { start: i64, end: i64 },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LocusAnchor<T> {
    pub at: T,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Locus {
    Textual {
        file: Symbol,
        extent: Rc<Extent>,
    },
    NodeLocus {
        anchor: Rc<LocusAnchor<Rc<Node>>>,
    },
    PortLocus {
        anchor: Rc<LocusAnchor<Symbol>>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub reason: Symbol,
    pub at: Rc<Locus>,
    pub correction: Rc<Correction>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NonEmptyDiagnostics {
    pub head: Rc<Diagnostic>,
    pub tail: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Diagnostics {
    None,
    Some {
        diagnostics: Rc<NonEmptyDiagnostics>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Outcome<T> {
    Accepted {
        value: T,
        diagnostics: Rc<Diagnostics>,
    },
    Rejected {
        diagnostics: Rc<NonEmptyDiagnostics>,
    },
}

pub fn diagnostics_singleton(d: Rc<Diagnostic>) -> Rc<NonEmptyDiagnostics> {
    Rc::new(NonEmptyDiagnostics {
        head: d,
        tail: Rc::new(Vec::new()),
    })
}

pub fn node_locus(node: Rc<Node>) -> Rc<Locus> {
    Rc::new(Locus::NodeLocus {
        anchor: Rc::new(LocusAnchor { at: node }),
    })
}

pub fn port_locus(port: Symbol) -> Rc<Locus> {
    Rc::new(Locus::PortLocus {
        anchor: Rc::new(LocusAnchor { at: port }),
    })
}

pub fn textual_whole_file_locus(file: Symbol) -> Rc<Locus> {
    Rc::new(Locus::Textual {
        file,
        extent: Rc::new(Extent::WholeFile),
    })
}

pub fn outcome_accepted<T>(value: T) -> Rc<Outcome<T>> {
    Rc::new(Outcome::Accepted {
        value,
        diagnostics: Rc::new(Diagnostics::None),
    })
}

pub fn outcome_rejected<T>(d: Rc<Diagnostic>) -> Rc<Outcome<T>> {
    Rc::new(Outcome::Rejected {
        diagnostics: diagnostics_singleton(d),
    })
}

pub fn bind_outcome<T: Clone, U, F>(o: Rc<Outcome<T>>, f: F) -> Rc<Outcome<U>>
where
    F: Fn(T) -> Rc<Outcome<U>> + Clone,
{
    match &*o {
        Outcome::Accepted { value, .. } => f(value.clone()),
        Outcome::Rejected { diagnostics } => Rc::new(Outcome::Rejected {
            diagnostics: diagnostics.clone(),
        }),
    }
}
