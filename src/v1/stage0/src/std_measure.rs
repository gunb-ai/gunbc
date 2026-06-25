use self::Quantity::*;
use self::Scale::*;
pub use crate::std_nat::Nat;
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
pub enum Quantity {
    Time,
    Length,
    Mass,
    Memory,
    Information,
    DataRate,
    Frequency,
    Count,
    Currency,
    Power,
    ElectricPotential,
    ElectricCurrent,
    Resistance,
    Capacitance,
    Inductance,
    ElectricCharge,
    MagneticFlux,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum Scale {
    Atto,
    Femto,
    Pico,
    Nano,
    Micro,
    Milli,
    One,
    Kilo,
    Mega,
    Giga,
    Tera,
    Peta,
    Exa,
    Kibi,
    Mebi,
    Gibi,
    Tebi,
}

pub fn scale_exponent(s: Scale) -> i64 {
    match s {
        Scale::Atto => -18,
        Scale::Femto => -15,
        Scale::Pico => -12,
        Scale::Nano => -9,
        Scale::Micro => -6,
        Scale::Milli => -3,
        Scale::One => 0,
        Scale::Kilo => 3,
        Scale::Mega => 6,
        Scale::Giga => 9,
        Scale::Tera => 12,
        Scale::Peta => 15,
        Scale::Exa => 18,
        Scale::Kibi => 10,
        Scale::Mebi => 20,
        Scale::Gibi => 30,
        Scale::Tebi => 40,
    }
}

pub fn memory_scale_factor_bytes(s: Scale) -> Nat {
    match s {
        Scale::One => 1,
        Scale::Kibi => 1024,
        Scale::Mebi => 1048576,
        Scale::Gibi => 1073741824,
        Scale::Tebi => 1099511627776,
        _ => 1,
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Measure<Q, S, M> {
    pub count: Box<M>,
}

pub fn measure_count<Q, S, M>(m: Rc<Measure<Q, S, M>>) -> M {
    m.count.clone()
}

pub fn time_measure<S>(count: Nat) -> Rc<Measure<Time, S, Nat>> {
    Rc::new(Measure {
        count: Box::new(count),
    })
}

pub type ByteSize = Rc<Measure<Memory, One, Nat>>;

pub type Gibibyte = Rc<Measure<Memory, Gibi, Nat>>;

pub fn gibibyte(count: Nat) -> Gibibyte {
    Gibibyte { count: count }
}

pub fn gibibyte_count(g: Gibibyte) -> Nat {
    compile_error!("field access missing reconcile summary for 'count'")
}

pub fn gibibyte_to_byte_size(g: Gibibyte) -> ByteSize {
    byte_size((gibibyte_count(g) * memory_scale_factor_bytes(Scale::Gibi)))
}

pub type BitWidth = Rc<Measure<Information, One, Nat>>;

pub type Hertz = Rc<Measure<Frequency, One, Nat>>;

pub type HardwareThreadCount = Rc<Measure<Count, One, Nat>>;

pub type Watt = Rc<Measure<Power, One, Nat>>;

pub fn watt(count: Nat) -> Watt {
    Watt { count: count }
}

pub fn watt_count(w: Watt) -> Nat {
    compile_error!("field access missing reconcile summary for 'count'")
}

pub type MoneyAmount<S> = Rc<Measure<Currency, S, Nat>>;

pub type MoneyAmountMicro = MoneyAmount;

pub fn money_amount_micro(count: Nat) -> MoneyAmountMicro {
    MoneyAmountMicro { count: count }
}

pub fn money_amount_micro_count(m: MoneyAmountMicro) -> Nat {
    compile_error!("field access missing reconcile summary for 'count'")
}

pub fn byte_size(count: Nat) -> ByteSize {
    ByteSize { count: count }
}

pub fn byte_size_count(b: ByteSize) -> Nat {
    compile_error!("field access missing reconcile summary for 'count'")
}

pub fn bit_width(count: Nat) -> BitWidth {
    BitWidth { count: count }
}

pub fn bit_width_count(b: BitWidth) -> Nat {
    compile_error!("field access missing reconcile summary for 'count'")
}

pub fn hertz(count: Nat) -> Hertz {
    Hertz { count: count }
}

pub fn hertz_count(h: Hertz) -> Nat {
    compile_error!("field access missing reconcile summary for 'count'")
}

pub fn hardware_thread_count(count: Nat) -> HardwareThreadCount {
    HardwareThreadCount { count: count }
}

pub fn hardware_thread_count_value(t: HardwareThreadCount) -> Nat {
    compile_error!("field access missing reconcile summary for 'count'")
}

pub struct Time;
pub struct Length;
pub struct Mass;
pub struct Memory;
pub struct Information;
pub struct DataRate;
pub struct Frequency;
pub struct Count;
pub struct Currency;
pub struct Power;
pub struct ElectricPotential;
pub struct ElectricCurrent;
pub struct Resistance;
pub struct Capacitance;
pub struct Inductance;
pub struct ElectricCharge;
pub struct MagneticFlux;
pub struct Atto;
pub struct Femto;
pub struct Pico;
pub struct Nano;
pub struct Micro;
pub struct Milli;
pub struct One;
pub struct Kilo;
pub struct Mega;
pub struct Giga;
pub struct Tera;
pub struct Peta;
pub struct Exa;
pub struct Kibi;
pub struct Mebi;
pub struct Gibi;
pub struct Tebi;
