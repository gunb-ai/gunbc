//! Semantics-preservation proof: native fold_list / fold_list_right fast paths match the
//! interpreted .dag bodies on order- and associativity-sensitive inputs.

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const SEMANTICS_ENTRY: &str = "src/v4/test/claim/manual/fold_list_native_semantics.dag";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn semantics_sources() -> Vec<Rc<v2_compiler::v2_compiler_compile::SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(SEMANTICS_ENTRY))
        .unwrap_or_else(|e| panic!("read {SEMANTICS_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(
        SEMANTICS_ENTRY,
        &entry_content,
        &v4_source_roots(),
    )
    .iter()
    .map(|s| {
        Rc::new(v2_compiler::v2_compiler_compile::SourceFile {
            path: s.path.clone(),
            content: s.content.clone(),
        })
    })
    .collect()
}

fn assert_resolved_ok(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph for {SEMANTICS_ENTRY}, got {msgs:?}"
    );
}

fn run_witness(function: &str, disable_native: bool) -> Value {
    v2_interpreter::fold_native_hit_counts_reset();
    let resolved = compile_to_resolved(Rc::new(semantics_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v2_interpreter::InterpContext::new(graph, resolved.source_indices.clone(), false);
    if disable_native {
        // SAFETY: single-threaded test; cleared before return.
        unsafe { std::env::set_var("GUNBC_INTERP_DISABLE_FOLD_NATIVE", "1") };
    }
    let result = v2_interpreter::run_in_context(&ctx, function, false)
        .unwrap_or_else(|e| panic!("run {function} (native_disabled={disable_native}): {e:?}"));
    if disable_native {
        unsafe { std::env::remove_var("GUNBC_INTERP_DISABLE_FOLD_NATIVE") };
    }
    let (fold_list, fold_list_right) = v2_interpreter::fold_native_hit_counts_snapshot();
    if disable_native {
        assert_eq!(
            (fold_list, fold_list_right),
            (0, 0),
            "{function}: interpreted path must not invoke native fold fast paths"
        );
    } else {
        assert!(
            fold_list > 0 || fold_list_right > 0,
            "{function}: native fast-path must be engaged (fold_list={fold_list}, fold_list_right={fold_list_right})"
        );
    }
    result
}

fn assert_native_matches_interpreted(function: &str) {
    let native = run_witness(function, false);
    let interpreted = run_witness(function, true);
    assert_eq!(
        format!("{native:?}"),
        format!("{interpreted:?}"),
        "{function}: native fast-path must match interpreted .dag body"
    );
}

#[test]
fn fold_list_left_order_sensitive_matches_interpreted() {
    assert_native_matches_interpreted("fold_list_left_order_sensitive_holds");
}

#[test]
fn fold_list_right_order_sensitive_matches_interpreted() {
    assert_native_matches_interpreted("fold_list_right_order_sensitive_holds");
}

#[test]
fn fold_list_empty_matches_interpreted() {
    assert_native_matches_interpreted("fold_list_empty_holds");
}

#[test]
fn fold_list_right_empty_matches_interpreted() {
    assert_native_matches_interpreted("fold_list_right_empty_holds");
}

#[test]
fn fold_list_single_element_matches_interpreted() {
    assert_native_matches_interpreted("fold_list_single_element_holds");
}

#[test]
fn fold_list_right_single_element_matches_interpreted() {
    assert_native_matches_interpreted("fold_list_right_single_element_holds");
}
