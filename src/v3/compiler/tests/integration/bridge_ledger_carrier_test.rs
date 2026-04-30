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
