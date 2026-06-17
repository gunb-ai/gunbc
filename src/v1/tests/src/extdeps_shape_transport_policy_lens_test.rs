//! Parse + resolve receipts for `v2.lens.extdeps_shape_transport_policy`.

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

#[test]
fn extdeps_shape_transport_policy_lens_parses_and_runs_witnesses() {
    let lens_entry = "src/v2/lens/extdeps_shape_transport_policy.dag";
    let lens_content = std::fs::read_to_string(workspace_root().join(lens_entry))
        .unwrap_or_else(|e| panic!("read {lens_entry}: {e}"));
    assert_resolved_no_hard_errors(&compile_to_resolved(std::rc::Rc::new(
        resolve_imports_transitively_with_source_roots(lens_entry, &lens_content, &v2_source_roots()),
    )));

    let policy_leak_entry =
        "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/policy_leak_cargo_build.dag";
    let policy_leak_content = std::fs::read_to_string(workspace_root().join(policy_leak_entry))
        .unwrap_or_else(|e| panic!("read {policy_leak_entry}: {e}"));
    match run_v4_module(
        policy_leak_entry,
        &policy_leak_content,
        "policy_leak_cargo_build_is_red_holds",
    ) {
        Value::Bool(true) => {}
        other => panic!("expected policy leak witness true, got {other:?}"),
    }

    let clean_entry = "src/v2/compiler/extdeps_shape_transport_policy/lens_unit/clean_git_diff.dag";
    let clean_content = std::fs::read_to_string(workspace_root().join(clean_entry))
        .unwrap_or_else(|e| panic!("read {clean_entry}: {e}"));
    match run_v4_module(clean_entry, &clean_content, "clean_git_diff_is_green_holds") {
        Value::Bool(true) => {}
        other => panic!("expected clean git diff witness true, got {other:?}"),
    }
}
