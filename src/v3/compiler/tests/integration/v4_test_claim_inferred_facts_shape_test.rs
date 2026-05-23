//! **Layer:** integration
//!
//! R2-7 fixture drift guard: every `InferredFacts {` constructor in
//! `src/v4/test/claim/**/*.dag` must carry the PR #3587 / AI-5 shape from
//! `src/v4/compiler/04_infer.dag` (`grounding: CanonicalGrounding`, `descent`).
//! Prevents silent semantic gaps after substrate shape sweeps (e.g. f98e8315).
//!
//! **ROADMAP:** T-PB-B / `pb_rust_tests_outside_residual_zero`; dissolves when
//! `.dag` TestClaim execution validates fixture shapes structurally.

use std::fs;
use std::path::{Path, PathBuf};

const REQUIRED_FIELDS: [&str; 2] = ["grounding:", "descent:"];
const FORBIDDEN_FIELDS: [&str; 3] = ["resolved_type:", "inhabits:", "canonical:"];
const CONSTRUCTOR_NEEDLE: &str = "InferredFacts {";

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("workspace root is three ancestors above src/v3/compiler/")
        .to_path_buf()
}

fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for ent in entries {
        let ent = ent.unwrap_or_else(|e| panic!("read_dir entry {}: {e}", dir.display()));
        let p = ent.path();
        if p.is_dir() {
            collect_dag_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "dag") {
            out.push(p);
        }
    }
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Extract the brace-balanced block starting at `start` (index of `{` after `InferredFacts`).
fn inferred_facts_block(source: &str, start: usize) -> Option<&str> {
    let open = source[start..].find('{')? + start;
    let mut depth = 0usize;
    for (i, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&source[open..open + i + 1]);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
fn v4_test_claim_inferred_facts_constructors_match_grounding_descent_shape() {
    let root = workspace_root();
    let claim_root = root.join("src/v4/test/claim");
    assert!(
        claim_root.is_dir(),
        "expected test claim tree {} — scope drift?",
        claim_root.display()
    );

    let mut paths = Vec::new();
    collect_dag_files(&claim_root, &mut paths);
    paths.sort();

    let mut violations = Vec::new();
    for path in paths {
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let mut search_from = 0usize;
        while let Some(rel) = source[search_from..].find(CONSTRUCTOR_NEEDLE) {
            let abs = search_from + rel;
            let block_start = abs + CONSTRUCTOR_NEEDLE.len() - 1;
            let Some(block) = inferred_facts_block(&source, block_start) else {
                violations.push(format!(
                    "{}: unbalanced `{CONSTRUCTOR_NEEDLE}` block",
                    rel_path(&root, &path)
                ));
                break;
            };
            for field in REQUIRED_FIELDS {
                if !block.contains(field) {
                    violations.push(format!(
                        "{}: `{CONSTRUCTOR_NEEDLE}` missing `{field}` (authority: 04_infer.dag AI-5 shape)",
                        rel_path(&root, &path)
                    ));
                }
            }
            for field in FORBIDDEN_FIELDS {
                if block.contains(field) {
                    violations.push(format!(
                        "{}: `{CONSTRUCTOR_NEEDLE}` still uses retired field `{field}` (migrate to `grounding:` + `descent:`)",
                        rel_path(&root, &path)
                    ));
                }
            }
            search_from = abs + CONSTRUCTOR_NEEDLE.len();
        }
    }

    assert!(
        violations.is_empty(),
        "R2-7 InferredFacts fixture drift: {} violation(s):\n{}",
        violations.len(),
        violations.join("\n")
    );
}
