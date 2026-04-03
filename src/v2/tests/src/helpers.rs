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

pub fn tokenize(source: &str) -> Rc<Vec<Rc<Token>>> {
    v2_compiler::v2_compiler_tokenize::tokenize(source.to_string(), "test.dag".to_string())
}

pub fn parse_source(source: &str) -> Rc<ParseResult> {
    let tokens = tokenize(source);
    v2_compiler::v2_compiler_parse::parse(tokens)
}

pub fn assert_parses(source: &str, label: &str) {
    let result = parse_source(source);
    if let Some(ref err) = result.error {
        panic!("{} had parse error: {}", label, v2_compiler::v2_std_core::diagnostic_to_message(err.diagnostic.clone()));
    }
    assert!(result.module.is_some(), "{} produced no module", label);
}

pub fn assert_parses_strict(relative_path: &str) {
    let source = read_v2_file(relative_path);
    let result = parse_source(&source);
    if let Some(ref err) = result.error {
        let span = {
            let s = v2_compiler::v2_std_core::diagnostic_to_span(err.diagnostic.clone());
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
            v2_compiler::v2_std_core::diagnostic_to_message(err.diagnostic.clone()),
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
// Module resolution is convention-based: `std.types` → `dsl/std/types.dag`.
// No upfront scan of all files. Files are found and parsed only when
// actually imported.

use std::collections::HashMap;

/// Source roots where .dag files can be found. Module path segments map
/// to directory structure: `std.types` → `<root>/std/types.dag`.
fn source_roots() -> Vec<std::path::PathBuf> {
    let ws = workspace_root();
    vec![
        ws.join("dsl"),       // std.types → dsl/std/types.dag
        ws.join("src/v2"),    // v2.std.core → src/v2/00_core.dag (needs glob fallback)
    ]
}

/// Resolve a module path to a file path using convention, then glob fallback.
///
/// Convention: `std.types` → split on `.` → try `<root>/std/types.dag` in each
/// source root. This handles dsl/ where directory structure mirrors module path.
///
/// Flat fallback: for roots with flat layouts (src/v2/), glob for
/// `*<last_segment>.dag` directly in the root. Handles numeric prefixes
/// like `02_parse.dag` for `module v2.compiler.parse`.
fn resolve_module_to_path(module_path: &str) -> Option<std::path::PathBuf> {
    let segments: Vec<&str> = module_path.split('.').collect();
    if segments.is_empty() {
        return None;
    }
    for root in source_roots() {
        // Convention: join all segments as subdirectories, add .dag
        let mut conventional = root.clone();
        for seg in &segments {
            conventional.push(seg);
        }
        conventional.set_extension("dag");
        if conventional.exists() {
            return Some(conventional);
        }

        // Fallback: scan the root directory for a file whose `module`
        // declaration matches. Handles flat layouts with numeric prefixes
        // (src/v2/02_parse.dag for module v2.compiler.parse). Only reads
        // the first non-comment line from each .dag file — no full parse.
        if let Some(found) = find_module_in_dir(&root, module_path) {
            return Some(found);
        }
    }
    None
}

/// Find a file in `dir` whose `module` declaration matches `module_path`.
/// Only reads the first `module` line from candidate files — no full parse.
fn find_module_in_dir(dir: &std::path::Path, module_path: &str) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let expected = format!("module {}", module_path);
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "dag").unwrap_or(false) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Scan for `module X` line — skip comments and blanks.
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("//") {
                        continue;
                    }
                    if trimmed == expected || trimmed.starts_with(&format!("{} ", expected)) {
                        return Some(path);
                    }
                    break; // first non-comment line wasn't a module decl
                }
            }
        }
    }
    None
}

/// Extract import module paths using the actual parser — no parallel
/// string-scanning implementation. The parser is the single authority
/// for import syntax.
fn extract_imports(source: &str) -> Vec<String> {
    let result = parse_source(source);
    match &result.module {
        Some(module) => {
            v2_compiler::v2_std_core::module_imports(module.clone())
                .iter()
                .map(|imp| imp.name.clone())
                .collect()
        }
        None => vec![],
    }
}

/// Resolve imports transitively from an entry source. Returns the minimal
/// set of SourceFiles needed — each module loaded exactly once.
///
/// Uses convention-based file lookup (no upfront global scan). Only parses
/// files that are actually imported.
fn resolve_imports_transitively(
    entry_path: &str,
    entry_content: &str,
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
            if let Some(file_path) = resolve_module_to_path(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(&file_path) {
                    let rel_path = file_path
                        .strip_prefix(&ws)
                        .unwrap_or(&file_path)
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
    let sources = resolve_imports_transitively(filename, source);
    v2_compiler::v2_compiler_compile::compile_sources(Rc::new(sources), target)
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
    v2_compiler::v2_compiler_compile::compile_sources(Rc::new(sources), target)
}

// ── Result inspection ────────────────────────────────────────────────────

pub fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result.diagnostics.iter().map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone())).collect()
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
