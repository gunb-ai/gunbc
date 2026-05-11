//! **Layer:** integration
//!
//! R3 T-V2-Retirement — gate **`v2_oracle_no_remaining_test_consumers`**
//! ([`docs/r3-structure.md`](../../../../../../docs/r3-structure.md) §T-V2-Retirement;
//! program plan #41).
//!
//! Mechanical receipt: after comment-aware stripping, no Rust source under `src/` outside
//! `src/v2/` references the legacy `v2-compiler` / `v2-compiler-tests` crates, and no workspace
//! `Cargo.toml` under `src/` (plus the repo root manifest) declares a path dependency on
//! `v2-compiler`. Matches the G-1 guard in
//! [`docs/audit/t-v2-g2-deletion-plan-and-guardrails.md`](../../../../../../docs/audit/t-v2-g2-deletion-plan-and-guardrails.md).

use crate::common::strip_rust_comments;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

/// `path` is under `src_root` (the workspace `src/` directory). True when the relative path's
/// first component is `v2`.
fn is_under_src_v2(path: &Path, src_root: &Path) -> bool {
    if let Ok(rel) = path.strip_prefix(src_root) {
        if let Some(std::path::Component::Normal(first)) = rel.components().next() {
            return first == "v2";
        }
    }
    false
}

fn collect_rs_files(dir: &Path, src_root: &Path, out: &mut Vec<PathBuf>) {
    if is_under_src_v2(dir, src_root) {
        return;
    }
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for ent in entries {
        let ent = ent.unwrap_or_else(|e| panic!("read_dir entry {}: {e}", dir.display()));
        let p = ent.path();
        if p.is_dir() {
            collect_rs_files(&p, src_root, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn collect_cargo_toml_files(dir: &Path, src_root: &Path, out: &mut Vec<PathBuf>) {
    if is_under_src_v2(dir, src_root) {
        return;
    }
    let cargo = dir.join("Cargo.toml");
    if cargo.is_file() {
        out.push(cargo);
    }
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for ent in entries {
        let ent = ent.unwrap_or_else(|e| panic!("read_dir entry {}: {e}", dir.display()));
        let p = ent.path();
        if p.is_dir() {
            collect_cargo_toml_files(&p, src_root, out);
        }
    }
}

fn first_v2_compiler_crate_reference(stripped: &str) -> Option<&'static str> {
    /// Built with `concat!` so this ratchet file does not embed a contiguous
    /// `v2_` + `compiler` + `::` source span that would trip its own scan.
    const NEEDLES: &[&str] = &[
        concat!("v2_", "compiler", "::"),
        concat!("extern ", "crate ", "v2_", "compiler"),
        concat!("v2_", "compiler", "_", "tests", "::"),
        concat!("extern ", "crate ", "v2_", "compiler", "_", "tests"),
    ];
    for needle in NEEDLES {
        if stripped.contains(needle) {
            return Some(needle);
        }
    }
    for needle in [
        concat!("use ", "v2_", "compiler", ";"),
        concat!("use ", "v2_", "compiler", "::"),
        concat!("use ", "v2_", "compiler", "::", "{"),
        concat!("use ", "v2_", "compiler", " as "),
        concat!("use ", "v2_", "compiler", "_", "tests", ";"),
        concat!("use ", "v2_", "compiler", "_", "tests", "::"),
        concat!("use ", "v2_", "compiler", "_", "tests", "::", "{"),
        concat!("use ", "v2_", "compiler", "_", "tests", " as "),
    ] {
        if stripped.contains(needle) {
            return Some(needle);
        }
    }
    None
}

/// True when a non-table, non-comment line declares a path dependency on `v2-compiler`.
fn cargo_toml_declares_v2_compiler_path_dep(manifest: &str) -> bool {
    let mut in_dep_table = false;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let header_inner = line
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim();
            in_dep_table = header_inner == "dependencies"
                || header_inner.ends_with(".dependencies")
                || header_inner == "dev-dependencies"
                || header_inner.ends_with(".dev-dependencies")
                || header_inner == "build-dependencies"
                || header_inner.ends_with(".build-dependencies");
            continue;
        }
        if !in_dep_table {
            continue;
        }
        let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.starts_with("v2-compiler=") && compact.contains("path=") {
            return true;
        }
    }
    false
}

#[test]
fn v2_oracle_no_remaining_test_consumers_rust_sources() {
    let workspace_root = workspace_root();
    let src_root = workspace_root.join("src");
    assert!(
        src_root.is_dir(),
        "expected src/ at {}",
        src_root.display()
    );

    let mut rs_files = Vec::new();
    collect_rs_files(&src_root, &src_root, &mut rs_files);
    rs_files.sort();

    for path in rs_files {
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let stripped = strip_rust_comments(&raw);
        if let Some(needle) = first_v2_compiler_crate_reference(&stripped) {
            let rel = path.strip_prefix(&workspace_root).unwrap_or(&path);
            panic!(
                "v2_oracle_no_remaining_test_consumers: `{}` references the legacy v2 compiler \
                 crate ({needle:?} in live syntax after comment strip). Remove the dependency \
                 or relocate under `src/v2/`.",
                rel.display(),
            );
        }
    }
}

#[test]
fn v2_oracle_no_remaining_test_consumers_workspace_manifests() {
    let workspace_root = workspace_root();
    let src_root = workspace_root.join("src");

    let mut manifests = Vec::with_capacity(16);
    let root_toml = workspace_root.join("Cargo.toml");
    assert!(
        root_toml.is_file(),
        "expected workspace Cargo.toml at {}",
        root_toml.display()
    );
    manifests.push(root_toml);

    if src_root.is_dir() {
        collect_cargo_toml_files(&src_root, &src_root, &mut manifests);
    }
    manifests.sort();

    for path in manifests {
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            !cargo_toml_declares_v2_compiler_path_dep(&raw),
            "v2_oracle_no_remaining_test_consumers: `{}` must not declare a path dependency on \
             crate `v2-compiler` outside the legacy v2 subtree.",
            path.strip_prefix(&workspace_root)
                .unwrap_or(&path)
                .display(),
        );
    }
}
