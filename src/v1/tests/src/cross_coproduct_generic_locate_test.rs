use crate::helpers::{assert_parses_strict, compile_dag, diagnostic_messages};
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn blocking_messages(source: &str) -> Vec<String> {
    let result = compile_dag(source);
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn prelude() -> &'static str {
    "module locate.cross\n\ntype P = P1 { x: Int } | P2 { y: Int }\ntype Q = Q1 { x: Int } | Q2 { y: Int }\n\n"
}

fn same_arm_call(a: &str, b: &str) -> String {
    format!(
        "{}fn same_arm<T>(a: T, b: T) -> Bool {{ true }}\nfn probe() -> Bool {{ same_arm(a: {}, b: {}) }}\n",
        prelude(),
        a,
        b
    )
}

fn has_generic_rebind(msgs: &[String]) -> bool {
    msgs.iter()
        .any(|m| m.contains("type variable T bound as") && m.contains("later argument"))
}

#[test]
fn rfm_row_parses() {
    assert_parses_strict(
        "dag/gunbc/recurring_failure_mode/one_type_variable_bound_to_two_distinct_coproducts.dag",
    );
}

#[test]
fn same_arm_cross_coproduct_is_refused() {
    let msgs = blocking_messages(&same_arm_call("P1 { x: 1 }", "Q1 { x: 1 }"));
    assert!(
        has_generic_rebind(&msgs),
        "cross-coproduct same_arm must refuse a T rebind: {msgs:?}"
    );
}

#[test]
fn same_arm_int_bool_is_refused() {
    let msgs = blocking_messages(
        "module locate.ib\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe() -> Bool { same_arm(a: 1, b: true) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "Int vs Bool same_arm must refuse a T rebind: {msgs:?}"
    );
}

#[test]
fn same_arm_same_coproduct_is_accepted() {
    let msgs = blocking_messages(&same_arm_call("P1 { x: 1 }", "P2 { y: 2 }"));
    assert!(
        !has_generic_rebind(&msgs),
        "same-coproduct same_arm must not refuse a T rebind: {msgs:?}"
    );
}

#[test]
fn same_arm_int_int_is_accepted() {
    let msgs = blocking_messages(
        "module locate.ii\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe() -> Bool { same_arm(a: 1, b: 2) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "Int vs Int same_arm must not refuse a T rebind: {msgs:?}"
    );
}

#[test]
fn uninformative_then_int_then_bool_is_refused() {
    let msgs = blocking_messages(
        "module locate.tv3\nfn same_arm<T>(a: T, b: T, c: T) -> Bool { true }\nfn probe<U>(u: U) -> Bool { same_arm(a: u, b: 1, c: true) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "U then Int then Bool must refuse the later concrete rebind: {msgs:?}"
    );
}

#[test]
fn uninformative_then_int_then_int_is_accepted() {
    let msgs = blocking_messages(
        "module locate.tv3ok\nfn same_arm<T>(a: T, b: T, c: T) -> Bool { true }\nfn probe<U>(u: U) -> Bool { same_arm(a: u, b: 1, c: 2) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "U then Int then Int must accept after the uninformative completes: {msgs:?}"
    );
}

#[test]
fn same_arm_two_outcome_accepted_ints_is_observed() {
    let msgs = blocking_messages(
        "module locate.out\ntype Outcome<A> = Accepted { value: A } | Rejected { d: String }\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe() -> Bool { same_arm(a: Accepted { value: 1 }, b: Accepted { value: 2 }) }\n",
    );
    eprintln!("outcome_accepted_ints blocking={msgs:?}");
}

#[test]
fn same_arm_two_optional_int_params_is_observed() {
    let msgs = blocking_messages(
        "module locate.opt\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Int?, y: Int?) -> Bool { same_arm(a: x, b: y) }\n",
    );
    eprintln!("optional_int_params blocking={msgs:?}");
}

#[test]
fn same_arm_two_lambdas_does_not_compare_return_to_callable() {
    let msgs = blocking_messages(
        "module locate.lam\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe() -> Bool { same_arm(a: (x: Int) => x, b: (y: Int) => y) }\n",
    );
    assert!(
        !msgs
            .iter()
            .any(|m| m.contains("type variable T bound as") && m.contains("later argument")),
        "two lambdas at T must not TypeMismatch Callable vs the lambda return: {msgs:?}"
    );
}

#[test]
fn same_arm_two_applied_box_int_is_accepted() {
    let msgs = blocking_messages(
        "module locate.box\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    assert!(
        msgs.iter()
            .filter(|m| m.contains("type variable A bound as") && m.contains("later argument"))
            .count()
            == 0,
        "two Box<Int> actuals must not refuse an applied-constructor rebind: {msgs:?}"
    );
}

#[test]
fn nested_list_int_vs_bool_is_refused() {
    let msgs = blocking_messages(
        "module locate.nlist\nfn nest<T>(a: List<T>, b: List<T>) -> Bool { true }\nfn probe(x: List<Int>, y: List<Bool>) -> Bool { nest(a: x, b: y) }\n",
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("type variable T bound as") && m.contains("later argument")),
        "List<Int> vs List<Bool> at nest<T> must refuse a nested T rebind: {msgs:?}"
    );
}

#[test]
fn record_box_int_vs_bool_is_refused() {
    let msgs = blocking_messages(
        "module locate.nbox\ntype Box<T> { value: T }\nfn nest<T>(a: Box<T>, b: Box<T>) -> Bool { true }\nfn probe() -> Bool { nest(a: Box { value: 1 }, b: Box { value: true }) }\n",
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("type variable T bound as") && m.contains("later argument")),
        "Box {1} vs Box {true} must refuse a record-field T rebind: {msgs:?}"
    );
}

#[test]
fn same_arm_applied_box_int_vs_bool_is_refused() {
    let msgs = blocking_messages(
        "module locate.boxx\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Bool>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("type variable A bound as") && m.contains("later argument")),
        "Box<Int> vs Box<Bool> must refuse a T rebind: {msgs:?}"
    );
}

#[test]
fn same_arm_two_list_int_params_is_observed() {
    let msgs = blocking_messages(
        "module locate.lst\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: List<Int>, y: List<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    eprintln!("list_int_params blocking={msgs:?}");
}

#[test]
fn phantom_constructor_tag_equality_is_observed() {
    let source = format!(
        "{}type ConstructorTag<T>\nfn probe() -> Bool {{ ConstructorTag<P> {{}} == ConstructorTag<Q> {{}} }}\n",
        prelude()
    );
    let result = compile_dag(&source);
    let all = diagnostic_messages(&result);
    let blocking = blocking_messages(&source);
    eprintln!("phantom_eq blocking={blocking:?} all={all:?}");
    assert!(
        blocking.is_empty(),
        "phantom ConstructorTag<P>==ConstructorTag<Q> is observed accepted (follow-up C if this starts refusing): {blocking:?}"
    );
}
