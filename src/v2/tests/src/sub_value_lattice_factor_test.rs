//! P2 / single-authority: `meet_sub_value` and `join_sub_value` must not drop
//! cost-relevant `ShrinkFactor` when field or param keys align (PR #726 review).
//!
//! PATH-2-FULL (operator-ratified 2026-05-21): SubValueRelation now inhabits a lawful
//! `BoundedLattice` with the new `NonIncreasingValue` and `IncomparableValue` inhabitants.
//! Where two distinct structural witnesses meet, the result is `NonIncreasingValue` (their
//! greatest common lower bound — both witness at least non-increase) rather than the prior
//! fail-closed `SubValueUnknown` collapse; dually, joining incomparable structural witnesses
//! yields `IncomparableValue` rather than `SubValueUnknown`. This preserves termination
//! soundness — every prior conservatively-`SubValueUnknown` meet now lands at or above
//! `NonIncreasingValue`, so downstream `sub_value_to_evidence` still emits `NonIncreasing`
//! (never strengthens to `Strict` without a structural witness). Same-field-different-factor
//! still collapses to `NonIncreasingValue` (cannot witness identical shrink — but both arms
//! prove non-increase, so the lattice carries that fact forward).

use std::rc::Rc;

use v2_compiler::std_computation::ShrinkFactor;
use v2_compiler::std_induction::{
    inductive_field_eq, join_sub_value, meet_sub_value, InductiveField, RecursionShape,
    SubValueRelation,
};
use v2_compiler::std_termination::PositiveDescentAmount;

fn dummy_field() -> Rc<InductiveField> {
    Rc::new(InductiveField {
        type_name: String::from("T"),
        variant_name: String::from("V"),
        field_name: String::from("f"),
        shape: RecursionShape::DirectRecursion,
        element_type: String::from("E"),
    })
}

#[test]
fn meet_join_strict_same_field_mismatched_constant_shrink_lands_in_lawful_lattice() {
    // PATH-2-FULL: two `StrictSubValue` witnesses on the same field but distinct shrink
    // factors are incomparable structural witnesses (cannot mint a single shrink rate),
    // so the meet lands at `NonIncreasingValue` (both arms prove non-increase) and the
    // join lands at `IncomparableValue` (least common upper bound). Prior code collapsed
    // both to `SubValueUnknown`, silently dropping the "at least non-increasing" fact.
    let field = dummy_field();
    let one = Rc::new(PositiveDescentAmount::OneStep);
    let two = Rc::new(PositiveDescentAmount::AdditionalStep {
        previous: one.clone(),
    });
    let a = Rc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Rc::new(ShrinkFactor::ConstantShrink { steps: one }),
    });
    let b = Rc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Rc::new(ShrinkFactor::ConstantShrink { steps: two }),
    });
    assert!(matches!(
        *meet_sub_value(a.clone(), b.clone()),
        SubValueRelation::NonIncreasingValue
    ));
    assert!(matches!(
        *join_sub_value(a, b),
        SubValueRelation::IncomparableValue
    ));
}

#[test]
fn meet_join_strict_same_field_matching_factor_is_commutative() {
    let field = dummy_field();
    let steps = Rc::new(PositiveDescentAmount::OneStep);
    let fac = Rc::new(ShrinkFactor::ConstantShrink {
        steps: steps.clone(),
    });
    let left = Rc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: fac.clone(),
    });
    let right = Rc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: fac,
    });
    for (x, y) in [(left.clone(), right.clone()), (right, left)] {
        match (*meet_sub_value(x.clone(), y.clone())).clone() {
            SubValueRelation::StrictSubValue { field: f, factor } => {
                assert!(inductive_field_eq(&f, &field));
                assert!(matches!(
                    (*factor).clone(),
                    ShrinkFactor::ConstantShrink { steps: s } if *s == *steps
                ));
            }
            other => panic!("expected StrictSubValue, got {other:?}"),
        }
        match (*join_sub_value(x, y)).clone() {
            SubValueRelation::StrictSubValue { field: f, factor } => {
                assert!(inductive_field_eq(&f, &field));
                assert!(matches!(
                    (*factor).clone(),
                    ShrinkFactor::ConstantShrink { steps: s } if *s == *steps
                ));
            }
            other => panic!("expected StrictSubValue, got {other:?}"),
        }
    }
}

#[test]
fn meet_join_arithmetic_same_param_mismatched_factors_lands_in_lawful_lattice() {
    // Same rationale as the StrictSubValue case: two `ArithmeticDescent` witnesses on the
    // same ring parameter but distinct shrink factors are incomparable. PATH-2-FULL routes
    // their meet to `NonIncreasingValue` and their join to `IncomparableValue` rather than
    // the prior `SubValueUnknown` collapse — sound and strictly more informative.
    let one = Rc::new(PositiveDescentAmount::OneStep);
    let a = Rc::new(SubValueRelation::ArithmeticDescent {
        param: String::from("n"),
        factor: Rc::new(ShrinkFactor::UnitShrink),
    });
    let b = Rc::new(SubValueRelation::ArithmeticDescent {
        param: String::from("n"),
        factor: Rc::new(ShrinkFactor::ConstantShrink { steps: one }),
    });
    assert!(matches!(
        *meet_sub_value(a.clone(), b.clone()),
        SubValueRelation::NonIncreasingValue
    ));
    assert!(matches!(
        *join_sub_value(a, b),
        SubValueRelation::IncomparableValue
    ));
}
