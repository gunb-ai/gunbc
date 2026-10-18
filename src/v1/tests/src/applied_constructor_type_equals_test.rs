use crate::helpers::compile_dag;
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
fn list_int_equals_freemonoid_int() {
    let result = compile_dag(
        "module locate.listalias\nimport std.types { List }\nimport std.algebra { FreeMonoid }\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: List<Int>, y: FreeMonoid<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(!has_rebind(&msgs), "List<Int> is FreeMonoid<Int>: {msgs:?}");
}
