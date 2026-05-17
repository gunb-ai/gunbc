//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/std/fact_density.dag` — T-30 substrate must
//! lower+infer with **zero** module diagnostics (same bar as
//! `v4_extdeps_typescript_dag_smoke_test`). Practice-8 structural mirror:
//! `v3_compiler::v4_hollow_alias_gate`.
//!
//! **INVARIANTS §P5 Dispatch-Discipline Mechanism (b):** this path’s SG-0 census
//! line + matching `INVARIANTS.md` table row land in the same PR as the harness
//! (home-of-record for the hand-Rust receipt).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const FACT_DENSITY_DAG: &str = include_str!("../../../../v4/std/fact_density.dag");
const FACT_DENSITY_PATH: &str = "src/v4/std/fact_density.dag";

#[test]
fn v4_std_fact_density_dag_compiles() {
    match compile_to_dag(FACT_DENSITY_DAG, FACT_DENSITY_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{FACT_DENSITY_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{FACT_DENSITY_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{FACT_DENSITY_PATH}: {other:?}"),
    }
}
