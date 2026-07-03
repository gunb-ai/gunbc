#![allow(clippy::disallowed_macros)]

//! Host-physics floor witness for the v1 compiler diagnostic oracle suite
//! (`src/v1/tests/src/diagnostics.rs`, 11 active `#[test]` fns; `duplicate_module_detected`
//! remains `#[ignore]` under PERF track). Exercises compile-time diagnostic shape,
//! message text, and span→line/col mapping via `compile_multi` until that harness is
//! witness-layer importable from pure `.dag` floor witnesses.

use std::collections::HashMap;
use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, PipelineResult, SourceFile};
use v1_compiler::v1_std_core::{
    build_newline_index, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    CompilerDiagnostic, ErrorNode,
};

type ModuleIndex = HashMap<String, std::path::PathBuf>;
type WitnessCase = (&'static str, fn(&ModuleIndex));

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("diagnostics_witness: {msg}");
    ExitCode::from(1)
}

fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v1"), ws.join("dag")]
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

fn scan_dag_files(dir: &std::path::Path, index: &mut ModuleIndex) {
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

fn build_module_index() -> ModuleIndex {
    let mut index = HashMap::new();
    for root in source_roots() {
        if root.exists() {
            scan_dag_files(&root, &mut index);
        }
    }
    index
}

fn extract_imports(source: &str) -> Vec<String> {
    let tokens =
        v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), "test.dag".to_string());
    let source_index =
        v1_compiler::v1_std_core::build_newline_index("test.dag".to_string(), source.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert("test.dag".to_string(), source_index);
    let result = v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    match &result.module {
        Some(module) => v1_compiler::v1_std_core::module_imports(module.clone())
            .iter()
            .map(|imp| imp.name.clone())
            .collect(),
        None => vec![],
    }
}

fn resolve_imports_transitively(
    entry_path: &str,
    entry_content: &str,
    module_index: &ModuleIndex,
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue = vec![(entry_path.to_string(), entry_content.to_string())];

    while let Some((_path, content)) = queue.pop() {
        for module_path in extract_imports(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = module_index.get(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    let rel_path = file_path
                        .strip_prefix(&ws)
                        .unwrap_or(file_path)
                        .to_string_lossy()
                        .to_string();
                    seen.insert(
                        module_path.clone(),
                        Rc::new(SourceFile {
                            path: rel_path.clone(),
                            content: file_content.clone(),
                        }),
                    );
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

fn compile_multi(_module_index: &ModuleIndex, files: &[(&str, &str)]) -> Rc<PipelineResult> {
    let mut all_sources: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for (path, content) in files {
        let resolved = resolve_imports_transitively(path, content, _module_index);
        for src in resolved {
            all_sources.entry(src.path.clone()).or_insert(src);
        }
    }
    let sources: Vec<Rc<SourceFile>> = all_sources.into_values().collect();
    compile_sources(Rc::new(sources), RenderTarget::Rust)
}

fn first_diag(module_index: &ModuleIndex, files: &[(&str, &str)]) -> Rc<ErrorNode> {
    let result = compile_multi(module_index, files);
    assert!(
        !result.diagnostics.is_empty(),
        "expected at least one diagnostic, got none"
    );
    result.diagnostics[0].clone()
}

fn diag_line_col(diag: &ErrorNode, source: &str, file: &str) -> (i64, i64) {
    let span = diagnostic_to_span(diag.diagnostic.clone());
    let idx = build_newline_index(file.to_string(), source.to_string());
    let lc = byte_to_line_col(idx, span.start);
    (lc.line, lc.col)
}

fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn missing_export_points_at_name(module_index: &ModuleIndex) {
    let source = "module provider\ntype User { name: String }\n";
    let bad = "module consumer\nimport provider { NonExistent }\n";
    let result = compile_multi(
        module_index,
        &[("provider.dag", source), ("consumer.dag", bad)],
    );

    assert_eq!(result.diagnostics.len(), 1);
    let d = &result.diagnostics[0];

    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::MissingExport { .. }),
        "expected MissingExport, got: {:?}",
        d.diagnostic
    );

    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(
        msg.contains("NonExistent"),
        "message should name the missing export: {msg}"
    );
    assert!(
        msg.contains("provider"),
        "message should name the target module: {msg}"
    );
    assert!(
        msg.contains("consumer"),
        "message should name the importing module: {msg}"
    );

    let (line, col) = diag_line_col(d, bad, "consumer.dag");
    assert_eq!(line, 2, "should be on line 2 (the import line)");
    assert_eq!(
        col, 19,
        "should point at 'NonExistent' (col 19), not 'import' (col 1)"
    );
}

fn variant_not_reexported_through_type_only_import(module_index: &ModuleIndex) {
    let files = &[
        ("def.dag", "module self_gen8_def\ntype E = A | B\n"),
        (
            "proxy.dag",
            "module self_gen8_proxy\nimport self_gen8_def { E }\n",
        ),
        (
            "use_mod.dag",
            "module self_gen8_use\nimport self_gen8_proxy { B }\n",
        ),
    ];
    let result = compile_multi(module_index, files);

    assert_eq!(result.diagnostics.len(), 1);
    let d = &result.diagnostics[0];
    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::MissingExport { .. }),
        "expected MissingExport for variant not in proxy export surface, got: {:?}",
        d.diagnostic
    );
    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(
        msg.contains("B"),
        "message should name the missing variant export: {msg}"
    );
    assert!(
        msg.contains("self_gen8_proxy"),
        "message should name the proxy module: {msg}"
    );
}

fn multiple_missing_exports_each_have_own_span(module_index: &ModuleIndex) {
    let source = "module provider\ntype User { name: String }\n";
    let bad = "module consumer\nimport provider { Foo, Bar }\n";
    let result = compile_multi(
        module_index,
        &[("provider.dag", source), ("consumer.dag", bad)],
    );

    assert_eq!(
        result.diagnostics.len(),
        2,
        "expected 2 diagnostics for 2 missing names"
    );

    let msg0 = diagnostic_to_message(result.diagnostics[0].diagnostic.clone());
    let msg1 = diagnostic_to_message(result.diagnostics[1].diagnostic.clone());
    assert!(
        msg0.contains("Foo"),
        "first diagnostic should mention Foo: {msg0}"
    );
    assert!(
        msg1.contains("Bar"),
        "second diagnostic should mention Bar: {msg1}"
    );

    let (_, col0) = diag_line_col(&result.diagnostics[0], bad, "consumer.dag");
    let (_, col1) = diag_line_col(&result.diagnostics[1], bad, "consumer.dag");
    assert_ne!(
        col0, col1,
        "Foo and Bar should have different column positions"
    );
}

fn unresolved_import_names_module(module_index: &ModuleIndex) {
    let bad = "module consumer\nimport nonexistent { Thing }\n";
    let d = first_diag(module_index, &[("consumer.dag", bad)]);

    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedImport { .. }),
        "expected UnresolvedImport, got: {:?}",
        d.diagnostic
    );

    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(
        msg.contains("nonexistent"),
        "should name the missing module: {msg}"
    );
    assert!(
        msg.contains("consumer"),
        "should name the importing module: {msg}"
    );
}

fn unresolved_type_in_field(module_index: &ModuleIndex) {
    let source = "module types\ntype Wrapper { inner: Bogus }\n";
    let result = compile_multi(module_index, &[("types.dag", source)]);

    let type_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedType { .. }))
        .collect();

    assert!(
        !type_diags.is_empty(),
        "expected UnresolvedType diagnostic, got: {:?}",
        diagnostic_messages(&result)
    );

    let msg = diagnostic_to_message(type_diags[0].diagnostic.clone());
    assert!(
        msg.contains("Bogus"),
        "should name the unresolved type: {msg}"
    );
}

fn bare_container_type_detected(module_index: &ModuleIndex) {
    let source = "module bare\nimport std.types { List }\ntype Foo { items: List }\n";
    let result = compile_multi(module_index, &[("bare.dag", source)]);

    let arity_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }))
        .collect();

    assert!(
        !arity_diags.is_empty(),
        "expected ArityMismatch diagnostic for bare List, got: {:?}",
        diagnostic_messages(&result)
    );

    let msg = diagnostic_to_message(arity_diags[0].diagnostic.clone());
    assert!(
        msg.contains("List"),
        "should name the bare container type: {msg}"
    );
}

fn parameterized_container_no_false_positive(module_index: &ModuleIndex) {
    let source = "module param\nimport std.types { List }\ntype Foo { items: List<Int> }\n";
    let result = compile_multi(module_index, &[("param.dag", source)]);

    let arity_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }))
        .collect();

    assert!(
        arity_diags.is_empty(),
        "parameterized List<Int> should not trigger ArityMismatch, got: {:?}",
        diagnostic_messages(&result)
    );
}

fn unknown_type_name_no_arity_false_positive(module_index: &ModuleIndex) {
    let source = "module custom\ntype Widget { label: String }\ntype Bag { item: Widget }\n";
    let result = compile_multi(module_index, &[("custom.dag", source)]);

    let arity_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }))
        .collect();

    assert!(
        arity_diags.is_empty(),
        "user-defined type should not trigger ArityMismatch, got: {:?}",
        diagnostic_messages(&result)
    );
}

fn empty_list_wrong_expected_type(module_index: &ModuleIndex) {
    let source = "module elist\nfn make_stuff() -> String {\n  []\n}\n";
    let result = compile_multi(module_index, &[("elist.dag", source)]);

    let internal_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.module_name == "elist"
                && match &*d.diagnostic {
                    CompilerDiagnostic::InternalError { message, .. } => {
                        message.contains("empty list literal")
                    }
                    _ => false,
                }
        })
        .collect();

    assert!(
        !internal_diags.is_empty(),
        "expected diagnostic for empty list with non-collection expected type, got: {:?}",
        diagnostic_messages(&result)
    );
}

fn empty_list_with_type_context_no_false_positive(module_index: &ModuleIndex) {
    let source =
        "module elist_ok\nimport std.types { List }\nfn make_list() -> List<String> {\n  []\n}\n";
    let result = compile_multi(module_index, &[("elist_ok.dag", source)]);

    let empty_list_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.module_name == "elist_ok"
                && match &*d.diagnostic {
                    CompilerDiagnostic::InternalError { message, .. } => {
                        message.contains("empty list literal")
                    }
                    _ => false,
                }
        })
        .collect();

    assert!(
        empty_list_diags.is_empty(),
        "empty list with type context should not trigger diagnostic, got: {:?}",
        diagnostic_messages(&result)
    );
}

fn clean_compile_produces_zero_diagnostics(module_index: &ModuleIndex) {
    let source = "module clean\ntype Widget { label: String, count: Int }\n";
    let result = compile_multi(module_index, &[("clean.dag", source)]);
    assert!(
        result.diagnostics.is_empty(),
        "clean source should produce 0 diagnostics, got: {:?}",
        diagnostic_messages(&result)
    );
}

fn main() -> ExitCode {
    let module_index = build_module_index();

    let tests: Vec<WitnessCase> = vec![
        (
            "missing_export_points_at_name",
            missing_export_points_at_name,
        ),
        (
            "variant_not_reexported_through_type_only_import",
            variant_not_reexported_through_type_only_import,
        ),
        (
            "multiple_missing_exports_each_have_own_span",
            multiple_missing_exports_each_have_own_span,
        ),
        (
            "unresolved_import_names_module",
            unresolved_import_names_module,
        ),
        ("unresolved_type_in_field", unresolved_type_in_field),
        ("bare_container_type_detected", bare_container_type_detected),
        (
            "parameterized_container_no_false_positive",
            parameterized_container_no_false_positive,
        ),
        (
            "unknown_type_name_no_arity_false_positive",
            unknown_type_name_no_arity_false_positive,
        ),
        (
            "empty_list_wrong_expected_type",
            empty_list_wrong_expected_type,
        ),
        (
            "empty_list_with_type_context_no_false_positive",
            empty_list_with_type_context_no_false_positive,
        ),
        (
            "clean_compile_produces_zero_diagnostics",
            clean_compile_produces_zero_diagnostics,
        ),
    ];

    for (name, test) in tests {
        let index = module_index.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(&index)));
        if result.is_err() {
            return fail(format!("{name} panicked"));
        }
    }

    ExitCode::SUCCESS
}
