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
