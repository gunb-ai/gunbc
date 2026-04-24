//! M9 / P4: Peano literal bridges cap at 256 — oversize `Int` inputs must fail closed (`none`),
//! not deep-recurse or wrap (PR #726 review: boundary regression).

use std::rc::Rc;

use v2_compiler::std_induction::{cost_bound_is_sum_bound, cost_constant, sum_bound};
use v2_compiler::std_termination::{
    positive_descent_amount_from_positive_int, proportional_divisor_from_int_at_least_two,
};

#[test]
fn positive_descent_amount_rejects_above_256() {
    assert!(positive_descent_amount_from_positive_int(300).is_none());
    assert!(positive_descent_amount_from_positive_int(257).is_none());
    assert!(positive_descent_amount_from_positive_int(256).is_some());
    assert!(positive_descent_amount_from_positive_int(1).is_some());
}

#[test]
fn proportional_divisor_rejects_above_256() {
    assert!(proportional_divisor_from_int_at_least_two(300).is_none());
    assert!(proportional_divisor_from_int_at_least_two(257).is_none());
    assert!(proportional_divisor_from_int_at_least_two(256).is_some());
    assert!(proportional_divisor_from_int_at_least_two(2).is_some());
}

#[test]
fn sum_bound_has_cost_bound_introspection_consumer() {
    let terms = Rc::new(vec![cost_constant()]);
    let b = sum_bound(terms);
    assert!(cost_bound_is_sum_bound(b));
    assert!(!cost_bound_is_sum_bound(cost_constant()));
}
