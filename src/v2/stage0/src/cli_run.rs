// cli_run.rs — Hand-maintained Run subcommand handler.
// Not generated — survives stage0 regeneration.
// The generated main.rs calls handle_run() for the Run subcommand.

use std::collections::HashMap;
use std::rc::Rc;

use crate::v2_compiler_compile;
use crate::v2_std_core::{
    diagnostic_to_message, diagnostic_to_span,
    byte_to_line_col, is_interpreter_blocking_diagnostic, NewlineIndex,
};
use crate::v2_interpreter;

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

/// Build module index: module_path → file_path.
fn build_module_index(source_roots: &[String]) -> HashMap<String, std::path::PathBuf> {
    let mut index = HashMap::new();
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
                index.insert(module_path, path);
            }
        }
    }
    index
}

/// Resolve imports transitively. Returns sorted sources.
fn resolve_transitively(
    entry_sources: Vec<(String, String)>,
    index: &HashMap<String, std::path::PathBuf>,
    mut seen: HashMap<String, Rc<v2_compiler_compile::SourceFile>>,
) -> Vec<Rc<v2_compiler_compile::SourceFile>> {
    let mut queue = entry_sources;
    while let Some((_path, content)) = queue.pop() {
        for module_path in extract_import_paths(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = index.get(&module_path) {
                let file_content = std::fs::read_to_string(file_path)
                    .unwrap_or_else(|e| panic!("failed to read imported module '{}': {}", module_path, e));
                let rel_path = file_path.to_string_lossy().to_string();
                let source = Rc::new(v2_compiler_compile::SourceFile {
                    path: rel_path.clone(),
                    content: file_content.clone(),
                });
                seen.insert(module_path, source);
                queue.push((rel_path, file_content));
            }
        }
    }
    let mut result: Vec<_> = seen.into_values().collect();
    result.sort_by(|a, b| a.path.cmp(&b.path));
    result
}

/// Load and resolve sources from source roots.
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
        if let Some(mod_path) = extract_module_path(content) {
            seen.insert(mod_path, Rc::new(v2_compiler_compile::SourceFile {
                path: path.clone(),
                content: content.clone(),
            }));
        }
        entry_for_queue.push((path.clone(), content.clone()));
    }

    let mut sources = resolve_transitively(entry_for_queue, &index, seen);
    for (path, content) in entry_files {
        if !sources.iter().any(|s| s.path == path) {
            sources.push(Rc::new(v2_compiler_compile::SourceFile { path, content }));
        }
    }
    sources
}

/// Entry point for `dag run`. Called from the generated main.rs.
pub fn handle_run(source_roots: Vec<String>, function: String) {
    handle_run_with_options(source_roots, function, false);
}

/// Entry point with options for dry-run mode.
pub fn handle_run_with_options(source_roots: Vec<String>, function: String, dry_run: bool) {
    if source_roots.is_empty() {
        eprintln!("error: provide at least one --source-root");
        std::process::exit(1);
    }

    let sources = load_sources(&source_roots);
    eprintln!("resolved {} sources", sources.len());

    // Compile through validation (no emission)
    let result = v2_compiler_compile::compile_to_resolved(Rc::new(sources));

    // Check for errors
    let has_errors = result.diagnostics.iter().any(|d| {
        is_interpreter_blocking_diagnostic(d.diagnostic.clone())
    });
    if has_errors {
        let si: HashMap<String, Rc<NewlineIndex>> = result.newline_indices.iter()
            .map(|idx| (idx.file.clone(), idx.clone()))
            .collect();
        for d in result.diagnostics.iter() {
            if !is_interpreter_blocking_diagnostic(d.diagnostic.clone()) {
                continue;
            }
            let span = diagnostic_to_span(d.diagnostic.clone());
            let loc = match si.get(&span.file) {
                Some(idx) => {
                    let lc = byte_to_line_col(idx, span.start);
                    format!("{}:{}:{}", span.file, lc.line, lc.col)
                }
                None => span.file.clone(),
            };
            eprintln!("{}: error: {}", loc, diagnostic_to_message(d.diagnostic.clone()));
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
    match v2_interpreter::run_with_options(graph, result.source_indices.clone(), &function, dry_run) {
        Ok(val) => {
            println!("{}", val);
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
                         Wrap your rich result type in ExitSuccess / ExitFailure.",
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
    NotProcessExit { type_name: String },
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
        v2_interpreter::Value::Variant { type_name, variant_name, fields } => {
            if type_name != "ProcessExit" {
                return ExitClass::NotProcessExit {
                    type_name: type_name.clone(),
                };
            }
            match variant_name.as_str() {
                "ExitSuccess" => ExitClass::Success,
                "ExitFailure" => {
                    match fields.get("code") {
                        Some(v2_interpreter::Value::Int(n)) => ExitClass::Failure(*n as i32),
                        _ => ExitClass::Failure(1),
                    }
                }
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
