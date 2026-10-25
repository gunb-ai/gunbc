use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_std_core::{
    diagnostic_disposition, diagnostic_to_message, diagnostic_to_span, source_text_at,
    CompilerDiagnostic, DiagnosticGateDisposition,
};

// THE FLOOR (§4b): a value inhabits its DECLARED type, and the values a type name admits are fixed
// by the shape that name declares. A name declaring a coproduct carries NO record constructor, so
// `Union { field: ... }` names a value no type inhabits --- whatever the fields, wherever the
// literal stands.
//
// WHAT SLIPPED: extdeps.systemd.systemd_run `SystemdRunProperty` was a record until #13942 turned
// it into the union `NamedUnitSetting { property: ExecStopProperty, value: String } |
// EmptyCapabilityBoundingSet`. #14149, green on its own base, then wrote the OLD record shape
// `SystemdRunProperty { property: ..., value: ... }`; the seed typechecker accepted it at authoring
// and main refused only at EVAL (`//gunbc/instruments:compute-run-unit-cold`, run 38905139153,
// 'non-exhaustive pattern match'). The reason it was accepted is that `record_lit_expected_fields`
// reads the union's CHILD VARIANTS as the field list, so the field walk could only ever ask "is
// this field in the union of the variants' fields" --- a question with no answer to the real one,
// "does this name have a record constructor at all". So the refusal is taken off the resolved
// declaration, at the literal's name span, BEFORE the field walk, and it is prepended into the
// diagnostic concat rather than returned early (an early return absorbs the later TypeMismatch the
// stop-observation witness claims --- the fail-open DESIGN §5 refuses).

/// The typecheck pass, which is the boundary this refusal belongs to: it resolves, reconciles and
/// typechecks, and keeps the source indices so a diagnostic's span can be read back as text.
fn compile(
    path: &str,
    content: &str,
) -> Rc<v1_compiler::v1_compiler_compile::ResolvedPipelineResult> {
    compile_to_resolved(Rc::new(im::vector![Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })]))
}

/// What "accepted" means here: nothing gate-blocking. Advisory diagnostics are reported and do not
/// refuse the program, so an acceptance control that demanded an empty list would test something
/// other than acceptance.
fn blocking(r: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| {
            matches!(
                diagnostic_disposition(d.diagnostic.clone()).gate.as_ref(),
                DiagnosticGateDisposition::GateBlocking
            )
        })
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

// The union mirrors #13942's SystemdRunProperty shape exactly (Zz-prefixed so no corpus name
// collides): one arm carrying two fields, one arm carrying none.
const UNION_DECL: &str = "module zz_union_probe\n\
     type ZzA { x: Int }\n\
     type ZzU = ZzNamed { property: ZzA, value: Int } | ZzEmpty\n";

#[derive(Debug)]
struct Refusal {
    message: String,
    /// The source text the located span actually covers --- the test of location, since a span is
    /// two offsets into an indexed file and only the text says which fact it points at.
    at: String,
}

fn refusals(
    r: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
    path: &str,
) -> Vec<Refusal> {
    let index = r.source_indices.get(path).expect("fixture source index");
    r.diagnostics
        .iter()
        .filter(|d| {
            matches!(
                *d.diagnostic,
                CompilerDiagnostic::TypeNameCarriesNoRecordConstructor { .. }
            )
        })
        .map(|d| Refusal {
            message: diagnostic_to_message(d.diagnostic.clone()),
            at: source_text_at(index.clone(), diagnostic_to_span(d.diagnostic.clone())),
        })
        .collect()
}

#[test]
fn record_literal_naming_a_union_type_refuses_at_every_position() {
    // Each position is one the specimen reached typecheck in: a data binding, a function return,
    // a List element (work_provider_local.dag's fixed site), and a call argument (the parameter
    // compute_run_unit_cold passes to the stop-observation site). All four were accepted before
    // this refusal existed, and the EVAL failure appeared pages away from the authoring.
    let cases: Vec<(&str, &str, &str)> = vec![
        (
            "data position",
            "zz_union_data.dag",
            "data zz_d: ZzU = ZzU { property: ZzA { x: 1 }, value: 2 }\n",
        ),
        (
            "function return",
            "zz_union_ret.dag",
            "fn zz_ret() -> ZzU { ZzU { property: ZzA { x: 1 }, value: 2 } }\n",
        ),
        (
            "list element",
            "zz_union_list.dag",
            "fn zz_props() -> List<ZzU> { [ZzU { property: ZzA { x: 1 }, value: 2 }] }\n",
        ),
        (
            "call argument",
            "zz_union_call.dag",
            "fn zz_take(property: List<ZzU>) -> Int { 1 }\nfn zz_call() -> Int { zz_take(property: [ZzU { property: ZzA { x: 1 }, value: 2 }]) }\n",
        ),
    ];
    for (label, path, body) in cases {
        let r = compile(path, &format!("{UNION_DECL}{body}"));
        let got = refusals(&r, path);
        assert_eq!(
            got.len(),
            1,
            "{label}: the union-naming literal must refuse once, at the name; got {:?}",
            r.diagnostics
                .iter()
                .map(|d| diagnostic_to_message(d.diagnostic.clone()))
                .collect::<Vec<_>>()
        );
        assert!(
            got[0].message.contains("ZzNamed") && got[0].message.contains("ZzEmpty"),
            "{label}: the refusal must name the variants the author could have meant: {}",
            got[0].message
        );
        // THE LOCATION IS THE NAME. Not the literal's braces, not a field: the defect is the type
        // name written, so that is what the reader's caret must land on.
        assert_eq!(
            got[0].at, "ZzU",
            "{label}: the refusal must be located at the literal's type name: {:?}",
            got
        );
        let disposition = diagnostic_disposition(r.diagnostics[0].diagnostic.clone());
        assert!(
            matches!(
                disposition.gate.as_ref(),
                DiagnosticGateDisposition::GateBlocking
            ),
            "{label}: a value that inhabits no declared type is gate-blocking, not advisory: {:?}",
            disposition
        );
    }
}

#[test]
fn refusal_does_not_depend_on_the_fields_written() {
    // A field set the union's variants could satisfy (above) is the interesting case; these two
    // show the refusal is about the NAME, so it fires when no variant could satisfy the shape
    // either. Before the fix both were accepted --- the field walk read the union of the variants'
    // fields and found nothing to object to.
    let missing_field = compile(
        "zz_union_missing.dag",
        &format!("{UNION_DECL}fn zz_ret() -> ZzU {{ ZzU {{ property: ZzA {{ x: 1 }} }} }}\n"),
    );
    let got_missing = refusals(&missing_field, "zz_union_missing.dag");
    assert_eq!(
        got_missing.len(),
        1,
        "a shape no variant satisfies must refuse: {:?}",
        missing_field.diagnostics
    );
    assert_eq!(
        got_missing[0].at, "ZzU",
        "located at the name, not the missing field"
    );

    let bogus_field = compile(
        "zz_union_bogus.dag",
        &format!("{UNION_DECL}fn zz_ret() -> ZzU {{ ZzU {{ not_a_field: 1 }} }}\n"),
    );
    let got_bogus = refusals(&bogus_field, "zz_union_bogus.dag");
    assert_eq!(
        got_bogus.len(),
        1,
        "a field no variant declares must refuse: {:?}",
        bogus_field.diagnostics
    );
    assert_eq!(
        got_bogus[0].at, "ZzU",
        "located at the name, not the unknown field"
    );
}

#[test]
fn a_name_declaring_nothing_keeps_its_own_refusal() {
    // The new refusal must not swallow the unresolved-name diagnostic: an unknown name is a
    // different fact (no declaration to consult at all), located by a different rule.
    let unknown = compile(
        "zz_union_unknown.dag",
        &format!("{UNION_DECL}fn zz_ret() -> Int {{ ZzNowhere {{ anything: 1 }} }}\n"),
    );
    assert!(
        refusals(&unknown, "zz_union_unknown.dag").is_empty(),
        "an undeclared name is not a union: {:?}",
        refusals(&unknown, "zz_union_unknown.dag")
    );
    let messages: Vec<String> = unknown
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(
        messages.iter().any(|m| m.contains("ZzNowhere")),
        "the unresolved-name refusal must still be reported: {messages:?}"
    );
}

#[test]
fn variant_constructions_of_a_union_type_are_accepted() {
    // POSITIVE CONTROL: the two spellings an author of this union actually has. If the refusal
    // were keyed on the union name in general --- or on a field walk that misreads the arms ---
    // this case would refuse too.
    let control = compile(
        "zz_union_ok.dag",
        &format!(
            "{UNION_DECL}fn zz_named() -> ZzU {{ ZzNamed {{ property: ZzA {{ x: 1 }}, value: 2 }} }}\nfn zz_empty() -> ZzU {{ ZzEmpty }}\n"
        ),
    );
    assert_eq!(
        blocking(&control),
        Vec::<String>::new(),
        "constructing a variant of the union is the correct construction"
    );
}

#[test]
fn a_record_type_name_and_an_aliased_record_name_are_still_accepted() {
    // MORE POSITIVE CONTROL, for the two ways the refusal could be too wide: a plain record name
    // (the shape every existing literal in the corpus uses) and a name that ALIASES a record,
    // which resolves through the same lookup and must still be a constructor.
    let ok = compile(
        "zz_record_ok.dag",
        "module zz_record_ok\n\
         type ZzA { x: Int }\n\
         type ZzAlias = ZzA\n\
         fn f() -> ZzA { ZzA { x: 1 } }\n\
         fn g() -> ZzAlias { ZzAlias { x: 2 } }\n",
    );
    assert_eq!(
        blocking(&ok),
        Vec::<String>::new(),
        "record and aliased-record literals are constructors"
    );
}

#[test]
fn an_aliased_union_name_refuses_and_names_its_variants() {
    // The specimen's own shape was reached through `type SystemdRunProperty` written directly, but
    // an author can name the union through an alias; the refusal reads the EXPANSION, so the
    // message lists the variants rather than the alias's opaque name.
    let r = compile(
        "zz_union_alias.dag",
        "module zz_union_alias\n\
         type ZzA { x: Int }\n\
         type ZzU = ZzNamed { property: ZzA, value: Int } | ZzEmpty\n\
         type ZzT = ZzU\n\
         fn f() -> ZzT { ZzT { property: ZzA { x: 1 }, value: 2 } }\n",
    );
    let got = refusals(&r, "zz_union_alias.dag");
    assert_eq!(
        got.len(),
        1,
        "a union reached through an alias carries no record constructor either: {:?}",
        r.diagnostics
            .iter()
            .map(|d| diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
    assert!(
        got[0].message.contains("ZzNamed") && got[0].message.contains("ZzEmpty"),
        "the message must name the variants, not the alias: {}",
        got[0].message
    );
    assert_eq!(got[0].at, "ZzT", "located at the name the author wrote");
}

#[test]
fn an_imported_variant_name_is_accepted_under_a_pool_collision() {
    // POSITIVE CONTROL FOR THE RESOLUTION, not for the shape. The refusal asks the question of the
    // declaration THIS module's binding chain resolves the head to; it must never ask it of the
    // corpus-wide pool. The witness that measured this (pre-fix, on main) is
    // dag/test/claim/workspace_commissioning_apply_readback_witness.dag: it constructs the ARM
    // UnitPropertyRead { value: status } of runner_microvm_slot_start's union
    // `type UnitPropertyReading = UnitPropertyRead { value: String } | UnitPropertyUnread {...}`,
    // with UnitPropertyRead imported BY NAME from that module --- and was refused, because the pool
    // independently declares `type UnitPropertyRead = UnitPropertyValue | UnitPropertyUnreadable` at
    // dag/gunbc/live_deploy/unit_standing.dag:39, a module the witness never imports. A lookup that
    // answers "is this name a union" from the bare global overlay turns every arm whose name
    // collides with an unrelated union elsewhere into a refusal.
    //
    // Two SourceFiles, because the collision only exists ACROSS modules: one file declares the union
    // whose arm is constructed, the other declares an unrelated union UNDER THE ARM'S NAME, and a
    // third compiles the construction that imports the first by name but not the second.
    let files = im::vector![
        Rc::new(SourceFile {
            // The arm's owner. The arm's authored name is UnitPropertyRead. Mirrors the real site's
            // shape: the union has one arm carrying a NESTED record field, so the field walk -- the
            // very code the refusal sits in front of -- is exercised on a legal construction.
            path: "zz_owner.dag".to_string(),
            content: "module zz_owner\n\
                      type ZzTag { name: String }\n\
                      type UnitPropertyReading\n\
                      \x20 = UnitPropertyRead { value: String, tag: ZzTag }\n\
                      \x20 | UnitPropertyUnread { property: String, detail: String }\n"
                .to_string(),
        }),
        Rc::new(SourceFile {
            // The collision: a DIFFERENT union, authored under the arm's name, in a module the
            // consumer below never imports. Its variants are deliberately distinct so the refusal's
            // message would name THEM if the resolution ever reached this declaration.
            path: "zz_collision.dag".to_string(),
            content: "module zz_collision\n\
                      type UnitPropertyRead\n\
                      \x20 = UnitPropertyValue { value: String }\n\
                      \x20 | UnitPropertyInvisible\n"
                .to_string(),
        }),
        Rc::new(SourceFile {
            // The arm, constructed in FIELD POSITION inside a record literal whose declared field type
            // is the union, and consumed by a match on that field -- the shape the real site uses
            // (SlotStartCompleted { exec_main_status: UnitPropertyRead { value: status }, .. } in
            // test.claim.workspace_commissioning_apply_readback_witness). Position matters: a
            // top-level `data x: Union = Arm { .. }` is settled by the expected-variant route before
            // this refusal runs, so an unnested fixture says nothing about resolution at all.
            //
            // WHAT THIS FIXTURE DOES AND DOES NOT ESTABLISH. It is a total-acceptance control: the
            // three-module program compiles with NO gate-blocking diagnostic, so no refusal --
            // this one or any other -- fires on a legal construction whose arm name collides with an
            // unrelated union elsewhere in the pool. It is NOT the evidence that the narrowed
            // resolution is what makes it pass: measured at this head, reverting record_lit_name_declares_union
            // to its pre-narrowing text (global pool lookup, no arm exemption) leaves this fixture
            // GREEN too. The pool overlay answers only for names bound nowhere in the chain, and the
            // whole-corpus index that made the real site resolve to the colliding union is not
            // reproducible from a three-module unit compile. The discrimination for that ruling is
            // the whole-corpus census: pre-narrowing it produced the 2 false refusals at
            // test.claim.workspace_commissioning_apply_readback_witness (PR body, Run 1), and at this
            // head it produces 0 (Run 3/4) -- same loader, same tree, opposite counts.
            path: "zz_consumer.dag".to_string(),
            content: "module zz_consumer\nimport zz_owner { UnitPropertyReading, UnitPropertyRead, UnitPropertyUnread, ZzTag }\ntype ZzHolder { exec_main_status: UnitPropertyReading }\nfn read_exec(holder: ZzHolder) -> Bool {\n  match holder.exec_main_status {\n    UnitPropertyRead { value, tag } => value == \"running\"\n    UnitPropertyUnread { property, detail } => property == \"ExecMainStatus\"\n  }\n}\ndata zz_owner_status: Bool = read_exec(holder: ZzHolder { exec_main_status: UnitPropertyRead { value: \"running\", tag: ZzTag { name: \"s\" } } })\ndata zz_owner_unread: Bool = read_exec(holder: ZzHolder { exec_main_status: UnitPropertyUnread { property: \"ExecMainStatus\", detail: \"unreadable\" } })\n"
                .to_string(),
        }),
    ];
    let r = compile_to_resolved(Rc::new(files));
    let messages: Vec<String> = r
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    // Asserted EMPTY, not by string-absence: these two constructions are legal, so NO gate-blocking
    // diagnostic may exist. A control that only denies one class is green for any unrelated blocking
    // refusal; the acceptance it certifies is total acceptance (side-chat ruling on #14258). The two
    // attribution asserts below still say WHICH refusal must not be the one that appears.
    assert_eq!(
        blocking(&r),
        Vec::<String>::new(),
        "importing an arm by name must compile clean even when its name is an unrelated union \
         elsewhere in the pool: {messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|m| m.contains("declares no record constructor")),
        "an imported arm whose name is an unrelated union elsewhere in the pool is still a \
         constructor; the refusal consulted the pool, not the module's own binding: {messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m.contains("UnitPropertyValue")),
        "the colliding union's variants must never appear; reaching them means the head resolved \
         against zz_collision.dag: {messages:?}"
    );
}

#[test]
fn the_refusal_message_says_what_it_says() {
    // The message is the repair instruction, so it is tested rather than eyeballed: it names the
    // type written, says the declaration carries no record constructor, and lists the variants
    // available. (A refusal that only said "type mismatch" sent the author to the EVAL error.)
    let r = compile(
        "zz_union_msg.dag",
        &format!(
            "{UNION_DECL}fn zz_ret() -> ZzU {{ ZzU {{ property: ZzA {{ x: 1 }}, value: 2 }} }}\n"
        ),
    );
    let got = refusals(&r, "zz_union_msg.dag");
    assert_eq!(got.len(), 1, "{:?}", r.diagnostics);
    let message = &got[0].message;
    assert!(
        message.contains("record literal names 'ZzU'"),
        "must quote the name written: {message}"
    );
    assert!(
        message.contains("declares no record constructor"),
        "must state the fact: {message}"
    );
    assert!(
        message.contains("[ZzNamed, ZzEmpty]"),
        "must list the variants the author could have meant: {message}"
    );
    assert!(
        message.contains("construct one of its variants instead"),
        "must name the repair: {message}"
    );
}
