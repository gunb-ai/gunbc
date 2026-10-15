use crate::helpers::{compile_dag, compile_multi, diagnostic_messages};
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn blocking_of(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn has_rebind(msgs: &[String]) -> bool {
    msgs.iter().any(|m| {
        m.contains("type variable") && m.contains("bound as") && m.contains("later argument")
    })
}

#[test]
fn same_applied_box_int_is_accepted() {
    let result = compile_dag(
        "module locate.boxeq\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(
        !has_rebind(&msgs),
        "two Box<Int> must compare equal: {msgs:?}"
    );
}

#[test]
fn applied_box_int_vs_bool_is_refused() {
    let result = compile_dag(
        "module locate.boxne\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Bool>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(
        has_rebind(&msgs),
        "Box<Int> vs Box<Bool> must refuse: {msgs:?} all={:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn list_int_equals_freemonoid_int() {
    let result = compile_dag(
        "module locate.listalias\nimport std.types { List }\nimport std.algebra { FreeMonoid }\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: List<Int>, y: FreeMonoid<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(!has_rebind(&msgs), "List<Int> is FreeMonoid<Int>: {msgs:?}");
}

#[test]
fn bare_box_vs_qualified_other_decl_is_refused() {
    let result = compile_multi(&[
        ("q/boxmod.dag", "module q.boxmod\ntype Box<T>\n"),
        (
            "locate/bareq.dag",
            "module locate.bareq\nimport q.boxmod\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: q.boxmod.Box<Int>) -> Bool { same_arm(a: x, b: y) }\n",
        ),
    ]);
    let msgs = blocking_of(&result);
    assert!(
        has_rebind(&msgs)
            || msgs.iter().any(|m| m.contains("UnresolvedDecl")
                || m.contains("TypeMismatch")
                || m.contains("type mismatch")),
        "bare Box<Int> vs q.boxmod.Box<Int> must not equal by last segment: blocking={msgs:?} all={:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn two_decls_same_leaf_applied_compare_unequal() {
    let result = compile_multi(&[
        ("p/boxmod.dag", "module p.boxmod\ntype Box<T>\n"),
        ("q/boxmod.dag", "module q.boxmod\ntype Box<T>\n"),
        (
            "locate/hom.dag",
            "module locate.hom\nimport p.boxmod\nimport q.boxmod\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: p.boxmod.Box<Int>, y: q.boxmod.Box<Int>) -> Bool { same_arm(a: x, b: y) }\n",
        ),
    ]);
    let msgs = blocking_of(&result);
    assert!(
        has_rebind(&msgs)
            || msgs.iter().any(|m| m.contains("TypeMismatch") || m.contains("type mismatch")),
        "p.boxmod.Box<Int> vs q.boxmod.Box<Int> must not compare equal by leaf name: blocking={msgs:?} all={:?}",
        diagnostic_messages(&result)
    );
}
