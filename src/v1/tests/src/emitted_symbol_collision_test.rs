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

fn git_worktree_type_and_distinct_service() -> &'static str {
    "module esc.probe\n\
     type GitWorktree { n: Int }\n\
     service git.Other {\n\
       operation Add {\n\
         input { n: Int }\n\
         output { success: Bool from \"exit_success\" }\n\
         transport shell { argv: [\"true\"] }\n\
       }\n\
     }\n"
}

#[test]
fn type_gitworktree_and_service_git_worktree_refuse_rather_than_e0428() {
    let red = compile("esc/probe.dag", git_worktree_type_and_colliding_service());
    let collisions: Vec<_> = red
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                *d.diagnostic,
                CompilerDiagnostic::EmittedSymbolCollision { .. }
            )
        })
        .collect();
    assert_eq!(
        collisions.len(),
        1,
        "expected EmittedSymbolCollision, got {:?}",
        red.diagnostics
    );
    assert!(
        red.files.is_empty(),
        "a colliding pair must not emit a crate rustc would refuse with E0428: {} files",
        red.files.len()
    );
}

#[test]
fn distinct_emitted_symbols_still_emit_one_gitworktree_struct() {
    let green = compile("esc/probe.dag", git_worktree_type_and_distinct_service());
    assert!(
        green.diagnostics.is_empty(),
        "distinct identities must emit: {:?}",
        green.diagnostics
    );
    let joined = green
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let count = joined.matches("pub struct GitWorktree").count();
    assert_eq!(
        count, 1,
        "one type declaration must emit one struct; got {count} in {joined}"
    );
}
