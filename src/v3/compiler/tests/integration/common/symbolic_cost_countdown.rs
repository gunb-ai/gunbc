//! Structural assertions for the canonical recursive **countdown** fixture — pins linear-family
//! semantics without weakening the regression to “any non-trivial iterate-shaped carrier”.
//!
//! After per-call recurrence wiring, normalization may keep **`ProductCost(Linear, Linear)`**
//! when the iterate bound and body spine carry **distinct** `SizeVariable`s (same underlying `n`,
//! different ports) — see DB-7 `product_of_linears_over_different_vars_stays_product`. That shape
//! is still **linear-family** (no `PolynomialCost`, no `UnknownCost`), and same-var products fold to
//! `PolynomialCost(n, 2)` instead — so we reject same-var `Product(Linear, Linear)` as an integrity
//! violation.

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

/// Countdown must present **`LinearCost`** (single linear factor) **or** the honest iterate split:
/// binary **`ProductCost`** of two **`LinearCost`** factors on **distinct** ports (distinct
/// `SizeVariable` keys — otherwise semiring normalization folds to `PolynomialCost`).
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
            "recursive countdown must be LinearCost(O(n)) or iterate-shaped Product(Linear, Linear) \
             with distinct SizeVariables per DB-7; got {cost:?}"
        ),
    }
}
