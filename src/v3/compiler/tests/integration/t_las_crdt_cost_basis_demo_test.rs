//! **Layer:** integration
//!
//! Gate **#93** `crdt_cost_basis_demonstrated` — CRDT per-write cost basis via the
//! symbolic-cost dimension (`analyze_symbolic_cost_dimension`) + `dominates` as the
//! stand-in for cost `LensEnforcement::violates` until `apply_lens(cost, …)` lowers
//! through the compiler fold.
//!
//! **Normalized root cost:** `analyze_symbolic_cost_dimension` on `my_crdt_field`
//! may algebra-normalize to **`LinearCost(num_writes)`** at the workflow root while
//! each per-write step still lowers from division on `replicas`. Under the MVP
//! `dominates` relation in `dag_cost_generated.rs`, **any** `LinearCost` dominates
//! **any** `LogCost`, so `dominates(my_crdt_field_composed, LogCost(replicas))` is
//! *vacuous* and is **not** asserted here.
//!
//! **What we pin instead:** (1) **`try_build_per_write_log_cost_basis_declaration`** — single-authority
//! materialization of **`CostBasisDeclaration`** from the lowered **`Dag`** (subject `DeclarationId`,
//! `PerWrite`, **`LogCost(merge_replicas_port)`**, bind **`span`**). The `.dag` carrier remains the
//! type definitions in `lenses.cost` (`lens_cost_symbolic_generated.rs`). (2) **Full symbolic-cost lens table**
//! ([`compute_symbolic_costs`]): some port still **`Hit`s `LogCost(merge_replicas_port)`**
//! from divide lowering inside `crdt_merge_step` — the per-write O(log replicas)
//! factor **survives in lens output** even though dimension `composed` at the
//! recursive workflow root normalizes to `LinearCost(num_writes)` and dimension
//! `witnesses` may only show that summary. (3) **`!dominates(basis.cost, composed)`**
//! — the per-write log budget is **not** a sound ceiling for the full workflow.
//!
//! Reading **`CostBasisDeclaration` rows from `.dag` / folding them into**
//! **`compute_symbolic_costs`** remains follow-on; **`basis.cost`** still matches the
//! **`LogCost(merge_replicas_port)`** rows `compute_symbolic_costs` assigns.
//!
//! Fixture companion: `src/v3/compiler/tests/fixtures/t_las_crdt_cost_basis_demo.dag`.
//! Design: `docs/design-lens-application-surface.md` §4.2 + cost-basis audit
//! `docs/audit/t-user-authored-cost-basis-discipline-worked-examples.md`.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{dominates, Behavior, Lookup, PortId, SizeVariable, SymbolicCost};
use v3_compiler::lens_cost_symbolic::{
    compute_symbolic_costs, CostBasisDeclaration, CostBasisKind,
};
use v3_compiler::try_build_per_write_log_cost_basis_declaration;
use v3_compiler::{analyze_symbolic_cost_dimension, DimensionReport};

use crate::common::cached_compile_to_dag;

const FIXTURE_DAG: &str = include_str!("../fixtures/t_las_crdt_cost_basis_demo.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/t_las_crdt_cost_basis_demo.dag";

/// Recursive “N writes” over a per-write step modeled as integer divide; the
/// symbolic-cost lens charges **`LogCost(dividend_port)`** on `TransformTarget::Operator(Div)`
/// (`transform_cost_for_target` in `src/v3/lenses/cost.dag`).
const MY_CRDT_FIELD: &str = "\
import std.error_primitives { DivError, Result }

fn crdt_merge_step(replicas: Int) -> Int =
  match replicas / 2 {
    Ok { value: x } => x
    Err { value: _ } => 0
  }

fn my_crdt_field(num_writes: Int, replicas: Int) -> Int =
  if num_writes == 0 then 0
  else crdt_merge_step(replicas) + my_crdt_field(num_writes - 1, replicas)

let _: Int = my_crdt_field(1, 2)
";

/// Shared `compile_to_dag` file label so `cached_compile_to_dag` hits one cell for all tests.
const CRDT_FIELD_PROGRAM_FILE: &str = "t_las_crdt_cost_basis_demo.v3";

fn run_with_symbolic_cost_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("t-las-crdt-cost-basis".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn t-las stack thread")
        .join()
        .expect("t-las thread should not panic");
}

fn find_bind<'a>(dag: &'a v3_compiler::dag::Dag, name: &str) -> &'a v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

/// `SizeVariable` per `algebra.dag` (`source_port`, `display_name: String?`); Rust
/// [`PartialEq`](v3_compiler::dag::SizeVariable) ignores `display_name` and keys on
/// `source_port` only (same as `size_variable_eq` in `.dag`).
fn size_var(port: PortId) -> SizeVariable {
    SizeVariable {
        source_port: port,
        display_name: None,
    }
}

fn symbolic_cost_lens_table_includes_log_on_port(
    dag: &v3_compiler::dag::Dag,
    port: PortId,
) -> bool {
    compute_symbolic_costs(dag).iter().any(|e| {
        matches!(
            &e.cost,
            Lookup::Hit(SymbolicCost::LogCost { _0: sv }) if sv.source_port == port
        )
    })
}

fn symbolic_cost_lens_table_includes_cost(
    dag: &v3_compiler::dag::Dag,
    want: &SymbolicCost,
) -> bool {
    compute_symbolic_costs(dag)
        .iter()
        .any(|e| matches!(&e.cost, Lookup::Hit(c) if c == want))
}

/// Declared per-op O(log replicas) budget from §4.2 (`O_log_replicas`), keyed to the
/// **`crdt_merge_step` parameter port** that is the dividend of `replicas / 2` (same logical
/// field, distinct `PortId` from `my_crdt_field`'s `replicas` parameter). The symbolic-cost
/// lens assigns `LogCost` on that port via **`TransformTarget::Operator(Div)`** in
/// `transform_cost_for_target` (`src/v3/lenses/cost.dag`).
fn per_write_log_replicas_budget(merge_replicas_port: PortId) -> SymbolicCost {
    SymbolicCost::LogCost {
        _0: size_var(merge_replicas_port),
    }
}

fn crdt_field_dimension_and_basis() -> (v3_compiler::dag::Dag, SymbolicCost, CostBasisDeclaration) {
    let dag = cached_compile_to_dag(MY_CRDT_FIELD, CRDT_FIELD_PROGRAM_FILE);
    let root = find_bind(&dag, "my_crdt_field");
    let basis =
        try_build_per_write_log_cost_basis_declaration(&dag, "my_crdt_field", "crdt_merge_step")
            .expect("fixture DAG has my_crdt_field + crdt_merge_step with a replicas param");
    let composed = match analyze_symbolic_cost_dimension(&dag, root.id) {
        DimensionReport::DimensionOk { composed, .. } => composed,
        DimensionReport::DimensionFail { violations, .. } => {
            panic!("expected DimensionOk, got failures: {violations:?}")
        }
    };
    (dag, composed, basis)
}

#[test]
fn crdt_cost_basis_fixture_dag_compiles_on_bootstrap() {
    run_with_symbolic_cost_stack(|| {
        let dag = compile_to_dag(FIXTURE_DAG, FIXTURE_PATH).expect("fixture should compile");
        assert!(
            dag.diagnostics().is_empty(),
            "fixture diagnostics: {:?}",
            dag.diagnostics()
        );
    });
}

#[test]
fn crdt_cost_basis_demonstrated_log_replicas_in_symbolic_cost_lens_table() {
    run_with_symbolic_cost_stack(|| {
        let dag = cached_compile_to_dag(MY_CRDT_FIELD, CRDT_FIELD_PROGRAM_FILE);
        let merge = find_bind(&dag, "crdt_merge_step");
        let merge_replicas_port = *merge
            .params
            .first()
            .expect("crdt_merge_step should take replicas: Int");
        assert!(
            symbolic_cost_lens_table_includes_log_on_port(&dag, merge_replicas_port),
            "expected some `SymbolicCostEntry` Hit(LogCost(merge_replicas_port)) from divide in \
             crdt_merge_step (per-write O(log replicas) survives in lens table)"
        );
    });
}

#[test]
fn crdt_cost_basis_demonstrated_basis_cost_hits_symbolic_cost_lens_table() {
    run_with_symbolic_cost_stack(|| {
        let (dag, _composed, basis) = crdt_field_dimension_and_basis();
        assert!(
            symbolic_cost_lens_table_includes_cost(&dag, &basis.cost),
            "CostBasisDeclaration.cost should match a Hit row in compute_symbolic_costs (fold \
             consumption follow-on; today `basis.cost` **is** the same SymbolicCost the lens \
             lowers for that port); basis.cost={:?}",
            basis.cost
        );
    });
}

#[test]
fn crdt_cost_basis_demonstrated_declaration_pins_subject_span_and_per_write_log_budget() {
    run_with_symbolic_cost_stack(|| {
        let (dag, _composed, basis) = crdt_field_dimension_and_basis();
        let root = find_bind(&dag, "my_crdt_field");
        let merge = find_bind(&dag, "crdt_merge_step");
        let merge_replicas_port = *merge
            .params
            .first()
            .expect("crdt_merge_step should take replicas: Int");
        let decl = dag
            .declaration_by_name("my_crdt_field")
            .expect("my_crdt_field declaration");
        assert_eq!(basis.subject, decl.id);
        assert!(matches!(basis.kind, CostBasisKind::PerWrite));
        assert_eq!(
            basis.cost,
            per_write_log_replicas_budget(merge_replicas_port)
        );
        assert_eq!(basis.span, root.span);
    });
}

#[test]
fn crdt_cost_basis_demonstrated_per_op_budget_is_not_sound_ceiling_for_composed_workflow() {
    run_with_symbolic_cost_stack(|| {
        let (_dag, composed, basis) = crdt_field_dimension_and_basis();
        assert!(
            !dominates(&basis.cost, &composed),
            "per-op basis.cost (declared Log replicas) must not dominate full composed cost \
             (otherwise enforcement could not fire); composed={composed:?} basis.cost={:?}",
            basis.cost
        );
    });
}

#[test]
fn crdt_cost_basis_demonstrated_unknown_ceiling_covers_composed_workflow() {
    run_with_symbolic_cost_stack(|| {
        let (_dag, composed, _) = crdt_field_dimension_and_basis();

        let loose = SymbolicCost::UnknownCost {
            _0: "demo ceiling / opt-in slack budget".to_string(),
        };
        assert!(
            dominates(&loose, &composed),
            "an honest upper-bound budget should dominate (soundly majorize) the composed cost; \
             composed={composed:?}"
        );
    });
}
