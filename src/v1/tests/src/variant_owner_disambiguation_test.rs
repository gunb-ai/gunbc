//! Discriminating witnesses for variant-owner resolution under the
//! constructor-owner ruling (§1c, operator 2026-07-04).
//!
//! These tests previously witnessed the *disambiguation* semantics
//! (expected-type-at-site picks, local-shadows-transitive, per-site
//! independence). The operator ruled that ambiguity support itself is the
//! bug: one arm name bound to two different owners in one scope is a
//! VariantCollision, never a pick. The fixtures survive as the red controls
//! for that wall, plus a positive control that the binding edge alone names
//! the owner all the way through emission.
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_std_core::CompilerDiagnostic;

use crate::helpers::compile_multi_target;

/// RED control (import-vs-local): `SharedV` arrives via a specific-name
/// import AND a local coproduct declares it — two owners in one scope is a
/// collision, not a local-wins shadow.
#[test]
fn shared_arm_via_import_and_local_decl_is_a_collision() {
    let transitive = (
        "dag/test/disc_transitive.dag",
        "module test.disc_transitive\ntype ATransitiveOwner = SharedV | TransitiveOnly",
    );
    let source = (
        "dag/test/disc_source.dag",
        "module test.disc_source\n\
         import test.disc_transitive { SharedV, ATransitiveOwner }\n\
         type ZLocalOwner = SharedV | LocalOnly",
    );
    let result = compile_multi_target(&[transitive, source], RenderTarget::Rust);
    let has_collision = result.diagnostics.iter().any(|d| {
        matches!(
            &*d.diagnostic,
            CompilerDiagnostic::VariantCollision { variant, .. } if variant == "SharedV"
        )
    });
    assert!(
        has_collision,
        "an arm bound by both an import and a local declaration must collide, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
}

/// RED control (the deleted expected-type picker): two locally-defined
/// owners share an arm; the site's declared return type may NOT pick a
/// winner — the scope is malformed and fails closed.
#[test]
fn two_local_owners_for_one_arm_collide_instead_of_expected_type_pick() {
    let source = (
        "dag/test/disc_ambig.dag",
        "module test.disc_ambig\n\
         type ZLaterEnum = SharedVarAmbig | ZLaterOnly\n\
         type AEarlyEnum = SharedVarAmbig | AEarlyOnly\n\
         fn make_z() -> ZLaterEnum { SharedVarAmbig }",
    );
    let result = compile_multi_target(&[source], RenderTarget::Rust);
    let has_collision = result.diagnostics.iter().any(|d| {
        matches!(
            &*d.diagnostic,
            CompilerDiagnostic::VariantCollision { variant, enum1, enum2, .. }
                if variant == "SharedVarAmbig"
                    && ((enum1 == "ZLaterEnum" && enum2 == "AEarlyEnum")
                        || (enum1 == "AEarlyEnum" && enum2 == "ZLaterEnum"))
        )
    });
    assert!(
        has_collision,
        "two local owners for one arm must collide (expected-type picking is deleted), got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
}

/// GREEN control (emission axis): with exactly one owner in scope, the
/// binding edge names the owner and the emitted Rust carries it — no
/// expected type needed at either site.
#[test]
fn unambiguous_arm_emits_binding_edge_owner() {
    let source = (
        "dag/test/disc_unambig.dag",
        "module test.disc_unambig\n\
         type OnlyOwner = SoleArm | OtherArm\n\
         fn make_it() -> OnlyOwner { SoleArm }",
    );
    let result = compile_multi_target(&[source], RenderTarget::Rust);
    let diags: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(diags.is_empty(), "expected no diagnostics, got: {diags:?}");
    let emitted: String = result
        .files
        .iter()
        .filter(|f| f.path.contains("disc_unambig"))
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("OnlyOwner::SoleArm"),
        "emitted code must carry the binding-edge owner:\n{emitted}"
    );
}
