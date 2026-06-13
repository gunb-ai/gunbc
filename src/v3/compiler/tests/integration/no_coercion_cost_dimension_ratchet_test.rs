//! **Layer:** integration
//!
//! R3 §1.8 gate **#39** `no_coercion_cost_dimension` (T-CostLens-Composition):
//! the substrate must not introduce a parallel **coercion-cost** carrier
//! alongside `SymbolicCost` (see `src/v3/lenses/cost.dag` disposition notes).
//!
//! Mechanical receipt: walk `src/v3/{std,lenses,spec}/**/*.dag` and assert no
//! `CoercionCost` identifier appears outside line comments (with `://` skipped so
//! hypothetical URL literals do not false-positive). Matches only at identifier
//! boundaries so names like `NoCoercionCost` do not false-positive.

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // Same contract as `self_gen0_census_test::workspace_root`: `CARGO_MANIFEST_DIR` is
    // `src/v3/compiler/`; the repo root is the third ancestor.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("workspace root is three ancestors above src/v3/compiler/")
        .to_path_buf()
}

/// Strip a trailing `// …` line comment when it is not part of a `scheme://`
/// spelling (so `http://` does not truncate the line at `//`).
fn dag_line_without_trailing_line_comment(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.is_empty() {
        return "";
    }
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' {
            if i >= 1 && bytes[i - 1] == b':' {
                i += 2;
                continue;
            }
            return line[..i].trim_end();
        }
        i += 1;
    }
    line.trim_end()
}

fn is_dag_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// True when `needle` occurs in `s` as its own identifier / word (not as a substring of a longer
/// identifier such as `NoCoercionCost`).
fn contains_standalone_needle(s: &str, needle: &str) -> bool {
    let bytes = s.as_bytes();
    for (idx, _) in s.match_indices(needle) {
        let before_ok = idx == 0 || !is_dag_ident_continue(bytes[idx - 1]);
        let after_idx = idx + needle.len();
        let after_ok = after_idx >= bytes.len() || !is_dag_ident_continue(bytes[after_idx]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
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

#[test]
fn no_coercion_cost_dimension_substrate_dag_has_no_coercion_cost_carrier_token() {
    let root = workspace_root();
    let mut paths = Vec::new();
    for rel in ["src/v3/std", "src/v3/lenses", "src/v3/spec"] {
        let dir = root.join(rel);
        assert!(
            dir.is_dir(),
            "expected substrate directory {} — gate #39 scope drift?",
            dir.display()
        );
        collect_dag_files(&dir, &mut paths);
    }
    paths.sort();
    paths.dedup();

    const NEEDLE: &str = "CoercionCost";
    let mut hits = Vec::new();

    for path in paths {
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for (line_no, line) in source.lines().enumerate() {
            let code = dag_line_without_trailing_line_comment(line);
            if contains_standalone_needle(code, NEEDLE) {
                hits.push(format!(
                    "{}:{}: {}",
                    path.strip_prefix(&root).unwrap_or(&path).display(),
                    line_no + 1,
                    line.trim_end()
                ));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "R3 gate #39 (`no_coercion_cost_dimension`): standalone `CoercionCost` identifier must \
         not appear in v3 substrate `.dag` sources outside comments — parallel coercion-cost \
         dimension is forbidden. Offenders:\n{}",
        hits.join("\n")
    );
}

#[cfg(test)]
mod standalone_needle_tests {
    use super::contains_standalone_needle;

    #[test]
    fn coercion_cost_needle_rejects_longer_identifier_containing_suffix() {
        assert!(!contains_standalone_needle(
            "type NoCoercionCost = Int",
            "CoercionCost"
        ));
    }

    #[test]
    fn coercion_cost_needle_finds_exact_type_name() {
        assert!(contains_standalone_needle(
            "type CoercionCost = Int",
            "CoercionCost"
        ));
    }
}
