use std::sync::Arc;

use v1_compiler::std_computation::ShrinkFactor;
use v1_compiler::std_induction::{derive_bound, CostBound};
use v1_compiler::std_termination::PositiveDescentAmount;

#[test]
fn derive_bound_rejects_non_positive_branch_count() {
    let unit = Arc::new(ShrinkFactor::UnitShrink);
    assert!(matches!(
        derive_bound("n".to_string(), 0, unit.clone(), 0).as_ref(),
        CostBound::ErrorBound
    ));
    assert!(matches!(
        derive_bound("n".to_string(), -3, unit, 0).as_ref(),
        CostBound::ErrorBound
    ));
}

#[test]
fn derive_bound_rejects_invalid_work_exponent_on_linear_paths() {
    let unit = Arc::new(ShrinkFactor::UnitShrink);
    assert!(matches!(
        derive_bound("n".to_string(), 1, unit.clone(), -1).as_ref(),
        CostBound::ErrorBound
    ));
    assert!(matches!(
        derive_bound("n".to_string(), 1, unit, 257).as_ref(),
        CostBound::ErrorBound
    ));

    let constant = Arc::new(ShrinkFactor::ConstantShrink {
        steps: Arc::new(PositiveDescentAmount::OneStep),
    });
    assert!(matches!(
        derive_bound("n".to_string(), 1, constant.clone(), -5).as_ref(),
        CostBound::ErrorBound
    ));
    assert!(matches!(
        derive_bound("n".to_string(), 1, constant, 300).as_ref(),
        CostBound::ErrorBound
    ));
}
