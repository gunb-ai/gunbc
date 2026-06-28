use self::VariantEncoding::*;
use self::VariantNaming::*;
use self::WireFormat::*;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub use crate::std_decl_ref::DeclarationRef;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum VariantNaming {
    AsAuthored,
    SnakeCase,
    ScreamingSnakeCase,
    StripPrefixAndSnakeCase {
        prefix: Rc<FreeMonoid<Nat>>,
    },
    StripSuffixAndSnakeCase {
        suffix: Rc<FreeMonoid<Nat>>,
    },
    StripPrefixSuffixAndSnakeCase {
        prefix: Rc<FreeMonoid<Nat>>,
        suffix: Rc<FreeMonoid<Nat>>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum VariantEncoding {
    StringVariant {
        naming: Rc<VariantNaming>,
    },
    InternallyTaggedObject {
        tag_field: Rc<FreeMonoid<Nat>>,
        naming: Rc<VariantNaming>,
    },
    UntaggedVariant,
    TaggedVariant,
}
impl VariantEncoding {
    pub fn naming(&self) -> Rc<VariantNaming> {
        match self {
            VariantEncoding::StringVariant { naming: __val, .. } => __val.clone(),
            VariantEncoding::InternallyTaggedObject { naming: __val, .. } => __val.clone(),
            VariantEncoding::UntaggedVariant => panic!("no naming on unit variant"),
            VariantEncoding::TaggedVariant => panic!("no naming on unit variant"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CoproductWireContract {
    pub coproduct: Box<DeclarationRef>,
    pub encoding: Rc<VariantEncoding>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WireContract {
    pub variant_encoding: Rc<VariantEncoding>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum WireFormat {
    Json,
    Text,
}

pub struct Json;
pub struct Text;
