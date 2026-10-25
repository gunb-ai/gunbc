//! Demand-acquired module index for plain `gunbc run`.
//!
//! `resolve_entry_graph` still builds a whole-tree `MultiEntryIndex`. That is the
//! remaining consumer of tree-scale tokenize for compile and the floor — not a
//! `gunbc run` fallback. This module is the cut the operator asked for on the wet
//! `gunbc run` path: locate the entry, follow import lines and dotted module
//! prefixes through `source_layout_lookup`, and hand resolve an index whose
//! `source_files` *are* that closure.
//!
//! Remaining whole-tree index: none. Unique-bare fill runs over the acquired
//! same-root files (`tree_bare_census_for_root` on that index), not a walk of
//! `--source-root`. Governed roots `dag` and `src/v2` stay a layout lookup.
//! Any other `--source-root` (a temp fixture, an out-of-tree tree) is indexed
//! by walking *that* root only — never the 8409-file repo trees. `gunbc compile`
//! still walks the tree.
//!
//! Seed-growth receipt: `gunbc.run_demand_acquire_seed_growth`
//! `run_demand_acquire_seed_growth_justification`.

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
    // GENERATED PROVIDERS ON THE DEMAND-ACQUIRED PATH. The acquired closure holds only what the entry's
    // authored references reach, so a provider a generated reference demands is outside it. The index
    // carries the layout's answer for such a module (provider_layout); the resolve core records every
    // provider the model asks to acquire that the layout holds and the index does not, and this loop
    // rebuilds the closure seeded with them. The seeds only grow and the layout is finite, so the loop
    // ends at a resolve that needs nothing outside, or at a refusal that names nothing new.
    let custom = Rc::new(custom_root_module_census(source_roots)?);
    let mut seeds: Vec<(String, Rc<v1_compiler_compile::SourceFile>)> = Vec::new();
    loop {
        let index = build_demand_acquired_index(source_roots, entry_file, &custom, &seeds)?;
        let roots = source_roots.to_vec();
        let census = custom.clone();
        *index.provider_layout.borrow_mut() = Some(Rc::new(move |module: &str| {
            layout_provider_answer(&roots, &census, module)
        }));
        match super::entry_resolve::resolve_entry_typed(&index, entry_file)? {
            super::entry_resolve::CoreResolve::Resolved(graph, si, _) => return Ok((graph, si)),
            super::entry_resolve::CoreResolve::NeedsAcquisition(providers) => {
                let fresh: Vec<_> = providers
                    .into_iter()
                    .filter(|(m, _)| !seeds.iter().any(|(s, _)| s == m))
                    .collect();
                if fresh.is_empty() {
                    return Err("generated provider acquisition made no progress on the demand-acquired path: every provider it asked for is already seeded".to_string());
                }
                seeds.extend(fresh);
            }
        }
    }
}

fn build_demand_acquired_index(
    source_roots: &[String],
    entry_file: &str,
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    provider_seeds: &[(String, Rc<v1_compiler_compile::SourceFile>)],
) -> Result<Rc<MultiEntryIndex>, String> {
    super::pre_entry_phase::mark_demand_acquired_path();
    let source_files = super::pre_entry_phase::timed(
        "demand_layout_acquire",
        super::pre_entry_phase::PhaseScale::Closure,
        || acquire_layout_closure_seeded(source_roots, entry_file, custom, provider_seeds),
    )?;
    super::pre_entry_phase::record_demand_acquired_modules(source_files.len());
    let facts = module_graph_facts_from_acquired(&source_files, source_roots);
    Ok(Rc::new(new_multi_entry_index_acquired(
        source_files,
        source_roots,
        facts,
    )))
}

/// The layout's typed answer for one module: the source it read, no module, or a module whose bytes
/// cannot be read. The read is the one `read_layout_module` performs, so Holds carries exactly the
/// bytes the rebuild will acquire.
pub(crate) fn layout_provider_answer(
    source_roots: &[String],
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    module: &str,
) -> super::entry_resolve::ProviderLayoutAnswer {
    match read_layout_module(source_roots, custom, module) {
        Ok(Some(source)) => super::entry_resolve::ProviderLayoutAnswer::Holds(source),
        Ok(None) => super::entry_resolve::ProviderLayoutAnswer::Lacks,
        Err(_) => super::entry_resolve::ProviderLayoutAnswer::Unreadable,
    }
}

pub(crate) fn acquire_layout_closure(
    source_roots: &[String],
    entry_file: &str,
) -> Result<ModuleSourceIndex, String> {
    let custom = custom_root_module_census(source_roots)?;
    acquire_layout_closure_seeded(source_roots, entry_file, &custom, &[])
}

/// The entry's layout closure, also closing over `provider_seeds` -- providers generated references
/// demand that no authored reference reaches, each with the exact source the pool answer read. A
/// seed is inserted from that snapshot, never read again; only its own imports are read, as any
/// closure's are. `custom` is the caller's custom-root census, taken once: walking it again here
/// would re-read every seed's file.
pub(crate) fn acquire_layout_closure_seeded(
    source_roots: &[String],
    entry_file: &str,
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    provider_seeds: &[(String, Rc<v1_compiler_compile::SourceFile>)],
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
    for (seed, snapshot) in provider_seeds {
        if index.contains_key(seed) {
            continue;
        }
        index.insert(seed.clone(), snapshot.clone());
        queue.push_back(seed.clone());
    }
    let mut known: HashSet<String> = index.keys().cloned().collect();
    while let Some(module) = queue.pop_front() {
        let Some(source) = index.get(&module).cloned() else {
            continue;
        };
        let demanded = demanded_module_names(&source.path, &source.content, &known, &custom);
        for name in demanded {
            if index.contains_key(&name) {
                continue;
            }
            let Some(acquired) = read_layout_module(source_roots, &custom, &name)? else {
                continue;
            };
            known.insert(name.clone());
            index.insert(name.clone(), acquired);
            queue.push_back(name);
        }
    }
    Ok(index)
}

fn demanded_module_names(
    path: &str,
    content: &str,
    known: &HashSet<String>,
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
) -> Vec<String> {
    let mut names: HashSet<String> = extract_import_paths(content).into_iter().collect();
    let self_module = extract_module_path(content).unwrap_or_default();
    if let Ok(refs) = parsed_file_references(path, content, &self_module, known) {
        for import in &refs.imports {
            names.insert(import.clone());
        }
        // Value/method chains are not module names: treating `x.foo.bar` as a layout
        // prefix pulled compiler/std modules the import graph never named, and that
        // set is what typecheck still pays ~9 GiB for on mtcollins1_boot_run.
        // Authored types and dotted free names are the remaining qualified demands
        // (e.g. `std.optional.Optional` without a matching import line).
        for name in refs.bare.iter().chain(refs.authored_types.iter()) {
            if name.contains('.') {
                let segs: Vec<String> = name.split('.').map(str::to_string).collect();
                if let Some(found) = longest_layout_module_prefix(&segs, custom) {
                    names.insert(found);
                }
            }
        }
    }
    names.into_iter().collect()
}

/// `dag` and `src/v2` are the layout trees. Every other `--source-root` is a custom
/// tree whose files are not at `<root(a)>/b/c.dag` under the workspace.
fn source_root_is_governed_layout(root: &str) -> bool {
    let spelling = root.trim_end_matches('/');
    if spelling == "dag" || spelling == "src/v2" {
        return true;
    }
    let anchored = super::anchor_source_root(root);
    let abs = Path::new(&anchored);
    let abs = if abs.is_absolute() {
        abs.to_path_buf()
    } else {
        workspace_root().join(abs)
    };
    let Ok(canon) = abs.canonicalize() else {
        return false;
    };
    let ws = workspace_root();
    matches!(ws.join("dag").canonicalize(), Ok(d) if d == canon)
        || matches!(ws.join("src/v2").canonicalize(), Ok(v) if v == canon)
}

/// Walk only non-governed roots. A wet `gunbc run --source-root dag --source-root src/v2`
/// never enters this walk.
fn custom_root_module_census(
    source_roots: &[String],
) -> Result<std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>, String> {
    let mut by_module: std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>> =
        std::collections::HashMap::new();
    for root in source_roots {
        if source_root_is_governed_layout(root) {
            continue;
        }
        let anchored = super::anchor_source_root(root);
        let root_path = Path::new(&anchored);
        if !root_path.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        super::collect_dag_files_result(root_path, &mut files)?;
        for acquired in files {
            let path = acquired.path();
            let content = acquired
                .read()
                .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
            let Some(name) = extract_module_path(&content) else {
                continue;
            };
            let src = Rc::new(v1_compiler_compile::SourceFile {
                path: super::module_index_path_key(path),
                content,
            });
            if let Some(existing) = by_module.get(&name) {
                if existing.path != src.path {
                    return Err(format!(
                        "module {name} is declared in both {} and {}",
                        existing.path, src.path
                    ));
                }
            }
            by_module.insert(name, src);
        }
    }
    Ok(by_module)
}

thread_local! {
    static LAYOUT_FILE_EXISTS: std::cell::RefCell<std::collections::HashMap<String, bool>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn layout_module_file_exists(name: &str) -> bool {
    LAYOUT_FILE_EXISTS.with(|c| {
        let mut c = c.borrow_mut();
        if let Some(hit) = c.get(name) {
            return *hit;
        }
        let exists = module_storage_relative(name)
            .ok()
            .is_some_and(|rel| workspace_root().join(&rel).is_file());
        c.insert(name.to_string(), exists);
        exists
    })
}

fn longest_layout_module_prefix(
    chain: &[String],
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
) -> Option<String> {
    if chain.len() < 2 {
        return None;
    }
    for len in (2..=chain.len()).rev() {
        let name = chain[..len].join(".");
        if custom.contains_key(&name) || layout_module_file_exists(&name) {
            return Some(name);
        }
    }
    None
}

fn read_layout_module(
    source_roots: &[String],
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    name: &str,
) -> Result<Option<Rc<v1_compiler_compile::SourceFile>>, String> {
    if let Some(derived) = read_derived_roster_module(name)? {
        return Ok(Some(derived));
    }
    if let Some(custom_src) = custom.get(name) {
        return Ok(Some(custom_src.clone()));
    }
    if !source_roots
        .iter()
        .any(|root| source_root_is_governed_layout(root))
    {
        return Ok(None);
    }
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

/// `gunbc.rung_drop.roster` and `gunbc.recurring_failure_mode.roster` have no file;
/// the tree walk mints them in memory (`derived_row_roster::acquire_dir_files`).
fn read_derived_roster_module(
    name: &str,
) -> Result<Option<Rc<v1_compiler_compile::SourceFile>>, String> {
    let Some(roster) = super::derived_row_roster::DERIVED_ROW_ROSTERS
        .iter()
        .copied()
        .find(|r| r.roster_module == name)
    else {
        return Ok(None);
    };
    let dir = workspace_root().join("dag").join(roster.row_dir_rel);
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for ent in std::fs::read_dir(&dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
        let ent = ent.map_err(|e| format!("read {}: {e}", dir.display()))?;
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            files.push(path);
        }
    }
    files.sort();
    let acquired = super::derived_row_roster::acquire_dir_files(&dir, files)?;
    let Some(roster_src) = acquired.into_iter().find(|a| a.is_derived()) else {
        return Ok(None);
    };
    let content = roster_src
        .read()
        .map_err(|e| format!("derived roster {name}: {e}"))?;
    let rel = format!("dag/{}/roster.dag", roster.row_dir_rel);
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
        build_id: crate::cli_run::next_facts_build_id(),
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

    fn write_identity_fixture(dir: &PathBuf) -> (String, String) {
        fs::write(
            dir.join("imported.dag"),
            "module test.scoped_identity.imported\n\nfn imported_claim() -> Bool { true }\n",
        )
        .unwrap();
        let entry = dir.join("entry.dag");
        fs::write(
            &entry,
            "module test.scoped_identity.entry\n\nimport test.scoped_identity.imported { imported_claim }\n",
        )
        .unwrap();
        (
            dir.to_string_lossy().into_owned(),
            entry.to_string_lossy().into_owned(),
        )
    }

    fn acquired_module_names(g: &crate::v1_compiler_compile::ResolvedGraph) -> Vec<String> {
        let mut names: Vec<String> = g
            .modules
            .iter()
            .map(|m| m.type_env.module_path.clone())
            .collect();
        names.sort();
        names
    }

    /// Custom `--source-root` files are not at `dag/<a>/<b>.dag`. Acquisition must
    /// still bind `import test.scoped_identity.imported` to `imported.dag` in the
    /// same temp tree (`scoped_declaration_identity_follows_the_selected_function_node`).
    #[test]
    fn custom_source_root_acquires_an_imported_sibling() {
        let dir = fixture_dir();
        let (root, entry) = write_identity_fixture(&dir);
        let roots = [root];
        let outcome = resolve_entry_graph_demand_acquired(&roots, &entry);
        let names = match &outcome {
            Ok((g, _)) => acquired_module_names(g),
            Err(e) => {
                let _ = fs::remove_dir_all(&dir);
                panic!("demand acquire of custom-root import: {e}");
            }
        };
        let _ = fs::remove_dir_all(&dir);
        assert!(
            names.iter().any(|n| n == "test.scoped_identity.imported"),
            "imported sibling must be in the acquired graph: {names:?}"
        );
    }

    /// Demand-acquire and the tree walk of the *same* custom root (not `dag`/`src/v2`)
    /// both bind the imported module. Walking the custom root is the preserved
    /// `--source-root` semantics; it is not a walk of the repo trees.
    #[test]
    fn custom_source_root_demand_and_tree_arms_acquire_the_imported_module() {
        let dir = fixture_dir();
        let (root, entry) = write_identity_fixture(&dir);
        let roots = [root];
        let demand = resolve_entry_graph_demand_acquired(&roots, &entry);
        let tree = crate::cli_run::resolve_entry_graph(&roots, &entry);
        let demand_names = match &demand {
            Ok((g, _)) => acquired_module_names(g),
            Err(e) => {
                let _ = fs::remove_dir_all(&dir);
                panic!("demand arm: {e}");
            }
        };
        let tree_names = match &tree {
            Ok((g, _)) => acquired_module_names(g),
            Err(e) => {
                let _ = fs::remove_dir_all(&dir);
                panic!("tree arm: {e}");
            }
        };
        let _ = fs::remove_dir_all(&dir);
        assert!(
            demand_names
                .iter()
                .any(|n| n == "test.scoped_identity.imported"),
            "demand arm missing imported: {demand_names:?}"
        );
        assert!(
            tree_names
                .iter()
                .any(|n| n == "test.scoped_identity.imported"),
            "tree arm missing imported: {tree_names:?}"
        );
    }

    fn diag_msgs(g: &crate::v1_compiler_compile::ResolvedGraph) -> Vec<String> {
        let mut v: Vec<String> = g
            .diagnostics
            .iter()
            .map(|d| crate::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect();
        v.sort();
        v
    }

    fn function_result(
        graph: &crate::v1_compiler_compile::ResolvedGraph,
        si: std::rc::Rc<im::HashMap<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>>,
        function: &str,
    ) -> String {
        let ctx = crate::cli_run::make_eval_context(
            graph,
            si,
            crate::v1_interpreter::ExecutionMode::Hermetic,
        );
        let tag = match crate::v1_interpreter::run_in_context_with_args(&ctx, function, &[], true) {
            Ok(_) => "ok".to_string(),
            Err(crate::v1_interpreter::InterpError::NoSuchFunction { name }) => {
                format!("NoSuchFunction:{name}")
            }
            Err(e) => format!("{e:?}"),
        };
        std::mem::forget(ctx);
        tag
    }

    /// One arm per process: `GUNBC_EQ_ARM=demand` or `tree`. Paired mode is unwritable
    /// (`PROCESS_RESOLVE_INDEX` / `DEMAND_PATH` leak). Ignored: tree arm ~8 GiB.
    #[test]
    #[ignore = "whole-tree shared index; memory-capped only"]
    fn demand_matches_whole_tree_resolve_on_selected_entries() {
        let root = repo_root();
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&root).expect("chdir");
        let roots = ["dag".to_string(), "src/v2".to_string()];
        let entries = [
            "dag/gunbc/machine_intake/mtcollins1_boot_admission.dag",
            "dag/gunbc/machine_intake/mtcollins1_boot_run.dag",
            "dag/gunbc/machine_intake/mtcollins1_sol_notice.dag",
            "dag/std/optional.dag",
            "dag/std/syllogism.dag",
            "dag/extdeps/network/ipv4.dag",
            "dag/gunbc/source_layout.dag",
            "dag/gunbc/v1/v1_maintenance_standing.dag",
        ];
        let function = "__no_such_function__";
        let selected: Vec<&str> = match std::env::var("GUNBC_EQ_ENTRY") {
            Ok(raw) if !raw.is_empty() => {
                let wanted: Vec<&str> = raw
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();
                entries
                    .iter()
                    .copied()
                    .filter(|e| {
                        wanted
                            .iter()
                            .any(|w| *e == *w || e.ends_with(w) || e.contains(w))
                    })
                    .collect()
            }
            _ => entries.to_vec(),
        };
        assert!(
            !selected.is_empty(),
            "GUNBC_EQ_ENTRY matched no probe entry"
        );
        let arm = std::env::var("GUNBC_EQ_ARM").unwrap_or_default();
        assert!(
            arm == "demand" || arm == "tree",
            "paired demand-then-tree in one process is forbidden: PROCESS_RESOLVE_INDEX / DEMAND_PATH leak. Set GUNBC_EQ_ARM=demand or GUNBC_EQ_ARM=tree in a fresh process"
        );
        eprintln!("eq_arm={arm}");
        for rel in selected {
            let entry = root.join(rel).to_string_lossy().into_owned();
            let started = std::time::Instant::now();
            eprintln!("eq_start {arm} {rel}");
            let outcome = if arm == "demand" {
                match super::resolve_entry_graph_demand_acquired(&roots, &entry) {
                    Ok((g, s)) => {
                        let diags = diag_msgs(&g);
                        let f = function_result(&g, s, function);
                        drop(g);
                        Ok((diags, f))
                    }
                    Err(e) => Err(e),
                }
            } else {
                match crate::cli_run::resolve_entry_graph(&roots, &entry) {
                    Ok((g, s)) => {
                        let diags = diag_msgs(&g);
                        let f = function_result(&g, s, function);
                        drop(g);
                        Ok((diags, f))
                    }
                    Err(e) => Err(e),
                }
            };
            match outcome {
                Ok((diags, f)) => {
                    eprintln!(
                        "eq_arm {arm} {rel} resolve=Ok elapsed_s={:.3} function={f} diag_count={}",
                        started.elapsed().as_secs_f64(),
                        diags.len()
                    );
                    for d in &diags {
                        eprintln!("eq_diag {arm} {d}");
                    }
                }
                Err(e) => {
                    eprintln!(
                        "eq_arm {arm} {rel} resolve=Err elapsed_s={:.3} err={e}",
                        started.elapsed().as_secs_f64()
                    );
                }
            }
        }
        let _ = std::env::set_current_dir(prev);
    }

    /// The demand-acquired layout answer at its real read boundary, under the governed `dag` root:
    /// a readable module is Holds with the bytes read, an absent one is Lacks, and a file whose bytes
    /// are not UTF-8 is Unreadable -- never Lacks.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn layout_provider_answer_keeps_unreadable_distinct_from_absent() {
        use super::super::entry_resolve::ProviderLayoutAnswer;
        let ws = super::super::process_workspace_root();
        let roots = vec![ws.join("dag").to_string_lossy().to_string()];
        let custom = std::collections::HashMap::new();
        let name = format!("test.fixture.gp_layout_probe_{}", std::process::id());
        let file = ws.join(format!(
            "dag/test/fixture/gp_layout_probe_{}.dag",
            std::process::id()
        ));
        std::fs::write(
            &file,
            format!("module {name}\n\nfn probe() -> Int {{ 1 }}\n"),
        )
        .expect("probe file");
        let holds = matches!(super::layout_provider_answer(&roots, &custom, &name), ProviderLayoutAnswer::Holds(src) if src.content.contains("fn probe"));
        std::fs::write(&file, [0xffu8, 0xfe, 0xfd]).expect("corrupt the probe");
        let unreadable = matches!(
            super::layout_provider_answer(&roots, &custom, &name),
            ProviderLayoutAnswer::Unreadable
        );
        let _ = std::fs::remove_file(&file);
        let lacks = matches!(
            super::layout_provider_answer(&roots, &custom, &name),
            ProviderLayoutAnswer::Lacks
        );
        assert!(holds, "a readable module must be Holds with its bytes");
        assert!(
            unreadable,
            "an unreadable module must be Unreadable, not Lacks"
        );
        assert!(lacks, "an absent module must be Lacks");
    }

    /// The seeded rebuild under a custom source root uses the retained snapshot: the provider's file
    /// is corrupted after the layout answered Holds, and acquisition still succeeds with the bytes
    /// the answer read. A census taken inside the rebuild would read the corrupted file and refuse.
    #[test]
    fn seeded_rebuild_under_a_custom_root_uses_the_snapshot_and_reads_nothing_again() {
        use super::super::entry_resolve::ProviderLayoutAnswer;
        let dir = fixture_dir();
        let roots = vec![dir.to_string_lossy().to_string()];
        let entry = dir.join("entry.dag");
        std::fs::write(&entry, "module probe_entry\n\nfn main() -> Int { 1 }\n").expect("entry");
        let provider = dir.join("provider.dag");
        std::fs::write(
            &provider,
            "module probe_provider\n\nfn route() -> Int { 2 }\n",
        )
        .expect("provider");
        let custom = super::custom_root_module_census(&roots).expect("census");
        let ProviderLayoutAnswer::Holds(snapshot) =
            super::layout_provider_answer(&roots, &custom, "probe_provider")
        else {
            panic!("the custom-root provider must be Holds");
        };
        std::fs::write(&provider, [0xffu8, 0xfe, 0xfd]).expect("corrupt the provider");
        let rebuilt = super::acquire_layout_closure_seeded(
            &roots,
            &entry.to_string_lossy(),
            &custom,
            &[("probe_provider".to_string(), snapshot)],
        );
        let _ = std::fs::remove_dir_all(&dir);
        let index = rebuilt.expect("the seeded rebuild must not read the provider again");
        assert!(
            index["probe_provider"].content.contains("fn route"),
            "the rebuild must hold the snapshot's bytes"
        );
    }
}
