//! INSTRUMENT (not a PR carrier): disposes the families the #14152 PR-2 equality widening admits, by
//! MEASUREMENT at the fold rather than by reading node_type_equals_core.
//!
//! WHY THIS EXISTS. stern-bat-401 (msg_356ca7c1) ruled that the Acc head's mismatch rows must be
//! dispositioned BY FAMILY — for each, is it (i) a true disagreement in the source, which the Acc
//! correctly surfaces, or (ii) an Acc defect, too strict — and decided "with a minimal two-arm fixture,
//! not by reading the code". The corpus cannot answer it: the majority of what the widening admits prints
//! expected X / got X IDENTICALLY, and node_type_shape never prints a Disj's children nor any node's
//! `declaration` field, so two nodes can differ only where nothing is printed. This head's own
//! cross_coproduct_generic_locate_test.rs records the same blindness ("the CHILDREN ARE NEVER PRINTED")
//! and answers it "by construction instead"; I reuse its harness.
//!
//! THE MEASUREMENT THAT SETS THE SHAPE OF THESE ROWS. The strip run showed the widening is PURELY
//! SUPPRESSIVE: refusals disappear (319 occurrences, 96 distinct) and none appear. So every row it admits
//! is a pair the relation refused, and the admission must come from one of exactly two arms, from
//! src/v1/04_types.dag's type_heads_agree:
//!   arm 1 (carrier roster)  container_template_algebra says both heads denote one algebra;
//!   arm 2 (declaration id)  both heads are applied AND both carry a DeclarationRef AND
//!                           ld.module_path == rd.module_path && ld.decl_name == rd.decl_name.
//! The arms have OPPOSITE dispositions and are separable at the fold by construction:
//!   arm 2 can only fire when both sides denote the SAME declaration -> the pre-widening refusal called
//!          one declaration two different things -> family (ii), the Acc too strict;
//!   arm 1 admits two DIFFERENT names declared to share one algebra -> declared semantic equality, not a
//!          defect in the checker.
//! And one arm is already excluded for the dominant family by reading the DATA, not the code: Map is not
//! a carrier (algebra_carriers() declares Int, Nat, Float, BooleanAlgebra, FinitelySupportedFunction,
//! FinitePowerSet, PartialFunction, PointwisePower, FreeMonoid, String), so a Map-vs-Map row cannot reach
//! arm 1 — it reaches arm 2, or nothing. d5 measures that instead of trusting the inference.
//!
//! RUNG, stated not claimed (DESIGN §4b). These are Rust unit tests. DESIGN's Building & checks says the
//! required path runs neither clippy nor the v1 unit tests, so this file is GREEN BY VACANCY in CI and is
//! an INSTRUMENT only — it must be EXECUTED, and it is not PR-2's discriminator. PR-2's shipped
//! discriminators stay in dag/test/claim per stern-bat-401's ruling (d). Every row must be run at TWO
//! heads, the strip (widening reverted) and the Acc head; a row that behaves identically at both is not
//! discriminating and is reported as such by the HEAD line it prints.

use std::rc::Rc;
use v1_compiler::std_decl_ref::decl_ref;
use v1_compiler::v1_compiler_infer::{unify_generics, UnifyGenericsAcc};
use v1_compiler::v1_compiler_infer_types::{node_type_shape, TextJudgment, TextNotAskedReason};
use v1_compiler::v1_std_core::{no_span, Cardinality, Connective, DeclarationMarker, ExprData, Node,
                              NodeOccurrenceIdentity, ParsedModuleItemKind};

fn fold_leaf(name: &str) -> Rc<Node> {
    let span = no_span();
    Rc::new(Node {
        name: name.to_string(),
        ident: None,
        span: span.clone(),
        // ident_span MUST be Some: v1.std.core authored_name_at answers node.name only on that arm and
        // returns "" on the None arm; leaving it None would exercise a different path. (Same reason as
        // the harness this row borrows.)
        ident_span: Some(span),
        children: Rc::new(im::Vector::new()),
        connective: Connective::NoConnective,
        params: Rc::new(im::Vector::new()),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(im::Vector::new()),
        body: None,
        transport: None,
        properties: Rc::new(im::Vector::new()),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: ParsedModuleItemKind::NotAModuleItem,
        declaration_marker: DeclarationMarker::Unmarked,
        declaration: None,
        expr_data: Rc::new(ExprData::NoExprData),
        occurrence_identity: Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
    })
}

fn applied(head: &str, args: &[&str]) -> Rc<Node> {
    let mut n = (*fold_leaf(head)).clone();
    n.children = Rc::new(im::Vector::from(args.iter().map(|a| fold_leaf(a)).collect::<Vec<_>>()));
    Rc::new(n)
}

/// Attach a real DeclarationRef through the seed's own constructor, so the field the widening's arm 2
/// compares (module_path, decl_name) is genuinely populated rather than approximated.
fn with_decl(n: Rc<Node>, module_path: &str, decl_name: &str) -> Rc<Node> {
    let mut m = (*n).clone();
    m.declaration = Some(decl_ref(module_path.to_string(), decl_name.to_string()));
    Rc::new(m)
}

fn shape(n: Rc<Node>) -> String {
    node_type_shape(
        n,
        Rc::new(im::HashMap::new()),
        Rc::new(TextJudgment::TextNotAsked {
            reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
        }),
    )
}

/// ANY diagnostic from the fold step is the refusal. Deliberately NOT filtered on message text: filtering
/// on a substring is how a false green is manufactured when the wording moves.
fn refused(acc: &Rc<UnifyGenericsAcc>) -> bool {
    !acc.diagnostics.is_empty()
}

/// One fold step: T is already bound to `bound`, and the call now hands T the `actual`.
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
        "fold.arm.disposition".to_string(),
    )
}

/// Which head is running, derived from the tree rather than from a label I could get wrong: the widening
/// leaves call sites in the compiled mirror, and the strip has none (its only occurrence is the surviving
/// definition). §6: derived by the instrument, not transcribed.
fn head_tag() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../stage0/src/v1_compiler_infer_types.rs");
    match std::fs::read_to_string(path) {
        Ok(t) => format!("type_heads_agree_occurrences_in_compiled_mirror={}", t.matches("type_heads_agree(").count()),
        Err(e) => format!("HEAD_UNREADABLE:{e}"),
    }
}

/// Print the pair as the Acc would print it, so a reader can see the expected X / got X form directly.
fn report(tag: &str, bound: &Rc<Node>, actual: &Rc<Node>, acc: &Rc<UnifyGenericsAcc>) -> String {
    format!(
        "{tag}\n  expected 'type variable T bound as {} (first argument)', got '{} (later argument)'\n  diagnostics={} refused={}",
        shape(bound.clone()),
        shape(actual.clone()),
        acc.diagnostics.iter().map(|d| format!("{d:?}")).collect::<Vec<_>>().join(" | "),
        refused(acc)
    )
}

// ---------------------------------------------------------------------------
// D1 — arm 2's own ground. One applied head, the SAME declaration on both sides, equal arguments.
// MUST REFUSE on the strip (main's relation has no arm 2) and MUST ACCEPT on the Acc head (arm 2 fires).
// That pair is the whole argument: same declaration, same arguments, refused by main. It is family (ii),
// the Acc too strict, and it is the single largest thing the widening buys back. If it refuses at BOTH
// heads the row is inert and I say so rather than keeping it as coverage.
// ---------------------------------------------------------------------------
#[test]
fn d1_one_declaration_two_printings_of_it() {
    let left = with_decl(applied("DemandEntry", &["Int"]), "v2.compiler.target_model", "DemandEntry");
    let right = with_decl(applied("DemandEntry", &["Int"]), "v2.compiler.target_model", "DemandEntry");
    assert_eq!(shape(left.clone()), shape(right.clone()), "the fixture's sides must print identically, the way the corpus rows do; got {:?} vs {:?}", shape(left.clone()), shape(right.clone()));
    let acc = fold_step(left.clone(), right.clone());
    let out = report(&head_tag(), &left, &right, &acc);
    println!("D1 {out}");
    assert!(
        !refused(&acc),
        "D1: ONE declaration applied to the SAME arguments is one type. If this refuses at a head that has \
         the widening, arm 2 is not reachable from the fold and the corpus rows attributed to it belong to \
         someone else. If it refuses at main, that refusal is the over-refusal the widening exists to fix. \
         {out}"
    );
}

// ---------------------------------------------------------------------------
// D2 — the over-admission guard for arm 2: one spelling, TWO declarations. Whatever the corpus needs,
// the widening must refuse here; admitting it is the silent wrong binding §5 treats as unforgivable, and
// it is precisely the risk stern-bat-401 named for a widening (msg_37357ae1). Must refuse at BOTH heads.
// ---------------------------------------------------------------------------
#[test]
fn d2_one_spelling_two_declarations_must_refuse() {
    let left = with_decl(applied("DemandEntry", &["Int"]), "v2.compiler.target_model", "DemandEntry");
    let right = with_decl(applied("DemandEntry", &["Int"]), "v2.compiler.somewhere_else", "DemandEntry");
    assert_eq!(shape(left.clone()), shape(right.clone()), "same printed shape by construction: only the declaration differs, which is what the renderer cannot show");
    let acc = fold_step(left.clone(), right.clone());
    let out = report(&head_tag(), &left, &right, &acc);
    println!("D2 {out}");
    assert!(refused(&acc), "D2: two distinct declarations under one spelling are two types; a widening that admits them is silently wrong. {out}");
}

// ---------------------------------------------------------------------------
// D3 — no DeclarationRef on either side. Arm 2 answers false on Absent, so this must REFUSE at both
// heads. It is the guard against the other direction: green here would mean unqualified names are equal
// by spelling alone, the printed-name class review 80660 named.
// ---------------------------------------------------------------------------
#[test]
fn d3_no_declaration_on_either_side_must_refuse() {
    let left = applied("DemandEntry", &["Int"]);
    let right = applied("DemandEntry", &["Int"]);
    let acc = fold_step(left.clone(), right.clone());
    let out = report(&head_tag(), &left, &right, &acc);
    println!("D3 {out}");
    assert!(refused(&acc), "D3: with no DeclarationRef on either applied side, arm 2 must answer unequal; green here means bare spellings compare equal. {out}");
}

// ---------------------------------------------------------------------------
// D4 — arm 1 measured, not assumed. 'List<Int>' vs 'FreeMonoid<Int>' is ONE declared algebra in two
// spellings: std/algebra.dag declares List as the FreeMonoid carrier's pascal alias
// (pascal_alias_spelling(text: "List"), container_alias_row RowPresent). Expected to ACCEPT at both heads,
// because arm 1 answers by carrier and never reads declarations. Its value is the measurement: if it
// REFUSES, the carrier roster is not reachable from the fold at all, and every corpus row I attributed to
// arm 1 belongs somewhere else.
// ---------------------------------------------------------------------------
#[test]
fn d4_declared_carrier_alias_pair() {
    let left = applied("List", &["Int"]);
    let right = applied("FreeMonoid", &["Int"]);
    let acc = fold_step(left.clone(), right.clone());
    let out = report(&head_tag(), &left, &right, &acc);
    println!("D4 {out}");
    assert!(
        !refused(&acc),
        "D4: List and FreeMonoid are one declared algebra (std/algebra.dag declares List as the FreeMonoid \
         carrier's pascal alias), so the fold must not call them a conflict. If this refuses, the carrier \
         roster is unreachable from the fold and the arm-1 attribution is wrong. {out}"
    );
}

// ---------------------------------------------------------------------------
// D5 — the dominant family, disposed. Map is NOT a carrier (algebra_carriers() has Int, Nat, Float,
// BooleanAlgebra, FinitelySupportedFunction, FinitePowerSet, PartialFunction, PointwisePower, FreeMonoid,
// String), so a Map-vs-Map row cannot reach arm 1. This row measures whether an unqualified Map pair is
// refused; combined with d6 (Map WITH one declaration on both sides) it assigns the dominant family to
// arm 2 or to nothing. Reported as a disposition, and asserted only in the direction the guard requires.
// ---------------------------------------------------------------------------
#[test]
fn d5_map_pair_without_declarations() {
    let left = applied("Map", &["Symbol", "NonEmptyDiagnostics"]);
    let right = applied("Map", &["Symbol", "NonEmptyDiagnostics"]);
    let acc = fold_step(left.clone(), right.clone());
    let out = report(&head_tag(), &left, &right, &acc);
    println!("D5 {out}");
    println!(
        "D5 DISPOSITION: {}",
        if refused(&acc) {
            "REFUSED without declarations -> since Map is not a carrier, the dominant Map family's admission \
             can only have come from arm 2 (real DeclarationRefs), so those rows are one declaration in two \
             printings = family (ii), the Acc too strict."
        } else {
            "ACCEPTED without declarations -> bare unqualified Map spellings are equal by name; then the Map \
             rows were never arm 2 and the widening's carrier arm is doing the work, which is a different \
             disposition."
        }
    );
}

/// The complement of d5: the SAME Map pair, one declaration on both sides. d5-refused + d6-accepted is the
/// signature that says the widening's effect on the dominant family runs entirely through declaration
/// identity — which is the finding the family verdict needs, established by two arms rather than by a
/// reading. d5-refused + d6-REFUSED would mean the widening does not touch this family at all, and any
/// attribution of Map rows to it would be false.
#[test]
fn d6_map_pair_with_one_declaration_on_both_sides() {
    let left = with_decl(applied("Map", &["Symbol", "NonEmptyDiagnostics"]), "std.algebra", "Map");
    let right = with_decl(applied("Map", &["Symbol", "NonEmptyDiagnostics"]), "std.algebra", "Map");
    let acc = fold_step(left.clone(), right.clone());
    let out = report(&head_tag(), &left, &right, &acc);
    println!("D6 {out}");
    assert!(!refused(&acc), "D6: one declaration, equal arguments — the pair the Acc head must accept via arm 2. Refusing it is the over-refusal the widening exists to fix. {out}");
}

// ---------------------------------------------------------------------------
// D7 — the renderer blind spot, pinned as a RED so printed-shape triage can never be trusted again by
// accident. Two Disj nodes, one with children and one without, printing IDENTICALLY. This is the reason
// the corpus's expected X / got X class cannot be dispositioned from logs, and it is quiet-eagle-395's
// finding reproduced independently.
// ---------------------------------------------------------------------------
#[test]
fn d7_disj_children_are_invisible_to_the_renderer() {
    let mut plain = (*fold_leaf("Coproduct")).clone();
    plain.connective = Connective::Disj;
    let plain = Rc::new(plain);
    let mut hidden = (*fold_leaf("Coproduct")).clone();
    hidden.connective = Connective::Disj;
    hidden.children = Rc::new(im::Vector::from(vec![fold_leaf("Bool"), fold_leaf("Int")]));
    let hidden = Rc::new(hidden);
    let a = shape(plain.clone());
    let b = shape(hidden.clone());
    println!("D7 shapes=({a:?},{b:?}) head={}", head_tag());
    assert_eq!(
        a, b,
        "D7 pins the renderer blind spot that makes printed-shape triage unusable: a Disj's children must \
         not change its printed shape. If this goes GREEN (shapes differ), the renderer got fixed and the \
         triage can be re-run honestly — which is the outcome prereq (a) is aiming at."
    );
}
