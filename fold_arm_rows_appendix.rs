// APPENDED ROWS — instrument only, not a PR carrier.
//
// This block is PREPENDED at run time to a slice of the head's own
// src/v1/tests/src/cross_coproduct_generic_locate_test.rs (its imports and its fold_leaf / fold_app /
// fold_disj builders), so the Node it builds is the Node the harness already compiles, and no field list
// is hand-copied here. It is NOT a copy of that file and does not replace it: that file is quiet-hawk-507's
// carrier and is untouched.
//
// PURPOSE (stern-bat-401 msg_356ca7c1): dispose the families the #14152 PR-2 equality widening admits —
// (i) a true disagreement the Acc correctly surfaces, or (ii) an Acc defect, too strict — "with a minimal
// two-arm fixture, not by reading the code". Reading is unavailable here: the majority of what the widening
// admits prints expected X / got X IDENTICALLY, because node_type_shape prints neither a Disj's children
// nor any node's declaration field (that file's own comment: "the CHILDREN ARE NEVER PRINTED"). So the
// fixtures go to the fold, where unify_generics itself produces the conflict (src/v1/04_infer.dag, the
// else arm that emits `type variable T bound as ...`), and nothing but the relation can change the answer.
//
// THE MEASUREMENT THAT SHAPES THESE ROWS. The strip run showed the widening is PURELY SUPPRESSIVE: 319
// refusal occurrences vanish and none appear. So each admitted row was refused before, and the admission
// must come from one of type_heads_agree's two arms:
//   arm 1 (carrier roster)   container_template_algebra says both heads denote one algebra;
//   arm 2 (declaration id)   both applied AND both carry a DeclarationRef AND module_path == module_path
//                            AND decl_name == decl_name.
// They have opposite dispositions, separable at the fold by construction: arm 2 fires only when both sides
// denote the SAME declaration, so its refusals were the relation calling one declaration two things ->
// (ii) too strict; arm 1 admits two DIFFERENT names declared to share one algebra -> declared semantic
// equality, not a checker defect. And one arm is excluded for the dominant family from the DATA, not the
// code: Map is not a carrier (algebra_carriers() declares Int, Nat, Float, BooleanAlgebra,
// FinitelySupportedFunction, FinitePowerSet, PartialFunction, PointwisePower, FreeMonoid, String), so a
// Map-vs-Map row cannot reach arm 1. d5 measures that instead of trusting the inference.
//
// RUNG, stated not claimed (DESIGN §4b). v1 Rust unit tests; DESIGN's Building & checks says the required
// path runs neither clippy nor the v1 unit tests, so these are GREEN BY VACANCY in CI. They are an
// INSTRUMENT to be EXECUTED, at two heads, and are not PR-2's discriminator — PR-2's discriminators stay
// in dag/test/claim per stern-bat-401's ruling (d). A row that behaves the same at both heads is not
// discriminating; the HEAD line each row prints makes the mislabelling impossible to hide.

/// Attach a real DeclarationRef through the seed's own constructor, so the field arm 2 compares
/// (module_path, decl_name) is genuinely populated rather than approximated. No test in this repo has ever
/// built one, so the path is taken from the compiler's own re-export (v1_compiler_infer_types re-exports
/// decl_ref publicly) rather than guessed.
fn refed(n: Rc<Node>, module_path: &str, decl_name: &str) -> Rc<Node> {
    let mut m = (*n).clone();
    m.declaration = Some(v1_compiler::v1_compiler_infer_types::decl_ref(
        module_path.to_string(),
        decl_name.to_string(),
    ));
    Rc::new(m)
}

fn shape_of(n: Rc<Node>) -> String {
    node_type_shape(n, Rc::new(im::HashMap::new()))
}

fn applied_of(head: &str, args: &[&str]) -> Rc<Node> {
    fold_app(head, &args.iter().map(|a| fold_leaf(a)).collect::<Vec<_>>())
}

/// ANY diagnostic is the refusal. Deliberately not filtered on message text: filtering on a substring is
/// how a false green gets manufactured the first time the wording moves.
fn refused(acc: &Rc<UnifyGenericsAcc>) -> bool {
    !acc.diagnostics.is_empty()
}

/// One fold step, spelled exactly as the head's harness spells it (verified against
/// cross_coproduct_generic_locate_test.rs at the head): T already bound to `bound`, now handed `actual`.
fn fold_one(bound: Rc<Node>, actual: Rc<Node>) -> Rc<UnifyGenericsAcc> {
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

/// Which head is running, DERIVED from the tree rather than from a label I could get wrong: the widening
/// leaves call sites in the compiled mirror and the strip keeps only the surviving definition. §6: derived
/// by the instrument, not transcribed.
fn head_tag() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../stage0/src/v1_compiler_infer_types.rs");
    match std::fs::read_to_string(path) {
        Ok(t) => format!(
            "type_heads_agree_occurrences_in_compiled_mirror={}",
            t.matches("type_heads_agree(").count()
        ),
        Err(e) => format!("HEAD_UNREADABLE:{e}"),
    }
}

/// Print the pair exactly as the Acc would print it, so a reader sees the expected X / got X form that the
/// corpus rows take, next to whether the fold refused.
fn say(tag: &str, row: &str, bound: &Rc<Node>, actual: &Rc<Node>, acc: &Rc<UnifyGenericsAcc>) -> String {
    format!(
        "{row} [{}]\n  expected 'type variable T bound as {} (first argument)', got '{} (later argument)'\n  refused={} diagnostics={}",
        tag,
        shape_of(bound.clone()),
        shape_of(actual.clone()),
        refused(acc),
        acc.diagnostics.iter().map(|d| format!("{d:?}")).collect::<Vec<_>>().join(" | ")
    )
}

// ---------------------------------------------------------------------------
// D1 — arm 2's ground truth. ONE declaration on both applied sides, equal arguments.
// Expected pair: REFUSES at the strip (main has no arm 2) and ACCEPTS at the Acc head (arm 2 fires). That
// pair IS the finding: same declaration, same arguments, refused by main's relation -> family (ii), the Acc
// too strict, and the largest thing the widening buys back. If it refuses at a head that HAS the widening,
// arm 2 is unreachable from the fold and every corpus row attributed to it belongs to someone else.
// ---------------------------------------------------------------------------
#[test]
fn d1_one_declaration_two_printings_of_it() {
    let left = refed(applied_of("DemandEntry", &["Int"]), "v2.compiler.target_model", "DemandEntry");
    let right = refed(applied_of("DemandEntry", &["Int"]), "v2.compiler.target_model", "DemandEntry");
    assert_eq!(shape_of(left.clone()), shape_of(right.clone()), "the fixture's sides must print identically, the way the corpus rows do; got {:?} vs {:?}", shape_of(left.clone()), shape_of(right.clone()));
    let acc = fold_one(left.clone(), right.clone());
    let out = say(&head_tag(), "D1", &left, &right, &acc);
    println!("{out}");
    assert!(
        !refused(&acc),
        "D1: ONE declaration applied to the SAME arguments is one type. Refusing it at main is the \
         over-refusal the widening exists to fix; refusing it at a head that HAS the widening means arm 2 \
         is not reachable from the fold and the arm-2 attribution is false. {out}"
    );
}

// ---------------------------------------------------------------------------
// D2 — the over-admission guard for arm 2: one spelling, TWO declarations. Whatever the corpus wants, the
// widening must refuse here. Admitting it is the silent wrong binding §5 treats as unforgivable, and it is
// exactly the risk stern-bat-401 named for a widening (msg_37357ae1). Must refuse at BOTH heads.
// ---------------------------------------------------------------------------
#[test]
fn d2_one_spelling_two_declarations_must_refuse() {
    let left = refed(applied_of("DemandEntry", &["Int"]), "v2.compiler.target_model", "DemandEntry");
    let right = refed(applied_of("DemandEntry", &["Int"]), "v2.compiler.somewhere_else", "DemandEntry");
    assert_eq!(shape_of(left.clone()), shape_of(right.clone()), "same printed shape by construction: only the declaration differs, which is precisely what the renderer cannot show");
    let acc = fold_one(left.clone(), right.clone());
    let out = say(&head_tag(), "D2", &left, &right, &acc);
    println!("{out}");
    assert!(refused(&acc), "D2: two distinct declarations under one spelling are two types; a widening that admits them is silently wrong, which is the outcome §5 forbids. {out}");
}

// ---------------------------------------------------------------------------
// D3 — no DeclarationRef on either side. Arm 2 answers false on Absent, so this must REFUSE at both heads.
// Green here would mean unqualified names are equal by spelling alone — the printed-name class review
// 80660 named, and the reason the head's own harness calls name-only equality "the defect review 80660 named".
// ---------------------------------------------------------------------------
#[test]
fn d3_no_declaration_on_either_side_must_refuse() {
    let left = applied_of("DemandEntry", &["Int"]);
    let right = applied_of("DemandEntry", &["Int"]);
    let acc = fold_one(left.clone(), right.clone());
    let out = say(&head_tag(), "D3", &left, &right, &acc);
    println!("{out}");
    assert!(refused(&acc), "D3: with no DeclarationRef on either applied side arm 2 must answer unequal; green here means bare spellings compare equal. {out}");
}

// ---------------------------------------------------------------------------
// D4 — arm 1 measured rather than assumed. 'List<Int>' vs 'FreeMonoid<Int>' is ONE declared algebra in two
// spellings: std/algebra.dag declares List as the FreeMonoid carrier's pascal alias
// (pascal_alias_spelling(text: "List"), container_alias_row RowPresent). Expected to ACCEPT at both heads,
// because arm 1 answers by carrier and never reads declarations. Its value is the measurement: if it
// REFUSES, the carrier roster is unreachable from the fold and every corpus row I attributed to arm 1
// belongs somewhere else — which is the finding, not a failure to hide.
// ---------------------------------------------------------------------------
#[test]
fn d4_declared_carrier_alias_pair() {
    let left = applied_of("List", &["Int"]);
    let right = applied_of("FreeMonoid", &["Int"]);
    let acc = fold_one(left.clone(), right.clone());
    let out = say(&head_tag(), "D4", &left, &right, &acc);
    println!("{out}");
    assert!(
        !refused(&acc),
        "D4: List and FreeMonoid are one declared algebra (std/algebra.dag declares List as the FreeMonoid \
         carrier's pascal alias), so the fold must not call them a conflict. If this refuses, the carrier \
         roster is not reachable from the fold and the arm-1 attribution is wrong. {out}"
    );
}

// ---------------------------------------------------------------------------
// D5 — the dominant family, disposed. Map is NOT a carrier, so a Map-vs-Map row cannot reach arm 1. This
// row measures whether an unqualified Map pair is refused; with D6 it assigns the dominant family to arm 2
// or to nothing. REPORTED as a disposition rather than asserted, because whichever way it goes it is a
// measurement, and asserting it would let a real flip read as a bug in the test.
// ---------------------------------------------------------------------------
#[test]
fn d5_map_pair_without_declarations() {
    let left = applied_of("Map", &["Symbol", "NonEmptyDiagnostics"]);
    let right = applied_of("Map", &["Symbol", "NonEmptyDiagnostics"]);
    let acc = fold_one(left.clone(), right.clone());
    let out = say(&head_tag(), "D5", &left, &right, &acc);
    println!("{out}");
    println!(
        "D5 DISPOSITION: {}",
        if refused(&acc) {
            "REFUSED without declarations -> since Map is not a carrier, the dominant Map family's admission \
             can only have come from arm 2 (real DeclarationRefs): one declaration in two printings, family \
             (ii), the Acc too strict."
        } else {
            "ACCEPTED without declarations -> bare unqualified Map spellings are equal by name, so those rows \
             never came from arm 2 and the carrier arm is doing the work: a different disposition, and it \
             says the relation is looser than its commit message claims."
        }
    );
}

// ---------------------------------------------------------------------------
// D6 — D5's other arm: the SAME Map pair with ONE declaration on both sides. D5-refused + D6-accepted is
// the signature that the widening's whole effect on the dominant family runs through declaration identity,
// established by two arms rather than by a reading. D5-refused + D6-REFUSED would mean the widening does
// not touch this family at all, and attributing Map rows to it would be false.
// ---------------------------------------------------------------------------
#[test]
fn d6_map_pair_with_one_declaration_on_both_sides() {
    let left = refed(applied_of("Map", &["Symbol", "NonEmptyDiagnostics"]), "std.algebra", "Map");
    let right = refed(applied_of("Map", &["Symbol", "NonEmptyDiagnostics"]), "std.algebra", "Map");
    let acc = fold_one(left.clone(), right.clone());
    let out = say(&head_tag(), "D6", &left, &right, &acc);
    println!("{out}");
    assert!(!refused(&acc), "D6: one declaration, equal arguments — the pair the Acc head must accept via arm 2. Refusing it is the over-refusal the widening exists to fix. {out}");
}

// ---------------------------------------------------------------------------
// D7 — the renderer blind spot, pinned so printed-shape triage can never be trusted by accident again. A
// Disj with two children and a Disj with none print IDENTICALLY: that is why the corpus's expected X / got
// X class cannot be dispositioned from logs at all, and it reproduces quiet-eagle-395's Box<Int> finding at
// a different shape. GREEN pins the blind spot; RED means prereq (a) landed and the triage must be re-run.
// ---------------------------------------------------------------------------
#[test]
fn d7_disj_children_are_invisible_to_the_renderer() {
    let plain = fold_disj("Coproduct", &[]);
    let hidden = fold_disj("Coproduct", &[fold_leaf("Bool"), fold_leaf("Int")]);
    let a = shape_of(plain.clone());
    let b = shape_of(hidden.clone());
    println!("D7 [{}] shapes=({a:?},{b:?})", head_tag());
    assert_eq!(
        a, b,
        "D7 pins the renderer blind spot that makes printed-shape triage unusable: a Disj's children must \
         not change its printed shape. GREEN here PINS it (children hidden). RED means the renderer now \
         prints what the comparator compares — prereq (a) landed — and the corpus triage must be re-run \
         honestly before any 19/11/13-style count is trusted."
    );
}
