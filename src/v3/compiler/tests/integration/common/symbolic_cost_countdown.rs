//! Regression helper for the canonical recursive **countdown** fixture.
//!
//! Normalized output may be a bare **`LinearCost`** or, when **both** a **Loop** iterate bound and
//! an inner recurrence **Transform** contribute distinct `SizeVariable` ports, a binary
//! **`ProductCost(Linear, Linear)`** — still **linear-family** (no `PolynomialCost` / `UnknownCost`).
//! Same-var `Linear × Linear` folds to **`PolynomialCost(n, 2)`** in the semiring, not `ProductCost`.

use v3_compiler::dag::SymbolicCost;

/// Returns `true` when `cost` contains a `PolynomialCost` or `UnknownCost` anywhere under a
/// composite tree walk (super-linear / ambiguous bound carriers).
fn cost_contains_polynomial_or_unknown(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::PolynomialCost { .. } | SymbolicCost::UnknownCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } | SymbolicCost::ProductCost { _0: terms } => terms
            .iter()
            .any(|t| cost_contains_polynomial_or_unknown(t.as_ref())),
        _ => false,
    }
}

/// Pins **linear-family** symbolic cost for tail-recursive countdown: **LinearCost**, or **binary**
/// **Product(Linear, Linear)** on **distinct** ports (iterate / loop × inner spine — DB-7
/// `product_of_linears_over_different_vars_stays_product`). Rejects carriers that admit super-linear
/// growth (`PolynomialCost`) or reflection holes (`UnknownCost`).
pub fn assert_recursive_countdown_linear_semantics(cost: &SymbolicCost) {
    assert!(
        !cost_contains_polynomial_or_unknown(cost),
        "recursive countdown must not admit Polynomial / Unknown carriers (super-linear or \
         ambiguous bound): got {cost:?}"
    );

    match cost {
        SymbolicCost::LinearCost { .. } => {}
        SymbolicCost::ProductCost { _0: terms } if terms.rest.is_empty() => {
            match (terms.first.as_ref(), terms.second.as_ref()) {
                (SymbolicCost::LinearCost { _0: va }, SymbolicCost::LinearCost { _0: vb }) => {
                    assert_ne!(
                        va, vb,
                        "same-var Linear × Linear should normalize to PolynomialCost(n, 2), not \
                         ProductCost — structural mismatch, got {cost:?}"
                    );
                }
                _ => panic!(
                    "iterate-shaped countdown must be Product(Linear, Linear) when not a bare \
                     LinearCost; got {cost:?}"
                ),
            }
        }
        _ => panic!(
            "recursive countdown must be LinearCost or iterate-shaped Product(Linear, Linear) with \
             distinct SizeVariables; got {cost:?}"
        ),
    }
}
