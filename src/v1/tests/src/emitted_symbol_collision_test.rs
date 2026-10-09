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

fn git_worktree_type_and_colliding_service() -> &'static str {
    "module esc.probe\n\
     type GitWorktree { n: Int }\n\
     service git.Worktree {\n\
       operation Add {\n\
         input { n: Int }\n\
         output { success: Bool from \"exit_success\" }\n\
         transport shell { argv: [\"true\"] }\n\
       }\n\
     }\n"
}

fn a_type_authored_into_the_dotted_service_space() -> &'static str {
    "module esc.probe\n\
     type Git__Worktree { n: Int }\n\
     service git.Worktree {\n\
       operation Add {\n\
         input { n: Int }\n\
         output { success: Bool from \"exit_success\" }\n\
         transport shell { argv: [\"true\"] }\n\
       }\n\
     }\n"
}

fn emitted_text(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> String {
    result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn collision_count(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> usize {
    result
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                *d.diagnostic,
                CompilerDiagnostic::EmittedSymbolCollision { .. }
            )
        })
        .count()
}

/// The live `extdeps.git` shape: the dotted service keeps its segment boundary, so the type and the
/// service emit two structs and nothing refuses.
#[test]
fn type_gitworktree_and_service_git_worktree_emit_two_distinct_structs() {
    let green = compile("esc/probe.dag", git_worktree_type_and_colliding_service());
    assert_eq!(collision_count(&green), 0, "{:?}", green.diagnostics);
    assert!(green.diagnostics.is_empty(), "{:?}", green.diagnostics);
    let joined = emitted_text(&green);
    assert_eq!(
        joined.matches("pub struct GitWorktree ").count(),
        1,
        "{joined}"
    );
    assert_eq!(
        joined.matches("pub struct Git__Worktree ").count(),
        1,
        "{joined}"
    );
    assert_eq!(
        joined
            .matches("#[allow(non_camel_case_types)]\npub struct Git__Worktree ")
            .count(),
        1,
        "the `__` symbol carries its lint allowance at the definition: {joined}"
    );
}

/// The authored residue still refuses: a type spelled with `__` lands in the dotted-service space,
/// and the wall -- not rustc's E0428 -- is what observes it.
#[test]
fn a_type_spelled_into_the_service_space_still_refuses_rather_than_e0428() {
    let red = compile(
        "esc/probe.dag",
        a_type_authored_into_the_dotted_service_space(),
    );
    assert_eq!(collision_count(&red), 1, "{:?}", red.diagnostics);
    assert!(
        red.files.is_empty(),
        "a colliding pair must not emit a crate rustc would refuse with E0428: {} files",
        red.files.len()
    );
}
