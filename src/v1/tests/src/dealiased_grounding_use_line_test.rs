//! E0425 class: dealiased grounding aliases (`Nat = CommutativeSemiring<Magnitude>`)
//! emit foreign RHS leaf types (e.g. `Magnitude`) unqualified while the import
//! graph suppresses them as non-emittable opaque decls. Witness: supplemental
//! use-line emission + authored-alias preservation in field types.

use crate::helpers::{
    assert_no_diagnostics, compile_dag_named_with_source_roots, find_file, source_roots,
};
use v1_compiler::v1_compiler_artifact::RenderTarget;

#[test]
fn std_nat_alias_rhs_emits_magnitude_use_line() {
    let ws = crate::helpers::workspace_root();
    let source = std::fs::read_to_string(ws.join("dag/std/nat.dag")).expect("read dag/std/nat.dag");
    let result = compile_dag_named_with_source_roots(
        "dag/std/nat.dag",
        &source,
        RenderTarget::Rust,
        &source_roots(),
    );
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/std_nat.rs");
    assert!(
        content.contains("use crate::std_magnitude::Magnitude"),
        "dealiased grounding alias RHS must import foreign leaf `Magnitude`, got:\n{content}"
    );
    assert!(
        content.contains("Magnitude"),
        "expected `Magnitude` in alias RHS, got:\n{content}"
    );
}

#[test]
fn imported_nat_field_preserves_authored_alias_not_dealiased_rhs() {
    let source = concat!(
        "module dealiased_grounding.fixture\n\n",
        "import std.nat { Nat }\n\n",
        "type Box {\n",
        "  value: Nat\n",
        "}\n"
    );
    let result = compile_dag_named_with_source_roots(
        "src/v1/dealiased_grounding_fixture.dag",
        source,
        RenderTarget::Rust,
        &source_roots(),
    );
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/dealiased_grounding_fixture.rs");
    assert!(
        content.contains("value: Nat") || content.contains("value: Rc<Nat>"),
        "field typed `Nat` must preserve the authored alias name, got:\n{content}"
    );
    assert!(
        !content.contains("CommutativeSemiring"),
        "field typed `Nat` must not peel to dealiased grounding RHS, got:\n{content}"
    );
}
