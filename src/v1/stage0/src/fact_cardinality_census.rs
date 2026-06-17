//! Cross-tree named-declaration census for `v2.lens.fact_cardinality`.
//!
//! **SCAFFOLD (DESIGN.md §7)** — bootstrap text-scan census only. Shrink target: delete
//! this module when Node-tree decl projection + substrate `content_hash` own the census
//! in `v2.lens.fact_cardinality` (dissolve-on named in that lens header).
//!
//! Host-reads `dsl/` and `src/v2/` without cross-resolve: each `.dag` file is scanned
//! for column-0 `type` / `data` / `fn` items, normalized body text is hashed, and
//! `{rel_path_within_tree}:{decl_name}` keys present in BOTH trees with >1 distinct hash
//! are forks (e.g. `std/algebra.dag:FreeMonoid`).
//!
//! 🟡 bridge-boundary — `tree: String` here is host transport only (`"dsl"` / `"v2"`);
//! modeled facts use `FactCardinalityTree = Dsl | V2` in the lens.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Must match `fact_cardinality_cross_tree_fork_baseline` in `src/v2/lens/fact_cardinality.dag`.
pub const FACT_CARDINALITY_CROSS_TREE_FORK_BASELINE: i64 = 34;

const ITEM_KEYWORDS: [&str; 8] = [
    "data ",
    "fn ",
    "func ",
    "type ",
    "service ",
    "const ",
    "pattern ",
    "resource ",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactCardinalityDeclRecord {
    pub rel_path_decl_key: String,
    pub tree: String,
    pub content_hash: String,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn normalize_decl_body(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(line).trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decl_body_hash(body: &str) -> String {
    crate::v1_rt::atom_identity_hash(normalize_decl_body(body))
}

fn extract_top_level_decls(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("test ") {
            i += 1;
            continue;
        }
        let Some(kw) = ITEM_KEYWORDS.iter().find(|kw| line.starts_with(*kw)) else {
            i += 1;
            continue;
        };
        let rest = &line[kw.len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut body = String::new();
        body.push_str(line);
        body.push('\n');
        i += 1;
        let mut depth = line.chars().filter(|c| *c == '{').count() as i32
            - line.chars().filter(|c| *c == '}').count() as i32;
        while i < lines.len() {
            let next = lines[i];
            if depth <= 0
                && ITEM_KEYWORDS.iter().any(|kw| next.starts_with(kw))
                && !next.starts_with("test ")
            {
                break;
            }
            body.push_str(next);
            body.push('\n');
            depth += next.chars().filter(|c| *c == '{').count() as i32;
            depth -= next.chars().filter(|c| *c == '}').count() as i32;
            i += 1;
            if depth <= 0 && !body.contains('{') {
                break;
            }
        }
        out.push((name, decl_body_hash(&body)));
    }
    out
}

fn read_dag_source(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "fact_cardinality_census: failed to read {}: {e}",
            path.display()
        )
    })
}

fn rel_path_within_tree(top_root: &Path, path: &Path) -> String {
    path.strip_prefix(top_root)
        .unwrap_or_else(|_| {
            panic!(
                "fact_cardinality_census: path {} is not under tree root {}",
                path.display(),
                top_root.display()
            )
        })
        .to_string_lossy()
        .replace('\\', "/")
}

fn walk_tree_dir(
    top_root: &Path,
    dir: &Path,
    tree: &str,
    records: &mut Vec<FactCardinalityDeclRecord>,
) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "fact_cardinality_census: failed to read dir {}: {e}",
            dir.display()
        )
    });
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_tree_dir(top_root, &path, tree, records);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("dag") {
            continue;
        }
        let rel = rel_path_within_tree(top_root, &path);
        let content = read_dag_source(&path);
        for (name, hash) in extract_top_level_decls(&content) {
            records.push(FactCardinalityDeclRecord {
                rel_path_decl_key: format!("{rel}:{name}"),
                tree: tree.to_string(),
                content_hash: hash,
            });
        }
    }
}

fn walk_tree(top_root: &Path, tree: &str, records: &mut Vec<FactCardinalityDeclRecord>) {
    if !top_root.is_dir() {
        panic!(
            "fact_cardinality_census: tree root {} does not exist",
            top_root.display()
        );
    }
    walk_tree_dir(top_root, top_root, tree, records);
}

pub fn cross_tree_decl_records() -> Vec<FactCardinalityDeclRecord> {
    let ws = workspace_root();
    let mut records = Vec::new();
    walk_tree(&ws.join("dsl"), "dsl", &mut records);
    walk_tree(&ws.join("src/v2"), "v2", &mut records);
    records
}

pub fn cross_tree_fork_keys(records: &[FactCardinalityDeclRecord]) -> Vec<String> {
    let mut by_key: HashMap<String, HashMap<String, HashSet<String>>> = HashMap::new();
    for record in records {
        by_key
            .entry(record.rel_path_decl_key.clone())
            .or_default()
            .entry(record.content_hash.clone())
            .or_default()
            .insert(record.tree.clone());
    }
    let mut forks = Vec::new();
    for (key, hash_map) in by_key {
        if hash_map.len() <= 1 {
            continue;
        }
        let trees: HashSet<String> = hash_map
            .values()
            .flat_map(|trees| trees.iter().cloned())
            .collect();
        if trees.contains("dsl") && trees.contains("v2") {
            forks.push(key);
        }
    }
    forks.sort();
    forks
}

pub fn cross_tree_fork_count() -> i64 {
    cross_tree_fork_keys(&cross_tree_decl_records()).len() as i64
}

pub fn cross_tree_is_fork(rel_path_decl_key: String) -> bool {
    let forks: HashSet<String> = cross_tree_fork_keys(&cross_tree_decl_records())
        .into_iter()
        .collect();
    forks.contains(&rel_path_decl_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_monoid_is_cross_tree_fork() {
        assert!(cross_tree_is_fork("std/algebra.dag:FreeMonoid".to_string()));
    }

    #[test]
    fn lattice_is_not_cross_tree_fork() {
        assert!(!cross_tree_is_fork("std/algebra.dag:Lattice".to_string()));
    }

    #[test]
    fn cross_tree_fork_count_matches_baseline() {
        let count = cross_tree_fork_count();
        assert_eq!(
            count, FACT_CARDINALITY_CROSS_TREE_FORK_BASELINE,
            "fork count drifted from lens baseline {}; retune both if intentional",
            FACT_CARDINALITY_CROSS_TREE_FORK_BASELINE
        );
    }
}
