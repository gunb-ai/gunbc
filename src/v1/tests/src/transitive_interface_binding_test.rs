//! Discriminating control for the interface-grain resolver increment (S2a move 2
//! increment B, resolver-graph-major-design.md §7): a dependent must see the
//! STRUCTURE of a type it never imports when that type reaches it through a direct
//! import's exported signature (A imports B, B imports C, B's exported fn returns a
//! C-declared record; A projects a field of that record). This is exactly the
//! regression class of #6304 (ancestry cache-sharing dropped transitive bindings,
//! fixed in #6310): a synthesized/flattened parent env that drops the transitive
//! type identity makes the GREEN case fail — and the RED control below proves the
//! checker genuinely reads the transitive structure rather than waving projections
//! through, so a fail-open flatten cannot pass both arms.

use crate::helpers::{compile_multi, diagnostic_messages};
use v1_compiler::v1_compiler_infer::is_error_diagnostic;

const LIB_C: &str = "module fixture.c\n\
    type CPayload { amount: Int }\n";

const LIB_B: &str = "module fixture.b\n\
    import fixture.c { CPayload }\n\
    fn make_payload() -> CPayload { CPayload { amount: 7 } }\n";

fn error_messages(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| format!("{:?}", d.diagnostic))
        .collect()
}

// GREEN arm: the projection through the transitive chain typechecks today.
// This is the arm interface-grain parents must keep green — it goes red the
// moment the flatten drops C's type structure from what flows through B.
#[test]
fn transitive_type_structure_reaches_dependent_through_direct_import_signature() {
    let entry = "module fixture.a\n\
        import fixture.b { make_payload }\n\
        fn read_amount() -> Int { make_payload().amount }\n";
    let result = compile_multi(&[("c.dag", LIB_C), ("b.dag", LIB_B), ("a.dag", entry)]);
    let errors = error_messages(&result);
    assert!(
        errors.is_empty(),
        "A's projection of a C-declared field through B's exported signature must \
         typecheck (transitive type identity flows through the direct import's \
         interface). Errors:\n{}\nAll diagnostics:\n{}",
        errors.join("\n"),
        diagnostic_messages(&result).join("\n")
    );
}

// RED control: a bogus field on the same transitive record MUST produce an error.
// This is what makes the green arm discriminating — a parent-env flatten that is
// blind to CPayload's structure would wave both projections through, and this arm
// catches that fail-open before the green arm's silence can be misread as success.
#[test]
fn bogus_field_on_transitive_record_is_a_typed_error() {
    let entry = "module fixture.a\n\
        import fixture.b { make_payload }\n\
        fn read_bogus() -> Int { make_payload().no_such_field }\n";
    let result = compile_multi(&[("c.dag", LIB_C), ("b.dag", LIB_B), ("a.dag", entry)]);
    let errors = error_messages(&result);
    assert!(
        !errors.is_empty(),
        "projecting a nonexistent field of the transitive record must be a typed \
         error — silence here means inference is blind to the transitive structure. \
         All diagnostics:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}
