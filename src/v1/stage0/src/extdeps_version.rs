pub use crate::std_algebra::Ordering;
use crate::std_algebra::Ordering::*;
pub use crate::std_types::NonEmptyStr;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub type VersionIdentity = String;

#[derive(Clone)]
pub struct VersionScheme {
    pub compare: Rc<dyn Fn(VersionIdentity, VersionIdentity) -> Ordering>,
}

pub type VersionConstraint = String;
