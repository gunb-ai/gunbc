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
// shape as #5433 and the doc-graph reachability wall (`cli_run.rs` host census +
// `v2.lens.doc_reachability` verdict).
//
// DEFINITION — the COVERAGE-BY-ILLUSION class exactly (bright-stag's deliverable 1 wording: "defined
// + self-tested + ZERO real consumer"):
//   A carrier is inert iff (a) it is DECLARED in a non-test file, (b) it is SELF-TESTED — its NAME
//   (as a whole identifier token) appears in at least one `*_test.dag` file, and (c) it has ZERO
//   real consumer — its name appears in NO non-test `.dag` file other than its own declaring file.
//
//   The (b) self-tested gate is what makes this a clean keystone rather than "every unused type":
//   the corpus is full of carriers modeled ahead of their consumer (extdeps API surfaces, the
//   realization loop) — those are the project's deliberate model-first discipline and are NOT
//   coverage-by-illusion. A carrier someone bothered to write a TEST for but nothing consumes is the
//   precise §5 trap: it LOOKS covered (a green test) yet drives no production behavior. Comments are
//   stripped before tokenizing, so a real consumer means a code reference, never prose (a code use is
//   never inside a comment → the self-tested+unconsumed verdict cannot false-flag a genuinely-used
//   carrier).
//
// This is host-fed today (no whole-corpus reference enumeration exists in `.dag` — see
// docs/plans/inert-layer-lens.md §3 Tier 2). DISSOLUTION TRIGGER: when `.dag` gains compile-graph /
// reference-edge access (gunbc#5364, the `concept_index` trigger), the token scan folds into a pure
// `.dag` reader over `BindsTo` edges and this Rust census deletes. It does NOT touch cli_run.rs's
// #5433 closure — it is the additive corpus-gate builtin seam (sibling to doc_graph_orphan_count /
// fact_cardinality_*).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::cli_run::{brace_delta, corpus_dag_files, is_test_dag, strip_line_comment};

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
    // SEEDED FROM THE LIVE CENSUS 2026-06-22 (re-derive with the live_tree gate below). Each is a
    // self-tested type carrier whose name appears in exactly its single declaring file plus a
    // `*_test.dag` — DESIGN §5 coverage-by-illusion (a green test, no production consumer).
    // dissolve-on per entry: wire the real consumer, then DELETE the entry (the stale-roster ratchet
    // reds the floor until you do). A NEW self-tested-but-unconsumed carrier not listed here reds.
    "AccessPolicy", // dsl/std/access.dag — access-policy model; no authz consumer wired yet.
    "CargoDependency", // dsl/extdeps/rust/cargo.dag — Cargo manifest dep row; emit/ingest unwired.
    "CargoPackage", // dsl/extdeps/rust/cargo.dag — Cargo manifest package row; unwired.
    "FilePermissions", // dsl/extdeps/access/posix.dag — POSIX mode permission carrier; no fs consumer.
    "FloorWitnessRow", // src/v2/workflow/affected_set_floor_runner.dag — runner row; self-only.
    "FreeOutput", // dsl/extdeps/os/free.dag — host metrics shape; shadow lane, no workflow consumer yet.
    "GitCliReportedVersion", // dsl/extdeps/git/versioning.dag — git --version parse target; unwired.
    "MergeStateStatus", // dsl/extdeps/github/merge_state.dag — GitHub GraphQL mergeStateStatus; no parse fn (transport deferred), no workflow consumer yet.
    "ProcMeminfo", // dsl/extdeps/os/proc_meminfo.dag — /proc/meminfo parsed shape; shadow lane, no workflow consumer yet.
    "ReactHookSite", // src/v2/extdeps/frameworks/react.dag — React hook-site model; unwired.
    "SecretValue", // dsl/std/types.dag — secret-string carrier; no redaction consumer wired.
    "SystemdUnitStatus", // dsl/extdeps/os/systemd.dag — systemd unit status shape; shadow lane, no workflow consumer yet.
];

/// Extract top-level `type NAME` carrier declarations with their full declaration BLOCK text.
/// Carriers are the type abstractions (coproducts + records + aliases); `data`/`fn`/`service` are
/// values/behavior, out of scope for this keystone. The block runs from the `type` line until the
/// next top-level item (matching the corpus's brace-depth convention, as in fact_cardinality_census).
/// Returning the block lets the consumer count distinguish a carrier's OWN self-references (the
/// declaration, recursive arms) from a real USE elsewhere — including a use by another fn in the same
/// declaring file (a lens-local fact type IS consumed by its lens fn; only a use outside the type
/// block counts). The corpus walk + lexical normalization live in `crate::cli_run` (shared with
/// the non-fold-residue census — DESIGN §2/§3: one authority for "what is code text").
fn type_carrier_blocks(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let Some(rest) = trimmed.strip_prefix("type ") else {
            i += 1;
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut block = String::new();
        block.push_str(lines[i]);
        block.push('\n');
        let mut depth = brace_delta(lines[i]);
        i += 1;
        while i < lines.len() {
            let nt = lines[i].trim_start();
            if depth <= 0 {
                // At depth 0 the block continues only across a `=`/`|` sum continuation; anything
                // else (next item, blank line, prose) ends it.
                if !(nt.starts_with('|') || nt.starts_with('=')) {
                    break;
                }
            }
            block.push_str(lines[i]);
            block.push('\n');
            depth += brace_delta(lines[i]);
            i += 1;
        }
        out.push((name, block));
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

/// Count whole-identifier occurrences of `name` in `text` (comments stripped).
fn count_token(text: &str, name: &str) -> i64 {
    let mut n = 0i64;
    for raw in text.lines() {
        for tok in identifier_tokens(&strip_line_comment(raw)) {
            if tok == name {
                n += 1;
            }
        }
    }
    n
}

struct InertCarrierReport {
    /// All declared type carriers (name -> declaring file), declared in non-test files.
    declared: BTreeMap<String, String>,
    /// Inert carrier names (self-tested ∧ zero use outside their own declaration block), sorted.
    inert: Vec<String>,
}

/// The census core, over an explicit `(rel_path, content)` corpus — so the live `build_report` and
/// the synthetic discrimination controls run the SAME predicate (DESIGN §5: prove by execution, not
/// by a re-implementation that could drift from production).
fn compute_report(files: &[(String, String)]) -> InertCarrierReport {
    // 1. Declaration universe + each carrier's OWN self-reference count (occurrences inside its
    //    declaration block — the `type` line, recursive arms — which are not uses).
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    let mut decl_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut self_block_refs: BTreeMap<String, i64> = BTreeMap::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        for (name, block) in type_carrier_blocks(content) {
            declared.entry(name.clone()).or_insert_with(|| rel.clone());
            *decl_count.entry(name.clone()).or_insert(0) += 1;
            *self_block_refs.entry(name.clone()).or_insert(0) += count_token(&block, &name);
        }
    }

    // 2. Total occurrences across non-test files (real-consumer candidates), and self-tested set.
    let names: BTreeSet<String> = declared.keys().cloned().collect();
    let mut nontest_occ: BTreeMap<String, i64> = BTreeMap::new();
    let mut self_tested: BTreeSet<String> = BTreeSet::new();
    for (rel, content) in files {
        let mut local: BTreeMap<String, i64> = BTreeMap::new();
        for raw in content.lines() {
            for tok in identifier_tokens(&strip_line_comment(raw)) {
                if names.contains(&tok) {
                    *local.entry(tok).or_insert(0) += 1;
                }
            }
        }
        if is_test_dag(rel) {
            for (k, _) in local {
                self_tested.insert(k);
            }
        } else {
            for (k, v) in local {
                *nontest_occ.entry(k).or_insert(0) += v;
            }
        }
    }

    // 3. Inert = declared exactly once (a doubly-declared name cross-references itself → not flagged,
    //    the conservative direction) ∧ self-tested ∧ used by ZERO non-test code outside its own
    //    declaration block. `external_uses = total non-test occurrences − own-block self-references`.
    let mut inert: Vec<String> = Vec::new();
    for name in declared.keys() {
        if decl_count.get(name).copied().unwrap_or(0) != 1 {
            continue;
        }
        if !self_tested.contains(name) {
            continue; // not self-tested → merely-unused (model-first), not coverage-by-illusion.
        }
        let total = nontest_occ.get(name).copied().unwrap_or(0);
        let own = self_block_refs.get(name).copied().unwrap_or(0);
        if total - own <= 0 {
            inert.push(name.clone());
        }
    }
    inert.sort();
    inert.dedup();
    InertCarrierReport { declared, inert }
}

/// Memoized live-tree report. The on-disk corpus is fixed for a process's lifetime, so the four
/// `inert_carrier_*_count` builtins (called one-by-one per witness eval) and the live-tree tests share
/// a single `dsl/` + `src/v2/` walk + report instead of re-walking per call. The pure `compute_report`
/// is left taking `&[files]` so the synthetic RED/GREEN controls keep driving it with in-memory corpora
/// (which never touch this cache).
fn build_report() -> &'static InertCarrierReport {
    static REPORT: OnceLock<InertCarrierReport> = OnceLock::new();
    REPORT.get_or_init(|| compute_report(&corpus_dag_files()))
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
    fn type_carrier_blocks_extracts_names_and_bodies() {
        let c = "module m\ntype Connective = Atom | Conj\ntype WorkDemand {\n  field: Int\n}\nfn f() -> Int { 1 }\n";
        let blocks = type_carrier_blocks(c);
        let names: Vec<&String> = blocks.iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["Connective", "WorkDemand"]);
        // The WorkDemand block must include its `}` line but stop before `fn f`.
        let wd = &blocks.iter().find(|(n, _)| n == "WorkDemand").unwrap().1;
        assert!(wd.contains("field: Int") && wd.contains('}'));
        assert!(!wd.contains("fn f"));
    }

    #[test]
    fn identifier_tokens_are_whole_words() {
        // Placement must NOT match inside PlacementSupply (word-boundary discrimination).
        let toks = identifier_tokens("  field: PlacementSupply = foo(Placement)");
        assert!(toks.contains(&"PlacementSupply".to_string()));
        assert!(toks.contains(&"Placement".to_string()));
        assert!(toks.contains(&"field".to_string()));
    }

    // The DISCRIMINATING controls (DESIGN §5: green-by-execution, RED on a real defect). They run
    // the PRODUCTION predicate (`compute_report`) over a synthetic in-memory corpus, so the
    // discrimination is proven on the same code main ships.
    fn report_of(files: &[(&str, &str)]) -> Vec<String> {
        let owned: Vec<(String, String)> = files
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect();
        compute_report(&owned).inert
    }

    #[test]
    fn red_control_self_tested_zero_consumer_carrier_is_inert() {
        // A carrier defined + SELF-TESTED but mentioned in NO other non-test file → inert (RED: the
        // lens fires). This is coverage-by-illusion: a green test, no production consumer.
        let inert = report_of(&[
            ("a.dag", "module a\ntype Lonely { x: Int }\n"),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Lonely { x: 1 } == Lonely { x: 1 } }\n",
            ),
        ]);
        assert!(
            inert.contains(&"Lonely".to_string()),
            "a self-tested carrier with no real consumer must be flagged inert; got {inert:?}"
        );
    }

    #[test]
    fn green_control_carrier_with_real_consumer_is_not_inert() {
        // Same carrier, now mentioned in a second non-test file → NOT inert (GREEN: the lens stays
        // silent). The ONLY difference from the RED control is a real consumer — the discrimination.
        let inert = report_of(&[
            ("a.dag", "module a\ntype Used { x: Int }\n"),
            (
                "b.dag",
                "module b\nimport a { Used }\nfn f(u: Used) -> Int { u.x }\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Used { x: 1 } == Used { x: 1 } }\n",
            ),
        ]);
        assert!(
            !inert.contains(&"Used".to_string()),
            "a carrier with a real (non-test, cross-file) consumer must NOT be flagged; got {inert:?}"
        );
    }

    #[test]
    fn green_control_same_file_consumer_is_not_inert() {
        // A lens-local fact type: declared AND consumed by a fn in the SAME file (outside its type
        // block), plus self-tested. It is genuinely load-bearing (the lens reads it) → NOT inert.
        // This is the discrimination the use-outside-block rule buys over a declaring-file exclusion.
        let inert = report_of(&[
            (
                "lens.dag",
                "module lens\ntype LocalFact { x: Int }\nfn clean(fs: LocalFact) -> Bool { fs.x == 0 }\n",
            ),
            ("lens_test.dag", "module t\nfn t() -> Bool { clean(fs: LocalFact { x: 0 }) }\n"),
        ]);
        assert!(
            !inert.contains(&"LocalFact".to_string()),
            "a carrier consumed by a fn in its own file is NOT inert; got {inert:?}"
        );
    }

    #[test]
    fn green_control_untested_unused_carrier_is_not_flagged() {
        // A carrier with NEITHER a test NOR a consumer is merely-unused (model-first staging), NOT
        // coverage-by-illusion. The self-tested gate excludes it — this keeps the wall off the
        // project's deliberate ahead-of-consumer modeling (extdeps surfaces, the realization loop).
        let inert = report_of(&[("a.dag", "module a\ntype Staged { x: Int }\n")]);
        assert!(
            !inert.contains(&"Staged".to_string()),
            "an untested unused carrier must NOT be flagged (it is model-first, not illusion); got {inert:?}"
        );
    }

    #[test]
    fn comment_reference_is_not_a_real_consumer() {
        // A mention only in another file's COMMENT is not a code consumer (comments are stripped),
        // so a self-tested carrier referenced elsewhere only in prose is still inert. A code use can
        // never live in a comment, so this never false-flags a genuinely-used carrier.
        let inert = report_of(&[
            ("a.dag", "module a\ntype Noted { x: Int }\n"),
            (
                "b.dag",
                "module b\n// Noted is described here\nfn f() -> Int { 1 }\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Noted { x: 1 } == Noted { x: 1 } }\n",
            ),
        ]);
        assert!(inert.contains(&"Noted".to_string()));
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
}
