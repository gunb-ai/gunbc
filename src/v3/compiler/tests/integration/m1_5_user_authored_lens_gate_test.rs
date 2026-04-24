//! **Layer:** integration
//!
//! Day-1 R1 gate `user_authored_lens_compiles`: a user `.dag` lens under
//! `src/v3/lenses/` resolves in the bootstrap `Dag`, and a minimal `.v3`
//! fixture that calls it compiles cleanly.

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::CompileError;

fn bootstrapped_dag() -> &'static Dag {
    static DAG: OnceLock<Dag> = OnceLock::new();
    DAG.get_or_init(Dag::new)
}

const USER_LENS_FIXTURE_V3: &str = "\
import lenses.named_function_count { named_function_count }
import std.substrate { Dag }

fn count_fns(d: Dag) -> Int = named_function_count(d)
";

#[test]
fn user_authored_lens_compiles_fixture() {
    let dag = bootstrapped_dag();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load staged std + user lens cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    assert!(
        dag.declaration_by_name("named_function_count").is_some(),
        "bootstrap Dag should expose `named_function_count` from lenses.named_function_count"
    );
    assert!(
        dag.declaration_by_name("user_authored_lens_compiles_gate")
            .is_some(),
        "bootstrap Dag should load `user_authored_lens_compiles_gate` from std.r1_gates"
    );

    match compile_to_dag(USER_LENS_FIXTURE_V3, "user_lens_fixture.v3") {
        Ok(compiled) => assert!(
            compiled.diagnostics().is_empty(),
            "user lens fixture should compile with no diagnostics, got {:?}",
            compiled.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(d)) => panic!(
            "fixture compile failed: {:?}",
            d.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error: {other:?}"),
    }
}
