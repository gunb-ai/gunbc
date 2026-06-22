use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::cli_run::{
    build_module_path_index, collect_dag_files_tolerant, extract_import_paths, workspace_root,
};

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

pub fn import_resolution_facts(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ImportResolutionFactRaw> {
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
