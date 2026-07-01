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
    [ws.join("src/v1"), ws.join("dsl")]
}

pub fn v2_layer_roots() -> Vec<std::path::PathBuf> {
    let ws = workspace_root();
    vec![ws.join("src/v2"), ws.join("dsl")]
}

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

use std::collections::HashMap;
use std::sync::OnceLock;

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

static MODULE_INDEX: OnceLock<HashMap<String, std::path::PathBuf>> = OnceLock::new();

fn module_index() -> &'static HashMap<String, std::path::PathBuf> {
    MODULE_INDEX.get_or_init(build_module_index)
}

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
        }
    }

    let mut sources: Vec<Rc<SourceFile>> = seen.into_values().collect();
    sources.push(Rc::new(SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    }));
    sources
}

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

        let mut index = HashMap::new();
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
