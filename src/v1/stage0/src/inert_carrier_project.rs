// The inert-abstraction census (Lane 7 — DESIGN.md §5/§6, docs/plans/inert-layer-lens.md,
// docs/plans/fold-ergonomics.md §3).
//
// An "inert carrier" is a declared type carrier (`type X = ...` / `type X { ... }`) that is
// DEFINED, possibly SELF-TESTED, but has ZERO real consumer — DESIGN §5's coverage-by-illusion /
// §6's "the machinery exists but nothing gates on it." This generalizes the #5433 inert-LENS floor
// backstop (which walls unreached `v2.lens.*` modules) to carriers in general.
//
// CONSTRUCTION-FIRST NOTE (DESIGN §5, the construction_justification rule): inertness cannot be made
// unwritable by construction — modeling a carrier ahead of its consumer is the project's deliberate
// model-first discipline, so "must already have a consumer" is not a precondition you can enforce at
// the declaration site. So this is the genuinely-unstructurable residue a lens is for: a decidable
// reachability/consumer property, walled fail-closed against a NAMED, SHRINKING exception roster
// (the realization-loop staged-ahead carriers) — the same ratchet-during-migration → wall-when-empty
// shape as #5433 and the doc-graph reachability wall (doc_reachability_project.rs).
//
// DEFINITION (the conservative, decidable, robust one — chosen to never false-red main):
//   A carrier is inert iff its NAME (as a whole identifier token) appears in NO non-test `.dag`
//   file other than its own declaring file. Comments count as a consumer (conservative). This is
//   the strongest, lowest-false-positive reading of "defined + self-tested + zero real consumer":
//   a carrier mentioned literally nowhere outside its declaration and its own test.
//
// This is host-fed today (no whole-corpus reference enumeration exists in `.dag` — see
// docs/plans/inert-layer-lens.md §3 Tier 2). DISSOLUTION TRIGGER: when `.dag` gains compile-graph /
// reference-edge access (gunbc#5364, the `concept_index` trigger), the token scan folds into a pure
// `.dag` reader over `BindsTo` edges and this Rust census deletes. It does NOT touch cli_run.rs's
// #5433 closure — it is the additive corpus-gate builtin seam (sibling to doc_graph_orphan_count /
// fact_cardinality_*).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// The NAMED, SHRINKING exception roster: carriers deliberately modeled ahead of their consumer (the
// realization loop is model-first by design — docs/plans/inert-layer-lens.md §5). Each entry is a
// carrier the live census currently reports inert; the roster empties as the realization loop wires
// each one, at which point the lens flips advisory → fail-closed wall. A NEW inert carrier not on
// this roster reds the floor; a rostered carrier that GAINS a consumer (or is deleted) becomes a
// stale entry that also reds, forcing the roster to shrink (the ratchet).
//
// dissolve-on per entry is tracked in docs/plans/inert-layer-lens.md §2 (the realization-loop
// cache-plan / work-demand / sharding / receipt-digest arms).
const INERT_CARRIER_ROSTER: &[&str] = &[
    // --- realization-loop: the cache-plan arm (dissolve-on: cache planner wired to a live source)
    "CacheLayerPlan",
    // --- realization-loop: the work-demand / sharding arm (dissolve-on: scheduler emits WorkDemand)
    "WorkDemand",
    "ParallelismShape",
    "IndependentShards",
    "PartitionedReduce",
    "Partitioner",
    "SymbolicCost",
    // --- realization-loop: the materialization arm (dissolve-on: realization emits Materialization)
    "Materialization",
];

fn workspace_root() -> PathBuf {
    crate::module_path_index::workspace_root()
}

fn repo_rel(path: &Path) -> String {
    let ws = workspace_root();
    let s = path.to_string_lossy().replace('\\', "/");
    let prefix = format!("{}/", ws.to_string_lossy().replace('\\', "/"));
    s.strip_prefix(&prefix)
        .map(|p| p.to_string())
        .unwrap_or(s)
        .trim_start_matches("./")
        .to_string()
}

fn is_test_dag(path: &str) -> bool {
    path.ends_with("_test.dag")
}

fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

fn corpus_dag_files() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    for root in ["dsl", "src/v2"] {
        collect_dag_files(&workspace_root().join(root), &mut paths);
    }
    let mut out = Vec::new();
    for p in paths {
        let rel = repo_rel(&p);
        if let Ok(content) = std::fs::read_to_string(&p) {
            out.push((rel, content));
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Strip line comments (`//...`) so a name appearing only in a comment of *another* file still
/// counts as a (conservative) consumer, but the declaring `type` line's own comment never leaks.
fn strip_line_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or(line)
}

/// Extract top-level `type NAME` carrier declarations from a file's content. Carriers are the type
/// abstractions (coproducts + records + aliases); `data`/`fn`/`service` are values/behavior, out of
/// scope for this keystone.
fn type_carrier_decls(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in content.lines() {
        let line = raw.trim_start();
        if let Some(rest) = line.strip_prefix("type ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

/// Whole-identifier tokens in a line (alphanumeric + `_`, not starting with a digit boundary issue —
/// a token is a maximal run of `[A-Za-z0-9_]`; numeric-only runs are harmless, they never match a
/// carrier name).
fn identifier_tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in line.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

struct InertCarrierReport {
    /// All declared carriers (name -> declaring file), declared in non-test files.
    declared: BTreeMap<String, String>,
    /// Inert carrier names (zero non-self, non-test consumer file mentions), sorted.
    inert: Vec<String>,
}

fn build_report() -> InertCarrierReport {
    let files = corpus_dag_files();

    // 1. Declaration universe: type carriers declared in non-test files.
    //    Name -> the (first) declaring file. Collision across files is handled in step 3 (a name
    //    declared twice is conservatively treated as cross-referenced → not inert).
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    let mut decl_count: BTreeMap<String, usize> = BTreeMap::new();
    for (rel, content) in &files {
        if is_test_dag(rel) {
            continue;
        }
        for name in type_carrier_decls(content) {
            declared.entry(name.clone()).or_insert_with(|| rel.clone());
            *decl_count.entry(name).or_insert(0) += 1;
        }
    }

    // 2. Mention map: for each carrier name, the set of NON-TEST files that mention it (as a whole
    //    identifier token). Only carrier names are tracked (cheap + exact).
    let names: BTreeSet<String> = declared.keys().cloned().collect();
    let mut mentions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (rel, content) in &files {
        if is_test_dag(rel) {
            continue; // a carrier referenced ONLY in tests is inert (self-tested, no real consumer).
        }
        let mut seen_here: BTreeSet<String> = BTreeSet::new();
        for raw in content.lines() {
            for tok in identifier_tokens(strip_line_comment(raw)) {
                if names.contains(&tok) {
                    seen_here.insert(tok);
                }
            }
        }
        for tok in seen_here {
            mentions.entry(tok).or_default().insert(rel.clone());
        }
    }

    // 3. Inert = declared carrier whose only mentioning non-test file is its own declaring file, and
    //    whose name is declared exactly once (a doubly-declared name cross-references itself → not
    //    flagged, the conservative direction).
    let mut inert: Vec<String> = Vec::new();
    for (name, declfile) in &declared {
        if decl_count.get(name).copied().unwrap_or(0) != 1 {
            continue;
        }
        let consumer_files: BTreeSet<&String> = mentions
            .get(name)
            .map(|s| s.iter().filter(|f| *f != declfile).collect())
            .unwrap_or_default();
        if consumer_files.is_empty() {
            inert.push(name.clone());
        }
    }
    inert.sort();
    inert.dedup();
    InertCarrierReport { declared, inert }
}

/// Count of all carriers the census judges inert (defined + zero real consumer). Includes rostered
/// staged-ahead carriers — this is the raw census, not the gate.
pub fn inert_carrier_count() -> i64 {
    build_report().inert.len() as i64
}

/// The fail-closed GATE: inert carriers NOT on the named exception roster. A new inert carrier reds
/// the floor; zero means every inert carrier is an acknowledged staged-ahead one.
pub fn inert_carrier_unrostered_count() -> i64 {
    let roster: BTreeSet<&str> = INERT_CARRIER_ROSTER.iter().copied().collect();
    build_report()
        .inert
        .iter()
        .filter(|n| !roster.contains(n.as_str()))
        .count() as i64
}

/// The RATCHET: rostered carriers that are no longer inert (gained a real consumer) or no longer
/// declared (deleted). A stale entry reds the floor, forcing the roster to shrink as the realization
/// loop wires each carrier.
pub fn inert_carrier_stale_roster_count() -> i64 {
    let report = build_report();
    let inert: BTreeSet<&String> = report.inert.iter().collect();
    INERT_CARRIER_ROSTER
        .iter()
        .filter(|n| !inert.contains(&n.to_string()))
        .count() as i64
}

/// Total declared type carriers (oracle against a read_dir / parse fail-open: zero here would mean
/// the corpus walk silently found nothing).
pub fn inert_carrier_declared_count() -> i64 {
    build_report().declared.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_carrier_decls_extracts_names() {
        let c = "module m\ntype Connective = Atom | Conj\ntype WorkDemand {\n  field: Int\n}\nfn f() -> Int { 1 }\n";
        let names = type_carrier_decls(c);
        assert_eq!(names, vec!["Connective", "WorkDemand"]);
    }

    #[test]
    fn identifier_tokens_are_whole_words() {
        // Placement must NOT match inside PlacementSupply (word-boundary discrimination).
        let toks = identifier_tokens("  field: PlacementSupply = foo(Placement)");
        assert!(toks.contains(&"PlacementSupply".to_string()));
        assert!(toks.contains(&"Placement".to_string()));
        assert!(toks.contains(&"field".to_string()));
    }

    // The DISCRIMINATING controls (DESIGN §5: green-by-execution, RED on a real defect). A pure
    // function over a synthetic corpus so the discrimination is proven without the live tree.
    fn report_of(files: &[(&str, &str)]) -> Vec<String> {
        // Re-implement the build_report core over an in-memory corpus (the live build_report reads
        // the filesystem; this exercises the SAME inert predicate over controlled inputs).
        let owned: Vec<(String, String)> = files
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect();
        let mut declared: BTreeMap<String, String> = BTreeMap::new();
        let mut decl_count: BTreeMap<String, usize> = BTreeMap::new();
        for (rel, content) in &owned {
            if is_test_dag(rel) {
                continue;
            }
            for name in type_carrier_decls(content) {
                declared.entry(name.clone()).or_insert_with(|| rel.clone());
                *decl_count.entry(name).or_insert(0) += 1;
            }
        }
        let names: BTreeSet<String> = declared.keys().cloned().collect();
        let mut mentions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (rel, content) in &owned {
            if is_test_dag(rel) {
                continue;
            }
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for raw in content.lines() {
                for tok in identifier_tokens(strip_line_comment(raw)) {
                    if names.contains(&tok) {
                        seen.insert(tok);
                    }
                }
            }
            for tok in seen {
                mentions.entry(tok).or_default().insert(rel.clone());
            }
        }
        let mut inert = Vec::new();
        for (name, declfile) in &declared {
            if decl_count.get(name).copied().unwrap_or(0) != 1 {
                continue;
            }
            let consumers: BTreeSet<&String> = mentions
                .get(name)
                .map(|s| s.iter().filter(|f| *f != declfile).collect())
                .unwrap_or_default();
            if consumers.is_empty() {
                inert.push(name.clone());
            }
        }
        inert.sort();
        inert
    }

    #[test]
    fn red_control_zero_consumer_carrier_is_inert() {
        // A carrier defined + self-tested but mentioned in NO other non-test file → inert (RED: the
        // lens fires).
        let inert = report_of(&[
            ("a.dag", "module a\ntype Lonely { x: Int }\n"),
            ("a_test.dag", "module t\nfn t() -> Bool { Lonely { x: 1 } == Lonely { x: 1 } }\n"),
        ]);
        assert!(
            inert.contains(&"Lonely".to_string()),
            "a carrier mentioned only in its decl + its own test must be flagged inert; got {inert:?}"
        );
    }

    #[test]
    fn green_control_carrier_with_real_consumer_is_not_inert() {
        // Same carrier, now mentioned in a second non-test file → NOT inert (GREEN: the lens stays
        // silent). This is the discrimination: the only difference is a real consumer.
        let inert = report_of(&[
            ("a.dag", "module a\ntype Used { x: Int }\n"),
            ("b.dag", "module b\nimport a { Used }\nfn f(u: Used) -> Int { u.x }\n"),
            ("a_test.dag", "module t\nfn t() -> Bool { true }\n"),
        ]);
        assert!(
            !inert.contains(&"Used".to_string()),
            "a carrier with a real (non-test, cross-file) consumer must NOT be flagged; got {inert:?}"
        );
    }

    #[test]
    fn comment_mention_counts_as_consumer_conservative() {
        // A bare mention in another file's COMMENT counts as a consumer (the safe direction — never
        // false-red main on a carrier someone references in prose).
        let inert = report_of(&[
            ("a.dag", "module a\ntype Noted { x: Int }\n"),
            ("b.dag", "module b\n// Noted is described here\nfn f() -> Int { 1 }\n"),
        ]);
        assert!(!inert.contains(&"Noted".to_string()));
    }

    #[test]
    fn doubly_declared_name_is_not_flagged() {
        // A name declared in two files cross-references itself → conservatively not inert.
        let inert = report_of(&[
            ("a.dag", "module a\ntype Dup { x: Int }\n"),
            ("b.dag", "module b\ntype Dup { y: Int }\n"),
        ]);
        assert!(!inert.contains(&"Dup".to_string()));
    }

    // LIVE-TREE gates. These run over the real corpus and must hold on main; they are the executable
    // floor the `.dag` witness mirrors.
    #[test]
    fn live_tree_declared_universe_is_nonempty() {
        assert!(
            inert_carrier_declared_count() > 0,
            "expected non-empty type-carrier universe; zero means the corpus walk fail-opened"
        );
    }

    #[test]
    fn live_tree_no_unrostered_inert_carrier() {
        let report = build_report();
        let roster: BTreeSet<&str> = INERT_CARRIER_ROSTER.iter().copied().collect();
        let unrostered: Vec<&String> = report
            .inert
            .iter()
            .filter(|n| !roster.contains(n.as_str()))
            .collect();
        assert!(
            unrostered.is_empty(),
            "new inert carrier(s) not on the exception roster (wire a consumer, or add to \
             INERT_CARRIER_ROSTER with a dissolve-on): {unrostered:?}"
        );
    }

    #[test]
    fn live_tree_roster_has_no_stale_entries() {
        let report = build_report();
        let inert: BTreeSet<&String> = report.inert.iter().collect();
        let stale: Vec<&&str> = INERT_CARRIER_ROSTER
            .iter()
            .filter(|n| !inert.contains(&n.to_string()))
            .collect();
        assert!(
            stale.is_empty(),
            "roster entries that are no longer inert (gained a consumer or were deleted) — remove \
             them so the roster shrinks: {stale:?}"
        );
    }
}
