pub use crate::std_algebra::{AbelianGroup, GroupCompletion};
pub use crate::std_machine_constraints::{Compose, MachineWidth, PointerWidth};
pub use crate::std_nat::Nat;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub type Int8 = crate::std_machine_constraints::Compose;

pub type Int16 = crate::std_machine_constraints::Compose;

pub type Int32 = crate::std_machine_constraints::Compose;

pub type Int64 = crate::std_machine_constraints::Compose;

pub type Int128 = crate::std_machine_constraints::Compose;

pub type UInt8 = crate::std_machine_constraints::Compose;

pub type UInt16 = crate::std_machine_constraints::Compose;

pub type UInt32 = crate::std_machine_constraints::Compose;

pub type UInt64 = crate::std_machine_constraints::Compose;

pub type UInt128 = crate::std_machine_constraints::Compose;

pub type Int = Rc<crate::std_algebra::AbelianGroup<crate::std_algebra::GroupCompletion<Nat>>>;

pub type UInt = Nat;

pub type IntPlatform = crate::std_machine_constraints::Compose;

pub type UIntPlatform = crate::std_machine_constraints::Compose;

pub type NonNegativeInt = Nat;

pub type PositiveInt = Nat;
