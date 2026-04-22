//! **Layer:** integration
//!
//! Band-C cementing dispatch for lenses enumerated in `regen.dag` and
//! described in `docs/v3-lens-capability-register.md`.
//!
//! - `TESTING.md` — *Cementing tests (Band C — lens subsumption)*
//! - `src/v3/compiler/regen.dag` — header comment on cementing dispatch
//!
//! **Authority cross-check.** `CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS` is not a
//! second source of truth: `cementing_escalation_slice_matches_capability_register`
//! derives the required registry `name` keys from `docs/v3-lens-capability-register.md`
//! (capability table) plus `regen.dag` (`LensRegistryEntry` rows) and asserts the
//! slice matches exactly.
//!
//! **Provenance cementing split.** The `compile_to_dag` exemplar below compares
//! `origin_of` to a **local** five-way `Behavior → Origin` projection that mirrors
//! `provenance.dag`’s emitted `origin_for_behavior` (kept crate-private). That keeps
//! the public `lens_provenance` surface to `origin_of` / `Origin` only. This is a
//! **seam** check (`port` → `produced_by` → `node` → classification) on the live
//! lowering path; the `Behavior → Origin` table is also pinned under
//! `#[cfg(test)]` in `lib.rs::lens_provenance::tests` with minimal hand-built `Dag`
//! shapes (see `TESTING.md` — v3-native `COMPLETE` path). Regenerate / update the
//! local mirror when `provenance.dag` changes that table.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::common::integration_rs_active_line_contains;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, Declaration, FieldValue, LiteralBits, ValueBody};
use v3_compiler::lens_provenance::{origin_of, Origin};

/// Structural equality for the full published `Origin` carrier (every variant).
/// `Origin` is generated without `PartialEq`; this is the integration-test oracle.
fn assert_origin_carriers_equal(a: &Origin, b: &Origin, context: &str) {
    match (a, b) {
        (Origin::NoProducer, Origin::NoProducer) => {}
        (Origin::MissingPort, Origin::MissingPort) => {}
        (Origin::MissingBehavior, Origin::MissingBehavior) => {}
        (Origin::Source { _0: x }, Origin::Source { _0: y }) => {
            assert_eq!(x, y, "{context}: Source NodeId mismatch")
        }
        (Origin::Computed { _0: x }, Origin::Computed { _0: y }) => {
            assert_eq!(x, y, "{context}: Computed NodeId mismatch")
        }
        (Origin::Selected { _0: x }, Origin::Selected { _0: y }) => {
            assert_eq!(x, y, "{context}: Selected NodeId mismatch")
        }
        (Origin::Accumulated { _0: x }, Origin::Accumulated { _0: y }) => {
            assert_eq!(x, y, "{context}: Accumulated NodeId mismatch")
        }
        _ => panic!("{context}: full Origin carrier mismatch\n  got: {a:?}\n  exp: {b:?}"),
    }
}

/// `origin_of` must agree with the provenance lens’s `Behavior → Origin` table on
/// the value port's producer after the `produced_by` walk.
///
/// The table is duplicated here (not re-exported from the crate) so integration
/// tests do not widen the public API with `origin_for_behavior`. Keep this match in
/// lockstep with `src/v3/lenses/provenance.dag` / `lens_provenance_generated.rs`.
/// Independent minimal-`Dag` pins live in `lens_provenance::tests`.
fn assert_provenance_origin_matches_lens_authority(dag: &Dag, bind_name: &str, context: &str) {
    let port = find_bind_value_port(dag, bind_name);
    let got = origin_of(dag, &port);
    let produced_by = dag
        .port_opt(&port)
        .unwrap_or_else(|| panic!("{context}: missing port for bind `{bind_name}`"))
        .produced_by
        .unwrap_or_else(|| {
            panic!("{context}: bind `{bind_name}` value port has no producer (expected one)")
        });
    let behavior = dag
        .nodes()
        .iter()
        .find(|b| b.id() == produced_by)
        .unwrap_or_else(|| panic!("{context}: missing producer node {produced_by:?}"));
    let expected = expected_origin_from_producer_behavior(behavior);
    assert_origin_carriers_equal(&got, &expected, context);
}

/// Mirrors `origin_for_behavior` from `provenance.dag` (see `lens_provenance_generated.rs`).
fn expected_origin_from_producer_behavior(behavior: &Behavior) -> Origin {
    match behavior {
        Behavior::Value(v) => Origin::Source { _0: v.id },
        Behavior::Transform(t) => Origin::Computed { _0: t.id },
        Behavior::Branch(b) => Origin::Selected { _0: b.id },
        Behavior::Loop(l) => Origin::Accumulated { _0: l.id },
        Behavior::Bind(bind) => Origin::Source { _0: bind.id },
    }
}

/// Pairs of (`regen_lens --lens <name>` registry key, cementing module stem
/// under `tests/integration/cementing/` without `.rs`).
///
/// **Must match register + regen mechanically** — see
/// `cementing_escalation_slice_matches_capability_register`. Append when the
/// capability table row is plain `COMPLETE` (not `**PROXY**` / `**STUB**` / `N/A`)
/// **and** the v2 counterpart cell names a real v2 path (not `None (v3-native)` /
/// not `N/A` per `TESTING.md` Band-C). Land the new
/// `cementing/<stem>.rs` module and a `#[path = ...]` line in
/// `tests/integration.rs` in the same PR.
const CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS: &[(&str, &str)] = &[];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("expected src/v3/compiler -> workspace root")
        .to_path_buf()
}

fn md_table_cells(line: &str) -> Vec<String> {
    // Capability rows escape `|` inside cells as `\|` (see v3 output column).
    let tmp = line.replace("\\|", "\u{241f}");
    tmp.split('|')
        .map(|s| s.replace('\u{241f}', "|").trim().to_string())
        .collect()
}

/// Band-C register column: v2 counterpart exists for cross-implementation cementing.
/// Must match `TESTING.md` (Cementing tests): treat `None (v3-native)` and `N/A`
/// as absent — same policy as prose, not a second authority.
fn register_row_has_real_v2_counterpart(v2_cell: &str) -> bool {
    let v2 = v2_cell.trim().trim_matches('`');
    !(v2.contains("None (v3-native)") || v2 == "N/A")
}

#[test]
fn register_row_has_real_v2_counterpart_matches_testing_md_band_c() {
    assert!(!register_row_has_real_v2_counterpart("None (v3-native)"));
    assert!(!register_row_has_real_v2_counterpart(
        "  None (v3-native)  "
    ));
    assert!(!register_row_has_real_v2_counterpart("N/A"));
    assert!(!register_row_has_real_v2_counterpart("  `N/A`  "));
    assert!(register_row_has_real_v2_counterpart(
        "src/v2/complexity.dag (5488L)"
    ));
}

#[test]
fn integration_rs_active_line_contains_rejects_commented_cementing_path() {
    let src = r#"
// #[path = "integration/cementing/ghost.rs"]
// mod ghost;
#[path = "integration/cementing/real.rs"]
mod real;
"#;
    assert!(!integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/ghost.rs"]"#,
    ));
    assert!(integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/real.rs"]"#,
    ));
}

#[test]
fn integration_rs_active_line_contains_rejects_block_commented_cementing_path() {
    let src = r#"/*
#[path = "integration/cementing/ghost.rs"]
mod ghost;
*/
#[path = "integration/cementing/real.rs"]
mod real;
"#;
    assert!(!integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/ghost.rs"]"#,
    ));
    assert!(integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/real.rs"]"#,
    ));
}

#[test]
fn integration_rs_active_line_ignores_needle_inside_string_literal() {
    // Tombstone text appears only inside a `"..."` string — must not satisfy the ratchet.
    let src = concat!(
        "let _ = \"#[path = \\\"integration/cementing/decoy.rs\\\"]\";\n",
        "#[path = \"integration/cementing/real.rs\"]\n",
        "mod real;\n",
    );
    assert!(!integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/decoy.rs"]"#,
    ));
    assert!(integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/real.rs"]"#,
    ));
}

#[test]
fn md_table_cells_preserves_escaped_pipes_for_register_capability_rows() {
    // Pin the markdown-as-data split: cementing escalation reads
    // `docs/v3-lens-capability-register.md` through this helper (see review on #638).
    let row = "| a.dag | TERMINAL | COMPLETE | v2 path | FoundCost(Int) \\| MissingCost | note |";
    let cells = md_table_cells(row);
    assert!(
        cells.len() >= 5,
        "capability ratchet expects Lens, Structural, Behavioral, v2, … columns; got {cells:?}"
    );
    assert_eq!(cells[1].trim(), "a.dag");
    assert_eq!(cells[3].trim(), "COMPLETE");
    assert_eq!(cells[4].trim(), "v2 path");
    assert_eq!(
        cells[5].trim(),
        "FoundCost(Int) | MissingCost",
        "cell-internal `|` must survive `\\|` escaping (v3 output column)"
    );
}

/// Lens basenames (e.g. `complexity.dag`) whose register row is `COMPLETE` with a
/// real v2 counterpart (excludes `None (v3-native)` and `N/A` — `TESTING.md`).
fn lens_basenames_requiring_v2_cementing_from_register_md() -> BTreeSet<String> {
    let md_path = workspace_root().join("docs/v3-lens-capability-register.md");
    let md = std::fs::read_to_string(&md_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", md_path.display()));
    let mut basenames = BTreeSet::new();
    for raw in md.lines() {
        let line = raw.trim();
        if !line.starts_with('|') || line.starts_with("|---") || line.contains("---|---") {
            continue;
        }
        let cells = md_table_cells(line);
        if cells.len() < 5 {
            continue;
        }
        let lens_cell = cells[1].trim();
        if lens_cell == "Lens" || !lens_cell.contains(".dag") {
            continue;
        }
        let behavioral = cells[3].trim();
        let v2 = cells[4].trim();
        if behavioral != "COMPLETE" {
            continue;
        }
        if !register_row_has_real_v2_counterpart(v2) {
            continue;
        }
        let basename = lens_cell.trim_matches('`').trim();
        if !basename.ends_with(".dag") {
            continue;
        }
        basenames.insert(basename.to_string());
    }
    basenames
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

fn read_lens_registry_name_lens_file_pairs(dag: &Dag) -> Vec<(String, String)> {
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
            (
                string_field(fields, "name", &binding),
                string_field(fields, "lens_file", &binding),
            )
        })
        .collect()
}

/// `regen_lens --lens <name>` keys that must have cementing modules per the
/// capability register escalation rule.
fn registry_names_required_by_register_and_regen() -> BTreeSet<String> {
    let basenames = lens_basenames_requiring_v2_cementing_from_register_md();
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load `src/v3/compiler/regen.dag` cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let rows = read_lens_registry_name_lens_file_pairs(&dag);
    let mut names = BTreeSet::new();
    let mut matched_basenames = BTreeSet::new();
    for (name, lens_file) in rows {
        let basename = Path::new(&lens_file)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| {
                panic!("registry entry `{name}` has lens_file without basename: {lens_file}")
            });
        if basenames.contains(basename) {
            names.insert(name);
            matched_basenames.insert(basename.to_string());
        }
    }
    let missing: Vec<_> = basenames.difference(&matched_basenames).cloned().collect();
    assert!(
        missing.is_empty(),
        "docs/v3-lens-capability-register.md escalates v2 cementing for lens basenames {missing:?}, \
         but no `LensRegistryEntry` in src/v3/compiler/regen.dag names those files — \
         fix the register table or add registry entries."
    );
    names
}

fn integration_rs_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("integration.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn assert_cementing_stem_wired_in_integration_rs(
    integration_rs: &str,
    stem: &str,
    registry_name: &str,
) {
    let expected = format!(r#"#[path = "integration/cementing/{stem}.rs"]"#);
    assert!(
        integration_rs_active_line_contains(integration_rs, &expected),
        "registry lens `{registry_name}` lists cementing stem `{stem}` but \
         `tests/integration.rs` has no **active** (non-`//`-comment) line with `{expected}` — add \
         `#[path = \"integration/cementing/{stem}.rs\"]` plus `mod {stem};` in the same PR as the \
         on-disk module. Commented-out `// #[path = …]` does not count."
    );
    let mod_needle = format!("mod {stem}");
    assert!(
        integration_rs_active_line_contains(integration_rs, &mod_needle),
        "registry lens `{registry_name}` lists cementing stem `{stem}` but \
         `tests/integration.rs` has no active line declaring `{mod_needle};` — wire the module."
    );
}

fn find_bind_value_port(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

#[test]
fn provenance_origin_of_cements_complete_row_via_compile_to_dag_fixture() {
    // Register row: `provenance.dag` — BEHAVIORALLY COMPLETE, v3-native.
    // Integration crate cannot reach `Dag`'s `pub(crate)` builder helpers
    // (`alloc_port_with_shape`); `compile_to_dag` fixtures still cement the
    // shipped `origin_of` **walk + glue** on the live lowering path. The five-way
    // `Behavior → Origin` mirror in this file stays in sync with `provenance.dag`;
    // richer mapping pins stay in-crate unit tests.
    // Exhaustive `NoProducer` / `Missing*` cases stay in `lib.rs::lens_provenance::tests`.
    let dag = compile_to_dag("let lit: Int = 7", "cementing_provenance_lit.v3").expect("compiles");
    assert_provenance_origin_matches_lens_authority(&dag, "lit", "cementing_provenance_lit");

    let dag =
        compile_to_dag("let sum: Int = 1 + 2", "cementing_provenance_sum.v3").expect("compiles");
    assert_provenance_origin_matches_lens_authority(&dag, "sum", "cementing_provenance_sum");
}

#[test]
fn cementing_escalation_slice_matches_capability_register() {
    let expected = registry_names_required_by_register_and_regen();
    let declared: BTreeSet<&str> = CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS
        .iter()
        .map(|(name, _)| *name)
        .collect();
    let expected_refs: BTreeSet<&str> = expected.iter().map(String::as_str).collect();
    assert_eq!(
        declared, expected_refs,
        "`CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS` must list exactly the registry `name` keys \
         for rows where docs/v3-lens-capability-register.md marks `COMPLETE` with a real v2 \
         counterpart (not `None (v3-native)` / not `N/A` per `TESTING.md`) — update the slice \
         (and cementing modules + `#[path]` wiring) in the same PR as the register promotion."
    );
}

#[test]
fn cementing_test_modules_exist_for_escalated_v2_complete_registry_claims() {
    let cementing_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("integration")
        .join("cementing");
    let integration_rs = integration_rs_text();
    for (registry_name, stem) in CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS {
        let path = cementing_dir.join(format!("{stem}.rs"));
        assert!(
            path.is_file(),
            "registry lens `{registry_name}` is listed for v2-complete cementing; expected cementing module at {}",
            path.display()
        );
        assert_cementing_stem_wired_in_integration_rs(&integration_rs, stem, registry_name);
    }
}
