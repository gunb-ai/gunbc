use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{
    compile_sources_with_options, compile_to_resolved, CompilePipelineOptions, PipelineResult,
    ResolvedPipelineResult, SourceFile,
};
use v1_compiler::v1_compiler_parse::ParseResult;
use v1_compiler::v1_std_core::Token;

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

pub fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v1"), ws.join("dag")]
}

pub fn v2_layer_roots() -> Vec<std::path::PathBuf> {
    let ws = workspace_root();
    vec![ws.join("src/v2"), ws.join("dag")]
}

pub fn tokenize(source: &str) -> Rc<im::Vector<Rc<Token>>> {
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
    let mut source_indices = im::HashMap::new();
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

use im::HashMap;
use std::sync::OnceLock;

fn build_module_index_for_roots(
    roots: &[std::path::PathBuf],
) -> std::collections::HashMap<String, std::path::PathBuf> {
    let mut index = std::collections::HashMap::new();
    for root in roots {
        if root.exists() {
            scan_dag_files(root, &mut index);
        }
    }
    index
}

fn build_module_index() -> std::collections::HashMap<String, std::path::PathBuf> {
    build_module_index_for_roots(&source_roots())
}

fn scan_dag_files(
    dir: &std::path::Path,
    index: &mut std::collections::HashMap<String, std::path::PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
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

fn extract_module_declaration(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        return trimmed
            .strip_prefix("module ")
            .and_then(|rest| rest.split_whitespace().next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
    }
    None
}

static MODULE_INDEX: OnceLock<std::collections::HashMap<String, std::path::PathBuf>> =
    OnceLock::new();

fn module_index() -> &'static std::collections::HashMap<String, std::path::PathBuf> {
    MODULE_INDEX.get_or_init(build_module_index)
}

pub fn resolve_imports_transitively(entry_path: &str, entry_content: &str) -> Vec<Rc<SourceFile>> {
    resolve_references_transitively_for_roots(entry_path, entry_content, &source_roots())
}

pub fn resolve_imports_transitively_with_source_roots(
    entry_path: &str,
    entry_content: &str,
    source_roots: &[std::path::PathBuf],
) -> Vec<Rc<SourceFile>> {
    resolve_references_transitively_for_roots(entry_path, entry_content, source_roots)
}

fn display_source_path(path: &std::path::Path, ws: &std::path::Path) -> String {
    path.strip_prefix(ws)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

/// Declaration index over a root set, memoized per root set per thread.
///
/// Building it parses every candidate file, so it is paid once per thread and
/// shared by every test that thread runs. It is thread-local rather than a
/// process-wide static because the index holds `Rc<SourceFile>`, which is
/// neither `Send` nor `Sync`.
fn declaration_index_for(
    roots: &[std::path::PathBuf],
) -> Rc<v1_compiler::source_closure::DeclarationIndex> {
    thread_local! {
        static CACHE: std::cell::RefCell<
            HashMap<String, Rc<v1_compiler::source_closure::DeclarationIndex>>,
        > = std::cell::RefCell::new(HashMap::new());
    }
    let key = roots
        .iter()
        .map(|r| r.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("|");
    if let Some(hit) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return hit;
    }
    let ws = workspace_root();
    let (index, unparsed) = v1_compiler::source_closure::build_declaration_index(roots, &ws);
    assert!(
        unparsed.is_empty(),
        "declaration index refuses: {} source file(s) under {:?} did not parse, so the pool \
         would have a hole in it: {:?}",
        unparsed.len(),
        roots,
        unparsed.iter().take(5).collect::<Vec<_>>()
    );
    let index = Rc::new(index);
    CACHE.with(|c| c.borrow_mut().insert(key, index.clone()));
    index
}

fn resolve_references_transitively_for_roots(
    entry_path: &str,
    entry_content: &str,
    roots: &[std::path::PathBuf],
) -> Vec<Rc<SourceFile>> {
    let index = declaration_index_for(roots);
    v1_compiler::source_closure::closure_for_entry(entry_path, entry_content, &index)
}

fn analyze_complexity_options() -> Rc<CompilePipelineOptions> {
    Rc::new(CompilePipelineOptions {
        analyze_complexity: true,
        ..(*v1_compiler::v1_compiler_compile::default_compile_pipeline_options()).clone()
    })
}

pub fn compile_dag_analyze_complexity(source: &str) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively("test.dag", source);
    compile_sources_with_options(
        Rc::new(sources.into()),
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
    let sources: Vec<Rc<SourceFile>> = all_sources.into_iter().map(|(_, v)| v).collect();
    compile_sources_with_options(
        Rc::new(sources.into()),
        RenderTarget::Rust,
        analyze_complexity_options(),
    )
}

pub fn compile_dag(source: &str) -> Rc<PipelineResult> {
    compile_dag_named("test.dag", source, RenderTarget::Rust)
}

pub fn compile_dag_resolved(source: &str) -> Rc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", source);
    compile_to_resolved(Rc::new(sources.into()))
}

pub fn compile_dag_target(source: &str, target: RenderTarget) -> Rc<PipelineResult> {
    compile_dag_named("test.dag", source, target)
}

pub fn compile_dag_named(filename: &str, source: &str, target: RenderTarget) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively(filename, source);
    v1_compiler::v1_compiler_compile::compile_sources(Rc::new(sources.into()), target)
}

pub fn compile_dag_named_with_source_roots(
    filename: &str,
    source: &str,
    target: RenderTarget,
    source_roots: &[std::path::PathBuf],
) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively_with_source_roots(filename, source, source_roots);
    v1_compiler::v1_compiler_compile::compile_sources(Rc::new(sources.into()), target)
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
    let sources: Vec<Rc<SourceFile>> = all_sources.into_iter().map(|(_, v)| v).collect();
    v1_compiler::v1_compiler_compile::compile_sources(Rc::new(sources.into()), target)
}

pub fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

pub fn assert_no_diagnostics(result: &PipelineResult) {
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

/// Transient cargo/sccache failures seen under parallel nextest on self-hosted runners.
pub fn cargo_infra_failure_transient(stderr: &str) -> bool {
    stderr.contains("couldn't create a temp dir")
        || stderr.contains("Resource temporarily unavailable")
        || stderr.contains("failed to spawn")
        || stderr.contains("sccache: encountered fatal error")
}

/// Run a cargo subprocess with the same cold-retry ladder as `.github/workflows/ci.yml`.
pub fn run_cargo_with_infra_retry<F>(build: F) -> std::process::Output
where
    F: Fn() -> std::process::Command,
{
    let first = build().output().expect("failed to spawn cargo");
    if first.status.success()
        || !cargo_infra_failure_transient(&String::from_utf8_lossy(&first.stderr))
    {
        return first;
    }

    let mut retry = build();
    retry.env("CARGO_BUILD_JOBS", "1");
    let second = retry.output().expect("failed to spawn cargo retry");
    if second.status.success()
        || !cargo_infra_failure_transient(&String::from_utf8_lossy(&second.stderr))
    {
        return second;
    }

    let mut cold = build();
    cold.env_remove("RUSTC_WRAPPER");
    cold.env("CARGO_BUILD_JOBS", "1");
    cold.output().expect("failed to spawn cargo cold retry")
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
    fn scan_dag_files_last_wins_on_duplicate_module_names_without_panic() {
        let dir = temp_dir("duplicate-modules");
        let a = dir.join("a.dag");
        let b = dir.join("nested").join("b.dag");
        std::fs::create_dir_all(b.parent().expect("nested dir")).expect("create nested dir");
        std::fs::write(&a, "module duplicate.test\n").expect("write first module");
        std::fs::write(&b, "module duplicate.test\n").expect("write second module");

        let mut index = std::collections::HashMap::new();
        scan_dag_files(&dir, &mut index);

        assert_eq!(
            index.len(),
            1,
            "duplicate module paths within a root collapse via last-wins insert"
        );
        assert!(
            index.contains_key("duplicate.test"),
            "one duplicate.test binding remains"
        );
    }

    #[test]
    fn build_module_index_co_root_last_wins_on_duplicate_module_names() {
        let dir_a = temp_dir("overlay-root-a");
        let dir_b = temp_dir("overlay-root-b");
        std::fs::write(dir_a.join("a.dag"), "module duplicate.test\n").expect("write first");
        std::fs::write(dir_b.join("b.dag"), "module duplicate.test\n").expect("write second");

        let index_ab = build_module_index_for_roots(&[dir_a.clone(), dir_b.clone()]);
        assert_eq!(
            index_ab.get("duplicate.test"),
            Some(&dir_b.join("b.dag")),
            "later root wins on duplicate module paths"
        );

        let index_ba = build_module_index_for_roots(&[dir_b, dir_a.clone()]);
        assert_eq!(
            index_ba.get("duplicate.test"),
            Some(&dir_a.join("a.dag")),
            "root order reverses the winning file"
        );
    }
}
