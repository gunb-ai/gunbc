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
//! **Band-C v2 cementing (mechanical doc ↔ structural register).** The
//! markdown `## Capability table` rows that read `BEHAVIORALLY COMPLETE` with a
//! real v2 counterpart (not v3-native / bare `N/A`) must match the projection
//! from `std.verification` `lens_capability_register_rows` (`LensCapabilityBehavioralComplete`
//! and `LensCapabilityV2RealV2`). This preserves the retired
//! `cementing_lens_registry_dispatch_test` ratchet without reintroducing a second
//! register body in `cementing_dispatch.dag`.
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

use v3_compiler::cementing_dispatch;
use v3_compiler::dag::{Dag, Declaration, FieldValue, LiteralBits, ValueBody};
use v3_compiler::r3_gate_87_cementing_regen_runner_suites::r3_gate_87_cementing_regen_lens_names_for_runner_table;

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
/// same `|`-split discipline as the markdown capability table and the
/// `lens_basename` column in `src/v3/std/verification.dag`'s
/// `lens_capability_register_rows`,
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

fn normalize_capability_table_markdown_token(cell: &str) -> String {
    cell.trim().trim_matches('*').trim().to_ascii_uppercase()
}

/// True when the markdown **v2 counterpart** cell names a real v2 oracle path (so the row
/// belongs in the Band-C v2-cementing slice alongside `LensCapabilityV2RealV2` in
/// `lens_capability_register_rows`).
fn md_v2_counterpart_is_real_v2_oracle(v2_cell: &str) -> bool {
    let t = v2_cell.trim();
    if t.is_empty() {
        return false;
    }
    let lower = t.to_ascii_lowercase();
    if lower.contains("none (v3-native)") {
        return false;
    }
    if matches!(lower.as_str(), "n/a" | "—" | "-" | "none") {
        return false;
    }
    true
}

/// Lens basenames for markdown rows that read **behaviorally complete** with a **real v2**
/// counterpart column (excludes v3-native / bare `N/A` rows such as `idempotency.dag`).
fn capability_md_v2_cementing_basenames() -> BTreeSet<String> {
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
    let mut out = BTreeSet::new();
    for raw in md.lines() {
        let line = raw.trim();
        if line == CAPABILITY_TABLE_HEADING {
            in_capability_section = true;
            continue;
        }
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
        if cells.len() < 5 {
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
        let behavioral = normalize_capability_table_markdown_token(&cells[3]);
        if behavioral != "COMPLETE" {
            continue;
        }
        if !md_v2_counterpart_is_real_v2_oracle(&cells[4]) {
            continue;
        }
        out.insert(basename.to_string());
    }
    out
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
        Vec::<String>::new(),
        "Gate #83 is not ready to strict-fire until sibling lens-completion slices \
         remove all PROXY/STUB statuses. If this changed, update the strict-fire \
         posture in `r3_gate_83_lens_capability_register_has_zero_proxy_zero_stub`."
    );
}

#[test]
fn lens_capability_register_rows_match_md_v2_cementing_projection() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap Dag should load cleanly for lens capability register ratchet, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let structural = cementing_dispatch::lens_capability_register_v2_cementing_basenames(&dag)
        .expect("read v2 cementing projection from lens_capability_register_rows");
    let markdown = capability_md_v2_cementing_basenames();
    assert_eq!(
        structural, markdown,
        "`docs/v3-lens-capability-register.md` Band-C v2-cementing slice (COMPLETE + real v2 counterpart column) \
         must match `std.verification` `lens_capability_register_rows` (`LensCapabilityBehavioralComplete` + \
         `LensCapabilityV2RealV2`). Update both in the same PR when promoting or narrowing a lens. \
         structural={structural:?} markdown={markdown:?}"
    );
}

// ---------------------------------------------------------------------------
// R3 gate #87 — D3 complete-lens register ratchet for promoted R3 lenses.
//
// Discipline rule: when a `regen.dag` `LensRegistryEntry` lens is marked
// `BEHAVIORALLY COMPLETE` in `docs/v3-lens-capability-register.md`, the gate-87
// cementing closure must be in place: the entry's `name` is in the runner
// inventory, and the per-lens `tests/dag/t_r3_gate_87_cementing_regen_<name>.dag`
// receipt exists. This fails closed on a PR that promotes a row to COMPLETE
// without landing the matching cementing surfaces in the same PR.
//
// See `docs/briefs/r3-gate-87-lens-completeness-test-discipline-decomposition.md`
// G87-D3 and `docs/briefs/r3-gate-87-lens-cementing-closure-audit.md`.

/// Lens basenames whose capability-table `behavioral` cell is `COMPLETE`
/// (any v2-counterpart column — Real v2 or v3-native). Bare `N/A`,
/// `PARTIAL`, `PROXY`, `STUB` are excluded.
fn capability_md_complete_basenames() -> BTreeSet<String> {
    capability_table_rows()
        .into_iter()
        .filter_map(|(basename, behavioral)| {
            (normalize_capability_table_markdown_token(&behavioral) == "COMPLETE")
                .then_some(basename)
        })
        .collect()
}

/// Map from `lens_file` basename (e.g. `complexity.dag`) to registry `name`
/// (e.g. `cost`) for every `LensRegistryEntry` in `regen.dag`.
fn regen_basename_to_name() -> std::collections::BTreeMap<String, String> {
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
            let name = string_field(fields, "name", &binding);
            let lens_file = string_field(fields, "lens_file", &binding);
            let basename = Path::new(&lens_file)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_else(|| {
                    panic!("registry entry `{binding}` has lens_file without basename: {lens_file}")
                })
                .to_string();
            (basename, name)
        })
        .collect()
}

#[test]
fn r3_gate_87_complete_md_regen_rows_have_runner_suite_and_receipt() {
    let complete = capability_md_complete_basenames();
    let regen = regen_basename_to_name();
    let runner_names = r3_gate_87_cementing_regen_lens_names_for_runner_table();
    let root = workspace_root();

    let mut missing_runner: Vec<String> = Vec::new();
    let mut missing_receipt: Vec<String> = Vec::new();
    for basename in &complete {
        let Some(name) = regen.get(basename) else {
            // Non-`regen` complete lenses are scoped to G87-D5
            // (`Non-regen Complete-Lens Census Reconciliation`), not this ratchet.
            continue;
        };
        if !runner_names.contains(name) {
            missing_runner.push(format!("{basename} (name={name})"));
        }
        let receipt_rel =
            format!("src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_{name}.dag");
        if !root.join(&receipt_rel).is_file() {
            missing_receipt.push(receipt_rel);
        }
    }
    assert!(
        missing_runner.is_empty() && missing_receipt.is_empty(),
        "R3 gate #87 D3 complete-lens register ratchet: every `regen.dag` lens marked \
         `BEHAVIORALLY COMPLETE` in `docs/v3-lens-capability-register.md` must have a \
         matching runner suite (in `R3_GATE_87_CEMENTING_REGEN_SUITES`) and a \
         `tests/dag/t_r3_gate_87_cementing_regen_<name>.dag` cementing TestClaim. \
         Promoting a row to COMPLETE without landing both surfaces in the same PR is a \
         process failure. \
         missing_runner_entries={missing_runner:?} missing_receipt_files={missing_receipt:?}"
    );
}

#[test]
#[ignore = "strict-fire G87-D3 / G87-D1+D2; unignore when no `BEHAVIORALLY COMPLETE` \
            regen row's `t_r3_gate_87_cementing_regen_<name>.dag` still uses the bare \
            `Compiles` placeholder. Today `variant_payload.dag` (COMPLETE) remains \
            `Compiles` pending `VariantPayloadShapeLookup` expected-literal authoring \
            per `docs/briefs/r3-gate-87-lens-cementing-closure-audit.md` P5 deferral."]
fn r3_gate_87_complete_md_regen_receipts_use_behavioral_predicate() {
    let complete = capability_md_complete_basenames();
    let regen = regen_basename_to_name();
    let root = workspace_root();

    let mut bare_compiles: Vec<String> = Vec::new();
    for basename in &complete {
        let Some(name) = regen.get(basename) else {
            continue;
        };
        let receipt_rel =
            format!("src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_{name}.dag");
        let path = root.join(&receipt_rel);
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let uses_bare_compiles = source
            .lines()
            .filter_map(|l| l.split("//").next())
            .any(|code| {
                code.split_whitespace()
                    .collect::<Vec<_>>()
                    .windows(2)
                    .any(|w| w == ["predicate:", "Compiles,"] || w == ["predicate:", "Compiles"])
            });
        if uses_bare_compiles {
            bare_compiles.push(receipt_rel);
        }
    }
    assert!(
        bare_compiles.is_empty(),
        "R3 gate #87 D3 strict-fire: `BEHAVIORALLY COMPLETE` regen rows must cement with a \
         behavioral `TestPredicate` (`DifferentialEquals` / `LensOutputEquals` / \
         `SymbolicCostExprEquals` / `BinaryDimensionReportEquals`) — not bare `Compiles`. \
         Receipts still on bare `Compiles`: {bare_compiles:?}"
    );
}
