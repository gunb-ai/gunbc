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
    let sources = vec![Rc::new(SourceFile {
        path: filename.to_string(),
        content: source.to_string(),
    })];
    v2_compiler::v2_compiler_compile::compile_sources(sources, target)
}

pub fn compile_multi(files: &[(&str, &str)]) -> Rc<PipelineResult> {
    compile_multi_target(files, RenderTarget::Rust)
}

pub fn compile_multi_target(files: &[(&str, &str)], target: RenderTarget) -> Rc<PipelineResult> {
    let sources: Vec<Rc<SourceFile>> = files
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        })
        .collect();
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
