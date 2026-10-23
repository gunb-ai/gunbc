//! The half of the item-kind climb that a source read cannot establish.
//!
//! Every emitter's item dispatch is now a `match` over `v1.std.core ParsedModuleItemKind`
//! (v1.compiler.emit_rust `emit_typed_item`, emit_go `emit_go_typed_item`, emit_python
//! `emit_py_typed_item`). That the dispatch is a match is visible in the source; that a match
//! missing a variant CANNOT BE ACCEPTED is a property of the substrate, and it is the whole
//! reason the dispatch is structural rather than merely tidy. Without it the climb would rest on
//! reading three files and believing they stay that way.
//!
//! DISCRIMINATING RED: fixture source whose match omits one variant. The forbidden state is
//! authorable HERE even though no accepted program contains it -- which is the case DESIGN
//! section 4b names when it says a compiler's regression probes are invalid programs, and the
//! reason "no accepted program can express it" is not grounds to skip the probe.
//!
//! POSITIVE CONTROL: the same fixture with every arm present must compile clean, so the RED
//! cannot be satisfied by a compiler that refuses this shape of program for any other reason.
//!
//! The same two arms were also executed against the REAL ParsedModuleItemKind rather than a
//! fixture enum, by compiling a probe module under the live source roots:
//!   gunbc run --source-root dag --source-root src/v1 --source-root src/v2 \
//!     --source-root <probe dir> --claim-run --entry <probe>.dag --function probe_holds
//! with the probe's `match` over the imported ParsedModuleItemKind missing NotAModuleItem. It
//! refused with `non-exhaustive match: missing variant(s) NotAModuleItem` while the seven-arm
//! control returned PASS. That probe is a throwaway invalid program and is deliberately not
//! committed -- an invalid module inside a source root fails every scan that reads the root --
//! so the recipe is recorded here and the enrolled form is the fixture below.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_std_core::{diagnostic_to_message, is_error_diagnostic};

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn error_diag_messages(sources: Vec<Rc<SourceFile>>) -> Vec<String> {
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    resolved
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

/// A closed item-kind coproduct of the same shape as ParsedModuleItemKind, and one dispatch over
/// it. `arms` is spliced in so the RED and the control differ in exactly one arm.
fn dispatch_fixture(arms: &str) -> Vec<Rc<SourceFile>> {
    vec![src(
        "kinddispatch.dag",
        &format!(
            "module kinddispatch.probe\n\n\
             type FixItemKind\n  \
               = FixTypeDeclaration\n  \
               | FixFunction\n  \
               | FixDataValue\n  \
               | FixService\n  \
               | FixResource\n  \
               | FixUnrecognized\n  \
               | FixNotAnItem\n\n\
             fn dispatch(k: FixItemKind) -> String {{\n  match k {{\n{arms}  }}\n}}\n"
        ),
    )]
}

const ALL_ARMS: &str = "    FixTypeDeclaration => \"type\"\n\
                        \x20   FixFunction => \"fn\"\n\
                        \x20   FixDataValue => \"data\"\n\
                        \x20   FixService => \"service\"\n\
                        \x20   FixResource => \"resource\"\n\
                        \x20   FixUnrecognized => \"unrecognized\"\n\
                        \x20   FixNotAnItem => \"not-an-item\"\n";

#[test]
fn dispatch_missing_one_item_kind_arm_is_refused() {
    let arms_without_not_an_item: String = ALL_ARMS
        .lines()
        .filter(|l| !l.contains("FixNotAnItem"))
        .map(|l| format!("{l}\n"))
        .collect();
    let messages = error_diag_messages(dispatch_fixture(&arms_without_not_an_item));
    assert!(
        messages
            .iter()
            .any(|m| m.contains("non-exhaustive match") && m.contains("FixNotAnItem")),
        "a dispatch over a closed item kind that omits one variant must be refused, and the \
         refusal must name the omitted variant; got: {messages:?}"
    );
}

#[test]
fn dispatch_covering_every_item_kind_is_accepted() {
    let messages = error_diag_messages(dispatch_fixture(ALL_ARMS));
    assert!(
        messages.is_empty(),
        "the seven-arm control must compile clean, or the RED above proves nothing about \
         exhaustiveness; got: {messages:?}"
    );
}
