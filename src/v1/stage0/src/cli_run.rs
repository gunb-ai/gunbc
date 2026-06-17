// cli_run.rs — Hand-maintained Run subcommand handler.
// Not generated — survives stage0 regeneration.
// The generated main.rs calls handle_run_with_options() for the Run subcommand.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::kernel_type_set;
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::lookup_type_by_name;
use crate::v1_compiler_infer_items::{ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_rt;
use crate::v1_std_core::{
    authored_name_at, build_newline_index, byte_to_line_col, diagnostic_to_message,
    diagnostic_to_span, empty_intern_table, expr_var_name_at, field_init_node_name_at,
    field_init_node_value, has_child_named, intern, is_error_diagnostic,
    is_interpreter_blocking_diagnostic, ErrorNode, ExprData, InferredNode, InternTable,
    NewlineIndex, Node,
};
use serde::Serialize;

use crate::resolved_graph_cache::{
    lookup as cross_process_lookup, resolved_graph_cache_root_from_env, subject_digest_for_closure,
    write as cross_process_write, CacheLookupResult,
};

/// Module that owns `UnifiedTestClaim` and its registration arms.
pub const UNIFIED_CLAIM_VERIFICATION_MODULE: &str = "v2.std.verification";
pub const BOOL_WITNESS_CLAIM_TYPE: &str = "BoolWitnessClaim";
pub const NODE_CORPUS_TYPE: &str = "NodeCorpus";

/// Recursively find all .dag files under a directory.
fn collect_dag_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read dir {:?}: {}", dir, e))
        .map(|e| e.unwrap_or_else(|e| panic!("failed to read dir entry: {}", e)))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, files);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            files.push(path);
        }
    }
}

/// Extract the `module x.y.z` declaration from a .dag file.
fn extract_module_path(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return Some(trimmed["module ".len()..].trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

/// Extract import module paths from a .dag file.
fn extract_import_paths(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            let rest = trimmed["import ".len()..].trim();
            let module_path = rest.split('{').next().unwrap_or(rest).trim();
            if !module_path.is_empty() {
                imports.push(module_path.to_string());
            }
        }
    }
    imports
}

/// module_path → pre-read source (built once; shared across per-entry resolves).
type ModuleSourceIndex = HashMap<String, Rc<v1_compiler_compile::SourceFile>>;

/// Build module index: read each `.dag` under `source_roots` once into `ModuleSourceIndex`.
fn build_module_index(source_roots: &[String]) -> ModuleSourceIndex {
    let mut index = ModuleSourceIndex::new();
    for root in source_roots {
        let root_path = std::path::Path::new(root);
        if !root_path.exists() {
            panic!("source root does not exist: {}", root);
        }
        let mut dag_files = Vec::new();
        collect_dag_files(root_path, &mut dag_files);
        for path in dag_files {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
            if let Some(module_path) = extract_module_path(&content) {
                let rel_path = path.to_string_lossy().to_string();
                index.insert(
                    module_path,
                    Rc::new(v1_compiler_compile::SourceFile {
                        path: rel_path,
                        content,
                    }),
                );
            }
        }
    }
    index
}

/// Resolve imports transitively. Returns sorted sources.
fn resolve_transitively(
    entry_sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index: &ModuleSourceIndex,
    mut seen: HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
) -> Vec<Rc<v1_compiler_compile::SourceFile>> {
    let mut queue = entry_sources;
    while let Some(source) = queue.pop() {
        for module_path in extract_import_paths(&source.content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(imported) = index.get(&module_path) {
                seen.insert(module_path, imported.clone());
                queue.push(imported.clone());
            }
        }
    }
    let mut result: Vec<_> = seen.into_values().collect();
    result.sort_by(|a, b| a.path.cmp(&b.path));
    result
}

/// Load one entry `.dag` file plus its transitive import closure (not the whole tree).
pub fn load_sources_for_entry(
    source_roots: &[String],
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let index = build_module_index(source_roots);
    load_sources_for_entry_with_index(&index, entry_path)
}

fn entry_source_from_index_or_disk(
    index: &ModuleSourceIndex,
    entry_path: &str,
) -> Result<Rc<v1_compiler_compile::SourceFile>, String> {
    let path = std::path::Path::new(entry_path);
    if !path.is_file() {
        return Err(format!(
            "entry file does not exist or is not a file: {}",
            entry_path
        ));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read entry {:?}: {}", path, e))?;
    let rel_path = path.to_string_lossy().to_string();
    if let Some(mod_path) = extract_module_path(&content) {
        if let Some(cached) = index.get(&mod_path) {
            if cached.path == rel_path {
                return Ok(cached.clone());
            }
        }
    }
    Ok(Rc::new(v1_compiler_compile::SourceFile {
        path: rel_path,
        content,
    }))
}

fn load_sources_for_entry_with_index(
    index: &ModuleSourceIndex,
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let entry_source = entry_source_from_index_or_disk(index, entry_path)?;
    let rel_path = entry_source.path.clone();

    let mut seen: HashMap<String, Rc<v1_compiler_compile::SourceFile>> = HashMap::new();
    if let Some(mod_path) = extract_module_path(&entry_source.content) {
        seen.insert(mod_path, entry_source.clone());
    }
    let mut sources = resolve_transitively(vec![entry_source.clone()], index, seen);
    // Module-less entry files have no `module` line, so they never enter `seen`; ensure the
    // entry path is still in the closure result.
    if !sources.iter().any(|s| s.path == rel_path) {
        sources.push(entry_source);
    }
    Ok(sources)
}

/// Load and resolve sources from source roots (every `.dag` under the first root).
fn load_sources(source_roots: &[String]) -> Vec<Rc<v1_compiler_compile::SourceFile>> {
    let index = build_module_index(source_roots);
    let first_root = std::path::Path::new(&source_roots[0]);
    let mut entry_files = Vec::new();
    if first_root.is_dir() {
        let mut dag_paths = Vec::new();
        collect_dag_files(first_root, &mut dag_paths);
        for path in dag_paths {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
            entry_files.push((path.to_string_lossy().to_string(), content));
        }
    }

    let mut seen: HashMap<String, Rc<v1_compiler_compile::SourceFile>> = HashMap::new();
    let mut entry_for_queue = Vec::new();
    for (path, content) in &entry_files {
        let source = Rc::new(v1_compiler_compile::SourceFile {
            path: path.clone(),
            content: content.clone(),
        });
        if let Some(mod_path) = extract_module_path(content) {
            seen.insert(mod_path, source.clone());
        }
        entry_for_queue.push(source);
    }

    let mut sources = resolve_transitively(entry_for_queue, &index, seen);
    for (path, content) in entry_files {
        if !sources.iter().any(|s| s.path == path) {
            sources.push(Rc::new(v1_compiler_compile::SourceFile { path, content }));
        }
    }
    sources
}

/// Outcome of running a single Bool witness (`--claim-run` semantics), without
/// touching the process exit code. The exit-code contract lives in the caller.
pub enum ClaimOutcome {
    /// Function returned `Bool(true)` — witness holds.
    Pass,
    /// Function returned `Bool(false)` — witness fails (the perturb-red signal).
    Fail,
    /// Function returned a non-Bool value; under `--claim-run` the entry must
    /// return Bool. Carries the rendered value for diagnostics.
    NotBool { got: String },
    /// The interpreter raised a runtime error.
    RuntimeError { message: String },
}

/// Resolve one entry `.dag` file's transitive import closure into a typed graph,
/// or return formatted blocking diagnostics. This is the expensive step
/// (`build_module_index` + closure resolve + full compile); callers that run
/// many witnesses against the SAME entry should call this once and reuse the
/// returned graph (see the `claim_batch` bin).
///
/// Single-authority note: this reuses the exact primitives the per-run path in
/// `handle_run_with_options` uses (`load_sources_for_entry`,
/// `compile_to_resolved`, `is_interpreter_blocking_diagnostic`). It is an
/// alternate ORCHESTRATION over those primitives (resolve-once / run-many), not
/// a second copy of the resolve logic.
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
    let index = build_module_index(source_roots);
    resolve_entry_graph_with_index(&index, entry_file)
}

/// Opaque module source index built from a set of source roots. Pass to
/// `resolve_entry_with_index` to resolve multiple entries without re-scanning
/// the filesystem per entry. Used by the `claim_batch` multi-entry green pass.
pub struct MultiEntryIndex {
    source_files: ModuleSourceIndex,
    /// Lazily-accumulated intern table. Starts empty; grows as new files are
    /// parsed for the first time. `parse_with_table` advances it via its
    /// returned `intern_table` field, so every token string gets a stable ID
    /// regardless of which entry first triggers its parse.
    intern_table: RefCell<Rc<InternTable>>,
    /// Lazily-populated parse cache: file path → (ParseResult, NewlineIndex).
    /// Populated on first access for each file; shared across all entry resolves.
    parse_cache: RefCell<HashMap<String, (Rc<v1_compiler_parse::ParseResult>, Rc<NewlineIndex>)>>,
    /// Lazily-populated typed-module cache: module name → its
    /// `TypecheckModuleResult` (typed module + its diagnostics). Populated on
    /// first type-reconciliation of each module; reused across all entry resolves
    /// that share this index.
    ///
    /// Why module name is a sound (alias-free) key WITHIN one index: each entry's
    /// closure is read once into `source_files`, so a module name maps to exactly
    /// one immutable source file. A module's typed result is a pure function of
    /// (its own resolved AST) + (the typed results of the modules it imports,
    /// looked up by name) + (the foundational `std.types` env) — `typecheck_module`
    /// consults `parent_index` only for those (v1_compiler_infer.rs build_type_env
    /// / collect_parent_envs), never the rest of the closure, and modules are
    /// processed in topological order so a module's imports are always already
    /// typed. The shared interner table grows monotonically, so a token's id is
    /// stable across entries. Therefore the same name always yields the same typed
    /// result, and reuse is byte-identical to recomputation. This is the
    /// resolve-cost lever PR1: it collapses N near-identical full
    /// type-reconciliations (one per entry, ~30-50 modules each, heavily
    /// overlapping) into one shared core typed once plus per-entry leaves. Sibling
    /// to `parse_cache`.
    typed_module_cache: RefCell<HashMap<String, Rc<v1_compiler_infer::TypecheckModuleResult>>>,
}

/// Pre-seed `table` with the fixed type-name set that `build_type_env`
/// (v1_compiler_infer) interns AT TYPE TIME: `kernel_type_set()` (String, Int,
/// Bool, Json, Unit, …), the Optional family (`Optional`/`Present`/`Absent`/
/// `value`/`none`), and `compiler_recursive_types()`.
///
/// Why this is REQUIRED for the typed-module cache to be sound: `build_type_env`
/// interns these names into a *local clone* of whatever intern table it is given,
/// so their ids land after the parse tokens already in that table — i.e. they are
/// table-SIZE dependent. The shared index table grows as each entry parses, so
/// without this seed a module typed for an early entry bakes its kernel-type ids
/// at that entry's table size, while a module typed fresh for a later entry uses
/// different kernel-type ids; a reused binding (keyed by the early ids) then
/// misses on lookup and the type collapses to the `Json` fallback — an
/// order-dependent, verdict-affecting miscompile. Seeding these names into the
/// SHARED table BEFORE any parse makes `intern()` return the same ids for every
/// entry's `build_type_env`, so cached and freshly-typed modules cross-reference
/// correctly.
///
/// This only ADDS names `build_type_env` would intern anyway (idempotent intern);
/// it changes no typecheck semantics. The full claim-witness corpus across
/// permuted entry orders (vs the no-cache cold resolve as oracle) is the proof
/// that the seed is complete — see `resolve_typed_cache_equivalence_test`.
///
/// SOUNDNESS PROPERTY (born-mark) — INTERN-ID CONTENT-STABILITY.
/// A module's typed result is content-addressable (a pure function of its content
/// + its imports' identities) ONLY IF the type-time-interned kernel type-name ids
/// are content-stable, i.e. independent of how many tokens happen to precede them
/// in the ambient intern table. Content-addressed memoization (this typed-module
/// cache; the planned cross-process resolved-graph cache) is sound ONLY over a
/// pure unit, so this property is a PRECONDITION for caching resolve at all — not
/// a cache band-aid. It was discovered because the cache surfaced the latent
/// violation: `build_type_env` assigns kernel-type ids by ambient table size, so
/// the per-module typed result secretly depended on resolution-context state.
///   Enforced by: this function (seed the kernel names so their ids are fixed
///     across every entry in an index).
///   Witness: `resolve_typed_cache_equivalence_test` (order-permuted, cold-oracle).
///   TRIPWIRE: anyone who changes the kernel type-name set, the `build_type_env`
///     type-time interning, or this seed MUST keep the witness green; a red
///     witness means content-stability regressed and the cache is unsound. If the
///     instability proves broader than a fixed name-set (genuinely table-SIZE
///     dependent inside generated infer), that is a typechecker purity defect to
///     fix in infer, not to paper over by extending this seed.
fn seed_kernel_intern_names(table: Rc<InternTable>) -> Rc<InternTable> {
    let mut t = table;
    for name in v1_rt::map_keys(&kernel_type_set()).iter().cloned() {
        t = intern(t, name).table.clone();
    }
    for name in ["Optional", "Present", "Absent", "value", "none"] {
        t = intern(t, name.to_string()).table.clone();
    }
    for name in v1_rt::map_keys(&compiler_recursive_types()).iter().cloned() {
        t = intern(t, name).table.clone();
    }
    t
}

/// Scan the given source roots once and return a `MultiEntryIndex`. Only the
/// filesystem scan happens here; tokenise+parse work is deferred to the first
/// `resolve_entry_with_index` call that needs each file, so the index build cost
/// is proportional to the number of .dag files on disk — not to their parse time.
pub fn build_multi_entry_index(source_roots: &[String]) -> MultiEntryIndex {
    MultiEntryIndex {
        source_files: build_module_index(source_roots),
        intern_table: RefCell::new(seed_kernel_intern_names(empty_intern_table())),
        parse_cache: RefCell::new(HashMap::new()),
        typed_module_cache: RefCell::new(HashMap::new()),
    }
}

/// Resolve one entry's import closure using a pre-built `MultiEntryIndex`.
/// Uses cached parse trees for all files in the source root, skipping
/// tokenize+parse per entry.
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

fn resolve_entry_graph_with_index(
    index: &ModuleSourceIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let sources = load_sources_for_entry_with_index(index, entry_file)?;
    resolved_graph_from_sources(sources)
}

/// Resolve one entry's closure using a lazily-populated parse cache from
/// `MultiEntryIndex`. Each file is tokenised and parsed at most once per
/// session; shared std/ files are reused across all entry resolves.
///
/// Cache miss path: tokenise the file, advance the global intern table via
/// `parse_with_table` (which pre-interns internally), store the result, and
/// continue. The intern table only grows, so cached parse results stay valid
/// for all future entries.
// TODO(dissolution): this function duplicates the tokenize→parse→resolve→normalize→reconcile→
// ownership pipeline that `resolved_graph_from_sources` drives through `compile_to_resolved`.
// The duplication exists solely to thread the lazy intern table across cached parses.  When
// `compile_to_resolved` (or a wrapper) learns to accept a pre-populated parse cache and intern
// table, fold this back and delete the inline pipeline.
fn resolve_entry_with_parse_cache(
    index: &MultiEntryIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let sources = load_sources_for_entry_with_index(&index.source_files, entry_file)?;

    if let Some(cache_root) = resolved_graph_cache_root_from_env() {
        let subject = subject_digest_for_closure(&sources);
        match cross_process_lookup(&cache_root, &subject) {
            CacheLookupResult::Hit(hit) => {
                return Ok((hit.graph, hit.source_indices));
            }
            CacheLookupResult::RejectedHit(_) | CacheLookupResult::Miss => {}
        }
    }

    let mut modules: Vec<Rc<Node>> = Vec::new();
    let mut si_map: HashMap<String, Rc<NewlineIndex>> = HashMap::new();

    for source in &sources {
        // Release immutable borrow before any potential mutation below.
        let cached = index.parse_cache.borrow().get(&source.path).cloned();

        let (parse_result, nl_index) = match cached {
            Some(entry) => entry,
            None => {
                // First encounter for this file: tokenise, parse with the
                // current accumulated intern table, advance the table, cache.
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
                // Advance the global intern table with tokens from this file.
                *index.intern_table.borrow_mut() = parsed.intern_table.clone();
                let entry = (parsed.result.clone(), nl_index);
                index
                    .parse_cache
                    .borrow_mut()
                    .insert(source.path.clone(), entry.clone());
                entry
            }
        };

        si_map.insert(nl_index.file.clone(), nl_index.clone());
        if let Some(err) = &parse_result.error {
            let span = diagnostic_to_span(err.diagnostic.clone());
            let loc = format_error_loc(&span.file, span.start, &si_map);
            return Err(format!(
                "{}: error: {}",
                loc,
                diagnostic_to_message(err.diagnostic.clone())
            ));
        }
        if let Some(module) = &parse_result.module {
            modules.push(module.clone());
        }
    }

    let source_indices = Rc::new(si_map);
    // Snapshot the accumulated intern table after all files in this closure
    // have been parsed; pass it to reconcile for type-name lookup.
    let global_table = index.intern_table.borrow().clone();

    let graph = v1_compiler_resolve::resolve_modules(Rc::new(modules), source_indices.clone());

    if graph
        .diagnostics
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(format_error_nodes(&graph.diagnostics, &source_indices));
    }

    let norm = v1_compiler_normalize::normalize_graph(graph.clone(), source_indices.clone());

    if norm
        .diagnostics
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(format_error_nodes(&norm.diagnostics, &source_indices));
    }

    let typed = reconcile_with_typed_cache(
        norm.graph.clone(),
        source_indices.clone(),
        global_table,
        &index.typed_module_cache,
    );

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
        return Err(msgs.join("\n"));
    }

    // Ownership validation — same check compile_to_resolved applies after
    // reconcile (P3 fail-closed parity: claim_batch must not green-light a
    // graph that gunbc run would block on ownership errors).
    let ownership = v1_compiler_compile::extract_ownership_proofs(typed.clone());
    let ownership_diags = v1_compiler_compile::ownership_diagnostics(ownership);
    if ownership_diags
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(format_error_nodes(&ownership_diags, &source_indices));
    }

    if let Some(cache_root) = resolved_graph_cache_root_from_env() {
        let subject = subject_digest_for_closure(&sources);
        let _ = cross_process_write(&cache_root, &subject, &typed, source_indices.as_ref());
    }

    Ok((typed, source_indices))
}

/// Host-side memoized form of `v1_compiler_infer::reconcile`. Produces the same
/// `ResolvedGraph` it would, but per-module typed results come from (and populate)
/// `typed_cache`, so modules shared across many entry resolves are type-reconciled
/// once instead of once per entry (the resolve-cost lever, PR1).
///
/// Single-authority note: this is an alternate ORCHESTRATION over the exact pure
/// primitives the generated `typecheck_modules` loop drives — `collect_parent_envs`,
/// `typecheck_module`, `expand_transitive_services`, `build_emit_graph_info` — not a
/// reimplementation of type checking. It mirrors that loop step-for-step (same per-
/// module ops, same topological module order, same diagnostic-chunk ordering, same
/// 5-pass service expansion) and threads a typed-module cache the way
/// `resolve_entry_with_parse_cache` threads a parse cache. With an empty cache the
/// output is byte-identical to `reconcile`; a cache hit reuses a result that is, by
/// the argument on `MultiEntryIndex::typed_module_cache`, equal to recomputation.
/// (Falsifier: witness verdicts must be byte-identical with the cache warm vs cold —
/// gated by the claim-witness corpus + perturb pass.)
fn reconcile_with_typed_cache(
    graph: Rc<v1_compiler_resolve::ModuleGraph>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    intern_table: Rc<InternTable>,
    typed_cache: &RefCell<HashMap<String, Rc<v1_compiler_infer::TypecheckModuleResult>>>,
) -> Rc<ResolvedGraph> {
    let mut modules: Rc<Vec<Rc<TypedModule>>> = Rc::new(Vec::new());
    let mut module_index: Rc<HashMap<String, Rc<TypedModule>>> = v1_rt::rc_empty_map();
    let mut item_registry: Rc<HashMap<String, Rc<ItemInfo>>> = v1_rt::rc_empty_map();
    let mut diag_chunks: Vec<Rc<Vec<Rc<ErrorNode>>>> = Vec::new();

    for resolved in graph.modules.iter().cloned() {
        // collect_parent_envs is cheap (import-scoped lookups) and its diagnostics
        // depend on the live module_index; recompute it every iteration exactly as
        // the generated loop does — only the expensive typecheck_module is cached.
        let parent_result = v1_compiler_infer::collect_parent_envs(
            resolved.clone(),
            module_index.clone(),
            source_indices.clone(),
        );
        let mod_name = authored_name_at(source_indices.clone(), resolved.module.clone());
        let cached = typed_cache.borrow().get(&mod_name).cloned();
        let tc_result = match cached {
            Some(hit) => hit,
            None => {
                let computed = v1_compiler_infer::typecheck_module(
                    resolved.clone(),
                    module_index.clone(),
                    source_indices.clone(),
                    intern_table.clone(),
                );
                typed_cache
                    .borrow_mut()
                    .insert(mod_name.clone(), computed.clone());
                computed
            }
        };
        let typed = tc_result.typed.clone();
        modules = v1_rt::rc_list_push(modules, typed.clone());
        module_index = v1_rt::rc_map_insert(
            module_index,
            authored_name_at(source_indices.clone(), typed.module.clone()),
            typed.clone(),
        );
        item_registry = v1_rt::rc_map_merge(item_registry, typed.item_registry.clone());
        diag_chunks.push(parent_result.diagnostics.clone());
        diag_chunks.push(tc_result.diagnostics.clone());
    }

    let expanded_registry =
        v1_compiler_infer::expand_transitive_services(modules.clone(), item_registry, 5);
    let diagnostics: Rc<Vec<Rc<ErrorNode>>> = Rc::new({
        let mut acc = Vec::new();
        for chunk in &diag_chunks {
            acc.extend(chunk.iter().cloned());
        }
        acc
    });
    let emit_graph_info = v1_compiler_infer::build_emit_graph_info(modules.clone());
    Rc::new(ResolvedGraph {
        modules,
        item_registry: expanded_registry,
        diagnostics,
        emit_graph_info,
    })
}

fn format_error_loc(file: &str, start: i64, si: &HashMap<String, Rc<NewlineIndex>>) -> String {
    match si.get(file) {
        Some(idx) => {
            let lc = byte_to_line_col(idx.clone(), start);
            format!("{}:{}:{}", file, lc.line, lc.col)
        }
        None => file.to_string(),
    }
}

fn format_error_node(
    d: &Rc<ErrorNode>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    let span = diagnostic_to_span(d.diagnostic.clone());
    let loc = format_error_loc(&span.file, span.start, source_indices);
    format!(
        "{}: error: {}",
        loc,
        diagnostic_to_message(d.diagnostic.clone())
    )
}

fn format_error_nodes(
    diags: &Rc<Vec<Rc<ErrorNode>>>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    diags
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| format_error_node(d, source_indices))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Compile an already-assembled source closure to a resolved graph, or return
/// formatted blocking diagnostics. Shared by the per-entry path
/// (`resolve_entry_graph_with_index`) and the batched discovery path
/// (`discover_owned_data_decls`), which merges many entry closures into one
/// compile.
fn resolved_graph_from_sources(
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources));

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

/// Build the evaluation context for a resolved graph. The context owns the
/// per-graph interpreter state (fn index, service registry, `data` cache), so
/// its lifetime IS the evaluation scope: callers running many functions over
/// one graph (see `claim_batch`) build this once and pass it to each
/// `run_claim`/`run_value` call; dropping it releases everything.
pub fn make_eval_context(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_fixture_store(graph, source_indices, execution_mode, None)
}

pub fn make_eval_context_with_fixture_store(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
) -> v1_interpreter::InterpContext {
    v1_interpreter::InterpContext::with_fixture_store(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
    )
}

/// Run one Bool witness function against an already-resolved graph, classifying
/// the result the same way `handle_run_with_options`'s `--claim-run` branch
/// does (Bool true → Pass, false → Fail, anything else → diagnostic), but
/// without calling `std::process::exit`. Eager data-env is disabled to match
/// claim-run behavior (witnesses pull data lazily).
pub fn run_claim(ctx: &v1_interpreter::InterpContext, function: &str) -> ClaimOutcome {
    match v1_interpreter::run_in_context(ctx, function, false) {
        Ok(v1_interpreter::Value::Bool(true)) => ClaimOutcome::Pass,
        Ok(v1_interpreter::Value::Bool(false)) => ClaimOutcome::Fail,
        Ok(other) => ClaimOutcome::NotBool {
            got: ctx.format_value(&other),
        },
        Err(e) => ClaimOutcome::RuntimeError {
            message: format!("{}", e),
        },
    }
}

/// Run a function against an already-resolved graph and return its raw
/// interpreter `Value`, without imposing the `--claim-run` Bool contract. This
/// is the host-transport read path: the batch executor evaluates a plan function
/// that returns a structured value (the executor-decided batches) and walks the
/// result, rather than collapsing it to a single Bool. Eager data-env is
/// disabled to match the witness/plan-run convention (values pull lazily).
pub fn run_value(
    ctx: &v1_interpreter::InterpContext,
    function: &str,
) -> Result<v1_interpreter::Value, String> {
    v1_interpreter::run_in_context(ctx, function, false).map_err(|e| format!("{}", e))
}

/// Entry point for `gunbc ci`. Delegates to dsl/tools/gunbc_ci.dag (CiSpec consumer).
pub fn handle_ci() {
    handle_run_with_options(
        vec!["dsl".to_string(), "src/v2".to_string()],
        "main".to_string(),
        Some("dsl/tools/gunbc_ci.dag".to_string()),
        false,
        false,
    );
}

/// Entry point for `dag run`. Called from the generated main.rs.
pub fn handle_run(
    source_roots: Vec<String>,
    function: String,
    entry_file: Option<String>,
    claim_run: bool,
) {
    handle_run_with_options(source_roots, function, entry_file, false, claim_run);
}

/// Entry point with options for dry-run mode.
pub fn handle_run_with_options(
    source_roots: Vec<String>,
    function: String,
    entry_file: Option<String>,
    dry_run: bool,
    claim_run: bool,
) {
    if source_roots.is_empty() {
        eprintln!("error: provide at least one --source-root");
        std::process::exit(1);
    }

    if claim_run && entry_file.is_none() {
        eprintln!(
            "error: --claim-run requires --entry <file.dag> (scoped import closure; \
             loading the whole --source-root tree is too large for witness runs)"
        );
        std::process::exit(1);
    }

    let sources = match entry_file.as_deref() {
        Some(path) => match load_sources_for_entry(&source_roots, path) {
            Ok(sources) => sources,
            Err(msg) => {
                eprintln!("error: {}", msg);
                std::process::exit(1);
            }
        },
        None => load_sources(&source_roots),
    };
    eprintln!("resolved {} sources", sources.len());

    // Compile through validation (no emission)
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources));

    // Check for errors
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
            eprintln!(
                "{}: error: {}",
                loc,
                diagnostic_to_message(d.diagnostic.clone())
            );
        }
        std::process::exit(1);
    }

    // Extract graph (guaranteed present when no errors)
    let graph = match result.graph.as_ref() {
        Some(g) => g,
        None => {
            eprintln!("error: compilation produced no graph");
            std::process::exit(1);
        }
    };

    // Run the interpreter — keep one context alive for symbol resolution while
    // printing and classifying the return value (ctrl#1533 phase 3).
    eprintln!("running {}()...", function);
    let execution_mode = if dry_run {
        v1_interpreter::ExecutionMode::Hermetic
    } else {
        v1_interpreter::ExecutionMode::Wet
    };
    let ctx =
        v1_interpreter::InterpContext::new(graph, result.source_indices.clone(), execution_mode);
    v1_interpreter::with_active_context(&ctx, || {
        match v1_interpreter::run_in_context(&ctx, &function, !claim_run) {
            Ok(val) => {
                println!("{}", val);
                if claim_run {
                    // Witness entry points return Bool; fail-closed like ProcessExit below.
                    match &val {
                        v1_interpreter::Value::Bool(false) => std::process::exit(1),
                        v1_interpreter::Value::Bool(true) => return,
                        other => {
                            eprintln!(
                                "error: function `{}` returned `{}`, not `Bool`. \
                                 With --claim-run the entry must return Bool (false → exit 1).",
                                function, other
                            );
                            std::process::exit(2);
                        }
                    }
                }
                // FAIL-CLOSED EXIT CODE CONTRACT
                //
                // Functions invoked via `dag run` MUST return std/process.dag's
                // ProcessExit variant. The host translates ExitSuccess → 0 and
                // ExitFailure { code } → code. Any other return value is a
                // programmer error: the host cannot tell whether the function
                // succeeded or failed, so it exits 2 with a clear diagnostic.
                //
                // This makes silent failure IMPOSSIBLE: a function whose result
                // type isn't structurally ProcessExit cannot accidentally exit 0
                // when its rich result represents failure. Compose internal
                // helpers (check_l1_ratchet → L1RatchetResult) freely; entry
                // points must wrap their result in ProcessExit explicitly.
                match classify_exit(&val, &ctx) {
                    ExitClass::Success => {} // exit 0 (default)
                    ExitClass::Failure(code) => std::process::exit(code),
                    ExitClass::NotProcessExit { type_name } => {
                        eprintln!(
                            "error: function `{}` returned `{}`, not `ProcessExit`. \
                             Functions invoked via `dag run` must return std/process.dag's \
                             ProcessExit so the host can map success/failure to an exit code. \
                             Wrap your rich result type in ExitSuccess / ExitFailure, or pass \
                             --claim-run for Bool witness entry points under src/v2.",
                            function, type_name
                        );
                        std::process::exit(2);
                    }
                }
            }
            Err(e) => {
                eprintln!("runtime error: {}", e);
                std::process::exit(1);
            }
        }
    });
}

/// Classification of a `dag run` return value for exit-code mapping.
enum ExitClass {
    Success,
    Failure(i32),
    /// The value is not a ProcessExit variant. Carries the actual type
    /// for the diagnostic.
    NotProcessExit {
        type_name: String,
    },
}

/// Map a Value to its exit-code class. Structural — checks the specific
/// type and variant names from std/process.dag, never substrings or
/// naming conventions.
///
///   ProcessExit::ExitSuccess              → Success
///   ProcessExit::ExitFailure { code, .. } → Failure(code)
///   anything else                         → NotProcessExit (fail-closed at host)
fn classify_exit(val: &v1_interpreter::Value, ctx: &v1_interpreter::InterpContext) -> ExitClass {
    match val {
        v1_interpreter::Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            if !ctx.sym_eq(*type_name, "ProcessExit") {
                return ExitClass::NotProcessExit {
                    type_name: ctx.resolve(*type_name),
                };
            }
            if ctx.sym_eq(*variant_name, "ExitSuccess") {
                ExitClass::Success
            } else if ctx.sym_eq(*variant_name, "ExitFailure") {
                match ctx.field(fields, "code") {
                    Some(v1_interpreter::Value::Int(n)) => ExitClass::Failure(*n as i32),
                    _ => ExitClass::Failure(1),
                }
            } else {
                ExitClass::NotProcessExit {
                    type_name: format!("ProcessExit::{}", ctx.resolve(*variant_name)),
                }
            }
        }
        _ => ExitClass::NotProcessExit {
            type_name: "<non-variant>".to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// discover_owned_data — host transport for Consolidation #4553 resolved-type glob
// ---------------------------------------------------------------------------
// These structs serde-mirror the modeled types in v2.compiler.discovery_enumeration
// (promoted there from v2.test.claim.workflow.discovery_types; the emitted manifest below
// imports that module). Keep the field shapes in lockstep with that .dag authority.

/// Resolved declaration identity from the typed graph (not authored surface names).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedDeclRef {
    pub module: String,
    pub name: String,
}

/// Initializer coproduct — arm-specific transport fields only on the matching arm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "arm", rename_all = "snake_case")]
pub enum OwnedDataDeclInitializer {
    BoolWitnessClaim {
        witness_entry: String,
        witness_function: String,
    },
    NodeCorpus,
    Other {
        resolved: ResolvedDeclRef,
    },
}

/// Neutral owned top-level `data` declaration fact (no membership filtering).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedDataDeclRecord {
    pub entry: String,
    pub module: String,
    pub decl_name: String,
    pub initializer: OwnedDataDeclInitializer,
}

fn literal_string_from_expr(node: &Rc<Node>) -> Option<String> {
    if let ExprData::ExprLiteral { value } = &*node.expr_data {
        if let LiteralValue::LitStr { value: s } = value.as_ref() {
            return Some(s.clone());
        }
    }
    None
}

fn symbol_name_from_expr(
    node: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<String> {
    binding_name_from_expr(node, source_indices)
}

fn field_init_label(
    field_init: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> String {
    let si = Rc::new(source_indices.clone());
    let authored = field_init_node_name_at(field_init.clone(), si);
    if !authored.is_empty() {
        return authored;
    }
    field_init.name.clone()
}

fn field_init_named<'a>(
    record: &'a Rc<Node>,
    field: &str,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<Rc<Node>> {
    for child in record.children.iter() {
        if field_init_label(child, source_indices) == field {
            return Some(field_init_node_value(child.clone()));
        }
    }
    None
}

fn binding_name_from_expr(
    node: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<String> {
    if let ExprData::ExprVar { .. } = &*node.expr_data {
        let name = expr_var_name_at(node.clone(), Rc::new(source_indices.clone()));
        if !name.is_empty() {
            return Some(name);
        }
    }
    let name = authored_name_at(Rc::new(source_indices.clone()), node.clone());
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn extract_bool_witness_transport(
    claim_body: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> (String, String) {
    let Some(witness_node) = field_init_named(claim_body, "witness", source_indices) else {
        return (String::new(), String::new());
    };
    let entry = field_init_named(&witness_node, "entry", source_indices)
        .and_then(|n| literal_string_from_expr(&n))
        .unwrap_or_default();
    let function = field_init_named(&witness_node, "function", source_indices)
        .and_then(|n| symbol_name_from_expr(&n, source_indices))
        .unwrap_or_default();
    (entry, function)
}

fn defining_module_for_resolved_type(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    type_name: &str,
) -> Option<String> {
    let si = Rc::new(source_indices.clone());
    for tm in graph.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        if lookup_type_by_name(tm.type_env.clone(), type_name.to_string()).is_some() {
            return Some(mod_name);
        }
    }
    let parent_enum = graph
        .emit_graph_info
        .variant_to_enum
        .get(type_name)
        .cloned()?;
    for tm in graph.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        if lookup_type_by_name(tm.type_env.clone(), parent_enum.clone()).is_some() {
            return Some(mod_name);
        }
    }
    None
}

fn lookup_resolved_type_node(graph: &ResolvedGraph, type_name: &str) -> Option<Rc<Node>> {
    for tm in graph.modules.iter() {
        if let Some(node) = lookup_type_by_name(tm.type_env.clone(), type_name.to_string()) {
            return Some(node);
        }
    }
    None
}

fn declared_type_name_from_annotation(
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    type_annotation: &Rc<Node>,
) -> Option<String> {
    let si = Rc::new(source_indices.clone());
    let name = authored_name_at(si, type_annotation.clone());
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn resolved_decl_ref_from_type_name(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    name: &str,
) -> Result<ResolvedDeclRef, String> {
    let module = defining_module_for_resolved_type(graph, source_indices, name)
        .ok_or_else(|| format!("no defining module for resolved type '{}'", name))?;
    Ok(ResolvedDeclRef {
        module,
        name: name.to_string(),
    })
}

fn resolved_initializer_decl_ref(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    body: &Rc<Node>,
    type_annotation: Option<&Rc<Node>>,
) -> Result<ResolvedDeclRef, String> {
    let si = Rc::new(source_indices.clone());
    if let ExprData::ExprRecordLit { parent_enum } = &*body.expr_data {
        if let Some(parent_name) = parent_enum.as_deref() {
            let variant_name = authored_name_at(si.clone(), body.clone());
            if variant_name.is_empty() {
                return Err(
                    "coproduct variant initializer missing constructor identity".to_string()
                );
            }
            let parent_type = lookup_resolved_type_node(graph, parent_name).ok_or_else(|| {
                format!(
                    "resolved parent coproduct '{}' not found in typed graph",
                    parent_name
                )
            })?;
            if !has_child_named(parent_type, variant_name.clone(), si.clone()) {
                return Err(format!(
                    "'{}' is not a resolved variant arm of coproduct '{}'",
                    variant_name, parent_name
                ));
            }
            let module = defining_module_for_resolved_type(graph, source_indices, parent_name)
                .ok_or_else(|| {
                    format!(
                        "no defining module for resolved coproduct '{}'",
                        parent_name
                    )
                })?;
            return Ok(ResolvedDeclRef {
                module,
                name: variant_name,
            });
        }
    }

    let inferred_name = match body.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => {
            let resolved_name = authored_name_at(si.clone(), node.clone());
            if resolved_name.is_empty() {
                None
            } else {
                Some(resolved_name)
            }
        }
        Some(InferredNode::CompilerError { message, .. }) => {
            return Err(format!("unresolved initializer type: {}", message));
        }
        Some(InferredNode::TypeVariable { .. }) => {
            return Err("unresolved initializer type variable".to_string());
        }
        None => None,
    };
    if let Some(name) = inferred_name {
        return resolved_decl_ref_from_type_name(graph, source_indices, &name);
    }
    if let Some(ann) = type_annotation {
        if let Some(name) = declared_type_name_from_annotation(source_indices, ann) {
            return resolved_decl_ref_from_type_name(graph, source_indices, &name);
        }
    }
    Err(
        "resolved initializer has empty type identity (no inferred head or declared annotation)"
            .to_string(),
    )
}

fn is_resolved_bool_witness_claim(resolved: &ResolvedDeclRef) -> bool {
    resolved.module == UNIFIED_CLAIM_VERIFICATION_MODULE && resolved.name == BOOL_WITNESS_CLAIM_TYPE
}

fn is_resolved_node_corpus(resolved: &ResolvedDeclRef) -> bool {
    resolved.module == UNIFIED_CLAIM_VERIFICATION_MODULE && resolved.name == NODE_CORPUS_TYPE
}

fn entry_typed_module(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_module: &str,
) -> Result<Rc<TypedModule>, String> {
    let si = Rc::new(source_indices.clone());
    graph
        .modules
        .iter()
        .find(|tm| authored_name_at(si.clone(), tm.module.clone()) == entry_module)
        .cloned()
        .ok_or_else(|| {
            format!(
                "entry module '{}' not found in resolved graph",
                entry_module
            )
        })
}

fn owned_data_initializer_from_body(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_path: &str,
    decl_name: &str,
    body: &Rc<Node>,
    type_annotation: Option<&Rc<Node>>,
) -> Result<OwnedDataDeclInitializer, String> {
    let resolved_initializer =
        resolved_initializer_decl_ref(graph, source_indices, body, type_annotation)
            .map_err(|e| format!("{entry_path}: owned data '{decl_name}': {e}"))?;
    if is_resolved_bool_witness_claim(&resolved_initializer) {
        let (witness_entry, witness_function) =
            extract_bool_witness_transport(body, source_indices);
        if witness_entry.is_empty() || witness_function.is_empty() {
            return Err(format!(
                "{}: owned data '{}' has malformed BoolWitnessClaim witness (missing entry and/or function)",
                entry_path, decl_name
            ));
        }
        return Ok(OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        });
    }
    if is_resolved_node_corpus(&resolved_initializer) {
        return Ok(OwnedDataDeclInitializer::NodeCorpus);
    }
    Ok(OwnedDataDeclInitializer::Other {
        resolved: resolved_initializer,
    })
}

/// Owned top-level `data` decls declared in the entry module (not imported-closure decls).
pub fn owned_data_decls_for_entry(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_path: &str,
    entry_module: &str,
) -> Result<Vec<OwnedDataDeclRecord>, String> {
    let si = Rc::new(source_indices.clone());
    let typed_module = entry_typed_module(graph, source_indices, entry_module)
        .map_err(|e| format!("{entry_path}: {e}"))?;

    let mut records = Vec::new();
    for item in typed_module.items.iter() {
        if item.body.is_none() || item.type_annotation.is_none() {
            continue;
        }
        let decl_name = authored_name_at(si.clone(), item.clone());
        if decl_name.is_empty() {
            return Err(format!(
                "{entry_path}: owned data item in module '{}' missing authored name",
                entry_module
            ));
        }
        let info = graph.item_registry.get(&decl_name).ok_or_else(|| {
            format!(
                "{entry_path}: owned data '{}' missing from item_registry",
                decl_name
            )
        })?;
        if info.kind != ItemKind::DataItem {
            continue;
        }
        if info.module_name != entry_module {
            return Err(format!(
                "{entry_path}: item_registry module mismatch for '{}' (expected {}, got {})",
                decl_name, entry_module, info.module_name
            ));
        }
        if info.name != decl_name {
            return Err(format!(
                "{entry_path}: item_registry name mismatch for '{}' (registry name '{}')",
                decl_name, info.name
            ));
        }
        // Consolidation #4553 co-location convention: corpus markers are named
        // `data unified_claim_*` (not name-agnostic resolved-type scan). Membership
        // arm is still resolved from the typed graph; the prefix is an authored
        // roster anchor, not a grep substitute.
        if !decl_name.starts_with("unified_claim_") {
            continue;
        }
        let body = item.body.as_ref().ok_or_else(|| {
            format!(
                "{entry_path}: owned data '{}' missing initializer body",
                decl_name
            )
        })?;
        let initializer = owned_data_initializer_from_body(
            graph,
            source_indices,
            entry_path,
            &decl_name,
            body,
            item.type_annotation.as_ref(),
        )?;
        records.push(OwnedDataDeclRecord {
            entry: entry_path.to_string(),
            module: entry_module.to_string(),
            decl_name,
            initializer,
        });
    }

    let discovered: HashSet<&str> = records.iter().map(|r| r.decl_name.as_str()).collect();
    for (decl_name, info) in graph.item_registry.iter() {
        if info.kind == ItemKind::DataItem
            && info.module_name == entry_module
            && decl_name.starts_with("unified_claim_")
        {
            if !discovered.contains(decl_name.as_str()) {
                return Err(format!(
                    "{entry_path}: item_registry data '{}' not found in entry module items",
                    decl_name
                ));
            }
        }
    }

    records.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.decl_name.cmp(&b.decl_name))
    });
    Ok(records)
}

fn path_excluded(path: &Path, exclude_subpaths: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    exclude_subpaths
        .iter()
        .any(|ex| !ex.is_empty() && path_str.contains(ex))
}

/// Cheap pre-scan before per-entry resolve: corpus markers are co-located
/// `data unified_claim_*: UnifiedTestClaim = ...` owned decls.
fn entry_likely_has_unified_claim_owned_data(content: &str) -> bool {
    content
        .lines()
        .any(|line| line.trim_start().starts_with("data unified_claim_"))
}

/// Top-level decl names declared by one source file (column-0 item keywords).
/// Used ONLY to group entry closures into collision-free merged resolves; the
/// discovered facts themselves still come exclusively from the resolved graph.
fn top_level_decl_names(content: &str) -> Vec<String> {
    const ITEM_KEYWORDS: [&str; 8] = [
        "data ",
        "fn ",
        "func ",
        "type ",
        "service ",
        "const ",
        "pattern ",
        "resource ",
    ];
    let mut names = Vec::new();
    for line in content.lines() {
        let Some(rest) = ITEM_KEYWORDS.iter().find_map(|kw| line.strip_prefix(kw)) else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            names.push(name);
        }
    }
    names
}

/// One merged resolve unit: entry files whose combined import closures declare
/// no top-level name twice, so a single `compile_to_resolved` over the union
/// yields the same per-entry facts as per-entry resolves.
struct DiscoveryResolveGroup {
    /// (entry path, entry module, count of column-0 `data unified_claim_` markers).
    entries: Vec<(String, String, usize)>,
    /// file path -> source, union of member entry closures.
    sources: HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    /// top-level decl name -> declaring file path.
    decl_names: HashMap<String, String>,
}

/// `None` if the closure can merge into the group; otherwise the first
/// top-level decl-name collision, as `(name, file already in group, new file)`.
fn closure_group_conflict(
    group: &DiscoveryResolveGroup,
    closure: &[Rc<v1_compiler_compile::SourceFile>],
    names_by_file: &HashMap<String, Rc<Vec<String>>>,
) -> Option<(String, String, String)> {
    for source in closure {
        if group.sources.contains_key(&source.path) {
            continue;
        }
        for name in names_by_file[&source.path].iter() {
            if let Some(existing) = group.decl_names.get(name) {
                if existing != &source.path {
                    return Some((name.clone(), existing.clone(), source.path.clone()));
                }
            }
        }
    }
    None
}

fn add_closure_to_group(
    group: &mut DiscoveryResolveGroup,
    closure: Vec<Rc<v1_compiler_compile::SourceFile>>,
    names_by_file: &HashMap<String, Rc<Vec<String>>>,
) {
    for source in closure {
        if group.sources.contains_key(&source.path) {
            continue;
        }
        for name in names_by_file[&source.path].iter() {
            group.decl_names.insert(name.clone(), source.path.clone());
        }
        group.sources.insert(source.path.clone(), source);
    }
}

/// Discovery output: resolved-type owned-data records plus the resolve-count
/// receipt consumed by the CI latency ratchet (`--max-resolves`).
pub struct OwnedDataDiscovery {
    pub records: Vec<OwnedDataDeclRecord>,
    pub entry_count: usize,
    /// Number of `compile_to_resolved` graph resolves performed. 1 unless a
    /// top-level decl-name collision between entry closures forces a split.
    pub graph_resolves: usize,
    /// One line per forced group split: the decl-name collision (name, file
    /// already in the group, new file) that made the entry start a new group.
    pub group_split_collisions: Vec<String>,
}

/// Glob claim corpus files, resolve fail-closed, expose owned `data` facts.
///
/// Latency shape (the #4633→ratchet history): a resolve costs ~O(closure), and
/// entry closures overlap almost entirely, so this batches all entries into
/// collision-free groups and resolves each group ONCE instead of resolving per
/// entry (formerly ~46 near-identical full resolves per CI run). Facts are
/// still read per entry from the typed graph; grouping is a pure orchestration
/// change over the same resolve + extraction primitives.
pub fn discover_owned_data_decls(
    source_roots: &[String],
    scan_dir: &str,
    exclude_subpaths: &[String],
) -> Result<OwnedDataDiscovery, String> {
    let scan_path = Path::new(scan_dir);
    if !scan_path.is_dir() {
        return Err(format!("scan dir does not exist: {}", scan_dir));
    }

    let mut files = Vec::new();
    collect_dag_files(scan_path, &mut files);
    files.retain(|p| !path_excluded(p, exclude_subpaths));

    let module_index = build_module_index(source_roots);

    let mut names_by_file: HashMap<String, Rc<Vec<String>>> = HashMap::new();
    let mut groups: Vec<DiscoveryResolveGroup> = Vec::new();
    let mut group_split_collisions: Vec<String> = Vec::new();
    let mut entry_count = 0usize;
    for path in files {
        let entry = path.to_string_lossy().to_string();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {:?}: {}", path, e))?;
        if !entry_likely_has_unified_claim_owned_data(&content) {
            continue;
        }
        let entry_module = extract_module_path(&content).ok_or_else(|| {
            format!(
                "missing module declaration in entry {}; cannot classify owned decls",
                entry
            )
        })?;
        let marker_count = content
            .lines()
            .filter(|line| line.starts_with("data unified_claim_"))
            .count();
        entry_count += 1;

        let closure = load_sources_for_entry_with_index(&module_index, &entry)?;
        for source in &closure {
            names_by_file
                .entry(source.path.clone())
                .or_insert_with(|| Rc::new(top_level_decl_names(&source.content)));
        }

        let member = (entry, entry_module, marker_count);
        let mut first_conflict: Option<(String, String, String)> = None;
        match groups.iter_mut().find(|g| {
            match closure_group_conflict(g, &closure, &names_by_file) {
                None => true,
                Some(conflict) => {
                    first_conflict.get_or_insert(conflict);
                    false
                }
            }
        }) {
            Some(group) => {
                group.entries.push(member);
                add_closure_to_group(group, closure, &names_by_file);
            }
            None => {
                if let Some((name, existing_file, new_file)) = first_conflict {
                    group_split_collisions.push(format!(
                        "entry {} split off over decl `{}` ({} vs {})",
                        member.0, name, existing_file, new_file
                    ));
                }
                let mut group = DiscoveryResolveGroup {
                    entries: vec![member],
                    sources: HashMap::new(),
                    decl_names: HashMap::new(),
                };
                add_closure_to_group(&mut group, closure, &names_by_file);
                groups.push(group);
            }
        }
    }

    let graph_resolves = groups.len();
    let mut all_records = Vec::new();
    for group in groups {
        let mut sources: Vec<Rc<v1_compiler_compile::SourceFile>> =
            group.sources.into_values().collect();
        sources.sort_by(|a, b| a.path.cmp(&b.path));
        let (graph, source_indices) = resolved_graph_from_sources(sources)?;
        let si: HashMap<String, Rc<NewlineIndex>> = source_indices
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (entry, entry_module, marker_count) in group.entries {
            let records = owned_data_decls_for_entry(&graph, &si, &entry, &entry_module)?;
            // Fail-closed guard on the merged-resolve path: every column-0
            // `data unified_claim_` marker must surface as a resolved record;
            // a shortfall means a registry collision swallowed a decl.
            if records.len() != marker_count {
                return Err(format!(
                    "{}: merged-resolve discovery found {} owned unified_claim record(s) but the entry declares {} top-level `data unified_claim_` marker(s)",
                    entry,
                    records.len(),
                    marker_count
                ));
            }
            all_records.extend(records);
        }
    }

    all_records.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.decl_name.cmp(&b.decl_name))
    });
    verify_bool_witness_transport_projection_complete(&all_records)?;
    Ok(OwnedDataDiscovery {
        records: all_records,
        entry_count,
        graph_resolves,
        group_split_collisions,
    })
}

pub fn bool_witness_claim_arm_count(records: &[OwnedDataDeclRecord]) -> usize {
    records
        .iter()
        .filter(|r| {
            matches!(
                r.initializer,
                OwnedDataDeclInitializer::BoolWitnessClaim { .. }
            )
        })
        .count()
}

fn unified_claim_arm_count(records: &[OwnedDataDeclRecord]) -> usize {
    records
        .iter()
        .filter(|r| {
            matches!(
                r.initializer,
                OwnedDataDeclInitializer::BoolWitnessClaim { .. }
                    | OwnedDataDeclInitializer::NodeCorpus
            )
        })
        .count()
}

fn illegal_other_init_count(records: &[OwnedDataDeclRecord]) -> usize {
    records
        .iter()
        .filter(|r| {
            let OwnedDataDeclInitializer::Other { resolved } = &r.initializer else {
                return false;
            };
            is_resolved_bool_witness_claim(resolved) || is_resolved_node_corpus(resolved)
        })
        .count()
}

/// Host-computed discovery receipt for modeled standing gates (scalar checks, no list fold).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedDataDiscoveryReceipt {
    pub unified_claim_arm_count: usize,
    pub bool_witness_claim_arm_count: usize,
    pub illegal_other_init_count: usize,
    pub bool_witness_transport_row_count: usize,
    pub transport_projection_complete: bool,
}

/// Inline manifest list only for small fixture-scale scans; large corpus uses receipt + TSV.
pub const MANIFEST_INLINE_LIST_MAX: usize = 64;

pub fn compute_owned_data_discovery_receipt(
    records: &[OwnedDataDeclRecord],
) -> Result<OwnedDataDiscoveryReceipt, String> {
    verify_bool_witness_transport_projection_complete(records)?;
    let bool_witness_transport_row_count = owned_data_bool_witness_transport_tsv(records)?
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let bool_witness_claim_arm_count = bool_witness_claim_arm_count(records);
    let illegal = illegal_other_init_count(records);
    Ok(OwnedDataDiscoveryReceipt {
        unified_claim_arm_count: unified_claim_arm_count(records),
        bool_witness_claim_arm_count,
        illegal_other_init_count: illegal,
        bool_witness_transport_row_count,
        transport_projection_complete: illegal == 0
            && bool_witness_claim_arm_count == bool_witness_transport_row_count,
    })
}

/// Fail-closed: every BoolWitnessClaim arm projects to exactly one transport row.
pub fn verify_bool_witness_transport_projection_complete(
    records: &[OwnedDataDeclRecord],
) -> Result<(), String> {
    let arm_count = bool_witness_claim_arm_count(records);
    let tsv = owned_data_bool_witness_transport_tsv(records)?;
    let row_count = tsv.lines().filter(|l| !l.is_empty()).count();
    if arm_count != row_count {
        return Err(format!(
            "BoolWitnessClaim arm count ({arm_count}) != transport projection row count ({row_count})"
        ));
    }
    Ok(())
}

fn dag_string_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn manifest_symbol_for_resolved_decl(module: &str, name: &str) -> String {
    match (module, name) {
        (UNIFIED_CLAIM_VERIFICATION_MODULE, BOOL_WITNESS_CLAIM_TYPE) => {
            "unified_claim_arm_bool_witness_claim".to_string()
        }
        (UNIFIED_CLAIM_VERIFICATION_MODULE, NODE_CORPUS_TYPE) => {
            "unified_claim_arm_node_corpus".to_string()
        }
        _ => format!("^{}", name),
    }
}

fn emit_owned_data_initializer(initializer: &OwnedDataDeclInitializer) -> String {
    match initializer {
        OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        } => format!(
            "    initializer: OwnedBoolWitnessClaimInit {{\n      witness_entry: \"{}\",\n      witness_function: \"{}\"\n    }}",
            dag_string_escape(witness_entry),
            dag_string_escape(witness_function)
        ),
        OwnedDataDeclInitializer::NodeCorpus => {
            "    initializer: OwnedNodeCorpusInit".to_string()
        }
        OwnedDataDeclInitializer::Other { resolved } => format!(
            "    initializer: OwnedOtherInit {{\n      resolved: ResolvedDeclRef {{\n        module: \"{}\",\n        name: {}\n      }}\n    }}",
            dag_string_escape(&resolved.module),
            manifest_symbol_for_resolved_decl(&resolved.module, &resolved.name)
        ),
    }
}

/// Emit an ephemeral importable `.dag` manifest (never committed).
pub fn emit_owned_data_manifest(
    path: &Path,
    records: &[OwnedDataDeclRecord],
) -> Result<(), String> {
    let receipt = compute_owned_data_discovery_receipt(records)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create manifest parent {:?}: {}", parent, e))?;
    }

    let mut out = String::new();
    out.push_str(
        "// GENERATED by discover_owned_data — ephemeral host transport. DO NOT COMMIT.\n",
    );
    out.push_str("module v2.test.claim.workflow.host_discovered_owned_data_manifest\n\n\n");
    out.push_str("import v2.std.collection { List }\n");
    out.push_str("import v2.std.logic { Bool }\n");
    out.push_str(
        "import v2.compiler.discovery_enumeration {\n  OwnedBoolWitnessClaimInit,\n  OwnedDataDeclRecord,\n  OwnedDataDiscoveryReceipt,\n  OwnedNodeCorpusInit,\n  OwnedOtherInit,\n  ResolvedDeclRef,\n  unified_claim_arm_bool_witness_claim,\n  unified_claim_arm_node_corpus\n}\n\n\n",
    );
    out.push_str("data host_owned_data_discovery_receipt: OwnedDataDiscoveryReceipt = OwnedDataDiscoveryReceipt {\n");
    out.push_str(&format!(
        "  unified_claim_arm_count: {},\n",
        receipt.unified_claim_arm_count
    ));
    out.push_str(&format!(
        "  bool_witness_claim_arm_count: {},\n",
        receipt.bool_witness_claim_arm_count
    ));
    out.push_str(&format!(
        "  illegal_other_init_count: {},\n",
        receipt.illegal_other_init_count
    ));
    out.push_str(&format!(
        "  bool_witness_transport_row_count: {},\n",
        receipt.bool_witness_transport_row_count
    ));
    out.push_str(&format!(
        "  transport_projection_complete: {}\n",
        receipt.transport_projection_complete
    ));
    out.push_str("}\n\n\n");
    let inline_records = if records.len() <= MANIFEST_INLINE_LIST_MAX {
        records
    } else {
        &[]
    };
    if inline_records.is_empty() && !records.is_empty() {
        out.push_str(
            "// Large corpus: inline list omitted; standing gates use host_owned_data_discovery_receipt + transport sidecar.\n",
        );
    }
    out.push_str("data host_discovered_owned_decls: List<OwnedDataDeclRecord> = [\n");
    for (idx, rec) in inline_records.iter().enumerate() {
        if idx > 0 {
            out.push(',');
            out.push('\n');
        }
        out.push_str("  OwnedDataDeclRecord {\n");
        out.push_str(&format!(
            "    entry: \"{}\",\n",
            dag_string_escape(&rec.entry)
        ));
        out.push_str(&format!(
            "    module: \"{}\",\n",
            dag_string_escape(&rec.module)
        ));
        out.push_str(&format!(
            "    decl_name: \"{}\",\n",
            dag_string_escape(&rec.decl_name)
        ));
        out.push_str(&format!(
            "{}\n",
            emit_owned_data_initializer(&rec.initializer)
        ));
        out.push_str("  }");
    }
    out.push_str("\n]\n");

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

/// TSV rows for BoolWitnessClaim transport: `label<TAB>entry<TAB>function`.
pub fn owned_data_bool_witness_transport_tsv(
    records: &[OwnedDataDeclRecord],
) -> Result<String, String> {
    let mut rows = Vec::new();
    for rec in records {
        let OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        } = &rec.initializer
        else {
            continue;
        };
        if witness_entry.is_empty() || witness_function.is_empty() {
            return Err(format!(
                "{}: owned data '{}' has malformed BoolWitnessClaim witness transport (missing entry and/or function)",
                rec.entry, rec.decl_name
            ));
        }
        let label = rec
            .decl_name
            .strip_prefix("unified_claim_")
            .unwrap_or(rec.decl_name.as_str());
        rows.push(format!("{label}\t{witness_entry}\t{witness_function}"));
    }
    rows.sort();
    let mut out = rows.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// discover_source_root_ingest — host transport for Stage C Lane 3a SourceRootIngest
// ---------------------------------------------------------------------------
// Mirrors v2.compiler.source_authority.{DagSourceReadWitness, SourceRootIngest,
// SourceRootProvenanceCoverageReceipt}. Keep field shapes in lockstep with that .dag authority.

/// One host-read `.dag` source file projected to the modeled ingest witness shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceRootReadRecord {
    pub file_path: String,
    pub module_path: String,
    pub source: String,
}

fn source_root_ingest_symbol_from_stem(stem: &str) -> String {
    let mut out = String::from('^');
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out == "^" {
        out.push_str("host_sr_empty");
    }
    out
}

fn source_root_ingest_artifact_id_for_path(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("host_sr");
    source_root_ingest_symbol_from_stem(stem)
}

fn source_root_ingest_compilation_unit_for_path(path: &str) -> String {
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
    let mut hash = 0xcbf29ce484222325u64;
    for byte in material.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn path_matches_any_subpath(path: &str, subpaths: &[String]) -> bool {
    subpaths
        .iter()
        .any(|sub| path.contains(sub) || path.ends_with(sub))
}

/// Walk `scan_dir` for `.dag` files, read source text, fail-closed on missing module
/// headers or duplicate module paths. `source_roots` must exist (compile overlay roots).
pub fn discover_source_root_reads(
    source_roots: &[String],
    scan_dir: &str,
    exclude_subpaths: &[String],
) -> Result<Vec<SourceRootReadRecord>, String> {
    for root in source_roots {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(format!(
                "discover_source_root_ingest: source root does not exist: {}",
                root
            ));
        }
    }

    let scan_path = Path::new(scan_dir);
    if !scan_path.is_dir() {
        return Err(format!(
            "discover_source_root_ingest: scan dir does not exist: {}",
            scan_dir
        ));
    }

    let mut records: Vec<SourceRootReadRecord> = Vec::new();
    let mut seen_modules: HashMap<String, String> = HashMap::new();
    let mut dag_files = Vec::new();
    collect_dag_files(scan_path, &mut dag_files);

    for path in dag_files {
        let rel_forward = path.to_string_lossy().replace('\\', "/");
        if path_matches_any_subpath(&rel_forward, exclude_subpaths) {
            continue;
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {:?}: {}", path, e))?;
        let module_path = extract_module_path(&content).ok_or_else(|| {
            format!(
                "discover_source_root_ingest: no module declaration in {}",
                rel_forward
            )
        })?;
        if let Some(prior) = seen_modules.insert(module_path.clone(), rel_forward.clone()) {
            return Err(format!(
                "discover_source_root_ingest: duplicate module path '{}' in {} and {}",
                module_path, prior, rel_forward
            ));
        }
        records.push(SourceRootReadRecord {
            file_path: rel_forward,
            module_path,
            source: content,
        });
    }

    records.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(records)
}

fn emit_source_root_read_witness(rec: &SourceRootReadRecord) -> String {
    let artifact_id = source_root_ingest_artifact_id_for_path(&rec.file_path);
    let compilation_unit = source_root_ingest_compilation_unit_for_path(&rec.file_path);
    format!(
        "DagSourceReadWitness {{\n  source: \"{}\",\n  artifact: Artifact {{\n    kind: SourceFile,\n    id: {artifact_id},\n    file_path: \"{}\"\n  }},\n  compilation_unit: {compilation_unit}\n}}",
        dag_string_escape(&rec.source),
        dag_string_escape(&rec.file_path),
    )
}

fn emit_source_root_ingest_monoid(records: &[SourceRootReadRecord]) -> String {
    let mut witness_nodes: Vec<String> = records
        .iter()
        .map(emit_source_root_read_witness)
        .collect();
    let mut out = String::from("Empty");
    while let Some(head) = witness_nodes.pop() {
        out = format!("Cons {{\n  head: {head},\n  tail: {out}\n}}");
    }
    out
}

/// Emit an ephemeral importable `.dag` manifest (never committed).
pub fn emit_source_root_ingest_manifest(
    path: &Path,
    records: &[SourceRootReadRecord],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create manifest parent {:?}: {}", parent, e))?;
    }

    let content_hash = source_root_ingest_content_hash_fnv1a64(records);
    let read_count = records.len();
    let inline_records = if read_count <= MANIFEST_INLINE_LIST_MAX {
        records
    } else {
        &[]
    };

    let mut out = String::new();
    out.push_str(
        "// GENERATED by discover_source_root_ingest — ephemeral host transport. DO NOT COMMIT.\n",
    );
    out.push_str("module v2.test.workflow.host_source_root_ingest_manifest\n\n\n");
    out.push_str("import v2.compiler.source_authority {\n");
    out.push_str("  DagSourceReadWitness,\n");
    out.push_str("  SourceRootIngest,\n");
    out.push_str("  SourceRootProvenanceCoverageReceipt\n");
    out.push_str("}\n");
    out.push_str("import v2.std.algebra { Cons, Empty }\n");
    out.push_str("import v2.std.artifact { Artifact, SourceFile }\n");
    out.push_str("import v2.std.text { String }\n\n\n");
    out.push_str(&format!(
        "data host_source_root_ingest_content_hash: String = \"{}\"\n\n\n",
        dag_string_escape(&content_hash)
    ));
    out.push_str("data host_source_root_ingest_coverage_receipt: SourceRootProvenanceCoverageReceipt = SourceRootProvenanceCoverageReceipt {\n");
    out.push_str(&format!("  ingest_read_count: {read_count},\n"));
    out.push_str(&format!("  produced_row_count: {read_count},\n"));
    out.push_str("  coverage_complete: true\n");
    out.push_str("}\n\n\n");
    if inline_records.is_empty() && !records.is_empty() {
        out.push_str(
            "// Large corpus: inline ingest omitted; standing gates use host_source_root_ingest_coverage_receipt scalars.\n",
        );
    }
    out.push_str("data host_source_root_ingest: SourceRootIngest = ");
    if inline_records.is_empty() {
        out.push_str("Empty\n");
    } else {
        out.push_str(&emit_source_root_ingest_monoid(inline_records));
        out.push('\n');
    }

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

fn resolved_pipeline_from_cached_graph(
    graph: Rc<ResolvedGraph>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Rc<v1_compiler_compile::ResolvedPipelineResult> {
    let ownership = v1_compiler_compile::extract_ownership_proofs(graph.clone());
    let ownership_diags = v1_compiler_compile::ownership_diagnostics(ownership.clone());
    let newline_indices = Rc::new(source_indices.values().cloned().collect::<Vec<_>>());
    Rc::new(v1_compiler_compile::ResolvedPipelineResult {
        graph: Some(graph),
        diagnostics: ownership_diags,
        source_indices,
        complexity: crate::v1_compiler_complexity::empty_complexity_report(),
        ownership,
        newline_indices,
    })
}

fn resolved_has_blocking_errors(resolved: &v1_compiler_compile::ResolvedPipelineResult) -> bool {
    resolved
        .diagnostics
        .iter()
        .any(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
}

/// Whole-tree compile with resolved_graph_cache lookup/write (kernel extension for
/// dsl_compile_clean; authority: dsl/tools/dsl_compile_clean_memo.dag).
pub fn compile_sources_with_resolved_graph_cache(
    sources: Rc<Vec<Rc<v1_compiler_compile::SourceFile>>>,
    target: crate::v1_compiler_artifact::RenderTarget,
) -> Rc<v1_compiler_compile::PipelineResult> {
    let subject = subject_digest_for_closure(sources.as_ref());
    if let Some(cache_root) = resolved_graph_cache_root_from_env() {
        if let CacheLookupResult::Hit(hit) = cross_process_lookup(&cache_root, &subject) {
            let resolved =
                resolved_pipeline_from_cached_graph(hit.graph.clone(), hit.source_indices.clone());
            if !resolved_has_blocking_errors(resolved.as_ref()) {
                return v1_compiler_compile::emit_resolved_for_target(resolved, target);
            }
        }
    }

    let resolved = v1_compiler_compile::compile_to_resolved(sources.clone());
    if !resolved_has_blocking_errors(resolved.as_ref()) {
        if let (Some(cache_root), Some(graph)) =
            (resolved_graph_cache_root_from_env(), resolved.graph.as_ref())
        {
            let si: HashMap<String, Rc<NewlineIndex>> = resolved
                .source_indices
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let _ = cross_process_write(&cache_root, &subject, graph, &si);
        }
    }
    v1_compiler_compile::emit_resolved_for_target(resolved, target)
}

/// Import-driven source closure for `gunbc compile --source-root` (first root = entries).
pub fn load_compile_sources(source_roots: &[String]) -> Vec<Rc<v1_compiler_compile::SourceFile>> {
    load_sources(source_roots)
}
