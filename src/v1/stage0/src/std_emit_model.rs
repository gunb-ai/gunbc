use self::PlaceholderConvention::*;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub use crate::std_decl_ref::DeclarationRef;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum PlaceholderConvention {
    IndexedArgs,
    NamedArg,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SimpleMethodSpec {
    pub method_name: String,
    pub template: String,
    pub wraps_result: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MethodTemplateContract {
    pub dag_method: Box<DeclarationRef>,
    pub runtime_template: String,
    pub emit_template: String,
    pub wraps_result: bool,
    pub placeholder_convention: PlaceholderConvention,
}
