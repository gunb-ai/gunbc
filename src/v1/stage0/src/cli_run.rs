// cli_run.rs — Hand-maintained Run subcommand handler.
// Not generated — survives stage0 regeneration.
// The generated main.rs calls handle_run_with_options() for the Run subcommand.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::{kernel_type_set, SourceSpan};
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::lookup_type_by_name;
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_rt;
use crate::v1_std_core::{
    authored_name_at, build_newline_index, byte_to_line_col, diagnostic_to_message,
    diagnostic_to_span, empty_intern_table, expr_var_name_at, field_init_node_name_at,
    field_init_node_value, has_child_named, intern,
    is_discovery_corpus_advisory_typecheck_diagnostic, is_discovery_corpus_blocking_diagnostic,
    is_error_diagnostic, is_interpreter_blocking_diagnostic, CompilerDiagnostic, ErrorNode,
    ExprData, InferredNode, InternTable, NewlineIndex, Node,
};
use serde::Serialize;

use crate::resolved_graph_cache::{
    lookup as cross_process_lookup, resolved_graph_cache_root_from_env, subject_digest_for_closure,
    write as cross_process_write, CacheLookupResult, CacheWriteOutcome,
};

/// Which typecheck diagnostics block entry resolve. Strict is the default for
/// gunbc run/compile and all non-discovery consumers; DiscoveryCorpusAdvisory
/// is the SCAFFOLD carve-out (00_core.dag) for floor discovery scan-coverage only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveTypecheckGate {
    Strict,
    DiscoveryCorpusAdvisory,
}

fn is_resolve_typecheck_blocking(d: Rc<CompilerDiagnostic>, gate: ResolveTypecheckGate) -> bool {
    match gate {
        ResolveTypecheckGate::Strict => is_interpreter_blocking_diagnostic(d),
        ResolveTypecheckGate::DiscoveryCorpusAdvisory => is_discovery_corpus_blocking_diagnostic(d),
    }
}

fn log_discovery_advisory_typecheck(
    d: &Rc<ErrorNode>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    gate: ResolveTypecheckGate,
) {
    if gate != ResolveTypecheckGate::DiscoveryCorpusAdvisory {
        return;
    }
    if is_discovery_corpus_advisory_typecheck_diagnostic(d.diagnostic.clone())
        && is_interpreter_blocking_diagnostic(d.diagnostic.clone())
    {
        let span = diagnostic_to_span(d.diagnostic.clone());
        let loc = format_error_loc(&span.file, span.start, source_indices);
        eprintln!(
            "advisory(typecheck): {}: error: {}",
            loc,
            diagnostic_to_message(d.diagnostic.clone())
        );
    }
}

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
pub(crate) fn extract_import_paths(content: &str) -> Vec<String> {
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

/// Workspace root (parent of `src/`).
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

/// `module_path` → repo-relative `.dag` path; fail-closed on duplicate module paths.
pub fn build_module_path_index(source_roots: &[String]) -> HashMap<String, String> {
    let ws = workspace_root();
    let mut index = HashMap::new();
    for root in source_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut dag_files = Vec::new();
        collect_dag_files(root_path, &mut dag_files);
        for path in dag_files {
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("build_module_path_index: failed to read {:?}: {}", path, e)
            });
            if let Some(module_path) = extract_module_path(&content) {
                let rel = path
                    .strip_prefix(&ws)
                    .unwrap_or_else(|_| {
                        panic!(
                            "build_module_path_index: path {} is not under workspace {}",
                            path.display(),
                            ws.display()
                        )
                    })
                    .to_string_lossy()
                    .replace('\\', "/");
                // Co-root overlay: later `source_roots` win on duplicate module paths
                // (same last-wins policy as `build_module_index`). TRANSITIONAL (slice 3):
                // enables v2 overlay without panic while dsl+src/v2 coexist. DISSOLUTION
                // RESTORE (final slice): when witness_layer_roots reverts to a single root and
                // dsl/extdeps/shell is deleted, restore fail-closed panic on duplicate module_path
                // so an accidental name collision is loud again (§5).
                index.insert(module_path.clone(), rel);
            }
        }
    }
    index
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
/// Live host memory budget in bytes — the guaranteed slice a process may use before the cgroup
/// OOM-killer fires. The §1-C / DESIGN §3 "read, don't commit" half: the cgroup budget is authored
/// by an EXTERNAL system (the host/scheduler), so the host realizer reads it LIVE while OUR per-shard
/// peak is the committed measured fact. Single authority for budget-reading, shared by the floor
/// scheduler (`claim_executor`) and the co-running cache-purity audit orchestrator.
///
/// Source order: cgroup v2 unified `memory.max` walked leaf→root, reduced by `min` (the EFFECTIVE
/// limit is the tightest ancestor cap), then `/proc/meminfo` `MemAvailable`/`MemTotal` as a fallback
/// for hosts with no cgroup cap. `None` when nothing is readable → the `.dag` width fold falls back
/// to its conservative committed width (fail-closed: never OOM, never fabricated).
pub fn read_host_memory_budget_bytes() -> Option<u64> {
    // cgroup v2 unified hierarchy: `/proc/self/cgroup` is a single `0::/<rel>` line.
    if let Ok(self_cg) = std::fs::read_to_string("/proc/self/cgroup") {
        if let Some(rel) = self_cg
            .lines()
            .find_map(|l| l.strip_prefix("0::"))
            .map(|p| p.trim().trim_start_matches('/').to_string())
        {
            let root = Path::new("/sys/fs/cgroup");
            let mut dir = root.join(&rel);
            let mut effective: Option<u64> = None;
            loop {
                if let Ok(s) = std::fs::read_to_string(dir.join("memory.max")) {
                    let s = s.trim();
                    if s != "max" {
                        if let Ok(v) = s.parse::<u64>() {
                            effective = Some(effective.map_or(v, |cur| cur.min(v)));
                        }
                    }
                }
                if dir == root || !dir.pop() {
                    break;
                }
            }
            if let Some(v) = effective {
                return Some(v);
            }
        }
    }
    // Fallback (no cgroup cap): /proc/meminfo. MemAvailable is volatile/multi-tenant, so it is the
    // fallback only — a transient dip should not collapse width when a real cgroup cap exists.
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for key in ["MemAvailable", "MemTotal"] {
            if let Some(kb) = meminfo
                .lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<u64>().ok())
            {
                return Some(kb.saturating_mul(1024));
            }
        }
    }
    None
}

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
    resolve_entry_with_parse_cache(index, entry_file, ResolveTypecheckGate::Strict)
}

/// Discovery-corpus resolve: threads the SCAFFOLD typecheck advisory gate (floor
/// scan-coverage only). All other entry points use [`resolve_entry_with_index`]
/// (Strict).
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
    resolved_graph_from_sources(sources, ResolveTypecheckGate::Strict)
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
    typecheck_gate: ResolveTypecheckGate,
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
                // Served WARM — this resolve returns the cached bytes without re-auditing (they were
                // audited at their cold write). Record it (under audit) so an all-warm run reads as
                // warm-served, NOT as proof-of-execution (DESIGN §5/§6 — see claim_executor's report).
                if crate::cache_purity_audit::cache_purity_audit_enabled() {
                    crate::cache_purity_audit::record_audit_warm_served();
                }
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

    for d in typed.diagnostics.iter() {
        log_discovery_advisory_typecheck(d, &source_indices, typecheck_gate);
    }
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
        let outcome = cross_process_write(&cache_root, &subject, &typed, source_indices.as_ref());

        // PIGGYBACK warm==cold purity audit (DESIGN §5; ROADMAP §2 P3), armed only on the CI floor
        // run-step (GUNBC_CACHE_PURITY_AUDIT=1). `typed` is the COLD value we just wrote, so the
        // audit costs one warm read-back + a structural `==` (~1×, ~0 added peak) — no second
        // resolve, no second process. Production (flag unset) keeps the best-effort `let _`.
        if crate::cache_purity_audit::cache_purity_audit_enabled() {
            match &outcome {
                Ok(CacheWriteOutcome::Written) => {
                    let reach = crate::cache_purity_audit::audit_warm_readback_against_cold(
                        &cache_root,
                        &subject,
                        &typed,
                        source_indices.as_ref(),
                        entry_file,
                    )?;
                    // Observability (DESIGN §6): record that the audit RAN over this entry, so a
                    // green floor can SHOW audited-N>0 rather than pass vacuously.
                    crate::cache_purity_audit::record_audit_reach(reach);
                }
                // Lost a write race: the bytes another process wrote were audited at THAT cold write
                // (codec-in-key re-keys a codec change → re-write → re-audit). Counts as warm-served.
                Ok(CacheWriteOutcome::AlreadyExists) => {
                    crate::cache_purity_audit::record_audit_warm_served();
                }
                // A write we cannot complete cannot be proven cached==cold — fail closed under audit.
                Err(e) => {
                    return Err(format!(
                        "cache purity audit (in-process): the resolved-graph cache WRITE failed for \
                         entry {entry_file}: {e} — fail-closed (cannot guarantee a cached==cold \
                         round-trip; DESIGN §5)"
                    ));
                }
            }
        }
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
    typecheck_gate: ResolveTypecheckGate,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let result = match typecheck_gate {
        ResolveTypecheckGate::Strict => v1_compiler_compile::compile_to_resolved(Rc::new(sources)),
        ResolveTypecheckGate::DiscoveryCorpusAdvisory => {
            v1_compiler_compile::compile_to_resolved_discovery_corpus_advisory(Rc::new(sources))
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
                log_discovery_advisory_typecheck(d, &si, typecheck_gate);
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
    make_eval_context_with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        None,
    )
}

/// Like [`make_eval_context_with_fixture_store`], but threads M4.1 whole-tree precomputed
/// published-mock keys so hermetic governance is independent of the entry import closure.
pub fn make_eval_context_with_runtime_options(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
) -> v1_interpreter::InterpContext {
    v1_interpreter::InterpContext::with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        whole_tree_published_keys,
    )
}

/// Source roots that name a `dsl/` tree (whole-tree published-mock-corpus authority for M4.1).
fn dsl_source_roots(source_roots: &[String]) -> Vec<String> {
    let mut dsl: Vec<String> = source_roots
        .iter()
        .filter(|r| {
            let p = Path::new(r.as_str());
            p.ends_with("dsl") || p.file_name().is_some_and(|n| n == "dsl")
        })
        .cloned()
        .collect();
    // Workspace-only `--source-root <repo>` still indexes dsl/ via build_multi_entry_index;
    // include `<root>/dsl` when present so universal corpus precompute matches.
    for root in source_roots {
        let child = Path::new(root).join("dsl");
        if child.is_dir() {
            dsl.push(child.to_string_lossy().into_owned());
        }
    }
    dsl.sort();
    dsl.dedup();
    dsl
}

/// Precompute the published mock corpus operation-key set from the whole dsl/ source tree.
/// Independent of any witness entry's import closure — the SAME authority the M2
/// `mock_totality_lens` reads module-by-module.
pub fn precompute_whole_tree_published_mock_keys(
    source_roots: &[String],
) -> Result<std::collections::HashSet<String>, String> {
    let dsl_roots = dsl_source_roots(source_roots);
    if dsl_roots.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let index = build_module_index(&dsl_roots);
    let all_sources: Vec<Rc<v1_compiler_compile::SourceFile>> = index.values().cloned().collect();
    if all_sources.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let (graph, source_indices) =
        resolved_graph_from_sources(all_sources, ResolveTypecheckGate::Strict)?;
    let ctx = v1_interpreter::InterpContext::with_runtime_options(
        &graph,
        source_indices,
        v1_interpreter::ExecutionMode::Wet,
        None,
        None,
    );
    v1_interpreter::resolve_published_mock_keys(&ctx)
        .map_err(|e| format!("whole-tree published mock corpus precompute: {e}"))
}

pub fn closure_subject_for_entry(index: &MultiEntryIndex, entry: &str) -> Result<String, String> {
    let sources = load_sources_for_entry_with_index(&index.source_files, entry)?;
    Ok(subject_digest_for_closure(&sources))
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

/// Run one Bool witness with cache-subject-keyed measurement. Sets the eval-tap
/// subject, records wall + eval self-time into a `PerformanceReceipt`, and leaves
/// verdict classification to the caller. Eval self-time accumulates only when
/// `GUNBC_INTERP_PROFILE=1` (zero overhead on the default green path).
///
/// SCAFFOLD — v1 host wiring only. P5 receipt: CI floor `DiscoverySummary` Measured
/// roll-up + keystone witness. dissolve-on: v2 witness runner Realization carrier
/// (`docs/plans/realization-measurement-loop.md` Phase 0 table).
pub fn run_claim_measured(
    ctx: &v1_interpreter::InterpContext,
    closure_subject_digest: &str,
    function: &str,
) -> (ClaimOutcome, v1_interpreter::PerformanceReceipt) {
    let subject_key =
        crate::resolved_graph_cache::witness_work_subject_key(closure_subject_digest, function);
    v1_interpreter::eval_profile_reset();
    v1_interpreter::eval_subject_set(subject_key.clone());
    let started = std::time::Instant::now();
    let outcome = run_claim(ctx, function);
    let wall_nanos = started.elapsed().as_nanos();
    v1_interpreter::eval_subject_clear();
    let receipt =
        v1_interpreter::performance_receipt_from_witness(subject_key, function, wall_nanos);
    (outcome, receipt)
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
        let (graph, source_indices) =
            resolved_graph_from_sources(sources, ResolveTypecheckGate::DiscoveryCorpusAdvisory)?;
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

fn dag_string_escape_core(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Fail-closed: manifest scalar fields (paths, names, hashes) must not contain `{`/`}`.
fn dag_manifest_scalar_escape(s: &str) -> Result<String, String> {
    if s.contains('{') || s.contains('}') {
        return Err(format!(
            "manifest scalar field must be brace-free (got '{{' or '}}'): {s:?}"
        ));
    }
    Ok(dag_string_escape_core(s))
}

/// Embedded raw `.dag` source in manifest string literals — brace-escape pairs with
/// `v1_compiler_tokenize::process_escapes` (`\{`/`\}` → literal braces).
fn dag_embedded_dag_source_escape(s: &str) -> String {
    dag_string_escape_core(s)
        .replace('{', "\\{")
        .replace('}', "\\}")
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

fn emit_owned_data_initializer(initializer: &OwnedDataDeclInitializer) -> Result<String, String> {
    match initializer {
        OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        } => Ok(format!(
            "    initializer: OwnedBoolWitnessClaimInit {{\n      witness_entry: \"{}\",\n      witness_function: \"{}\"\n    }}",
            dag_manifest_scalar_escape(witness_entry)?,
            dag_manifest_scalar_escape(witness_function)?
        )),
        OwnedDataDeclInitializer::NodeCorpus => {
            Ok("    initializer: OwnedNodeCorpusInit".to_string())
        }
        OwnedDataDeclInitializer::Other { resolved } => Ok(format!(
            "    initializer: OwnedOtherInit {{\n      resolved: ResolvedDeclRef {{\n        module: \"{}\",\n        name: {}\n      }}\n    }}",
            dag_manifest_scalar_escape(&resolved.module)?,
            manifest_symbol_for_resolved_decl(&resolved.module, &resolved.name)
        )),
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
            dag_manifest_scalar_escape(&rec.entry)?
        ));
        out.push_str(&format!(
            "    module: \"{}\",\n",
            dag_manifest_scalar_escape(&rec.module)?
        ));
        out.push_str(&format!(
            "    decl_name: \"{}\",\n",
            dag_manifest_scalar_escape(&rec.decl_name)?
        ));
        out.push_str(&format!(
            "{}\n",
            emit_owned_data_initializer(&rec.initializer)?
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

// ── Floor discovery corpus (shared by claim_batch + claim_executor) ──────────
//
// The discovery ROSTER (which `(entry, function)` Bool witnesses make up the CI
// floor corpus) is a single authority (DESIGN §3): the `--roster-from-discovery`
// scan in `claim_batch` and the scheduler's `DiscoveryBatch` node (driven by
// `claim_executor`) both reflect over the SAME corpus via the functions below,
// rather than either re-coining the roster or duplicating the scan. The exclude
// set, the `unified_claim_*`/`test fn`/`test data` reflection, and the
// `*_test.dag`-only convention live here once.

/// One discovered witness row. `label` is for diagnostics; the `(entry,
/// function)` pair is what gets resolved and run.
#[derive(Clone)]
pub struct DiscoveryRow {
    pub label: String,
    pub entry: String,
    pub function: String,
}

/// Per-entry resolve timing (clock 1 of the audit's two-clock split).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryResolveReceipt {
    pub entry: String,
    pub closure_subject: String,
    pub resolve_nanos: u128,
}

/// One witness verdict from a discovery-corpus run (for wet/hermetic equivalence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryWitnessOutcome {
    pub entry: String,
    pub function: String,
    pub outcome: ClaimOutcome,
}

/// Summary of running the floor discovery corpus: how many witnesses ran, how
/// many passed, how many were skipped (stateless node-frontier), and a rendered
/// failure line per non-pass (empty on full green).
pub struct DiscoverySummary {
    pub total: usize,
    pub passed: usize,
    pub skipped: usize,
    pub failures: Vec<String>,
    /// Per-witness outcomes in roster order (wet/hermetic equivalence gate).
    pub witness_outcomes: Vec<DiscoveryWitnessOutcome>,
    /// Per-entry resolve wall time keyed by closure cache-subject.
    pub entry_resolve_receipts: Vec<EntryResolveReceipt>,
    /// Aggregate resolve wall time across distinct entries (nanoseconds).
    pub total_resolve_nanos: u128,
    /// Per-witness PerformanceReceipt rows keyed by cache-subject (clock 2).
    pub performance_receipts: Vec<v1_interpreter::PerformanceReceipt>,
    /// Aggregate measured witness-eval wall time (nanoseconds) — CostAccount.time roll-up.
    pub total_measured_nanos: u128,
}

/// Enrolled witness `.dag` carrying the scaffold roster entry-prefix authority.
pub const WET_HERMETIC_EQUIVALENCE_WITNESS_ENTRY: &str =
    "dsl/test/claim/wet_hermetic_equivalence_witness_test.dag";
/// `data` binding name for the single-authority roster entry prefix (§3).
pub const WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA: &str =
    "wet_hermetic_equivalence_representative_prefix";

fn resolve_entry_file_under_roots(source_roots: &[String], entry: &str) -> Result<String, String> {
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

/// Load the scaffold roster entry-prefix from the enrolled witness `.dag` data
/// binding. Rust filter code must use this — no parallel substring authority.
pub fn wet_hermetic_scaffold_roster_entry_prefix(
    source_roots: &[String],
) -> Result<String, String> {
    let entry =
        resolve_entry_file_under_roots(source_roots, WET_HERMETIC_EQUIVALENCE_WITNESS_ENTRY)?;
    let (graph, source_indices) = resolve_entry_graph(source_roots, &entry)?;
    let sources = load_sources_for_entry(source_roots, &entry)?;
    let entry_source = sources
        .iter()
        .find(|s| s.path == entry || s.path.ends_with(WET_HERMETIC_EQUIVALENCE_WITNESS_ENTRY))
        .ok_or_else(|| format!("{entry}: missing from entry closure"))?;
    let entry_module = extract_module_path(&entry_source.content)
        .ok_or_else(|| format!("{entry}: missing module declaration"))?;
    let typed_module = entry_typed_module(&graph, &source_indices, &entry_module)?;
    let si = Rc::new((*source_indices).clone());
    for item in typed_module.items.iter() {
        if item.body.is_none() {
            continue;
        }
        let decl_name = authored_name_at(si.clone(), item.clone());
        if decl_name != WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA {
            continue;
        }
        let body = item.body.as_ref().ok_or_else(|| {
            format!("{entry}: data '{WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA}' missing body")
        })?;
        return literal_string_from_expr(body).ok_or_else(|| {
            format!(
                "{entry}: data '{WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA}' must be a string literal"
            )
        });
    }
    Err(format!(
        "{entry}: missing data '{WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA}'"
    ))
}

/// Scaffold roster filter: one discoverable `test fn` per extdeps layer with a
/// published mock corpus under `prefix` (from
/// `wet_hermetic_scaffold_roster_entry_prefix`). These witnesses exercise the M4
/// hermetic-realization *model* in pure `.dag` — they do NOT traverse
/// `eval_service_call` under either Wet or Hermetic, so they cannot observe
/// mock-vs-live faithfulness. Dissolution: replace with witnesses that dispatch
/// live transport under Wet and published-mock under Hermetic.
pub fn is_governed_service_representative_row(row: &DiscoveryRow, prefix: &str) -> bool {
    !prefix.is_empty() && row.entry.contains(prefix)
}

/// Row-for-row outcome comparator for the P3c scaffold gate. Returns divergence
/// lines (empty when equivalent). Teeth: unit-tested with synthetic vectors;
/// roster integration is vacuous until live-transport witnesses enroll.
pub fn wet_hermetic_discovery_outcome_divergences(
    wet: &[DiscoveryWitnessOutcome],
    hermetic: &[DiscoveryWitnessOutcome],
) -> Vec<String> {
    let mut divergences = Vec::new();
    if wet.len() != hermetic.len() {
        divergences.push(format!(
            "roster size mismatch: wet={} hermetic={}",
            wet.len(),
            hermetic.len()
        ));
        return divergences;
    }
    for (w, h) in wet.iter().zip(hermetic.iter()) {
        if w.entry != h.entry || w.function != h.function {
            divergences.push(format!(
                "roster order mismatch: wet=({},{}) hermetic=({},{})",
                w.function, w.entry, h.function, h.entry
            ));
            continue;
        }
        if w.outcome != h.outcome {
            divergences.push(format!(
                "{} ({}): wet={:?} hermetic={:?}",
                w.function, w.entry, w.outcome, h.outcome
            ));
        }
    }
    divergences
}

/// Default exclude set for floor discovery. Manifest/law files that import the
/// ephemeral discovery output would otherwise re-enter discovery acyclically;
/// the manual lane (`test/manual/`) carries its own ExpectPass|ExpectFail
/// expected-outcome (some pinned red to tracking anchors), so a universal-green
/// floor must not run them as must-pass. Single home (was duplicated in the
/// claim_batch bin as `DISCOVERY_EXCLUDES`).
pub const FLOOR_DISCOVERY_EXCLUDES: &[&str] = &[
    "impossible_bug",
    "test/manual/",
    "glob_discovery.dag",
    "glob_discovery_law.dag",
    "host_discovered_owned_data_manifest.dag",
    "host_source_root_ingest_manifest.dag",
    // Gate-only: requires host manifest overlay (v2.workflow.source_root_ingest_gate).
    "program_assembly/real_ingest_test.dag",
    "self_host/compiler_closure_emit_from_ingest_test.dag",
    "unified_test_claim_substrate_equivalence.dag",
];

/// True if `path` matches any `FLOOR_DISCOVERY_EXCLUDES` subpath. Excluded files
/// are skipped by BOTH the `unified_claim_*` scan and the `test fn`/`test data`
/// scan — gate-only / manifest / manual-lane witnesses must not enter the floor.
pub fn floor_discovery_path_excluded(path: &str) -> bool {
    FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .any(|sub| path.contains(sub))
}

/// Tolerant `.dag` walk (silently skips unreadable dirs — a gate must not panic
/// on a transient read error). The eager `collect_dag_files` above panics, which
/// is wrong for the floor scan.
pub(crate) fn collect_dag_files_tolerant(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files_tolerant(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

/// Extract `NAME` from every `test fn NAME(...)` / `test data NAME: ...`
/// declaration in source text (the v2 parser drops the contextual `test`
/// keyword, so the marker is detected in source text — same posture as the
/// `data unified_claim_` scan).
fn scan_test_decl_names(content: &str) -> Vec<String> {
    scan_test_decl_lines(content)
        .into_iter()
        .map(|(name, _line)| name)
        .collect()
}

/// Like `scan_test_decl_names` but pairs each name with its 1-based declaration line. A
/// `test fn` is not an AST `module.items` entry (the parser drops the contextual `test`
/// keyword before the span is recorded), so the node-frontier partition gets its decl line
/// from source text — the only way to bound a witness body the way an item span bounds a
/// `data` decl.
fn scan_test_decl_lines(content: &str) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let rest = trimmed
            .strip_prefix("test fn ")
            .or_else(|| trimmed.strip_prefix("test data "));
        if let Some(rest) = rest {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, (i + 1) as i64));
            }
        }
    }
    out
}

/// Fail-closed filename hygiene: `.dag` basenames under the source roots must not
/// contain `__` (legacy flat-dir encoding; use subdirectories). Preserved from
/// the retired `ci_claim_gate` (#5051).
pub fn check_floor_filename_hygiene(source_roots: &[String]) -> Result<(), String> {
    let mut violations: Vec<String> = Vec::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(Path::new(root), &mut dag_files);
        for path in dag_files {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.contains("__"))
            {
                violations.push(path.to_string_lossy().into_owned());
            }
        }
    }
    if violations.is_empty() {
        return Ok(());
    }
    violations.sort();
    Err(format!(
        "filename hygiene: `.dag` basenames must not contain `__` (use subdirectories); \
         offending file(s): {}",
        violations.join(", ")
    ))
}

/// Reflect over the scan dirs for the floor corpus roster: every discovered
/// `unified_claim_*` BoolWitness decl plus every single-representation
/// `test fn`/`test data` decl (the latter only in `*_test.dag` files — a `test`
/// decl found in an implementation file is a hard error). Rows are deduped on
/// `(entry, function)` and sorted. This is the single roster authority.
pub fn discover_floor_corpus_rows(
    source_roots: &[String],
    scan_dirs: &[String],
) -> Result<Vec<DiscoveryRow>, String> {
    let excludes: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut rows: Vec<DiscoveryRow> = Vec::new();
    let mut seen: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    for scan_dir in scan_dirs {
        let discovery = discover_owned_data_decls(source_roots, scan_dir, &excludes)?;
        for rec in discovery.records {
            if let OwnedDataDeclInitializer::BoolWitnessClaim {
                witness_entry,
                witness_function,
            } = rec.initializer
            {
                if witness_entry.is_empty() || witness_function.is_empty() {
                    return Err(format!(
                        "discovered decl '{}' has malformed BoolWitness transport (entry/function)",
                        rec.decl_name
                    ));
                }
                if seen.insert((witness_entry.clone(), witness_function.clone())) {
                    let label = rec
                        .decl_name
                        .strip_prefix("unified_claim_")
                        .unwrap_or(&rec.decl_name)
                        .to_string();
                    rows.push(DiscoveryRow {
                        label,
                        entry: witness_entry,
                        function: witness_function,
                    });
                }
            }
        }
    }

    let mut test_fn_violations: Vec<String> = Vec::new();
    // Import-closure graph captured during the single walk (zero extra IO) — feeds the
    // inert-lens hygiene backstop below (DESIGN.md §6: an inert lens is a lie). Keyed on
    // repo-relative paths so it is stable whether callers pass absolute (tests) or relative
    // (`claim_batch --source-root src/v2`) roots, matching authored `unified_claim_*` entries.
    let mut path_imports: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut module_to_path: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    // Authoring-time construction-justification rule (DESIGN.md §5/§6, ROADMAP §0): every
    // top-level lens module (`v2.lens.<name>`) must RECORD why its bad-state class is a lens
    // (validation) rather than construction. The judgment is human residue; its PRESENCE is
    // structurable, so we capture which lenses carry the `construction_justification` decl during
    // the same walk (zero extra IO) and fail closed below on any that lack it. This LAYERS ON TOP
    // of the inert-lens backstop — it does not supersede it (DESIGN.md §6).
    let mut lens_with_justification: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(Path::new(root), &mut dag_files);
        dag_files.sort();
        for path in dag_files {
            let entry = path.to_string_lossy().into_owned();
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            // Capture the import graph for EVERY file (even floor-excluded ones — a lens may be
            // reached only through a non-witness implementation file). Seeds are rows only.
            let rel = repo_relative_dag_path(&entry);
            if let Some(m) = extract_module_path(&content) {
                if is_top_level_lens_module(&m) && declares_construction_justification(&content) {
                    lens_with_justification.insert(m.clone());
                }
                module_to_path.insert(m, rel.clone());
            }
            path_imports.insert(rel, extract_import_paths(&content));
            if floor_discovery_path_excluded(&entry) {
                continue;
            }
            let names = scan_test_decl_names(&content);
            if names.is_empty() {
                continue;
            }
            if !entry.ends_with("_test.dag") {
                test_fn_violations.push(entry);
                continue;
            }
            for name in names {
                if seen.insert((entry.clone(), name.clone())) {
                    rows.push(DiscoveryRow {
                        label: name.clone(),
                        entry: entry.clone(),
                        function: name,
                    });
                }
            }
        }
    }
    if !test_fn_violations.is_empty() {
        test_fn_violations.sort();
        return Err(format!(
            "`test`-marked tests must live in `*_test.dag` files; found a `test` decl in: {}",
            test_fn_violations.join(", ")
        ));
    }
    rows.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    // Inert-lens hygiene backstop (DESIGN.md §6): every `v2.lens.*` module must be a discovered
    // fail-closed witness or be deleted — an inert lens is a lie. This runs over the corpus on
    // every floor discovery and fails closed if any lens is unreached by the discovered roster.
    let inert = inert_lens_modules(&rows, &path_imports, &module_to_path);
    if !inert.is_empty() {
        return Err(format!(
            "inert-lens hygiene (DESIGN.md §6): {} lens module(s) under `v2.lens.*` are authored \
             but unreached by any discovered floor witness — an inert lens is a lie. Wire each \
             with a discovered fail-closed witness (a `*_test.dag` `test fn`/`test data`, or a \
             scan-dir `unified_claim_*`) or delete it: {}",
            inert.len(),
            inert.join(", ")
        ));
    }
    // Construction-justification hygiene (DESIGN.md §5/§6, ROADMAP §0): layered ON TOP of the
    // inert-lens backstop above — every top-level lens module must RECORD its construction
    // justification (which §5 class + why it is residue, not construction). The judgment's
    // correctness is unstructurable human residue; the REQUIREMENT to have recorded one is
    // structurable and fails closed here. (Does NOT supersede the wired-or-deleted check above.)
    let unjustified = unjustified_lens_modules(&module_to_path, &lens_with_justification);
    if !unjustified.is_empty() {
        return Err(format!(
            "construction-justification (DESIGN.md §5/§6): {} lens module(s) under `v2.lens.*` do \
             not record a `construction_justification` — before adding a lens you must justify why \
             the bad-state class cannot be made unwritable by construction. Add a `data \
             construction_justification: ConstructionJustification = …` decl (see \
             v2.lens.common.construction_justification) classifying it as WallNow / \
             WallAfterGrounding / RatchetForever with a rationale: {}",
            unjustified.len(),
            unjustified.join(", ")
        ));
    }
    Ok(rows)
}

/// `true` iff `content` declares the fixed-name `construction_justification` carrier typed as
/// `ConstructionJustification` (the authoring-time construction-justification rule, DESIGN.md
/// §5/§6). A text scan over the file — same style as the floor-discovery hygiene gates — so it
/// runs during the single corpus walk with no resolve cost. The judgment in the decl is the
/// unstructurable residue; only its PRESENCE is checked here.
fn declares_construction_justification(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("data construction_justification")
            && trimmed.contains("ConstructionJustification")
    })
}

/// Top-level lens modules (`v2.lens.<name>`) authored under the source roots that do NOT record a
/// `construction_justification` (DESIGN.md §5/§6). Sorted, deduped. An empty result means every
/// authored lens carries its authoring-time justification. Reuses `is_top_level_lens_module` (the
/// same single authority for "what is a lens" the inert-lens backstop uses) so the two checks
/// cover exactly the same set — this one layers on top, it does not redefine the set.
fn unjustified_lens_modules(
    module_to_path: &std::collections::HashMap<String, String>,
    justified: &std::collections::BTreeSet<String>,
) -> Vec<String> {
    let mut missing: Vec<String> = module_to_path
        .keys()
        .filter(|m| is_top_level_lens_module(m) && !justified.contains(*m))
        .cloned()
        .collect();
    missing.sort();
    missing.dedup();
    missing
}

/// Repo-relative, forward-slash form of a walked `.dag` path (strips the absolute workspace
/// prefix and any leading `./`), so the import-closure graph and authored `unified_claim_*`
/// `witness_entry` strings key identically regardless of how `source_roots` were spelled.
fn repo_relative_dag_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let ws = workspace_root();
    let ws_prefix = format!("{}/", ws.to_string_lossy().replace('\\', "/"));
    let stripped = normalized
        .strip_prefix(&ws_prefix)
        .map(|s| s.to_string())
        .unwrap_or(normalized);
    stripped.trim_start_matches("./").to_string()
}

/// `true` iff `module` is a top-level lens module `v2.lens.<name>` (exactly three segments —
/// support/witness sub-modules like `v2.lens.extdeps_shape_transport_policy.module_refs` are
/// excluded; only the lens itself is held to the wired-or-deleted contract).
fn is_top_level_lens_module(module: &str) -> bool {
    match module.strip_prefix("v2.lens.") {
        Some(rest) => !rest.is_empty() && !rest.contains('.'),
        None => false,
    }
}

/// Lens modules (`v2.lens.<name>`) authored under the source roots but NOT reachable from any
/// discovered witness via transitive imports — the inert set the backstop fails closed on.
/// Sorted, deduped. An empty result means every authored lens is wired.
fn inert_lens_modules(
    rows: &[DiscoveryRow],
    path_imports: &std::collections::HashMap<String, Vec<String>>,
    module_to_path: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut reached: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();
    let path_to_module: std::collections::HashMap<&String, &String> =
        module_to_path.iter().map(|(m, p)| (p, m)).collect();
    // Seed: every discovered witness entry file — its own module plus its direct imports.
    let entry_paths: std::collections::BTreeSet<String> = rows
        .iter()
        .map(|r| repo_relative_dag_path(&r.entry))
        .collect();
    for ep in &entry_paths {
        if let Some(module) = path_to_module.get(ep) {
            if reached.insert((*module).clone()) {
                queue.push((*module).clone());
            }
        }
        if let Some(imports) = path_imports.get(ep) {
            for imp in imports {
                if reached.insert(imp.clone()) {
                    queue.push(imp.clone());
                }
            }
        }
    }
    // Transitive closure over the module import graph.
    while let Some(module) = queue.pop() {
        if let Some(mpath) = module_to_path.get(&module) {
            if let Some(imports) = path_imports.get(mpath) {
                for imp in imports {
                    if reached.insert(imp.clone()) {
                        queue.push(imp.clone());
                    }
                }
            }
        }
    }
    let mut inert: Vec<String> = module_to_path
        .keys()
        .filter(|m| is_top_level_lens_module(m) && !reached.contains(*m))
        .cloned()
        .collect();
    inert.sort();
    inert.dedup();
    inert
}

/// Phase 1.5a REDO v2 — stateless node-frontier floor skip (mirrors RunnableDiscoveryBatch).
pub struct DiscoveryCorpusOptions {
    pub skip_unaffected_node_frontier: bool,
    /// When true, skip marker/`test fn` roster discovery and run only `explicit_entries`.
    /// Host tests use this to exercise skip transport without walking the full floor corpus.
    pub explicit_roster_only: bool,
}

impl Default for DiscoveryCorpusOptions {
    fn default() -> Self {
        Self {
            skip_unaffected_node_frontier: false,
            explicit_roster_only: false,
        }
    }
}

/// Default diff policy — witness-gated to `floor_skip_git_diff_policy()` /
/// `gunbc.ci_spec.diff_policy` (`floor_test_diff_policy_matches_ci_spec_holds`).
const FLOOR_CI_DIFF_POLICY_BASE: &str = "origin/main";
const FLOOR_CI_DIFF_POLICY_HEAD: &str = "HEAD";

enum FloorGitDiffOutcome {
    ObservationFailClosed { reason: String },
    UnifiedProduced(String),
}

/// New-side inclusive line range from one unified-diff hunk (`git diff -U0`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileLineRange {
    start: i64,
    end: i64,
}

fn floor_git_diff_range() -> Result<String, String> {
    // Injection seam: the unified diff text is the real input to the affected-set; git is
    // one transport for obtaining it. A caller (host test / CI step) may supply the text
    // directly via GUNBC_CI_DIFF_UNIFIED; production leaves it unset and shells to git.
    if let Ok(injected) = std::env::var("GUNBC_CI_DIFF_UNIFIED") {
        return Ok(injected);
    }
    let base = std::env::var("GUNBC_CI_DIFF_BASE")
        .unwrap_or_else(|_| FLOOR_CI_DIFF_POLICY_BASE.to_string());
    let head = std::env::var("GUNBC_CI_DIFF_HEAD")
        .unwrap_or_else(|_| FLOOR_CI_DIFF_POLICY_HEAD.to_string());
    let merge_base = std::env::var("GUNBC_CI_DIFF_MERGE_BASE")
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true);
    let range = if merge_base {
        format!("{base}...{head}")
    } else {
        format!("{base} {head}")
    };
    let output = Command::new("git")
        .args(["diff", "-U0", &range])
        .output()
        .map_err(|e| format!("git diff spawn failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git diff -U0 {} failed (status {})",
            range, output.status
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn normalize_repo_path(path: &str) -> String {
    path.strip_prefix("./").unwrap_or(path).replace('\\', "/")
}

/// Match a diff-side repo-relative path to a witness entry path (absolute or relative).
fn diff_file_matches_entry(diff_file: &str, entry_path: &str) -> bool {
    let file = normalize_repo_path(diff_file);
    let entry = normalize_repo_path(entry_path);
    file == entry || entry.ends_with(&file) || file.ends_with(&entry)
}

/// Parse unified diff output into new-side line ranges per repo-relative file path.
fn parse_unified_diff_line_ranges(diff_text: &str) -> HashMap<String, Vec<FileLineRange>> {
    let mut out: HashMap<String, Vec<FileLineRange>> = HashMap::new();
    let mut current_file: Option<String> = None;
    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            current_file = Some(normalize_repo_path(rest));
        } else if line.starts_with("@@ ") {
            let Some(file) = current_file.clone() else {
                continue;
            };
            let plus = line.split_whitespace().nth(2).unwrap_or("");
            let plus = plus.trim_start_matches('+');
            let (start, count) = if let Some((s, c)) = plus.split_once(',') {
                (s.parse::<i64>().unwrap_or(1), c.parse::<i64>().unwrap_or(1))
            } else {
                (plus.parse::<i64>().unwrap_or(1), 1)
            };
            let end = if count <= 0 { start } else { start + count - 1 };
            out.entry(file)
                .or_default()
                .push(FileLineRange { start, end });
        }
    }
    out
}

fn newline_index_for_span<'a>(
    span: &SourceSpan,
    source_indices: &'a HashMap<String, Rc<NewlineIndex>>,
) -> Option<&'a Rc<NewlineIndex>> {
    let file = normalize_repo_path(&span.file);
    source_indices.get(&span.file).or_else(|| {
        source_indices.iter().find_map(|(path, idx)| {
            let norm = normalize_repo_path(path);
            if norm == file || file.ends_with(&norm) || norm.ends_with(&file) {
                Some(idx)
            } else {
                None
            }
        })
    })
}

/// Fuzzy repo-path equality (abs vs repo-relative): one path ends with the other after
/// normalization. Mirrors the lookup in `newline_index_for_span`.
fn span_file_matches(span_file: &str, target_norm: &str) -> bool {
    let s = normalize_repo_path(span_file);
    s == target_norm || s.ends_with(target_norm) || target_norm.ends_with(&s)
}

fn value_is_test_claim(val: &v1_interpreter::Value, ctx: &v1_interpreter::InterpContext) -> bool {
    match val {
        v1_interpreter::Value::Variant { variant_name, .. } => matches!(
            ctx.resolve(*variant_name).as_str(),
            "EqualsClaim"
                | "CompilesClaim"
                | "DiagnosticClaim"
                | "StructuralEqualsClaim"
                | "RoundTripClaim"
                | "BoolWitnessClaim"
        ),
        _ => false,
    }
}

/// Is `val` a substrate `Node` record value?
fn value_is_node(val: &v1_interpreter::Value, ctx: &v1_interpreter::InterpContext) -> bool {
    matches!(
        val,
        v1_interpreter::Value::Record { type_name, .. } if ctx.resolve(*type_name).as_str() == "Node"
    )
}

/// Collect every `Node`-valued sub-tree reachable from `val` (the evaluated value of a
/// changed data item). A changed data item conservatively contributes every Node it
/// denotes to the rerun frontier — seeding only from changed *TestClaims* is fail-open
/// (editing a plain `Node` datum or a std fn a witness depends on would seed nothing and
/// the witness would skip silently). The witness-touch check (`==` / subtree-by-identity)
/// then decides which witnesses actually depend on a changed node. Over-collect within a
/// changed item, never under-collect (fail-closed).
fn collect_node_values(
    val: &v1_interpreter::Value,
    ctx: &v1_interpreter::InterpContext,
    out: &mut Vec<v1_interpreter::Value>,
) {
    if value_is_node(val, ctx) {
        out.push(val.clone());
    }
    match val {
        v1_interpreter::Value::Record { fields, .. }
        | v1_interpreter::Value::Variant { fields, .. } => {
            for v in fields.values() {
                collect_node_values(v, ctx, out);
            }
        }
        v1_interpreter::Value::List(items) => {
            for v in items.iter() {
                collect_node_values(v, ctx, out);
            }
        }
        _ => {}
    }
}

fn call_test_claim_fn_bool(
    ctx: &v1_interpreter::InterpContext,
    fn_name: &str,
    claim: &v1_interpreter::Value,
    frontier: &v1_interpreter::Value,
    claim_param: &str,
) -> Result<Option<bool>, String> {
    if !ctx.item_registry.contains_key(fn_name) {
        return Ok(None);
    }
    let args = [
        (Some(claim_param.to_string()), claim.clone()),
        (Some("frontier".to_string()), frontier.clone()),
    ];
    match v1_interpreter::run_in_context_with_args(ctx, fn_name, &args, false) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(Some(b)),
        Ok(other) => Err(format!(
            "{} returned `{}`, expected Bool",
            fn_name,
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("{}: {}", fn_name, e)),
    }
}

fn list_value_from_vec(items: Vec<v1_interpreter::Value>) -> v1_interpreter::Value {
    v1_interpreter::list_value(items)
}

/// Node-precise skip seeds from git unified-diff line ranges. String-keyed (no `Value`)
/// so the struct is `Send` and a clone can ride into each width-shard thread.
#[derive(Clone, Default)]
struct NodeFrontierSeeds {
    /// `(repo-relative file, data item name)` for every `data` declaration whose span
    /// overlaps the diff (re-evaluated per entry into substrate `Node`s).
    overlapping_data_items: HashSet<(String, String)>,
    /// `(repo-relative file, witness fn name)` for every `test fn` whose own body span was
    /// edited — that witness's behavior IS the diff, so it must run (finding 1, fail-closed).
    edited_test_fns: HashSet<(String, String)>,
    /// Fail-closed escape hatch: a change the node-frontier cannot bound — a non-`.dag`
    /// file (e.g. the Rust seed), a `.dag` that fails to resolve (finding 2), an
    /// unparsed/empty `.dag`, a module/import (preamble) edit, or a `fn`/`type`/`service`
    /// declaration edit — disables the skip so the full corpus runs.
    force_run_all: bool,
}

impl NodeFrontierSeeds {
    fn run_all() -> Self {
        Self {
            force_run_all: true,
            ..Default::default()
        }
    }
}

/// Node-precise frontier seeds: classify each changed line of each changed `.dag` file by
/// the top-level declaration whose source span owns it. A `data` decl joins the rerun
/// frontier; a `test fn` decl force-runs that witness; anything the frontier cannot bound
/// (see `force_run_all`) fails closed to a full run. Pure over the AST — the per-entry
/// touch check (`entry_touches_frontier_seeds`) does the substrate evaluation.
fn collect_frontier_seeds_from_diff_line_ranges(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
) -> Result<NodeFrontierSeeds, String> {
    let mut overlapping_data_items = HashSet::new();
    let mut edited_test_fns = HashSet::new();
    for (file_path, ranges) in line_ranges_by_file {
        if !file_path.ends_with(".dag") {
            // A non-`.dag` change (the Rust seed / interpreter) can flip any witness's
            // result; the node-frontier only models the `.dag` substrate → fail closed.
            return Ok(NodeFrontierSeeds::run_all());
        }
        let file_norm = normalize_repo_path(file_path);
        let (graph, source_indices) = match resolve_entry_with_index(index, file_path) {
            // Finding 2: a changed `.dag` we cannot resolve contributes nothing to the
            // frontier; witnesses depending on it must not be assumed green → fail closed.
            Ok(pair) => pair,
            Err(_) => return Ok(NodeFrontierSeeds::run_all()),
        };
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Ok(NodeFrontierSeeds::run_all()),
        };
        // `test fn` is stripped of its `test` marker before parse, so the AST cannot tell a
        // witness from a plain `fn`; match names against the same text scan discovery uses.
        let test_fn_names: HashSet<String> = scan_test_decl_names(&content).into_iter().collect();
        // Unified, source-ordered line partition of every top-level declaration. `data`/`fn`/
        // `type`/`service` come from the AST (`module.items`) at their decl line; a `test fn`
        // is NOT an AST item, so its decl line comes from the same text scan discovery uses.
        // Both kinds are needed as boundaries: without the scanned `test fn` lines every
        // witness body would fall into the *last* `data` item's trailing span, so a
        // witness-body edit would be misattributed to that node and the witness could skip on
        // a change to its own behavior — a fail-open hole (finding 1). Each declaration owns
        // [its line .. next decl's line − 1] (EOF for the last), so every changed line maps to
        // exactly one declaration. (line, name, is_data)
        let mut decls: Vec<(i64, String, bool)> = Vec::new();
        for module in graph.modules.iter() {
            for item in module.items.iter() {
                if !span_file_matches(&item.span.file, &file_norm) {
                    continue;
                }
                let Some(nl) = newline_index_for_span(&item.span, &source_indices).cloned() else {
                    return Ok(NodeFrontierSeeds::run_all());
                };
                let line = byte_to_line_col(nl, item.span.start).line;
                let name = authored_name_at(source_indices.clone(), item.clone());
                let is_data = item_kind(item.clone()) == ItemKind::DataItem;
                decls.push((line, name, is_data));
            }
        }
        for (name, line) in scan_test_decl_lines(&content) {
            if !decls.iter().any(|(_, n, _)| n == &name) {
                decls.push((line, name, false));
            }
        }
        if decls.is_empty() {
            return Ok(NodeFrontierSeeds::run_all());
        }
        decls.sort_by_key(|(line, _, _)| *line);
        // A change in the preamble (module decl / imports, before the first declaration)
        // can re-resolve every symbol in the file → unbounded effect → fail closed.
        if ranges.iter().any(|r| r.start < decls[0].0) {
            return Ok(NodeFrontierSeeds::run_all());
        }
        for i in 0..decls.len() {
            let (line, name, is_data) = &decls[i];
            let decl_end = decls.get(i + 1).map(|(l, _, _)| l - 1).unwrap_or(i64::MAX);
            if !ranges.iter().any(|r| *line <= r.end && decl_end >= r.start) {
                continue;
            }
            if test_fn_names.contains(name) {
                edited_test_fns.insert((file_norm.clone(), name.clone()));
            } else if *is_data {
                overlapping_data_items.insert((file_norm.clone(), name.clone()));
            } else {
                // A changed `fn`/`type`/`service` declaration can affect any dependent
                // witness transitively; the node-frontier can't bound that → fail closed.
                return Ok(NodeFrontierSeeds::run_all());
            }
        }
    }
    Ok(NodeFrontierSeeds {
        overlapping_data_items,
        edited_test_fns,
        force_run_all: false,
    })
}

/// Re-evaluate changed data items in the *entry* context so frontier `Node` values share
/// the same `InterpContext` as the witness claims (`==` / subtree checks are ctx-sensitive).
fn entry_frontier_nodes_from_seeds(
    ctx: &v1_interpreter::InterpContext,
    entry_path: &str,
    seeds: &NodeFrontierSeeds,
) -> Result<Vec<v1_interpreter::Value>, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for (_file, name) in &seeds.overlapping_data_items {
        // A witness depends on a changed node whether it lives in the entry's OWN file or an
        // imported one, so the gate is "does this entry's closure declare `name`?" — not
        // "is the changed file the entry file?". The old file-equals-entry filter was a §5
        // fail-open hole: an edit to a shared imported `.dag` (e.g. a std node) was invisible
        // to every importing witness, which then skipped on a change it actually depends on.
        if !ctx.item_registry.contains_key(name) {
            continue;
        }
        let Some(val) = v1_interpreter::with_active_context(ctx, || {
            v1_interpreter::eval_data_item_value(ctx, name)
        })
        .map_err(|e| format!("re-eval `{name}` in {entry_path}: {e}"))?
        else {
            continue;
        };
        let mut item_nodes = Vec::new();
        collect_node_values(&val, ctx, &mut item_nodes);
        for node in item_nodes {
            let key = ctx.format_value(&node);
            if seen.insert(key) {
                out.push(node);
            }
        }
    }
    Ok(out)
}

/// Does any witness in `entry_path` depend on a changed `data` node? Re-evaluates the
/// overlapping data items in *this* entry's context (a claim edited in its own file lands
/// here too: its `lhs`/`rhs` `Node`s join the frontier and the claim touches them). An
/// edited `test fn` body is handled separately at the row level (`edited_test_fns`).
fn entry_touches_frontier_seeds(
    ctx: &v1_interpreter::InterpContext,
    entry_path: &str,
    seeds: &NodeFrontierSeeds,
) -> Result<bool, String> {
    let entry_frontier = entry_frontier_nodes_from_seeds(ctx, entry_path, seeds)?;
    if entry_frontier.is_empty() {
        return Ok(false);
    }
    entry_claims_touch_frontier(ctx, &list_value_from_vec(entry_frontier))
}

fn entry_claims_touch_frontier(
    ctx: &v1_interpreter::InterpContext,
    frontier: &v1_interpreter::Value,
) -> Result<bool, String> {
    let mut saw_claim = false;
    // Same active-context guard as the frontier-seed path: the data initializers
    // include FreeMonoid Variant chains (`test_claim_evaluation_nodes`) that only
    // decode while ACTIVE_CTX is set (root fix, #5295).
    let initializer_values = v1_interpreter::with_active_context(ctx, || {
        v1_interpreter::eval_data_initializer_values(ctx)
    })
    .map_err(|e| format!("{e}"))?;
    for val in initializer_values {
        if !value_is_test_claim(&val, ctx) {
            continue;
        }
        saw_claim = true;
        match call_test_claim_fn_bool(
            ctx,
            "test_claim_evaluation_touches_rerun_frontier",
            &val,
            frontier,
            "c",
        ) {
            Ok(Some(true)) => return Ok(true),
            Ok(Some(false)) | Ok(None) => {}
            Err(msg) => {
                eprintln!(
                    "claim_executor: test_claim_evaluation_touches_rerun_frontier failed ({msg}) — fail-closed, running entry witnesses"
                );
                return Ok(true);
            }
        }
        match call_test_claim_fn_bool(
            ctx,
            "floor_claim_touches_rerun_frontier",
            &val,
            frontier,
            "claim",
        ) {
            Ok(Some(true)) => return Ok(true),
            Ok(Some(false)) | Ok(None) => {}
            Err(msg) => {
                eprintln!(
                    "claim_executor: floor_claim_touches_rerun_frontier failed ({msg}) — fail-closed, running entry witnesses"
                );
                return Ok(true);
            }
        }
    }
    // Fail-closed: no TestClaim surface in this entry → always run the witness.
    Ok(!saw_claim)
}

/// Run the whole floor discovery corpus through one shared module index
/// (resolve-once-per-entry), returning a pass/fail summary. `explicit_entries`
/// are appended to the discovered roster with `(entry, function)` dedup — exactly
/// the `claim_batch --roster-from-discovery` + `--entry`/`--function` shape, so
/// one `DiscoveryBatch` node equals the whole `claim_batch` floor step.
/// Fail-closed: an empty roster is an error (a zero-witness corpus is never a
/// successful run). This is the `DiscoveryBatch` scheduler-node handler;
/// `claim_batch` keeps its own timing/stats loop but shares the roster.
pub fn run_discovery_corpus(
    source_roots: &[String],
    scan_dirs: &[String],
    explicit_entries: &[(String, String)],
    execution_mode: v1_interpreter::ExecutionMode,
    parallel_width: usize,
) -> Result<DiscoverySummary, String> {
    run_discovery_corpus_with_options(
        source_roots,
        scan_dirs,
        explicit_entries,
        execution_mode,
        parallel_width,
        DiscoveryCorpusOptions::default(),
    )
}

pub fn run_discovery_corpus_with_options(
    source_roots: &[String],
    scan_dirs: &[String],
    explicit_entries: &[(String, String)],
    execution_mode: v1_interpreter::ExecutionMode,
    parallel_width: usize,
    options: DiscoveryCorpusOptions,
) -> Result<DiscoverySummary, String> {
    check_floor_filename_hygiene(source_roots)?;
    // Roster-only (skip the whole-tree walk): either a caller that pins a
    // representative witness set via `explicit_roster_only` (host skip tests), or the
    // wet/hermetic equivalence gate (empty scan_dirs + explicit entries).
    let mut rows =
        if options.explicit_roster_only || (scan_dirs.is_empty() && !explicit_entries.is_empty()) {
            Vec::new()
        } else {
            discover_floor_corpus_rows(source_roots, scan_dirs)?
        };
    // Append explicit rows with (entry, function) dedup, then re-sort so each
    // entry's witnesses stay grouped for the resolve-once loop below.
    let mut seen: std::collections::BTreeSet<(String, String)> = rows
        .iter()
        .map(|r| (r.entry.clone(), r.function.clone()))
        .collect();
    for (entry, function) in explicit_entries {
        if seen.insert((entry.clone(), function.clone())) {
            rows.push(DiscoveryRow {
                label: function.clone(),
                entry: entry.clone(),
                function: function.clone(),
            });
        }
    }
    rows.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    if rows.is_empty() {
        return Err("discovery roster produced no rows (empty corpus → fail closed)".to_string());
    }
    let whole_tree_published_keys = match precompute_whole_tree_published_mock_keys(source_roots) {
        Ok(keys) if keys.is_empty() => None,
        Ok(keys) => Some(keys),
        Err(e) => {
            return Err(format!(
                "whole-tree published mock corpus precompute failed: {e}"
            ));
        }
    };
    let index = build_multi_entry_index(source_roots);

    let diff_outcome = if options.skip_unaffected_node_frontier {
        match floor_git_diff_range() {
            Ok(text) => FloorGitDiffOutcome::UnifiedProduced(text),
            Err(msg) => {
                eprintln!(
                    "claim_executor: git diff unavailable ({msg}) — fail-closed, running full corpus"
                );
                FloorGitDiffOutcome::ObservationFailClosed { reason: msg }
            }
        }
    } else {
        FloorGitDiffOutcome::UnifiedProduced(String::new())
    };
    let line_ranges_by_file = match diff_outcome {
        FloorGitDiffOutcome::ObservationFailClosed { .. } => HashMap::new(),
        FloorGitDiffOutcome::UnifiedProduced(text) => parse_unified_diff_line_ranges(&text),
    };
    let (skip_enabled, frontier_seeds) = if options.skip_unaffected_node_frontier
        && !line_ranges_by_file.is_empty()
    {
        // The frontier analysis resolves only the changed `.dag` files, to locate which of
        // their declarations moved. `resolve_entry_with_index` warms the index's
        // interior-mutable caches (parse / intern / typed-module), so resolving over the
        // corpus's own `index` would let this read-only analysis pre-seed — and, under the
        // known v2 generic-instantiation bootstrap limitation, corrupt — a module the corpus
        // later reuses: a workflow entry resolved first caches `FreeMonoid<T>` without its
        // variants, and every later witness then reports `Empty`/`Cons` "not found". Resolve
        // over a throwaway index so the analysis cannot perturb the corpus it decides over
        // (the width>1 shards already build a fresh index per thread; this gives the width==1
        // path the same isolation). Cache impurity oracle: cold-vs-warm resolve must agree.
        let frontier_index = build_multi_entry_index(source_roots);
        match collect_frontier_seeds_from_diff_line_ranges(&frontier_index, &line_ranges_by_file) {
            // `force_run_all` (a change the frontier can't bound) disables the skip.
            Ok(seeds) => (!seeds.force_run_all, seeds),
            Err(msg) => {
                eprintln!(
                    "claim_executor: node-frontier population failed ({msg}) — fail-closed, running full corpus"
                );
                (false, NodeFrontierSeeds::default())
            }
        }
    } else {
        (false, NodeFrontierSeeds::default())
    };

    // Width realization (§3 peripheral): shard whole entry-groups across worker threads;
    // the topology (which witnesses run vs. skip) is unchanged — only host fan-out differs.
    // Seeds are string-keyed (Send), so each shard shares a clone of the skip decision.
    let width = parallel_width.max(1);
    if width == 1 {
        return run_discovery_rows(
            &rows,
            &index,
            execution_mode,
            skip_enabled,
            &frontier_seeds,
            whole_tree_published_keys.clone(),
        );
    }
    let shards = shard_row_indices_by_entry(&rows, width);
    eprintln!(
        "run_discovery_corpus: parallel_width={} ({} entry-group shard(s))",
        width,
        shards.iter().filter(|s| !s.is_empty()).count()
    );
    let source_roots_owned = source_roots.to_vec();
    let mut handles = Vec::new();
    for shard in shards {
        if shard.is_empty() {
            continue;
        }
        let shard_rows: Vec<DiscoveryRow> = shard.iter().map(|&i| rows[i].clone()).collect();
        let roots = source_roots_owned.clone();
        let seeds = frontier_seeds.clone();
        let keys = whole_tree_published_keys.clone();
        handles.push(std::thread::spawn(move || {
            let index = build_multi_entry_index(&roots);
            run_discovery_rows(
                &shard_rows,
                &index,
                execution_mode,
                skip_enabled,
                &seeds,
                keys,
            )
        }));
    }
    let mut summaries = Vec::new();
    for handle in handles {
        summaries.push(
            handle
                .join()
                .map_err(|_| "discovery corpus shard thread panicked".to_string())??,
        );
    }
    Ok(merge_discovery_summaries(summaries))
}

/// Partition row indices by whole entry groups (preserve resolve-once warmth), then
/// round-robin groups across `parallel_width` shards.
fn shard_row_indices_by_entry(rows: &[DiscoveryRow], parallel_width: usize) -> Vec<Vec<usize>> {
    let width = parallel_width.max(1);
    let mut entry_groups: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut current_entry: Option<&str> = None;
    for (i, row) in rows.iter().enumerate() {
        if current_entry != Some(row.entry.as_str()) {
            if !current.is_empty() {
                entry_groups.push(current);
            }
            current = vec![i];
            current_entry = Some(&row.entry);
        } else {
            current.push(i);
        }
    }
    if !current.is_empty() {
        entry_groups.push(current);
    }
    let mut shards: Vec<Vec<usize>> = vec![Vec::new(); width];
    for (gi, group) in entry_groups.into_iter().enumerate() {
        shards[gi % width].extend(group);
    }
    shards
}

fn merge_discovery_summaries(summaries: Vec<DiscoverySummary>) -> DiscoverySummary {
    let mut merged = DiscoverySummary {
        total: 0,
        passed: 0,
        skipped: 0,
        failures: Vec::new(),
        witness_outcomes: Vec::new(),
        entry_resolve_receipts: Vec::new(),
        total_resolve_nanos: 0,
        performance_receipts: Vec::new(),
        total_measured_nanos: 0,
    };
    for summary in summaries {
        merged.total += summary.total;
        merged.passed += summary.passed;
        merged.skipped += summary.skipped;
        merged.failures.extend(summary.failures);
        merged.witness_outcomes.extend(summary.witness_outcomes);
        merged
            .entry_resolve_receipts
            .extend(summary.entry_resolve_receipts);
        merged.total_resolve_nanos += summary.total_resolve_nanos;
        merged
            .performance_receipts
            .extend(summary.performance_receipts);
        merged.total_measured_nanos += summary.total_measured_nanos;
    }
    merged
}

/// Run an ordered slice of discovery rows against a shared multi-entry index. Rows must be
/// sorted by `(entry, function)` so resolve-once-per-entry holds. `skip_enabled` +
/// `frontier_seeds` carry the node-precise floor skip decision.
fn run_discovery_rows(
    rows: &[DiscoveryRow],
    index: &MultiEntryIndex,
    execution_mode: v1_interpreter::ExecutionMode,
    skip_enabled: bool,
    frontier_seeds: &NodeFrontierSeeds,
    whole_tree_published_keys: Option<std::collections::HashSet<String>>,
) -> Result<DiscoverySummary, String> {
    let mut summary = DiscoverySummary {
        total: rows.len(),
        passed: 0,
        skipped: 0,
        failures: Vec::new(),
        witness_outcomes: Vec::with_capacity(rows.len()),
        entry_resolve_receipts: Vec::new(),
        total_resolve_nanos: 0,
        performance_receipts: Vec::new(),
        total_measured_nanos: 0,
    };
    let mut current_entry: Option<String> = None;
    let mut current_closure_subject: Option<String> = None;
    let mut ctx: Option<v1_interpreter::InterpContext> = None;
    let mut current_entry_touches = true;
    let whole_tree_published_keys = whole_tree_published_keys.map(Rc::new);
    for row in rows {
        if current_entry.as_deref() != Some(row.entry.as_str()) {
            let sources = load_sources_for_entry_with_index(&index.source_files, &row.entry)
                .map_err(|msg| format!("load sources failed for {}: {}", row.entry, msg))?;
            let closure_subject = subject_digest_for_closure(&sources);
            let resolve_started = std::time::Instant::now();
            let (graph, source_indices) =
                resolve_entry_with_index_for_discovery_corpus(index, &row.entry)
                    .map_err(|msg| format!("resolve failed for {}: {}", row.entry, msg))?;
            let resolve_nanos = resolve_started.elapsed().as_nanos();
            summary.total_resolve_nanos += resolve_nanos;
            summary.entry_resolve_receipts.push(EntryResolveReceipt {
                entry: row.entry.clone(),
                closure_subject: closure_subject.clone(),
                resolve_nanos,
            });
            current_closure_subject = Some(closure_subject);
            let entry_ctx = make_eval_context_with_runtime_options(
                &graph,
                source_indices,
                execution_mode,
                None,
                whole_tree_published_keys.clone(),
            );
            current_entry_touches = if skip_enabled {
                entry_touches_frontier_seeds(&entry_ctx, &row.entry, frontier_seeds)?
            } else {
                true
            };
            ctx = Some(entry_ctx);
            current_entry = Some(row.entry.clone());
        }
        // Finding 1 (fail-closed): a witness whose own `test fn` body span was edited must
        // run even when no `data` node it reads changed — its behavior IS the diff.
        let function_edited = skip_enabled
            && frontier_seeds.edited_test_fns.iter().any(|(file, func)| {
                diff_file_matches_entry(file, &row.entry) && func == &row.function
            });
        if skip_enabled && !current_entry_touches && !function_edited {
            summary.skipped += 1;
            eprintln!(
                "SKIP [assumed-green node-frontier] {} ({})",
                row.function, row.entry
            );
            continue;
        }
        let ctx_ref = ctx.as_ref().expect("ctx set above");
        let closure_subject = current_closure_subject
            .as_deref()
            .expect("closure subject set above");
        let (outcome, receipt) = run_claim_measured(ctx_ref, closure_subject, &row.function);
        summary.total_measured_nanos += receipt.wall_nanos;
        summary.performance_receipts.push(receipt);
        summary.witness_outcomes.push(DiscoveryWitnessOutcome {
            entry: row.entry.clone(),
            function: row.function.clone(),
            outcome: outcome.clone(),
        });
        match outcome {
            ClaimOutcome::Pass => summary.passed += 1,
            ClaimOutcome::Fail => summary.failures.push(format!(
                "{} ({}) returned Bool(false)",
                row.function, row.entry
            )),
            ClaimOutcome::NotBool { got } => summary.failures.push(format!(
                "{} ({}) returned `{}`, not Bool",
                row.function, row.entry, got
            )),
            ClaimOutcome::RuntimeError { message } => summary.failures.push(format!(
                "{} ({}) runtime error: {}",
                row.function, row.entry, message
            )),
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod floor_skip_frontier_tests {
    use super::{
        build_multi_entry_index, collect_frontier_seeds_from_diff_line_ranges,
        entry_touches_frontier_seeds, parse_unified_diff_line_ranges, scan_test_decl_lines,
        FileLineRange,
    };
    use crate::v1_compiler_infer_items::{item_kind, ItemKind, ResolvedGraph};
    use crate::v1_interpreter::ExecutionMode;
    use crate::v1_std_core::{authored_name_at, byte_to_line_col};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn fixture_path() -> String {
        "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag".to_string()
    }

    fn data_item_line(
        fixture: &str,
        source_indices: &std::rc::Rc<
            HashMap<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>,
        >,
        graph: &std::rc::Rc<ResolvedGraph>,
        name: &str,
    ) -> i64 {
        for module in graph.modules.iter() {
            for item in module.items.iter() {
                if item_kind(item.clone()) != ItemKind::DataItem {
                    continue;
                }
                if authored_name_at(source_indices.clone(), item.clone()) != name {
                    continue;
                }
                let span = &item.span;
                let index = source_indices.get(&span.file).expect("newline index");
                return byte_to_line_col(index.clone(), span.start).line;
            }
        }
        panic!("data item `{name}` not found in {fixture}");
    }

    fn unified_diff_for_line(file: &str, line: i64) -> String {
        format!(
            "diff --git a/{file} b/{file}\n--- a/{file}\n+++ b/{file}\n@@ -{line},0 +{line},1 @@\n+// node-precise touch\n"
        )
    }

    #[test]
    fn parse_unified_diff_extracts_new_side_line_ranges() {
        let diff = "\
diff --git a/src/v2/lens/affected_set.dag b/src/v2/lens/affected_set.dag
--- a/src/v2/lens/affected_set.dag
+++ b/src/v2/lens/affected_set.dag
@@ -100,0 +101,3 @@
+line1
+line2
+line3
";
        let ranges = parse_unified_diff_line_ranges(diff);
        let file = "src/v2/lens/affected_set.dag";
        assert_eq!(
            ranges.get(file),
            Some(&vec![FileLineRange {
                start: 101,
                end: 103
            }])
        );
    }

    /// `test fn`s are not AST items, so the partition gets their decl lines from source text.
    /// Without these boundaries a witness-body edit would fall into the last `data` item's
    /// trailing span (the finding-1 fail-open). Pin the 1-based line pairing.
    #[test]
    fn scan_test_decl_lines_pairs_names_with_1_based_lines() {
        let source = "module m\n\ndata d: Int = 1\n\ntest fn witness_a() -> Bool { true }\n\ntest data witness_b: Int = 2\n";
        let pairs = scan_test_decl_lines(source);
        assert_eq!(
            pairs,
            vec![("witness_a".to_string(), 5), ("witness_b".to_string(), 7)]
        );
    }

    /// §5 discriminating receipt: same FILE, two different nodes. A node a claim references
    /// → the entry touches the frontier (runs). An ORPHAN node referenced by no claim → the
    /// entry does NOT touch (skips). Same file, different nodes, different outcomes: only
    /// true node precision (not file-level) can produce the orphan skip.
    #[test]
    fn node_precise_same_file_referenced_vs_orphan_discriminates() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let fixture = fixture_path();
        let roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dsl").to_string_lossy().into_owned(),
        ];
        let index = build_multi_entry_index(&roots);
        let (graph, source_indices) = super::resolve_entry_with_index(&index, &fixture)
            .expect("discriminator fixture resolves");
        let referenced_line =
            data_item_line(&fixture, &source_indices, &graph, "floor_disc_node_c");
        let orphan_line =
            data_item_line(&fixture, &source_indices, &graph, "floor_disc_orphan_node");
        assert_ne!(
            referenced_line, orphan_line,
            "fixture must place the two nodes on distinct lines"
        );

        let ctx = super::make_eval_context(&graph, source_indices.clone(), ExecutionMode::Wet);

        let referenced_ranges =
            parse_unified_diff_line_ranges(&unified_diff_for_line(&fixture, referenced_line));
        let referenced_seeds =
            collect_frontier_seeds_from_diff_line_ranges(&index, &referenced_ranges)
                .expect("frontier for referenced-node diff");
        assert!(
            entry_touches_frontier_seeds(&ctx, &fixture, &referenced_seeds)
                .expect("touch check (referenced)"),
            "a diff on a node some claim references must touch the entry (runs)"
        );

        let orphan_ranges =
            parse_unified_diff_line_ranges(&unified_diff_for_line(&fixture, orphan_line));
        let orphan_seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &orphan_ranges)
            .expect("frontier for orphan-node diff");
        assert!(
            !entry_touches_frontier_seeds(&ctx, &fixture, &orphan_seeds)
                .expect("touch check (orphan)"),
            "a diff on an orphan node (no claim references it) must NOT touch the entry (skips)"
        );
    }
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
    /// Grounded source-tree provenance: the `SourceRootRef` variant token (`V2Tree` / `DslTree`)
    /// for the `--source-root` this file was actually read from. Filesystem truth at ingest —
    /// keep in lockstep with `v2.std.cross_tree.import_model.SourceRootRef`.
    pub source_root: String,
}

/// Map a `--source-root` path to its `SourceRootRef` variant token. Fail-closed on an
/// unrecognized root. Authority: `gunbc.ci_layer_roots.witness_layer_roots = [src/v2, dsl]`
/// realized as the closed coproduct `SourceRootRef = V2Tree | DslTree`.
fn source_root_ref_variant_for_root(root: &str) -> Result<String, String> {
    match root.trim_end_matches('/') {
        "src/v2" => Ok("V2Tree".to_string()),
        "dsl" => Ok("DslTree".to_string()),
        other => Err(format!(
            "source_root tagging: unknown --source-root '{other}' \
             (authority gunbc.ci_layer_roots.witness_layer_roots = [src/v2, dsl] -> \
             SourceRootRef {{V2Tree, DslTree}})"
        )),
    }
}

/// Attribute a host-read file to the `--source-root` it came from (grounded in the actual
/// filesystem path, NOT the module's self-declared QN prefix), then map to its
/// `SourceRootRef` variant. Fail-closed: a path matching zero or 2+ roots is an error.
fn source_root_ref_token_for_path(
    file_path: &str,
    source_roots: &[String],
) -> Result<String, String> {
    let matched: Vec<&String> = source_roots
        .iter()
        .filter(|r| {
            let r = r.trim_end_matches('/');
            file_path == r || file_path.starts_with(&format!("{r}/"))
        })
        .collect();
    match matched.as_slice() {
        [] => Err(format!(
            "source_root tagging: file '{file_path}' matches no --source-root {source_roots:?}"
        )),
        [one] => source_root_ref_variant_for_root(one),
        _ => Err(format!(
            "source_root tagging: file '{file_path}' matches multiple --source-root {matched:?}"
        )),
    }
}

fn source_root_ingest_symbol_from_stem(stem: &str) -> String {
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
    } else if body.as_bytes()[0].is_ascii_digit() {
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

fn source_root_ingest_compilation_unit_for_path(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("host_sr");
    source_root_ingest_symbol_from_stem(stem)
}

/// Parsed `Admission` facts for the scoped `--entry` module (subject + import targets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRootEntryAdmission {
    pub subject: Vec<String>,
    pub imports: Vec<Vec<String>>,
}

fn parse_dotted_module_path(path: &str) -> Option<Vec<String>> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let segments: Vec<String> = trimmed
        .split('.')
        .filter(|seg| !seg.is_empty())
        .map(str::to_string)
        .collect();
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

/// Parse `module …` and `import …` lines from an entry `.dag` source for manifest admission.
/// Bootstrap host transport only — same lightweight line-scan shape as `extract_module_path`;
/// dissolves when v2 owns ingest-side admission projection.
pub fn parse_source_root_entry_admission(source: &str) -> Result<SourceRootEntryAdmission, String> {
    let mut subject: Option<Vec<String>> = None;
    let mut imports: Vec<Vec<String>> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for line in source.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        if let Some(rest) = line.strip_prefix("module ") {
            subject = parse_dotted_module_path(rest);
        } else if let Some(rest) = line.strip_prefix("import ") {
            let module_path = rest.split_whitespace().next().unwrap_or("");
            if let Some(segments) = parse_dotted_module_path(module_path) {
                if seen.insert(segments.clone()) {
                    imports.push(segments);
                }
            }
        }
    }

    subject
        .map(|subject| SourceRootEntryAdmission { subject, imports })
        .ok_or_else(|| "entry source missing `module` declaration".to_string())
}

fn emit_qualified_name_dag(segments: &[String]) -> String {
    if segments.is_empty() {
        return "QnEmpty".to_string();
    }
    let mut out = String::from("QnEmpty");
    for seg in segments.iter().rev() {
        out = format!("QnCons {{ head: ^{seg}, tail: {out} }}");
    }
    out
}

#[cfg(test)]
mod manifest_emit_tests {
    use super::{
        dag_embedded_dag_source_escape, dag_manifest_scalar_escape, emit_qualified_name_dag,
    };

    #[test]
    fn emit_qualified_name_dag_three_segment_path() {
        assert_eq!(
            emit_qualified_name_dag(&["v2".into(), "compiler".into(), "compile".into()]),
            "QnCons { head: ^v2, tail: QnCons { head: ^compiler, tail: QnCons { head: ^compile, tail: QnEmpty } } }"
        );
    }

    #[test]
    fn emit_qualified_name_dag_empty_is_qn_empty() {
        assert_eq!(emit_qualified_name_dag(&[]), "QnEmpty");
    }

    #[test]
    fn manifest_scalar_escape_rejects_braces() {
        assert!(dag_manifest_scalar_escape("src/v2/foo.dag").is_ok());
        assert!(dag_manifest_scalar_escape("fnv1a64:abc").is_ok());
        assert!(dag_manifest_scalar_escape("has{brace").is_err());
        assert!(dag_manifest_scalar_escape("has}brace").is_err());
    }

    #[test]
    fn embedded_dag_source_escape_preserves_braces_as_escapes() {
        assert_eq!(
            dag_embedded_dag_source_escape("match x { A => 1 }"),
            "match x \\{ A => 1 \\}"
        );
    }

    use super::source_root_ref_token_for_path;

    // Discriminating: grounded source-root tag comes from the FILE'S actual --source-root
    // (filesystem truth), so a `dsl/`-rooted file tags DslTree even when its module decl
    // would QN-guess otherwise, and a `src/v2/` file tags V2Tree. Fail-closed on a path
    // under no root (or 2+).
    #[test]
    fn source_root_token_grounds_in_filesystem_location() {
        let roots = vec!["src/v2".to_string(), "dsl".to_string()];
        assert_eq!(
            source_root_ref_token_for_path("src/v2/std/algebra.dag", &roots).unwrap(),
            "V2Tree"
        );
        assert_eq!(
            source_root_ref_token_for_path("dsl/std/algebra.dag", &roots).unwrap(),
            "DslTree"
        );
        // src/v2 file declaring a non-`v2.`-prefixed module (e.g. extdeps.shell) still tags
        // V2Tree — the FS truth, not the QN guess (this is exactly the case the QN-prefix
        // fallback mis-tagged as DslTree).
        assert_eq!(
            source_root_ref_token_for_path("src/v2/extdeps/shell.dag", &roots).unwrap(),
            "V2Tree"
        );
        // fail-closed: path under no declared root.
        assert!(source_root_ref_token_for_path("src/v1/stage0/x.dag", &roots).is_err());
        // boundary: a sibling dir sharing a prefix must not match (src/v20 ≠ src/v2).
        assert!(source_root_ref_token_for_path("src/v20/x.dag", &roots).is_err());
    }
}

fn emit_import_admission_list(imports: &[Vec<String>]) -> String {
    let mut out = String::from("Empty");
    for import in imports.iter().rev() {
        out = format!(
            "Cons {{\n  head: Import {{\n    target: {},\n    visibility: ImportVisible\n  }},\n  tail: {out}\n}}",
            emit_qualified_name_dag(import)
        );
    }
    out
}

fn emit_source_root_entry_admission_data(admission: &SourceRootEntryAdmission) -> String {
    format!(
        "data host_compiler_closure_admission: Admission = Admission {{\n  subject: ResolutionSubject {{\n    name: {}\n  }},\n  imports: {}\n}}\n\n\n",
        emit_qualified_name_dag(&admission.subject),
        emit_import_admission_list(&admission.imports)
    )
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
        let source_root = source_root_ref_token_for_path(&rel_forward, source_roots)?;
        records.push(SourceRootReadRecord {
            file_path: rel_forward,
            module_path,
            source: content,
            source_root,
        });
    }

    records.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(records)
}

/// Parse-level import closure for one entry `.dag` file — same scope as
/// `--claim-run --entry` / `load_sources_for_entry`, projected to ingest witnesses.
pub fn discover_source_root_reads_for_entry(
    source_roots: &[String],
    entry_path: &str,
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

    let closure = load_sources_for_entry(source_roots, entry_path)
        .map_err(|msg| format!("discover_source_root_ingest: entry closure load failed: {msg}"))?;

    let mut records: Vec<SourceRootReadRecord> = Vec::new();
    for source in closure {
        let rel_forward = source.path.replace('\\', "/");
        if path_matches_any_subpath(&rel_forward, exclude_subpaths) {
            continue;
        }
        let module_path = extract_module_path(&source.content).ok_or_else(|| {
            format!(
                "discover_source_root_ingest: no module declaration in {}",
                rel_forward
            )
        })?;
        let source_root = source_root_ref_token_for_path(&rel_forward, source_roots)?;
        records.push(SourceRootReadRecord {
            file_path: rel_forward,
            module_path,
            source: source.content.clone(),
            source_root,
        });
    }

    records.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(records)
}

fn emit_source_root_read_witness(rec: &SourceRootReadRecord) -> Result<String, String> {
    let artifact_id = source_root_ingest_artifact_id_for_path(&rec.file_path);
    let compilation_unit = source_root_ingest_compilation_unit_for_path(&rec.file_path);
    Ok(format!(
        "DagSourceReadWitness {{\n  source: Medium {{ carried: \"{}\", fidelity: Lossless }},\n  artifact: Artifact {{\n    kind: SourceFile,\n    id: {artifact_id},\n    file_path: \"{}\"\n  }},\n  compilation_unit: {compilation_unit},\n  source_root: {}\n}}",
        dag_embedded_dag_source_escape(&rec.source),
        dag_manifest_scalar_escape(&rec.file_path)?,
        rec.source_root,
    ))
}

fn emit_source_root_ingest_monoid(records: &[SourceRootReadRecord]) -> Result<String, String> {
    let mut witness_nodes: Vec<String> = records
        .iter()
        .map(emit_source_root_read_witness)
        .collect::<Result<_, _>>()?;
    let mut out = String::from("Empty");
    while let Some(head) = witness_nodes.pop() {
        out = format!("Cons {{\n  head: {head},\n  tail: {out}\n}}");
    }
    Ok(out)
}

/// Emit an ephemeral importable `.dag` manifest (never committed).
pub fn emit_source_root_ingest_manifest(
    path: &Path,
    records: &[SourceRootReadRecord],
    entry_admission: Option<&SourceRootEntryAdmission>,
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
    out.push_str("import extdeps.communication.medium { Lossless, Medium }\n");
    out.push_str("import v2.std.algebra { Cons, Empty }\n");
    out.push_str("import v2.std.artifact { Artifact, SourceFile }\n");
    out.push_str("import v2.std.text { String }\n");
    if entry_admission.is_some() {
        out.push_str("import v2.compiler.name_resolve {\n");
        out.push_str("  Admission,\n");
        out.push_str("  Import,\n");
        out.push_str("  ImportVisible,\n");
        out.push_str("  ResolutionSubject\n");
        out.push_str("}\n");
        out.push_str("import v2.std.qualified_name { QnCons, QnEmpty }\n");
    }
    out.push('\n');
    out.push_str(&format!(
        "data host_source_root_ingest_content_hash: String = \"{}\"\n\n\n",
        dag_manifest_scalar_escape(&content_hash)?
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
        out.push_str(&emit_source_root_ingest_monoid(inline_records)?);
        out.push('\n');
    }
    if let Some(admission) = entry_admission {
        out.push('\n');
        out.push_str(&emit_source_root_entry_admission_data(admission));
    }

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

#[cfg(test)]
mod inert_lens_hygiene_tests {
    use super::{
        discover_floor_corpus_rows, inert_lens_modules, is_top_level_lens_module, DiscoveryRow,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn row(entry: &str, function: &str) -> DiscoveryRow {
        DiscoveryRow {
            label: function.to_string(),
            entry: entry.to_string(),
            function: function.to_string(),
        }
    }

    #[test]
    fn top_level_lens_module_predicate() {
        assert!(is_top_level_lens_module("v2.lens.effect"));
        assert!(is_top_level_lens_module(
            "v2.lens.extdeps_shape_transport_policy"
        ));
        // support/witness sub-modules are NOT the lens itself.
        assert!(!is_top_level_lens_module(
            "v2.lens.extdeps_shape_transport_policy.module_refs"
        ));
        assert!(!is_top_level_lens_module(
            "v2.test.lens_effect.effect_depends_on"
        ));
        assert!(!is_top_level_lens_module("v2.std.algebra"));
        assert!(!is_top_level_lens_module("v2.lens."));
    }

    // Discriminating witness for the detector: it goes RED on an unreached lens, GREEN once a
    // discovered witness reaches it (directly or transitively). An always-green check would be
    // the very coverage-by-illusion (DESIGN.md §6) the backstop exists to kill.
    #[test]
    fn detector_red_on_unreached_green_on_wired() {
        let mut module_to_path: HashMap<String, String> = HashMap::new();
        let mut path_imports: HashMap<String, Vec<String>> = HashMap::new();
        module_to_path.insert(
            "v2.lens.demo".to_string(),
            "src/v2/lens/demo.dag".to_string(),
        );
        path_imports.insert("src/v2/lens/demo.dag".to_string(), vec![]);

        // No discovered witness reaches it → inert (RED).
        let inert = inert_lens_modules(&[], &path_imports, &module_to_path);
        assert_eq!(inert, vec!["v2.lens.demo".to_string()]);

        // A discovered witness importing it → wired (GREEN).
        module_to_path.insert(
            "v2.test.lens_demo.w".to_string(),
            "src/v2/workflow/lens_demo_family_eval_test.dag".to_string(),
        );
        path_imports.insert(
            "src/v2/workflow/lens_demo_family_eval_test.dag".to_string(),
            vec!["v2.lens.demo".to_string()],
        );
        let rows = vec![row("src/v2/workflow/lens_demo_family_eval_test.dag", "w")];
        assert!(
            inert_lens_modules(&rows, &path_imports, &module_to_path).is_empty(),
            "wiring a discovered witness must clear the inert flag"
        );

        // Transitive: a sibling lens reached only through `demo` (lens-imports-lens) is wired.
        module_to_path.insert("v2.lens.sib".to_string(), "src/v2/lens/sib.dag".to_string());
        path_imports.insert("src/v2/lens/sib.dag".to_string(), vec![]);
        path_imports.insert(
            "src/v2/lens/demo.dag".to_string(),
            vec!["v2.lens.sib".to_string()],
        );
        assert!(
            inert_lens_modules(&rows, &path_imports, &module_to_path).is_empty(),
            "a transitively-reached sibling lens must count as wired"
        );
    }

    // Whole-corpus enforcement: floor discovery over the real witness roots must succeed, which
    // (per the backstop wired into `discover_floor_corpus_rows`) means zero inert lenses.
    #[test]
    fn floor_corpus_has_no_inert_lenses() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = vec![
            ws.join("dsl").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let scan_dirs = vec![
            "dsl/test/claim".to_string(),
            "src/v2/compiler/manual".to_string(),
        ];
        let result = discover_floor_corpus_rows(&roots, &scan_dirs);
        assert!(
            result.is_ok(),
            "floor discovery must succeed — every v2.lens.* is wired or deleted: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[cfg(test)]
mod construction_justification_hygiene_tests {
    use super::{
        declares_construction_justification, discover_floor_corpus_rows, unjustified_lens_modules,
    };
    use std::collections::{BTreeSet, HashMap};
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    // The scan predicate is the structurable half of the rule: it sees the recorded decl, not its
    // (unstructurable) correctness. RED when the carrier is absent — exactly the strip-and-revert
    // discriminator the brief requires.
    #[test]
    fn justification_scan_predicate() {
        let with = "module v2.lens.demo\n\
            import v2.lens.common.construction_justification { ConstructionJustification, RatchetForever }\n\
            data construction_justification: ConstructionJustification = ConstructionJustification {\n\
              class: RatchetForever { undecidable_because: \"x\" },\n\
              rationale: \"y\"\n\
            }\n";
        assert!(declares_construction_justification(with));

        // A bare name without the type annotation is NOT a recorded justification.
        assert!(!declares_construction_justification(
            "data construction_justification_note: String = \"todo\"\n"
        ));
        // Stripped of the decl entirely → RED.
        assert!(!declares_construction_justification(
            "module v2.lens.demo\ndata other: String = \"z\"\n"
        ));
    }

    // Discriminating witness for the detector: a top-level lens with NO recorded justification is
    // RED; recording one clears it. An always-green check would be the coverage-by-illusion
    // (DESIGN.md §6) this rule layers onto the backstop to prevent.
    #[test]
    fn detector_red_on_missing_green_on_recorded() {
        let mut module_to_path: HashMap<String, String> = HashMap::new();
        module_to_path.insert(
            "v2.lens.demo".to_string(),
            "src/v2/lens/demo.dag".to_string(),
        );
        // Support/sub-modules and non-lens modules are NOT subject to the rule (same set as the
        // inert-lens backstop, via is_top_level_lens_module).
        module_to_path.insert(
            "v2.lens.common.construction_justification".to_string(),
            "src/v2/lens/common/construction_justification.dag".to_string(),
        );
        module_to_path.insert("v2.std.text".to_string(), "src/v2/std/text.dag".to_string());

        let none: BTreeSet<String> = BTreeSet::new();
        assert_eq!(
            unjustified_lens_modules(&module_to_path, &none),
            vec!["v2.lens.demo".to_string()],
            "an unjustified top-level lens must go RED"
        );

        let mut justified: BTreeSet<String> = BTreeSet::new();
        justified.insert("v2.lens.demo".to_string());
        assert!(
            unjustified_lens_modules(&module_to_path, &justified).is_empty(),
            "recording a justification must clear the violation"
        );
    }

    // Whole-corpus enforcement, green-by-execution: floor discovery over the real roots succeeds,
    // which (per the check wired into `discover_floor_corpus_rows`) means every authored
    // `v2.lens.*` module records its construction-justification. Strip one and this goes RED.
    #[test]
    fn floor_corpus_every_lens_is_justified() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = vec![
            ws.join("dsl").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let scan_dirs = vec![
            "dsl/test/claim".to_string(),
            "src/v2/compiler/manual".to_string(),
        ];
        let result = discover_floor_corpus_rows(&roots, &scan_dirs);
        assert!(
            result.is_ok(),
            "floor discovery must succeed — every v2.lens.* records a construction-justification: {}",
            result.err().unwrap_or_default()
        );
    }
}
