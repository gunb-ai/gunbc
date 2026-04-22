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

fn register_lens_basenames() -> BTreeSet<String> {
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
    let mut basenames = BTreeSet::new();
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
        // `|`-split with `\|` escape (capability rows use it in the
        // v3-output column).
        let tmp = line.replace("\\|", "\u{241f}");
        let cells: Vec<String> = tmp
            .split('|')
            .map(|s| s.replace('\u{241f}', "|").trim().to_string())
            .collect();
        if cells.len() < 2 {
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
        basenames.insert(basename.to_string());
    }
    basenames
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
