// A QUALIFIED CALLEE IN PIPE POSITION DENOTES THAT CALLEE, NOT A METHOD NAMED BY ITS ROOT SEGMENT.
//
// WHAT WAS WRONG. `parse_pipe_rhs` consumed exactly one identifier after `|>`, so `xs |> a.b.f()`
// parsed as `xs |> a` -- a method call named `a` on the receiver -- and the remaining `.b.f(...)`
// folded on top of it as field access and a call. The refusal that reached the author therefore
// named the ROOT SEGMENT as a missing method on the receiver's type ("method 'a' not found on
// receiver Container(List, Primitive(Int))"), which points at neither the callee nor the defect.
// The same callee written in ordinary call position -- `a.b.f(xs: xs)` -- resolved and still does:
// one authored denotation, two answers, decided by which syntactic position it stood in.
//
// WHY THE PAIR IS THE TEST. Each half alone is satisfiable by the wrong compiler. The direct-call
// case passes today and passed before the fix, so it proves nothing on its own; the pipe case is
// the discriminating input, and it is red on the pre-fix parser. Held together they say the two
// positions agree, which is the actual claim -- not "the pipe case compiles".
//
// THE THIRD CASE IS THE NON-REGRESSION HALF, and it is not decoration: the fix changes the node a
// pipe produces, so a bare `|> count` -- kernel method dispatch, the shape dag/std relies on
// throughout -- must still lower to a method call on the receiver and not to anything else.
//
// HERMETIC BY CONSTRUCTION. Every case hands `compile_sources` its own in-memory sources, so this
// test reads no repository file and depends on no module index. That is why it can be a test at
// all: the `compile_dag_*` builtins that back the `.dag` witness family resolve against
// `build_module_path_index_from_witness_roots`, which walks the live tree, so a witness written on
// them declares `ReadsLiveTree` and is declined by the required floor rather than executed.

use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

/// The provider module every case resolves its qualified callee against.
fn provider_source() -> (&'static str, &'static str) {
    (
        "fx/a/b.dag",
        "module a.b\n\nfn f(xs: List<Int>) -> List<Int> {\n  xs\n}\n",
    )
}

/// Compile the given `(path, content)` sources together and return every diagnostic message.
///
/// Returns messages rather than a bool so a failing case names what the compiler said: an
/// assertion that only counts diagnostics reports "1 != 0" for a defect and for an unrelated
/// typo alike.
fn diagnostics_of(sources: &[(&str, &str)]) -> Vec<String> {
    let files: Vec<Rc<SourceFile>> = sources
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: (*path).to_string(),
                content: (*content).to_string(),
            })
        })
        .collect();
    compile_sources(Rc::new(files), RenderTarget::Rust)
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

/// Compile one consumer module beside the provider above.
fn diagnostics_for_consumer(content: &str) -> Vec<String> {
    let (provider_path, provider_content) = provider_source();
    diagnostics_of(&[
        (provider_path, provider_content),
        ("fx/consumer.dag", content),
    ])
}

#[test]
fn qualified_callee_in_call_position_resolves() {
    let messages = diagnostics_for_consumer(
        "module consumer\n\nimport a.b { f }\n\nfn direct() -> Int {\n  a.b.f(xs: [1, 0, 2]) |> count\n}\n",
    );
    assert!(
        messages.is_empty(),
        "qualified callee in ordinary call position must resolve: {messages:?}"
    );
}

#[test]
fn qualified_callee_in_pipe_position_resolves() {
    let messages = diagnostics_for_consumer(
        "module consumer\n\nimport a.b { f }\n\nfn piped() -> Int {\n  [1, 0, 2] |> a.b.f() |> count\n}\n",
    );
    assert!(
        messages.is_empty(),
        "qualified callee in pipe position must denote the same callee as in call position: {messages:?}"
    );
}

#[test]
fn bare_callee_in_pipe_position_is_unchanged() {
    let messages = diagnostics_for_consumer(
        "module consumer\n\nimport a.b { f }\n\nfn bare() -> Int {\n  [1, 0, 2] |> count\n}\n",
    );
    assert!(
        messages.is_empty(),
        "a bare pipe callee must still lower to kernel method dispatch on the receiver: {messages:?}"
    );
}
