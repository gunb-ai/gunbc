//! **Layer:** integration
//!
//! Boundary Discipline guard for `v4.std.text` (host-crate structural lint).
//!
//! **Host-test preservation justification (W1-W4 qualifying bar — provably
//! runtime-inexpressible):** non-behavioral: asserts type-ABSENCE of
//! ByteString/FileBody/FileContent/TargetSource in text.dag — no runtime witness can
//! express type-non-existence.
//!
//! The behavioral content of the retired `v4_std_text_dag_smoke_test.rs` migrated to
//! discriminating claim-run witnesses in `src/v4/test/claim/std_text/carrier_claims.dag`
//! (executed through v2, mutation-proven). One assertion in that smoke test was NOT
//! behavioral and therefore cannot ride a runtime witness: the **type-absence** boundary
//! lint that `v4.std.text` must not redeclare byte/file/target carriers as text. Per
//! TESTING discipline a source-structure grep belongs in a host crate, not a `.dag`
//! claim, so it is preserved here — a minimal structural guard, not the full smoke.
//!
//! **P5 receipt (INVARIANTS.md §P5 — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST` same-path
//! registration, mechanism (b)):** the matching `EXPECTED_HAND_AUTHORED_TEST` row in
//! `self_gen0_census_test.rs` lands in the same PR. Net of the fold-delete remains strongly
//! negative (bulk parse-surface smoke deleted; only this absence-lint retained).
//! Dissolves when a `.dag`/host structural mechanism can assert a module's declared-type
//! set directly (type-absence as data).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const TEXT_DAG: &str = include_str!("../../../../v4/std/text.dag");
const TEXT_PATH: &str = "src/v4/std/text.dag";

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum { name: decl_name, .. }
                | SurfaceItem::TypeRecord { name: decl_name, .. }
                | SurfaceItem::TypeAlias { name: decl_name, .. }
                | SurfaceItem::TypeAtom { name: decl_name, .. }
                if decl_name == name
        )
    })
}

#[test]
fn v4_std_text_dag_does_not_redeclare_byte_file_target_carriers_as_text() {
    let tokens = tokenize_for_test(TEXT_DAG, TEXT_PATH)
        .unwrap_or_else(|e| panic!("{TEXT_PATH}: tokenize: {e:?}"));
    let module =
        parse_for_test(&tokens, TEXT_PATH).unwrap_or_else(|e| panic!("{TEXT_PATH}: parse: {e:?}"));

    for forbidden in ["ByteString", "FileBody", "FileContent", "TargetSource"] {
        assert!(
            !surface_declares_type(&module, forbidden),
            "{TEXT_PATH}: text module must not redeclare byte/file/target carriers as text \
             (found `{forbidden}`); these remain byte carriers until a separate encoding \
             witness admits them as text"
        );
    }
}
