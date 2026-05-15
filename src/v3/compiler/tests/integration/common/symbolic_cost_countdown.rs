//! Regression helper for the canonical recursive **countdown** fixture.
//!
//! Unary tail recursion composes a loop iterate bound with the recurrence body; after gate **#78**
//! descent-operand discipline and `collapse_unary_bind_tail_iterate_linear_product_if_duplicate_induction`,
//! the symbolic carrier for the canonical `fn countdown(n: Int) -> Int` fixture must normalize to a bare
//! degree-1 **`PolynomialCost`** on the parameter — not a product shell keyed off two
//! distinct `PortId`s for the same induction chain.

use v3_compiler::dag::{NonZeroRational, SymbolicCost};

/// Returns `true` when `cost` contains a product or unknown anywhere under a
/// composite tree walk (super-linear / ambiguous bound carriers).
fn cost_contains_product_or_unknown(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::ProductCost { .. } | SymbolicCost::UnknownCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } => terms
            .iter()
            .any(|t| cost_contains_product_or_unknown(t.as_ref())),
        _ => false,
    }
}

/// Pins linear symbolic cost for tail-recursive unary countdown: a single degree-1
/// `PolynomialCost` on the formal parameter. Rejects `UnknownCost` and accidental
/// `ProductCost` shells that would mask double-count
/// regressions for gate **#78**.
pub fn assert_recursive_countdown_linear_semantics(cost: &SymbolicCost) {
    assert!(
        !cost_contains_product_or_unknown(cost),
        "recursive countdown must not admit Product / Unknown carriers (super-linear or \
         ambiguous bound): got {cost:?}"
    );

    assert!(
        matches!(
            cost,
            SymbolicCost::PolynomialCost {
                degree,
                ..
            } if degree == &NonZeroRational::ONE
        ),
        "recursive countdown must normalize to degree-1 PolynomialCost on the unary parameter (gate #78 \
         oracle); got {cost:?}"
    );
}
