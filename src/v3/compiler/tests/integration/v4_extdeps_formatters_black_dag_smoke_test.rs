//! Smoke `compile_to_dag` on `src/v4/extdeps/formatters/black.dag` — T-4.16
//! `ConfigPatchRecord` / `config_patch_layer` consumers must lower+infer with zero
//! module diagnostics (no `apply_field_patch` in consumer imports).
//!
//! Single-file [`compile_to_dag`] cannot load `v4.std.patch` peers; this harness
//! lowers `node` → `algebra` → `patch` → `black` in order (flat declaration table).
//!
//! **White-box sweep (operator 2026-06-07):** declaration-shape pin slices deleted —
//! the `.dag` model is the authority. This harness retains only the **0-diag compile**
//! consumer.

use v3_compiler::compile_to_dag_modules_in_order;
use v3_compiler::CompileError;

const NODE_DAG: &str = include_str!("../../../../v4/std/node.dag");
const NODE_PATH: &str = "src/v4/std/node.dag";
const ALGEBRA_DAG: &str = include_str!("../../../../v4/std/algebra.dag");
const ALGEBRA_PATH: &str = "src/v4/std/algebra.dag";
const PATCH_DAG: &str = include_str!("../../../../v4/std/patch.dag");
const PATCH_PATH: &str = "src/v4/std/patch.dag";
const BLACK_DAG: &str = include_str!("../../../../v4/extdeps/formatters/black.dag");
const BLACK_PATH: &str = "src/v4/extdeps/formatters/black.dag";

fn black_dag_or_panic() -> v3_compiler::dag::Dag {
    let sources = [
        (NODE_DAG, NODE_PATH),
        (ALGEBRA_DAG, ALGEBRA_PATH),
        (PATCH_DAG, PATCH_PATH),
        (BLACK_DAG, BLACK_PATH),
    ];
    match compile_to_dag_modules_in_order(&sources) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "{BLACK_PATH}: semantic errors: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(other) => panic!("{BLACK_PATH}: {other:?}"),
    }
}

#[test]
fn v4_extdeps_formatters_black_dag_compiles_with_zero_diagnostics() {
    let dag = black_dag_or_panic();
    assert!(
        dag.diagnostics().is_empty(),
        "{BLACK_PATH}: expected empty diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}
