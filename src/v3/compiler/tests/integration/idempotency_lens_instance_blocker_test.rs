//! **Layer:** integration
//!
//! #1262 idempotency lens instance migration blocker receipt.
//!
//! The real `data idempotency_workflow_lens: Lens<...>` instance is deferred:
//! current lowering rejects the generic function-valued data fields needed for
//! `Lens<C>` / `Monoid<C>`, and function-body lowering can misclassify ordinary
//! sum-return helper calls as constructor expressions. These tests pin those
//! prerequisite gaps without introducing a Rust-only or test-only bridge.

use v3_compiler::{compile_to_dag, CompileError, Diagnostic};

fn semantic_diagnostics(source: &str, file: &str) -> Vec<Diagnostic> {
    let err = compile_to_dag(source, file).expect_err("fixture should pin current lowerer gap");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic lowerer gap for {file}, got {err:?}");
    };
    dag.diagnostics()
        .iter()
        .map(|(_, diagnostic)| diagnostic.clone())
        .collect()
}

fn has_function_field_ref_resolve_error(
    diagnostics: &[Diagnostic],
    source: &str,
    file: &str,
    expected_field_ref: &str,
) -> bool {
    diagnostics.iter().any(|diagnostic| {
        let Diagnostic::ResolveError { span, .. } = diagnostic else {
            return false;
        };
        if span.file != file {
            return false;
        }
        let Ok(start) = usize::try_from(span.byte_start) else {
            return false;
        };
        let Ok(end) = usize::try_from(span.byte_end) else {
            return false;
        };
        source
            .get(start..end)
            .is_some_and(|text| text.contains(expected_field_ref))
    })
}

#[test]
fn generic_lens_monoid_function_field_refs_are_current_lowerer_gap() {
    let source = r#"
module blocker.lens_data_fields

import std.algebra { Monoid }

type MiniLens<C> {
  read: fn(Int) -> C
  sequential: Monoid<C>
}

fn read_int(x: Int) -> Int = x
fn add_int(a: Int, b: Int) -> Int = a + b

data int_monoid: Monoid<Int> = {
  op: add_int,
  identity: 0
}

data int_lens: MiniLens<Int> = {
  read: read_int,
  sequential: int_monoid
}
"#;

    let file = "idempotency_lens_function_field_gap.dag";
    let diagnostics = semantic_diagnostics(source, file);

    assert!(
        has_function_field_ref_resolve_error(&diagnostics, source, file, "op: add_int")
            || has_function_field_ref_resolve_error(&diagnostics, source, file, "read: read_int"),
        "expected generic data-body function refs to hit the Lens/Monoid lowerer gap; got: {diagnostics:?}"
    );
}

#[test]
fn imported_sum_return_helper_calls_are_ordinary_calls() {
    let source = r#"
module blocker.imported_sum_return_helper

import std.effects {
  WorkflowIdempotencyReport,
  report_unsupported_workflow_variant
}

fn use_imported_helper() -> WorkflowIdempotencyReport =
  report_unsupported_workflow_variant(
    "BranchEffect",
    "lane2_stage2b_idempotency_lens",
    "branch-wise idempotency composition is not available in this lane"
  )
"#;

    let dag = compile_to_dag(source, "idempotency_lens_sum_helper_call.dag")
        .expect("imported sum-return helper calls should lower as ordinary calls");
    assert!(
        dag.diagnostics().is_empty(),
        "imported sum-return helper fixture should compile cleanly; got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}
