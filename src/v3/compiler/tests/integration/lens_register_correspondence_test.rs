//! **Layer:** integration
//!
//! Ratchet for Discipline rule #1 of
//! `docs/v3-lens-capability-register.md`:
//!
//! > Every registry entry in `src/v3/compiler/regen.dag` requires a
//! > row here.
//!
//! Until now this was manual — a new `LensRegistryEntry` could land
//! in `regen.dag` without a corresponding capability-register row,
//! and the discipline held only by reviewer attention. This test
//! automates it: every `LensRegistryEntry` in `regen.dag` must name
//! a `lens_file` whose basename appears as a lens row in the
//! capability table. On drift, the assert prints both the missing
//! basenames (regen entries absent from the register) and the
//! registry-visible set, so the fix is a single register edit.
//!
//! Directionality is the one written into the register's Discipline
//! section — regen → register is required; extra register rows are
//! allowed. `idempotency.dag` and `parallelism.dag` are the current
//! example: both have register rows (as `BEHAVIORALLY STUB` lenses
//! whose authority lives in Rust) but no `regen.dag` entry, because
//! they are not regenerated into a `lens_*_generated.rs`. That is
//! exactly the posture the register documents; a bidirectional
//! ratchet would misread those rows as drift.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use v3_compiler::dag::{Dag, Declaration, FieldValue, LiteralBits, ValueBody};

const R3_LENS_BEHAVIORAL_PARITY_SCOPE: &[&str] = &[
    "complexity.dag",
    "cost.dag",
    "parallelism.dag",
    "effect_enumeration.dag",
];

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

/// `(name, lens_file_basename)` for every `LensRegistryEntry` in
/// `src/v3/compiler/regen.dag`. The basename is what the register's
/// first column shows (e.g. `complexity.dag`).
fn regen_lens_file_basenames() -> BTreeSet<String> {
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
            let lens_file = string_field(fields, "lens_file", &binding);
            Path::new(&lens_file)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_else(|| {
                    panic!("registry entry `{binding}` has lens_file without basename: {lens_file}")
                })
                .to_string()
        })
        .collect()
}

/// Lens basenames (first column of the capability table) that appear
/// as rows in `docs/v3-lens-capability-register.md`. Scoped to the
/// `## Capability table` section — any future table elsewhere in the
/// document (examples, migration notes, appendix) must not produce a
/// false pass when the capability row itself is missing. Uses the
/// same `|`-split discipline as `cementing_lens_registry_dispatch_test`,
/// but with no behavioral/v2-column filters — any row mentioning a
/// `.dag` lens inside the capability section counts here.
const CAPABILITY_TABLE_HEADING: &str = "## Capability table";

fn md_table_cells(line: &str) -> Vec<String> {
    // Capability rows escape `|` inside cells as `\|` (see v3 output column).
    let tmp = line.replace("\\|", "\u{241f}");
    tmp.split('|')
        .map(|s| s.replace('\u{241f}', "|").trim().to_string())
        .collect()
}

fn capability_table_rows() -> Vec<(String, String)> {
    let md_path = workspace_root().join("docs/v3-lens-capability-register.md");
    let md = std::fs::read_to_string(&md_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", md_path.display()));
    assert!(
        md.lines().any(|l| l.trim() == CAPABILITY_TABLE_HEADING),
        "docs/v3-lens-capability-register.md must contain a `{CAPABILITY_TABLE_HEADING}` \
         heading; this test scopes its table scan to that section and cannot run \
         without it. If the heading was renamed, update `CAPABILITY_TABLE_HEADING` \
         in this test in the same PR."
    );
    let mut in_capability_section = false;
    let mut rows = Vec::new();
    for raw in md.lines() {
        let line = raw.trim();
        if line == CAPABILITY_TABLE_HEADING {
            in_capability_section = true;
            continue;
        }
        // Any subsequent `## ` heading closes the section (sub-headings
        // `### ` inside capability-table prose are fine; only a peer
        // `## ` heading ends it).
        if in_capability_section && line.starts_with("## ") {
            in_capability_section = false;
        }
        if !in_capability_section {
            continue;
        }
        if !line.starts_with('|') || line.starts_with("|---") || line.contains("---|---") {
            continue;
        }
        let cells = md_table_cells(line);
        if cells.len() < 4 {
            continue;
        }
        let lens_cell = cells[1].trim();
        if lens_cell == "Lens" {
            continue;
        }
        let basename = lens_cell.trim_matches('`').trim();
        if !basename.ends_with(".dag") {
            continue;
        }
        rows.push((basename.to_string(), cells[3].trim().to_string()));
    }
    rows
}

fn register_lens_basenames() -> BTreeSet<String> {
    capability_table_rows()
        .into_iter()
        .map(|(basename, _)| basename)
        .collect()
}

#[test]
fn every_regen_lens_entry_has_a_capability_register_row() {
    let regen = regen_lens_file_basenames();
    let register = register_lens_basenames();
    let missing: Vec<&String> = regen.difference(&register).collect();
    assert!(
        missing.is_empty(),
        "Discipline rule #1 of `docs/v3-lens-capability-register.md` \
         requires every `LensRegistryEntry` in `src/v3/compiler/regen.dag` \
         to have a matching row in the capability table. The following \
         lens basename(s) are in `regen.dag` but have no register row: \
         {missing:?}. Fix: add a row to the `## Capability table` section \
         of `docs/v3-lens-capability-register.md` for each missing lens, \
         declaring both its structural and behavioral status. Current \
         register-visible basenames: {register:?}."
    );
}

#[test]
fn r3_gate_83_lens_capability_register_scope_is_explicit() {
    let register = register_lens_basenames();
    let missing: Vec<_> = R3_LENS_BEHAVIORAL_PARITY_SCOPE
        .iter()
        .copied()
        .filter(|basename| !register.contains(*basename))
        .collect();
    assert!(
        missing.is_empty(),
        "R3 gate #83 (`lens_capability_register_zero_proxy_zero_stub`) is defined over \
         the four T-Lens-Behavioral-Parity rows: complexity, cost, parallelism, and \
         effect_enumeration. Missing capability-register row(s): {missing:?}."
    );
}

#[test]
#[ignore = "strict-fire gate #83; unignore when cost/parallelism sibling slices remove PROXY/STUB"]
fn r3_gate_83_lens_capability_register_has_zero_proxy_zero_stub() {
    let rows = capability_table_rows();
    let blockers: Vec<_> = R3_LENS_BEHAVIORAL_PARITY_SCOPE
        .iter()
        .copied()
        .filter_map(|basename| {
            let behavioral = rows
                .iter()
                .find(|(row_basename, _)| row_basename == basename)
                .map(|(_, behavioral)| behavioral.as_str())
                .unwrap_or("<missing>");
            let normalized = behavioral
                .trim()
                .trim_matches('*')
                .trim()
                .to_ascii_uppercase();
            (normalized == "PROXY" || normalized == "STUB")
                .then(|| format!("{basename}: {behavioral}"))
        })
        .collect();
    assert!(
        blockers.is_empty(),
        "R3 gate #83 requires ZERO PROXY / ZERO STUB in the capability register for \
         the four in-R3 T-Lens-Behavioral-Parity lenses. Remaining blocker(s): {blockers:?}."
    );
}

#[test]
fn r3_gate_83_current_register_blockers_are_explicit() {
    let rows = capability_table_rows();
    let blockers: Vec<_> = R3_LENS_BEHAVIORAL_PARITY_SCOPE
        .iter()
        .copied()
        .filter_map(|basename| {
            let behavioral = rows
                .iter()
                .find(|(row_basename, _)| row_basename == basename)
                .map(|(_, behavioral)| behavioral.as_str())
                .unwrap_or("<missing>");
            let normalized = behavioral
                .trim()
                .trim_matches('*')
                .trim()
                .to_ascii_uppercase();
            (normalized == "PROXY" || normalized == "STUB")
                .then(|| format!("{basename}: {behavioral}"))
        })
        .collect();
    assert_eq!(
        blockers,
        vec!["cost.dag: **PROXY**", "parallelism.dag: **STUB**"],
        "Gate #83 is not ready to strict-fire until sibling lens-completion slices \
         remove all PROXY/STUB statuses. If this changed, update the strict-fire \
         posture in `r3_gate_83_lens_capability_register_has_zero_proxy_zero_stub`."
    );
}
