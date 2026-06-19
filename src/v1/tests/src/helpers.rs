//! Test helpers for the v2 compiler test suite.
//!
//! All helpers call stage0 functions directly — no v1 interpreter, no Value wrapping.

use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{
    compile_sources_with_options, compile_to_resolved, CompilePipelineOptions, PipelineResult,
    ResolvedPipelineResult, SourceFile,
};
use v1_compiler::v1_compiler_parse::ParseResult;
use v1_compiler::v1_std_core::Token;

// ── Workspace helpers ────────────────────────────────────────────────────

pub fn workspace_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("could not find workspace root")
        .to_path_buf()
}

pub fn read_v2_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

/// Source roots where `.dag` files can be found.
pub fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v1"), ws.join("dsl")]
}

// ── Tokenize + Parse ─────────────────────────────────────────────────────

pub fn tokenize(source: &str) -> Rc<Vec<Rc<Token>>> {
    v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), "test.dag".to_string())
}

pub fn parse_source(source: &str) -> Rc<ParseResult> {
    parse_source_named("test.dag", source)
}

pub fn parse_source_named(filename: &str, source: &str) -> Rc<ParseResult> {
    let tokens =
        v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), filename.to_string());
    let source_index =
        v1_compiler::v1_std_core::build_newline_index(filename.to_string(), source.to_string());
    let mut source_indices = std::collections::HashMap::new();
    source_indices.insert(filename.to_string(), source_index);
    v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices))
}

pub fn assert_parses(source: &str, label: &str) {
    let result = parse_source(source);
    if let Some(ref err) = result.error {
        panic!(
            "{} had parse error: {}",
            label,
            v1_compiler::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    assert!(result.module.is_some(), "{} produced no module", label);
}

pub fn assert_parses_strict(relative_path: &str) {
    let source = read_v2_file(relative_path);
    let result = parse_source(&source);
    if let Some(ref err) = result.error {
        let span = {
            let s = v1_compiler::v1_std_core::diagnostic_to_span(err.diagnostic.clone());
            let line = source[..s.start.max(0) as usize]
                .chars()
                .filter(|c| *c == '\n')
                .count()
                + 1;
            format!(" (line {})", line)
        };
        panic!(
            "{} had parse error: {}{}",
            relative_path,
            v1_compiler::v1_std_core::diagnostic_to_message(err.diagnostic.clone()),
            span
        );
    }
    assert!(
        result.module.is_some(),
        "{} produced no module and no error",
        relative_path
    );
}

// ── Import-driven source resolution (FF-9) ──────────────────────────────
//
// The compiler takes a flat List<SourceFile>. This layer resolves imports
// transitively: parse the entry source, discover its imports, load them
// from source roots, recurse. Each module is loaded exactly once (memoized
// by module path). The result is the minimal transitive closure.
//
// Module identity comes from the parser (single authority). The module
// index is built once via OnceLock and maps module_path → file_path.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Build module index using the parser as single authority for module names.
/// Scans source roots recursively, tokenizes+parses each .dag file to
/// extract the module declaration. Built once via OnceLock.
fn build_module_index_for_roots(
    roots: &[std::path::PathBuf],
) -> HashMap<String, std::path::PathBuf> {
    let mut index = HashMap::new();
    for root in roots {
        if root.exists() {
            scan_dag_files(root, &mut index);
        }
    }
    index
}

fn build_module_index() -> HashMap<String, std::path::PathBuf> {
    build_module_index_for_roots(&source_roots())
}

fn scan_dag_files(dir: &std::path::Path, index: &mut HashMap<String, std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dag_files(&path, index);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            if let Some(module_path) = extract_module_declaration(&path) {
                // Co-root overlay: later roots win (matches cli_run build_module_index).
                index.insert(module_path, path);
            }
        }
    }
}

/// Extract module declaration using the parser — single authority.
fn extract_module_declaration(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let result = parse_source(&content);
    result.module.as_ref().map(|m| m.name.clone())
}

static MODULE_INDEX: OnceLock<HashMap<String, std::path::PathBuf>> = OnceLock::new();

fn module_index() -> &'static HashMap<String, std::path::PathBuf> {
    MODULE_INDEX.get_or_init(build_module_index)
}

/// Extract import module paths using the actual parser — no parallel
/// string-scanning implementation. The parser is the single authority
/// for import syntax.
fn extract_imports(source: &str) -> Vec<String> {
    let result = parse_source(source);
    match &result.module {
        Some(module) => v1_compiler::v1_std_core::module_imports(module.clone())
            .iter()
            .map(|imp| imp.name.clone())
            .collect(),
        None => vec![],
    }
}

/// Resolve imports transitively from an entry source. Returns the minimal
/// set of SourceFiles needed — each module loaded exactly once.
/// Lookups use the cached module index (parser-backed, built once).
pub fn resolve_imports_transitively(entry_path: &str, entry_content: &str) -> Vec<Rc<SourceFile>> {
    resolve_imports_transitively_with_index(entry_path, entry_content, module_index())
}

pub fn resolve_imports_transitively_with_source_roots(
    entry_path: &str,
    entry_content: &str,
    source_roots: &[std::path::PathBuf],
) -> Vec<Rc<SourceFile>> {
    let index = build_module_index_for_roots(source_roots);
    resolve_imports_transitively_with_index(entry_path, entry_content, &index)
}

fn display_source_path(path: &std::path::Path, ws: &std::path::Path) -> String {
    path.strip_prefix(ws)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn resolve_imports_transitively_with_index(
    entry_path: &str,
    entry_content: &str,
    module_index: &HashMap<String, std::path::PathBuf>,
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue: Vec<(String, String)> = Vec::new(); // (path, content)

    // Seed with the entry
    queue.push((entry_path.to_string(), entry_content.to_string()));

    while let Some((_path, content)) = queue.pop() {
        let imports = extract_imports(&content);
        for module_path in imports {
            if seen.contains_key(&module_path) {
                continue; // already loaded — O(1) check
            }
            if let Some(file_path) = module_index.get(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    let rel_path = display_source_path(file_path, &ws);
                    let source = Rc::new(SourceFile {
                        path: rel_path.clone(),
                        content: file_content.clone(),
                    });
                    seen.insert(module_path.clone(), source);
                    queue.push((rel_path, file_content));
                }
            }
            // If not found, the compiler's resolve stage will
            // report the unresolved import — no silent fallback.
        }
    }

    // Return: dependencies first (they'll be sorted by resolve_modules anyway),
    // then the entry source last.
    let mut sources: Vec<Rc<SourceFile>> = seen.into_values().collect();
    sources.push(Rc::new(SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    }));
    sources
}

// ── Full pipeline ────────────────────────────────────────────────────────

fn analyze_complexity_options() -> CompilePipelineOptions {
    CompilePipelineOptions {
        analyze_complexity: true,
    }
}

pub fn compile_dag_analyze_complexity(source: &str) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively("test.dag", source);
    compile_sources_with_options(
        Rc::new(sources),
        RenderTarget::Rust,
        analyze_complexity_options(),
    )
}

pub fn compile_multi_analyze_complexity(files: &[(&str, &str)]) -> Rc<PipelineResult> {
    let mut all_sources: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for (path, content) in files {
        let resolved = resolve_imports_transitively(path, content);
        for src in resolved {
            all_sources.entry(src.path.clone()).or_insert(src);
        }
    }
    let sources: Vec<Rc<SourceFile>> = all_sources.into_values().collect();
    compile_sources_with_options(
        Rc::new(sources),
        RenderTarget::Rust,
        analyze_complexity_options(),
    )
}

pub fn compile_dag(source: &str) -> Rc<PipelineResult> {
    compile_dag_named("test.dag", source, RenderTarget::Rust)
}

pub fn compile_dag_resolved(source: &str) -> Rc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", source);
    compile_to_resolved(Rc::new(sources))
}

pub fn compile_dag_target(source: &str, target: RenderTarget) -> Rc<PipelineResult> {
    compile_dag_named("test.dag", source, target)
}

pub fn compile_dag_named(filename: &str, source: &str, target: RenderTarget) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively(filename, source);
    v1_compiler::v1_compiler_compile::compile_sources(Rc::new(sources), target)
}

pub fn compile_dag_named_with_source_roots(
    filename: &str,
    source: &str,
    target: RenderTarget,
    source_roots: &[std::path::PathBuf],
) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively_with_source_roots(filename, source, source_roots);
    v1_compiler::v1_compiler_compile::compile_sources(Rc::new(sources), target)
}

pub fn compile_multi(files: &[(&str, &str)]) -> Rc<PipelineResult> {
    compile_multi_target(files, RenderTarget::Rust)
}

pub fn compile_multi_target(files: &[(&str, &str)], target: RenderTarget) -> Rc<PipelineResult> {
    let mut all_sources: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for (path, content) in files {
        let resolved = resolve_imports_transitively(path, content);
        for src in resolved {
            all_sources.entry(src.path.clone()).or_insert(src);
        }
    }
    let sources: Vec<Rc<SourceFile>> = all_sources.into_values().collect();
    v1_compiler::v1_compiler_compile::compile_sources(Rc::new(sources), target)
}

// ── Result inspection ────────────────────────────────────────────────────

pub fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

pub fn assert_no_diagnostics(result: &PipelineResult) {
    // Complexity violations are non-blocking analyzer limitations.
    // Only assert on hard errors (type/resolve/ownership).
    let msgs: Vec<_> = diagnostic_messages(result)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty(),
        "expected 0 hard diagnostics, got {}: {:?}",
        msgs.len(),
        msgs,
    );
}

pub fn find_file(result: &PipelineResult, path: &str) -> String {
    result
        .files
        .iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| {
            let available: Vec<_> = result.files.iter().map(|f| f.path.as_str()).collect();
            panic!(
                "missing emitted file '{}', available: {:?}",
                path, available
            )
        })
        .content
        .clone()
}

pub fn has_file(result: &PipelineResult, path: &str) -> bool {
    result.files.iter().any(|f| f.path == path)
}

pub fn emitted_file_paths(result: &PipelineResult) -> Vec<String> {
    result.files.iter().map(|f| f.path.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = format!(
            "v2-helper-tests-{}-{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock before unix epoch")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn scan_dag_files_panics_on_duplicate_module_names() {
        let dir = temp_dir("duplicate-modules");
        let a = dir.join("a.dag");
        let b = dir.join("nested").join("b.dag");
        std::fs::create_dir_all(b.parent().expect("nested dir")).expect("create nested dir");
        std::fs::write(&a, "module duplicate.test\n").expect("write first module");
        std::fs::write(&b, "module duplicate.test\n").expect("write second module");

        let result = std::panic::catch_unwind(|| {
            let mut index = HashMap::new();
            scan_dag_files(&dir, &mut index);
        });

        let panic_payload = result.expect_err("expected duplicate module panic");
        let message = if let Some(message) = panic_payload.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = panic_payload.downcast_ref::<&str>() {
            message.to_string()
        } else {
            String::new()
        };

        assert!(
            message.contains("duplicate module declaration for duplicate.test"),
            "panic should identify duplicate module names, got: {message}"
        );
    }

    #[test]
    fn resolver_imports_ephemeral_generated_source_root() {
        let entry_root = temp_dir("entry-root");
        let generated_root = temp_dir("generated-root");
        let generated_dir = generated_root.join("generated");
        std::fs::create_dir_all(&generated_dir).expect("create generated dir");
        std::fs::write(
            generated_dir.join("method_template_projection.dag"),
            "module generated.method_template_projection\n\nfn generated_answer() -> Int { 41 }\n",
        )
        .expect("write generated module");

        let entry_source = "\
module ephemeral.entry

import generated.method_template_projection { generated_answer }

fn main() -> Int { generated_answer() }
";
        let result = compile_dag_named_with_source_roots(
            "ephemeral/entry.dag",
            entry_source,
            RenderTarget::Dag,
            &[entry_root.clone(), generated_root.clone()],
        );

        assert_no_diagnostics(&result);
        let loaded_paths: Vec<_> = result
            .newline_indices
            .iter()
            .map(|index| index.file.as_str())
            .collect();
        assert!(
            loaded_paths
                .iter()
                .any(|path| path.contains("generated/method_template_projection.dag")),
            "expected generated temp-root module to be loaded, got: {loaded_paths:?}"
        );
        assert!(
            !loaded_paths.iter().any(|path| path.starts_with("src/")),
            "ephemeral generated dependency must not be committed under src/: {loaded_paths:?}"
        );

        let _ = std::fs::remove_dir_all(entry_root);
        let _ = std::fs::remove_dir_all(generated_root);
    }
}
