use self::LensVerdict::*;
use self::LensVerdictLocus::*;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum LensVerdictLocus {
    ModuleWholeFile { module_name: Rc<FreeMonoid<Nat>> },
}
impl LensVerdictLocus {
    pub fn module_name(&self) -> Rc<FreeMonoid<Nat>> {
        match self {
            LensVerdictLocus::ModuleWholeFile {
                module_name: __val, ..
            } => __val.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LensVerdictDiagnostic {
    pub reason: Rc<FreeMonoid<Nat>>,
    pub at: Rc<LensVerdictLocus>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum LensVerdict {
    Holds,
    Violation {
        diagnostic: Rc<LensVerdictDiagnostic>,
    },
    NotApplicable {
        diagnostic: Rc<LensVerdictDiagnostic>,
    },
    Unrealized {
        diagnostic: Rc<LensVerdictDiagnostic>,
    },
}
impl LensVerdict {
    pub fn diagnostic(&self) -> Rc<LensVerdictDiagnostic> {
        match self {
            LensVerdict::Holds => panic!("no diagnostic on unit variant"),
            LensVerdict::Violation {
                diagnostic: __val, ..
            } => __val.clone(),
            LensVerdict::NotApplicable {
                diagnostic: __val, ..
            } => __val.clone(),
            LensVerdict::Unrealized {
                diagnostic: __val, ..
            } => __val.clone(),
        }
    }
}
