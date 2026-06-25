//! E0624: when a method is called on an Optional receiver, emit_typed_method_call must insert
//! `.expect("fail-closed: ...")` between the receiver and the method template.  Without the
//! unwrap, `Option<Vec<T>>.len()` fails at rustc because Option has no `.len()` method.
//!
//! Root cause: the AlgebraMethodSemantics else-branch emitted the raw recv_str directly into the
//! `{recv}` template slot without checking `resolved_type(receiver).return_cardinality`.  Fix:
//! detect `CardOptional` on the receiver and append `.expect(...)` before template expansion.
//!
//! Gate is the Optional-receiver subset.  A call on a non-Optional List receiver must NOT gain
//! a spurious `.expect(...)` (negative control confirms the gate is not too broad).

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module optional_receiver.fixture\n\n",
    // Optional receiver: schedule.first() -> Optional<List<T>>, then .count()
    "fn count_first_batch<T>(schedule: List<List<T>>) -> Int {\n",
    "  schedule.first().count()\n",
    "}\n\n",
    // Non-Optional receiver: items is already List<T>, .count() directly
    "fn count_items<T>(items: List<T>) -> Int {\n",
    "  items.count()\n",
    "}\n",
);

fn emit_host() -> String {
    compile_dag_named(
        "src/v1/optional_receiver_fixture.dag",
        FIXTURE,
        RenderTarget::Rust,
    )
    .files
    .iter()
    .map(|f| f.content.clone())
    .collect::<Vec<_>>()
    .join("\n")
}

#[test]
fn optional_receiver_gets_expect_unwrap() {
    let emitted = emit_host();
    assert!(
        emitted.contains(".expect(\"fail-closed: Optional receiver for method count"),
        "calling a method on an Optional receiver must emit .expect(fail-closed) before the method:\n{emitted}"
    );
}

#[test]
fn non_optional_receiver_has_no_expect() {
    let emitted = emit_host();
    // count_items has a required List<T> receiver — no Optional, no .expect().
    // Find the fn body for count_items and assert no .expect( in it.
    let needle = "fn count_items";
    let start = emitted
        .find(needle)
        .unwrap_or_else(|| panic!("count_items not found in:\n{emitted}"));
    let body = &emitted[start..];
    let end = body.find("\n}").map(|i| i + 2).unwrap_or(body.len());
    let fn_body = &body[..end];
    assert!(
        !fn_body.contains(".expect("),
        "a non-Optional receiver must NOT emit .expect() (gate is the Optional subset only):\n{fn_body}"
    );
}
