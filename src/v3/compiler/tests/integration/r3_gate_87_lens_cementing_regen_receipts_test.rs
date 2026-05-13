//! **Layer:** integration
//!
//! R3 gate #87 (`lens_cementing_test_discipline_complete`) — paired **Rust** receipts for
//! `src/v3/compiler/regen.dag` registry lenses while the `.dag`
//! `tests/dag/t_r3_gate_87_cementing_regen_*.dag` harnesses either use narrow behavioral
//! `LensOutputEquals` / `DifferentialEquals` witnesses or remain explicit temporary `Compiles`
//! placeholders where no public behavior carrier is authorable yet.
//!
//! **Lane-E + symbolic-cost** `.dag` receipts are exercised by `t_pb_b_1_dag_runner_test`.
//! `unused_parameters` and `structural_resolution` use Int-projection `.dag` claims until strict
//! user modules can freeze the corresponding list carriers without M1(2.8) opaque-body diagnostics;
//! Rust receipts below keep covering `UnusedParametersLens` / `lens_structural_resolution::check`.
//! Helper-only rows (`infer_helpers`, `lower_helpers`, `variant_payload`) stay explicit `Compiles`
//! placeholders with per-file dissolution triggers in their `.dag` harness comments.
//!
//! **INVARIANTS P5(b):** Gate-#87 work is **merge-visible** as this module,
//! `r3_gate_87_cementing_regen_runner_suites` plus `t_pb_b_1_dag_runner_test` wiring, and the
//! harness files under `tests/dag/t_r3_gate_87_cementing_regen_*.dag` (confirm with
//! `git diff origin/main...HEAD --stat` and path grep). Registry `name` inventory matches
//! `r3_gate_87_cementing_regen_runner_suites::r3_gate_87_cementing_regen_lens_names_for_runner_table`
//! derived from `R3_GATE_87_CEMENTING_REGEN_SUITES` (single authority, no parallel hand list).
//!
//! **Cementing-test discipline ratchet (`TESTING.md` §4 "One claim per test"):** every new
//! `#[test]` / `data foo: TestClaim` in this lane makes **one** structural claim; cross-suite
//! drive tests assert `ClaimResult` by shape (`== ClaimResult::Pass` or `matches!(_, Pass)`),
//! never by stringified message. When porting any Rust receipt below to a `.dag` `TestClaim`,
//! the same PR removes its row from `sg0_census_test::EXPECTED_HAND_AUTHORED_TEST` — no
//! parallel cementing inventory is allowed to track the Rust→`.dag` migration separately.
//! Per `INVARIANTS.md` §P5(b), the **single checkable net paydown receipt** (delete path, SG-0
//! census shrink with counts, or cited `ROADMAP.md` deferral) must live in **PR #2639’s
//! description**; module comments must not assert deletes for paths that never existed on
//! `origin/main`. §1.8 gate-#87 **PASSING** is indexed in `docs/r3-program-plan.md` (row 87);
//! the canonical Pass-condition body is `r3-structure.md` §"Acceptance"
//! (`lens_cementing_test_discipline_complete`). Broader Band-C work for lenses outside
//! `regen.dag` continues through `docs/v3-lens-capability-register.md` +
//! `cementing_lens_registry_dispatch_test.rs` + `ROADMAP.md` honesty pass.

use std::collections::BTreeSet;
use std::path::PathBuf;

use v3_compiler::r3_gate_87_cementing_regen_runner_suites::{
    r3_gate_87_cementing_regen_harnesses, r3_gate_87_cementing_regen_lens_names_for_runner_table,
};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Declaration, FieldValue, LiteralBits, TypeConnective, ValueBody};
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

fn sum_variant_label(dag: &Dag, sum_type_name: &str, value: &FieldValue) -> String {
    let sum_decl = dag
        .declaration_by_name(sum_type_name)
        .unwrap_or_else(|| panic!("bootstrap must declare `{sum_type_name}`"));
    let TypeConnective::Disj { variants } = &sum_decl.connective else {
        panic!("`{sum_type_name}` must be a Disj coproduct");
    };
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected `{sum_type_name}` variant value, got {value:?}");
    };
    assert!(
        payload.is_empty(),
        "`{sum_type_name}` status arms should remain nullary"
    );
    variants
        .iter()
        .find(|variant| variant.ty == *constructor)
        .map(|variant| variant.label.clone())
        .unwrap_or_else(|| {
            panic!("constructor id {constructor:?} is not a `{sum_type_name}` variant")
        })
}

#[derive(Debug, Clone)]
struct CapabilityRow {
    lens_name: String,
    behavioral: String,
    v2_counterpart: String,
}

fn lens_capability_rows_by_runner_name() -> Vec<CapabilityRow> {
    let dag = Dag::new();
    let rows_decl = dag
        .declaration_by_name("lens_capability_register_rows")
        .expect("bootstrap must declare `lens_capability_register_rows`");
    let Some(ValueBody::List(rows)) = &rows_decl.value_body else {
        panic!("`lens_capability_register_rows` must be a list");
    };
    rows.iter()
        .map(|row| {
            let FieldValue::Record(fields) = row else {
                panic!("lens capability row must be a record, got {row:?}");
            };
            let basename = string_field(fields, "lens_basename", "lens_capability_register_rows");
            let lens_name = basename
                .strip_suffix(".dag")
                .unwrap_or_else(|| panic!("lens basename `{basename}` must end in `.dag`"))
                .to_string();
            let behavioral = fields
                .iter()
                .find(|(label, _)| label == "behavioral")
                .map(|(_, value)| sum_variant_label(&dag, "LensCapabilityBehavioralStatus", value))
                .expect("lens capability row must carry `behavioral`");
            let v2_counterpart = fields
                .iter()
                .find(|(label, _)| label == "v2_counterpart")
                .map(|(_, value)| sum_variant_label(&dag, "LensCapabilityV2Counterpart", value))
                .expect("lens capability row must carry `v2_counterpart`");
            CapabilityRow {
                lens_name,
                behavioral,
                v2_counterpart,
            }
        })
        .collect()
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
         `v3_compiler::r3_gate_87_cementing_regen_runner_suites::R3_GATE_87_CEMENTING_REGEN_SUITES`: extend the runner table + \
         `tests/dag/t_r3_gate_87_cementing_regen_*.dag` in the same PR as any new registry row."
    );
}

#[test]
fn r3_gate_87_v3_native_and_helper_placeholders_stay_explicit() {
    let harnesses = r3_gate_87_cementing_regen_harnesses()
        .map(|harness| {
            (
                harness
                    .file
                    .strip_prefix("src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_")
                    .and_then(|rest| rest.strip_suffix(".dag"))
                    .unwrap_or_else(|| panic!("unexpected gate-87 harness path `{}`", harness.file))
                    .to_string(),
                harness,
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let allowed_compiles_placeholders = BTreeSet::from([
        "infer_helpers".to_string(),
        "lower_helpers".to_string(),
        "variant_payload".to_string(),
    ]);

    for row in lens_capability_rows_by_runner_name() {
        let Some(harness) = harnesses.get(&row.lens_name) else {
            continue;
        };
        let uses_compiles = harness.source.contains("predicate: Compiles");
        let uses_lens_output = harness.source.contains("LensOutputEquals");

        if row.v2_counterpart == "LensCapabilityV2NotApplicable" {
            assert!(
                uses_compiles,
                "helper-only `{}` must stay an explicit `Compiles` placeholder until a public \
                 carrier exists",
                row.lens_name
            );
        }

        if row.v2_counterpart == "LensCapabilityV2NoneV3Native"
            && row.behavioral == "LensCapabilityBehavioralComplete"
            && !allowed_compiles_placeholders.contains(&row.lens_name)
        {
            assert!(
                uses_lens_output,
                "complete v3-native `{}` must use `LensOutputEquals`, not a source-compilation \
                 placeholder",
                row.lens_name
            );
            assert!(
                !uses_compiles,
                "complete v3-native `{}` must not also carry `Compiles`",
                row.lens_name
            );
        }

        if uses_compiles {
            assert!(
                allowed_compiles_placeholders.contains(&row.lens_name),
                "`{}` uses `Compiles`; only named helper/v3-native temporary placeholders may do so",
                row.lens_name
            );
            assert!(
                harness
                    .source
                    .contains("temporary source-compilation placeholder")
                    && harness.source.contains("Dissolution trigger:")
                    && harness.source.contains("Temporary Rust receipt:"),
                "`{}` `Compiles` placeholder must document the placeholder class, dissolution \
                 trigger, and paired Rust pin receipt in the `.dag` harness",
                row.lens_name
            );
        }
    }
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
    let resolved_name = type_realization_meta(&dag).and_then(|d| d.name);
    assert_eq!(
        resolved_name.as_deref(),
        Some("TypeRealization"),
        "type_realization_meta must resolve the substrate `TypeRealization` declaration \
         (declaration_by_name contract used by cost_target_realization.dag)"
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
