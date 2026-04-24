//! P2 / single-authority: `meet_sub_value` and `join_sub_value` must not drop
//! cost-relevant `ShrinkFactor` when field or param keys align (PR #726 review).

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
fn meet_join_strict_same_field_mismatched_constant_shrink_is_unknown() {
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
        SubValueRelation::SubValueUnknown
    ));
    assert!(matches!(
        *join_sub_value(a, b),
        SubValueRelation::SubValueUnknown
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
fn meet_join_arithmetic_same_param_mismatched_factors_is_unknown() {
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
        SubValueRelation::SubValueUnknown
    ));
    assert!(matches!(
        *join_sub_value(a, b),
        SubValueRelation::SubValueUnknown
    ));
}
