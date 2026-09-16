//! THE WAVE-ADMISSION WALL: what a namespace change does to closure, subject membership
//! and binding, adjudicated before it merges.
//!
//! WHY IT EXISTS. `gunbc.plans.import_namespace_program` §9 records that "no CI mechanism
//! enforces any of this — no ratchet, no phase, no gate over the import population", and the
//! 2026-08-26 operator ruling at `gunbc.compiler_frontend_program_interlock` makes that a
//! BLOCKER: no change that can alter which modules enter a subject, or what an occurrence
//! denotes, may merge before this wall exists. `milestone_prerequisites` gates
//! `NamespaceFirstSemanticWave` on `NamespaceWaveAdmissionEnrolled` by name. A plan that reads
//! as governed when it is not is worse than one that reads as unguarded.
//!
//! THE ADMISSION PREDICATE, AND THE ONE WORD IT TURNS ON. The wall admits when the UNADJUDICATED
//! delta is empty, never when the delta is empty: expected cut motion may occur, unevaluated or
//! unexplained motion may not. A zero-delta wall would refuse the cut it governs and be
//! weakened.
//!
//! WHY THE CHANGE CLASS IS DERIVED AND NEVER DECLARED. The ruling's `NamespaceChangeClass`
//! splits preparatory work from work altering membership or binding. The wall computes the
//! delta rather than asking the author, so `PreparatoryNoSemanticMotion` is a MEASURED property
//! of a diff, not a PR-body claim. Construction over validation (DESIGN §5).
//!
//! ── THE GRAIN, AND WHY IT IS NOT OCCURRENCE GRAIN ──
//!
//! An occurrence-grain delta between two arbitrary trees IS NOT COMPUTABLE — a closed result.
//! `v2.workflow.legacy_binding_delta` states it: `std.occurrence_identity`'s scope law forbids
//! filename, span, authored name, structural equality and content hash as identity inputs, and
//! an `OccurrenceId` is a monotone counter in walk order, so it encodes POSITION and shifts under
//! any edit above it. A cross-compile correspondence is something a TRANSFORMATION EMITS, and
//! between a merge base and a PR head there is none. So this wall reads the grain
//! `legacy_binding_observation` `legacy_subject_identity` folds for its own subjects: authored
//! containment identity — module path, enclosing declaration, and the LEAF SEGMENT of the
//! reference. The leaf, not the spelling: the segments before it name the ROUTE, the leaf the
//! DECLARATION. Keyed on the spelling, qualifying a reference would read as one name losing its
//! declaration and another appearing, and requalification is the namespace program's core
//! motion. See `binding_rows`.
//!
//! WHAT THAT COSTS, NAMED RATHER THAN LEFT TO BE FOUND. Two occurrences of one spelling inside
//! one declaration — a `let` binder shadowing an imported name, a match-arm binder — share a
//! row. The repair is NOT to pick a winner (the silent selection the namespace authority exists
//! to delete): a row's value is the SET of declaring identities the spelling admits and a delta
//! is a set difference, so shadowing is REPRESENTED, not collapsed. Which occurrence took which
//! member is beyond the ceiling; the next rung is a projector-emitted correspondence (E.1,
//! `ProjectionProvenanceEntry`), not a finer key invented here.
//!
//! ── WHY THE REFERENCE CHANNEL IS NOT THE IMPORT CHANNEL ──
//!
//! A wall reading bindings only through import members would see the import-name universe
//! deleted and then nothing — blind on the change it gates. So the binding channel is every
//! authored NAME OCCURRENCE in a module's own parsed tree (`ModuleDeclarationRecord::referenced`),
//! resolved independently; it never depended on the construct being cut.
//!
//! ── WHAT THIS DOES WITH CLOSURE, AND THE ARM IT DELIBERATELY DOES NOT AUTHOR ──
//!
//! Closure is a pure function of membership, so "closure moved, no membership moved" is not a
//! state any fixture can author; an arm for it would be permanently green — the decoration
//! DESIGN §4b calls worse than absent. Closure is MEASURED and ATTRIBUTED: every closure row is
//! grouped under the membership delta generating it, so a refusal names its blast radius.
//! Adjudicating the consequence as well as the generator would be a second representation of
//! one fact (DESIGN §2/§3).

// CLIPPY ROSTER -- 5 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::manual_contains,  // 1
    clippy::useless_format,  // 4
)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::rc::Rc;

use crate::cli_run::declaration_index::{
    import_surface_has, index_get, index_records, DeclarationIndex, ModuleDeclarationRecord,
};
use crate::v1_std_core::qualified_last_segment;

/// The declaration identity of an ambient kernel type in binding rows.
///
/// Kernel types have no declaring module, but that does not make them unresolved:
/// `v1.compiler.resolve` appends `std.types.kernel_type_set` to every visible-name set. Keeping
/// this identity distinct from every module path prevents an empty candidate set from conflating
/// "resolved by the kernel" with "denotes nothing".
const KERNEL_DECLARATION_IDENTITY: &str = "<kernel>";

/// The nine dispositions of `gunbc.compiler_frontend_program_interlock`
/// `NamespaceDeltaDisposition`, realized for the host reader.
///
/// THE VOCABULARY IS THE CARRIER'S, NOT THIS FILE'S. The `.dag` coproduct is the authority; the
/// auto-admitted/refusing partition is the operator's, recorded there and transcribed only as
/// the exhaustive match below, so a variant added here and not there fails to compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NamespaceDeltaDisposition {
    SameDeclarationIdentityRebind,
    UnusedSubjectMembershipRemoved,
    ExplicitlyEvaluatedZeroDelta,
    TargetChanged,
    NewAmbiguity,
    NewUnresolvedness,
    NewPoolCoincidenceResolution,
    AuthoredReferenceResolution,
    UnexplainedSubjectMotion,
    NotEvaluated,
}

/// FREE FUNCTIONS RATHER THAN INHERENT METHODS, THROUGHOUT THIS MODULE: `std.decl_ref` offers
/// `WholeDeclaration` or `NamedField`, neither naming an `impl` method, so a method would be an
/// UNCITABLE seed-growth item the roster in `gunbc.namespace_wave_admission` cannot enumerate.
/// `gunbc.declaration_index_seed_growth` records the same decision.
pub fn disposition_label(d: NamespaceDeltaDisposition) -> &'static str {
    match d {
        NamespaceDeltaDisposition::SameDeclarationIdentityRebind => "SameDeclarationIdentityRebind",
        NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved => {
            "UnusedSubjectMembershipRemoved"
        }
        NamespaceDeltaDisposition::ExplicitlyEvaluatedZeroDelta => "ExplicitlyEvaluatedZeroDelta",
        NamespaceDeltaDisposition::TargetChanged => "TargetChanged",
        NamespaceDeltaDisposition::NewAmbiguity => "NewAmbiguity",
        NamespaceDeltaDisposition::NewUnresolvedness => "NewUnresolvedness",
        NamespaceDeltaDisposition::NewPoolCoincidenceResolution => "NewPoolCoincidenceResolution",
        NamespaceDeltaDisposition::AuthoredReferenceResolution => "AuthoredReferenceResolution",
        NamespaceDeltaDisposition::UnexplainedSubjectMotion => "UnexplainedSubjectMotion",
        NamespaceDeltaDisposition::NotEvaluated => "NotEvaluated",
    }
}

/// The operator's partition, verbatim from the carrier: a same-declaration-identity rebind, a
/// removal of genuinely unused subject membership, and an explicitly evaluated zero delta are
/// auto-admitted. Every other disposition — including `NotEvaluated` — refuses unless an exact
/// transition admission names it.
///
/// THE MATCH IS EXHAUSTIVE, which protects only THIS FILE's consistency: a variant added to the
/// `.dag` authority and not here compiles. `vocabulary_findings` closes that.
pub fn disposition_auto_admitted(d: NamespaceDeltaDisposition) -> bool {
    match d {
        NamespaceDeltaDisposition::SameDeclarationIdentityRebind
        | NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved
        | NamespaceDeltaDisposition::ExplicitlyEvaluatedZeroDelta
        | NamespaceDeltaDisposition::AuthoredReferenceResolution => true,
        NamespaceDeltaDisposition::TargetChanged
        | NamespaceDeltaDisposition::NewAmbiguity
        | NamespaceDeltaDisposition::NewUnresolvedness
        | NamespaceDeltaDisposition::NewPoolCoincidenceResolution
        | NamespaceDeltaDisposition::UnexplainedSubjectMotion
        | NamespaceDeltaDisposition::NotEvaluated => false,
    }
}

/// The `.dag` coproduct this enum realizes, and the declaration whose variants it must equal.
pub const DISPOSITION_AUTHORITY_MODULE: &str = "gunbc.compiler_frontend_program_interlock";
pub const DISPOSITION_AUTHORITY_DECL: &str = "NamespaceDeltaDisposition";

/// Every label this host enum carries, in the authority's own spelling.
pub const DISPOSITION_LABELS: [&str; 10] = [
    "SameDeclarationIdentityRebind",
    "UnusedSubjectMembershipRemoved",
    "ExplicitlyEvaluatedZeroDelta",
    "TargetChanged",
    "NewAmbiguity",
    "NewUnresolvedness",
    "NewPoolCoincidenceResolution",
    "AuthoredReferenceResolution",
    "UnexplainedSubjectMotion",
    "NotEvaluated",
];

/// Refuse if the host realization and the `.dag` authority disagree about the vocabulary.
///
/// WHY THIS EXISTS AT ALL. The enum above is a SECOND REPRESENTATION of a `.dag` coproduct, and
/// DESIGN §3 says two representations diverge on the first amendment. The exhaustive `match` in
/// `auto_admitted` protects only INTERNAL consistency: a variant added to the carrier and not
/// here compiles, and the wall silently adjudicates against a superseded vocabulary.
///
/// IT IS A JOIN AND NOT A COUNT: set equality over variant names in both directions, so `here
/// and not there` and `there and not here` are separate findings. The index already carries the
/// authority's variants, so this is one keyed lookup and no walk.
///
/// AND ITS ABSENCE REFUSES. An authority module not in the index — renamed, deleted, or moved
/// out of the swept roots — is the state in which nothing checks the vocabulary, not permission
/// to proceed on the host's say-so.
pub fn vocabulary_findings(index: &DeclarationIndex) -> Vec<String> {
    let Some(record) = index_get(index, DISPOSITION_AUTHORITY_MODULE) else {
        return vec![format!(
            "the disposition authority `{DISPOSITION_AUTHORITY_MODULE}` is absent from the \
             index, so nothing joins this host enum to the ruling it realizes"
        )];
    };
    let Some(authored) = record.decl_fields.get(DISPOSITION_AUTHORITY_DECL) else {
        return vec![format!(
            "`{DISPOSITION_AUTHORITY_MODULE}` declares no `{DISPOSITION_AUTHORITY_DECL}`"
        )];
    };
    let here: BTreeSet<String> = DISPOSITION_LABELS.iter().map(|l| l.to_string()).collect();
    let mut findings = Vec::new();
    for missing in authored.difference(&here) {
        findings.push(format!(
            "`{DISPOSITION_AUTHORITY_DECL}` declares `{missing}` and this host enum does not \
             carry it — the wall would adjudicate against a superseded vocabulary"
        ));
    }
    for extra in here.difference(authored) {
        findings.push(format!(
            "this host enum carries `{extra}` and `{DISPOSITION_AUTHORITY_DECL}` does not \
             declare it — a disposition with no authority"
        ));
    }
    findings
}

/// What a delta is ABOUT. Two shapes, because membership and binding are two questions:
/// one is which modules enter a subject, the other is what a name denotes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeltaSubject {
    /// `module` gained or lost `target` as a direct dependency.
    Membership { module: String, target: String },
    /// A spelling inside one top-level declaration changed which declarations it admits.
    Binding {
        module: String,
        in_declaration: String,
        spelling: String,
    },
}

pub fn delta_subject_render(subject: &DeltaSubject) -> String {
    match subject {
        DeltaSubject::Membership { module, target } => format!("membership {module} -> {target}"),
        DeltaSubject::Binding {
            module,
            in_declaration,
            spelling,
        } => format!("binding {module}::{in_declaration} `{spelling}`"),
    }
}

/// One adjudicated delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceDelta {
    pub subject: DeltaSubject,
    pub disposition: NamespaceDeltaDisposition,
    /// The two sides, rendered. Never a summary: a refusal a reader cannot act on withholds the
    /// analysis.
    pub detail: String,
    /// Modules whose transitive closure moves because of THIS delta, when that question was
    /// ASKED. Closure is a pure function of membership, so only a membership delta generates
    /// closure motion and only a membership row can answer. `None` is the binding row saying the
    /// question does not apply to it — it is NOT a measured zero, and the renderer omits the
    /// clause entirely rather than printing one. The field was a bare `usize` until 2026-09-01,
    /// which gave those two states one spelling: every binding row carried a literal `0` and
    /// rendered identically to a membership row whose closure genuinely moved nothing, so a
    /// reader could not tell an unasked question from a measured answer.
    pub closure_blast_radius: Option<usize>,
    /// Set when a transition admission covers this exact subject and disposition.
    pub admitted_by: Option<String>,
}

/// The authored pattern naming one exact runtime delta subject.
///
/// Permission is the directory of authored `.dag` rows, not a const and not a value computed
/// from observed deltas. Runtime observations remain owned `DeltaSubject` values — a distinct
/// type from an authored pattern. Binding `module` / `in_declaration` realize
/// `gunbc.namespace.transition_admission` `AdmissionSubject.Binding.enclosing` (`DeclarationRef`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionSubject {
    Membership {
        module: String,
        target: String,
    },
    Binding {
        module: String,
        in_declaration: String,
        spelling: String,
        /// Exact candidate set after the admitted transition, checked at the head before
        /// admission and at the base to derive consumption. A set, never merely one member
        /// whose presence could hide unexpected candidates.
        expected_candidates: Vec<String>,
    },
}

pub fn admission_subject_matches(pattern: &AdmissionSubject, subject: &DeltaSubject) -> bool {
    match (pattern, subject) {
        (
            AdmissionSubject::Membership { module, target },
            DeltaSubject::Membership {
                module: observed_module,
                target: observed_target,
            },
        ) => module == observed_module && target == observed_target,
        (
            AdmissionSubject::Binding {
                module,
                in_declaration,
                spelling,
                expected_candidates: _,
            },
            DeltaSubject::Binding {
                module: observed_module,
                in_declaration: observed_declaration,
                spelling: observed_spelling,
            },
        ) => {
            module == observed_module
                && in_declaration == observed_declaration
                && spelling == observed_spelling
        }
        _ => false,
    }
}

pub fn admission_subject_render(subject: &AdmissionSubject) -> String {
    match subject {
        AdmissionSubject::Membership { module, target } => {
            format!("membership {module} -> {target}")
        }
        AdmissionSubject::Binding {
            module,
            in_declaration,
            spelling,
            expected_candidates,
        } => format!("binding {module}::{in_declaration} `{spelling}` -> {expected_candidates:?}"),
    }
}

/// An operator-authored admission for one exact subject under one exact disposition.
///
/// THE GRAIN IS EXACT ON PURPOSE, AND THE COARSE FORM IS NOT BUILT HERE. The first semantic wave
/// is expected to produce THOUSANDS of transitions (measured by the owning session against the
/// import-strip receipts' class taxonomy — stale as a count, sound as an order of magnitude), so
/// a wave will want a class admission bounded BY ENUMERATED IDENTITY — "these exact bindings,
/// from the pre-deletion baseline observation, become unresolved" — never by a predicate like
/// "unresolvedness is expected during the wave", which admits everything and zeroes the wall's
/// deficit frequency (DESIGN §5, the absorbing fallback). That carrier is NOT authored here: it
/// would have no consumer until the first wave (DESIGN §6, experimental residue). What is fixed
/// now is that its population must be an enumeration, never a predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionAdmission {
    pub label: String,
    pub subject: AdmissionSubject,
    pub disposition: NamespaceDeltaDisposition,
    /// The pull request that deletes this row once its own merge has consumed it, authored by the
    /// row's owner BEFORE the owning change is enqueued.
    pub deletion_follow_up: DeletionFollowUp,
    /// The pull request that authored this row -- the owner a consumed-row receipt names. Typed
    /// rather than read out of `label`, whose `gunbc#N` prefix is a convention nothing enforces.
    pub owner_pull_request: u32,
}

/// ONE OWNED CONSUMED ROW, AS A TYPED AND LOCATED RECEIPT (lane ruling X, fierce-lark-661,
/// 2026-09-13). A row consumed at the base whose owner authored a deletion follow-up is a declared
/// frontier, not a refusal: the owner has DECLARED a deletion follow-up number -- this module reads
/// no forge, so nothing here establishes that the referenced pull request exists, is open, or
/// deletes these rows -- and refusing the next unrelated
/// composition while that follow-up is still open would bill a bystander for the owner's window --
/// the §5 externalization review 65313 found the previous arms still committed. Every run that sees
/// the row prints this receipt, so the window is visible per run.
///
/// WHAT THIS BINARY CANNOT SEE, AND WHO DOES. Whether the follow-up is OPEN (frontier), CLOSED
/// UNMERGED (the row is an orphan and must refuse at its next touch), or MERGED with the row still
/// present (the deletion landed without deleting, and must refuse) is forge state, and this module
/// reads no forge -- AND NO EXECUTING ROUTE IN THIS REPOSITORY READS IT EITHER (review 65476,
/// verified: there is no `landing_tally` symbol, and nothing outside this module consumes
/// `deletion_follow_up`). The pre-enqueue landing procedure that reads it is out-of-band human
/// review, so the follow-up's forge state is OUTSIDE THE MODELED GUARANTEE (DESIGN section 4b)
/// rather than a checked property, and this receipt exists to make that unchecked window visible
/// on every run. The trigger that brings it inside is the typed repository/forge read this
/// module's CLASS B acquisition boundary already waits on: when a fold can ask the forge for a
/// pull request's state, these three dispositions become a wall instead of a printed receipt.
/// A receipt
/// is also not a verdict: the retained roster-touch and `base == head` rules still refuse runs that
/// carry owned rows, so a printed receipt and a refusal on the same run are the expected
/// coexistence, not a contradiction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedRowReceipt {
    pub label: String,
    pub owner_pull_request: u32,
    pub deletion_follow_up_pull_request: u32,
}

/// WHO OWES THE DELETION, AUTHORED WHERE THE DEBT IS CREATED.
///
/// `deletion_follow_up` RECORDS A DEBT; NOTHING ENFORCES IT IN ADVANCE, AND THAT IS DELIBERATE.
/// The field stays on the row and stays read, so a row can say which pull request is meant to
/// delete it once its owner lands. What was removed (2026-09-16, operator ruling) are the two
/// `merge_group` arms that refused on its absence: `OwnerFollowUpAbsent`, charging a row's owner at
/// their own queue run, and `ConsumedRowOwnerChargeBypassed`, charging a BYSTANDER composition for
/// someone else's unauthored follow-up.
///
/// THE DEBT IS STILL ENFORCED WHERE IT BECOMES REAL. A consumed row still refuses at landing
/// (`base == head`) or on a roster-source edit, so the deletion is still compelled -- just at the
/// point the row is actually spent rather than in advance of it.
///
/// WHY THE ADVANCE CHARGE WAS NOT WORTH ITS COST. Its admitting side was free: the wall established
/// that a NUMBER was authored and nothing more. Whether that number named an open pull request that
/// deletes these rows was checked by no executing route in this repository -- not by this binary,
/// which reads no forge, and not by any other consumer (review 65476, verified). A fabricated
/// number passed it and was caught by nothing. A check whose RED is authorable but whose GREEN is
/// free buys the APPEARANCE of a wall (DESIGN section 4b), and the second arm made a bystander pay
/// for it. Reference verification remains out of band or absent, which is now stated rather than
/// implied by a wall that could not perform it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletionFollowUp {
    NotAuthored,
    PullRequest(u32),
}

/// THE CI EVENT WHOSE SUBJECT THIS RUN ADJUDICATES. The consumption obligation differs by subject,
/// so the verdict needs the event as an input rather than inferring it from the shape of the diff:
/// a `merge_group` composition is the tree about to BECOME the default branch, so it is where the
/// owner's follow-up is charged. A base-consumed row seen there does NOT by itself mean that charge
/// was bypassed -- review 65313 disproved that inference -- since it may equally be an owned row
/// inside its declared deletion window, which is why `adjudicate` partitions on the authored
/// follow-up rather than refusing on consumption alone. `Local` is a run with no CI event at all (an author's machine); it takes the
/// pull_request policy. An event name this enum does not model is refused by
/// `adjudication_event_from_name` rather than defaulted to either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjudicationEvent {
    PullRequest,
    MergeGroup,
    Push,
    WorkflowDispatch,
    Local,
}

pub fn adjudication_event_from_name(name: Option<&str>) -> Result<AdjudicationEvent, String> {
    match name {
        None => Ok(AdjudicationEvent::Local),
        Some("pull_request") => Ok(AdjudicationEvent::PullRequest),
        Some("merge_group") => Ok(AdjudicationEvent::MergeGroup),
        Some("push") => Ok(AdjudicationEvent::Push),
        Some("workflow_dispatch") => Ok(AdjudicationEvent::WorkflowDispatch),
        Some(other) => Err(format!(
            "GITHUB_EVENT_NAME `{other}` is not an event the wave-admission consumption policy \
             models, so which obligation this run owes is unknown; refusing rather than applying \
             the pull_request policy to it"
        )),
    }
}

/// The denominators a green must name (DESIGN §5): a run that cannot say what it covered is an
/// instrument failure wearing coverage's clothes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WaveAdmissionPopulation {
    /// Modules present on BOTH sides — the only ones a delta can be about.
    pub modules_compared: usize,
    pub modules_added: usize,
    pub modules_removed: usize,
    pub membership_edges_head: usize,
    pub binding_rows_compared: usize,
    /// Closure rows that moved, over all modules. Attributed, never adjudicated.
    pub closure_rows_moved: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveAdmissionReport {
    pub population: WaveAdmissionPopulation,
    pub deltas: Vec<NamespaceDelta>,
    /// Admission rows that matched no delta in this run.
    pub stale_admissions: Vec<String>,
    /// Rows whose admitted relocation the BASE already satisfies — consumed by their own merge.
    /// Typed receipts, never refusals for an unrelated run: the deletion obligation they carry
    /// stands on the roster's own next touch (see the executor's roster-touched arm). Entered
    /// only on the POSITIVE proof `admission_satisfied_at`, never as the else-arm of "did
    /// not match a delta" — a row provable against neither side stays an UnmatchedAdmission
    /// refusal in `stale_admissions`.
    pub consumed_admissions: Vec<String>,
    /// Rows USED to admit a delta in this run whose owner authored no deletion follow-up. Each is
    /// satisfied at the candidate, so it will be consumed when the candidate lands; on a
    /// merge_group run that is the owner's refusal.
    pub used_without_follow_up: Vec<String>,
    /// The subset of `consumed_admissions` whose owner authored NO deletion follow-up.
    ///
    /// NO PRODUCTION READER SINCE gunbc#11481, FLAGGED RATHER THAN HIDDEN. This was the population
    /// `ConsumedRowOwnerChargeBypassed` refused on; that arm is removed and declared as the drop
    /// `gunbc.rung_drop.consumed_row_owner_charge_unenforced`. The field is still POPULATED and is
    /// read only by tests, so it is a DESIGN 3c dangling field today -- kept because it is the exact
    /// population that drop's restoration trigger has to re-cover, and deleting it would discard the
    /// one derivation a restoration would need. Its honest disposition is decided when that drop is
    /// retired: consumed by the restored charge, or removed with the drop row.
    pub consumed_without_follow_up: Vec<String>,
    /// The complement: consumed rows with an authored follow-up, carried as receipts.
    pub owned_consumed_receipts: Vec<ConsumedRowReceipt>,
}

/// The wall's verdict: every delta is either auto-admitted or named by an admission.
pub fn report_unadjudicated(report: &WaveAdmissionReport) -> Vec<&NamespaceDelta> {
    report
        .deltas
        .iter()
        .filter(|d| !disposition_auto_admitted(d.disposition) && d.admitted_by.is_none())
        .collect()
}

// ---------------------------------------------------------------------------
// FACTS — derived from one module's own record, never across files
// ---------------------------------------------------------------------------

/// The modules one module reaches directly: its import targets, plus every dotted spelling
/// in its own tree whose prefix IS a module.
///
/// BOTH CHANNELS, ON BOTH SIDES, is what survives the cut: before Step 1 the import claims carry
/// most of it, after Step 1 the reference channel carries all of it. The FUNCTION does not
/// change, so base and head are measured by one instrument.
fn direct_membership(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for claim in &record.imports {
        if !claim.target.is_empty() && index_get(index, &claim.target).is_some() {
            out.insert(claim.target.clone());
        }
    }
    // Both authored-reference channels, the union `membership_bound_through` takes, for the same
    // measured reason: a declared type is parked in `inferred`, unvisited by the walk behind
    // `referenced`, so a module reaching another only via a declared type contributed no edge
    // -- closure and blast radius under-reported.
    for (_, spelling) in record
        .referenced
        .iter()
        .chain(record.authored_type_references.iter())
    {
        if let Some((module, _leaf)) = module_prefix_of(index, spelling) {
            if module != record.module_path {
                out.insert(module);
            }
        }
    }
    out.remove(&record.module_path);
    out
}

/// The longest dotted prefix of `spelling` that is a module in the index, with the segment
/// that follows it. `None` when no prefix names a module — a host name, a kernel name, or an
/// ordinary field access on a value.
fn module_prefix_of(index: &DeclarationIndex, spelling: &str) -> Option<(String, String)> {
    let segments: Vec<&str> = spelling.split('.').collect();
    if segments.len() < 2 {
        return None;
    }
    // Longest first: `a.b.c` prefers module `a.b` over module `a`.
    for split in (1..segments.len()).rev() {
        let candidate = segments[..split].join(".");
        if index_get(index, &candidate).is_some() {
            return Some((candidate, segments[split].to_string()));
        }
    }
    None
}

/// The declaring identities a spelling admits inside one module — module paths for authored
/// declarations and `<kernel>` for an ambient kernel type. An empty set is
/// unresolved-at-this-grain; two or more is ambiguity.
fn declaring_candidates(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
    spelling: &str,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some((module, leaf)) = module_prefix_of(index, spelling) {
        if let Some(target) = index_get(index, &module) {
            if import_surface_has(target, &leaf) {
                out.insert(declarer_of(index, &module, &leaf));
            }
        }
        return out;
    }
    if spelling.contains('.') {
        return out;
    }
    let locally_declared = record.declared.contains(spelling) || record.variants.contains(spelling);
    if locally_declared {
        out.insert(record.module_path.clone());
    }
    // Locals precede kernel names, and kernel names precede imports in the compiler's one
    // precedence authority. A structural `String` import therefore never makes bare `String`
    // denote that module: both with and without the import it denotes the ambient kernel type.
    // The early return is the discriminator the previous module-only set lacked.
    if !locally_declared && crate::std_types::kernel_type_set().contains_key(spelling) {
        out.insert(KERNEL_DECLARATION_IDENTITY.to_string());
        return out;
    }
    for claim in &record.imports {
        let Some(target) = index_get(index, &claim.target) else {
            continue;
        };
        // An `import m` with no member list exposes the target's whole surface; a member
        // list exposes exactly what it names.
        let claimed = if claim.members.is_empty() {
            import_surface_has(target, spelling)
        } else {
            claim.members.iter().any(|(m, _)| m == spelling)
        };
        if claimed && import_surface_has(target, spelling) {
            out.insert(declarer_of(index, &claim.target, spelling));
        }
    }
    out
}

/// Where a name reached through `module` is actually DECLARED. A re-export names the wrong
/// authority (DESIGN §3 — a fact's home is its declaring module), so the chain is followed to
/// the declarer, bounded by a visited set so a re-export cycle terminates at the last module
/// reached.
fn declarer_of(index: &DeclarationIndex, module: &str, name: &str) -> String {
    let mut current = module.to_string();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return current;
        }
        let Some(record) = index_get(index, &current) else {
            return current;
        };
        if record.declared.contains(name) || record.variants.contains(name) {
            return current;
        }
        let next = record.imports.iter().find_map(|claim| {
            let target = index_get(index, &claim.target)?;
            let claimed = if claim.members.is_empty() {
                import_surface_has(target, name)
            } else {
                claim.members.iter().any(|(m, _)| m == name)
            };
            if claimed {
                Some(claim.target.clone())
            } else {
                None
            }
        });
        match next {
            Some(n) => current = n,
            None => return current,
        }
    }
}

/// One module's binding rows: `(enclosing declaration, LEAF NAME) -> the declaring modules
/// that leaf admits`, unioned over every spelling in that declaration whose last segment is
/// the leaf.
///
/// WHY THE KEY IS THE LEAF AND NOT THE SPELLING — a measured correction. Keyed on the spelling,
/// `widget` and `probe.home.widget` are two rows, so QUALIFYING A REFERENCE reads as one row
/// losing its declaration and an unrelated row appearing — `NewUnresolvedness`, refused.
/// Requalification is the namespace program's core motion (its projection P(B) inserts qualifier
/// segments), so a spelling-keyed wall would refuse the whole program and be weakened — the
/// failure the ruling's `SameDeclarationIdentityRebind` auto-admission exists to prevent. Found by
/// `dropping_an_import_for_a_qualified_spelling_keeps_the_declarer_and_is_admitted`.
///
/// THE LEAF NAMES THE DECLARATION; the segments before it name the ROUTE. Keying on the leaf and
/// valuing on the declaring set is the ruling's rebind (route moved, identity held) vs target
/// change (identity moved), read off the structure rather than asserted.
///
/// AND IT IS AN INVARIANT OF THE OPERATION THE CUT PERFORMS: a requalification wave prepends the
/// declarer's path and leaves the last segment unchanged BY CONSTRUCTION. The reduction is not
/// coined here: `v1.05_emit_rust` `rust_fn_sig_leaf_name_dotted_note` names
/// `qualified_last_segment` as the single authority for an authored spelling's last segment. The
/// converse is the wall working, not to be softened: a cut repointing a reference to a DIFFERENT
/// declaration with a different leaf moves the key and refuses.
///
/// THE UNION IS THE CEILING, STATED WHERE IT IS TAKEN: two references to one leaf inside one
/// declaration share a row, so one requalified and the other not is unobservable here — the
/// module header's ceiling arriving through the key instead of shadowing, with the same next rung.
fn binding_rows(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
) -> BTreeMap<(String, String), BTreeSet<String>> {
    let mut out: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    // Both authored-reference channels, as in `direct_membership` and `membership_bound_through`:
    // a cut repointing a DECLARED type (visible only via the parser's stamped channel) must
    // produce a row on both sides.
    for (in_declaration, spelling) in record
        .referenced
        .iter()
        .chain(record.authored_type_references.iter())
    {
        let leaf = qualified_last_segment(spelling.clone());
        let candidates = declaring_candidates(index, record, spelling);
        out.entry((in_declaration.clone(), leaf))
            .or_default()
            .extend(candidates);
    }
    out
}

/// Transitive closure of direct membership, per module.
fn closure_of(
    membership: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();
    for start in membership.keys() {
        let mut reached: BTreeSet<String> = BTreeSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(start.clone());
        while let Some(node) = queue.pop_front() {
            let Some(edges) = membership.get(&node) else {
                continue;
            };
            for edge in edges {
                if reached.insert(edge.clone()) {
                    queue.push_back(edge.clone());
                }
            }
        }
        reached.remove(start);
        out.insert(start.clone(), reached);
    }
    out
}

fn membership_map(index: &DeclarationIndex) -> BTreeMap<String, BTreeSet<String>> {
    index_records(index)
        .into_iter()
        .map(|r| (r.module_path.clone(), direct_membership(index, r)))
        .collect()
}

// ---------------------------------------------------------------------------
// THE DEPENDENTS DIRECTION — match-bearing consumers of a coproduct whose arm set changed
// ---------------------------------------------------------------------------

/// One coproduct whose arm set differs between the base and head indexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArmSetChange {
    pub module_path: String,
    pub declaration: String,
    pub arms_added: Vec<String>,
    pub arms_removed: Vec<String>,
}

/// How a consumer's match arm was bound to the changed coproduct, carried so the receipt can
/// name the two populations apart: a read whose candidate set names the declaring module, and a
/// bare read whose candidate set is EMPTY at this grain -- the flat last-writer-wins channel the
/// namespace cut is retiring. The second is planned too (it is a consumer in the compiler's
/// eyes, and a missed one is exactly the silent class this selector closes), but it is counted
/// under its own name so the deficit stays visible instead of being absorbed into the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArmConsumerBinding {
    BoundToDeclaringModule,
    BoundThroughFlatBareChannel,
}

/// One module that carries a `match` naming an arm of a changed coproduct, with the declaring
/// module that arm resolved to and the declarations in the consumer that carry the match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArmSetMatchConsumer {
    pub changed_module_path: String,
    pub changed_declaration: String,
    pub consumer_module_path: String,
    pub consumer_rel_path: String,
    pub in_declarations: Vec<String>,
    pub binding: ArmConsumerBinding,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ArmSetConsumerSelection {
    pub changes: Vec<ArmSetChange>,
    pub consumers: Vec<ArmSetMatchConsumer>,
}

/// THE SELECTOR THE REQUIRED FLOOR'S PLANNING ROW CONSUMES, derived from declarations and
/// never from paths or names (DESIGN §3c: a declaration's consumers are a fact the namespace
/// tree carries; the planned set is producer-derived, never a path filter).
///
/// A `match` over a closed coproduct that was exhaustive when it landed goes stale when the
/// coproduct grows an arm in ANOTHER module: the match site has an empty diff, so no
/// diff-keyed selector can see it, and the required floor's prepared subject is the gate
/// closure plus the changed set -- the consumer is never Strict-prepared and `check_match`
/// never runs on it (gunbc#11194, found by a person reading arms). This is the DEPENDENTS
/// direction; `touched_entry_files` seeding is the DEPENDENCY direction, and neither closes
/// the class alone.
///
/// THE RELATION IS THE ONE THE WALL ALREADY USES. A consumer is a module whose `matched_arms`
/// (a pattern head, read at the one site that has no transport entry) names an arm of the
/// changed coproduct and whose `declaring_candidates` for that spelling include the declaring
/// module -- on EITHER side, because a match naming a REMOVED arm has no head-side candidate
/// (the surface no longer exports it) while its base-side one names the declarer exactly. No
/// second consumer relation is minted here; this is `declaring_candidates` asked one more
/// question.
///
/// WHAT IS NOT SELECTED, deliberately: the declaring module itself (its own file is in the
/// diff, so the dependency direction already seeds it); a coproduct that is NEW at head (no
/// consumer can have matched it exhaustively before it existed); and a module whose match
/// names the arm but whose candidate set names a DIFFERENT declarer (a same-spelled arm of an
/// unrelated coproduct -- a consumer of that one, not of this one). A match that names none
/// of the coproduct's arms -- a wildcard, or arms of another type -- is not a consumer, and it
/// is also not stale.
pub(crate) fn arm_set_changed_match_consumers(
    base: &DeclarationIndex,
    head: &DeclarationIndex,
) -> ArmSetConsumerSelection {
    let mut changes: Vec<ArmSetChange> = Vec::new();
    for head_record in index_records(head) {
        let Some(base_record) = index_get(base, &head_record.module_path) else {
            continue;
        };
        for (declaration, head_arms) in &head_record.coproduct_arms {
            let Some(base_arms) = base_record.coproduct_arms.get(declaration) else {
                continue;
            };
            if head_arms == base_arms {
                continue;
            }
            changes.push(ArmSetChange {
                module_path: head_record.module_path.clone(),
                declaration: declaration.clone(),
                arms_added: head_arms.difference(base_arms).cloned().collect(),
                arms_removed: base_arms.difference(head_arms).cloned().collect(),
            });
        }
    }
    let mut consumers: Vec<ArmSetMatchConsumer> = Vec::new();
    for change in &changes {
        let universe: BTreeSet<&String> = {
            let head_arms = &index_get(head, &change.module_path)
                .expect("a change names a head module")
                .coproduct_arms[&change.declaration];
            let base_arms = &index_get(base, &change.module_path)
                .expect("a change names a base module")
                .coproduct_arms[&change.declaration];
            head_arms.iter().chain(base_arms.iter()).collect()
        };
        for consumer in index_records(head) {
            if consumer.module_path == change.module_path {
                continue;
            }
            let mut in_declarations: BTreeSet<String> = BTreeSet::new();
            let mut binding: Option<ArmConsumerBinding> = None;
            for (in_declaration, spelling) in &consumer.matched_arms {
                let leaf = qualified_last_segment(spelling.clone());
                if !universe.contains(&leaf) {
                    continue;
                }
                let mut candidates = declaring_candidates(head, consumer, spelling);
                candidates.extend(declaring_candidates(base, consumer, spelling));
                let bound = if candidates.contains(&change.module_path) {
                    ArmConsumerBinding::BoundToDeclaringModule
                } else if candidates.is_empty() {
                    ArmConsumerBinding::BoundThroughFlatBareChannel
                } else {
                    // Bound to another declarer of a same-spelled arm: not this coproduct's consumer.
                    continue;
                };
                in_declarations.insert(in_declaration.clone());
                // A declarer-bound read wins over a flat one for the module's disposition: the
                // module IS a resolved consumer if any read resolves, and the flat count is for
                // modules that reach the coproduct by no other route.
                binding = Some(match (binding, bound) {
                    (Some(ArmConsumerBinding::BoundToDeclaringModule), _)
                    | (_, ArmConsumerBinding::BoundToDeclaringModule) => {
                        ArmConsumerBinding::BoundToDeclaringModule
                    }
                    _ => ArmConsumerBinding::BoundThroughFlatBareChannel,
                });
            }
            if let Some(binding) = binding {
                consumers.push(ArmSetMatchConsumer {
                    changed_module_path: change.module_path.clone(),
                    changed_declaration: change.declaration.clone(),
                    consumer_module_path: consumer.module_path.clone(),
                    consumer_rel_path: consumer.rel_path.clone(),
                    in_declarations: in_declarations.into_iter().collect(),
                    binding,
                });
            }
        }
    }
    ArmSetConsumerSelection { changes, consumers }
}

// ---------------------------------------------------------------------------
// THE ADJUDICATION
// ---------------------------------------------------------------------------

/// Compare two indexes and adjudicate every delta.
///
/// TAKES TWO INDEXES AND NO GIT. Production reconstructs the base index from the diff; a fixture
/// authors both sides. That boundary makes the RED authorable — DESIGN §4b judges reachability
/// against what a FIXTURE may author, and a wall reachable only through history has no fixture.
pub fn adjudicate(
    base: &DeclarationIndex,
    head: &DeclarationIndex,
    admissions: &[TransitionAdmission],
) -> WaveAdmissionReport {
    let base_membership = membership_map(base);
    let head_membership = membership_map(head);
    let base_closure = closure_of(&base_membership);
    let head_closure = closure_of(&head_membership);

    let mut population = WaveAdmissionPopulation {
        membership_edges_head: head_membership.values().map(|s| s.len()).sum(),
        ..Default::default()
    };
    for module in head_membership.keys() {
        if !base_membership.contains_key(module) {
            population.modules_added += 1;
        }
    }
    for module in base_membership.keys() {
        if !head_membership.contains_key(module) {
            population.modules_removed += 1;
        }
    }

    // CLOSURE MOTION, MEASURED FIRST AND ATTRIBUTED BELOW. A dependency's membership can move a
    // module's subject without its own moving — the blast radius a refusal must name, hence
    // closure over the whole graph rather than per changed file.
    let mut closure_moved_for: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (module, head_reach) in &head_closure {
        let Some(base_reach) = base_closure.get(module) else {
            continue;
        };
        let moved: BTreeSet<String> = head_reach
            .symmetric_difference(base_reach)
            .cloned()
            .collect();
        if !moved.is_empty() {
            population.closure_rows_moved += moved.len();
            closure_moved_for.insert(module.clone(), moved);
        }
    }

    let mut deltas: Vec<NamespaceDelta> = Vec::new();

    for head_record in index_records(head) {
        let module = &head_record.module_path;
        let Some(base_record) = index_get(base, module) else {
            // A module with no base side has no prior meaning to change. Counted above.
            continue;
        };
        population.modules_compared += 1;

        // ── BINDING DELTAS ──
        //
        // ONLY ROWS PRESENT ON BOTH SIDES. A newly authored name has no prior denotation to
        // change; a deleted name has none to protect. The subject is what an EXISTING reference
        // denotes; widening to new authorship would make every PR an adjudication — the tax that
        // gets a wall weakened.
        let base_bindings = binding_rows(base, base_record);
        let head_bindings = binding_rows(head, head_record);
        for (key, head_set) in &head_bindings {
            let Some(base_set) = base_bindings.get(key) else {
                continue;
            };
            population.binding_rows_compared += 1;
            if base_set == head_set {
                continue;
            }
            let disposition = binding_disposition(
                base_set,
                head_set,
                locally_authored_claim_added(head, base_record, head_record, &key.1),
            );
            deltas.push(NamespaceDelta {
                subject: DeltaSubject::Binding {
                    module: module.clone(),
                    in_declaration: key.0.clone(),
                    spelling: key.1.clone(),
                },
                disposition,
                detail: format!(
                    "base {{{}}} -> head {{{}}}",
                    base_set.iter().cloned().collect::<Vec<_>>().join(", "),
                    head_set.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                closure_blast_radius: None,
                admitted_by: None,
            });
        }

        // ── MEMBERSHIP DELTAS ──
        let empty = BTreeSet::new();
        let base_edges = base_membership.get(module).unwrap_or(&empty);
        let head_edges = head_membership.get(module).unwrap_or(&empty);
        for target in head_edges.difference(base_edges) {
            let supported = membership_declared(head, head_record, target);
            deltas.push(NamespaceDelta {
                subject: DeltaSubject::Membership {
                    module: module.clone(),
                    target: target.clone(),
                },
                // An edge whose names all still denote the same is evaluated motion changing no
                // meaning; an edge nothing in the module reaches is unaccounted motion.
                disposition: if supported {
                    NamespaceDeltaDisposition::ExplicitlyEvaluatedZeroDelta
                } else {
                    NamespaceDeltaDisposition::UnexplainedSubjectMotion
                },
                detail: if supported {
                    format!("added; reached by a name this module authors")
                } else {
                    format!("added; NO name in this module resolves into it")
                },
                closure_blast_radius: Some(blast_radius(&closure_moved_for, target)),
                admitted_by: None,
            });
        }
        for target in base_edges.difference(head_edges) {
            let was_used = membership_bound_through(base, base_record, target);
            deltas.push(NamespaceDelta {
                subject: DeltaSubject::Membership {
                    module: module.clone(),
                    target: target.clone(),
                },
                // A removed edge nothing bound through is unused membership going away; one
                // whose names still denote the same declarations is the rebind the ruling
                // auto-admits. Anything else is refused HERE only if the binding channel did not
                // already refuse it, so one motion is never charged twice.
                disposition: if !was_used {
                    NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved
                } else {
                    NamespaceDeltaDisposition::SameDeclarationIdentityRebind
                },
                detail: if !was_used {
                    format!("removed; no name in this module bound through it")
                } else {
                    format!(
                        "removed; every name it supplied still denotes the same declaration \
                         (a binding that did not would be refused on its own row)"
                    )
                },
                closure_blast_radius: Some(blast_radius(&closure_moved_for, target)),
                admitted_by: None,
            });
        }
    }

    deltas.sort_by(|a, b| {
        a.subject
            .cmp(&b.subject)
            .then(a.disposition.cmp(&b.disposition))
    });

    // ── ADMISSIONS ──
    let mut used: BTreeSet<usize> = BTreeSet::new();
    let mut invalid_admissions = BTreeMap::new();
    for delta in deltas.iter_mut() {
        for (i, admission) in admissions.iter().enumerate() {
            if admission_subject_matches(&admission.subject, &delta.subject)
                && admission.disposition == delta.disposition
            {
                if matches!(admission.subject, AdmissionSubject::Binding { .. }) {
                    if let Err(mismatch) = admission_satisfied_at(admission, head, &head_membership)
                    {
                        invalid_admissions.insert(i, format!("head {mismatch}"));
                        continue;
                    }
                }
                delta.admitted_by = Some(admission.label.clone());
                used.insert(i);
                break;
            }
        }
    }
    // ── THE UNUSED-ROW SPLIT ──
    //
    // Consumed-by-merge and author-error were one refusal for eight roster generations, and the
    // conflation billed the cleanup to bystanders: a row is REQUIRED at instant N (its own PR's
    // CI) and poisonous at instant N+1 (everyone else's), because after the squash-merge base and
    // head both carry the relocation and the row can never match a delta again. Eight dissolution
    // PRs (#9797 the seventh, #9820 the eighth) each spent hours of unrelated-lane red as the
    // roster's garbage collector — externalized degradation (DESIGN §5).
    //
    // The two states are decidable apart, so this is a wall, not a ratchet: a consumed row's
    // admitted relocation ALREADY HOLDS AT THE BASE (`admission_satisfied_at`, a positive
    // check against the base index), while an author-error row is provable against neither side.
    // Only the proven arm is typed ConsumedByMerge; everything else unused remains an
    // UnmatchedAdmission. The consumed arm does not widen: it is unreachable by fallthrough.
    // Unmatched rows still refuse every PR; consumed rows come due on landing or roster edits.
    //
    // RETIRED BY: admissions bound to the delta content they admit, adjudicated per run and never
    // resident on main — the capability that makes a stale-able row unwritable. Until that
    // carrier exists, consumed rows persist as typed receipts and their deletion is enforced on
    // the roster file's own next touch.
    let used_without_follow_up = used
        .iter()
        .map(|&i| &admissions[i])
        .filter(|a| a.deletion_follow_up == DeletionFollowUp::NotAuthored)
        .map(|a| {
            format!(
                "{} ({} {}) is used by this candidate and will be consumed when it lands, but \
                 its owner authored no deletion follow-up (follow-up PR absent)",
                a.label,
                disposition_label(a.disposition),
                admission_subject_render(&a.subject)
            )
        })
        .collect::<Vec<_>>();
    let mut stale_admissions = Vec::new();
    let mut consumed_admissions = Vec::new();
    let mut consumed_without_follow_up = Vec::new();
    let mut owned_consumed_receipts = Vec::new();
    for (i, a) in admissions.iter().enumerate() {
        if used.contains(&i) {
            continue;
        }
        let satisfaction = match invalid_admissions.get(&i) {
            Some(mismatch) => Err(mismatch.clone()),
            None => admission_satisfied_at(a, base, &base_membership)
                .map_err(|mismatch| format!("base {mismatch}")),
        };
        match satisfaction {
            Ok(()) => {
                let rendered = format!(
                    "{} ({} {}) already satisfied at the base — consumed by its own merge; \
                     deletion is owed on landing or the roster's next touch; owner gunbc#{}; {}",
                    a.label,
                    disposition_label(a.disposition),
                    admission_subject_render(&a.subject),
                    a.owner_pull_request,
                    match a.deletion_follow_up {
                        DeletionFollowUp::NotAuthored =>
                            "no deletion follow-up authored".to_string(),
                        DeletionFollowUp::PullRequest(n) => format!("deletion follow-up gunbc#{n}"),
                    }
                );
                match a.deletion_follow_up {
                    DeletionFollowUp::NotAuthored => {
                        consumed_without_follow_up.push(rendered.clone())
                    }
                    DeletionFollowUp::PullRequest(n) => {
                        owned_consumed_receipts.push(ConsumedRowReceipt {
                            label: a.label.to_string(),
                            owner_pull_request: a.owner_pull_request,
                            deletion_follow_up_pull_request: n,
                        })
                    }
                }
                consumed_admissions.push(rendered);
            }
            Err(mismatch) => stale_admissions.push(format!(
                "{} ({} {}) has no valid admission in this run: {mismatch}",
                a.label,
                disposition_label(a.disposition),
                admission_subject_render(&a.subject)
            )),
        }
    }

    WaveAdmissionReport {
        population,
        deltas,
        stale_admissions,
        consumed_admissions,
        used_without_follow_up,
        consumed_without_follow_up,
        owned_consumed_receipts,
    }
}

/// Prove the admitted result at one index, or name its expected and observed candidates.
/// The head check prevents authoring an unreachable lifecycle; the base check derives
/// consumption. Membership keeps its existing presence proof; binding requires set equality.
fn admission_satisfied_at(
    a: &TransitionAdmission,
    index: &DeclarationIndex,
    membership: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), String> {
    match &a.subject {
        AdmissionSubject::Membership { module, target } => {
            if membership
                .get(module)
                .is_some_and(|members| members.contains(target))
            {
                Ok(())
            } else {
                Err(format!(
                    "expected membership {module} -> {target}, found {:?}",
                    membership.get(module)
                ))
            }
        }
        AdmissionSubject::Binding {
            module,
            in_declaration,
            spelling,
            expected_candidates,
        } => {
            let expected: BTreeSet<String> = expected_candidates.iter().cloned().collect();
            let Some(record) = index_get(index, module) else {
                return Err(format!(
                    "expected candidates {expected:?}, found no module {module}"
                ));
            };
            let rows = binding_rows(index, record);
            match rows.get(&(in_declaration.clone(), spelling.clone())) {
                Some(found) if *found == expected => Ok(()),
                Some(found) => Err(format!(
                    "expected candidates {expected:?}, found candidates {found:?}"
                )),
                None => Err(format!(
                    "expected candidates {expected:?}, found no binding row"
                )),
            }
        }
    }
}

/// Which disposition a changed candidate SET carries.
///
/// EVERY ARM IS OVER SETS, NOT WINNERS. `1 -> 0` stopped denoting anything; `1 -> 2` now admits
/// two declarations, which the namespace authority refuses at the reference rather than
/// resolving by nearness. Any other non-empty pair is a changed target.
///
/// `0 -> 1` IS TWO STATES, NOT ONE, AND THE SET PAIR CANNOT TELL THEM APART. An earlier revision
/// read every `0 -> 1` as resolution from a pool — a name denoting something WITHOUT ANYONE
/// AUTHORING A REFERENCE, the coincidence the containment rule removes, caused in ANOTHER module.
/// The other is this module's author writing the import that resolves a name already spelled —
/// the repair the wall wants, and the state it refused. Opposite owners, opposite repairs, one
/// symbol: DESIGN's state-space conflation.
///
/// THE DISCRIMINATOR IS THE MODULE'S OWN SOURCE, available for free — the membership arm already
/// consults authorship (`membership_declared`, over `membership_bound_through`), which admitted
/// the membership edge of the very change this arm refused. So `authored_here` is passed in, not
/// re-derived: see `locally_authored_claim_added`.
fn binding_disposition(
    base: &BTreeSet<String>,
    head: &BTreeSet<String>,
    authored_here: bool,
) -> NamespaceDeltaDisposition {
    if head.is_empty() {
        return NamespaceDeltaDisposition::NewUnresolvedness;
    }
    if base.is_empty() {
        return if authored_here {
            NamespaceDeltaDisposition::AuthoredReferenceResolution
        } else {
            NamespaceDeltaDisposition::NewPoolCoincidenceResolution
        };
    }
    if head.len() > base.len() && head.len() > 1 {
        return NamespaceDeltaDisposition::NewAmbiguity;
    }
    NamespaceDeltaDisposition::TargetChanged
}

/// Whether `module` DECLARES a dependency on `target` -- the ADD direction's question.
///
/// AN AUTHORED IMPORT CLAIM IS THE ANSWER, NOT EVIDENCE TOWARD IT. On the add side
/// `DeltaSubject::Membership` asks whether `module` gained `target` as a DIRECT DEPENDENCY, and
/// `import <target> { .. }` is that dependency in authored syntax. The reference set is a lossy
/// DOWNSTREAM PROXY: a name used ONLY as a match-arm pattern head is unreachable from
/// `for_each_node` IN PRINCIPLE, because `MatchPattern::VariantPattern.name` is a `String`, never
/// a `Node`. So a module importing a coproduct purely for pattern variants had its declared
/// dependency refused as `UnexplainedSubjectMotion`. Reading the weaker of two representations of
/// one fact is DESIGN 3, not leniency.
///
/// THE SPLIT FROM `membership_bound_through` IS THE FINDING, NOT A TIDY-UP. One predicate served
/// both directions until an executed RED showed they ask OPPOSITE questions. Add asks *does this
/// module depend on target* — an import claim answers outright. Removal asks *was anything bound
/// through it* — an import claim CANNOT answer, since an unused import is declared and bound
/// through by nothing. Widening the SHARED predicate made every unused-import removal report
/// `SameDeclarationIdentityRebind` instead of `UnusedSubjectMembershipRemoved`: state-space
/// conflation, one symbol answering two questions with opposite owners and repairs. Caught by
/// the sibling test going red, not review, which is why the fixture below is enrolled.
fn membership_declared(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
    target: &str,
) -> bool {
    record.imports.iter().any(|claim| claim.target == target)
        || membership_bound_through(index, record, target)
}

/// Did THIS module's own source gain a claim on `leaf` between the two sides?
///
/// EVERY ARM IS SCOPED TO `leaf`, AND THE BLANKET ARM IS WHERE THAT IS EASY TO GET WRONG. A
/// member-list import and a self-declaration name the leaf, so they answer from the two
/// `ModuleDeclarationRecord`s alone. A blanket `import m` names no leaf, and a first revision
/// admitted authorship whenever ANY blanket target was new — so an unrelated new blanket import
/// auto-admitted a genuine pool coincidence, a fail-open widening in the one direction this
/// function must never move (review 56882, on gunbc#9495, before it merged).
///
/// SO THE BLANKET ARM IS A CONJUNCTION, AND THE ORDER OF ITS TWO HALVES IS THE WHOLE POINT. The
/// claim must be NEW IN THIS MODULE'S SOURCE **and** the head target must supply the leaf. The
/// second alone is the conflation: an unchanged blanket import whose target grew `leaf` would
/// read as authorship. Gating on the first makes the index consultation safe: a claim the author
/// did not write never reaches the surface check.
///
/// FALSE IS THE FAIL-CLOSED ANSWER: the delta stays on the refusing arm for a human. A target
/// absent from the index answers false for the same reason.
fn locally_authored_claim_added(
    head_index: &DeclarationIndex,
    base_record: &ModuleDeclarationRecord,
    head_record: &ModuleDeclarationRecord,
    leaf: &str,
) -> bool {
    let declares =
        |r: &ModuleDeclarationRecord| r.declared.contains(leaf) || r.variants.contains(leaf);
    if declares(head_record) && !declares(base_record) {
        return true;
    }
    let names_leaf = |r: &ModuleDeclarationRecord| {
        r.imports
            .iter()
            .any(|c| c.members.iter().any(|(m, _)| m == leaf))
    };
    if names_leaf(head_record) && !names_leaf(base_record) {
        return true;
    }
    let blanket_targets = |r: &ModuleDeclarationRecord| {
        r.imports
            .iter()
            .filter(|c| c.members.is_empty())
            .map(|c| c.target.clone())
            .collect::<BTreeSet<String>>()
    };
    let base_blanket = blanket_targets(base_record);
    blanket_targets(head_record)
        .difference(&base_blanket)
        .any(|target| {
            index_get(head_index, target)
                .map(|t| import_surface_has(t, leaf))
                .unwrap_or(false)
        })
}

/// Whether any name this module authors reaches into `target`'s surface.
fn membership_bound_through(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
    target: &str,
) -> bool {
    // THE UNION OF BOTH AUTHORED-REFERENCE CHANNELS, peers with different authorities (see
    // `authored_type_references`): `referenced` is the index's walk over the final tree; the
    // other is the parser's stamped answer, reaching a declared type parked in `inferred` that no
    // tree walk sees. "Is anything bound through that import" wants both -- asking only the walk
    // let this predicate report a live import as unused.
    record
        .referenced
        .iter()
        .chain(record.authored_type_references.iter())
        .any(|(_, spelling)| {
            declaring_candidates(index, record, spelling).contains(target)
                || module_prefix_of(index, spelling)
                    .map(|(m, _)| m == target)
                    .unwrap_or(false)
        })
}

/// How many modules' closures moved in a way this target participates in.
fn blast_radius(moved: &BTreeMap<String, BTreeSet<String>>, target: &str) -> usize {
    moved.values().filter(|s| s.contains(target)).count()
}

pub fn render_delta(delta: &NamespaceDelta) -> String {
    let admitted = match &delta.admitted_by {
        Some(label) => format!(" ADMITTED-BY {label}"),
        None => String::new(),
    };
    // The clause is printed ONLY where the question was asked. A row that did not ask it says
    // nothing, rather than saying zero: a measurement-shaped output on a row that measured
    // nothing is fabricated plausible output, and it was read as evidence of containment.
    let radius = match delta.closure_blast_radius {
        Some(n) => format!(" [closure blast radius: {n} module(s)]"),
        None => String::new(),
    };
    format!(
        "{} {} — {}{}{}",
        disposition_label(delta.disposition),
        delta_subject_render(&delta.subject),
        delta.detail,
        admitted,
        radius
    )
}

// ---------------------------------------------------------------------------
// THE PRODUCTION RUN — the base index reconstructed from the diff
// ---------------------------------------------------------------------------

use std::path::Path;
use std::process::Command;

use crate::cli_run::declaration_index::record_from_module;
use crate::cli_run::{workspace_root, DAG_PARSE_SWEEP_ROOTS};

/// What one required run of the wall answers.
///
/// `NotEvaluated` IS A REFUSAL AND NOT A SKIP, which is why it is a variant rather than an `Err`
/// folded in with a spawn failure: "could not see what changed" and "nothing changed" have
/// different remedies (DESIGN §5, the empty-observation narrow), and the ruling puts
/// `NotEvaluated` on the refusing side explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaveAdmissionOutcome {
    /// The baseline resolves to the head, so there is no diff. A push to `main` after a squash
    /// merge is the whole population. NOT an admission: nothing was compared, and the phase
    /// reports it under its own name. Only an empty roster may take this arm: landing still
    /// adjudicates and refuses stale or consumed rows when there is no diff.
    NoSubject { head: String },
    /// The baseline could not be observed. Refuses.
    NotEvaluated { reason: String },
    Adjudicated {
        base: String,
        head: String,
        /// Boxed because the report dwarfs the other arms (clippy `large_enum_variant`).
        report: Box<WaveAdmissionReport>,
        /// Whether this diff touches the roster source. Consumed rows come due here
        /// and on main (base == head); stale rows refuse regardless of this flag.
        roster_touched: bool,
        /// The CI event whose subject this run adjudicated.
        event: AdjudicationEvent,
    },
}

/// Directory whose membership IS the transition-admission roster (trailing slash so the type
/// module `dag/gunbc/namespace/transition_admission.dag` is not a roster touch).
pub const ADMISSION_ROSTER_REL_PATH: &str = "dag/gunbc/namespace/transition_admission/";

/// True when a diff path is a row file (or the directory itself) under the roster prefix.
pub fn admission_roster_path_touched(rel: &str) -> bool {
    rel == ADMISSION_ROSTER_REL_PATH.trim_end_matches('/')
        || rel.starts_with(ADMISSION_ROSTER_REL_PATH)
}

const ROW_MODULE_PREFIX: &str = "gunbc.namespace.transition_admission.";

/// Load production admissions from a workspace: the directory fold.
///
/// AUTHORING IS SAFETY, NOT CONST-NESS. The permission set must be authored and
/// reviewable, never derived from the delta it admits. `const` used to make that
/// mechanically true; each permission is now an authored `.dag` row in the PR
/// diff. `read_dir` enumerates files git already carries — it does not mint
/// permission. A computed predicate over observed deltas still has no constructor.
///
/// Missing directory (`ErrorKind::NotFound`) is the empty roster: git cannot carry an empty
/// directory, so absence IS the resting empty-const state. That arm yields fewer admissions,
/// never more — fail-closed on the admission axis. Every other `read_dir`, dirent, stem, parse,
/// or type error refuses, located, and is never skipped. Standing census of whether the
/// directory is empty lives on `gunbc.namespace.transition_admission`, not here.
pub fn load_transition_admissions(workspace: &Path) -> Result<Vec<TransitionAdmission>, String> {
    load_transition_admissions_from_dir(&workspace.join(ADMISSION_ROSTER_REL_PATH))
}

pub fn load_transition_admissions_from_dir(dir: &Path) -> Result<Vec<TransitionAdmission>, String> {
    match std::fs::read_dir(dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!(
            "transition-admission roster directory {} is unreadable: {e}",
            dir.display()
        )),
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                let entry = entry.map_err(|e| {
                    format!(
                        "transition-admission roster directory {} dirent failed: {e}",
                        dir.display()
                    )
                })?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("dag") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                    format!(
                        "transition-admission row {} has a non-utf8 stem",
                        path.display()
                    )
                })?;
                if stem == "roster" {
                    return Err(format!(
                        "transition-admission row {} is named roster.dag; this roster has no \
                         committed list (directory membership is the list)",
                        path.display()
                    ));
                }
                files.push((stem.to_string(), path));
            }
            files.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = Vec::with_capacity(files.len());
            for (stem, path) in files {
                let rel = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("row.dag");
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    format!(
                        "transition-admission row {} is unreadable: {e}",
                        path.display()
                    )
                })?;
                out.push(parse_transition_admission_row(&path, rel, &stem, &content)?);
            }
            Ok(out)
        }
    }
}

fn parse_transition_admission_row(
    path: &Path,
    rel: &str,
    stem: &str,
    content: &str,
) -> Result<TransitionAdmission, String> {
    let located = |msg: String| format!("{}: {msg}", path.display());
    let fill = crate::v1_compiler_compile::parse_census_fill_sources_with_environment(
        Rc::new(
            vec![Rc::new(crate::v1_compiler_compile::SourceFile {
                path: rel.to_string(),
                content: content.to_string(),
            })]
            .into(),
        ),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    if !fill.diagnostics.is_empty() {
        return Err(located(format!(
            "does not parse ({} diagnostic(s))",
            fill.diagnostics.len()
        )));
    }
    let module = fill
        .modules
        .iter()
        .next()
        .ok_or_else(|| located("parsed to no module".to_string()))?;
    let source_indices: Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>> = Rc::new(
        fill.newline_indices
            .iter()
            .fold(im::HashMap::new(), |acc, i| {
                acc.update(i.file.clone(), i.clone())
            }),
    );
    let module_path = crate::v1_std_core::authored_name_at(source_indices.clone(), module.clone());
    let expected_module = format!("{ROW_MODULE_PREFIX}{stem}");
    if module_path != expected_module {
        return Err(located(format!(
            "module `{module_path}` must be `{expected_module}` (stem is the relocation identity)"
        )));
    }
    let mut data_items = Vec::new();
    for item in crate::v1_std_core::module_items(module.clone()).iter() {
        if crate::v1_compiler_infer_items::item_kind(item.clone())
            == crate::v1_compiler_infer_items::ItemKind::DataItem
        {
            data_items.push(item.clone());
        }
    }
    if data_items.len() != 1 {
        return Err(located(format!(
            "must declare exactly one data row, found {}",
            data_items.len()
        )));
    }
    let item = &data_items[0];
    let decl_name = crate::v1_std_core::authored_name_at(source_indices.clone(), item.clone());
    if decl_name != stem {
        return Err(located(format!(
            "data declaration `{decl_name}` must equal file stem `{stem}`"
        )));
    }
    let Some(body) = item.body.clone() else {
        return Err(located("data row has no initializer".to_string()));
    };
    parse_transition_admission_expr(body, &source_indices).map_err(located)
}

fn peel_expr(expr: Rc<crate::v1_std_core::Node>) -> Rc<crate::v1_std_core::Node> {
    match (*expr.expr_data).clone() {
        crate::v1_std_core::ExprData::ExprCast => expr
            .children
            .iter()
            .next()
            .cloned()
            .map(peel_expr)
            .unwrap_or(expr),
        _ => expr,
    }
}

fn expr_string(expr: Rc<crate::v1_std_core::Node>) -> Result<String, String> {
    let expr = peel_expr(expr);
    crate::v1_std_core::expr_literal_string_optional(expr)
        .ok_or_else(|| "expected a string literal".to_string())
}

fn expr_leaf_name(
    expr: Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    let raw = crate::v1_std_core::authored_name_at(source_indices.clone(), peel_expr(expr));
    qualified_last_segment(raw)
}

fn parse_transition_admission_expr(
    expr: Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<TransitionAdmission, String> {
    let expr = peel_expr(expr);
    if expr_leaf_name(expr.clone(), source_indices) != "TransitionAdmission" {
        return Err(format!(
            "initializer must construct TransitionAdmission, found `{}`",
            expr_leaf_name(expr, source_indices)
        ));
    }
    let Some(label_e) = crate::v1_std_core::record_lit_named_field_value_optional(
        expr.clone(),
        "label".to_string(),
        source_indices.clone(),
    ) else {
        return Err("TransitionAdmission is missing field `label`".to_string());
    };
    let Some(subject_e) = crate::v1_std_core::record_lit_named_field_value_optional(
        expr.clone(),
        "subject".to_string(),
        source_indices.clone(),
    ) else {
        return Err("TransitionAdmission is missing field `subject`".to_string());
    };
    let Some(disposition_e) = crate::v1_std_core::record_lit_named_field_value_optional(
        expr.clone(),
        "disposition".to_string(),
        source_indices.clone(),
    ) else {
        return Err("TransitionAdmission is missing field `disposition`".to_string());
    };
    let Some(follow_up_e) = crate::v1_std_core::record_lit_named_field_value_optional(
        expr.clone(),
        "deletion_follow_up".to_string(),
        source_indices.clone(),
    ) else {
        return Err("TransitionAdmission is missing field `deletion_follow_up`".to_string());
    };
    let Some(owner_e) = crate::v1_std_core::record_lit_named_field_value_optional(
        expr.clone(),
        "owner_pull_request".to_string(),
        source_indices.clone(),
    ) else {
        return Err("TransitionAdmission is missing field `owner_pull_request`".to_string());
    };
    Ok(TransitionAdmission {
        label: expr_string(label_e)?,
        subject: parse_admission_subject(subject_e, source_indices)?,
        disposition: parse_disposition(disposition_e, source_indices)?,
        deletion_follow_up: parse_deletion_follow_up(follow_up_e, source_indices)?,
        owner_pull_request: expr_u32(owner_e)?,
    })
}

fn expr_u32(expr: Rc<crate::v1_std_core::Node>) -> Result<u32, String> {
    let expr = peel_expr(expr);
    crate::v1_std_core::expr_literal_int_optional(expr)
        .ok_or_else(|| "expected an integer literal".to_string())
        .and_then(|n| u32::try_from(n).map_err(|_| format!("integer {n} is not a u32")))
}

fn parse_deletion_follow_up(
    expr: Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<DeletionFollowUp, String> {
    let expr = peel_expr(expr);
    match expr_leaf_name(expr.clone(), source_indices).as_str() {
        "NotAuthored" => Ok(DeletionFollowUp::NotAuthored),
        "PullRequest" => {
            let number = crate::v1_std_core::record_lit_named_field_value_optional(
                expr.clone(),
                "number".to_string(),
                source_indices.clone(),
            )
            .or_else(|| expr.children.iter().next().cloned())
            .ok_or_else(|| "PullRequest is missing `number`".to_string())?;
            Ok(DeletionFollowUp::PullRequest(expr_u32(number)?))
        }
        other => Err(format!("unknown DeletionFollowUp `{other}`")),
    }
}

fn parse_disposition(
    expr: Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<NamespaceDeltaDisposition, String> {
    match expr_leaf_name(expr, source_indices).as_str() {
        "SameDeclarationIdentityRebind" => {
            Ok(NamespaceDeltaDisposition::SameDeclarationIdentityRebind)
        }
        "UnusedSubjectMembershipRemoved" => {
            Ok(NamespaceDeltaDisposition::UnusedSubjectMembershipRemoved)
        }
        "ExplicitlyEvaluatedZeroDelta" => {
            Ok(NamespaceDeltaDisposition::ExplicitlyEvaluatedZeroDelta)
        }
        "TargetChanged" => Ok(NamespaceDeltaDisposition::TargetChanged),
        "NewAmbiguity" => Ok(NamespaceDeltaDisposition::NewAmbiguity),
        "NewUnresolvedness" => Ok(NamespaceDeltaDisposition::NewUnresolvedness),
        "NewPoolCoincidenceResolution" => {
            Ok(NamespaceDeltaDisposition::NewPoolCoincidenceResolution)
        }
        "AuthoredReferenceResolution" => Ok(NamespaceDeltaDisposition::AuthoredReferenceResolution),
        "UnexplainedSubjectMotion" => Ok(NamespaceDeltaDisposition::UnexplainedSubjectMotion),
        "NotEvaluated" => Ok(NamespaceDeltaDisposition::NotEvaluated),
        other => Err(format!("unknown NamespaceDeltaDisposition `{other}`")),
    }
}

fn parse_admission_subject(
    expr: Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<AdmissionSubject, String> {
    let expr = peel_expr(expr);
    match expr_leaf_name(expr.clone(), source_indices).as_str() {
        "Binding" => {
            let Some(enclosing) = crate::v1_std_core::record_lit_named_field_value_optional(
                expr.clone(),
                "enclosing".to_string(),
                source_indices.clone(),
            ) else {
                return Err("Binding is missing field `enclosing`".to_string());
            };
            let Some(spelling) = crate::v1_std_core::record_lit_named_field_value_optional(
                expr.clone(),
                "spelling".to_string(),
                source_indices.clone(),
            ) else {
                return Err("Binding is missing field `spelling`".to_string());
            };
            let Some(candidates) = crate::v1_std_core::record_lit_named_field_value_optional(
                expr,
                "expected_candidates".to_string(),
                source_indices.clone(),
            ) else {
                return Err("Binding is missing field `expected_candidates`".to_string());
            };
            let (module, in_declaration) = parse_decl_ref(enclosing, source_indices)?;
            let spelling = expr_string(spelling)?;
            let mut expected_candidates = Vec::new();
            for (candidate_module, candidate_decl) in
                parse_decl_ref_list(candidates, source_indices)?
            {
                if candidate_decl != spelling {
                    return Err(format!(
                        "expected_candidates DeclarationRef.decl_name `{candidate_decl}` must equal Binding.spelling `{spelling}`"
                    ));
                }
                expected_candidates.push(candidate_module);
            }
            Ok(AdmissionSubject::Binding {
                module,
                in_declaration,
                spelling,
                expected_candidates,
            })
        }
        "Membership" => {
            let Some(from_module) = crate::v1_std_core::record_lit_named_field_value_optional(
                expr.clone(),
                "from_module".to_string(),
                source_indices.clone(),
            ) else {
                return Err("Membership is missing field `from_module`".to_string());
            };
            let Some(target_module) = crate::v1_std_core::record_lit_named_field_value_optional(
                expr,
                "target_module".to_string(),
                source_indices.clone(),
            ) else {
                return Err("Membership is missing field `target_module`".to_string());
            };
            Ok(AdmissionSubject::Membership {
                module: expr_string(from_module)?,
                target: expr_string(target_module)?,
            })
        }
        other => Err(format!("unknown AdmissionSubject `{other}`")),
    }
}

fn parse_decl_ref(
    expr: Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<(String, String), String> {
    let expr = peel_expr(expr);
    if expr_leaf_name(expr.clone(), source_indices) == "DeclarationRef" {
        let module = crate::v1_std_core::record_lit_named_field_value_optional(
            expr.clone(),
            "module_path".to_string(),
            source_indices.clone(),
        )
        .ok_or_else(|| "DeclarationRef is missing `module_path`".to_string())?;
        let decl = crate::v1_std_core::record_lit_named_field_value_optional(
            expr,
            "decl_name".to_string(),
            source_indices.clone(),
        )
        .ok_or_else(|| "DeclarationRef is missing `decl_name`".to_string())?;
        return Ok((expr_string(module)?, expr_string(decl)?));
    }
    match (*expr.expr_data).clone() {
        crate::v1_std_core::ExprData::ExprCall { .. } => {
            let mut module = None;
            let mut decl = None;
            let mut positional = Vec::new();
            for child in expr.children.iter() {
                let value = crate::v1_std_core::arg_value(child.clone());
                match crate::v1_std_core::arg_name_at(child.clone(), source_indices.clone()) {
                    Some(name) if name == "module_path" => module = Some(expr_string(value)?),
                    Some(name) if name == "decl_name" => decl = Some(expr_string(value)?),
                    Some(_) => {}
                    None => {
                        if let Ok(s) = expr_string(value) {
                            positional.push(s);
                        }
                    }
                }
            }
            if let (Some(m), Some(d)) = (module, decl) {
                return Ok((m, d));
            }
            if positional.len() >= 2 {
                return Ok((positional[0].clone(), positional[1].clone()));
            }
            Err("decl_ref / DeclarationRef did not yield module_path and decl_name".to_string())
        }
        _ => Err("enclosing must be a DeclarationRef or decl_ref(...)".to_string()),
    }
}

fn parse_decl_ref_list(
    expr: Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<Vec<(String, String)>, String> {
    let expr = peel_expr(expr);
    match (*expr.expr_data).clone() {
        crate::v1_std_core::ExprData::ExprListLit => expr
            .children
            .iter()
            .map(|c| parse_decl_ref(c.clone(), source_indices))
            .collect(),
        _ => Err("expected_candidates must be a list of DeclarationRef".to_string()),
    }
}

/// Whether one adjudicated run REFUSES, and the sentence naming why — the wall's verdict, held by
/// the wall.
///
/// IT LIVES HERE SO ITS RED IS AUTHORABLE WHERE THE VERDICT IS ACTUALLY REACHED. The decision was
/// interleaved with the executor's printing, in a private function of a binary, so nothing outside
/// that binary could construct the refusal and no test could discriminate the roster-touched arm
/// on the path CI runs. DESIGN §4b puts that squarely: a wall whose RED cannot be authored on the
/// acceptance path is a decoration, and the missing harness is the trigger rather than a ceiling.
/// The executor keeps the receipts — it is the thing with a stderr — and asks this for the verdict,
/// so "does this run refuse" has one authority instead of one authority and one printer.
///
/// Stale rows and unadjudicated deltas always refuse. Consumed rows refuse at landing
/// (base == head) or on a roster-source edit. THAT IS THE WHOLE REFUSAL SET.
///
/// Two `merge_group` arms were removed on 2026-09-16 (operator ruling): `OwnerFollowUpAbsent`,
/// which refused a used row whose owner had authored no `deletion_follow_up`, and
/// `ConsumedRowOwnerChargeBypassed`, which refused a bystander composition for a prior owner's
/// unauthored follow-up. Neither took a guarantee with it. The first established only that a
/// number had been typed -- its own message conceded it "checks that a number is authored, never
/// that it names an open or deleting pull request" -- so its admitting side was free and a
/// fabricated number passed. The second billed a change for a debt its own comment said was not
/// its own.
///
/// WHAT THIS COSTS, STATED PLAINLY AND DECLARED AS A DROP. An earlier revision of this note claimed
/// the landing arm still compels a consumed row's deletion. On the REQUIRED path it does not.
/// Lane ruling (fierce-lark-661, 2026-09-13): the merge queue moved the required verdict off the
/// push to the default branch, and with it the only run where base == head -- so `roster_due`
/// reduces to `roster_touched` alone there. With `ConsumedRowOwnerChargeBypassed` gone, a
/// base-consumed row whose owner authored no follow-up refuses on NO required run until somebody
/// happens to edit the roster directory. That arm's RED discriminated on the ABSENCE of any number,
/// so unlike `OwnerFollowUpAbsent` -- whose admitting side was free and which was a decoration --
/// removing it is a COVERAGE LOSS. It is declared as a 4b(3) rung drop,
/// `gunbc.rung_drop.consumed_row_owner_charge_unenforced`, not passed off as a no-op.
///
/// `used_without_follow_up` is still COUNTED in the message, so the debt stays visible as a
/// receipt; it just no longer refuses. Lifecycle is derived from the candidate-set
/// proof, never predicted by an authored row. Policy authority:
/// `gunbc.namespace_wave_admission` `namespace_wave_admission_note`.
pub fn wave_admission_refusal(outcome: &WaveAdmissionOutcome) -> Option<String> {
    match outcome {
        WaveAdmissionOutcome::NoSubject { head: _ } => None,
        WaveAdmissionOutcome::NotEvaluated { reason: _ } => {
            Some("namespace-wave-admission (NotEvaluated)".to_string())
        }
        WaveAdmissionOutcome::Adjudicated {
            base,
            head,
            report,
            roster_touched,
            // NO READER LEFT, AND SAID RATHER THAN HIDDEN. `event` existed here to compute
            // `composition` for the two removed merge_group arms; nothing in this function reads it
            // now, and `claim_executor`'s destructuring already ignored it. So the variant's
            // `event` field is currently constructed everywhere and read nowhere -- a DESIGN 3c
            // dangling field that this removal created. It is left in place rather than pulled out
            // of ~15 construction sites in the same change; the honest disposition is a follow-up
            // that either finds it a consumer or deletes it.
            event: _,
        } => {
            let unadjudicated = report_unadjudicated(report);
            let roster_due = base == head || *roster_touched;
            // THE FOLLOW-UP ARMS ARE GONE, AND THE DEBT IS STILL ENFORCED. Two merge_group arms
            // used to refuse here: OwnerFollowUpAbsent (a used row whose owner authored no
            // deletion_follow_up) and ConsumedRowOwnerChargeBypassed (a bystander composition
            // charged for someone else's unauthored follow-up). Both are removed.
            //
            // NO GUARANTEE FALLS WITH THEM. `deletion_follow_up` remains on the row and is still
            // read, so the debt is still RECORDED; and `consumed_due` below still refuses a
            // consumed row at landing or on a roster-source edit, so the deletion is still
            // ENFORCED at the point it becomes real. What the arms added was an ADVANCE
            // commitment whose admitting side was free -- the wall's own words were that it
            // "checks that a number is authored, never that it names an open or deleting pull
            // request" -- so any digits satisfied it. A check whose RED is authorable but whose
            // GREEN is unverified buys the appearance of a wall (DESIGN 4b), and the second arm
            // billed a BYSTANDER for an owner's debt, which its own comment flagged as the thing
            // to avoid.
            let consumed_due = roster_due && !report.consumed_admissions.is_empty();
            let stale_due = !report.stale_admissions.is_empty();
            if unadjudicated.is_empty() && !stale_due && !consumed_due {
                return None;
            }
            let remedy = if stale_due || consumed_due {
                let rows = report
                    .stale_admissions
                    .iter()
                    .chain(&report.consumed_admissions)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ");
                format!(
                    "; delete these rows from {ADMISSION_ROSTER_REL_PATH}; declared transition \
                     labels (including their trigger PR where authored): {rows}"
                )
            } else {
                String::new()
            };
            Some(format!(
                "namespace-wave-admission ({} unadjudicated delta(s), {} stale admission(s), {} \
                 consumed admission(s){}, {} used row(s) without a deletion follow-up){remedy}",
                unadjudicated.len(),
                report.stale_admissions.len(),
                report.consumed_admissions.len(),
                if consumed_due || stale_due {
                    " due for correction or deletion"
                } else {
                    ""
                },
                report.used_without_follow_up.len(),
            ))
        }
    }
}

/// Run git in the workspace and return stdout with TRAILING whitespace removed, or a refusal
/// naming the command.
///
/// `trim_end`, not `trim`, and the asymmetry is load-bearing: one caller reads FILE CONTENT at a
/// ref (`git show <ref>:<path>`), and a `.dag` module with an indented first line would lose that
/// indentation -- a different file from the committed one, compared against an intact head read.
/// Every other caller (`rev-parse`, `merge-base`, `diff --name-only`, `status --porcelain`) has no
/// leading whitespace or re-trims. `status --porcelain` lines BEGIN with the two-column XY code,
/// so a leading `trim` would corrupt a status read.
///
/// THE ONE COPY. `claim_executor` carried a byte-identical private copy and this module's first
/// draft a second — the §3 fork this wall refuses elsewhere. `pub` here because the wall is a
/// library fold and the bin one of its callers; the bin's copy is deleted and its five call sites
/// read this one.
pub fn git_stdout(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .env("GIT_PAGER", "cat")
        .output()
        .map_err(|e| format!("spawn git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// Whether a repository path is one the head sweep would have parsed. The two exclusions mirror
/// `run_dag_parse_sweep`'s: build output, and the parser's deliberately malformed fixtures. A
/// base file the head sweep would not read must not enter the base index, or the two sides are
/// measured by different instruments.
///
/// THIS ANSWERS THE PARSER'S QUESTION AND NOTHING ELSE, AND IT IS APPLIED AT THE POINT OF USE.
/// It used to be applied inside `diff_sides`, so the only available answer to "what did this
/// change touch" was already narrowed to `.dag` — and a second consumer asking a DIFFERENT
/// question read that narrowed list as if it were the diff. `roster_touched` asks about a `.rs`
/// path, a `.rs` path cannot survive a `.dag` filter, and the arm it fed was therefore false on
/// every production run: an upstream filter written for one question silently deciding another,
/// with nothing joining them (`gunbc.recurring_failure_mode` `incidental_denominator_as_wall`).
/// The repair is not a second path list — that would be two representations of one fact, the same
/// class one step later. `diff_sides` reports what the diff touched, once; every consumer applies
/// the scope its own question needs, here.
pub fn in_sweep_scope(rel: &str) -> bool {
    rel.ends_with(".dag")
        && DAG_PARSE_SWEEP_ROOTS
            .iter()
            .any(|root| rel.starts_with(&format!("{root}/")))
        && !rel.contains("/target/")
        && !rel.contains("/tests/fixtures/")
}

/// The two sides of a diff, at file grain: which head paths the change touched, and which base
/// paths must be re-read to reconstruct the baseline.
///
/// THEY ARE NOT THE SAME SET, AND A RENAME IS WHERE THEY COME APART. `git diff --name-only`
/// reports a detected rename as ONE path -- the destination (rename detection is on by default).
/// An earlier revision read that list as both sides, so a renamed module lost its base side: the
/// source was never re-read and the destination is absent from the base tree, so every
/// declaration read as newly added and the wall could refuse an ordinary `.dag` rename over an
/// invented delta. Review 56471 was right to reject it.
///
/// So the diff is read rename-aware, `--name-status -z -M`, each entry contributing to the sides
/// SEPARATELY: a rename gives its destination to head and its source to base, an addition only a
/// head path, a deletion only a base path, a modification the same path to both.
///
/// THE SIDES ARE UNFILTERED, WHICH IS WHAT MAKES THIS ONE AUTHORITY FOR WHAT THE DIFF TOUCHED.
/// Scope is not applied here: it belongs to the QUESTION being asked, not to the diff, and two
/// consumers downstream ask different ones — the base-index reconstruction wants the parser's
/// `in_sweep_scope`; `roster_touched` matches the roster prefix, including a directory path
/// that predicate would drop.
/// A rename may still cross a scope boundary either way, so each consumer applies its own scope
/// PER SIDE at its call site.
pub fn diff_sides(name_status_z: &str) -> (Vec<String>, Vec<String>) {
    let mut head_touched = Vec::new();
    let mut base_side = Vec::new();
    let mut fields = name_status_z.split('\0').filter(|f| !f.is_empty());
    while let Some(status) = fields.next() {
        // A rename or copy carries a similarity score after the letter and TWO paths after that.
        let renamed = status.starts_with('R') || status.starts_with('C');
        let Some(first) = fields.next() else { break };
        if renamed {
            let Some(second) = fields.next() else { break };
            base_side.push(first.to_string());
            head_touched.push(second.to_string());
        } else if status.starts_with('A') {
            head_touched.push(first.to_string());
        } else if status.starts_with('D') {
            base_side.push(first.to_string());
        } else {
            base_side.push(first.to_string());
            head_touched.push(first.to_string());
        }
    }
    (head_touched, base_side)
}

/// One base-side source parsed into records, or a refusal naming what could not be read.
///
/// A BASE FILE THAT DOES NOT PARSE IS AN UNOBSERVABLE BASELINE, NOT AN EMPTY ONE, and the
/// difference decides the verdict. An earlier revision returned an empty vec as "the conservative
/// direction" because it only made the wall quieter. Backwards, and review 56449 was right to
/// reject it: rows with no base side are not deltas, so every row that file carried silently
/// STOPS BEING COMPARED while the run answers `Adjudicated` — the empty-observation narrow,
/// ⊥-as-answer conflated with ⊥-as-ignorance, strictly worse than the widen §5 forbids: a widen
/// is expensive, a narrow is silently uncovered.
///
/// The head sweep refuses on diagnostics, so refusing here keeps both sides on ONE instrument.
/// History is not this PR's to repair — but "I cannot see the baseline" is a refusal to state,
/// not a fact to assume.
///
/// Annotation-grain refusals are the exception named by
/// `fail_closed_gate_refuses_its_own_repair`: the parser still produced the module (annotation
/// bind runs after parse), so the baseline IS observable. Treating those diagnostics as
/// unreadable base sealed the transition that only moves `//` onto the declaration the grain
/// admits. Any other diagnostic, or a file that produced no module, stays unobservable.
/// THE ENVIRONMENT IS REQUIRED, NOT DEFAULTED. A default would be the HEAD grammar, and this
/// function's entire job is reading the BASE revision -- so a forgetful caller would read base text
/// under head rules and answer confidently, which is
/// `gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change` itself. A caller
/// that genuinely means the head environment says so at the call site.
pub fn base_records(
    rel: &str,
    content: &str,
    environment: std::rc::Rc<crate::std_syntax::ParseEnvironment>,
) -> Result<Vec<ModuleDeclarationRecord>, String> {
    let fill = crate::v1_compiler_compile::parse_census_fill_sources_with_environment(
        std::rc::Rc::new(
            vec![std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                path: rel.to_string(),
                content: content.to_string(),
            })]
            .into(),
        ),
        environment,
    );
    let annotation_erased_readable = !fill.modules.is_empty()
        && !fill.diagnostics.is_empty()
        && fill.diagnostics.iter().all(|d| {
            matches!(
                *d.diagnostic,
                crate::v1_std_core::CompilerDiagnostic::SourceAnnotationRefused { .. }
            )
        });
    if !fill.diagnostics.is_empty() && !annotation_erased_readable {
        return Err(format!(
            "{rel} does not parse at the base revision ({} diagnostic(s)), so its base-side \
             declarations cannot be read",
            fill.diagnostics.len()
        ));
    }
    let source_indices: std::rc::Rc<
        im::HashMap<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>,
    > = std::rc::Rc::new(
        fill.newline_indices
            .iter()
            .fold(im::HashMap::new(), |acc, i| {
                acc.update(i.file.clone(), i.clone())
            }),
    );
    Ok(fill
        .modules
        .iter()
        .map(|module| record_from_module(module, &source_indices, rel, &fill.occurrence_transport))
        .collect())
}

/// Run the wall for one required CI invocation.
///
/// THE BASE INDEX IS THE HEAD INDEX WITH THE DIFF APPLIED IN REVERSE, at file grain — the
/// construction, not an optimisation. An untouched file has the same record on both sides;
/// deriving it twice is the second corpus walk DESIGN §6 names. Only changed files are re-parsed
/// from their base blobs and substituted; closure and bindings are then recomputed over BOTH
/// whole graphs, since an unmoved module's subject or bindings can be moved by one that did.
pub fn run_required_wave_admission(
    head_index: &DeclarationIndex,
    event: AdjudicationEvent,
) -> Result<WaveAdmissionOutcome, String> {
    let workspace = workspace_root();
    let head = git_stdout(&workspace, &["rev-parse", "HEAD"])?;
    let base = match git_stdout(&workspace, &["merge-base", "origin/main", "HEAD"]) {
        Ok(base) => base,
        Err(e) => {
            return Ok(WaveAdmissionOutcome::NotEvaluated {
                reason: format!(
                    "the merge base against origin/main does not resolve ({e}). The subject is \
                     NOT widened to the whole corpus and it is NOT admitted: an unobservable \
                     baseline is `NotEvaluated`, which the ruling puts on the refusing side. \
                     Fetch the base first: `git fetch origin main:refs/remotes/origin/main`"
                ),
            })
        }
    };
    run_wave_admission_between(&workspace, &base, &head, head_index, event)
}

/// The wave adjudication over an EXPLICIT repository and revision pair.
///
/// Split from the production entry so the adjudication can be driven over a repository that is not
/// this process's workspace and a base/head pair that is not `merge-base origin/main HEAD` -- which
/// is the only way the grammar-differs arm below can carry executed evidence. Production reaches
/// this through `run_required_wave_admission`; a witness reaches it with a scratch repository whose
/// base and head speak different grammars. Nothing about the adjudication differs between the two
/// callers: the seam selects the subject, never the rules.
/// The base side of one change, reconstructed from the head index, at file grain.
///
/// LIFTED OUT OF `run_wave_admission_between` so the required floor's planning row can ask the
/// same question over ITS OWN comparison window. The two callers resolve different windows on
/// purpose -- the wall compares against the merge base with `origin/main`, the floor against
/// `v2.workflow.floor_diff_observe`'s resolved baseline -- so the refs are parameters and the
/// reconstruction is one function. It is `pub(crate)`: its only callers are in this crate, and a
/// public export would be seed surface growth under the freeze.
pub(crate) enum BaselineReconstruction {
    /// The window's base IS its head: nothing to reconstruct, and not a refusal.
    NoSubject { head: String },
    /// The base could not be observed. NOT an empty base: the two are different states with
    /// different remedies, and conflating them is the empty-observation narrow.
    NotEvaluated { reason: String },
    Reconstructed {
        base: String,
        head: String,
        base_index: DeclarationIndex,
        /// Every head path the diff touched, UNFILTERED -- consumers apply their own scope.
        head_touched: Vec<String>,
    },
}

/// THE BASE INDEX IS THE HEAD INDEX WITH THE DIFF APPLIED IN REVERSE, at file grain -- the
/// construction, not an optimisation -- unless the two revisions speak different grammars, in
/// which case the whole base side is read under the base's own environment (see below). Only
/// changed files are re-parsed from their base blobs and substituted on the ordinary route.
pub(crate) fn reconstruct_base_index(
    workspace: &std::path::Path,
    base: &str,
    head: &str,
    head_index: &DeclarationIndex,
) -> Result<BaselineReconstruction, String> {
    let base = base.to_string();
    let head = head.to_string();
    let workspace = workspace.to_path_buf();
    if base == head {
        return Ok(BaselineReconstruction::NoSubject { head });
    }

    // WHICH GRAMMAR DOES THE BASE SPEAK? Everything below reads base-side declarations, and reading
    // them under the head's grammar is the defect this phase kept tripping over: a change that edits
    // the language refuses in proportion to how thoroughly it succeeded
    // (gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change).
    //
    // The cheap answer comes first. Object identity over the files declaring the environment settles
    // "same grammar?" in a few `rev-parse` calls, so the ordinary pull request -- which changes no
    // grammar -- pays nothing, and only a real grammar change pays to materialize and evaluate the
    // base corpus.
    let agreement = match environment_agreement(&workspace, &base, &head) {
        Ok(a) => a,
        // A REFUSAL HERE IS NOT A LICENCE TO USE THE HEAD'S. Not knowing which grammar the base
        // speaks makes every base-side declaration unreadable, which is ignorance, and ignorance is
        // NotEvaluated rather than a confident answer under the wrong rules.
        Err(e) => {
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!(
                    "the base revision's parse environment could not be established ({}), so its declarations cannot be read under any grammar this run can justify", environment_load_refusal_text(&e)
                ),
            })
        }
    };

    // THE KERNEL HALF, GUARDED NARROWLY. `declaring_candidates` consults this binary's own
    // `kernel_type_set`, a head fact. Equal declaring blobs mean both revisions name the same kernel
    // and one map serves; different blobs leave the question open, and an open question refuses.
    match kernel_set_serves_both(&workspace, &base, &head) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!(
                    "{KERNEL_TYPES_PATH} differs between {base} and {head}, so the kernel-name set this binary carries cannot speak for the base side"
                ),
            })
        }
        Err(e) => {
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!("the kernel declaring file could not be compared ({})", environment_load_refusal_text(&e)),
            })
        }
    }

    let name_status = git_stdout(
        &workspace,
        &["diff", "--name-status", "-z", "-M", &base, &head],
    )?;
    let (head_touched, base_side) = diff_sides(&name_status);

    // THE PARSER'S SCOPE IS APPLIED HERE, WHERE THE PARSER'S QUESTION IS ASKED, AND NOWHERE ELSE.
    // `head_touched` and `base_side` are what the diff touched; these two are what the baseline
    // reconstruction may read. Filtering per side rather than once is not redundancy: a rename may
    // cross the sweep boundary in either direction, which is why `diff_sides` splits the sides in
    // the first place. Everything below that asks a DIFFERENT question — `roster_touched` — reads
    // the unfiltered list, because prefix match is not the parse-sweep predicate.
    let head_parsed: Vec<&String> = head_touched.iter().filter(|p| in_sweep_scope(p)).collect();
    let base_parsed: Vec<&String> = base_side.iter().filter(|p| in_sweep_scope(p)).collect();

    // A DERIVED ARTIFACT IS NEVER IN THE DIFF, SO IT MUST NOT BE INHERITED FROM THE HEAD.
    // The baseline is reconstructed by carrying every head record the diff did not touch and
    // re-reading the rest from the base tree. `roster.dag` is gitignored and written on the read
    // path, so the diff can never name it — and carrying it made the HEAD's roster stand as the
    // BASE's. Its base side is not read from git either (the tree does not carry it); it is
    // DERIVED from the base tree's row membership, below, by the same renderer the writer uses.
    //
    // CARRYING UNTOUCHED HEAD RECORDS IS VALID ONLY WHILE THE GRAMMARS AGREE. The reconstruction
    // below keeps every head record the diff did not touch, which assumes an untouched FILE has an
    // untouched PARSE. That holds when both revisions speak one grammar and fails exactly when they
    // do not: a keyword, literal, operator or item-form change gives an untouched file a different
    // parse, so its head records are not its base records. When the environments differ, nothing is
    // carried and every base-side file in sweep scope is read under the base's own environment.
    let (base_environment, full_base_parse, differing) = match &agreement {
        EnvironmentAgreement::Identical => (
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
            false,
            Vec::new(),
        ),
        EnvironmentAgreement::Differs {
            base_environment,
            differing_paths,
        } => (base_environment.clone(), true, differing_paths.clone()),
    };
    if full_base_parse {
        eprintln!(
            "namespace-wave-admission: the base and head parse environments differ ({}), so the \
             baseline is read in full under the base's own grammar rather than reconstructed from \
             untouched head records",
            differing.join(", ")
        );
    }

    let mut base_index = DeclarationIndex::default();
    if !full_base_parse {
        for record in index_records(head_index) {
            if crate::cli_run::derived_row_roster::is_derived_roster_path(&record.rel_path) {
                continue;
            }
            if !head_parsed.iter().any(|c| *c == &record.rel_path) {
                crate::cli_run::declaration_index::index_insert(&mut base_index, record.clone());
            }
        }
    }
    // ABSENCE AT THE BASE IS ESTABLISHED FROM AN AUTHORITATIVE LISTING, NEVER INFERRED FROM A
    // FAILURE. An earlier revision treated ANY `git show <base>:<path>` error as proof the path
    // was ADDED — a read fault, corrupt object or permission problem all read as "new file", and
    // the module's base side vanished while the run answered `Adjudicated`. Review 56449 was
    // right to reject it: only ONE cause means added; the rest are ignorance wearing its clothes.
    //
    // `ls-tree` answers what the base tree CONTAINS: a path missing from its output is absent,
    // and a failure to obtain the listing is a refusal, not an empty answer.
    let base_paths = git_stdout(&workspace, &["ls-tree", "-r", "--name-only", &base])?;
    let base_paths: BTreeSet<String> = base_paths.lines().map(|l| l.trim().to_string()).collect();
    // WHEN THE GRAMMARS DIFFER THE READ SET IS THE WHOLE BASE SIDE, not the diff's. The diff is a
    // statement about bytes; a grammar change is a statement about every file's parse.
    let owned_full: Vec<String> = if full_base_parse {
        base_paths
            .iter()
            .filter(|p| in_sweep_scope(p))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let read_set: Vec<&String> = if full_base_parse {
        owned_full.iter().collect()
    } else {
        base_parsed.clone()
    };
    // ONE ACQUISITION FOR THE WHOLE READ SET, not one `git show` per path. On the grammar-differs
    // route the read set is every base-side file in sweep scope -- thousands -- and the process-
    // per-file shape this prerequisite removed from the loader must not survive one layer down in
    // its consumer. The paths the base actually carries are archived once and read locally.
    let present: Vec<&str> = read_set
        .iter()
        .filter(|rel| base_paths.contains(**rel))
        .map(|rel| rel.as_str())
        .collect();
    let base_tree = workspace.join("target").join(format!(
        "gunbc-wave-base-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    if !present.is_empty() {
        if let Err(e) = materialize_revision_paths(&workspace, &base, &base_tree, &present) {
            let _ = std::fs::remove_dir_all(&base_tree);
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!(
                    "the base revision's files could not be materialized ({}), so the baseline is \
                     unobservable and no verdict is available",
                    environment_load_refusal_text(&e)
                ),
            });
        }
    }
    for rel in &read_set {
        if !base_paths.contains(*rel) {
            // Genuinely added by this change: no base side to read, established by the listing.
            continue;
        }
        // The listing says the base carries this path, so a read failure here is UNOBSERVABLE
        // BASELINE, not news about the file.
        let content = match std::fs::read_to_string(base_tree.join(rel)) {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&base_tree);
                return Ok(BaselineReconstruction::NotEvaluated {
                    reason: format!(
                        "cannot read {rel} at the base revision {base} ({e}), so the baseline is \
                         partially unobservable and no verdict is available"
                    ),
                });
            }
        };
        match base_records(rel, &content, base_environment.clone()) {
            Ok(records) => {
                for record in records {
                    crate::cli_run::declaration_index::index_insert(&mut base_index, record);
                }
            }
            // A BASE THIS PARSER CANNOT READ IS UNEVALUATED, FULL STOP. The annotation case that
            // used to be repaired here is now answered truthfully upstream: `base_records` reads
            // the real base declarations out of an annotation-refused parse via
            // `annotation_erased_readable`, so the only reason left to reach this arm is a base
            // carrying NON-annotation diagnostics. Substituting head records there is exactly the
            // fabricated parse the readable path exists to avoid -- the remainders would still
            // compare equal whenever the head touched only comments, so the old discriminator
            // would happily certify a baseline for a file that failed to parse for an unrelated
            // reason. The residual case argues for deletion, not for retention.
            Err(reason) => {
                let _ = std::fs::remove_dir_all(&base_tree);
                return Ok(BaselineReconstruction::NotEvaluated { reason });
            }
        }
    }
    let _ = std::fs::remove_dir_all(&base_tree);

    // THE DERIVED ROSTER'S BASE SIDE, from the base tree's row membership. `base_paths` is the
    // authoritative listing already in hand, so this asks the same question the writer asks of a
    // directory. A base tree carrying no row files under that root has no roster module at all,
    // and `roster_from_path_listing` answers `None` rather than fabricating a present empty list.
    let base_path_refs: Vec<&str> = base_paths.iter().map(|p| p.as_str()).collect();
    for record in index_records(head_index) {
        let Some(root) = crate::cli_run::derived_row_roster::roster_root_prefix(&record.rel_path)
        else {
            continue;
        };
        let Some(content) = crate::cli_run::derived_row_roster::roster_from_path_listing(
            base_path_refs.iter().copied(),
            root,
        ) else {
            continue;
        };
        // SYNTHESIZED BY THE CURRENT RENDERER, SO PARSED UNDER THE CURRENT GRAMMAR. This content is
        // not bytes read from the base tree; it is new source the head's roster writer produced from
        // the base tree's path membership. The base environment is reserved for bytes that actually
        // came out of the base revision -- sending renderer output through an older grammar could
        // refuse text no historical source ever carried.
        match base_records(
            &record.rel_path,
            &content,
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        ) {
            Ok(records) => {
                for record in records {
                    crate::cli_run::declaration_index::index_insert(&mut base_index, record);
                }
            }
            Err(reason) => return Ok(BaselineReconstruction::NotEvaluated { reason }),
        }
    }

    Ok(BaselineReconstruction::Reconstructed {
        base,
        head,
        base_index,
        head_touched,
    })
}

/// The wave adjudication over an EXPLICIT repository and revision pair.
///
/// Split from the production entry so the adjudication can be driven over a repository that is not
/// this process's workspace and a base/head pair that is not `merge-base origin/main HEAD` -- which
/// is the only way the grammar-differs arm below can carry executed evidence. Production reaches
/// this through `run_required_wave_admission`; a witness reaches it with a scratch repository whose
/// base and head speak different grammars. Nothing about the adjudication differs between the two
/// callers: the seam selects the subject, never the rules. The CI event travels with the subject
/// for the same reason: the consumption obligation differs by event (`AdjudicationEvent`), so the
/// caller states which run this is rather than the seam inferring it.
pub fn run_wave_admission_between(
    workspace: &std::path::Path,
    base: &str,
    head: &str,
    head_index: &DeclarationIndex,
    event: AdjudicationEvent,
) -> Result<WaveAdmissionOutcome, String> {
    let admissions = load_transition_admissions(workspace)?;
    let (base, head, base_index, head_touched) =
        match reconstruct_base_index(workspace, base, head, head_index)? {
            BaselineReconstruction::NoSubject { head } => {
                if admissions.is_empty() {
                    return Ok(WaveAdmissionOutcome::NoSubject { head });
                }
                // Landing owns roster debt even though it has no namespace delta to compare.
                return Ok(WaveAdmissionOutcome::Adjudicated {
                    base: head.clone(),
                    head,
                    report: Box::new(adjudicate(head_index, head_index, &admissions)),
                    roster_touched: false,
                    event,
                });
            }
            BaselineReconstruction::NotEvaluated { reason } => {
                return Ok(WaveAdmissionOutcome::NotEvaluated { reason })
            }
            BaselineReconstruction::Reconstructed {
                base,
                head,
                base_index,
                head_touched,
            } => (base, head, base_index, head_touched),
        };

    // READ FROM THE UNFILTERED HEAD SIDE. `roster_touched` matches the roster directory prefix,
    // not `in_sweep_scope`; row files are `.dag` and in sweep, but a directory path is not.
    let roster_touched = head_touched
        .iter()
        .any(|p| admission_roster_path_touched(p));
    let report = adjudicate(&base_index, head_index, &admissions);
    Ok(WaveAdmissionOutcome::Adjudicated {
        base,
        head,
        report: Box::new(report),
        roster_touched,
        event,
    })
}

// THE PARSE ENVIRONMENT OF A REVISION THAT IS NOT THE RUNNING BINARY'S.
//
// `ParseEnvironment` (`std.syntax`) is threaded through the tokenizer and parser so that reading a
// revision's source does not mean reading it under whatever grammar this binary was built with.
// That thread is inert until something can PRODUCE an environment other than the compiled-in
// `dag_parse_environment`. This is that producer, and the revision SELECTS THE SOURCE: the bytes
// evaluated are the bytes git holds at that revision, not the worktree's.
//
// WHY: `gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change`. A gate that
// compares base-side declarations against head-side ones parses both with one compiler, and has no
// representable arm for "the base is well formed under its OWN grammar and unreadable only under
// the head's" -- so a change that edits the grammar is refused in proportion to how thoroughly it
// succeeded.
//
// ONE MATERIALIZATION, THEN THE REPOSITORY'S OWN INDEX. An earlier revision of this code listed the
// whole `.dag` tree and ran one `git show` per file -- 5,306 subprocesses on every required run --
// and recognized `module` and `import` with its own line-prefix scanner, which is a second grammar
// for declarations the module index already recognizes (section 3). Both are gone: one
// `git archive` writes the revision's `dag/` tree into a caller-owned directory, and the real
// module index and entry resolver read it from there. The loader therefore cannot disagree with the
// compiler about what a module is, because it does not decide.
//
// DECODE IS NOT HAND-WRITTEN. `Value` -> `value_to_wire_json` -> `serde_json::from_value`: the wire
// encoder resolves its tag policy from the same emitter that wrote the `#[serde(...)]` attributes
// on the mirror struct, so encoder and decoder cannot disagree about shape unless the emitter
// disagrees with itself. A hand-written decoder would fork the type's shape across nine types and
// drift the first time a field was added to `SyntaxSpec`.
//
// WHAT IT DOES NOT COVER. This reproduces the DECLARATIVE environment: which words are keywords,
// which item forms exist, which operators bind how. It does NOT reproduce the revision's PARSER --
// body parsers are dispatched on `body_kind` to code compiled into this binary, so a revision whose
// body parser behaved differently is not reproduced by supplying its environment and must not be
// claimed to be. That population stays outside the covered set.

/// The module whose declarations ARE the dag realization's parse environment.
const ENVIRONMENT_MODULE: &str = "extdeps.languages.dag.syntax";
/// The data item within it that carries the environment value.
const ENVIRONMENT_ITEM: &str = "dag_parse_environment";
/// The path, relative to the repository root, of the file declaring `ENVIRONMENT_MODULE`.
const ENVIRONMENT_MODULE_PATH: &str = "dag/extdeps/languages/dag/syntax.dag";
/// Where the corpus of `.dag` declarations lives, relative to the repository root.
const DAG_SOURCE_ROOT: &str = "dag";

/// Why an environment could not be produced for a revision.
///
/// EVERY ARM IS A REFUSAL, NEVER A SUBSTITUTION. The tempting arm when a base environment cannot be
/// read is to fall back to the head's -- precisely the assumption this code exists to remove, and it
/// would fail open on exactly the changes that alter the grammar. Section 5's absorbing fallback in
/// its purest form: nothing is missed, so the arm reads as safe, while the only signal that the base
/// was unreadable is destroyed. So the failure is typed and located and the caller decides what an
/// unreadable base means for its own verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentLoadRefusal {
    /// `git` could not be run, or answered non-zero, while reading this revision.
    RevisionUnreadable {
        revision: String,
        step: String,
        cause: String,
    },
    /// The revision's materialized tree has no file at the environment module's path.
    EnvironmentModuleMissing { revision: String, path: String },
    /// The materialized corpus did not resolve, or the item did not evaluate.
    ClosureNotEvaluable { revision: String, cause: String },
    /// `ENVIRONMENT_ITEM` is not declared by `ENVIRONMENT_MODULE` at this revision.
    ///
    /// Separate from `ClosureNotEvaluable` because it is the HOMONYM refusal: the interpreter
    /// resolves a data item by bare name across the whole closure, so without this check a
    /// `dag_parse_environment` declared anywhere else could silently supply the grammar. The
    /// environment must come from the declaration that owns it.
    EnvironmentItemNotOwned {
        revision: String,
        item: String,
        module: String,
    },
    /// The item evaluated, but its value did not decode into the typed environment.
    ///
    /// The arm that fires on emitted-schema drift -- a field the wire encoder omits that the mirror
    /// struct requires. Separate from `ClosureNotEvaluable` because the owners differ: that one is a
    /// defect in the revision being read, this one is THIS binary disagreeing with its own emitter.
    ValueNotDecodable { revision: String, cause: String },
}

/// The operator-facing text of a refusal.
///
/// A FREE FUNCTION, NOT A `Display` IMPL, because this module's seed-growth roster enumerates every
/// declaration it carries by `DeclarationRef`, and an `impl` block is the one item that roster
/// structurally cannot cite -- the reason an earlier lane converted the module's methods to free
/// functions. An earlier revision of this change added `impl Display` here and left the roster's
/// "carries no impl block" sentence standing over it; this keeps the sentence true.
pub fn environment_load_refusal_text(refusal: &EnvironmentLoadRefusal) -> String {
    match refusal {
        EnvironmentLoadRefusal::RevisionUnreadable {
            revision,
            step,
            cause,
        } => format!("reading revision {revision} failed at {step}: {cause}"),
        EnvironmentLoadRefusal::EnvironmentModuleMissing { revision, path } => format!(
            "{path} does not exist at revision {revision}, so that revision's parse \
                 environment cannot be read"
        ),
        EnvironmentLoadRefusal::ClosureNotEvaluable { revision, cause } => format!(
            "the parse environment closure at revision {revision} did not evaluate: {cause}"
        ),
        EnvironmentLoadRefusal::EnvironmentItemNotOwned {
            revision,
            item,
            module,
        } => format!(
            "`{item}` is not declared by `{module}` at revision {revision}, so the value a \
                 bare-name lookup would return is not the grammar authority"
        ),
        EnvironmentLoadRefusal::ValueNotDecodable { revision, cause } => format!(
            "the parse environment at revision {revision} evaluated but did not decode into \
                 this binary's `ParseEnvironment`: {cause}"
        ),
    }
}

/// Run one `git` invocation to completion, or refuse with what it said.
fn git_capture(
    repo: &std::path::Path,
    revision: &str,
    step: &str,
    args: &[&str],
) -> Result<Vec<u8>, EnvironmentLoadRefusal> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: step.to_string(),
            cause: format!("git failed to start: {e}"),
        })?;
    if !out.status.success() {
        return Err(EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: step.to_string(),
            cause: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    Ok(out.stdout)
}

/// The object id git holds for one path at one revision, or `None` if the path is absent.
///
/// Used to decide whether two revisions share a parse environment WITHOUT materializing either:
/// object identity is content identity, so equal ids over the environment's declaring files mean the
/// environments are equal by construction rather than by comparison.
pub fn blob_id_at(
    repo: &std::path::Path,
    revision: &str,
    path: &str,
) -> Result<Option<String>, EnvironmentLoadRefusal> {
    let spec = format!("{revision}:{path}");
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &spec])
        .current_dir(repo)
        .output()
        .map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: format!("rev-parse {spec}"),
            cause: format!("git failed to start: {e}"),
        })?;
    if !out.status.success() {
        return Ok(None);
    }
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

/// Materialize one revision's ENVIRONMENT CLOSURE under `dest`, in a single git invocation.
///
/// ONLY THE CLOSURE, NEVER THE WHOLE TREE, and the reason is the class this loader exists to
/// repair, met one level down. The module index that resolves the environment parses every `.dag`
/// file it is shown with THIS binary's grammar. Materializing the whole `dag/` tree therefore
/// parsed every base-side file under the head's grammar in order to learn the base's grammar --
/// and a base file written in the base's grammar refused inside the loader before the environment
/// was ever evaluated. The grammar-differs witness caught exactly that. So the index is shown only
/// the files the environment's declaring closure consists of.
///
/// THE HONEST BOUNDARY THIS DRAWS: the loader can read a base whose grammar differs from the head's
/// so long as the base's ENVIRONMENT CLOSURE is itself readable under the head's grammar. A change
/// that alters the grammar AND uses the altered grammar inside `std.syntax`'s own closure is outside
/// any head-built loader's reach -- the same bootstrap boundary the compiler itself has -- and it
/// refuses as `ClosureNotEvaluable`, never answering under the wrong rules.
///
/// `git archive | tar -x` over the named paths rather than a read per file: acquisition cost must
/// not scale with the corpus (section 6, bare minimum cost). `dest` is the caller's to remove.
fn materialize_environment_closure_at(
    repo: &std::path::Path,
    revision: &str,
    dest: &std::path::Path,
    closure_paths: &BTreeSet<String>,
) -> Result<(), EnvironmentLoadRefusal> {
    let paths: Vec<&str> = closure_paths.iter().map(String::as_str).collect();
    materialize_revision_paths(repo, revision, dest, &paths)?;
    if !dest.join(ENVIRONMENT_MODULE_PATH).exists() {
        return Err(EnvironmentLoadRefusal::EnvironmentModuleMissing {
            revision: revision.to_string(),
            path: ENVIRONMENT_MODULE_PATH.to_string(),
        });
    }
    Ok(())
}

/// Materialize the named paths of one revision under `dest`: one `git archive`, one `tar -x`.
///
/// THE ONE ACQUISITION ROUTE. Every consumer that needs a revision's bytes on disk -- the
/// environment loader above for its closure, the grammar-differs and provenance witnesses for a
/// whole `dag/` tree -- comes through here, so acquisition is checked once and a defect in it is
/// found once. An earlier shape had the witnesses carrying their own `sh -c "git archive | tar"`
/// string beside this function: the same pipeline twice over, one of them unchecked hand-shell. A
/// pathspec may name a directory (`dag`) or a file; git archive accepts both.
pub fn materialize_revision_paths(
    repo: &std::path::Path,
    revision: &str,
    dest: &std::path::Path,
    paths: &[&str],
) -> Result<(), EnvironmentLoadRefusal> {
    std::fs::create_dir_all(dest).map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
        revision: revision.to_string(),
        step: "create materialization directory".to_string(),
        cause: e.to_string(),
    })?;
    let mut args: Vec<&str> = vec!["archive", "--format=tar", revision];
    args.extend_from_slice(paths);
    let archive = git_capture(repo, revision, "archive", &args)?;
    let tar_path = dest.join("dag-tree.tar");
    std::fs::write(&tar_path, &archive).map_err(|e| {
        EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: "write archive".to_string(),
            cause: e.to_string(),
        }
    })?;
    let extract = std::process::Command::new("tar")
        .arg("-xf")
        .arg(&tar_path)
        .arg("-C")
        .arg(dest)
        .output()
        .map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: "tar -xf".to_string(),
            cause: format!("tar failed to start: {e}"),
        })?;
    if !extract.status.success() {
        return Err(EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: "tar -xf".to_string(),
            cause: String::from_utf8_lossy(&extract.stderr).trim().to_string(),
        });
    }
    let _ = std::fs::remove_file(&tar_path);
    Ok(())
}

/// Decode an evaluated environment value into this binary's `ParseEnvironment`.
pub fn decode_environment_value(
    value: &crate::v1_interpreter::Value,
    ctx: &crate::v1_interpreter::InterpContext,
    revision: &str,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let wire = super::value_to_wire_json(value, ctx).map_err(|e| {
        EnvironmentLoadRefusal::ValueNotDecodable {
            revision: revision.to_string(),
            cause: format!("wire-encode: {e}"),
        }
    })?;
    serde_json::from_value::<crate::std_syntax::ParseEnvironment>(wire)
        .map(std::rc::Rc::new)
        .map_err(|e| EnvironmentLoadRefusal::ValueNotDecodable {
            revision: revision.to_string(),
            cause: e.to_string(),
        })
}

/// Evaluate the parse environment out of an already-materialized corpus rooted at `root`.
///
/// Split from acquisition so the decode seam can be exercised against an independent oracle -- the
/// compiled-in `dag_parse_environment()` over the live tree -- without a revision in the way. The
/// revision string here is diagnostic ONLY; callers that mean "the environment AT a revision" must
/// use `load_parse_environment_at`, which selects the source.
pub fn evaluate_environment_in(
    root: &std::path::Path,
    revision: &str,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let entry = root.join(ENVIRONMENT_MODULE_PATH);
    if !entry.exists() {
        return Err(EnvironmentLoadRefusal::EnvironmentModuleMissing {
            revision: revision.to_string(),
            path: entry.display().to_string(),
        });
    }
    let dag_root = root.join(DAG_SOURCE_ROOT);
    let index = super::build_multi_entry_index(&[dag_root.display().to_string()]);
    let entry_display = entry.display().to_string();
    let (graph, indices) =
        super::resolve_entry_with_index_for_discovery_corpus(&index, &entry_display).map_err(
            |e| EnvironmentLoadRefusal::ClosureNotEvaluable {
                revision: revision.to_string(),
                cause: e,
            },
        )?;
    // HERMETIC, NOT WET. A static grammar declaration has no business acquiring permission to
    // perform host effects while it is being decoded; `Wet` here would let a corpus under
    // examination act during examination.
    let ctx = super::make_eval_context(
        &graph,
        indices,
        crate::v1_interpreter::ExecutionMode::Hermetic,
    );
    // EXACT OWNERSHIP, NOT A BARE NAME. `eval_data_item_value` resolves by bare name across the
    // closure, so a homonymous `dag_parse_environment` elsewhere in the corpus would silently
    // supply the grammar. The environment must come from the declaration that owns it.
    if !super::data_item_declared_in_file(&ctx, ENVIRONMENT_ITEM, &entry_display) {
        return Err(EnvironmentLoadRefusal::EnvironmentItemNotOwned {
            revision: revision.to_string(),
            item: ENVIRONMENT_ITEM.to_string(),
            module: ENVIRONMENT_MODULE.to_string(),
        });
    }
    let value = crate::v1_interpreter::with_active_context(&ctx, || {
        crate::v1_interpreter::eval_data_item_value(&ctx, ENVIRONMENT_ITEM)
    })
    .map_err(|e| EnvironmentLoadRefusal::ClosureNotEvaluable {
        revision: revision.to_string(),
        cause: format!("eval {ENVIRONMENT_ITEM}: {e}"),
    })?
    .ok_or_else(|| EnvironmentLoadRefusal::ClosureNotEvaluable {
        revision: revision.to_string(),
        cause: format!("{ENVIRONMENT_ITEM} is not a data item in `{ENVIRONMENT_MODULE}`"),
    })?;
    decode_environment_value(&value, &ctx, revision)
}

/// THE LOADER: the parse environment git holds at `revision`.
///
/// The revision selects the bytes. Materialization happens into a temporary directory this function
/// owns and removes, so nothing about the caller's worktree is read or written.
pub fn load_parse_environment_at(
    repo: &std::path::Path,
    revision: &str,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let closure = environment_closure_paths()?;
    load_parse_environment_with_closure(repo, revision, &closure)
}

/// The loader over an ALREADY-RESOLVED closure.
///
/// `environment_agreement` resolves the closure to decide whether the grammars differ and then, on
/// the differing path, loads the base's environment -- the same closure, same inputs, with the
/// caller already holding the answer. Section 2: carry the first value rather than recompute it at
/// the least common ancestor. This is that carried value; `load_parse_environment_at` resolves once
/// for callers that hold nothing.
pub fn load_parse_environment_with_closure(
    repo: &std::path::Path,
    revision: &str,
    closure: &BTreeSet<String>,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dest =
        // UNDER THE WORKSPACE, NOT /tmp. The module index and entry resolver refuse any path outside
        // the workspace root (`repo_relative_path_normalized`), so a corpus materialized into the
        // system temp directory cannot be read by the repository's own machinery -- the loader would
        // refuse every call, in production as much as in test. `target/` is where generated and
        // scratch trees already live (`target/stage0-regen-candidate` is the precedent).
        super::workspace_root()
            .join("target")
            .join(format!("gunbc-parse-env-{}-{}", std::process::id(), stamp));
    // If the base's closure has a member the head's does not, the materialized set is incomplete
    // and resolution refuses as ClosureNotEvaluable -- a located refusal, not a fabricated read.
    let outcome = materialize_environment_closure_at(repo, revision, &dest, closure)
        .and_then(|()| evaluate_environment_in(&dest, revision));
    let _ = std::fs::remove_dir_all(&dest);
    outcome
}

/// The repo-relative files that declare the parse environment's closure, per the real resolver.
///
/// ASKED OF THE RESOLVER, NOT LISTED. The closure is 13 modules today and that number appears
/// nowhere: a hardcoded roster would be a second authority for what the grammar depends on and would
/// go stale, silently, the first time `std.syntax` gained an import -- a stale roster still resolves.
/// The resolved graph's own span files ARE the closure.
pub fn environment_closure_paths() -> Result<BTreeSet<String>, EnvironmentLoadRefusal> {
    let root = super::workspace_root();
    let entry = root.join(ENVIRONMENT_MODULE_PATH);
    let dag_root = root.join(DAG_SOURCE_ROOT);
    let index = super::build_multi_entry_index(&[dag_root.display().to_string()]);
    let (graph, _indices) =
        super::resolve_entry_with_index_for_discovery_corpus(&index, &entry.display().to_string())
            .map_err(|e| EnvironmentLoadRefusal::ClosureNotEvaluable {
                revision: "live-tree".to_string(),
                cause: e,
            })?;
    let mut paths = BTreeSet::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            let file = item.span.file.clone();
            if let Some(idx) = file.find(&format!("{DAG_SOURCE_ROOT}/")) {
                paths.insert(file[idx..].to_string());
            }
        }
    }
    if paths.is_empty() {
        return Err(EnvironmentLoadRefusal::ClosureNotEvaluable {
            revision: "live-tree".to_string(),
            cause: "the resolved environment closure named no files, so no agreement check is \
                    possible"
                .to_string(),
        });
    }
    Ok(paths)
}

/// Whether two revisions share a parse environment, and the base's environment when they do not.
#[derive(Debug, Clone)]
pub enum EnvironmentAgreement {
    /// Every file declaring the environment is byte-identical across the two revisions.
    ///
    /// Equal object ids are equal content, so the environments are identical BY CONSTRUCTION rather
    /// than by a comparison that could be wrong. Nothing needs loading, and the changed-file
    /// baseline reconstruction stays valid.
    Identical,
    /// The declaring files differ, so the base must be read under its own environment.
    Differs {
        base_environment: std::rc::Rc<crate::std_syntax::ParseEnvironment>,
        differing_paths: Vec<String>,
    },
}

/// Decide whether the base and head grammars agree, loading the base's environment only if not.
///
/// THE CHEAP CHECK COMES FIRST because the expensive one must not be paid on every run: object
/// identity over the closure's files answers "same grammar?" with a handful of `rev-parse` calls,
/// and only a real difference pays for materializing and evaluating the base corpus. The ordinary
/// pull request changes no grammar and therefore costs nothing here.
pub fn environment_agreement(
    repo: &std::path::Path,
    base: &str,
    head: &str,
) -> Result<EnvironmentAgreement, EnvironmentLoadRefusal> {
    let closure = environment_closure_paths()?;
    let mut differing = Vec::new();
    for path in &closure {
        if blob_id_at(repo, base, path)? != blob_id_at(repo, head, path)? {
            differing.push(path.clone());
        }
    }
    if differing.is_empty() {
        return Ok(EnvironmentAgreement::Identical);
    }
    Ok(EnvironmentAgreement::Differs {
        base_environment: load_parse_environment_with_closure(repo, base, &closure)?,
        differing_paths: differing,
    })
}

/// The path whose declarations the kernel-name set is derived from.
const KERNEL_TYPES_PATH: &str = "dag/std/types.dag";

/// Whether the kernel type set this binary carries can speak for both revisions.
///
/// NARROW ON PURPOSE. `declaring_candidates` consults the RUNNING compiler's `kernel_type_set`,
/// which is a fact about the head. Threading distinct base and head kernel maps is the general
/// repair and is not this change's subject; what is needed here is honesty about when the single map
/// is adequate. Equal blobs for the declaring file means both revisions name the same kernel, so one
/// map serves. Different blobs means the question is open, and an open question is `NotEvaluated` --
/// not a guess that the head's map is close enough.
pub fn kernel_set_serves_both(
    repo: &std::path::Path,
    base: &str,
    head: &str,
) -> Result<bool, EnvironmentLoadRefusal> {
    Ok(blob_id_at(repo, base, KERNEL_TYPES_PATH)? == blob_id_at(repo, head, KERNEL_TYPES_PATH)?)
}
