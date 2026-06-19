//! Structural projection for `v2.lens.resolved_imports`.
//!
//! Enumerates `import <module>` facts from the `.dag` files under a set of importer
//! roots and tags each with whether its target module is DECLARED anywhere in the pool
//! roots — the structural form of the resolver's `UnresolvedImport` rule
//! (`find_module(module_index, import_path) == Absent`, src/v1/03_resolve.dag:170-173).
//!
//! Single authority (DESIGN §3): the declared-module pool REUSES the resolver's own
//! `build_module_path_index` (the same module-name index the pipeline resolves against,
//! which is itself fail-closed on duplicate module paths) rather than re-implementing a
//! `module ` header scan; the import-line enumeration REUSES `extract_import_paths` and
//! the tolerant `.dag` walk `collect_dag_files_tolerant` — exactly the primitives
//! `layering_imports_project` shares. So this projection forks no enumeration logic; it
//! only carries the structural import-resolution VERDICT that the parse-gated whole-tree
//! compile (`front_end_sources` short-circuits to `graph: none` on any parse error)
//! never reaches.
//!
//! Pure + deterministic: output is sorted by (path, import order), so a Wet run is
//! byte-identical across invocations. The importer-set policy (which paths are excluded,
//! e.g. the `/test/fixture/` text fixtures that intentionally carry broken imports) is
//! NOT decided here — the caller passes `exclude_substrings` (DESIGN §3 (c): the
//! scan-scope policy is workflow, not a fact baked into the projection).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::cli_run::{build_module_path_index, collect_dag_files_tolerant, extract_import_paths};

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
    let declared: HashSet<String> = build_module_path_index(pool_roots).into_keys().collect();
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
