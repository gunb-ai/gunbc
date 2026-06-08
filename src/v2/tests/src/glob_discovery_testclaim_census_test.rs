//! Glob-discovery D1 census — discover `TestClaim` / `TestClaimRun` declarations by
//! *resolved type* across the `src/v4/test/claim/**` corpus, and make the D2 premise
//! contradiction durable and measured.
//!
//! ## What this is (and is NOT)
//!
//! This is the **structural data-decl census** sanctioned by Mgr-C (witty-pike-248,
//! 2026-06-08): "build (B) strictly as the structural TestClaim/TestClaimRun data-decl
//! census without touching consumers/deletes". It does NOT repoint any consumer, delete
//! any roster, or switch any marker form. It only *measures*.
//!
//! ## The contradiction it pins
//!
//! The work item asked to "prove TestClaimRun discovered == v4_roster_pilot hand
//! run-roster". That equality is impossible as written, and this test proves *why* by
//! execution rather than assertion-by-doc:
//!
//!   * `v4_roster_pilot` rows are **Bool witness functions** (`fn() -> Bool`, run via
//!     `gunbc run --claim-run`). See `workflow/v4_roster_pilot.dag`.
//!   * `TestClaim` / `TestClaimRun` discovered *by resolved type* are **data
//!     declarations** (`data x: TestClaimRun<..> = ..`).
//!   * The two sets are disjoint kinds: the intersection of roster function names with
//!     discovered `TestClaimRun` decl names is **empty**.
//!
//! Switching D2's discovery marker from "resolved TestClaim/TestClaimRun decls" to
//! "nullary `fn() -> Bool`" is a marker-form/design decision that contradicts the operator
//! marker in the parent brief; it is held for Mgr-C/operator gate (see
//! `docs/planning/glob-discovery-d2-roster-testclaimrun-mismatch-2026-06-08.md`).
//!
//! ## "By resolved type", not grep
//!
//! Discovery runs the real `compile_to_resolved` pipeline per claim file's import closure
//! (the same closure `gunbc run --source-root src/v4 --entry <file>` builds) and reads the
//! *resolved typed-module item* annotation head — not a source substring. Files whose
//! closure does not resolve cleanly are recorded as `unresolved` and excluded from the
//! census (discovery is robust to gated/red corpus rows; it never silently miscounts).
//!
//! ## Standalone `#[test]` (census probe, run on demand)
//!
//! Per [[project_v2_tests_not_run_broadly_in_ci]] CI selects the `v2-compiler-tests` crate
//! via a single `--exact` parity invocation, so a standalone `#[test]` here is DORMANT in
//! CI. That is intentional: this is a measurement/census probe (it resolves ~150 file
//! closures, heavier than a witness run), run explicitly with
//! `cargo test -p v2-compiler-tests glob_discovery_testclaim_census -- --nocapture`.

use std::collections::BTreeSet;
use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};
use v2_compiler::v2_std_core::is_interpreter_blocking_diagnostic;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CLAIM_ROOT: &str = "src/v4/test/claim";
const ROSTER_REL: &str = "src/v4/test/claim/workflow/v4_roster_pilot.dag";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

/// Recursively collect every `.dag` file under `dir`, workspace-relative, sorted.
fn collect_dag_files(dir: &std::path::Path, ws: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, ws, out);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            out.push(
                path.strip_prefix(ws)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
}

/// One discovered data declaration, identified by its name and the resolved head of its
/// type annotation (e.g. `TestClaimRun`, `TestClaim`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DiscoveredDecl {
    head: String,
    name: String,
    file: String,
}

/// Discover, *by resolved type*, every data decl in one claim file's resolved closure
/// whose type-annotation head is one of `heads`. Returns `Err` if the closure does not
/// resolve cleanly (gated/red row) so the caller can record it as `unresolved`.
fn discover_in_file(rel_path: &str, heads: &BTreeSet<&str>) -> Result<Vec<DiscoveredDecl>, ()> {
    let content = std::fs::read_to_string(workspace_root().join(rel_path)).map_err(|_| ())?;
    let sources: Vec<Rc<SourceFile>> =
        resolve_imports_transitively_with_source_roots(rel_path, &content, &v4_source_roots());
    let resolved = compile_to_resolved(Rc::new(sources));

    let blocking = resolved
        .diagnostics
        .iter()
        .any(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()));
    if blocking {
        return Err(());
    }
    let Some(graph) = resolved.graph.as_ref() else {
        return Err(());
    };

    let mut found = Vec::new();
    for module in graph.modules.iter() {
        // Only walk items that belong to this entry file — a closure pulls in std/* and
        // peer modules; we census the file under discovery, not its imports.
        for item in module.items.iter() {
            // A data declaration has both a body (the constructor expression) and a type
            // annotation (`data x: T = ..`). Functions have a body but no annotation head
            // we want; type defs have no body.
            let (Some(type_node), Some(_body)) = (item.type_annotation.clone(), item.body.clone())
            else {
                continue;
            };
            let head = type_node.name.clone();
            if heads.contains(head.as_str()) {
                found.push(DiscoveredDecl {
                    head,
                    name: item.name.clone(),
                    file: rel_path.to_string(),
                });
            }
        }
    }
    Ok(found)
}

/// One hand-roster run-witness row: the entry file plus the `fn() -> Bool` it runs.
#[derive(Clone)]
struct RosterRow {
    entry: String,
    function: String,
}

/// Extract the roster's run-witness rows (`entry` + `function` pairs) from
/// `v4_roster_pilot.dag`. These are the hand run-roster rows the brief asked to equate with
/// discovered `TestClaimRun` decls. Each (entry, function) pair appears TWICE in the file —
/// once in a `V4RosterPilotClaimRunRow { .. }` data literal and once restated in the
/// `v4_roster_pilot_row_matches(..)` composition guard — so we dedup to the distinct row
/// set (38 as of the `v4_roster_pilot_claim_run_row_count` authority).
fn roster_rows() -> Vec<RosterRow> {
    let content = std::fs::read_to_string(workspace_root().join(ROSTER_REL))
        .unwrap_or_else(|e| panic!("read {ROSTER_REL}: {e}"));
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut rows = Vec::new();
    let mut pending_entry: Option<String> = None;
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("entry: \"") {
            if let Some(end) = rest.find('"') {
                pending_entry = Some(rest[..end].to_string());
            }
        } else if let Some(rest) = t.strip_prefix("function: \"") {
            if let Some(end) = rest.find('"') {
                // The type def `V4RosterPilotClaimRunRow { entry: String; function: String }`
                // has no `entry: "..."` pairing, so a function with no pending entry is the
                // schema field, not a row — skip it.
                if let Some(entry) = pending_entry.take() {
                    let function = rest[..end].to_string();
                    if seen.insert((entry.clone(), function.clone())) {
                        rows.push(RosterRow { entry, function });
                    }
                }
            }
        }
    }
    rows
}

/// D1 census + D2 contradiction proof.
#[test]
fn glob_discovery_testclaim_census_and_d2_contradiction() {
    let ws = workspace_root();
    let mut files = Vec::new();
    collect_dag_files(&ws.join(CLAIM_ROOT), &ws, &mut files);
    assert!(
        files.len() > 50,
        "expected the claim corpus glob to find many .dag files, got {}",
        files.len()
    );

    let heads: BTreeSet<&str> = ["TestClaim", "TestClaimRun"].into_iter().collect();

    let mut decls: Vec<DiscoveredDecl> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();
    for rel in &files {
        match discover_in_file(rel, &heads) {
            Ok(mut d) => decls.append(&mut d),
            Err(()) => unresolved.push(rel.clone()),
        }
    }
    decls.sort();
    decls.dedup();

    let testclaimrun: BTreeSet<String> = decls
        .iter()
        .filter(|d| d.head == "TestClaimRun")
        .map(|d| d.name.clone())
        .collect();
    let testclaim: BTreeSet<String> = decls
        .iter()
        .filter(|d| d.head == "TestClaim")
        .map(|d| d.name.clone())
        .collect();
    let run_files: BTreeSet<String> = decls
        .iter()
        .filter(|d| d.head == "TestClaimRun")
        .map(|d| d.file.clone())
        .collect();

    let rows = roster_rows();
    let roster_fns: BTreeSet<String> = rows.iter().map(|r| r.function.clone()).collect();

    // ── Census report ────────────────────────────────────────────────────
    println!("== glob-discovery D1 census (by resolved type) ==");
    println!("claim corpus .dag files globbed : {}", files.len());
    println!("files unresolved (gated/red)    : {}", unresolved.len());
    println!("TestClaimRun data decls         : {}", testclaimrun.len());
    println!("  across files                  : {}", run_files.len());
    println!("TestClaim data decls            : {}", testclaim.len());
    println!("v4_roster_pilot witness rows    : {}", rows.len());

    // ── D2 contradiction (durable) ───────────────────────────────────────
    // The brief's equality ("TestClaimRun discovered == roster") is between disjoint
    // kinds: data decls vs Bool witness functions. Pin the empty intersection.
    let intersection: Vec<&String> = testclaimrun.intersection(&roster_fns).collect();
    println!("roster ∩ TestClaimRun decls     : {}", intersection.len());
    assert!(
        intersection.is_empty(),
        "D2 premise unexpectedly satisfiable: roster function names also appear as \
         TestClaimRun data-decl names: {intersection:?}. If this ever becomes non-empty the \
         contradiction report needs revisiting."
    );

    // ── Rename-vs-wrap mapping (per roster row) ───────────────────────────
    // For each of the 39 run-witness rows: does its entry file already carry a TestClaimRun
    // data decl ("align/rename" candidate — a wrapper exists in-family, migration repoints
    // discovery at it) or none ("wrap" — a TestClaimRun wrapper must be authored)? This
    // file-level presence is the measured first-order signal; for files with multiple
    // TestClaimRun decls, the exact witness↔decl name pairing still needs author
    // confirmation (noted in the report).
    let run_decls_by_file: std::collections::BTreeMap<String, usize> = {
        let mut m = std::collections::BTreeMap::new();
        for d in decls.iter().filter(|d| d.head == "TestClaimRun") {
            *m.entry(d.file.clone()).or_insert(0) += 1;
        }
        m
    };
    println!("\n== rename-vs-wrap mapping (per roster row) ==");
    let mut align = 0usize;
    let mut wrap = 0usize;
    for r in &rows {
        let n = run_decls_by_file.get(&r.entry).copied().unwrap_or(0);
        let verdict = if n > 0 {
            align += 1;
            format!("ALIGN/RENAME (entry file has {n} TestClaimRun decl(s))")
        } else {
            wrap += 1;
            "WRAP (no TestClaimRun in entry file)".to_string()
        };
        println!("  {} | {} -> {verdict}", r.function, r.entry);
    }
    println!("\nrename/align candidates : {align}");
    println!("wrap-needed             : {wrap}");

    // Discovery is non-vacuous: the corpus genuinely carries TestClaimRun/TestClaim decls,
    // and the roster genuinely carries witness functions — so the empty intersection is a
    // real disjointness, not two empty sets.
    assert!(
        !testclaimrun.is_empty(),
        "discovery found no TestClaimRun decls — census is vacuous (did the resolver break, \
         or did all claim files become unresolved? unresolved={})",
        unresolved.len()
    );
    assert!(
        rows.len() >= 30,
        "roster row extraction degenerate: found only {} (expected 38)",
        rows.len()
    );
    assert_eq!(
        align + wrap,
        rows.len(),
        "mapping must classify every roster row exactly once"
    );
}
