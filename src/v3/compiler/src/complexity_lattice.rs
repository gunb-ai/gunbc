//! Rust execution mirror of **degree-refined** `asymptotic_dominates` for T-LAS
//! enforcement: matches `v3.std.algebra::asymptotic_dominates` on non-poly arms and
//! adds `ClassPolynomial` degree order (Peano / `positive_descent_count` grade).
//! Stay aligned with `algebra.dag` + `complexity_enforcement_violates` (see comments
//! on `AsymptoticClass` and `complexity.dag`).

use crate::dag::{positive_descent_count, AsymptoticClass};

pub fn asymptotic_dominates(a: &AsymptoticClass, b: &AsymptoticClass) -> bool {
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
        // Poly–poly: compare `degree` via `positive_descent_count` (matches `algebra.dag`).
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

#[cfg(test)]
mod asymptotic_dominates_tests {
    use super::asymptotic_dominates;
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
        assert!(asymptotic_dominates(&p5, &p3));
        assert!(!asymptotic_dominates(&p3, &p5));
        assert!(asymptotic_dominates(&p3, &p3));
    }
}
