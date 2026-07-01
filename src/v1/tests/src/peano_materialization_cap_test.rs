use std::rc::Rc;

use v1_compiler::std_induction::{
    cost_bound_is_sum_bound, cost_constant, master_theorem, sum_bound, CostBound, RecurrenceForm,
};
use v1_compiler::std_termination::{
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

#[test]
fn sum_bound_rejects_empty_alternative_stack() {
    let terms = Rc::new(vec![]);
    assert!(matches!(sum_bound(terms).as_ref(), CostBound::ErrorBound));
}

#[test]
fn master_theorem_rejects_work_exponent_above_peano_cap() {
    let form = Rc::new(RecurrenceForm {
        param: "n".to_string(),
        branches: 2,
        divisor: 2,
        work_exponent: 257,
    });
    assert!(matches!(
        master_theorem(form).as_ref(),
        CostBound::ErrorBound
    ));
}

#[test]
fn master_theorem_rejects_negative_work_exponent() {
    let form = Rc::new(RecurrenceForm {
        param: "n".to_string(),
        branches: 2,
        divisor: 2,
        work_exponent: -1,
    });
    assert!(matches!(
        master_theorem(form).as_ref(),
        CostBound::ErrorBound
    ));
}
