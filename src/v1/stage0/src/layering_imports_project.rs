//! Structural projection for `v2.lens.layering_imports`.
//!
//! Enumerates `import <module>` facts from the `.dag` files under a set of layer roots,
//! projecting one `LayerImportFact { layer, path, import_module }` per import line. This
//! is the single-authority replacement for the deleted `scripts/layering-imports-scan.sh`
//! enumerator: it REUSES the resolver's own `collect_dag_files_tolerant` (recursive `.dag`
//! walk) and `extract_import_paths` (import-line extraction) rather than re-implementing
//! the `find | grep | sed` pipeline (which forked that authority).
//!
//! Pure + deterministic: the output is sorted by (path, import order), so a Wet run is
//! byte-identical across invocations. Layer tagging is NOT decided here — the caller
//! passes the std-layer roots and extdeps-layer roots separately (DESIGN §3 (c): the
//! layer<->root assignment is workflow policy, not a fact baked into the projection).

use std::path::{Path, PathBuf};

use crate::cli_run::{collect_dag_files_tolerant, extract_import_paths};

/// One projected import fact: the layer label variant name, the repo-relative `.dag`
/// path, and the imported module path.
pub struct LayerImportFactRaw {
    pub layer: &'static str,
    pub path: String,
    pub import_module: String,
}

const LAYER_STD: &str = "LayerStd";
const LAYER_EXTDEPS: &str = "LayerExtdeps";

fn rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn project_root(root: &str, layer: &'static str, out: &mut Vec<LayerImportFactRaw>) {
    let root_path = Path::new(root);
    if !root_path.is_dir() {
        return;
    }
    let mut dag_files: Vec<PathBuf> = Vec::new();
    collect_dag_files_tolerant(root_path, &mut dag_files);
    dag_files.sort();
    for file in dag_files {
        let content = match std::fs::read_to_string(&file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = rel_path(&file);
        for import_module in extract_import_paths(&content) {
            out.push(LayerImportFactRaw {
                layer,
                path: rel.clone(),
                import_module,
            });
        }
    }
}

/// Project `LayerImportFact` rows for the given std-layer and extdeps-layer roots.
pub fn layer_import_facts(
    std_roots: &[String],
    extdeps_roots: &[String],
) -> Vec<LayerImportFactRaw> {
    let mut out = Vec::new();
    for root in std_roots {
        project_root(root, LAYER_STD, &mut out);
    }
    for root in extdeps_roots {
        project_root(root, LAYER_EXTDEPS, &mut out);
    }
    out
}
