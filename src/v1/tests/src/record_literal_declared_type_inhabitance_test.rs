use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::CompilerDiagnostic;

fn compile(path: &str, content: &str) -> Rc<v1_compiler::v1_compiler_compile::PipelineResult> {
    compile_sources(
        Rc::new(im::vector![Rc::new(SourceFile {
            path: path.to_string(),
            content: content.to_string(),
        })]),
        RenderTarget::Rust,
    )
}

#[test]
fn concrete_record_literal_identity_is_checked_at_declared_positions() {
    let red = compile(
        "record_red.dag",
        "module record_red\n\
         type ZzA { x: Int }\n\
         type ZzB { y: Int }\n\
         type ZzAlias = ZzA\n\
         fn wrong_return() -> ZzA { ZzB { y: 1 } }\n\
         data wrong_data: ZzA = ZzB { y: 2 }\n\
         fn wrong_let() -> Int { let v: ZzA = ZzB { y: 3 } 1 }\n\
         fn wrong_alias() -> ZzAlias { ZzB { y: 4 } }\n",
    );
    let mismatches: Vec<_> = red
        .diagnostics
        .iter()
        .filter(|d| matches!(*d.diagnostic, CompilerDiagnostic::TypeMismatch { .. }))
        .collect();
    assert_eq!(
        mismatches.len(),
        4,
        "each wrong return/data/let/alias record literal must refuse: {:?}",
        red.diagnostics
    );

    let green = compile(
        "record_green.dag",
        "module record_green\n\
         type ZzA { x: Int }\n\
         type ZzAlias = ZzA\n\
         fn right_return() -> ZzA { ZzA { x: 1 } }\n\
         data right_data: ZzA = ZzA { x: 2 }\n\
         fn right_let() -> Int { let v: ZzA = ZzA { x: 3 } 1 }\n\
         fn right_alias() -> ZzAlias { ZzA { x: 4 } }\n",
    );
    assert!(
        green.diagnostics.is_empty(),
        "matching literals prove inference reached every position without a false refusal: {:?}",
        green.diagnostics
    );
}

#[test]
fn generic_record_application_at_coproduct_payload_formal_is_refused() {
    let red = compile(
        "wrap_at_payload.dag",
        "module wrap_at_payload\n\
         type Payload = Observed { n: Int } | Unobserved { reason: String }\n\
         type Wrap<T> { result: T, close: Int }\n\
         fn takes_payload(x: Payload) -> Int { match x { Observed { n } => n Unobserved { reason: _ } => 0 } }\n\
         fn make_wrap() -> Wrap<Payload> { Wrap { result: Observed { n: 1 }, close: 0 } }\n\
         fn probe() -> Int { takes_payload(x: make_wrap()) }\n",
    );
    let refusals: Vec<_> = red
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                *d.diagnostic,
                CompilerDiagnostic::DeclaredTypeNotInhabited { .. }
            )
        })
        .collect();
    assert!(
        !refusals.is_empty(),
        "Wrap<Payload> at a Payload formal must refuse inhabitance, got: {:?}",
        red.diagnostics
    );

    let green = compile(
        "wrap_payload_field.dag",
        "module wrap_payload_field\n\
         type Payload = Observed { n: Int } | Unobserved { reason: String }\n\
         type Wrap<T> { result: T, close: Int }\n\
         fn takes_payload(x: Payload) -> Int { match x { Observed { n } => n Unobserved { reason: _ } => 0 } }\n\
         fn make_wrap() -> Wrap<Payload> { Wrap { result: Observed { n: 1 }, close: 0 } }\n\
         fn probe() -> Int { takes_payload(x: make_wrap().result) }\n",
    );
    let green_refusals: Vec<_> = green
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                *d.diagnostic,
                CompilerDiagnostic::DeclaredTypeNotInhabited { .. }
            )
        })
        .collect();
    assert!(
        green_refusals.is_empty(),
        "the payload field at that formal must stay clean, got: {:?}",
        green.diagnostics
    );
}
