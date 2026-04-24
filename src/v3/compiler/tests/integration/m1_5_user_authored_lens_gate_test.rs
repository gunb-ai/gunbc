//! **Layer:** integration
//!
//! Day-1 R1 gate `user_authored_lens_compiles`: the gate fixture carries a
//! `TestClaim` whose `source` / `file_name` are the single authority for what a
//! future runner would compile; this module asserts lockstep with the on-disk
//! lens file, then compiles that payload via `compile_to_dag` on top of the
//! standard bootstrap context (`Dag::new()`), without bundling the lens into the
//! bootstrap.
//!
//! **Behavior receipt (TESTING.md):** `user_authored_lens_testclaim_payload_tracks_on_disk_lens_and_compiles`
//! lowers the gate fixture, reads `source` off the lowered `TestClaim`, and runs
//! `compile_to_dag` on that string — this is the executable `Compiles` path, not
//! merely “the record literal typechecks.” The payload is the **entire**
//! `lenses.named_function_count` module text (same bytes as the `.dag` file); it
//! intentionally does **not** use `import lenses.named_function_count { ... }`
//! from a second file, because that pattern would require the lens to live in
//! `Dag::new()` bootstrap, which this demo deliberately avoids.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody};
use v3_compiler::CompileError;

const R1_GATES_SOURCE: &str = include_str!("../fixtures/r1_gates.dag");
const ON_DISK_LENS: &str = include_str!("../../../lenses/named_function_count.dag");
const ON_DISK_LENS_COMPOSITION_WITNESS: &str =
    include_str!("../../../lenses/lens_composition_associative_witness.dag");

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
fn r1_gates_fixture_compiles_against_bootstrap_context() {
    assert_compile_clean(
        R1_GATES_SOURCE,
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
        "gate fixture `r1_gates.dag`",
    );
}

#[test]
fn user_authored_lens_testclaim_payload_tracks_on_disk_lens_and_compiles() {
    let gate_dag = match compile_to_dag(
        R1_GATES_SOURCE,
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
    ) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(d)) => panic!(
            "gate fixture compile failed: {:?}",
            d.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("gate fixture: unexpected compile error: {other:?}"),
    };
    assert!(
        gate_dag.diagnostics().is_empty(),
        "gate fixture should load cleanly, got {:?}",
        gate_dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let source = testclaim_string_field(&gate_dag, "user_authored_lens_compiles_gate", "source");
    let file_name =
        testclaim_string_field(&gate_dag, "user_authored_lens_compiles_gate", "file_name");

    assert_eq!(
        source, ON_DISK_LENS,
        "`TestClaim.source` must stay byte-identical to `src/v3/lenses/named_function_count.dag` \
         (single authority for the user lens program; update both together)"
    );
    assert_eq!(
        file_name, "src/v3/lenses/named_function_count.dag",
        "`TestClaim.file_name` should name the on-disk lens path for the Day-1 witness"
    );

    assert_compile_clean(
        &source,
        &file_name,
        "TestClaim `source` payload (read from gate fixture)",
    );
}

#[test]
fn lens_composition_associative_testclaim_source_tracks_on_disk_witness() {
    let gate_dag = match compile_to_dag(
        R1_GATES_SOURCE,
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
    ) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(d)) => panic!(
            "gate fixture compile failed: {:?}",
            d.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("gate fixture: unexpected compile error: {other:?}"),
    };
    assert!(
        gate_dag.diagnostics().is_empty(),
        "gate fixture should load cleanly, got {:?}",
        gate_dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let source = testclaim_string_field(&gate_dag, "lens_composition_associative_gate", "source");
    let file_name =
        testclaim_string_field(&gate_dag, "lens_composition_associative_gate", "file_name");

    assert_eq!(
        source, ON_DISK_LENS_COMPOSITION_WITNESS,
        "`TestClaim.source` must stay byte-identical to \
         `src/v3/lenses/lens_composition_associative_witness.dag` \
         (single authority for the witness program; update both together)"
    );
    assert_eq!(
        file_name, "src/v3/lenses/lens_composition_associative_witness.dag",
        "`TestClaim.file_name` should name the on-disk witness path"
    );

    assert_compile_clean(
        &source,
        &file_name,
        "TestClaim `source` payload (lens_composition_associative witness)",
    );
}
