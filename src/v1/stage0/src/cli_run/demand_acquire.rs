//! Demand-acquired module index for plain `gunbc run`.
//!
//! `resolve_entry_graph` still builds a whole-tree `MultiEntryIndex`. That is the
//! remaining consumer of tree-scale tokenize for compile and the floor. This
//! module is the cut the operator asked for on the wet `gunbc run` path: locate
//! the entry, follow import lines and dotted module prefixes through
//! `source_layout_lookup`, and hand resolve an index whose `source_files` *are*
//! that closure.
//!
//! Remaining tree-scale reads on this path: none. Census, graph facts and
//! reconcile underlays run over the acquired set. `gunbc compile` still walks
//! the tree (parse-as-check); that is a named other consumer, not this verb.

use super::entry_resolve::{
    new_multi_entry_index_acquired, parsed_file_references, resolve_entry_with_index,
    ReferenceSelectionTier,
};
use super::source_layout_lookup::module_storage_relative;
use super::{
    extract_import_paths, extract_module_path, workspace_relative_repo_path, workspace_root,
    ImportResolutionFactRaw, ModuleDeclarationFactRaw, ModuleGraphFactsLive, ModuleSourceIndex,
    MultiEntryIndex,
};
use crate::v1_compiler_compile;
use crate::v1_std_core::NewlineIndex;
use im::HashMap;
use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::rc::Rc;

pub fn resolve_entry_graph_demand_acquired(
    source_roots: &[String],
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let index = build_demand_acquired_index(source_roots, entry_file)?;
    resolve_entry_with_index(&index, entry_file)
}

fn build_demand_acquired_index(
    source_roots: &[String],
    entry_file: &str,
) -> Result<Rc<MultiEntryIndex>, String> {
    let source_files = acquire_layout_closure(source_roots, entry_file)?;
    let facts = module_graph_facts_from_acquired(&source_files, source_roots);
    Ok(Rc::new(new_multi_entry_index_acquired(
        source_files,
        source_roots,
        facts,
    )))
}

pub(crate) fn acquire_layout_closure(
    source_roots: &[String],
    entry_file: &str,
) -> Result<ModuleSourceIndex, String> {
    let path = Path::new(entry_file);
    if !path.is_file() {
        return Err(format!(
            "entry file does not exist or is not a file: {entry_file}"
        ));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read entry {entry_file}: {e}"))?;
    let Some(entry_module) = extract_module_path(&content) else {
        return Err(format!(
            "entry {entry_file} has no `module` declaration, so a layout closure cannot start"
        ));
    };
    let entry_rel = workspace_relative_repo_path(entry_file);
    let mut index: ModuleSourceIndex = HashMap::new();
    index.insert(
        entry_module.clone(),
        Rc::new(v1_compiler_compile::SourceFile {
            path: entry_rel.clone(),
            content,
        }),
    );
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(entry_module);
    let mut known: HashSet<String> = index.keys().cloned().collect();
    while let Some(module) = queue.pop_front() {
        let Some(source) = index.get(&module).cloned() else {
            continue;
        };
        let demanded = demanded_module_names(&source.path, &source.content, &known);
        for name in demanded {
            if index.contains_key(&name) {
                continue;
            }
            let Some(acquired) = read_layout_module(source_roots, &name)? else {
                continue;
            };
            known.insert(name.clone());
            index.insert(name.clone(), acquired);
            queue.push_back(name);
        }
    }
    Ok(index)
}

fn demanded_module_names(path: &str, content: &str, known: &HashSet<String>) -> Vec<String> {
    let mut names: HashSet<String> = extract_import_paths(content).into_iter().collect();
    let self_module = extract_module_path(content).unwrap_or_default();
    if let Ok(refs) = parsed_file_references(path, content, &self_module, known) {
        for import in &refs.imports {
            names.insert(import.clone());
        }
        let chains = refs
            .chains
            .iter()
            .chain(refs.positions.method_chains.iter());
        for chain in chains {
            if let Some(name) = longest_layout_module_prefix(chain) {
                names.insert(name);
            }
        }
        for name in refs
            .bare
            .iter()
            .chain(refs.authored_types.iter())
            .chain(refs.positions.undotted.iter())
        {
            if name.contains('.') {
                let segs: Vec<String> = name.split('.').map(str::to_string).collect();
                if let Some(found) = longest_layout_module_prefix(&segs) {
                    names.insert(found);
                }
            }
        }
    }
    names.into_iter().collect()
}

fn longest_layout_module_prefix(chain: &[String]) -> Option<String> {
    if chain.len() < 2 {
        return None;
    }
    for len in (2..=chain.len()).rev() {
        let name = chain[..len].join(".");
        if module_storage_relative(&name)
            .ok()
            .is_some_and(|rel| workspace_root().join(&rel).is_file())
        {
            return Some(name);
        }
    }
    None
}

fn read_layout_module(
    source_roots: &[String],
    name: &str,
) -> Result<Option<Rc<v1_compiler_compile::SourceFile>>, String> {
    let Ok(rel) = module_storage_relative(name) else {
        return Ok(None);
    };
    let abs = workspace_root().join(&rel);
    if !abs.is_file() {
        return Ok(None);
    }
    if !under_source_roots(source_roots, &abs) {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&abs).map_err(|e| {
        format!(
            "failed to read layout module {name} at {}: {e}",
            abs.display()
        )
    })?;
    let declared = extract_module_path(&content);
    if declared.as_deref() != Some(name) {
        return Err(format!(
            "layout file {} for module {name} declares {}",
            abs.display(),
            declared.unwrap_or_else(|| "<no module line>".to_string())
        ));
    }
    Ok(Some(Rc::new(v1_compiler_compile::SourceFile {
        path: workspace_relative_repo_path(&rel),
        content,
    })))
}

fn under_source_roots(source_roots: &[String], abs: &Path) -> bool {
    let ws = workspace_root();
    source_roots.iter().any(|root| {
        let anchored = super::anchor_source_root(root);
        let root_path = Path::new(&anchored);
        let root_abs = if root_path.is_absolute() {
            root_path.to_path_buf()
        } else {
            ws.join(root_path)
        };
        abs.starts_with(&root_abs)
    })
}

fn module_graph_facts_from_acquired(
    source_files: &ModuleSourceIndex,
    source_roots: &[String],
) -> ModuleGraphFactsLive {
    let mut nodes: Vec<ModuleDeclarationFactRaw> = source_files
        .iter()
        .map(|(module, source)| ModuleDeclarationFactRaw {
            module: module.clone(),
            path: source.path.clone(),
        })
        .collect();
    nodes.sort_by(|a, b| a.module.cmp(&b.module));
    let mut edges: Vec<ImportResolutionFactRaw> = Vec::new();
    for source in source_files.values() {
        for import_module in extract_import_paths(&source.content) {
            let target_declared = source_files.contains_key(&import_module);
            edges.push(ImportResolutionFactRaw {
                path: source.path.clone(),
                import_module,
                target_declared,
            });
        }
    }
    let adjacency = super::entry_resolve::build_import_adjacency(&edges, &nodes);
    let declared_paths: std::collections::HashSet<String> = nodes
        .iter()
        .map(|n| workspace_relative_repo_path(&n.path))
        .collect();
    let path_to_module: HashMap<String, String> = nodes
        .iter()
        .map(|n| (workspace_relative_repo_path(&n.path), n.module.clone()))
        .collect();
    let selection = Rc::new(ReferenceSelectionTier::new(
        source_roots.to_vec(),
        edges,
        &nodes,
    ));
    ModuleGraphFactsLive {
        nodes,
        adjacency,
        declared_paths,
        selection,
        path_to_module,
        read_refusals: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_entry_graph_demand_acquired;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static FIXTURE: AtomicUsize = AtomicUsize::new(0);

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root")
    }

    fn fixture_dir() -> PathBuf {
        let dir = repo_root().join("target").join(format!(
            "gunbc-demand-acquire-{}-{}",
            std::process::id(),
            FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).expect("fixture dir");
        dir
    }

    #[test]
    fn unparseable_module_outside_the_closure_does_not_change_resolve() {
        let dir = fixture_dir();
        let entry = dir.join("entry.dag");
        fs::write(
            &entry,
            "module test.fixtures.demand_acquire_control\nfn main() -> Bool { true }\n",
        )
        .unwrap();
        let roots = [dir.to_string_lossy().into_owned()];
        let entry_s = entry.to_string_lossy().into_owned();
        let without = resolve_entry_graph_demand_acquired(&roots, &entry_s);
        fs::write(dir.join("garbage.dag"), "this is not a module ((((").unwrap();
        let with = resolve_entry_graph_demand_acquired(&roots, &entry_s);
        let _ = fs::remove_dir_all(&dir);
        assert!(without.is_ok(), "{without:?}");
        assert_eq!(
            without.is_ok(),
            with.is_ok(),
            "an unparseable sibling outside the layout closure must not change resolve: without={without:?} with={with:?}"
        );
    }
}
