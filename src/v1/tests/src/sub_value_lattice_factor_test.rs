use std::sync::Arc;

use v1_compiler::std_computation::ShrinkFactor;
use v1_compiler::std_induction::{
    inductive_field_eq, join_sub_value, meet_sub_value, InductiveField, RecursionShape,
    SubValueRelation,
};
use v1_compiler::std_termination::PositiveDescentAmount;

fn dummy_field() -> Arc<InductiveField> {
    Arc::new(InductiveField {
        type_name: String::from("T"),
        variant_name: String::from("V"),
        field_name: String::from("f"),
        shape: RecursionShape::DirectRecursion,
        element_type: String::from("E"),
    })
}

#[test]
fn meet_join_strict_same_field_mismatched_constant_shrink_lands_in_lawful_lattice() {
    let field = dummy_field();
    let one = Arc::new(PositiveDescentAmount::OneStep);
    let two = Arc::new(PositiveDescentAmount::AdditionalStep {
        previous: one.clone(),
    });
    let a = Arc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Arc::new(ShrinkFactor::ConstantShrink { steps: one }),
    });
    let b = Arc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Arc::new(ShrinkFactor::ConstantShrink { steps: two }),
    });
    assert!(matches!(
        *meet_sub_value(a.clone(), b.clone()),
        SubValueRelation::NonIncreasingValue
    ));
    assert!(matches!(
        *join_sub_value(a, b),
        SubValueRelation::StrictAxisErased
    ));
}

#[test]
fn meet_join_strict_same_field_matching_factor_is_commutative() {
    let field = dummy_field();
    let steps = Arc::new(PositiveDescentAmount::OneStep);
    let fac = Arc::new(ShrinkFactor::ConstantShrink {
        steps: steps.clone(),
    });
    let left = Arc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: fac.clone(),
    });
    let right = Arc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: fac,
    });
    for (x, y) in [(left.clone(), right.clone()), (right, left)] {
        match (*meet_sub_value(x.clone(), y.clone())).clone() {
            SubValueRelation::StrictSubValue { field: f, factor } => {
                assert!(inductive_field_eq(f.clone(), field.clone()));
                assert!(matches!(
                    (*factor).clone(),
                    ShrinkFactor::ConstantShrink { steps: s } if *s == *steps
                ));
            }
            other => panic!("expected StrictSubValue, got {other:?}"),
        }
        match (*join_sub_value(x, y)).clone() {
            SubValueRelation::StrictSubValue { field: f, factor } => {
                assert!(inductive_field_eq(f.clone(), field.clone()));
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
    let one = Arc::new(PositiveDescentAmount::OneStep);
    let a = Arc::new(SubValueRelation::ArithmeticDescent {
        param: String::from("n"),
        factor: Arc::new(ShrinkFactor::UnitShrink),
    });
    let b = Arc::new(SubValueRelation::ArithmeticDescent {
        param: String::from("n"),
        factor: Arc::new(ShrinkFactor::ConstantShrink { steps: one }),
    });
    assert!(matches!(
        *meet_sub_value(a.clone(), b.clone()),
        SubValueRelation::NonIncreasingValue
    ));
    assert!(matches!(
        *join_sub_value(a, b),
        SubValueRelation::StrictAxisErased
    ));
}

#[test]
fn lattice_idempotence_on_non_parameterized_variants() {
    let cases = [
        Arc::new(SubValueRelation::PreservedValue),
        Arc::new(SubValueRelation::NonIncreasingValue),
        Arc::new(SubValueRelation::StrictAxisErased),
        Arc::new(SubValueRelation::MixedTop),
        Arc::new(SubValueRelation::SubValueUnknown),
    ];
    for r in cases.iter() {
        assert_eq!(*meet_sub_value(r.clone(), r.clone()), **r);
        assert_eq!(*join_sub_value(r.clone(), r.clone()), **r);
    }
}

#[test]
fn join_preserved_with_strict_lands_at_mixed_top() {
    let field = dummy_field();
    let preserved = Arc::new(SubValueRelation::PreservedValue);
    let strict = Arc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Arc::new(ShrinkFactor::UnitShrink),
    });
    assert!(matches!(
        *join_sub_value(preserved.clone(), strict.clone()),
        SubValueRelation::MixedTop
    ));
    assert!(matches!(
        *join_sub_value(strict, preserved),
        SubValueRelation::MixedTop
    ));
}

#[test]
fn meet_strict_axis_erased_with_strict_preserves_strict() {
    let field = dummy_field();
    let sae = Arc::new(SubValueRelation::StrictAxisErased);
    let strict = Arc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Arc::new(ShrinkFactor::UnitShrink),
    });
    assert!(matches!(
        *meet_sub_value(sae.clone(), strict.clone()),
        SubValueRelation::StrictSubValue { .. }
    ));
    assert!(matches!(
        *meet_sub_value(strict, sae),
        SubValueRelation::StrictSubValue { .. }
    ));
}

#[test]
fn meet_strict_axis_erased_with_preserved_drops_to_non_increasing() {
    let sae = Arc::new(SubValueRelation::StrictAxisErased);
    let preserved = Arc::new(SubValueRelation::PreservedValue);
    assert!(matches!(
        *meet_sub_value(sae.clone(), preserved.clone()),
        SubValueRelation::NonIncreasingValue
    ));
    assert!(matches!(
        *meet_sub_value(preserved, sae),
        SubValueRelation::NonIncreasingValue
    ));
}

#[test]
fn meet_mixed_top_drops_to_non_increasing() {
    let field = dummy_field();
    let mixed = Arc::new(SubValueRelation::MixedTop);
    let strict = Arc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Arc::new(ShrinkFactor::UnitShrink),
    });
    assert!(matches!(
        *meet_sub_value(mixed.clone(), strict.clone()),
        SubValueRelation::NonIncreasingValue
    ));
    assert!(matches!(
        *meet_sub_value(strict, mixed),
        SubValueRelation::NonIncreasingValue
    ));
}
