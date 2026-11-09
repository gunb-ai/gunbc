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
use super::format_error_node;
use super::source_layout_lookup::module_storage_relative;
use super::{
    extract_import_paths, extract_module_path, workspace_relative_repo_path, workspace_root,
    ImportResolutionFactRaw, ModuleDeclarationFactRaw, ModuleGraphFactsLive, ModuleSourceIndex,
    MultiEntryIndex,
};
use crate::v1_compiler_compile;
use crate::v1_std_core::{
    is_error_diagnostic, is_interpreter_blocking_diagnostic, ErrorNode, NewlineIndex,
};
use im::HashMap;
use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::rc::Rc;

// THE READ-SET RECEIPT: the one seed addition the operator admitted for the planning fast lane
// (@ref(gunbc.design_ruling::seed_read_set_receipt_is_admitted)). Every filesystem fact demand acquisition
// consumes is recorded: a file it read, a path it probed and found absent (or could not admit), and a
// directory it enumerated. A change to any recorded path -- or under any recorded directory -- can change
// what this process acquires; a change to no recorded path cannot. When `GUNBC_READ_SET_OUT` names a file,
// each new fact is appended to it as one `<kind> <workspace-relative path>` line (kinds: file, probe, dir);
// unset, nothing is recorded or written. Appending as facts arrive (rather than at exit) keeps the receipt
// complete on every exit route, including process::exit and a refusal.
thread_local! {
    static READ_SET_SEEN: std::cell::RefCell<HashSet<String>> = std::cell::RefCell::new(HashSet::new());
}

pub(crate) fn read_set_record(kind: &str, path: &str) {
    read_set_record_to(std::env::var("GUNBC_READ_SET_OUT").ok(), kind, path);
}

/// The recorder with its destination passed in, so a test never touches the process-wide variable.
fn read_set_record_to(out: Option<String>, kind: &str, path: &str) {
    let Some(out) = out.filter(|o| !o.is_empty()) else {
        return;
    };
    let line = format!("{kind} {path}");
    let fresh = READ_SET_SEEN.with(|seen| seen.borrow_mut().insert(format!("{out}\t{line}")));
    if !fresh {
        return;
    }
    use std::io::Write;
    let written = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&out)
        .and_then(|mut f| writeln!(f, "{line}"));
    if let Err(e) = written {
        // A receipt that silently lost a fact would let a skip decision trust an incomplete read set.
        eprintln!("error: read-set receipt {out} could not record `{line}`: {e}");
        std::process::exit(2);
    }
}

/// THE ONE PROVIDER-SETTLING LOOP FOR `gunbc run`. `gunbc run` settles its entry closure through the
/// SAME authority the entry compile uses -- `demand_acquired_entry_resolved` -> the complete `.dag`
/// compile, every generated provider joining the closure each round -- and consumes that settled
/// graph. There is no second, lossy resolve: the old `demand_acquired_provider_fixed_point` loop
/// resolved through an empty occurrence transport and is deleted here.
///
/// DIAGNOSTIC BOUNDARY (the correctness condition this preserves): the lossy core refused
/// parse / resolve / normalize / typed-blocking failures BEFORE a graph was ever handed to the
/// interpreter. The `.dag` pipeline instead returns those failures as DATA -- and some live OUTSIDE
/// `graph.diagnostics`, only in `ResolvedPipelineResult.diagnostics` (front-end and normalize diags).
/// So the COMPLETE settled set is classified here, in the lossy stage order, and an absent graph is
/// refused -- before any interpreter effect or listener is created. run's own downstream gate (on
/// `graph.diagnostics` alone) cannot see the front-end / normalize diags; this wrapper must.
///
/// The run-settle option set (complexity / ownership / test-reference off) keeps run's diagnostic set
/// and cost equal to the loop it replaces: run's consumer reads no ownership facts and no complexity
/// report, and the interpreter does not enforce the test-reference gate (a `test` declaration is
/// entered only by the witness runner), so none of the three may be added to a run.
#[allow(clippy::type_complexity)]
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
    let (_sources, resolved) = demand_acquired_entry_resolved(
        source_roots,
        entry_file,
        CustomRootDuplicate::Refuse,
        v1_compiler_compile::run_settle_compile_options(),
    )?;

    let graph = match resolved.graph.clone() {
        Some(graph) => graph,
        // No settled graph: a front-end failure (parse / resolve / occurrence). Reproduce the lossy
        // parse / resolve refusal text. Rebuild the source-index map from the front-end's newline
        // indices so locations still carry line:column even with no graph.
        None => {
            let source_indices = Rc::new(
                resolved
                    .newline_indices
                    .iter()
                    .map(|nl| (nl.file.clone(), nl.clone()))
                    .collect::<HashMap<String, Rc<NewlineIndex>>>(),
            );
            return Err(super::format_error_nodes(
                &resolved.diagnostics,
                &source_indices,
            ));
        }
    };
    // The pipeline carries a `std` source-index map; the run contract carries an `im` map, as before.
    let source_indices: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new(
        resolved
            .source_indices
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<String, Rc<NewlineIndex>>>(),
    );
    // Resolve / normalize / front-end errors: settled diagnostics that are errors but are NOT part of
    // the typed graph (front-end + normalize diags live only on the pipeline result). The lossy core
    // refused these with `is_error_diagnostic` BEFORE its typed stage -- reproduce that ordering.
    let pre_typed: Vec<&Rc<ErrorNode>> = resolved
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .filter(|d| !graph.diagnostics.iter().any(|g| Rc::ptr_eq(g, d)))
        .collect();
    if !pre_typed.is_empty() {
        return Err(pre_typed
            .iter()
            .map(|d| format_error_node(d, &source_indices))
            .collect::<Vec<_>>()
            .join("\n"));
    }
    // Typed stage: the interpreter-blocking typecheck diagnostics -- exactly the lossy typed branch
    // (filter `is_interpreter_blocking_diagnostic`, one `format_error_node` each, joined).
    let typed_blocking: Vec<String> = graph
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| format_error_node(d, &source_indices))
        .collect();
    if !typed_blocking.is_empty() {
        return Err(typed_blocking.join("\n"));
    }
    // No blocking diagnostic (clean, or advisory-only): run may evaluate -- it re-gates on
    // `graph.diagnostics` downstream, which now carries the full typed set the interpreter enforces.
    Ok((graph, source_indices))
}

/// What an ENTRY COMPILE emits from: the final ordered closure and the ONE complete `.dag` resolution
/// of it. Generated providers the entry demands join the closure exactly as `gunbc run` acquires
/// them (same layout answers, same snapshot-seeded rebuild for a provider outside the acquired
/// index), but every provider round is a complete `.dag` compile and the round that settles is the
/// result emission starts from: the settled closure is never resolved a second time.
pub(crate) fn demand_acquired_entry_resolved(
    source_roots: &[String],
    entry_file: &str,
    duplicates: CustomRootDuplicate,
    options: v1_compiler_compile::CompilePipelineOptions,
) -> Result<
    (
        Vec<Rc<v1_compiler_compile::SourceFile>>,
        Rc<v1_compiler_compile::ResolvedPipelineResult>,
    ),
    String,
> {
    use super::provider_acquire::{ClosureOfError, PoolAnswer};
    use std::cell::RefCell;
    let custom = Rc::new(custom_root_module_census_under(source_roots, duplicates)?);
    struct Acquired {
        index: Rc<MultiEntryIndex>,
        seeds: Vec<(String, Rc<v1_compiler_compile::SourceFile>)>,
        snapshots: std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    }
    let entry_sources = |index: &Rc<MultiEntryIndex>,
                         seeds: &[(String, Rc<v1_compiler_compile::SourceFile>)]|
     -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
        let mut sources =
            super::entry_resolve::load_sources_for_entry_with_pool(index, entry_file)?;
        for (module, _) in seeds {
            let closure = super::entry_resolve::provider_closure_of(index, module)
                .map_err(|e| format!("acquired provider {module}: {e:?}"))?;
            for s in closure {
                if !sources.iter().any(|x| x.path == s.path) {
                    sources.push(s);
                }
            }
        }
        Ok(sources)
    };
    let index = build_demand_acquired_index(source_roots, entry_file, &custom, &[], duplicates)?;
    let sources = entry_sources(&index, &[])?;
    let state = RefCell::new(Acquired {
        index,
        seeds: Vec::new(),
        snapshots: std::collections::HashMap::new(),
    });
    // A module the index holds was read when the index was built. Outside it only the layout has an
    // answer, and a Holds keeps the bytes it read so the rebuild acquires exactly those.
    let pool = |m: &str| {
        if state.borrow().index.source_files.contains_key(m) {
            return PoolAnswer::Holds;
        }
        match layout_provider_answer(source_roots, &custom, m, duplicates) {
            super::entry_resolve::ProviderLayoutAnswer::Holds(source) => {
                state.borrow_mut().snapshots.insert(m.to_string(), source);
                PoolAnswer::Holds
            }
            super::entry_resolve::ProviderLayoutAnswer::Unreadable => PoolAnswer::Unreadable,
            super::entry_resolve::ProviderLayoutAnswer::Lacks => PoolAnswer::Lacks,
        }
    };
    let closure_of = |m: &str| -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, ClosureOfError> {
        let index = state.borrow().index.clone();
        super::entry_resolve::provider_closure_of(&index, m)
    };
    let rebuild = |outside: &[String], _current: &[Rc<v1_compiler_compile::SourceFile>]| {
        let mut st = state.borrow_mut();
        let fresh: Vec<_> = outside
            .iter()
            .filter(|m| !st.seeds.iter().any(|(s, _)| s == *m))
            .filter_map(|m| st.snapshots.get(m).map(|src| (m.clone(), src.clone())))
            .collect();
        if fresh.is_empty() {
            return Err("generated provider acquisition made no progress on the demand-acquired path: every provider it asked for is already seeded".to_string());
        }
        st.seeds.extend(fresh);
        let index =
            build_demand_acquired_index(source_roots, entry_file, &custom, &st.seeds, duplicates)?;
        let sources = entry_sources(&index, &st.seeds)?;
        st.index = index;
        Ok(sources)
    };
    super::provider_acquire::compile_to_provider_fixed_point_with(
        sources.clone(),
        options,
        &pool,
        &closure_of,
        &rebuild,
    )
}

fn build_demand_acquired_index(
    source_roots: &[String],
    entry_file: &str,
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    provider_seeds: &[(String, Rc<v1_compiler_compile::SourceFile>)],
    duplicates: CustomRootDuplicate,
) -> Result<Rc<MultiEntryIndex>, String> {
    super::pre_entry_phase::mark_demand_acquired_path();
    let source_files = super::pre_entry_phase::timed(
        "demand_layout_acquire",
        super::pre_entry_phase::PhaseScale::Closure,
        || {
            acquire_layout_closure_seeded(
                source_roots,
                entry_file,
                custom,
                provider_seeds,
                duplicates,
            )
        },
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
    duplicates: CustomRootDuplicate,
) -> super::entry_resolve::ProviderLayoutAnswer {
    match read_layout_module(source_roots, custom, module, duplicates) {
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
    acquire_layout_closure_seeded(
        source_roots,
        entry_file,
        &custom,
        &[],
        CustomRootDuplicate::Refuse,
    )
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
    duplicates: CustomRootDuplicate,
) -> Result<ModuleSourceIndex, String> {
    let path = Path::new(entry_file);
    read_set_record("file", &workspace_relative_repo_path(entry_file));
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
            let Some(acquired) = read_layout_module(source_roots, &custom, &name, duplicates)?
            else {
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
pub(super) fn source_root_is_governed_layout(root: &str) -> bool {
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
/// WHAT A MODULE DECLARED IN TWO NON-GOVERNED ROOTS MEANS. `gunbc run` has no precedence policy, so
/// it refuses. A compile whose request declares `primary_precedence` keeps the FIRST root's
/// declaration, which is what the whole-tree index did for it (review 90897): the acquired route
/// honours the field the caller declared rather than silently changing its meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomRootDuplicate {
    Refuse,
    FirstRootWins,
}

pub(super) fn custom_root_module_census(
    source_roots: &[String],
) -> Result<std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>, String> {
    custom_root_module_census_under(source_roots, CustomRootDuplicate::Refuse)
}

fn custom_root_module_census_under(
    source_roots: &[String],
    duplicates: CustomRootDuplicate,
) -> Result<std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>, String> {
    let mut by_module: std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>> =
        std::collections::HashMap::new();
    for root in source_roots {
        if source_root_is_governed_layout(root) {
            continue;
        }
        // A ROOT THAT IS NOT A DIRECTORY REFUSES; IT IS NOT SKIPPED. Skipping dropped a typo'd or
        // uncommitted root from the population silently (review 90892); the admission is the
        // whole-tree builder's own (`admit_source_root`).
        let anchored = super::admit_source_root(root)?;
        read_set_record("dir", &workspace_relative_repo_path(&anchored));
        let root_path = Path::new(&anchored);
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
                    match duplicates {
                        CustomRootDuplicate::Refuse => {
                            return Err(format!(
                                "module {name} is declared in both {} and {}",
                                existing.path, src.path
                            ))
                        }
                        CustomRootDuplicate::FirstRootWins => continue,
                    }
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
        let exists = module_storage_relative(name).ok().is_some_and(|rel| {
            let found = workspace_root().join(&rel).is_file();
            read_set_record(if found { "file" } else { "probe" }, &rel);
            found
        });
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

/// ONE MODULE'S SOURCE UNDER THE POOL ROOTS, BY SOURCE-ROOT AUTHORITY: each root, in order, is asked
/// whether it holds `name` -- a governed root through the layout path (`read_governed_layout_module`),
/// a custom root through the custom census (only a file that lies under that root). The first root
/// that holds it owns it under `FirstRootWins`; under `Refuse` a second DISTINCT file in another root
/// is the duplicate refusal, whichever kinds the two roots are. This is the whole-tree builders'
/// meaning (`try_build_module_index[_primary_precedence]`), which the census-first read had changed:
/// the custom census skips governed roots, so a governed/custom duplicate was never seen and custom
/// won silently under either policy and either root order. `gunbc run` (Refuse) shares the rule.
fn read_layout_module(
    source_roots: &[String],
    custom: &std::collections::HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    name: &str,
    duplicates: CustomRootDuplicate,
) -> Result<Option<std::rc::Rc<crate::v1_compiler_compile::SourceFile>>, String> {
    if let Some(derived) = read_derived_roster_module(name)? {
        return Ok(Some(derived));
    }
    let mut owner: Option<Rc<v1_compiler_compile::SourceFile>> = None;
    for root in source_roots {
        let held = if source_root_is_governed_layout(root) {
            read_governed_layout_module(std::slice::from_ref(root), name)?
        } else {
            custom
                .get(name)
                .filter(|src| {
                    let path = Path::new(&src.path);
                    let abs = if path.is_absolute() {
                        path.to_path_buf()
                    } else {
                        workspace_root().join(path)
                    };
                    under_source_roots(std::slice::from_ref(root), &abs)
                })
                .cloned()
        };
        let Some(held) = held else { continue };
        match &owner {
            None => {
                if duplicates == CustomRootDuplicate::FirstRootWins {
                    return Ok(Some(held));
                }
                owner = Some(held);
            }
            Some(first) if first.path == held.path => {}
            Some(first) => {
                return Err(format!(
                    "module {name} is declared in both {} and {}",
                    first.path, held.path
                ))
            }
        }
    }
    Ok(owner)
}

/// The layout-path read of a governed module, admitted only when its file lies under one of `roots`.
fn read_governed_layout_module(
    source_roots: &[String],
    name: &str,
) -> Result<Option<std::rc::Rc<crate::v1_compiler_compile::SourceFile>>, String> {
    let Some((_, rel, content)) =
        read_layout_file(name, |abs| under_source_roots(source_roots, abs))?
    else {
        return Ok(None);
    };
    Ok(Some(Rc::new(v1_compiler_compile::SourceFile {
        path: workspace_relative_repo_path(&rel),
        content,
    })))
}

/// THE ONE PATH-DERIVED READ of a layout module: `gunbc.source_layout` maps `name` to its file,
/// the file is read, and its own `module` line must say `name` (a misfiled file is a refusal, never
/// a quiet miss). `None` when the name has no layout path or no file stands there. `admit` is asked before the read (a file outside the roots is never opened). Shared by
/// `read_layout_module` (closure acquisition) and the keyed module/declaration reads
/// (`keyed_module_lookup`): the name-to-path mapping is applied in one place. Returns the absolute
/// path, the workspace-relative layout path, and the content.
pub(super) fn read_layout_file(
    name: &str,
    admit: impl Fn(&Path) -> bool,
) -> Result<Option<(std::path::PathBuf, String, String)>, String> {
    let Ok(rel) = module_storage_relative(name) else {
        return Ok(None);
    };
    let abs = workspace_root().join(&rel);
    if !abs.is_file() || !admit(&abs) {
        read_set_record("probe", &rel);
        return Ok(None);
    }
    read_set_record("file", &rel);
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
    Ok(Some((abs, rel, content)))
}

/// The `DERIVED_ROW_ROSTERS` modules (`gunbc.rung_drop.roster`, `gunbc.recurring_failure_mode.roster`,
/// `gunbc.guarantee_stall.roster`) have no file;
/// the tree walk mints them in memory (`derived_row_roster::acquire_dir_files`).
pub(super) fn read_derived_roster_module(
    name: &str,
) -> Result<Option<std::rc::Rc<crate::v1_compiler_compile::SourceFile>>, String> {
    let Some(roster) = super::derived_row_roster::DERIVED_ROW_ROSTERS
        .iter()
        .copied()
        .find(|r| r.roster_module == name)
    else {
        return Ok(None);
    };
    let dir = workspace_root().join("dag").join(roster.row_dir_rel);
    read_set_record("dir", &format!("dag/{}", roster.row_dir_rel));
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

pub(super) fn under_source_roots(source_roots: &[String], abs: &Path) -> bool {
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
    // THE READ-SET RECEIPT records each fact once, in arrival order, as `<kind> <path>`; with no
    // destination it writes nothing (the control: no file appears). The destination is passed in, so
    // this test never sets GUNBC_READ_SET_OUT and cannot race a sibling test's acquisition.
    #[test]
    fn read_set_receipt_records_each_fact_once_and_only_when_asked() {
        let dir = std::env::temp_dir().join(format!("gunbc-read-set-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let out = dir.join("read-set.txt");
        let _ = std::fs::remove_file(&out);
        let dest = Some(out.to_string_lossy().into_owned());
        super::read_set_record_to(None, "file", "dag/never/recorded.dag");
        super::read_set_record_to(Some(String::new()), "file", "dag/never/recorded.dag");
        assert!(!out.exists(), "no destination must write nothing");
        super::read_set_record_to(dest.clone(), "file", "dag/a.dag");
        super::read_set_record_to(dest.clone(), "probe", "dag/b.dag");
        super::read_set_record_to(dest.clone(), "file", "dag/a.dag");
        super::read_set_record_to(dest, "dir", "dag/gunbc/rung_drop");
        let text = std::fs::read_to_string(&out).expect("receipt written");
        assert_eq!(
            text,
            "file dag/a.dag\nprobe dag/b.dag\ndir dag/gunbc/rung_drop\n"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    use super::{
        build_demand_acquired_index, custom_root_module_census_under,
        demand_acquired_entry_resolved, resolve_entry_graph_demand_acquired, CustomRootDuplicate,
    };
    use crate::v1_compiler_compile;
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;
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
        let ws = crate::cli_run::process_workspace_root();
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
        let holds = matches!(super::layout_provider_answer(&roots, &custom, &name, super::CustomRootDuplicate::Refuse), ProviderLayoutAnswer::Holds(src) if src.content.contains("fn probe"));
        std::fs::write(&file, [0xffu8, 0xfe, 0xfd]).expect("corrupt the probe");
        let unreadable = matches!(
            super::layout_provider_answer(
                &roots,
                &custom,
                &name,
                super::CustomRootDuplicate::Refuse
            ),
            ProviderLayoutAnswer::Unreadable
        );
        let _ = std::fs::remove_file(&file);
        let lacks = matches!(
            super::layout_provider_answer(
                &roots,
                &custom,
                &name,
                super::CustomRootDuplicate::Refuse
            ),
            ProviderLayoutAnswer::Lacks
        );
        assert!(holds, "a readable module must be Holds with its bytes");
        assert!(
            unreadable,
            "an unreadable module must be Unreadable, not Lacks"
        );
        assert!(lacks, "an absent module must be Lacks");
    }

    /// A MODULE DECLARED IN TWO CUSTOM ROOTS: `gunbc run` (Refuse) refuses and names both files; an
    /// entry compile declaring primary precedence (FirstRootWins) keeps the FIRST root's file, as the
    /// whole-tree index did (review 90897). RED: drop the FirstRootWins arm and the second assertion
    /// fails with the duplicate refusal.
    #[test]
    fn a_module_in_two_custom_roots_refuses_on_run_and_keeps_the_first_root_under_precedence() {
        let first = fixture_dir();
        let second = fixture_dir();
        std::fs::write(
            first.join("m.dag"),
            "module dup_probe\n\nfn which() -> Int { 1 }\n",
        )
        .expect("first");
        std::fs::write(
            second.join("m.dag"),
            "module dup_probe\n\nfn which() -> Int { 2 }\n",
        )
        .expect("second");
        let roots = vec![
            first.to_string_lossy().to_string(),
            second.to_string_lossy().to_string(),
        ];
        let refused =
            super::custom_root_module_census_under(&roots, super::CustomRootDuplicate::Refuse);
        assert!(
            matches!(&refused, Err(cause) if cause.contains("module dup_probe is declared in both")),
            "run must refuse a module declared in two roots: {refused:?}"
        );
        let kept = super::custom_root_module_census_under(
            &roots,
            super::CustomRootDuplicate::FirstRootWins,
        )
        .expect("primary precedence admits the duplicate");
        assert!(
            kept.get("dup_probe")
                .is_some_and(|src| src.content.contains("{ 1 }")),
            "primary precedence keeps the first root's declaration"
        );
    }

    /// GOVERNED/CUSTOM DUPLICATE, BOTH ROOT ORDERS x BOTH POLICIES (operator finding 2): the module
    /// `test.fixture.gc_dup_<pid>` is declared under the governed `dag` root AND in a custom root.
    /// Whole-tree meaning: FirstRootWins keeps the first root's file; Refuse refuses either order.
    /// RED: the census-first read picked custom in all four cells.
    #[test]
    fn a_governed_custom_duplicate_obeys_root_order_and_policy() {
        let ws = crate::cli_run::process_workspace_root();
        let pid = std::process::id();
        let name = format!("test.fixture.gc_dup_{pid}");
        let gfile = ws.join(format!("dag/test/fixture/gc_dup_{pid}.dag"));
        std::fs::write(
            &gfile,
            format!("module {name}\n\nfn which() -> Int {{ 1 }}\n"),
        )
        .expect("gov");
        let dir = fixture_dir();
        std::fs::write(
            dir.join("c.dag"),
            format!("module {name}\n\nfn which() -> Int {{ 2 }}\n"),
        )
        .expect("custom");
        let dag = ws.join("dag").to_string_lossy().to_string();
        let cust = dir.to_string_lossy().to_string();
        let read = |roots: &Vec<String>, d| {
            let census = super::custom_root_module_census_under(roots, d).expect("census");
            super::read_layout_module(roots, &census, &name, d)
        };
        let gov_first = vec![dag.clone(), cust.clone()];
        let cust_first = vec![cust, dag];
        let which =
            |r: Result<Option<std::rc::Rc<crate::v1_compiler_compile::SourceFile>>, String>| {
                r.expect("admitted")
                    .expect("held")
                    .content
                    .contains("{ 1 }")
            };
        let outcome = (
            which(read(&gov_first, super::CustomRootDuplicate::FirstRootWins)),
            !which(read(&cust_first, super::CustomRootDuplicate::FirstRootWins)),
            read(&gov_first, super::CustomRootDuplicate::Refuse).err(),
            read(&cust_first, super::CustomRootDuplicate::Refuse).err(),
        );
        let _ = std::fs::remove_file(&gfile);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(outcome.0, "[dag, custom] primary precedence: governed wins");
        assert!(outcome.1, "[custom, dag] primary precedence: custom wins");
        assert!(
            outcome
                .2
                .as_deref()
                .is_some_and(|c| c.contains("declared in both")),
            "[dag, custom] strict refuses: {:?}",
            outcome.2
        );
        assert!(
            outcome
                .3
                .as_deref()
                .is_some_and(|c| c.contains("declared in both")),
            "[custom, dag] strict refuses: {:?}",
            outcome.3
        );
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
        let ProviderLayoutAnswer::Holds(snapshot) = super::layout_provider_answer(
            &roots,
            &custom,
            "probe_provider",
            super::CustomRootDuplicate::Refuse,
        ) else {
            panic!("the custom-root provider must be Holds");
        };
        std::fs::write(&provider, [0xffu8, 0xfe, 0xfd]).expect("corrupt the provider");
        let rebuilt = super::acquire_layout_closure_seeded(
            &roots,
            &entry.to_string_lossy(),
            &custom,
            &[("probe_provider".to_string(), snapshot)],
            super::CustomRootDuplicate::Refuse,
        );
        let _ = std::fs::remove_dir_all(&dir);
        let index = rebuilt.expect("the seeded rebuild must not read the provider again");
        assert!(
            index["probe_provider"].content.contains("fn route"),
            "the rebuild must hold the snapshot's bytes"
        );
    }

    /// CONTROL FOR THE COMPILE PROVIDER LOOP: `host_text_crossing_entry_witness` imports only
    /// `v2.std.text`, so its authored closure lacks the coercion route its text crossing demands;
    /// only the provider loop acquires it. RED: the unseeded authored closure (what the entry
    /// compile resolved over before it shared run's loop) is strictly smaller and lacks the route.
    #[test]
    fn an_entry_compile_acquires_the_generated_provider_a_crossing_demands() {
        let ws = crate::cli_run::process_workspace_root();
        let entry = ws
            .join("dag/test/claim/host_text_crossing_entry_witness.dag")
            .to_string_lossy()
            .to_string();
        let roots = vec![
            ws.join("dag").to_string_lossy().to_string(),
            ws.join("src/v2").to_string_lossy().to_string(),
        ];
        let (acquired, resolved) = demand_acquired_entry_resolved(
            &roots,
            &entry,
            CustomRootDuplicate::FirstRootWins,
            v1_compiler_compile::default_compile_pipeline_options(),
        )
        .expect("the loop settles");
        // The returned resolution IS the compile of the returned ordered closure: emission starts
        // from it with no second resolve, so its module set is exactly the closure's.
        let graph = resolved
            .graph
            .clone()
            .expect("the settled closure resolves");
        assert_eq!(graph.modules.len(), acquired.len());
        let custom = custom_root_module_census_under(&roots, CustomRootDuplicate::FirstRootWins)
            .expect("census");
        let unseeded_index = build_demand_acquired_index(
            &roots,
            &entry,
            &custom,
            &[],
            CustomRootDuplicate::FirstRootWins,
        )
        .expect("index");
        let unseeded = crate::cli_run::entry_resolve::load_sources_for_entry_with_pool(
            &unseeded_index,
            &entry,
        )
        .expect("authored closure");
        let has_route = |v: &[Rc<v1_compiler_compile::SourceFile>]| {
            v.iter().any(|s| s.path.ends_with("coercion.dag"))
        };
        assert!(
            !has_route(&unseeded),
            "authored closure must lack the route"
        );
        assert!(has_route(&acquired), "the provider loop must acquire it");
        assert!(acquired.len() > unseeded.len());
    }
}
