//! E0599: when a function's return type is a bare generic param (e.g. `M` in
//! `fn unwrap_value<M>(w: Wrapper<M>) -> M`), emit_fn_def must add `M: Clone` to the generic
//! param list.  Without the bound, `w.value.clone()` fails at rustc because `Box<M>: Clone`
//! requires `M: Clone`.
//!
//! Root cause: emit_fn_def called emit_type_params (no bounds) even when the return type is a
//! bare generic param.  Fix: detect "authored name of return type ∈ generic_param_names" and
//! switch to emit_type_params_with_clone_bound for that param.
//!
//! Gate is the bare-generic-return subset.  A function returning a concrete type (Int) must NOT
//! gain a spurious `: Clone` bound (negative control confirms the gate is not too broad).

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module generic_return.fixture\n\n",
    // Two fields prevent single-field collapse; `value: M` is the bare-generic-return field.
    "type Wrapper<M> {\n",
    "  value: M\n",
    "  tag: Int\n",
    "}\n\n",
    // Bare generic return — must get M: Clone.
    "fn unwrap_value<M>(w: Wrapper<M>) -> M {\n",
    "  w.value\n",
    "}\n\n",
    // Concrete return — must NOT add a spurious Clone bound.
    "fn constant_forty_two() -> Int {\n",
    "  42\n",
    "}\n",
);

fn emit_host() -> String {
    compile_dag_named(
        "src/v1/generic_return_fixture.dag",
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
fn bare_generic_return_gets_clone_bound() {
    let emitted = emit_host();
    assert!(
        emitted.contains("fn unwrap_value<M: Clone>"),
        "a function returning a bare generic param M must emit `M: Clone` in the type params:\n{emitted}"
    );
}

#[test]
fn concrete_return_has_no_clone_bound() {
    let emitted = emit_host();
    // constant_forty_two returns Int (concrete), no generic params at all.
    assert!(
        !emitted.contains("fn constant_forty_two<"),
        "a function returning a concrete type must NOT emit generic params with a Clone bound:\n{emitted}"
    );
}
