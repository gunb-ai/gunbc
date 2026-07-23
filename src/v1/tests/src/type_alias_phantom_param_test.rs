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

use crate::helpers::{
    compile_dag_named, compile_dag_named_with_source_roots, find_file, v2_layer_roots,
};
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

// Sibling defect, same phantom-marker machinery: `src/v2/std/integer.dag`'s `StandardIntegerType`
// coproduct reuses the exact identifiers (`Int128`, `Int64`, ... `UInt8`) of the module's own
// `type IntN = Compose<Int, MachineWidth<WordN>>` aliases. Where a `StandardIntegerType` variant
// is used as a bare type-argument (as in `src/v2/compiler/fold_lowering.dag`'s closure), the
// phantom-marker collector used to synthesize a second, bogus
// `#[derive(...)]\npub struct Int128;` unconditionally — colliding with the real
// `pub type Int128 = Compose<...>;` alias and producing genuine rustc E0428 ("the name `Int128`
// is defined multiple times"). Fix: `phantom_marker_name_shadowed_by_real_type_item` excludes any
// name that already has a real type-alias or type-decl item in the same module from the
// phantom-marker roster.
//
// RED control: this compiles the real `fold_lowering.dag` closure (the exact corpus entry that
// exercised the bug) through the full source-root resolution the production emitter uses, not a
// hand-trimmed fixture — so a regression in either the `.dag` source or a stage0 Rust seed left
// out of sync with it fails this test.
#[test]
fn integer_module_has_no_duplicate_int128_definition() {
    let roots = v2_layer_roots();
    let entry = "src/v2/compiler/fold_lowering.dag";
    let source = crate::helpers::read_v2_file(entry);
    let result = compile_dag_named_with_source_roots(entry, &source, RenderTarget::Rust, &roots);

    let emitted = find_file(&result, "src/v2_std_integer.rs");

    assert!(
        !emitted.contains("pub struct Int128;"),
        "phantom-ZST marker for `Int128` must not be emitted when a real `Int128` type alias \
         already exists in the module — got:\n{emitted}"
    );

    let type_alias_count = emitted.matches("pub type Int128 =").count();
    assert_eq!(
        type_alias_count, 1,
        "expected exactly one `Int128` type alias, found {type_alias_count} in:\n{emitted}"
    );

    // A dangling `#[derive(...)]` (no item following) is invalid Rust ("expected item after
    // attributes") — guard against any residual line-strip leaving one behind.
    let lines: Vec<&str> = emitted.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("#[derive(") {
            let next_nonblank = lines[i + 1..]
                .iter()
                .find(|l| !l.trim().is_empty())
                .unwrap_or(&"");
            assert!(
                next_nonblank.contains("struct ")
                    || next_nonblank.contains("enum ")
                    || next_nonblank.contains("#["),
                "dangling `#[derive(...)]` with no following item at line {i}:\n{emitted}"
            );
        }
    }
}
