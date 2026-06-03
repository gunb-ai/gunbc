// cli_test.rs — Hand-maintained Test subcommand handler (RR-A / W3 Rung 1).
// Not generated — survives stage0 regeneration.
//
// Runs the bootstrap manual-corpus harness via the v2 interpreter on a transitive
// import closure seeded from one entry module (not a full source-root scan).

use std::collections::HashMap;
use std::rc::Rc;

use crate::cli_run::{
    build_module_index, extract_import_paths, extract_module_path, resolve_transitively,
};
use crate::v2_compiler_compile;
use crate::v2_interpreter;
use crate::v2_std_core::{
    byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    is_interpreter_blocking_diagnostic, NewlineIndex,
};

/// Modeled `BootstrapEvaluatorCorpusHarnessEntry` for the manual T-38 wedge
/// (`bootstrap.dag` / `cli.dag`). Host transport must stay aligned with these pins.
pub const HARNESS_SOURCE_ROOTS: &[&str] = &["src/v4", "dsl/std"];
pub const HARNESS_ENTRY_MODULE: &str = "v4.test.claim.workflow.manual_corpus_eval";
/// `std.process.ProcessExit` host entry — evaluates `run_manual_testclaim_corpus_eval` at runtime.
pub const HARNESS_ENTRY_FUNCTION: &str = "run_manual_testclaim_corpus_eval_host_exit";

fn load_sources_for_entry_module(
    source_roots: &[String],
    entry_module: &str,
) -> Vec<Rc<v2_compiler_compile::SourceFile>> {
    let index = build_module_index(source_roots);
    let file_path = index.get(entry_module).unwrap_or_else(|| {
        panic!(
            "harness entry module '{}' not found under source roots {:?}",
            entry_module, source_roots
        )
    });
    let content = std::fs::read_to_string(file_path).unwrap_or_else(|e| {
        panic!(
            "failed to read harness entry module '{}' at {:?}: {}",
            entry_module, file_path, e
        )
    });
    let rel_path = file_path.to_string_lossy().to_string();
    let mut seen: HashMap<String, Rc<v2_compiler_compile::SourceFile>> = HashMap::new();
    seen.insert(
        entry_module.to_string(),
        Rc::new(v2_compiler_compile::SourceFile {
            path: rel_path.clone(),
            content: content.clone(),
        }),
    );
    resolve_transitively(
        vec![(rel_path, content)],
        &index,
        seen,
    )
}

/// Entry point for `gunbc test` (manual corpus harness). Called from generated main.rs.
pub fn handle_test() {
    let source_roots: Vec<String> = HARNESS_SOURCE_ROOTS
        .iter()
        .map(|s| s.to_string())
        .collect();
    handle_test_with_entry(
        source_roots,
        HARNESS_ENTRY_MODULE.to_string(),
        HARNESS_ENTRY_FUNCTION.to_string(),
        false,
    );
}

pub fn handle_test_with_entry(
    source_roots: Vec<String>,
    entry_module: String,
    function: String,
    dry_run: bool,
) {
    if source_roots.is_empty() {
        eprintln!("error: harness requires at least one --source-root");
        std::process::exit(1);
    }

    let sources = load_sources_for_entry_module(&source_roots, &entry_module);
    eprintln!(
        "gunbc test: resolved {} sources for entry module {}",
        sources.len(),
        entry_module
    );

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

    eprintln!("gunbc test: running {}()...", function);
    match v2_interpreter::run_with_options(graph, result.source_indices.clone(), &function, dry_run)
    {
        Ok(val) => match classify_process_exit(&val) {
            ExitClass::Success => {}
            ExitClass::Failure(code) => std::process::exit(code),
            ExitClass::NotProcessExit { type_name } => {
                eprintln!(
                    "error: function `{}` returned `{}`, not `ProcessExit`. \
                     gunbc test harness entries must return std/process.dag ProcessExit.",
                    function, type_name
                );
                std::process::exit(2);
            }
        },
        Err(e) => {
            eprintln!("runtime error: {}", e);
            std::process::exit(1);
        }
    }
}

enum ExitClass {
    Success,
    Failure(i32),
    NotProcessExit { type_name: String },
}

fn classify_process_exit(val: &v2_interpreter::Value) -> ExitClass {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_entry_module_resolves_transitive_closure_without_full_v4_scan() {
        let roots: Vec<String> = HARNESS_SOURCE_ROOTS
            .iter()
            .map(|s| s.to_string())
            .collect();
        let sources = load_sources_for_entry_module(&roots, HARNESS_ENTRY_MODULE);
        assert!(
            sources.len() < 120,
            "harness closure should be bounded (got {} sources); full src/v4 scan is forbidden",
            sources.len()
        );
        assert!(
            sources.iter().any(|s| s.path.contains("manual_corpus_eval.dag")),
            "closure must include manual_corpus_eval entry module"
        );
    }
}
