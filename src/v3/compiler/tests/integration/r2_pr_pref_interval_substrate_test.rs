//! **Layer:** integration
//!
//! PR-PreF structural acceptance: `Interval<D>` shared parent for ordered-numeric
//! bound carriers (`docs/briefs/r2-substrate-manager.md`).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    size_bound_cardinal_interval, Cardinal, CardinalityBound, DescentEvidence, Interval, Ordinal,
    PositiveDescentAmount, SizeBound,
};

#[test]
fn interval_d_shared_parent_consolidation_landed() {
    assert!(matches!(
        CardinalityBound::AT_MOST_ONE,
        CardinalityBound::AtMostOne
    ));
    assert_eq!(
        CardinalityBound::AT_MOST_ONE
            .try_as_cardinal_interval()
            .expect("AtMostOne"),
        Interval::try_exact_interval(0, 1).expect("0..=1")
    );
    let dag = compile_to_dag("data probe: Int = 0\n", "pr_pref_substrate_bootstrap.v3")
        .expect("trivial program compiles");
    let _ = dag;

    let zero = size_bound_cardinal_interval(&SizeBound::ExplicitCountZero)
        .expect("ExplicitCountZero maps to interval");
    assert_eq!(
        zero,
        Interval::try_exact_interval(0, 0).expect("singleton zero")
    );

    let steps = PositiveDescentAmount::AdditionalStep {
        previous: Box::new(PositiveDescentAmount::OneStep),
    };
    let pos = size_bound_cardinal_interval(&SizeBound::ExplicitCountPositive { steps })
        .expect("ExplicitCountPositive maps");
    assert_eq!(
        pos,
        Interval::try_exact_interval(2, 2).expect("exactly two steps")
    );

    assert!(
        size_bound_cardinal_interval(&SizeBound::Forever).is_none(),
        "Forever uses constant_bound_value / forever_iteration_bound, not Interval projection"
    );
    assert!(size_bound_cardinal_interval(&SizeBound::TreeSize {
        param: "x".to_string()
    })
    .is_none());
}

#[test]
fn bound_carrier_parent_matches_algebra_shape() {
    fn assert_cardinal_interval(_: Interval<Cardinal>) {}
    fn assert_ordinal_interval(_: Interval<Ordinal>) {}
    assert_cardinal_interval(
        CardinalityBound::UNBOUNDED
            .try_as_cardinal_interval()
            .expect("Unbounded"),
    );
    assert_ordinal_interval(Interval::Unbounded);
}

#[test]
fn no_lattice_to_interval_collapse_bridge() {
    assert_ne!(
        std::any::TypeId::of::<DescentEvidence>(),
        std::any::TypeId::of::<Interval<Ordinal>>()
    );
}

#[test]
fn interval_try_exact_interval_rejects_inverted_bounds() {
    assert!(Interval::<Cardinal>::try_exact_interval(2, 1).is_none());
    let ok = Interval::try_exact_interval(1, 2).expect("ordered");
    assert!(ok.is_ordered_closed());
    let forged = Interval::ExactInterval { lo: 2, hi: 1 };
    assert!(!forged.is_ordered_closed());
}
