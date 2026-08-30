// A QUALIFIED CALLEE IN PIPE POSITION DENOTES THAT CALLEE, NOT A METHOD NAMED BY ITS ROOT SEGMENT.
//
// WHAT WAS WRONG. `parse_pipe_rhs` consumed exactly one identifier after `|>`, so `xs |> a.b.f()`
// parsed as `xs |> a` (a method call named `a`) with `.b.f(...)` folded on top as field access and
// a call. The refusal named the ROOT SEGMENT as a missing method on the receiver's type ("method
// 'a' not found on receiver Container(List, Primitive(Int))"), pointing at neither the callee nor
// the defect. The same callee in call position -- `a.b.f(xs: xs)` -- resolved and still does: one
// denotation, two answers, decided by syntactic position.
//
// WHY THE PAIR IS THE TEST. Each half alone is satisfiable by the wrong compiler: the direct-call
// case passed before the fix and proves nothing alone; the pipe case is the discriminating input,
// red on the pre-fix parser. Together they say the two positions agree -- the actual claim.
//
// THE THIRD CASE IS THE NON-REGRESSION HALF: the fix changes the node a pipe produces, so a bare
// `|> count` -- kernel method dispatch, the shape dag/std relies on throughout -- must still lower
// to a method call on the receiver.
//
// HERMETIC BY CONSTRUCTION. Every case hands `compile_sources` in-memory sources, so this test
// reads no repository file and needs no module index. That is why it can be a test at all: the
// `compile_dag_*` builtins backing the `.dag` witness family resolve against
// `build_module_path_index_from_witness_roots`, which walks the live tree, so a witness on them
// declares `ReadsLiveTree` and is declined by the required floor rather than executed.

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
/// Returns messages rather than a bool so a failing case names what the compiler said: a count
/// assertion reports "1 != 0" for a defect and an unrelated typo alike.
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
    compile_sources(Rc::new(files.into()), RenderTarget::Rust)
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

/// A dotted pipe callee with NO application is a method followed by field access; this case
/// decides the rule: `v1.compiler.emit_rust` writes `field_names |> first.value`, so a rule that
/// consumed every dotted segment after the pipe callee would re-denote a live, correct site as a
/// call into a module named `first`.
#[test]
fn dotted_pipe_callee_without_application_is_method_then_field() {
    let messages = diagnostics_for_consumer(
        "module consumer\n\ntype Row {\n  value: Int\n}\n\nfn method_then_field(rows: List<Row>) -> Int {\n  let r = rows |> first.value\n  r.value\n}\n",
    );
    assert!(
        messages.is_empty(),
        "a dotted pipe callee with no application must stay method-then-field: {messages:?}"
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
