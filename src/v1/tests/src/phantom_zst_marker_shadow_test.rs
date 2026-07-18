//! Discriminating receipt for the phantom-ZST-marker/real-type-alias collision.
//!
//! `src/v2/std/integer.dag`'s `StandardIntegerType` coproduct reuses the exact identifiers
//! (`Int128`, `Int64`, ... `UInt8`) of the module's own `type IntN = Compose<Int,
//! MachineWidth<WordN>>` aliases. Where a `StandardIntegerType` variant is used as a bare
//! type-argument (as in `src/v2/compiler/fold_lowering.dag`'s closure), the Rust emitter's
//! `collect_phantom_zst_marker_names`/`emit_module_phantom_zst_markers` used to synthesize a
//! second, bogus `#[derive(...)]\npub struct Int128;` unconditionally — colliding with the real
//! `pub type Int128 = Compose<...>;` alias and producing genuine rustc E0428
//! ("the name `Int128` is defined multiple times").
//!
//! Fix: `phantom_marker_name_shadowed_by_real_type_item` excludes any name that already has a
//! real type-alias or type-decl item in the same module from the phantom-marker roster.
//!
//! RED control: this test compiles the real `fold_lowering.dag` closure (the exact corpus entry
//! that exercised the bug) through the full source-root resolution the production emitter uses,
//! not a hand-trimmed fixture — so a regression in either the `.dag` source or a stage0 Rust
//! seed left out of sync with it (the review-caught gap this receipt exists to prevent) fails
//! this test.

use crate::helpers::{compile_dag_named_with_source_roots, find_file, v2_layer_roots};
use v1_compiler::v1_compiler_artifact::RenderTarget;

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
