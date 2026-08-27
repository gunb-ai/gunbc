// THE FIXTURE-BOUNDARY RED FOR THE NAMESPACE WAVE-ADMISSION WALL.
//
// DESIGN §4b requires the question BEFORE the check is written: is the forbidden state
// authorable anywhere the check can run? For this wall the corpus boundary cannot decide it —
// on any given pull request the live delta may be empty, and a wall that is green because
// nothing moved is indistinguishable from a wall that is green because it cannot move. The
// FIXTURE boundary decides it, and this file is the receipt: `adjudicate` takes TWO INDEXES,
// so a fixture authors a base tree and a head tree and reads back the exact disposition.
//
// EVERY ARM BELOW IS A MUTATION OF ONE OTHER ARM. The wall's own auto-admitted case
// (`same_module_moved_route_same_declarer_is_a_rebind`) is the positive control; each refusing
// arm changes ONE fact about it and requires a different disposition. An arm that passed
// whatever the fixture said would be the decoration this file exists to disprove.

use std::path::{Path, PathBuf};

use v1_compiler::cli_run::namespace_wave_admission::{
    adjudicate, base_records, diff_sides, disposition_label, report_unadjudicated,
    AdmissionSubject, DeltaSubject, NamespaceDeltaDisposition, TransitionAdmission,
    WaveAdmissionReport,
};
use v1_compiler::cli_run::run_dag_parse_sweep;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gunbc_wave_admission_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("probe_root")).expect("scratch tree");
    dir
}

fn author(dir: &Path, basename: &str, source: &str) {
    std::fs::write(dir.join("probe_root").join(basename), source).expect("fixture source");
}

/// Index one authored tree. Panics on a parse refusal, because a fixture that does not parse
/// is testing the parser rather than the wall — the malformed-plant-versus-broken-guard
/// ambiguity, refused up front.
fn index_of(dir: &Path) -> v1_compiler::cli_run::declaration_index::DeclarationIndex {
    match run_dag_parse_sweep(dir, &["probe_root"]) {
        Ok(sweep) => sweep.index,
        Err(errors) => panic!("fixture must parse; sweep refused: {errors:?}"),
    }
}

/// The two sides of one scenario, adjudicated with an empty admission roster.
fn compare(name: &str, base: &[(&str, &str)], head: &[(&str, &str)]) -> WaveAdmissionReport {
    compare_with(name, base, head, &[])
}

fn compare_with(
    name: &str,
    base: &[(&str, &str)],
    head: &[(&str, &str)],
    admissions: &[TransitionAdmission],
) -> WaveAdmissionReport {
    let base_dir = scratch(&format!("{name}_base"));
    let head_dir = scratch(&format!("{name}_head"));
    for (file, source) in base {
        author(&base_dir, file, source);
    }
    for (file, source) in head {
        author(&head_dir, file, source);
    }
    let base_index = index_of(&base_dir);
    let head_index = index_of(&head_dir);
    // PLANT WELL-FORMEDNESS: both sides indexed something. A tree that produced no modules
    // makes every arm below vacuous, and a vacuous pass reads exactly like a real one.
    assert!(
        v1_compiler::cli_run::declaration_index::index_population(&base_index).modules > 0,
        "PLANT MALFORMED: the base fixture indexed no modules"
    );
    assert!(
        v1_compiler::cli_run::declaration_index::index_population(&head_index).modules > 0,
        "PLANT MALFORMED: the head fixture indexed no modules"
    );
    adjudicate(&base_index, &head_index, admissions)
}

fn dispositions_for(
    report: &WaveAdmissionReport,
    spelling: &str,
) -> Vec<NamespaceDeltaDisposition> {
    report
        .deltas
        .iter()
        .filter(|d| match &d.subject {
            DeltaSubject::Binding { spelling: s, .. } => s == spelling,
            _ => false,
        })
        .map(|d| d.disposition)
        .collect()
}

fn membership_dispositions(
    report: &WaveAdmissionReport,
    target: &str,
) -> Vec<NamespaceDeltaDisposition> {
    report
        .deltas
        .iter()
        .filter(|d| match &d.subject {
            DeltaSubject::Membership { target: t, .. } => t == target,
            _ => false,
        })
        .map(|d| d.disposition)
        .collect()
}

// ── THE SHARED SCENARIO ──
//
// `probe.home` declares `widget`. `probe.consumer` names it. Every arm below varies exactly
// one thing about that.

const HOME: &str = "module probe.home\n\ndata widget: String = \"w\"\n";
const OTHER: &str = "module probe.other\n\ndata widget: String = \"o\"\n";

const CONSUMER_IMPORTS_HOME: &str =
    "module probe.consumer\n\nimport probe.home { widget }\n\nfn use_it() -> String { widget }\n";
const CONSUMER_QUALIFIES_HOME: &str =
    "module probe.consumer\n\nfn use_it() -> String { probe.home.widget }\n";
const CONSUMER_IMPORTS_OTHER: &str =
    "module probe.consumer\n\nimport probe.other { widget }\n\nfn use_it() -> String { widget }\n";
const CONSUMER_IMPORTS_BOTH: &str = "module probe.consumer\n\nimport probe.home { widget }\nimport probe.other { widget }\n\nfn use_it() -> String { widget }\n";
const CONSUMER_IMPORTS_NOTHING: &str =
    "module probe.consumer\n\nfn use_it() -> String { widget }\n";
// THE POOL-COINCIDENCE PAIR. The consumer's own source is BYTE-IDENTICAL on both sides and
// reaches `probe.home` through a blanket import; the target is what grows `widget`. That is
// the only way to author a `0 -> 1` whose cause is in another module, and it is what
// separates the coincidence from the authored repair below.
const HOME_WITHOUT_WIDGET: &str = "module probe.home\n\ndata unrelated: String = \"u\"\n";
const CONSUMER_BLANKET_HOME: &str =
    "module probe.consumer\n\nimport probe.home\n\nfn use_it() -> String { widget }\n";
// THE SAME CONSUMER PLUS ONE UNRELATED NEW BLANKET IMPORT. `probe.other` here does NOT supply
// `widget`, so nothing about this addition can explain `widget` beginning to resolve.
const OTHER_WITHOUT_WIDGET: &str = "module probe.other\n\ndata gadget: String = \"g\"\n";
const CONSUMER_BLANKET_HOME_AND_OTHER: &str = "module probe.consumer\n\nimport probe.home\nimport probe.other\n\nfn use_it() -> String { widget }\n";

// ── THE POSITIVE CONTROL: an admitted run over a real, non-empty comparison ──

#[test]
fn an_unchanged_tree_carries_no_delta_and_names_what_it_compared() {
    let files = [("home.dag", HOME), ("consumer.dag", CONSUMER_IMPORTS_HOME)];
    let report = compare("unchanged", &files, &files);
    assert!(
        report.deltas.is_empty(),
        "an unchanged tree must carry no delta, got: {:?}",
        report.deltas
    );
    // THE DENOMINATOR IS THE OTHER HALF OF THIS ASSERTION. Zero deltas over zero compared
    // rows is not a pass, and without this the arm would green over an empty index.
    assert_eq!(report.population.modules_compared, 2);
    assert!(
        report.population.binding_rows_compared > 0,
        "no binding row was compared, so the zero above is ignorance rather than agreement"
    );
    assert!(report_unadjudicated(&report).is_empty());
}

// ── AUTO-ADMITTED: the route moved and the declaring identity did not ──

#[test]
fn dropping_an_import_for_a_qualified_spelling_keeps_the_declarer_and_is_admitted() {
    let report = compare(
        "rebind",
        &[("home.dag", HOME), ("consumer.dag", CONSUMER_IMPORTS_HOME)],
        &[
            ("home.dag", HOME),
            ("consumer.dag", CONSUMER_QUALIFIES_HOME),
        ],
    );
    // The membership edge SURVIVES — the qualified spelling reaches the same module — so the
    // only motion is the route. Nothing here may refuse.
    assert!(
        report_unadjudicated(&report).is_empty(),
        "a route change that preserves every declaring identity must be admitted, got: {:?}",
        report_unadjudicated(&report)
            .iter()
            .map(|d| (disposition_label(d.disposition), d.detail.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn removing_membership_nothing_bound_through_is_admitted_as_unused() {
    // THE UNUSED EDGE MUST SUPPLY A NAME NOTHING MENTIONS. An earlier version of this fixture
    // imported the SAME leaf the module reaches by another route, so the edge was not unused at
    // all and the wall correctly said so — a malformed plant reading as a wrong verdict, which
    // is why the plant assertion below exists.
    let gadget_home = "module probe.other\n\ndata gadget: String = \"g\"\n";
    let unused_base = "module probe.consumer\n\nimport probe.other { gadget }\n\nfn use_it() -> String { probe.home.widget }\n";
    let report = compare(
        "unused",
        &[
            ("home.dag", HOME),
            ("other.dag", gadget_home),
            ("consumer.dag", unused_base),
        ],
        &[
            ("home.dag", HOME),
            ("other.dag", gadget_home),
            ("consumer.dag", CONSUMER_QUALIFIES_HOME),
        ],
    );
    assert!(
        report
            .deltas
            .iter()
            .any(|d| matches!(&d.subject, DeltaSubject::Membership { target, .. } if target == "probe.other")),
        "PLANT NEVER REACHED: no membership delta for probe.other, so the disposition below \
         would be a verdict about nothing, got: {:?}",
        report.deltas
    );
    assert!(
        membership_dispositions(&report, "probe.other")
            .contains(&NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved),
        "removing membership no name bound through must be the unused-removal disposition, got: {:?}",
        report.deltas
    );
}

// ── REFUSING ARMS: one mutation each ──

#[test]
fn moving_a_declaration_to_another_module_refuses_as_target_changed() {
    let report = compare(
        "target_changed",
        &[
            ("home.dag", HOME),
            ("other.dag", OTHER),
            ("consumer.dag", CONSUMER_IMPORTS_HOME),
        ],
        &[
            ("home.dag", HOME),
            ("other.dag", OTHER),
            ("consumer.dag", CONSUMER_IMPORTS_OTHER),
        ],
    );
    assert!(
        dispositions_for(&report, "widget").contains(&NamespaceDeltaDisposition::TargetChanged),
        "a spelling that denotes a different module must be TargetChanged, got: {:?}",
        report.deltas
    );
    assert!(
        !report_unadjudicated(&report).is_empty(),
        "TargetChanged is on the refusing side of the ruling's partition"
    );
}

#[test]
fn a_second_declarer_on_the_chain_refuses_as_new_ambiguity() {
    let report = compare(
        "ambiguity",
        &[
            ("home.dag", HOME),
            ("other.dag", OTHER),
            ("consumer.dag", CONSUMER_IMPORTS_HOME),
        ],
        &[
            ("home.dag", HOME),
            ("other.dag", OTHER),
            ("consumer.dag", CONSUMER_IMPORTS_BOTH),
        ],
    );
    assert!(
        dispositions_for(&report, "widget").contains(&NamespaceDeltaDisposition::NewAmbiguity),
        "two declarers for one spelling must be NewAmbiguity — NOT a silent pick, got: {:?}",
        report.deltas
    );
}

#[test]
fn a_spelling_that_stops_denoting_anything_refuses_as_new_unresolvedness() {
    let report = compare(
        "unresolved",
        &[("home.dag", HOME), ("consumer.dag", CONSUMER_IMPORTS_HOME)],
        &[
            ("home.dag", HOME),
            ("consumer.dag", CONSUMER_IMPORTS_NOTHING),
        ],
    );
    assert!(
        dispositions_for(&report, "widget").contains(&NamespaceDeltaDisposition::NewUnresolvedness),
        "a name that lost its declaration must be NewUnresolvedness, got: {:?}",
        report.deltas
    );
    assert!(!report_unadjudicated(&report).is_empty());
}

// ── `0 -> 1` IS TWO STATES, AND THESE TWO ARMS ARE THE DISCRIMINATING PAIR ──
//
// The two fixtures differ in ONE fact: which side's author wrote something. In the first the
// consumer's own source is unchanged and the target grew the name; in the second the target is
// unchanged and the consumer authored the import. Both are `0 -> 1` at the set grain, and
// reading them as one symbol is what made the wall refuse the repair it exists to want.
//
// THE SECOND ARM WAS THE FIRST ARM'S FIXTURE UNTIL 2026-08-27. This file asserted
// NewPoolCoincidenceResolution over a head that authored `import probe.home { widget }` — it
// planted the repair and named it the coincidence, which is why the conflation survived
// review. Re-deriving it required authoring a coincidence that is actually one.

#[test]
fn a_spelling_the_target_began_supplying_refuses_as_pool_coincidence() {
    let report = compare(
        "coincidence",
        &[
            ("home.dag", HOME_WITHOUT_WIDGET),
            ("consumer.dag", CONSUMER_BLANKET_HOME),
        ],
        &[("home.dag", HOME), ("consumer.dag", CONSUMER_BLANKET_HOME)],
    );
    assert!(
        dispositions_for(&report, "widget")
            .contains(&NamespaceDeltaDisposition::NewPoolCoincidenceResolution),
        "a name the TARGET began supplying, with the consumer's source unchanged, must be \
         NewPoolCoincidenceResolution, got: {:?}",
        report.deltas
    );
    // AND IT MUST STILL REFUSE. The split adds an auto-admitted arm; if it had widened this
    // one the wall would have lost the class it was built for.
    assert!(
        !report_unadjudicated(&report).is_empty(),
        "a pool coincidence must remain unadjudicated, got: {:?}",
        report.deltas
    );
}

#[test]
fn an_unrelated_new_blanket_import_does_not_launder_a_pool_coincidence() {
    // THE FAIL-OPEN THIS ARM EXISTS TO REFUSE (review 56882, on gunbc#9495). A blanket import
    // names no leaf, so a leaf-blind "any new blanket target" test would read this unrelated
    // addition as authorship of `widget` and auto-admit the coincidence beside it. The target
    // that grew `widget` is `probe.home`, whose blanket import is UNCHANGED; the target the
    // author added is `probe.other`, which does not supply `widget` at all.
    let report = compare(
        "unrelated_blanket",
        &[
            ("home.dag", HOME_WITHOUT_WIDGET),
            ("other.dag", OTHER_WITHOUT_WIDGET),
            ("consumer.dag", CONSUMER_BLANKET_HOME),
        ],
        &[
            ("home.dag", HOME),
            ("other.dag", OTHER_WITHOUT_WIDGET),
            ("consumer.dag", CONSUMER_BLANKET_HOME_AND_OTHER),
        ],
    );
    assert!(
        dispositions_for(&report, "widget")
            .contains(&NamespaceDeltaDisposition::NewPoolCoincidenceResolution),
        "a pool coincidence beside an unrelated new blanket import must stay \
         NewPoolCoincidenceResolution, got: {:?}",
        report.deltas
    );
    assert!(
        !report_unadjudicated(&report).is_empty(),
        "it must still refuse, got: {:?}",
        report.deltas
    );
}

#[test]
fn a_new_blanket_import_that_does_supply_the_leaf_is_an_authored_reference_resolution() {
    // THE ARM'S OTHER HALF, so the scoping above is not just a narrowing to nothing: when the
    // NEW blanket target is the one supplying the leaf, the author did write the claim that
    // resolves it, and it auto-admits like any other authored reference.
    let report = compare(
        "supplying_blanket",
        &[
            ("home.dag", HOME),
            ("consumer.dag", CONSUMER_IMPORTS_NOTHING),
        ],
        &[("home.dag", HOME), ("consumer.dag", CONSUMER_BLANKET_HOME)],
    );
    assert!(
        dispositions_for(&report, "widget")
            .contains(&NamespaceDeltaDisposition::AuthoredReferenceResolution),
        "a new blanket import that supplies the leaf must be AuthoredReferenceResolution, \
         got: {:?}",
        report.deltas
    );
    assert!(
        report_unadjudicated(&report).is_empty(),
        "it must auto-admit, got: {:?}",
        report_unadjudicated(&report)
    );
}

#[test]
fn a_spelling_this_module_authored_the_import_for_is_an_authored_reference_resolution() {
    let report = compare(
        "authored_resolution",
        &[
            ("home.dag", HOME),
            ("consumer.dag", CONSUMER_IMPORTS_NOTHING),
        ],
        &[("home.dag", HOME), ("consumer.dag", CONSUMER_IMPORTS_HOME)],
    );
    assert!(
        dispositions_for(&report, "widget")
            .contains(&NamespaceDeltaDisposition::AuthoredReferenceResolution),
        "a dangling name resolved by an import this module's own author wrote must be \
         AuthoredReferenceResolution, got: {:?}",
        report.deltas
    );
    // THE WHOLE POINT OF THE SPLIT: this is the shape of gunbc#9485, and it must merge without
    // an admission row. Asserting the disposition alone would leave the wall refusing it.
    assert!(
        report_unadjudicated(&report).is_empty(),
        "an authored-reference resolution must be auto-admitted, got: {:?}",
        report_unadjudicated(&report)
    );
}

// ── THE ADMISSION PATH, AND ITS OWN RED ──

#[test]
fn an_exact_transition_admission_admits_that_delta_and_only_that_delta() {
    let base = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_HOME),
    ];
    let head = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_OTHER),
    ];
    let refused = compare("admission_before", &base, &head);
    assert!(
        !report_unadjudicated(&refused).is_empty(),
        "PLANT NEVER REACHED: the un-admitted arm must refuse, or the admission below proves nothing"
    );

    let admissions = [TransitionAdmission {
        label: "fixture-transition",
        subject: AdmissionSubject::Binding {
            module: "probe.consumer",
            in_declaration: "use_it",
            spelling: "widget",
        },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    }];
    let admitted = compare_with("admission_after", &base, &head, &admissions);
    assert!(
        admitted
            .deltas
            .iter()
            .any(|d| d.admitted_by.as_deref() == Some("fixture-transition")),
        "the admission must attach to its exact subject, got: {:?}",
        admitted.deltas
    );
    assert!(
        admitted.stale_admissions.is_empty(),
        "an admission that matched must not also be reported stale"
    );
}

// THE ROSTER IS INHABITABLE, AND THIS TEST IS THE ONLY THING THAT SAYS SO.
//
// The two arms above prove the MATCHING mechanism works, and they proved it while the production
// roster could not hold a single row. They build their admissions in a `let` with `.to_string()`,
// so nothing in them ever touched the constraint that actually bound
// `NAMESPACE_TRANSITION_ADMISSIONS`: it is a `const`, and `String::from` is not callable there.
// Every authorable production row had to name the empty module, which matches no delta.
//
// SO THE COVERAGE MADE THE DEFECT MORE HIDDEN RATHER THAN LESS. A reader auditing the admission
// path found two thorough tests, green, exercising exact-match and wrong-subject-refuses — and
// none of it was evidence about the roster a human would actually author. That is the shape where
// a fixture cannot see the carrier it is a fixture for.
//
// THE DISCRIMINATING PROPERTY IS COMPILATION, NOT THE ASSERTION, and the boundary is exact rather
// than "a const row would not compile". Measured against main's own types, both arms:
//
//   module: String::from("probe.consumer")  ->  error[E0015]: cannot call non-const associated
//                                               function `<String as From<&str>>::from` in
//                                               constants — three times, one per field
//   module: String::new()                   ->  compiles clean, emits metadata
//
// So a const row was always AUTHORABLE; what no const row could do was NAME A REAL MODULE. Stating
// it as "the roster could not hold a row" would have been the overclaim — it held exactly one
// shape, the shape that matches nothing, and the arm below pins what that shape does.
//
// This test stays enrolled after the climb rather than dissolving with the defect, because what it
// now guards is that the roster REMAINS authorable AT A REAL MODULE NAME (DESIGN §4b — a climb
// deletes the redundant production machinery, never the evidence).
const AUTHORED_LIKE_PRODUCTION: &[TransitionAdmission] = &[TransitionAdmission {
    label: "authored-in-a-const",
    subject: AdmissionSubject::Binding {
        module: "probe.consumer",
        in_declaration: "use_it",
        spelling: "widget",
    },
    disposition: NamespaceDeltaDisposition::TargetChanged,
}];

#[test]
fn a_row_authored_in_a_const_admits_its_delta_exactly_as_a_runtime_row_would() {
    let base = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_HOME),
    ];
    let head = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_OTHER),
    ];

    // THE POSITIVE CONTROL FIRST: without it, an admitted result is equally consistent with this
    // pair producing no delta at all, and the test would pass while proving nothing.
    let refused = compare("const_admission_before", &base, &head);
    assert!(
        !report_unadjudicated(&refused).is_empty(),
        "PLANT NEVER REACHED: the un-admitted arm must refuse, or admitting below proves nothing"
    );

    let admitted = compare_with(
        "const_admission_after",
        &base,
        &head,
        AUTHORED_LIKE_PRODUCTION,
    );
    assert!(
        admitted
            .deltas
            .iter()
            .any(|d| d.admitted_by.as_deref() == Some("authored-in-a-const")),
        "a row authored in a const must admit its exact subject, got: {:?}",
        admitted.deltas
    );
    assert!(
        admitted.stale_admissions.is_empty(),
        "a const-authored admission that matched must not also be reported stale"
    );
}

// AN EMPTY-MODULE ROW REFUSES, LOUDLY, AND THIS ARM RECORDS THAT THE OLD DEFECT WAS NEVER SILENT.
//
// Before `AdmissionSubject`, the only authorable row named the empty module. It was reported as an
// escape hatch that accepts a row and silently matches nothing; that reading was withdrawn, and
// this is the executing evidence for why. Such a row matches no delta, lands in
// `stale_admissions`, and the ADMITTED arm is conjoined on that list being empty — so it REFUSES,
// with its own named cause. The defect was a hatch with no working position, never one that lied.
#[test]
fn a_row_naming_the_empty_module_refuses_rather_than_admitting_silently() {
    let base = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_HOME),
    ];
    let head = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_OTHER),
    ];
    let admissions = [TransitionAdmission {
        label: "names-the-empty-module",
        subject: AdmissionSubject::Binding {
            module: "",
            in_declaration: "",
            spelling: "",
        },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    }];
    let report = compare_with("empty_module_row", &base, &head, &admissions);
    assert!(
        report.deltas.iter().all(|d| d.admitted_by.is_none()),
        "a row naming the empty module must admit nothing, got: {:?}",
        report.deltas
    );
    assert!(
        report
            .stale_admissions
            .iter()
            .any(|s| s.contains("names-the-empty-module")),
        "an unmatched row must be reported stale BY LABEL, or its refusal is unattributable: {:?}",
        report.stale_admissions
    );
}

#[test]
fn an_admission_naming_a_different_subject_does_not_admit_and_reports_stale() {
    let base = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_HOME),
    ];
    let head = [
        ("home.dag", HOME),
        ("other.dag", OTHER),
        ("consumer.dag", CONSUMER_IMPORTS_OTHER),
    ];
    // ONE FACT CHANGED FROM THE ARM ABOVE: the spelling the admission names. An admission
    // roster that admitted by disposition alone would green this, which is the coarse-grain
    // failure the ruling's "exact operator-authored transition admission" forbids.
    let admissions = [TransitionAdmission {
        label: "wrong-subject",
        subject: AdmissionSubject::Binding {
            module: "probe.consumer",
            in_declaration: "use_it",
            spelling: "gadget",
        },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    }];
    let report = compare_with("admission_wrong", &base, &head, &admissions);
    assert!(
        !report_unadjudicated(&report).is_empty(),
        "an admission for another subject must not admit this one"
    );
    assert_eq!(
        report.stale_admissions.len(),
        1,
        "an admission matching no delta must be reported stale, got: {:?}",
        report.stale_admissions
    );
}

// ── THE SHADOWING CEILING, ASSERTED RATHER THAN ASSUMED ──

#[test]
fn a_local_declaration_beside_an_import_is_a_two_member_set_not_a_winner() {
    // The wall cannot see WHICH occurrence took which declaration — that needs a
    // projector-emitted correspondence and is this class's next rung. What it must NOT do is
    // pick one and report a single binding, because a silent pick is precisely the mechanism
    // the namespace authority exists to delete. So the head here has two declarers on the
    // chain and the wall must say so.
    let shadowing = "module probe.consumer\n\nimport probe.home { widget }\n\ndata widget: String = \"local\"\n\nfn use_it() -> String { widget }\n";
    let report = compare(
        "shadow",
        &[("home.dag", HOME), ("consumer.dag", CONSUMER_IMPORTS_HOME)],
        &[("home.dag", HOME), ("consumer.dag", shadowing)],
    );
    assert!(
        dispositions_for(&report, "widget").contains(&NamespaceDeltaDisposition::NewAmbiguity),
        "a local declaration arriving beside an imported one must widen the set, not replace \
         it, got: {:?}",
        report.deltas
    );
}

// ── THE VOCABULARY JOIN: the host enum against the .dag coproduct it realizes ──
//
// The `match` in `auto_admitted` is exhaustive, which protects the host file's internal
// consistency and says NOTHING about the carrier: a variant added to the ruling and not here
// compiles perfectly. These two arms are the join, and the fixture boundary is where their red
// is authorable — a fixture may author a module AT THE AUTHORITY'S OWN PATH with any variant
// set it likes, which the accepted corpus cannot.

use v1_compiler::cli_run::namespace_wave_admission::{vocabulary_findings, DISPOSITION_LABELS};

fn interlock_fixture(variants: &[&str]) -> String {
    format!(
        "module gunbc.compiler_frontend_program_interlock\n\ntype NamespaceDeltaDisposition\n  = {}\n",
        variants.join("\n  | ")
    )
}

#[test]
fn a_vocabulary_matching_the_ruling_is_clean() {
    let dir = scratch("vocab_ok");
    author(
        &dir,
        "interlock.dag",
        &interlock_fixture(&DISPOSITION_LABELS),
    );
    let index = index_of(&dir);
    let findings = vocabulary_findings(&index);
    assert!(
        findings.is_empty(),
        "the exact ruling vocabulary must join cleanly, got: {findings:?}"
    );
}

#[test]
fn a_disposition_the_ruling_adds_and_the_host_lacks_refuses() {
    // ONE FACT CHANGED from the arm above.
    let mut variants: Vec<&str> = DISPOSITION_LABELS.to_vec();
    variants.push("SomeDispositionRuledLater");
    let dir = scratch("vocab_extra_there");
    author(&dir, "interlock.dag", &interlock_fixture(&variants));
    let index = index_of(&dir);
    let findings = vocabulary_findings(&index);
    assert!(
        findings
            .iter()
            .any(|f| f.contains("SomeDispositionRuledLater")),
        "a variant the ruling declares and the host does not carry must refuse, got: {findings:?}"
    );
}

#[test]
fn a_disposition_the_host_carries_and_the_ruling_lacks_refuses() {
    let variants: Vec<&str> = DISPOSITION_LABELS.iter().skip(1).copied().collect();
    let dir = scratch("vocab_extra_here");
    author(&dir, "interlock.dag", &interlock_fixture(&variants));
    let index = index_of(&dir);
    let findings = vocabulary_findings(&index);
    assert!(
        findings.iter().any(|f| f.contains(DISPOSITION_LABELS[0])),
        "a host disposition the ruling does not declare must refuse, got: {findings:?}"
    );
}

#[test]
fn an_absent_authority_refuses_rather_than_proceeding_on_the_hosts_say_so() {
    let dir = scratch("vocab_absent");
    author(&dir, "unrelated.dag", HOME);
    let index = index_of(&dir);
    let findings = vocabulary_findings(&index);
    assert_eq!(
        findings.len(),
        1,
        "an absent authority is the state in which nothing checks the vocabulary; it must \
         refuse, got: {findings:?}"
    );
}

// THE BASELINE-OBSERVABILITY ARMS (review 56449).
//
// These two are not mutations of the positive control above, because their subject is a
// different question: not "what disposition does this delta get" but "is there a baseline to
// compare against at all". A base file that does not parse used to return an EMPTY record
// vec, and empty is indistinguishable from "this module declared nothing" — so every row that
// file carried stopped being compared while the run still answered `Adjudicated`. Quieter, and
// silently uncovered.

/// RED for the empty-observation narrow at the base side.
///
/// The plant assertion is what keeps this arm honest: if the malformed source ever started
/// parsing, `base_records` would legitimately return `Ok` and this arm would pass for a reason
/// that has nothing to do with the wall. The control below is the other half — a source that
/// DOES parse must return records, or "refuses on everything" would satisfy the same assertion.
#[test]
fn a_base_side_source_that_does_not_parse_refuses_instead_of_reading_as_empty() {
    let malformed = "module probe.home\n\nfn broken( -> { this is not a program";
    let refused = base_records("dag/probe/home.dag", malformed);
    assert!(
        refused.is_err(),
        "an unparseable baseline is UNOBSERVABLE, not empty -- returning an empty record set \
         deletes every row that file would have carried from the comparison while the run still \
         reports Adjudicated. Got: {:?}",
        refused.map(|r| r.len())
    );

    // POSITIVE CONTROL ON THE SAME FUNCTION: a well-formed base source still yields records, so
    // the arm above is discriminating rather than a function that refuses unconditionally.
    let wellformed = "module probe.home\n\ndata widget: Int = 1\n";
    let read = base_records("dag/probe/home.dag", wellformed);
    assert!(
        read.as_ref().map(|r| !r.is_empty()).unwrap_or(false),
        "a base source that parses must produce records, else the refusal above proves nothing \
         about parsing. Got: {read:?}"
    );
}

/// A RENAME HAS TWO SIDES AND THE DIFF NAMES ONLY ONE OF THEM.
///
/// `git diff --name-only` reports a detected rename as its destination alone, so reading that
/// single list as both the head-touched set and the base-side set drops the source path: the
/// baseline for a renamed module is never read, every declaration in it looks newly added, and
/// the wall can refuse an ordinary `.dag` rename over a delta it invented. The arm drives the
/// rename-aware form and asserts the two sides come apart -- destination on the head side, source
/// on the base side -- and the controls beside it assert the other three statuses still land where
/// they belong, so the arm cannot pass by putting every path on both sides.
#[test]
fn a_rename_contributes_its_source_to_the_base_side_and_its_destination_to_the_head_side() {
    let z = "R096\0dag/probe/old.dag\0dag/probe/new.dag\0";
    let (head, base) = diff_sides(z);
    assert_eq!(
        head,
        vec!["dag/probe/new.dag".to_string()],
        "the head side of a rename is its DESTINATION"
    );
    assert_eq!(
        base,
        vec!["dag/probe/old.dag".to_string()],
        "the base side of a rename is its SOURCE -- dropping it is the baseline hole review 56471 \
         found, and it makes the whole module read as newly added"
    );

    let (head, base) = diff_sides("A\0dag/probe/added.dag\0");
    assert_eq!(head, vec!["dag/probe/added.dag".to_string()]);
    assert!(base.is_empty(), "an ADDED path has no base side to read");

    let (head, base) = diff_sides("D\0dag/probe/gone.dag\0");
    assert!(head.is_empty(), "a DELETED path has no head side");
    assert_eq!(base, vec!["dag/probe/gone.dag".to_string()]);

    let (head, base) = diff_sides("M\0dag/probe/same.dag\0");
    assert_eq!(head, vec!["dag/probe/same.dag".to_string()]);
    assert_eq!(
        base,
        vec!["dag/probe/same.dag".to_string()],
        "an ordinary modification is the one status whose two sides ARE the same path"
    );

    let (head, base) = diff_sides("M\0src/v1/stage0/src/lib.rs\0R100\0README.md\0LICENSE\0");
    assert!(
        head.is_empty() && base.is_empty(),
        "scope is applied per side: a non-`.dag` path enters neither"
    );
}

// ── THE FIELD-LABEL PAIR: what a name OCCURRENCE has to be before it can carry a verdict ──
//
// The reference channel this wall reads once collected EVERY authored name in a module's tree.
// A record literal's field label has an authored name, so `Row { widget: "x" }` contributed a
// reference to `widget` — and the supplier set a reference is asked for is a function of the
// CORPUS, so deleting an unrelated declaration of that spelling moved it to empty and the wall
// refused a correct cut. The specimen was gunbc#9106: twelve labels, twelve fabricated
// `NewUnresolvedness` rows. The two arms below are one mutation apart — the SAME deletion of
// the SAME spelling, reached once from a field label and once from a real reference — because
// collecting less is how a wall becomes a decoration, and only the pair can tell the repair
// from that.

const HOME_ROW: &str =
    "module probe.home\n\ntype Row { widget: String }\n\ndata other: String = \"o\"\n";

#[test]
fn deleting_a_declaration_a_record_field_label_merely_spells_carries_no_delta() {
    let base = "module probe.consumer\n\nimport probe.home { Row }\n\ndata widget: String = \"w\"\n\ndata sample: Row = Row { widget: \"x\" }\n";
    let head = "module probe.consumer\n\nimport probe.home { Row }\n\ndata sample: Row = Row { widget: \"x\" }\n";
    let report = compare(
        "field_label",
        &[("home.dag", HOME_ROW), ("consumer.dag", base)],
        &[("home.dag", HOME_ROW), ("consumer.dag", head)],
    );
    // THE PLANT REACHED THE WALL: the record literal survives both sides and the comparison is
    // non-empty, so the emptiness below is agreement rather than an index that saw nothing.
    assert!(
        report.population.binding_rows_compared > 0,
        "no binding row was compared, so the absence below is ignorance rather than agreement"
    );
    assert!(
        dispositions_for(&report, "widget").is_empty(),
        "`widget` here is a FIELD LABEL of `Row`, not a reference to the deleted `data widget`. \
         It never bound to that declaration and needs no supplier, so a delta about it is true \
         of the declaration and false of the site it names. got: {:?}",
        report
            .deltas
            .iter()
            .map(|d| (disposition_label(d.disposition), d.detail.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn deleting_a_declaration_a_body_still_references_is_still_unresolvedness() {
    // ONE MUTATION FROM THE ARM ABOVE: the same deletion of the same spelling, reached from a
    // real reference instead of a label. If this arm ever goes quiet the repair above has
    // stopped the wall seeing genuine unresolvedness, which is worse than the defect it fixed.
    let base = "module probe.consumer\n\ndata widget: String = \"w\"\n\nfn use_it() -> String { widget }\n";
    let head = "module probe.consumer\n\nfn use_it() -> String { widget }\n";
    let report = compare(
        "real_reference",
        &[("consumer.dag", base)],
        &[("consumer.dag", head)],
    );
    assert_eq!(
        dispositions_for(&report, "widget"),
        vec![NamespaceDeltaDisposition::NewUnresolvedness],
        "a body that names `widget` REFERENCES it; deleting its only declaration leaves the \
         reference denoting nothing and the wall must say so. got: {:?}",
        report.deltas
    );
}

#[test]
fn deleting_a_declaration_a_qualified_spelling_reaches_is_still_unresolvedness() {
    // THE PROJECTION HALF OF THE REPAIR, controlled. A field projection's MEMBER name stopped
    // being collected as a bare reference — but a module-qualified spelling is authored as the
    // same field-access shape, and its whole dotted spelling is still recorded, so this arm is
    // the one that fails if the repair took the spelling instead of the member.
    let home_head = "module probe.home\n\ndata other: String = \"o\"\n";
    let report = compare(
        "qualified_reference",
        &[
            ("home.dag", HOME),
            ("consumer.dag", CONSUMER_QUALIFIES_HOME),
        ],
        &[
            ("home.dag", home_head),
            ("consumer.dag", CONSUMER_QUALIFIES_HOME),
        ],
    );
    assert_eq!(
        dispositions_for(&report, "widget"),
        vec![NamespaceDeltaDisposition::NewUnresolvedness],
        "`probe.home.widget` reaches a declaration that no longer exists. got: {:?}",
        report.deltas
    );
}

// AN AUTHORED IMPORT CLAIM IS THE MEMBERSHIP FACT, NOT EVIDENCE TOWARD IT.
//
// `probe.coproduct` declares a coproduct; the consumer imports two of its VARIANTS and mentions
// them ONLY as match-arm pattern heads. `declaration_index`'s walk cannot reach either name IN
// PRINCIPLE -- `MatchPattern::VariantPattern.name` is a `String`, never a `Node`, so no node
// walker reaches it -- so the reference set is empty for that target and the add-side predicate
// answered "NO name in this module resolves into it" about a module the consumer explicitly
// imports.
//
// THE SCRUTINEE'S TYPE IS DELIBERATELY NOT NAMED IN THE CONSUMER, and that is the whole reason
// this fixture is trustworthy. An earlier revision wrote `fn decide(v: Verdict)`, which made the
// test PASS BOTH WAYS: `Verdict` in the signature is an ordinary reachable `Node`, so the
// reference-set path already answered membership and the import clause was never load-bearing.
// Measured -- green with the fix AND green with the fix reverted -- which is the decoration
// DESIGN §4b names as worse than absent, because it would have been cited as coverage. The
// scrutinee now arrives from a third module, so the ONLY names this import contributes are the
// two variant heads, which is exactly the population no walker reaches.
//
// THE RED IS AUTHORABLE HERE AND WAS AUTHORED. On the corrected fixture, with the add-side
// predicate reading the reference set alone, this arm reports `UnexplainedSubjectMotion`. It is
// retained as the regression control per §4b(4): the climb deletes the redundant production
// handling, never the executing evidence that the higher rung still holds.
const COPRODUCT_HOME: &str = "module probe.coproduct\n\ntype Verdict\n  = Accepted\n  | Refused\n";

const VERDICT_HOLDER: &str = "module probe.holder\n\nimport probe.coproduct { Verdict, Accepted }\n\nfn fetch() -> Verdict { Accepted }\n";

const PATTERN_ONLY_CONSUMER: &str = "module probe.consumer\n\nimport probe.holder { fetch }\nimport probe.coproduct { Accepted, Refused }\n\nfn decide() -> String {\n  match fetch() {\n    Accepted => \"yes\"\n    Refused => \"no\"\n  }\n}\n";

#[test]
fn membership_declared_by_an_import_whose_names_appear_only_in_pattern_arms() {
    let base = "module probe.consumer\n\nfn decide() -> String { \"unset\" }\n";
    let report = compare(
        "pattern_only_membership",
        &[
            ("coproduct.dag", COPRODUCT_HOME),
            ("holder.dag", VERDICT_HOLDER),
            ("consumer.dag", base),
        ],
        &[
            ("coproduct.dag", COPRODUCT_HOME),
            ("holder.dag", VERDICT_HOLDER),
            ("consumer.dag", PATTERN_ONLY_CONSUMER),
        ],
    );

    // PLANT ASSERTION FIRST, for the reason the sibling test states: a disposition read off a
    // delta that was never produced is a verdict about nothing.
    assert!(
        report.deltas.iter().any(|d| matches!(
            &d.subject,
            DeltaSubject::Membership { target, .. } if target == "probe.coproduct"
        )),
        "PLANT NEVER REACHED: no membership delta for probe.coproduct, got: {:?}",
        report.deltas
    );

    let dispositions = membership_dispositions(&report, "probe.coproduct");
    assert!(
        !dispositions.contains(&NamespaceDeltaDisposition::UnexplainedSubjectMotion),
        "an explicit `import probe.coproduct {{ .. }}` states the dependency; refusing it \
         because the reference set cannot see a pattern-arm name reads the weaker of two \
         representations of one fact, got: {:?}",
        dispositions
            .iter()
            .map(|d| disposition_label(*d))
            .collect::<Vec<_>>()
    );
    assert!(
        report_unadjudicated(&report).is_empty(),
        "nothing here needs adjudicating, got: {:?}",
        report_unadjudicated(&report)
            .iter()
            .map(|d| (disposition_label(d.disposition), d.detail.clone()))
            .collect::<Vec<_>>()
    );
}

// THE FIVE CONTROLS FOR THE AUTHORED-REFERENCE CHANNELS, AND THEY ARE ON THE REMOVAL SIDE
// DELIBERATELY -- WHICH IS THE WHOLE REASON THEY DISCRIMINATE ANYTHING.
//
// The add side cannot test this repair any more. #9490 made an authored import claim answer
// membership outright for ADD, so an add-side fixture over either channel goes green whether or
// not `referenced` can see the reference -- the disjunct answers first. That is precisely the
// green-with-and-without decoration this file already records one instance of, and building four
// more of them would have looked like coverage of a repair that was never exercised.
//
// The REMOVAL side has no such disjunct and cannot have one: an import claim states that a
// dependency was DECLARED, and removal asks whether anything was BOUND THROUGH it. Only seeing the
// reference answers that. So `membership_bound_through` is the one predicate these channels are
// load-bearing for, and every arm below drops an import while KEEPING the reference that needs it.
//
// WHAT THE FALSE GREEN WAS: the gate reported `UnusedSubjectMembershipRemoved` -- "removed, nothing
// bound through it" -- about an import whose name is still referenced in the very tree it just
// examined. That verdict tells a reader the import is dead, so the natural next action is to delete
// it, and the wall that exists to catch unsound namespace motion is the thing recommending it. A
// refusal is loud and gets investigated; this is a green, and nothing stops.
//
// EVERY ARM'S RED IS AUTHORABLE BY ONE ISOLATED MUTATION, and the three mutations below were
// EXECUTED after the reader was rebuilt on the parser's transport -- not carried over from the
// earlier Node-reading construction, which no longer exists. A receipt about deleted code is
// worse than no receipt, because it reads as evidence for what is actually there.
//
//   the pattern-head String read deleted:        2 failed
//     a_removed_import_whose_names_survive_only_as_pattern_heads_is_not_reported_unused
//     each_authored_channel_lands_in_the_field_whose_authority_supplied_it
//   the transport reader yielding nothing:       3 failed
//     a_removed_import_whose_name_survives_only_as_a_declared_field_type_is_not_reported_unused
//     a_removed_import_whose_name_survives_only_as_a_type_alias_right_hand_side_is_not_reported_unused
//     each_authored_channel_lands_in_the_field_whose_authority_supplied_it
//   the import-member filter deleted:            2 failed
//     an_import_member_name_is_not_an_authored_reference
//     removing_membership_nothing_bound_through_is_admitted_as_unused   <- PRE-EXISTING
//   nothing mutated:                             27 passed
//
// THE THIRD MUTATION IS THE ONE WORTH READING. Its second casualty is an arm nobody wrote for
// this change: consuming every `TypeOccurrence` the transport carries folds in IMPORT MEMBER
// NAMES, so each import becomes bound-through by its own member and
// `UnusedSubjectMembershipRemoved` stops being reachable at all. A disposition going
// permanently quiet is worse than the false green this file exists to close, and no arm added
// here would have caught it -- the existing suite did. That is the argument for running the
// whole file under each mutation rather than the arms one believes are relevant.
//
// THE MORE INFORMATIVE HALF REMAINS WHAT DOES NOT MOVE. Under BOTH channel mutations, the
// pre-existing add-side arm `membership_declared_by_an_import_whose_names_appear_only_in_pattern_arms`
// stayed GREEN: gunbc#9490's import-claim disjunct answers first, so an add-side fixture over
// these channels passes with the repair and without it. Anyone extending this family must run
// the revert arm before believing a new fixture covers anything.
//
// Two arms stay green under every mutation BY DESIGN and bound the repair from the other side:
// one requires that a locally declared spelling fabricate no support for a foreign target, and
// one requires that an ordinary Node-visible reference still be seen. Without them, every arm
// above is satisfied by a reader that simply collects more -- which, as the third mutation
// shows, is not a hypothetical failure mode here.

const FIELD_TYPE_HOME: &str =
    "module probe.payload\n\ntype Wrapper\n  = Wrapped { inner: String }\n  | Empty\n";

// The name appears in EXACTLY ONE position: a declared field type inside a variant payload. Nothing
// else in the module spells `Wrapper`, so the only thing that can support this membership is the
// channel that reads a field's declared type out of `inferred`.
const FIELD_TYPE_ONLY: &str = "module probe.consumer\n\ntype Holder = Held { w: Wrapper }\n";

const FIELD_TYPE_ONLY_WITH_IMPORT: &str =
    "module probe.consumer\n\nimport probe.payload { Wrapper }\n\ntype Holder = Held { w: Wrapper }\n";

#[test]
fn a_removed_import_whose_name_survives_only_as_a_declared_field_type_is_not_reported_unused() {
    let report = compare(
        "field_type_only_removal",
        &[
            ("payload.dag", FIELD_TYPE_HOME),
            ("consumer.dag", FIELD_TYPE_ONLY_WITH_IMPORT),
        ],
        &[
            ("payload.dag", FIELD_TYPE_HOME),
            ("consumer.dag", FIELD_TYPE_ONLY),
        ],
    );

    // PLANT ASSERTION FIRST: a disposition read off a delta that was never produced is a verdict
    // about nothing.
    assert!(
        report.deltas.iter().any(|d| matches!(
            &d.subject,
            DeltaSubject::Membership { target, .. } if target == "probe.payload"
        )),
        "PLANT NEVER REACHED: no membership delta for probe.payload, got: {:?}",
        report.deltas
    );

    let dispositions = membership_dispositions(&report, "probe.payload");
    assert!(
        !dispositions.contains(&NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved),
        "the head still references `Wrapper` as a declared field type, so this import was \
         load-bearing and its removal is a breakage -- reporting it as unused tells a reader to \
         delete a live import, got: {:?}",
        dispositions
            .iter()
            .map(|d| disposition_label(*d))
            .collect::<Vec<_>>()
    );
}

const PATTERN_ONLY_REMOVAL_BASE: &str = "module probe.consumer\n\nimport probe.holder { fetch }\nimport probe.coproduct { Accepted, Refused }\n\nfn decide() -> String {\n  match fetch() {\n    Accepted => \"yes\"\n    Refused => \"no\"\n  }\n}\n";

const PATTERN_ONLY_REMOVAL_HEAD: &str = "module probe.consumer\n\nimport probe.holder { fetch }\n\nfn decide() -> String {\n  match fetch() {\n    Accepted => \"yes\"\n    Refused => \"no\"\n  }\n}\n";

#[test]
fn a_removed_import_whose_names_survive_only_as_pattern_heads_is_not_reported_unused() {
    let report = compare(
        "pattern_only_removal",
        &[
            ("coproduct.dag", COPRODUCT_HOME),
            ("holder.dag", VERDICT_HOLDER),
            ("consumer.dag", PATTERN_ONLY_REMOVAL_BASE),
        ],
        &[
            ("coproduct.dag", COPRODUCT_HOME),
            ("holder.dag", VERDICT_HOLDER),
            ("consumer.dag", PATTERN_ONLY_REMOVAL_HEAD),
        ],
    );

    assert!(
        report.deltas.iter().any(|d| matches!(
            &d.subject,
            DeltaSubject::Membership { target, .. } if target == "probe.coproduct"
        )),
        "PLANT NEVER REACHED: no membership delta for probe.coproduct, got: {:?}",
        report.deltas
    );

    let dispositions = membership_dispositions(&report, "probe.coproduct");
    assert!(
        !dispositions.contains(&NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved),
        "the head still names `Accepted` and `Refused` as pattern heads, so the base import was \
         bound through -- reporting it unused is the false green this channel exists to close, \
         got: {:?}",
        dispositions
            .iter()
            .map(|d| disposition_label(*d))
            .collect::<Vec<_>>()
    );
}

// BOTH CHANNELS IN ONE MODULE, AND THE ASSERTION IS ABOUT DEDUPLICATION RATHER THAN PRESENCE.
// `referenced` is a set keyed on (enclosing declaration, name), so one name reached through two
// channels must land as ONE relation, not two. A projection that appended instead of inserting
// would still pass both arms above and would inflate every downstream count that reads this set.
const BOTH_CHANNELS_HOME: &str =
    "module probe.payload\n\ntype Wrapper\n  = Wrapped { inner: String }\n  | Empty\n";

const BOTH_CHANNELS_HEAD: &str = "module probe.consumer\n\ntype Holder = Held { w: Wrapper }\n\nfn describe(h: Holder) -> String {\n  match h.w {\n    Wrapped { inner: i } => i\n    Empty => \"empty\"\n  }\n}\n";

#[test]
fn each_authored_channel_lands_in_the_field_whose_authority_supplied_it() {
    let dir = scratch("both_channels");
    author(&dir, "payload.dag", BOTH_CHANNELS_HOME);
    author(&dir, "consumer.dag", BOTH_CHANNELS_HEAD);
    let index = index_of(&dir);
    let record = v1_compiler::cli_run::declaration_index::index_get(&index, "probe.consumer")
        .expect("PLANT NEVER REACHED: the consumer module must be indexed");

    // THE DECLARED FIELD TYPE COMES FROM THE PARSER, and it must arrive in the parser's field.
    // Asserting the UNION here would pass on a build that had quietly re-derived it from the
    // Node, which is exactly the second authority the ruling forbids -- so the arm pins the
    // channel, not merely the presence.
    let wrapper_rows: Vec<_> = record
        .authored_type_references
        .iter()
        .filter(|(_, name)| name == "Wrapper")
        .collect();
    assert_eq!(
        wrapper_rows.len(),
        1,
        "`Wrapper` is authored once as the declared type of `Held.w`, and the transport is what \
         sees it; a second row means the reader appended rather than inserted, and zero rows \
         means the transport was not consumed, got: {:?}",
        wrapper_rows
    );
    assert!(
        !record.referenced.iter().any(|(_, name)| name == "Wrapper"),
        "a declared field type must NOT also appear in the Node walk's own set -- if it does, \
         something reconstructed it from the tree and the two fields are no longer one \
         authority each, got: {:?}",
        record.referenced
    );

    // AND THE PATTERN HEAD COMES FROM THE AUTHORED STRING, because the parser mints no
    // occurrence for it. It belongs in `referenced` and must be absent from the parser's field:
    // were it to appear there, the transport would have started carrying it and this reader's
    // whole justification would have expired.
    assert!(
        record.referenced.iter().any(|(_, name)| name == "Wrapped"),
        "the variant-pattern head `Wrapped` must be an authored reference, got: {:?}",
        record.referenced
    );
    assert!(
        !record
            .authored_type_references
            .iter()
            .any(|(_, name)| name == "Wrapped"),
        "the transport does not stamp a pattern head today; if this fires, it now does, and the \
         String-reading block in `collect_reference_occurrences` should be deleted rather than \
         kept beside it, got: {:?}",
        record.authored_type_references
    );
}

// AN IMPORT MEMBER IS NOT A REFERENCE, and this arm exists because the first cut of the
// transport reader treated it as one. The transport stamps an import's member name as a
// `TypeOccurrence` enclosed by the import target, so consuming every TypeOccurrence made each
// import bound-through by its own member and `UnusedSubjectMembershipRemoved` became
// unreachable. That is a disposition going permanently quiet, which is worse than the false
// green this change closes -- and it was caught by a PRE-EXISTING arm rather than by review.
#[test]
fn an_import_member_name_is_not_an_authored_reference() {
    let dir = scratch("import_member_not_reference");
    author(
        &dir,
        "other.dag",
        "module probe.other\n\ndata gadget: String = \"g\"\n",
    );
    author(
        &dir,
        "consumer.dag",
        "module probe.consumer\n\nimport probe.other { gadget }\n\nfn use_it() -> String { \"x\" }\n",
    );
    let index = index_of(&dir);
    let record = v1_compiler::cli_run::declaration_index::index_get(&index, "probe.consumer")
        .expect("PLANT NEVER REACHED: the consumer module must be indexed");

    assert!(
        !record
            .authored_type_references
            .iter()
            .any(|(enclosing, _)| enclosing == "probe.other"),
        "`probe.other` is an import target, not a declaration this module declares, so nothing \
         may be keyed under it; a row here means an import member is being counted as a use of \
         itself, got: {:?}",
        record.authored_type_references
    );
}

// THE ARM THAT CATCHES A LAZY PROJECTION. `Wrapper` is declared LOCALLY here and named nowhere
// else, so nothing in this module supports membership in probe.payload. A projection that
// collected the spelling without caring where it resolves would fabricate that support -- and a
// fabricated membership looks exactly like a real one, which is why the operator ruling forbids
// minting membership from anything but authored references and why this arm is not optional.
const LOCAL_SHADOW_CONSUMER: &str = "module probe.consumer\n\ntype Wrapper = Local { n: String }\n\ntype Holder = Held { w: Wrapper }\n";

#[test]
fn a_locally_declared_spelling_fabricates_no_support_for_a_foreign_target() {
    let report = compare(
        "local_shadow",
        &[
            ("payload.dag", FIELD_TYPE_HOME),
            (
                "consumer.dag",
                "module probe.consumer\n\ndata unset: String = \"u\"\n",
            ),
        ],
        &[
            ("payload.dag", FIELD_TYPE_HOME),
            ("consumer.dag", LOCAL_SHADOW_CONSUMER),
        ],
    );

    let dispositions = membership_dispositions(&report, "probe.payload");
    assert!(
        dispositions.is_empty(),
        "probe.consumer declares its own `Wrapper` and imports nothing from probe.payload, so no \
         membership edge to it exists in either direction; any delta here is fabricated support, \
         got: {:?}",
        dispositions
            .iter()
            .map(|d| disposition_label(*d))
            .collect::<Vec<_>>()
    );
}

// THE UNCHANGED CONTROL. An ordinary Node-visible reference -- a function call through an imported
// name -- was reachable by the seven-slot walk before this change and must still be. This is the
// arm that catches a projection which broke what already worked, and without it every arm above is
// satisfied by a reader that collects too much.
const ORDINARY_HOME: &str = "module probe.lib\n\nfn helper() -> String { \"h\" }\n";
const ORDINARY_BASE: &str =
    "module probe.consumer\n\nimport probe.lib { helper }\n\nfn use_it() -> String { helper() }\n";
const ORDINARY_HEAD: &str = "module probe.consumer\n\nfn use_it() -> String { \"inlined\" }\n";

#[test]
fn an_ordinary_node_visible_reference_removal_is_still_reported_unchanged() {
    let report = compare(
        "ordinary_control",
        &[("lib.dag", ORDINARY_HOME), ("consumer.dag", ORDINARY_BASE)],
        &[("lib.dag", ORDINARY_HOME), ("consumer.dag", ORDINARY_HEAD)],
    );

    assert!(
        report.deltas.iter().any(|d| matches!(
            &d.subject,
            DeltaSubject::Membership { target, .. } if target == "probe.lib"
        )),
        "PLANT NEVER REACHED: no membership delta for probe.lib, got: {:?}",
        report.deltas
    );
}

// A SIXTH ARM, ADDED BECAUSE THE MEASUREMENT SAID SO RATHER THAN BECAUSE THE ARGUMENT DID.
//
// A type alias's right-hand side is the same authored channel as a declared field type -- the
// parser parks BOTH in `inferred` -- so the block above was expected to cover it. Expected is not
// measured, and the projection is generic over the slot rather than special-cased to a field, so
// whether it reaches an alias RHS is a fact about the parser's shape and not about the reader's
// intent. Executed both ways: with channel two present the wall reports
// `SameDeclarationIdentityRebind`; with channel two deleted it reports
// `UnusedSubjectMembershipRemoved`. That is a real specimen of the same false green, closed by the
// same block, so it is an arm here rather than a second change.
const ALIAS_RHS_HOME: &str = "module probe.payload\n\ntype QualifiedName = Qn { text: String }\n";
const ALIAS_RHS_BASE: &str = "module probe.consumer\n\nimport probe.payload { QualifiedName }\n\ntype ModulePath = QualifiedName\n";
const ALIAS_RHS_HEAD: &str = "module probe.consumer\n\ntype ModulePath = QualifiedName\n";

#[test]
fn a_removed_import_whose_name_survives_only_as_a_type_alias_right_hand_side_is_not_reported_unused(
) {
    let report = compare(
        "alias_rhs_removal",
        &[
            ("payload.dag", ALIAS_RHS_HOME),
            ("consumer.dag", ALIAS_RHS_BASE),
        ],
        &[
            ("payload.dag", ALIAS_RHS_HOME),
            ("consumer.dag", ALIAS_RHS_HEAD),
        ],
    );

    assert!(
        report.deltas.iter().any(|d| matches!(
            &d.subject,
            DeltaSubject::Membership { target, .. } if target == "probe.payload"
        )),
        "PLANT NEVER REACHED: no membership delta for probe.payload, got: {:?}",
        report.deltas
    );

    let dispositions = membership_dispositions(&report, "probe.payload");
    assert!(
        !dispositions.contains(&NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved),
        "the head's alias still names `QualifiedName`, so the dropped import was bound through \
         it, got: {:?}",
        dispositions
            .iter()
            .map(|d| disposition_label(*d))
            .collect::<Vec<_>>()
    );
}

// AND ONE MEASURED NEGATIVE, RECORDED RATHER THAN ENROLLED, because an arm here would be the
// permanently-green decoration DESIGN §4b names as worse than absent.
//
// A function's RETURN TYPE was the other candidate from the six-site parser reading, and the same
// argument covered it. It does not reproduce: measured, `fn make() -> QualifiedName` with the
// import dropped reports `SameDeclarationIdentityRebind` WITH channel two and
// `SameDeclarationIdentityRebind` WITHOUT it -- identical, because a return type is already
// reachable through an ordinary Node slot, so it was never invisible. An arm asserting the correct
// verdict there would pass with this repair and pass with it reverted, carrying no information
// about the thing it appeared to cover. The argument was good and the receipt refused it, which is
// the whole reason the receipt is taken.
