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
        "dsl/test/disc_transitive.dag",
        "module test.disc_transitive\ntype ATransitiveOwner = SharedV | TransitiveOnly",
    );
    // Source module: imports ATransitiveOwner from transitive (pollutes its type_env),
    // then locally defines ZLocalOwner (the correct owner of SharedV).
    let source = (
        "dsl/test/disc_source.dag",
        "module test.disc_source\n\
         import test.disc_transitive { SharedV, ATransitiveOwner }\n\
         type ZLocalOwner = SharedV | LocalOnly",
    );
    // Consumer: imports SharedV from source; fn return type is ZLocalOwner.
    // Wrong owner → ATransitiveOwner::SharedV → type mismatch in inference → diagnostic.
    // Correct owner → ZLocalOwner::SharedV → valid, emitted code contains ZLocalOwner::SharedV.
    let consumer = (
        "dsl/test/disc_consumer.dag",
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
/// `SharedVarAmbig` belongs to TWO locally-defined enums in the same module:
///   `AEarlyEnum` (sorts first alphabetically — the wrong default)
///   `ZLaterEnum` (sorts last)
///
/// The function return type is `ZLaterEnum`, so at the call site the expected type is
/// `ZLaterEnum`.  The correct emit is `ZLaterEnum::SharedVarAmbig`.
/// The alpha-sort fallback would emit `AEarlyEnum::SharedVarAmbig` — wrong.
///
/// Negative control: `AEarlyEnum` MUST NOT appear in the emitted return expression.
#[test]
fn shared_variant_resolves_by_expected_type_not_alpha_order() {
    // Module defines both enums locally: AEarlyEnum (alpha-first, wrong) and ZLaterEnum
    // (alpha-last, correct for the declared return type).
    let owner_mod = (
        "dsl/test/disc_ambig_owner.dag",
        "module test.disc_ambig_owner\n\
         type AEarlyEnum = SharedVarAmbig | AEarlyOnly\n\
         type ZLaterEnum = SharedVarAmbig | ZLaterOnly",
    );
    // Consumer: imports SharedVarAmbig from the owner module.
    // Return type is ZLaterEnum — the expected type at this site must pick ZLaterEnum.
    let consumer = (
        "dsl/test/disc_ambig_consumer.dag",
        "module test.disc_ambig_consumer\n\
         import test.disc_ambig_owner { SharedVarAmbig, AEarlyEnum, ZLaterEnum }\n\
         fn make_z() -> ZLaterEnum { SharedVarAmbig }",
    );

    let result = compile_multi_target(&[owner_mod, consumer], RenderTarget::Rust);
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
        .filter(|f| f.path.contains("disc_ambig_consumer"))
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("ZLaterEnum::SharedVarAmbig"),
        "emitted code must use the expected-type owner ZLaterEnum:\n{emitted}"
    );
    assert!(
        !emitted.contains("AEarlyEnum::SharedVarAmbig"),
        "emitted code must NOT use the alpha-order owner AEarlyEnum:\n{emitted}"
    );
}

/// §2b — per-function expected-type grounding: two functions with DIFFERENT declared
/// return types, both returning the same shared variant.  Each must emit the enum
/// matching ITS OWN declared return type, proving the fix is per-site not per-module.
///
/// `make_early() -> AEarlyEnum` → must emit `AEarlyEnum::SharedVarField`
/// `make_late()  -> ZLaterEnum` → must emit `ZLaterEnum::SharedVarField`
///
/// Both variants live in the same scope (same import list), so without expected-type
/// grounding both would collapse to `AEarlyEnum` (alpha-first).
#[test]
fn shared_variant_resolves_per_site_independently() {
    let owner_mod = (
        "dsl/test/disc_per_site_owner.dag",
        "module test.disc_per_site_owner\n\
         type AEarlyEnum = SharedVarField | AEarlyOnlyF\n\
         type ZLaterEnum = SharedVarField | ZLaterOnlyF",
    );
    let consumer = (
        "dsl/test/disc_per_site_consumer.dag",
        "module test.disc_per_site_consumer\n\
         import test.disc_per_site_owner { SharedVarField, AEarlyEnum, ZLaterEnum }\n\
         fn make_early() -> AEarlyEnum { SharedVarField }\n\
         fn make_late() -> ZLaterEnum { SharedVarField }",
    );

    let result = compile_multi_target(&[owner_mod, consumer], RenderTarget::Rust);
    let diags: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        diags.is_empty(),
        "expected no diagnostics, got: {diags:?}"
    );

    let emitted: String = result
        .files
        .iter()
        .filter(|f| f.path.contains("disc_per_site_consumer"))
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
