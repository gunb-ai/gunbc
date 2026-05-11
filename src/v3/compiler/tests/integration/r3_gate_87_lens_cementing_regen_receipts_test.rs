//! **Layer:** integration
//!
//! R3 gate #87 (`lens_cementing_test_discipline_complete`) — paired **Rust** receipts for
//! `src/v3/compiler/regen.dag` registry lenses whose `tests/dag/t_r3_gate_87_cementing_regen_*.dag`
//! harnesses use runner-evaluated predicates on a minimal program (`let lit: Int = 7` — e.g.
//! `PortHasState` for eight regen rows; **Lane-E** `DifferentialEquals` + **symbolic-cost**
//! `SymbolicCostExprEquals` on other rows). Rust receipts below remain where a frozen
//! `LensOutputEquals` / richer oracle witness is not yet authored as strict `.dag` data.
//!
//! **Lane-E + symbolic-cost** `.dag` receipts are exercised by `t_pb_b_1_dag_runner_test`.
//! `unused_parameters` / `structural_resolution` / `provenance` / `effect_enumeration` /
//! `cost_target_realization` / helper rows use `.dag` `PortHasState` plus these Rust receipts for
//! lens-specific APIs until M1(2.8) carrier literals can freeze full parity in `.dag` alone.
//!
//! **INVARIANTS P5(b):** Against `origin/main...HEAD`, gate-#87 work is **merge-visible** as this
//! module (two new `mod` lines in `tests/integration.rs`), `t_pb_b_1_dag_runner_test`’s
//! `R3_GATE_87_CEMENTING_REGEN_SUITES`, and the `tests/dag/t_r3_gate_87_cementing_regen_*.dag`
//! harness files — reviewers can confirm with `git diff origin/main...HEAD --stat` / path grep.
//! Registry `name` inventory is cross-checked against
//! `t_pb_b_1_dag_runner_test::r3_gate_87_cementing_regen_lens_names_for_runner_table` (derived from
//! `R3_GATE_87_CEMENTING_REGEN_SUITES` paths — single authority, no parallel hand list).
//! Per `INVARIANTS.md` §P5(b), the **single checkable net paydown receipt** (delete path, SG-0
//! census shrink with counts, or cited `ROADMAP.md` deferral) must live in **PR #2639’s
//! description**; module comments must not assert deletes for paths that never existed on
//! `origin/main`. Remaining §Acceptance (frozen v2-oracle cementing): `ROADMAP.md` **v3 lens
//! capability honesty pass** bullet.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::t_pb_b_1_dag_runner_test::r3_gate_87_cementing_regen_lens_names_for_runner_table;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Declaration, FieldValue, LiteralBits, ValueBody};
use v3_compiler::lens_cost_target_realization::type_realization_meta;
use v3_compiler::lens_effect_enumeration::{enumerate_effects, TransactionalPattern};
use v3_compiler::lens_provenance::{origin_of, Origin};
use v3_compiler::lens_structural_resolution;
use v3_compiler::lens_unused_parameters::{UnusedParametersConfig, UnusedParametersLens};
use v3_compiler::Dag;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("expected src/v3/compiler -> workspace root")
        .to_path_buf()
}

fn structural_fields(decl: &Declaration) -> &[(String, FieldValue)] {
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        panic!(
            "lens registry entry `{}` must carry a structural value body",
            decl.name.as_deref().unwrap_or("<anonymous>")
        );
    };
    fields.as_slice()
}

fn string_field(fields: &[(String, FieldValue)], label: &str, binding: &str) -> String {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("lens registry entry `{binding}` is missing a String `{label}` field")
        })
}

fn regen_lens_registry_names() -> BTreeSet<String> {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load `src/v3/compiler/regen.dag` cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let entry_type_id = dag
        .declaration_by_name("LensRegistryEntry")
        .map(|decl| decl.id)
        .expect("regen.dag must declare `LensRegistryEntry`");
    dag.declarations()
        .iter()
        .filter(|decl| decl.meta_tag == Some(entry_type_id))
        .map(|decl| {
            let binding = decl
                .name
                .clone()
                .unwrap_or_else(|| "<anonymous>".to_string());
            let fields = structural_fields(decl);
            string_field(fields, "name", &binding)
        })
        .collect()
}

fn read_lens_source(rel: &str) -> String {
    let path = workspace_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn assert_lens_dag_compiles(rel: &str) {
    let source = read_lens_source(rel);
    let dag = compile_to_dag(&source, rel).unwrap_or_else(|diag| {
        panic!("{rel} should compile cleanly, got {diag:?}");
    });
    assert!(
        dag.diagnostics().is_empty(),
        "{rel} should have no module diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

fn find_bind_value_port(dag: &Dag, name: &str) -> v3_compiler::dag::PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

#[test]
fn r3_gate_87_regen_lens_registry_names_match_fixture_inventory() {
    let actual = regen_lens_registry_names();
    let expected = r3_gate_87_cementing_regen_lens_names_for_runner_table();
    assert_eq!(
        actual, expected,
        "`src/v3/compiler/regen.dag` registry drift vs \
         `t_pb_b_1_dag_runner_test::R3_GATE_87_CEMENTING_REGEN_SUITES`: extend the runner table + \
         `tests/dag/t_r3_gate_87_cementing_regen_*.dag` in the same PR as any new registry row."
    );
}

#[test]
fn r3_gate_87_effect_enumeration_rust_receipt_on_minimal_program() {
    let dag =
        compile_to_dag("let lit: Int = 7", "r3_gate_87_effect_enum_receipt.v3").expect("compile");
    let report = enumerate_effects(&dag);
    assert!(
        matches!(report.transaction, TransactionalPattern::NoTransaction),
        "effect enumeration transaction scaffold must remain explicit"
    );
    assert!(
        report.facts.len() <= dag.nodes().len(),
        "effect facts should not exceed walked node count"
    );
}

#[test]
fn r3_gate_87_provenance_origin_rust_receipt_on_literal_bind() {
    let dag =
        compile_to_dag("let lit: Int = 7", "r3_gate_87_provenance_receipt.v3").expect("compile");
    let port = find_bind_value_port(&dag, "lit");
    let got = origin_of(&dag, &port);
    assert!(
        matches!(got, Origin::Source { .. }),
        "literal bind should classify as Source(..), got {got:?}"
    );
}

#[test]
fn r3_gate_87_cost_target_realization_rust_receipt_resolves_type_realization_row() {
    let dag = compile_to_dag(
        "let lit: Int = 7",
        "r3_gate_87_cost_target_realization_receipt.v3",
    )
    .expect("compile");
    let meta = type_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "type_realization_meta must resolve the substrate `TypeRealization` declaration \
         (declaration_by_name contract used by cost_target_realization.dag)"
    );
    assert_eq!(
        meta.unwrap().name.as_deref(),
        Some("TypeRealization"),
        "cost_target_realization meta lookup must return the named realization row, not \
         another declaration"
    );
}

#[test]
fn r3_gate_87_infer_helpers_lens_source_compiles() {
    assert_lens_dag_compiles("src/v3/lenses/infer_helpers.dag");
}

#[test]
fn r3_gate_87_variant_payload_lens_source_compiles() {
    assert_lens_dag_compiles("src/v3/lenses/variant_payload.dag");
}

#[test]
fn r3_gate_87_lower_helpers_lens_source_compiles() {
    assert_lens_dag_compiles("src/v3/lenses/lower_helpers.dag");
}

#[test]
fn r3_gate_87_structural_resolution_rust_receipt_on_literal_program() {
    let dag = compile_to_dag(
        "let lit: Int = 7",
        "r3_gate_87_structural_resolution_receipt.v3",
    )
    .expect("compile");
    assert!(
        lens_structural_resolution::check(&dag).is_empty(),
        "clean literal program should surface zero Pending-arrow violations"
    );
}

#[test]
fn r3_gate_87_unused_parameters_rust_receipt_on_literal_program() {
    let dag = compile_to_dag(
        "let lit: Int = 7",
        "r3_gate_87_unused_parameters_receipt.v3",
    )
    .expect("compile");
    assert!(
        UnusedParametersLens::new(&dag)
            .query(&UnusedParametersConfig::default())
            .is_empty(),
        "literal bind should not surface unused-parameter findings"
    );
}
