//! **Layer:** integration
//!
//! Gate **#93** `crdt_cost_basis_demonstrated` — CRDT per-write cost basis via the
//! symbolic-cost dimension (`analyze_symbolic_cost_dimension`) + `dominates` as the
//! stand-in for cost `LensEnforcement::violates` until `apply_lens(cost, …)` lowers
//! through the compiler fold.
//!
//! **Normalized root cost:** the dimension report’s `composed` carrier is
//! algebra-normalized at the workflow root. For this fixture it is often
//! `LinearCost(num_writes)` even though each per-write step lowers from division on
//! `replicas` (`LogCost` in the cost lens). The gate still witnesses §4.2 via
//! `dominates(composed, per_op)` for `per_op = LogCost(replicas)` — the composed bound
//! majorizes the per-op O(log replicas) budget after loop multiplication.
//!
//! Fixture companion: `src/v3/compiler/tests/fixtures/t_las_crdt_cost_basis_demo.dag`.
//! Design: `docs/design-lens-application-surface.md` §4.2 + cost-basis audit
//! `docs/audit/t-user-authored-cost-basis-discipline-worked-examples.md`.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{dominates, Behavior, PortId, SizeVariable, SymbolicCost};
use v3_compiler::{analyze_symbolic_cost_dimension, DimensionReport};

use crate::common::cached_compile_to_dag;

const FIXTURE_DAG: &str = include_str!("../fixtures/t_las_crdt_cost_basis_demo.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/t_las_crdt_cost_basis_demo.dag";

/// Recursive “N writes” over a per-write step modeled as integer divide
/// (`ArithmeticDivideCall` → `LogCost` in the cost lens; see `lens_cost_generated`).
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

/// `SizeVariable` substrate: `source_port` + `display_name: String?`
/// (`src/v3/std/algebra.dag`). `None` is the ordinary unnamed witness, matching
/// `lane2_stage_2d_symbolic_cost_test::size_var` / `unnamed_size_variable` in `.dag`.
fn size_var(port: PortId) -> SizeVariable {
    SizeVariable {
        source_port: port,
        display_name: None,
    }
}

/// Declared per-op O(log replicas) budget from §4.2 (`O_log_replicas`), keyed to the
/// `replicas` parameter port. The cost lens uses this same size variable for
/// `ArithmeticDivideCall` on `replicas` inside `crdt_merge_step`.
fn per_write_log_replicas_budget(replicas_port: PortId) -> SymbolicCost {
    SymbolicCost::LogCost {
        _0: size_var(replicas_port),
    }
}

fn crdt_field_dimension_ok() -> (v3_compiler::dag::Dag, SymbolicCost, SymbolicCost) {
    let dag = cached_compile_to_dag(MY_CRDT_FIELD, CRDT_FIELD_PROGRAM_FILE);
    let root = find_bind(&dag, "my_crdt_field");
    let replicas_port = *root
        .params
        .get(1)
        .expect("my_crdt_field should have replicas parameter port");
    let per_op = per_write_log_replicas_budget(replicas_port);
    let composed = match analyze_symbolic_cost_dimension(&dag, root.id) {
        DimensionReport::DimensionOk { composed, .. } => composed,
        DimensionReport::DimensionFail { violations, .. } => {
            panic!("expected DimensionOk, got failures: {violations:?}")
        }
    };
    (dag, composed, per_op)
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
fn crdt_cost_basis_demonstrated_composed_exceeds_per_op_log_budget() {
    run_with_symbolic_cost_stack(|| {
        let (_dag, composed, per_op) = crdt_field_dimension_ok();
        assert!(
            dominates(&composed, &per_op),
            "composed workflow cost should **exceed** a per-op O(log replicas) budget \
             when num_writes is a size variable (N writes × per-write log); \
             composed={composed:?} per_op={per_op:?}"
        );
    });
}

#[test]
fn crdt_cost_basis_demonstrated_per_op_budget_is_not_sound_ceiling_for_composed_workflow() {
    run_with_symbolic_cost_stack(|| {
        let (_dag, composed, per_op) = crdt_field_dimension_ok();
        assert!(
            !dominates(&per_op, &composed),
            "per-op budget must not dominate full composed cost (otherwise enforcement could not fire); \
             composed={composed:?} per_op={per_op:?}"
        );
    });
}

#[test]
fn crdt_cost_basis_demonstrated_unknown_ceiling_covers_composed_workflow() {
    run_with_symbolic_cost_stack(|| {
        let dag = cached_compile_to_dag(MY_CRDT_FIELD, CRDT_FIELD_PROGRAM_FILE);
        let root = find_bind(&dag, "my_crdt_field");
        let composed = match analyze_symbolic_cost_dimension(&dag, root.id) {
            DimensionReport::DimensionOk { composed, .. } => composed,
            DimensionReport::DimensionFail { violations, .. } => {
                panic!("expected DimensionOk, got failures: {violations:?}")
            }
        };

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
