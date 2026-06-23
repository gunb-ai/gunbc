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

/// One row of the module-name -> declaring-path node-resolution table: the
/// asymmetric endpoint `import_resolution_facts` does NOT expose. An import
/// edge's target is a module NAME; to continue an import-graph walk in `.dag`
/// you need that name's declaring path. This row is exactly that join key.
///
/// It is the SHARED primitive feeding both intent-linearity's ImportGraph axis
/// (the transitive consumed-input closure) and the LayerDAG roster axis (the
/// derived module roster `declared roster == module_declaration_facts(roots)`):
/// the host EMITS the node table, the `.dag` substrate does the traversal.
pub struct ModuleDeclarationFactRaw {
    pub module: String,
    pub path: String,
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

/// Project the module-name -> declaring-path index that `import_resolution_facts`
/// already builds internally (to set `target_declared`) into explicit rows. The
/// host emits this node-resolution table ONLY; the transitive walk that joins it
/// against the import edges stays in `.dag` (`v2.lens.module_graph`). `pool_roots`
/// is workspace-relative (e.g. `src/v2`, `dsl`) and the emitted `path` is the same
/// workspace-relative, forward-slash form `import_resolution_facts` emits for its
/// edge endpoints, so the two tables join on identical path strings.
pub fn module_declaration_facts(pool_roots: &[String]) -> Vec<ModuleDeclarationFactRaw> {
    let ws = workspace_root();
    let abs_pool_roots: Vec<String> = pool_roots
        .iter()
        .map(|r| ws.join(r).to_string_lossy().into_owned())
        .collect();
    let mut out: Vec<ModuleDeclarationFactRaw> = build_module_path_index(&abs_pool_roots)
        .into_iter()
        .map(|(module, path)| ModuleDeclarationFactRaw { module, path })
        .collect();
    out.sort_by(|a, b| a.module.cmp(&b.module));
    out
}
