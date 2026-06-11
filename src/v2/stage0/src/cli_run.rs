// cli_run.rs — Hand-maintained Run subcommand handler.
// Not generated — survives stage0 regeneration.
// The generated main.rs calls handle_run() for the Run subcommand.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::std_syntax::LiteralValue;
use crate::v2_compiler_compile;
use crate::v2_compiler_infer_env::lookup_type_by_name;
use crate::v2_compiler_infer_items::{ItemKind, ResolvedGraph, TypedModule};
use crate::v2_interpreter;
use crate::v2_std_core::{
    authored_name_at, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    expr_var_name_at, field_init_node_name_at, field_init_node_value, has_child_named,
    is_interpreter_blocking_diagnostic, ExprData, InferredNode, NewlineIndex, Node,
};
use serde::Serialize;

/// Module that owns `UnifiedTestClaim` and its registration arms.
pub const UNIFIED_CLAIM_VERIFICATION_MODULE: &str = "v4.std.verification";
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
type ModuleSourceIndex = HashMap<String, Rc<v2_compiler_compile::SourceFile>>;

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
                    Rc::new(v2_compiler_compile::SourceFile {
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
    entry_sources: Vec<Rc<v2_compiler_compile::SourceFile>>,
    index: &ModuleSourceIndex,
    mut seen: HashMap<String, Rc<v2_compiler_compile::SourceFile>>,
) -> Vec<Rc<v2_compiler_compile::SourceFile>> {
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
) -> Result<Vec<Rc<v2_compiler_compile::SourceFile>>, String> {
    let index = build_module_index(source_roots);
    load_sources_for_entry_with_index(&index, entry_path)
}

fn entry_source_from_index_or_disk(
    index: &ModuleSourceIndex,
    entry_path: &str,
) -> Result<Rc<v2_compiler_compile::SourceFile>, String> {
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
    Ok(Rc::new(v2_compiler_compile::SourceFile {
        path: rel_path,
        content,
    }))
}

fn load_sources_for_entry_with_index(
    index: &ModuleSourceIndex,
    entry_path: &str,
) -> Result<Vec<Rc<v2_compiler_compile::SourceFile>>, String> {
    let entry_source = entry_source_from_index_or_disk(index, entry_path)?;
    let rel_path = entry_source.path.clone();

    let mut seen: HashMap<String, Rc<v2_compiler_compile::SourceFile>> = HashMap::new();
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
fn load_sources(source_roots: &[String]) -> Vec<Rc<v2_compiler_compile::SourceFile>> {
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

    let mut seen: HashMap<String, Rc<v2_compiler_compile::SourceFile>> = HashMap::new();
    let mut entry_for_queue = Vec::new();
    for (path, content) in &entry_files {
        let source = Rc::new(v2_compiler_compile::SourceFile {
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
            sources.push(Rc::new(v2_compiler_compile::SourceFile { path, content }));
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
        Rc<v2_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let index = build_module_index(source_roots);
    resolve_entry_graph_with_index(source_roots, &index, entry_file)
}

fn resolve_entry_graph_with_index(
    source_roots: &[String],
    index: &ModuleSourceIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v2_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let _ = source_roots;
    let sources = load_sources_for_entry_with_index(index, entry_file)?;
    resolved_graph_from_sources(sources)
}

/// Compile an already-assembled source closure to a resolved graph, or return
/// formatted blocking diagnostics. Shared by the per-entry path
/// (`resolve_entry_graph_with_index`) and the batched discovery path
/// (`discover_owned_data_decls`), which merges many entry closures into one
/// compile.
fn resolved_graph_from_sources(
    sources: Vec<Rc<v2_compiler_compile::SourceFile>>,
) -> Result<
    (
        Rc<v2_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let result = v2_compiler_compile::compile_to_resolved(Rc::new(sources));

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
    graph: &v2_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> v2_interpreter::InterpContext {
    v2_interpreter::InterpContext::new(graph, source_indices, false)
}

/// Run one Bool witness function against an already-resolved graph, classifying
/// the result the same way `handle_run_with_options`'s `--claim-run` branch
/// does (Bool true → Pass, false → Fail, anything else → diagnostic), but
/// without calling `std::process::exit`. Eager data-env is disabled to match
/// claim-run behavior (witnesses pull data lazily).
pub fn run_claim(ctx: &v2_interpreter::InterpContext, function: &str) -> ClaimOutcome {
    match v2_interpreter::run_in_context(ctx, function, false) {
        Ok(v2_interpreter::Value::Bool(true)) => ClaimOutcome::Pass,
        Ok(v2_interpreter::Value::Bool(false)) => ClaimOutcome::Fail,
        Ok(other) => ClaimOutcome::NotBool {
            got: format!("{}", other),
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
    ctx: &v2_interpreter::InterpContext,
    function: &str,
) -> Result<v2_interpreter::Value, String> {
    v2_interpreter::run_in_context(ctx, function, false).map_err(|e| format!("{}", e))
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
    let result = v2_compiler_compile::compile_to_resolved(Rc::new(sources));

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

    // Run the interpreter
    eprintln!("running {}()...", function);
    match v2_interpreter::run_with_options(
        graph,
        result.source_indices.clone(),
        &function,
        dry_run,
        !claim_run,
    ) {
        Ok(val) => {
            println!("{}", val);
            if claim_run {
                // Witness entry points return Bool; fail-closed like ProcessExit below.
                match &val {
                    v2_interpreter::Value::Bool(false) => std::process::exit(1),
                    v2_interpreter::Value::Bool(true) => return,
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
            match classify_exit(&val) {
                ExitClass::Success => {} // exit 0 (default)
                ExitClass::Failure(code) => std::process::exit(code),
                ExitClass::NotProcessExit { type_name } => {
                    eprintln!(
                        "error: function `{}` returned `{}`, not `ProcessExit`. \
                         Functions invoked via `dag run` must return std/process.dag's \
                         ProcessExit so the host can map success/failure to an exit code. \
                         Wrap your rich result type in ExitSuccess / ExitFailure, or pass \
                         --claim-run for Bool witness entry points under src/v4.",
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
fn classify_exit(val: &v2_interpreter::Value) -> ExitClass {
    match val {
        v2_interpreter::Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            if type_name != "ProcessExit" {
                return ExitClass::NotProcessExit {
                    type_name: type_name.clone(),
                };
            }
            match variant_name.as_str() {
                "ExitSuccess" => ExitClass::Success,
                "ExitFailure" => match fields.get("code") {
                    Some(v2_interpreter::Value::Int(n)) => ExitClass::Failure(*n as i32),
                    _ => ExitClass::Failure(1),
                },
                _ => ExitClass::NotProcessExit {
                    type_name: format!("ProcessExit::{}", variant_name),
                },
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

/// Glob claim corpus files, resolve each entry fail-closed, expose owned `data` facts.
pub fn discover_owned_data_decls(
    source_roots: &[String],
    scan_dir: &str,
    exclude_subpaths: &[String],
) -> Result<Vec<OwnedDataDeclRecord>, String> {
    let scan_path = Path::new(scan_dir);
    if !scan_path.is_dir() {
        return Err(format!("scan dir does not exist: {}", scan_dir));
    }

    let mut files = Vec::new();
    collect_dag_files(scan_path, &mut files);
    files.retain(|p| !path_excluded(p, exclude_subpaths));

    let module_index = build_module_index(source_roots);
    let mut all_records = Vec::new();
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

        let (graph, source_indices) =
            resolve_entry_graph_with_index(source_roots, &module_index, &entry)?;
        let si: HashMap<String, Rc<NewlineIndex>> = source_indices
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        all_records.extend(owned_data_decls_for_entry(
            &graph,
            &si,
            &entry,
            &entry_module,
        )?);
    }

    all_records.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.decl_name.cmp(&b.decl_name))
    });
    verify_bool_witness_transport_projection_complete(&all_records)?;
    Ok(all_records)
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
    s.replace('\\', "\\\\").replace('"', "\\\"")
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
    out.push_str("module v4.test.claim.workflow.host_discovered_owned_data_manifest\n\n\n");
    out.push_str("import v4.std.collection { List }\n");
    out.push_str("import v4.std.logic { Bool }\n");
    out.push_str(
        "import v4.test.claim.workflow.discovery_types {\n  OwnedBoolWitnessClaimInit,\n  OwnedDataDeclRecord,\n  OwnedDataDiscoveryReceipt,\n  OwnedNodeCorpusInit,\n  OwnedOtherInit,\n  ResolvedDeclRef,\n  unified_claim_arm_bool_witness_claim,\n  unified_claim_arm_node_corpus\n}\n\n\n",
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
