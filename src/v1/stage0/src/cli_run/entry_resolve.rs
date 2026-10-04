// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 31 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::disallowed_macros,  // 10
    clippy::doc_lazy_continuation,  // 2
    clippy::items_after_test_module,  // 1
    clippy::redundant_closure,  // 1
    clippy::type_complexity,  // 8
    dead_code,  // 9
    unused_imports,  // 0 -- pre-existing
)]
// cli_run.rs is this module's PARENT, and an `#![allow]` there reaches every module
// under it -- the same cascade this commit removed at the crate root, one level down.
// These are the names its roster carries that this module does not trip, restored to
// warn so `-D warnings` still judges them here. A name moves from this list to the
// allow list above only with a counted site, never silently.
#![warn(
    clippy::assertions_on_constants,
    clippy::clone_on_copy,
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_str_replace,
    clippy::empty_line_after_doc_comments,
    clippy::enum_variant_names,
    clippy::iter_kv_map,
    clippy::manual_is_multiple_of,
    clippy::manual_strip,
    clippy::map_identity,
    clippy::missing_const_for_thread_local,
    clippy::needless_borrow,
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::single_char_add_str,
    clippy::too_many_arguments,
    clippy::unnecessary_to_owned,
    clippy::unneeded_struct_pattern,
    clippy::useless_vec,
    unused_mut
)]

use super::*;
use im::HashMap;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::coproduct_reflection::{decl_facts_corpus_walk, DeclFactRaw};
use crate::module_path_index::{
    parse_module_binding, ModuleBindingOutcome, ModuleBindingRefusal, ParsedModuleBinding,
};
use crate::shared_typecheck_store::{self, SharedTypecheckCaches};
use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::{kernel_type_set, SourceSpan};
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::{
    lookup_binding_by_name, lookup_type_by_name, qualified_all_but_last, symbol_index_insert,
    symbol_index_lookup, GlobalBareLookupState, SymbolIndex, TypeEnv,
};
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_infer_lookup::global_bare_callable_node;
use crate::v1_compiler_infer_method::infer_builtin_call_type;
use crate::v1_compiler_infer_sigs::{lookup_resolved_sig, ResolvedFuncEnv, ResolvedFuncSig};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_interpreter::str_value;
use crate::v1_interpreter::Value;
use crate::v1_rt;
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_body, arm_pattern, authored_name_at, block_stmts,
    build_newline_index, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    empty_intern_table, empty_node_list, expr_call_func_at, expr_method_name_at, expr_var_name_at,
    field_access_base, field_access_field_at, field_init_node_name_at, field_init_node_value,
    has_child_named, inferred_to_node, intern, is_error_diagnostic,
    is_interpreter_blocking_diagnostic, let_binding_name_at, let_value, make_error_node,
    match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver, module_items, no_span,
    param_node_name_at, param_node_type_expr, Cardinality, CompilerDiagnostic, Connective,
    ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable, MatchPattern, NewlineIndex,
    Node,
};
use serde::Serialize;

pub fn build_module_path_index(source_roots: &[String]) -> HashMap<String, String> {
    with_module_path_index(source_roots, |index| index.clone())
}

/// THE CACHED MODULE-PATH INDEX, BORROWED. A keyed question -- is this module declared, where does
/// it live -- reads one entry, so it must not pay a copy of every entry to ask. `build_module_path_index`
/// hands its caller an owned index and therefore clones the cached one on every call; a keyed read
/// made once per visited module or per edge target turned that into corpus-sized work per step.
/// This is the same cache, the same fill and the same hit accounting, lent for the duration of `f`.
pub fn with_module_path_index<R>(
    source_roots: &[String],
    f: impl FnOnce(&HashMap<String, String>) -> R,
) -> R {
    let key = source_roots
        .iter()
        .map(|r| anchor_source_root(r))
        .collect::<Vec<_>>()
        .join("\u{1f}");
    let cached = MODULE_PATH_INDEX_CACHE.with(|cache| cache.borrow().contains_key(&key));
    if cached {
        shared_fill::record_hit("module_path_index", &key);
    } else {
        shared_fill::begin_fill();
        let start = std::time::Instant::now();
        let index = build_module_path_index_uncached(source_roots);
        shared_fill::record_fill("module_path_index", &key, start.elapsed().as_nanos() as u64);
        MODULE_PATH_INDEX_CACHE.with(|cache| cache.borrow_mut().insert(key.clone(), index));
    }
    MODULE_PATH_INDEX_CACHE.with(|cache| {
        let cache = cache.borrow();
        f(cache.get(&key).expect("module path index inserted above"))
    })
}

pub(crate) fn build_module_path_index_uncached(source_roots: &[String]) -> HashMap<String, String> {
    let mut index: HashMap<String, String> = HashMap::new();
    for_each_parsed_module_binding(source_roots, |root_idx, path, binding| {
        let rel = module_index_path_key(path);
        if manifest_stub_superseded_by_overlay(&rel, source_roots, root_idx) {
            return;
        }
        if let Some(existing) = index.get(&binding.module_path) {
            if existing == &rel || same_canonical_file(existing, &rel) {
                return;
            }
            if root_idx > 0 {
                // Primary-precedence multi-root: root[0] owns overlapping module paths;
                // later roots contribute only absent modules (build_module_index_primary_precedence).
                return;
            }
        }
        insert_module_path(&mut index, &binding.module_path, rel);
    });
    index
}

/// Project `whole_tree_strict_resolve_exclusion_substrings` out of the ci_layer_roots authority.
pub(crate) fn whole_tree_strict_resolve_exclusion_substrings_from_source(
    content: &str,
) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(
        content,
        WHOLE_TREE_STRICT_RESOLVE_EXCLUSION_SUBSTRINGS_DATA_NAME,
    )
}

/// Whole-tree strict-resolve probe exclusions — `gunbc.ci_layer_roots.whole_tree_strict_resolve_exclusion_substrings`.
pub fn whole_tree_strict_resolve_exclusion_substrings() -> Vec<String> {
    static EXCLUDES: OnceLock<Vec<String>> = OnceLock::new();
    EXCLUDES
        .get_or_init(|| {
            whole_tree_strict_resolve_exclusion_substrings_from_source(
                ci_layer_roots_authority_content(),
            )
        })
        .clone()
}

/// Floor discovery ∪ whole-tree probe pattern policy — `gunbc.ci_layer_roots.whole_tree_resolve_exclusion_substrings`.
pub fn whole_tree_resolve_exclusion_substrings() -> Vec<String> {
    let mut excludes = witness_exclusion_substrings();
    excludes.extend(whole_tree_strict_resolve_exclusion_substrings());
    excludes
}

/// Whole-tree strict-walk probe exclusion authority — pattern rows ∪ derived module-path
/// closure (`census_exclude_derive`). Replaces hand-pinned `--exclude-subpath` lists.
pub fn whole_tree_probe_exclusion_substrings() -> Vec<String> {
    census_exclude_derive::whole_tree_probe_exclusion_substrings()
}

pub fn build_module_path_index_from_witness_roots() -> HashMap<String, String> {
    build_module_path_index(&default_source_roots())
}

/// THE PANICKING WRAPPER, RETAINED FOR CALLERS WHOSE INPUTS ARE ALREADY
/// INVARIANT-ESTABLISHED. New callers on a user-supplied path must use
/// `try_build_module_index` -- a missing root, an unreadable directory or a non-UTF-8
/// source is an ORDINARY, USER-CAUSED condition and owes a typed located refusal, not a
/// panic four frames down (DESIGN §5).
pub(crate) fn build_module_index(source_roots: &[String]) -> ModuleSourceIndex {
    try_build_module_index(source_roots).unwrap_or_else(|e| panic!("{e}"))
}

/// `primary-precedence` pool indexing: the first root is authoritative; later roots
/// fill only modules not already present (matches `gunbc compile --dependency-pool-index
/// primary-precedence` in `dag_compile_clean_transport`).
pub(crate) fn build_module_index_primary_precedence(source_roots: &[String]) -> ModuleSourceIndex {
    try_build_module_index_primary_precedence(source_roots).unwrap_or_else(|e| panic!("{e}"))
}

pub(crate) fn import_closure_dag_files(
    workspace: &Path,
    source_roots: &[PathBuf],
    seed_entries: &[&str],
) -> Result<HashSet<String>, String> {
    let index = dag_module_index(source_roots)?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = Vec::new();
    for rel in seed_entries {
        let path = workspace.join(rel);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("read declared Class B gate entry {rel}: {e}"))?;
        seen.insert(normalize_repo_path(rel));
        queue.push(content);
    }
    while let Some(content) = queue.pop() {
        for module_path in extract_import_paths(&content) {
            let Some(candidates) = index.get(&module_path) else {
                continue;
            };
            for path in candidates {
                let rel =
                    normalize_repo_path(&workspace_relative_repo_path(&path.to_string_lossy()));
                if !seen.insert(rel) {
                    continue;
                }
                let file_content = std::fs::read_to_string(path)
                    .map_err(|e| format!("read imported module {}: {e}", path.display()))?;
                queue.push(file_content);
            }
        }
    }
    Ok(seen)
}

/// Normalize `source_roots` to the workspace-relative form `import_resolution_facts` /
/// `module_declaration_facts` expect when invoked from `.dag` (`witness_layer_roots` style).
pub(crate) fn pool_roots_for_module_graph_closure(source_roots: &[String]) -> Vec<String> {
    source_roots
        .iter()
        .map(|r| {
            let p = Path::new(r);
            if p.is_absolute() {
                repo_relative_path_normalized(p)
            } else {
                r.replace('\\', "/")
            }
        })
        .collect()
}

pub(crate) fn path_to_source_lookup(
    index: &ModuleSourceIndex,
) -> HashMap<String, Rc<v1_compiler_compile::SourceFile>> {
    let mut out = HashMap::new();
    for sf in index.values() {
        let rel = workspace_relative_repo_path(&sf.path);
        out.insert(rel, sf.clone());
        out.insert(sf.path.clone(), sf.clone());
    }
    out
}

pub(crate) fn build_import_adjacency(
    edges: &[ImportResolutionFactRaw],
    nodes: &[ModuleDeclarationFactRaw],
) -> HashMap<String, Vec<String>> {
    let mut module_to_path: HashMap<String, String> = HashMap::new();
    for node in nodes {
        module_to_path.insert(
            node.module.clone(),
            workspace_relative_repo_path(&node.path),
        );
    }

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        let Some(imported) = module_to_path.get(&edge.import_module) else {
            continue;
        };
        let importer = workspace_relative_repo_path(&edge.path);
        let entry = adjacency.entry(importer).or_default();
        if !entry.iter().any(|p| p == imported) {
            entry.push(imported.clone());
        }
    }
    adjacency
}

/// Worklist BFS over pre-normalized adjacency (O(V+E) per entry).
pub fn import_closure_from_adjacency(
    entry_path: &str,
    adjacency: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let entry_path = workspace_relative_repo_path(entry_path);
    let mut reached: HashSet<String> = HashSet::new();
    reached.insert(entry_path.clone());
    let mut queue: VecDeque<String> = VecDeque::from([entry_path]);

    while let Some(importer) = queue.pop_front() {
        let Some(targets) = adjacency.get(&importer) else {
            continue;
        };
        for path in targets {
            if reached.insert(path.clone()) {
                queue.push_back(path.clone());
            }
        }
    }

    let mut result: Vec<String> = reached.into_iter().collect();
    result.sort();
    result
}

pub(crate) fn build_module_graph_facts_live_uncached(
    pool_roots: &[String],
) -> ModuleGraphFactsLive {
    #[cfg(test)]
    MODULE_GRAPH_FACTS_BUILD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    const EXCLUDE: &[String] = &[];
    let roots = pool_roots_for_module_graph_closure(pool_roots);
    // NOTE: the module-graph LOADER closure stays import-derived for now (Blocker-1 part 1). A
    // reference-derived closure changes every witness's load set tree-wide and surfaces latent issues
    // (import-less-but-referencing std files, witnesses that need src/v1 in their pool, the
    // pre-existing fleet_converge Srv3 red, and homonyms the bright-cat lane must qualify), so the
    // loader repoint is staged as a separate part after those land. The REFERENCE producer below is
    // was, until gunbc#8141, already live via the inert-lens reach (the strips' documented CI
    // blocker); that consumer is deleted, so the reference producer's remaining live readers are
    // affected-set selection and the loader closure.
    //
    // EDGE SOURCE — the swap `module_graph.dag`'s `dependency_edge_source_migration_note` designates:
    // "when [the namespace terminal step] lands, `dependency_resolution_facts_live` swaps to the
    // reference-derived producer and nothing above it changes". Imports were deleted from most of the
    // corpus without this half landing, which left ~530 claim modules with an empty adjacency and a
    // widen arm below that answered "affected" for all of them — the absorbing fallback DESIGN §5
    // names verbatim ("can't compute the affected set → rerun the entire suite").
    //
    // Attempted 2026-07-14 and REVERTED: unioning `reference_edges_as_import_facts(..., false)`
    // ballooned a single small witness entry's load set from 27 to 424 resolved sources. That
    // measurement was correct and its conclusion ("the information is unusable here") was not — it
    // was taken at the `strict = false` tier, which keeps `AmbiguousBare` edges, so every ubiquitous
    // homonym fans its referrers out across the pool (median closure 1136 of 2240 modules).
    //
    // The tier is the fix. `strict = true` keeps Qualified + UniqueBare and drops AmbiguousBare:
    // median closure 96, p95 554 — the same order as the import-only baseline's 54/175 — and 522 of
    // the 530 edgeless claim modules gain a real edge. Measured over 14 merged diffs the selected
    // witness share goes 70.3% → 49.4% (the `false` tier goes the wrong way, to 83.4%).
    //
    // The two consumers take DIFFERENT tiers on purpose, and conflating them is what made this look
    // impossible: for the LOADER an over-connected edge is harmless (a superset just compiles extra
    // modules), while for SELECTION it is precisely the thing that destroys the answer. The loader
    // (`extend_with_bare_reference_closure`) is deliberately left alone.
    //
    // Import-bearing files emit no reference edges at all (see `reference_resolution_facts` pass 2),
    // so on an un-stripped file the union is a no-op and the graph is byte-identical to before.
    // FIVE ROWS, EACH NET OF THE OTHERS. The import-edge facts are the module path index's first
    // demander, and that index is the first demander of every pool file's lexing, newline index
    // and heads parse (`pool_acquire`, which records those three as `pool_source_tokenize`,
    // `pool_source_newline_index` and `pool_heads_parse`). So the index is forced here on
    // the key the facts read it under and timed net of the acquisitions it forces, and the facts
    // row is what remains: the walk, the read and the import-line scan.
    let acquired_before = super::pool_acquire::attributed();
    let index_started = std::time::Instant::now();
    with_module_path_index(&pool_roots_abs(&roots), |_| ());
    super::pre_entry_phase::record(
        "module_path_index_build",
        super::pre_entry_phase::PhaseScale::Tree,
        index_started
            .elapsed()
            .saturating_sub(super::pool_acquire::attributed() - acquired_before),
    );
    let observation = super::pre_entry_phase::timed(
        "graph_facts_import_edges",
        super::pre_entry_phase::PhaseScale::Tree,
        || import_resolution_facts_with_observation(&roots, &roots, EXCLUDE),
    );
    let edges = observation.facts;
    let nodes = super::pre_entry_phase::timed(
        "graph_facts_module_declarations",
        super::pre_entry_phase::PhaseScale::Tree,
        || module_declaration_facts(&roots),
    );
    // Loader tier: import edges only, unchanged. Every consumer that goes on to RESOLVE what it
    // reaches reads this one.
    let adjacency = build_import_adjacency(&edges, &nodes);
    // Selection tier: import edges + strict reference edges. PRODUCED ON DEMAND, not here: the
    // resolve path asks it per closure file (`reference_only_direct_import_paths`), and only the
    // affected-set consumers ask it for the whole pool (`selection_adjacency`), so building it
    // eagerly made every `gunbc run` full-parse the pool for the edges of a few modules. See
    // `ReferenceSelectionTier`.
    let selection = Rc::new(ReferenceSelectionTier::new(
        roots.clone(),
        edges.clone(),
        &nodes,
    ));
    let declared_paths = nodes
        .iter()
        .map(|n| workspace_relative_repo_path(&n.path))
        .collect();
    let path_to_module: HashMap<String, String> = nodes
        .iter()
        .map(|n| (workspace_relative_repo_path(&n.path), n.module.clone()))
        .collect();
    ModuleGraphFactsLive {
        nodes,
        adjacency,
        selection,
        declared_paths,
        path_to_module,
        read_refusals: observation.read_refusals,
    }
}

pub fn build_module_graph_facts_live(pool_roots: &[String]) -> ModuleGraphFactsLive {
    let key = pool_roots_for_module_graph_closure(pool_roots).join("\u{1f}");
    MODULE_GRAPH_FACTS_CACHE.with(|cache| {
        if let Some(facts) = cache.borrow().get(&key) {
            shared_fill::record_hit("module_graph_facts", &key);
            return facts.clone();
        }
        shared_fill::begin_fill();
        let start = std::time::Instant::now();
        let facts = build_module_graph_facts_live_uncached(pool_roots);
        shared_fill::record_fill(
            "module_graph_facts",
            &key,
            start.elapsed().as_nanos() as u64,
        );
        cache.borrow_mut().insert(key, facts.clone());
        facts
    })
}

/// Host realization of `v2.lens.module_graph.import_closure_live`.
pub fn import_closure_live_paths(
    entry_path: &str,
    pool_roots: &[String],
) -> Result<Vec<String>, String> {
    let facts = build_module_graph_facts_live(pool_roots);
    Ok(import_closure_live_paths_with_facts(entry_path, &facts))
}

pub fn import_closure_live_paths_with_facts(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
) -> Vec<String> {
    import_closure_from_adjacency(entry_path, &facts.adjacency)
}

/// Axis (iv) of the fourth-axis law (`live-read-witness-classification-design (plan doc deleted 2026-08-28)`
/// §7): does `entry_path`'s import closure reach a declared live-read carrier home, and is
/// any path touched at all? This is a G1-only (module-closure) mirror of the landed G2
/// call-reachability lens (`v2.lens.live_read_classification`) — G2's carrier set is always
/// a superset of G1's under the same closure (`merge_g1_and_g2_carriers`), so this coarser
/// Rust check is fail-closed-safe relative to the full `.dag` authority: it may over-report
/// (an extra witness run) but never under-report (a missed run). It does not attempt to
/// prove which touched path a reached carrier actually reads at runtime (that precision is
/// G2/G3's job) — reachability plus any touch is treated as a hit.
pub(crate) fn import_closure_module_reaches_carrier_home(
    closure_modules: &HashSet<String>,
    carrier_home: &str,
) -> bool {
    closure_modules.iter().any(|module| {
        module == carrier_home
            || module
                .strip_prefix(carrier_home)
                .is_some_and(|suffix| suffix.starts_with('.'))
    })
}

pub fn load_sources_for_entry(
    source_roots: &[String],
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let index = process_shared_index(source_roots);
    load_sources_for_entry_with_pool(&index, entry_path)
}

/// Pool-index-aware variant of `load_sources_for_entry`: builds the closure
/// index with the SAME dependency-pool policy the census pool uses, so the
/// reference-derived closure and the whole-tree name census agree on which root
/// provides a cross-root homonym module (DESIGN §3 single authority — the two
/// membership authorities must not fork on a duplicated module path). Without
/// this the `--entry` compile built its closure strict (all roots compete,
/// duplicate module path panics) while the census honored
/// `--dependency-pool-index primary-precedence` (root[0] wins, later roots fill
/// only absent modules), so the requested pool policy was silently ignored on
/// the closure side. `primary_precedence=true` selects root[0]-wins; `false`
/// keeps strict.
pub fn load_sources_for_entry_with_pool_index(
    source_roots: &[String],
    entry_path: &str,
    primary_precedence: bool,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    if primary_precedence {
        let index = build_multi_entry_index_primary_precedence(source_roots);
        return load_sources_for_entry_with_pool(&index, entry_path);
    }
    // Strict pool policy routes through the process-shared index (DESIGN §3 —
    // one index authority per (thread, canonical roots)). Before this, a
    // `gunbc compile --entry` process built its closure index here, dropped it,
    // and then the first compile-clean diagnostic classification rebuilt the
    // SAME index inside resolve_entry_graph_shared to evaluate the
    // compile_clean_diagnostic_policy row — a second full pool_parse of the
    // corpus (measured: 5,450 pool_parse files for a 2,725-module pool, two
    // tree censuses per root) to read one policy Bool. Same construction fn
    // (build_multi_entry_index), same canonical roots key, so sharing is a
    // cache hit, not a behavior change; primary-precedence keeps its own
    // fresh build (process_shared_index only builds strict).
    let index = process_shared_index(source_roots);
    load_sources_for_entry_with_pool(&index, entry_path)
}

// ── THE V2 EMISSION TRANSACTION: ONE PRODUCER, TWO CONSUMERS ─────────────────
//
// WHY THIS EXISTS AT ALL. On 2026-08-23 `gunbc compile --source-root dag --source-root
// src/v2 --entry src/v2/compiler/03_ingest.dag` refused outright on main -- no emitted
// tree, no cargo log -- while every required phase stayed green for hours. The required
// run parses `src/v1` .dag, compares the regen mirrors, and folds the witness floor;
// NONE OF THE THREE COMPILES A v2 ENTRY, so the emission path had no observer at all.
// The break was a trailing `//` annotation block with no declaration after it, authored
// under `dag/test/manual/`, which no required phase reads either.
//
// WHY IT IS A SHARED FUNCTION AND NOT A SECOND CALLER. The gate MUST refuse wherever
// the cargo board's producer refuses, or it can green while the board is broken -- two
// answers to one question, which is the failure this whole lane keeps finding. The board
// runs `gunbc compile --entry M --target rust --dependency-pool-index primary-precedence`
// (`docs/probes/curated_cargo_probe_one.sh`, whose EMIT_REFUSE verdict is exactly that
// command exiting nonzero). So the emission transaction lives HERE, and BOTH the CLI's
// `--entry` arm and the required phase call it. Keeping two callers equal by hand would
// have been a fork with three live parameters to drift on -- the pool-index policy, the
// census population, and (until it was deleted as permanently green) the CLI's silent-pick
// gate -- and the first draft of this phase had already drifted on the first of them.
//
// THE ENTRY IS THE v2 COMPILER ROOT, AND THE PARAGRAPH THAT USED TO SIT HERE ARGUED FOR
// THE OPPOSITE (operator ruling, 2026-08-25). It chose the smallest entry in the tree and
// its reasoning was sound on the facts it had: an `--entry` compile is scoped in what it
// EMITS (the reference-derived closure) and whole-tree in what it PARSES -- every indexed
// module outside the closure enters the name census, so the census parse reaches the whole
// of `dag` + `src/v2` whichever entry is named. The class that escaped on 2026-08-23
// therefore refuses on the small entry too, and the large entry's extra minutes bought
// emission coverage of the compiler's own closure rather than coverage of that class.
// Against a SERIAL required run, that was the wrong trade.
//
// WHAT CHANGED IS THE DENOMINATOR, NOT THE ARGUMENT. The required run is now two parallel
// jobs, and this phase rides the `build` lane opposite a witnesses lane that costs an order
// of magnitude more, so the extra minutes are free rather than added. They buy exactly what
// the old paragraph said they buy and declined: emission coverage of the v2 compiler's own
// closure. The roster row and the one file of coverage the change gives up are in
// `gunbc.ci_layer_roots` `required_v2_emission_entries`; the durations are not restated
// here, because the producer named above re-derives them and a transcribed number rots.
//
// WHAT THE INVARIANT IS, AND WHAT IT IS NOT. NOT a file count: a legitimate compiler
// change may alter a closure's size, so `emitted == 177` is a CHANGE DETECTOR wearing an
// invariant's clothes. The invariant is that emission COMPLETED -- the transaction ran to
// its end and produced a tree -- and the refusal predicate is not restated here either:
// it is `v1_compiler_compile` `stage0_self_compile_refusal_message`, the same authority
// the CLI already stops on (a blocking diagnostic, or an empty emitted file set) -- AND
// NOTHING ELSE. It formerly named the CLI's own silent-pick gate as a second refusal
// source; that gate was deleted as permanently green, so the sentence is corrected here
// rather than left standing, because a reader of this refusal contract would otherwise
// conclude silent-pick is walled on this path. A gate that refused on ANY diagnostic would be
// permanently red -- the v2 compiler closure carries hundreds of advisory diagnostics
// against zero blocking, which is a standing property of the corpus rather than a figure
// worth pinning -- so advisory diagnostics are COUNTED and reported and never refused on.
// Ratcheting that advisory population is a separate construction and is deliberately not
// attempted here: a merge-blocking comparison against a count measured on the current tree
// is the tree-copied census oracle DESIGN §5 rejects, and the honest form is a monotone
// debt contract at IDENTITY grain over an independently closed subject universe.
//
// WHAT THIS DOES NOT CATCH, named rather than left to be inferred: a rustc error in the
// emitted tree (nothing here compiles the emission), a semantic regression that still
// emits, or an emission break confined to a closure the configured entry does not reach.

pub(crate) fn load_sources_for_entry_with_pool(
    index: &MultiEntryIndex,
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let cache_key = workspace_relative_entry_path(entry_path);
    if let Some(cached) = index.entry_closure_sources.borrow().get(&cache_key) {
        return Ok(cached.clone());
    }
    // The import closure only: the module-path reference half is the fixpoint's own
    // (`extend_with_reference_closure_for_pool`), read from the index's one parse per file.
    // Running `extend_with_reference_closure` first answered the same question from a second,
    // per-entry full parse of every closure file.
    let sources =
        load_import_closure_for_entry(&index.source_files, &index.module_graph_facts, entry_path)?;
    let sources = extend_sources_to_both_closure_fixpoint(sources, index)?;
    index
        .entry_closure_sources
        .borrow_mut()
        .insert(cache_key, sources.clone());
    Ok(sources)
}

pub(crate) fn load_sources_for_entry_with_index(
    index: &MultiEntryIndex,
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let sources =
        load_import_closure_for_entry(&index.source_files, &index.module_graph_facts, entry_path)?;
    let mut sources = extend_with_reference_closure(sources, index)?;
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    sources.dedup_by(|a, b| a.path == b.path);
    Ok(sources)
}

/// An entry and its import closure, with no reference edges followed.
fn load_import_closure_for_entry(
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let entry_source = entry_source_from_index_or_disk(index, entry_path)?;
    let rel_path = entry_source.path.clone();

    let import_closure_started = std::time::Instant::now();
    let sources = resolve_transitively(vec![entry_source.clone()], index, facts)?;
    resolve_stage_slot_add(|s| {
        s.load_import_closure += import_closure_started.elapsed().as_nanos()
    });
    let mut sources = sources;
    if !sources
        .iter()
        .any(|s| s.path == rel_path || same_canonical_file(&s.path, &rel_path))
    {
        sources.push(entry_source);
    }
    Ok(sources)
}

pub fn resolve_entry_graph(
    source_roots: &[String],
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    // Route through the same loader engine as `resolve_entry_graph_shared`
    // (proven behaviorally identical to the cold import-adjacency resolve by
    // resolve_typed_cache_equivalence_test): with imports stripped (namespace
    // wave 1) an entry's dependencies are name-derived, and the old
    // `load_sources_for_entry_with_index` walk only follows import edges — a
    // stripped fixed entry (e.g. the floor runner) failed to resolve at all.
    let index = process_shared_index(source_roots);
    resolve_entry_with_index(&index, entry_file)
}

// Process-level (per-thread) resolve store — the S1a increment of the resolver
// graph-major design (resolver-graph-major-design (plan doc deleted 2026-08-28)). Within one
// process the source tree is a fixed snapshot, so a resolved entry graph is a
// pure fact of (source_roots, entry) — the same purity assumption the walk memo
// (M1) and typed_module_cache already ship on. Routing every fixed-entry
// consumer (floor runner context, diff observer, output policy, group syntax,
// the executor's plan entry) through this store makes "resolve the same declared
// machinery twice in one process" unwritable on these paths, with failure
// semantics unchanged: a miss resolves exactly as before, including the typed
// error path. Thread-local by design: resolved graphs are Rc-based (not Send);
// shard threads keep their own store rather than smuggling Rc across threads.
thread_local! {
    #[allow(clippy::type_complexity)]
    pub(crate) static PROCESS_RESOLVE_STORE: RefCell<
        HashMap<
            (String, String),
            (
                Rc<v1_compiler_compile::ResolvedGraph>,
                Rc<HashMap<String, Rc<NewlineIndex>>>,
            ),
        >,
    > = RefCell::new(HashMap::new());

    // The thread's ONE shared resolve index (union-resolve S1,
    // resolver-graph-major-design (plan doc deleted 2026-08-28) §7). Every fixed-entry consumer routed
    // through resolve_entry_graph_shared (the executor prelude: plan entry + output
    // policy + group syntax, plus the floor runner) resolves against this single
    // MultiEntryIndex, so its parse/typed caches share the union of all those closures:
    // the shared std/spec prefix typechecks ONCE, not once per prelude entry. Keyed by
    // source_roots — a run's roots are fixed, so this is a get-or-build over exactly one pool
    // per slot (below). Thread-local by the same Rc-not-Send reason as the store:
    // each shard keeps its own index rather than smuggling Rc across threads.
    //
    // TWO SLOTS, ONE PER POOL SEMANTICS, and the pair is what makes the memo safe rather than
    // merely faster. `primary-precedence` (root[0] wins, later roots fill only absent modules)
    // and strict are DIFFERENT POOLS over the same roots: serving one where the other was asked
    // for is a silently divergent resolution, which is the §5 fail-open this cache would
    // otherwise introduce. So precedence is part of the identity of the slot, not a build flag
    // applied to a shared one -- a roots-keyed single slot cannot express the distinction and
    // would answer whichever mode ran first.
    #[allow(clippy::type_complexity)]
    //
    // ONE RESIDENT POOL PER SLOT, REFUSED AT BUILD. The map is keyed by canonical roots so a
    // second demand for the resident roots is a hit, never an eviction and rebuild; a demand for
    // DIFFERENT roots is refused by `try_process_shared_index_for_pool`
    // (`SharedIndexSecondResidentPool`) before anything is walked, so a slot never holds more
    // than one entry. A one-shot reader of another pool (a fixture, the regen round's emitted
    // tree) owns its index instead. `shared_index_residency_control` reads the bound back at
    // the end of the floor and of a regen round as the positive control.
    pub(crate) static PROCESS_RESOLVE_INDEX: RefCell<[BTreeMap<String, Rc<MultiEntryIndex>>; 2]> =
        const { RefCell::new([BTreeMap::new(), BTreeMap::new()]) };
}

/// Canonical spelling for the shared-index roots — both the key AND the build
/// inputs: an absolute root under the workspace normalizes to its repo-relative
/// form, so the executor's CLI `$ROOT/dag` and the plan's declared `dag`
/// (`gunbc.ci_layer_roots` witness_layer_roots) address ONE index. Without this,
/// the compile-clean receipt (armed from CLI roots) and batch-2 discovery (plan
/// roots) keyed two separate typed universes in CI and the gate's warm store was
/// silently replaced before the corpus read it (review 39118 on PR #6783). Order
/// is preserved (primary-precedence pool semantics); a root outside the workspace
/// keeps its spelling — it is genuinely a different pool.
pub(crate) fn canonical_shared_index_roots(source_roots: &[String]) -> Vec<String> {
    source_roots
        .iter()
        .map(|r| {
            let p = Path::new(r);
            if p.is_absolute() {
                try_repo_relative_path_normalized(p).unwrap_or_else(|| r.replace('\\', "/"))
            } else {
                r.replace('\\', "/")
            }
        })
        .collect()
}

/// The thread-local shared resolve index for `source_roots` (union-resolve S1). Built once
/// per (thread, canonical roots) and reused, so consumers that resolve distinct entries
/// against it share one typed_module_cache — the union closure typechecks once per node.
/// Roots are canonicalized (`canonical_shared_index_roots`) before both keying and
/// building, so path-spelling variants of the same pool cannot fork the INDEX.
/// This does not canonicalize independently-read `SourceFile` objects: a consumer
/// that joins absolute-path reads to this relative-path index can still fork source
/// identity. The divergence census walls that site with parent-owned `Rc` identity;
/// the class-wide next rung is canonical `SourceFile` identity at construction.
#[track_caller]
pub fn process_shared_index(source_roots: &[String]) -> Rc<MultiEntryIndex> {
    try_process_shared_index(source_roots).unwrap_or_else(|e| panic!("{e}"))
}

#[cfg(test)]
type LivePoolJob = Box<dyn FnOnce() + Send>;

/// The live-pool thread's inbox and handle, `None` while no thread is alive. Senders hold this lock
/// while they send, and every path that ends the thread takes it first, so a claim is never sent to
/// a thread that is already leaving.
#[cfg(test)]
static LIVE_POOL_THREAD: Mutex<
    Option<(
        std::sync::mpsc::Sender<LivePoolJob>,
        std::thread::JoinHandle<()>,
    )>,
> = Mutex::new(None);

/// Set while the live-pool thread runs a claim. A pool built by a thread the claim itself spawned
/// is that claim's business and must not wait on the thread that is running it.
#[cfg(test)]
static LIVE_POOL_BUSY: AtomicBool = AtomicBool::new(false);

/// A LEAK BACKSTOP, not the release. The pool is released by the demand that conflicts with it
/// (`yield_live_pool_before_building_another`). This timer only bounds how long it can outlive its
/// block when no later test builds a pool of its own, so it never decides what a claim answers and
/// the bound does not depend on it for any test that builds one.
#[cfg(test)]
const LIVE_POOL_IDLE_BACKSTOP: std::time::Duration = std::time::Duration::from_secs(10);

#[cfg(test)]
fn spawn_live_pool_thread() -> (
    std::sync::mpsc::Sender<LivePoolJob>,
    std::thread::JoinHandle<()>,
) {
    use std::sync::mpsc::RecvTimeoutError;
    let (tx, rx) = std::sync::mpsc::channel::<LivePoolJob>();
    let handle = std::thread::Builder::new()
        .name("live-pool".to_string())
        // The largest stack any live-pool claim spawned for itself before it ran here.
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            loop {
                match rx.recv_timeout(LIVE_POOL_IDLE_BACKSTOP) {
                    Ok(job) => {
                        LIVE_POOL_BUSY.store(true, Ordering::SeqCst);
                        job();
                        LIVE_POOL_BUSY.store(false, Ordering::SeqCst);
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        let mut slot = LIVE_POOL_THREAD
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        if let Ok(job) = rx.try_recv() {
                            drop(slot);
                            LIVE_POOL_BUSY.store(true, Ordering::SeqCst);
                            job();
                            LIVE_POOL_BUSY.store(false, Ordering::SeqCst);
                            continue;
                        }
                        // Detach: nobody joins a thread that leaves on its own.
                        *slot = None;
                        break;
                    }
                    // The inbox was taken by `yield_live_pool_before_building_another`, which joins.
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            // Drop the pool now and hand the freed heap back, rather than leave it for the
            // thread-local destructors after the trim could run.
            reset_process_shared_index_for_test();
            trim_retained_heap();
        })
        .expect("the live-pool thread starts");
    (tx, handle)
}

/// THE PROCESS HOLDS ONE LIVE WHOLE-POOL INDEX AT A TIME. Called by every whole-pool index build
/// (`try_process_shared_index_for_pool`, `build_multi_entry_index`): before any other thread builds
/// one, the live-pool thread's pool is released -- the thread exits, which drops its index and
/// every thread-local cache built over it, and the freed heap is handed back. This is the release,
/// tied to the demand that conflicts with the pool rather than to a clock (measured under the
/// hosted runner's 12 GiB `memory.max`: the ~5.2 GiB pool held past its block, plus the next test's
/// own index, was OOM-killed at the bound). A claim's own nested builds are exempt: the live-pool
/// thread itself, and any thread a running claim spawned (`LIVE_POOL_BUSY`).
#[cfg(test)]
pub(crate) fn yield_live_pool_before_building_another() {
    if std::thread::current().name() == Some("live-pool") || LIVE_POOL_BUSY.load(Ordering::SeqCst) {
        return;
    }
    let taken = LIVE_POOL_THREAD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take();
    let Some((sender, handle)) = taken else {
        return;
    };
    drop(sender);
    handle.join().expect("the live-pool thread leaves cleanly");
    trim_retained_heap();
}

/// THE LIVE POOL'S ONE HOLDER UNDER TEST. `process_shared_index` is per-thread because the index
/// is `Rc`-based, and libtest runs every test on a fresh thread -- so every claim that resolves an
/// entry over the live `[dag, src/v2]` pool rebuilt the whole-pool index and re-typechecked the
/// shared prefix, the same fact once per claim with its least common ancestor at the PROCESS
/// (DESIGN §2 demand minimization; gunbc#12450 measured ~18 such claims at ~90s each). The
/// entry-scoped loader cannot stand in: bare references resolve through a census of the whole pool
/// (`admit_pool_bare_references`), so no entry resolves without it.
///
/// So the fact moves to its ancestor: one thread holds the index in ITS thread-local slot for as
/// long as claims keep arriving, and a live-pool claim runs its body there. Nothing about
/// resolution changes -- the same `process_shared_index` builds the same pool on first demand --
/// only how many times. `RUST_TEST_THREADS=1` (`.cargo/config.toml`) already serializes the suite,
/// so the queue costs no parallelism. The pool is released when another thread builds one
/// (`yield_live_pool_before_building_another`). A panic in the body is caught on the thread and re-raised on the
/// calling test's thread with its original payload, so a failing claim reds exactly as before and
/// the thread survives for the next one.
#[cfg(test)]
pub(crate) fn on_live_pool_thread<T: Send + 'static>(
    body: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let job: LivePoolJob = Box::new(move || {
        // Freed-but-retained heap from the tests that ran on their own threads before this claim
        // is returned first: those threads' glibc arenas are not the one this long-lived thread
        // allocates from, so without the trim the pool is built ON TOP of their residue (measured
        // under the hosted runner's 12 GiB `memory.max`: 7.6 GiB retained before the first claim,
        // OOM-killed at the bound as the pool was built; 1.4 GiB with the trim).
        trim_retained_heap();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        release_per_entry_graphs_on_live_pool_thread();
        trim_retained_heap();
        let _ = done_tx.send(outcome);
    });
    {
        let mut slot = LIVE_POOL_THREAD
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        slot.get_or_insert_with(spawn_live_pool_thread)
            .0
            .send(job)
            .expect("the live-pool thread accepts work");
    }
    match done_rx.recv().expect("the live-pool thread answers") {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// What the live-pool thread keeps between claims, and what it drops. The shared FACT is the
/// pool: its module index, graph facts, heads parse, censuses and the capped typed-module cache,
/// which every claim's entry reuses. A claim's own resolved entry graphs are not shared -- no
/// later claim asks for the same entry -- so holding them would only grow the thread's resident
/// set claim by claim (measured: 39 live-pool claims peaked at 10.6 GiB with them held, against a
/// 12 GiB `memory.max` on the hosted runner). They are dropped after every claim.
#[cfg(test)]
fn release_per_entry_graphs_on_live_pool_thread() {
    PROCESS_RESOLVE_STORE.with(|s| s.borrow_mut().clear());
    PROCESS_RESOLVE_INDEX.with(|slots| {
        for index in slots.borrow().iter().flat_map(|slot| slot.values()) {
            clear_resolved_graph_memo_for_test(index);
        }
    });
}

/// AT MOST ONE RESIDENT POOL PER SLOT, the positive control of the build-site refusal in
/// `try_process_shared_index_for_pool`. That refusal makes a second pool unbuildable through the
/// memo; this reads the memo back at the end of the floor and of a regen round, so a route that
/// ever installed one another way would still refuse rather than print.
pub(crate) fn shared_index_residency_control() -> Result<usize, String> {
    PROCESS_RESOLVE_INDEX.with(|s| {
        let slots = s.borrow();
        for (slot, pools) in slots.iter().enumerate() {
            if pools.len() > 1 {
                let roots: Vec<Vec<&str>> =
                    pools.keys().map(|k| k.split('\u{1f}').collect()).collect();
                return Err(format!(
                    "SharedIndexMoreThanOneResidentPool: slot {slot} holds {} pools {roots:?} -- \
                     a one-shot reader of another pool must own its index, not leave it in the \
                     shared memo",
                    pools.len()
                ));
            }
        }
        Ok(slots.iter().map(|pools| pools.len()).sum())
    })
}

/// The strict-pool shared index for `source_roots` IF this thread already built it; never builds
/// one. For readers that report on a resolve after the fact (`pre_entry_phase`), where building
/// would repeat the discovery a refused resolve just failed.
pub(crate) fn memoized_process_shared_index(
    source_roots: &[String],
) -> Option<Rc<MultiEntryIndex>> {
    let roots_key = canonical_shared_index_roots(source_roots).join("\u{1f}");
    PROCESS_RESOLVE_INDEX.with(|s| s.borrow()[0].get(&roots_key).cloned())
}

/// THE STRICT INDEX FOR A READER WHOSE ROOTS ARE A BUILTIN'S ARGUMENT. Over the layer roots
/// (`witness_layer_roots`) it is the run's pool and is read from the shared memo; over any other
/// roots -- a fixture a witness names -- it is a one-shot read of a pool nothing else reads, so
/// the reader owns the index and drops it with the read. The same split
/// `emit_host::resolve_roots_for_call_edge_pool` makes; it keeps a fixture out of the shared
/// memo, where `try_process_shared_index_for_pool` refuses a second pool.
#[track_caller]
pub(crate) fn try_index_for_run_or_owned_pool(
    source_roots: &[String],
) -> Result<Rc<MultiEntryIndex>, String> {
    let layers = super::witness_layer_roots();
    if !source_roots.is_empty()
        && source_roots
            .iter()
            .all(|root| layers.iter().any(|layer| layer == root))
    {
        return try_process_shared_index(source_roots);
    }
    let roots = canonical_shared_index_roots(source_roots);
    Ok(Rc::new(new_multi_entry_index_shell(
        try_build_module_index(&roots)?,
        &roots,
        None,
    )))
}

/// Fallible twin of `process_shared_index`. The MEMO IS ONLY WRITTEN ON SUCCESS -- a failed
/// discovery must not install a partial index that every later caller in the process would
/// then read as complete.
#[track_caller]
pub fn try_process_shared_index(source_roots: &[String]) -> Result<Rc<MultiEntryIndex>, String> {
    try_process_shared_index_for_pool(source_roots, false)
}

/// The shared index for `source_roots` under a NAMED POOL SEMANTICS.
///
/// WHY THIS EXISTS, and it is a §2 cost-shape repair rather than a new capability. The compile
/// transaction's two subjects took opposite routes over the same caches: the strict arm resolved
/// through the process-shared index above -- so a second `--entry` compile in one process is a
/// cache hit -- while the primary-precedence arm built a FRESH `MultiEntryIndex` per call and
/// threw it away. Every cache that index owns is per-call under that arm: the parse cache, the
/// typed-module cache, the pool census, the interned names. So a run that compiles N entries
/// typechecks the shared prefix N times, and the prefix is nearly the whole closure -- most of
/// `dag/std` sits in almost every entry's closure. The dominant cost of the emit-compile phase
/// (`compile.reconcile`) is therefore paid INDEPENDENTLY PER ENTRY over closures that overlap
/// almost entirely, which is why a per-entry cover cannot reach the corpus at any budget: the
/// unit of computation was the closure and the unit of fact was the entry.
///
/// WHAT CHANGES AND WHAT DOES NOT. Only WHICH index the primary arm reaches; the index's own
/// semantics are untouched -- it is still built by `try_build_module_index_primary_precedence`
/// over the same canonicalized roots, so the pool it presents is the same pool. The typed cache
/// is keyed by authored name and content (`typed_module_content_key`), and the collision-honesty
/// guard in `reconcile_with_typed_cache` already refuses loudly when one name resolves from two
/// declaring files across co-resident entries -- which is exactly the co-residence this memo
/// creates more of, so the wall is upstream of the change rather than owed by it.
///
/// PRECEDENCE IS PART OF THE SLOT IDENTITY, never a parameter applied to a shared slot: see the
/// two-slot note on `PROCESS_RESOLVE_INDEX`.
#[track_caller]
pub fn try_process_shared_index_for_pool(
    source_roots: &[String],
    primary_precedence: bool,
) -> Result<Rc<MultiEntryIndex>, String> {
    let slot = usize::from(primary_precedence);
    let roots = canonical_shared_index_roots(source_roots);
    let roots_key = roots.join("\u{1f}");
    let existing = PROCESS_RESOLVE_INDEX.with(|s| s.borrow()[slot].get(&roots_key).cloned());
    if let Some(idx) = existing {
        return Ok(idx);
    }
    // A SECOND POOL IS REFUSED WHERE IT WOULD BE BUILT, before it is walked, parsed and held
    // beside the first. Every production caller passes the run's own roots or owns a private
    // index for a pool nothing else reads (the audit that retired
    // `gunbc.rung_drop.shared_index_residency_asserted_after_the_run`), so a different key here
    // is a new demand for a second resident pool: carry it to its own index
    // (`build_multi_entry_index`), never into the shared memo.
    let resident = PROCESS_RESOLVE_INDEX.with(|s| s.borrow()[slot].keys().next().cloned());
    if let Some(resident) = resident {
        let site = std::panic::Location::caller();
        return Err(format!(
            "SharedIndexSecondResidentPool: slot {slot} already holds roots {:?}; roots {roots:?} \
             demanded at {}:{} would be a second resident pool on this thread -- a reader of \
             another pool owns its index (build_multi_entry_index), not a slot in the shared memo",
            resident.split('\u{1f}').collect::<Vec<_>>(),
            site.file(),
            site.line()
        ));
    }
    #[cfg(test)]
    yield_live_pool_before_building_another();
    let build_started = std::time::Instant::now();
    let walk_started = std::time::Instant::now();
    let module_index = if primary_precedence {
        try_build_module_index_primary_precedence(&roots)?
    } else {
        try_build_module_index(&roots)?
    };
    super::pre_entry_phase::record(
        "source_root_walk_and_read",
        super::pre_entry_phase::PhaseScale::Tree,
        walk_started.elapsed(),
    );
    let idx = Rc::new(new_multi_entry_index_shell(module_index, &roots, None));
    discovery_phase_totals::add(
        &discovery_phase_totals::SHARED_INDEX_BUILD_MS,
        build_started.elapsed(),
    );
    PROCESS_RESOLVE_INDEX.with(|s| {
        s.borrow_mut()[slot].insert(roots_key, idx.clone());
    });
    Ok(idx)
}

/// WHAT THE PROCESS-LIFETIME RESOLVE STORE HOLDS, as counts, for the floor's seam readings: the
/// number of resolved entry graphs kept, and per shared-index slot its parse, typed-module and
/// resolved-graph memo sizes. A seam that shows heap still in use after a step whose result is
/// small needs to know whether these stores grew, and whether a later step ever reads them --
/// the first says retention, the second says whether it is useful cache or dead weight. Counts,
/// not bytes: an entry's size is not knowable without walking it, and a walk would perturb the
/// run. Read-only.
pub(crate) fn process_resolve_census() -> String {
    let store = PROCESS_RESOLVE_STORE.with(|s| s.borrow().len());
    let slots = PROCESS_RESOLVE_INDEX.with(|s| {
        s.borrow()
            .iter()
            .map(|slot| {
                if slot.is_empty() {
                    return "empty".to_string();
                }
                slot.values()
                    .map(|idx| {
                        format!(
                            "gen{}:parse={}:typed={}:graphs={}",
                            idx.generation,
                            idx.parse_cache.borrow().len(),
                            idx.typed_module_cache.borrow().len(),
                            idx.resolved_graph_memo.borrow().len()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .collect::<Vec<_>>()
            .join(",")
    });
    format!("resolve_store_entries={store} shared_index=[{slots}]")
}

pub fn resolve_entry_graph_shared(
    source_roots: &[String],
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let key = (source_roots.join("\u{1f}"), entry_file.to_string());
    let hit = PROCESS_RESOLVE_STORE.with(|s| s.borrow().get(&key).cloned());
    if let Some(found) = hit {
        return Ok(found);
    }
    // Resolve through the thread's shared index instead of a fresh per-call module index.
    // resolve_entry_with_index is proven behaviorally identical to the cold resolve_entry_graph
    // by resolve_typed_cache_equivalence_test (cached == cold across every resolve order); the
    // win is that the union of all fixed-entry closures now typechecks once per node.
    let index = process_shared_index(source_roots);
    let resolved = resolve_entry_with_index(&index, entry_file)?;
    PROCESS_RESOLVE_STORE.with(|s| {
        s.borrow_mut().insert(key, resolved.clone());
    });
    Ok(resolved)
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn typed_module_cache_len_for_test(index: &MultiEntryIndex) -> usize {
    index.typed_module_cache.borrow().len()
}

/// Test-only projection of the durable typed-cache authority: the content keys whose
/// computations populated this private index. A fresh, non-evicting test index makes
/// this exactly the distinct-computation set without retaining request attribution.
#[cfg(any(test, feature = "interp_test_witness"))]
pub fn typed_module_cache_content_keys_for_test(
    index: &MultiEntryIndex,
) -> std::collections::BTreeSet<String> {
    index.typed_module_cache.borrow().keys().cloned().collect()
}

/// Test witness for the sample-once cap: exposes the same accessor runtime
/// call sites use, so a test can observe that repeated calls against one
/// `index` return the identical value even as the underlying signal moves —
/// the property the 2026-07-21 fleet OOM fix depends on.
#[cfg(any(test, feature = "interp_test_witness"))]
pub fn typed_module_cache_cap_for_test(index: &MultiEntryIndex) -> usize {
    typed_module_cache_cap(index)
}

/// Keys currently in `resolved_graph_memo`. The harness diffs this across one resolve to
/// recover that entry's subject key, which is what `index_schedule_entry_completed` needs
/// to drop the entry's assembled graph — recovered by observation rather than by
/// re-deriving the subject digest in a second place.
#[cfg(any(test, feature = "interp_test_witness"))]
pub fn resolved_graph_memo_keys_for_test(index: &MultiEntryIndex) -> Vec<String> {
    index.resolved_graph_memo.borrow().keys().cloned().collect()
}

#[track_caller]
pub(crate) fn new_multi_entry_index_shell(
    source_files: ModuleSourceIndex,
    source_roots: &[String],
    cross_worker_store: Option<Arc<RwLock<SharedTypecheckCaches>>>,
) -> MultiEntryIndex {
    record_multi_entry_index_site(std::panic::Location::caller(), &source_files);
    MultiEntryIndex {
        generation: next_index_generation(),
        source_files,
        module_graph_facts: build_module_graph_facts_live(source_roots),
        typed_module_cache: RefCell::new(std::collections::HashMap::new()),
        typed_cache_evictions: Cell::new(0),
        typed_cache_evicted_keys: RefCell::new(std::collections::HashSet::new()),
        typed_cache_readmissions: Cell::new(0),
        memory_stall_window: RefCell::new(None),
        typed_module_cache_cap: std::cell::OnceCell::new(),
        source_hash_by_file: RefCell::new(std::collections::HashMap::new()),
        module_source_identity: RefCell::new(std::collections::HashMap::new()),
        cross_worker_store,
        intern_table: RefCell::new(seed_kernel_intern_names(empty_intern_table())),
        parse_cache: RefCell::new(std::collections::HashMap::new()),
        normalize_diag_cache: RefCell::new(std::collections::HashMap::new()),
        ownership_diag_cache: RefCell::new(std::collections::HashMap::new()),
        resolved_graph_memo: RefCell::new(HashMap::new()),
        schedule_retention: RefCell::new(None),
        source_roots: source_roots.to_vec(),
        pool_parse: RefCell::new(None),
        reference_pool_names: RefCell::new(None),
        pool_qualified_fill: RefCell::new(None),
        tree_bare_census: RefCell::new(std::collections::HashMap::new()),
        #[cfg(any(test, feature = "interp_test_witness"))]
        pool_bare_census: RefCell::new(None),
        entry_closure_sources: RefCell::new(HashMap::new()),
        both_closure_edges: RefCell::new(None),
        closure_name_censuses: RefCell::new(HashMap::new()),
        bare_reference_admission: RefCell::new(HashMap::new()),
        pool_module_names: std::cell::OnceCell::new(),
        pool_path_lookup: std::cell::OnceCell::new(),
        reference_reading_parses: std::cell::Cell::new(0),
        parsed_references: RefCell::new(HashMap::new()),
        live_read_manifest: RefCell::new(None),
    }
}

pub(crate) fn typed_module_content_key(
    index: &MultiEntryIndex,
    resolved: &Rc<v1_compiler_resolve::ResolvedModule>,
    mod_name: &str,
    interface_hash_by_name: &std::collections::HashMap<String, String>,
    closure_names: &std::collections::HashSet<&str>,
    closure_path_to_authored_name: &HashMap<String, &str>,
    include_reference_derived_term: bool,
) -> Result<String, String> {
    let file = &resolved.module.span.file;
    let source_hash = index
        .source_hash_by_file
        .borrow()
        .get(file)
        .cloned()
        .ok_or_else(|| {
            format!(
                "typed-module content key refused: no source hash recorded for '{file}' \
                 (module '{mod_name}') — every reconciled module must pass the parse loop \
                 in this process before its typed result is keyed"
            )
        })?;
    let mut import_hashes: im::Vector<String> = im::Vector::new();
    for import in resolved.resolved_imports.iter() {
        let hash = interface_hash_by_name
            .get(&import.module_path)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "typed-module content key refused: direct import '{}' of module \
                     '{mod_name}' has no interface hash yet — imports must be typechecked \
                     (or cache-served) before their dependents are keyed",
                    import.module_path
                )
            })?;
        import_hashes.push_back(hash);
    }
    // Stripped (no `import` line) modules resolve their dependencies through the corpus-wide
    // bare-name census instead — real cross-module dependencies that `resolved_imports` above
    // cannot see (PR #6848, namespace wave 1: 815/2301 modules). Without this term a stripped
    // dependent's content key is invariant under a provider's export-surface change, which is
    // cache impurity (DESIGN §5/recurring-failure-modes: key on declared-input content). The
    // reference-only targets come from the SAME `selection_adjacency` authority affected-set
    // selection already consumes (DESIGN §3: no second edge producer), strict-tier only
    // (Qualified + UniqueBare — an AmbiguousBare homonym is not a declared dependency).
    let file_rel = workspace_relative_repo_path(file);
    if include_reference_derived_term {
        for dep_path in index
            .module_graph_facts
            .reference_only_direct_import_paths(&file_rel)
        {
            // The loader is deliberately import-only (build_module_graph_facts_live_uncached):
            // selection_adjacency's reference-derived edges are census-wide and tuned for the
            // affected-set consumer's tolerance of an over-connected false positive, not as a
            // hard dependency authority. A target this resolve's closure never loaded cannot
            // have influenced this compile's typed result, so it is not a term in this key —
            // mirroring module_schedule_batches's existing dangling-edge tolerance, not an
            // absorbing fallback (DESIGN §5): the exclusion is structurally forced, not a
            // substitute for a precision we failed to compute.
            let Some(dep_authored) = closure_path_to_authored_name.get(dep_path.as_str()) else {
                continue;
            };
            if !closure_names.contains(dep_authored) {
                continue;
            }
            let hash = interface_hash_by_name
                .get(*dep_authored)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "typed-module content key refused: reference-derived dependency \
                     '{dep_authored}' (path '{dep_path}') of stripped module '{mod_name}' has \
                     no interface hash yet — its bare-name dependencies must be typechecked \
                     (or cache-served) before it is keyed"
                    )
                })?;
            import_hashes.push_back(hash);
        }
    }
    fn structural_from_wire(hex: String) -> Rc<crate::std_content_hash::Fnv1a64Structural> {
        fnv1a64_structural_hex_digest(hex).unwrap_or_else(|| {
            panic!("typed-module content key refused: reconcile digest is not a valid fnv1a64 structural wire form")
        })
    }
    Ok(typed_module_key(
        module_key(
            structural_from_wire(source_hash),
            Rc::new(
                import_hashes
                    .iter()
                    .cloned()
                    .map(|h| structural_from_wire(h))
                    .collect(),
            ),
        ),
        structural_from_wire(transform_content_digest()),
    )
    .digest
    .clone())
}

/// One derivation of the typed-cache entry cap: env override, else the host
/// budget divided by the per-entry estimate. Returns `(cap, source_label,
/// degraded)` — `degraded` is true exactly when the budget did not come from
/// a private cgroup `memory.max`/`memory.high` (i.e. it fell through to the
/// host-wide `MemAvailable`/`MemTotal` last resort, or no budget was found at
/// all and the ceiling was used). Pure and side-effect-free: callers decide
/// how often to invoke it and whether to log the degraded case.
pub(crate) fn typed_module_cache_cap_derivation() -> (usize, String, bool) {
    if let Ok(raw) = std::env::var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES") {
        if let Ok(n) = raw.trim().parse::<usize>() {
            if n > 0 {
                return (
                    n.min(TYPED_MODULE_CACHE_MAX_ENTRIES_CEIL),
                    "env override GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES".to_string(),
                    false,
                );
            }
            // Zero override is invalid — fall through to derived cap.
        }
        // Malformed override — fall through to derived cap (fail-closed).
    }
    let resolution = crate::memory_governor::read_host_budget_resolution();
    let source_label = resolution.label();
    // GROUNDED ON THE DISCRIMINANT, not on the rendering. This read
    // `!(source_label.contains("memory.max") || source_label.contains("memory.high"))` and the
    // comment beside it named the fix it was waiting for: "if it ever grows a typed source
    // enum, ground this check on the enum instead of re-parsing its display label (§3, avoid
    // a second representation)". It has one, so this does. The scan was not merely
    // stylistically wrong — it graded the operator's own GUNBC_MEMORY_BUDGET_BYTES override
    // as degraded, because that label mentions no cgroup file.
    //
    // An unreadable budget has no source, so `degraded_source` is `None` there and the
    // question is not answered with a boolean; the refusal below runs first regardless.
    let degraded = resolution.degraded_source().unwrap_or(true);
    let budget = resolution.bytes();
    // REFUSE rather than widen when no budget is readable (operator ruling 2026-08-05;
    // authority `dag/gunbc/host/host_budget_source.dag` `HostBudgetUnreadable`).
    //
    // This was `.unwrap_or(TYPED_MODULE_CACHE_MAX_ENTRIES_CEIL)`: a budget that could not
    // be computed became the MOST PERMISSIVE cap available — top-as-answer conflated with
    // top-as-ignorance, the absorbing fallback DESIGN section 5 forbids by name. It is not
    // hypothetical. It OOM-killed the full witness corpus twice on a macOS dev machine
    // (exit 137): nothing readable, cap defaults to the ceiling, nothing bounds the resolve,
    // kernel ends the process. The deficit's frequency was zero by construction, so it never
    // ranked for fixing, and the cost arrived as a dead process instead of a diagnostic.
    //
    // Reaching this arm means the host declared no bound this process can read: no cgroup
    // memory.high or memory.max, no operator override, and no Darwin physical-memory read.
    // That is the ordinary state on a remote-execution runner whose slot cap lives outside
    // the container's cgroup namespace (BuildBuddy, measured 2026-08-30), and it used to be
    // answered with the machine's MemAvailable capped at this fleet's own declared slot line
    // — two substitutions stacked, and an rc=137 SIGKILL of `main_wet` with no diagnostic.
    // An environment declaration cannot replace that missing observation: it constrains no
    // allocation. Panicking here is a hard
    // stop by design: this runs inside resolution, there is no caller that could honour a
    // typed refusal without threading Result through the cache seam, and continuing is the
    // one option ruled out.
    let Some(budget_bytes) = budget else {
        panic!(
            "HostBudgetUnreadable: no modeled host memory source answered ({source_label}). \
             The typed-module cache cap bounds the memory used to RESOLVE the corpus, so an \
             unknown budget cannot be defaulted — the previous default was the ceiling, which \
             OOM-killed this process rather than refusing, and the MemAvailable arm that \
             replaced it substituted the MACHINE's memory for this slot's and was SIGKILLed \
             at rc=137 instead. Configure the executor to expose a cgroup memory limit; \
             GUNBC_MEMORY_BUDGET_BYTES may only request a lower planning ceiling \
             (dag/gunbc/host/host_budget_source.dag)."
        );
    };
    let cap = ((budget_bytes / TYPED_MODULE_BYTES_PER_ENTRY_ESTIMATE) as usize).clamp(
        TYPED_MODULE_CACHE_MAX_ENTRIES_FLOOR,
        TYPED_MODULE_CACHE_MAX_ENTRIES_CEIL,
    );
    (cap, source_label, degraded)
}

/// The typed-cache cap for `index`, sampled exactly ONCE for this index's
/// lifetime (a run-start fact, never re-read per insert). On first call, if
/// the budget source is degraded — a real reading, but of the MACHINE rather
/// than of a bound on this process — emits a typed, counted
/// `[floor-drain] degraded_budget_source` diagnostic. An honesty arm, not a
/// widened failure: the cap derives from a source the platform genuinely has,
/// it is simply named so the degraded case is observable and prioritizable
/// rather than silent.
///
/// Since the meminfo arms were deleted this is reachable only where the kernel
/// has no private-limit mechanism at all (Darwin, `sysctl hw.memsize`). On a
/// kernel that HAS cgroups, a missing limit is a missing bound and the
/// derivation refuses instead of degrading — that path used to answer with the
/// host's MemAvailable and got `main_wet` SIGKILLed at rc=137.
pub(crate) fn typed_module_cache_cap(index: &MultiEntryIndex) -> usize {
    *index.typed_module_cache_cap.get_or_init(|| {
        let (cap, source, degraded) = typed_module_cache_cap_derivation();
        if degraded {
            eprintln!(
                "[floor-drain] degraded_budget_source: cap={cap} source={source} \
                 (this kernel has no private-limit mechanism, so the reading is host-shared)"
            );
        }
        cap
    })
}

pub fn resolve_entry_with_index(
    index: &MultiEntryIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    resolve_entry_with_parse_cache(index, entry_file)
}

/// Cumulative per-worker stage attribution across every entry resolve this thread
/// has run (the per-entry slot folded in at each reset, plus the live slot). Read by
/// `claim_batch`'s `[assembly-split]` receipt, which — unlike `claim_executor`'s
/// discovery summary — has no per-entry receipt list to sum.
pub fn resolve_stage_totals() -> ResolveStageNanos {
    let mut total = RESOLVE_STAGE_TOTAL.with(|t| *t.borrow());
    total.accumulate(&resolve_stage_slot_snapshot());
    total
}

pub(crate) fn resolve_stage_slot_reset() {
    let carried = resolve_stage_slot_snapshot();
    RESOLVE_STAGE_TOTAL.with(|t| t.borrow_mut().accumulate(&carried));
    RESOLVE_STAGE_SLOT.with(|s| s.set(ResolveStageNanos::default()));
}

pub(crate) fn resolve_stage_slot_add(update: impl FnOnce(&mut ResolveStageNanos)) {
    RESOLVE_STAGE_SLOT.with(|s| {
        let mut v = s.get();
        update(&mut v);
        s.set(v);
    });
}

pub(crate) fn resolve_stage_slot_snapshot() -> ResolveStageNanos {
    RESOLVE_STAGE_SLOT.with(|s| s.get())
}

/// Per-entry stage rows for this thread.
pub fn resolve_stage_rows_by_entry() -> HashMap<String, ResolveStageNanos> {
    RESOLVE_STAGE_BY_ENTRY.with(|m| m.borrow().clone())
}

/// Cumulative span account for this thread.
pub fn resolve_span_account() -> ResolveSpanAccount {
    RESOLVE_SPAN_ACCOUNT.with(|s| s.get())
}

/// Per-entry span rows for this thread, descending by summed nanos.
pub fn resolve_span_rows_by_entry() -> Vec<(String, u64, u128, ResolveStageNanos)> {
    let stages = resolve_stage_rows_by_entry();
    let mut rows: Vec<(String, u64, u128, ResolveStageNanos)> = RESOLVE_SPAN_BY_ENTRY.with(|m| {
        m.borrow()
            .iter()
            .map(|(k, (n, ns))| {
                (
                    k.clone(),
                    *n,
                    *ns,
                    stages.get(k).copied().unwrap_or_default(),
                )
            })
            .collect()
    });
    rows.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    rows
}

pub(crate) fn resolve_span_enter() -> u32 {
    RESOLVE_SPAN_ACCOUNT.with(|s| {
        let mut v = s.get();
        v.depth += 1;
        if v.depth > 1 {
            v.nested_spans += 1;
        }
        s.set(v);
        v.depth
    })
}

pub(crate) fn resolve_span_exit(depth: u32, entry_file: &str, elapsed_nanos: u128) {
    RESOLVE_SPAN_ACCOUNT.with(|s| {
        let mut v = s.get();
        v.depth = v.depth.saturating_sub(1);
        // Only top-level spans are summed: a nested span's time is already inside its
        // parent's, so adding it would be the double-count this account exists to expose.
        if depth == 1 {
            v.span_nanos += elapsed_nanos;
            v.spans += 1;
        }
        s.set(v);
    });
    if depth == 1 {
        let key = workspace_relative_entry_path(entry_file);
        RESOLVE_SPAN_BY_ENTRY.with(|m| {
            let mut map = m.borrow_mut();
            let slot = map.entry(key.clone()).or_insert((0, 0));
            slot.0 += 1;
            slot.1 += elapsed_nanos;
        });
        // The slot was reset at span entry and nothing has reset it since, so it holds
        // exactly this entry's rows.
        let this_entry = resolve_stage_slot_snapshot();
        RESOLVE_STAGE_BY_ENTRY.with(|m| {
            m.borrow_mut()
                .entry(key.clone())
                .or_default()
                .accumulate(&this_entry);
        });
    }
}

pub(crate) fn resolve_entry_with_parse_cache(
    index: &MultiEntryIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let depth = resolve_span_enter();
    let span_started = std::time::Instant::now();
    let out = resolve_entry_with_parse_cache_inner(index, entry_file);
    resolve_span_exit(depth, entry_file, span_started.elapsed().as_nanos());
    out
}

pub(crate) fn resolve_entry_with_parse_cache_inner(
    index: &MultiEntryIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    resolve_stage_slot_reset();
    set_phase(FloorPhase::Resolve, entry_file);
    let (sources, load_nanos) =
        nanos_net_of_pool_parse(|| load_sources_for_entry_with_pool(index, entry_file));
    let sources = sources?;
    resolve_stage_slot_add(|s| s.load += load_nanos);
    resolved_graph_from_sources_with_index(
        index,
        sources,
        entry_file,
        ResolvedGraphMemoShare::Memoize,
    )
    .map(|(graph, si, _compile_clean_diags)| (graph, si))
}

pub(crate) fn via_index_source_annotation_diagnostics(
    source: &v1_compiler_compile::SourceFile,
    occurrence_transport: Rc<crate::std_occurrence_identity::OccurrenceTransport>,
    captures: Rc<im::Vector<Rc<crate::std_source_annotation::UnboundAnnotationCapture>>>,
) -> im::Vector<Rc<ErrorNode>> {
    let bound = v1_compiler_compile::admit_source_annotations(
        occurrence_transport,
        captures,
        v1_rt::string_length(&source.content),
    );
    bound
        .diagnostics
        .iter()
        .cloned()
        .map(|d| make_error_node(d, source.path.clone()))
        .collect()
}

/// Census-fill admission for a parse_cache hit that never ran `admit_source_annotations`.
/// Filters to `SourceAnnotationRefused` because that is all `admit_source_annotations`
/// emits today (both sites in `v1_compiler_annotation_bind`). The miss path keeps
/// every diagnostic that function returns; the two agree until a third diagnostic
/// appears — latent coupling, not a live divergence.
pub(crate) fn via_index_census_fill_annotation_diags(
    source: &Rc<v1_compiler_compile::SourceFile>,
) -> im::Vector<Rc<ErrorNode>> {
    let fill = v1_compiler_compile::parse_census_fill_sources(Rc::new(vec![source.clone()].into()));
    fill.diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.diagnostic.as_ref(),
                CompilerDiagnostic::SourceAnnotationRefused { .. }
            )
        })
        .cloned()
        .collect()
}

/// Parse one via-index source, admitting annotations into the cache entry so a later
/// `Memoize` consumer (`handle_serve`) cannot observe a different population than a
/// cold parse of the same bytes.
pub(crate) fn via_index_parse_one_source(
    index: &MultiEntryIndex,
    source: &Rc<v1_compiler_compile::SourceFile>,
) -> ParseCacheEntry {
    let cached = index.parse_cache.borrow().get(&source.path).cloned();
    if let Some(entry) = cached {
        if entry.annotation_diags.is_some() {
            return entry;
        }
        let refused = via_index_census_fill_annotation_diags(source);
        let upgraded = ParseCacheEntry {
            annotation_diags: Some(Rc::new(refused)),
            ..entry
        };
        index
            .parse_cache
            .borrow_mut()
            .insert(source.path.clone(), upgraded.clone());
        return upgraded;
    }
    // Ordinary frontend (`front_end_sources`) keeps tokenize_artifact
    // captures and admits them against this file's occurrence transport.
    // Annotation-erasing `tokenize` here let a touched in-closure file
    // compile on the floor while missing the class #8204 claims to close.
    // One acquisition, not one per walk -- see `cli_run::pool_acquire`. The pool census already
    // tokenized these bytes under this spelling; the artifact keeps the annotation channel.
    let artifact = super::pool_acquire::artifact_for(&source.path, &source.content);
    let nl_index = super::pool_acquire::newline_index_for(&source.path, &source.content);
    let current_table = index.intern_table.borrow().clone();
    let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
        let mut m = HashMap::new();
        m.insert(source.path.clone(), nl_index.clone());
        m
    });
    let parsed =
        v1_compiler_parse::parse_with_table(artifact.tokens.clone(), single_si, current_table);
    *index.intern_table.borrow_mut() = parsed.intern_table.clone();
    let annotation_diags = via_index_source_annotation_diagnostics(
        source,
        parsed.occurrence_transport.clone(),
        artifact.annotations.clone(),
    );
    let entry = ParseCacheEntry {
        parse_result: parsed.result.clone(),
        newline_index: nl_index,
        annotation_diags: Some(Rc::new(annotation_diags)),
    };
    index
        .parse_cache
        .borrow_mut()
        .insert(source.path.clone(), entry.clone());
    entry
}

/// The sources-taking core of `resolve_entry_with_parse_cache`: parse → resolve →
/// normalize → `reconcile_with_typed_cache` → ownership, every stage through the
/// index's per-module memo tiers (parse/normalize/typed/ownership caches + the
/// resolved-graph subject memo). Extracted so a whole-tree SOURCE SET — the
/// compile-clean gate's closure, which has no single entry file — rides the same
/// cached path as entry-file resolves: one process, ONE typecheck universe, so the
/// floor's gate compile and batch-2's witness resolves share every module's
/// content-keyed typecheck instead of double-paying it (typecheck investigation,
/// PR #6766).
///
/// Failure semantics: collect-then-refuse per stage — a stage gathers ALL of its
/// diagnostics before refusing (parse errors across every file, resolve/normalize/
/// typecheck/ownership across every module), so a multi-error tree reports its full
/// failing-stage set in one run, never one error per run. Hardness predicates:
/// typecheck refusals use `is_interpreter_blocking_diagnostic` and the
/// other stages use `is_error_diagnostic` — both reduce
/// to the `00_core.dag` interpreter-blocking authority on every class those stages
/// can produce (`ComplexityUnknown`, the sole class where the predicates differ, is
/// only produced by complexity analysis, which does not run on this path).
pub(crate) fn resolved_graph_from_sources_with_index(
    index: &MultiEntryIndex,
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    phase_label: &str,
    memo_share: ResolvedGraphMemoShare,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
        Rc<im::Vector<Rc<ErrorNode>>>,
    ),
    String,
> {
    let entry_file = phase_label;
    index
        .module_graph_facts
        .selection
        .admit_pool_names(|| reference_pool_names_for_index(index))?;
    let subject = subject_digest_for_closure(&sources);
    // In-process share tier (resolved_graph_memo): always on. Every value in it was computed by
    // this process, so a hit is an in-run judgment.
    if let Some((graph, si, compile_clean_diags)) = index.resolved_graph_memo.borrow().get(&subject)
    {
        record_required_lane_judged_sources(&sources);
        return Ok((graph.clone(), si.clone(), compile_clean_diags.clone()));
    }

    let mut modules: Vec<Rc<Node>> = Vec::new();
    let mut si_map: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    let mut parse_error_msgs: Vec<String> = Vec::new();
    let mut annotation_diags: im::Vector<Rc<ErrorNode>> = im::Vector::new();

    let parse_started = std::time::Instant::now();
    for source in &sources {
        note_source_hash(index, source);
        let entry = via_index_parse_one_source(index, source);
        if let Some(stored) = &entry.annotation_diags {
            annotation_diags.extend(stored.iter().cloned());
        }
        let parse_result = entry.parse_result.clone();
        let nl_index = entry.newline_index.clone();

        si_map.insert(nl_index.file.clone(), nl_index.clone());
        if let Some(err) = &parse_result.error {
            // Collect-then-refuse: gather every file's parse error before refusing,
            // so a multi-file parse red reports its full set in one run.
            let span = diagnostic_to_span(err.diagnostic.clone());
            let loc = format_error_loc(&span.file, span.start, &si_map);
            parse_error_msgs.push(format!(
                "{}: error: {}",
                loc,
                diagnostic_to_message(err.diagnostic.clone())
            ));
            continue;
        }
        if let Some(module) = &parse_result.module {
            modules.push(module.clone());
        }
    }
    if !parse_error_msgs.is_empty() {
        let source_indices = Rc::new(si_map);
        return Err(join_via_index_stage_refusal(
            &annotation_diags,
            &source_indices,
            parse_error_msgs.join("\n"),
        ));
    }

    let source_indices = Rc::new(si_map);
    let global_table = index.intern_table.borrow().clone();
    resolve_stage_slot_add(|s| s.parse += parse_started.elapsed().as_nanos());

    let resolve_started = std::time::Instant::now();
    let graph =
        v1_compiler_resolve::resolve_modules(Rc::new(modules.into()), source_indices.clone());

    if graph
        .diagnostics
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(join_via_index_stage_refusal(
            &annotation_diags,
            &source_indices,
            format_error_nodes(&graph.diagnostics, &source_indices),
        ));
    }
    resolve_stage_slot_add(|s| s.resolve += resolve_started.elapsed().as_nanos());

    let normalize_started = std::time::Instant::now();
    // Per-module memo (normalize_diag_cache): normalize is diagnostics-only — the
    // authority passes the graph through unchanged (v1.compiler.normalize
    // `NormalizeResult { graph: graph, .. }`) — and its per-module row
    // `normalize_module_diagnostics` is a pure function of the parsed module node,
    // so an entry pays only for modules this process has not normalized before
    // (resolve-split receipt: normalize was 8% of whole-corpus resolve, recomputed
    // per entry at zero marginal information).
    let mut norm_diag_vec: im::Vector<Rc<ErrorNode>> = im::Vector::new();
    for m in graph.modules.iter() {
        let key = m.module.span.file.clone();
        let cached = index.normalize_diag_cache.borrow().get(&key).cloned();
        let module_diags = match cached {
            Some(hit) => hit,
            None => {
                let computed = v1_compiler_normalize::normalize_module_diagnostics(
                    m.clone(),
                    source_indices.clone(),
                );
                index
                    .normalize_diag_cache
                    .borrow_mut()
                    .insert(key, computed.clone());
                computed
            }
        };
        norm_diag_vec.extend(module_diags.iter().cloned());
    }
    let norm_diags = Rc::new(norm_diag_vec);

    if norm_diags
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(join_via_index_stage_refusal(
            &annotation_diags,
            &source_indices,
            format_error_nodes(&norm_diags, &source_indices),
        ));
    }
    resolve_stage_slot_add(|s| s.normalize += normalize_started.elapsed().as_nanos());

    set_phase(FloorPhase::Typecheck, entry_file);
    let reconcile_attributed_before = resolve_stage_slot_snapshot().reconcile_attributed_total();
    let reconcile_started = std::time::Instant::now();
    let typed =
        reconcile_with_typed_cache(graph.clone(), source_indices.clone(), global_table, index)
            .map_err(|e| join_via_index_stage_refusal(&annotation_diags, &source_indices, e))?;
    // `typed` carries the completed per-module typecheck and its diagnostics. Record the
    // verdict before testing whether it is positive: a blocking diagnostic is stronger
    // evidence of visibility than a green result. Parse/resolve aborts and a reconcile that
    // produced no typed result never reach this boundary and are not credited.
    record_required_lane_judged_sources(&sources);
    // Assembly `other` is derived only when the exclusive reconcile rows fit inside the
    // containing reconcile span. A timing overlap is an attribution refusal, never a
    // saturating clamp to a plausible zero.
    let reconcile_total = reconcile_started.elapsed().as_nanos();
    let measured = resolve_stage_slot_snapshot();
    let reconcile_attributed_after = measured.reconcile_attributed_total();
    let reconcile_attributed = reconcile_attributed_after
        .checked_sub(reconcile_attributed_before)
        .ok_or_else(|| {
            format!(
                "assembly attribution refused: NestedSpanAttribution {{ before_nanos: \
             {reconcile_attributed_before}, after_nanos: {reconcile_attributed_after} }}"
            )
        })
        .map_err(|e| join_via_index_stage_refusal(&annotation_diags, &source_indices, e))?;
    let assembly_other = reconcile_total
        .checked_sub(reconcile_attributed)
        .ok_or_else(|| {
            format!(
                "assembly attribution refused: OverAttributed {{ sum_exclusive_nanos: \
             {reconcile_attributed}, parent_span_nanos: {reconcile_total} }}"
            )
        })
        .map_err(|e| join_via_index_stage_refusal(&annotation_diags, &source_indices, e))?;
    resolve_stage_slot_add(|s| s.reconcile_assembly += assembly_other);

    let has_type_errors = typed
        .diagnostics
        .iter()
        .any(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()));
    if has_type_errors {
        let msgs: Vec<String> = typed
            .diagnostics
            .iter()
            .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
            .map(|d| format_error_node(d, &source_indices))
            .collect();
        return Err(join_via_index_stage_refusal(
            &annotation_diags,
            &source_indices,
            msgs.join("\n"),
        ));
    }

    let ownership_started = std::time::Instant::now();
    // Per-module memo (ownership_diag_cache): ownership proofs are a pure per-module
    // map (v1.compiler.compile `module_ownership_proofs`; the authority's graph fold
    // is exactly this row flat_mapped in module order) and `ownership_diagnostics`
    // distributes over per-module concatenation, so the diagnostic list assembled in
    // `typed.modules` order is identical to the graph-grain computation — a module
    // with no bodied items contributes the same empty row the authority's filter
    // skips. First-touch per module; the per-entry graph-grain rerun (7% of
    // whole-corpus resolve in the resolve-split receipt) collapses to cache reads.
    let mut ownership_diag_vec: im::Vector<Rc<ErrorNode>> = im::Vector::new();
    for m in typed.modules.iter() {
        let key = m.module.span.file.clone();
        let cached = index.ownership_diag_cache.borrow().get(&key).cloned();
        let module_diags = match cached {
            Some(hit) => hit,
            None => {
                let proofs = v1_compiler_compile::module_ownership_proofs(m.clone());
                let computed = v1_compiler_compile::ownership_diagnostics(proofs);
                index
                    .ownership_diag_cache
                    .borrow_mut()
                    .insert(key, computed.clone());
                computed
            }
        };
        ownership_diag_vec.extend(module_diags.iter().cloned());
    }
    let ownership_diags = Rc::new(ownership_diag_vec);
    if ownership_diags
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(join_via_index_stage_refusal(
            &annotation_diags,
            &source_indices,
            format_error_nodes(&ownership_diags, &source_indices),
        ));
    }
    resolve_stage_slot_add(|s| s.ownership += ownership_started.elapsed().as_nanos());

    let compile_clean_diags = prepend_via_index_annotation_diags(
        annotation_diags,
        compile_clean_diags_from_resolved_stages(
            &graph.diagnostics,
            &norm_diags,
            &typed,
            &ownership_diags,
        ),
    );

    // Install into the in-process share so same-subject re-resolves skip assembly —
    // UNLESS this is an Ephemeral gate resolve (the compile-clean whole-tree gate): its
    // aggregate graph strong-Rc-pins every TypedModule in the tree and is never re-hit by
    // discovery's per-entry subjects, so memoizing it is the 9.2GB-class resident-retention
    // leak D0.1 removes (ci-two-tier §5). Per-module typed-cache warming already happened
    // above, in reconcile, and is unaffected.
    if memo_share == ResolvedGraphMemoShare::Memoize {
        index.resolved_graph_memo.borrow_mut().insert(
            subject.clone(),
            (
                typed.clone(),
                source_indices.clone(),
                compile_clean_diags.clone(),
            ),
        );
    }

    Ok((typed, source_indices, compile_clean_diags))
}

pub(crate) fn parse_module_node_from_index_source(
    index: &MultiEntryIndex,
    source: Rc<v1_compiler_compile::SourceFile>,
) -> Result<(Rc<Node>, Rc<NewlineIndex>), String> {
    note_source_hash(index, &source);
    let cached = index.parse_cache.borrow().get(&source.path).cloned();
    let (parse_result, nl_index) = match cached {
        Some(entry) => (entry.parse_result, entry.newline_index),
        None => {
            // One acquisition, not one per walk -- see `cli_run::pool_acquire`.
            let tokens = super::pool_acquire::tokens_for(&source.path, &source.content);
            let nl_index = super::pool_acquire::newline_index_for(&source.path, &source.content);
            let current_table = index.intern_table.borrow().clone();
            let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
                let mut m = HashMap::new();
                m.insert(source.path.clone(), nl_index.clone());
                m
            });
            let parsed = v1_compiler_parse::parse_with_table(tokens, single_si, current_table);
            *index.intern_table.borrow_mut() = parsed.intern_table.clone();
            let entry = ParseCacheEntry {
                parse_result: parsed.result.clone(),
                newline_index: nl_index.clone(),
                annotation_diags: None,
            };
            index
                .parse_cache
                .borrow_mut()
                .insert(source.path.clone(), entry.clone());
            (entry.parse_result, entry.newline_index)
        }
    };
    if let Some(err) = &parse_result.error {
        let span = diagnostic_to_span(err.diagnostic.clone());
        let loc = format_error_loc(&span.file, span.start, &Rc::new(HashMap::new()));
        return Err(format!(
            "symbol_index qualified-projection census refused: parse failed for {}: {}",
            loc,
            diagnostic_to_message(err.diagnostic.clone())
        ));
    }
    match &parse_result.module {
        Some(module) => Ok((module.clone(), nl_index)),
        None => Err(format!(
            "symbol_index qualified-projection census refused: no module in {}",
            source.path
        )),
    }
}

pub(crate) fn resolved_graph_from_sources(
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let judged_sources = sources.clone();
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));

    if result.graph.is_some() {
        // A produced graph is the compile-to-resolved path's completed strict judgment receipt,
        // whether its diagnostics are positive or negative.
        record_required_lane_judged_sources(&judged_sources);
    }

    let has_errors = result
        .diagnostics
        .iter()
        .any(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()));
    if has_errors {
        let si: HashMap<String, Rc<NewlineIndex>> = result
            .newline_indices
            .iter()
            .map(|idx| (idx.file.clone(), idx.clone()))
            .collect();
        // THE REFUSAL COUNTS WHAT IT COUNTS. Every line below is a BLOCKING diagnostic -- advisories
        // are filtered out above and never printed -- and the count heads the list, because the
        // adjudication that wraps this text reports one blocked PHASE and a reader of
        // `blocked_phases=1` must not have to infer how many diagnostics block it. Each line names
        // the owning module beside its location, so a line is attributable without mapping a path.
        let mut msgs = Vec::new();
        for d in result.diagnostics.iter() {
            if !is_interpreter_blocking_diagnostic(d.diagnostic.clone()) {
                continue;
            }
            let span = diagnostic_to_span(d.diagnostic.clone());
            let loc = match si.get(&span.file) {
                Some(idx) => {
                    let lc = byte_to_line_col(idx.clone(), span.start);
                    format!("{}:{}:{}", span.file, lc.line, lc.col)
                }
                None => span.file.clone(),
            };
            msgs.push(format!(
                "{}: error: [module {}] {}",
                loc,
                d.module_name,
                diagnostic_to_message(d.diagnostic.clone())
            ));
        }
        // The refused graph is discarded here either way; attributing it takes the only
        // owner, so an `Rc` another holder keeps reports itself as unattributable.
        if let Ok(owned) = Rc::try_unwrap(result) {
            if let Some(graph) = owned.graph {
                typed_graph_byte_attribution("strict-refused", graph);
            }
        }
        return Err(format!(
            "blocking_diagnostics={}\n{}",
            msgs.len(),
            msgs.join("\n")
        ));
    }

    let graph = result
        .graph
        .clone()
        .ok_or_else(|| "compilation produced no graph".to_string())?;
    Ok((graph, result.source_indices.clone()))
}

fn required_lane_judged_module_identities_store() -> &'static Mutex<BTreeSet<String>> {
    static STORE: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(BTreeSet::new()))
}

fn record_required_lane_judged_sources(sources: &[Rc<v1_compiler_compile::SourceFile>]) {
    let mut identities = required_lane_judged_module_identities_store()
        .lock()
        .expect("required-lane resolved-module identity store poisoned");
    for source in sources {
        if let Some(module_path) = extract_module_path(&source.content) {
            identities.insert(module_path);
        }
    }
}

pub fn required_lane_judged_module_identities() -> Vec<String> {
    required_lane_judged_module_identities_store()
        .lock()
        .expect("required-lane resolved-module identity store poisoned")
        .iter()
        .cloned()
        .collect()
}

pub fn whole_tree_strict_sources(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<WholeTreeStrictSources, String> {
    let index = build_module_index(source_roots);
    let total = index.len();
    let all_sources: Vec<Rc<v1_compiler_compile::SourceFile>> = index
        .iter()
        .filter(|(module_path, sf)| {
            crate::cli_run::prepared_subject_exclusion_row_for(
                &sf.path,
                module_path,
                exclude_substrings,
            )
            .is_none()
        })
        .map(|(_, sf)| sf.clone())
        .collect();
    if all_sources.is_empty() {
        return Err("whole-tree corpus is empty (no .dag modules under source roots)".to_string());
    }
    let modules_excluded = total - all_sources.len();
    Ok(WholeTreeStrictSources {
        sources: all_sources,
        modules_resolved: total - modules_excluded,
        modules_excluded,
    })
}

pub fn whole_tree_resolved_ctx(
    source_roots: &[String],
    exclude_substrings: &[String],
    execution_mode: v1_interpreter::ExecutionMode,
) -> Result<WholeTreeCtx, String> {
    let picked = whole_tree_strict_sources(source_roots, exclude_substrings)?;
    let modules_resolved = picked.modules_resolved;
    let modules_excluded = picked.modules_excluded;
    let (graph, source_indices) = resolved_graph_from_sources(picked.sources)?;
    Ok(WholeTreeCtx {
        ctx: v1_interpreter::InterpContext::with_runtime_options(
            graph.as_ref(),
            source_indices,
            execution_mode,
            None,
            None,
        ),
        modules_resolved,
        modules_excluded,
    })
}

/// THE RETAINED SET BY STRUCTURE, AT A FLOOR SEAM. The seam beats (`floor_seam`) say how many
/// bytes a phase left resident; they cannot say WHICH structure holds them. This census can, at
/// entry grain: for every typed module held by the named graphs it sums each typecheck-env map's
/// size (`retained`, what is resident if each module's spine were its own allocation) and each
/// distinct `Rc` spine once (`distinct`, what is actually allocated). `dup = retained/distinct`
/// is the per-module materialization factor; a factor well above 1 on the ancestry maps is the
/// quadratic retention the M0 probe (v1-run-stability-throughline) was written to locate, and
/// this is that probe moved onto the graphs the floor really holds instead of a second whole-tree
/// resolve nobody called.
///
/// ENTRIES, NOT BYTES. These maps are `im::HashMap` -- persistent HAMTs -- and a module's map is
/// built by merging its parents', which shares internal nodes. `distinct` dedupes the map ROOT,
/// not the nodes under it, so it counts shared structure once per root and overstates bytes by
/// an amount it cannot itself report.
///
/// It also answers the overlap question across graphs at identity grain: `module_paths` counts
/// distinct module identities, `typed_modules` distinct `TypedModule` allocations. More
/// allocations than identities means one module was typechecked into two resident copies, which
/// is authored duplication (DESIGN §2) rather than sharing. A pure reader: O(modules), no clone.
pub(crate) fn floor_retention_census(
    seam: &str,
    graphs: &[(&str, &v1_compiler_compile::ResolvedGraph)],
) {
    struct FieldTally {
        name: &'static str,
        retained: usize,
        distinct: usize,
        spines: HashSet<usize>,
    }
    impl FieldTally {
        fn add<T>(&mut self, spine: &Rc<T>, entries: usize) {
            self.retained += entries;
            if self.spines.insert(Rc::as_ptr(spine) as *const () as usize) {
                self.distinct += entries;
            }
        }
    }
    let mut tallies: Vec<FieldTally> = [
        "tec.str_bindings",
        "tec.deps_map",
        "tec.variant_locals",
        "te.str_bindings",
        "te.ancestry_str_bindings",
        "te.bindings",
        "te.source_visible_names",
        "te.inductive_fields",
        "module_item_registry",
        "module_items",
    ]
    .iter()
    .map(|name| FieldTally {
        name,
        retained: 0,
        distinct: 0,
        spines: HashSet::new(),
    })
    .collect();
    let mut typed_modules: HashSet<usize> = HashSet::new();
    let mut module_paths: HashSet<String> = HashSet::new();
    let mut held = 0usize;
    for (_, graph) in graphs {
        for m in graph.modules.iter() {
            held += 1;
            if !typed_modules.insert(Rc::as_ptr(m) as usize) {
                continue;
            }
            module_paths.insert(m.type_env.module_path.clone());
            let (te, tec) = (&m.type_env, &m.type_env_cache);
            tallies[0].add(&tec.str_bindings, tec.str_bindings.len());
            tallies[1].add(&tec.deps_map, tec.deps_map.len());
            tallies[2].add(&tec.variant_locals, tec.variant_locals.len());
            tallies[3].add(&te.str_bindings, te.str_bindings.len());
            tallies[4].add(&te.ancestry_str_bindings, te.ancestry_str_bindings.len());
            tallies[5].add(&te.bindings, te.bindings.len());
            tallies[6].add(&te.source_visible_names, te.source_visible_names.len());
            tallies[7].add(&te.inductive_fields, te.inductive_fields.len());
            tallies[8].add(&m.item_registry, m.item_registry.len());
            tallies[9].add(&m.items, m.items.len());
        }
    }
    // THE ENTRY-INDEPENDENCE DIFFERENTIAL, over every module path held by more than one
    // allocation. Each graph's assembly rewires its own copy (`finish_resolved_graph_assembly`);
    // sharing one wired copy per module at the pool's index is lawful only if every copy is the
    // same wiring. Compared at IDENTITY grain -- which node each binding resolves to, by `Rc`
    // pointer, and which module each parent link names -- not by content, so an equal-looking
    // copy that binds a different declaration reads as differing. A derived `==` would recurse
    // through every parent environment without a pointer short-cut.
    let mut copies: BTreeMap<String, Vec<&Rc<crate::v1_compiler_infer_items::TypedModule>>> =
        BTreeMap::new();
    let mut seen: HashSet<usize> = HashSet::new();
    for (_, graph) in graphs {
        for m in graph.modules.iter() {
            if seen.insert(Rc::as_ptr(m) as usize) {
                // KEYED BY SOURCE, NOT ONLY BY MODULE PATH: the planning row reads the diff base's
                // version of a changed module from a separate checkout, and two SOURCES under one
                // module path are two modules, which legitimately wire differently.
                copies
                    .entry(format!("{}@{}", m.type_env.module_path, m.module.span.file))
                    .or_default()
                    .push(m);
            }
        }
    }
    let mut identical = 0usize;
    let mut differing: Vec<String> = Vec::new();
    for (path, ms) in copies.iter().filter(|(_, ms)| ms.len() > 1) {
        let prints: Vec<[u64; 6]> = ms.iter().map(|m| wiring_identity(m)).collect();
        let parts = [
            "module",
            "type_bindings",
            "ancestry",
            "type_parents",
            "func_local",
            "func_parents",
        ];
        let diff: Vec<&str> = (0..6)
            .filter(|i| prints.iter().any(|p| p[*i] != prints[0][*i]))
            .map(|i| parts[i])
            .collect();
        if diff.is_empty() {
            identical += 1;
        } else {
            differing.push(format!("{path}:{}", diff.join("+")));
        }
    }
    eprintln!(
        "[floor-heap] retained seam={seam} duplicated_paths={} identical_wiring={identical} \
         differing_wiring={} [{}]",
        identical + differing.len(),
        differing.len(),
        differing.join(","),
    );
    let names: Vec<String> = graphs
        .iter()
        .map(|(name, g)| format!("{name}:{}", g.modules.len()))
        .collect();
    eprintln!(
        "[floor-heap] retained seam={seam} graphs=[{}] held_modules={held} typed_modules={} \
         module_paths={}",
        names.join(","),
        typed_modules.len(),
        module_paths.len(),
    );
    for t in &tallies {
        eprintln!(
            "[floor-heap] retained seam={seam} field={} retained={} distinct={} spines={}",
            t.name,
            t.retained,
            t.distinct,
            t.spines.len(),
        );
    }
}

/// One copy's wiring at identity grain: the module node, each type binding's resolved node, each
/// parent environment's module, each function signature, and each parent function environment's
/// name -- the facts a rewire decides -- each folded to one hash so copies compare component-wise.
fn wiring_identity(m: &crate::v1_compiler_infer_items::TypedModule) -> [u64; 6] {
    use std::hash::{Hash, Hasher};
    fn fold<I: IntoIterator<Item = (String, usize)>>(items: I) -> u64 {
        let mut rows: Vec<(String, usize)> = items.into_iter().collect();
        rows.sort();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        rows.hash(&mut h);
        h.finish()
    }
    let te = &m.type_env;
    let fe = &m.func_env;
    [
        fold([(String::new(), Rc::as_ptr(&m.module) as usize)]),
        fold(
            te.str_bindings
                .iter()
                .map(|(k, b)| (k.clone(), Rc::as_ptr(&b.resolved) as usize))
                .chain(
                    te.bindings
                        .iter()
                        .map(|(k, b)| (k.to_string(), Rc::as_ptr(&b.resolved) as usize)),
                ),
        ),
        fold(
            te.ancestry_str_bindings
                .iter()
                .map(|(k, b)| (k.clone(), Rc::as_ptr(&b.resolved) as usize)),
        ),
        fold(
            te.parents
                .iter()
                .enumerate()
                .map(|(i, p)| (p.module_path.clone(), i)),
        ),
        fold(
            fe.local
                .iter()
                .map(|(k, sig)| (k.clone(), Rc::as_ptr(sig) as usize)),
        ),
        fold(
            fe.parents
                .iter()
                .enumerate()
                .map(|(i, p)| (p.name.clone(), i)),
        ),
    ]
}

/// Every graph the thread's process resolve store holds, by entry — the planning and prelude
/// authorities resolved through `resolve_entry_graph_shared`, which live until the thread ends.
pub(crate) fn process_resolve_store_graphs() -> Vec<(String, Rc<v1_compiler_compile::ResolvedGraph>)>
{
    PROCESS_RESOLVE_STORE.with(|s| {
        let mut graphs: Vec<_> = s
            .borrow()
            .iter()
            .map(|((_, entry), (graph, _))| (entry.clone(), graph.clone()))
            .collect();
        graphs.sort_by(|a, b| a.0.cmp(&b.0));
        graphs
    })
}

/// Companion to a Bool witness: `emit_on_demand_family_crate_pr_native_agreement_holds`
/// → `emit_on_demand_family_crate_pr_native_agreement_failure_receipt`.
///
/// Both corpus naming conventions are normalized away — `_holds` (claim witnesses) and
/// `_passes` (the cheap-floor gate witnesses in `tools.floor_effect_gate_witness`) — and so
/// is neither: a name carrying no suffix projects to its own stem. That is the 2026-08-24
/// change. Recognizing only `_holds` once left the gate witnesses unreachable from this
/// channel, which is why ten consecutive `extdeps_scope_placement_gate_passes` reds reported
/// nothing but `returned Bool(false)`; recognizing exactly two suffixes left 84.8% of the
/// discovered roster in the same silence, for the same reason one layer out. Widening the
/// derivation to all names cannot invent a required hook for a witness that has none: a
/// companion that does not exist yields an empty receipt and appends nothing.
/// Delegates suffix derivation to `gunbc.test_module_hygiene.failure_receipt_companion`
/// (single authority — orphan reachability and claim_executor share the same rule).
/// The projection is TOTAL — every witness name maps to a companion spelling, and the suffix
/// gets no vote on whether something is a witness (that question belongs to floor discovery).
/// `AuthorityRefused` is a located lookup failure and must not be rendered as a missing
/// companion; a companion that simply does not exist surfaces as an empty receipt, appended
/// as nothing.
pub use test_module_hygiene_bridge::FailureReceiptCompanionLookup;

pub(crate) fn resolve_entry_file_under_roots(
    source_roots: &[String],
    entry: &str,
) -> Result<String, String> {
    let path = Path::new(entry);
    if path.is_file() {
        return Ok(path.to_string_lossy().into_owned());
    }
    for root in source_roots {
        let root_path = Path::new(root);
        let root_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !root_name.is_empty() {
            let prefix = format!("{root_name}/");
            if let Some(suffix) = entry.strip_prefix(&prefix) {
                let candidate = root_path.join(suffix);
                if candidate.is_file() {
                    return Ok(candidate.to_string_lossy().into_owned());
                }
            }
        }
        let candidate = root_path.join(entry);
        if candidate.is_file() {
            return Ok(candidate.to_string_lossy().into_owned());
        }
    }
    Err(format!(
        "entry file does not exist or is not a file: {}",
        entry
    ))
}

pub(crate) fn import_closure_files_from_graph(
    graph: &v1_compiler_compile::ResolvedGraph,
) -> HashSet<String> {
    let mut files = HashSet::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            files.insert(normalize_repo_path(&item.span.file));
        }
    }
    files
}

pub(crate) fn import_closure_repo_paths_for_entry(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
) -> HashSet<String> {
    import_closure_live_paths_with_facts(entry_path, facts)
        .into_iter()
        .map(|p| workspace_relative_repo_path(&p))
        .collect()
}

pub(crate) fn source_root_ref_variant_for_root(root: &str) -> Result<String, String> {
    match root.trim_end_matches('/') {
        "src/v2" => Ok("V2Tree".to_string()),
        "dag" => Ok("DagTree".to_string()),
        other => Err(format!(
            "source_root tagging: unknown --source-root '{other}' \
             (authority gunbc.ci_layer_roots.witness_layer_roots = [src/v2, dag] -> \
             SourceRootRef {{V2Tree, DagTree}})"
        )),
    }
}

pub(crate) fn source_root_ref_token_for_path(
    file_path: &str,
    source_roots: &[String],
) -> Result<String, String> {
    let rel_path = repo_relative_dag_path(file_path);
    let matched: Vec<String> = source_roots
        .iter()
        .map(|r| repo_relative_dag_path(r))
        .filter(|r| {
            let r = r.trim_end_matches('/');
            rel_path == r || rel_path.starts_with(&format!("{r}/"))
        })
        .collect();
    match matched.as_slice() {
        [] => Err(format!(
            "source_root tagging: file '{file_path}' (repo-relative '{rel_path}') matches no \
             --source-root {source_roots:?}"
        )),
        [one] => source_root_ref_variant_for_root(one),
        _ => Err(format!(
            "source_root tagging: file '{file_path}' matches multiple --source-root {matched:?}"
        )),
    }
}

pub(crate) fn source_root_ingest_symbol_from_stem(stem: &str) -> String {
    let mut body = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            body.push(ch);
        } else {
            body.push('_');
        }
    }
    if body.is_empty() {
        body.push_str("host_sr_empty");
    } else if body.as_bytes()[0].is_ascii_digit()
        || v1_compiler_tokenize::is_keyword_text(
            body.clone(),
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        )
    {
        // THE THIRD ESCAPE ARM, AND THE CORPUS ALREADY CONTAINED ITS CASE.
        //
        // This minted `^{stem}` after escaping non-identifier characters and a leading digit,
        // and stopped there. A stem that IS a keyword still lexes as a keyword, so `dag/std/
        // import.dag` emitted `^import` and the manifest failed to parse -- measured, at the
        // caret: "expected identifier or `(` after `^`, found keyword". Two stems in the tree
        // reach it today, `dag/std/import.dag` and `dag/extdeps/languages/go/module.dag`.
        //
        // The mint promised a valid symbol and returned an unparseable one with no refusal,
        // which is why nothing caught it: the emitter succeeded, the file was written, and the
        // only executing consumer is a `long/` `ReadsLiveTree` witness the floor declines.
        //
        // The keyword test routes through `v1_compiler_tokenize` `is_keyword_text`, whose set is
        // derived from the grammar. A literal list here would be a second authority for the
        // keyword vocabulary (DESIGN §3) and would go stale the next time the grammar gains one.
        body = format!("sr_{body}");
    }
    format!("^{body}")
}

pub fn source_root_ingest_artifact_id_for_path(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("host_sr");
    source_root_ingest_symbol_from_stem(stem)
}

pub fn source_root_ingest_content_hash_fnv1a64(records: &[SourceRootReadRecord]) -> String {
    let mut material = String::new();
    for rec in records {
        material.push_str(&rec.file_path);
        material.push('\0');
        material.push_str(&rec.source);
        material.push('\0');
    }
    fnv1a64_digest_of_material(&material)
}

pub(crate) fn pool_roots_abs(pool_roots: &[String]) -> Vec<String> {
    pool_roots.iter().map(|r| anchor_source_root(r)).collect()
}

#[cfg(test)]
pub(crate) fn import_resolution_facts_call_count_for_test() -> usize {
    IMPORT_RESOLUTION_FACTS_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

pub(crate) fn import_resolution_facts_with_observation(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> ImportResolutionObservation {
    #[cfg(test)]
    IMPORT_RESOLUTION_FACTS_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let abs_importer_roots = pool_roots_abs(importer_roots);
    let declared: HashSet<String> = build_module_path_index(&abs_pool_roots)
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    let mut out = Vec::new();
    let mut observed_paths: HashSet<String> = HashSet::new();
    let mut read_refusals: Vec<(String, String)> = Vec::new();
    for root in &abs_importer_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut dag_files);
        dag_files.sort();
        for file in dag_files {
            let rel = rel_path_for_layer_import(&file);
            if is_excluded_import_path(&rel, exclude_substrings) {
                continue;
            }
            observed_paths.insert(workspace_relative_repo_path(&rel));
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    read_refusals.push((workspace_relative_repo_path(&rel), e.to_string()));
                    continue;
                }
            };
            out.extend(import_facts_for_file(&rel, &content, |m| {
                declared.contains(m)
            }));
        }
    }
    ImportResolutionObservation {
        facts: out,
        observed_paths,
        read_refusals,
    }
}

/// THE PER-FILE HALF of `import_resolution_facts`: one importer's `import` lines, each marked
/// declared or not against the pool's module index. One authority for both demands on it -- the
/// population walk above maps it over every importer file, and
/// `cli_run` `dependency_resolution_facts_at` asks it for one.
pub(crate) fn import_facts_for_file(
    rel: &str,
    content: &str,
    is_declared: impl Fn(&str) -> bool,
) -> Vec<ImportResolutionFactRaw> {
    extract_import_paths(content)
        .into_iter()
        .map(|import_module| ImportResolutionFactRaw {
            path: rel.to_string(),
            target_declared: is_declared(&import_module),
            import_module,
        })
        .collect()
}

pub fn import_resolution_facts(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ImportResolutionFactRaw> {
    import_resolution_facts_with_observation(pool_roots, importer_roots, exclude_substrings).facts
}

pub fn module_declaration_facts(pool_roots: &[String]) -> Vec<ModuleDeclarationFactRaw> {
    #[cfg(test)]
    MODULE_DECLARATION_FACTS_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let mut out: Vec<ModuleDeclarationFactRaw> = build_module_path_index(&abs_pool_roots)
        .into_iter()
        .map(|(module, path)| ModuleDeclarationFactRaw { module, path })
        .collect();
    out.sort_by(|a, b| a.module.cmp(&b.module));
    out
}

/// The KEYED read of the same module census `module_declaration_facts` publishes: the row for
/// `module_path`, or none.
///
/// The population read exists because a consumer wanted every module; this exists because most
/// consumers want ONE — "is this module declared, and where does it live". Both project the one
/// `build_module_path_index` authority, so a hit here is the same `(module, path)` pair the list
/// carries; the difference is that the keyed form neither materializes nor sorts ~thousands of rows
/// to answer about one, and the interpreter never marshals them.
///
/// `Option`, not a bool: the two questions a keyed caller asks — declared? at which path? — are one
/// lookup, and answering only the first would send the second back through the population read.
pub fn module_declaration_fact_at(
    pool_roots: &[String],
    module_path: &str,
) -> Option<ModuleDeclarationFactRaw> {
    let abs_pool_roots = pool_roots_abs(pool_roots);
    with_module_path_index(&abs_pool_roots, |index| {
        index.get(module_path).map(|path| ModuleDeclarationFactRaw {
            module: module_path.to_string(),
            path: path.clone(),
        })
    })
}

/// Project reference edges into the `ImportResolutionFactRaw` channel the module-graph adjacency and
/// closure consumers already read (the `module_graph.dag` single-swap-point contract — downstream is
/// edge-source-agnostic). `strict` drops `AmbiguousBare` edges.
///
/// The tier is per-CONSUMER and the two are not interchangeable:
///   - `false` (keep AmbiguousBare) for the LOADER — over-connection is harmless there, since a
///     superset only compiles extra modules.
///   - `true` for SELECTION — over-connection is not a safety problem
///     here, it is what destroys the answer. Measured: at `false` an entry's median closure is
///     1136 of 2240 modules (homonyms fan every referrer across the pool); at `true` it is 96,
///     the same order as the import-only baseline's 54.
/// Grouping these two under one tier is what made the 2026-07-14 selection repoint look
/// impossible — see `build_module_graph_facts_live_uncached`.
pub fn reference_edges_as_import_facts(
    edges: &[ReferenceEdgeRaw],
    strict: bool,
) -> Vec<ImportResolutionFactRaw> {
    edges
        .iter()
        .filter(|e| !strict || e.resolution != RefEdgeResolution::AmbiguousBare)
        .map(|e| ImportResolutionFactRaw {
            path: e.path.clone(),
            import_module: e.target_module.clone(),
            target_declared: true,
        })
        .collect()
}

/// Parse a `.dag` module's source text through the real front-end. Returns the module node, or
/// `None` on a parse error (the whole-tree compile reports such errors loudly; the module graph
/// simply omits its edges, and the corpus stays green because a syntax-broken file never resolves).
pub(crate) fn parse_module_node_tolerant(
    rel: &str,
    content: &str,
) -> Option<Rc<crate::v1_std_core::Node>> {
    let filename = rel.to_string();
    // One acquisition, not one per walk -- see `cli_run::pool_acquire`.
    let tokens = super::pool_acquire::tokens_for(&filename, content);
    let source_index = super::pool_acquire::newline_index_for(&filename, content);
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let result = crate::v1_compiler_parse::parse(tokens, std::rc::Rc::new(source_indices));
    if result.error.is_some() {
        return None;
    }
    result.module.clone()
}

/// THE POOL-WIDE HALF OF THE REFERENCE-EDGE PRODUCER: which pool module declares each exported
/// name, and the set of declared module names. It is the only input to a file's reference edges
/// that is a fact about the whole pool, and it is a fact about declaration HEADS, so it is read
/// with the heads reading of the grammar (`v1.compiler.parse.parse_heads_with_table`) -- the same
/// reading `module_path_index::parse_module_binding` and the pool census already take, over the
/// same shared token acquisition (`pool_acquire`). Bodies are not built here: a body's
/// grammaticality is owned by the required parse sweep and by the front end of whatever closure
/// is compiled, so building every pool body to read its declaration names was work no consumer of
/// this half demanded (DESIGN §2, and the precedent stated at `parse_module_binding`).
///
/// What that narrows, stated exactly: a pool file whose heads parse but whose BODY does not now
/// contributes its declared names, where the full reading skipped it. Such a file refuses at the
/// required parse sweep and in any closure that compiles it.
pub(crate) struct ReferencePoolNames {
    pub(crate) decl_index: HashMap<String, BTreeSet<String>>,
    pub(crate) module_names: HashSet<String>,
}

thread_local! {
    static REFERENCE_POOL_NAMES_CACHE: RefCell<HashMap<String, Rc<ReferencePoolNames>>> =
        RefCell::new(HashMap::new());
}

impl ReferencePoolNames {
    /// THE ONE NAME DERIVATION, over declaration heads in pool-precedence order: a module name
    /// already claimed does not re-contribute (first-root-wins, as `build_module_path_index`).
    /// Both acquisitions below feed it, so the index is one function of the heads reading and not
    /// two walks that agree by care.
    fn from_heads_modules<'a>(
        modules: impl IntoIterator<Item = (String, &'a Rc<crate::v1_std_core::Node>)>,
    ) -> Self {
        let mut decl_index: HashMap<String, BTreeSet<String>> = HashMap::new();
        let mut module_names: HashSet<String> = HashSet::new();
        for (module_name, tree) in modules {
            if !module_names.insert(module_name.clone()) {
                continue;
            }
            for name in collect_module_decl_names(tree) {
                decl_index
                    .entry(name)
                    .or_default()
                    .insert(module_name.clone());
            }
        }
        ReferencePoolNames {
            decl_index,
            module_names,
        }
    }
}

/// The name index from the POOL CENSUS'S OWN heads reading (`pool_parse`), which every resolve
/// through this index already forces for its qualified fill and bare census. A resolve therefore
/// reads the pool's heads once, not once for the census and again for reference edges.
///
/// ONE DERIVATION PER INDEX. The index is a function of `pool_parse`, which this index holds for
/// its life, and it is demanded once per out-of-tree bare name (`pool_census_for_name`) -- so its
/// least common ancestor is the index, and it is derived there once rather than rebuilt from the
/// whole pool's heads on every demand (measured: ~230ms per rebuild, 3,852 demands in one
/// whole-pool admission).
pub(crate) fn reference_pool_names_for_index(
    index: &MultiEntryIndex,
) -> Result<Rc<ReferencePoolNames>, String> {
    if let Some(names) = index.reference_pool_names.borrow().clone() {
        return Ok(names);
    }
    let pool = pool_parse(index)?;
    let started = std::time::Instant::now();
    let names = Rc::new(ReferencePoolNames::from_heads_modules(
        pool.nodes_by_file
            .iter()
            .map(|(_, node)| (node.name.clone(), node)),
    ));
    super::pre_entry_phase::record(
        "reference_pool_names_from_census",
        super::pre_entry_phase::PhaseScale::Tree,
        started.elapsed(),
    );
    *index.reference_pool_names.borrow_mut() = Some(names.clone());
    Ok(names)
}

/// The name index for a consumer that holds no resolve index (the whole-pool selection tier's
/// affected-set demand): its own heads walk over the roots, into the same derivation.
pub(crate) fn reference_pool_names(pool_roots: &[String]) -> Rc<ReferencePoolNames> {
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let key = abs_pool_roots.join("\u{1e}");
    if let Some(hit) = REFERENCE_POOL_NAMES_CACHE.with(|c| c.borrow().get(&key).cloned()) {
        shared_fill::record_hit("reference_pool_names", &key);
        return hit;
    }
    // A PROCESS-WIDE POOL FILL, ACCOUNTED AS ONE. The heads index is built once per pool per
    // process and then served to every consumer -- the same class as module_path_index and
    // reference_edges, which already report through shared_fill. Unrecorded, it was billed as the
    // own CPU of whichever claim first reached an import-less file.
    shared_fill::begin_fill();
    let started = std::time::Instant::now();
    let mut heads: Vec<(String, Rc<crate::v1_std_core::Node>)> = Vec::new();
    for root in &abs_pool_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut files);
        files.sort();
        for file in files {
            let rel = rel_path_for_layer_import(&file);
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            let Some(module_name) = extract_module_path(&content) else {
                continue;
            };
            let Some(tree) = parse_module_heads_tolerant(&rel, &content) else {
                continue;
            };
            heads.push((module_name, tree));
        }
    }
    let names = Rc::new(ReferencePoolNames::from_heads_modules(
        heads.iter().map(|(m, t)| (m.clone(), t)),
    ));
    super::pre_entry_phase::record(
        "reference_pool_names_heads",
        super::pre_entry_phase::PhaseScale::Tree,
        started.elapsed(),
    );
    shared_fill::record_fill(
        "reference_pool_names",
        &key,
        started.elapsed().as_nanos() as u64,
    );
    REFERENCE_POOL_NAMES_CACHE.with(|c| c.borrow_mut().insert(key, names.clone()));
    names
}

/// The heads reading of one pool file, `None` on a heads-grammar refusal.
fn parse_module_heads_tolerant(rel: &str, content: &str) -> Option<Rc<crate::v1_std_core::Node>> {
    let filename = rel.to_string();
    let tokens = super::pool_acquire::tokens_for(&filename, content);
    let source_index = super::pool_acquire::newline_index_for(&filename, content);
    let mut source_indices = HashMap::new();
    source_indices.insert(filename, source_index);
    let result = crate::v1_compiler_parse::parse_heads_with_table(
        tokens,
        Rc::new(source_indices),
        crate::v1_std_core::empty_intern_table(),
    )
    .result
    .clone();
    if result.error.is_some() {
        return None;
    }
    result.module.clone()
}

const REFERENCE_SELECTION_EXCLUDE: &[String] = &[];

/// THE SELECTION TIER OF THE MODULE GRAPH, produced on demand at the grain it is demanded.
///
/// Two demands, one producer (`reference_edges_for_file`): the whole-pool adjacency and its
/// unaccounted set are built once, the first time an affected-set consumer asks
/// (`selection_adjacency`, `reference_unaccounted`); a single file's strict reference targets are
/// produced when the resolve path asks for that file, and memoized per file. Both are the same
/// function of (file bytes, pool name index), so a per-file answer equals the whole-pool row for
/// that file by construction rather than by a second producer agreeing with the first.
pub struct ReferenceSelectionTier {
    roots: Vec<String>,
    import_edges: Vec<ImportResolutionFactRaw>,
    nodes: Vec<ModuleDeclarationFactRaw>,
    module_to_path: HashMap<String, String>,
    whole: std::cell::OnceCell<(HashMap<String, Vec<String>>, HashSet<String>)>,
    per_file: RefCell<HashMap<String, Vec<String>>>,
    names: std::cell::OnceCell<Rc<ReferencePoolNames>>,
}

impl ReferenceSelectionTier {
    pub(crate) fn new(
        roots: Vec<String>,
        import_edges: Vec<ImportResolutionFactRaw>,
        nodes: &[ModuleDeclarationFactRaw],
    ) -> Self {
        let module_to_path = nodes
            .iter()
            .map(|n| (n.module.clone(), workspace_relative_repo_path(&n.path)))
            .collect();
        ReferenceSelectionTier {
            roots,
            import_edges,
            nodes: nodes.to_vec(),
            module_to_path,
            whole: std::cell::OnceCell::new(),
            per_file: RefCell::new(HashMap::new()),
            names: std::cell::OnceCell::new(),
        }
    }

    /// A tier whose whole-pool answer is supplied rather than produced (synthetic fixtures).
    #[cfg(test)]
    pub(crate) fn supplied(adjacency: HashMap<String, Vec<String>>) -> Self {
        let tier = ReferenceSelectionTier::new(Vec::new(), Vec::new(), &[]);
        let _ = tier.whole.set((adjacency, HashSet::new()));
        tier
    }

    fn whole(&self) -> &(HashMap<String, Vec<String>>, HashSet<String>) {
        self.whole.get_or_init(|| {
            let started = std::time::Instant::now();
            let mut selection_edges = self.import_edges.clone();
            selection_edges.extend(reference_edges_as_import_facts(
                &reference_resolution_facts(&self.roots, &self.roots, REFERENCE_SELECTION_EXCLUDE),
                /* strict */ true,
            ));
            let adjacency = build_import_adjacency(&selection_edges, &self.nodes);
            let unaccounted = reference_accounting_refusals(
                &self.roots,
                &self.roots,
                REFERENCE_SELECTION_EXCLUDE,
            )
            .into_iter()
            .map(|r| workspace_relative_repo_path(&r.path))
            .collect();
            super::pre_entry_phase::record(
                "selection_tier_whole_pool",
                super::pre_entry_phase::PhaseScale::Tree,
                started.elapsed(),
            );
            (adjacency, unaccounted)
        })
    }

    /// Whole-pool selection adjacency: import edges plus strict-tier reference edges.
    pub(crate) fn selection_adjacency(&self) -> &HashMap<String, Vec<String>> {
        &self.whole().0
    }

    /// Import-less files the reference-edge producer could not answer for.
    pub(crate) fn reference_unaccounted(&self) -> &HashSet<String> {
        &self.whole().1
    }

    /// Admit the pool name index a resolve index already derives from its census reading
    /// (`reference_pool_names_for_index`), before any per-file demand. Called at the head of
    /// every resolve through an index, where a census refusal can still refuse the resolve.
    pub(crate) fn admit_pool_names(
        &self,
        produce: impl FnOnce() -> Result<Rc<ReferencePoolNames>, String>,
    ) -> Result<(), String> {
        if self.whole.get().is_some() || self.names.get().is_some() {
            return Ok(());
        }
        let _ = self.names.set(produce()?);
        Ok(())
    }

    /// Strict-tier reference targets of ONE file, as workspace-relative paths. When the whole
    /// tier has already been produced it is read; otherwise only this file is.
    pub(crate) fn strict_reference_targets(&self, file_rel: &str) -> Vec<String> {
        if let Some((adjacency, _)) = self.whole.get() {
            return adjacency.get(file_rel).cloned().unwrap_or_default();
        }
        if let Some(hit) = self.per_file.borrow().get(file_rel) {
            return hit.clone();
        }
        let started = std::time::Instant::now();
        let targets = self.produce_one(file_rel);
        super::pre_entry_phase::record(
            "selection_tier_closure_files",
            super::pre_entry_phase::PhaseScale::Closure,
            started.elapsed(),
        );
        self.per_file
            .borrow_mut()
            .insert(file_rel.to_string(), targets.clone());
        targets
    }

    fn produce_one(&self, file_rel: &str) -> Vec<String> {
        // The whole-pool producer walks the pool roots; a file outside them has no row there,
        // so it has none here.
        let abs = workspace_root().join(file_rel);
        let under_roots = pool_roots_abs(&self.roots)
            .iter()
            .any(|r| abs.starts_with(Path::new(r)));
        if !under_roots || !abs.is_file() {
            return Vec::new();
        }
        let content = std::fs::read_to_string(&abs).ok();
        // The resolve path admitted the census-derived index; a consumer that reached here without
        // one walks the roots itself, into the same derivation.
        let names = match self.names.get() {
            Some(n) => n.clone(),
            None => reference_pool_names(&self.roots),
        };
        let FileReferenceEdges::Edges(edges) =
            reference_edges_for_file(file_rel, content.as_deref(), &names)
        else {
            return Vec::new();
        };
        let mut out: Vec<String> = Vec::new();
        for fact in reference_edges_as_import_facts(&edges, /* strict */ true) {
            if let Some(path) = self.module_to_path.get(&fact.import_module) {
                if !out.contains(path) {
                    out.push(path.clone());
                }
            }
        }
        out
    }
}

/// One file's answer from the reference-edge producer.
pub(crate) enum FileReferenceEdges {
    /// The file carries `import` lines: its edges are owned EXACTLY by `import_resolution_facts`,
    /// and emitting reference edges for it would only over-connect.
    ImportBearing,
    /// The file's reference edges, every tier (`reference_edges_as_import_facts` filters).
    Edges(Vec<ReferenceEdgeRaw>),
    /// An import-less file the producer could not answer for, with the located cause.
    Unaccounted(&'static str),
}

/// ONE FILE'S FREE NAMES, read from its FULL parse by the one structural walk
/// (`collect_node_refs`). Two consumers: the reference-edge producer below, and the bare-provider
/// gate (`cli_run::parsed_bare_candidates`), which before this read the same file with a byte
/// scanner that had no grammar -- `response {` in a service operation read as a reference to any
/// top-level `fn response` in the pool.
pub(crate) struct ParsedFileReferences {
    pub(crate) bare: std::collections::HashSet<String>,
    pub(crate) chains: Vec<Vec<String>>,
    pub(crate) positions: super::BarePositions,
    /// Type names at the AUTHORED type positions of the raw parse, which `collect_node_refs`
    /// does not read because it was written for the prepared tree: there a parameter's type is
    /// its `type_annotation`, while the parser puts it in the parameter's children and puts a
    /// return or field type in `inferred: Resolved`. Kept apart from `bare` so the reference-edge
    /// producer's population is unchanged by this reader.
    pub(crate) authored_types: std::collections::HashSet<String>,
    /// The module paths the file's `import` lines name, as the parser read them.
    pub(crate) imports: Vec<String>,
    /// The names this module binds for itself (`module_self_bound_names`): a bare occurrence of
    /// one is bound by that declaration and is never a reference out.
    pub(crate) self_declared: BTreeSet<String>,
}

/// The authored type positions of a RAW parse (see `ParsedFileReferences::authored_types`),
/// over every slot of every node.
fn raw_parse_authored_type_names(
    node: &Rc<crate::v1_std_core::Node>,
    out: &mut std::collections::HashSet<String>,
) {
    if let Some(ty) = raw_declared_type(node) {
        raw_type_names(&ty, out);
    }
    if let Some(ty) = &node.type_annotation {
        raw_type_names(ty, out);
    }
    // A `uses` binding (`uses net: std.resources.Network`) carries its resource type the way a
    // parameter carries its type: as the binding node's child. Missing it dropped the declaring
    // module from both the module-path closure and the bare gate.
    for binding in node.params.iter().chain(node.uses.iter()) {
        for ty in binding.children.iter() {
            raw_type_names(ty, out);
        }
    }
    let slots = node
        .children
        .iter()
        .chain(node.params.iter())
        .chain(node.properties.iter())
        .chain(node.uses.iter())
        .chain(node.body.iter())
        .chain(node.transport.iter());
    for child in slots {
        raw_parse_authored_type_names(child, out);
    }
}

/// The type the parser recorded as `inferred: Resolved` on a declaration, field or operation.
fn raw_declared_type(node: &Rc<crate::v1_std_core::Node>) -> Option<Rc<crate::v1_std_core::Node>> {
    match node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node: ty }) => Some(ty.clone()),
        _ => None,
    }
}

/// The type names of one raw type node. Only expression-free nodes are types: a parameter's
/// children also carry its default VALUE, which is an expression, not a type. A child carrying its
/// own declared type is a FIELD of a record type (an operation's `output { context: T }`, a
/// variant's payload): its name is a label and only its type is read.
fn raw_type_names(ty: &Rc<crate::v1_std_core::Node>, out: &mut std::collections::HashSet<String>) {
    use crate::v1_std_core::ExprData;
    if !matches!(&*ty.expr_data, ExprData::NoExprData) {
        return;
    }
    if !ty.name.is_empty() {
        out.insert(ty.name.clone());
    }
    for child in ty.children.iter().chain(ty.params.iter()) {
        match raw_declared_type(child) {
            Some(field_type) => raw_type_names(&field_type, out),
            None => raw_type_names(child, out),
        }
    }
    if let Some(annotation) = &ty.type_annotation {
        raw_type_names(annotation, out);
    }
}

/// The walk behind `ParsedFileReferences`, or the located cause it could not answer. A binder
/// form the collector cannot name, or an occurrence count that does not reconcile, is a refusal
/// rather than a smaller answer: either would publish a binder set that is known incomplete.
pub(crate) fn parsed_file_references(
    rel: &str,
    content: &str,
    self_module: &str,
    module_names: &HashSet<String>,
) -> Result<ParsedFileReferences, &'static str> {
    let Some(tree) = parse_module_node_tolerant(rel, content) else {
        return Err("parse-failed");
    };
    let mut bare: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut chains: Vec<Vec<String>> = Vec::new();
    // `collect_node_refs` recovers a lambda parameter's name from its authored span, so the
    // file's newline index rides beside the tree; a missing index would leave those names unbound
    // and let them resolve as references.
    let mut file_indices: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    file_indices.insert(
        rel.to_string(),
        super::pool_acquire::newline_index_for(rel, content),
    );
    let file_indices = Rc::new(file_indices);
    // THIS PRODUCER ANSWERS BEFORE ANY SUBJECT IS PREPARED, so it has no declaration index to
    // classify against and cannot decide DeclaredElsewhere / NamesNothingKnown: every unbound
    // name classifies as a reference, and the consumer's census decides what it names.
    let mut scratch_tally: BTreeMap<ExprVarClass, usize> = BTreeMap::new();
    let mut scratch_unclassified: Vec<String> = Vec::new();
    let mut classify = ExprVarClassification {
        decl_index: None,
        module_names: Some(module_names),
        module_path_heads: std::collections::HashSet::new(),
        dotted_head_nodes: std::collections::HashSet::new(),
        tally: &mut scratch_tally,
        unclassified: &mut scratch_unclassified,
        module: self_module.to_string(),
        value_refs: std::collections::BTreeSet::new(),
        occurrences: 0,
        free_reference_edges: 0,
        bound_occurrences_suppressed: 0,
        chain_head_occurrences: 0,
        refusals: Vec::new(),
        bare_positions: Default::default(),
    };
    for item in tree.children.iter() {
        collect_node_refs(item, &mut bare, &mut chains, &file_indices, &mut classify);
    }
    // TWO CAUSES, NOT ONE, because they have different remedies: a binder refusal names a
    // syntax the collector must learn, and a reconciliation miss names an accounting defect in
    // the collector itself (review 55667).
    if !classify.refusals.is_empty() {
        return Err("binder-refusal");
    }
    if !classify.reconciles() {
        return Err("occurrence-accounting-mismatch");
    }
    let positions = std::mem::take(&mut classify.bare_positions);
    let mut authored_types = std::collections::HashSet::new();
    raw_parse_authored_type_names(&tree, &mut authored_types);
    let imports = crate::v1_std_core::module_imports(tree.clone())
        .iter()
        .map(|import| import.name.clone())
        .filter(|path| !path.is_empty())
        .collect();
    Ok(ParsedFileReferences {
        bare,
        chains,
        positions,
        authored_types,
        imports,
        self_declared: module_self_bound_names(&tree),
    })
}

/// THE PER-FILE HALF: one file's reference edges, from that file's own full parse and the pool
/// name index. ONE authority for both demands on it -- the whole-pool producer below maps it over
/// every importer file for the affected-set consumers, and the resolve path asks it only for the
/// files of the closure it compiles (`ModuleGraphFactsLive::reference_only_direct_import_paths`),
/// so a `gunbc run` no longer full-parses the pool to read the edges of a few modules.
pub(crate) fn reference_edges_for_file(
    rel: &str,
    content: Option<&str>,
    names: &ReferencePoolNames,
) -> FileReferenceEdges {
    reference_edges_for_file_on_demand(rel, content, || names)
}

/// The same per-file answer with the pool name index DEMANDED rather than supplied. An unreadable
/// or import-bearing file is decided from its own bytes, so a caller asking about one such file
/// never builds the whole-pool heads index; only an import-less file, whose references must be
/// resolved against pool names, forces it.
pub(crate) fn reference_edges_for_file_on_demand<
    R: std::ops::Deref<Target = ReferencePoolNames>,
>(
    rel: &str,
    content: Option<&str>,
    names: impl FnOnce() -> R,
) -> FileReferenceEdges {
    let Some(content) = content else {
        return FileReferenceEdges::Unaccounted("unreadable");
    };
    if !extract_import_paths(content).is_empty() {
        return FileReferenceEdges::ImportBearing;
    }
    let names = names();
    let names: &ReferencePoolNames = &names;
    let Some(self_module) = extract_module_path(content) else {
        return FileReferenceEdges::Unaccounted("no-module-line");
    };
    let refs = match parsed_file_references(rel, content, &self_module, &names.module_names) {
        Ok(refs) => refs,
        Err(cause) => return FileReferenceEdges::Unaccounted(cause),
    };
    let ParsedFileReferences { bare, chains, .. } = refs;
    // Resolve to per-file (target_module → strongest confidence).
    let mut file_edges: std::collections::BTreeMap<String, RefEdgeResolution> =
        std::collections::BTreeMap::new();
    let mut upgrade = |m: String, res: RefEdgeResolution| {
        let entry = file_edges.entry(m).or_insert(res);
        if res.rank() > entry.rank() {
            *entry = res;
        }
    };
    for chain in &chains {
        if let Some(m) = longest_declared_module_prefix(chain, &names.module_names) {
            if m != self_module {
                upgrade(m, RefEdgeResolution::Qualified);
            }
        }
    }
    for name in &bare {
        // A kernel or container spelling binds no module (`is_substrate_vocabulary`), so
        // it is never an edge: `String` in `std.primitives` once resolved UniqueBare to
        // std.string_type, a module the resolver never loads for that spelling.
        if super::is_substrate_vocabulary(name) {
            continue;
        }
        if let Some(mods) = names.decl_index.get(name) {
            // Same-module declaration wins by lexical scope (namespace-only): a bare name the
            // referencing file itself declares resolves LOCALLY — no cross-module edge. This
            // is what keeps a ubiquitous fixture `data` (e.g. `live_tree_disposition`,
            // declared top-level in ~670 test files) from fanning every referrer out to every
            // declarer.
            if mods.contains(&self_module) {
                continue;
            }
            // Proximity disambiguation (namespace-only "nearest in the containment tree"):
            // among declarers, prefer the one sharing the longest module-path prefix with the
            // referencing module. A single nearest → UniqueBare; a tie at the nearest depth →
            // AmbiguousBare (a genuine homonym the source must qualify — the bright-cat lane).
            let mut best_len = 0usize;
            let mut winners: Vec<&String> = Vec::new();
            for m in mods.iter() {
                let shared = module_prefix_shared_len(&self_module, m);
                if winners.is_empty() || shared > best_len {
                    best_len = shared;
                    winners.clear();
                    winners.push(m);
                } else if shared == best_len {
                    winners.push(m);
                }
            }
            match winners.len() {
                0 => {}
                1 => upgrade(winners[0].clone(), RefEdgeResolution::UniqueBare),
                _ => {
                    // Homonym-qualification worklist dump (bright-cat lane (c) seed): each
                    // AmbiguousBare is a bare ref, in a file that does not declare it, whose
                    // nearest declarers tie — the definitive "needs qualification" site.
                    if std::env::var("REFAMBIG_DUMP").is_ok() {
                        let is_witness = rel.contains("/test/") || rel.ends_with("_test.dag");
                        let cands: Vec<String> = winners.iter().map(|s| (*s).clone()).collect();
                        eprintln!(
                            "REFAMBIG\t{}\t{}\t{}\t{}",
                            if is_witness { "witness" } else { "compile" },
                            rel,
                            name,
                            cands.join(",")
                        );
                    }
                    for t in winners {
                        upgrade(t.clone(), RefEdgeResolution::AmbiguousBare);
                    }
                }
            }
        }
    }
    FileReferenceEdges::Edges(
        file_edges
            .into_iter()
            .map(|(m, res)| ReferenceEdgeRaw {
                path: rel.to_string(),
                target_module: m,
                resolution: res,
            })
            .collect(),
    )
}

/// Reference-derived analogue of `import_resolution_facts`: emit one edge per (file, referenced
/// module). Same row shape channel as import facts, plus a `resolution` confidence tag. Cached by
/// (pool_roots, importer_roots, excludes). The whole-pool map of `reference_edges_for_file`; its
/// demand is the affected-set consumers', which ask about every file.
pub fn reference_resolution_facts(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ReferenceEdgeRaw> {
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let abs_importer_roots = pool_roots_abs(importer_roots);
    let cache_key = format!(
        "{}\u{1f}{}\u{1f}{}",
        abs_pool_roots.join("\u{1e}"),
        abs_importer_roots.join("\u{1e}"),
        exclude_substrings.join("\u{1e}")
    );
    if let Some(cached) = REFERENCE_EDGE_CACHE.with(|c| c.borrow().get(&cache_key).cloned()) {
        shared_fill::record_hit("reference_edges", &cache_key);
        return cached;
    }
    shared_fill::begin_fill();
    let reference_edges_fill_start = std::time::Instant::now();
    let names = reference_pool_names(pool_roots);
    let mut unaccounted: Vec<ReferenceAccountingRefusal> = Vec::new();
    let mut edges: Vec<ReferenceEdgeRaw> = Vec::new();
    for root in &abs_importer_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut files);
        files.sort();
        for file in files {
            let rel = rel_path_for_layer_import(&file);
            if is_excluded_import_path(&rel, exclude_substrings) {
                continue;
            }
            let content = std::fs::read_to_string(&file).ok();
            match reference_edges_for_file(&rel, content.as_deref(), &names) {
                FileReferenceEdges::ImportBearing => {}
                FileReferenceEdges::Edges(file_edges) => edges.extend(file_edges),
                FileReferenceEdges::Unaccounted(cause) => {
                    unaccounted.push(ReferenceAccountingRefusal {
                        path: rel.clone(),
                        cause,
                    })
                }
            }
        }
    }

    unaccounted.sort_by(|a, b| a.path.cmp(&b.path));
    shared_fill::record_fill(
        "reference_edges",
        &cache_key,
        reference_edges_fill_start.elapsed().as_nanos() as u64,
    );
    REFERENCE_UNACCOUNTED_CACHE.with(|c| c.borrow_mut().insert(cache_key.clone(), unaccounted));
    REFERENCE_EDGE_CACHE.with(|c| c.borrow_mut().insert(cache_key, edges.clone()));
    edges
}

#[cfg(test)]
mod live_pool_thread_tests {
    use super::*;

    fn one_module_pool(tag: &str) -> (PathBuf, Vec<String>) {
        // Under the workspace `target/` (gitignored): the module-graph facts normalize every pool
        // path repo-relative and refuse one outside the workspace.
        let root = process_workspace_root().join("target").join(format!(
            "gunbc-live-pool-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("fx")).unwrap();
        std::fs::write(
            root.join("fx/one.dag"),
            "module fx.one\nfn one() -> Int { 1 }\n",
        )
        .unwrap();
        let roots = vec![root.to_string_lossy().into_owned()];
        (root, roots)
    }

    /// THE ROUTE, not only the answer: two separate claims reach ONE index. The discriminating red
    /// is the per-test-thread shape this replaces -- the second call below, run on a fresh thread,
    /// builds a new index with a new generation.
    #[test]
    fn two_claims_on_the_live_pool_thread_share_one_index() {
        let (root, roots) = one_module_pool("route");
        let first = {
            let roots = roots.clone();
            on_live_pool_thread(move || process_shared_index(&roots).generation)
        };
        let second = {
            let roots = roots.clone();
            on_live_pool_thread(move || process_shared_index(&roots).generation)
        };
        let elsewhere = std::thread::spawn(move || process_shared_index(&roots).generation)
            .join()
            .unwrap();
        let _ = std::fs::remove_dir_all(root);
        assert_eq!(first, second, "the live-pool thread kept its index");
        assert_ne!(first, elsewhere, "a fresh thread builds its own");
    }

    /// THE POOL DOES NOT OUTLIVE ITS BLOCK. When another thread builds a pool of its own, the
    /// live-pool thread's pool is released first, so the next claim meets a rebuilt index. The red
    /// this discriminates is a thread that holds its pool under every later test's own pool.
    #[test]
    fn another_threads_pool_build_releases_the_live_pool() {
        let (root, roots) = one_module_pool("release");
        let (other_root, other_roots) = one_module_pool("release-other");
        let generation = |roots: Vec<String>| {
            on_live_pool_thread(move || process_shared_index(&roots).generation)
        };
        let first = generation(roots.clone());
        let held = generation(roots.clone());
        std::thread::spawn(move || {
            process_shared_index(&other_roots);
        })
        .join()
        .unwrap();
        let after_release = generation(roots);
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(other_root);
        assert_eq!(first, held, "back-to-back claims share the pool");
        assert_ne!(
            first, after_release,
            "the pool outlived another thread's build"
        );
    }

    /// A claim that fails on the live-pool thread reds its own test with its own message, and the
    /// thread survives to serve the next claim.
    #[test]
    fn a_panic_on_the_live_pool_thread_reds_the_caller_and_the_thread_survives() {
        let payload = std::panic::catch_unwind(|| {
            on_live_pool_thread::<()>(|| panic!("planted live-pool failure"))
        })
        .expect_err("the planted panic reaches the caller");
        assert_eq!(
            payload.downcast_ref::<&str>().copied(),
            Some("planted live-pool failure")
        );
        assert_eq!(on_live_pool_thread(|| 7), 7);
    }
}

/// THE INSTRUMENT for a cold entry resolve's per-term cost on the live `[dag, src/v2]` pool: one
/// fresh process acquires the pool's tokens, builds the module path index (the heads reading
/// `parse_module_binding` takes), builds the shared index, and resolves two small workflow
/// entries, printing `PROBE` rows and every `pre_entry_phase` term. The first two terms are split
/// out so the per-file token acquisition, which later readings reuse through `pool_acquire`, is
/// not charged to whichever walk happens to run first. It reports; it asserts only that the
/// entries resolve. Run it with
/// `cargo test --release -p v1-compiler --lib live_pool_entry_resolve_attribution -- --ignored --nocapture`.
#[cfg(test)]
mod live_pool_entry_resolve_attribution {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn live_pool_entry_resolve_attribution() {
        let t0 = std::time::Instant::now();
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let t = std::time::Instant::now();
        let mut files = 0usize;
        for r in &roots {
            let mut dag_files = Vec::new();
            collect_dag_files_tolerant(Path::new(r), &mut dag_files);
            for f in dag_files {
                let content = std::fs::read_to_string(&f).expect("read pool file");
                // The spelling `parse_module_binding` acquires under (`source_key`).
                let key = f
                    .strip_prefix(&root)
                    .unwrap_or(&f)
                    .to_string_lossy()
                    .into_owned();
                let _ = super::pool_acquire::tokens_for(&key, &content);
                files += 1;
            }
        }
        eprintln!(
            "PROBE pool token acquisition {:?} files={files}",
            t.elapsed()
        );
        let t = std::time::Instant::now();
        let n = build_module_path_index(&pool_roots_for_module_graph_closure(&roots)).len();
        eprintln!(
            "PROBE module_path_index (heads parse) {:?} modules={n}",
            t.elapsed()
        );
        let t = std::time::Instant::now();
        let index = process_shared_index(&roots);
        eprintln!("PROBE shared_index {:?}", t.elapsed());
        for e in [
            "src/v2/workflow/regen_convergence_transaction.dag",
            "src/v2/workflow/required_regen.dag",
        ] {
            let entry = root.join(e);
            let t = std::time::Instant::now();
            let r = resolve_entry_with_index(&index, &entry.to_string_lossy());
            assert!(r.is_ok(), "{e} resolves");
            eprintln!("PROBE resolve {e} {:?}", t.elapsed());
            eprintln!("PROBE   stages {:?}", resolve_stage_totals());
            for line in super::pre_entry_phase::take_lines() {
                eprintln!("PROBE   phase {line}");
            }
        }
        eprintln!("PROBE total {:?}", t0.elapsed());
    }
}

/// THE IDENTITY DIFFERENTIAL for the closure front end reading its lexical artifact from
/// `pool_acquire` instead of re-lexing: the only input that change alters is the artifact the
/// closure parser receives, so it is compared at that grain over every file of the live
/// `[dag, src/v2]` pool, under the spelling the shared index gives it -- tokens AND the annotation
/// channel, against a fresh `tokenize_artifact` of the same bytes. A divergence names the file.
#[cfg(test)]
mod closure_parse_acquisition_differential {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn pooled_closure_artifacts_equal_fresh_lexing_on_the_live_pool() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let index = process_shared_index(&roots);
        let mut compared = 0usize;
        let mut divergent: Vec<String> = Vec::new();
        for source in index.source_files.values() {
            let pooled = super::pool_acquire::artifact_for(&source.path, &source.content);
            let fresh = v1_compiler_tokenize::tokenize_artifact(
                source.content.clone(),
                source.path.clone(),
                crate::extdeps_languages_dag_syntax::dag_parse_environment(),
            );
            if *pooled != *fresh {
                divergent.push(source.path.clone());
            }
            let pooled_nl = super::pool_acquire::newline_index_for(&source.path, &source.content);
            if *pooled_nl != *build_newline_index(source.path.clone(), source.content.clone()) {
                divergent.push(format!("{} (newline index)", source.path));
            }
            compared += 1;
        }
        eprintln!("DIFF compared={compared} divergent={}", divergent.len());
        assert!(compared > 1000, "the live pool was read ({compared} files)");
        assert!(
            divergent.is_empty(),
            "pooled artifacts diverge: {divergent:?}"
        );
    }
}

/// THE LIVE IDENTITY DIFFERENTIAL for the census projecting rather than re-parsing: every file
/// of the `[dag, src/v2]` shared index, in `pool_parse`'s order, through both readings, compared
/// by `heads_projection_divergences`.
#[cfg(test)]
mod heads_projection_live_differential {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn projected_heads_equal_the_threaded_parse_on_the_live_pool() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let index = process_shared_index(&roots);
        let mut keys: Vec<String> = index.source_files.keys().cloned().collect();
        keys.sort();
        let files: Vec<(String, String)> = keys
            .iter()
            .map(|k| {
                let sf = &index.source_files[k];
                (sf.path.clone(), sf.content.clone())
            })
            .collect();
        let (n, divergent) = super::super::census_heads::heads_projection_divergences(&files);
        eprintln!("DIFF compared={n} divergent={}", divergent.len());
        assert!(n > 1000, "the live pool was read ({n} files)");
        assert!(
            divergent.is_empty(),
            "divergent: {:?}",
            &divergent[..divergent.len().min(20)]
        );
    }
}

/// THE IDENTITY DIFFERENTIAL for the tree census upgrading the memoized raw census instead of
/// rebuilding it: for every source root of the live `[dag, src/v2]` index, the census
/// `tree_bare_census_for_root` now serves agrees with the direct
/// `build_symbol_index_census_nodes(tree_census_nodes(root))` on every field its one production
/// reader, `symbol_index_with_bare_fill`, consumes (bare lookup states and candidates, services,
/// alias reps, exposures), its `entries` are the raw census, and the composed underlay the
/// reconcile builds from it is equal whichever census it is composed from.
#[cfg(test)]
mod tree_census_from_raw_differential {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn tree_census_from_memoized_raw_equals_the_direct_build_on_the_live_pool() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let index = process_shared_index(&roots);
        let pool = super::super::pool_parse(&index).expect("pool parse");
        let mut compared = 0usize;
        for r in index.source_roots.iter() {
            let served = super::super::tree_bare_census_for_root(&index, r).expect("served");
            let nodes = super::super::tree_census_nodes(&index, r).expect("tree nodes");
            let direct =
                v1_compiler_infer::build_symbol_index_census_nodes(nodes, pool.combined_si.clone());
            // Every field the bare fill reads must equal the direct build; `entries` is the raw
            // census, which no production reader of this census consumes.
            let raw = super::super::closure_name_census(&index, Some(r)).expect("raw census");
            let fill_equal = served.global_bare == direct.global_bare
                && served.services == direct.services
                && served.transparent_alias_rep == direct.transparent_alias_rep
                && served.type_head_exposures == direct.type_head_exposures;
            let entries_raw = served.entries == raw.entries;
            let composed_equal =
                *v1_compiler_infer::symbol_index_with_bare_fill(raw.clone(), served.clone())
                    == *v1_compiler_infer::symbol_index_with_bare_fill(raw.clone(), direct.clone());
            eprintln!(
                "DIFF root={r} entries={} bare={} fill_equal={fill_equal} entries_raw={entries_raw} \
                 composed_equal={composed_equal}",
                direct.entries.len(),
                direct.global_bare.len(),
            );
            assert!(fill_equal, "tree census bare fill for {r} diverges");
            assert!(
                entries_raw,
                "tree census entries for {r} are not the raw census"
            );
            assert!(composed_equal, "bare-fill composition for {r} diverges");
            compared += 1;
        }
        assert!(compared >= 2, "both live roots compared ({compared})");
    }
}

thread_local! {
    /// Set by the required floor only, so the byte attribution below reads the floor's own graphs
    /// and no other caller of `resolved_graph_from_sources` pays for it or prints it.
    static FLOOR_BYTE_ATTRIBUTION_ARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn arm_floor_byte_attribution() {
    FLOOR_BYTE_ATTRIBUTION_ARMED.with(|a| a.set(true));
}

fn floor_heap_in_use() -> Option<u64> {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        // SAFETY: mallinfo2 reads allocator bookkeeping and changes nothing.
        let mi = unsafe { libc::mallinfo2() };
        Some((mi.uordblks + mi.hblkhd) as u64)
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        None
    }
}

/// THE TYPED GRAPH'S BYTES BY COMPONENT CLASS, read by FREEING it one class at a time at the point
/// the floor frees it anyway: the strict resolve's refusal path, where the graph is discarded, and
/// the prepared repository's teardown. `floor_retention_census` counts entries, which persistent
/// maps share, so it cannot say what a structure COSTS; the allocator can, as the live bytes a
/// drop returns. Nothing is read after a class is dropped, so no answer the floor gives changes.
///
/// ORDER-DEPENDENT BY CONSTRUCTION, and reported as such. Sequential drops telescope -- the parts
/// sum to the total freed -- but a node two classes share is freed by whichever drops LAST, so a
/// class dropped early reports only its EXCLUSIVE bytes. The type environment is split into its
/// maps and dropped first, so each map's figure is a lower bound on what it alone costs; `shells`
/// is the environments themselves once their maps are held elsewhere.
///
/// A graph, module list or module another owner still holds is not attributable -- dropping it
/// would free nothing -- and is reported as that, never as a zero.
pub(crate) fn typed_graph_byte_attribution(
    label: &str,
    graph: Rc<v1_compiler_compile::ResolvedGraph>,
) {
    if !FLOOR_BYTE_ATTRIBUTION_ARMED.with(|a| a.get()) {
        return;
    }
    let Some(start) = floor_heap_in_use() else {
        eprintln!(
            "[floor-heap] bytes label={label} unattributable: no allocator reading on this target"
        );
        return;
    };
    let graph = match Rc::try_unwrap(graph) {
        Ok(g) => g,
        Err(g) => {
            eprintln!(
                "[floor-heap] bytes label={label} unattributable: the graph has {} other owner(s)",
                Rc::strong_count(&g) - 1
            );
            return;
        }
    };
    let v1_compiler_compile::ResolvedGraph {
        modules,
        item_registry,
        item_leaf_owner_modules,
        diagnostics,
    } = graph;
    let modules: im::Vector<Rc<crate::v1_compiler_infer_items::TypedModule>> = match Rc::try_unwrap(
        modules,
    ) {
        Ok(m) => m,
        Err(m) => {
            eprintln!(
                    "[floor-heap] bytes label={label} unattributable: the module list has {} other owner(s)",
                    Rc::strong_count(&m) - 1
                );
            return;
        }
    };
    let module_count = modules.len();
    let mut shared_modules = Vec::new();
    let (
        mut te_str,
        mut te_anc,
        mut te_bind,
        mut te_vis,
        mut te_ind,
        mut te_sym,
        mut te_intern,
        mut te_unit,
    ) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let (mut shells, mut tec, mut fe, mut iface, mut reg, mut items, mut nodes, mut occ) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    for m in modules {
        match Rc::try_unwrap(m) {
            Ok(m) => {
                te_str.push(m.type_env.str_bindings.clone());
                te_anc.push(m.type_env.ancestry_str_bindings.clone());
                te_bind.push(m.type_env.bindings.clone());
                te_vis.push(m.type_env.source_visible_names.clone());
                te_ind.push(m.type_env.inductive_fields.clone());
                te_sym.push(m.type_env.symbol_index.clone());
                te_intern.push(m.type_env.intern_table.clone());
                te_unit.push(m.type_env.unit_variant_index.clone());
                shells.push(m.type_env);
                tec.push(m.type_env_cache);
                fe.push(m.func_env);
                iface.push(m.interface);
                reg.push(m.item_registry);
                items.push(m.items);
                nodes.push(m.module);
                occ.push(m.occurrence_transport);
            }
            Err(shared) => shared_modules.push(shared),
        }
    }
    let mut last = floor_heap_in_use().unwrap_or(start);
    let mut parts: Vec<(&str, u64)> = Vec::new();
    macro_rules! drop_class {
        ($name:expr, $class:expr) => {{
            drop($class);
            let now = floor_heap_in_use().unwrap_or(last);
            parts.push(($name, last.saturating_sub(now)));
            last = now;
        }};
    }
    drop_class!("type_env_shells", shells);
    drop_class!("te.ancestry_str_bindings", te_anc);
    drop_class!("te.str_bindings", te_str);
    drop_class!("te.bindings", te_bind);
    drop_class!("te.source_visible_names", te_vis);
    drop_class!("te.inductive_fields", te_ind);
    drop_class!("te.unit_variant_index", te_unit);
    drop_class!("te.symbol_index", te_sym);
    drop_class!("te.intern_table", te_intern);
    drop_class!("type_env_cache", tec);
    drop_class!("func_env", fe);
    drop_class!("interface", iface);
    drop_class!("module_item_registry", reg);
    drop_class!("items", items);
    drop_class!("module_nodes", nodes);
    drop_class!("occurrence_transport", occ);
    drop_class!("shared_modules", shared_modules.clone());
    drop_class!("graph_item_registry", item_registry);
    drop_class!("item_leaf_owner_modules", item_leaf_owner_modules);
    drop_class!("diagnostics", diagnostics);
    let sum: u64 = parts.iter().map(|(_, b)| *b).sum();
    eprintln!(
        "[floor-heap] bytes label={label} modules={module_count} shared_modules={} start_in_use={start} \
         end_in_use={last} total_freed={} sum_of_parts={sum} (sequential: early classes are exclusive bytes)",
        shared_modules.len(),
        start.saturating_sub(last),
    );
    // EVERY LINE SAYS WHAT KIND OF READING IT IS, because the figure alone invites the wrong one:
    // a class dropped after others is also credited every node it SHARED with them, so its
    // `freed` is what its last reference kept alive, not what removing it would save. Read as a
    // saving, this probe's `emit_graph_info` figure predicted a peak cut that the floor-memory
    // qualification A/B on gunbc#12832 did not find. Only the first class dropped
    // reads exclusive bytes; the TypeEnv maps split before it are exclusive only after the
    // environment shells went. A removal's saving is a leave-one-out reading, not this one.
    for (order, (name, bytes)) in parts.into_iter().enumerate() {
        let reading = match order {
            0 => "exclusive",
            1..=8 => "exclusive_lower_bound_after_shells",
            _ => "includes_shared_residue",
        };
        eprintln!(
            "[floor-heap] bytes label={label} order={order} class={name} freed={bytes} reading={reading}"
        );
    }
}

/// ONE HEADS PARSE PER (SPELLING, BYTES), asserted: over the live `[dag, src/v2]` pool, building
/// the module path index (`parse_module_binding`) and the pool census (`pool_parse`) -- the two
/// consumers of `pool_acquire::heads_reading_for` -- parses each acquisition key's heads exactly
/// once, and every pool file was read.
#[cfg(test)]
mod heads_parse_count {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn each_pool_file_is_heads_parsed_once_on_the_live_pool() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let _ = build_module_path_index(&pool_roots_for_module_graph_closure(&roots));
        let index = process_shared_index(&roots);
        let _ = super::super::pool_parse(&index).expect("pool parse");
        let (keys, max, over): (usize, usize, Vec<String>) = super::pool_acquire::HEADS_PARSES
            .with(|p| {
                let p = p.borrow();
                (
                    p.len(),
                    p.values().copied().max().unwrap_or(0),
                    p.iter()
                        .filter(|(_, n)| **n > 1)
                        .map(|((f, _, _), n)| format!("{f} x{n}"))
                        .take(20)
                        .collect(),
                )
            });
        eprintln!("HEADS keys={keys} max_parses_per_key={max}");
        assert!(keys > 5000, "the live pool was read ({keys} keys)");
        assert!(over.is_empty(), "heads parsed more than once: {over:?}");
    }
}

/// THE CROSS-TREE CENSUS, over the whole live `[dag, src/v2]` pool: every import-less file's bare
/// references resolved against its own tree census, with no whole-pool census. A name the tree
/// does not provide but another tree does refuses (`CrossTreeBareReference`) exactly where the
/// deleted fallback silently pulled a provider; this lists every such refusal, so the pool's
/// dependence on the deleted fallback is counted, by identity, rather than argued. Before the
/// change the fallback census found 13 pool-provided rows in 5 files: 12 the import migration in
/// this change qualifies, and the builtin `get` the builtin arm now keeps from being pulled.
#[cfg(test)]
mod cross_tree_bare_census {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn no_import_less_file_reaches_across_trees_on_the_live_pool() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let index = process_shared_index(&roots);
        let mut sources: Vec<_> = index.source_files.values().cloned().collect();
        sources.sort_by(|a, b| a.path.cmp(&b.path));
        let mut scanned = 0usize;
        let mut refusals: Vec<String> = Vec::new();
        for sf in &sources {
            if super::super::source_declares_import_lines(&sf.content) {
                continue;
            }
            scanned += 1;
            let r = super::super::visit_bare_reference_providers(
                sf,
                &index,
                |root| super::super::closure_name_census(&index, root),
                |_, _, _, _| Ok(()),
            );
            if let Err(e) = r {
                refusals.push(e);
            }
        }
        let pool_census_built = index.closure_name_censuses.borrow().contains_key(&None);
        eprintln!(
            "CROSSTREE scanned={scanned} refusals={} pool_census_built={pool_census_built}",
            refusals.len()
        );
        for r in &refusals {
            eprintln!("CROSSTREE refusal {r}");
        }
        assert!(scanned > 100, "the live pool was read ({scanned} files)");
        assert!(
            !pool_census_built,
            "a bare resolution still built the whole-pool census"
        );
        assert!(
            refusals.is_empty(),
            "{} files reach across trees",
            refusals.len()
        );
    }
}

#[cfg(test)]
mod cross_tree_bare_reference_tests {
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).expect("mkdir");
        std::fs::write(&p, content).expect("write dag");
    }

    /// Two source trees. `a/user.dag` references `helper`, declared only in tree `b`, and a builtin
    /// `get`, which tree `b` also declares as an ordinary function. Returns the admission verdict
    /// of `a/user.dag`.
    fn admit_user(tag: &str, user_imports: &str, call_helper: bool) -> Result<(), String> {
        let base = process_workspace_root()
            .join("target")
            .join(format!("gunbc-crosstree-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (a, b) = (base.join("a"), base.join("b"));
        let main_body = if call_helper { "helper()" } else { "1" };
        write(
            &b,
            "helper.dag",
            "module tb.helper\n\nfn helper() -> Int {\n  1\n}\n\nfn get(x: Int) -> Int {\n  x\n}\n",
        );
        write(
            &a,
            "user.dag",
            &format!(
                "module ta.user\n{user_imports}\nfn main() -> Int {{\n  {main_body}\n}}\n\nfn first() -> Bool {{\n  match get(xs: [1, 2], index: 0) {{\n    Present {{ value: _ }} => true\n    Absent => false\n  }}\n}}\n"
            ),
        );
        let roots = vec![
            a.to_string_lossy().into_owned(),
            b.to_string_lossy().into_owned(),
        ];
        let index = build_multi_entry_index(&roots);
        let user = index
            .source_files
            .values()
            .find(|sf| sf.path.ends_with("a/user.dag"))
            .cloned()
            .expect("user source indexed");
        let verdict = super::super::admit_bare_references_of_file(&index, &user);
        let _ = std::fs::remove_dir_all(&base);
        verdict
    }

    fn admit_user_qualified(tag: &str) -> Result<(), String> {
        let base = process_workspace_root()
            .join("target")
            .join(format!("gunbc-crosstree-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (a, b) = (base.join("a"), base.join("b"));
        write(
            &b,
            "helper.dag",
            "module tb.helper\n\nfn helper() -> Int {\n  1\n}\n",
        );
        write(
            &a,
            "user.dag",
            "module ta.user\n\nfn main() -> Int {\n  tb.helper.helper()\n}\n",
        );
        let roots = vec![
            a.to_string_lossy().into_owned(),
            b.to_string_lossy().into_owned(),
        ];
        let index = build_multi_entry_index(&roots);
        let user = index
            .source_files
            .values()
            .find(|sf| sf.path.ends_with("a/user.dag"))
            .cloned()
            .expect("user source indexed");
        let verdict = super::super::admit_bare_references_of_file(&index, &user);
        let _ = std::fs::remove_dir_all(&base);
        verdict
    }

    /// THE RED: an unimported bare reference to a name only another source tree declares refuses,
    /// typed and located, where the deleted pool fallback silently resolved it.
    #[test]
    fn an_unimported_cross_tree_bare_name_refuses() {
        let err = admit_user("red", "", true).expect_err("a cross-tree bare reference must refuse");
        assert!(err.contains("CrossTreeBareReference"), "{err}");
        assert!(err.contains("'helper'"), "{err}");
        assert!(err.contains("`tb.helper.helper`"), "{err}");
    }

    /// A BUILTIN IS NOT A CROSS-TREE REFERENCE: `get` is the builtin even though tree `b` declares
    /// an ordinary `fn get`. The deleted fallback pulled that function in; the rule neither pulls
    /// it nor refuses.
    #[test]
    fn a_builtin_named_like_another_trees_function_admits() {
        admit_user("builtin", "", false).expect("the builtin get admits without a provider");
    }

    /// The positive control: the same reference written qualified (`tb.helper.helper()`) admits,
    /// and the file stays import-less, so its other bare references are still followed.
    #[test]
    fn the_same_reference_written_qualified_admits() {
        admit_user_qualified("green").expect("a qualified cross-tree reference admits");
    }
}

/// The five files the cross-tree census enumerated resolve as entries over the live pool after
/// the migration: the four that now reference across trees qualified, and the builtin `get` claim that no
/// longer pulls another tree's `fn get`.
#[cfg(test)]
mod cross_tree_migrated_entries_resolve {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn migrated_entries_resolve_on_the_live_pool() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let index = process_shared_index(&roots);
        for e in [
            "src/v2/test/claim/auth_declared_but_unwired_witness_test.dag",
            "src/v2/test/claim/bootstrap_test.dag",
            "src/v2/test/claim/infer_semantics_witness_test.dag",
            "src/v2/test/claim/manual/path_y_fidelity_successor_test.dag",
            "dag/test/claim/builtin_get_resolver_test.dag",
        ] {
            let entry = root.join(e);
            let r = resolve_entry_with_index(&index, &entry.to_string_lossy());
            eprintln!("MIGRATED {e} ok={}", r.is_ok());
            if let Err(err) = &r {
                eprintln!("MIGRATED   {}", err.chars().take(600).collect::<String>());
            }
            assert!(r.is_ok(), "{e} resolves");
        }
    }
}

/// THE PER-NAME CLAIM, for every name of the live pool: the whole-pool name census's entry for a
/// name (bare lookup state with candidates, and service entry) equals the entry
/// `pool_census_for_name` builds over the name's declaring modules alone. This is what licenses
/// answering the loader's out-of-tree question without building the pool census.
#[cfg(test)]
mod pool_census_for_name_differential {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn per_name_census_equals_the_pool_census_for_every_name_on_the_live_pool() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let index = process_shared_index(&roots);
        let pool = super::super::closure_name_census(&index, None).expect("pool census");
        let decl = reference_pool_names_for_index(&index).expect("names");
        let mut names: BTreeSet<String> = BTreeSet::new();
        names.extend(v1_rt::sorted_map_keys(&pool.global_bare));
        names.extend(v1_rt::sorted_map_keys(&pool.services));
        names.extend(decl.decl_index.keys().cloned());
        let mut divergent: Vec<String> = Vec::new();
        {
            for name in &names {
                let local =
                    super::super::pool_census_for_name(&index, name).expect("per-name census");
                if v1_rt::map_get(&pool.global_bare, name.clone())
                    != v1_rt::map_get(&local.global_bare, name.clone())
                {
                    divergent.push(format!("{name} (bare)"));
                }
                if v1_rt::map_get(&pool.services, name.clone())
                    != v1_rt::map_get(&local.services, name.clone())
                {
                    divergent.push(format!("{name} (service)"));
                }
            }
        }
        eprintln!(
            "PERNAME names={} divergent={}",
            names.len(),
            divergent.len()
        );
        for d in divergent.iter().take(30) {
            eprintln!("PERNAME divergent {d}");
        }
        assert!(names.len() > 1000, "the live pool was read");
        assert!(divergent.is_empty(), "{} names diverge", divergent.len());
    }
}

/// THE SPECIMEN of `gunbc.recurring_failure_mode.an_import_turns_an_ambiguous_bare_name_into_a_transitive_pick`,
/// a v1 semantic defect owned by the resolver lane (routed by neat-boar-16), not by this change.
/// One source tree declares `bar` in `ta.dep` and `ta.other`. With no imports a bare `bar()`
/// refuses as ambiguous; with one unrelated import whose module imports `ta.dep`, the same call
/// RESOLVES -- silently binding `bar` to the transitively reached `ta.dep.bar`. This test pins
/// that behaviour as observed. WHEN THE DEFECT IS FIXED IT MUST FAIL: flip its second assertion
/// to expect the ambiguity refusal and keep it as the regression control (DESIGN §4b(4)).
#[cfg(test)]
mod import_transitive_bare_pick_specimen {
    use super::*;

    fn w(root: &Path, rel: &str, c: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).expect("mkdir");
        std::fs::write(p, c).expect("write dag");
    }

    #[test]
    fn an_unrelated_import_turns_an_ambiguous_bare_name_into_a_transitive_pick() {
        let base = process_workspace_root()
            .join("target")
            .join(format!("gunbc-import-pick-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let a = base.join("a");
        w(
            &a,
            "dep.dag",
            "module ta.dep\n\nfn bar() -> Int {\n  1\n}\n\nfn z() -> Int {\n  0\n}\n",
        );
        w(
            &a,
            "other.dag",
            "module ta.other\n\nfn bar() -> Int {\n  2\n}\n",
        );
        w(
            &a,
            "lib.dag",
            "module ta.lib\n\nimport ta.dep { z }\n\nfn y() -> Int {\n  z()\n}\n",
        );
        w(
            &a,
            "bare_user.dag",
            "module ta.bare_user\n\nfn main() -> Int {\n  bar()\n}\n",
        );
        w(
            &a,
            "import_user.dag",
            "module ta.import_user\n\nimport ta.lib { y }\n\nfn main() -> Int {\n  bar()\n}\n",
        );
        let index = build_multi_entry_index(&[a.to_string_lossy().into_owned()]);
        let bare = resolve_entry_with_index(&index, &a.join("bare_user.dag").to_string_lossy());
        let imported =
            resolve_entry_with_index(&index, &a.join("import_user.dag").to_string_lossy());
        let _ = std::fs::remove_dir_all(&base);
        let err = bare.expect_err("with no imports, an ambiguous bare name refuses");
        assert!(err.contains("ambiguous reference 'bar'"), "{err}");
        // THE DEFECT, pinned as observed: flip to expect_err when it is fixed.
        imported.expect("observed: one unrelated import makes the same bare name resolve");
    }
}

/// THE INSTRUMENT for the floor's whole-pool bare admission (`admit_pool_bare_references`) on the
/// live `[dag, src/v2]` pool, with what both routes share -- the pool heads parse, both tree name
/// censuses and the heads name index -- warmed first, so the timed term is the admission alone.
/// It prints the admission time, the per-name census calls and time (`ResolveStageNanos`) and the
/// distinct names asked; calls against distinct names is the repetition a shared answer would
/// remove. It reports; it asserts only that the admission completes.
#[cfg(test)]
mod live_pool_bare_admission_attribution {
    use super::*;
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn live_pool_bare_admission_attribution() {
        let root = process_workspace_root();
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| root.join(r).to_string_lossy().into_owned())
            .collect();
        let index = process_shared_index(&roots);
        let _ = super::super::pool_parse(&index).expect("pool parse");
        for r in index.source_roots.iter() {
            let _ = super::super::closure_name_census(&index, Some(r)).expect("tree census");
        }
        let _ = reference_pool_names_for_index(&index).expect("names");
        let before = resolve_stage_totals();
        let t = std::time::Instant::now();
        let verdict = super::super::admit_pool_bare_references(&index);
        let elapsed = t.elapsed();
        let after = resolve_stage_totals();
        let distinct = super::super::PER_NAME_CENSUS_DISTINCT.with(|d| d.borrow().len());
        eprintln!(
            "ADMIT whole_pool_admission={elapsed:?} ok={} per_name_calls={} per_name_ms={} \
             per_name_distinct={distinct} pool_census_built={}",
            verdict.is_ok(),
            after.bare_per_name_census_calls - before.bare_per_name_census_calls,
            (after.bare_per_name_census - before.bare_per_name_census) / 1_000_000,
            index.closure_name_censuses.borrow().contains_key(&None),
        );
        verdict.expect("the live pool admits");
    }
}

/// THE MODULE-LEVEL CLASSES A LEAVE-ONE-OUT READING CAN REMOVE WHOLE. Each is one field of every
/// `TypedModule`, dropped for ALL modules at once, so a structure one module links from another
/// (a TypeEnv parent, an interface import) goes with its class rather than surviving through the
/// link. A field INSIDE one of these (a TypeEnv map) cannot be removed this way: every environment
/// holds its own copy and other environments reach it through their parent links, so its exclusive
/// bytes are not observable by dropping, and its enclosing class's figure is their upper bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TypedModuleClass {
    TypeEnv,
    TypeEnvCache,
    FuncEnv,
    Interface,
    ModuleItemRegistry,
    Items,
    ModuleNodes,
    OccurrenceTransport,
}

impl TypedModuleClass {
    pub(crate) const ALL: [TypedModuleClass; 8] = [
        TypedModuleClass::TypeEnv,
        TypedModuleClass::TypeEnvCache,
        TypedModuleClass::FuncEnv,
        TypedModuleClass::Interface,
        TypedModuleClass::ModuleItemRegistry,
        TypedModuleClass::Items,
        TypedModuleClass::ModuleNodes,
        TypedModuleClass::OccurrenceTransport,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            TypedModuleClass::TypeEnv => "type_env",
            TypedModuleClass::TypeEnvCache => "type_env_cache",
            TypedModuleClass::FuncEnv => "func_env",
            TypedModuleClass::Interface => "interface",
            TypedModuleClass::ModuleItemRegistry => "module_item_registry",
            TypedModuleClass::Items => "items",
            TypedModuleClass::ModuleNodes => "module_nodes",
            TypedModuleClass::OccurrenceTransport => "occurrence_transport",
        }
    }
}

/// One leave-one-out drop, as raw readings: the allocator's live bytes with every class held,
/// after dropping ONE class first, and after dropping the rest. What they mean -- the class's
/// exclusive bytes, the graph's total, what the classes share -- is decided by
/// `gunbc.typed_graph_exclusive_bytes` `typed_graph_exclusive_report`; this reader only reads. A
/// graph or module list another owner keeps refuses, because dropping it would free nothing.
pub(crate) struct ExclusiveBytesReading {
    /// How many classes were dropped together: 1 for a single class, more for a joint set.
    pub members: usize,
    pub modules: usize,
    pub in_use_all: u64,
    pub in_use_after_class: u64,
    pub in_use_end: u64,
    pub ancestry_entries: u64,
    pub own_entries: u64,
}

/// Drops every class in `classes` first, together, while every other class is held: one class gives
/// that class's exclusive bytes; a set gives the bytes the set holds JOINTLY -- what removing all of it
/// would save, including nodes its members share with each other and with nothing else.
pub(crate) fn typed_module_class_exclusive_bytes(
    graph: Rc<v1_compiler_compile::ResolvedGraph>,
    classes: &[TypedModuleClass],
) -> Result<ExclusiveBytesReading, String> {
    let graph = Rc::try_unwrap(graph)
        .map_err(|g| format!("the graph has {} other owner(s)", Rc::strong_count(&g) - 1))?;
    let modules = Rc::try_unwrap(graph.modules).map_err(|m| {
        format!(
            "the module list has {} other owner(s)",
            Rc::strong_count(&m) - 1
        )
    })?;
    let module_count = modules.len();
    let ancestry_entries: u64 = modules
        .iter()
        .map(|m| m.type_env.ancestry_str_bindings.len() as u64)
        .sum();
    let own_entries: u64 = modules
        .iter()
        .map(|m| m.type_env.str_bindings.len() as u64)
        .sum();
    let mut owned = Vec::with_capacity(module_count);
    for m in modules {
        owned.push(Rc::try_unwrap(m).map_err(|m| {
            format!(
                "module {} has {} other owner(s)",
                m.type_env.module_path,
                Rc::strong_count(&m) - 1
            )
        })?);
    }
    // Every class of every module is moved into its own column, so the chosen column is the only
    // thing dropped and the rest -- including the graph-level registry and diagnostics -- stays held.
    let mut chosen: Vec<Box<dyn std::any::Any>> = Vec::with_capacity(module_count);
    let mut kept: Vec<Box<dyn std::any::Any>> = Vec::with_capacity(module_count * 8);
    for m in owned {
        let fields: [(TypedModuleClass, Box<dyn std::any::Any>); 8] = [
            (TypedModuleClass::TypeEnv, Box::new(m.type_env)),
            (TypedModuleClass::TypeEnvCache, Box::new(m.type_env_cache)),
            (TypedModuleClass::FuncEnv, Box::new(m.func_env)),
            (TypedModuleClass::Interface, Box::new(m.interface)),
            (
                TypedModuleClass::ModuleItemRegistry,
                Box::new(m.item_registry),
            ),
            (TypedModuleClass::Items, Box::new(m.items)),
            (TypedModuleClass::ModuleNodes, Box::new(m.module)),
            (
                TypedModuleClass::OccurrenceTransport,
                Box::new(m.occurrence_transport),
            ),
        ];
        for (k, v) in fields {
            if classes.contains(&k) {
                chosen.push(v);
            } else {
                kept.push(v);
            }
        }
    }
    let in_use_all = floor_heap_in_use().ok_or("no allocator reading on this target")?;
    drop(chosen);
    let after = floor_heap_in_use().ok_or("no allocator reading on this target")?;
    drop(kept);
    drop(graph.item_registry);
    drop(graph.diagnostics);
    let end = floor_heap_in_use().ok_or("no allocator reading on this target")?;
    Ok(ExclusiveBytesReading {
        members: classes.len(),
        modules: module_count,
        in_use_all,
        in_use_after_class: after,
        in_use_end: end,
        ancestry_entries,
        own_entries,
    })
}

/// THE STRICT REFUSAL COUNTS ONLY WHAT BLOCKS, AND NAMES THE MODULE. One fixture carries exactly one
/// blocking diagnostic and one advisory (a call through a function value, reported as a lower-bound
/// effect summary with non-error severity). The refusal must head its list with
/// `blocking_diagnostics=1`, print that one line with its module, and leave the advisory out -- and
/// the advisory must really have been raised, or its absence would prove nothing.
#[cfg(test)]
mod strict_refusal_counts_blocking_diagnostics {
    use super::*;

    const FIXTURE: &str = "module refusal_count_fixture\n\
        fn host(agree: fn(Int, Int) -> Bool) -> Bool { agree(1, 2) }\n\
        fn broken(x: NoSuchDeclaredType) -> Int { 1 }\n";

    fn fixture_sources() -> Vec<Rc<v1_compiler_compile::SourceFile>> {
        vec![Rc::new(v1_compiler_compile::SourceFile {
            path: "refusal_count_fixture.dag".to_string(),
            content: FIXTURE.to_string(),
        })]
    }

    #[test]
    fn one_blocker_is_counted_and_named_and_the_advisory_is_not() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let raw = v1_compiler_compile::compile_to_resolved(Rc::new(
                    fixture_sources().into(),
                ));
                let advisories = raw
                    .diagnostics
                    .iter()
                    .filter(|d| {
                        !is_interpreter_blocking_diagnostic(d.diagnostic.clone())
                    })
                    .count();
                assert!(
                    advisories >= 1,
                    "the fixture must raise a non-blocking diagnostic for its absence to mean anything: {:?}",
                    raw.diagnostics
                );
                let refusal =
                    match resolved_graph_from_sources(fixture_sources()) {
                        Err(text) => text,
                        Ok(_) => panic!("a fixture with a blocking diagnostic must refuse"),
                    };
                let lines: Vec<&str> = refusal.lines().collect();
                assert_eq!(lines.first(), Some(&"blocking_diagnostics=1"), "{refusal}");
                let error_lines: Vec<&&str> =
                    lines.iter().filter(|l| l.contains(": error: ")).collect();
                assert_eq!(error_lines.len(), 1, "{refusal}");
                assert!(
                    error_lines[0].starts_with("refusal_count_fixture.dag:")
                        && error_lines[0].contains(": error: [module refusal_count_fixture] "),
                    "the blocker must carry its location and its module: {refusal}"
                );
                assert!(
                    !refusal.contains("effect summary incomplete"),
                    "the advisory must not be printed or counted: {refusal}"
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("one_blocker_is_counted_and_named_and_the_advisory_is_not panicked");
    }
}
