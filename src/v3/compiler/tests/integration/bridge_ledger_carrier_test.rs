//! **Layer:** integration
//!
//! T-Verification-BridgeLedger carrier shape ratchet for
//! `src/v3/std/bridge_ledger.dag`. Pins the substrate facts
//! Verification's `BridgeLedgerZero` `.dag` `TestClaim` will fold:
//!
//! - `BridgeLedgerRow` is a record carrying `{ name, owner, status,
//!   authority }`.
//! - `BridgeStatus` is a closed two-variant coproduct (`Retired` /
//!   `Open`); no stringly status, no third state.
//! - `bridge_ledger` lowers as `List<BridgeLedgerRow>` with exactly the
//!   five canonical bridge names from `docs/r3-structure.md:79-83`.
//! - Each row's `status` resolves to one of the two `BridgeStatus`
//!   constructors structurally (not a string check).

use std::collections::{BTreeSet, HashSet};
use v3_compiler::dag::{Dag, FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

const BRIDGE_LEDGER: &str = "bridge_ledger";

const CANONICAL_BRIDGES: &[&str] = &[
    "bridge_source_span_file_participation_retired",
    "bridge_mark_bootstrap_secret_nominal_opacity_retired",
    "bridge_canonical_lens_name_dispatch_retired",
    "bridge_include_str_side_channels_retired",
    "bridge_exact_string_patching_residual_retired",
];

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn disj_variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("`{name}` is not a Disj: {other:?}"),
    }
}

fn list_value_body<'a>(dag: &'a Dag, name: &str) -> &'a Vec<FieldValue> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.value_body {
        Some(ValueBody::List(rows)) => rows,
        Some(other) => panic!("`{name}` value_body is not a List: {other:?}"),
        None => panic!("`{name}` has no value_body"),
    }
}

fn record_field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> &'a FieldValue {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("record missing `{label}` field"))
}

fn string_literal(value: &FieldValue) -> &str {
    match value {
        FieldValue::Literal(LiteralBits::String(s)) => s.as_str(),
        other => panic!("expected String literal, got {other:?}"),
    }
}

#[test]
fn bridge_ledger_row_field_set_is_name_owner_status_authority() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "BridgeLedgerRow")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["name", "owner", "status", "authority"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        labels, expected,
        "BridgeLedgerRow must carry exactly `{{ name, owner, status, authority }}` \
         per the dispatch contract — adding/removing fields requires an explicit \
         substrate amendment, since Verification's `BridgeLedgerZero` fold reads \
         this shape."
    );
}

#[test]
fn bridge_status_is_closed_two_variant_coproduct() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = disj_variant_labels(&dag, "BridgeStatus")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["Retired", "Open"].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        labels, expected,
        "BridgeStatus must be the closed `Retired | Open` coproduct. A stringly \
         status field, an `InProgress`/`Partial` state, or any other variant \
         requires an explicit substrate amendment landing here before \
         Verification's fold can read the new shape."
    );
}

#[test]
fn bridge_ledger_lowers_as_list_of_bridge_ledger_row() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, BRIDGE_LEDGER);
    assert_eq!(
        rows.len(),
        CANONICAL_BRIDGES.len(),
        "`{BRIDGE_LEDGER}` must carry exactly the {} canonical bridge rows from \
         `docs/r3-structure.md:79-83`. Got {} rows.",
        CANONICAL_BRIDGES.len(),
        rows.len()
    );
}

#[test]
fn bridge_ledger_carries_canonical_five_names_in_doc_order() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, BRIDGE_LEDGER);
    let actual: Vec<String> = rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let FieldValue::Record(fields) = row else {
                panic!("row {idx} in `{BRIDGE_LEDGER}` is not a record literal: {row:?}");
            };
            string_literal(record_field(fields, "name")).to_string()
        })
        .collect();
    let expected: Vec<String> = CANONICAL_BRIDGES.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        actual, expected,
        "`{BRIDGE_LEDGER}` row names must match `docs/r3-structure.md:79-83` \
         (in document order). Authoring drift on either side fails closed here."
    );
}

#[test]
fn bridge_ledger_names_are_unique() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, BRIDGE_LEDGER);
    let names: Vec<String> = rows
        .iter()
        .map(|row| {
            let FieldValue::Record(fields) = row else {
                panic!("non-record row in `{BRIDGE_LEDGER}`: {row:?}");
            };
            string_literal(record_field(fields, "name")).to_string()
        })
        .collect();
    let unique: BTreeSet<String> = names.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        names.len(),
        "`{BRIDGE_LEDGER}` must not carry duplicate `name` rows; got {names:?}"
    );
}

#[test]
fn bridge_ledger_status_resolves_to_bridge_status_constructor() {
    // Every row's `status` field is a structural Variant, not a string;
    // its constructor must be one of the two `BridgeStatus` variants.
    // This is the property Verification's fold relies on to partition
    // rows without name-matching.
    let dag = generated_full_bootstrap_dag();
    let bridge_status = dag
        .declaration_by_name("BridgeStatus")
        .expect("BridgeStatus missing from full bootstrap");
    let allowed_constructors: HashSet<_> = match &bridge_status.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.ty).collect(),
        other => panic!("BridgeStatus is not a Disj: {other:?}"),
    };

    let rows = list_value_body(&dag, BRIDGE_LEDGER);
    for (idx, row) in rows.iter().enumerate() {
        let FieldValue::Record(fields) = row else {
            panic!("row {idx} not a record");
        };
        let status_field = record_field(fields, "status");
        let constructor = match status_field {
            FieldValue::Variant { constructor, .. } => *constructor,
            other => panic!(
                "row {idx} `status` must be a Variant carrying a `BridgeStatus` \
                 constructor, not a string or other shape; got {other:?}"
            ),
        };
        assert!(
            allowed_constructors.contains(&constructor),
            "row {idx} `status` constructor (DeclarationId {:?}) is not one of \
             `BridgeStatus`'s declared variants. Drift here means a row landed \
             with a status outside the closed coproduct.",
            constructor
        );
    }
}

// ── TestPredicate::BridgeLedgerZero shape + runner ratchets ─────────

fn compile_clean(source: &str, file: &str) -> Dag {
    match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{file} should compile cleanly, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(err) => panic!("{file} should compile cleanly, got {err:?}"),
    }
}

#[test]
fn bridge_ledger_zero_predicate_carries_only_ledger_declaration_ref() {
    // Verification's BridgeLedgerZero predicate contract: a single
    // structural payload field `ledger: DeclarationRef`. Adding fields
    // (e.g. an `expected_status: BridgeStatus` filter) requires an
    // explicit substrate amendment so the runner branch can't silently
    // diverge from the carrier's authority.
    let dag = generated_full_bootstrap_dag();
    let predicate = dag
        .declaration_by_name("TestPredicate")
        .expect("TestPredicate missing from bootstrap");
    let TypeConnective::Disj { variants } = &predicate.connective else {
        panic!("TestPredicate is not a Disj");
    };
    let bridge_variant = variants
        .iter()
        .find(|v| v.label == "BridgeLedgerZero")
        .expect("TestPredicate::BridgeLedgerZero variant missing");
    let payload = dag.declaration(bridge_variant.ty);
    let labels: HashSet<String> = match &payload.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("BridgeLedgerZero payload must be a Conj record; got {other:?}"),
    };
    let expected: HashSet<String> = ["ledger"].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        labels, expected,
        "BridgeLedgerZero must carry exactly `{{ ledger: DeclarationRef }}` per \
         the Verification dispatch contract."
    );
    let ledger_field = match &payload.connective {
        TypeConnective::Conj { children } => children
            .iter()
            .find(|f| f.label == "ledger")
            .expect("ledger field missing"),
        _ => unreachable!(),
    };
    let ledger_decl = dag.declaration(ledger_field.ty);
    assert_eq!(
        ledger_decl.name.as_deref(),
        Some("DeclarationRef"),
        "BridgeLedgerZero.ledger must be `DeclarationRef`; got {:?}",
        ledger_decl.name
    );
}

#[test]
fn bridge_ledger_zero_runner_fails_with_named_open_rows_at_head() {
    // At HEAD the canonical ledger has three Open rows
    // (source_span_file_participation, include_str_side_channels,
    // exact_string_patching_residual). The runner must `Fail` and name
    // every Open row in the diagnostic; do NOT pretend ledger-zero is
    // already true. Once an `Open` row flips to `Retired` upstream the
    // diagnostic narrows to the remaining open names; once all five are
    // `Retired` this test re-arms as a `Pass` ratchet.
    let source = r#"
data ledger_zero_claim: TestClaim = {
  name: "bridge_ledger_zero_at_head",
  source: "let x: Int = 1",
  file_name: "bridge_ledger_zero_runner.v3",
  predicate: BridgeLedgerZero { ledger: bridge_ledger },
  requires: []
}

data suite: TestSuite = {
  name: "bridge_ledger_zero_runner_suite",
  claims: [ledger_zero_claim]
}
"#;
    let dag = compile_clean(source, "bridge_ledger_zero_runner.dag");
    let results = TestRunner::new(&dag).run_suite("suite");
    assert_eq!(results.len(), 1);
    let reason = match &results[0].result {
        ClaimResult::Fail(reason) => reason.clone(),
        other => panic!("expected `Fail` (open rows present at HEAD); got {other:?}"),
    };
    // Diagnostic must name every currently-Open row so Verification
    // surfaces the residual debt. Source-of-truth status verdicts are
    // documented in `src/v3/std/bridge_ledger.dag` per-row rationale
    // and `docs/r3-structure.md:79-83`. If a row flips to `Retired`
    // upstream, drop it from this expected set in the same PR that
    // updates the carrier so the gate stays honest.
    let expected_open_rows = [
        "bridge_source_span_file_participation_retired",
        "bridge_include_str_side_channels_retired",
        "bridge_exact_string_patching_residual_retired",
    ];
    for row in expected_open_rows {
        assert!(
            reason.contains(row),
            "BridgeLedgerZero failure message must name `{row}`; got: {reason}"
        );
    }
    // And the two Retired rows must not appear in the failure list.
    let expected_retired_rows = [
        "bridge_mark_bootstrap_secret_nominal_opacity_retired",
        "bridge_canonical_lens_name_dispatch_retired",
    ];
    for row in expected_retired_rows {
        assert!(
            !reason.contains(row),
            "BridgeLedgerZero failure must NOT name retired row `{row}`; got: {reason}"
        );
    }
}

#[test]
fn bridge_ledger_zero_runner_fails_closed_on_wrong_ledger_type() {
    // Fail-closed type check at the claim boundary: a claim that points
    // `BridgeLedgerZero { ledger: ... }` at any list-shaped declaration
    // other than `List<BridgeLedgerRow>` must be rejected before row
    // scanning. Without this guard, any list whose records happen to
    // carry `name`/`status` would be silently accepted as a ledger; the
    // predicate consumes the substrate carrier type, not look-alikes.
    let source = r#"
type FakeStatus = Yes | No

type FakeLedgerRow {
  name: String
  status: FakeStatus
}

data fake_ledger: List<FakeLedgerRow> = [
  {
    name: "fake",
    status: Yes
  }
]

data wrong_type_claim: TestClaim = {
  name: "wrong_ledger_type",
  source: "let x: Int = 1",
  file_name: "bridge_ledger_zero_wrong_type.v3",
  predicate: BridgeLedgerZero { ledger: fake_ledger },
  requires: []
}

data suite: TestSuite = {
  name: "bridge_ledger_zero_wrong_type_suite",
  claims: [wrong_type_claim]
}
"#;
    let dag = compile_clean(source, "bridge_ledger_zero_wrong_type.dag");
    let results = TestRunner::new(&dag).run_suite("suite");
    assert_eq!(results.len(), 1);
    let reason = match &results[0].result {
        ClaimResult::Fail(reason) => reason.clone(),
        other => panic!("expected `Fail` for wrong-typed ledger; got {other:?}"),
    };
    // Failure must call out the type mismatch — not silently scan
    // FakeLedgerRow as if it were BridgeLedgerRow.
    assert!(
        reason.contains("BridgeLedgerRow"),
        "BridgeLedgerZero failure for wrong ledger type must name the expected \
         element type `BridgeLedgerRow`; got: {reason}"
    );
    assert!(
        reason.contains("FakeLedgerRow") || reason.contains("List<"),
        "failure should reference the wrong type; got: {reason}"
    );
}
