// Shared std-bridge shim — curated minimal v2.std.diagnostic surface.
//
// AUTHORITY: src/v2/std/diagnostic.dag. This file is a hand-authored scaffold standing in
// for the emitted module while the emitted v2 std closure still fails to compile (Clone-bound
// derivation on emitted generics; see cssl_std_bridge_scaffold_dissolution_trigger). It is a
// parallel representation of a canonical fact and is therefore drift-prone BY CONSTRUCTION —
// the public type surface here must stay a superset of what emitted entry modules import.
//
// DRIFT DETECTION TODAY — stated plainly because it is weak: the only thing that catches a
// missing name is the emitted crate's own cargo build inside the wet behavioral receipt, and
// that receipt runs on the falsifier cadence, not per-PR (the selection axis is reads, and
// src/v2/std/*.dag is this file's AUTHORITY, not its read — see
// tools.self_host_03_normalize_declared_source_refs's authority-gap note). That is how a
// widened v2.std.diagnostic (Locus struct -> coproduct, plus Extent / LocusAnchor /
// Diagnostics / NonEmptyDiagnostics) rotted here unnoticed for days. This file re-syncs that
// widening; it does NOT close the gap. The intended mechanism is a shim-surface wall reading
// the emitted `use crate::<shim_module>::{..}` names and refusing when this file does not
// provide them; it DOES NOT EXIST YET and its medium is undecided.
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
    SourcePortLocus {
        anchor: Rc<LocusAnchor<Symbol>>,
        file: Symbol,
        extent: Rc<Extent>,
    },
    InvariantPortLocus {
        anchor: Rc<LocusAnchor<Symbol>>,
        invariant: Symbol,
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

pub fn source_port_locus(port: Symbol, file: Symbol, extent: Rc<Extent>) -> Rc<Locus> {
    Rc::new(Locus::SourcePortLocus {
        anchor: Rc::new(LocusAnchor { at: port }),
        file,
        extent,
    })
}

pub fn invariant_port_locus(port: Symbol, invariant: Symbol) -> Rc<Locus> {
    Rc::new(Locus::InvariantPortLocus {
        anchor: Rc::new(LocusAnchor { at: port }),
        invariant,
    })
}

pub fn textual_whole_file_locus(file: Symbol) -> Rc<Locus> {
    Rc::new(Locus::Textual {
        file,
        extent: Rc::new(Extent::WholeFile),
    })
}

pub fn diag_none() -> Rc<Diagnostics> {
    Rc::new(Diagnostics::None)
}

pub fn diagnostics_merge(outer: Rc<Diagnostics>, inner: Rc<Diagnostics>) -> Rc<Diagnostics> {
    match (outer.as_ref(), inner.as_ref()) {
        (Diagnostics::None, Diagnostics::None) => Rc::new(Diagnostics::None),
        (Diagnostics::None, Diagnostics::Some { diagnostics }) => Rc::new(Diagnostics::Some {
            diagnostics: diagnostics.clone(),
        }),
        (Diagnostics::Some { diagnostics: a }, Diagnostics::None) => Rc::new(Diagnostics::Some {
            diagnostics: a.clone(),
        }),
        (Diagnostics::Some { diagnostics: a }, Diagnostics::Some { diagnostics: b }) => {
            let mut merged = a.tail.as_ref().clone();
            merged.push_back(a.head.clone());
            merged.extend(b.tail.iter().cloned());
            merged.push_back(b.head.clone());
            let head = merged.pop_front().unwrap();
            Rc::new(Diagnostics::Some {
                diagnostics: Rc::new(NonEmptyDiagnostics {
                    head,
                    tail: Rc::new(merged),
                }),
            })
        }
    }
}

pub fn rejected_with_pending(
    pending: Rc<Diagnostics>,
    rejected: Rc<NonEmptyDiagnostics>,
) -> Rc<NonEmptyDiagnostics> {
    match pending.as_ref() {
        Diagnostics::None => rejected,
        Diagnostics::Some { diagnostics: p } => {
            let mut merged = p.tail.as_ref().clone();
            merged.push_back(p.head.clone());
            merged.extend(rejected.tail.iter().cloned());
            merged.push_back(rejected.head.clone());
            let head = merged.pop_front().unwrap();
            Rc::new(NonEmptyDiagnostics {
                head,
                tail: Rc::new(merged),
            })
        }
    }
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
