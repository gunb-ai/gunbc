//! **Layer:** integration
//!
//! T-Lens-Application-Surface gate **#94** `memory_peak_cost_basis_demonstrated`.
//!
//! Exercises the cost-lens memory-peak authority in `v3_compiler::memory_peak_cost`:
//! branch-arm peaks compose with [`v3_compiler::memory_peak_cost::compose_branch_memory_peak`]
//! (**max dominance**, shared with branch work-cost aggregation in `lenses/cost.dag`).
//! Enforcement matches `LensEnforcement` orientation from `src/v3/std/lens_application.dag`
//! (`violates(declared_budget, observed)` ⇒ observed dominates declared).
//!
//! **Interim receipt.** Parser-level `apply_lens(cost, DeclarationScope,
//! Enforce { budget: SymbolicCost { dimension: Memory, … } })` awaits the Slice B lens-fold
//! consumer (`docs/r3-program-plan.md` gate #91).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, BindNode, DegreeAtLeastTwo, SizeVariable, SymbolicCost};
use v3_compiler::memory_peak_cost::{compose_branch_memory_peak, memory_peak_enforcement_violates};

fn find_bind<'a>(dag: &'a v3_compiler::dag::Dag, name: &str) -> &'a BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

fn run_with_cost_lens_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn memory-peak demo thread")
        .join()
        .expect("memory-peak demo thread should not panic");
}

/// Design-lock name from `docs/design-lens-application-surface.md` §4.3 (traceability anchor).
#[test]
fn memory_peak_cost_basis_demo_function_models_branch_peak_exceeding_linear_budget() {
    run_with_cost_lens_stack(|| {
        let dag = compile_to_dag(
            "fn my_memory_intensive_function(n: Int) -> Int =\n  if n <= 0 then 0 else n",
            "t_las_memory_peak_gate94.v3",
        )
        .expect("fixture compiles");

        let bind = find_bind(&dag, "my_memory_intensive_function");
        let param = bind
            .params
            .first()
            .copied()
            .expect("model function should expose one SizeVariable-bearing parameter port");

        let n = SizeVariable {
            source_port: param,
            display_name: None,
        };

        // Model two branch arms whose live peaks are quadratic vs constant —
        // memory peak for the conditional is max(O(n²), O(1)).
        let arm_heavy = SymbolicCost::PolynomialCost {
            var: n.clone(),
            degree: DegreeAtLeastTwo::TWO,
        };
        let arm_light = SymbolicCost::ConstantCost { _0: 0 };
        let observed_peak = compose_branch_memory_peak(arm_heavy, arm_light);

        // User-authored budgets are **thresholds**, not duplicated basis declarations
        // (`docs/audit/t-user-authored-cost-basis-discipline-worked-examples.md`).
        let declared_budget_mem_dim = SymbolicCost::LinearCost { _0: n };

        assert!(
            memory_peak_enforcement_violates(&declared_budget_mem_dim, &observed_peak),
            "O(n²) modeled branch peak must exceed O(n) memory budget on the same size variable \
             (budget={declared_budget_mem_dim:?}; observed_peak={observed_peak:?})",
        );
    });
}
