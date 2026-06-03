//! Regression: module-level fn items resolve as first-class callable values (gap-a).

use std::path::PathBuf;
use std::rc::Rc;

use v2_compiler::cli_run;
use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

fn workspace_v4_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("src/v4")
}

fn mvp_int_claim_entry() -> PathBuf {
    workspace_v4_root().join("test/claim/manual/mvp_int_cross_target_coercion.dag")
}

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

#[test]
fn fn_item_bound_and_called_via_first_class_reference() {
    let src = r#"module test.fn_as_value
fn add(a: Int, b: Int) -> Int { a + b }
fn use_via_binding() -> Int {
  let f = add
  f(a: 2, b: 3)
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_via_binding") {
        Ok(Value::Int(5)) => {}
        other => panic!("expected Int(5), got {other:?}"),
    }
}

#[test]
fn scoped_entry_resolves_import_closure_not_entire_v4_tree() {
    let v4 = workspace_v4_root();
    let entry = mvp_int_claim_entry();
    assert!(entry.is_file(), "missing {}", entry.display());
    let roots = vec![v4.to_string_lossy().into_owned()];
    let scoped = cli_run::load_sources_for_entry(&roots, entry.to_str().unwrap());
    assert!(
        scoped.len() < 80,
        "expected scoped closure << full v4 tree, got {} modules",
        scoped.len()
    );
    assert!(
        scoped.len() >= 8,
        "expected non-trivial import closure, got {} modules",
        scoped.len()
    );
}
