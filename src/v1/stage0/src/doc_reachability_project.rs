use std::collections::{BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};

const PLAN_DOC_ROOTS: &[&str] = &["ROADMAP.md", "DESIGN.md"];

const RUNBOOK_ROOT: &str = "docs/runbooks/README.md";

fn workspace_root() -> PathBuf {
    crate::cli_run::workspace_root()
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

fn resolve_link(from: &str, target: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let from_dir = Path::new(from).parent().unwrap_or_else(|| Path::new(""));
    candidates.push(normalize(&from_dir.join(target)));
    candidates.push(normalize(Path::new(target)));
    candidates.dedup();
    candidates
}

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

fn dag_comment_bind_doc_refs() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for root in crate::cli_run::witness_layer_roots() {
        let mut dag_files = Vec::new();
        collect_dag_files(&workspace_root().join(&root), &mut dag_files);
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

struct DocGraphReport {
    orphans: Vec<String>,
    dangling: Vec<(String, String)>,
}

fn build_doc_graph_report() -> DocGraphReport {
    let universe = doc_universe();
    let bind_refs = dag_comment_bind_doc_refs();

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

pub fn doc_graph_orphan_count() -> i64 {
    build_doc_graph_report().orphans.len() as i64
}

pub fn doc_graph_dangling_link_count() -> i64 {
    build_doc_graph_report().dangling.len() as i64
}

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

    #[test]
    fn dangling_detection_flags_missing_md_only() {
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
}
