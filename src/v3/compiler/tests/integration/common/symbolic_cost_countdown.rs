//! Regression helper for the canonical recursive **countdown** fixture: pins **LinearCost** after
//! `recursive_transform_cost` matches the complexity spine (no redundant sequential constant under
//! `iterate`).

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

/// Canonical **`fn countdown(n: Int) -> Int = … countdown(n-1)`** fixture: must present as a
/// single **`LinearCost`** on the parameter-sized witness after recurrence lowering — matching the
/// complexity lens spine (`recursive_transform_summary` composes `compose_many_inputs` without an
/// extra `ConstantCost(1)` sequential wrapper under `iterate`; cost lens now mirrors that shape via
/// `sum_costs` alone in `recursive_transform_cost`).
pub fn assert_recursive_countdown_linear_semantics(cost: &SymbolicCost) {
    assert!(
        !cost_contains_polynomial_or_unknown(cost),
        "recursive countdown must not admit Polynomial / Unknown carriers (super-linear or \
         ambiguous bound): got {cost:?}"
    );
    assert!(
        matches!(cost, SymbolicCost::LinearCost { .. }),
        "recursive countdown must normalize to LinearCost(O(n) in `n`); got {cost:?}"
    );
}
