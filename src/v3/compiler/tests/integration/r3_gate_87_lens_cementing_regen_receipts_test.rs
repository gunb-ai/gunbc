//! **Layer:** integration
//!
//! R3 gate #87 (`lens_cementing_test_discipline_complete`) — residual **Rust** ratchets for
//! `src/v3/compiler/regen.dag` registry lenses after the behavior receipts moved into
//! `tests/dag/t_r3_gate_87_cementing_regen_*.dag` harnesses. This module now holds only the
//! registry-inventory ratchet plus helper source-compilation pins where no public behavior carrier
//! is authorable yet.
//!
//! **Lane-E + symbolic-cost** `.dag` receipts are exercised by `t_pb_b_1_dag_runner_test`.
//! `effect_enumeration`, `provenance`, `cost_target_realization`, `unused_parameters`, and
//! `structural_resolution` also use Int-projection `.dag` claims over their published runner
//! projections, so they no longer need parallel Rust receipt tests here.
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

use v3_compiler::r3_gate_87_cementing_regen_runner_suites::r3_gate_87_cementing_regen_lens_names_for_runner_table;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Declaration, FieldValue, LiteralBits, ValueBody};
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
