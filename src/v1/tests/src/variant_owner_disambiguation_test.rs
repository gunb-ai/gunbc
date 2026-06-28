/// Discriminating witness for variant owner-selection correctness.
///
/// Root: when a variant name appears in two coproducts and one of those coproducts
/// is transitively imported into the source module (not locally defined), the emitter
/// must resolve the variant to the LOCALLY-DEFINED owner — not the transitive one.
///
/// Designed so the wrong owner (ATransitiveOwner) sorts BEFORE the correct owner
/// (ZLocalOwner) alphabetically; the buggy transitive-type_env path emits
/// `ATransitiveOwner::SharedV`, which would type-mismatch against the declared return
/// type `ZLocalOwner`.  The correct local-items path emits `ZLocalOwner::SharedV`.
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
