//! R3 gate #62 — `substrate_gap_file_ingestion_closed` plumbing receipt.
//!
//! Exercises compile-time `read_utf8_file("…")` lowering (UTF-8 relative to the
//! compilation unit path) plus a checked-in `TestClaim` + `TestRunner` Compiles pass.

use std::path::PathBuf;

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_DAG: &str = include_str!("../fixtures/r3_substrate_gap_file_ingestion.dag");

fn fixture_v3_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/r3_substrate_gap_file_ingestion_external.v3")
}

fn fixture_dag_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/r3_substrate_gap_file_ingestion.dag")
        .to_string_lossy()
        .into_owned()
}

const SUITE_NAME: &str = "r3_substrate_gap_file_ingestion_suite";
const CLAIM_NAME: &str = "substrate_gap_file_ingestion_closed";

#[test]
fn r3_gate_62_file_ingestion_program_compiles_cleanly() {
    let path = fixture_v3_path();
    let v3_source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let file = path.to_string_lossy();
    let dag = compile_to_dag(&v3_source, file.as_ref()).unwrap_or_else(|err| {
        panic!("expected clean compile for {}: {err:?}", path.display());
    });
    assert!(
        dag.diagnostics().is_empty(),
        "{}: expected empty diagnostics, got {:?}",
        path.display(),
        dag.diagnostics()
    );
}

#[test]
fn r3_gate_62_file_ingestion_rejects_parent_dir_in_path() {
    let path = fixture_v3_path();
    let file = path.to_string_lossy();
    let bad = "data _: String = read_utf8_file(\"../secrets.txt\")\n";
    let err = compile_to_dag(bad, file.as_ref()).expect_err("expected `..` rejection");
    let CompileError::Semantic(dag) = err else {
        panic!("expected Semantic(Dag), got {err:?}");
    };
    assert!(
        dag.diagnostics().iter().any(|(_, d)| {
            d.message()
                .contains("`read_utf8_file` path must not contain `..`")
        }),
        "expected ParentDir diagnostic, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn r3_gate_62_substrate_gap_file_ingestion_claim_passes() {
    let dag_path = fixture_dag_path();
    let dag = match compile_to_dag(FIXTURE_DAG, dag_path.as_str()) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{dag_path}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{dag_path} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {dag_path}: {other:?}"),
    };

    let results = TestRunner::new(&dag).run_suite(SUITE_NAME);
    let result = results
        .iter()
        .find(|r| r.claim_name == CLAIM_NAME)
        .unwrap_or_else(|| panic!("missing `{CLAIM_NAME}` in `{SUITE_NAME}` results: {results:?}"));
    assert!(
        matches!(result.result, ClaimResult::Pass),
        "`{CLAIM_NAME}` should Pass (Compiles over read_utf8_file program), got {:?}",
        result.result
    );
}
