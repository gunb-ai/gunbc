//! **Layer:** integration
//!
//! Day-1 R1 gate `user_authored_lens_compiles`: **canonical** program text is
//! `src/v3/lenses/named_function_count.dag`. The gate fixture `r1_gates.dag` is
//! **build-generated** from `r1_gates.template.dag` (`v3-compiler/build.rs` splices the
//! lens bytes into `TestClaim.source`). This module ratchets the lowered `source` field
//! against `include_str!(.../named_function_count.dag)` then runs `compile_to_dag` on
//! that payload over the standard bootstrap (`Dag::new()`), without bundling the lens
//! into the bootstrap.
//!
//! **Behavior receipt (TESTING.md):** `user_authored_lens_testclaim_payload_tracks_on_disk_lens_and_compiles`
//! lowers the gate fixture, reads `source` off the lowered `TestClaim`, and runs
//! `compile_to_dag` on that string — this is the executable `Compiles` path, not
//! merely “the record literal typechecks.” The lowered payload must match the on-disk
//! lens byte-for-byte. The fixture does **not** use `import lenses.named_function_count { ... }`
//! from a second file, because that pattern would require the lens in `Dag::new()`
//! bootstrap, which this demo deliberately avoids.
//!
//! D4 splice: ROADMAP.md "Scheduled cleanups: LensOutputEquals runner and R1 gate fixtures" item 3.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody};
use v3_compiler::CompileError;

const R1_GATES_SOURCE: &str = include_str!("../fixtures/r1_gates.dag");
const R1_GATES_TEMPLATE: &str = include_str!("../fixtures/r1_gates.template.dag");
const R1_MOCK_BACKED_INVARIANT_GATE_SOURCE: &str =
    include_str!("../fixtures/r1_mock_backed_invariant_gate.dag");
const ON_DISK_LENS: &str = include_str!("../../../lenses/named_function_count.dag");

/// Parallel-copy ratchet: must match `src/v3/lenses/named_function_count.dag` lines 15–25 and
/// the stub `fn` block in `r1_gates.template.dag` (LensOutputEquals evaluates these fixture decls).
const NAMED_FUNCTION_COUNT_FIXTURE_FN_BLOCK: &str = "fn count_named_bind(behavior: Behavior) -> Int =\n  match behavior {\n    Value(v) => 0\n    Transform(t) => 0\n    Branch(b) => 0\n    Loop(l) => 0\n    Bind(bind) => if bind.name == \"\" then 0 else 1\n  }\n\nfn named_function_count(d: Dag) -> Int =\n  fold(d.nodes, 0, |acc, behavior| acc + count_named_bind(behavior))";
const ON_DISK_LENS_COMPOSITION_WITNESS: &str =
    include_str!("../../../lenses/lens_composition_associative_witness.dag");

// `r1_gates.dag` carries a parallel `fn lens_composition_op` for `DeclarationRef` lowering;
// the runner checks the witness by name in `program_dag` only, but the bodies must not drift.
const LENS_COMPOSITION_OP_DEF: &str = "fn lens_composition_op(a: Int, b: Int) -> Int = a + b";

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
fn r1_mock_backed_invariant_gate_fixture_compiles_against_bootstrap_context() {
    assert_compile_clean(
        R1_MOCK_BACKED_INVARIANT_GATE_SOURCE,
        "src/v3/compiler/tests/fixtures/r1_mock_backed_invariant_gate.dag",
        "gate fixture `r1_mock_backed_invariant_gate.dag`",
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
        "`TestClaim.source` in generated `r1_gates.dag` must match the canonical \
         `src/v3/lenses/named_function_count.dag` (build.rs splice ratchet)"
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

#[test]
fn r1_gates_template_named_function_count_stubs_lockstep_on_disk_lens() {
    assert!(
        ON_DISK_LENS.contains(NAMED_FUNCTION_COUNT_FIXTURE_FN_BLOCK),
        "on-disk `named_function_count.dag` should still carry the stub `fn` block this test pins"
    );
    assert!(
        R1_GATES_TEMPLATE.contains(NAMED_FUNCTION_COUNT_FIXTURE_FN_BLOCK),
        "`r1_gates.template.dag` fixture stubs must stay byte-identical to the canonical lens \
         (LensOutputEquals applies the fixture DAG, not `program_dag`, until DeclarationRef cleanup)"
    );
}

#[test]
fn lens_composition_associative_r1_gates_stub_locksteps_witness_operator_line() {
    // Byte-identical full line (not a substring match): keeps the parallel `fn lens_composition_op`
    // in `r1_gates.dag` and `lens_composition_associative_witness.dag` from drifting apart.
    for (label, source) in [
        (
            "src/v3/lenses/lens_composition_associative_witness.dag",
            ON_DISK_LENS_COMPOSITION_WITNESS,
        ),
        (
            "src/v3/compiler/tests/fixtures/r1_gates.dag",
            R1_GATES_SOURCE,
        ),
    ] {
        assert!(
            source.lines().any(|line| line == LENS_COMPOSITION_OP_DEF),
            "{label} must include a source line exactly equal to the shared stub:\n{LENS_COMPOSITION_OP_DEF:?}"
        );
    }
}
