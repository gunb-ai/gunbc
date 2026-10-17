use crate::helpers::{assert_parses_strict, compile_dag, diagnostic_messages};
use std::rc::Rc;
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;
use v1_compiler::v1_std_core::{Connective, ExprData, Node, NodeOccurrenceIdentity};

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

// ---------------------------------------------------------------------------
// VARIABLE-CAPTURE DISCRIMINATOR (stern-bat-401 msg_d5018d11). Hypothesis under test:
// the fold keys its binding map by type-variable NAME without scoping per callee
// INSTANCE, so a nested or sibling call to a DIFFERENT generic function that also spells
// its variable T makes the two callees' variables collide -- which would explain the
// Product(Unit<>) family, where the first binding's span is the return annotation of
// fn outcome_accepted<T> (src/v2/std/diagnostic.dag:6513-6520) while the later argument's
// span is the return annotation of fn bind_outcome<T, U> (:14331-14338): two annotations
// from two different generic functions, neither a settled value expression.
// Each row below is measured, not asserted-from-hope: the two ACCEPT rows are programs the
// corpus needs to keep accepting (a capture bug would refuse them, and the refusal would
// name a variable bound from two callees); the REJECT twin is the same call shape with one
// callee's own two arguments settled to different types, which must keep refusing.
// ---------------------------------------------------------------------------

fn capture_prelude() -> &'static str {
    "module locate.capture\n\ntype P = P1 { x: Int } | P2 { y: Int }\ntype Q = Q1 { x: Int } | Q2 { y: Int }\n\n"
}

// Two DIFFERENT generic callees, both spelling their variable T, called in one
// expression. Legally typed: pair binds T := Int from both of its own arguments; consumes
// binds its own T := Bool. Nothing is shared between the two callees, so a refusal here is
// capture, not a misbinding.
#[test]
fn sibling_generic_calls_with_the_same_variable_letter_are_accepted() {
    let src = format!(
        "{}fn pair<T>(a: T, b: T) -> Bool {{ true }}\nfn consumes<T>(x: T) -> Bool {{ true }}\nfn probe(i: Int, j: Int, b: Bool) -> Bool {{ pair(a: i, b: j) && consumes(x: b) }}\n",
        capture_prelude()
    );
    let msgs = blocking_messages(&src);
    assert!(
        !has_generic_rebind(&msgs),
        "two separate callees each binding their own T must not collide; got {msgs:?}"
    );
}

// The NESTED form the corpus actually has (bind_outcome(outcome_accepted(..), ..)):
// inner callee's T settles to Int, outer callee's T settles to Int through the inner
// result and a sibling argument. Legal whatever the letters are.
#[test]
fn nested_generic_call_with_the_same_variable_letter_is_accepted() {
    let src = format!(
        "{}fn inner1<T>(x: T) -> T {{ x }}\nfn outer2<T>(a: T, b: T) -> Bool {{ true }}\nfn probe(i: Int, j: Int) -> Bool {{ outer2(a: inner1(x: i), b: j) }}\n",
        capture_prelude()
    );
    let msgs = blocking_messages(&src);
    assert!(
        !has_generic_rebind(&msgs),
        "a nested callee whose variable is spelled the same must not collide; got {msgs:?}"
    );
}

// The same nesting with genuinely DIFFERENT settled types for the two callees: inner1's T
// := Bool (legal, it only flows through inner1), outer2's T := Int from both of ITS
// arguments. Still legal, so still must accept. This is the row that separates 'keys are
// per-callee' (accepts) from 'keys are global by letter' (refuses, naming T).
#[test]
fn nested_generic_call_with_different_settled_types_per_callee_is_accepted() {
    let src = format!(
        "{}fn inner1<T>(x: T) -> T {{ x }}\nfn outer2<T>(a: T, b: T) -> Bool {{ true }}\nfn probe(i: Int, j: Int) -> Bool {{ outer2(a: inner1(x: i), b: j) && inner1(x: true) == true }}\n",
        capture_prelude()
    );
    let msgs = blocking_messages(&src);
    assert!(
        !has_generic_rebind(&msgs),
        "inner callee settling its T to Bool while outer settles T to Int is legal; got {msgs:?}"
    );
}

// REJECT TWIN: ONE callee, its OWN two arguments settled to two distinct types. Not
// capture -- a genuine misbinding -- and must keep refusing, or the three rows above prove
// nothing (an implementation that simply stopped binding would pass them all).
#[test]
fn one_callee_two_distinct_settled_arguments_still_refuses() {
    let src = format!(
        "{}fn pair<T>(a: T, b: T) -> Bool {{ true }}\nfn probe(i: Int) -> Bool {{ pair(a: i, b: true) }}\n",
        capture_prelude()
    );
    let msgs = blocking_messages(&src);
    assert!(
        has_generic_rebind(&msgs),
        "one callee binding T to Int and Bool must refuse; got {msgs:?}"
    );
}

// ---------------------------------------------------------------------------
// DIRECT-CALL CONTROLS for the unsettled-binding predicate (stern-bat-401
// msg_0306d56f part (2), predicate APPROVED). The first two controls for this
// predicate were written as SOURCE-LEVEL programs and the vacuity check on the
// SHIPPED seed caught them as unsound: the accept row passed pre-predicate (so it
// proved nothing) and the refusal twin FAILED pre-predicate (so the fold never
// reached the comparison at all on that program -- the source route, not the
// predicate, decided it). Source-level rows cannot attribute behaviour to the fold
// when the route may never reach it, so these call v1.compiler.infer
// unify_generics directly with the two nodes, where nothing but the predicate can
// change the answer. RED-before / GREEN-after is stated per row.
// ---------------------------------------------------------------------------

use v1_compiler::v1_compiler_infer::{unify_generics, UnifyGenericsAcc};
use v1_compiler::v1_compiler_infer_types::{TextJudgment, TextNotAskedReason};
use v1_compiler::v1_std_core::{no_span, InferredNode};

fn fold_leaf(name: &str) -> Rc<Node> {
    let span = no_span();
    Rc::new(Node {
        name: name.to_string(),
        ident: None,
        span: span.clone(),
        // ident_span MUST be Some: v1.std.core authored_name_at answers node.name only on
        // that arm, and its None arm returns "" -- which would make type_node_label fall
        // through to n.name anyway, but leaving it None would test a different path.
        ident_span: Some(span),
        children: Rc::new(im::Vector::new()),
        connective: Connective::NoConnective,
        params: Rc::new(im::Vector::new()),
        inferred: None,
        return_cardinality: v1_compiler::v1_std_core::Cardinality::Required,
        uses: Rc::new(im::Vector::new()),
        body: None,
        transport: None,
        properties: Rc::new(im::Vector::new()),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        declaration_marker: v1_compiler::v1_std_core::DeclarationMarker::Unmarked,
        declaration: None,
        expr_data: Rc::new(ExprData::NoExprData),
        occurrence_identity: Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
    })
}

/// A leaf that IS still a type variable: inferred = TypeVariable. This is what an
/// unsettled parameter looks like, and what the fabricated-Unit measurement replaced.
fn fold_tv_leaf(id: &str) -> Rc<Node> {
    let mut n = (*fold_leaf(id)).clone();
    n.inferred = Some(Rc::new(InferredNode::TypeVariable { id: id.to_string() }));
    Rc::new(n)
}

fn fold_app(head: &str, args: &[Rc<Node>]) -> Rc<Node> {
    let mut n = (*fold_leaf(head)).clone();
    n.children = Rc::new(im::Vector::from(args.to_vec()));
    Rc::new(n)
}

/// One fold step: variable T already bound to `bound`, now handed `actual`.
fn fold_step(bound: Rc<Node>, actual: Rc<Node>) -> Rc<UnifyGenericsAcc> {
    let si = Rc::new(im::HashMap::new());
    let mut acc: im::HashMap<String, Rc<Node>> = im::HashMap::new();
    acc.insert("T".to_string(), bound);
    unify_generics(
        fold_leaf("T"),
        actual,
        Rc::new(im::Vector::from(vec!["T".to_string()])),
        si.clone(),
        Rc::new(acc),
        Rc::new(TextJudgment::TextNotAsked {
            reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
        }),
        "locate.fold".to_string(),
    )
}

/// ACCEPT (RED on the shipped seed, GREEN once the predicate is in the seed): the
/// stored binding still carries a TYPE-VARIABLE leaf, so it is not settled evidence
/// and must be replaced by the settled actual instead of refused against it. The
/// old authority (v1.compiler.infer unify_binding_is_uninformative) follows only a
/// single-child chain, so Map(<var C>, Int) looked settled and was refused against
/// Map(String, Int) -- a refusal about a variable that could BE String.
#[test]
fn fold_replaces_a_binding_that_still_carries_a_type_variable_leaf() {
    let bound = fold_app("Map", &[fold_tv_leaf("C"), fold_leaf("Int")]);
    let actual = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        acc.diagnostics.is_empty(),
        "a binding whose element is still a variable is unsettled; refusing it against a          settled Map(String, Int) is a refusal about an unsettled variable: {:?}",
        acc.diagnostics
    );
}

/// ACCEPT (RED pre-predicate, GREEN after): the mirror arm -- the stored binding is
/// settled and the ACTUAL still carries a variable, so the actual cannot falsify the
/// binding and the binding is kept.
#[test]
fn fold_keeps_a_settled_binding_against_an_actual_that_still_carries_a_type_variable() {
    let bound = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);
    let actual = fold_app("Map", &[fold_tv_leaf("C"), fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        acc.diagnostics.is_empty(),
        "a variable-bearing actual cannot falsify a settled binding: {:?}",
        acc.diagnostics
    );
}

/// REJECT TWIN (GREEN before AND after -- sound on the shipped seed, verified):
/// identical shape, no variable anywhere, genuinely distinct settled types. This is
/// the row that makes the two accepts above non-vacuous: an implementation that
/// stopped refusing altogether would fail it.
#[test]
fn fold_refuses_two_distinct_settled_bindings_of_the_same_shape() {
    let bound = fold_app("Map", &[fold_leaf("Int"), fold_leaf("Int")]);
    let actual = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        !acc.diagnostics.is_empty(),
        "Map(Int, Int) then Map(String, Int) for one variable is a genuine misbinding and          must refuse; got no diagnostics"
    );
}

/// REJECT TWIN (GREEN before AND after): the classic scalar case, at fold grain.
#[test]
fn fold_refuses_int_bound_then_bool() {
    let acc = fold_step(fold_leaf("Int"), fold_leaf("Bool"));
    assert!(
        !acc.diagnostics.is_empty(),
        "T bound as Int then Bool must refuse; got no diagnostics"
    );
}
