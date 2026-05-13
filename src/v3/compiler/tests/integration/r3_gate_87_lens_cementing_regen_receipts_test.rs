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
//! **Placeholder dissolution ledger:** `docs/r3-gate-87-cementing-placeholder-dissolution-ledger.md`
//! lists temporary `.dag` harness placeholders + Rust bridge paths; keep it aligned with
//! `r3_gate_87_placeholder_dissolution_ledger_matches_authority`.
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

use v3_compiler::r3_gate_87_cementing_regen_runner_suites::r3_gate_87_cementing_regen_lens_names_for_runner_table;

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
         `v3_compiler::r3_gate_87_cementing_regen_runner_suites::R3_GATE_87_CEMENTING_REGEN_SUITES`: extend the runner table + \
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

/// `LensRegistryEntry.name` values for `tests/dag/t_r3_gate_87_cementing_regen_*.dag` harnesses
/// that intentionally remain **placeholder** receipts (`Compiles` or minimal `Int` projection)
/// until the dissolution triggers in each harness header land.
const G87_DAG_PLACEHOLDER_LENS_NAMES: &[&str] = &[
    "infer_helpers",
    "lower_helpers",
    "structural_resolution",
    "unused_parameters",
    "variant_payload",
];

/// Rust paths indexed by `docs/r3-gate-87-cementing-placeholder-dissolution-ledger.md` (sorted).
const G87_GATE87_CEMENTING_RUST_RECEIPT_PATHS: &[&str] = &[
    "src/v3/compiler/src/cementing_dispatch.rs",
    "src/v3/compiler/src/integration_rs_wiring_scan.rs",
    "src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs",
    "src/v3/compiler/tests/integration/cementing/cementing_provenance_origin_integration_test.rs",
    "src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs",
    "src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs",
    "src/v3/compiler/tests/integration/common/wiring_scanner_test.rs",
    "src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs",
];

const G87_DAG_PLACEHOLDER_KEYS_BEGIN: &str = "<!-- G87_CEMENTING_DAG_PLACEHOLDER_KEYS_BEGIN -->";
const G87_DAG_PLACEHOLDER_KEYS_END: &str = "<!-- G87_CEMENTING_DAG_PLACEHOLDER_KEYS_END -->";
const G87_RUST_RECEIPT_PATHS_BEGIN: &str = "<!-- G87_CEMENTING_RUST_RECEIPT_PATHS_BEGIN -->";
const G87_RUST_RECEIPT_PATHS_END: &str = "<!-- G87_CEMENTING_RUST_RECEIPT_PATHS_END -->";

fn parse_g87_ledger_marker_block(doc: &str, begin: &str, end: &str) -> BTreeSet<String> {
    let start_idx = doc.find(begin).unwrap_or_else(|| {
        panic!("gate-#87 dissolution ledger must contain marker `{begin}`");
    });
    let rest = &doc[start_idx + begin.len()..];
    let end_rel = rest.find(end).unwrap_or_else(|| {
        panic!("gate-#87 dissolution ledger must contain marker `{end}`");
    });
    let body = &rest[..end_rel];
    body.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("<!--"))
        .map(String::from)
        .collect()
}

#[test]
fn r3_gate_87_placeholder_dissolution_ledger_matches_authority() {
    let ledger_path =
        workspace_root().join("docs/r3-gate-87-cementing-placeholder-dissolution-ledger.md");
    let doc = std::fs::read_to_string(&ledger_path).unwrap_or_else(|e| {
        panic!("read gate-#87 dissolution ledger {}: {e}", ledger_path.display());
    });

    let ledger_dag_keys = parse_g87_ledger_marker_block(
        &doc,
        G87_DAG_PLACEHOLDER_KEYS_BEGIN,
        G87_DAG_PLACEHOLDER_KEYS_END,
    );
    let expected_dag_keys: BTreeSet<String> = G87_DAG_PLACEHOLDER_LENS_NAMES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        ledger_dag_keys, expected_dag_keys,
        "update `docs/r3-gate-87-cementing-placeholder-dissolution-ledger.md` \
         G87_CEMENTING_DAG_PLACEHOLDER_KEYS block to match G87_DAG_PLACEHOLDER_LENS_NAMES \
         (or vice versa) when placeholder harness membership changes"
    );

    let runner_names = r3_gate_87_cementing_regen_lens_names_for_runner_table();
    for name in G87_DAG_PLACEHOLDER_LENS_NAMES {
        assert!(
            runner_names.contains(*name),
            "G87_DAG_PLACEHOLDER_LENS_NAMES names `{name}` missing from \
             R3_GATE_87_CEMENTING_REGEN_SUITES-derived inventory"
        );
    }

    let ledger_rust_paths = parse_g87_ledger_marker_block(
        &doc,
        G87_RUST_RECEIPT_PATHS_BEGIN,
        G87_RUST_RECEIPT_PATHS_END,
    );
    let expected_rust_paths: BTreeSet<String> = G87_GATE87_CEMENTING_RUST_RECEIPT_PATHS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        ledger_rust_paths, expected_rust_paths,
        "update `docs/r3-gate-87-cementing-placeholder-dissolution-ledger.md` \
         G87_CEMENTING_RUST_RECEIPT_PATHS block to match G87_GATE87_CEMENTING_RUST_RECEIPT_PATHS \
         (or vice versa) when the Rust bridge inventory changes"
    );
}
