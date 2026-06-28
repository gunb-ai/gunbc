use crate::std_algebra::Ordering::{Equal, Greater, Less};
pub use crate::std_algebra::{CommutativeSemiring, Ordering};
pub use crate::std_magnitude::Magnitude;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub type Nat = Rc<crate::std_algebra::CommutativeSemiring<Magnitude>>;

pub fn nat_compare(a: Nat, b: Nat) -> Ordering {
    if (a.clone() < b.clone()) {
        Ordering::Less
    } else {
        if (a.clone() > b.clone()) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

pub fn nat_max(a: Nat, b: Nat) -> Nat {
    if (a.clone() > b.clone()) {
        a.clone()
    } else {
        b.clone()
    }
}

pub fn nat_min(a: Nat, b: Nat) -> Nat {
    if (a.clone() < b.clone()) {
        a.clone()
    } else {
        b.clone()
    }
}
