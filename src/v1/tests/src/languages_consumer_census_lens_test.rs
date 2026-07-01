use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

fn run_v4_module(entry: &str, content: &str, witness_fn: &str) -> Value {
    let sources: Vec<std::rc::Rc<SourceFile>> =
        resolve_imports_transitively_with_source_roots(entry, content, &v2_source_roots());
    let resolved = compile_to_resolved(std::rc::Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    v1_interpreter::run(graph, resolved.source_indices.clone(), witness_fn)
        .unwrap_or_else(|e| panic!("run {witness_fn}: {e:?}"))
}

fn assert_witness_true(entry: &str, witness_fn: &str) {
    let content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read {entry}: {e}"));
    match run_v4_module(entry, &content, witness_fn) {
        Value::Bool(true) => {}
        other => panic!("expected {witness_fn} true, got {other:?}"),
    }
}

#[test]
fn languages_consumer_census_lens_parses_and_runs_witnesses() {
    let lens_entry = "src/v2/lens/languages_consumer_census.dag";
    let lens_content = std::fs::read_to_string(workspace_root().join(lens_entry))
        .unwrap_or_else(|e| panic!("read {lens_entry}: {e}"));
    assert_resolved_no_hard_errors(&compile_to_resolved(std::rc::Rc::new(
        resolve_imports_transitively_with_source_roots(
            lens_entry,
            &lens_content,
            &v2_source_roots(),
        ),
    )));

    assert_witness_true(
        "src/v2/test/claim/languages_consumer_census/corpus/data_decl_ratchet_test.dag",
        "corpus_data_decl_ratchet_holds",
    );
    assert_witness_true(
        "src/v2/test/claim/languages_consumer_census/corpus/per_language_row_ratchet_test.dag",
        "corpus_per_language_row_ratchet_holds",
    );
    assert_witness_true(
        "src/v2/test/claim/languages_consumer_census/corpus/rust_statements_composition_only_test.dag",
        "corpus_rust_statements_is_composition_only",
    );
    assert_witness_true(
        "src/v2/test/claim/languages_consumer_census/corpus/rust_spec_external_consumer_test.dag",
        "corpus_rust_spec_has_external_consumer",
    );
    assert_witness_true(
        "src/v2/test/claim/languages_consumer_census/corpus/rust_language_external_consumer_test.dag",
        "corpus_rust_language_has_external_consumer",
    );
}
