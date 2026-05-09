//! Memory-peak composition for the symbolic cost lens (gate #94 — `memory_peak_cost_basis_demonstrated`).
//!
//! Work-style costs compose with `sequential` / `iterate` (time adds, loops multiply). Memory
//! **peak** over alternative control-flow paths uses **max**, not sum — see
//! `docs/design-lens-application-surface.md` §4.3 and
//! `docs/audit/t-user-authored-cost-basis-discipline-worked-examples.md` (worked example 2).
//!
//! This module is the interim **cost-lens-owned** Rust authority until class-5 bodies can name
//! memory-dimension folds in `v3.std.algebra` and until the `EnforcedApplication` fold consumes
//! `apply_lens(cost, …)` sites end-to-end (`docs/r3-program-plan.md` gate #91 consumer).

use crate::dag::{dominates, max_path, SymbolicCost};

/// Peak memory for two **sequential** regions whose live ranges do not overlap: the peak is the
/// asymptotic **maximum** of the two region peaks (same rule as `max_path` over branch arms).
#[must_use]
pub fn compose_sequential_memory_peak(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    max_path(&[a, b])
}

/// Enforce-mode lens check: observed peak **exceeds** the user-declared budget.
///
/// Mirrors `LensEnforcement.violates(declared_budget, projected_observed)` orientation in
/// `src/v3/std/lens_application.dag`: returns `true` iff the observed cost dominates the budget
/// under [`dominates`].
#[must_use]
pub fn memory_peak_enforcement_violates(
    declared_budget: &SymbolicCost,
    observed_peak: &SymbolicCost,
) -> bool {
    dominates(observed_peak, declared_budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{max_path, DegreeAtLeastTwo, PortId, SizeVariable};

    fn var(p: PortId) -> SizeVariable {
        SizeVariable {
            source_port: p,
            display_name: None,
        }
    }

    #[test]
    fn compose_sequential_memory_peak_delegates_to_max_path() {
        // Pin the authoritative composition operator: branch / peak merges must not drift.
        let p0 = PortId::test_raw(110);
        let p1 = PortId::test_raw(111);
        let a = SymbolicCost::LinearCost { _0: var(p0) };
        let b = SymbolicCost::LogCost { _0: var(p1) };
        assert_eq!(
            compose_sequential_memory_peak(a.clone(), b.clone()),
            max_path(&[a, b])
        );
    }

    #[test]
    fn enforcement_violates_when_peak_dominates_budget() {
        let p = PortId::test_raw(200);
        let n = var(p);
        let budget = SymbolicCost::LogCost { _0: n.clone() };
        let peak = SymbolicCost::LinearCost { _0: n };
        assert!(memory_peak_enforcement_violates(&budget, &peak));
    }

    #[test]
    fn enforcement_clean_when_budget_covers_peak() {
        let p = PortId::test_raw(201);
        let n = var(p);
        let budget = SymbolicCost::LinearCost { _0: n.clone() };
        let peak = SymbolicCost::LogCost { _0: n };
        assert!(!memory_peak_enforcement_violates(&budget, &peak));
    }

    #[test]
    fn branch_style_peak_over_two_quadratic_arms_normalizes_budget_sharpness_check() {
        let p = PortId::test_raw(300);
        let n = var(p);
        let q = SymbolicCost::PolynomialCost {
            var: n.clone(),
            degree: DegreeAtLeastTwo::TWO,
        };
        let peak_branch = compose_sequential_memory_peak(q.clone(), q);
        assert!(
            memory_peak_enforcement_violates(
                &SymbolicCost::LinearCost { _0: n.clone() },
                &peak_branch
            ),
            "O(n²) peak should exceed O(n) declared budget",
        );
    }
}
