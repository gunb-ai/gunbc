//! Structural projection for `v2.lens.resolved_imports`.
//!
//! Enumerates `import <module>` facts from the `.dag` files under a set of importer
//! roots and tags each with whether its target module is DECLARED anywhere in the pool
//! roots — the structural form of the resolver's `UnresolvedImport` rule
//! (`find_module(module_index, import_path) == Absent`, src/v1/03_resolve.dag:170-173).
//!
//! Single authority (DESIGN §3): the declared-module pool REUSES the resolver's own
//! `build_module_path_index` (a resolver-owned primitive — the `module `-header path
//! scan, itself fail-closed on duplicate module paths — NOT the live resolve symbol index,
//! which keys on `authored_name_at`, `v1_compiler_resolve.rs:69-78`) rather than
//! re-implementing a `module ` header scan; the import-line enumeration REUSES
//! `extract_import_paths` and
//! the tolerant `.dag` walk `collect_dag_files_tolerant` — exactly the primitives
//! `layering_imports_project` shares. So this projection forks no enumeration logic; it
//! only carries the structural import-resolution VERDICT that the parse-gated whole-tree
//! compile (`front_end_sources` short-circuits to `graph: none` on any parse error)
//! never reaches.
//!
//! The AUTHORITATIVE producer of "this import is unresolvable" is the resolver:
//! `resolve_import` emits `UnresolvedImport` at `v1_compiler_resolve.rs:271` (.dag mirror
//! `src/v1/03_resolve.dag:170-173`) when `find_module(module_index, import_path) == Absent`.
//! This projection PROJECTS that same rule into a cheap CI gate; it exists only because the
//! parse-gate prevents tree-wide resolve from running (the v2-compile-no-entry-parse-only
//! finding), so it is a projection of the single authority, not a fork — whether or not the
//! root parse-resilience fix ever lands.
//!
//! Pure + deterministic: within each importer root the `.dag` files are path-sorted and
//! imports keep source order, and the roots are walked in caller order — so for a fixed
//! `importer_roots` a Wet run is byte-identical across invocations (the ordering is
//! per-root, not a single global path sort across roots). The importer-set policy (which paths are excluded,
//! e.g. the `/test/fixture/` text fixtures that intentionally carry broken imports) is
//! NOT decided here — the caller passes `exclude_substrings` (DESIGN §3 (c): the
//! scan-scope policy is workflow, not a fact baked into the projection).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::cli_run::{
    build_module_path_index, collect_dag_files_tolerant, extract_import_paths, workspace_root,
};

/// One projected import-resolution fact: the repo-relative `.dag` path, the imported
/// module path, and whether that module is declared anywhere in the pool roots.
pub struct ImportResolutionFactRaw {
    pub path: String,
    pub import_module: String,
    pub target_declared: bool,
}

fn rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_excluded(rel: &str, exclude_substrings: &[String]) -> bool {
    exclude_substrings.iter().any(|s| rel.contains(s.as_str()))
}

/// Project `ImportResolutionFact` rows: the declared-module pool comes from
/// `pool_roots` (REUSING the resolver's `build_module_path_index` key set — single
/// authority for "what module names exist"); the imports come from every `.dag` under
/// `importer_roots` whose repo-relative path contains none of `exclude_substrings`.
pub fn import_resolution_facts(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ImportResolutionFactRaw> {
    // LOAD-BEARING anchoring (not benign glue): build_module_path_index strips the
    // workspace prefix off each discovered path and PANICS if a path is not under `ws`, so
    // it requires workspace-absolute roots. The witness passes repo-relative roots, so we
    // must anchor them to the workspace first. ws.join(r) is idempotent if r is absolute.
    let ws = workspace_root();
    let abs_pool_roots: Vec<String> = pool_roots
        .iter()
        .map(|r| ws.join(r).to_string_lossy().into_owned())
        .collect();
    let declared: HashSet<String> = build_module_path_index(&abs_pool_roots)
        .into_keys()
        .collect();
    let mut out = Vec::new();
    for root in importer_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut dag_files);
        dag_files.sort();
        for file in dag_files {
            let rel = rel_path(&file);
            if is_excluded(&rel, exclude_substrings) {
                continue;
            }
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for import_module in extract_import_paths(&content) {
                let target_declared = declared.contains(&import_module);
                out.push(ImportResolutionFactRaw {
                    path: rel.clone(),
                    import_module,
                    target_declared,
                });
            }
        }
    }
    out
}
