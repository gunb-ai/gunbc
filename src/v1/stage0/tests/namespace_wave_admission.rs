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
    adjudicate, base_records, diff_sides, disposition_label, report_unadjudicated, DeltaSubject,
    NamespaceDeltaDisposition, TransitionAdmission, WaveAdmissionReport,
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
        subject: DeltaSubject::Binding {
            module: "probe.consumer".to_string(),
            in_declaration: "use_it".to_string(),
            spelling: "widget".to_string(),
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
        subject: DeltaSubject::Binding {
            module: "probe.consumer".to_string(),
            in_declaration: "use_it".to_string(),
            spelling: "gadget".to_string(),
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
