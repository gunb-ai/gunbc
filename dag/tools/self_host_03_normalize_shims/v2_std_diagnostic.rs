use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

use crate::v2_std_node::Node;

#[derive(Debug, Clone, PartialEq)]
pub enum Extent {
    WholeFile,
    ByteRange { start: i64, end: i64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocusAnchor<T> {
    pub at: T,
    pub _phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Locus {
    Textual {
        file: String,
        extent: Rc<Extent>,
    },
    NodeLocus {
        anchor: Rc<LocusAnchor<Rc<Node>>>,
    },
    PortLocus {
        anchor: Rc<LocusAnchor<String>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoCorrectionReason {
    UserInputBoundary,
    ExternalContractUnknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Correction {
    Unavailable { reason: NoCorrectionReason },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub reason: String,
    pub at: Rc<Locus>,
    pub correction: Rc<Correction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonEmptyDiagnostics {
    pub head: Rc<Diagnostic>,
    pub tail: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Diagnostics {
    None,
    Some {
        diagnostics: Rc<NonEmptyDiagnostics>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<T> {
    Accepted {
        value: T,
        diagnostics: Rc<Diagnostics>,
    },
    Rejected {
        diagnostics: Rc<NonEmptyDiagnostics>,
    },
}

pub fn diag_none() -> Rc<Diagnostics> {
    Rc::new(Diagnostics::None)
}

pub fn node_locus(node: Rc<Node>) -> Rc<Locus> {
    Rc::new(Locus::NodeLocus {
        anchor: Rc::new(LocusAnchor {
            at: node,
            _phantom: std::marker::PhantomData,
        }),
    })
}

pub fn diagnostics_singleton(d: Rc<Diagnostic>) -> Rc<NonEmptyDiagnostics> {
    Rc::new(NonEmptyDiagnostics {
        head: d,
        tail: Rc::new(vec![]),
    })
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
