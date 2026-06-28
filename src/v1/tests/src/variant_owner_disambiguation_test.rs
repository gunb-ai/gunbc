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

/// §2 — field-level expected-type grounding (struct literal site).
///
/// A struct has two fields with DIFFERENT enum types, both sharing the same variant name.
/// The emitter must pick the correct owner for each field independently.
///
///   struct TwoFields { first: AEarlyEnum, second: ZLaterEnum }
///
/// A constructor `{ first: SharedVarField, second: SharedVarField }` must emit:
///   first: AEarlyEnum::SharedVarField
///   second: ZLaterEnum::SharedVarField
///
/// If the emitter uses a single corpus-global owner for SharedVarField it would emit
/// the same qualifier for both fields — one will be wrong.
#[test]
fn shared_variant_resolves_per_field_by_expected_type() {
    let types_mod = (
        "dsl/test/disc_field_types.dag",
        "module test.disc_field_types\n\
         type AEarlyEnum = SharedVarField | AEarlyOnlyF\n\
         type ZLaterEnum = SharedVarField | ZLaterOnlyF\n\
         type TwoFields = { first: AEarlyEnum, second: ZLaterEnum }",
    );
    let consumer = (
        "dsl/test/disc_field_consumer.dag",
        "module test.disc_field_consumer\n\
         import test.disc_field_types { SharedVarField, AEarlyEnum, ZLaterEnum, TwoFields }\n\
         fn make_two() -> TwoFields { { first: SharedVarField, second: SharedVarField } }",
    );

    let result = compile_multi_target(&[types_mod, consumer], RenderTarget::Rust);
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
        .filter(|f| f.path.contains("disc_field_consumer"))
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("AEarlyEnum::SharedVarField"),
        "first field must emit AEarlyEnum::SharedVarField:\n{emitted}"
    );
    assert!(
        emitted.contains("ZLaterEnum::SharedVarField"),
        "second field must emit ZLaterEnum::SharedVarField:\n{emitted}"
    );
}
