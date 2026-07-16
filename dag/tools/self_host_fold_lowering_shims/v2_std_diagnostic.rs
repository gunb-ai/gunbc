use im_rc::{vector as vec, Vector as Vec};
use std::rc::Rc;

use crate::v2_std_node::{Node, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
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
    pub _phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Locus {
    Textual { file: String, extent: Rc<Extent> },
    NodeLocus { anchor: Rc<LocusAnchor<Rc<Node>>> },
    PortLocus { anchor: Rc<LocusAnchor<String>> },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub reason: String,
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
    Some { diagnostics: Rc<NonEmptyDiagnostics> },
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

pub fn node_locus(node: Rc<Node>) -> Rc<Locus> {
    Rc::new(Locus::NodeLocus {
        anchor: Rc::new(LocusAnchor {
            at: node,
            _phantom: std::marker::PhantomData,
        }),
    })
}

pub fn outcome_accepted<T: Clone>(value: T) -> Rc<Outcome<T>> {
    Rc::new(Outcome::Accepted {
        value,
        diagnostics: Rc::new(Diagnostics::None),
    })
}

pub fn outcome_rejected<T>(d: Rc<Diagnostic>) -> Rc<Outcome<T>> {
    Rc::new(Outcome::Rejected {
        diagnostics: Rc::new(NonEmptyDiagnostics {
            head: d,
            tail: Rc::new(vec![]),
        }),
    })
}
