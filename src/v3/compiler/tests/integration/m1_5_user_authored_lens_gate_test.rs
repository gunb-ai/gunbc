//! **Layer:** integration
//!
//! Day-1 R1 gate `user_authored_lens_compiles`: **canonical** program text is
//! `src/v3/lenses/named_function_count.dag`. The gate fixture embeds a duplicate
//! `TestClaim.source` string (P2 parallel copy — see `r1_gates.dag` header); this
//! module ratchets fixture bytes against `include_str!(.../named_function_count.dag)`
//! then runs `compile_to_dag` on the extracted `source` over the standard bootstrap
//! (`Dag::new()`), without bundling the lens into the bootstrap.
//!
//! **Behavior receipt (TESTING.md):** `user_authored_lens_testclaim_payload_tracks_on_disk_lens_and_compiles`
//! lowers the gate fixture, reads `source` off the lowered `TestClaim`, and runs
//! `compile_to_dag` on that string — this is the executable `Compiles` path, not
//! merely “the record literal typechecks.” The lowered payload must match the on-disk
//! lens byte-for-byte. The fixture does **not** use `import lenses.named_function_count { ... }`
//! from a second file, because that pattern would require the lens in `Dag::new()`
//! bootstrap, which this demo deliberately avoids.
//!
//! TODO(dissolve: P2 parallel copy): same triggers as `r1_gates.dag` header (generated /
//! path-backed single authority, or runner-resolved lens via `DeclarationRef` from the
//! bootstrap DAG — then delete this lockstep ratchet).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody};
use v3_compiler::CompileError;

const R1_GATES_SOURCE: &str = include_str!("../fixtures/r1_gates.dag");
const R1_LENS_OUTPUT_EQUALS_GATE_SOURCE: &str =
    include_str!("../fixtures/r1_lens_output_equals_gate.dag");
const ON_DISK_LENS: &str = include_str!("../../../lenses/named_function_count.dag");

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
fn r1_lens_output_equals_gate_fixture_compiles_against_bootstrap_context() {
    assert_compile_clean(
        R1_LENS_OUTPUT_EQUALS_GATE_SOURCE,
        "src/v3/compiler/tests/fixtures/r1_lens_output_equals_gate.dag",
        "gate fixture `r1_lens_output_equals_gate.dag`",
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
        "`TestClaim.source` in `r1_gates.dag` must stay byte-identical to the canonical \
         `src/v3/lenses/named_function_count.dag` (P2 parallel copy ratchet; dissolve when \
         the fixture no longer duplicates lens bytes — see TODO in `r1_gates.dag`)"
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
