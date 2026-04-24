//! **Layer:** integration
//!
//! Day-1 R1 gate `user_authored_lens_compiles`: a user `.dag` lens and the gate
//! declaration compile via `compile_to_dag` on top of the standard bootstrap
//! context (`Dag::new()`), **without** bundling either file into the bootstrap.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

// Lens compiles against the bootstrap context — not inside it.
const USER_LENS_SOURCE: &str = include_str!("../../../lenses/named_function_count.dag");

// Gate declaration compiles against the bootstrap context (test fixture).
const R1_GATES_SOURCE: &str = include_str!("../fixtures/r1_gates.dag");

fn assert_compile_clean(source: &str, file_name: &str, label: &str) {
    match compile_to_dag(source, file_name) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{label} should compile with no diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(d)) => panic!(
            "{label} compile failed: {:?}",
            d.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{label}: unexpected compile error: {other:?}"),
    }
}

#[test]
fn user_authored_lens_dag_compiles_against_bootstrap_context() {
    assert_compile_clean(
        USER_LENS_SOURCE,
        "src/v3/lenses/named_function_count.dag",
        "user lens `named_function_count.dag`",
    );
}

#[test]
fn r1_gates_fixture_compiles_against_bootstrap_context() {
    assert_compile_clean(
        R1_GATES_SOURCE,
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
        "gate fixture `r1_gates.dag`",
    );
}
