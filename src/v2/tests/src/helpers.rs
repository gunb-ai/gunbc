//! Test helpers for the v2 compiler test suite.
//!
//! All helpers call stage0 functions directly — no v1 interpreter, no Value wrapping.

use std::rc::Rc;
use v2_compiler::v2_compiler_artifact::RenderTarget;
use v2_compiler::v2_compiler_compile::{PipelineResult, SourceFile};
use v2_compiler::v2_compiler_parse::ParseResult;
use v2_compiler::v2_std_core::Token;

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

// ── Tokenize + Parse ─────────────────────────────────────────────────────

pub fn tokenize(source: &str) -> Vec<Rc<Token>> {
    v2_compiler::v2_compiler_tokenize::tokenize(source.to_string())
}

pub fn parse_source(source: &str) -> Rc<ParseResult> {
    let tokens = tokenize(source);
    v2_compiler::v2_compiler_parse::parse(tokens)
}

pub fn assert_parses(source: &str, label: &str) {
    let result = parse_source(source);
    if let Some(ref err) = result.error {
        panic!("{} had parse error: {}", label, err.name);
    }
    assert!(result.module.is_some(), "{} produced no module", label);
}

pub fn assert_parses_strict(relative_path: &str) {
    let source = read_v2_file(relative_path);
    let result = parse_source(&source);
    if let Some(ref err) = result.error {
        let span = {
            let s = &err.span;
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
            err.name,
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

use std::collections::HashMap;

/// Source roots where .dag files can be found. Module path segments map
/// to directory structure: `std.types` → `<root>/std/types.dag`.
fn source_roots() -> Vec<std::path::PathBuf> {
    let ws = workspace_root();
    vec![
        ws.join("dsl"),       // std.types → dsl/std/types.dag
        ws.join("src/v2"),    // v2.std.core → src/v2/00_core.dag (needs index)
    ]
}

/// Build an index of module_path → file_path by scanning source roots.
/// Only reads the `module` declaration line from each file — O(files) I/O,
/// one line per file.
fn build_module_index() -> HashMap<String, std::path::PathBuf> {
    let mut index = HashMap::new();
    for root in source_roots() {
        if root.exists() {
            scan_dag_files(&root, &mut index);
        }
    }
    index
}

fn scan_dag_files(dir: &std::path::Path, index: &mut HashMap<String, std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dag_files(&path, index);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            if let Some(module_path) = extract_module_declaration(&path) {
                index.insert(module_path, path);
            }
        }
    }
}

/// Read just the `module` declaration from a .dag file. Scans for the
/// first line matching `module <path>` — O(1) lines for well-formed files.
fn extract_module_declaration(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return Some(trimmed["module ".len()..].trim().to_string());
        }
        // Skip comments and blank lines
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break; // module declaration must come before any other content
        }
    }
    None
}

/// Extract import module paths from source text. Scans for `import <path> {`.
fn extract_imports(source: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            // import std.types { ... } → extract "std.types"
            let rest = trimmed["import ".len()..].trim();
            if let Some(space_pos) = rest.find(|c: char| c == ' ' || c == '{') {
                imports.push(rest[..space_pos].trim().to_string());
            }
        }
    }
    imports
}

/// Resolve imports transitively from an entry source. Returns the minimal
/// set of SourceFiles needed — each module loaded exactly once.
fn resolve_imports_transitively(
    entry_path: &str,
    entry_content: &str,
    index: &HashMap<String, std::path::PathBuf>,
) -> Vec<Rc<SourceFile>> {
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue: Vec<(String, String)> = Vec::new(); // (path, content)

    // Seed with the entry
    queue.push((entry_path.to_string(), entry_content.to_string()));

    while let Some((path, content)) = queue.pop() {
        let imports = extract_imports(&content);
        for module_path in imports {
            if seen.contains_key(&module_path) {
                continue; // already loaded — O(1) check
            }
            if let Some(file_path) = index.get(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    let rel_path = file_path
                        .strip_prefix(workspace_root())
                        .unwrap_or(file_path)
                        .to_string_lossy()
                        .to_string();
                    let source = Rc::new(SourceFile {
                        path: rel_path.clone(),
                        content: file_content.clone(),
                    });
                    seen.insert(module_path.clone(), source);
                    queue.push((rel_path, file_content));
                }
            }
            // If not found in index, the compiler's resolve stage will
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

// Lazily built module index — shared across all tests in a run.
use std::sync::OnceLock;
static MODULE_INDEX: OnceLock<HashMap<String, std::path::PathBuf>> = OnceLock::new();

fn module_index() -> &'static HashMap<String, std::path::PathBuf> {
    MODULE_INDEX.get_or_init(build_module_index)
}

// ── Full pipeline ────────────────────────────────────────────────────────

pub fn compile_dag(source: &str) -> Rc<PipelineResult> {
    compile_dag_named("test.dag", source, RenderTarget::Rust)
}

pub fn compile_dag_target(source: &str, target: RenderTarget) -> Rc<PipelineResult> {
    compile_dag_named("test.dag", source, target)
}

pub fn compile_dag_named(
    filename: &str,
    source: &str,
    target: RenderTarget,
) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively(filename, source, module_index());
    v2_compiler::v2_compiler_compile::compile_sources(sources, target)
}

pub fn compile_multi(files: &[(&str, &str)]) -> Rc<PipelineResult> {
    compile_multi_target(files, RenderTarget::Rust)
}

pub fn compile_multi_target(files: &[(&str, &str)], target: RenderTarget) -> Rc<PipelineResult> {
    // For multi-file compilations, resolve imports from all entry files.
    let index = module_index();
    let mut all_sources: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for (path, content) in files {
        let resolved = resolve_imports_transitively(path, content, index);
        for src in resolved {
            all_sources.entry(src.path.clone()).or_insert(src);
        }
    }
    let sources: Vec<Rc<SourceFile>> = all_sources.into_values().collect();
    v2_compiler::v2_compiler_compile::compile_sources(sources, target)
}

// ── Result inspection ────────────────────────────────────────────────────

pub fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result.diagnostics.iter().map(|d| d.name.clone()).collect()
}

pub fn assert_no_diagnostics(result: &PipelineResult) {
    let msgs = diagnostic_messages(result);
    assert!(
        msgs.is_empty(),
        "expected 0 diagnostics, got {}: {:?}",
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
