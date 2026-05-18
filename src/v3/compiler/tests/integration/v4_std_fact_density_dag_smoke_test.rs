//! **Layer:** integration
//!
//! **P2 / Practice 5 (single authority):** This harness proves **parse + inference cleanliness**
//! for the nominal `SourceSpecReadFact` carrier only (`compile_to_dag`, empty diagnostics) — it
//! does **not** claim a **generated** substrate consumer for that type (INVARIANTS §P2: declaration
//! without generated consumer = staging). The Practice-8 hollow predicate’s authority remains the
//! handwritten mirror `src/v3/compiler/src/v4_hollow_alias_gate.rs` (**private** `mod` in
//! `v3_compiler`, not `pub` API; `#[cfg_attr(not(test), allow(dead_code))]` in that file until a
//! production consumer exists) until the generated `.dag` checker replaces it (`INVARIANTS.md`
//! §P5(b)
//! dissolution on that path). **Interim-floor authority** is
//! `docs/modeling-discipline.md` Practice 8 (landed on `main`: **#3226** `77b9e7d72`;
//! Practice 9 **#3234** `125fc88c8`).
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
