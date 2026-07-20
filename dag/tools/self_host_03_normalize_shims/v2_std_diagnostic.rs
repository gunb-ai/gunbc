use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

use crate::v2_std_node::{Node, Symbol};

#[derive(Debug, Clone, PartialEq)]
pub struct Locus;

#[derive(Debug, Clone, PartialEq)]
pub enum NoCorrectionReason {
    UserInputBoundary,
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
pub struct NonEmptyDiagnostics(Rc<Vec<Rc<Diagnostic>>>);

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

pub fn node_locus(_node: Rc<Node>) -> Rc<Locus> {
    Rc::new(Locus)
}

pub fn diagnostics_singleton(d: Rc<Diagnostic>) -> Rc<NonEmptyDiagnostics> {
    Rc::new(NonEmptyDiagnostics(Rc::new(vec![d])))
}

pub fn diagnostics_merge(outer: Rc<Diagnostics>, inner: Rc<Diagnostics>) -> Rc<Diagnostics> {
    match (outer.as_ref(), inner.as_ref()) {
        (Diagnostics::None, Diagnostics::None) => Rc::new(Diagnostics::None),
        (Diagnostics::None, Diagnostics::Some { diagnostics }) => {
            Rc::new(Diagnostics::Some {
                diagnostics: diagnostics.clone(),
            })
        }
        (Diagnostics::Some { diagnostics: a }, Diagnostics::None) => Rc::new(Diagnostics::Some {
            diagnostics: a.clone(),
        }),
        (Diagnostics::Some { diagnostics: a }, Diagnostics::Some { diagnostics: b }) => {
            let mut merged = a.0.as_ref().clone();
            merged.extend(b.0.iter().cloned());
            Rc::new(Diagnostics::Some {
                diagnostics: Rc::new(NonEmptyDiagnostics(Rc::new(merged))),
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
            let mut merged = p.0.as_ref().clone();
            merged.extend(rejected.0.iter().cloned());
            Rc::new(NonEmptyDiagnostics(Rc::new(merged)))
        }
    }
}
