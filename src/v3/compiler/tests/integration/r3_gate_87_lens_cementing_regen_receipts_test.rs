//! **Layer:** integration
//!
//! R3 gate #87 (`lens_cementing_test_discipline_complete`) — paired **Rust** receipts for
//! `src/v3/compiler/regen.dag` registry lenses whose `tests/dag/t_r3_gate_87_cementing_regen_*.dag`
//! harnesses are still `Compiles` placeholders (bootstrap-scale / multi-arg lenses where a frozen
//! `LensOutputEquals` / `DifferentialEquals` witness is not yet authored as `.dag` data).
//!
//! **Lane-E + symbolic-cost** `.dag` receipts are exercised by `t_pb_b_1_dag_runner_test`.
//! `unused_parameters` and `structural_resolution` stay `Compiles` placeholders in `.dag` until
//! strict user modules can freeze the corresponding list carriers without M1(2.8) opaque-body
//! diagnostics; Rust receipts below cover `UnusedParametersLens` / `lens_structural_resolution::check`.

use std::collections::BTreeSet;
use std::path::PathBuf;

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

/// Sorted `LensRegistryEntry.name` keys pinned to `src/v3/compiler/regen.dag` at HEAD.
/// When adding a registry row, land `tests/dag/t_r3_gate_87_cementing_regen_<name>.dag` + a
/// `t_pb_b_1_dag_runner_test` receipt in the same PR (and extend Rust receipts below when the
/// harness remains a `Compiles` placeholder).
const EXPECTED_REGEN_LENS_NAMES: &[&str] = &[
    "cost",
    "cost_symbolic",
    "cost_target_realization",
    "effect_enumeration",
    "infer_helpers",
    "lower_helpers",
    "provenance",
    "structural_resolution",
    "unused_parameters",
    "variant_payload",
];

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
    let expected: BTreeSet<String> = EXPECTED_REGEN_LENS_NAMES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        actual, expected,
        "`src/v3/compiler/regen.dag` registry drift: update EXPECTED_REGEN_LENS_NAMES and the \
         `tests/dag/t_r3_gate_87_cementing_regen_*.dag` + `t_pb_b_1_dag_runner_test` harness list in \
         the same PR."
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
