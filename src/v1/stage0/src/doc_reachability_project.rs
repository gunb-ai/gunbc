//! Live-tree projection for the **doc-graph reachability-completeness wall**
//! (docs/plans/inert-layer-lens.md §8).
//!
//! One rule, N substrates (DESIGN.md §2-horizontal): *every declared node in a graph must be
//! reachable from a root, on an exception roster, or deleted.* It already runs over the **lens**
//! graph (#5433, `cli_run::inert_lens_modules`). This module applies the **same BFS shape** to the
//! **doc** graph — the cheapest instance (pure link reachability; no host bridges beyond reading the
//! files). It does NOT fork the #5433 closure: the reachability primitive here (`reachable_set`) is
//! the same seed→transitive-closure fixpoint, re-expressed over doc nodes/edges, and `cli_run.rs`
//! (the load-bearing #5433 file) is left untouched.
//!
//! - **nodes** — every `docs/**/*.md`.
//! - **edges** — markdown `](path)` links (resolved relative to the linking file's dir or repo root)
//!   plus reflective `.dag`-comment `bind: …/x.md` refs (DESIGN open thread; §8 condition 2: a doc
//!   consumed only through a `bind:` ref is *reached*, invisible to a pure markdown walk).
//! - **roots** — per doc *kind* (§8 condition 1): plan docs root at `ROADMAP.md` + `DESIGN.md`;
//!   runbooks root at their own index `docs/runbooks/README.md` (else a runbook false-positives as a
//!   plan orphan).
//! - **inert** = a doc not in the reachable set → an orphan (the wall fails closed on any).
//! - **dangling** = a `](path)` link whose `.md` target does not exist on disk.
//!
//! Host-fed builtins expose the two scalar verdicts to a `.dag` `test fn` witness
//! (`doc_graph_orphan_count` / `doc_graph_dangling_link_count`), exactly as the
//! extdeps-external-authority live-corpus gate exposes `external_authority_live_clean_tree_holds`.
//!
//! DISSOLUTION TRIGGER (§7, plan Tier-2): the *orphan* half needs filesystem **enumeration** (the
//! universe minus the reachable set), for which no list-dir host effect exists today — so this Rust
//! census is the host realization of the one rule. When `.dag` gains list-dir / compile-graph access
//! (gunbc#5364, the plan Tier-2 note), the dir-walk + BFS fold into a pure `.dag` reader and this
//! module deletes. The *dangling* half alone is already expressible in pure `.dag` via
//! `filesystem_read` BFS from roots (the §8 no-host-bridge claim holds only for that half).

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};

/// Plan-doc roots: a plan doc is reachable iff `ROADMAP.md`/`DESIGN.md` reach it (directly or
/// transitively over doc links). These are the roadmap/design authorities.
const PLAN_DOC_ROOTS: &[&str] = &["ROADMAP.md", "DESIGN.md"];

/// Runbook-kind root (§8 condition 1): runbooks are operational, not roadmap items, so they have
/// their own index root — without it every runbook false-positives as an orphan.
const RUNBOOK_ROOT: &str = "docs/runbooks/README.md";

fn workspace_root() -> PathBuf {
    crate::module_path_index::workspace_root()
}

/// Repo-relative, forward-slash path string for a file under the workspace.
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

/// Every `docs/**/*.md` under the workspace, repo-relative, sorted — the node universe.
fn doc_universe() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let docs_dir = workspace_root().join("docs");
    collect_md_files(&docs_dir, &mut out);
    out
}

fn collect_md_files(dir: &Path, out: &mut BTreeSet<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.insert(repo_rel(&path));
        }
    }
}

/// Markdown `](target)` link targets in `content`, each split at any `#anchor` and trimmed.
/// External (`http`, `mailto`) targets are dropped — only repo-local file refs form doc edges.
fn markdown_link_targets(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' {
            if let Some(end) = content[i + 2..].find(')') {
                let raw = &content[i + 2..i + 2 + end];
                let target = raw.split('#').next().unwrap_or("").trim();
                if !target.is_empty()
                    && !target.starts_with("http://")
                    && !target.starts_with("https://")
                    && !target.starts_with("mailto:")
                {
                    out.push(target.to_string());
                }
                i = i + 2 + end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Resolve a markdown link `target` found in the file at repo-relative `from` to a repo-relative
/// path, trying file-dir-relative first then repo-root-relative. Returns the normalized candidate
/// (existence is the caller's concern).
fn resolve_link(from: &str, target: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let from_dir = Path::new(from).parent().unwrap_or_else(|| Path::new(""));
    candidates.push(normalize(&from_dir.join(target)));
    candidates.push(normalize(Path::new(target)));
    candidates.dedup();
    candidates
}

/// Lexical path normalization (collapse `.`/`..`) without touching the filesystem.
fn normalize(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for comp in path.to_string_lossy().replace('\\', "/").split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other.to_string()),
        }
    }
    parts.join("/")
}

/// Reflective edges (§8 condition 2): `bind: …/x.md` refs in `.dag` comments across `dsl/**` and
/// `src/**`. A doc named by such a ref counts as reached. Returns the set of repo-relative `.md`
/// targets so referenced.
fn dag_comment_bind_doc_refs() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for root in ["dsl", "src"] {
        let mut dag_files = Vec::new();
        collect_dag_files(&workspace_root().join(root), &mut dag_files);
        for path in dag_files {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for target in bind_md_refs(&content) {
                out.insert(target);
            }
        }
    }
    out
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

/// `.md` targets named by a `bind:` ref in the text — `bind:` then the following token ending `.md`.
fn bind_md_refs(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (idx, _) in content.match_indices("bind:") {
        let rest = content[idx + "bind:".len()..].trim_start();
        let token: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ')' && *c != '"' && *c != '`')
            .collect();
        if token.ends_with(".md") {
            out.push(normalize(Path::new(&token)));
        }
    }
    out
}

/// Generic seed→transitive-closure reachability over the doc edge graph — the **same BFS shape** as
/// `cli_run::inert_lens_modules` (DESIGN.md §3 single authority; the doc instance of the one rule,
/// not a forked concept). `roots` seeds the frontier; `edges` maps each node to its out-neighbors.
fn reachable_set(
    roots: &BTreeSet<String>,
    edges: &HashMap<String, Vec<String>>,
) -> BTreeSet<String> {
    let mut reached: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    for r in roots {
        if reached.insert(r.clone()) {
            queue.push_back(r.clone());
        }
    }
    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = edges.get(&node) {
            for n in neighbors {
                if reached.insert(n.clone()) {
                    queue.push_back(n.clone());
                }
            }
        }
    }
    reached
}

/// The doc-graph reachability report over the live tree: orphan docs + dangling `.md` links.
struct DocGraphReport {
    orphans: Vec<String>,
    dangling: Vec<(String, String)>,
}

fn build_doc_graph_report() -> DocGraphReport {
    let universe = doc_universe();
    let bind_refs = dag_comment_bind_doc_refs();

    // Roots: plan-doc authorities + the runbook index (each only if it exists) + any doc named by a
    // reflective `bind:` ref (a doc consumed only that way is reached — §8 condition 2).
    let mut roots: BTreeSet<String> = BTreeSet::new();
    for r in PLAN_DOC_ROOTS {
        roots.insert((*r).to_string());
    }
    if workspace_root().join(RUNBOOK_ROOT).is_file() {
        roots.insert(RUNBOOK_ROOT.to_string());
    }
    for b in &bind_refs {
        if universe.contains(b) {
            roots.insert(b.clone());
        }
    }

    // Edges: markdown links from every doc node *and* from the plan-doc roots (which live at repo
    // root, outside `docs/`). Dangling `.md` targets are collected in the same pass.
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    let mut dangling: Vec<(String, String)> = Vec::new();
    let mut sources: Vec<String> = universe.iter().cloned().collect();
    for r in PLAN_DOC_ROOTS {
        sources.push((*r).to_string());
    }
    if roots.contains(RUNBOOK_ROOT) {
        sources.push(RUNBOOK_ROOT.to_string());
    }
    sources.sort();
    sources.dedup();
    for src in &sources {
        let content = match std::fs::read_to_string(workspace_root().join(src)) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut out_edges: Vec<String> = Vec::new();
        for target in markdown_link_targets(&content) {
            let candidates = resolve_link(src, &target);
            let existing = candidates
                .iter()
                .find(|c| workspace_root().join(c).is_file())
                .cloned();
            match existing {
                Some(path) => out_edges.push(path),
                None => {
                    // Only `.md` links participate in the doc graph; a broken one is dangling.
                    if target.ends_with(".md") {
                        dangling.push((src.clone(), target.clone()));
                    }
                }
            }
        }
        edges.insert(src.clone(), out_edges);
    }

    let reached = reachable_set(&roots, &edges);
    let orphans: Vec<String> = universe
        .iter()
        .filter(|d| !reached.contains(*d))
        .cloned()
        .collect();
    dangling.sort();
    dangling.dedup();
    DocGraphReport { orphans, dangling }
}

/// Number of orphan docs on the live tree — a `docs/**/*.md` reachable from no root over doc links
/// or reflective `bind:` refs. The wall is GREEN iff this is `0`.
pub fn doc_graph_orphan_count() -> i64 {
    build_doc_graph_report().orphans.len() as i64
}

/// Number of dangling `.md` markdown links on the live tree — a `](x.md)` whose target is missing.
pub fn doc_graph_dangling_link_count() -> i64 {
    build_doc_graph_report().dangling.len() as i64
}

/// Number of `docs/**/*.md` discovered on the live tree (the node universe size). The §5
/// empty-result-must-RED backstop: `doc_universe()` fail-OPENS on a `read_dir` error (an empty
/// universe yields 0 orphans = a false green), so the witness asserts this is `> 0` — zero
/// discovered docs is itself the bug (`docs/` vanished or unreadable), mirroring the extdeps gate's
/// `external_authority_live_roster_module_count() > 150` non-emptiness floor.
pub fn doc_graph_doc_count() -> i64 {
    doc_universe().len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges_of(pairs: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, vs)| {
                (
                    (*k).to_string(),
                    vs.iter().map(|s| (*s).to_string()).collect(),
                )
            })
            .collect()
    }

    // RED control: an orphan (no inbound edge from any root) is flagged by the reachability fixpoint.
    #[test]
    fn reachable_set_flags_orphan_node() {
        let roots: BTreeSet<String> = ["root.md".to_string()].into_iter().collect();
        let edges = edges_of(&[("root.md", &["linked.md"]), ("orphan.md", &[])]);
        let reached = reachable_set(&roots, &edges);
        assert!(reached.contains("root.md"));
        assert!(reached.contains("linked.md"));
        assert!(
            !reached.contains("orphan.md"),
            "an unlinked node must be unreachable (the orphan witness)"
        );
    }

    // A self-referencing dead cluster is still inert (reference-count != reachability — §1).
    #[test]
    fn reachable_set_inert_cluster_stays_unreached() {
        let roots: BTreeSet<String> = ["root.md".to_string()].into_iter().collect();
        let edges = edges_of(&[
            ("root.md", &["a.md"]),
            ("a.md", &[]),
            ("dead1.md", &["dead2.md"]),
            ("dead2.md", &["dead1.md"]),
        ]);
        let reached = reachable_set(&roots, &edges);
        assert!(reached.contains("a.md"));
        assert!(!reached.contains("dead1.md") && !reached.contains("dead2.md"));
    }

    // GREEN control: transitive reach through a chain.
    #[test]
    fn reachable_set_transitive_chain() {
        let roots: BTreeSet<String> = ["r.md".to_string()].into_iter().collect();
        let edges = edges_of(&[
            ("r.md", &["a.md"]),
            ("a.md", &["b.md"]),
            ("b.md", &["c.md"]),
        ]);
        let reached = reachable_set(&roots, &edges);
        for n in ["r.md", "a.md", "b.md", "c.md"] {
            assert!(reached.contains(n), "{n} should be reached");
        }
    }

    #[test]
    fn markdown_link_targets_basic() {
        let c = "see [x](docs/plans/x.md) and [y](y.md#anchor) and [ext](https://e.com) and [z](./z.md)";
        let t = markdown_link_targets(c);
        assert_eq!(t, vec!["docs/plans/x.md", "y.md", "./z.md"]);
    }

    // RED control for the dangling half: a `.md` link to a missing target is dangling; a link to an
    // existing file (or a non-`.md` target) is not — so the count discriminates.
    #[test]
    fn dangling_detection_flags_missing_md_only() {
        // `markdown_link_targets` + a missing-target predicate is the dangling kernel; here we drive
        // the predicate directly with a synthetic doc (the live half is covered by the live tests).
        let doc = "[ok](https://x) [broken](docs/plans/does-not-exist-xyz.md) [code](src/lib.rs)";
        let targets = markdown_link_targets(doc);
        let dangling: Vec<&String> = targets
            .iter()
            .filter(|t| {
                t.ends_with(".md") && !workspace_root().join(normalize(Path::new(t))).is_file()
            })
            .collect();
        assert_eq!(
            dangling.len(),
            1,
            "exactly the missing .md link is dangling (not the http or the existing code link): {dangling:?}"
        );
    }

    #[test]
    fn bind_md_refs_basic() {
        let c = "// bind: docs/planning/foo.md (provenance)\n// no bind here\n// bind: bar.md";
        let t = bind_md_refs(c);
        assert_eq!(t, vec!["docs/planning/foo.md", "bar.md"]);
    }

    // Live-tree witnesses: the wall must be clean on `main` (the cleaning links land in this PR).
    #[test]
    fn live_tree_has_no_orphan_docs() {
        let report = build_doc_graph_report();
        assert!(
            report.orphans.is_empty(),
            "orphan docs (unreachable from any root): {:?}",
            report.orphans
        );
    }

    // §5 non-emptiness floor: a real repo always has docs, so a zero universe means `read_dir`
    // failed (fail-open) — the witness must catch it.
    #[test]
    fn live_tree_doc_universe_is_nonempty() {
        assert!(
            doc_graph_doc_count() > 0,
            "expected a non-empty docs/ universe; zero means read_dir fail-open"
        );
    }

    #[test]
    fn live_tree_has_no_dangling_md_links() {
        let report = build_doc_graph_report();
        assert!(
            report.dangling.is_empty(),
            "dangling .md links: {:?}",
            report.dangling
        );
    }
}
