//! P2 / single-authority: `meet_sub_value` and `join_sub_value` must not drop
//! cost-relevant `ShrinkFactor` when field or param keys align (PR #726 review).
//!
//! PATH-2-FULL → option (A) Inc-split (2026-05-21): SubValueRelation
//! merge algebra over seven lawful inhabitants (plus `SubValueUnknown` ⊥). The two top
//! inhabitants — `StrictAxisErased` (above strict-style witnesses, proj=Strict) and
//! `MixedTop` (overall join-⊤, proj=NonIncreasing) — resolve the codex #15892 vs #15942
//! contradiction that a single Inc top could not satisfy: split joins by whether the
//! operands cross the strict/non-strict boundary, and project each top independently.
//!
//! Tests in this file exercise:
//!   - meet/join on same-field different-shrink-factor (mismatched but strict-style):
//!     meet → NonIncreasingValue, join → StrictAxisErased.
//!   - meet/join on same-param different-ArithmeticDescent factors: same lawful pair.
//!   - structural commutativity on matching field/factor (idempotent return).
//!   - lattice idempotence on every non-parameterised variant (each must self-equal so
//!     `sub_value_structural_eq` short-circuits before level analysis).
//!   - **join-projection safety**: join(PreservedValue, StrictSubValue) → MixedTop, NOT
//!     StrictAxisErased — Preserved is non-strict, so the join cannot claim Strict.
//!     (codex #15942)
//!   - **meet-monotonicity**: meet(StrictAxisErased, StrictSubValue{f}) → StrictSubValue{f}
//!     — both project to Strict, so meet does not strengthen evidence. (codex #15892)
//!   - strict-cone boundary: meet(StrictAxisErased, PreservedValue) drops to
//!     NonIncreasingValue (the GLB across the strict/non-strict boundary).
//!   - **meet vs MixedTop**: meet(MixedTop, X) → NonIncreasingValue — MixedTop is join-⊤
//!     only; meet must not strengthen proj(meet) above NonIncreasing. (follow-up to #3505)

use std::rc::Rc;

use v1_compiler::std_computation::ShrinkFactor;
use v1_compiler::std_induction::{
    inductive_field_eq, join_sub_value, meet_sub_value, InductiveField, RecursionShape,
    SubValueRelation,
};
use v1_compiler::std_termination::PositiveDescentAmount;

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
    // PATH-2-FULL → option (A): two `StrictSubValue` witnesses on the same field but
    // distinct shrink factors are incomparable strict-style witnesses. Their meet lands
    // at `NonIncreasingValue` (both arms prove non-increase, drop the axis); their join
    // lands at `StrictAxisErased` (the all-strict top — both arms project to Strict, so
    // the join can soundly claim "strict, axis erased"). Prior single-Inc code collapsed
    // both to `SubValueUnknown`; PATH-2-FULL pre-split routed the join to
    // `IncomparableValue` which conflated the all-strict and mixed-strict cases.
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
        SubValueRelation::StrictAxisErased
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
    // Same rationale as the StrictSubValue case but on `ArithmeticDescent`. Both
    // operands are strict-style structurals → join lands at `StrictAxisErased`;
    // meet lands at `NonIncreasingValue`.
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
        SubValueRelation::StrictAxisErased
    ));
}

#[test]
fn lattice_idempotence_on_non_parameterized_variants() {
    // Merge-helper idempotence: meet(a, a) = a, join(a, a) = a for every inhabitant.
    // The non-parameterised variants (PreservedValue, NonIncreasingValue, StrictAxisErased,
    // MixedTop, SubValueUnknown) have no payload but still need reflexive structural equality
    // so sub_value_structural_eq's short-circuit fires; without it, meet(Preserved, Preserved)
    // would fall through level analysis to NonIncreasingValue, breaking idempotence.
    let cases = [
        Rc::new(SubValueRelation::PreservedValue),
        Rc::new(SubValueRelation::NonIncreasingValue),
        Rc::new(SubValueRelation::StrictAxisErased),
        Rc::new(SubValueRelation::MixedTop),
        Rc::new(SubValueRelation::SubValueUnknown),
    ];
    for r in cases.iter() {
        assert_eq!(*meet_sub_value(r.clone(), r.clone()), **r);
        assert_eq!(*join_sub_value(r.clone(), r.clone()), **r);
    }
}

#[test]
fn join_preserved_with_strict_lands_at_mixed_top() {
    // Inc-split: join(PreservedValue, StrictSubValue{f}) crosses the strict/non-strict
    // boundary, so the join lands at `MixedTop` (proj=NonIncreasing) — not at
    // `StrictAxisErased` (which would claim Strict descent the Preserved branch did not
    // witness). Resolves codex review #15942 join-projection-safety.
    let field = dummy_field();
    let preserved = Rc::new(SubValueRelation::PreservedValue);
    let strict = Rc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Rc::new(ShrinkFactor::UnitShrink),
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
    // Inc-split: meet(StrictAxisErased, StrictSubValue{f}) returns the specific witness
    // StrictSubValue{f} (StrictAxisErased dominates strict-style witnesses). Both have
    // proj=Strict, so meet-monotonicity holds within the strict cone. Resolves codex
    // review #15892 meet-monotonicity concern.
    let field = dummy_field();
    let sae = Rc::new(SubValueRelation::StrictAxisErased);
    let strict = Rc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Rc::new(ShrinkFactor::UnitShrink),
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
    // StrictAxisErased and PreservedValue are incomparable (one is strict-style, the
    // other is non-strict). Their meet is the GLB: `NonIncreasingValue`. Confirms the
    // strict-cone boundary in the lattice.
    let sae = Rc::new(SubValueRelation::StrictAxisErased);
    let preserved = Rc::new(SubValueRelation::PreservedValue);
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
    // MixedTop is join-⊤ only. meet(MixedTop, X) must not return strict-style X — that
    // would strengthen proj(meet) from NonIncreasing (proj MixedTop) to Strict. The
    // Projection-sound meet drops to NonIncreasingValue for every distinct pair
    // involving MixedTop (idempotence on MixedTop itself is unchanged).
    let field = dummy_field();
    let mixed = Rc::new(SubValueRelation::MixedTop);
    let strict = Rc::new(SubValueRelation::StrictSubValue {
        field: field.clone(),
        factor: Rc::new(ShrinkFactor::UnitShrink),
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
