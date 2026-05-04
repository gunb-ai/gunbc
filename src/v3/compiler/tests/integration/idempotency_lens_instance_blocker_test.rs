//! **Layer:** integration
//!
//! #1262 idempotency lens instance migration blocker receipt.
//!
//! The real `data idempotency_workflow_lens: Lens<...>` instance is deferred,
//! but E6-G0 closes the generic function-valued structural data field blocker
//! for `Lens<C>` / `Monoid<C>`-shaped values. These tests keep the historical
//! blocker receipt current without introducing a Rust-only or test-only bridge.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{FieldValue, ValueBody};

#[test]
fn generic_lens_monoid_function_field_refs_lower_structurally() {
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
    let dag = compile_to_dag(source, file)
        .expect("E6-G0 should lower generic Lens/Monoid function field refs structurally");
    assert!(
        dag.diagnostics().is_empty(),
        "generic Lens/Monoid function field fixture should compile cleanly; got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let read_int = dag.declaration_by_name("read_int").unwrap().id;
    let int_monoid = dag.declaration_by_name("int_monoid").unwrap().id;
    let int_lens = dag.declaration_by_name("int_lens").unwrap();
    let Some(ValueBody::Structural { fields }) = &int_lens.value_body else {
        panic!("int_lens must lower to ValueBody::Structural, got {:?}", int_lens.value_body);
    };
    assert_eq!(
        fields.iter().find(|(label, _)| label == "read").map(|(_, value)| value),
        Some(&FieldValue::Reference(read_int))
    );
    assert_eq!(
        fields
            .iter()
            .find(|(label, _)| label == "sequential")
            .map(|(_, value)| value),
        Some(&FieldValue::Reference(int_monoid))
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
