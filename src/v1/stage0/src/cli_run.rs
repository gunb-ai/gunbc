use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

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
                index.insert(module_path.clone(), rel);
            }
        }
    }
    index
}

const CI_LAYER_ROOTS_AUTHORITY_REL: &str = "dsl/gunbc/ci_layer_roots.dag";
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

pub fn qualified_name_value_to_module_path(value: &v1_interpreter::Value) -> String {
    v1_interpreter::qualified_name_value_to_module_path(value)
}

pub fn qualified_name_value_from_dotted_string(
    ctx: &v1_interpreter::InterpContext,
    dotted: &str,
) -> v1_interpreter::Value {
    use v1_interpreter::{sorted_fields, Value};

    let qn_variant = |variant: &str, fields: Vec<_>| Value::Variant {
        type_name: ctx.sym("QualifiedName"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(fields),
    };
    if dotted.is_empty() {
        return qn_variant("QnEmpty", vec![]);
    }
    let mut qn = qn_variant("QnEmpty", vec![]);
    for seg in dotted.split('.').rev() {
        qn = qn_variant(
            "QnCons",
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
    if !sources.iter().any(|s| s.path == rel_path) {
        sources.push(entry_source);
    }
    Ok(sources)
}

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

    let skipped_moduleless = moduleless_dag_entry_paths(&entry_files);
    report_moduleless_dag_entry_skips(&skipped_moduleless);

    let mut seen: HashMap<String, Rc<v1_compiler_compile::SourceFile>> = HashMap::new();
    let mut entry_for_queue = Vec::new();
    for (path, content) in &entry_files {
        if let Some(mod_path) = extract_module_path(content) {
            let source = Rc::new(v1_compiler_compile::SourceFile {
                path: path.clone(),
                content: content.clone(),
            });
            seen.insert(mod_path, source.clone());
            entry_for_queue.push(source);
        }
    }

    let mut sources = resolve_transitively(entry_for_queue, &index, seen);
    for (path, content) in entry_files {
        if extract_module_path(&content).is_none() {
            continue;
        }
        if !sources.iter().any(|s| s.path == path) {
            sources.push(Rc::new(v1_compiler_compile::SourceFile { path, content }));
        }
    }
    sources
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
    resolve_entry_graph_with_index(&index, entry_file)
}

pub struct MultiEntryIndex {
    source_files: ModuleSourceIndex,
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
    let entry = "dsl/gunbc/output_policy.dag";
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
    let entry = "dsl/extdeps/render/surface.dag";
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

fn dsl_source_roots(source_roots: &[String]) -> Vec<String> {
    let mut dsl: Vec<String> = source_roots
        .iter()
        .filter(|r| {
            let p = Path::new(r.as_str());
            p.ends_with("dsl") || p.file_name().is_some_and(|n| n == "dsl")
        })
        .cloned()
        .collect();
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

pub fn precompute_whole_tree_published_mock_keys(
    source_roots: &[String],
) -> Result<std::collections::HashSet<String>, String> {
    let dsl_roots = dsl_source_roots(source_roots);
    if dsl_roots.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let index = build_module_index(&dsl_roots);
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
    let mut seen: HashMap<String, Rc<v1_compiler_compile::SourceFile>> = HashMap::new();
    for d in &declarers {
        if let Some(mp) = extract_module_path(&d.content) {
            seen.insert(mp, d.clone());
        }
    }
    let all_sources = resolve_transitively(declarers, &index, seen);
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
    let sources = load_sources_for_entry_with_index(&index.source_files, entry)?;
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
            ExitClass::Failure(_) => ClaimOutcome::Fail,
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
        Some("dsl/tools/gunbc_ci.dag".to_string()),
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
        None => load_sources(&source_roots),
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

enum ExitClass {
    Success,
    Failure(i32),
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
// itself without circularity). The *rendering* of these timings now lives in `dsl/gunbc/ci_render.dag`
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
    "dsl/test/claim/wet_hermetic_equivalence_witness_test.dag";
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
                crate::module_path_index::fact_cardinality_census::extract_top_level_decls(content)
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
            let decl_end = decls.get(i + 1).map(|(l, _, _)| l - 1).unwrap_or(i64::MAX);
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
    entry_claims_touch_frontier(ctx, &list_value_from_vec(entry_frontier))
}

fn entry_claims_touch_frontier(
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
        let frontier_index = build_multi_entry_index(source_roots);
        match collect_frontier_seeds_from_diff_line_ranges(&frontier_index, &line_ranges_by_file) {
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

// SCAFFOLD: folds into a .dag execution witness when the discovery/diff seed plumbing
// migrates off the v1 host layer (§6 dissolution trigger)
#[cfg(test)]
mod node_frontier_plumbing_controls {
    use super::{
        build_multi_entry_index, collect_frontier_seeds_from_diff_line_ranges,
        entry_touches_frontier_seeds, parse_unified_diff_line_ranges,
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
    // If this line shifts the test may generate force_run_all — a loud failure, not a silent pass.
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

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dsl").to_string_lossy().into_owned(),
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
        let ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .expect("seeds from outside-file diff");
        assert!(
            !seeds.force_run_all,
            "diff on a .dag file at a data-item line must not force_run_all \
             (if it does, OUTSIDE_DATA_LINE may have drifted off the data declaration)"
        );

        // FIXTURE's context does not hold OUTSIDE_FILE's items → frontier empty → skip.
        let ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        assert!(
            !entry_touches_frontier_seeds(&ctx, &abs(&ws, FIXTURE), &seeds).expect("touch check"),
            "entry must NOT touch frontier when diff is on a file outside its import closure"
        );
    }

    // Control 2 (RED/function_edited): diff edits a test fn declaration →
    // edited_test_fns populated → function_edited=true forces run for that row.
    #[test]
    fn red_function_edited_populates_edited_test_fns() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        // Line 74: `test fn floor_disc_witness_a_only_holds() -> Bool {`
        let diff = diff_at(&abs(&ws, FIXTURE), 74);
        let ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .expect("seeds from test-fn-line diff");
        assert!(
            !seeds.force_run_all,
            "editing a test fn declaration line must not force_run_all"
        );
        assert!(
            seeds
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "diff at test fn declaration line must populate edited_test_fns with the function name"
        );
    }

    // Control 3 (RED/node_frontier): diff on a data item referenced by a claim →
    // entry_touches_frontier_seeds returns true → runs.
    #[test]
    fn red_node_frontier_fires_for_referenced_data_item() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        // Line 15: `data floor_disc_node_a` — directly referenced by floor_disc_claim_on_a.
        let diff = diff_at(&abs(&ws, FIXTURE), 15);
        let ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .expect("seeds from referenced-node diff");
        assert!(
            !seeds.force_run_all,
            "diff on a referenced data item must not force_run_all"
        );
        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &abs(&ws, FIXTURE)).expect("fixture resolves");
        let ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        assert!(
            entry_touches_frontier_seeds(&ctx, &abs(&ws, FIXTURE), &seeds).expect("touch check"),
            "entry must touch frontier when diff is on a data item referenced by a claim"
        );
    }

    // Control 4 (fail-closed): non-.dag changed file → force_run_all.
    #[test]
    fn fail_closed_non_dag_file_forces_run_all() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = "diff --git a/src/v1/stage0/src/cli_run.rs b/src/v1/stage0/src/cli_run.rs\n\
                    --- a/src/v1/stage0/src/cli_run.rs\n\
                    +++ b/src/v1/stage0/src/cli_run.rs\n\
                    @@ -1,0 +2,1 @@\n+// synthetic\n";
        let ranges = parse_unified_diff_line_ranges(diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .expect("seeds from .rs diff");
        assert!(
            seeds.force_run_all,
            "diff on a non-.dag file must force_run_all (fail-closed)"
        );
    }

    // Control 5 (fail-closed): diff before first declaration in a .dag file → force_run_all.
    // The module header (line 1) precedes the first data/fn declaration.
    #[test]
    fn fail_closed_edit_before_first_decl_forces_run_all() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = diff_at(&abs(&ws, FIXTURE), 1);
        let ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .expect("seeds from pre-decl diff");
        assert!(
            seeds.force_run_all,
            "diff before first declaration must force_run_all (fail-closed)"
        );
    }

    // Control 6 (fail-closed / Q2 resolve-failure): diff names a .dag path that does not
    // exist → resolve_entry_with_index fails → force_run_all. Exercises the
    // collect_frontier_seeds_from_diff_line_ranges:Err arm at the resolve site.
    #[test]
    fn fail_closed_nonexistent_dag_path_forces_run_all() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = diff_at(&abs(&ws, "src/v2/lens/does_not_exist_sentinel.dag"), 10);
        let ranges = parse_unified_diff_line_ranges(&diff);
        let seeds = collect_frontier_seeds_from_diff_line_ranges(&index, &ranges)
            .expect("seeds from nonexistent-path diff");
        assert!(
            seeds.force_run_all,
            "diff naming a non-existent .dag path must force_run_all (resolve failure → fail-closed)"
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
        "dsl" => Ok("DslTree".to_string()),
        other => Err(format!(
            "source_root tagging: unknown --source-root '{other}' \
             (authority gunbc.ci_layer_roots.witness_layer_roots = [src/v2, dsl] -> \
             SourceRootRef {{V2Tree, DslTree}})"
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
            ws.join("dsl").to_string_lossy().into_owned(),
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
                ws.join("dsl/std/algebra.dag").to_str().unwrap(),
                &abs_roots
            )
            .unwrap(),
            "DslTree"
        );
        assert_eq!(
            source_root_ref_token_for_path("dsl/std/algebra.dag", &abs_roots).unwrap(),
            "DslTree"
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
            ws.join("dsl").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let scan_dirs = vec![
            "dsl/test/claim".to_string(),
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
            ws.join("dsl").to_string_lossy().into_owned(),
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
    path.to_string_lossy().replace('\\', "/")
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

const LANGUAGES_AUTHORITY_REL: &str = "dsl/std/languages.dag";

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
    rel == "src/v1/stage0/src/languages_consumer_census.rs"
        || rel.starts_with("src/v2/test/claim/languages_consumer_census/")
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
    for tree in &["dsl", "src"] {
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

#[cfg(test)]
mod module_path_index_tests {
    use super::*;

    #[test]
    fn cargo_build_resolves_by_module_path_not_directory_nickname() {
        let path = source_path_for_module_path("extdeps.cargo_build".to_string());
        assert_eq!(path, "dsl/extdeps/rust/cargo_build.dag");
    }

    #[test]
    fn git_module_resolves() {
        let path = source_path_for_module_path("extdeps.git".to_string());
        assert_eq!(path, "dsl/extdeps/git/git.dag");
    }

    #[test]
    fn co_root_overlay_last_root_wins_on_duplicate_module_path() {
        let path = source_path_for_module_path("extdeps.shell".to_string());
        assert_eq!(path, "src/v2/extdeps/shell.dag");
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
            vec!["dsl".to_string(), "src/v2".to_string()],
            "live authority value drifted from the expected [dsl, src/v2]"
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
                ws.join("dsl").to_string_lossy().into_owned(),
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
                "dsl/test/claim".to_string(),
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
            roster.iter().any(|p| p == "dsl/gunbc/ci_workflow.dag"),
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
