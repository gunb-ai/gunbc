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
    // Matched on the fold's own message SHAPE, not on a variable letter: the text is
    // `type variable <bind_name> bound as <shape> (first argument)` vs `<shape> (later
    // argument)` (v1.compiler.infer unify_generics, rebind-branch expected/got construction). Hardcoding "T" makes every row over a
    // differently-named generic vacuously green -- which is how the Box<Int> vs
    // Box<Bool> row read as a failure on both seeds while the fold was in fact refusing
    // it (measured at 2751b200: the message was present, under the name A).
    msgs.iter().any(|m| {
        m.contains("bound as") && m.contains("(first argument)") && m.contains("(later argument)")
    })
}

#[test]
fn rfm_row_parses() {
    assert_parses_strict(
        "dag/gunbc/recurring_failure_mode/one_type_variable_bound_to_two_distinct_coproducts.dag",
    );
}

// THE DISCRIMINATING RED. Every accept row above asserts an ABSENCE, and an
// absence-only suite passes on a compiler that never implements the rule — which is
// how the seed passed these while unify_generics bound first-wins. This asserts the
// PRESENCE of the refusal the fold emits, and names its text, so it is false before
// the fold executes and true after. Its positive control is
// same_arm_int_int_is_accepted: same generic, same arity, same call shape, and only
// the two actuals' identity differs.
#[test]
fn same_arm_two_distinct_coproducts_refuses_with_the_fold_message() {
    let msgs = blocking_messages(&same_arm_call("P", "Q"));
    assert!(
        has_generic_rebind(&msgs),
        "one T bound by P and Q must refuse with the fold's own diagnostic; got {msgs:?}"
    );
}

#[test]
fn same_arm_int_bool_refuses_with_the_fold_message() {
    let msgs = blocking_messages(
        "module locate.ib\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe() -> Bool { same_arm(a: 1, b: true) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "one T bound by Int and Bool must refuse; got {msgs:?}"
    );
}

// The nested arm: heads agree, the ARGUMENT LISTS differ. W { v: W2 { v: 1 } } and
// W { v: W2 { v: true } } are both of type W — the tag, not its payloads — so they
// are genuinely the same type and must be accepted (that row is
// same_arm_two_applied_box_int_is_accepted's shape, one level up). What separates the
// nested arm from the head arm is a type applied to different arguments: Box<Int>
// against Box<Bool>. Head agrees, so first-wins accepted it; the fold's recursion has
// to compare the argument lists.
#[test]
fn applied_constructor_same_head_differing_argument_list_refuses() {
    let msgs = blocking_messages(
        "module locate.arg\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Bool>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "one A bound by Box<Int> and Box<Bool> must refuse on the argument list; got {msgs:?}"
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
fn same_arm_two_outcome_accepted_ints_is_accepted() {
    let msgs = blocking_messages(
        "module locate.out\ntype Outcome<A> = Accepted { value: A } | Rejected { d: String }\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe() -> Bool { same_arm(a: Accepted { value: 1 }, b: Accepted { value: 2 }) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "two Accepted<Int> same_arm must not refuse: {msgs:?}"
    );
}

#[test]
fn same_arm_two_optional_int_params_is_accepted() {
    let msgs = blocking_messages(
        "module locate.opt\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Int?, y: Int?) -> Bool { same_arm(a: x, b: y) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "two Int? same_arm must not refuse: {msgs:?}"
    );
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
fn same_arm_two_list_int_params_is_accepted() {
    let msgs = blocking_messages(
        "module locate.lst\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: List<Int>, y: List<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "two List<Int> same_arm must not refuse: {msgs:?}"
    );
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

// ---------------------------------------------------------------------------
// ACCEPTANCE BAR for the unification relation (v1.compiler.infer generic_types_unify),
// parent ruling msg_20536b59: the fold must ACCEPT what node_type_equals refuses for
// REPRESENTATION reasons, and keep refusing what it refuses for IDENTITY reasons. Each
// accept control is paired with the nearest refusal twin, so a relation that loosened
// past unifiability fails here rather than in the field.
// ---------------------------------------------------------------------------

#[test]
fn same_arm_string_and_free_monoid_char_is_accepted() {
    // DESIGN section 4: host text and the Char sequence are ONE type on two axes. The old
    // fold refused this pair (measured, /tmp/sg.log: 'Primitive(String)' vs
    // 'UnresolvedDecl(FreeMonoid<Primitive(Char)>)' -- the 220-diagnostic CI failure class).
    let msgs = blocking_messages(
        "module locate.txt\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe(s: String, c: FreeMonoid<Char>) -> Bool { same_arm(a: s, b: c) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "String vs FreeMonoid<Char> are one type on two representation axes; the fold must not rebind-refuse: {msgs:?}"
    );
}

#[test]
fn same_arm_string_and_free_monoid_int_still_refuses() {
    // The twin: an Int sequence is NOT text. If the text-pair arm ever matched by carrier
    // name alone, this row is what catches it.
    let msgs = blocking_messages(
        "module locate.txtneg\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe(s: String, c: FreeMonoid<Int>) -> Bool { same_arm(a: s, b: c) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "String vs FreeMonoid<Int> must still refuse -- an Int sequence is not text; got {msgs:?}"
    );
}

#[test]
fn same_arm_bare_head_and_its_application_is_accepted() {
    // The CI class: 'Outcome' (unapplied head) against 'Outcome<Int>' (the same carrier,
    // applied). node_type_equals has no arm relating head to application (measured).
    let msgs = blocking_messages(
        "module locate.head\ntype Outcome<T>\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe(o: Outcome, oi: Outcome<Int>) -> Bool { same_arm(a: o, b: oi) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "a bare carrier head must unify with an application of the same carrier: {msgs:?}"
    );
}

#[test]
fn same_arm_alias_and_its_applied_expansion_is_accepted() {
    // Coproduct(FreeMonoid) vs FreeMonoid<X> class, spelled as the corpus spells it: an
    // alias whose right-hand side is APPLIED. The equality's alias arm unfolds only an
    // UNAPPLIED spelling, so the alias never reached a comparable pair (measured:
    // 'Node(QualifiedName<Symbol>)' vs 'UnresolvedDecl(FreeMonoid<Symbol>)').
    let msgs = blocking_messages(
        "module locate.alias\ntype Symbol\ntype QualifiedName = FreeMonoid<Symbol>\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe(q: QualifiedName, f: FreeMonoid<Symbol>) -> Bool { same_arm(a: q, b: f) }\n",
    );
    assert!(
        !has_generic_rebind(&msgs),
        "an alias must unify with its own applied expansion: {msgs:?}"
    );
}

#[test]
fn same_arm_alias_and_foreign_expansion_still_refuses() {
    // The twin of the alias arm: QualifiedName expands to FreeMonoid<Symbol>, so it must
    // keep refusing a DIFFERENT expansion of the same carrier.
    let msgs = blocking_messages(
        "module locate.aliasneg\ntype Symbol\ntype Other\ntype QualifiedName = FreeMonoid<Symbol>\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe(q: QualifiedName, f: FreeMonoid<Other>) -> Bool { same_arm(a: q, b: f) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "FreeMonoid<Symbol> vs FreeMonoid<Other> must still refuse after the alias unfolds: {msgs:?}"
    );
}
