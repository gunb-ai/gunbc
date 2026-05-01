//! **Layer:** integration
//!
//! R3 downstream consumer fixtures for substrate-owned structural equality
//! predicates. These tests compile the authored `.dag` consumers and assert
//! their current runner disposition without duplicating their substrate facts.

use v3_compiler::dag::{FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

const BRIDGE_LEDGER_ZERO_SOURCE: &str =
    include_str!("../fixtures/r3_bridge_retirement_ledger_zero.dag");
const BRIDGE_LEDGER_ZERO_PATH: &str =
    "src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag";

const RUST_DAG_ISOMORPHISM_SOURCE: &str =
    include_str!("../fixtures/rust_dag_isomorphism_consumer.dag");
const RUST_DAG_ISOMORPHISM_PATH: &str =
    "src/v3/compiler/tests/fixtures/rust_dag_isomorphism_consumer.dag";

fn compile_fixture(source: &str, path: &str) -> Result<v3_compiler::dag::Dag, CompileError> {
    compile_to_dag(source, path)
}

fn record_field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> &'a FieldValue {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("record missing `{label}` field"))
}

fn string_literal(value: &FieldValue) -> &str {
    match value {
        FieldValue::Literal(LiteralBits::String(s)) => s.as_str(),
        other => panic!("expected String literal, got {other:?}"),
    }
}

fn bridge_ledger_open_row_names() -> Vec<String> {
    let dag = generated_full_bootstrap_dag();
    let retired_constructor = {
        let bridge_status = dag
            .declaration_by_name("BridgeStatus")
            .expect("BridgeStatus missing from full bootstrap");
        let TypeConnective::Disj { variants } = &bridge_status.connective else {
            panic!("BridgeStatus is not a Disj");
        };
        variants
            .iter()
            .find(|variant| variant.label == "Retired")
            .expect("Retired variant missing")
            .ty
    };
    let bridge_ledger = dag
        .declaration_by_name("bridge_ledger")
        .expect("bridge_ledger missing from full bootstrap");
    let Some(ValueBody::List(rows)) = &bridge_ledger.value_body else {
        panic!("bridge_ledger must lower as a List value body");
    };

    rows.iter()
        .filter_map(|row| {
            let FieldValue::Record(fields) = row else {
                panic!("bridge_ledger row is not a record: {row:?}");
            };
            let constructor = match record_field(fields, "status") {
                FieldValue::Variant { constructor, .. } => *constructor,
                other => panic!("bridge_ledger status is not a variant: {other:?}"),
            };
            if constructor == retired_constructor {
                None
            } else {
                Some(string_literal(record_field(fields, "name")).to_string())
            }
        })
        .collect()
}

#[test]
fn r3_bridge_retirement_ledger_zero_fixture_reports_open_rows_at_head() {
    let dag = compile_fixture(BRIDGE_LEDGER_ZERO_SOURCE, BRIDGE_LEDGER_ZERO_PATH)
        .expect("bridge ledger zero fixture compiles");
    let results = TestRunner::new(&dag).run_suite("r3_bridge_retirement_ledger_zero_suite");
    assert_eq!(results.len(), 1);
    let reason = match &results[0].result {
        ClaimResult::Fail(reason) => reason,
        other => panic!("expected bridge ledger zero to be red at HEAD; got {other:?}"),
    };

    let open_rows = bridge_ledger_open_row_names();
    assert!(
        !open_rows.is_empty(),
        "when the canonical bridge ledger reaches zero open rows, re-arm this \
         fixture expectation as a Pass ratchet in the same PR"
    );
    for row in open_rows {
        assert!(
            reason.contains(&row),
            "BridgeLedgerZero diagnostic must name open row `{row}`; got: {reason}"
        );
    }
}

#[test]
fn rust_dag_isomorphism_consumer_reaches_binary_report_shape_gate() {
    let dag = compile_fixture(RUST_DAG_ISOMORPHISM_SOURCE, RUST_DAG_ISOMORPHISM_PATH)
        .expect("RustDagIsomorphism consumer fixture compiles");
    let results = TestRunner::new(&dag).run_suite("rust_dag_isomorphism_consumer_suite");
    assert_eq!(results.len(), 1);
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::NotYetImplemented(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("structural shape is valid")
                    && reason.contains("RustEnumExtractionDagShapeReport")
                    && reason.contains("DagReflectionDagShapeReport")
        ),
        "expected RustDagIsomorphism consumer to reach the current \
         BinaryDimensionReportEquals shape-valid path; got {:?}",
        results[0].result
    );
}
