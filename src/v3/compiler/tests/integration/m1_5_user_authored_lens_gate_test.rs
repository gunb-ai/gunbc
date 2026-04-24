//! **Layer:** integration
//!
//! Day-1 R1 gate `user_authored_lens_compiles`: a user `.dag` lens under
//! `src/v3/lenses/bootstrap/` resolves in the bootstrap `Dag`, and the staged
//! `TestClaim` compiles the same `source` / `file_name` the gate declares.

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody};
use v3_compiler::CompileError;

/// Two tests share one `Dag::new()` clone — bootstrap is large enough that
/// repeating a cold clone per `#[test]` is noticeable on integration binaries.
fn bootstrapped_dag() -> &'static Dag {
    static DAG: OnceLock<Dag> = OnceLock::new();
    DAG.get_or_init(Dag::new)
}

fn testclaim_string_field(dag: &Dag, gate_name: &str, label: &str) -> String {
    let decl = dag
        .declaration_by_name(gate_name)
        .unwrap_or_else(|| panic!("missing declaration `{gate_name}`"));
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        panic!("`{gate_name}` should carry a structural value body");
    };
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{gate_name}` missing String field `{label}`"))
}

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

    let source = testclaim_string_field(dag, "user_authored_lens_compiles_gate", "source");
    let file_name = testclaim_string_field(dag, "user_authored_lens_compiles_gate", "file_name");

    match compile_to_dag(&source, &file_name) {
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

#[test]
fn r1_gates_dag_stages_against_bootstrap_snapshot() {
    let dag = bootstrapped_dag();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load `src/v3/std/r1_gates.dag` (staged after `verification.dag` via `build.rs`) with no diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let gate = dag
        .declaration_by_name("user_authored_lens_compiles_gate")
        .expect("`user_authored_lens_compiles_gate` should stage from std.r1_gates");
    assert_eq!(
        gate.span.file.as_str(),
        "src/v3/std/r1_gates.dag",
        "gate declaration should retain r1_gates.dag provenance for the Day-1 witness"
    );
}
