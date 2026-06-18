//! Parse + resolve receipts for `v2.lens.extdeps_shape_transport_policy`.

use v1_compiler::extdeps_shape_transport_policy_project;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v2")]
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
fn extdeps_argv_projection_cargo_fmt_doc_defused_on_live_tree() {
    let root = workspace_root();
    let path = root
        .join("dsl/extdeps/rust/cargo_build.dag")
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        extdeps_shape_transport_policy_project::dead_param_count_for_operation(
            path.clone(),
            "cargo.Build".to_string(),
            "Fmt".to_string(),
        ),
        0
    );
    assert_eq!(
        extdeps_shape_transport_policy_project::dead_param_count_for_operation(
            path,
            "cargo.Build".to_string(),
            "Doc".to_string(),
        ),
        0
    );
}

#[test]
fn extdeps_argv_projection_cargo_clippy_defused_on_live_tree() {
    assert_eq!(
        extdeps_shape_transport_policy_project::dead_param_count_for_module_path(
            "extdeps.cargo_build".to_string(),
            "cargo.Build".to_string(),
            "Clippy".to_string(),
        ),
        0
    );
}

#[test]
fn extdeps_shape_transport_policy_lens_parses_and_runs_witnesses() {
    let lens_entry = "src/v2/lens/extdeps_shape_transport_policy.dag";
    let lens_content = std::fs::read_to_string(workspace_root().join(lens_entry))
        .unwrap_or_else(|e| panic!("read {lens_entry}: {e}"));
    assert_resolved_no_hard_errors(&compile_to_resolved(std::rc::Rc::new(
        resolve_imports_transitively_with_source_roots(
            lens_entry,
            &lens_content,
            &v2_source_roots(),
        ),
    )));

    for (entry, witness_fn) in [
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/policy_leak_cargo_build_test.dag",
            "policy_leak_cargo_build_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/policy_leak_cargo_fmt_test.dag",
            "policy_leak_cargo_fmt_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/policy_leak_cargo_doc_test.dag",
            "policy_leak_cargo_doc_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/clean_git_diff_test.dag",
            "clean_git_diff_is_green_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/dead_param_cargo_build_test.dag",
            "dead_param_cargo_build_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/dead_param_gcp_login_test.dag",
            "dead_param_gcp_login_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/dead_param_cargo_clippy_test.dag",
            "dead_param_cargo_clippy_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/transport_fusion_gcp_oauth_test.dag",
            "transport_fusion_gcp_oauth_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/module_path_rename_test.dag",
            "module_path_rename_resolves_by_qn_not_filepath_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/module_path_rename_test.dag",
            "module_path_rename_unknown_qn_does_not_resolve_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/cargo_clippy_dead_param_test.dag",
            "corpus_cargo_clippy_dead_param_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/cargo_fmt_dead_param_test.dag",
            "corpus_cargo_fmt_dead_param_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/cargo_doc_dead_param_test.dag",
            "corpus_cargo_doc_dead_param_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/gcp_login_dead_param_test.dag",
            "corpus_gcp_login_dead_param_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/cargo_build_policy_leak_test.dag",
            "corpus_cargo_build_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/git_policy_leak_test.dag",
            "corpus_git_policy_leak_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/gcp_oauth_fusion_fork_test.dag",
            "corpus_gcp_oauth_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/gist_create_policy_leak_test.dag",
            "corpus_gist_create_defused_holds",
        ),
    ] {
        assert_witness_true(entry, witness_fn);
    }

    for (entry, witness_fn) in [
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/embedded_policy_literal_local_test.dag",
            "embedded_policy_literal_local_is_red_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/corpus/runtime_local_embedded_policy_test.dag",
            "corpus_runtime_local_embedded_policy_defused_holds",
        ),
        (
            "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/clean_gist_create_test.dag",
            "clean_gist_create_is_green_holds",
        ),
    ] {
        assert_witness_true(entry, witness_fn);
    }
}

#[test]
fn extdeps_embedded_policy_projection_catches_pre_5109_class() {
    assert_eq!(
        extdeps_shape_transport_policy_project::embedded_policy_literal_count_for_module_path(
            "extdeps.runtime.local".to_string(),
        ),
        0
    );
}
