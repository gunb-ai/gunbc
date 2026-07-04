use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

use crate::coproduct_reflection::{decl_facts_corpus_walk, DeclFactRaw};
use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::{kernel_type_set, SourceSpan};
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::lookup_type_by_name;
use crate::v1_compiler_infer_env::{
    maybe_print_type_env_lookup_profile, reset_type_env_lookup_profile,
};
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_rt;
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_pattern, authored_name_at, block_stmts, build_newline_index,
    byte_to_line_col, diagnostic_to_message, diagnostic_to_span, empty_intern_table,
    expr_method_name_at, expr_var_name_at, field_access_base, field_access_field_at,
    field_init_node_name_at, field_init_node_value, has_child_named, intern,
    is_discovery_corpus_advisory_typecheck_diagnostic, is_discovery_corpus_blocking_diagnostic,
    is_error_diagnostic, is_interpreter_blocking_diagnostic, let_binding_name_at, let_value,
    match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver, param_node_name_at,
    param_node_type_expr, CompilerDiagnostic, ErrorNode, ExprData, InferredNode, InternTable,
    MatchPattern, NewlineIndex, Node,
};
use serde::Serialize;

#[path = "phase_profile.rs"]
mod phase_profile;
pub use phase_profile::{set_phase, FloorPhase, PhaseProfile};

use crate::resolved_graph_cache::{
    lookup as cross_process_lookup, resolved_graph_cache_root_from_env, subject_digest_for_closure,
    write as cross_process_write, CacheLookupResult,
};

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

pub const UNIFIED_CLAIM_VERIFICATION_MODULE: &str = "v2.std.verification";
pub const BOOL_WITNESS_CLAIM_TYPE: &str = "BoolWitnessClaim";
pub const NODE_CORPUS_TYPE: &str = "NodeCorpus";

// cargo's build-output dir (a `target` dir beside a Cargo.toml) is realization
// output, not source: a corpus copy materialized under it (e.g.
// target/func_env_semantic_baseline_corpus/dag/**) must never enter a module
// index alongside the tree it was copied from. A source root passed FROM
// inside target/ is still walked — only descent into the output dir is refused.
pub(crate) fn is_cargo_target_output_dir(
    parent: &std::path::Path,
    child: &std::path::Path,
) -> bool {
    child.file_name().and_then(|n| n.to_str()) == Some("target")
        && parent.join("Cargo.toml").is_file()
}

fn collect_dag_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read dir {:?}: {}", dir, e))
        .map(|e| e.unwrap_or_else(|e| panic!("failed to read dir entry: {}", e)))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if is_cargo_target_output_dir(dir, &path) {
                continue;
            }
            collect_dag_files(&path, files);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            files.push(path);
        }
    }
}

pub(crate) fn extract_module_path(content: &str) -> Option<String> {
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

/// Module-less `.dag` fragments (parse fixtures) are excluded from the compile entry
/// set. Fail-closed visibility: list every skipped path so a forgotten `module` decl
/// in real source is surfaced, not silently dropped.
pub fn report_moduleless_dag_entry_skips(skipped_paths: &[String]) {
    if skipped_paths.is_empty() {
        return;
    }
    eprintln!(
        "skipped {} module-less .dag file(s) from compile entry set (no `module` declaration):",
        skipped_paths.len()
    );
    for path in skipped_paths {
        eprintln!("  {path}");
    }
}

pub fn moduleless_dag_entry_paths(entry_files: &[(String, String)]) -> Vec<String> {
    entry_files
        .iter()
        .filter(|(_, content)| extract_module_path(content).is_none())
        .map(|(path, _)| path.clone())
        .collect()
}

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

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

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
                if let Some(existing) = index.get(&module_path) {
                    if existing != &rel {
                        panic!(
                            "module-path collision: module '{}' is declared by both '{}' and '{}' — one module, one authority (DESIGN §3); silent last-root-wins shadowing broke the floor (extdeps.shell, 2026-07-01) — de-fork or rename one side",
                            module_path, existing, rel
                        );
                    }
                }
                index.insert(module_path.clone(), rel);
            }
        }
    }
    index
}

const CI_LAYER_ROOTS_AUTHORITY_REL: &str = "dag/gunbc/ci_layer_roots.dag";
const WITNESS_LAYER_ROOTS_DATA_NAME: &str = "witness_layer_roots";
const WITNESS_DISCOVERY_SCAN_DIRS_DATA_NAME: &str = "witness_discovery_scan_dirs";

fn ci_layer_roots_authority_content() -> &'static str {
    static CONTENT: OnceLock<String> = OnceLock::new();
    CONTENT
        .get_or_init(|| {
            let path = workspace_root().join(CI_LAYER_ROOTS_AUTHORITY_REL);
            std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "ci_layer_roots authority: failed to read {}: {e}",
                    path.display()
                )
            })
        })
        .as_str()
}

/// Project a `List<String>` data literal out of a `.dag` module's SOURCE TEXT via the real front-end
/// (`tokenize` + `parse`) — no second hand-rolled scanner. Pure (text in, list out) so a synthetic
/// authority carrying non-default values can drive it: a reader that ignored its input and returned
/// a hardcoded copy fails that control — the by-construction discrimination (DESIGN §5). Fail-closed:
/// a parse error, a missing data def, a non-string-list body, or (when `allow_empty` is false) an
/// empty list is a loud panic, never a silent fallback that would re-open the drift.
pub(crate) fn string_list_data_from_module_source(
    module_rel_path: &str,
    content: &str,
    data_name: &str,
    allow_empty: bool,
) -> Vec<String> {
    use crate::v1_std_core::{ExprData, LiteralValue};

    let filename = module_rel_path.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), filename.clone());
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let result = crate::v1_compiler_parse::parse(tokens, std::rc::Rc::new(source_indices));
    if let Some(err) = result.error.as_ref() {
        panic!(
            "lens table reader: parse error in {module_rel_path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .unwrap_or_else(|| panic!("lens table reader: {module_rel_path} parsed to no module"));
    for item in module.children.iter() {
        if item.name != data_name
            || !crate::v1_compiler_emit_core_support::is_data_def_item(item.clone())
        {
            continue;
        }
        let body = item.body.as_ref().unwrap_or_else(|| {
            panic!("lens table reader: `data {data_name}` in {module_rel_path} has no value body")
        });
        if !matches!(body.expr_data.as_ref(), ExprData::ExprListLit) {
            panic!(
                "lens table reader: `data {data_name}` in {module_rel_path} is not a \
                 `List<String>` literal"
            );
        }
        let mut values = Vec::new();
        for el in body.children.iter() {
            match el.expr_data.as_ref() {
                ExprData::ExprLiteral { value } => match value.as_ref() {
                    LiteralValue::LitStr { value } => values.push(value.clone()),
                    _ => panic!(
                        "lens table reader: an element of `{data_name}` in {module_rel_path} is not \
                         a string literal"
                    ),
                },
                _ => panic!(
                    "lens table reader: an element of `{data_name}` in {module_rel_path} is not a \
                     literal"
                ),
            }
        }
        if values.is_empty() && !allow_empty {
            panic!("lens table reader: `{data_name}` in {module_rel_path} is empty (fail-closed)");
        }
        return values;
    }
    panic!("lens table reader: no `data {data_name}` def in {module_rel_path}")
}

/// Read a `List<String>` data table from a live `.dag` lens authority on disk.
pub fn lens_string_list_data(
    module_rel_path: &str,
    data_name: &str,
    allow_empty: bool,
) -> Vec<String> {
    let path = workspace_root().join(module_rel_path);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("lens table reader: failed to read {}: {e}", path.display()));
    string_list_data_from_module_source(module_rel_path, &content, data_name, allow_empty)
}

/// Project a `List<String>` data literal out of the ci_layer_roots authority's SOURCE TEXT via the
/// real front-end (`tokenize` + `parse`) — no second hand-rolled scanner.
pub(crate) fn string_list_data_from_ci_layer_roots_source(
    content: &str,
    data_name: &str,
) -> Vec<String> {
    string_list_data_from_module_source(CI_LAYER_ROOTS_AUTHORITY_REL, content, data_name, false)
}

/// Project the `witness_layer_roots` `List<String>` literal out of the ci_layer_roots authority.
pub(crate) fn witness_layer_roots_from_source(content: &str) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(content, WITNESS_LAYER_ROOTS_DATA_NAME)
}

/// Project the `witness_discovery_scan_dirs` `List<String>` literal out of the ci_layer_roots
/// authority.
pub(crate) fn witness_discovery_scan_dirs_from_source(content: &str) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(content, WITNESS_DISCOVERY_SCAN_DIRS_DATA_NAME)
}

/// The witness layer roots, read live from the single .dag authority and memoized.
pub(crate) fn witness_layer_roots() -> Vec<String> {
    static ROOTS: OnceLock<Vec<String>> = OnceLock::new();
    ROOTS
        .get_or_init(|| witness_layer_roots_from_source(ci_layer_roots_authority_content()))
        .clone()
}

/// Witness discovery scan dirs, read live from `gunbc.ci_layer_roots.witness_discovery_scan_dirs`.
pub(crate) fn witness_discovery_scan_dirs() -> Vec<String> {
    static SCAN_DIRS: OnceLock<Vec<String>> = OnceLock::new();
    SCAN_DIRS
        .get_or_init(|| witness_discovery_scan_dirs_from_source(ci_layer_roots_authority_content()))
        .clone()
}

pub fn census_corpus_roots_follow_layer_authority() -> bool {
    let synthetic = "module gunbc.ci_layer_roots\n\n\
         data witness_layer_roots: List<String> = [\"alpha_layer_root\", \"beta_layer_root\", \"gamma_layer_root\"]\n";
    let follows = witness_layer_roots_from_source(synthetic)
        == ["alpha_layer_root", "beta_layer_root", "gamma_layer_root"];
    let live_nonempty = !witness_layer_roots().is_empty();
    follows && live_nonempty
}

pub(crate) fn default_source_roots() -> Vec<String> {
    let ws = workspace_root();
    witness_layer_roots()
        .iter()
        .map(|r| ws.join(r).to_string_lossy().into_owned())
        .collect()
}

pub fn build_module_path_index_from_witness_roots() -> HashMap<String, String> {
    build_module_path_index(&default_source_roots())
}

pub fn source_path_for_module_path(module_path: String) -> String {
    let index = build_module_path_index_from_witness_roots();
    index
        .get(&module_path)
        .cloned()
        .unwrap_or_else(|| panic!("module_path_index: unknown module path '{module_path}'"))
}

pub fn free_monoid_symbol_value_to_dotted_string(value: &v1_interpreter::Value) -> String {
    v1_interpreter::free_monoid_symbol_value_to_dotted_string(value)
}

pub fn free_monoid_symbol_value_from_dotted_string(
    ctx: &v1_interpreter::InterpContext,
    dotted: &str,
) -> v1_interpreter::Value {
    use v1_interpreter::{sorted_fields, Value};

    let fm_variant = |variant: &str, fields: Vec<_>| Value::Variant {
        type_name: ctx.sym("FreeMonoid"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(fields),
    };
    if dotted.is_empty() {
        return fm_variant("Empty", vec![]);
    }
    let mut qn = fm_variant("Empty", vec![]);
    for seg in dotted.split('.').rev() {
        qn = fm_variant(
            "Cons",
            sorted_fields(vec![
                (ctx.sym("head"), Value::Str(seg.to_string())),
                (ctx.sym("tail"), qn),
            ]),
        );
    }
    qn
}

pub(crate) fn repo_rel(path: &Path) -> String {
    let ws = workspace_root();
    let s = path.to_string_lossy().replace('\\', "/");
    let prefix = format!("{}/", ws.to_string_lossy().replace('\\', "/"));
    s.strip_prefix(&prefix)
        .map(|p| p.to_string())
        .unwrap_or(s)
        .trim_start_matches("./")
        .to_string()
}

pub(crate) fn is_test_dag(path: &str) -> bool {
    path.ends_with("_test.dag")
}

pub(crate) fn corpus_dag_files() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    for root in witness_layer_roots() {
        collect_dag_files_tolerant(&workspace_root().join(&root), &mut paths);
    }
    let mut out = Vec::new();
    for p in paths {
        let rel = repo_rel(&p);
        if let Ok(content) = std::fs::read_to_string(&p) {
            out.push((rel, content));
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(crate) fn strip_line_comment(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                out.push(b' ');
                escaped = false;
            } else if b == b'\\' {
                out.push(b' ');
                escaped = true;
            } else if b == b'"' {
                out.push(b'"');
                in_string = false;
            } else {
                out.push(b' ');
            }
        } else if b == b'"' {
            in_string = true;
            out.push(b'"');
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            break;
        } else {
            out.push(b);
        }
        i += 1;
    }
    String::from_utf8(out).expect("strip_line_comment output is valid UTF-8")
}

pub(crate) fn brace_delta(line: &str) -> i32 {
    let c = strip_line_comment(line);
    c.matches('{').count() as i32 - c.matches('}').count() as i32
}

type ModuleSourceIndex = HashMap<String, Rc<v1_compiler_compile::SourceFile>>;

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
                if let Some(existing) = index.get(&module_path) {
                    if existing.path != rel_path {
                        panic!(
                            "module-path collision: module '{}' is declared by both '{}' and '{}' — one module, one authority (DESIGN §3); silent last-root-wins shadowing broke the floor (extdeps.shell, 2026-07-01) — de-fork or rename one side",
                            module_path, existing.path, rel_path
                        );
                    }
                }
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

/// Workspace-relative path for module-graph closure queries (`v2.lens.module_graph`).
fn workspace_relative_repo_path(path: &str) -> String {
    let norm = path.strip_prefix("./").unwrap_or(path).replace('\\', "/");
    let p = Path::new(&norm);
    if p.is_absolute() {
        let ws = workspace_root();
        p.strip_prefix(&ws)
            .map(|rp| rp.to_string_lossy().replace('\\', "/"))
            .unwrap_or(norm)
    } else {
        norm
    }
}

/// Normalize `source_roots` to the workspace-relative form `import_resolution_facts` /
/// `module_declaration_facts` expect when invoked from `.dag` (`witness_layer_roots` style).
fn pool_roots_for_module_graph_closure(source_roots: &[String]) -> Vec<String> {
    let ws = workspace_root();
    source_roots
        .iter()
        .map(|r| {
            let p = Path::new(r);
            if p.is_absolute() {
                p.strip_prefix(&ws)
                    .map(|rp| rp.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| r.replace('\\', "/"))
            } else {
                r.replace('\\', "/")
            }
        })
        .collect()
}

fn path_to_source_lookup(
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

/// Host realization of `v2.lens.module_graph.import_closure` over modeled fact rows.
/// Authority: `src/v2/lens/module_graph.dag` — this is the consumer repoint surface for
/// `cli_run.rs` resolve/reconcile (Phase 1 de-fork); fact extraction stays on the existing
/// `import_resolution_facts` / `module_declaration_facts` builtins.
pub fn import_closure_from_facts(
    entry_path: &str,
    edges: &[ImportResolutionFactRaw],
    nodes: &[ModuleDeclarationFactRaw],
) -> Vec<String> {
    let entry_path = workspace_relative_repo_path(entry_path);
    let mut reached: Vec<String> = vec![entry_path];
    let fuel = nodes.len();
    for _ in 0..fuel {
        let before = reached.len();
        let mut next = reached.clone();
        for importer in &reached {
            let importer_norm = workspace_relative_repo_path(importer);
            for edge in edges {
                if workspace_relative_repo_path(&edge.path) != importer_norm {
                    continue;
                }
                for node in nodes {
                    if node.module == edge.import_module {
                        let path = workspace_relative_repo_path(&node.path);
                        if !next.iter().any(|p| p == &path) {
                            next.push(path);
                        }
                    }
                }
            }
        }
        if next.len() == before {
            break;
        }
        reached = next;
    }
    reached
}

/// Pre-built `import_resolution_facts` / `module_declaration_facts` rows for one pool-root
/// set. Built once per `MultiEntryIndex` / resolve pass so closure queries do not re-scan the
/// corpus on every `resolve_transitively` call (Phase 1 perf receipt, DESIGN §2).
pub struct ModuleGraphFactsLive {
    edges: Vec<ImportResolutionFactRaw>,
    nodes: Vec<ModuleDeclarationFactRaw>,
}

#[cfg(test)]
static MODULE_GRAPH_FACTS_BUILD_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn reset_module_graph_facts_build_count_for_test() {
    MODULE_GRAPH_FACTS_BUILD_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn module_graph_facts_build_count_for_test() -> usize {
    MODULE_GRAPH_FACTS_BUILD_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn build_module_graph_facts_live(pool_roots: &[String]) -> ModuleGraphFactsLive {
    #[cfg(test)]
    MODULE_GRAPH_FACTS_BUILD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    const EXCLUDE: &[String] = &[];
    let roots = pool_roots_for_module_graph_closure(pool_roots);
    ModuleGraphFactsLive {
        edges: import_resolution_facts(&roots, &roots, EXCLUDE),
        nodes: module_declaration_facts(&roots),
    }
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
    import_closure_from_facts(entry_path, &facts.edges, &facts.nodes)
}

#[cfg(test)]
fn resolve_transitively_bfs_legacy(
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

fn resolve_transitively(
    entry_sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let mut path_lookup = path_to_source_lookup(index);
    for entry in &entry_sources {
        let rel = workspace_relative_repo_path(&entry.path);
        path_lookup.entry(rel).or_insert_with(|| entry.clone());
        path_lookup
            .entry(entry.path.clone())
            .or_insert_with(|| entry.clone());
    }

    let mut all_paths: BTreeSet<String> = BTreeSet::new();
    for entry in &entry_sources {
        let entry_rel = workspace_relative_repo_path(&entry.path);
        for path in import_closure_live_paths_with_facts(&entry_rel, facts) {
            all_paths.insert(workspace_relative_repo_path(&path));
        }
    }

    let mut result = Vec::with_capacity(all_paths.len());
    for path in all_paths {
        let sf = path_lookup.get(&path).cloned().ok_or_else(|| {
            format!(
                "import_closure_live: closure path '{path}' has no provenance in module index (fail-closed)"
            )
        })?;
        result.push(sf);
    }
    result.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}

pub fn load_sources_for_entry(
    source_roots: &[String],
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let index = build_module_index(source_roots);
    let facts = build_module_graph_facts_live(source_roots);
    load_sources_for_entry_with_index(&index, &facts, entry_path)
}

fn load_sources_for_entry_with_index(
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let entry_source = entry_source_from_index_or_disk(index, entry_path)?;
    let rel_path = entry_source.path.clone();

    let sources = resolve_transitively(vec![entry_source.clone()], index, facts)?;
    let mut sources = sources;
    if !sources.iter().any(|s| s.path == rel_path) {
        sources.push(entry_source);
    }
    Ok(sources)
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

fn load_sources(
    source_roots: &[String],
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let index = build_module_index(source_roots);
    let facts = build_module_graph_facts_live(source_roots);
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

    let skipped_moduleless = moduleless_dag_entry_paths(&entry_files);
    report_moduleless_dag_entry_skips(&skipped_moduleless);

    let mut entry_for_queue = Vec::new();
    for (path, content) in &entry_files {
        if let Some(_mod_path) = extract_module_path(content) {
            let source = Rc::new(v1_compiler_compile::SourceFile {
                path: path.clone(),
                content: content.clone(),
            });
            entry_for_queue.push(source);
        }
    }

    let mut sources = resolve_transitively(entry_for_queue, &index, &facts)?;
    for (path, content) in entry_files {
        if extract_module_path(&content).is_none() {
            continue;
        }
        if !sources.iter().any(|s| s.path == path) {
            sources.push(Rc::new(v1_compiler_compile::SourceFile { path, content }));
        }
    }
    Ok(sources)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimOutcome {
    Pass,
    Fail,
    NotBool { got: String },
    RuntimeError { message: String },
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
    let index = build_module_index(source_roots);
    let facts = build_module_graph_facts_live(source_roots);
    resolve_entry_graph_with_index(&index, &facts, entry_file)
}

pub struct MultiEntryIndex {
    source_files: ModuleSourceIndex,
    module_graph_facts: ModuleGraphFactsLive,
    intern_table: RefCell<Rc<InternTable>>,
    parse_cache: RefCell<HashMap<String, (Rc<v1_compiler_parse::ParseResult>, Rc<NewlineIndex>)>>,
    typed_module_cache: RefCell<HashMap<String, Rc<v1_compiler_infer::TypecheckModuleResult>>>,
}

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

pub fn build_multi_entry_index(source_roots: &[String]) -> MultiEntryIndex {
    MultiEntryIndex {
        source_files: build_module_index(source_roots),
        module_graph_facts: build_module_graph_facts_live(source_roots),
        intern_table: RefCell::new(seed_kernel_intern_names(empty_intern_table())),
        parse_cache: RefCell::new(HashMap::new()),
        typed_module_cache: RefCell::new(HashMap::new()),
    }
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

fn resolve_entry_graph_with_index(
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    set_phase(FloorPhase::Resolve, entry_file);
    let sources = load_sources_for_entry_with_index(index, facts, entry_file)?;
    set_phase(FloorPhase::Typecheck, entry_file);
    resolved_graph_from_sources(sources, ResolveTypecheckGate::Strict)
}

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
    set_phase(FloorPhase::Resolve, entry_file);
    let sources = load_sources_for_entry_with_index(
        &index.source_files,
        &index.module_graph_facts,
        entry_file,
    )?;

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
        let cached = index.parse_cache.borrow().get(&source.path).cloned();

        let (parse_result, nl_index) = match cached {
            Some(entry) => entry,
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

    set_phase(FloorPhase::Typecheck, entry_file);
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

fn reconcile_with_typed_cache(
    graph: Rc<v1_compiler_resolve::ModuleGraph>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    intern_table: Rc<InternTable>,
    typed_cache: &RefCell<HashMap<String, Rc<v1_compiler_infer::TypecheckModuleResult>>>,
) -> Rc<ResolvedGraph> {
    reset_type_env_lookup_profile();
    let mut modules: Rc<Vec<Rc<TypedModule>>> = Rc::new(Vec::new());
    let mut module_index: Rc<HashMap<String, Rc<TypedModule>>> = v1_rt::rc_empty_map();
    let mut item_registry: Rc<HashMap<String, Rc<ItemInfo>>> = v1_rt::rc_empty_map();
    let mut diag_chunks: Vec<Rc<Vec<Rc<ErrorNode>>>> = Vec::new();

    for resolved in graph.modules.iter().cloned() {
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
    let modules =
        v1_compiler_infer::rewire_type_env_parent_links(modules.clone(), source_indices.clone());
    let modules = v1_compiler_infer::rewire_type_env_import_str_binding_identity(
        modules.clone(),
        source_indices.clone(),
    );
    let modules =
        v1_compiler_infer::rewire_func_env_parent_links(modules.clone(), source_indices.clone());
    let emit_graph_info = v1_compiler_infer::build_emit_graph_info(modules.clone());
    maybe_print_type_env_lookup_profile();
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

pub fn make_eval_context(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_fixture_store(graph, source_indices, execution_mode, None)
}

/// Evaluate `gunbc.output_policy.resolve_channel_policy` from the .dag authority at
/// the current CLI verbosity and install the per-channel decisions for the
/// interpreter's host-effect trace funnel (`v1_interpreter::output_decision`). The
/// decision logic lives entirely in .dag; this only transports the evaluated
/// verdicts across the seed↔.dag boundary. Best-effort: if the policy module can't
/// be resolved/evaluated, the funnel keeps its `Full` fallback (pre-funnel behavior).
pub fn install_output_policy(source_roots: &[String]) {
    use v1_interpreter::{OutputDecision, Value};
    let (verbose, quiet) = match v1_interpreter::cli_verbosity() {
        v1_interpreter::Verbosity::Verbose => (true, false),
        v1_interpreter::Verbosity::Quiet => (false, true),
        v1_interpreter::Verbosity::Normal => (false, false),
    };
    let entry = "dag/gunbc/output_policy.dag";
    let (graph, indices) = match resolve_entry_graph(source_roots, entry) {
        Ok(g) => g,
        Err(_) => return,
    };
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let policy = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "resolve_channel_policy",
        &[
            (Some("verbose".to_string()), Value::Bool(verbose)),
            (Some("quiet".to_string()), Value::Bool(quiet)),
        ],
        false,
    ) {
        Ok(v) => v,
        Err(_) => return,
    };
    let Value::Record { fields, .. } = &policy else {
        return;
    };
    let decision = |name: &str| -> OutputDecision {
        match ctx.field(fields, name) {
            Some(Value::Variant { variant_name, .. }) => {
                if ctx.sym_eq(*variant_name, "Suppressed") {
                    OutputDecision::Suppressed
                } else if ctx.sym_eq(*variant_name, "Condensed") {
                    OutputDecision::Condensed
                } else {
                    OutputDecision::Full
                }
            }
            _ => OutputDecision::Full,
        }
    };
    v1_interpreter::set_output_policy([
        decision("diagnostic"),
        decision("claim_result"),
        decision("progress"),
        decision("shell_trace"),
        decision("instrumentation"),
    ]);
}

/// Evaluate `extdeps.render.surface.resolve_group_syntax(github_actions)` from the
/// .dag authority and install the per-target group-marker strings for the host-effect
/// trace grouping (`v1_interpreter::group_begin`/`group_end`). `github_actions` is
/// read from the environment (`GITHUB_ACTIONS=true`, the runner's own signal) — the
/// ONLY seed-side fact; which markers that target implies stays the .dag authority's.
/// Best-effort: if the module can't resolve/evaluate, grouping stays off (ungrouped,
/// pre-grouping behavior).
pub fn install_group_syntax(source_roots: &[String]) {
    use v1_interpreter::{InstalledGroupSyntax, Value};
    let github_actions = std::env::var("GITHUB_ACTIONS")
        .map(|v| v == "true")
        .unwrap_or(false);
    let entry = "dag/extdeps/render/surface.dag";
    let (graph, indices) = match resolve_entry_graph(source_roots, entry) {
        Ok(g) => g,
        Err(_) => return,
    };
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let syntax = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "resolve_group_syntax",
        &[(
            Some("github_actions".to_string()),
            Value::Bool(github_actions),
        )],
        false,
    ) {
        Ok(v) => v,
        Err(_) => return,
    };
    let Value::Record { fields, .. } = &syntax else {
        return;
    };
    let str_field = |name: &str| -> Option<String> {
        match ctx.field(fields, name) {
            Some(Value::Str(s)) => Some(s.clone()),
            _ => None,
        }
    };
    let (Some(open_prefix), Some(open_suffix)) =
        (str_field("open_prefix"), str_field("open_suffix"))
    else {
        return;
    };
    // close_line is an Optional: Present { value: "::endgroup::" } | Absent (none).
    let close_line = match ctx.field(fields, "close_line") {
        Some(Value::Variant {
            variant_name,
            fields: vf,
            ..
        }) if ctx.sym_eq(*variant_name, "Present") => match ctx.field(vf, "value") {
            Some(Value::Str(s)) => Some(s.clone()),
            _ => None,
        },
        _ => None,
    };
    v1_interpreter::set_group_syntax(InstalledGroupSyntax {
        open_prefix,
        open_suffix,
        close_line,
    });
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

fn dag_source_roots(source_roots: &[String]) -> Vec<String> {
    let mut dag: Vec<String> = source_roots
        .iter()
        .filter(|r| {
            let p = Path::new(r.as_str());
            p.ends_with("dag") || p.file_name().is_some_and(|n| n == "dag")
        })
        .cloned()
        .collect();
    for root in source_roots {
        let child = Path::new(root).join("dag");
        if child.is_dir() {
            dag.push(child.to_string_lossy().into_owned());
        }
    }
    dag.sort();
    dag.dedup();
    dag
}

pub fn precompute_whole_tree_published_mock_keys(
    source_roots: &[String],
) -> Result<std::collections::HashSet<String>, String> {
    let dag_roots = dag_source_roots(source_roots);
    if dag_roots.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let index = build_module_index(&dag_roots);
    // Only modules that DECLARE a `PublishedMockCase` corpus can contribute keys —
    // `resolve_published_mock_keys` reads them by exact type annotation. Strict-
    // resolving the whole 600+ module tree to find the ~13 declarers is §2
    // irrelevant work, and that transient whole-tree `ResolvedGraph` is the floor's
    // dominant RSS (measured ~1.46 GiB to produce ~58 strings). Select the
    // declarers and resolve only their transitive import closures. The `.contains`
    // prefilter is a safe over-inclusive candidate set: `.dag` has no comment
    // syntax (a string match is structural), and the downstream
    // `type_annotation_names(.., "PublishedMockCase")` check is exact, so a
    // false-positive file only widens the closure slightly — it cannot fabricate a key.
    let declarers: Vec<Rc<v1_compiler_compile::SourceFile>> = index
        .values()
        .filter(|sf| sf.content.contains("PublishedMockCase"))
        .cloned()
        .collect();
    if declarers.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let facts = build_module_graph_facts_live(&dag_roots);
    let all_sources = resolve_transitively(declarers, &index, &facts)?;
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

/// Build an interpreter context over the WHOLE source-root corpus (every `.dag`
/// module under `source_roots`), resolved in one pass under the Strict gate — the
/// same whole-tree resolve `precompute_whole_tree_published_mock_keys` performs,
/// but retaining the context so a `.dag` reflection accessor (e.g.
/// `fn_arrow_decl_facts_live`) walks `ctx.modules == the whole tree` rather than a
/// single entry's import closure. This is the #5364 widening substrate: coverage
/// goes from per-entry resolve-closure to whole-tree-in-one-pass. The marshaling
/// runs in THIS context's interner, so reflected `Node` values are self-consistent
/// (no cross-context Symbol mismatch).
/// `exclude_substrings` drop modules whose source path contains any listed
/// substring BEFORE the resolve. This is required, not optional: the corpus
/// contains intentionally-malformed scanner fixture inputs (e.g.
/// `src/v2/test/fixture/layering_scan/**/plant.dag` declaring imports of modules
/// that do not exist) which are test DATA referenced by string path, not live
/// code — a Strict whole-tree resolve over them fails on the deliberate
/// `unresolved import`. Excluding them is a coverage decision, so the count of
/// dropped modules is returned for the caller to log (DESIGN §6 — no silent cap).
pub struct WholeTreeCtx {
    pub ctx: v1_interpreter::InterpContext,
    pub modules_resolved: usize,
    pub modules_excluded: usize,
}

pub struct WholeTreeStrictSources {
    pub sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    pub modules_resolved: usize,
    pub modules_excluded: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WholeCorpusSemanticOracle {
    pub diagnostic_fingerprint: String,
    pub rust_corpus_repr: String,
    /// Canonical JSON identity hash of the full `EmitGraphInfo` (resolved emit repr).
    pub emit_graph_fingerprint: String,
    /// Aggregate per-module diagnostics + emit-repr rows + graph-level emit metadata.
    pub corpus_fingerprint: String,
    pub modules_resolved: usize,
    pub per_module_rows: usize,
}

fn sort_json_object_keys(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for key in keys {
                if let Some(child) = map.get(&key) {
                    out.insert(key, sort_json_object_keys(child.clone()));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .into_iter()
                .map(sort_json_object_keys)
                .collect::<Vec<_>>(),
        ),
        other => other,
    }
}

fn canonical_json_identity_hash<T: Serialize>(value: &T) -> Result<String, String> {
    let raw = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let sorted = sort_json_object_keys(raw);
    let bytes = serde_json::to_vec(&sorted).map_err(|e| e.to_string())?;
    Ok(v1_rt::bytes_identity_hash(&bytes))
}

fn module_defined_type_names(
    module: &TypedModule,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> BTreeSet<String> {
    use ItemKind::{DataItem, TypeItem};
    let mut names = BTreeSet::new();
    for item in module.items.iter() {
        if matches!(item_kind(item.clone()), TypeItem | DataItem) {
            names.insert(authored_name_at(source_indices.clone(), item.clone()));
        }
    }
    names
}

fn module_emit_repr_fingerprint(
    module: &TypedModule,
    emit_info: &crate::v1_compiler_infer_emit_info::EmitGraphInfo,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Result<String, String> {
    use crate::v1_compiler_infer_emit_info::TypeSummary;

    let type_names = module_defined_type_names(module, source_indices);
    let mut type_summaries = BTreeMap::<String, TypeSummary>::new();
    for name in type_names {
        if let Some(summary) = emit_info.type_summaries.get(&name) {
            type_summaries.insert(name, summary.as_ref().clone());
        }
    }

    canonical_json_identity_hash(&type_summaries)
}

pub fn whole_corpus_semantic_oracle_snapshot(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<WholeCorpusSemanticOracle, String> {
    use crate::v1_compiler_infer_emit_info::RustCorpusRepr::{FaithfulFreeMonoid, HostNative};

    let picked = whole_tree_strict_sources(source_roots, exclude_substrings)?;
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(picked.sources));
    let graph = result.graph.as_ref().ok_or_else(|| {
        let si: HashMap<String, Rc<NewlineIndex>> = result
            .newline_indices
            .iter()
            .map(|idx| (idx.file.clone(), idx.clone()))
            .collect();
        format!(
            "whole-corpus strict resolve failed:\n{}",
            format_error_nodes(&result.diagnostics, &Rc::new(si))
        )
    })?;
    let source_indices = result.source_indices.clone();
    let mut diag_lines: Vec<String> = graph
        .diagnostics
        .iter()
        .map(|d| v1_compiler_compile::serialize_diagnostic(d.clone()))
        .collect();
    diag_lines.sort();
    let diagnostic_fingerprint = v1_rt::bytes_identity_hash(diag_lines.join("\n").as_bytes());
    let rust_corpus_repr = match graph.emit_graph_info.corpus_repr {
        HostNative => "HostNative".to_string(),
        FaithfulFreeMonoid => "FaithfulFreeMonoid".to_string(),
    };
    let emit_graph_fingerprint = canonical_json_identity_hash(graph.emit_graph_info.as_ref())?;

    let mut modules: Vec<Rc<TypedModule>> = graph.modules.iter().cloned().collect();
    modules.sort_by(|left, right| {
        let left_path = authored_name_at(source_indices.clone(), left.module.clone());
        let right_path = authored_name_at(source_indices.clone(), right.module.clone());
        left_path.cmp(&right_path)
    });

    let mut per_module_lines = Vec::with_capacity(modules.len());
    for module in &modules {
        let module_path = authored_name_at(source_indices.clone(), module.module.clone());
        let mut module_diag_lines: Vec<String> = graph
            .diagnostics
            .iter()
            .filter(|diag| diag.module_name.as_str() == module_path.as_str())
            .map(|diag| v1_compiler_compile::serialize_diagnostic(diag.clone()))
            .collect();
        module_diag_lines.sort();
        let module_diag_fingerprint =
            v1_rt::bytes_identity_hash(module_diag_lines.join("\n").as_bytes());
        let module_emit_fingerprint = module_emit_repr_fingerprint(
            module.as_ref(),
            graph.emit_graph_info.as_ref(),
            source_indices.clone(),
        )?;
        per_module_lines.push(format!(
            "{module_path}\t{module_diag_fingerprint}\t{module_emit_fingerprint}"
        ));
    }

    let per_module_rows = per_module_lines.len();
    let per_module_blob = per_module_lines.join("\n");
    let corpus_fingerprint = v1_rt::bytes_identity_hash(
        format!(
            "diagnostic_fingerprint={diagnostic_fingerprint}\n\
             emit_graph_fingerprint={emit_graph_fingerprint}\n\
             rust_corpus_repr={rust_corpus_repr}\n\
             per_module:\n{per_module_blob}"
        )
        .as_bytes(),
    );

    Ok(WholeCorpusSemanticOracle {
        diagnostic_fingerprint,
        rust_corpus_repr,
        emit_graph_fingerprint,
        corpus_fingerprint,
        modules_resolved: picked.modules_resolved,
        per_module_rows,
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

pub fn closure_subject_for_entry(index: &MultiEntryIndex, entry: &str) -> Result<String, String> {
    let sources =
        load_sources_for_entry_with_index(&index.source_files, &index.module_graph_facts, entry)?;
    Ok(subject_digest_for_closure(&sources))
}

pub fn run_claim(ctx: &v1_interpreter::InterpContext, function: &str) -> ClaimOutcome {
    // ProcessExit is the wet-gate return convention (ExitSuccess => Pass, ExitFailure => Fail).
    // NotProcessExit stays NotBool — fail-closed preserved for genuine type errors. Reuses
    // pre-existing classify_exit. Required: emitted pre-push drift --wet gate runs through
    // claim_batch -> run_claim; without this mapping ExitSuccess -> exit 1 false-blocks push
    // (receipt: claim_batch rebuilt on reverted seed reproduced the false-block).
    match v1_interpreter::run_in_context(ctx, function, false) {
        Ok(v1_interpreter::Value::Bool(true)) => ClaimOutcome::Pass,
        Ok(v1_interpreter::Value::Bool(false)) => ClaimOutcome::Fail,
        Ok(other) => match classify_exit(&other, ctx) {
            ExitClass::Success => ClaimOutcome::Pass,
            ExitClass::Failure { .. } => ClaimOutcome::Fail,
            ExitClass::NotProcessExit { type_name } => ClaimOutcome::NotBool { got: type_name },
        },
        Err(e) => ClaimOutcome::RuntimeError {
            message: format!("{}", e),
        },
    }
}

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

pub fn run_value(
    ctx: &v1_interpreter::InterpContext,
    function: &str,
) -> Result<v1_interpreter::Value, String> {
    v1_interpreter::run_in_context(ctx, function, false).map_err(|e| format!("{}", e))
}

pub fn handle_ci() {
    handle_run_with_options(
        witness_layer_roots(),
        "main".to_string(),
        Some("dag/tools/gunbc_ci.dag".to_string()),
        false,
        false,
    );
}

pub fn handle_run(
    source_roots: Vec<String>,
    function: String,
    entry_file: Option<String>,
    claim_run: bool,
) {
    handle_run_with_options(source_roots, function, entry_file, false, claim_run);
}

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
        None => match load_sources(&source_roots) {
            Ok(sources) => sources,
            Err(msg) => {
                eprintln!("error: {}", msg);
                std::process::exit(1);
            }
        },
    };
    eprintln!("resolved {} sources", sources.len());

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

    let graph = match result.graph.as_ref() {
        Some(g) => g,
        None => {
            eprintln!("error: compilation produced no graph");
            std::process::exit(1);
        }
    };

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
                match classify_exit(&val, &ctx) {
                    ExitClass::Success => {}
                    ExitClass::Failure { code, reason } => {
                        if let Some(message) = reason {
                            eprintln!("{message}");
                        }
                        std::process::exit(code);
                    }
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

enum ExitClass {
    Success,
    Failure { code: i32, reason: Option<String> },
    NotProcessExit { type_name: String },
}

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
                let code = match ctx.field(fields, "code") {
                    Some(v1_interpreter::Value::Int(n)) => *n as i32,
                    _ => 1,
                };
                let reason = match ctx.field(fields, "reason") {
                    Some(v1_interpreter::Value::Str(s)) if !s.is_empty() => Some(s.clone()),
                    _ => None,
                };
                ExitClass::Failure { code, reason }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedDeclRef {
    pub module: String,
    pub name: String,
}

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

fn entry_likely_has_unified_claim_owned_data(content: &str) -> bool {
    content
        .lines()
        .any(|line| line.trim_start().starts_with("data unified_claim_"))
}

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

struct DiscoveryResolveGroup {
    entries: Vec<(String, String, usize)>,
    sources: HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    decl_names: HashMap<String, String>,
}

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

pub struct OwnedDataDiscovery {
    pub records: Vec<OwnedDataDeclRecord>,
    pub entry_count: usize,
    pub graph_resolves: usize,
    pub group_split_collisions: Vec<String>,
}

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
    let module_graph_facts = build_module_graph_facts_live(source_roots);

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

        let closure =
            load_sources_for_entry_with_index(&module_index, &module_graph_facts, &entry)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedDataDiscoveryReceipt {
    pub unified_claim_arm_count: usize,
    pub bool_witness_claim_arm_count: usize,
    pub illegal_other_init_count: usize,
    pub bool_witness_transport_row_count: usize,
    pub transport_projection_complete: bool,
}

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

fn dag_manifest_scalar_escape(s: &str) -> Result<String, String> {
    if s.contains('{') || s.contains('}') {
        return Err(format!(
            "manifest scalar field must be brace-free (got '{{' or '}}'): {s:?}"
        ));
    }
    Ok(dag_string_escape_core(s))
}

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

#[derive(Clone)]
pub struct DiscoveryRow {
    pub label: String,
    pub entry: String,
    pub function: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryResolveReceipt {
    pub entry: String,
    pub closure_subject: String,
    pub resolve_nanos: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryWitnessOutcome {
    pub entry: String,
    pub function: String,
    pub outcome: ClaimOutcome,
}

pub struct DiscoverySummary {
    pub total: usize,
    pub passed: usize,
    pub skipped: usize,
    pub failures: Vec<String>,
    pub witness_outcomes: Vec<DiscoveryWitnessOutcome>,
    pub entry_resolve_receipts: Vec<EntryResolveReceipt>,
    pub total_resolve_nanos: u128,
    pub performance_receipts: Vec<v1_interpreter::PerformanceReceipt>,
    pub total_measured_nanos: u128,
}

#[derive(Debug, Clone)]
pub struct TimingPercentiles {
    pub p50: u128,
    pub p90: u128,
    pub p95: u128,
    pub p99: u128,
    pub p100: u128,
}

pub fn compute_percentiles(mut values: Vec<u128>) -> TimingPercentiles {
    if values.is_empty() {
        return TimingPercentiles {
            p50: 0,
            p90: 0,
            p95: 0,
            p99: 0,
            p100: 0,
        };
    }
    values.sort_unstable();
    let len = values.len();
    let clamp_idx = |f: f64| {
        let idx = (len as f64 * f) as usize;
        idx.min(len - 1)
    };

    TimingPercentiles {
        p50: values[clamp_idx(0.50)],
        p90: values[clamp_idx(0.90)],
        p95: values[clamp_idx(0.95)],
        p99: values[clamp_idx(0.99)],
        p100: values[len - 1],
    }
}

// SCAFFOLD (§7 hand-Rust shrink-to-zero, dissolution named): the v1 evaluator measures its own
// per-witness resolve+eval percentiles here — seed-side justified (the evaluator cannot measure
// itself without circularity). The *rendering* of these timings now lives in `dag/gunbc/ci_render.dag`
// (boxed Frames over `std.render`, width-parameterized by the medium's `Viewport.width`); this Rust
// only produces the measured data. Full dissolution: ROADMAP lane "CI observability" emits the
// `TimingPercentiles` rows as a substrate value so a .dag witness measures + histograms natively,
// at which point this measurement struct collapses too.
pub struct HistogramData {
    pub included: usize,
    pub skipped: usize,
    pub total: TimingPercentiles,
    pub resolve: TimingPercentiles,
    pub eval: TimingPercentiles,
}

/// One witness row with per-witness eval time and its entry's amortized resolve cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessTimingRow {
    pub entry: String,
    pub function: String,
    pub eval_nanos: u128,
    pub resolve_nanos: u128,
    pub total_nanos: u128,
}

pub const DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N: usize = 15;

pub fn compute_witness_timing_rows(
    summary: &DiscoverySummary,
) -> Result<Vec<WitnessTimingRow>, String> {
    if summary.performance_receipts.len() != summary.witness_outcomes.len() {
        return Err(format!(
            "[attribution] SKIPPED: mismatched vector lengths (performance_receipts={}, witness_outcomes={}) — timings unreliable",
            summary.performance_receipts.len(),
            summary.witness_outcomes.len()
        ));
    }

    let mut entry_resolve_map: HashMap<String, u128> = HashMap::new();
    for receipt in &summary.entry_resolve_receipts {
        entry_resolve_map.insert(receipt.entry.clone(), receipt.resolve_nanos);
    }

    let mut rows: Vec<WitnessTimingRow> = Vec::new();
    for (perf, outcome) in summary
        .performance_receipts
        .iter()
        .zip(summary.witness_outcomes.iter())
    {
        let Some(resolve_nanos) = entry_resolve_map.get(&outcome.entry).copied() else {
            continue;
        };
        let eval_nanos = perf.wall_nanos;
        rows.push(WitnessTimingRow {
            entry: outcome.entry.clone(),
            function: outcome.function.clone(),
            eval_nanos,
            resolve_nanos,
            total_nanos: resolve_nanos + eval_nanos,
        });
    }
    Ok(rows)
}

/// Return the top `n` witnesses ranked by eval time (descending), stable on function name.
pub fn top_n_slowest_witnesses(rows: &[WitnessTimingRow], n: usize) -> Vec<WitnessTimingRow> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| {
        b.eval_nanos
            .cmp(&a.eval_nanos)
            .then_with(|| a.function.cmp(&b.function))
            .then_with(|| a.entry.cmp(&b.entry))
    });
    sorted.truncate(n);
    sorted
}

pub fn compute_histogram_data(summary: &DiscoverySummary) -> Result<HistogramData, String> {
    if summary.performance_receipts.len() != summary.witness_outcomes.len() {
        return Err(format!(
            "[histogram] SKIPPED: mismatched vector lengths (performance_receipts={}, witness_outcomes={}) — timings unreliable",
            summary.performance_receipts.len(),
            summary.witness_outcomes.len()
        ));
    }

    let mut entry_resolve_map: HashMap<String, u128> = HashMap::new();
    for receipt in &summary.entry_resolve_receipts {
        entry_resolve_map.insert(receipt.entry.clone(), receipt.resolve_nanos);
    }

    let mut total_times: Vec<u128> = Vec::new();
    let mut resolve_times: Vec<u128> = Vec::new();
    let mut eval_times: Vec<u128> = Vec::new();
    let mut skipped_missing_entry_resolve = 0;

    // performance_receipts and witness_outcomes are both generated in the same discovery pass
    // with matching cardinality and order, so positional matching is stable across discovery runs.
    for (perf, outcome) in summary
        .performance_receipts
        .iter()
        .zip(summary.witness_outcomes.iter())
    {
        let resolve_nanos = match entry_resolve_map.get(&outcome.entry).copied() {
            Some(nanos) => nanos,
            None => {
                skipped_missing_entry_resolve += 1;
                continue;
            }
        };
        let eval_nanos = perf.wall_nanos;
        let total_nanos = resolve_nanos + eval_nanos;

        total_times.push(total_nanos);
        resolve_times.push(resolve_nanos);
        eval_times.push(eval_nanos);
    }

    Ok(HistogramData {
        included: total_times.len(),
        skipped: skipped_missing_entry_resolve,
        total: compute_percentiles(total_times),
        resolve: compute_percentiles(resolve_times),
        eval: compute_percentiles(eval_times),
    })
}

pub const WET_HERMETIC_EQUIVALENCE_WITNESS_ENTRY: &str =
    "dag/test/claim/wet_hermetic_equivalence_witness_test.dag";
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

pub fn is_governed_service_representative_row(row: &DiscoveryRow, prefix: &str) -> bool {
    !prefix.is_empty() && row.entry.contains(prefix)
}

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

/// Peak resident set from `/proc/self/status` VmHWM (high water mark), in bytes.
pub fn peak_rss_vhwm_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// Mirror of `gunbc.ci_layer_roots.witness_exclusion_substrings` — the .dag model is the
/// single authority; plan-driven paths (`claim_executor` + `ci_floor_plan.dag`) read from
/// `RunnableDiscoveryBatch.exclude_substrings`. SCAFFOLD — two remaining consumers to migrate:
/// (a) `claim_batch.rs`: pre-push hook runner reads this constant directly (not plan-driven);
///     dissolve when pre-push hooks fold over a `RunnableDiscoveryBatch` from the model, or
///     when `claim_batch` reads `witness_exclusion_substrings` from the v2 evaluator;
/// (b) `DiscoveryCorpusOptions::default()` + test call sites in `pipeline.rs` /
///     `wet_hermetic_equivalence_test.rs`: pass explicit excludes from the model authority.
/// Dissolution trigger: FLOOR_DISCOVERY_EXCLUDES has zero call sites outside this comment.
pub const FLOOR_DISCOVERY_EXCLUDES: &[&str] = &[
    "impossible_bug",
    "test/manual/",
    "glob_discovery.dag",
    "glob_discovery_law.dag",
    "host_discovered_owned_data_manifest.dag",
    "host_source_root_ingest_manifest.dag",
    "program_assembly/real_ingest_test.dag",
    "self_host/compiler_closure_emit_from_ingest_test.dag",
    "unified_test_claim_substrate_equivalence.dag",
    "ci_exclusion_proof_test.dag",
    "test/claim/execution/",
];

pub fn floor_discovery_path_excluded(path: &str) -> bool {
    FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .any(|sub| path.contains(sub))
}

pub(crate) fn collect_dag_files_tolerant(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_cargo_target_output_dir(dir, &path) {
                continue;
            }
            collect_dag_files_tolerant(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

fn scan_test_decl_names(content: &str) -> Vec<String> {
    scan_test_decl_lines(content)
        .into_iter()
        .map(|(name, _line)| name)
        .collect()
}

fn scan_wire_contract_decl_names(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("data ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                continue;
            }
            let after_name = rest.get(name.len()..).unwrap_or("").trim_start();
            if after_name.starts_with(": CoproductWireContract")
                || after_name.starts_with(": VariantEncoding")
            {
                out.push(name);
            }
        }
    }
    out
}

struct SidecarPlacementRule {
    required_suffix: &'static str,
    decl_description: &'static str,
    scan: fn(&str) -> Vec<String>,
    emit_discovery: bool,
}

const SIDECAR_PLACEMENT_RULES: &[SidecarPlacementRule] = &[
    SidecarPlacementRule {
        required_suffix: "_test.dag",
        decl_description: "`test`-marked decls",
        scan: scan_test_decl_names,
        emit_discovery: true,
    },
    SidecarPlacementRule {
        required_suffix: "_contracts.dag",
        decl_description:
            "wire-contract decls (`CoproductWireContract` and `VariantEncoding` data items)",
        scan: scan_wire_contract_decl_names,
        emit_discovery: false,
    },
];

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

pub fn discover_floor_corpus_rows(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
) -> Result<Vec<DiscoveryRow>, String> {
    discover_floor_corpus_rows_inner(source_roots, scan_dirs, exclude_substrings, &[])
}

pub fn discover_floor_corpus_rows_scoped(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
) -> Result<Vec<DiscoveryRow>, String> {
    discover_floor_corpus_rows_inner(
        source_roots,
        scan_dirs,
        exclude_substrings,
        discovery_scope_dirs,
    )
}

struct FloorLensHygieneGraph {
    rows: Vec<DiscoveryRow>,
    path_imports: std::collections::HashMap<String, Vec<String>>,
    module_to_path: std::collections::HashMap<String, String>,
    lens_with_justification: std::collections::BTreeSet<String>,
}

fn build_floor_lens_hygiene_graph(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
) -> Result<FloorLensHygieneGraph, String> {
    let excludes: Vec<String> = exclude_substrings.to_vec();
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

    let mut sidecar_violations: Vec<Vec<String>> =
        SIDECAR_PLACEMENT_RULES.iter().map(|_| Vec::new()).collect();
    let mut path_imports: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut module_to_path: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
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
            let rel = repo_relative_dag_path(&entry);
            if let Some(m) = extract_module_path(&content) {
                if is_top_level_lens_module(&m) && declares_construction_justification(&content) {
                    lens_with_justification.insert(m.clone());
                }
                module_to_path.insert(m, rel.clone());
            }
            path_imports.insert(rel, extract_import_paths(&content));
            if excludes.iter().any(|sub| entry.contains(sub.as_str())) {
                continue;
            }
            if !discovery_scope_dirs.is_empty()
                && !discovery_scope_dirs
                    .iter()
                    .any(|d| entry.contains(d.as_str()))
            {
                continue;
            }
            let rule_decls: Vec<Vec<String>> = SIDECAR_PLACEMENT_RULES
                .iter()
                .map(|rule| (rule.scan)(&content))
                .collect();
            for (i, (rule, names)) in SIDECAR_PLACEMENT_RULES
                .iter()
                .zip(rule_decls.iter())
                .enumerate()
            {
                if !names.is_empty() && !entry.ends_with(rule.required_suffix) {
                    sidecar_violations[i].push(entry.clone());
                }
                if rule.emit_discovery && entry.ends_with(rule.required_suffix) {
                    for name in names {
                        if seen.insert((entry.clone(), name.clone())) {
                            rows.push(DiscoveryRow {
                                label: name.clone(),
                                entry: entry.clone(),
                                function: name.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    for (rule, violations) in SIDECAR_PLACEMENT_RULES
        .iter()
        .zip(sidecar_violations.iter())
    {
        if !violations.is_empty() {
            let mut sorted = violations.clone();
            sorted.sort();
            return Err(format!(
                "{} must live in `*{}` files; found in: {}",
                rule.decl_description,
                rule.required_suffix,
                sorted.join(", ")
            ));
        }
    }
    rows.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    Ok(FloorLensHygieneGraph {
        rows,
        path_imports,
        module_to_path,
        lens_with_justification,
    })
}

fn default_floor_lens_hygiene_excludes() -> Vec<String> {
    FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Floor witness builtin (#5433 sibling to `doc_graph_orphan_count`): unreached top-level
/// `v2.lens.*` module count. Returns `-1` when the corpus walk fails closed.
pub fn inert_lens_unreached_module_count() -> i64 {
    match build_floor_lens_hygiene_graph(
        &default_source_roots(),
        &witness_discovery_scan_dirs(),
        &default_floor_lens_hygiene_excludes(),
        &[],
    ) {
        Ok(graph) => {
            inert_lens_modules(&graph.rows, &graph.path_imports, &graph.module_to_path).len() as i64
        }
        Err(_) => -1,
    }
}

/// Floor witness builtin: declared top-level `v2.lens.*` module count (non-vacuity oracle).
pub fn inert_lens_top_level_module_count() -> i64 {
    match build_floor_lens_hygiene_graph(
        &default_source_roots(),
        &witness_discovery_scan_dirs(),
        &default_floor_lens_hygiene_excludes(),
        &[],
    ) {
        Ok(graph) => graph
            .module_to_path
            .keys()
            .filter(|m| is_top_level_lens_module(m))
            .count() as i64,
        Err(_) => -1,
    }
}

fn discover_floor_corpus_rows_inner(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
) -> Result<Vec<DiscoveryRow>, String> {
    let graph = build_floor_lens_hygiene_graph(
        source_roots,
        scan_dirs,
        exclude_substrings,
        discovery_scope_dirs,
    )?;
    let FloorLensHygieneGraph {
        rows,
        path_imports,
        module_to_path,
        lens_with_justification,
    } = graph;
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
    let unjustified = unjustified_lens_modules(&module_to_path, &lens_with_justification);
    if !unjustified.is_empty() {
        return Err(format!(
            "construction-justification (DESIGN.md §5/§6): {} lens module(s) under `v2.lens.*` do \
             not record a `construction_justification` — before adding a lens you must justify why \
             the bad-state class cannot be made unwritable by construction. Add a `data \
             construction_justification: ConstructionJustification = …` decl (see \
             v2.lens.common.construction_justification) classifying it as WallNow / \
             WallAfterGrounding / RatchetForever: {}",
            unjustified.len(),
            unjustified.join(", ")
        ));
    }
    Ok(rows)
}

fn declares_construction_justification(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("data construction_justification")
            && trimmed.contains("ConstructionJustification")
    })
}

// ITEM 2 (reference grounding): the construction->authority graph witness.
//
// `WallNow.construction` was free-text prose; it is now
// `WallNow { mechanism: ConstructionMechanism, authority: DeclarationRef }`, so "this
// lens chains to a real construction" becomes a WALKABLE graph property: every WallNow
// authority must resolve to a real top-level decl in the corpus. The witness below proves
// that graph is TOTAL and goes RED if any binding dangles.
//
// SCAFFOLD (DESIGN §6): resolution is done HOST-SIDE here (extract authority refs from
// source + the kind-agnostic `extract_top_level_decls` over the resolved module), standing
// in for a not-yet-exposed unified .dag decl-resolution primitive. It keys on identity
// (module_path + decl_name), kind-agnostically — NOT a per-kind union (no fn-index ∪
// type-index fork). Dissolve-on: item (ii) "unified kind-agnostic decl-resolution authority
// exposed to .dag" (coordinator-tracked, resolver/spine lane) — when it lands, this witness
// re-expresses over the .dag primitive and the host-side resolution is deleted.

/// Extract every `authority: DeclarationRef { module_path: "..", decl_name: ".." }` in a
/// source file as `(module_path, decl_name)`. The field name `authority` typed
/// `DeclarationRef` is unique to `WallNow`, so this captures exactly the WallNow authorities.
pub fn wall_now_authority_refs(content: &str) -> Vec<(String, String)> {
    const NEEDLE: &str = "authority: DeclarationRef {";
    let mut out = Vec::new();
    let mut rest = content;
    while let Some(pos) = rest.find(NEEDLE) {
        let after = &rest[pos + NEEDLE.len()..];
        if let (Some(mp), Some(dn)) = (
            quoted_field_value(after, "module_path:"),
            quoted_field_value(after, "decl_name:"),
        ) {
            out.push((mp, dn));
        }
        rest = after;
    }
    out
}

/// The string literal following `<field>` (the next `"..."`), whitespace/newline tolerant.
fn quoted_field_value(s: &str, field: &str) -> Option<String> {
    let start = s.find(field)? + field.len();
    let rest = &s[start..];
    let open = rest.find('"')? + 1;
    let tail = &rest[open..];
    let close = tail.find('"')?;
    Some(tail[..close].to_string())
}

/// Resolve WallNow authorities against the kind-agnostic top-level decl table of their
/// declaring module. Returns the unresolved refs as `(declaring_file, module_path, decl_name)`;
/// empty = the construction->authority graph is TOTAL.
pub fn construction_authority_unresolved(
    module_to_content: &std::collections::HashMap<String, String>,
    authorities: &[(String, String, String)],
) -> Vec<(String, String, String)> {
    authorities
        .iter()
        .filter(|(_, module_path, decl_name)| {
            !module_to_content.get(module_path).is_some_and(|content| {
                extract_top_level_decls(content)
                    .iter()
                    .any(|(name, _)| name == decl_name)
            })
        })
        .cloned()
        .collect()
}

/// Walk the corpus, collect WallNow authorities + the module->source map, and return the
/// unresolved refs (empty = total). The live driver behind the witness test.
pub fn construction_authority_graph_unresolved(
    source_roots: &[String],
) -> Result<Vec<(String, String, String)>, String> {
    let mut module_to_content: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut authorities: Vec<(String, String, String)> = Vec::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(Path::new(root), &mut dag_files);
        dag_files.sort();
        for path in dag_files {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            let file = path.to_string_lossy().into_owned();
            for (module_path, decl_name) in wall_now_authority_refs(&content) {
                authorities.push((file.clone(), module_path, decl_name));
            }
            if let Some(m) = extract_module_path(&content) {
                module_to_content.insert(m, content);
            }
        }
    }
    Ok(construction_authority_unresolved(
        &module_to_content,
        &authorities,
    ))
}

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

fn is_top_level_lens_module(module: &str) -> bool {
    match module.strip_prefix("v2.lens.") {
        Some(rest) => !rest.is_empty() && !rest.contains('.'),
        None => false,
    }
}

fn inert_lens_modules(
    rows: &[DiscoveryRow],
    path_imports: &std::collections::HashMap<String, Vec<String>>,
    module_to_path: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut reached: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();
    let path_to_module: std::collections::HashMap<&String, &String> =
        module_to_path.iter().map(|(m, p)| (p, m)).collect();
    // Seed reachability from ALL *_test.dag files found in the source tree (not
    // just enrolled rows), so that witnesses in the execution corpus also count
    // for lens coverage even though they are excluded from the main corpus rows.
    let entry_paths: std::collections::BTreeSet<String> = {
        let mut s: std::collections::BTreeSet<String> = rows
            .iter()
            .map(|r| repo_relative_dag_path(&r.entry))
            .collect();
        for path in path_imports.keys() {
            if path.ends_with("_test.dag") {
                s.insert(path.clone());
            }
        }
        s
    };
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

pub struct DiscoveryCorpusOptions {
    pub skip_unaffected_node_frontier: bool,
    pub explicit_roster_only: bool,
    /// Path-substring exclusion list. Non-plan callers default to FLOOR_DISCOVERY_EXCLUDES;
    /// plan-driven paths supply this from RunnableDiscoveryBatch.exclude_substrings (the model authority).
    pub exclude_substrings: Vec<String>,
    /// When non-empty, scopes the source-root `test fn` tree walk to files under one of these
    /// directories. Import resolution still uses the full source_roots. Empty = full walk.
    pub discovery_scope_dirs: Vec<String>,
    /// When > 0, caps the effective spawn_width for this discovery corpus to this value.
    /// Derived from RunnableDiscoveryBatch.spawn_width_cap (provisioned from runner alloc ÷
    /// per-witness memory reservation). 0 = use the batch-wide spawn_width.
    pub spawn_width_cap: usize,
}

impl Default for DiscoveryCorpusOptions {
    fn default() -> Self {
        Self {
            skip_unaffected_node_frontier: false,
            explicit_roster_only: false,
            exclude_substrings: FLOOR_DISCOVERY_EXCLUDES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            discovery_scope_dirs: vec![],
            spawn_width_cap: 0,
        }
    }
}

enum FloorGitDiffOutcome {
    ObservationFailClosed { reason: String },
    UnifiedProduced(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileLineRange {
    start: i64,
    end: i64,
}

fn floor_git_diff_range() -> Result<String, String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_git_diff_unified_for_ci", false)
            .map_err(|e| format!("floor_observe_git_diff_unified_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffOk") => match ctx.field(fields, "text") {
            Some(Value::Str(s)) => Ok(s.clone()),
            _ => Err("UnifiedDiffOk missing `text` field".to_string()),
        },
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffFail") => match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Err(r.clone()),
            _ => Err("git diff observation failed (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_git_diff_unified_for_ci returned `{}`, expected FloorUnifiedDiffResult",
            ctx.format_value(other)
        )),
    }
}

fn normalize_repo_path(path: &str) -> String {
    path.strip_prefix("./").unwrap_or(path).replace('\\', "/")
}

fn diff_file_matches_entry(diff_file: &str, entry_path: &str) -> bool {
    let file = normalize_repo_path(diff_file);
    let entry = normalize_repo_path(entry_path);
    file == entry || entry.ends_with(&file) || file.ends_with(&entry)
}

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

fn parse_unified_diff_changed_new_lines(diff_text: &str) -> HashMap<String, HashSet<i64>> {
    let mut out: HashMap<String, HashSet<i64>> = HashMap::new();
    let mut current_file: Option<String> = None;
    let mut new_line: i64 = 0;
    let mut in_hunk = false;
    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            current_file = Some(normalize_repo_path(rest));
            in_hunk = false;
            continue;
        }
        if line.starts_with("@@ ") {
            let plus = line.split_whitespace().nth(2).unwrap_or("");
            let plus = plus.trim_start_matches('+');
            new_line = if let Some((s, _)) = plus.split_once(',') {
                s.parse::<i64>().unwrap_or(1)
            } else {
                plus.parse::<i64>().unwrap_or(1)
            };
            in_hunk = true;
            continue;
        }
        if !in_hunk {
            continue;
        }
        let Some(file) = current_file.clone() else {
            continue;
        };
        if let Some(_add) = line.strip_prefix('+') {
            out.entry(file.clone()).or_default().insert(new_line);
            new_line += 1;
        } else if line.starts_with('-') {
            // Pure deletions advance only the old-file cursor; attribute at the new-file
            // position where the removal occurred (same line for consecutive `-` rows).
            out.entry(file).or_default().insert(new_line);
        } else if line.starts_with(' ') {
            new_line += 1;
        }
    }
    out
}

fn changed_new_lines_for_file(
    changed_new_lines_by_file: &HashMap<String, HashSet<i64>>,
    file_path: &str,
    file_norm: &str,
) -> HashSet<i64> {
    changed_new_lines_by_file
        .get(file_norm)
        .or_else(|| changed_new_lines_by_file.get(file_path))
        .cloned()
        .unwrap_or_default()
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

fn span_file_matches(span_file: &str, target_norm: &str) -> bool {
    let s = normalize_repo_path(span_file);
    s == target_norm || s.ends_with(target_norm) || target_norm.ends_with(&s)
}

fn import_closure_files_from_graph(graph: &v1_compiler_compile::ResolvedGraph) -> HashSet<String> {
    let mut files = HashSet::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            files.insert(normalize_repo_path(&item.span.file));
        }
    }
    files
}

fn touched_file_in_import_closure(touched_file: &str, closure_files: &HashSet<String>) -> bool {
    let norm = normalize_repo_path(touched_file);
    closure_files
        .iter()
        .any(|closure_file| span_file_matches(closure_file, &norm))
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

fn value_is_node(val: &v1_interpreter::Value, ctx: &v1_interpreter::InterpContext) -> bool {
    matches!(
        val,
        v1_interpreter::Value::Record { type_name, .. } if ctx.resolve(*type_name).as_str() == "Node"
    )
}

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
            for (_, v) in fields.iter() {
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

#[derive(Clone, Default)]
struct NodeFrontierSeeds {
    overlapping_data_items: HashSet<(String, String)>,
    edited_test_fns: HashSet<(String, String)>,
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

fn decl_span_end_line(sorted_decl_lines: &[i64], decl_line: i64) -> i64 {
    sorted_decl_lines
        .iter()
        .position(|&line| line == decl_line)
        .map(|idx| {
            sorted_decl_lines
                .get(idx + 1)
                .map(|&next| next - 1)
                .unwrap_or(i64::MAX)
        })
        .unwrap_or(i64::MAX)
}

fn collect_sorted_decl_lines_for_file(
    index: &MultiEntryIndex,
    file_path: &str,
) -> Result<Vec<i64>, String> {
    let file_norm = normalize_repo_path(file_path);
    let (graph, source_indices) = resolve_entry_with_index(index, file_path)?;
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("read {file_path} for decl span: {e}"))?;
    let mut decls: Vec<i64> = Vec::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            if !span_file_matches(&item.span.file, &file_norm) {
                continue;
            }
            let Some(nl) = newline_index_for_span(&item.span, &source_indices).cloned() else {
                return Err(format!(
                    "newline index missing for decl span in {file_path}"
                ));
            };
            decls.push(byte_to_line_col(nl, item.span.start).line);
        }
    }
    for (_, line) in scan_test_decl_lines(&content) {
        if !decls.contains(&line) {
            decls.push(line);
        }
    }
    decls.sort_unstable();
    Ok(decls)
}

fn collect_frontier_seeds_from_diff_line_ranges(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
) -> Result<NodeFrontierSeeds, String> {
    let mut overlapping_data_items = HashSet::new();
    let mut edited_test_fns = HashSet::new();
    for (file_path, ranges) in line_ranges_by_file {
        if !file_path.ends_with(".dag") {
            return Ok(NodeFrontierSeeds::run_all());
        }
        let file_norm = normalize_repo_path(file_path);
        let (graph, source_indices) = match resolve_entry_with_index(index, file_path) {
            Ok(pair) => pair,
            Err(_) => return Ok(NodeFrontierSeeds::run_all()),
        };
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Ok(NodeFrontierSeeds::run_all()),
        };
        let test_fn_names: HashSet<String> = scan_test_decl_names(&content).into_iter().collect();
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
        if ranges.iter().any(|r| r.start < decls[0].0) {
            return Ok(NodeFrontierSeeds::run_all());
        }
        for i in 0..decls.len() {
            let (line, name, is_data) = &decls[i];
            let decl_end =
                decl_span_end_line(&decls.iter().map(|(l, _, _)| *l).collect::<Vec<_>>(), *line);
            if !ranges.iter().any(|r| *line <= r.end && decl_end >= r.start) {
                continue;
            }
            if test_fn_names.contains(name) {
                edited_test_fns.insert((file_norm.clone(), name.clone()));
            } else if *is_data {
                overlapping_data_items.insert((file_norm.clone(), name.clone()));
            } else {
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

fn entry_frontier_nodes_from_seeds(
    ctx: &v1_interpreter::InterpContext,
    entry_path: &str,
    seeds: &NodeFrontierSeeds,
) -> Result<Vec<v1_interpreter::Value>, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for (_file, name) in &seeds.overlapping_data_items {
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

fn entry_touches_frontier_seeds(
    ctx: &v1_interpreter::InterpContext,
    entry_path: &str,
    seeds: &NodeFrontierSeeds,
) -> Result<bool, String> {
    let entry_frontier = entry_frontier_nodes_from_seeds(ctx, entry_path, seeds)?;
    if entry_frontier.is_empty() {
        return Ok(false);
    }
    entry_touches_rerun_frontier(ctx, &list_value_from_vec(entry_frontier))
}

// SCAFFOLD (DESIGN §6–§7): host-side diff→declaration attribution
// (`floor_diff_edits_from_line_ranges`) and per-entry frontier materialization
// (`rerun_frontier_nodes_for_entry`, `entry_touches_rerun_frontier`) are Implementation 2
// in docs/plans/affected-set-precompute-pruning.md — parallel to `v2.lens.affected_set`
// until ROADMAP `1-affected-set-defork` Steps 3–5 land. Skip/precompute **verdicts** already
// read `.dag` via `floor_kernel_would_skip` / `floor_kernel_precompute_would_skip`; this block
// is the I/O + frontier-input bridge only, not a second skip policy.
// Dissolve-on: `affected_set_reading_from_git_diff_provenance` + floor-runtime provenance ingest
// expose edit-locus → delete `floor_diff_edits_from_line_ranges`, `rerun_frontier_nodes_for_entry`,
// `entry_touches_rerun_frontier`, and the `resolve_floor_runner_context` host wrappers (census:
// `rg 'floor_diff_edits_from_line_ranges|rerun_frontier_nodes_for_entry' src/v1/stage0/src/cli_run.rs`
// must be empty). `entry_file_touched` is marshaled from the entry's transitive import-closure
// (not entry-path equality alone) so cross-file helper-fn edits fail-closed for importers until
// the real `v2.lens.affected_set` fn axis lands (ROADMAP `1-affected-set-defork` Step 5).
//
// Host-side diff→declaration attribution only (line-range I/O). Skip verdicts live in
// `v2.workflow.affected_set_floor_runner` — the executor reads `.dag`, never recomputes frontier.
#[derive(Clone, Debug, Default)]
struct FloorDiffEdits {
    overlapping_data_items: HashSet<(String, String)>,
    edited_test_fns: HashSet<(String, String)>,
    /// `.dag` files with a non-data, non-test-fn declaration touched — run that entry's roster.
    touched_entry_files: HashSet<String>,
}

const FLOOR_RUNNER_ENTRY: &str = "src/v2/workflow/affected_set_floor_runner.dag";
// Keep in sync with `floor_host_scaffold_witness_marker` in affected_set_floor_runner.dag.
const FLOOR_HOST_SCAFFOLD_WITNESS_MARKER: &str = "floor:host_scaffold";

fn resolve_floor_runner_context(
    source_roots: &[String],
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    resolve_entry_graph(source_roots, FLOOR_RUNNER_ENTRY)
}

fn call_floor_kernel_would_skip(
    ctx: &v1_interpreter::InterpContext,
    changed_paths: &[String],
    frontier_nodes: &[v1_interpreter::Value],
    touches_frontier: bool,
    function_edited: bool,
    entry_file_touched: bool,
) -> Result<bool, String> {
    if !ctx.item_registry.contains_key("floor_kernel_would_skip") {
        return Err("floor_kernel_would_skip missing from floor runner context".to_string());
    }
    let paths: Vec<v1_interpreter::Value> = changed_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("changed_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("frontier_nodes".to_string()),
            list_value_from_vec(frontier_nodes.to_vec()),
        ),
        (
            Some("touches_frontier".to_string()),
            v1_interpreter::Value::Bool(touches_frontier),
        ),
        (
            Some("function_edited".to_string()),
            v1_interpreter::Value::Bool(function_edited),
        ),
        (
            Some("entry_file_touched".to_string()),
            v1_interpreter::Value::Bool(entry_file_touched),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(ctx, "floor_kernel_would_skip", &args, false) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "floor_kernel_would_skip returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("floor_kernel_would_skip: {e}")),
    }
}

fn call_floor_kernel_precompute_would_skip(
    ctx: &v1_interpreter::InterpContext,
    changed_paths: &[String],
    frontier_node_count: usize,
    edited_test_fn_count: usize,
    touched_entry_file_count: usize,
) -> Result<bool, String> {
    if !ctx
        .item_registry
        .contains_key("floor_kernel_precompute_would_skip")
    {
        return Err(
            "floor_kernel_precompute_would_skip missing from floor runner context".to_string(),
        );
    }
    let paths: Vec<v1_interpreter::Value> = changed_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("changed_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("frontier_node_count".to_string()),
            v1_interpreter::Value::Int(frontier_node_count as i64),
        ),
        (
            Some("edited_test_fn_count".to_string()),
            v1_interpreter::Value::Int(edited_test_fn_count as i64),
        ),
        (
            Some("touched_entry_file_count".to_string()),
            v1_interpreter::Value::Int(touched_entry_file_count as i64),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(
        ctx,
        "floor_kernel_precompute_would_skip",
        &args,
        false,
    ) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "floor_kernel_precompute_would_skip returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("floor_kernel_precompute_would_skip: {e}")),
    }
}

fn call_floor_host_scaffold_would_skip(
    ctx: &v1_interpreter::InterpContext,
    changed_paths: &[String],
    frontier_nodes: &[v1_interpreter::Value],
    touches_frontier: bool,
    function_edited: bool,
    entry_file_touched: bool,
) -> Result<bool, String> {
    if !ctx
        .item_registry
        .contains_key("floor_host_scaffold_would_skip")
    {
        return Err("floor_host_scaffold_would_skip missing from floor runner context".to_string());
    }
    let paths: Vec<v1_interpreter::Value> = changed_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("changed_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("frontier_nodes".to_string()),
            list_value_from_vec(frontier_nodes.to_vec()),
        ),
        (
            Some("touches_frontier".to_string()),
            v1_interpreter::Value::Bool(touches_frontier),
        ),
        (
            Some("function_edited".to_string()),
            v1_interpreter::Value::Bool(function_edited),
        ),
        (
            Some("entry_file_touched".to_string()),
            v1_interpreter::Value::Bool(entry_file_touched),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(
        ctx,
        "floor_host_scaffold_would_skip",
        &args,
        false,
    ) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "floor_host_scaffold_would_skip returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("floor_host_scaffold_would_skip: {e}")),
    }
}

fn call_floor_host_scaffold_precompute_would_skip(
    ctx: &v1_interpreter::InterpContext,
    changed_paths: &[String],
    frontier_node_count: usize,
    edited_test_fn_count: usize,
    touched_entry_file_count: usize,
) -> Result<bool, String> {
    if !ctx
        .item_registry
        .contains_key("floor_host_scaffold_precompute_would_skip")
    {
        return Err(
            "floor_host_scaffold_precompute_would_skip missing from floor runner context"
                .to_string(),
        );
    }
    let paths: Vec<v1_interpreter::Value> = changed_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("changed_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("frontier_node_count".to_string()),
            v1_interpreter::Value::Int(frontier_node_count as i64),
        ),
        (
            Some("edited_test_fn_count".to_string()),
            v1_interpreter::Value::Int(edited_test_fn_count as i64),
        ),
        (
            Some("touched_entry_file_count".to_string()),
            v1_interpreter::Value::Int(touched_entry_file_count as i64),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(
        ctx,
        "floor_host_scaffold_precompute_would_skip",
        &args,
        false,
    ) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "floor_host_scaffold_precompute_would_skip returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("floor_host_scaffold_precompute_would_skip: {e}")),
    }
}

fn entry_text_indicates_live_host_scan(text: &str) -> bool {
    text.contains(FLOOR_HOST_SCAFFOLD_WITNESS_MARKER)
        || text.contains("layer_import_facts")
        || text.contains("_live(")
        || text.contains("_facts_live(")
}

fn witness_test_fn_uses_live_host_scan(entry_content: &str, function: &str) -> bool {
    // Fail-closed (file-wide): any live-tree scan signal anywhere in the entry
    // classifies every witness in that file as host-scaffold — nested helper
    // chains (test → helper_a → helper_b → _live) cannot fall back to kernel-skip.
    if entry_text_indicates_live_host_scan(entry_content) {
        return true;
    }
    let decl_needle = format!("test fn {function}");
    if let Some(start) = entry_content.find(&decl_needle) {
        let decl_tail =
            &entry_content[start..entry_content.len().min(start + decl_needle.len() + 120)];
        if decl_tail.contains(FLOOR_HOST_SCAFFOLD_WITNESS_MARKER) {
            return true;
        }
    }
    false
}

fn read_entry_content_for_host_scaffold(entry: &str) -> (String, bool) {
    match std::fs::read_to_string(entry) {
        Ok(content) => (content, false),
        Err(e) => {
            eprintln!(
                "claim_executor: failed to read entry {entry} for host-scaffold classification ({e}) — fail-closed, treating as host-scaffold"
            );
            (String::new(), true)
        }
    }
}

fn discovery_rows_include_host_scaffold(rows: &[DiscoveryRow]) -> bool {
    rows.iter().any(|row| {
        let (content, read_failed) = read_entry_content_for_host_scaffold(&row.entry);
        read_failed || witness_test_fn_uses_live_host_scan(&content, &row.function)
    })
}

fn floor_diff_edits_from_diff_text(
    index: &MultiEntryIndex,
    diff_text: &str,
) -> Result<FloorDiffEdits, String> {
    let line_ranges = parse_unified_diff_line_ranges(diff_text);
    let changed = parse_unified_diff_changed_new_lines(diff_text);
    floor_diff_edits_from_line_ranges(index, &line_ranges, &changed)
}

fn floor_diff_edits_from_line_ranges(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
    changed_new_lines_by_file: &HashMap<String, HashSet<i64>>,
) -> Result<FloorDiffEdits, String> {
    let mut overlapping_data_items = HashSet::new();
    let mut edited_test_fns = HashSet::new();
    let mut touched_entry_files = HashSet::new();
    let mut saw_non_dag = false;
    let mut saw_dag = false;
    for (file_path, ranges) in line_ranges_by_file {
        if !file_path.ends_with(".dag") {
            saw_non_dag = true;
            continue;
        }
        saw_dag = true;
        let file_norm = normalize_repo_path(file_path);
        let (graph, source_indices) = match resolve_entry_with_index(index, file_path) {
            Ok(pair) => pair,
            Err(e) => return Err(format!("resolve failed for {file_path}: {e}")),
        };
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => return Err(format!("read failed for {file_path}: {e}")),
        };
        let test_fn_names: HashSet<String> = scan_test_decl_names(&content).into_iter().collect();
        let mut decls: Vec<(i64, String, bool)> = Vec::new();
        for module in graph.modules.iter() {
            for item in module.items.iter() {
                if !span_file_matches(&item.span.file, &file_norm) {
                    continue;
                }
                let Some(nl) = newline_index_for_span(&item.span, &source_indices).cloned() else {
                    return Err(format!(
                        "newline index missing for declaration in {file_path}"
                    ));
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
            return Err(format!("no declarations in {file_path}"));
        }
        decls.sort_by_key(|(line, _, _)| *line);
        let first_decl_line = decls[0].0;
        let mut changed =
            changed_new_lines_for_file(changed_new_lines_by_file, file_path, &file_norm);
        // Deletion-only hunks (`-` rows, zero `+` width) still carry a new-side anchor in the
        // hunk header; fall back to parsed ranges when no `+`/`-` rows were attributed.
        if changed.is_empty() {
            for r in ranges {
                let end = if r.end < r.start { r.start } else { r.end };
                for l in r.start..=end {
                    changed.insert(l);
                }
            }
        }
        // Module-line edits (line 1) stay fail-closed — renaming can change entry identity.
        if changed.contains(&1) {
            return Err(format!("diff before first declaration in {file_path}"));
        }
        let has_pre_decl = changed.iter().any(|&l| l < first_decl_line);
        let has_post_decl = changed.iter().any(|&l| l >= first_decl_line);
        if has_pre_decl {
            touched_entry_files.insert(file_norm.clone());
            if !has_post_decl {
                continue;
            }
        }
        for i in 0..decls.len() {
            let (line, name, is_data) = &decls[i];
            let decl_end = decls.get(i + 1).map(|(l, _, _)| l - 1).unwrap_or(i64::MAX);
            if !changed.iter().any(|&l| l >= *line && l <= decl_end) {
                continue;
            }
            if test_fn_names.contains(name) {
                edited_test_fns.insert((file_norm.clone(), name.clone()));
            } else if *is_data {
                overlapping_data_items.insert((file_norm.clone(), name.clone()));
            } else {
                touched_entry_files.insert(file_norm.clone());
            }
        }
    }
    if saw_non_dag && !saw_dag {
        return Err("non-.dag file changed with no .dag paths in diff".to_string());
    }
    Ok(FloorDiffEdits {
        overlapping_data_items,
        edited_test_fns,
        touched_entry_files,
    })
}

fn rerun_frontier_nodes_for_entry(
    ctx: &v1_interpreter::InterpContext,
    entry_path: &str,
    edits: &FloorDiffEdits,
) -> Result<Vec<v1_interpreter::Value>, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for (_file, name) in &edits.overlapping_data_items {
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

fn entry_touches_rerun_frontier(
    ctx: &v1_interpreter::InterpContext,
    frontier: &v1_interpreter::Value,
) -> Result<bool, String> {
    let mut saw_claim = false;
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
    Ok(!saw_claim)
}

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
    let mut rows =
        if options.explicit_roster_only || (scan_dirs.is_empty() && !explicit_entries.is_empty()) {
            Vec::new()
        } else {
            discover_floor_corpus_rows_scoped(
                source_roots,
                scan_dirs,
                &options.exclude_substrings,
                &options.discovery_scope_dirs,
            )?
        };
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
    set_phase(FloorPhase::Discovery, "discovery-roster");
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
    let line_ranges_by_file = match &diff_outcome {
        FloorGitDiffOutcome::ObservationFailClosed { .. } => HashMap::new(),
        FloorGitDiffOutcome::UnifiedProduced(text) => parse_unified_diff_line_ranges(text),
    };
    let changed_new_lines_by_file = match &diff_outcome {
        FloorGitDiffOutcome::ObservationFailClosed { .. } => HashMap::new(),
        FloorGitDiffOutcome::UnifiedProduced(text) => parse_unified_diff_changed_new_lines(text),
    };
    let changed_paths: Vec<String> = line_ranges_by_file.keys().cloned().collect();
    let (skip_enabled, diff_edits) = if options.skip_unaffected_node_frontier
        && !line_ranges_by_file.is_empty()
    {
        let frontier_index = build_multi_entry_index(source_roots);
        match floor_diff_edits_from_line_ranges(
            &frontier_index,
            &line_ranges_by_file,
            &changed_new_lines_by_file,
        ) {
            Ok(edits) => (true, edits),
            Err(msg) => {
                eprintln!(
                    "claim_executor: node-frontier population fail-closed ({msg}) — running full corpus"
                );
                (false, FloorDiffEdits::default())
            }
        }
    } else {
        (false, FloorDiffEdits::default())
    };
    let floor_runner_ctx = if options.skip_unaffected_node_frontier {
        match resolve_floor_runner_context(source_roots) {
            Ok((graph, source_indices)) => {
                Some(make_eval_context(&graph, source_indices, execution_mode))
            }
            Err(msg) => {
                eprintln!(
                    "claim_executor: floor runner resolve failed ({msg}) — fail-closed, running full corpus"
                );
                None
            }
        }
    } else {
        None
    };
    let skip_precompute = if skip_enabled {
        let host_scaffold_corpus = discovery_rows_include_host_scaffold(&rows);
        match floor_runner_ctx.as_ref() {
            Some(ctx) => {
                let precompute = if host_scaffold_corpus {
                    call_floor_host_scaffold_precompute_would_skip(
                        ctx,
                        &changed_paths,
                        diff_edits.overlapping_data_items.len(),
                        diff_edits.edited_test_fns.len(),
                        diff_edits.touched_entry_files.len(),
                    )
                } else {
                    call_floor_kernel_precompute_would_skip(
                        ctx,
                        &changed_paths,
                        diff_edits.overlapping_data_items.len(),
                        diff_edits.edited_test_fns.len(),
                        diff_edits.touched_entry_files.len(),
                    )
                };
                match precompute {
                    Ok(skip) => skip,
                    Err(msg) => {
                        eprintln!(
                            "claim_executor: floor precompute_would_skip failed ({msg}) — fail-closed, running precompute"
                        );
                        false
                    }
                }
            }
            None => false,
        }
    } else {
        false
    };
    let whole_tree_published_keys = if skip_precompute {
        eprintln!(
            "run_discovery_corpus: skipping whole-tree published-mock precompute (scoped diff, empty node frontier, no edited test fns, no entry-file fn edits)"
        );
        None
    } else {
        match precompute_whole_tree_published_mock_keys(source_roots) {
            Ok(keys) if keys.is_empty() => None,
            Ok(keys) => Some(keys),
            Err(e) => {
                return Err(format!(
                    "whole-tree published mock corpus precompute failed: {e}"
                ));
            }
        }
    };
    let index = build_multi_entry_index(source_roots);

    let capped_width = if options.spawn_width_cap > 0 {
        parallel_width.min(options.spawn_width_cap)
    } else {
        parallel_width
    };
    let width = capped_width.max(1);
    if width == 1 {
        return run_discovery_rows(
            &rows,
            &index,
            execution_mode,
            skip_enabled,
            &changed_paths,
            &diff_edits,
            floor_runner_ctx.as_ref(),
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
    let skip_for_shards = skip_enabled;
    let mut handles = Vec::new();
    for shard in shards {
        if shard.is_empty() {
            continue;
        }
        let shard_rows: Vec<DiscoveryRow> = shard.iter().map(|&i| rows[i].clone()).collect();
        let roots = source_roots_owned.clone();
        let seeds = diff_edits.clone();
        let paths = changed_paths.clone();
        let keys = whole_tree_published_keys.clone();
        handles.push(std::thread::spawn(move || {
            let index = build_multi_entry_index(&roots);
            let runner = if skip_for_shards {
                match resolve_floor_runner_context(&roots) {
                    Ok((graph, source_indices)) => Some(make_eval_context(
                        &graph,
                        source_indices,
                        execution_mode,
                    )),
                    Err(msg) => {
                        eprintln!(
                            "claim_executor: floor runner resolve failed in shard ({msg}) — fail-closed, running all rows in shard"
                        );
                        None
                    }
                }
            } else {
                None
            };
            run_discovery_rows(
                &shard_rows,
                &index,
                execution_mode,
                skip_for_shards,
                &paths,
                &seeds,
                runner.as_ref(),
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

fn run_discovery_rows(
    rows: &[DiscoveryRow],
    index: &MultiEntryIndex,
    execution_mode: v1_interpreter::ExecutionMode,
    skip_enabled: bool,
    changed_paths: &[String],
    diff_edits: &FloorDiffEdits,
    floor_runner_ctx: Option<&v1_interpreter::InterpContext>,
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
    let mut current_entry_frontier_nodes: Vec<v1_interpreter::Value> = Vec::new();
    let mut current_entry_closure_files: HashSet<String> = HashSet::new();
    let mut current_entry_content: String = String::new();
    let mut current_entry_host_scaffold_fail_closed: bool = false;
    let whole_tree_published_keys = whole_tree_published_keys.map(Rc::new);
    for row in rows {
        if current_entry.as_deref() != Some(row.entry.as_str()) {
            let sources = load_sources_for_entry_with_index(
                &index.source_files,
                &index.module_graph_facts,
                &row.entry,
            )
            .map_err(|msg| format!("load sources failed for {}: {}", row.entry, msg))?;
            let closure_subject = subject_digest_for_closure(&sources);
            let resolve_started = std::time::Instant::now();
            set_phase(FloorPhase::Resolve, &row.entry);
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
            current_entry_closure_files = import_closure_files_from_graph(&graph);
            let entry_ctx = make_eval_context_with_runtime_options(
                &graph,
                source_indices,
                execution_mode,
                None,
                whole_tree_published_keys.clone(),
            );
            if skip_enabled {
                current_entry_frontier_nodes =
                    rerun_frontier_nodes_for_entry(&entry_ctx, &row.entry, diff_edits)?;
                current_entry_touches = if current_entry_frontier_nodes.is_empty() {
                    false
                } else {
                    entry_touches_rerun_frontier(
                        &entry_ctx,
                        &list_value_from_vec(current_entry_frontier_nodes.clone()),
                    )?
                };
            } else {
                current_entry_frontier_nodes.clear();
                current_entry_touches = true;
            }
            ctx = Some(entry_ctx);
            current_entry = Some(row.entry.clone());
            let (content, read_failed) = read_entry_content_for_host_scaffold(&row.entry);
            current_entry_content = content;
            current_entry_host_scaffold_fail_closed = read_failed;
        }
        let function_edited = skip_enabled
            && diff_edits.edited_test_fns.iter().any(|(file, func)| {
                diff_file_matches_entry(file, &row.entry) && func == &row.function
            });
        let entry_file_touched = skip_enabled
            && diff_edits
                .touched_entry_files
                .iter()
                .any(|file| touched_file_in_import_closure(file, &current_entry_closure_files));
        let should_skip = if skip_enabled {
            let host_scaffold_witness = current_entry_host_scaffold_fail_closed
                || witness_test_fn_uses_live_host_scan(&current_entry_content, &row.function);
            match floor_runner_ctx {
                Some(runner_ctx) => {
                    let skip = if host_scaffold_witness {
                        call_floor_host_scaffold_would_skip(
                            runner_ctx,
                            changed_paths,
                            &current_entry_frontier_nodes,
                            current_entry_touches,
                            function_edited,
                            entry_file_touched,
                        )
                    } else {
                        call_floor_kernel_would_skip(
                            runner_ctx,
                            changed_paths,
                            &current_entry_frontier_nodes,
                            current_entry_touches,
                            function_edited,
                            entry_file_touched,
                        )
                    };
                    match skip {
                        Ok(skip) => skip,
                        Err(msg) => {
                            eprintln!(
                                "claim_executor: floor would_skip failed ({msg}) — fail-closed, running {} ({})",
                                row.function, row.entry
                            );
                            false
                        }
                    }
                }
                None => false,
            }
        } else {
            false
        };
        if should_skip {
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
        set_phase(
            FloorPhase::Eval,
            &format!("{}::{}", row.entry, row.function),
        );
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
        build_multi_entry_index, entry_touches_rerun_frontier, floor_diff_edits_from_diff_text,
        list_value_from_vec, parse_unified_diff_changed_new_lines, parse_unified_diff_line_ranges,
        rerun_frontier_nodes_for_entry, scan_test_decl_lines, FileLineRange,
    };
    use crate::v1_compiler_infer_items::{item_kind, ItemKind, ResolvedGraph};
    use crate::v1_interpreter::ExecutionMode;
    use crate::v1_std_core::{authored_name_at, byte_to_line_col};
    use std::collections::{HashMap, HashSet};
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

    #[test]
    fn parse_unified_diff_changed_new_lines_includes_deletions() {
        let diff = "\
diff --git a/src/v2/lens/affected_set.dag b/src/v2/lens/affected_set.dag
--- a/src/v2/lens/affected_set.dag
+++ b/src/v2/lens/affected_set.dag
@@ -42,2 +42,0 @@
-removed_a
-removed_b
";
        let changed = parse_unified_diff_changed_new_lines(diff);
        let file = "src/v2/lens/affected_set.dag";
        assert_eq!(changed.get(file), Some(&HashSet::from([42])));
    }

    #[test]
    fn scan_test_decl_lines_pairs_names_with_1_based_lines() {
        let source = "module m\n\ndata d: Int = 1\n\ntest fn witness_a() -> Bool { true }\n\ntest data witness_b: Int = 2\n";
        let pairs = scan_test_decl_lines(source);
        assert_eq!(
            pairs,
            vec![("witness_a".to_string(), 5), ("witness_b".to_string(), 7)]
        );
    }

    #[test]
    fn witness_test_fn_uses_live_host_scan_detects_live_calls() {
        let source = "module m\n\ntest fn clean_tree_holds() -> Bool {\n  realization_vocab_containment_clean_live(scan_roots: roots)\n}\n\ntest fn pure_holds() -> Bool { true }\n";
        assert!(super::witness_test_fn_uses_live_host_scan(
            source,
            "clean_tree_holds"
        ));
        // File-wide fail-closed: sibling witnesses in the same entry also run.
        assert!(super::witness_test_fn_uses_live_host_scan(
            source,
            "pure_holds"
        ));
    }

    #[test]
    fn witness_test_fn_uses_live_host_scan_pure_entry_stays_kernel_eligible() {
        let source = "module m\n\ntest fn pure_holds() -> Bool { true }\n\ntest fn also_pure() -> Bool { false }\n";
        assert!(!super::witness_test_fn_uses_live_host_scan(
            source,
            "pure_holds"
        ));
        assert!(!super::witness_test_fn_uses_live_host_scan(
            source,
            "also_pure"
        ));
    }

    #[test]
    fn witness_test_fn_uses_live_host_scan_detects_declared_marker() {
        let source =
            "module m\n\ntest fn marked_holds() -> Bool { // floor:host_scaffold\n  true\n}\n";
        assert!(super::witness_test_fn_uses_live_host_scan(
            source,
            "marked_holds"
        ));
    }

    #[test]
    fn witness_test_fn_uses_live_host_scan_follows_same_file_helper() {
        let source = "module m\n\nfn helper_holds() -> Bool {\n  layer_import_facts(std_roots: [], extdeps_roots: [])\n}\n\ntest fn witness_holds() -> Bool {\n  helper_holds()\n}\n";
        assert!(super::witness_test_fn_uses_live_host_scan(
            source,
            "witness_holds"
        ));
    }

    #[test]
    fn witness_test_fn_uses_live_host_scan_follows_nested_same_file_helper() {
        let source = "module m\n\nfn helper_b() -> Bool {\n  realization_vocab_containment_clean_live(scan_roots: roots)\n}\n\nfn helper_a() -> Bool {\n  helper_b()\n}\n\ntest fn witness_holds() -> Bool {\n  helper_a()\n}\n";
        assert!(super::witness_test_fn_uses_live_host_scan(
            source,
            "witness_holds"
        ));
    }

    #[test]
    fn host_scaffold_witness_not_skipped_on_unrelated_diff() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ];
        let (runner_graph, runner_indices) =
            super::resolve_entry_graph(&roots, super::FLOOR_RUNNER_ENTRY)
                .expect("floor runner resolves");
        let runner_ctx =
            super::make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let entry = "src/v2/test/claim/realization_vocabulary_containment/clean_tree_test.dag";
        let content = std::fs::read_to_string(entry).expect("clean_tree readable");
        assert!(super::witness_test_fn_uses_live_host_scan(
            &content,
            "realization_vocab_clean_tree_holds"
        ));
        let changed_paths = vec!["src/v2/lens/affected_set.dag".to_string()];
        let skip = super::call_floor_host_scaffold_would_skip(
            &runner_ctx,
            &changed_paths,
            &[],
            false,
            false,
            false,
        )
        .expect("host scaffold skip");
        assert!(
            !skip,
            "host-scaffold witness must not skip on unrelated node-frontier diff"
        );
    }

    #[test]
    fn node_precise_same_file_referenced_vs_orphan_discriminates() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let fixture = fixture_path();
        let roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
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

        let referenced_diff = unified_diff_for_line(&fixture, referenced_line);
        let referenced_seeds = floor_diff_edits_from_diff_text(&index, &referenced_diff)
            .expect("frontier for referenced-node diff");
        assert!(
            entry_touches_rerun_frontier(
                &ctx,
                &list_value_from_vec(
                    rerun_frontier_nodes_for_entry(&ctx, &fixture, &referenced_seeds)
                        .expect("nodes")
                )
            )
            .expect("touch check (referenced)"),
            "a diff on a node some claim references must touch the entry (runs)"
        );

        let orphan_diff = unified_diff_for_line(&fixture, orphan_line);
        let orphan_seeds = floor_diff_edits_from_diff_text(&index, &orphan_diff)
            .expect("frontier for orphan-node diff");
        let orphan_nodes =
            rerun_frontier_nodes_for_entry(&ctx, &fixture, &orphan_seeds).expect("nodes");
        assert!(
            orphan_nodes.is_empty()
                || !entry_touches_rerun_frontier(&ctx, &list_value_from_vec(orphan_nodes))
                    .expect("touch check (orphan)"),
            "a diff on an orphan node (no claim references it) must NOT touch the entry (skips)"
        );
    }
}

#[cfg(test)]
mod floor_disposition_kernel_alignment {
    use super::{
        build_multi_entry_index, collect_frontier_seeds_from_diff_line_ranges,
        diff_file_matches_entry, entry_touches_frontier_seeds, make_eval_context,
        parse_unified_diff_line_ranges, resolve_entry_with_index, DiscoveryRow,
    };
    use crate::v1_interpreter::{self, ExecutionMode, Value};
    use std::path::PathBuf;

    const FIXTURE_REL: &str = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    const FLOOR_RUNNER_TEST: &str = "src/v2/workflow/affected_set_floor_runner_test.dag";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ]
    }

    fn fixture_line(text: &str, needle: &str) -> i64 {
        text.lines()
            .position(|l| l.contains(needle))
            .map(|i| (i + 1) as i64)
            .unwrap_or_else(|| panic!("fixture missing line containing `{needle}`"))
    }

    fn unified_diff_for_line(rel_path: &str, line: i64) -> String {
        format!(
            "diff --git a/{rel_path} b/{rel_path}\n--- a/{rel_path}\n+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n+// witness-a touch\n"
        )
    }

    fn list_value_from_strings(items: &[String]) -> Value {
        v1_interpreter::list_value(
            items
                .iter()
                .map(|s| Value::Str(s.clone()))
                .collect::<Vec<_>>(),
        )
    }

    fn call_floor_kernel_would_skip(
        ctx: &v1_interpreter::InterpContext,
        changed_paths: &[String],
        frontier_nodes: &[Value],
        touches_frontier: bool,
        function_edited: bool,
        entry_file_touched: bool,
    ) -> Result<bool, String> {
        if !ctx.item_registry.contains_key("floor_kernel_would_skip") {
            return Err("floor_kernel_would_skip not in context".to_string());
        }
        let args = [
            (
                Some("changed_paths".to_string()),
                list_value_from_strings(changed_paths),
            ),
            (
                Some("frontier_nodes".to_string()),
                v1_interpreter::list_value(frontier_nodes.to_vec()),
            ),
            (
                Some("touches_frontier".to_string()),
                Value::Bool(touches_frontier),
            ),
            (
                Some("function_edited".to_string()),
                Value::Bool(function_edited),
            ),
            (
                Some("entry_file_touched".to_string()),
                Value::Bool(entry_file_touched),
            ),
        ];
        match v1_interpreter::run_in_context_with_args(ctx, "floor_kernel_would_skip", &args, true)
        {
            Ok(Value::Bool(b)) => Ok(b),
            Ok(other) => Err(format!(
                "floor_kernel_would_skip returned `{}`, expected Bool",
                ctx.format_value(&other)
            )),
            Err(e) => Err(format!("floor_kernel_would_skip: {e}")),
        }
    }

    fn rust_row_would_skip(skip_enabled: bool, entry_touches: bool, function_edited: bool) -> bool {
        skip_enabled && !entry_touches && !function_edited
    }

    fn function_edited_for_row(seeds: &super::NodeFrontierSeeds, row: &DiscoveryRow) -> bool {
        seeds
            .edited_test_fns
            .iter()
            .any(|(file, func)| diff_file_matches_entry(file, &row.entry) && func == &row.function)
    }

    struct Scenario {
        label: &'static str,
        diff_line_needle: &'static str,
        expect_node_frontier_fires: bool,
        expect_function_edited_fires: bool,
    }

    fn assert_disposition_kernel_alignment_for_scenario(
        ws: &PathBuf,
        scenario: &Scenario,
        roster: &[DiscoveryRow],
    ) {
        let roots = setup_roots(ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = ws.join(FIXTURE_REL).to_string_lossy().into_owned();
        let text = std::fs::read_to_string(ws.join(FIXTURE_REL))
            .expect("node_precise_discriminator fixture readable");
        let line = fixture_line(&text, scenario.diff_line_needle);
        let diff = unified_diff_for_line(FIXTURE_REL, line);
        let ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .unwrap_or_else(|e| panic!("{}: seeds collection failed: {e}", scenario.label));
        assert!(
            !seeds.force_run_all,
            "{}: diff must be node-precise (not force_run_all)",
            scenario.label
        );

        let (graph, source_indices) =
            resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let entry_ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let entry_touches = entry_touches_frontier_seeds(&entry_ctx, &fixture_abs, &seeds)
            .unwrap_or_else(|e| panic!("{}: entry touch check failed: {e}", scenario.label));

        if scenario.expect_node_frontier_fires {
            assert!(
                entry_touches,
                "{}: node-frontier axis must fire for this diff",
                scenario.label
            );
        }
        if scenario.expect_function_edited_fires {
            assert!(
                seeds
                    .edited_test_fns
                    .iter()
                    .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
                "{}: function-edited axis must populate edited_test_fns (got {:?})",
                scenario.label,
                seeds.edited_test_fns
            );
        }

        let (runner_graph, runner_indices) = resolve_entry_with_index(&index, FLOOR_RUNNER_TEST)
            .expect("floor runner test entry resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let changed_paths = vec![FIXTURE_REL.to_string()];

        let mut saw_node_frontier_run = false;
        let mut saw_function_edited_run = false;

        for row in roster {
            let function_edited = function_edited_for_row(&seeds, row);
            let rust_skip = rust_row_would_skip(true, entry_touches, function_edited);
            let dag_skip = call_floor_kernel_would_skip(
                &runner_ctx,
                &changed_paths,
                &[],
                entry_touches,
                function_edited,
                false,
            )
            .unwrap_or_else(|e| {
                panic!(
                    "{}: dag floor_kernel_would_skip failed for {} ({}): {e}",
                    scenario.label, row.function, row.entry
                )
            });
            assert_eq!(
                rust_skip, dag_skip,
                "{}: run/skip mismatch for {} ({}): rust_skip={rust_skip} dag_skip={dag_skip} \
                 entry_touches={entry_touches} function_edited={function_edited}",
                scenario.label, row.function, row.entry
            );
            if !rust_skip && entry_touches {
                saw_node_frontier_run = true;
            }
            if !rust_skip && function_edited {
                saw_function_edited_run = true;
            }
        }

        if scenario.expect_node_frontier_fires {
            assert!(
                saw_node_frontier_run,
                "{}: expected at least one witness to RUN via node-frontier axis",
                scenario.label
            );
        }
        if scenario.expect_function_edited_fires {
            assert!(
                saw_function_edited_run,
                "{}: expected at least one witness to RUN via function-edited axis",
                scenario.label
            );
        }
    }

    #[test]
    fn disposition_kernel_aligns_on_discriminator_fixture() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let fixture_abs = ws.join(FIXTURE_REL).to_string_lossy().into_owned();
        let roster = vec![
            DiscoveryRow {
                label: "floor_disc_witness_a_only".into(),
                entry: fixture_abs.clone(),
                function: "floor_disc_witness_a_only_holds".into(),
            },
            DiscoveryRow {
                label: "floor_disc_witness_b_only".into(),
                entry: fixture_abs.clone(),
                function: "floor_disc_witness_b_only_holds".into(),
            },
            DiscoveryRow {
                label: "floor_disc_witness_transitive".into(),
                entry: fixture_abs.clone(),
                function: "floor_disc_witness_transitive_holds".into(),
            },
        ];

        assert_disposition_kernel_alignment_for_scenario(
            &ws,
            &Scenario {
                label: "node-frontier (referenced data item C)",
                diff_line_needle: "^floor_disc_node_c_symbol",
                expect_node_frontier_fires: true,
                expect_function_edited_fires: false,
            },
            &roster,
        );

        assert_disposition_kernel_alignment_for_scenario(
            &ws,
            &Scenario {
                label: "function-edited (witness A declaration)",
                diff_line_needle: "test fn floor_disc_witness_a_only_holds",
                expect_node_frontier_fires: false,
                expect_function_edited_fires: true,
            },
            &roster,
        );

        assert_disposition_kernel_alignment_for_scenario(
            &ws,
            &Scenario {
                label: "orphan node (both axes false for transitive witness)",
                diff_line_needle: "^floor_disc_orphan_symbol",
                expect_node_frontier_fires: false,
                expect_function_edited_fires: false,
            },
            &roster,
        );
    }

    #[test]
    fn disposition_kernel_seeds_populated_on_referenced_node_diff() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let text = std::fs::read_to_string(ws.join(FIXTURE_REL)).expect("fixture readable");
        let line = fixture_line(&text, "^floor_disc_node_c_symbol");
        let diff = unified_diff_for_line(FIXTURE_REL, line);
        let ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .expect("seeds from referenced-node diff");
        assert!(
            !seeds.overlapping_data_items.is_empty(),
            "referenced-node diff must populate overlapping_data_items"
        );
        assert!(
            seeds.edited_test_fns.is_empty(),
            "data-item diff must not populate edited_test_fns"
        );
    }
}

// Step 3 witness (a) PARTIAL — impl-vs-impl PROVE gate (#5994).
// Stable floor witnesses use deterministic fixture unified diffs (same structured shape as CI
// git diff parsing) so every checkout executes the proof — not branch-only origin/main...HEAD
// asserts. Node-frontier axis vs Rust NodeFrontierSeeds on whole-tree InferredTree remains
// blocked on resolve grounding (ROADMAP 1-affected-set-defork); receipt in
// docs/plans/affected-set-precompute-pruning.md §Step 3 partial.

#[cfg(test)]
mod floor_witness_a_prove {
    use super::{
        build_multi_entry_index, collect_frontier_seeds_from_diff_line_ranges,
        diff_file_matches_entry, entry_touches_frontier_seeds, make_eval_context,
        parse_unified_diff_line_ranges, resolve_entry_with_index, scan_test_decl_lines,
        DiscoveryRow, FileLineRange, NodeFrontierSeeds,
    };
    use crate::v1_interpreter::{self, ExecutionMode, Value};
    use std::collections::HashMap;
    use std::path::PathBuf;

    const FIXTURE_REL: &str = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    const FLOOR_RUNNER: &str = "src/v2/workflow/affected_set_floor_runner.dag";
    const WITNESS_A_PROVE: &str = "src/v2/test/claim/affected_set_witness_a_prove_test.dag";
    const AFFECTED_SET_MID_PATH: &str = "src/v2/lens/affected_set.dag";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ]
    }

    fn fixture_line(text: &str, needle: &str) -> i64 {
        text.lines()
            .position(|l| l.contains(needle))
            .map(|i| (i + 1) as i64)
            .unwrap_or_else(|| panic!("fixture missing line containing `{needle}`"))
    }

    fn unified_diff_for_line(rel_path: &str, line: i64) -> String {
        format!(
            "diff --git a/{rel_path} b/{rel_path}\n--- a/{rel_path}\n+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n+// witness-a touch\n"
        )
    }

    fn discriminator_roster(fixture_abs: &str) -> Vec<DiscoveryRow> {
        vec![
            DiscoveryRow {
                label: "floor_disc_witness_a".into(),
                entry: fixture_abs.to_string(),
                function: "floor_disc_witness_a_only_holds".into(),
            },
            DiscoveryRow {
                label: "floor_disc_witness_b".into(),
                entry: fixture_abs.to_string(),
                function: "floor_disc_witness_b_only_holds".into(),
            },
            DiscoveryRow {
                label: "floor_disc_witness_transitive".into(),
                entry: fixture_abs.to_string(),
                function: "floor_disc_witness_transitive_holds".into(),
            },
        ]
    }

    fn diff_line_touches_from_ranges(
        line_ranges: &HashMap<String, Vec<FileLineRange>>,
    ) -> Vec<(String, i64, i64)> {
        let mut out = Vec::new();
        for (path, ranges) in line_ranges {
            for range in ranges {
                out.push((path.clone(), range.start, range.end));
            }
        }
        out.sort();
        out
    }

    fn int_value(n: i64) -> Value {
        Value::Int(n)
    }

    fn diff_line_touch_value(
        ctx: &v1_interpreter::InterpContext,
        path: &str,
        start: i64,
        end: i64,
    ) -> Value {
        use std::rc::Rc;
        Value::Record {
            type_name: ctx.sym("FloorDiffLineTouch"),
            fields: Rc::new(vec![
                (ctx.sym("path"), Value::Str(path.to_string())),
                (ctx.sym("start_line"), int_value(start)),
                (ctx.sym("end_line"), int_value(end)),
            ]),
        }
    }

    fn call_floor_test_fn_declaration_edited(
        ctx: &v1_interpreter::InterpContext,
        touches: &[(String, i64, i64)],
        file_path: &str,
        decl_line: i64,
        decl_end_line: i64,
    ) -> Result<bool, String> {
        let touch_values: Vec<Value> = touches
            .iter()
            .map(|(p, s, e)| diff_line_touch_value(ctx, p, *s, *e))
            .collect();
        let args = [
            (
                Some("touches".to_string()),
                v1_interpreter::list_value(touch_values),
            ),
            (
                Some("file_path".to_string()),
                Value::Str(file_path.to_string()),
            ),
            (Some("test_fn_decl_line".to_string()), int_value(decl_line)),
            (
                Some("test_fn_decl_end_line".to_string()),
                int_value(decl_end_line),
            ),
        ];
        match v1_interpreter::run_in_context_with_args(
            ctx,
            "floor_test_fn_declaration_edited",
            &args,
            true,
        ) {
            Ok(Value::Bool(b)) => Ok(b),
            Ok(other) => Err(format!(
                "floor_test_fn_declaration_edited returned `{}`",
                ctx.format_value(&other)
            )),
            Err(e) => Err(format!("floor_test_fn_declaration_edited: {e}")),
        }
    }

    fn call_floor_rust_run_implies_dag_run(
        ctx: &v1_interpreter::InterpContext,
        rust_touches: bool,
        rust_func: bool,
        dag_touches: bool,
        dag_func: bool,
    ) -> Result<bool, String> {
        let args = [
            (
                Some("rust_touches_frontier".to_string()),
                Value::Bool(rust_touches),
            ),
            (
                Some("rust_function_edited".to_string()),
                Value::Bool(rust_func),
            ),
            (
                Some("dag_touches_frontier".to_string()),
                Value::Bool(dag_touches),
            ),
            (
                Some("dag_function_edited".to_string()),
                Value::Bool(dag_func),
            ),
        ];
        match v1_interpreter::run_in_context_with_args(
            ctx,
            "floor_rust_run_implies_dag_run",
            &args,
            true,
        ) {
            Ok(Value::Bool(b)) => Ok(b),
            Ok(other) => Err(format!(
                "floor_rust_run_implies_dag_run returned `{}`",
                ctx.format_value(&other)
            )),
            Err(e) => Err(format!("floor_rust_run_implies_dag_run: {e}")),
        }
    }

    fn rust_function_edited_for_row(seeds: &NodeFrontierSeeds, row: &DiscoveryRow) -> bool {
        seeds
            .edited_test_fns
            .iter()
            .any(|(file, func)| diff_file_matches_entry(file, &row.entry) && func == &row.function)
    }

    fn dag_function_edited_for_row(
        ctx: &v1_interpreter::InterpContext,
        index: &super::MultiEntryIndex,
        touches: &[(String, i64, i64)],
        row: &DiscoveryRow,
    ) -> Result<bool, String> {
        let file_path = touches
            .iter()
            .find(|(path, _, _)| diff_file_matches_entry(path, &row.entry))
            .map(|(path, _, _)| path.clone())
            .unwrap_or_else(|| super::normalize_repo_path(&row.entry));
        let content = std::fs::read_to_string(&row.entry)
            .map_err(|e| format!("read {} for decl scan: {e}", row.entry))?;
        let decl_line = scan_test_decl_lines(&content)
            .into_iter()
            .find(|(name, _)| name == &row.function)
            .map(|(_, line)| line)
            .ok_or_else(|| {
                format!(
                    "witness row {} ({}) has no test fn declaration in entry",
                    row.function, row.entry
                )
            })?;
        let sorted_decls = super::collect_sorted_decl_lines_for_file(index, &row.entry)?;
        let decl_end = super::decl_span_end_line(&sorted_decls, decl_line);
        call_floor_test_fn_declaration_edited(ctx, touches, &file_path, decl_line, decl_end)
    }

    fn frontier_list_len(
        prove_ctx: &v1_interpreter::InterpContext,
        frontier: &v1_interpreter::Value,
    ) -> Result<usize, String> {
        let len = v1_interpreter::with_active_context(prove_ctx, || {
            v1_interpreter::free_monoid_to_vec(frontier).map(|items| items.len())
        });
        len.ok_or_else(|| {
            format!(
                "expected list frontier from .dag affected_set_closure, got `{}`",
                prove_ctx.format_value(frontier)
            )
        })
    }

    fn dag_affected_frontier_for_changed_path(
        prove_ctx: &v1_interpreter::InterpContext,
        changed_path: &str,
    ) -> Result<v1_interpreter::Value, String> {
        let args = [(
            Some("changed".to_string()),
            Value::Str(changed_path.to_string()),
        )];
        v1_interpreter::run_in_context_with_args(
            prove_ctx,
            "witness_a_dag_affected_nodes_for_path",
            &args,
            true,
        )
        .map_err(|e| format!("witness_a_dag_affected_nodes_for_path: {e}"))
    }

    fn dag_entry_touches_frontier_independently(
        prove_ctx: &v1_interpreter::InterpContext,
        entry_ctx: &v1_interpreter::InterpContext,
        changed_path: &str,
    ) -> Result<bool, String> {
        let frontier = dag_affected_frontier_for_changed_path(prove_ctx, changed_path)?;
        super::entry_touches_rerun_frontier(entry_ctx, &frontier)
    }

    fn assert_superset_on_fixture_with_real_diff_shape(
        ws: &PathBuf,
        diff_text: &str,
        roster: &[DiscoveryRow],
    ) {
        let roots = setup_roots(ws);
        let index = build_multi_entry_index(&roots);
        let line_ranges = parse_unified_diff_line_ranges(diff_text);
        assert!(
            !line_ranges.is_empty(),
            "PROVE diff must contain at least one .dag hunk"
        );
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &line_ranges)
            .unwrap_or_else(|e| panic!("real-diff seeds collection failed: {e}"));
        assert!(
            !seeds.force_run_all,
            "dag-only real diff must not hit force_run_all during PROVE"
        );
        let touches = diff_line_touches_from_ranges(&line_ranges);

        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let (prove_graph, prove_indices) =
            resolve_entry_with_index(&index, WITNESS_A_PROVE).expect("witness a prove resolves");
        let prove_ctx = make_eval_context(&prove_graph, prove_indices, ExecutionMode::Wet);

        let fixture_abs = ws.join(FIXTURE_REL).to_string_lossy().into_owned();
        let (graph, source_indices) =
            resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let entry_ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let rust_entry_touches = entry_touches_frontier_seeds(&entry_ctx, &fixture_abs, &seeds)
            .expect("rust entry touch check");

        let changed_paths: Vec<String> = line_ranges.keys().cloned().collect();
        let mid_in_diff = changed_paths
            .iter()
            .any(|p| super::normalize_repo_path(p) == AFFECTED_SET_MID_PATH);
        let dag_entry_touches = if mid_in_diff {
            dag_entry_touches_frontier_independently(&prove_ctx, &entry_ctx, AFFECTED_SET_MID_PATH)
                .unwrap_or_else(|e| panic!("independent dag node-frontier: {e}"))
        } else {
            false
        };

        let mut saw_node_frontier_run = false;
        let mut saw_function_edited_run = false;

        for row in roster {
            let rust_func = rust_function_edited_for_row(&seeds, row);
            let dag_func = dag_function_edited_for_row(&runner_ctx, &index, &touches, row)
                .unwrap_or_else(|e| panic!("dag function_edited for {}: {e}", row.function));
            let rust_touches = if diff_file_matches_entry(FIXTURE_REL, &row.entry) {
                rust_entry_touches
            } else {
                false
            };
            let dag_touches = if diff_file_matches_entry(FIXTURE_REL, &row.entry) && mid_in_diff {
                dag_entry_touches
            } else {
                false
            };
            assert!(
                call_floor_rust_run_implies_dag_run(
                    &runner_ctx,
                    rust_touches,
                    rust_func,
                    dag_touches,
                    dag_func
                )
                .unwrap_or_else(|e| panic!("superset predicate: {e}")),
                "superset violated for {} ({}): rust_touches={rust_touches} rust_func={rust_func} \
                 dag_touches={dag_touches} dag_func={dag_func}",
                row.function,
                row.entry
            );
            if rust_touches || rust_func {
                assert!(
                    !(call_floor_rust_run_implies_dag_run(
                        &runner_ctx,
                        rust_touches,
                        rust_func,
                        false,
                        false
                    ))
                    .unwrap_or(false),
                    "RED control sanity: strict-subset dag must fail superset for {}",
                    row.function
                );
            }
            if rust_touches && !rust_func {
                saw_node_frontier_run = true;
            }
            if rust_func {
                saw_function_edited_run = true;
            }
        }

        assert!(
            saw_node_frontier_run || saw_function_edited_run,
            "PROVE diff must fire at least one skip axis on the roster"
        );
    }

    #[test]
    fn witness_a_function_edited_axis_fixture_impl_vs_impl() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let text = std::fs::read_to_string(ws.join(FIXTURE_REL)).expect("fixture readable");
        let line = fixture_line(&text, "test fn floor_disc_witness_a_only_holds");
        let diff = unified_diff_for_line(FIXTURE_REL, line);
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let line_ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &line_ranges)
            .expect("seeds from function-edited fixture diff");
        assert!(
            !seeds.force_run_all,
            "fixture diff must be node-precise (not force_run_all)"
        );
        assert!(
            seeds
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "function-edited fixture must populate edited_test_fns"
        );
        let touches = diff_line_touches_from_ranges(&line_ranges);
        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);

        for (file, func) in &seeds.edited_test_fns {
            let content = std::fs::read_to_string(file)
                .unwrap_or_else(|e| panic!("read {file} for decl line: {e}"));
            let decl_line = scan_test_decl_lines(&content)
                .into_iter()
                .find(|(name, _)| name == func)
                .map(|(_, line)| line)
                .unwrap_or_else(|| panic!("edited_test_fns {file}::{func} missing decl line"));
            let sorted_decls = super::collect_sorted_decl_lines_for_file(&index, file)
                .expect("sorted decl lines for impl-vs-impl");
            let decl_end = super::decl_span_end_line(&sorted_decls, decl_line);
            let dag_edited = call_floor_test_fn_declaration_edited(
                &runner_ctx,
                &touches,
                file,
                decl_line,
                decl_end,
            )
            .expect("dag function_edited model");
            assert!(
                dag_edited,
                "function_edited axis: rust edited_test_fns ({file}, {func}) must be matched by \
                 independent .dag floor_test_fn_declaration_edited"
            );
        }
    }

    #[test]
    fn witness_a_function_edited_axis_body_touch_fixture_impl_vs_impl() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        // Body line inside floor_disc_witness_a_only_holds (rebased when floor_disc_helper_fn landed in #6061).
        let diff = unified_diff_for_line(FIXTURE_REL, 83);
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let line_ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &line_ranges)
            .expect("seeds from body-touch fixture diff");
        assert!(
            seeds
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "body-only diff touch must populate edited_test_fns via decl span (not decl line only)"
        );
        let touches = diff_line_touches_from_ranges(&line_ranges);
        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let file = FIXTURE_REL;
        let content = std::fs::read_to_string(ws.join(FIXTURE_REL)).expect("fixture readable");
        let decl_line = scan_test_decl_lines(&content)
            .into_iter()
            .find(|(name, _)| name == "floor_disc_witness_a_only_holds")
            .map(|(_, line)| line)
            .expect("witness_a decl line");
        let sorted_decls =
            super::collect_sorted_decl_lines_for_file(&index, file).expect("sorted decl lines");
        let decl_end = super::decl_span_end_line(&sorted_decls, decl_line);
        assert!(
            call_floor_test_fn_declaration_edited(&runner_ctx, &touches, file, decl_line, decl_end)
                .expect("dag function_edited model for body touch"),
            "body-only diff must match .dag floor_test_fn_declaration_edited when decl_end spans body"
        );
    }

    #[test]
    fn witness_a_red_control_under_selection_fails_superset() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        assert!(
            !call_floor_rust_run_implies_dag_run(&runner_ctx, true, false, false, false)
                .expect("superset must fail when dag under-selects node-frontier"),
            "mandatory RED: rust-run + dag-skip must violate superset (§5 fail-open guard)"
        );
        assert!(
            !call_floor_rust_run_implies_dag_run(&runner_ctx, false, true, false, false)
                .expect("superset must fail when dag under-selects function_edited"),
            "mandatory RED: rust function_edited run + dag skip must violate superset"
        );
    }

    #[test]
    fn witness_a_node_frontier_dag_closure_independent_on_fixture() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let (prove_graph, prove_indices) =
            resolve_entry_with_index(&index, WITNESS_A_PROVE).expect("witness a prove resolves");
        let prove_ctx = make_eval_context(&prove_graph, prove_indices, ExecutionMode::Wet);
        let affected = dag_affected_frontier_for_changed_path(&prove_ctx, AFFECTED_SET_MID_PATH)
            .expect("dag affected_set_closure frontier");
        let node_count = frontier_list_len(&prove_ctx, &affected)
            .expect("frontier must be a list (List or Cons carrier)");
        assert!(
            node_count > 0,
            ".dag affected_set_closure must produce non-empty frontier for {AFFECTED_SET_MID_PATH} \
             via provenance_producer fixture (Impl-1 not inert; whole-tree Rust equivalence deferred)"
        );
    }

    #[test]
    fn witness_a_superset_on_discriminator_function_edited_fixture() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let text = std::fs::read_to_string(ws.join(FIXTURE_REL)).expect("fixture readable");
        let line = fixture_line(&text, "test fn floor_disc_witness_a_only_holds");
        let diff = unified_diff_for_line(FIXTURE_REL, line);
        let fixture_abs = ws.join(FIXTURE_REL).to_string_lossy().into_owned();
        assert_superset_on_fixture_with_real_diff_shape(
            &ws,
            &diff,
            &discriminator_roster(&fixture_abs),
        );
    }
}

// SCAFFOLD: folds into a .dag execution witness when the discovery/diff seed plumbing
// migrates off the v1 host layer (§6 dissolution trigger)
#[cfg(test)]
mod node_frontier_plumbing_controls {
    use super::{
        build_multi_entry_index, call_floor_kernel_would_skip, entry_touches_rerun_frontier,
        floor_diff_edits_from_diff_text, list_value_from_vec, parse_unified_diff_line_ranges,
        rerun_frontier_nodes_for_entry,
    };
    use crate::v1_interpreter::ExecutionMode;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    const FIXTURE: &str = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    // File outside FIXTURE's import closure — precondition asserted at runtime in green control.
    // If a future import edge adds this file to FIXTURE's closure, the precondition assertion
    // fails loudly rather than letting the control silently degrade (§3 anti-drift).
    const OUTSIDE_FILE: &str = "src/v2/lens/affected_set.dag";
    // A known data-declaration line in OUTSIDE_FILE.
    // If this line shifts the test may fail seed collection — a loud failure, not a silent pass.
    const OUTSIDE_DATA_LINE: i64 = 1295;

    fn abs(ws: &PathBuf, rel: &str) -> String {
        ws.join(rel).to_string_lossy().into_owned()
    }

    // parse_unified_diff_line_ranges strips "+++ b/" prefix; "b//abs/path" yields "/abs/path".
    fn diff_at(file: &str, line: i64) -> String {
        format!(
            "diff --git a/{file} b/{file}\n--- a/{file}\n+++ b/{file}\n\
             @@ -{line},0 +{line},1 @@\n+// synthetic touch\n"
        )
    }

    fn deletion_diff_at(file: &str, line: i64) -> String {
        format!(
            "diff --git a/{file} b/{file}\n--- a/{file}\n+++ b/{file}\n\
             @@ -{line},1 +{line},0 @@\n-// synthetic deletion\n"
        )
    }

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ]
    }

    // Control 1 (GREEN/skip): diff on file outside FIXTURE's import closure → skip fires.
    // Q1 precondition asserted at runtime: if a future import edge adds OUTSIDE_FILE to
    // FIXTURE's closure, this assertion fires before the skip assertion can silently degrade.
    #[test]
    fn green_skip_for_file_outside_import_closure() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);

        // Q1 precondition: assert OUTSIDE_FILE is not in FIXTURE's transitive import closure.
        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &abs(&ws, FIXTURE)).expect("fixture resolves");
        let outside = OUTSIDE_FILE.replace('\\', "/");
        let in_closure = graph.modules.iter().any(|m| {
            m.items
                .iter()
                .any(|item| item.span.file.replace('\\', "/").contains(&outside))
        });
        assert!(
            !in_closure,
            "precondition: {OUTSIDE_FILE} must not be in {FIXTURE}'s import closure; \
             if it now is, update OUTSIDE_FILE to a different out-of-closure file"
        );

        // Build diff touching a data declaration in OUTSIDE_FILE (absolute path so parse_unified_diff
        // resolves it without process-global cwd — "b//abs" strips to "/abs" after the b/ prefix).
        let diff = diff_at(&abs(&ws, OUTSIDE_FILE), OUTSIDE_DATA_LINE);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from outside-file diff");
        let ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes =
            rerun_frontier_nodes_for_entry(&ctx, &abs(&ws, FIXTURE), &seeds).expect("nodes");
        assert!(
            nodes.is_empty()
                || !entry_touches_rerun_frontier(&ctx, &list_value_from_vec(nodes))
                    .expect("touch check"),
            "entry must NOT touch frontier when diff is on a file outside its import closure"
        );
    }

    // Control 2 (RED/function_edited): diff edits a test fn declaration →
    // edited_test_fns populated → function_edited=true forces run for that row.
    #[test]
    fn red_function_edited_populates_edited_test_fns() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let text = std::fs::read_to_string(FIXTURE).expect("fixture readable");
        let test_fn_line = text
            .lines()
            .position(|l| l.contains("test fn floor_disc_witness_a_only_holds"))
            .map(|i| (i + 1) as i64)
            .expect("witness A test fn line");
        let diff = diff_at(FIXTURE, test_fn_line);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from test-fn-line diff");
        assert!(
            seeds
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "diff at test fn declaration line must populate edited_test_fns with the function name"
        );
    }

    #[test]
    fn deletion_only_hunk_populates_edited_test_fns() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let text = std::fs::read_to_string(FIXTURE).expect("fixture readable");
        let test_fn_line = text
            .lines()
            .position(|l| l.contains("test fn floor_disc_witness_a_only_holds"))
            .map(|i| (i + 1) as i64)
            .expect("witness A test fn line");
        let diff = deletion_diff_at(FIXTURE, test_fn_line);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from deletion-only diff");
        assert!(
            seeds
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "deletion-only diff at test fn line must populate edited_test_fns"
        );
    }

    // Control 3 (RED/node_frontier): diff on a data item referenced by a claim →
    // entry_touches_rerun_frontier returns true → runs.
    #[test]
    fn red_node_frontier_fires_for_referenced_data_item() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        let text = std::fs::read_to_string(&fixture_abs).expect("fixture readable");
        let data_line = text
            .lines()
            .position(|l| l.contains("data floor_disc_node_a"))
            .map(|i| (i + 1) as i64)
            .expect("floor_disc_node_a line");
        let diff = diff_at(&fixture_abs, data_line);
        let seeds = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("seeds from referenced-node diff");
        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes = rerun_frontier_nodes_for_entry(&ctx, &fixture_abs, &seeds).expect("nodes");
        assert!(
            entry_touches_rerun_frontier(&ctx, &list_value_from_vec(nodes)).expect("touch check"),
            "entry must touch frontier when diff is on a data item referenced by a claim"
        );
    }

    // Control 4 (entry_file_touched / ROADMAP 1-affected-set-defork acceptance (a)):
    // non-data, non-test-fn declaration edit scopes runs to that entry only — the touched
    // entry's roster runs via `entry_file_touched`; unrelated entries skip when frontier empty.
    #[test]
    fn green_entry_file_helper_fn_edit_scopes_to_same_entry_only() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        let text = std::fs::read_to_string(&fixture_abs).expect("fixture readable");
        let helper_line = text
            .lines()
            .position(|l| l.contains("fn floor_disc_helper_fn"))
            .map(|i| (i + 1) as i64)
            .expect("helper fn line");
        let diff = diff_at(&fixture_abs, helper_line);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from helper-fn-line diff");
        assert!(
            seeds
                .touched_entry_files
                .iter()
                .any(|f| f.contains("node_precise_discriminator")),
            "helper fn edit must populate touched_entry_files"
        );
        assert!(
            seeds.overlapping_data_items.is_empty() && seeds.edited_test_fns.is_empty(),
            "helper fn edit must not populate data-item frontier or edited_test_fns"
        );

        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let entry_ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes =
            rerun_frontier_nodes_for_entry(&entry_ctx, &fixture_abs, &seeds).expect("nodes");
        assert!(
            nodes.is_empty(),
            "helper fn edit must not materialize data-item frontier nodes"
        );

        let (runner_graph, runner_indices) = super::resolve_entry_with_index(
            &index,
            &abs(&ws, "src/v2/workflow/affected_set_floor_runner.dag"),
        )
        .expect("floor runner resolves");
        let runner_ctx =
            super::make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let changed_paths = vec![fixture_abs.clone()];
        assert!(
            !call_floor_kernel_would_skip(&runner_ctx, &changed_paths, &nodes, false, false, true)
                .expect("skip verdict for touched entry"),
            "helper-fn edit must RUN witnesses in the touched entry (entry_file_touched)"
        );
        assert!(
            call_floor_kernel_would_skip(&runner_ctx, &changed_paths, &nodes, false, false, false)
                .expect("skip verdict for unrelated entry"),
            "helper-fn edit must SKIP witnesses in an unrelated entry when frontier is empty"
        );
    }

    // Control 4b (entry_file_touched / import-closure): non-data fn edit in an imported
    // module runs witnesses in the importing entry, not only when the entry file itself changed.
    #[test]
    fn green_import_closure_helper_fn_edit_runs_importer_entry() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let helper_rel = "src/v2/test/fixture/floor_skip/floor_disc_shared_helper.dag";
        let fixture_abs = abs(&ws, FIXTURE);
        let helper_abs = abs(&ws, helper_rel);
        let text = std::fs::read_to_string(&helper_abs).expect("shared helper readable");
        let helper_line = text
            .lines()
            .position(|l| l.contains("fn floor_disc_shared_helper"))
            .map(|i| (i + 1) as i64)
            .expect("shared helper fn line");
        let diff = diff_at(&helper_abs, helper_line);
        let seeds = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("seeds from cross-file helper-fn diff");
        assert!(
            seeds
                .touched_entry_files
                .iter()
                .any(|f| f.contains("floor_disc_shared_helper")),
            "cross-file helper fn edit must populate touched_entry_files"
        );

        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let closure_files = super::import_closure_files_from_graph(&graph);
        assert!(
            super::touched_file_in_import_closure(&helper_abs, &closure_files),
            "shared helper module must be in fixture import closure"
        );
        let entry_ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes =
            rerun_frontier_nodes_for_entry(&entry_ctx, &fixture_abs, &seeds).expect("nodes");
        assert!(
            nodes.is_empty(),
            "cross-file helper fn edit must not materialize data-item frontier nodes"
        );

        let (runner_graph, runner_indices) = super::resolve_entry_with_index(
            &index,
            &abs(&ws, "src/v2/workflow/affected_set_floor_runner.dag"),
        )
        .expect("floor runner resolves");
        let runner_ctx =
            super::make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let changed_paths = vec![helper_abs.clone()];
        assert!(
            !call_floor_kernel_would_skip(
                &runner_ctx,
                &changed_paths,
                &nodes,
                false,
                false,
                true
            )
            .expect("skip verdict for importing entry"),
            "cross-file helper-fn edit must RUN witnesses in importing entry (import-closure entry_file_touched)"
        );
        assert!(
            call_floor_kernel_would_skip(&runner_ctx, &changed_paths, &nodes, false, false, false)
                .expect("skip verdict for unrelated entry"),
            "cross-file helper-fn edit must SKIP witnesses in an unrelated entry when frontier is empty"
        );
    }

    // Control 5 (fail-closed): exclusively non-.dag diff → seed collection fails closed.
    #[test]
    fn fail_closed_non_dag_file_forces_run_all() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = "diff --git a/src/v1/stage0/src/cli_run.rs b/src/v1/stage0/src/cli_run.rs\n\
                    --- a/src/v1/stage0/src/cli_run.rs\n\
                    +++ b/src/v1/stage0/src/cli_run.rs\n\
                    @@ -1,0 +2,1 @@\n+// synthetic\n";
        let err = floor_diff_edits_from_diff_text(&index, &diff)
            .expect_err("diff on a non-.dag file must fail-closed");
        assert!(
            err.contains("non-.dag"),
            "expected non-.dag fail-closed, got: {err}"
        );
    }

    // Control 5 (fail-closed): diff before first declaration in a .dag file → fail-closed.
    // The module header (line 1) precedes the first data/fn declaration.
    #[test]
    fn fail_closed_edit_before_first_decl_forces_run_all() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = diff_at(&abs(&ws, FIXTURE), 1);
        let err = floor_diff_edits_from_diff_text(&index, &diff)
            .expect_err("diff before first declaration must fail-closed");
        assert!(
            err.contains("before first declaration"),
            "expected pre-decl fail-closed, got: {err}"
        );
    }

    #[test]
    fn import_preamble_plus_fn_body_populates_touched_entry_not_fail_closed() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let emit_rel = "dag/extdeps/languages/json/emit.dag";
        let diff = include_str!("../testdata/emit_import_preamble_fn_body.diff");
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("import+fn diff must not fail-closed to full corpus");
        assert!(
            edits.touched_entry_files.iter().any(|f| f == emit_rel),
            "import preamble + fn body must touch the entry file"
        );
    }

    #[test]
    fn mixed_dag_and_non_dag_diff_scopes_from_dag_only() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let emit_rel = "dag/extdeps/languages/json/emit.dag";
        let dag_diff = include_str!("../testdata/emit_import_preamble_fn_body.diff");
        let host_diff =
            "diff --git a/src/v1/stage0/src/cli_run.rs b/src/v1/stage0/src/cli_run.rs\n\
                          --- a/src/v1/stage0/src/cli_run.rs\n\
                          +++ b/src/v1/stage0/src/cli_run.rs\n\
                          @@ -1,0 +2,1 @@\n+// synthetic\n";
        let diff = format!("{dag_diff}\n{host_diff}");
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("mixed dag+host diff must scope from .dag paths only");
        assert!(
            edits.touched_entry_files.iter().any(|f| f == emit_rel),
            "mixed diff must still attribute .dag frontier seeds"
        );
    }

    // Control 6 (fail-closed / Q2 resolve-failure): diff names a .dag path that does not
    // exist → resolve_entry_with_index fails → fail-closed.
    #[test]
    fn fail_closed_nonexistent_dag_path_forces_run_all() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = diff_at(&abs(&ws, "src/v2/lens/does_not_exist_sentinel.dag"), 10);
        let err = floor_diff_edits_from_diff_text(&index, &diff)
            .expect_err("diff naming a non-existent .dag path must fail-closed");
        assert!(
            err.contains("resolve failed"),
            "expected resolve fail-closed, got: {err}"
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceRootReadRecord {
    pub file_path: String,
    pub module_path: String,
    pub source: String,
    pub source_root: String,
}

fn source_root_ref_variant_for_root(root: &str) -> Result<String, String> {
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

fn source_root_ref_token_for_path(
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

fn free_monoid_symbol_emit_dag(segments: &[String]) -> String {
    if segments.is_empty() {
        return "Empty".to_string();
    }
    let mut out = String::from("Empty");
    for seg in segments.iter().rev() {
        out = format!("Cons {{ head: ^{seg}, tail: {out} }}");
    }
    out
}

#[cfg(test)]
mod manifest_emit_tests {
    use super::{
        dag_embedded_dag_source_escape, dag_manifest_scalar_escape, free_monoid_symbol_emit_dag,
    };

    #[test]
    fn free_monoid_symbol_emit_dag_three_segment_path() {
        assert_eq!(
            free_monoid_symbol_emit_dag(&["v2".into(), "compiler".into(), "compile".into()]),
            "Cons { head: ^v2, tail: Cons { head: ^compiler, tail: Cons { head: ^compile, tail: Empty } } }"
        );
    }

    #[test]
    fn free_monoid_symbol_emit_dag_empty_is_empty_variant() {
        assert_eq!(free_monoid_symbol_emit_dag(&[]), "Empty");
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

    #[test]
    fn source_root_token_grounds_in_filesystem_location() {
        let roots = vec!["src/v2".to_string(), "dag".to_string()];
        assert_eq!(
            source_root_ref_token_for_path("src/v2/std/algebra.dag", &roots).unwrap(),
            "V2Tree"
        );
        assert_eq!(
            source_root_ref_token_for_path("dag/std/algebra.dag", &roots).unwrap(),
            "DagTree"
        );
        assert_eq!(
            source_root_ref_token_for_path("src/v2/extdeps/shell.dag", &roots).unwrap(),
            "V2Tree"
        );
        assert!(source_root_ref_token_for_path("src/v1/stage0/x.dag", &roots).is_err());
        assert!(source_root_ref_token_for_path("src/v20/x.dag", &roots).is_err());
    }

    #[test]
    fn source_root_token_admits_absolute_roots() {
        let ws = super::workspace_root();
        let abs_roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ];
        assert_eq!(
            source_root_ref_token_for_path(
                ws.join("src/v2/std/algebra.dag").to_str().unwrap(),
                &abs_roots
            )
            .unwrap(),
            "V2Tree"
        );
        assert_eq!(
            source_root_ref_token_for_path(
                ws.join("dag/std/algebra.dag").to_str().unwrap(),
                &abs_roots
            )
            .unwrap(),
            "DagTree"
        );
        assert_eq!(
            source_root_ref_token_for_path("dag/std/algebra.dag", &abs_roots).unwrap(),
            "DagTree"
        );
        assert!(source_root_ref_token_for_path(
            ws.join("src/v1/stage0/x.dag").to_str().unwrap(),
            &abs_roots
        )
        .is_err());
    }
}

fn emit_import_admission_list(imports: &[Vec<String>]) -> String {
    let mut out = String::from("Empty");
    for import in imports.iter().rev() {
        out = format!(
            "Cons {{\n  head: Import {{\n    target: {},\n    visibility: ImportVisible\n  }},\n  tail: {out}\n}}",
            free_monoid_symbol_emit_dag(import)
        );
    }
    out
}

fn emit_source_root_entry_admission_data(admission: &SourceRootEntryAdmission) -> String {
    format!(
        "data host_compiler_closure_admission: Admission = Admission {{\n  subject: ResolutionSubject {{\n    name: {}\n  }},\n  imports: {}\n}}\n\n\n",
        free_monoid_symbol_emit_dag(&admission.subject),
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
        out.push_str("import v2.std.algebra { Cons, Empty }\n");
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
        default_source_roots, discover_floor_corpus_rows, inert_lens_modules,
        is_top_level_lens_module, witness_discovery_scan_dirs, DiscoveryRow,
        FLOOR_DISCOVERY_EXCLUDES,
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
        assert!(!is_top_level_lens_module(
            "v2.lens.extdeps_shape_transport_policy.module_refs"
        ));
        assert!(!is_top_level_lens_module(
            "v2.test.lens_effect.effect_depends_on"
        ));
        assert!(!is_top_level_lens_module("v2.std.algebra"));
        assert!(!is_top_level_lens_module("v2.lens."));
    }

    #[test]
    fn detector_red_on_unreached_green_on_wired() {
        let mut module_to_path: HashMap<String, String> = HashMap::new();
        let mut path_imports: HashMap<String, Vec<String>> = HashMap::new();
        module_to_path.insert(
            "v2.lens.demo".to_string(),
            "src/v2/lens/demo.dag".to_string(),
        );
        path_imports.insert("src/v2/lens/demo.dag".to_string(), vec![]);

        let inert = inert_lens_modules(&[], &path_imports, &module_to_path);
        assert_eq!(inert, vec!["v2.lens.demo".to_string()]);

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

    #[test]
    fn builtin_inert_lens_counts_are_green_on_live_corpus() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        assert_eq!(
            super::inert_lens_unreached_module_count(),
            0,
            "every v2.lens.* must be reached by a floor witness"
        );
        assert!(
            super::inert_lens_top_level_module_count() > 0,
            "lens universe must be non-empty (non-vacuity oracle)"
        );
    }

    #[test]
    fn floor_corpus_has_no_inert_lenses() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = default_source_roots();
        let scan_dirs = witness_discovery_scan_dirs();
        let excludes: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
            .iter()
            .map(|s| s.to_string())
            .collect();
        let result = discover_floor_corpus_rows(&roots, &scan_dirs, &excludes);
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
        construction_authority_graph_unresolved, construction_authority_unresolved,
        declares_construction_justification, discover_floor_corpus_rows, unjustified_lens_modules,
        wall_now_authority_refs, FLOOR_DISCOVERY_EXCLUDES,
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

    #[test]
    fn justification_scan_predicate() {
        let with = "module v2.lens.demo\n\
            import v2.lens.common.construction_justification { ConstructionJustification, RatchetForever }\n\
            data construction_justification: ConstructionJustification = ConstructionJustification {\n\
              class: RatchetForever\n\
            }\n";
        assert!(declares_construction_justification(with));

        assert!(!declares_construction_justification(
            "data construction_justification_note: String = \"todo\"\n"
        ));
        assert!(!declares_construction_justification(
            "module v2.lens.demo\ndata other: String = \"z\"\n"
        ));
    }

    #[test]
    fn detector_red_on_missing_green_on_recorded() {
        let mut module_to_path: HashMap<String, String> = HashMap::new();
        module_to_path.insert(
            "v2.lens.demo".to_string(),
            "src/v2/lens/demo.dag".to_string(),
        );
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

    #[test]
    fn floor_corpus_every_lens_is_justified() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let scan_dirs = vec![
            "dag/test/claim".to_string(),
            "src/v2/test/claim/manual".to_string(),
        ];
        let excludes: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
            .iter()
            .map(|s| s.to_string())
            .collect();
        let result = discover_floor_corpus_rows(&roots, &scan_dirs, &excludes);
        assert!(
            result.is_ok(),
            "floor discovery must succeed — every v2.lens.* records a construction-justification: {}",
            result.err().unwrap_or_default()
        );
    }

    // ITEM 2 graph-property witness: the construction->authority graph is TOTAL over the
    // live corpus (every WallNow authority DeclarationRef resolves to a real top-level decl).
    // Perturb-to-RED: plant a dangling decl_name in any WallNow site -> this flips to a
    // non-empty unresolved list and the test fails. (SCAFFOLD, dissolves on item (ii) —
    // unified kind-agnostic decl-resolution exposed to .dag; see construction_authority_* docs.)
    #[test]
    fn wall_now_authority_graph_is_total() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let unresolved =
            construction_authority_graph_unresolved(&roots).expect("corpus walk must succeed");
        assert!(
            unresolved.is_empty(),
            "every WallNow construction-authority must resolve to a real decl; dangling: {unresolved:?}"
        );
    }

    // Discriminating control: the resolver detects a dangling authority and clears on a real one.
    #[test]
    fn dangling_authority_is_detected() {
        let mut module_to_content: HashMap<String, String> = HashMap::new();
        module_to_content.insert(
            "v2.std.node".to_string(),
            "module v2.std.node\ntype NodeKind\n  = TypeNode { connective: Connective }\n"
                .to_string(),
        );

        let real = vec![(
            "src/v2/lens/cost.dag".to_string(),
            "v2.std.node".to_string(),
            "NodeKind".to_string(),
        )];
        assert!(
            construction_authority_unresolved(&module_to_content, &real).is_empty(),
            "a real authority (v2.std.node.NodeKind) must resolve"
        );

        let dangling = vec![(
            "src/v2/lens/cost.dag".to_string(),
            "v2.std.node".to_string(),
            "NoSuchDecl".to_string(),
        )];
        assert_eq!(
            construction_authority_unresolved(&module_to_content, &dangling).len(),
            1,
            "a dangling decl_name must be flagged unresolved"
        );

        let missing_module = vec![(
            "src/v2/lens/cost.dag".to_string(),
            "v2.absent.module".to_string(),
            "NodeKind".to_string(),
        )];
        assert_eq!(
            construction_authority_unresolved(&module_to_content, &missing_module).len(),
            1,
            "an authority whose module is absent must be flagged unresolved"
        );
    }

    // Parse unit: extraction pulls (module_path, decl_name) from a WallNow authority,
    // whitespace/newline tolerant, and ignores non-WallNow DeclarationRef binds.
    #[test]
    fn wall_now_authority_refs_extraction() {
        let src = "  class: WallNow {\n    mechanism: SubstrateMandatoryTag,\n    authority: DeclarationRef { module_path: \"v2.std.node\", decl_name: \"NodeKind\", field: WholeDeclaration }\n  }\n";
        assert_eq!(
            wall_now_authority_refs(src),
            vec![("v2.std.node".to_string(), "NodeKind".to_string())]
        );
        // a Scaffold `bind: DeclarationRef { .. }` has no `authority:` field -> not captured.
        let other = "  bind: DeclarationRef { module_path: \"x.y\", decl_name: \"Z\", field: WholeDeclaration }\n";
        assert!(wall_now_authority_refs(other).is_empty());
    }
}

#[cfg(test)]
mod sidecar_placement_hygiene_tests {
    use super::{discover_floor_corpus_rows, scan_wire_contract_decl_names};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);
    fn tmp_dir() -> std::path::PathBuf {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "sidecar_placement_test_{}_{}",
            std::process::id(),
            id
        ))
    }

    #[test]
    fn scan_detects_coproduct_wire_contract_data() {
        let content =
            "data foo: CoproductWireContract = { coproduct: \"X\", encoding: UntaggedVariant }";
        assert_eq!(
            scan_wire_contract_decl_names(content),
            vec!["foo".to_string()]
        );
    }

    #[test]
    fn scan_detects_variant_encoding_data() {
        let content = "data bar: VariantEncoding = llm_snake_wire_contract";
        assert_eq!(
            scan_wire_contract_decl_names(content),
            vec!["bar".to_string()]
        );
    }

    #[test]
    fn scan_ignores_non_wire_contract_data() {
        let content = "data baz: Int = 42\ndata qux: String = \"hello\"\ndata flag: Bool = true";
        assert!(
            scan_wire_contract_decl_names(content).is_empty(),
            "should not fire on non-wire-contract data decls"
        );
    }

    #[test]
    fn misplaced_wire_contract_decl_drives_discover_to_err() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let file = dir.join("anthropic.dag");
        std::fs::write(
            &file,
            "data anthropic_chat_message_wire_contract: CoproductWireContract = { \
             coproduct: \"AnthropicChatMessage\", encoding: UntaggedVariant }\n",
        )
        .expect("write temp file");
        let root = dir.to_string_lossy().into_owned();
        let result = discover_floor_corpus_rows(&[root], &[], &[]);
        let _ = std::fs::remove_dir_all(&dir);
        let msg = result
            .err()
            .expect("misplaced wire-contract decl must drive discover_floor_corpus_rows to Err");
        assert!(
            msg.contains("wire-contract decls") && msg.contains("_contracts.dag"),
            "error must name the decl type and required suffix: {msg}"
        );
    }
}

#[cfg(test)]
mod moduleless_entry_skip_tests {
    use super::{extract_module_path, moduleless_dag_entry_paths};

    #[test]
    fn moduleless_dag_entry_paths_collects_fixture_like_fragments() {
        let entries = vec![
            (
                "/repo/src/v1/stage0/tests/fixtures/split.dag".to_string(),
                "data x: Int = 0\n".to_string(),
            ),
            (
                "/repo/src/v1/compile.dag".to_string(),
                "module v1.compile\n".to_string(),
            ),
        ];
        assert_eq!(
            moduleless_dag_entry_paths(&entries),
            vec!["/repo/src/v1/stage0/tests/fixtures/split.dag".to_string()]
        );
    }

    #[test]
    fn moduleless_dag_entry_paths_surfaces_real_source_without_module() {
        let entries = vec![(
            "/repo/src/v1/forgot_module.dag".to_string(),
            "data oops: Int = 0\n".to_string(),
        )];
        assert_eq!(
            moduleless_dag_entry_paths(&entries),
            vec!["/repo/src/v1/forgot_module.dag".to_string()]
        );
        assert!(extract_module_path(&entries[0].1).is_none());
    }
}

#[cfg(test)]
mod witness_timing_attribution_tests {
    use super::{
        compute_witness_timing_rows, top_n_slowest_witnesses, ClaimOutcome, DiscoverySummary,
        DiscoveryWitnessOutcome, EntryResolveReceipt,
    };
    use crate::v1_interpreter::PerformanceReceipt;

    fn sample_summary() -> DiscoverySummary {
        DiscoverySummary {
            total: 3,
            passed: 3,
            skipped: 0,
            failures: Vec::new(),
            witness_outcomes: vec![
                DiscoveryWitnessOutcome {
                    entry: "a.dag".to_string(),
                    function: "fast".to_string(),
                    outcome: ClaimOutcome::Pass,
                },
                DiscoveryWitnessOutcome {
                    entry: "b.dag".to_string(),
                    function: "slow".to_string(),
                    outcome: ClaimOutcome::Pass,
                },
                DiscoveryWitnessOutcome {
                    entry: "a.dag".to_string(),
                    function: "medium".to_string(),
                    outcome: ClaimOutcome::Pass,
                },
            ],
            entry_resolve_receipts: vec![
                EntryResolveReceipt {
                    entry: "a.dag".to_string(),
                    closure_subject: "subj-a".to_string(),
                    resolve_nanos: 100,
                },
                EntryResolveReceipt {
                    entry: "b.dag".to_string(),
                    closure_subject: "subj-b".to_string(),
                    resolve_nanos: 200,
                },
            ],
            total_resolve_nanos: 300,
            performance_receipts: vec![
                PerformanceReceipt {
                    subject_key: "subj-a".to_string(),
                    work_shape: "claim".to_string(),
                    wall_nanos: 1_000,
                    eval_self_nanos: 1_000,
                    sample_count: 1,
                },
                PerformanceReceipt {
                    subject_key: "subj-b".to_string(),
                    work_shape: "claim".to_string(),
                    wall_nanos: 50_000,
                    eval_self_nanos: 50_000,
                    sample_count: 1,
                },
                PerformanceReceipt {
                    subject_key: "subj-a".to_string(),
                    work_shape: "claim".to_string(),
                    wall_nanos: 5_000,
                    eval_self_nanos: 5_000,
                    sample_count: 1,
                },
            ],
            total_measured_nanos: 56_000,
        }
    }

    #[test]
    fn witness_timing_rows_pair_perf_with_outcomes() {
        let rows = compute_witness_timing_rows(&sample_summary()).expect("rows");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].function, "fast");
        assert_eq!(rows[0].eval_nanos, 1_000);
        assert_eq!(rows[0].resolve_nanos, 100);
        assert_eq!(rows[0].total_nanos, 1_100);
    }

    #[test]
    fn top_n_slowest_ranks_by_eval_descending() {
        let rows = compute_witness_timing_rows(&sample_summary()).expect("rows");
        let top = top_n_slowest_witnesses(&rows, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].function, "slow");
        assert_eq!(top[1].function, "medium");
    }
}

pub struct LayerImportFactRaw {
    pub layer: &'static str,
    pub path: String,
    pub import_module: String,
}

const LAYER_STD: &str = "LayerPrefixStd";
const LAYER_EXTDEPS: &str = "LayerPrefixExtdeps";

fn rel_path_for_layer_import(path: &Path) -> String {
    let ws = workspace_root();
    path.strip_prefix(&ws)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn pool_roots_abs(pool_roots: &[String]) -> Vec<String> {
    let ws = workspace_root();
    pool_roots
        .iter()
        .map(|r| {
            let p = Path::new(r);
            if p.is_absolute() {
                r.clone()
            } else {
                ws.join(p).to_string_lossy().into_owned()
            }
        })
        .collect()
}

fn project_layer_import_root(root: &str, layer: &'static str, out: &mut Vec<LayerImportFactRaw>) {
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
        let rel = rel_path_for_layer_import(&file);
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
        project_layer_import_root(root, LAYER_STD, &mut out);
    }
    for root in extdeps_roots {
        project_layer_import_root(root, LAYER_EXTDEPS, &mut out);
    }
    out
}

// Host-fed fact extraction for `v2.lens.fact_cardinality` — the lens `.dag` table owns
// verdict logic; this bridge only projects top-level decl keys + content hashes from the
// witness-layer trees. DISSOLUTION: node-tree reader at gunbc#5364; until then one shared
// host seam (Chunk D).
const FACT_CARDINALITY_ITEM_KEYWORDS: [&str; 8] = [
    "data ",
    "fn ",
    "func ",
    "type ",
    "service ",
    "const ",
    "pattern ",
    "resource ",
];

pub struct FactCardinalityDeclFactRaw {
    pub rel_path_decl_key: String,
    pub tree: String,
    pub content_hash: String,
}

fn normalize_decl_body(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(line).trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decl_body_hash(body: &str) -> String {
    crate::v1_rt::atom_identity_hash(normalize_decl_body(body))
}

/// Kind-agnostic top-level decl extraction (name, content-hash) for cross-tree cardinality.
pub fn extract_top_level_decls(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("test ") {
            i += 1;
            continue;
        }
        let Some(kw) = FACT_CARDINALITY_ITEM_KEYWORDS
            .iter()
            .find(|kw| line.starts_with(*kw))
        else {
            i += 1;
            continue;
        };
        let rest = &line[kw.len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut body = String::new();
        body.push_str(line);
        body.push('\n');
        i += 1;
        let mut depth = brace_delta(line);
        while i < lines.len() {
            let next = lines[i];
            if depth <= 0
                && FACT_CARDINALITY_ITEM_KEYWORDS
                    .iter()
                    .any(|kw| next.starts_with(kw))
                && !next.starts_with("test ")
            {
                break;
            }
            body.push_str(next);
            body.push('\n');
            depth += brace_delta(next);
            i += 1;
        }
        out.push((name, decl_body_hash(&body)));
    }
    out
}

fn rel_path_within_tree(top_root: &Path, path: &Path) -> String {
    path.strip_prefix(top_root)
        .unwrap_or_else(|_| {
            panic!(
                "fact_cardinality_decl_facts: path {} is not under tree root {}",
                path.display(),
                top_root.display()
            )
        })
        .to_string_lossy()
        .replace('\\', "/")
}

fn walk_fact_cardinality_tree_dir(
    top_root: &Path,
    dir: &Path,
    tree: &str,
    records: &mut Vec<FactCardinalityDeclFactRaw>,
) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "fact_cardinality_decl_facts: failed to read dir {}: {e}",
            dir.display()
        )
    });
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_fact_cardinality_tree_dir(top_root, &path, tree, records);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("dag") {
            continue;
        }
        let rel = rel_path_within_tree(top_root, &path);
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "fact_cardinality_decl_facts: failed to read {}: {e}",
                path.display()
            )
        });
        for (name, hash) in extract_top_level_decls(&content) {
            records.push(FactCardinalityDeclFactRaw {
                rel_path_decl_key: format!("{rel}:{name}"),
                tree: tree.to_string(),
                content_hash: hash,
            });
        }
    }
}

fn walk_fact_cardinality_tree(
    top_root: &Path,
    tree: &str,
    records: &mut Vec<FactCardinalityDeclFactRaw>,
) {
    if !top_root.is_dir() {
        panic!(
            "fact_cardinality_decl_facts: tree root {} does not exist",
            top_root.display()
        );
    }
    walk_fact_cardinality_tree_dir(top_root, top_root, tree, records);
}

pub fn fact_cardinality_decl_facts() -> Vec<FactCardinalityDeclFactRaw> {
    let ws = workspace_root();
    let mut records = Vec::new();
    for root in witness_layer_roots() {
        let tree = Path::new(&root)
            .file_name()
            .expect("ci_layer_roots: each root must have a file_name component")
            .to_string_lossy()
            .into_owned();
        walk_fact_cardinality_tree(&ws.join(&root), &tree, &mut records);
    }
    records
}

pub struct ImportResolutionFactRaw {
    pub path: String,
    pub import_module: String,
    pub target_declared: bool,
}

pub struct ModuleDeclarationFactRaw {
    pub module: String,
    pub path: String,
}

fn is_excluded_import_path(rel: &str, exclude_substrings: &[String]) -> bool {
    exclude_substrings.iter().any(|s| rel.contains(s.as_str()))
}

pub fn import_resolution_facts(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ImportResolutionFactRaw> {
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let abs_importer_roots = pool_roots_abs(importer_roots);
    let declared: HashSet<String> = build_module_path_index(&abs_pool_roots)
        .into_keys()
        .collect();
    let mut out = Vec::new();
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

pub fn module_declaration_facts(pool_roots: &[String]) -> Vec<ModuleDeclarationFactRaw> {
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let mut out: Vec<ModuleDeclarationFactRaw> = build_module_path_index(&abs_pool_roots)
        .into_iter()
        .map(|(module, path)| ModuleDeclarationFactRaw { module, path })
        .collect();
    out.sort_by(|a, b| a.module.cmp(&b.module));
    out
}

// ── Non-fold-residue census (DESIGN §6) ──────────────────────────────────────────────────────────
//
// Audits the corpus for `match` expressions whose scrutinee is a function parameter with a declared
// closed-coproduct type AND whose body has a top-level `_ =>` wildcard arm.
//
// Host-fed; DISSOLUTION: folds into a pure `.dag` Node-tree reader (match nodes + scrutinee type)
// when exhaustiveness-by-default / compile-graph access lands (gunbc#5364).

const NON_FOLD_RESIDUE_ROSTER: &[&str] = &[
    "dag/extdeps/bmc/webui/nbd_proxy_serve.dag::shell_command_leading_lit_text",
    "dag/extdeps/bmc/webui/nbd_proxy_serve.dag::shell_rawline_starts_with_tool",
    "dag/extdeps/languages/markdown.dag::md_nested",
    "dag/gunbc/generated_artifact.dag::artifact_eq",
    "dag/gunbc/commit_workflow.dag::commit_workflow_surface_eq",
    "dag/gunbc/commit_workflow.dag::gate_eq",
    "dag/gunbc/commit_workflow.dag::local_tidy_check_eq",
    "dag/gunbc/os_install_deduction.dag::runtime_verdict_from_kvm_attestation",
    "dag/gunbc/runner_unit_live_read.dag::converge_target_live_verdict",
    "dag/gunbc/srv3_bmc_credential_resolve.dag::bmc_credential_resolution_uses_factory",
    "dag/gunbc/srv3_bmc_credential_resolve.dag::bmc_credential_resolution_uses_secret_ref",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_boot_override_consumed_or_weak",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_srv3_install_post_boot",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_srv3_install_when_serve_observed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_srv3_install_when_serve_ready",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_weak_kvm_or_inconclusive",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_when_router_not_installed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_when_serve_ready",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_diagnostic_verdict_is_boot_override_consumed_failure",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_diagnostic_verdict_is_os_installed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_diagnostic_verdict_is_ready_to_boot",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_has_progress_evidence",
    "dag/gunbc/srv3_os_install_diagnostic.dag::parse_virtual_media_session_observation",
    "dag/gunbc/srv3_os_install_diagnostic.dag::router_lacks_os_installed_lease",
    "dag/gunbc/srv3_os_install_diagnostic.dag::sol_has_autoinstall_evidence",
    "dag/gunbc/srv3_os_install_diagnostic.dag::srv3_install_diagnostic_is_boot_override_consumed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::srv3_install_diagnostic_is_os_installed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::srv3_install_diagnostic_is_ready_to_boot",
    "dag/std/change.dag::keyed_diff_hunks_equal",
    "dag/std/computation.dag::constant_bound_value",
    "dag/std/computation.dag::is_constant_bound",
    "dag/std/effects.dag::create_double_init_collapsible",
    "dag/std/effects.dag::create_effect_is_dedupable",
    "dag/std/effects.dag::key_source_eq",
    "dag/std/encoding.dag::encoding_lattice_join",
    "dag/std/encoding.dag::encoding_lattice_meet",
    "dag/std/filesystem.dag::is_text_encoding",
    "dag/std/induction.dag::compose_sub_value",
    "dag/std/induction.dag::compose_sub_value_relations",
    "dag/std/induction.dag::is_strict_style_structural",
    "dag/std/induction.dag::recursion_shape_eq",
    "dag/std/induction.dag::shrink_factor_eq",
    "dag/std/induction.dag::sub_value_structural_eq",
    "dag/std/reducible.dag::reduce_verdict_combine",
    "dag/std/termination.dag::descent_evidence_lattice_join",
    "dag/std/termination.dag::descent_evidence_lattice_meet",
    "dag/std/termination.dag::promote_to_strict",
    "dag/tools/ci_gates.dag::exit_ok",
    "dag/tools/generated_artifact_gate.dag::exit_ok",
    "src/v2/compiler/01_tokenize.dag::lex_try_rules_prefer_longer",
    "src/v2/compiler/05_eval.dag::eval_branch_node_eval",
    "src/v2/compiler/05_eval.dag::eval_loop_node",
    "src/v2/compiler/05_eval.dag::eval_match_node_eval",
    "src/v2/compiler/05_eval.dag::eval_transform_node",
    "src/v2/compiler/05_eval.dag::eval_value_node",
    "src/v2/compiler/05_eval.dag::run_test_claim_assert_decided",
    "src/v2/compiler/05_eval.dag::run_test_claim_runtime_assert",
    "src/v2/compiler/06_translate.dag::translate_algebra_finalize",
    "src/v2/compiler/emit_host.dag::run_test_claim_emit_vs_eval_verdict",
    "src/v2/test/claim/manual/eval_runtime_mvp.dag::eval_mvp2_arg_is_two_literal",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_spec_from_component",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_topology_from_component",
    "src/v2/extdeps/runtimes/v2_effect_io_pure.dag::effect_io_pure_backends_match",
    "src/v2/lens/testgen.dag::algebra_law_subject_for_manual_anchor",
    "src/v2/lens/testgen.dag::nat_manual_anchor_key_eq",
    "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim",
    "src/v2/lens/testgen.dag::testgen_emit_refinement_preservation_claim",
    "src/v2/test/claim/generated/coproduct_exhaustiveness.dag::anchor_is",
    "src/v2/test/claim/generated/cross_representation_equality.dag::anchor_is_straddle",
    "src/v2/lens/complexity.dag::complexity_bound_dominates",
    "src/v2/lens/complexity.dag::complexity_bound_from_class",
    "src/v2/lens/cost.dag::asymptotic_class_dominates",
    "src/v2/lens/cost.dag::multiply_classes",
    "src/v2/lens/cost.dag::symbolic_cost_dominates",
    "src/v2/lens/cost.dag::symbolic_cost_witness",
    "src/v2/lens/cost.dag::symbolic_max",
    "src/v2/lens/cost.dag::symbolic_product",
    "src/v2/lens/cost.dag::symbolic_sequential",
    "src/v2/lens/fact_density.dag::connective_is_kernel_ambient_atom",
    "src/v2/lens/idempotency.dag::idempotency_verdict_eq",
    "src/v2/lens/ownership.dag::ownership_mode_eq",
    "src/v2/lens/parallelism.dag::parallelism_relation_eq",
    "src/v2/lens/registry.dag::lens_id_v0_eq",
    "src/v2/lens/unused_parameters.dag::use_relation_eq",
    "src/v2/program.dag::program_runtime_bool_false",
    "src/v2/program.dag::program_runtime_bool_true",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_bool",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_char",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_string",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_symbol",
    "src/v2/std/compilers/target_model.dag::target_type_expr_emitted_validate_wire_shape",
    "src/v2/std/compilers/target_model.dag::target_use_site_ownership_catalog_lookup_step",
    "src/v2/std/effects.dag::key_source_eq",
    "src/v2/std/determinism.dag::determinism_class_eq",
    "src/v2/std/determinism.dag::non_det_source_eq",
    "src/v2/std/decl_index.dag::decl_facts_is_fn_like",
    "src/v2/std/float.dag::float_body_is_nan",
    "src/v2/std/node_minimal.dag::node_superset_field_eq",
    "src/v2/std/probe_selector.dag::diagnostic_interface_kind_eq",
    "src/v2/std/qualified_name.dag::qn_fold_step",
];

fn nfr_strip_comments(content: &str) -> String {
    content
        .lines()
        .map(strip_line_comment)
        .collect::<Vec<_>>()
        .join("\n")
}

fn nfr_closed_coproduct_names(files: &[(String, String)]) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim_start();
            let Some(rest) = trimmed.strip_prefix("type ") else {
                i += 1;
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                i += 1;
                continue;
            }
            let mut block = String::new();
            block.push_str(&strip_line_comment(lines[i]));
            let mut depth = brace_delta(lines[i]);
            i += 1;
            while i < lines.len() {
                let nt = lines[i].trim_start();
                if depth <= 0 {
                    if nt.is_empty() {
                        i += 1;
                        continue;
                    }
                    if !(nt.starts_with('|') || nt.starts_with('=')) {
                        break;
                    }
                }
                block.push('\n');
                block.push_str(&strip_line_comment(lines[i]));
                depth += brace_delta(lines[i]);
                i += 1;
            }
            if block.contains('|') {
                out.insert(name);
            }
        }
    }
    out
}

fn nfr_is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().unwrap().is_ascii_digit()
}

fn nfr_matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

fn nfr_has_top_level_wildcard_arm(body: &str) -> bool {
    let bytes = body.as_bytes();
    let mut depth = 0i32;
    let mut k = 0;
    while k < bytes.len() {
        match bytes[k] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'_' => {
                let prev_ok = k == 0 || !nfr_is_ident_byte(bytes[k - 1]);
                let next_is_ident = k + 1 < bytes.len() && nfr_is_ident_byte(bytes[k + 1]);
                if depth == 0 && prev_ok && !next_is_ident {
                    let mut m = k + 1;
                    while m < bytes.len()
                        && (bytes[m] == b' ' || bytes[m] == b'\n' || bytes[m] == b'\t')
                    {
                        m += 1;
                    }
                    if m + 1 < bytes.len() && bytes[m] == b'=' && bytes[m + 1] == b'>' {
                        return true;
                    }
                }
            }
            _ => {}
        }
        k += 1;
    }
    false
}

fn nfr_is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

struct NfrFnSig {
    name: String,
    params: std::collections::BTreeMap<String, String>,
    body: String,
}

fn nfr_parse_fns(src: &str) -> Vec<NfrFnSig> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for (start, _) in src.match_indices("fn ") {
        if start > 0 && nfr_is_ident_byte(bytes[start - 1]) {
            continue;
        }
        let after = start + 3;
        let name: String = src[after..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let paren_open = match src[after..].find('(') {
            Some(p) => after + p,
            None => continue,
        };
        let paren_close = match nfr_matching_paren(bytes, paren_open) {
            Some(p) => p,
            None => continue,
        };
        let params = nfr_parse_params(&src[paren_open + 1..paren_close]);
        let brace_open = match src[paren_close..].find('{') {
            Some(b) => paren_close + b,
            None => continue,
        };
        let brace_close = match nfr_matching_brace(bytes, brace_open) {
            Some(b) => b,
            None => continue,
        };
        out.push(NfrFnSig {
            name,
            params,
            body: src[brace_open + 1..brace_close].to_string(),
        });
    }
    out
}

fn nfr_matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

fn nfr_parse_params(s: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut parts: Vec<String> = Vec::new();
    for ch in s.chars() {
        match ch {
            '<' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '>' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }
    for part in parts {
        let Some((name, ty)) = part.split_once(':') else {
            continue;
        };
        let name = name.trim();
        let ty_head: String = ty
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if nfr_is_ident(name) && !ty_head.is_empty() {
            out.insert(name.to_string(), ty_head);
        }
    }
    out
}

fn nfr_residue_sites(files: &[(String, String)]) -> Vec<String> {
    let coproducts = nfr_closed_coproduct_names(files);
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        let src = nfr_strip_comments(content);
        for sig in nfr_parse_fns(&src) {
            for (mi, _) in sig.body.match_indices("match ") {
                if mi > 0 && nfr_is_ident_byte(sig.body.as_bytes()[mi - 1]) {
                    continue;
                }
                let after = mi + "match ".len();
                let Some(brace_rel) = sig.body[after..].find('{') else {
                    continue;
                };
                let scrut = sig.body[after..after + brace_rel].trim();
                if !nfr_is_ident(scrut) {
                    continue;
                }
                let Some(ty) = sig.params.get(scrut) else {
                    continue;
                };
                if !coproducts.contains(ty) {
                    continue;
                }
                let body_bytes = sig.body.as_bytes();
                let brace_abs = after + brace_rel;
                let Some(close) = nfr_matching_brace(body_bytes, brace_abs) else {
                    continue;
                };
                let body = &sig.body[brace_abs + 1..close];
                if nfr_has_top_level_wildcard_arm(body) {
                    out.insert(format!("{}::{}", rel, sig.name));
                }
            }
        }
    }
    out.into_iter().collect()
}

struct NonFoldReport {
    sites: Vec<String>,
    coproduct_universe: usize,
    closed_coproduct_names: std::collections::BTreeSet<String>,
}

fn nfr_build_report() -> &'static NonFoldReport {
    static REPORT: std::sync::OnceLock<NonFoldReport> = std::sync::OnceLock::new();
    REPORT.get_or_init(|| {
        let files = corpus_dag_files();
        let closed_coproduct_names = nfr_closed_coproduct_names(&files);
        NonFoldReport {
            sites: nfr_residue_sites(&files),
            coproduct_universe: closed_coproduct_names.len(),
            closed_coproduct_names,
        }
    })
}

pub fn non_fold_residue_closed_coproduct_type_names() -> &'static std::collections::BTreeSet<String>
{
    &nfr_build_report().closed_coproduct_names
}

pub fn non_fold_residue_count() -> i64 {
    nfr_build_report().sites.len() as i64
}

pub fn non_fold_residue_unrostered_count() -> i64 {
    let roster: std::collections::BTreeSet<&str> =
        NON_FOLD_RESIDUE_ROSTER.iter().copied().collect();
    nfr_build_report()
        .sites
        .iter()
        .filter(|s| !roster.contains(s.as_str()))
        .count() as i64
}

pub fn non_fold_residue_site_is_rostered(site: &str) -> bool {
    NON_FOLD_RESIDUE_ROSTER.contains(&site)
}

pub fn non_fold_residue_stale_roster_count() -> i64 {
    let live: std::collections::BTreeSet<&str> = nfr_build_report()
        .sites
        .iter()
        .map(|s| s.as_str())
        .collect();
    NON_FOLD_RESIDUE_ROSTER
        .iter()
        .filter(|s| !live.contains(*s))
        .count() as i64
}

pub fn non_fold_residue_coproduct_universe_count() -> i64 {
    nfr_build_report().coproduct_universe as i64
}

pub fn non_fold_residue_live_sites() -> &'static [String] {
    &nfr_build_report().sites
}

pub fn non_fold_residue_roster_size() -> i64 {
    NON_FOLD_RESIDUE_ROSTER.len() as i64
}

#[cfg(test)]
mod nfr_tests {
    use super::*;

    fn files(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect()
    }

    #[test]
    fn coproduct_index_finds_sums_not_records() {
        let f = files(&[(
            "t.dag",
            "module t\ntype Mode = A | B | C\ntype Rec { x: Int }\ntype Alias = Witness<Int>\n",
        )]);
        let cps = nfr_closed_coproduct_names(&f);
        assert!(cps.contains("Mode"));
        assert!(!cps.contains("Rec"));
        assert!(!cps.contains("Alias"));
    }

    #[test]
    fn red_control_wildcard_over_closed_coproduct_is_residue() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            sites.contains(&"m.dag::f".to_string()),
            "a wildcard over a closed-coproduct param must be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_total_fold_is_not_residue() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    B => false\n    C => false\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "an exhaustive match (no wildcard) must NOT be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_wildcard_over_open_domain_is_not_residue() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn g(s: String) -> Bool {\n  match s {\n    \"y\" => true\n    _ => false\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::g".to_string()),
            "a wildcard over an open/primitive domain must NOT be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_field_placeholder_underscore_is_not_a_wildcard_arm() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A { v: Int } | B { v: Int }\nfn f(x: Mode) -> Int {\n  match x {\n    A { v: _ } => 1\n    B { v: _ } => 2\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "field-placeholder `_` is not a wildcard arm; got {sites:?}"
        );
    }

    #[test]
    fn nested_match_wildcard_is_attributed_to_its_own_match() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn eq(a: Mode, b: Mode) -> Bool {\n  match a {\n    A => match b { A => true _ => false }\n    B => match b { B => true _ => false }\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(sites.contains(&"m.dag::eq".to_string()));
    }

    #[test]
    fn green_control_wildcard_and_slashes_inside_string_literal_are_ignored() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn f(x: Mode) -> String {\n  match x {\n    A => \"see https://x/y and _ => z\"\n    B => \"b\"\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "`_ =>`/`//` inside a string literal must not be read as code; got {sites:?}"
        );
    }

    #[test]
    fn red_control_real_wildcard_survives_an_in_string_decoy() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> String {\n  match x {\n    A => \"see https://x/y and _ => z\"\n    _ => \"rest\"\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            sites.contains(&"m.dag::f".to_string()),
            "a real wildcard arm must still be flagged despite an in-string decoy; got {sites:?}"
        );
    }
}

const LANGUAGES_AUTHORITY_REL: &str = "dag/std/languages.dag";

fn languages_census_collect_source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            languages_census_collect_source_files(&path, out);
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "dag" || ext == "rs" {
                out.push(path);
            }
        }
    }
}

fn languages_census_strip_content(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            while chars.next().is_some_and(|ch| ch != '\n') {}
            out.push('\n');
            continue;
        }
        if c == '"' {
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    chars.next();
                    continue;
                }
                if ch == '"' {
                    break;
                }
            }
            out.push(' ');
            continue;
        }
        if c == '`' {
            while chars.next().is_some_and(|ch| ch != '`') {}
            out.push(' ');
            continue;
        }
        out.push(c);
    }
    out
}

fn languages_census_extract_data_decl_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("data ")?;
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

fn languages_census_is_infrastructure_path(rel: &str) -> bool {
    rel.starts_with("src/v2/test/claim/languages_consumer_census/")
        || rel == "src/v2/lens/languages_consumer_census.dag"
}

fn languages_census_tokenize(content: &str) -> HashSet<String> {
    let stripped = languages_census_strip_content(content);
    stripped
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguagesDeclConsumerRecord {
    pub decl_name: String,
    pub external_consumer_paths: Vec<String>,
}

fn languages_decl_records_inner() -> Vec<LanguagesDeclConsumerRecord> {
    let ws = workspace_root();
    let authority = ws.join(LANGUAGES_AUTHORITY_REL);
    let authority_content = std::fs::read_to_string(&authority).unwrap_or_else(|e| {
        panic!(
            "languages_consumer_census: failed to read {}: {e}",
            authority.display()
        )
    });
    let decl_names = languages_census_extract_data_decl_names(&authority_content);
    let decl_name_set: HashSet<String> = decl_names.iter().cloned().collect();

    let mut files = Vec::new();
    for tree in &["dag", "src"] {
        let root = ws.join(tree);
        if root.is_dir() {
            languages_census_collect_source_files(&root, &mut files);
        }
    }

    let mut by_decl: HashMap<String, HashSet<String>> = decl_names
        .iter()
        .map(|name| (name.clone(), HashSet::new()))
        .collect();

    for path in files {
        let rel = path
            .strip_prefix(&ws)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if rel == LANGUAGES_AUTHORITY_REL || languages_census_is_infrastructure_path(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let tokens = languages_census_tokenize(&content);
        for decl_name in tokens.intersection(&decl_name_set) {
            by_decl
                .get_mut(decl_name)
                .expect("decl map key")
                .insert(rel.clone());
        }
    }

    let mut records = Vec::new();
    for decl_name in decl_names {
        let mut paths: Vec<String> = by_decl
            .remove(&decl_name)
            .expect("decl map key")
            .into_iter()
            .collect();
        paths.sort();
        records.push(LanguagesDeclConsumerRecord {
            decl_name,
            external_consumer_paths: paths,
        });
    }
    records
}

fn languages_decl_records_cached() -> &'static [LanguagesDeclConsumerRecord] {
    static RECORDS: OnceLock<Vec<LanguagesDeclConsumerRecord>> = OnceLock::new();
    RECORDS.get_or_init(languages_decl_records_inner)
}

fn languages_decl_record_for(decl_name: &str) -> Option<&'static LanguagesDeclConsumerRecord> {
    languages_decl_records_cached()
        .iter()
        .find(|r| r.decl_name == decl_name)
}

pub fn languages_consumer_census_data_decl_count() -> i64 {
    languages_decl_records_cached().len() as i64
}

pub fn languages_consumer_census_per_language_row_count() -> i64 {
    languages_decl_records_cached()
        .iter()
        .filter(|r| !r.decl_name.ends_with("_format"))
        .count() as i64
}

pub fn languages_consumer_census_format_row_count() -> i64 {
    languages_decl_records_cached()
        .iter()
        .filter(|r| r.decl_name.ends_with("_format"))
        .count() as i64
}

pub fn languages_consumer_census_external_consumer_count(decl_name: String) -> i64 {
    languages_decl_record_for(&decl_name)
        .map(|r| r.external_consumer_paths.len() as i64)
        .unwrap_or(-1)
}

pub fn languages_consumer_census_is_composition_only(decl_name: String) -> bool {
    languages_decl_record_for(&decl_name)
        .map(|r| r.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

pub fn languages_consumer_census_has_external_consumer(decl_name: String) -> bool {
    languages_decl_record_for(&decl_name)
        .map(|r| !r.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

// --- Inert carrier census (folded from inert_carrier_project.rs) ---
//
// A type carrier is "inert" iff (a) declared in a non-test file, (b) its name appears in at least
// one *_test.dag file (self-tested), and (c) its name appears in NO non-test .dag file outside its
// own declaration block (zero real consumer). This is DESIGN §5 coverage-by-illusion.
// DISSOLUTION TRIGGER: when .dag gains compile-graph / reference-edge access (gunbc#5364), the
// token scan folds into a pure .dag reader over BindsTo edges and this Rust census deletes.

fn inert_carrier_identifier_tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in line.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn inert_carrier_count_token(text: &str, name: &str) -> i64 {
    let mut n = 0i64;
    for raw in text.lines() {
        for tok in inert_carrier_identifier_tokens(&strip_line_comment(raw)) {
            if tok == name {
                n += 1;
            }
        }
    }
    n
}

fn inert_carrier_type_carrier_blocks(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let Some(rest) = trimmed.strip_prefix("type ") else {
            i += 1;
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut block = String::new();
        block.push_str(lines[i]);
        block.push('\n');
        let mut depth = brace_delta(lines[i]);
        i += 1;
        while i < lines.len() {
            let nt = lines[i].trim_start();
            if depth <= 0 {
                if !(nt.starts_with('|') || nt.starts_with('=')) {
                    break;
                }
            }
            block.push_str(lines[i]);
            block.push('\n');
            depth += brace_delta(lines[i]);
            i += 1;
        }
        out.push((name, block));
    }
    out
}

const DOC_PLAN_ROOTS: &[&str] = &["ROADMAP.md", "DESIGN.md"];
const DOC_RUNBOOK_ROOT: &str = "docs/runbooks/README.md";

fn doc_repo_rel(path: &Path) -> String {
    let ws = workspace_root();
    let s = path.to_string_lossy().replace('\\', "/");
    let prefix = format!("{}/", ws.to_string_lossy().replace('\\', "/"));
    s.strip_prefix(&prefix)
        .map(|p| p.to_string())
        .unwrap_or(s)
        .trim_start_matches("./")
        .to_string()
}

fn doc_universe() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let docs_dir = workspace_root().join("docs");
    collect_md_files(&docs_dir, &mut out);
    out
}

fn collect_md_files(dir: &Path, out: &mut BTreeSet<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.insert(doc_repo_rel(&path));
        }
    }
}

fn markdown_link_targets(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' {
            if let Some(end) = content[i + 2..].find(')') {
                let raw = &content[i + 2..i + 2 + end];
                let target = raw.split('#').next().unwrap_or("").trim();
                if !target.is_empty()
                    && !target.starts_with("http://")
                    && !target.starts_with("https://")
                    && !target.starts_with("mailto:")
                {
                    out.push(target.to_string());
                }
                i = i + 2 + end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

struct InertCarrierData {
    declared_count: usize,
    inert_names: Vec<String>,
}

fn compute_inert_carrier_data(files: &[(String, String)]) -> InertCarrierData {
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    let mut decl_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut self_block_refs: BTreeMap<String, i64> = BTreeMap::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        for (name, block) in inert_carrier_type_carrier_blocks(content) {
            declared.entry(name.clone()).or_insert_with(|| rel.clone());
            *decl_count.entry(name.clone()).or_insert(0) += 1;
            *self_block_refs.entry(name.clone()).or_insert(0) +=
                inert_carrier_count_token(&block, &name);
        }
    }
    let names: BTreeSet<String> = declared.keys().cloned().collect();
    let mut nontest_occ: BTreeMap<String, i64> = BTreeMap::new();
    let mut self_tested: BTreeSet<String> = BTreeSet::new();
    for (rel, content) in files {
        let mut local: BTreeMap<String, i64> = BTreeMap::new();
        for raw in content.lines() {
            for tok in inert_carrier_identifier_tokens(&strip_line_comment(raw)) {
                if names.contains(&tok) {
                    *local.entry(tok).or_insert(0) += 1;
                }
            }
        }
        if is_test_dag(rel) {
            for (k, _) in local {
                self_tested.insert(k);
            }
        } else {
            for (k, v) in local {
                *nontest_occ.entry(k).or_insert(0) += v;
            }
        }
    }
    let mut inert_names: Vec<String> = Vec::new();
    for name in declared.keys() {
        if decl_count.get(name).copied().unwrap_or(0) != 1 {
            continue;
        }
        if !self_tested.contains(name) {
            continue;
        }
        let total = nontest_occ.get(name).copied().unwrap_or(0);
        let own = self_block_refs.get(name).copied().unwrap_or(0);
        if total - own <= 0 {
            inert_names.push(name.clone());
        }
    }
    inert_names.sort();
    inert_names.dedup();
    InertCarrierData {
        declared_count: declared.len(),
        inert_names,
    }
}

fn build_inert_carrier_data() -> &'static InertCarrierData {
    static CACHE: OnceLock<InertCarrierData> = OnceLock::new();
    CACHE.get_or_init(|| compute_inert_carrier_data(&corpus_dag_files()))
}

pub fn inert_carrier_names_live() -> Vec<String> {
    build_inert_carrier_data().inert_names.clone()
}

pub fn inert_carrier_declared_count_live() -> i64 {
    build_inert_carrier_data().declared_count as i64
}

#[cfg(test)]
mod inert_carrier_tests {
    use super::*;

    fn inert_names_of(files: &[(&str, &str)]) -> Vec<String> {
        let owned: Vec<(String, String)> = files
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect();
        compute_inert_carrier_data(&owned).inert_names
    }

    #[test]
    fn type_carrier_blocks_extracts_names_and_bodies() {
        let c = "module m\ntype Connective = Atom | Conj\ntype WorkDemand {\n  field: Int\n}\nfn f() -> Int { 1 }\n";
        let blocks = inert_carrier_type_carrier_blocks(c);
        let names: Vec<&String> = blocks.iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["Connective", "WorkDemand"]);
        let wd = &blocks.iter().find(|(n, _)| n == "WorkDemand").unwrap().1;
        assert!(wd.contains("field: Int") && wd.contains('}'));
        assert!(!wd.contains("fn f"));
    }

    #[test]
    fn identifier_tokens_are_whole_words() {
        let toks = inert_carrier_identifier_tokens("  field: PlacementSupply = foo(Placement)");
        assert!(toks.contains(&"PlacementSupply".to_string()));
        assert!(toks.contains(&"Placement".to_string()));
        assert!(toks.contains(&"field".to_string()));
    }

    #[test]
    fn red_control_self_tested_zero_consumer_carrier_is_inert() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Lonely { x: Int }\n"),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Lonely { x: 1 } == Lonely { x: 1 } }\n",
            ),
        ]);
        assert!(
            inert.contains(&"Lonely".to_string()),
            "a self-tested carrier with no real consumer must be flagged inert; got {inert:?}"
        );
    }

    #[test]
    fn green_control_carrier_with_real_consumer_is_not_inert() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Used { x: Int }\n"),
            (
                "b.dag",
                "module b\nimport a { Used }\nfn f(u: Used) -> Int { u.x }\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Used { x: 1 } == Used { x: 1 } }\n",
            ),
        ]);
        assert!(
            !inert.contains(&"Used".to_string()),
            "a carrier with a real (non-test, cross-file) consumer must NOT be flagged; got {inert:?}"
        );
    }

    #[test]
    fn green_control_same_file_consumer_is_not_inert() {
        let inert = inert_names_of(&[
            (
                "lens.dag",
                "module lens\ntype LocalFact { x: Int }\nfn clean(fs: LocalFact) -> Bool { fs.x == 0 }\n",
            ),
            ("lens_test.dag", "module t\nfn t() -> Bool { clean(fs: LocalFact { x: 0 }) }\n"),
        ]);
        assert!(
            !inert.contains(&"LocalFact".to_string()),
            "a carrier consumed by a fn in its own file is NOT inert; got {inert:?}"
        );
    }

    #[test]
    fn green_control_untested_unused_carrier_is_not_flagged() {
        let inert = inert_names_of(&[("a.dag", "module a\ntype Staged { x: Int }\n")]);
        assert!(
            !inert.contains(&"Staged".to_string()),
            "an untested unused carrier must NOT be flagged (it is model-first, not illusion); got {inert:?}"
        );
    }

    #[test]
    fn comment_reference_is_not_a_real_consumer() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Noted { x: Int }\n"),
            (
                "b.dag",
                "module b\n// Noted is described here\nfn f() -> Int { 1 }\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Noted { x: 1 } == Noted { x: 1 } }\n",
            ),
        ]);
        assert!(inert.contains(&"Noted".to_string()));
    }

    #[test]
    fn doubly_declared_name_is_not_flagged() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Dup { x: Int }\n"),
            ("b.dag", "module b\ntype Dup { y: Int }\n"),
        ]);
        assert!(!inert.contains(&"Dup".to_string()));
    }
}

// --- Complexity/linearity syntactic audit (folded from complexity_linearity_audit_project.rs) ---
//
// Thin host builtins over `decl_facts` + fn-body AST walk. Triage/bucket classification and the
// migration-debt roster live in `v2.lens.complexity_linearity_audit` (.dag).
// REMAINING GATE (#5364 partial): `decl_facts` exposes corpus `Node`s but v2 `.dag` has no
// `expr_data` / `MatchPattern` introspection — the wildcard-arm walk stays in this host seam
// until a `.dag`-accessible match-body reader lands (same residue class as inert_carrier_*).

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComplexityLinearityAuditFinding {
    pub site: String,
    pub lens: &'static str,
    pub rule: &'static str,
    pub triage: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct ComplexityLinearityAuditSummary {
    pub files_scanned: usize,
    pub files_parsed: usize,
    pub fns_scanned: usize,
    pub findings: Vec<ComplexityLinearityAuditFinding>,
}

fn cla_is_wildcard_arm(arm: &Rc<Node>) -> bool {
    matches!(arm_pattern(arm.clone()).as_ref(), MatchPattern::Wildcard)
}

fn cla_type_expr_head(ty: Rc<Node>, si: &Rc<HashMap<String, Rc<NewlineIndex>>>) -> String {
    let name = authored_name_at(si.clone(), ty);
    name.chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

fn cla_fn_param_type_heads(
    item: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for param in item.params.iter() {
        let pname = param_node_name_at(param.clone(), si.clone());
        if pname.is_empty() {
            continue;
        }
        let head = cla_type_expr_head(param_node_type_expr(param.clone()), si);
        if !head.is_empty() {
            out.insert(pname, head);
        }
    }
    out
}

fn cla_is_closed_coproduct_param_scrutinee(
    scrutinee_name: &str,
    param_types: &BTreeMap<String, String>,
    closed: &BTreeSet<String>,
) -> bool {
    param_types
        .get(scrutinee_name)
        .is_some_and(|ty| closed.contains(ty))
}

#[derive(Default)]
struct ClaFnBodyStats {
    node_count: usize,
    match_count: usize,
    wildcard_matches: usize,
    closed_coproduct_wildcard_matches: usize,
}

fn cla_walk_expr(
    node: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    param_types: &BTreeMap<String, String>,
    closed_coproducts: &BTreeSet<String>,
    stats: &mut ClaFnBodyStats,
) {
    stats.node_count += 1;
    if let ExprData::ExprMatch = node.expr_data.as_ref() {
        stats.match_count += 1;
        let scrutinee = match_scrutinee(node.clone());
        let scrutinee_name = expr_var_name_at(scrutinee, si.clone());
        let has_wildcard = match_arm_nodes(node.clone())
            .iter()
            .any(|arm| cla_is_wildcard_arm(arm));
        if has_wildcard {
            stats.wildcard_matches += 1;
            if !scrutinee_name.is_empty()
                && cla_is_closed_coproduct_param_scrutinee(
                    &scrutinee_name,
                    param_types,
                    closed_coproducts,
                )
            {
                stats.closed_coproduct_wildcard_matches += 1;
            }
        }
    }
    for child in node.children.iter() {
        cla_walk_expr(child, si, param_types, closed_coproducts, stats);
    }
}

fn cla_is_kernel_permanent_fn(fn_name: &str) -> bool {
    fn_name.ends_with("_eq")
        || fn_name.contains("dominates")
        || fn_name.contains("lattice_join")
        || fn_name.contains("lattice_meet")
        || fn_name == "exit_ok"
        || fn_name.contains("_relation_eq")
        || fn_name.contains("_mode_eq")
        || fn_name.ends_with("_combine")
        || fn_name == "constant_bound_value"
        || fn_name == "is_constant_bound"
        || fn_name == "create_double_init_collapsible"
        || fn_name == "create_effect_is_dedupable"
        || fn_name.starts_with("compose_sub_value")
        || fn_name == "promote_to_strict"
        || fn_name.starts_with("program_runtime_bool")
        || fn_name == "is_text_encoding"
        || fn_name == "is_strict_style_structural"
}

fn cla_triage_complexity(site: &str) -> &'static str {
    let fn_name = site.rsplit("::").next().unwrap_or("");
    if cla_is_kernel_permanent_fn(fn_name) {
        return "kernel-permanent";
    }
    if site.starts_with("dag/extdeps/")
        || site.starts_with("dag/ctrl/")
        || site.starts_with("dag/gunbc/plans/")
        || site.starts_with("dag/test/")
    {
        "open-domain"
    } else if site.starts_with("dag/std/") || site.starts_with("dag/gunbc/") {
        "kernel-permanent"
    } else {
        "open-domain"
    }
}

fn cla_audit_function_body(
    rel: &str,
    fn_name: &str,
    body: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    param_types: &BTreeMap<String, String>,
) -> Vec<ComplexityLinearityAuditFinding> {
    let closed = non_fold_residue_closed_coproduct_type_names();
    let mut stats = ClaFnBodyStats::default();
    cla_walk_expr(body, si, param_types, closed, &mut stats);
    let site = format!("{rel}::{fn_name}");
    let mut out = Vec::new();
    if stats.wildcard_matches > 0 {
        out.push(ComplexityLinearityAuditFinding {
            site: site.clone(),
            lens: "non_fold_residue",
            rule: "syntactic_match_wildcard_arm",
            triage: "wildcard-arm",
        });
    }
    if stats.match_count >= 8 || (stats.node_count >= 200 && stats.match_count >= 4) {
        out.push(ComplexityLinearityAuditFinding {
            site,
            lens: "cost",
            rule: "syntactic_high_match_fanout",
            triage: cla_triage_complexity(&format!("{rel}::{fn_name}")),
        });
    }
    out
}

fn cla_audit_decl_fact(
    fact: &DeclFactRaw,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<ComplexityLinearityAuditFinding> {
    let Some(body) = fact.node.body.as_ref() else {
        return Vec::new();
    };
    let param_types = cla_fn_param_type_heads(&fact.node, si);
    cla_audit_function_body(&fact.rel_path, &fact.name, body, si, &param_types)
}

pub fn complexity_linearity_audit_corpus_over_decl_facts(
    roots: &[String],
) -> ComplexityLinearityAuditSummary {
    let walk = decl_facts_corpus_walk(roots);
    let mut summary = ComplexityLinearityAuditSummary::default();
    summary.files_scanned = walk.files_scanned;
    summary.files_parsed = walk.files_parsed;

    for fact in &walk.facts {
        if !matches!(fact.kind, ItemKind::FnItem | ItemKind::FuncItem) {
            continue;
        }
        summary.fns_scanned += 1;
        summary
            .findings
            .extend(cla_audit_decl_fact(fact, &fact.source_indices));
    }
    summary.findings.sort();
    summary
}

pub fn complexity_linearity_audit_corpus_parse_only(
    roots: &[String],
) -> ComplexityLinearityAuditSummary {
    complexity_linearity_audit_corpus_over_decl_facts(roots)
}

pub fn complexity_linearity_audit_corpus_default_roots() -> ComplexityLinearityAuditSummary {
    complexity_linearity_audit_corpus_parse_only(&witness_layer_roots())
}

struct ClaAuditBuiltinCache {
    finding_count: i64,
    sites: BTreeSet<String>,
}

fn cla_cached_builtin_cache() -> &'static ClaAuditBuiltinCache {
    static CACHE: OnceLock<ClaAuditBuiltinCache> = OnceLock::new();
    CACHE.get_or_init(|| {
        let summary = complexity_linearity_audit_corpus_default_roots();
        ClaAuditBuiltinCache {
            finding_count: summary.findings.len() as i64,
            sites: summary.findings.iter().map(|f| f.site.clone()).collect(),
        }
    })
}

pub fn complexity_linearity_syntactic_finding_count() -> i64 {
    cla_cached_builtin_cache().finding_count
}

pub fn complexity_linearity_syntactic_site_fired(site: &str) -> bool {
    cla_cached_builtin_cache().sites.contains(site)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComplexityLinearityWildcardFactRaw {
    pub site: String,
    pub fn_name: String,
    pub closed_coproduct_wildcard: bool,
    pub rostered: bool,
}

struct ClaWildcardFactsCache {
    facts: Vec<ComplexityLinearityWildcardFactRaw>,
}

fn cla_compute_wildcard_facts(roots: &[String]) -> Vec<ComplexityLinearityWildcardFactRaw> {
    let walk = decl_facts_corpus_walk(roots);
    let closed = non_fold_residue_closed_coproduct_type_names();
    let mut out = Vec::new();
    for fact in &walk.facts {
        if !matches!(fact.kind, ItemKind::FnItem | ItemKind::FuncItem) {
            continue;
        }
        let Some(body) = fact.node.body.as_ref() else {
            continue;
        };
        let param_types = cla_fn_param_type_heads(&fact.node, &fact.source_indices);
        let mut stats = ClaFnBodyStats::default();
        cla_walk_expr(body, &fact.source_indices, &param_types, closed, &mut stats);
        if stats.wildcard_matches > 0 {
            let site = format!("{}::{}", fact.rel_path, fact.name);
            out.push(ComplexityLinearityWildcardFactRaw {
                fn_name: fact.name.clone(),
                closed_coproduct_wildcard: stats.closed_coproduct_wildcard_matches > 0,
                rostered: non_fold_residue_site_is_rostered(&site),
                site,
            });
        }
    }
    out.sort();
    out.dedup();
    out
}

fn cla_cached_wildcard_facts() -> &'static ClaWildcardFactsCache {
    static CACHE: OnceLock<ClaWildcardFactsCache> = OnceLock::new();
    CACHE.get_or_init(|| ClaWildcardFactsCache {
        facts: cla_compute_wildcard_facts(&witness_layer_roots()),
    })
}

pub fn complexity_linearity_wildcard_facts() -> &'static [ComplexityLinearityWildcardFactRaw] {
    &cla_cached_wildcard_facts().facts
}

#[cfg(test)]
mod complexity_linearity_audit_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_module(content: &str) -> String {
        let dir = std::env::temp_dir().join(format!(
            "complexity-linearity-audit-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join("audit_wildcard.dag");
        fs::write(&path, content).expect("write");
        path.to_string_lossy().to_string()
    }

    #[test]
    fn syntactic_wildcard_finding_on_closed_coproduct_match() {
        let path = write_temp_module(
            "module audit_wildcard\n\
             type Mode = A | B | C\n\
             fn f(x: Mode) -> Bool {\n\
               match x {\n\
                 A => true\n\
                 _ => false\n\
               }\n\
             }\n",
        );
        let root = Path::new(&path)
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let summary = complexity_linearity_audit_corpus_parse_only(&[root]);
        assert!(
            summary
                .findings
                .iter()
                .any(|f| { f.rule == "syntactic_match_wildcard_arm" && f.site.contains("::f") }),
            "expected wildcard finding; got {:?}",
            summary.findings
        );
    }

    #[test]
    fn eval_interpreter_handler_is_migration_debt_raw_fact() {
        let facts = complexity_linearity_wildcard_facts();
        let eval_bind_site = "src/v2/compiler/05_eval.dag::eval_bind_node_eval";
        assert!(
            !facts.iter().any(|f| f.site == eval_bind_site),
            "eval_bind_node_eval wildcard dissolved; should not appear in wildcard facts"
        );
        let site = "src/v2/compiler/05_eval.dag::eval_match_node_eval";
        let fact = facts.iter().find(|f| f.site == site);
        assert!(fact.is_some(), "expected wildcard fact for {site}");
        assert!(
            fact.unwrap().rostered,
            "{site} must be rostered (drives migration-debt/kernel-permanent triage in .dag)"
        );
    }

    #[test]
    fn testgen_anchor_match_is_migration_debt_raw_fact() {
        let site = "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim";
        let facts = complexity_linearity_wildcard_facts();
        assert!(
            facts.iter().any(|f| f.site == site),
            "expected wildcard fact for testgen anchor match"
        );
    }

    #[test]
    fn live_tree_parse_audit_runs_over_witness_roots() {
        let summary = complexity_linearity_audit_corpus_default_roots();
        assert!(summary.files_scanned > 100, "corpus walk fail-opened");
        assert!(summary.files_parsed > 50, "parse fail-opened");
        assert!(summary.fns_scanned > 100, "fn scan fail-opened");
        assert!(
            !summary.findings.is_empty(),
            "expected syntactic findings on the live corpus"
        );
    }
}

fn resolve_doc_link(from: &str, target: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let from_dir = Path::new(from).parent().unwrap_or_else(|| Path::new(""));
    candidates.push(normalize_doc_path(&from_dir.join(target)));
    candidates.push(normalize_doc_path(Path::new(target)));
    candidates.dedup();
    candidates
}

fn normalize_doc_path(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for comp in path.to_string_lossy().replace('\\', "/").split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other.to_string()),
        }
    }
    parts.join("/")
}

fn dag_comment_bind_doc_refs() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for root in witness_layer_roots() {
        let mut dag_files = Vec::new();
        collect_dag_files_tolerant(&workspace_root().join(&root), &mut dag_files);
        for path in dag_files {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for target in bind_md_refs(&content) {
                out.insert(target);
            }
        }
    }
    out
}

fn bind_md_refs(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (idx, _) in content.match_indices("bind:") {
        let rest = content[idx + "bind:".len()..].trim_start();
        let token: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ')' && *c != '"' && *c != '`')
            .collect();
        if token.ends_with(".md") {
            out.push(normalize_doc_path(Path::new(&token)));
        }
    }
    out
}

fn doc_reachable_set(
    roots: &BTreeSet<String>,
    edges: &HashMap<String, Vec<String>>,
) -> BTreeSet<String> {
    let mut reached: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    for r in roots {
        if reached.insert(r.clone()) {
            queue.push_back(r.clone());
        }
    }
    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = edges.get(&node) {
            for n in neighbors {
                if reached.insert(n.clone()) {
                    queue.push_back(n.clone());
                }
            }
        }
    }
    reached
}

struct DocGraphReport {
    doc_count: usize,
    orphans: Vec<String>,
    dangling: Vec<(String, String)>,
}

fn build_doc_graph_report() -> DocGraphReport {
    let universe = doc_universe();
    let bind_refs = dag_comment_bind_doc_refs();

    let mut roots: BTreeSet<String> = BTreeSet::new();
    for r in DOC_PLAN_ROOTS {
        roots.insert((*r).to_string());
    }
    if workspace_root().join(DOC_RUNBOOK_ROOT).is_file() {
        roots.insert(DOC_RUNBOOK_ROOT.to_string());
    }
    for b in &bind_refs {
        if universe.contains(b) {
            roots.insert(b.clone());
        }
    }

    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    let mut dangling: Vec<(String, String)> = Vec::new();
    let mut sources: Vec<String> = universe.iter().cloned().collect();
    for r in DOC_PLAN_ROOTS {
        sources.push((*r).to_string());
    }
    if roots.contains(DOC_RUNBOOK_ROOT) {
        sources.push(DOC_RUNBOOK_ROOT.to_string());
    }
    sources.sort();
    sources.dedup();
    for src in &sources {
        let content = match std::fs::read_to_string(workspace_root().join(src)) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut out_edges: Vec<String> = Vec::new();
        for target in markdown_link_targets(&content) {
            let candidates = resolve_doc_link(src, &target);
            let existing = candidates
                .iter()
                .find(|c| workspace_root().join(c).is_file())
                .cloned();
            match existing {
                Some(path) => out_edges.push(path),
                None => {
                    if target.ends_with(".md") {
                        dangling.push((src.clone(), target.clone()));
                    }
                }
            }
        }
        edges.insert(src.clone(), out_edges);
    }

    let reached = doc_reachable_set(&roots, &edges);
    let orphans: Vec<String> = universe
        .iter()
        .filter(|d| !reached.contains(*d))
        .cloned()
        .collect();
    dangling.sort();
    dangling.dedup();
    DocGraphReport {
        doc_count: universe.len(),
        orphans,
        dangling,
    }
}

fn doc_graph_report() -> &'static DocGraphReport {
    static REPORT: OnceLock<DocGraphReport> = OnceLock::new();
    REPORT.get_or_init(build_doc_graph_report)
}

pub fn doc_graph_orphan_count() -> i64 {
    doc_graph_report().orphans.len() as i64
}

pub fn doc_graph_dangling_link_count() -> i64 {
    doc_graph_report().dangling.len() as i64
}

pub fn doc_graph_doc_count() -> i64 {
    doc_graph_report().doc_count as i64
}

// Live derivation of docs/plans/seed-shrink-census.md §5B ("T2 coverage debt"): that table was a
// hand-maintained snapshot of v1 test modules with no floor `*_test.dag` equivalent. This walks
// `src/v1/tests/src/*.rs` (modules containing `#[test]`) and `corpus_dag_files()` (the same
// witness-layer-roots roster the floor uses) and diffs them by stem, so the debt roster tracks
// the live tree instead of drifting the moment either side changes.
struct TestMigrationDebtEntry {
    module: String,
    loc: i64,
    test_fn_count: i64,
}

struct TestMigrationDebtReport {
    entries: Vec<TestMigrationDebtEntry>,
}

fn test_migration_debt_v1_test_dir() -> PathBuf {
    workspace_root().join("src/v1/tests/src")
}

fn test_migration_debt_stem(name: &str) -> String {
    let stem = name
        .strip_suffix(".rs")
        .or_else(|| name.strip_suffix(".dag"))
        .unwrap_or(name);
    stem.strip_suffix("_test").unwrap_or(stem).to_string()
}

fn test_migration_debt_floor_stems() -> Vec<String> {
    let mut stems: Vec<String> = corpus_dag_files()
        .into_iter()
        .map(|(path, _)| path)
        .filter(|p| is_test_dag(p))
        .map(|p| {
            let file_name = std::path::Path::new(&p)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&p)
                .to_string();
            test_migration_debt_stem(&file_name)
        })
        .collect();
    stems.sort();
    stems.dedup();
    stems
}

// Exact-stem equality only. A substring match (either direction) was tried and reviewed
// unsound: e.g. v1 stem "pipeline" (the single largest debt module, 418 `#[test]` fns) is a
// substring of the floor stem "typescript_import_pipeline", so a fuzzy match falsely marked
// the whole module covered — hiding debt rather than counting it. Exact equality is decidable
// and cannot understate debt; it may list a module the operator judges topically covered by a
// differently-named floor witness, which is a correct false-debt (never a false-coverage) bias.
fn test_migration_debt_stem_covered(v1_stem: &str, floor_stems: &[String]) -> bool {
    floor_stems.iter().any(|floor_stem| floor_stem == v1_stem)
}

fn build_test_migration_debt_report() -> TestMigrationDebtReport {
    let dir = test_migration_debt_v1_test_dir();
    let floor_stems = test_migration_debt_floor_stems();
    let mut entries = Vec::new();
    let read_dir = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return TestMigrationDebtReport { entries },
    };
    let mut paths: Vec<std::path::PathBuf> = read_dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "rs").unwrap_or(false))
        .collect();
    paths.sort();
    for path in paths {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Line-anchored so `#[test]` mentioned in a comment/string/doc example doesn't inflate
        // the count (a `content.matches` substring scan would).
        let test_fn_count = content
            .lines()
            .filter(|line| line.trim() == "#[test]")
            .count() as i64;
        if test_fn_count == 0 {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let stem = test_migration_debt_stem(&file_name);
        if test_migration_debt_stem_covered(&stem, &floor_stems) {
            continue;
        }
        entries.push(TestMigrationDebtEntry {
            module: file_name,
            loc: content.lines().count() as i64,
            test_fn_count,
        });
    }
    TestMigrationDebtReport { entries }
}

fn test_migration_debt_report() -> &'static TestMigrationDebtReport {
    static REPORT: OnceLock<TestMigrationDebtReport> = OnceLock::new();
    REPORT.get_or_init(build_test_migration_debt_report)
}

pub fn test_migration_debt_module_count() -> i64 {
    test_migration_debt_report().entries.len() as i64
}

pub fn test_migration_debt_total_loc() -> i64 {
    test_migration_debt_report()
        .entries
        .iter()
        .map(|e| e.loc)
        .sum()
}

pub fn test_migration_debt_total_test_fns() -> i64 {
    test_migration_debt_report()
        .entries
        .iter()
        .map(|e| e.test_fn_count)
        .sum()
}

pub fn test_migration_debt_module_names() -> Vec<String> {
    test_migration_debt_report()
        .entries
        .iter()
        .map(|e| e.module.clone())
        .collect()
}

// Discriminating red witness for the stem matcher: `witness_option_bridge_test.rs` has a live
// floor counterpart (`witness_option_bridge_test.dag`) and must NOT appear in the debt roster.
// This goes red if the matcher regresses to comparing an un-stripped `.dag` suffix against a
// stripped `.rs` stem (as it did before this function existed), since every module would then
// spuriously report as debt.
pub fn test_migration_debt_known_covered_module_is_not_debt() -> bool {
    !test_migration_debt_module_names()
        .iter()
        .any(|m| m == "witness_option_bridge_test.rs")
}

// §5 hard gate per module at delete time: any `#[test]`-bearing v1 module deleted in the CI
// diff must already have an exact-stem floor `*_test.dag` witness on HEAD (same stem rule as the
// live debt roster). Uses the same `GUNBC_CI_DIFF_*` endpoints as `floor_diff_observe`.
fn test_migration_delete_guard_diff_endpoints() -> (String, String) {
    let base = std::env::var("GUNBC_CI_DIFF_BASE").unwrap_or_else(|_| "origin/main".to_string());
    let head = std::env::var("GUNBC_CI_DIFF_HEAD").unwrap_or_else(|_| "HEAD".to_string());
    (base, head)
}

fn test_migration_delete_guard_merge_base_mode() -> bool {
    match std::env::var("GUNBC_CI_DIFF_MERGE_BASE") {
        Ok(v) => v != "0" && v != "false",
        Err(_) => true,
    }
}

fn test_migration_delete_guard_run_git(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(workspace_root())
        .output()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn test_migration_v1_test_module_had_line_anchored_tests(content: &str) -> bool {
    content.lines().any(|line| line.trim() == "#[test]")
}

fn test_migration_delete_guard_deleted_v1_test_paths(
    base: &str,
    head: &str,
) -> Result<Vec<String>, String> {
    let out = if test_migration_delete_guard_merge_base_mode() {
        let range = format!("{base}...{head}");
        test_migration_delete_guard_run_git(&["diff", "--name-only", "--diff-filter=D", &range])?
    } else {
        test_migration_delete_guard_run_git(&[
            "diff",
            "--name-only",
            "--diff-filter=D",
            base,
            head,
        ])?
    };
    Ok(out
        .lines()
        .map(normalize_repo_path)
        .filter(|p| {
            p.starts_with("src/v1/tests/src/") && p.ends_with(".rs") && !p.ends_with("/lib.rs")
        })
        .collect())
}

fn test_migration_delete_guard_resolve_rev(r#ref: &str) -> Result<String, String> {
    match test_migration_delete_guard_run_git(&["rev-parse", r#ref]) {
        Ok(v) => Ok(v),
        Err(e) => {
            if r#ref == "origin/main" {
                test_migration_delete_guard_run_git(&["rev-parse", "main"]).or(Err(e))
            } else {
                Err(e)
            }
        }
    }
}

fn test_migration_delete_guard_uncovered_deletes_inner() -> Result<Vec<String>, String> {
    let (base, head) = test_migration_delete_guard_diff_endpoints();
    let ci_diff_configured = std::env::var("GUNBC_CI_DIFF_BASE").is_ok();
    let base_rev = match test_migration_delete_guard_resolve_rev(&base) {
        Ok(v) => v,
        Err(_) if !ci_diff_configured => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let head_rev = match test_migration_delete_guard_resolve_rev(&head) {
        Ok(v) => v,
        Err(_) if !ci_diff_configured => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    if base_rev == head_rev {
        return Ok(Vec::new());
    }
    let floor_stems = test_migration_debt_floor_stems();
    let deleted = test_migration_delete_guard_deleted_v1_test_paths(&base, &head)?;
    let mut violations = Vec::new();
    for path in deleted {
        let content = test_migration_delete_guard_run_git(&["show", &format!("{base}:{path}")])?;
        if !test_migration_v1_test_module_had_line_anchored_tests(&content) {
            continue;
        }
        let file_name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let stem = test_migration_debt_stem(file_name);
        if !test_migration_debt_stem_covered(&stem, &floor_stems) {
            violations.push(path);
        }
    }
    violations.sort();
    violations.dedup();
    Ok(violations)
}

pub fn test_migration_delete_guard_uncovered_deletes() -> Vec<String> {
    test_migration_delete_guard_uncovered_deletes_inner().unwrap_or_default()
}

pub fn test_migration_delete_guard_holds() -> bool {
    match test_migration_delete_guard_uncovered_deletes_inner() {
        Ok(violations) => violations.is_empty(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod test_migration_debt_tests {
    use super::*;

    #[test]
    fn stem_strips_rs_and_dag_suffixes_before_test_suffix() {
        assert_eq!(
            test_migration_debt_stem("witness_option_bridge_test.rs"),
            "witness_option_bridge"
        );
        assert_eq!(
            test_migration_debt_stem("witness_option_bridge_test.dag"),
            "witness_option_bridge"
        );
        assert_ne!(
            test_migration_debt_stem("typescript_import_pipeline_test.dag"),
            "pipeline"
        );
    }

    #[test]
    fn known_covered_module_is_not_debt() {
        assert!(test_migration_debt_known_covered_module_is_not_debt());
    }

    #[test]
    fn delete_guard_holds_with_no_v1_test_deletions_in_diff() {
        assert!(test_migration_delete_guard_holds());
    }

    #[test]
    fn delete_guard_rejects_uncovered_v1_test_delete() {
        let floor_stems = test_migration_debt_floor_stems();
        let stem = test_migration_debt_stem("cron_tag_test.rs");
        assert!(!test_migration_debt_stem_covered(&stem, &floor_stems));
    }
}

// Host-fed fact extraction for `v2.lens.host_language_transport_script` — the lens `.dag` table
// owns verdict logic; this bridge only projects `shell.Exec.Run` script-arg shapes from parsed
// modules. DISSOLUTION: node-tree reader at gunbc#5364; until then one shared host seam (Chunk D).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportScriptArgShape {
    ComputedApplication = 0,
    BareStringLiteral = 1,
    LetBoundStringLiteral = 2,
    StringInterpLiteralsOnly = 3,
}

impl TransportScriptArgShape {
    fn as_symbol(self) -> &'static str {
        match self {
            Self::ComputedApplication => "ComputedApplication",
            Self::BareStringLiteral => "BareStringLiteral",
            Self::LetBoundStringLiteral => "LetBoundStringLiteral",
            Self::StringInterpLiteralsOnly => "StringInterpLiteralsOnly",
        }
    }
}

pub struct TransportScriptPositionFactRaw {
    pub path: String,
    pub function: String,
    pub shape: &'static str,
}

fn resolve_dag_path_for_transport_script(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_file() {
        return candidate.to_path_buf();
    }
    let rooted = workspace_root().join(path);
    if rooted.is_file() {
        return rooted;
    }
    panic!("transport_script_position_facts: file not found: {path}");
}

fn parse_module_items_for_transport_script(
    path: &str,
) -> (Rc<Vec<Rc<Node>>>, Rc<HashMap<String, Rc<NewlineIndex>>>) {
    let resolved = resolve_dag_path_for_transport_script(path);
    let path_str = resolved.to_string_lossy();
    let content = std::fs::read_to_string(&resolved).unwrap_or_else(|e| {
        panic!("transport_script_position_facts: failed to read {path_str}: {e}")
    });
    let filename = resolved
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let tokens = v1_compiler_tokenize::tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.to_string(), source_index);
    let source_indices = Rc::new(source_indices);
    let result = v1_compiler_parse::parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "transport_script_position_facts: parse error in {path}: {}",
            diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .expect("transport_script_position_facts: missing module");
    (module.children.clone(), source_indices)
}

fn literal_string_value_transport_script(node: &Rc<Node>) -> bool {
    matches!(
        node.expr_data.as_ref(),
        ExprData::ExprLiteral {
            value: lit,
            ..
        } if matches!(lit.as_ref(), LiteralValue::LitStr { .. })
    )
}

fn classify_transport_script_arg(
    node: &Rc<Node>,
    let_literal_bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> TransportScriptArgShape {
    if literal_string_value_transport_script(node) {
        return TransportScriptArgShape::BareStringLiteral;
    }
    match node.expr_data.as_ref() {
        ExprData::ExprStringInterp => {
            for child in node.children.iter() {
                match child.expr_data.as_ref() {
                    ExprData::ExprLiteral { value, .. } => {
                        if !matches!(value.as_ref(), LiteralValue::LitStr { .. }) {
                            return TransportScriptArgShape::ComputedApplication;
                        }
                    }
                    ExprData::ExprVar { .. } => {
                        let name = expr_var_name_at(child.clone(), source_indices.clone());
                        if !let_literal_bindings.get(&name).copied().unwrap_or(false) {
                            return TransportScriptArgShape::ComputedApplication;
                        }
                    }
                    _ => return TransportScriptArgShape::ComputedApplication,
                }
            }
            TransportScriptArgShape::StringInterpLiteralsOnly
        }
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), source_indices.clone());
            if let_literal_bindings.get(&name).copied().unwrap_or(false) {
                TransportScriptArgShape::LetBoundStringLiteral
            } else {
                TransportScriptArgShape::ComputedApplication
            }
        }
        _ => TransportScriptArgShape::ComputedApplication,
    }
}

fn is_shell_exec_run_transport_script(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    match node.expr_data.as_ref() {
        ExprData::ExprMethodCall { .. } => {
            if expr_method_name_at(node.clone(), source_indices.clone()) != "Run" {
                return false;
            }
            let recv = method_receiver(node.clone());
            match recv.expr_data.as_ref() {
                ExprData::ExprFieldAccess { .. } => {
                    if field_access_field_at(recv.clone(), source_indices.clone()) != "Exec" {
                        return false;
                    }
                    let base = field_access_base(recv.clone());
                    match base.expr_data.as_ref() {
                        ExprData::ExprVar { .. } => {
                            expr_var_name_at(base.clone(), source_indices.clone()) == "shell"
                        }
                        _ => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn transport_script_arg_node(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    for arg in method_arg_nodes(node.clone()).iter() {
        if arg_name_at(arg.clone(), source_indices.clone()).as_deref() == Some("script") {
            return Some(arg_value(arg.clone()));
        }
    }
    None
}

fn binding_is_literal_shaped_transport_script(
    node: &Rc<Node>,
    bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    matches!(
        classify_transport_script_arg(node, bindings, source_indices),
        TransportScriptArgShape::BareStringLiteral
            | TransportScriptArgShape::LetBoundStringLiteral
            | TransportScriptArgShape::StringInterpLiteralsOnly
    )
}

fn collect_let_bindings_in_block_transport_script(
    block: &Rc<Node>,
    bindings: &mut HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) {
    for stmt in block_stmts(block.clone()).iter() {
        match stmt.expr_data.as_ref() {
            ExprData::ExprLet { .. } => {
                let name = let_binding_name_at(stmt.clone(), source_indices.clone());
                let val = let_value(stmt.clone());
                let literal_shaped =
                    binding_is_literal_shaped_transport_script(&val, bindings, source_indices);
                bindings.insert(name, literal_shaped);
            }
            _ => walk_transport_script_expr(stmt, bindings, source_indices, &mut |_| {}),
        }
    }
}

fn walk_transport_script_expr(
    node: &Rc<Node>,
    let_bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    on_run: &mut dyn FnMut(TransportScriptArgShape),
) {
    if is_shell_exec_run_transport_script(node, source_indices) {
        if let Some(script_node) = transport_script_arg_node(node, source_indices) {
            on_run(classify_transport_script_arg(
                &script_node,
                let_bindings,
                source_indices,
            ));
        }
    }
    for child in node.children.iter() {
        walk_transport_script_expr(child, let_bindings, source_indices, on_run);
    }
}

fn transport_script_facts_for_function_body(
    rel_path: &str,
    function: &str,
    body: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<TransportScriptPositionFactRaw> {
    let mut bindings = HashMap::new();
    if let ExprData::ExprBlock { .. } = body.expr_data.as_ref() {
        collect_let_bindings_in_block_transport_script(body, &mut bindings, source_indices);
    }
    let mut facts = Vec::new();
    walk_transport_script_expr(body, &bindings, source_indices, &mut |shape| {
        facts.push(TransportScriptPositionFactRaw {
            path: rel_path.to_string(),
            function: function.to_string(),
            shape: shape.as_symbol(),
        });
    });
    facts
}

pub fn transport_script_position_facts_for_path(
    path: String,
) -> Vec<TransportScriptPositionFactRaw> {
    let (items, source_indices) = parse_module_items_for_transport_script(&path);
    let mut facts = Vec::new();
    for item in items.iter() {
        let kind = item_kind(item.clone());
        if !matches!(kind, ItemKind::FuncItem | ItemKind::FnItem) {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            continue;
        };
        facts.extend(transport_script_facts_for_function_body(
            &path,
            &item.name,
            body,
            &source_indices,
        ));
    }
    facts
}

#[cfg(test)]
mod module_path_index_tests {
    use super::*;

    #[test]
    fn cargo_build_resolves_by_module_path_not_directory_nickname() {
        let path = source_path_for_module_path("extdeps.cargo_build".to_string());
        assert_eq!(path, "dag/extdeps/rust/cargo_build.dag");
    }

    #[test]
    fn git_module_resolves() {
        let path = source_path_for_module_path("extdeps.git".to_string());
        assert_eq!(path, "dag/extdeps/git/git.dag");
    }

    #[test]
    fn extdeps_shell_resolves_to_the_dag_authority() {
        let path = source_path_for_module_path("extdeps.shell".to_string());
        assert_eq!(path, "dag/extdeps/shell/shell.dag");
    }

    #[test]
    fn duplicate_module_path_across_roots_refuses_loudly() {
        let dir = std::env::temp_dir().join(format!(
            "gunbc-module-collision-wall-{}",
            std::process::id()
        ));
        let root_a = dir.join("root_a");
        let root_b = dir.join("root_b");
        std::fs::create_dir_all(&root_a).unwrap();
        std::fs::create_dir_all(&root_b).unwrap();
        std::fs::write(root_a.join("m.dag"), "module collision.example\n").unwrap();
        std::fs::write(root_b.join("m.dag"), "module collision.example\n").unwrap();
        let roots = vec![
            root_a.to_string_lossy().into_owned(),
            root_b.to_string_lossy().into_owned(),
        ];
        // RED control: same module declared in two files refuses loudly.
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }))
        .is_err();
        // GREEN control: distinct modules build fine.
        std::fs::write(root_b.join("m.dag"), "module collision.other\n").unwrap();
        let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }))
        .is_ok();
        std::fs::remove_dir_all(&dir).ok();
        assert!(refused, "collision must refuse loudly, not shadow silently");
        assert!(built, "distinct modules must still index");
    }

    #[test]
    fn cargo_target_dir_output_never_enters_the_module_index() {
        let dir =
            std::env::temp_dir().join(format!("gunbc-target-dir-exclusion-{}", std::process::id()));
        let root = dir.join("root");
        let baseline = root.join("target").join("baseline_corpus");
        std::fs::create_dir_all(&baseline).unwrap();
        std::fs::write(root.join("m.dag"), "module corpus.example\n").unwrap();
        std::fs::write(baseline.join("m.dag"), "module corpus.example\n").unwrap();
        let roots = vec![root.to_string_lossy().into_owned()];
        // With a Cargo.toml beside it, target/ is build output: the corpus
        // copy is skipped and the source file indexes alone (the CI regression:
        // target/func_env_semantic_baseline_corpus tripped the collision wall).
        std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
        let indexed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }));
        // RED control: without Cargo.toml the same layout is two source files
        // declaring one module — the wall must still refuse.
        std::fs::remove_file(root.join("Cargo.toml")).unwrap();
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }))
        .is_err();
        std::fs::remove_dir_all(&dir).ok();
        let index = indexed.expect("cargo target output must be excluded, not collide");
        assert!(
            index.contains_key("corpus.example"),
            "the source-tree declaration must still index"
        );
        assert!(
            refused,
            "a plain (non-cargo) target dir is source like any other — collision must refuse"
        );
    }

    #[test]
    fn reader_follows_synthetic_authority_with_nondefault_roots() {
        let synthetic = "module gunbc.ci_layer_roots\n\n\
             data witness_layer_roots: List<String> = [\"r_one\", \"r_two\", \"r_three\"]\n";
        assert_eq!(
            witness_layer_roots_from_source(synthetic),
            vec![
                "r_one".to_string(),
                "r_two".to_string(),
                "r_three".to_string()
            ],
            "the layer-roots reader must FOLLOW the authority, not a hardcoded copy"
        );
    }

    #[test]
    fn reader_projects_live_authority_value() {
        assert_eq!(
            witness_layer_roots(),
            vec!["dag".to_string(), "src/v2".to_string()],
            "live authority value drifted from the expected [dag, src/v2]"
        );
        assert!(
            census_corpus_roots_follow_layer_authority(),
            "census corpus roots must derive from the layer-roots authority"
        );
    }

    #[test]
    fn default_source_roots_derive_from_authority() {
        let ws = workspace_root();
        assert_eq!(
            default_source_roots(),
            vec![
                ws.join("dag").to_string_lossy().into_owned(),
                ws.join("src/v2").to_string_lossy().into_owned(),
            ]
        );
    }

    #[test]
    fn reader_follows_synthetic_authority_scan_dirs() {
        let synthetic = "module gunbc.ci_layer_roots\n\n\
             data witness_discovery_scan_dirs: List<String> = [\"scan/a\", \"scan/b\"]\n";
        assert_eq!(
            witness_discovery_scan_dirs_from_source(synthetic),
            vec!["scan/a".to_string(), "scan/b".to_string()],
            "the scan-dir reader must FOLLOW the authority, not a hardcoded copy"
        );
    }

    #[test]
    fn witness_discovery_scan_dirs_projects_live_authority_value() {
        assert_eq!(
            witness_discovery_scan_dirs(),
            vec![
                "dag/test/claim".to_string(),
                "src/v2/test/claim/manual".to_string(),
            ],
            "live authority scan-dir value drifted"
        );
    }

    #[test]
    fn lens_table_reader_projects_live_medium_structure_exception_roster() {
        const LENS: &str = "src/v2/lens/medium_structure_containment.dag";
        let roster = lens_string_list_data(LENS, "medium_structure_exception_roster", false);
        assert!(
            roster.iter().any(|p| p == "dag/gunbc/ci_workflow.dag"),
            "live lens authority roster must include a known exception path; got {roster:?}"
        );
    }

    #[test]
    fn lens_table_reader_allows_empty_when_explicit() {
        const LENS: &str = "src/v2/lens/medium_structure_containment.dag";
        assert!(
            lens_string_list_data(LENS, "empty_medium_marker_list", true).is_empty(),
            "allow_empty=true must permit intentionally empty lens tables"
        );
    }

    #[test]
    fn strip_blanks_string_interior_and_drops_comment() {
        let got = strip_line_comment("data u = \"https://x // y\" // real comment");
        assert!(got.starts_with("data u = \""));
        assert!(
            !got.contains("real comment"),
            "trailing // comment dropped: {got:?}"
        );
        assert!(!got.contains("https"), "string interior blanked: {got:?}");
        assert!(got.len() <= "data u = \"https://x // y\" // real comment".len());
    }

    #[test]
    fn brace_delta_ignores_braces_in_strings() {
        assert_eq!(brace_delta("fn f() {"), 1);
        assert_eq!(brace_delta("let s = \"{ { {\""), 0);
        assert_eq!(brace_delta("} // }"), -1);
    }

    #[test]
    fn is_test_dag_matches_suffix() {
        assert!(is_test_dag("src/v2/lens/x_test.dag"));
        assert!(!is_test_dag("src/v2/lens/x.dag"));
    }

    #[test]
    fn extract_top_level_decls_captures_split_brace_body() {
        let source = include_str!("../tests/fixtures/fact_cardinality_split_brace.dag");
        let decls = extract_top_level_decls(source);
        let sample = decls
            .iter()
            .find(|(name, _)| name == "split_brace_sample")
            .expect("split-brace decl must be captured");
        let expected = decl_body_hash(
            "data split_brace_sample: SplitBraceSample =\nSplitBraceSample {\n  field: \"x\"\n}\n",
        );
        assert_eq!(
            sample.1, expected,
            "split-brace body hash must include lines after the opener"
        );
    }
}

// SCAFFOLD — host-fed fact extraction for v2.lens.extdeps_shape_transport_policy (Concern A).
// Dissolution: when the Node-tree argv projection supersedes text scan (dissolve-on marker in
// extdeps_shape_transport_policy.dag construction_justification), replace this block with a
// Node-tree builtin and delete these structs. gunbc#5364 successor, Concern A lane.

pub struct ExtdepsArgvFactRaw {
    pub module_path: String,
    pub service: String,
    pub operation: String,
    pub transport_kind: &'static str,
    pub argv_index: i64,
    pub argv_token: String,
}

pub struct ExtdepsFusionFactRaw {
    pub module_path: String,
    pub endpoint_key: String,
    pub service_a: String,
    pub service_b: String,
}

pub struct ExtdepsInputFactRaw {
    pub module_path: String,
    pub service: String,
    pub operation: String,
    pub param_name: String,
}

pub struct ExtdepsEmbeddedFactRaw {
    pub module_path: String,
    pub data_name: String,
    pub field_name: String,
    pub literal_value: String,
}

pub struct ExtdepsShapeTransportPolicyModuleFacts {
    pub argv_facts: Vec<ExtdepsArgvFactRaw>,
    pub fusion_facts: Vec<ExtdepsFusionFactRaw>,
    pub input_facts: Vec<ExtdepsInputFactRaw>,
    pub embedded_facts: Vec<ExtdepsEmbeddedFactRaw>,
    pub source_nickname_literal_count: i64,
    pub gist_create_declares_filename_input: bool,
    pub gist_create_files_keyed_by_filename: bool,
}

pub fn parse_extdeps_module_items(
    path: &str,
) -> (
    Rc<Vec<Rc<crate::v1_std_core::Node>>>,
    Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    use crate::v1_compiler_parse::parse;
    use crate::v1_compiler_tokenize::tokenize;
    use crate::v1_std_core::build_newline_index;
    let candidate = std::path::Path::new(path);
    let resolved = if candidate.is_file() {
        candidate.to_path_buf()
    } else {
        let rooted = workspace_root().join(path);
        if rooted.is_file() {
            rooted
        } else {
            panic!("parse_extdeps_module_items: file not found: {path}");
        }
    };
    let path_str = resolved.to_string_lossy();
    let content = std::fs::read_to_string(&resolved)
        .unwrap_or_else(|e| panic!("parse_extdeps_module_items: failed to read {path_str}: {e}"));
    let filename = resolved
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let tokens = tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut source_indices_map = HashMap::new();
    source_indices_map.insert(filename.to_string(), source_index);
    let source_indices = Rc::new(source_indices_map);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "parse_extdeps_module_items: parse error in {path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .expect("parse_extdeps_module_items: missing module");
    (module.children.clone(), source_indices)
}

pub fn shell_argv_nodes_for_operation(
    path: String,
    service: String,
    operation: String,
) -> (
    Rc<Vec<Rc<crate::v1_std_core::Node>>>,
    Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    let (items, source_indices) = parse_extdeps_module_items(&path);
    for item in items.iter() {
        if item.name != service {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name != operation {
                continue;
            }
            let eff = crate::v1_compiler_emit::effective_operation_transport(
                op.clone(),
                fallback_transport.clone(),
            );
            return (eff.children.clone(), source_indices);
        }
    }
    panic!("shell_argv_nodes_for_operation: operation {service}.{operation} not found in {path}");
}

pub fn qualified_name_resolves_in_derived_module_set(qn: &crate::v1_interpreter::Value) -> bool {
    let module_path = free_monoid_symbol_value_to_dotted_string(qn);
    !module_path.is_empty()
        && build_module_path_index_from_witness_roots().contains_key(&module_path)
}

fn extdeps_argv_expr_token(
    node: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    use crate::v1_std_core::{expr_var_name_at, ExprData, LiteralValue};
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => value.clone(),
            other => format!("{other:?}"),
        },
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), source_indices.clone());
            if name.is_empty() {
                node.name.clone()
            } else {
                format!("{{{name}}}")
            }
        }
        ExprData::ExprStringInterp => node
            .children
            .iter()
            .map(|child| match child.expr_data.as_ref() {
                ExprData::ExprLiteral { value } => match value.as_ref() {
                    LiteralValue::LitStr { value } => value.clone(),
                    _ => String::new(),
                },
                ExprData::ExprVar { .. } => {
                    let name = expr_var_name_at(child.clone(), source_indices.clone());
                    if name.is_empty() {
                        child.name.clone()
                    } else {
                        format!("{{{name}}}")
                    }
                }
                _ => String::new(),
            })
            .collect(),
        _ => String::new(),
    }
}

fn extdeps_literal_string_value(node: &Rc<crate::v1_std_core::Node>) -> Option<String> {
    use crate::v1_std_core::{ExprData, LiteralValue};
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => Some(value.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn extdeps_record_field_value(
    record: &Rc<crate::v1_std_core::Node>,
    field_name: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<crate::v1_std_core::Node>> {
    use crate::v1_std_core::{field_init_node_name_at, field_init_node_value, ExprData};
    if !matches!(record.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        return None;
    }
    for field_init in record.children.iter() {
        let name = field_init_node_name_at(field_init.clone(), source_indices.clone());
        if name == field_name {
            return Some(field_init_node_value(field_init.clone()));
        }
    }
    None
}

fn extdeps_module_source_nickname_count_in_node(
    node: &Rc<crate::v1_std_core::Node>,
    real_paths: &std::collections::HashSet<String>,
) -> i64 {
    let mut count = 0i64;
    if let Some(lit) = extdeps_literal_string_value(node) {
        if real_paths.contains(&lit) {
            count += 1;
        }
    }
    if let Some(body) = node.body.as_ref() {
        count += extdeps_module_source_nickname_count_in_node(body, real_paths);
    }
    for child in node.children.iter() {
        count += extdeps_module_source_nickname_count_in_node(child, real_paths);
    }
    for param in node.params.iter() {
        count += extdeps_module_source_nickname_count_in_node(param, real_paths);
    }
    if let Some(type_annotation) = node.type_annotation.as_ref() {
        count += extdeps_module_source_nickname_count_in_node(type_annotation, real_paths);
    }
    count
}

fn extdeps_gist_create_declares_filename_for_items(
    items: &Rc<Vec<Rc<crate::v1_std_core::Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    use crate::v1_std_core::param_node_name_at;
    for item in items.iter() {
        if item.name != "github.Gist" {
            continue;
        }
        for op in item.children.iter() {
            if op.name != "Create" {
                continue;
            }
            for param in op.params.iter() {
                let name = param_node_name_at(param.clone(), source_indices.clone());
                if name == "filename" {
                    return true;
                }
            }
        }
    }
    false
}

fn extdeps_gist_map_keys_use_filename(
    map_node: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    use crate::v1_std_core::{field_init_node_name_at, ExprData};
    if !matches!(map_node.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        return false;
    }
    if map_node.children.is_empty() {
        return false;
    }
    for entry in map_node.children.iter() {
        let key = field_init_node_name_at(entry.clone(), source_indices.clone());
        if !(key == "filename" || key.contains("{filename}")) {
            return false;
        }
    }
    true
}

fn extdeps_gist_create_files_keyed_by_filename_for_items(
    items: &Rc<Vec<Rc<crate::v1_std_core::Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    use crate::v1_std_core::{is_rest_transport, transport_request_body};
    for item in items.iter() {
        if item.name != "github.Gist" {
            continue;
        }
        for op in item.children.iter() {
            if op.name != "Create" {
                continue;
            }
            let Some(transport) = op.transport.as_ref() else {
                return false;
            };
            if !is_rest_transport(transport.clone(), source_indices.clone()) {
                return false;
            }
            let Some(body) = transport_request_body(transport.clone(), source_indices.clone())
            else {
                return false;
            };
            let Some(files) = extdeps_record_field_value(&body, "files", source_indices) else {
                return false;
            };
            return extdeps_gist_map_keys_use_filename(&files, source_indices);
        }
    }
    false
}

pub fn extdeps_shape_transport_policy_module_facts(
    module_path: &str,
) -> ExtdepsShapeTransportPolicyModuleFacts {
    use crate::v1_compiler_emit::effective_operation_transport;
    use crate::v1_compiler_emit_core_support::is_data_def_item;
    use crate::v1_std_core::{
        field_init_node_name_at, field_init_node_value, param_node_name_at, ExprData,
    };

    let path = source_path_for_module_path(module_path.to_string());
    let (items, source_indices) = parse_extdeps_module_items(&path);

    let mut argv_facts: Vec<ExtdepsArgvFactRaw> = Vec::new();
    let mut input_facts: Vec<ExtdepsInputFactRaw> = Vec::new();

    for item in items.iter() {
        if item.name.is_empty() || item.children.is_empty() {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name.is_empty() {
                continue;
            }
            let eff = effective_operation_transport(op.clone(), fallback_transport.clone());
            let transport_kind =
                if crate::v1_std_core::is_rest_transport(eff.clone(), source_indices.clone()) {
                    "Rest"
                } else {
                    "Shell"
                };
            for (idx, arg) in eff.children.iter().enumerate() {
                let token = extdeps_argv_expr_token(arg, &source_indices);
                argv_facts.push(ExtdepsArgvFactRaw {
                    module_path: module_path.to_string(),
                    service: item.name.clone(),
                    operation: op.name.clone(),
                    transport_kind,
                    argv_index: idx as i64,
                    argv_token: token,
                });
            }
            for param in op.params.iter() {
                let name = param_node_name_at(param.clone(), source_indices.clone());
                if !name.is_empty() {
                    input_facts.push(ExtdepsInputFactRaw {
                        module_path: module_path.to_string(),
                        service: item.name.clone(),
                        operation: op.name.clone(),
                        param_name: name,
                    });
                }
            }
        }
    }

    let service_names: Vec<String> = items
        .iter()
        .filter(|item| !item.name.is_empty() && !item.children.is_empty())
        .map(|item| item.name.clone())
        .collect();
    let has_oauth_google = service_names.iter().any(|s| s == "oauth2.Google");
    let has_shell_oauth = service_names.iter().any(|s| s == "shell.OAuth2");
    let mut fusion_facts: Vec<ExtdepsFusionFactRaw> = Vec::new();
    if has_oauth_google && has_shell_oauth {
        fusion_facts.push(ExtdepsFusionFactRaw {
            module_path: module_path.to_string(),
            endpoint_key: "OAuth2.refresh".to_string(),
            service_a: "oauth2.Google".to_string(),
            service_b: "shell.OAuth2".to_string(),
        });
    }

    let mut embedded_facts: Vec<ExtdepsEmbeddedFactRaw> = Vec::new();
    for item in items.iter() {
        if !is_data_def_item(item.clone()) || item.name.is_empty() {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            continue;
        };
        if !matches!(body.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
            continue;
        }
        for field_init in body.children.iter() {
            let field_name = field_init_node_name_at(field_init.clone(), source_indices.clone());
            let value_node = field_init_node_value(field_init.clone());
            if let Some(literal) = extdeps_literal_string_value(&value_node) {
                embedded_facts.push(ExtdepsEmbeddedFactRaw {
                    module_path: module_path.to_string(),
                    data_name: item.name.clone(),
                    field_name,
                    literal_value: literal,
                });
            }
        }
    }

    let index = build_module_path_index_from_witness_roots();
    let real_paths: std::collections::HashSet<String> = index.into_values().collect();
    let mut source_nickname_literal_count = 0i64;
    for item in items.iter() {
        source_nickname_literal_count +=
            extdeps_module_source_nickname_count_in_node(item, &real_paths);
    }

    let gist_create_declares_filename_input =
        extdeps_gist_create_declares_filename_for_items(&items, &source_indices);
    let gist_create_files_keyed_by_filename =
        extdeps_gist_create_files_keyed_by_filename_for_items(&items, &source_indices);

    ExtdepsShapeTransportPolicyModuleFacts {
        argv_facts,
        fusion_facts,
        input_facts,
        embedded_facts,
        source_nickname_literal_count,
        gist_create_declares_filename_input,
        gist_create_files_keyed_by_filename,
    }
}

// SCAFFOLD — host-fed fact extraction for v2.lens.extdeps_external_authority (Concern B).
// Dissolution: when Node-tree anchor projection supersedes module parse (dissolve-on marker in
// extdeps_external_authority.dag construction_justification), replace this block with a
// Node-tree builtin and delete these structs. gunbc#5364 successor, Concern B lane.

pub struct ExtdepsExternalAuthorityModuleFacts {
    pub anchor_kind: String,
    pub scheme_identity: String,
    pub locator: String,
    pub is_backfill_pending: bool,
    pub is_machinery_exempt: bool,
    pub is_clean_tree_roster_excluded: bool,
    pub anchor_shadow_masked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExternalAuthorityAnchorProjection {
    Absent,
    Present {
        scheme_identity: String,
        locator: String,
    },
}

fn external_authority_uri_record_from_anchor_body(
    body: &Rc<crate::v1_std_core::Node>,
    variant: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<crate::v1_std_core::Node>> {
    match variant {
        "ExternalAuthority" | "StableAuthority" | "ExternalUri" => {
            extdeps_record_field_value(body, "uri", source_indices)
        }
        _ => None,
    }
}

fn external_authority_scheme_identity_from_value_node(
    node: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    use crate::v1_std_core::authored_name_at;
    authored_name_at(source_indices.clone(), node.clone())
}

fn read_external_authority_anchor_from_items(
    items: &Rc<Vec<Rc<crate::v1_std_core::Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> ExternalAuthorityAnchorProjection {
    use crate::v1_compiler_emit_core_support::is_data_def_item;
    use crate::v1_std_core::authored_name_at;
    for item in items.iter() {
        if !is_data_def_item(item.clone()) || item.name != "extdeps_external_authority_anchor" {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            return ExternalAuthorityAnchorProjection::Absent;
        };
        let variant = authored_name_at(source_indices.clone(), body.clone());
        let Some(uri_node) =
            external_authority_uri_record_from_anchor_body(body, variant.as_str(), source_indices)
        else {
            return ExternalAuthorityAnchorProjection::Absent;
        };
        let scheme = extdeps_record_field_value(&uri_node, "scheme", source_indices)
            .map(|n| external_authority_scheme_identity_from_value_node(&n, source_indices))
            .unwrap_or_default();
        let locator = extdeps_record_field_value(&uri_node, "locator", source_indices)
            .and_then(|n| extdeps_literal_string_value(&n))
            .unwrap_or_default();
        if scheme.is_empty() {
            return ExternalAuthorityAnchorProjection::Absent;
        }
        return ExternalAuthorityAnchorProjection::Present {
            scheme_identity: scheme,
            locator,
        };
    }
    ExternalAuthorityAnchorProjection::Absent
}

fn project_external_authority_anchor(module_path: &str) -> ExternalAuthorityAnchorProjection {
    let path = source_path_for_module_path(module_path.to_string());
    let (items, source_indices) = parse_extdeps_module_items(&path);
    read_external_authority_anchor_from_items(&items, &source_indices)
}

fn external_authority_backfill_pending_module_paths() -> &'static std::collections::HashSet<String>
{
    use std::collections::HashSet;
    use std::sync::OnceLock;
    static PATHS: OnceLock<HashSet<String>> = OnceLock::new();
    PATHS.get_or_init(|| {
        let path = workspace_root().join("dag/extdeps/external_authority_backfill_pending.txt");
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read backfill_pending snapshot {:?}: {e}", path));
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect()
    })
}

fn external_authority_machinery_exempt_module_paths() -> &'static [&'static str] {
    &["extdeps.uri", "extdeps.external_authority"]
}

fn external_authority_clean_tree_roster_exclusion_paths() -> &'static [&'static str] {
    &[
        "extdeps.fixture.external_authority_bogus_scheme",
        "extdeps.fixture.external_authority_missing",
        "extdeps.fixture.external_authority_clean_https_no_anchor",
        "extdeps.fixture.external_authority_file_anchor",
    ]
}

pub fn extdeps_derived_extdeps_module_paths() -> Vec<String> {
    let index = build_module_path_index_from_witness_roots();
    let mut paths: Vec<String> = index
        .keys()
        .filter(|k| k.starts_with("extdeps."))
        .cloned()
        .collect();
    paths.sort();
    paths
}

pub fn extdeps_derived_extdeps_modules_value(
    ctx: &crate::v1_interpreter::InterpContext,
) -> crate::v1_interpreter::Value {
    use crate::v1_interpreter::list_value;
    let items: Vec<_> = extdeps_derived_extdeps_module_paths()
        .iter()
        .map(|p| free_monoid_symbol_value_from_dotted_string(ctx, p))
        .collect();
    list_value(items)
}

pub fn extdeps_external_authority_backfill_pending_entries_value(
    ctx: &crate::v1_interpreter::InterpContext,
) -> crate::v1_interpreter::Value {
    use crate::v1_interpreter::list_value;
    let mut paths: Vec<String> = external_authority_backfill_pending_module_paths()
        .iter()
        .cloned()
        .collect();
    paths.sort();
    let items: Vec<_> = paths
        .iter()
        .map(|p| free_monoid_symbol_value_from_dotted_string(ctx, p))
        .collect();
    list_value(items)
}

fn external_authority_is_backfill_pending_for_module_path(module_path: &str) -> bool {
    external_authority_backfill_pending_module_paths().contains(module_path)
}

fn external_authority_is_machinery_exempt_for_module_path(module_path: &str) -> bool {
    external_authority_machinery_exempt_module_paths().contains(&module_path)
}

fn external_authority_is_clean_tree_roster_excluded_for_module_path(module_path: &str) -> bool {
    if module_path.starts_with("extdeps.fixture.") {
        return true;
    }
    if module_path.ends_with(".mock_corpus") {
        return true;
    }
    external_authority_clean_tree_roster_exclusion_paths().contains(&module_path)
}

fn external_authority_anchor_present_in_any_source_root(module_path: &str) -> bool {
    let ws = workspace_root();
    for root in default_source_roots() {
        let root_path = std::path::PathBuf::from(&root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        collect_dag_files_tolerant(&root_path, &mut files);
        for file in files {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            let declares = content.lines().find_map(|l| {
                l.trim()
                    .strip_prefix("module ")
                    .map(|m| m.trim().to_string())
            });
            if declares.as_deref() != Some(module_path) {
                continue;
            }
            let rel = file
                .strip_prefix(&ws)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| file.to_string_lossy().into_owned());
            let (items, source_indices) = parse_extdeps_module_items(&rel);
            if matches!(
                read_external_authority_anchor_from_items(&items, &source_indices),
                ExternalAuthorityAnchorProjection::Present { .. }
            ) {
                return true;
            }
        }
    }
    false
}

fn external_authority_shadow_plant_paired_extdeps_module_path(module_path: &str) -> Option<String> {
    module_path
        .strip_prefix("test.fixture.")
        .map(|leaf| format!("extdeps.fixture.{leaf}"))
}

fn external_authority_anchor_shadow_masked_for_module_path(module_path: &str) -> bool {
    match project_external_authority_anchor(module_path) {
        ExternalAuthorityAnchorProjection::Present { .. } => false,
        ExternalAuthorityAnchorProjection::Absent => {
            if external_authority_anchor_present_in_any_source_root(module_path) {
                return true;
            }
            if let Some(extdeps_path) =
                external_authority_shadow_plant_paired_extdeps_module_path(module_path)
            {
                return external_authority_anchor_present_in_any_source_root(&extdeps_path);
            }
            false
        }
    }
}

pub fn extdeps_external_authority_module_facts(
    module_path: &str,
) -> ExtdepsExternalAuthorityModuleFacts {
    let (anchor_kind, scheme_identity, locator) =
        match project_external_authority_anchor(module_path) {
            ExternalAuthorityAnchorProjection::Absent => {
                ("absent".to_string(), String::new(), String::new())
            }
            ExternalAuthorityAnchorProjection::Present {
                scheme_identity,
                locator,
            } => ("present".to_string(), scheme_identity, locator),
        };
    ExtdepsExternalAuthorityModuleFacts {
        anchor_kind,
        scheme_identity,
        locator,
        is_backfill_pending: external_authority_is_backfill_pending_for_module_path(module_path),
        is_machinery_exempt: external_authority_is_machinery_exempt_for_module_path(module_path),
        is_clean_tree_roster_excluded:
            external_authority_is_clean_tree_roster_excluded_for_module_path(module_path),
        anchor_shadow_masked: external_authority_anchor_shadow_masked_for_module_path(module_path),
    }
}

fn external_authority_live_violation_module_paths() -> Vec<String> {
    let backfill = external_authority_backfill_pending_module_paths();
    let mut violations = Vec::new();
    for path in extdeps_derived_extdeps_module_paths() {
        if external_authority_is_clean_tree_roster_excluded_for_module_path(&path) {
            continue;
        }
        if external_authority_is_machinery_exempt_for_module_path(&path) || backfill.contains(&path)
        {
            continue;
        }
        match project_external_authority_anchor(&path) {
            ExternalAuthorityAnchorProjection::Absent => violations.push(format!("missing:{path}")),
            ExternalAuthorityAnchorProjection::Present {
                scheme_identity, ..
            } if scheme_identity != "Http" && scheme_identity != "Https" => {
                violations.push(format!("non_external:{path}:{scheme_identity}"))
            }
            _ => {}
        }
    }
    violations
}

pub fn extdeps_external_authority_live_clean_tree_holds() -> bool {
    external_authority_live_violation_module_paths().is_empty()
}

pub fn extdeps_external_authority_live_roster_module_count() -> i64 {
    extdeps_derived_extdeps_module_paths()
        .into_iter()
        .filter(|path| !external_authority_is_clean_tree_roster_excluded_for_module_path(path))
        .count() as i64
}

pub fn extdeps_external_authority_live_shadow_mask_holds() -> bool {
    for path in extdeps_derived_extdeps_module_paths() {
        if external_authority_is_clean_tree_roster_excluded_for_module_path(&path)
            || external_authority_is_machinery_exempt_for_module_path(&path)
        {
            continue;
        }
        if external_authority_anchor_shadow_masked_for_module_path(&path) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod doc_reachability_tests {
    use super::*;
    use std::collections::HashMap;

    fn edges_of(pairs: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, vs)| {
                (
                    (*k).to_string(),
                    vs.iter().map(|s| (*s).to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn reachable_set_flags_orphan_node() {
        let roots: BTreeSet<String> = ["root.md".to_string()].into_iter().collect();
        let edges = edges_of(&[("root.md", &["linked.md"]), ("orphan.md", &[])]);
        let reached = doc_reachable_set(&roots, &edges);
        assert!(reached.contains("root.md"));
        assert!(reached.contains("linked.md"));
        assert!(
            !reached.contains("orphan.md"),
            "an unlinked node must be unreachable (the orphan witness)"
        );
    }

    #[test]
    fn reachable_set_inert_cluster_stays_unreached() {
        let roots: BTreeSet<String> = ["root.md".to_string()].into_iter().collect();
        let edges = edges_of(&[
            ("root.md", &["a.md"]),
            ("a.md", &[]),
            ("dead1.md", &["dead2.md"]),
            ("dead2.md", &["dead1.md"]),
        ]);
        let reached = doc_reachable_set(&roots, &edges);
        assert!(reached.contains("a.md"));
        assert!(!reached.contains("dead1.md") && !reached.contains("dead2.md"));
    }

    #[test]
    fn reachable_set_transitive_chain() {
        let roots: BTreeSet<String> = ["r.md".to_string()].into_iter().collect();
        let edges = edges_of(&[
            ("r.md", &["a.md"]),
            ("a.md", &["b.md"]),
            ("b.md", &["c.md"]),
        ]);
        let reached = doc_reachable_set(&roots, &edges);
        for n in ["r.md", "a.md", "b.md", "c.md"] {
            assert!(reached.contains(n), "{n} should be reached");
        }
    }

    #[test]
    fn markdown_link_targets_basic() {
        let c = "see [x](docs/plans/x.md) and [y](y.md#anchor) and [ext](https://e.com) and [z](./z.md)";
        let t = markdown_link_targets(c);
        assert_eq!(t, vec!["docs/plans/x.md", "y.md", "./z.md"]);
    }

    #[test]
    fn dangling_detection_flags_missing_md_only() {
        let doc = "[ok](https://x) [broken](docs/plans/does-not-exist-xyz.md) [code](src/lib.rs)";
        let targets = markdown_link_targets(doc);
        let dangling: Vec<&String> = targets
            .iter()
            .filter(|t| {
                t.ends_with(".md")
                    && !workspace_root()
                        .join(normalize_doc_path(Path::new(t)))
                        .is_file()
            })
            .collect();
        assert_eq!(
            dangling.len(),
            1,
            "exactly the missing .md link is dangling (not the http or the existing code link): {dangling:?}"
        );
    }

    #[test]
    fn bind_md_refs_basic() {
        let c = "// bind: docs/planning/foo.md (provenance)\n// no bind here\n// bind: bar.md";
        let t = bind_md_refs(c);
        assert_eq!(t, vec!["docs/planning/foo.md", "bar.md"]);
    }
}

// --- REST transport fact projection (folded from rest_transport_facts.rs) ---
// Pure Node-tree reader over transport annotations — zero host I/O.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredRestTransportOp {
    pub service: String,
    pub name: String,
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestTransportFactError {
    MissingServiceScope { operation: String },
    MissingMethodProperty { service: String, operation: String },
    MissingPathProperty { service: String, operation: String },
}

impl std::fmt::Display for RestTransportFactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestTransportFactError::MissingServiceScope { operation } => {
                write!(
                    f,
                    "REST transport without enclosing service scope (operation={operation})"
                )
            }
            RestTransportFactError::MissingMethodProperty { service, operation } => {
                write!(
                    f,
                    "missing method on rest transport for {service}::{operation}"
                )
            }
            RestTransportFactError::MissingPathProperty { service, operation } => {
                write!(
                    f,
                    "missing path on rest transport for {service}::{operation}"
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestTransportCollectResult {
    pub ops: Vec<DeclaredRestTransportOp>,
    pub errors: Vec<RestTransportFactError>,
}

fn rest_transport_field_string(
    props: Rc<Vec<Rc<Node>>>,
    prop_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    use crate::v1_std_core::{find_property, find_property_string, ExprData};
    find_property_string(props.clone(), prop_name.clone(), source_indices.clone()).or_else(|| {
        let n = find_property(props, prop_name, source_indices.clone())?;
        match (*n.expr_data).clone() {
            ExprData::ExprVar { .. } => {
                let s = authored_name_at(source_indices, n);
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            }
            _ => None,
        }
    })
}

pub fn collect_rest_transport_operations(
    module: &Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> RestTransportCollectResult {
    use crate::v1_std_core::{
        is_rest_transport, transport_method_key, transport_path_template_key,
    };
    let mut out = Vec::new();
    let mut errors = Vec::new();
    fn walk(
        n: &Rc<Node>,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        service_ctx: Option<String>,
        out: &mut Vec<DeclaredRestTransportOp>,
        errors: &mut Vec<RestTransportFactError>,
    ) {
        let ctx_for_children = match &n.transport {
            Some(t)
                if !is_rest_transport(t.clone(), source_indices.clone()) && !n.name.is_empty() =>
            {
                Some(n.name.clone())
            }
            _ => service_ctx.clone(),
        };

        if let Some(t) = &n.transport {
            if is_rest_transport(t.clone(), source_indices.clone()) {
                let Some(svc) = service_ctx.clone() else {
                    errors.push(RestTransportFactError::MissingServiceScope {
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                let method = rest_transport_field_string(
                    t.properties.clone(),
                    transport_method_key(),
                    source_indices.clone(),
                );
                let Some(method) = method else {
                    errors.push(RestTransportFactError::MissingMethodProperty {
                        service: svc.clone(),
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                let path = rest_transport_field_string(
                    t.properties.clone(),
                    transport_path_template_key(),
                    source_indices.clone(),
                );
                let Some(path) = path else {
                    errors.push(RestTransportFactError::MissingPathProperty {
                        service: svc.clone(),
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                out.push(DeclaredRestTransportOp {
                    service: svc,
                    name: n.name.clone(),
                    method,
                    path,
                });
            }
        }

        for c in n.children.iter() {
            walk(
                c,
                source_indices.clone(),
                ctx_for_children.clone(),
                out,
                errors,
            );
        }
    }
    walk(module, source_indices, None, &mut out, &mut errors);
    RestTransportCollectResult { ops: out, errors }
}

// --- Wire value serialization (folded from wire_value_serialize.rs) ---
// Pure coproduct wire-policy projection for interpreter REST bodies — zero host I/O.

type WireSerializeResult<T> = Result<T, String>;

pub fn resolve_coproduct_wire_policy(
    coproduct_name: &str,
    modules: &[Rc<TypedModule>],
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<Rc<crate::v1_compiler_emit_rust::RustEnumWireSerde>> {
    use crate::v1_compiler_emit_rust::resolve_local_coproduct_wire_policy;
    use crate::v1_std_core::module_imports;
    let si = Rc::new(source_indices.clone());
    let mut matches: Vec<Rc<crate::v1_compiler_emit_rust::RustEnumWireSerde>> = Vec::new();
    for tm in modules {
        let imports = module_imports(tm.module.clone());
        if let Some(local) = resolve_local_coproduct_wire_policy(
            coproduct_name.to_string(),
            false,
            tm.items.clone(),
            imports,
            si.clone(),
        ) {
            if local.error_message.is_none() {
                matches.push(local);
            }
        }
    }
    if matches.is_empty() {
        None
    } else if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        let first = &matches[0];
        if matches.iter().all(|m| m == first) {
            Some(first.clone())
        } else {
            None
        }
    }
}

fn wire_resolve_sym(ctx: &v1_interpreter::InterpContext, sym: v1_interpreter::Symbol) -> String {
    ctx.resolve(sym)
}

pub fn value_to_wire_json(
    val: &v1_interpreter::Value,
    ctx: &v1_interpreter::InterpContext,
) -> WireSerializeResult<serde_json::Value> {
    match val {
        v1_interpreter::Value::Variant {
            type_name,
            variant_name,
            fields,
        } => serialize_variant_to_wire_json(
            &wire_resolve_sym(ctx, *type_name),
            &wire_resolve_sym(ctx, *variant_name),
            fields,
            ctx,
        ),
        v1_interpreter::Value::Null => Ok(serde_json::Value::Null),
        v1_interpreter::Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        v1_interpreter::Value::Int(n) => Ok(serde_json::json!(*n)),
        v1_interpreter::Value::Float(f) => Ok(serde_json::json!(*f)),
        v1_interpreter::Value::Str(s) => {
            if s.starts_with('[') || s.starts_with('{') {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s) {
                    return Ok(parsed);
                }
            }
            Ok(serde_json::Value::String(s.clone()))
        }
        v1_interpreter::Value::List(items) => {
            let mut arr = Vec::with_capacity(items.len());
            for item in items.iter() {
                arr.push(value_to_wire_json(item, ctx)?);
            }
            Ok(serde_json::Value::Array(arr))
        }
        v1_interpreter::Value::Set(members) => Ok(serde_json::Value::Array(
            members
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        )),
        v1_interpreter::Value::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m.iter() {
                let key = match k.value_ref() {
                    v1_interpreter::Value::Str(s) => s.clone(),
                    other => {
                        return Err(format!(
                            "cannot serialize map with non-string key to JSON (got {other:?} key)"
                        ))
                    }
                };
                obj.insert(key, value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        v1_interpreter::Value::Record { fields, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, v1_interpreter::Value::Null) {
                    continue;
                }
                obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        v1_interpreter::Value::Unit => Ok(serde_json::Value::Null),
        v1_interpreter::Value::Closure { .. } => {
            Ok(serde_json::Value::String("<closure>".to_string()))
        }
        v1_interpreter::Value::Fn { node } => {
            Ok(serde_json::Value::String(format!("<fn {}>", node.name)))
        }
    }
}

fn serialize_variant_to_wire_json(
    type_name: &str,
    variant_name: &str,
    fields: &[(v1_interpreter::Symbol, v1_interpreter::Value)],
    ctx: &v1_interpreter::InterpContext,
) -> WireSerializeResult<serde_json::Value> {
    use crate::v1_compiler_emit_rust::{
        policy_is_string_variant, policy_is_untagged, policy_serde_tag_field,
        rust_serde_tag_attr, rust_tagged_object_policy, wire_variant_tag_for_policy,
    };
    let policy = resolve_coproduct_wire_policy(
        type_name,
        ctx.modules.iter().as_ref(),
        ctx.source_indices.as_ref(),
    )
    .unwrap_or_else(|| rust_tagged_object_policy());

    if policy.error_message.is_some() {
        return Err(policy
            .error_message
            .clone()
            .unwrap_or_else(|| format!("wire policy error for coproduct {type_name}")));
    }

    if policy_is_untagged(policy.clone()) {
        return serialize_untagged_variant(fields, ctx);
    }

    if policy_is_string_variant(policy.clone()) {
        let tag = wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .ok_or_else(|| format!("no wire tag for string variant {type_name}::{variant_name}"))?;
        return Ok(serde_json::Value::String(tag));
    }

    if let Some(tag_field) = policy_serde_tag_field(policy.clone()) {
        let wire_tag = wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .ok_or_else(|| {
                format!("no wire tag for internally-tagged variant {type_name}::{variant_name}")
            })?;
        let mut obj = serde_json::Map::new();
        obj.insert(tag_field, serde_json::Value::String(wire_tag));
        for (k, v) in fields.iter() {
            if matches!(v, v1_interpreter::Value::Null) {
                continue;
            }
            obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
        }
        return Ok(serde_json::Value::Object(obj));
    }

    let tag_key = policy_serde_tag_field(policy.clone()).unwrap_or_else(|| "_variant".to_string());
    let default_tag = if policy.enum_attr == rust_serde_tag_attr() {
        variant_name.to_string()
    } else {
        wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .unwrap_or_else(|| variant_name.to_string())
    };
    let mut obj = serde_json::Map::new();
    obj.insert(tag_key, serde_json::Value::String(default_tag));
    for (k, v) in fields.iter() {
        if matches!(v, v1_interpreter::Value::Null) {
            continue;
        }
        obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
    }
    Ok(serde_json::Value::Object(obj))
}

fn serialize_untagged_variant(
    fields: &[(v1_interpreter::Symbol, v1_interpreter::Value)],
    ctx: &v1_interpreter::InterpContext,
) -> WireSerializeResult<serde_json::Value> {
    let mut values: Vec<serde_json::Value> = fields
        .iter()
        .map(|(_, v)| v)
        .filter(|v| !matches!(v, v1_interpreter::Value::Null))
        .map(|v| value_to_wire_json(v, ctx))
        .collect::<Result<Vec<_>, _>>()?;
    match values.len() {
        0 => Ok(serde_json::Value::Null),
        1 => Ok(values.remove(0)),
        _ => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, v1_interpreter::Value::Null) {
                    continue;
                }
                obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
    }
}

#[cfg(test)]
mod import_closure_equivalence_tests {
    use super::{
        build_module_graph_facts_live, build_module_index, build_multi_entry_index,
        closure_subject_for_entry, default_source_roots, floor_discovery_path_excluded,
        import_closure_live_paths, module_graph_facts_build_count_for_test,
        reset_module_graph_facts_build_count_for_test, resolve_transitively,
        resolve_transitively_bfs_legacy, witness_layer_roots, workspace_relative_repo_path,
    };
    use std::collections::{BTreeSet, HashMap};
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn closure_paths(
        sources: &[Rc<crate::v1_compiler_compile::SourceFile>],
    ) -> std::collections::BTreeSet<String> {
        sources
            .iter()
            .map(|s| workspace_relative_repo_path(&s.path))
            .collect()
    }

    fn assert_bfs_matches_import_closure_live_with_facts(
        entry_rel: &str,
        index: &super::ModuleSourceIndex,
        facts: &super::ModuleGraphFactsLive,
    ) {
        let ws = workspace_root();
        let entry_abs = ws.join(entry_rel);
        let content =
            std::fs::read_to_string(&entry_abs).unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
        let entry_source = Rc::new(crate::v1_compiler_compile::SourceFile {
            path: entry_abs.to_string_lossy().into_owned(),
            content,
        });
        let mut seen: HashMap<String, Rc<crate::v1_compiler_compile::SourceFile>> = HashMap::new();
        if let Some(mod_path) = super::extract_module_path(&entry_source.content) {
            seen.insert(mod_path, entry_source.clone());
        }
        let bfs = resolve_transitively_bfs_legacy(vec![entry_source.clone()], index, seen);
        let repointed = resolve_transitively(vec![entry_source], index, facts)
            .unwrap_or_else(|e| panic!("resolve_transitively {entry_rel}: {e}"));
        let live = super::import_closure_live_paths_with_facts(entry_rel, facts);
        let bfs_paths = closure_paths(&bfs);
        let repointed_paths = closure_paths(&repointed);
        let live_paths: BTreeSet<String> = live
            .iter()
            .map(|p| workspace_relative_repo_path(p))
            .collect();
        assert_eq!(
            repointed_paths, bfs_paths,
            "repointed closure diverged from legacy BFS for {entry_rel}"
        );
        assert_eq!(
            live_paths, bfs_paths,
            "import_closure_live diverged from legacy BFS for {entry_rel}"
        );
    }

    fn assert_bfs_matches_import_closure_live(entry_rel: &str, pool_roots: &[String]) {
        let index = build_module_index(pool_roots);
        let facts = super::build_module_graph_facts_live(pool_roots);
        assert_bfs_matches_import_closure_live_with_facts(entry_rel, &index, &facts);
    }

    /// Floor witness entry paths enrolled by the source-root `*_test.dag` pass
    /// (`gunbc.ci_layer_roots.witness_layer_roots`), minus the model exclusion list.
    /// Avoids `discover_floor_corpus_rows` lens-hygiene work — closure set-identity
    /// only needs the witness entry roster, not inert-lens classification.
    fn floor_witness_entry_paths_for_oracle() -> BTreeSet<String> {
        let mut entries = BTreeSet::new();
        for root in default_source_roots() {
            let mut dag_files = Vec::new();
            super::collect_dag_files_tolerant(Path::new(&root), &mut dag_files);
            for path in dag_files {
                let rel = workspace_relative_repo_path(&path.to_string_lossy());
                if !rel.ends_with("_test.dag") || floor_discovery_path_excluded(&rel) {
                    continue;
                }
                entries.insert(rel);
            }
        }
        entries
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_whole_floor_corpus() {
        let roots = default_source_roots();
        let entries = floor_witness_entry_paths_for_oracle();
        assert!(
            entries.len() >= 4,
            "import-closure semantic oracle expects the full floor roster (got {})",
            entries.len()
        );
        let index = build_module_index(&roots);
        let facts = super::build_module_graph_facts_live(&roots);
        for entry_rel in entries {
            assert_bfs_matches_import_closure_live_with_facts(&entry_rel, &index, &facts);
        }
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_conformance_entry() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &roots,
        );
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_floor_gate_entry() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live("dag/tools/floor_effect_gate_witness.dag", &roots);
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_budget_roster_completeness() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live(
            "src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag",
            &roots,
        );
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_fold_list_generic_instantiation() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live(
            "src/v2/test/claim/fold_list_generic_instantiation.dag",
            &roots,
        );
    }

    fn module_paths_for_sources(
        sources: &[Rc<crate::v1_compiler_compile::SourceFile>],
    ) -> Vec<String> {
        let mut out: Vec<String> = sources
            .iter()
            .filter_map(|s| super::extract_module_path(&s.content))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    #[test]
    fn import_closure_module_path_set_identity_matches_legacy_bfs_on_witness_roots() {
        let roots = default_source_roots();
        let entries = [
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            "dag/tools/floor_effect_gate_witness.dag",
            "src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag",
            "src/v2/test/claim/fold_list_generic_instantiation.dag",
        ];
        for entry_rel in entries {
            let ws = workspace_root();
            let index = build_module_index(&roots);
            let content = std::fs::read_to_string(ws.join(entry_rel))
                .unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
            let entry_source = Rc::new(crate::v1_compiler_compile::SourceFile {
                path: ws.join(entry_rel).to_string_lossy().into_owned(),
                content,
            });
            let mut seen: HashMap<String, Rc<crate::v1_compiler_compile::SourceFile>> =
                HashMap::new();
            if let Some(mod_path) = super::extract_module_path(&entry_source.content) {
                seen.insert(mod_path, entry_source.clone());
            }
            let bfs = resolve_transitively_bfs_legacy(vec![entry_source.clone()], &index, seen);
            let facts = super::build_module_graph_facts_live(&roots);
            let repointed =
                resolve_transitively(vec![entry_source], &index, &facts).expect("repointed");
            let bfs_modules = module_paths_for_sources(&bfs);
            let repointed_modules = module_paths_for_sources(&repointed);
            assert_eq!(
                repointed_modules, bfs_modules,
                "module-path set identity diverged for {entry_rel}"
            );
            let live = super::import_closure_live_paths_with_facts(entry_rel, &facts);
            let live_modules: Vec<String> = live
                .iter()
                .filter_map(|p| {
                    let path = ws.join(super::workspace_relative_repo_path(p));
                    std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|c| super::extract_module_path(&c))
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            assert_eq!(
                live_modules, bfs_modules,
                "import_closure_live module-path set diverged for {entry_rel}"
            );
        }
    }

    #[test]
    fn module_graph_facts_scanned_once_per_multi_entry_index_hot_path() {
        reset_module_graph_facts_build_count_for_test();
        let ws = workspace_root();
        let roots = default_source_roots();
        let index = build_multi_entry_index(&roots);
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "module graph facts must be built once with MultiEntryIndex"
        );
        let budget = ws
            .join("src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag")
            .to_string_lossy()
            .into_owned();
        let fold = ws
            .join("src/v2/test/claim/fold_list_generic_instantiation.dag")
            .to_string_lossy()
            .into_owned();
        closure_subject_for_entry(&index, &budget).expect("budget_roster closure");
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "budget_roster closure must not re-scan corpus for facts"
        );
        closure_subject_for_entry(&index, &fold).expect("fold_list closure");
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "second entry closure must not re-scan corpus for facts"
        );
    }

    #[test]
    fn resolve_transitively_threads_prebuilt_facts_without_rescan() {
        reset_module_graph_facts_build_count_for_test();
        let roots = default_source_roots();
        let index = build_module_index(&roots);
        let facts = build_module_graph_facts_live(&roots);
        assert_eq!(module_graph_facts_build_count_for_test(), 1);
        let entries = [
            "src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag",
            "src/v2/test/claim/fold_list_generic_instantiation.dag",
        ];
        let mut entry_sources = Vec::new();
        for entry_rel in entries {
            let content = std::fs::read_to_string(workspace_root().join(entry_rel))
                .unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
            entry_sources.push(Rc::new(crate::v1_compiler_compile::SourceFile {
                path: workspace_root()
                    .join(entry_rel)
                    .to_string_lossy()
                    .into_owned(),
                content,
            }));
        }
        resolve_transitively(entry_sources, &index, &facts).expect("union closure");
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "multi-entry resolve_transitively must not re-scan when facts are threaded"
        );
    }

    #[test]
    fn import_closure_live_drift_discriminates_under_declaration() {
        let roots = default_source_roots();
        let live = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &roots,
        )
        .expect("live closure");
        let mut without_entry: std::collections::BTreeSet<String> = live
            .iter()
            .map(|p| workspace_relative_repo_path(p))
            .filter(|p| p != "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag")
            .collect();
        let repointed = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &roots,
        )
        .expect("live closure again");
        let full: std::collections::BTreeSet<String> = repointed
            .iter()
            .map(|p| workspace_relative_repo_path(p))
            .collect();
        assert_ne!(
            without_entry, full,
            "RED control: dropped entry must diverge"
        );
        without_entry.insert("src/v2/std/__bogus_never_imported__.dag".to_string());
        assert_ne!(
            without_entry, full,
            "RED control: bogus path must diverge from live closure"
        );
    }

    #[test]
    fn import_closure_live_uses_witness_layer_roots_without_extra_resolve() {
        let ws = workspace_root();
        let rel_roots: Vec<String> = witness_layer_roots();
        let abs_roots: Vec<String> = rel_roots
            .iter()
            .map(|r| ws.join(r).to_string_lossy().into_owned())
            .collect();
        let from_rel = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &rel_roots,
        )
        .expect("relative roots");
        let from_abs = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &abs_roots,
        )
        .expect("absolute roots");
        let norm = |paths: Vec<String>| {
            paths
                .into_iter()
                .map(|p| workspace_relative_repo_path(&p))
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(norm(from_rel), norm(from_abs));
    }
}
