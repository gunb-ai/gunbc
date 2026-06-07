//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/frameworks/react.dag` —
//! T-4.7 React framework substrate must lower+infer with **zero** module
//! diagnostics (same 0-diag gate as `v4_extdeps_typescript_dag_smoke_test`).
//!
//! **White-box sweep (operator 2026-06-07):** declaration-shape pin slices deleted —
//! the `.dag` model is the authority; structural receipts ride claim-run witnesses.
//! This harness retains only the **0-diag compile** consumer.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const REACT_DAG: &str = include_str!("../../../../v4/extdeps/frameworks/react.dag");
const REACT_PATH: &str = "src/v4/extdeps/frameworks/react.dag";

/// Panics unless `react.dag` compiles with **zero** module diagnostics.
fn react_extdeps_dag_or_panic() -> v3_compiler::Dag {
    match compile_to_dag(REACT_DAG, REACT_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{REACT_PATH}: expected empty diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{REACT_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{REACT_PATH}: {other:?}"),
    }
}

#[test]
fn v4_extdeps_react_dag_compiles() {
    let _dag = react_extdeps_dag_or_panic();
}
