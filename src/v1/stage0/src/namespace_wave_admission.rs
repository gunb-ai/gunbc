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
/// Its borrowed fields keep the admission roster const: no initializer can compute permission
/// from observed deltas, a file, or process state. Runtime observations remain owned
/// `DeltaSubject` values — a distinct type from an authored pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionSubject {
    Membership {
        module: &'static str,
        target: &'static str,
    },
    Binding {
        module: &'static str,
        in_declaration: &'static str,
        spelling: &'static str,
        /// Where the relocated spelling now resolves — the module the admission's own PR moved
        /// the name TO. Not consulted by delta matching (the delta subject carries no target);
        /// it is the referent of the CONSUMPTION proof: after the admitting PR merges, the base
        /// itself binds the spelling to exactly this module, which is the positive, decidable
        /// fact that separates a consumed row from an author-error row. A row that cannot name
        /// where its name went is not an admission of a relocation.
        target: &'static str,
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
        ) => *module == observed_module && *target == observed_target,
        (
            AdmissionSubject::Binding {
                module,
                in_declaration,
                spelling,
                target: _,
            },
            DeltaSubject::Binding {
                module: observed_module,
                in_declaration: observed_declaration,
                spelling: observed_spelling,
            },
        ) => {
            *module == observed_module
                && *in_declaration == observed_declaration
                && *spelling == observed_spelling
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
            target,
        } => format!("binding {module}::{in_declaration} `{spelling}` -> {target}"),
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
    pub label: &'static str,
    pub subject: AdmissionSubject,
    pub disposition: NamespaceDeltaDisposition,
}

/// EMPTY IS THE RESTING STATE between transitions, and it is not permissive: a run with a real
/// delta still refuses it as UNADJUDICATED, closed by authoring a row and never by a silent
/// admission. That is a claim about the MECHANISM and it holds whatever the roster contains.
///
/// TWENTIETH DISSOLUTION (2026-09-03). The 47 `rung drop authority consolidation gunbc#10106` rows
/// are removed by their own trigger, which fired: #10106 merged, so the base binds every one of
/// those spellings to its target and no run after it can produce those deltas. They arrived on this
/// branch through a merge of main rather than by being authored here, and the required run
/// (33772917866) reported each of them as `CONSUMED ADMISSION ... already satisfied at the base --
/// consumed by its own merge; deletion is owed on the roster's next touch`. The label constant they
/// shared went with them; a constant naming a landed subject has no second use.
///
/// THE OBLIGATION IS CHARGED TO THE TOUCHER AND THIS MERGE IS ONE. A row whose subject has landed is
/// not merely useless: it is stale on every subsequent run, so leaving it refuses unrelated changes.
/// The roster shrinks with its subject or it becomes a tax on everyone downstream. This is the same
/// rule the eighteenth and nineteenth dissolutions paid, and it is why no follow-up PR is owed.
///
/// THE CONFLICT RULE, RESTATED BECAUSE IT WAS ABOUT TO BE MISAPPLIED AGAIN: on a conflict in this
/// roster, union the rows whose transitions are still OPEN and delete the rows whose transitions
/// have MERGED. The discriminator is the RECEIPT, never which branch authored the row. Taking main's
/// side whole is what put 47 consumed rows here, and the gate is what said so.
///
/// ONE ROW STANDS (gunbc#10218), for an occurrence-binding relocation whose transition is open.
/// `physical_asset_identity_eq` was authored privately inside
/// product.printed_chassis.manufacturing_manifest and now resolves in product.placement_supply,
/// which OWNS PhysicalAssetIdentity and already carries host_identity_eq for the sibling branded
/// type. The private copy was tolerable while one module consumed it and stopped being tolerable
/// when product.inventory needed the same comparison: inventory is a generic authority, so importing
/// a specific product's helper to obtain an equality would invert the layering. The spelling is
/// unchanged on both sides and only its TARGET moved, which is exactly TargetChanged.
///
/// ITS CONSUMPTION IS DECIDABLE ON THE SAME RULE THE ROWS ABOVE PAID: once gunbc#10218 merges, the
/// base binds the spelling to product.placement_supply, the delta stops being producible, and this
/// row is owed deletion by the next roster-touching change.
pub const NAMESPACE_TRANSITION_ADMISSIONS: &[TransitionAdmission] = &[TransitionAdmission {
    label: "gunbc#10218 identity-equality re-home: PhysicalAssetIdentity comparison moves to \
            the module that owns the type",
    subject: AdmissionSubject::Binding {
        module: "product.printed_chassis.manufacturing_manifest",
        in_declaration: "scan_printer_assets",
        spelling: "physical_asset_identity_eq",
        target: "product.placement_supply",
    },
    disposition: NamespaceDeltaDisposition::TargetChanged,
}];

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
    /// only on the POSITIVE proof `admission_consumed_at_base`, never as the else-arm of "did
    /// not match a delta" — a row provable against neither side stays an UnmatchedAdmission
    /// refusal in `stale_admissions`.
    pub consumed_admissions: Vec<String>,
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
    for delta in deltas.iter_mut() {
        for (i, admission) in admissions.iter().enumerate() {
            if admission_subject_matches(&admission.subject, &delta.subject)
                && admission.disposition == delta.disposition
            {
                delta.admitted_by = Some(admission.label.to_string());
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
    // admitted relocation ALREADY HOLDS AT THE BASE (`admission_consumed_at_base`, a positive
    // check against the base index), while an author-error row is provable against neither side.
    // Only the proven arm is typed ConsumedByMerge; everything else unused still refuses as an
    // UnmatchedAdmission. The consumed arm does not widen: it is unreachable by fallthrough.
    //
    // RETIRED BY: admissions bound to the delta content they admit, adjudicated per run and never
    // resident on main — the capability that makes a stale-able row unwritable. Until that
    // carrier exists, consumed rows persist as typed receipts and their deletion is enforced on
    // the roster file's own next touch.
    let mut stale_admissions = Vec::new();
    let mut consumed_admissions = Vec::new();
    for (i, a) in admissions.iter().enumerate() {
        if used.contains(&i) {
            continue;
        }
        if admission_consumed_at_base(a, base, &base_membership) {
            consumed_admissions.push(format!(
                "{} ({} {}) already satisfied at the base — consumed by its own merge; deletion \
                 is owed on the roster's next touch",
                a.label,
                disposition_label(a.disposition),
                admission_subject_render(&a.subject)
            ));
        } else {
            stale_admissions.push(format!(
                "{} ({} {}) matches no delta in this run",
                a.label,
                disposition_label(a.disposition),
                admission_subject_render(&a.subject)
            ));
        }
    }

    WaveAdmissionReport {
        population,
        deltas,
        stale_admissions,
        consumed_admissions,
    }
}

/// The POSITIVE consumption proof: the base itself already satisfies the admitted relocation.
///
/// Binding rows: the base binds (module, in_declaration, spelling) to EXACTLY the admitted
/// target — a singleton set equal to it, not merely containing it. Membership rows: the base
/// module's direct membership already carries the target. Anything less provable is not
/// consumption; the caller refuses it as an UnmatchedAdmission.
fn admission_consumed_at_base(
    a: &TransitionAdmission,
    base: &DeclarationIndex,
    base_membership: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    match &a.subject {
        AdmissionSubject::Membership { module, target } => base_membership
            .get(*module)
            .is_some_and(|members| members.contains(*target)),
        AdmissionSubject::Binding {
            module,
            in_declaration,
            spelling,
            target,
        } => {
            let Some(record) = index_get(base, module) else {
                return false;
            };
            let rows = binding_rows(base, record);
            rows.get(&((*in_declaration).to_string(), (*spelling).to_string()))
                .is_some_and(|set| set.len() == 1 && set.contains(*target))
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
/// consults authorship (`membership_supported`), which admitted the membership edge of the very
/// change this arm refused. So `authored_here` is passed in, not re-derived: see
/// `locally_authored_claim_added`.
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
    /// reports it under its own name.
    NoSubject { head: String },
    /// The baseline could not be observed. Refuses.
    NotEvaluated { reason: String },
    Adjudicated {
        base: String,
        head: String,
        report: WaveAdmissionReport,
        /// Whether this run's diff touches the admission roster's own source file. Consumed
        /// rows are inert receipts for every other run; on this path their deletion is DUE, and
        /// the executor refuses until the touching change removes them. This is what moves the
        /// cleanup bill from bystanders to the roster: the next relocation PR by construction
        /// touches this file and therefore cannot land while consumed rows stand.
        roster_touched: bool,
    },
}

/// The roster's own source path, as the diff names it — the subject of the consumed-row
/// deletion obligation.
pub const ADMISSION_ROSTER_REL_PATH: &str = "src/v1/stage0/src/namespace_wave_admission.rs";

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
/// AN UNMATCHED ADMISSION REFUSES. A row provable against neither side is a permission standing
/// over nothing — author error, and leaving it means the roster stops being a fact about the
/// corpus.
///
/// A CONSUMED ADMISSION REFUSES ONLY THE ROSTER'S OWN PATH. Its relocation already holds at the
/// base (a positive proof), so for an unrelated run it is an inert typed receipt; billing its
/// cleanup to that run was the externalized degradation eight dissolution PRs paid for. The
/// deletion is due — and enforced — on the first change that touches the roster file itself,
/// which every future relocation PR does by construction.
pub fn wave_admission_refusal(outcome: &WaveAdmissionOutcome) -> Option<String> {
    match outcome {
        WaveAdmissionOutcome::NoSubject { head: _ } => None,
        WaveAdmissionOutcome::NotEvaluated { reason: _ } => {
            Some("namespace-wave-admission (NotEvaluated)".to_string())
        }
        WaveAdmissionOutcome::Adjudicated {
            base: _,
            head: _,
            report,
            roster_touched,
        } => {
            let unadjudicated = report_unadjudicated(report);
            let consumed_due = *roster_touched && !report.consumed_admissions.is_empty();
            if unadjudicated.is_empty() && report.stale_admissions.is_empty() && !consumed_due {
                return None;
            }
            Some(format!(
                "namespace-wave-admission ({} unadjudicated delta(s), {} stale admission(s), {} \
                 consumed admission(s){})",
                unadjudicated.len(),
                report.stale_admissions.len(),
                report.consumed_admissions.len(),
                if consumed_due {
                    " due for deletion on this roster-touching change"
                } else {
                    ""
                }
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
/// `in_sweep_scope`, and `roster_touched` wants a `.rs` path that predicate can never admit.
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
pub fn base_records(rel: &str, content: &str) -> Result<Vec<ModuleDeclarationRecord>, String> {
    let fill = crate::v1_compiler_compile::parse_census_fill_sources(std::rc::Rc::new(
        vec![std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
            path: rel.to_string(),
            content: content.to_string(),
        })]
        .into(),
    ));
    if !fill.diagnostics.is_empty() {
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
    if base == head {
        return Ok(WaveAdmissionOutcome::NoSubject { head });
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
    // the unfiltered list, because its subject is a `.rs` path this predicate cannot admit.
    let head_parsed: Vec<&String> = head_touched.iter().filter(|p| in_sweep_scope(p)).collect();
    let base_parsed: Vec<&String> = base_side.iter().filter(|p| in_sweep_scope(p)).collect();

    let mut base_index = DeclarationIndex::default();
    for record in index_records(head_index) {
        if !head_parsed.iter().any(|c| *c == &record.rel_path) {
            crate::cli_run::declaration_index::index_insert(&mut base_index, record.clone());
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
    let base_paths: std::collections::BTreeSet<String> =
        base_paths.lines().map(|l| l.trim().to_string()).collect();
    for rel in &base_parsed {
        if !base_paths.contains(*rel) {
            // Genuinely added by this change: no base side to read, established by the listing.
            continue;
        }
        // The listing says the base carries this path, so a read failure here is UNOBSERVABLE
        // BASELINE, not news about the file.
        let content = git_stdout(&workspace, &["show", &format!("{base}:{rel}")]).map_err(|e| {
            format!(
                "cannot read {rel} at the base revision {base} ({e}), so the baseline is \
                 partially unobservable and no verdict is available"
            )
        });
        let content = match content {
            Ok(c) => c,
            Err(reason) => return Ok(WaveAdmissionOutcome::NotEvaluated { reason }),
        };
        match base_records(rel, &content) {
            Ok(records) => {
                for record in records {
                    crate::cli_run::declaration_index::index_insert(&mut base_index, record);
                }
            }
            Err(reason) => return Ok(WaveAdmissionOutcome::NotEvaluated { reason }),
        }
    }

    // READ FROM THE UNFILTERED HEAD SIDE. This is the whole subject of the repair: the roster is a
    // `.rs` file, so while `diff_sides` narrowed its answer to the parser's `.dag` question this
    // predicate was false on every production run and the consumed-row deletion obligation it
    // gates could never come due.
    let roster_touched = head_touched.iter().any(|p| p == ADMISSION_ROSTER_REL_PATH);
    let report = adjudicate(&base_index, head_index, NAMESPACE_TRANSITION_ADMISSIONS);
    Ok(WaveAdmissionOutcome::Adjudicated {
        base,
        head,
        report,
        roster_touched,
    })
}
