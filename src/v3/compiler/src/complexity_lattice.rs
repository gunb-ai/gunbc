//! Rust execution mirror of `asymptotic_dominates` in `src/v3/std/algebra.dag`.
//!
//! `emit_rust_module(complexity.dag)` lowers calls in `complexity_enforcement_violates`
//! to this symbol. Keep this implementation aligned with the `.dag` lattice — the
//! structural authority remains `algebra.dag`; this module is the host bridge for
//! generated lens code.

use crate::dag::AsymptoticClass;

pub fn asymptotic_dominates(a: &AsymptoticClass, b: &AsymptoticClass) -> bool {
    use AsymptoticClass::*;
    match a {
        ClassUnknown => true,
        ClassExponential => match b {
            ClassUnknown => false,
            ClassExponential
            | ClassPolynomial { .. }
            | ClassQuadratic
            | ClassLinearithmic
            | ClassLinear
            | ClassLog
            | ClassConstant => true,
        },
        ClassPolynomial { .. } => match b {
            ClassUnknown | ClassExponential => false,
            ClassPolynomial { .. }
            | ClassQuadratic
            | ClassLinearithmic
            | ClassLinear
            | ClassLog
            | ClassConstant => true,
        },
        ClassQuadratic => match b {
            ClassConstant | ClassLog | ClassLinear | ClassLinearithmic | ClassQuadratic => true,
            ClassPolynomial { .. } | ClassExponential | ClassUnknown => false,
        },
        ClassLinearithmic => match b {
            ClassConstant | ClassLog | ClassLinear | ClassLinearithmic => true,
            ClassQuadratic | ClassPolynomial { .. } | ClassExponential | ClassUnknown => false,
        },
        ClassLinear => match b {
            ClassConstant | ClassLog | ClassLinear => true,
            ClassLinearithmic
            | ClassQuadratic
            | ClassPolynomial { .. }
            | ClassExponential
            | ClassUnknown => false,
        },
        ClassLog => match b {
            ClassConstant | ClassLog => true,
            ClassLinear
            | ClassLinearithmic
            | ClassQuadratic
            | ClassPolynomial { .. }
            | ClassExponential
            | ClassUnknown => false,
        },
        ClassConstant => match b {
            ClassConstant => true,
            ClassLog
            | ClassLinear
            | ClassLinearithmic
            | ClassQuadratic
            | ClassPolynomial { .. }
            | ClassExponential
            | ClassUnknown => false,
        },
    }
}
