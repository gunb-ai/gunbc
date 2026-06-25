//! E0091: a parametric type alias whose RHS does not reference its type params emits a bare ZST
//! (`Phantom`) with those params unused — Rust rejects unused type params with E0091.
//!
//! Root cause: the type-alias emit arm (`is_emittable_parametric_type_alias_item`) rendered the
//! RHS directly without checking whether params appear in it. When `type Compose<Algebra,
//! MachineConstraint> = Phantom` emits `pub type Compose<Algebra, MachineConstraint> = Phantom;`
//! the two params are absent from the RHS → E0091 × 2.
//!
//! Fix (precedent-application of the struct path at emit_fn_def line ~3229): detect params
//! absent from the RHS via `alias_unused_param_names` (mirrors `struct_unused_param_names`),
//! then substitute `std::marker::PhantomData<(unused_tuple)>` for the RHS — same
//! `rust_phantom_marker_inner` the struct path uses.
//!
//! Guard: only fire when params are genuinely absent. A param-using alias like
//! `type Wrapper<T> = List<T>` has `T` referenced in the RHS → no substitution → left unchanged.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module alias_phantom_param.fixture\n\n",
    "import std.constructors { Phantom }\n\n",
    // Unused-param alias — both type params absent from RHS (the Compose case).
    "type Compose<Algebra, MachineConstraint> = Phantom\n\n",
    // Param-using alias — T appears in the RHS; must NOT get PhantomData substitution.
    "type Wrapper<T> = List<T>\n\n",
    // A concrete type to make the module non-trivial.
    "type Tag {\n",
    "  id: Int\n",
    "}\n",
);

fn emit_host() -> String {
    compile_dag_named(
        "src/v1/alias_phantom_param_fixture.dag",
        FIXTURE,
        RenderTarget::Rust,
    )
    .files
    .iter()
    .map(|f| f.content.clone())
    .collect::<Vec<_>>()
    .join("\n")
}

fn alias_line(emitted: &str, name: &str) -> String {
    let needle = format!("type {name}<");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("alias `{name}` not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest.find('\n').unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn unused_param_alias_gets_phantom_data() {
    let emitted = emit_host();
    let compose = alias_line(&emitted, "Compose");
    assert!(
        compose.contains("PhantomData"),
        "Compose with unused params must emit PhantomData to satisfy E0091, got:\n{compose}"
    );
    assert!(
        compose.contains("Algebra") && compose.contains("MachineConstraint"),
        "both unused params must appear inside PhantomData, got:\n{compose}"
    );
}

#[test]
fn param_using_alias_has_no_phantom_data() {
    let emitted = emit_host();
    let wrapper = alias_line(&emitted, "Wrapper");
    assert!(
        !wrapper.contains("PhantomData"),
        "Wrapper<T> uses T in its RHS and must NOT get a PhantomData substitution, got:\n{wrapper}"
    );
}
