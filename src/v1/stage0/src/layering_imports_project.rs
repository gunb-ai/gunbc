use std::path::{Path, PathBuf};

use crate::cli_run::{collect_dag_files_tolerant, extract_import_paths};

pub struct LayerImportFactRaw {
    pub layer: &'static str,
    pub path: String,
    pub import_module: String,
}

const LAYER_STD: &str = "LayerPrefixStd";
const LAYER_EXTDEPS: &str = "LayerPrefixExtdeps";

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
