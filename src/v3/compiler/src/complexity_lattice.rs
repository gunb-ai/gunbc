//! **`complexity_enforcement_budget_dominates`** — executable **T-LAS enforcement**
//! ordering on `AsymptoticClass` for `complexity_enforcement_violates` /
//! `EnforcedApplication` (gate #92). It agrees with staged
//! `v3.std.algebra::asymptotic_dominates` on every arm **except**
//! `ClassPolynomial×ClassPolynomial`, where this function compares Peano degrees
//! (`positive_descent_count`), while the substrate `.dag` carrier stays tier-coarse
//! for `meet` / `join` until bootstrap can lower the same story in `algebra.dag`
//! (dissolution: program-plan gate **#92** / ROADMAP).
//!
//! **`asymptotic_dominates`** is a backward-compatible alias for tests and older
//! call sites; new enforcement code should call [`complexity_enforcement_budget_dominates`].
//!
//! Stay aligned with `complexity.dag` + `complexity_enforcement_violates` comments.

use crate::dag::{positive_descent_count, AsymptoticClass};

pub fn complexity_enforcement_budget_dominates(a: &AsymptoticClass, b: &AsymptoticClass) -> bool {
    use AsymptoticClass::*;
    match a {
        ClassUnknown => true,
        ClassExponential => match b {
            ClassUnknown => false,
            ClassExponential
            | ClassPolynomial { .. } // Any `ClassPolynomial` tier is below exponential.
            | ClassQuadratic
            | ClassLinearithmic
            | ClassLinear
            | ClassLog
            | ClassConstant => true,
        },
        ClassPolynomial { degree: da } => match b {
            ClassUnknown | ClassExponential => false,
            ClassPolynomial { degree: db } => {
                positive_descent_count(da) >= positive_descent_count(db)
            }
            ClassQuadratic | ClassLinearithmic | ClassLinear | ClassLog | ClassConstant => true,
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

/// Alias of [`complexity_enforcement_budget_dominates`]. Prefer the named helper for
/// new call sites so enforcement-grade poly refinement is not confused with substrate
/// `algebra.dag::asymptotic_dominates` at the documentation layer.
#[inline]
pub fn asymptotic_dominates(a: &AsymptoticClass, b: &AsymptoticClass) -> bool {
    complexity_enforcement_budget_dominates(a, b)
}

#[cfg(test)]
mod asymptotic_dominates_tests {
    use super::{asymptotic_dominates, complexity_enforcement_budget_dominates};
    use crate::dag::{positive_amount_from_i64, AsymptoticClass};

    fn poly(k: i64) -> AsymptoticClass {
        AsymptoticClass::ClassPolynomial {
            degree: positive_amount_from_i64(k).expect("test degree in range"),
        }
    }

    #[test]
    fn polynomial_degree_orders_within_class_polynomial() {
        let p5 = poly(5);
        let p3 = poly(3);
        assert!(complexity_enforcement_budget_dominates(&p5, &p3));
        assert!(!complexity_enforcement_budget_dominates(&p3, &p5));
        assert!(complexity_enforcement_budget_dominates(&p3, &p3));
    }

    #[test]
    fn alias_matches_budget_dominates() {
        let p3 = poly(3);
        let p5 = poly(5);
        assert_eq!(
            asymptotic_dominates(&p5, &p3),
            complexity_enforcement_budget_dominates(&p5, &p3)
        );
    }
}
