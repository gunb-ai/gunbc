use im::Vector as Vec;
use std::rc::Rc;

use crate::v2_std_node::{Node, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum NoCorrectionReason {
    ExternalContractUnknown,
    UserInputBoundary,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Correction {
    Unavailable { reason: NoCorrectionReason },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Locus {
    pub reason: Symbol,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub reason: Symbol,
    pub at: Rc<Locus>,
    pub correction: Rc<Correction>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Outcome<T> {
    Accepted {
        value: T,
        diagnostics: Rc<Vec<Rc<Diagnostic>>>,
    },
    Rejected {
        diagnostics: Rc<Vec<Rc<Diagnostic>>>,
    },
}

pub fn node_locus(node: Rc<Node>) -> Rc<Locus> {
    let _ = node;
    Rc::new(Locus {
        reason: "synthetic".to_string(),
    })
}

pub fn outcome_accepted<T>(value: T) -> Rc<Outcome<T>> {
    Rc::new(Outcome::Accepted {
        value,
        diagnostics: Rc::new(Vec::new()),
    })
}

pub fn outcome_rejected<T>(d: Rc<Diagnostic>) -> Rc<Outcome<T>> {
    let mut diags = Vec::new();
    diags.push_back(d);
    Rc::new(Outcome::Rejected {
        diagnostics: Rc::new(diags),
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
