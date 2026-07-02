/// Discriminating witness for variant owner-selection correctness.
///
/// Two grounding axes, one test each:
///
/// §1 — local-vs-transitive: when one owner is locally defined and another is transitively
///   imported, the locally-defined owner wins (PR #5879).
///
/// §2 — expected-type-at-site (this PR): when BOTH owners are locally defined (or both
///   imported) in the same scope, the EXPECTED/FIELD type at each individual call site
///   is the authoritative discriminant — not scope-order or alpha-sort. This is the
///   deeper §3 grounding: the single authority for which enum owns the variant at a
///   given site is the declared type of that site.
use v1_compiler::v1_compiler_artifact::RenderTarget;

use crate::helpers::compile_multi_target;

/// Two coproducts share the variant name `SharedV`.
/// `ATransitiveOwner` is defined in the "transitive" module and brought into "source"
/// via a specific-name import — so it lands in source's type_env.bindings transitively.
/// `ZLocalOwner` is LOCALLY DEFINED in "source".
///
/// "consumer" imports `SharedV` from "source".  The unique locally-defined parent for
/// `SharedV` in "source" is `ZLocalOwner`, so the emitter must emit `ZLocalOwner::SharedV`.
#[test]
fn shared_variant_resolves_to_locally_defined_owner_not_transitive() {
    // Module that provides ATransitiveOwner (wrong candidate, sorts first alphabetically).
    let transitive = (
        "dag/test/disc_transitive.dag",
        "module test.disc_transitive\ntype ATransitiveOwner = SharedV | TransitiveOnly",
    );
    // Source module: imports ATransitiveOwner from transitive (pollutes its type_env),
    // then locally defines ZLocalOwner (the correct owner of SharedV).
    let source = (
        "dag/test/disc_source.dag",
        "module test.disc_source\n\
         import test.disc_transitive { SharedV, ATransitiveOwner }\n\
         type ZLocalOwner = SharedV | LocalOnly",
    );
    // Consumer: imports SharedV from source; fn return type is ZLocalOwner.
    // Wrong owner → ATransitiveOwner::SharedV → type mismatch in inference → diagnostic.
    // Correct owner → ZLocalOwner::SharedV → valid, emitted code contains ZLocalOwner::SharedV.
    let consumer = (
        "dag/test/disc_consumer.dag",
        "module test.disc_consumer\n\
         import test.disc_source { SharedV, ZLocalOwner }\n\
         fn make_shared() -> ZLocalOwner { SharedV }",
    );

    let result = compile_multi_target(&[transitive, source, consumer], RenderTarget::Rust);
    let diags: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        diags.is_empty(),
        "expected no diagnostics (wrong owner causes type mismatch), got: {diags:?}"
    );

    let emitted: String = result
        .files
        .iter()
        .filter(|f| f.path.contains("disc_consumer"))
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("ZLocalOwner::SharedV"),
        "emitted code must use the locally-defined owner ZLocalOwner, not the transitive ATransitiveOwner:\n{emitted}"
    );
    assert!(
        !emitted.contains("ATransitiveOwner::SharedV"),
        "emitted code must NOT use the transitive owner ATransitiveOwner:\n{emitted}"
    );
}

/// §2 — expected-type-at-site grounding.
///
/// `SharedVarAmbig` belongs to TWO locally-defined enums in a single module:
///   `ZLaterEnum` (defined first — scope will be overwritten)
///   `AEarlyEnum` (defined second — wins scope by last-write in variant_locals_from_items)
///
/// The function return type is `ZLaterEnum`, so the expected type overrides the scope
/// winner `AEarlyEnum`.  The correct emit is `ZLaterEnum::SharedVarAmbig`.
/// Without the fix the scope winner `AEarlyEnum::SharedVarAmbig` causes a type mismatch.
///
/// Single-module (no imports): imported_enum_names is empty → no VariantCollision.
#[test]
fn shared_variant_resolves_by_expected_type_not_alpha_order() {
    // ZLaterEnum defined first, AEarlyEnum second → AEarlyEnum wins scope (last-write).
    // fn make_z return type is ZLaterEnum → expected-type override must fire.
    let source = (
        "dag/test/disc_ambig.dag",
        "module test.disc_ambig\n\
         type ZLaterEnum = SharedVarAmbig | ZLaterOnly\n\
         type AEarlyEnum = SharedVarAmbig | AEarlyOnly\n\
         fn make_z() -> ZLaterEnum { SharedVarAmbig }",
    );

    let result = compile_multi_target(&[source], RenderTarget::Rust);
    let diags: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        diags.is_empty(),
        "expected no diagnostics — wrong owner causes type mismatch, got: {diags:?}"
    );

    let emitted: String = result
        .files
        .iter()
        .filter(|f| f.path.contains("disc_ambig"))
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("ZLaterEnum::SharedVarAmbig"),
        "emitted code must use the expected-type owner ZLaterEnum:\n{emitted}"
    );
    assert!(
        !emitted.contains("AEarlyEnum::SharedVarAmbig"),
        "emitted code must NOT use the scope-order winner AEarlyEnum:\n{emitted}"
    );
}

/// §2b — per-site independence: two functions with DIFFERENT declared return types in the
/// same module, both returning the same shared variant.
///
/// `make_early() -> AEarlyEnum` → scope already picks AEarlyEnum (last-write wins) → no
///   override needed → must emit `AEarlyEnum::SharedVarField`
/// `make_late()  -> ZLaterEnum` → scope picks AEarlyEnum (wrong) → expected-type override
///   fires → must emit `ZLaterEnum::SharedVarField`
///
/// Both functions in the same scope prove the fix is per-site not per-module.
/// Single-module (no imports): no VariantCollision possible.
#[test]
fn shared_variant_resolves_per_site_independently() {
    // ZLaterEnum defined first, AEarlyEnum second → AEarlyEnum wins scope (last-write).
    // make_early return type matches scope winner → no override, correct trivially.
    // make_late return type is ZLaterEnum → override fires for that site only.
    let source = (
        "dag/test/disc_per_site.dag",
        "module test.disc_per_site\n\
         type ZLaterEnum = SharedVarField | ZLaterOnlyF\n\
         type AEarlyEnum = SharedVarField | AEarlyOnlyF\n\
         fn make_early() -> AEarlyEnum { SharedVarField }\n\
         fn make_late() -> ZLaterEnum { SharedVarField }",
    );

    let result = compile_multi_target(&[source], RenderTarget::Rust);
    let diags: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(diags.is_empty(), "expected no diagnostics, got: {diags:?}");

    let emitted: String = result
        .files
        .iter()
        .filter(|f| f.path.contains("disc_per_site"))
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("AEarlyEnum::SharedVarField"),
        "make_early must emit AEarlyEnum::SharedVarField:\n{emitted}"
    );
    assert!(
        emitted.contains("ZLaterEnum::SharedVarField"),
        "make_late must emit ZLaterEnum::SharedVarField:\n{emitted}"
    );
}
