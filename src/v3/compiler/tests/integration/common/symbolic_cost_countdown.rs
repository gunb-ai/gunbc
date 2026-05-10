//! Regression helper for the canonical recursive **countdown** fixture.
//!
//! Unary tail recursion composes a loop iterate bound with the recurrence body; after gate **#78**
//! descent-operand discipline and `collapse_unary_bind_tail_iterate_linear_product_if_duplicate_induction`,
//! the symbolic carrier for the canonical `fn countdown(n: Int) -> Int` fixture must normalize to a bare
//! **`LinearCost`** on the parameter — not a **`ProductCost(Linear, Linear)`** shell keyed off two
//! distinct `PortId`s for the same induction chain.

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

/// Pins **linear** symbolic cost for tail-recursive unary countdown: a single **`LinearCost`** on the
/// formal parameter (post unary-bind iterate collapse). Rejects **`PolynomialCost`** /
/// **`UnknownCost`** and rejects accidental **`ProductCost`** shells that would mask double-count
/// regressions for gate **#78**.
pub fn assert_recursive_countdown_linear_semantics(cost: &SymbolicCost) {
    assert!(
        !cost_contains_polynomial_or_unknown(cost),
        "recursive countdown must not admit Polynomial / Unknown carriers (super-linear or \
         ambiguous bound): got {cost:?}"
    );

    assert!(
        matches!(cost, SymbolicCost::LinearCost { .. }),
        "recursive countdown must normalize to a bare LinearCost on the unary parameter (gate #78 \
         oracle); got {cost:?}"
    );
}
