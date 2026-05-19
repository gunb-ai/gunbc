//! **Layer:** integration
//!
//! **P2 / Practice 5 (single authority):** This harness proves **parse + inference cleanliness**
//! for `src/v4/lens/registry.dag` only (`compile_to_dag`, empty diagnostics) — it does **not**
//! claim a **generated** substrate consumer for `LensRegistryEntryV0` / `LensModulePathV0`
//! (INVARIANTS §P2: declaration without generated consumer = staging; see `STRUCTURE.md` and
//! `docs/briefs/r4-lane-a-lens-interface-freeze-pin.md` §3). The operator pin’s §3 markdown
//! table remains a human mirror until a mechanical reader lands.
//!
//! **INVARIANTS §P5 Dispatch-Discipline Mechanism (b):** this path’s SG-0 census line + matching
//! `INVARIANTS.md` table row land in the same PR as the harness (home-of-record for the
//! hand-Rust receipt).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const REGISTRY_DAG: &str = include_str!("../../../../v4/lens/registry.dag");
const REGISTRY_PATH: &str = "src/v4/lens/registry.dag";

#[test]
fn v4_lens_registry_dag_compiles() {
    match compile_to_dag(REGISTRY_DAG, REGISTRY_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{REGISTRY_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{REGISTRY_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{REGISTRY_PATH}: {other:?}"),
    }
}
