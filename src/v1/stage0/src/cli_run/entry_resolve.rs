// Split from cli_run.rs (pure code motion; no semantic change).
#![allow(unused_imports)]
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
    has_child_named, inferred_to_node, intern, is_discovery_corpus_blocking_diagnostic,
    is_error_diagnostic, is_interpreter_blocking_diagnostic, let_binding_name_at, let_value,
    make_error_node, match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver,
    module_items, no_span, param_node_name_at, param_node_type_expr, Cardinality,
    CompilerDiagnostic, Connective, ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable,
    MatchPattern, NewlineIndex, Node,
};
use serde::Serialize;

pub fn build_module_path_index(source_roots: &[String]) -> HashMap<String, String> {
    let key = source_roots
        .iter()
        .map(|r| anchor_source_root(r))
        .collect::<Vec<_>>()
        .join("\u{1f}");
    MODULE_PATH_INDEX_CACHE.with(|cache| {
        if let Some(index) = cache.borrow().get(&key) {
            shared_fill::record_hit("module_path_index", &key);
            return index.clone();
        }
        shared_fill::begin_fill();
        let start = std::time::Instant::now();
        let index = build_module_path_index_uncached(source_roots);
        shared_fill::record_fill("module_path_index", &key, start.elapsed().as_nanos() as u64);
        cache.borrow_mut().insert(key, index.clone());
        index
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
    let observation = import_resolution_facts_with_observation(&roots, &roots, EXCLUDE);
    let edges = observation.facts;
    let nodes = module_declaration_facts(&roots);
    // Loader tier: import edges only, unchanged. Every consumer that goes on to RESOLVE what it
    // reaches reads this one.
    let adjacency = build_import_adjacency(&edges, &nodes);
    // Selection tier: import edges + strict reference edges.
    let mut selection_edges = edges.clone();
    selection_edges.extend(reference_edges_as_import_facts(
        &reference_resolution_facts(&roots, &roots, EXCLUDE),
        /* strict */ true,
    ));
    let selection_adjacency = build_import_adjacency(&selection_edges, &nodes);
    let reference_unaccounted: HashSet<String> =
        reference_accounting_refusals(&roots, &roots, EXCLUDE)
            .into_iter()
            .map(|r| workspace_relative_repo_path(&r.path))
            .collect();
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
        selection_adjacency,
        declared_paths,
        reference_unaccounted,
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
// census population, and the silent-pick gate -- and the first draft of this phase had
// already drifted on the first of them.
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
// the CLI already stops on (a blocking diagnostic, or an empty emitted file set), plus
// the CLI's own silent-pick gate. A gate that refused on ANY diagnostic would be
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
    let sources = load_sources_for_entry_with_index(
        &index.source_files,
        &index.module_graph_facts,
        entry_path,
    )?;
    let sources = extend_sources_to_both_closure_fixpoint(sources, index)?;
    index
        .entry_closure_sources
        .borrow_mut()
        .insert(cache_key, sources.clone());
    Ok(sources)
}

pub(crate) fn load_sources_for_entry_with_index(
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
    let mut sources = extend_with_reference_closure(sources, index, facts)?;
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    sources.dedup_by(|a, b| a.path == b.path);
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
    // source_roots — a run's roots are fixed, so this is a get-or-build, rebuilt only on
    // the rare roots change. Thread-local by the same Rc-not-Send reason as the store:
    // each shard keeps its own index rather than smuggling Rc across threads.
    //
    // TWO SLOTS, ONE PER POOL SEMANTICS, and the pair is what makes the memo safe rather than
    // merely faster. `primary-precedence` (root[0] wins, later roots fill only absent modules)
    // and strict are DIFFERENT POOLS over the same roots: serving one where the other was asked
    // for is a silently divergent resolution, which is the §5 fail-open this cache would
    // otherwise introduce. So precedence is part of the identity of the slot, not a build flag
    // applied to a shared one -- a roots-keyed single slot cannot express the distinction and
    // would answer whichever mode ran first. Each slot keeps the original single-entry,
    // rebuild-on-roots-change shape, so an index is never held for a pool nobody is asking about.
    #[allow(clippy::type_complexity)]
    pub(crate) static PROCESS_RESOLVE_INDEX: RefCell<[Option<(String, Rc<MultiEntryIndex>)>; 2]> =
        const { RefCell::new([None, None]) };

    // While loading the materialization-provider authority, cross-process disk hits
    // must not re-enter provider routing (review 44268: bootstrap recursion).
    pub(crate) static CROSS_PROCESS_PROVIDER_ROUTING_SUPPRESSED: Cell<usize> = const { Cell::new(0) };
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
pub fn process_shared_index(source_roots: &[String]) -> Rc<MultiEntryIndex> {
    try_process_shared_index(source_roots).unwrap_or_else(|e| panic!("{e}"))
}

/// Fallible twin of `process_shared_index`. The MEMO IS ONLY WRITTEN ON SUCCESS -- a failed
/// discovery must not install a partial index that every later caller in the process would
/// then read as complete.
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
pub fn try_process_shared_index_for_pool(
    source_roots: &[String],
    primary_precedence: bool,
) -> Result<Rc<MultiEntryIndex>, String> {
    let slot = usize::from(primary_precedence);
    let roots = canonical_shared_index_roots(source_roots);
    let roots_key = roots.join("\u{1f}");
    let existing = PROCESS_RESOLVE_INDEX.with(|s| {
        s.borrow()[slot].as_ref().and_then(|(k, idx)| {
            if *k == roots_key {
                Some(idx.clone())
            } else {
                None
            }
        })
    });
    if let Some(idx) = existing {
        return Ok(idx);
    }
    let build_started = std::time::Instant::now();
    let module_index = if primary_precedence {
        try_build_module_index_primary_precedence(&roots)?
    } else {
        try_build_module_index(&roots)?
    };
    let idx = Rc::new(new_multi_entry_index_shell(module_index, &roots, None));
    discovery_phase_totals::add(
        &discovery_phase_totals::SHARED_INDEX_BUILD_MS,
        build_started.elapsed(),
    );
    PROCESS_RESOLVE_INDEX.with(|s| {
        s.borrow_mut()[slot] = Some((roots_key, idx.clone()));
    });
    Ok(idx)
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

pub(crate) fn new_multi_entry_index_shell(
    source_files: ModuleSourceIndex,
    source_roots: &[String],
    cross_worker_store: Option<Arc<RwLock<SharedTypecheckCaches>>>,
) -> MultiEntryIndex {
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
        pool_qualified_fill: RefCell::new(None),
        tree_bare_census: RefCell::new(std::collections::HashMap::new()),
        pool_bare_census: RefCell::new(None),
        entry_closure_sources: RefCell::new(HashMap::new()),
        both_closure_edges: RefCell::new(None),
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
    // The bound the executor knows and the process cannot see is exactly what the env
    // override is for. Panicking here is a hard
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
             at rc=137 instead. Declare this slot's bound with GUNBC_MEMORY_BUDGET_BYTES, or \
             model this platform's memory source \
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
    resolve_entry_with_parse_cache(index, entry_file, ResolveTypecheckGate::Strict)
}

pub fn resolve_entry_with_index_for_discovery_corpus(
    index: &MultiEntryIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    resolve_entry_with_parse_cache(
        index,
        entry_file,
        ResolveTypecheckGate::DiscoveryCorpusAdvisory,
    )
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
    typecheck_gate: ResolveTypecheckGate,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let depth = resolve_span_enter();
    let span_started = std::time::Instant::now();
    let out = resolve_entry_with_parse_cache_inner(index, entry_file, typecheck_gate);
    resolve_span_exit(depth, entry_file, span_started.elapsed().as_nanos());
    out
}

pub(crate) fn resolve_entry_with_parse_cache_inner(
    index: &MultiEntryIndex,
    entry_file: &str,
    typecheck_gate: ResolveTypecheckGate,
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
        typecheck_gate,
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
    let artifact =
        v1_compiler_tokenize::tokenize_artifact(source.content.clone(), source.path.clone());
    let nl_index = build_newline_index(source.path.clone(), source.content.clone());
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
/// typecheck refusals use `is_resolve_typecheck_blocking(typecheck_gate)` and the
/// other stages use `is_error_diagnostic` — for the gate's `Strict` mode both reduce
/// to the `00_core.dag` interpreter-blocking authority on every class those stages
/// can produce (`ComplexityUnknown`, the sole class where the predicates differ, is
/// only produced by complexity analysis, which does not run on this path).
pub(crate) fn resolved_graph_from_sources_with_index(
    index: &MultiEntryIndex,
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    typecheck_gate: ResolveTypecheckGate,
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
    let subject = subject_digest_for_closure(&sources);
    // In-process share tier (resolved_graph_memo): always on — the ReferenceTier in
    // front of the opt-in cross-process store. A subject this process has already
    // assembled is served by reference, eliminating the per-entry reconcile assembly
    // residue on re-resolve (Track A denomination receipt, resolve-split #6535).
    if let Some((graph, si, compile_clean_diags)) = index.resolved_graph_memo.borrow().get(&subject)
    {
        return Ok((graph.clone(), si.clone(), compile_clean_diags.clone()));
    }
    // Cross-process store tier: opt-in via `GUNBC_RESOLVED_GRAPH_CACHE_DIR` only.
    // Installs into the share above on hit so later same-subject demands never
    // re-decode. Floor/CI leave it unset (mechanism-inventory-red-controls: inert
    // on floor); only explicit test harnesses arm the disk tier.
    if let Some(cache_root) = resolved_graph_cache_root_from_env() {
        if !cross_process_provider_routing_suppressed() {
            match cross_process_probe(&cache_root, &subject) {
                CacheProbeResult::Hit(probe) => {
                    if !supports_faithful_probe() {
                        return Err(format!(
                            "resolved-graph-cache provider refused faithful probe: {}",
                            faithful_probe_unavailable_gap()
                        ));
                    }
                    let parts = &probe.parts;
                    let closure_digest = closure_content_digest(&sources);
                    let compiler_digest = transform_content_digest();
                    match materialization_provider_consumer::serve_resolved_graph_stored_disk_probe(
                        &closure_digest,
                        &compiler_digest,
                        &probe.stored_request_key,
                        &probe.stored_semantic_digest,
                        parts,
                    ) {
                        Ok(ResolvedGraphProviderOutcome::Hit) => {
                            match cross_process_lookup_verified_probe(&cache_root, &subject, &probe)
                            {
                                CacheLookupResult::Hit(cached) => {
                                    return Ok(install_cross_process_materialization_hit(
                                        index, &subject, cached, memo_share,
                                    ));
                                }
                                CacheLookupResult::RejectedHit(reason) => {
                                    return Err(cross_process_cache_integrity_refusal(reason));
                                }
                                CacheLookupResult::Miss => {
                                    return Err(
                                        "resolved-graph-cache lookup miss after provider hit"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                        Ok(other) => {
                            if let Some(msg) = provider_integrity_refusal_message(other) {
                                return Err(msg);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                CacheProbeResult::LegacyMigrationRequired { .. } => {
                    // Cold rebuild is the declared migration disposition — never route
                    // legacy on-disk rows through the v3 provider probe.
                }
                CacheProbeResult::RejectedHit(CacheRejectReason::ContentDigestMismatch) => {
                    return Err(cross_process_cache_integrity_refusal(
                        CacheRejectReason::ContentDigestMismatch,
                    ));
                }
                CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed) => {
                    return Err(cross_process_cache_integrity_refusal(
                        CacheRejectReason::BackendKeyMalformed,
                    ));
                }
                CacheProbeResult::RejectedHit(CacheRejectReason::PartDecodeFailure) => {
                    return Err(cross_process_cache_integrity_refusal(
                        CacheRejectReason::PartDecodeFailure,
                    ));
                }
                CacheProbeResult::Miss => {}
            }
        }
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
        .any(|d| is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate));
    if has_type_errors {
        let msgs: Vec<String> = typed
            .diagnostics
            .iter()
            .filter(|d| is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate))
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
    // The store direction of the seam obeys the SAME bootstrap suppression as the
    // read direction. The flag names a window in which provider routing may not be
    // re-entered at all; honouring it on the probe alone left the store calling
    // `resolve_closure_request_key_from_digests` while the provider ctx was still
    // mid-construction, so `materialization_provider_ctx` saw an empty memo slot and
    // rebuilt the whole provider closure — a nested resolve that re-entered this same
    // store, unbounded, ~1GB per level (repeat-resolve OOM, root-caused 2026-08-03).
    // Counted, never silent: a suppressed store is a bounded bootstrap-window skip
    // whose frequency stays observable (§5 — a failure arm must refuse, never widen).
    if cross_process_provider_routing_suppressed() {
        record_provider_bootstrap_store_skip();
    } else if let Some(cache_root) = resolved_graph_cache_root_from_env() {
        // A failed store write is a disclosed refusal, never a silent shrug —
        // the swallowed error hid that big closures never landed on disk (only
        // the prelude artifact ever existed), which mis-shaped a whole OOM
        // investigation (receipt: eager-ram-612 bisect, 2026-07-10).
        let closure_digest = closure_content_digest(&sources);
        let compiler_digest = transform_content_digest();
        let encoded = match cross_process_prepare(
            &cache_root,
            &subject,
            &typed,
            source_indices.as_ref(),
            &compile_clean_diags,
        ) {
            Ok(encoded) => encoded,
            Err(e) => {
                eprintln!("[resolved-graph-cache] prepare refused subject={subject}: {e}");
                return Ok((typed, source_indices, compile_clean_diags));
            }
        };
        let stored_request_key =
            resolve_closure_request_key_from_digests(&closure_digest, &compiler_digest)?;
        let stored_semantic_digest = resolved_graph_parts_semantic_digest(
            &encoded.graph_digest,
            encoded.graph_bytes,
            &encoded.indices_digest,
            encoded.indices_bytes,
            &encoded.union_digest,
            encoded.union_bytes,
        )?;
        if let Err(e) = cross_process_write_prepared(
            &cache_root,
            &subject,
            encoded,
            &stored_request_key,
            &stored_semantic_digest,
        ) {
            eprintln!("[resolved-graph-cache] write refused subject={subject}: {e}");
        }
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
            let tokens =
                v1_compiler_tokenize::tokenize(source.content.clone(), source.path.clone());
            let nl_index = build_newline_index(source.path.clone(), source.content.clone());
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
    typecheck_gate: ResolveTypecheckGate,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let result = match typecheck_gate {
        ResolveTypecheckGate::Strict => {
            v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()))
        }
        ResolveTypecheckGate::DiscoveryCorpusAdvisory => {
            v1_compiler_compile::compile_to_resolved_discovery_corpus_advisory(Rc::new(
                sources.into(),
            ))
        }
    };

    let has_errors = result
        .diagnostics
        .iter()
        .any(|d| is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate));
    if has_errors {
        let si: HashMap<String, Rc<NewlineIndex>> = result
            .newline_indices
            .iter()
            .map(|idx| (idx.file.clone(), idx.clone()))
            .collect();
        let mut msgs = Vec::new();
        for d in result.diagnostics.iter() {
            if !is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate) {
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
                "{}: error: {}",
                loc,
                diagnostic_to_message(d.diagnostic.clone())
            ));
        }
        return Err(msgs.join("\n"));
    }

    let graph = result
        .graph
        .clone()
        .ok_or_else(|| "compilation produced no graph".to_string())?;
    Ok((graph, result.source_indices.clone()))
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
            let p = sf.path.replace('\\', "/");
            !exclude_substrings
                .iter()
                .any(|sub| p.contains(sub.as_str()) || module_path.contains(sub.as_str()))
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
    let (graph, source_indices) =
        resolved_graph_from_sources(picked.sources, ResolveTypecheckGate::Strict)?;
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

/// M0 ancestry-retention probe (v1-run-stability-throughline M0): per-module vs
/// distinct-spine entry counts for the typecheck-env maps — the quadratic witness the
/// deleted `cache_walk` (#5888, dissolved #5899) never measured (it counted payload-Rc
/// sharing, which is healthy; the byte carrier is the per-module materialized map SPINES).
/// Pure reader over one strict whole-tree resolve; prints `[ancestry]` lines and the peak
/// RSS; no behavior change anywhere else. `retained` sums every module's map sizes (what
/// the typed cache holds resident); `distinct` sums each unique Rc spine once (what is
/// actually allocated). `dup_factor = retained/distinct` — a factor ≫1 on the ancestry
/// maps is the located §2 duplication; flat ≈1 means spines are shared and M1 is done.
pub fn whole_tree_ancestry_retention_probe(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<(), String> {
    let picked = whole_tree_strict_sources(source_roots, exclude_substrings)?;
    let modules_resolved = picked.modules_resolved;
    let modules_excluded = picked.modules_excluded;
    let (graph, source_indices) =
        resolved_graph_from_sources(picked.sources, ResolveTypecheckGate::Strict)?;

    struct FieldTally {
        name: &'static str,
        retained_entries: usize,
        distinct_entries: usize,
        distinct_spines: std::collections::HashSet<usize>,
    }
    impl FieldTally {
        fn new(name: &'static str) -> Self {
            FieldTally {
                name,
                retained_entries: 0,
                distinct_entries: 0,
                distinct_spines: std::collections::HashSet::new(),
            }
        }
        fn add(&mut self, spine_ptr: usize, entries: usize) {
            self.retained_entries += entries;
            if self.distinct_spines.insert(spine_ptr) {
                self.distinct_entries += entries;
            }
        }
    }

    let mut tallies = [
        FieldTally::new("tec.str_bindings"),
        FieldTally::new("tec.deps_map"),
        FieldTally::new("tec.cycle_set_str"),
        FieldTally::new("tec.variant_locals"),
        FieldTally::new("te.str_bindings"),
        FieldTally::new("te.ancestry_str_bindings"),
        FieldTally::new("te.bindings"),
        FieldTally::new("te.source_visible_names"),
        FieldTally::new("te.inductive_fields.keys"),
        FieldTally::new("te.recursive_type_set"),
    ];
    // Inductive-field LIST mass (Σ list lengths) tracked separately from key count —
    // the concat-on-collision duplication class shows up in list length, not key count.
    let mut ind_lists_retained: usize = 0;
    let mut ind_lists_distinct: usize = 0;
    let mut ind_list_spines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    let mut per_module: Vec<(String, usize, usize, usize)> = Vec::new();

    for m in graph.modules.iter() {
        let te = &m.type_env;
        let tec = &m.type_env_cache;
        tallies[0].add(
            Rc::as_ptr(&tec.str_bindings) as usize,
            tec.str_bindings.len(),
        );
        tallies[1].add(Rc::as_ptr(&tec.deps_map) as usize, tec.deps_map.len());
        tallies[2].add(
            Rc::as_ptr(&tec.cycle_set_str) as usize,
            tec.cycle_set_str.len(),
        );
        tallies[3].add(
            Rc::as_ptr(&tec.variant_locals) as usize,
            tec.variant_locals.len(),
        );
        tallies[4].add(Rc::as_ptr(&te.str_bindings) as usize, te.str_bindings.len());
        tallies[5].add(
            Rc::as_ptr(&te.ancestry_str_bindings) as usize,
            te.ancestry_str_bindings.len(),
        );
        tallies[6].add(Rc::as_ptr(&te.bindings) as usize, te.bindings.len());
        tallies[7].add(
            Rc::as_ptr(&te.source_visible_names) as usize,
            te.source_visible_names.len(),
        );
        tallies[8].add(
            Rc::as_ptr(&te.inductive_fields) as usize,
            te.inductive_fields.len(),
        );
        tallies[9].add(
            Rc::as_ptr(&te.recursive_type_set) as usize,
            te.recursive_type_set.len(),
        );

        let module_ind_mass: usize = te.inductive_fields.iter().map(|(_, v)| v.len()).sum();
        ind_lists_retained += module_ind_mass;
        if ind_list_spines.insert(Rc::as_ptr(&te.inductive_fields) as usize) {
            ind_lists_distinct += module_ind_mass;
        }

        per_module.push((
            authored_name_at(source_indices.clone(), m.module.clone()),
            tec.str_bindings.len(),
            te.ancestry_str_bindings.len(),
            module_ind_mass,
        ));
    }

    eprintln!(
        "[ancestry] modules={modules_resolved} excluded={modules_excluded} (strict whole-tree resolve)"
    );
    let mut retained_total = 0usize;
    let mut distinct_total = 0usize;
    for t in &tallies {
        let dup = if t.distinct_entries > 0 {
            t.retained_entries as f64 / t.distinct_entries as f64
        } else {
            1.0
        };
        eprintln!(
            "[ancestry] field={} retained_entries={} distinct_spines={} distinct_entries={} dup_factor={:.2}",
            t.name,
            t.retained_entries,
            t.distinct_spines.len(),
            t.distinct_entries,
            dup
        );
        retained_total += t.retained_entries;
        distinct_total += t.distinct_entries;
    }
    let ind_dup = if ind_lists_distinct > 0 {
        ind_lists_retained as f64 / ind_lists_distinct as f64
    } else {
        1.0
    };
    eprintln!(
        "[ancestry] field=te.inductive_fields.list_mass retained={ind_lists_retained} distinct={ind_lists_distinct} dup_factor={ind_dup:.2}"
    );
    eprintln!(
        "[ancestry] TOTAL retained_entries={retained_total} distinct_entries={distinct_total} dup_factor={:.2}",
        if distinct_total > 0 {
            retained_total as f64 / distinct_total as f64
        } else {
            1.0
        }
    );

    per_module.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));
    for (name, tec_str, anc_str, ind_mass) in per_module.iter().take(10) {
        eprintln!(
            "[ancestry] top module={name} tec.str_bindings={tec_str} te.ancestry_str_bindings={anc_str} inductive_list_mass={ind_mass}"
        );
    }

    match peak_rss_vhwm_bytes() {
        Some(bytes) => {
            eprintln!("[ancestry] peak RSS: {bytes} bytes (VmHWM) modules={modules_resolved}")
        }
        None => eprintln!("[ancestry] peak RSS: unavailable (no /proc/self/status)"),
    }
    Ok(())
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
        || v1_compiler_tokenize::is_keyword_text(body.clone())
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
    ImportResolutionObservation {
        facts: out,
        observed_paths,
        read_refusals,
    }
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

/// Reference-derived analogue of `import_resolution_facts`: emit one edge per (file, referenced
/// module). Same row shape channel as import facts, plus a `resolution` confidence tag. Cached by
/// (pool_roots, importer_roots, excludes).
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
    // THE WHOLE-POOL PARSE PASS BELOW IS THE FLOOR'S LARGEST SHARED FILL. Timed and attributed
    // from here so the claim that happens to reach it first is not read as the claim that costs
    // it; see `shared_fill` for why the per-row number alone cannot answer that.
    shared_fill::begin_fill();
    let reference_edges_fill_start = std::time::Instant::now();
    let mut unaccounted: Vec<ReferenceAccountingRefusal> = Vec::new();

    // ── Pass 1: parse the pool once. Build the exported-name→module index (precedence: first root
    // wins, mirroring `build_module_path_index`) and the declared-module-name set. Keep each file's
    // parsed tree so edge emission does not re-parse.
    let mut decl_index: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
    let mut module_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_modules: std::collections::HashSet<String> = std::collections::HashSet::new();
    // `has_imports` decides per-file whether reference edges are emitted at all: a file that still
    // carries `import` lines is covered EXACTLY by `import_resolution_facts` (no regression, no
    // over-connection). Only an import-less (stripped) file falls back to reference edges. So on the
    // un-stripped tree this producer emits nothing and the module graph is byte-identical to before.
    let mut pool_trees: HashMap<String, (String, Rc<crate::v1_std_core::Node>, bool)> =
        HashMap::new();
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
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let module_name = match extract_module_path(&content) {
                Some(m) => m,
                None => continue,
            };
            let tree = match parse_module_node_tolerant(&rel, &content) {
                Some(t) => t,
                None => continue,
            };
            let has_imports = !extract_import_paths(&content).is_empty();
            // Precedence: a module name already claimed by an earlier root does not re-contribute
            // exported names (first-root-wins, as `build_module_path_index`).
            if seen_modules.insert(module_name.clone()) {
                module_names.insert(module_name.clone());
                for name in collect_module_decl_names(&tree) {
                    decl_index
                        .entry(name)
                        .or_default()
                        .insert(module_name.clone());
                }
            }
            pool_trees
                .entry(rel)
                .or_insert((module_name, tree, has_imports));
        }
    }

    // ── Pass 2: for each importer file, collect its reference use sites and resolve them to modules.
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
            let (self_module, tree) = match pool_trees.get(&rel) {
                // A file that still carries imports is covered exactly by `import_resolution_facts`;
                // emitting reference edges for it would only over-connect. Skip — reference edges are
                // for import-less (stripped) files.
                Some((_, _, true)) => continue,
                Some((m, t, false)) => (m.clone(), t.clone()),
                // Absent from pass 1 means pass 1 skipped it: unreadable, no module line, or a
                // parse failure. Each is the producer being UNABLE TO ASK what this file depends
                // on — ignorance, not an answer — so each is recorded as a located refusal rather
                // than silently yielding an edgeless file that downstream reads as "no
                // dependencies" (DESIGN §5: a failure arm must refuse, never widen).
                None => {
                    let content = match std::fs::read_to_string(&file) {
                        Ok(c) => c,
                        Err(_) => {
                            unaccounted.push(ReferenceAccountingRefusal {
                                path: rel.clone(),
                                cause: "unreadable",
                            });
                            continue;
                        }
                    };
                    // Import-bearing: accounted EXACTLY by `import_resolution_facts`, so this is
                    // not a refusal — the other producer owns this file's edges.
                    if !extract_import_paths(&content).is_empty() {
                        continue;
                    }
                    let module_name = match extract_module_path(&content) {
                        Some(m) => m,
                        None => {
                            unaccounted.push(ReferenceAccountingRefusal {
                                path: rel.clone(),
                                cause: "no-module-line",
                            });
                            continue;
                        }
                    };
                    match parse_module_node_tolerant(&rel, &content) {
                        Some(t) => (module_name, t),
                        None => {
                            unaccounted.push(ReferenceAccountingRefusal {
                                path: rel.clone(),
                                cause: "parse-failed",
                            });
                            continue;
                        }
                    }
                }
            };
            let mut bare: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut chains: Vec<Vec<String>> = Vec::new();
            // THE INDEX IS BUILT FROM THE BYTES THIS PRODUCER ALREADY READ, not fetched from a
            // prepared subject that does not exist at this point: `collect_node_refs` recovers a
            // lambda parameter's name from its authored span, and a missing index would leave
            // those names unbound and let them resolve as references.
            let mut file_indices: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
            if let Ok(content) = std::fs::read_to_string(&file) {
                file_indices.insert(
                    rel.clone(),
                    crate::v1_std_core::build_newline_index(rel.clone(), content),
                );
            }
            let file_indices = Rc::new(file_indices);
            // THIS PRODUCER ANSWERS A CORPUS-WIDE QUESTION BEFORE ANY SUBJECT IS PREPARED, so it
            // has no declaration index to classify against and cannot decide
            // DeclaredElsewhere / NamesNothingKnown. It supplies an EMPTY index, under which
            // every unbound name classifies as a reference — the behaviour this producer already
            // had, preserved deliberately rather than by omission. The totality guarantee is
            // therefore scoped to the floor's index build, which is the consumer that has the
            // declarations; this one keeps its own looser contract and its own counters.
            let mut scratch_tally: BTreeMap<ExprVarClass, usize> = BTreeMap::new();
            let mut scratch_unclassified: Vec<String> = Vec::new();
            let mut classify = ExprVarClassification {
                decl_index: None,
                tally: &mut scratch_tally,
                unclassified: &mut scratch_unclassified,
                module: self_module.clone(),
                occurrences: 0,
                free_reference_edges: 0,
                bound_occurrences_suppressed: 0,
                chain_head_occurrences: 0,
                refusals: Vec::new(),
            };
            for item in tree.children.iter() {
                collect_node_refs(item, &mut bare, &mut chains, &file_indices, &mut classify);
            }
            // THE SECOND CONSUMER ENFORCES THE SAME REFUSAL THE FIRST ONE DOES. `classify` reports
            // two failure states and neither is survivable for a graph that is about to be
            // published: an unsupported binder form means the binder set is incomplete, so a name
            // that IS bound can be recorded as a free reference (or the reverse); a reconciliation
            // miss means the occurrences do not add up, which is the same statement arrived at by
            // counting. Reading them and proceeding anyway would publish an under-bound, possibly
            // widened graph while the refusal sat unread in a field — the fail-open this
            // classification exists to remove, one consumer away from the floor path that checks
            // it correctly (review 55667).
            //
            // TWO CAUSES, NOT ONE, because they have different remedies: a binder refusal names a
            // syntax the collector must learn, and a reconciliation miss names an accounting
            // defect in the collector itself. Collapsing them would send both to whichever
            // remedy the shared symbol happened to suggest.
            //
            // The file is SKIPPED rather than published, matching this producer's three existing
            // refusal arms above (`unreadable`, `no-module-line`, `parse-failed`), which also skip
            // and record. The narrowing is therefore typed, located and countable through
            // `reference_accounting_refusals`, not silent — a skipped file is visible in that
            // channel, which is what separates it from the empty-observation narrow.
            if !classify.refusals.is_empty() {
                unaccounted.push(ReferenceAccountingRefusal {
                    path: rel.clone(),
                    cause: "binder-refusal",
                });
                continue;
            }
            if !classify.reconciles() {
                unaccounted.push(ReferenceAccountingRefusal {
                    path: rel.clone(),
                    cause: "occurrence-accounting-mismatch",
                });
                continue;
            }
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
                if let Some(m) = longest_declared_module_prefix(chain, &module_names) {
                    if m != self_module {
                        upgrade(m, RefEdgeResolution::Qualified);
                    }
                }
            }
            for name in &bare {
                if let Some(mods) = decl_index.get(name) {
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
                                let is_witness =
                                    rel.contains("/test/") || rel.ends_with("_test.dag");
                                let cands: Vec<String> =
                                    winners.iter().map(|s| (*s).clone()).collect();
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
            for (m, res) in file_edges {
                edges.push(ReferenceEdgeRaw {
                    path: rel.clone(),
                    target_module: m,
                    resolution: res,
                });
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
