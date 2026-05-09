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

/// Peak memory across **alternative** control-flow possibilities (typically **branch arms**):
/// asymptotic **`max`/dominance** of the modeled arm peaks—the same **`dominant` → `max_path`**
/// basis used for branch sibling arms in `src/v3/lenses/cost.dag`.
///
/// This is **not** work-style sequencing (`sequential` / sum along a single path). Separate
/// composition rules apply when allocations **overlap** in time (live-range overlap)—not modeled
/// here; callers encode overlap facts in the composed `SymbolicCost` before invoking this helper.
#[must_use]
pub fn compose_branch_memory_peak(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    max_path(&[a, b])
}

/// Enforce-mode lens check: observed peak **exceeds** the user-declared budget.
///
/// Aligns with `LensEnforcement.violates(declared_budget, projected)` (`src/v3/std/lens_application.dag`):
/// per substrate comment, **`violates` is true iff observed EXCEEDS the budget** — **reflexive**
/// dominance must **not** count as violation (equality is compliant).
///
/// Under the **`SymbolicCost` partial order** ([`dominates`]), **`declared_budget` dominates
/// `observed_peak`** means the budget asymptotically **covers** the peak (budget is looser /
/// tying on the declared bound). **`!dominates(budget, peak)`** therefore means the contract is
/// not certified: **strict exceed** where comparable, and **fail-closed** wherever the order is
/// **incomparable** (distinct size variables without a dominance edge).
#[must_use]
pub fn memory_peak_enforcement_violates(
    declared_budget: &SymbolicCost,
    observed_peak: &SymbolicCost,
) -> bool {
    !dominates(declared_budget, observed_peak)
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
    fn compose_branch_memory_peak_delegates_to_max_path() {
        // Pin the authoritative composition operator: branch / peak merges must not drift.
        let p0 = PortId::test_raw(110);
        let p1 = PortId::test_raw(111);
        let a = SymbolicCost::LinearCost { _0: var(p0) };
        let b = SymbolicCost::LogCost { _0: var(p1) };
        assert_eq!(
            compose_branch_memory_peak(a.clone(), b.clone()),
            max_path(&[a, b])
        );
    }

    #[test]
    fn enforcement_violates_when_peak_exceeds_budget() {
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
    fn enforcement_clean_when_budget_ties_observed_peak_under_dominance() {
        let p = PortId::test_raw(202);
        let n = var(p);
        let cost = SymbolicCost::LinearCost { _0: n.clone() };
        assert!(
            !memory_peak_enforcement_violates(&cost, &SymbolicCost::LinearCost { _0: n }),
            "reflexive asymptotic pairs must not violate (EXCEEDS is strict over = in the contract)"
        );
    }

    #[test]
    fn enforcement_violates_on_incomparable_size_variables_fail_closed() {
        let a = SymbolicCost::LinearCost {
            _0: var(PortId::test_raw(203)),
        };
        let b = SymbolicCost::LinearCost {
            _0: var(PortId::test_raw(204)),
        };
        assert!(
            memory_peak_enforcement_violates(&a, &b),
            "incomparable `LinearCost` keys must not pass Enforce silently"
        );
    }

    #[test]
    fn branch_style_peak_over_two_quadratic_arms_normalizes_budget_sharpness_check() {
        let p = PortId::test_raw(300);
        let n = var(p);
        let q = SymbolicCost::PolynomialCost {
            var: n.clone(),
            degree: DegreeAtLeastTwo::TWO,
        };
        let peak_branch = compose_branch_memory_peak(q.clone(), q);
        assert!(
            memory_peak_enforcement_violates(
                &SymbolicCost::LinearCost { _0: n.clone() },
                &peak_branch
            ),
            "O(n²) peak should exceed O(n) declared budget",
        );
    }
}
