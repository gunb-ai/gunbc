use self::Classical::*;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum Classical {
    True,
    False,
}

pub fn classical_not(a: Classical) -> Classical {
    match a {
        Classical::True => Classical::False,
        Classical::False => Classical::True,
    }
}

pub fn classical_and(a: Classical, b: Classical) -> Classical {
    match a {
        Classical::False => Classical::False,
        Classical::True => b,
    }
}

pub fn classical_or(a: Classical, b: Classical) -> Classical {
    match a {
        Classical::True => Classical::True,
        Classical::False => b,
    }
}
