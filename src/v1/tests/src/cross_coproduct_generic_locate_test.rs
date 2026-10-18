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
fn same_arm_bare_head_and_its_application_refuses() {
    // INVERTED FROM AN ACCEPT LOCK, per stern-bat-401 (msg_ad1a90c2, 2026-10-18): "a test asserting
    // acceptance that FLIPS when #14256 lands becomes a red that asserts obsolete behaviour. Don't keep
    // it as a lock. Once #14256 is on your base, invert it to assert REFUSAL (matching the current
    // authority) or delete it."
    //
    // It was green on the pre-S0 base and went red the moment S0 landed on main as cca99de010f: its last
    // mixed application-vs-non-application acceptance is deleted at 740eac5d64, so 'Outcome' (bare head)
    // and 'Outcome<Int>' (applied) are now two spellings that node_type_equals_core refuses. Measured at
    // config A of the A/B at this base (suite 27 passed / 3 failed), which is what makes this an
    // OBSOLETENESS record rather than an assertion about my fold: the refusal comes from the type
    // authority, NOT from the arms this PR adds. If anyone restores a head-vs-application acceptance at
    // the authority, this row goes red and says so.
    let msgs = blocking_messages(
        "module locate.head\ntype Outcome<T>\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe(o: Outcome, oi: Outcome<Int>) -> Bool { same_arm(a: o, b: oi) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "a bare carrier head against an application of the same carrier must refuse at the current \
         authority (#14256 @ 740eac5d64 deleted the mixed-spelling acceptance): {msgs:?}"
    );
}

#[test]
fn same_arm_alias_and_its_applied_expansion_refuses() {
    // INVERTED FROM AN ACCEPT LOCK for the same reason, and by the same ruling (msg_ad1a90c2): green on
    // the pre-S0 base, red from cca99de010f on.
    //
    // Coproduct(FreeMonoid) vs FreeMonoid<X> class, spelled as the corpus spells it: an alias whose
    // right-hand side is APPLIED. The equality's alias arm unfolds only an UNAPPLIED spelling, so the
    // alias never reaches a comparable pair (measured: 'Node(QualifiedName<Symbol>)' vs
    // 'UnresolvedDecl(FreeMonoid<Symbol>)'), and #14256 does not rely on alias peeling either. So this
    // row refuses at the current authority; it is NOT evidence about my fold, and the fold-decl control
    // (fold_replaces_a_binding_that_is_the_generic_declarations_own_form) is where acceptance is carried.
    // Its twin same_arm_alias_and_foreign_expansion_still_refuses covers the different-expansion case.
    let msgs = blocking_messages(
        "module locate.alias\ntype Symbol\ntype QualifiedName = FreeMonoid<Symbol>\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe(q: QualifiedName, f: FreeMonoid<Symbol>) -> Bool { same_arm(a: q, b: f) }\n",
    );
    assert!(
        has_generic_rebind(&msgs),
        "an alias against its own APPLIED expansion must refuse: the equality's alias arm unfolds only an \
         unapplied spelling, and #14256 does not rely on alias peeling: {msgs:?}"
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
use v1_compiler::v1_compiler_infer_types::{node_type_shape, TextJudgment, TextNotAskedReason};
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

/// The SAME applied spelling under the SUM connective, which is how the corpus prints it:
/// node_type_shape (src/v1/04_types.dag:948) emits 'Coproduct(' n_name ')' when
/// connective == Disj -- and the CHILDREN ARE NEVER PRINTED. So a Disj node here carries
/// arguments the printed shape hides, which is exactly the pair no corpus census can see.
fn fold_disj(head: &str, args: &[Rc<Node>]) -> Rc<Node> {
    let mut n = (*fold_app(head, args)).clone();
    n.connective = Connective::Disj;
    // Rc::new, not a bare n: the three earlier dispatches whose suite block printed NOTHING died
    // here, and my own grep ('^test |test result|^error') hid it twice before I looked at the raw
    // log. A test filter that cannot print a compile error reports a clean empty pass.
    Rc::new(n)
}

/// THE FAILING TEST PR #14256'S ARM NEEDS, at the grain only a source-level control can reach.
///
/// quiet-eagle-395 asked for 'any pair the arm admits whose arguments DIFFER, a parameterized
/// unapplied side vs a different instantiation', and I said my census is STRUCTURALLY BLIND to it:
/// 'Coproduct(Box)' and 'Coproduct(Box<Bool>)' print identically, so no self-host log can tell them
/// apart. This control answers it by construction instead.
///
/// left is applied ('Box<Int>': NoConnective, children); right is a Disj 'Box' that ALSO carries
/// children -- <Bool>, deliberately different from the left's <Int>. PR #14256's arm (commit
/// 92e2b987796, node_type_equals_core) fires on exactly this shape, (left_applied && (right_leaf ||
/// right_struct)), and answers by HEAD NAME ONLY -- so Box<Int> equals Box<Bool> and a genuine
/// misbinding is accepted. The sibling arm a few lines below in the SAME function does the opposite
/// on the same question: it requires type_heads_agree, which needs a resolved DeclarationRef on
/// BOTH applied heads -- its own commit message is 'Require DeclarationRef on both applied heads;
/// missing ref is unequal' (3a8c436a3aa). So the new arm is weaker than the arm next to it, which is
/// the printed-name class review 80660 named. Measured: GREEN without the arm, RED with it.
#[test]
fn fold_refuses_disj_and_applied_spellings_of_one_name_with_different_arguments() {
    let bound = fold_app("Box", &[fold_leaf("Int")]);
    let actual = fold_disj("Box", &[fold_leaf("Bool")]);
    let acc = fold_step(bound, actual);
    assert!(
        !acc.diagnostics.is_empty(),
        "Box<Int> applied vs a Disj Box<Bool> differs in its arguments; name-only equality accepts it and hides a real misbinding; got no diagnostics"
    );
}

/// THE INVERSION OF THE ROW REMOVED BELOW, meaningful only on a base that has S0 (#14256, main
/// cca99de010f). `Box<Int>` (applied spelling) vs a Disj `Box<Int>` used to be ADMITTED by
/// node_type_equals_core's mixed application-vs-non-application arm; S0 deleted that arm, and on this
/// base the pair REFUSES. Verified against the authority itself, not against a wish: `awk` over
/// node_type_equals_core in src/v1/04_types.dag at aed6e729e86 shows no mixed-spelling acceptance arm
/// left (the surviving arms split leaf / applied / struct and require argument agreement).
///
/// WHY THIS IS NOT THE DELETED ROW RE-LABELLED. The row below asserted ACCEPTANCE and would have been a
/// red asserting obsolete behaviour the moment S0 landed, which is what stern-bat-401 ruled out
/// (msg_ad1a90c2); the opposite assertion is only sound where the authority actually refuses, so it is
/// added HERE, on a base that includes S0, and not before. The BEFORE half is measured (at the pre-S0
/// relation head 835cbd38e1e this pair passed the fold with no diagnostics -- that green row is the one
/// deleted below). The AFTER half is asserted by this test and MEASURED BY THE RUN AT THIS HEAD, not
/// claimed here: if the fold still admits it, this test fails and that failure is reported, not deleted.
///
/// It is also the refusal twin of the accept control below, at relation grain rather than fold grain:
/// the accept control drives the same shape through Node.params and MY arm, so the two do not overlap.
#[test]
fn fold_refuses_an_applied_spelling_against_a_non_application_spelling_of_one_name() {
    let bound = fold_app("Box", &[fold_leaf("Int")]);
    let actual = fold_disj("Box", &[fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        !acc.diagnostics.is_empty(),
        "since S0 (#14256) deleted the mixed application-vs-non-application acceptance, Box<Int> vs a Disj \
         Box<Int> must REFUSE in the fold; if this goes green the relation grew a mixed arm back: {:?}",
        acc.diagnostics
            .iter()
            .map(|d| format!("{:?}", d))
            .collect::<Vec<String>>()
    );
}

// ROW REMOVED, deliberately: it asserted that `Box<Int>` (applied spelling) vs a Disj `Box<Int>` is
// ADMITTED -- one type in two printings. stern-bat-401 ruled (msg_be91625a, msg_ad1a90c2) that #14256 @
// 740eac5d64 deletes the last mixed application-vs-non-application acceptance, so an accept row for it
// would be a red asserting obsolete behaviour once that lands; inverting it here would have asserted a
// REFUSAL that the authority on THIS base still does not make. Either way it pins behaviour owned by the
// peer's type authority, which is not mine to lock. The refusal side is still covered by the row above
// (Box<Int> vs Disj Box<Bool>, green before AND after), and the accept duty for these rows now sits in
// fold_replaces_a_binding_that_is_the_generic_declarations_own_form, which drives MY arm via Node.params
// instead of depending on the relation. Do not re-add it, and do not loosen the relation to satisfy it.

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

/// RED-BEFORE CONTROL (measured FAILED on the SHIPPED seed, GREEN once the predicate is
/// carried by the seed). The stored binding carries a TYPE-VARIABLE leaf, so it is
/// unsettled evidence and must be REPLACED by the settled actual.
///
/// Measurement, on the shipped mirrors (v1_compiler_infer.rs sha256 5e9c37585fe088ac,
/// UnifyGenericsAcc count 36 -- i.e. the fold WITHOUT the new predicate), with this exact
/// test file: 3 passed / 1 failed, the failure being THIS row --
///   assertion `left == right` failed: an unsettled stored binding must be REPLACED by
///   the first settled actual
///     left:  "Map(Primitive(),Primitive(Int))"   (the variable-bearing binding, kept)
///     right: "Map(Primitive(String),Primitive(Int))"
/// So on main the fold KEEPS an unsettled binding and lets a settled actual go unrecorded;
/// with the predicate it replaces it. Two honesty rungs found on the way here:
///  (a) asserting only `diagnostics.is_empty()` is VACUOUS -- v1.compiler.types
///      node_type_compatible (src/v1/04_types.dag) answers true whenever either side's
///      inferred is a TypeVariable, so the pre-predicate path emitted no diagnostic
///      either. The assertion therefore has to be about WHICH node is stored.
///  (b) the variable-bearing leaf PRINTS as `Primitive()` (empty name), so the two
///      candidates are distinguishable only when one side is a variable; a fixture of two
///      settled nodes that compare equal prints equal, and no accept control can be built
///      that way. That is why the mirror row below is labelled a lock, not a control.
#[test]
fn fold_replaces_a_binding_that_still_carries_a_type_variable_leaf() {
    let bound = fold_app("Map", &[fold_tv_leaf("C"), fold_leaf("Int")]);
    let actual = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        acc.diagnostics.is_empty(),
        "a binding whose element is still a variable is unsettled; refusing it against a settled Map(String, Int) is a refusal about an unsettled variable: {:?}",
        acc.diagnostics
    );
    let kept = acc
        .subst
        .get("T")
        .cloned()
        .expect("T must still be bound after the step");
    assert_eq!(
        node_type_shape(kept, Rc::new(im::HashMap::new())),
        "Map(Primitive(String),Primitive(Int))",
        "an unsettled stored binding must be REPLACED by the first settled actual"
    );
}

/// ACCEPT: the mirror arm -- the stored binding is settled and the ACTUAL still
/// carries a variable, so the actual cannot falsify the binding and the binding is
/// kept. Same rung caveat as the row above: the no-diagnostic half was measured green
/// on the shipped seed (node_type_compatible answers true for a type variable), so the
/// discriminating assertion is the substitution content, RED before the predicate (the
/// settled Map(String, Int) would be OVERWRITTEN by the variable-bearing actual) and
/// GREEN after it (the settled binding is kept).
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
    let kept = acc
        .subst
        .get("T")
        .cloned()
        .expect("T must still be bound after the step");
    assert_eq!(
        node_type_shape(kept, Rc::new(im::HashMap::new())),
        "Map(Primitive(String),Primitive(Int))",
        "the settled binding must survive; a variable-bearing actual may not overwrite it"
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

/// An applied node that DECLARES parameters. Without this the arm under test is never
/// even considered -- `binding_is_generic_declaration_own_form` returns false on the
/// `(n.params |> count) == 0` guard before looking at anything else, so a fixture built
/// with fold_app would pass the accept row for the wrong reason and the twin would test
/// nothing. `params` is the field v1 and v2 never printed; the whole predicate reads it.
fn fold_decl(head: &str, params: &[&str], args: &[Rc<Node>]) -> Rc<Node> {
    let mut n = (*fold_app(head, args)).clone();
    n.params = Rc::new(im::Vector::from(
        params.iter().map(|p| fold_leaf(p)).collect::<Vec<_>>(),
    ));
    Rc::new(n)
}

/// ACCEPT CONTROL for 'the stored binding IS the generic declaration's own form'.
/// `Map` declares parameter A and still carries A as an element, so the binding was
/// never settled for this call -- A could denote exactly Map(String, Int). The arm must
/// REPLACE it with the settled actual.
///
/// CONSTRUCTED, not a corpus row: the corpus evidence is the rebind census (the 142
/// refusal rows whose bound side has its own params among its leaf names), reported by
/// identity in the PR body. This fixture establishes the MECHANISM; the census
/// establishes that the mechanism fires on real trees. Pre-arm RED / post-arm GREEN are
/// both measured on the emitted seed, because an earlier arm in unify_generics could
/// else admit this pair and leave the new arm untested while the test read green.
#[test]
fn fold_replaces_a_binding_that_is_the_generic_declarations_own_form() {
    let bound = fold_decl("Map", &["A"], &[fold_leaf("A"), fold_leaf("Int")]);
    let actual = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        acc.diagnostics.is_empty(),
        "a binding still carrying its own declaration's parameter is unsettled evidence, not a settled type: {:?}",
        acc.diagnostics
    );
    let kept = acc
        .subst
        .get("T")
        .cloned()
        .expect("T must still be bound after the step");
    assert_eq!(
        node_type_shape(kept, Rc::new(im::HashMap::new())),
        "Map(Primitive(String),Primitive(Int))",
        "the declaration's own form must be REPLACED by the first settled actual"
    );
}

/// ITS REFUSAL TWIN. Same head, same arity, the bound side DECLARES a parameter -- but
/// no leaf is that parameter, so the binding is settled and two distinct settled types
/// must still refuse. Without this row the accept above would be satisfied by an
/// implementation that admits anything merely because it declares parameters.
///
/// The census backs this at scale: of the 254 refusals, 69 are same-head/both-applied/
/// different-shape pairs and ZERO of them answer true to the predicate, so none of them
/// stops refusing. Measured, not asserted.
#[test]
fn fold_refuses_a_declared_parameter_type_bound_to_a_distinct_settled_instance() {
    let bound = fold_decl("Map", &["A"], &[fold_leaf("Int"), fold_leaf("Int")]);
    let actual = fold_app("Map", &[fold_leaf("String"), fold_leaf("Int")]);
    let acc = fold_step(bound, actual);
    assert!(
        !acc.diagnostics.is_empty(),
        "Map(Int, Int) then Map(String, Int) must refuse even when the bound node declares a parameter it does not use: {:?}",
        acc.diagnostics
    );
}
