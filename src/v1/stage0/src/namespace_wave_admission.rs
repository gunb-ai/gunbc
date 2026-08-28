//! THE WAVE-ADMISSION WALL: what a namespace change does to closure, subject membership
//! and binding, adjudicated before it merges.
//!
//! WHY IT EXISTS. `gunbc.plans.import_namespace_program` §9 records, in its own words, that
//! "no CI mechanism enforces any of this — no ratchet, no phase, no gate over the import
//! population", and the 2026-08-26 operator ruling carried at
//! `gunbc.compiler_frontend_program_interlock` converts that disclosure into a BLOCKER: no
//! change that can alter which modules enter a subject, or what an occurrence denotes, may
//! merge before this wall exists. `milestone_prerequisites` gates
//! `NamespaceFirstSemanticWave` on `NamespaceWaveAdmissionEnrolled` by name. The plan's own
//! sentence is the acceptance condition: a plan that reads as governed when it is not is
//! worse than one that reads as unguarded.
//!
//! THE ADMISSION PREDICATE, AND THE ONE WORD IT TURNS ON. The wall admits when the
//! UNADJUDICATED delta is empty, never when the delta is empty. Expected cut motion may
//! occur; unevaluated or unexplained motion may not. A wall demanding zero delta would
//! refuse the cut it exists to govern, and would then be repaired by weakening it.
//!
//! WHY THE CHANGE CLASS IS DERIVED AND NEVER DECLARED. The ruling's
//! `NamespaceChangeClass` splits preparatory work from work that alters membership or
//! binding. This wall does not ask an author which one they wrote: it computes the delta and
//! lets the answer fall out, so `PreparatoryNoSemanticMotion` is a MEASURED property of a
//! diff rather than a claim in a PR body. Construction over validation (DESIGN §5).
//!
//! ── THE GRAIN, AND WHY IT IS NOT OCCURRENCE GRAIN ──
//!
//! An occurrence-grain delta between two arbitrary trees IS NOT COMPUTABLE, and that is a
//! closed result rather than an unsolved problem. `v2.workflow.legacy_binding_delta` states
//! it: `std.occurrence_identity`'s scope law forbids filename, span, authored name,
//! structural equality and content hash as identity inputs, and an `OccurrenceId` is a
//! monotone counter consumed in walk order, so it encodes POSITION and shifts under any edit
//! above it. A cross-compile correspondence is therefore something a TRANSFORMATION EMITS,
//! not something two finished trees can be joined on — and between a merge base and a pull
//! request head there is no transformation to emit one. So this wall reads the grain at
//! which a cross-tree key legitimately exists, and it is the same grain
//! `legacy_binding_observation` `legacy_subject_identity` folds for its own subjects:
//! authored containment identity — module path, enclosing declaration, and the LEAF SEGMENT of
//! the reference. The leaf rather than the whole spelling, because the segments before it name
//! the ROUTE and the leaf names the DECLARATION: keyed on the spelling, qualifying a reference
//! would read as one name losing its declaration and another appearing, and requalification is
//! the namespace program's own core motion. See `binding_rows`.
//!
//! WHAT THAT COSTS, NAMED RATHER THAN LEFT TO BE FOUND. Two occurrences of one spelling
//! inside one declaration — a `let` binder shadowing an imported name, a match-arm binder —
//! share a row. The repair is NOT to pick a winner between them, which is exactly the silent
//! selection the namespace authority exists to delete: a row's value is the SET of declaring
//! identities the spelling admits, and a delta is a set difference. Shadowing is therefore
//! REPRESENTED (the set has two members) rather than collapsed. What remains beyond the
//! ceiling is which occurrence took which member, and that is the next rung: it arrives with
//! a projector-emitted correspondence (E.1, `ProjectionProvenanceEntry`), not with a finer
//! key invented here.
//!
//! ── WHY THE REFERENCE CHANNEL IS NOT THE IMPORT CHANNEL ──
//!
//! A wall that read bindings only through import members would observe the import-name
//! universe being deleted and then see nothing at all — blind on precisely the change it
//! gates. So the binding channel is every authored NAME OCCURRENCE in a module's own parsed
//! tree (`ModuleDeclarationRecord::referenced`), resolved independently. It survives the cut
//! because it never depended on the construct being cut.
//!
//! ── WHAT THIS DOES WITH CLOSURE, AND THE ARM IT DELIBERATELY DOES NOT AUTHOR ──
//!
//! Closure is a pure function of membership, so "the closure moved and no membership moved"
//! is not a state any fixture can author. An arm for it would be permanently green by
//! construction — the decoration DESIGN §4b calls worse than absent. Closure is therefore
//! MEASURED and ATTRIBUTED rather than separately adjudicated: every closure row is grouped
//! under the membership delta that generates it, so a refusal names its blast radius. The
//! adjudication happens at the generators, which is where the decision is; adjudicating the
//! consequence too would be a second representation of one fact (DESIGN §2/§3).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::cli_run::declaration_index::{
    import_surface_has, index_get, index_records, DeclarationIndex, ModuleDeclarationRecord,
};
use crate::v1_std_core::qualified_last_segment;

/// The nine dispositions of `gunbc.compiler_frontend_program_interlock`
/// `NamespaceDeltaDisposition`, realized for the host reader.
///
/// THE VOCABULARY IS THE CARRIER'S, NOT THIS FILE'S. The `.dag` coproduct is the authority
/// and this is one realization of it; the partition into auto-admitted and refusing is the
/// operator's, recorded on that carrier, and is transcribed here only as the match below —
/// which is exhaustive, so a variant added there and not here fails to compile rather than
/// silently falling into a default arm.
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

/// FREE FUNCTIONS RATHER THAN INHERENT METHODS, THROUGHOUT THIS MODULE, and it is not a style
/// choice. `std.decl_ref` offers `WholeDeclaration` or `NamedField` and neither names a method on
/// an `impl` block, so every method here would be an UNCITABLE seed-growth item — a declaration
/// the obligation roster in `gunbc.namespace_wave_admission` structurally cannot enumerate.
/// `gunbc.declaration_index_seed_growth` records the same decision and the same reason.
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
/// THE MATCH IS EXHAUSTIVE, which protects THIS FILE's consistency and nothing else: a variant
/// added to the `.dag` authority and not here compiles perfectly. `vocabulary_findings` is the
/// join that closes that.
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
/// WHY THIS EXISTS AT ALL. The enum above is a SECOND REPRESENTATION of a coproduct the
/// `.dag` carrier owns, and DESIGN §3 is unambiguous that two representations of one fact
/// diverge on the first amendment. The exhaustive `match` in `auto_admitted` protects this
/// file's INTERNAL consistency and says nothing about the carrier: a variant added there and
/// not here compiles perfectly, and the wall then adjudicates against a vocabulary the
/// operator has already superseded — silently, because nothing joins the two.
///
/// IT IS A JOIN AND NOT A COUNT. The check is set equality over variant names, in both
/// directions, so `here and not there` and `there and not here` are separate, named findings.
/// The index already carries the authority's variants from the authority's own parse, so this
/// costs one keyed lookup and no walk.
///
/// AND ITS ABSENCE REFUSES. If the authority module is not in the index at all — renamed,
/// deleted, or moved out of the swept roots — that is not permission to proceed on the host's
/// own say-so; it is the state in which nothing is checking the vocabulary, which is exactly
/// what this function exists to make impossible.
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
    /// The two sides, rendered. Never a summary: a refusal a reader cannot act on is a
    /// stopped line with the analysis withheld.
    pub detail: String,
    /// Modules whose transitive closure moves because of THIS delta. Measured, never
    /// separately adjudicated — see the module header.
    pub closure_blast_radius: usize,
    /// Set when a transition admission covers this exact subject and disposition.
    pub admitted_by: Option<String>,
}

/// The authored pattern naming one exact runtime delta subject.
///
/// Its borrowed fields keep the admission roster const: no initializer can compute permission
/// from the observed delta set, a file, or process state. Runtime observations remain owned
/// `DeltaSubject` values; an authored pattern and an observation are deliberately distinct types.
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
        } => format!("binding {module}::{in_declaration} `{spelling}`"),
    }
}

/// An operator-authored admission for one exact subject under one exact disposition.
///
/// THE GRAIN IS EXACT ON PURPOSE, AND THE COARSE FORM IS NOT BUILT HERE. The first semantic
/// wave is expected to produce THOUSANDS of transitions (measured by the owning session
/// against the import-strip receipts' class taxonomy — a population figure that is stale as
/// a count and sound as an order of magnitude), so a wave will want a class admission
/// bounded BY ENUMERATED IDENTITY — "these exact bindings, taken from the pre-deletion
/// baseline observation, become unresolved" — never by a predicate such as "unresolvedness
/// is expected during the wave", which admits everything and zeroes the wall's deficit
/// frequency by construction (DESIGN §5, the absorbing fallback). That coarse carrier is NOT
/// authored here because it would be a mechanism with no consumer until the first wave needs
/// it (DESIGN §6, experimental residue); what is fixed now is that its population must be an
/// enumeration and never a predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionAdmission {
    pub label: &'static str,
    pub subject: AdmissionSubject,
    pub disposition: NamespaceDeltaDisposition,
}

/// CONST-NESS IS SAFETY, NOT STORAGE STYLE. A const roster cannot be computed from observed
/// deltas, a file, environment state, or any other runtime input: its permission set is exactly
/// what an author wrote and a reviewer read. `AdmissionSubject` therefore carries `&'static str`
/// patterns distinct from runtime-owned `DeltaSubject` observations. The prior `String` subject
/// admitted only the all-empty shape in a const; it refused loudly as stale, but no const row
/// could name a real module.
///
/// EMPTY IS THE RESTING STATE, and it is where this roster belongs between transitions.
///
/// It carried 53 exact admissions for the owner-qualified call-target cut, each measured by the
/// required floor against the merge base after the namespace wall landed. That subject has
/// landed (#9436, #9504); #9400 itself closed unmerged and no successor is open. Every one of
/// the 53 therefore matched no delta, which is exactly the finding this roster's own rule
/// predicts -- "a row that no longer matches is itself a finding (`stale_admissions`), so this
/// temporary transition roster must shrink with its subject."
///
/// WHY LEAVING THEM WAS NOT A QUIET COST. `stale_admissions` is computed per RUN: a row is
/// stale unless some delta IN THAT RUN matches it. A pull_request build adjudicates the MERGE
/// commit, so once the rows were on main every open PR inherited all 53 -- and a PR that
/// touches no namespace at all is exactly the case that can never match them. The phase
/// therefore refused every unrelated change in the repository, which is why this shrink is the
/// fix rather than housekeeping.
///
/// EMPTY DOES NOT MEAN PERMISSIVE, which is the reason shrinking is safe. With no rows, a run
/// carrying no delta reports nothing and passes; a run carrying a real delta reports it as
/// UNADJUDICATED and refuses. The failure mode of having shrunk too early is therefore a loud
/// refusal naming the delta, which its author closes by authoring a row -- never a silent
/// admission. The next transition adds its rows here and removes them when its subject lands.
/// SECOND SHRINK, SAME RULE. Two `gunbc.ci_render` `plain_span` rows dissolved on
/// schedule: `ci_render` now imports `plain_span` from the `std.render` authority and declares
/// none, so no run can produce the deltas those rows named and both were reported stale on
/// every build. They are removed here by the trigger they were authored with, not by a
/// reinterpretation of it.
pub const NAMESPACE_TRANSITION_ADMISSIONS: &[TransitionAdmission] = &[
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.openbmc::bmc_firmware_factory_login BmcFirmwareFamily",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.openbmc", in_declaration: "bmc_firmware_factory_login", spelling: "BmcFirmwareFamily" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.openbmc::bmc_firmware_host_storage_visibility BmcFirmwareFamily",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.openbmc", in_declaration: "bmc_firmware_host_storage_visibility", spelling: "BmcFirmwareFamily" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.openbmc::bmc_firmware_supported_protocols BmcFirmwareFamily",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.openbmc", in_declaration: "bmc_firmware_supported_protocols", spelling: "BmcFirmwareFamily" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.openbmc::openbmc_supported_protocols BmcProtocol",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.openbmc", in_declaration: "openbmc_supported_protocols", spelling: "BmcProtocol" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.webui.nbd_proxy::bmcweb_nbd_proxy_path_params PathParamBinding",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.webui.nbd_proxy", in_declaration: "bmcweb_nbd_proxy_path_params", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.webui.nbd_proxy::bmcweb_nbd_proxy_ws_path render_path_template",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.webui.nbd_proxy", in_declaration: "bmcweb_nbd_proxy_ws_path", spelling: "render_path_template" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.webui.nbd_proxy::nbd_slot_endpoint_path_template ParamToken",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.webui.nbd_proxy", in_declaration: "nbd_slot_endpoint_path_template", spelling: "ParamToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.webui.nbd_proxy::nbd_slot_endpoint_path_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.webui.nbd_proxy", in_declaration: "nbd_slot_endpoint_path_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.webui.nbd_proxy::vm_slot_index_endpoint_path_template ParamToken",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.webui.nbd_proxy", in_declaration: "vm_slot_index_endpoint_path_template", spelling: "ParamToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.bmc.webui.nbd_proxy::vm_slot_index_endpoint_path_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "extdeps.bmc.webui.nbd_proxy", in_declaration: "vm_slot_index_endpoint_path_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.boot.emit::emit_pixel_writer_payload EntryPoint",
        subject: AdmissionSubject::Binding { module: "extdeps.boot.emit", in_declaration: "emit_pixel_writer_payload", spelling: "EntryPoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.halsey_165::halsey_165_cross_connect Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.halsey_165", in_declaration: "halsey_165_cross_connect", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.halsey_165::halsey_165_remote_hands Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.halsey_165", in_declaration: "halsey_165_remote_hands", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.interserver::interserver_half_rack_plan Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.interserver", in_declaration: "interserver_half_rack_plan", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.interserver::interserver_quarter_rack_plan Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.interserver", in_declaration: "interserver_quarter_rack_plan", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.interserver::interserver_single_server_1u_plan Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.interserver", in_declaration: "interserver_single_server_1u_plan", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.natcoweb::natcoweb_plan_10u Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.natcoweb", in_declaration: "natcoweb_plan_10u", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.natcoweb::natcoweb_plan_1u Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.natcoweb", in_declaration: "natcoweb_plan_1u", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.natcoweb::natcoweb_plan_20u Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.natcoweb", in_declaration: "natcoweb_plan_20u", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.natcoweb::natcoweb_plan_full_cabinet Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.natcoweb", in_declaration: "natcoweb_plan_full_cabinet", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.colo.natcoweb::natcoweb_remote_hands Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.colo.natcoweb", in_declaration: "natcoweb_remote_hands", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.energy.nj_electricity::nj_commercial_rate Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.energy.nj_electricity", in_declaration: "nj_commercial_rate", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.energy.nj_electricity::nj_industrial_rate Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.energy.nj_electricity", in_declaration: "nj_industrial_rate", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.energy.nj_electricity::nj_residential_rate Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.energy.nj_electricity", in_declaration: "nj_residential_rate", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.fans.dynatron::dynatron_df126025bu_pwmh_published_points tenths_dba",
        subject: AdmissionSubject::Binding { module: "extdeps.fans.dynatron", in_declaration: "dynatron_df126025bu_pwmh_published_points", spelling: "tenths_dba" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.fans.noctua::noctua_nf_a9_hs_pwm_published_points tenths_dba",
        subject: AdmissionSubject::Binding { module: "extdeps.fans.noctua", in_declaration: "noctua_nf_a9_hs_pwm_published_points", spelling: "tenths_dba" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.formats.elf.primitives::linux_x86_64_elf_format ExecutableFormat",
        subject: AdmissionSubject::Binding { module: "extdeps.formats.elf.primitives", in_declaration: "linux_x86_64_elf_format", spelling: "ExecutableFormat" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.formats.elf.segments::decode_segment_permissions SegmentPermission",
        subject: AdmissionSubject::Binding { module: "extdeps.formats.elf.segments", in_declaration: "decode_segment_permissions", spelling: "SegmentPermission" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.formats.elf.segments::elf64_phdr_to_load_segment LoadSegment",
        subject: AdmissionSubject::Binding { module: "extdeps.formats.elf.segments", in_declaration: "elf64_phdr_to_load_segment", spelling: "LoadSegment" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.formats.elf.types::elf64_ehdr_entry_point EntryPoint",
        subject: AdmissionSubject::Binding { module: "extdeps.formats.elf.types", in_declaration: "elf64_ehdr_entry_point", spelling: "EntryPoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.aws_ec2::aws_ec2_m7a_2xlarge_us_east_1 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.aws_ec2", in_declaration: "aws_ec2_m7a_2xlarge_us_east_1", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.aws_ec2::aws_ec2_m7a_4xlarge_us_east_1 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.aws_ec2", in_declaration: "aws_ec2_m7a_4xlarge_us_east_1", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.aws_ec2::aws_ec2_m7a_large_us_east_1 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.aws_ec2", in_declaration: "aws_ec2_m7a_large_us_east_1", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.aws_ec2::aws_ec2_m7a_medium_us_east_1 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.aws_ec2", in_declaration: "aws_ec2_m7a_medium_us_east_1", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.aws_ec2::aws_ec2_m7a_xlarge_us_east_1 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.aws_ec2", in_declaration: "aws_ec2_m7a_xlarge_us_east_1", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.github_actions::github_actions_linux_16core_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.github_actions", in_declaration: "github_actions_linux_16core_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.github_actions::github_actions_linux_2core_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.github_actions", in_declaration: "github_actions_linux_2core_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.github_actions::github_actions_linux_4core_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.github_actions", in_declaration: "github_actions_linux_4core_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.github_actions::github_actions_linux_8core_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.github_actions", in_declaration: "github_actions_linux_8core_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.github_actions::github_actions_linux_arm_2core_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.github_actions", in_declaration: "github_actions_linux_arm_2core_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.github_actions::github_actions_linux_slim_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.github_actions", in_declaration: "github_actions_linux_slim_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.gitlab_subscription::gitlab_compute_minute_overage_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.gitlab_subscription", in_declaration: "gitlab_compute_minute_overage_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.gitlab_subscription::gitlab_free_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.gitlab_subscription", in_declaration: "gitlab_free_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.gitlab_subscription::gitlab_premium_2023_price_increase Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.gitlab_subscription", in_declaration: "gitlab_premium_2023_price_increase", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.gitlab_subscription::gitlab_premium_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.gitlab_subscription", in_declaration: "gitlab_premium_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.gitlab_subscription::gitlab_ultimate_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.gitlab_subscription", in_declaration: "gitlab_ultimate_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.hetzner_dedicated::hetzner_ax102_1_ltd_price Eur",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.hetzner_dedicated", in_declaration: "hetzner_ax102_1_ltd_price", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.hetzner_dedicated::hetzner_ax41_1_ltd_price Eur",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.hetzner_dedicated", in_declaration: "hetzner_ax41_1_ltd_price", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.hetzner_dedicated::hetzner_ax42_1_ltd_price Eur",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.hetzner_dedicated", in_declaration: "hetzner_ax42_1_ltd_price", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.hetzner_dedicated::hetzner_ax42_1_price Eur",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.hetzner_dedicated", in_declaration: "hetzner_ax42_1_price", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_egress_first_10tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_egress_first_10tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_egress_next_100tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_egress_next_100tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_egress_next_40tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_egress_next_40tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_egress_over_150tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_egress_over_150tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_gzrs_tier0 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_gzrs_tier0", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_gzrs_tier_500tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_gzrs_tier_500tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_gzrs_tier_50tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_gzrs_tier_50tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_gzrs_write_operations Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_gzrs_write_operations", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_lrs_tier0 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_lrs_tier0", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_lrs_tier_500tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_lrs_tier_500tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_lrs_tier_50tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_lrs_tier_50tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_lrs_write_operations Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_lrs_write_operations", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_ra_gzrs_tier0 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_ra_gzrs_tier0", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_read_operations Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_read_operations", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_zrs_tier0 Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_zrs_tier0", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_zrs_tier_500tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_zrs_tier_500tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::azure_hot_zrs_tier_50tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "azure_hot_zrs_tier_50tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::b2_egress_beyond_free_allowance Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "b2_egress_beyond_free_allowance", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::gcs_default_replication Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "gcs_default_replication", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::gcs_egress_worldwide_first_10tib Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "gcs_egress_worldwide_first_10tib", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::gcs_standard_regional Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "gcs_standard_regional", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::r2_class_a_operations Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "r2_class_a_operations", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::r2_class_b_operations Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "r2_class_b_operations", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::r2_infrequent_access Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "r2_infrequent_access", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::r2_standard Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "r2_standard", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_egress_first_10tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_egress_first_10tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_egress_next_100tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_egress_next_100tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_egress_next_40tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_egress_next_40tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_egress_over_150tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_egress_over_150tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_standard_first_50tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_standard_first_50tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_standard_infrequent_access Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_standard_infrequent_access", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_standard_next_450tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_standard_next_450tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_standard_over_500tb Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_standard_over_500tb", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_tier1_requests Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_tier1_requests", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::s3_tier2_requests Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "s3_tier2_requests", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::storj_egress Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "storj_egress", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::storj_standard_storage Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "storj_standard_storage", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.object_storage::wasabi_hot_storage Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.object_storage", in_declaration: "wasabi_hot_storage", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.regional_compute::usd_per_hour Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.regional_compute", in_declaration: "usd_per_hour", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.ubicloud::ubicloud_arm_standard_16_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.ubicloud", in_declaration: "ubicloud_arm_standard_16_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.ubicloud::ubicloud_arm_standard_2_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.ubicloud", in_declaration: "ubicloud_arm_standard_2_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.ubicloud::ubicloud_premium_standard_16_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.ubicloud", in_declaration: "ubicloud_premium_standard_16_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.pricing.ubicloud::ubicloud_premium_standard_2_price Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.pricing.ubicloud", in_declaration: "ubicloud_premium_standard_2_price", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.realestate.nj_industrial::belleville_specimen Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.realestate.nj_industrial", in_declaration: "belleville_specimen", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.realestate.nj_industrial::nj_industrial_market_snapshot Usd",
        subject: AdmissionSubject::Binding { module: "extdeps.realestate.nj_industrial", in_declaration: "nj_industrial_market_snapshot", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aapcs64_linux_syscall_convention CallingConvention",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aapcs64_linux_syscall_convention", spelling: "CallingConvention" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aapcs64_wrong_syscall_number_convention CallingConvention",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aapcs64_wrong_syscall_number_convention", spelling: "CallingConvention" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aarch64_inhabitant_x0 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aarch64_inhabitant_x0", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aarch64_inhabitant_x1 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aarch64_inhabitant_x1", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aarch64_inhabitant_x2 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aarch64_inhabitant_x2", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aarch64_inhabitant_x3 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aarch64_inhabitant_x3", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aarch64_inhabitant_x4 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aarch64_inhabitant_x4", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aarch64_inhabitant_x5 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aarch64_inhabitant_x5", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.aapcs64::aarch64_inhabitant_x8 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.aapcs64", in_declaration: "aarch64_inhabitant_x8", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_r10 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_r10", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_r8 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_r8", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_r9 ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_r9", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_rax ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_rax", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_rcx ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_rcx", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_rdi ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_rdi", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_rdx ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_rdx", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::amd64_inhabitant_rsi ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "amd64_inhabitant_rsi", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::sysv_amd64_linux_syscall_convention CallingConvention",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "sysv_amd64_linux_syscall_convention", spelling: "CallingConvention" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.abi.sysv_amd64::sysv_amd64_wrong_arg3_convention CallingConvention",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.abi.sysv_amd64", in_declaration: "sysv_amd64_wrong_arg3_convention", spelling: "CallingConvention" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.architecture.aarch64_linux::aarch64_general_purpose_registers ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.architecture.aarch64_linux", in_declaration: "aarch64_general_purpose_registers", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.architecture.aarch64_linux::aarch64_stack_grows_down StackLayout",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.architecture.aarch64_linux", in_declaration: "aarch64_stack_grows_down", spelling: "StackLayout" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.architecture.x86_64_linux::x86_64_general_purpose_registers ArchitecturalRegister",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.architecture.x86_64_linux", in_declaration: "x86_64_general_purpose_registers", spelling: "ArchitecturalRegister" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.runtime.architecture.x86_64_linux::x86_64_stack_grows_down StackLayout",
        subject: AdmissionSubject::Binding { module: "extdeps.runtime.architecture.x86_64_linux", in_declaration: "x86_64_stack_grows_down", spelling: "StackLayout" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::match_path_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "match_path_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::match_path_tokens LiteralToken",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "match_path_tokens", spelling: "LiteralToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::match_path_tokens ParamToken",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "match_path_tokens", spelling: "ParamToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::match_path_tokens PathParamBinding",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "match_path_tokens", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::match_path_tokens UrlPathToken",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "match_path_tokens", spelling: "UrlPathToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::parse_path_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "parse_path_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::parse_segment_tokens LiteralToken",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "parse_segment_tokens", spelling: "LiteralToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: extdeps.uri_path::parse_segment_tokens ParamToken",
        subject: AdmissionSubject::Binding { module: "extdeps.uri_path", in_declaration: "parse_segment_tokens", spelling: "ParamToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.bmc_fan_converge::bmc_fan_parse_os_release_identity OpenBmc",
        subject: AdmissionSubject::Binding { module: "gunbc.bmc_fan_converge", in_declaration: "bmc_fan_parse_os_release_identity", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.bmc_fan_converge::bmc_firmware_family_content_hash BmcFirmwareFamily",
        subject: AdmissionSubject::Binding { module: "gunbc.bmc_fan_converge", in_declaration: "bmc_firmware_family_content_hash", spelling: "BmcFirmwareFamily" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.bmc_fan_converge::bmc_firmware_family_content_hash OpenBmc",
        subject: AdmissionSubject::Binding { module: "gunbc.bmc_fan_converge", in_declaration: "bmc_firmware_family_content_hash", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.bmc_onboarding::altra_onboarding_plan BmcEndpoint",
        subject: AdmissionSubject::Binding { module: "gunbc.bmc_onboarding", in_declaration: "altra_onboarding_plan", spelling: "BmcEndpoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.bmc_onboarding::altra_onboarding_plan Redfish",
        subject: AdmissionSubject::Binding { module: "gunbc.bmc_onboarding", in_declaration: "altra_onboarding_plan", spelling: "Redfish" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.ci_fleet::per_second_billed_cost_quote CurrencyCode",
        subject: AdmissionSubject::Binding { module: "gunbc.ci_fleet", in_declaration: "per_second_billed_cost_quote", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.econ.knowable_wedge::convert_money_per_minute_eur_to_usd Eur",
        subject: AdmissionSubject::Binding { module: "gunbc.econ.knowable_wedge", in_declaration: "convert_money_per_minute_eur_to_usd", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.econ.knowable_wedge::convert_money_per_minute_eur_to_usd Usd",
        subject: AdmissionSubject::Binding { module: "gunbc.econ.knowable_wedge", in_declaration: "convert_money_per_minute_eur_to_usd", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.econ.knowable_wedge::derive_github_hetzner_ci_minute_wedge Eur",
        subject: AdmissionSubject::Binding { module: "gunbc.econ.knowable_wedge", in_declaration: "derive_github_hetzner_ci_minute_wedge", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.econ.knowable_wedge::derive_github_hetzner_ci_minute_wedge Usd",
        subject: AdmissionSubject::Binding { module: "gunbc.econ.knowable_wedge", in_declaration: "derive_github_hetzner_ci_minute_wedge", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.econ.knowable_wedge::derive_monthly_eur_with_amortization Eur",
        subject: AdmissionSubject::Binding { module: "gunbc.econ.knowable_wedge", in_declaration: "derive_monthly_eur_with_amortization", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.econ.knowable_wedge::derive_runner_cost_floor_per_minute_usd Eur",
        subject: AdmissionSubject::Binding { module: "gunbc.econ.knowable_wedge", in_declaration: "derive_runner_cost_floor_per_minute_usd", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.econ.knowable_wedge::subtract_money_per_minute_usd Usd",
        subject: AdmissionSubject::Binding { module: "gunbc.econ.knowable_wedge", in_declaration: "subtract_money_per_minute_usd", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fabric_floor_dispatch::floor_supply_selection Usd",
        subject: AdmissionSubject::Binding { module: "gunbc.fabric_floor_dispatch", in_declaration: "floor_supply_selection", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fabric_witness_run::floor_buy_terms Usd",
        subject: AdmissionSubject::Binding { module: "gunbc.fabric_witness_run", in_declaration: "floor_buy_terms", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fabric_witness_run::floor_offer_affordability CurrencyCode",
        subject: AdmissionSubject::Binding { module: "gunbc.fabric_witness_run", in_declaration: "floor_offer_affordability", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_bmc_firmware_evidence::srv1_bmc_firmware_attestation OpenBmc",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_bmc_firmware_evidence", in_declaration: "srv1_bmc_firmware_attestation", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_bmc_firmware_evidence::srv2_bmc_firmware_attestation OpenBmc",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_bmc_firmware_evidence", in_declaration: "srv2_bmc_firmware_attestation", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_bmc_observation::srv3_bmc_firmware_2_07_00_observation OpenBmc",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_bmc_observation", in_declaration: "srv3_bmc_firmware_2_07_00_observation", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_bmc_observation::srv3_bmc_firmware_3_22_00_observation OpenBmc",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_bmc_observation", in_declaration: "srv3_bmc_firmware_3_22_00_observation", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_bmc_observation::srv4_bmc_firmware_observation OpenBmc",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_bmc_observation", in_declaration: "srv4_bmc_firmware_observation", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_fan_acoustic_evidence::fan_acoustic_evidence_witness dba_tenths",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_fan_acoustic_evidence", in_declaration: "fan_acoustic_evidence_witness", spelling: "dba_tenths" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv1_baseboard BmcEndpoint",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv1_baseboard", spelling: "BmcEndpoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv1_baseboard Redfish",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv1_baseboard", spelling: "Redfish" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv2_baseboard BmcEndpoint",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv2_baseboard", spelling: "BmcEndpoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv2_baseboard Redfish",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv2_baseboard", spelling: "Redfish" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv3_baseboard BmcEndpoint",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv3_baseboard", spelling: "BmcEndpoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv3_baseboard Redfish",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv3_baseboard", spelling: "Redfish" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv4_baseboard BmcEndpoint",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv4_baseboard", spelling: "BmcEndpoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.fleet_intent::srv4_baseboard Redfish",
        subject: AdmissionSubject::Binding { module: "gunbc.fleet_intent", in_declaration: "srv4_baseboard", spelling: "Redfish" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.gunbhub_serve::gunbhub_browse_path_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "gunbc.gunbhub_serve", in_declaration: "gunbhub_browse_path_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.gunbhub_serve::gunbhub_repo_path_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "gunbc.gunbhub_serve", in_declaration: "gunbhub_repo_path_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.gunbhub_serve::path_matches_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "gunbc.gunbhub_serve", in_declaration: "path_matches_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.node_http_server_emit::emit_path_template_regex_source LiteralToken",
        subject: AdmissionSubject::Binding { module: "gunbc.node_http_server_emit", in_declaration: "emit_path_template_regex_source", spelling: "LiteralToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.node_http_server_emit::emit_path_template_regex_source ParamToken",
        subject: AdmissionSubject::Binding { module: "gunbc.node_http_server_emit", in_declaration: "emit_path_template_regex_source", spelling: "ParamToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.node_http_server_emit::emit_path_template_regex_source PathTemplate",
        subject: AdmissionSubject::Binding { module: "gunbc.node_http_server_emit", in_declaration: "emit_path_template_regex_source", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::roadmap_serve_invoke PathParamBinding",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "roadmap_serve_invoke", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::roadmap_serve_invoke_for_instance PathParamBinding",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "roadmap_serve_invoke_for_instance", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_dispatch_handler_response PathParamBinding",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_dispatch_handler_response", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_dispatch_handler_response_for_instance PathParamBinding",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_dispatch_handler_response_for_instance", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_dispatch_handler_response_for_instance path_param_value",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_dispatch_handler_response_for_instance", spelling: "path_param_value" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_publish_handler_response_for_instance PathParamBinding",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_publish_handler_response_for_instance", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_publish_handler_response_for_instance path_param_value",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_publish_handler_response_for_instance", spelling: "path_param_value" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_stop_handler_response PathParamBinding",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_stop_handler_response", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_stop_handler_response_for_instance PathParamBinding",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_stop_handler_response_for_instance", spelling: "PathParamBinding" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_serve::serve_stop_handler_response_for_instance path_param_value",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_serve", in_declaration: "serve_stop_handler_response_for_instance", spelling: "path_param_value" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_static_site::roadmap_site_path_template LiteralToken",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_static_site", in_declaration: "roadmap_site_path_template", spelling: "LiteralToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_static_site::roadmap_site_path_template PathTemplate",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_static_site", in_declaration: "roadmap_site_path_template", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_static_site::served_static_route_literal_path LiteralToken",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_static_site", in_declaration: "served_static_route_literal_path", spelling: "LiteralToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: gunbc.roadmap_static_site::served_static_route_literal_path ParamToken",
        subject: AdmissionSubject::Binding { module: "gunbc.roadmap_static_site", in_declaration: "served_static_route_literal_path", spelling: "ParamToken" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_fulfillment::cash_step CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_fulfillment", in_declaration: "cash_step", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_fulfillment::incremental_cash_now CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_fulfillment", in_declaration: "incremental_cash_now", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_selection::build_standing CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_selection", in_declaration: "build_standing", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_selection::candidate_missing_inputs CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_selection", in_declaration: "candidate_missing_inputs", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_selection::candidate_readings CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_selection", in_declaration: "candidate_readings", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_selection::cash_missing CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_selection", in_declaration: "cash_missing", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_selection::cash_reading CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_selection", in_declaration: "cash_reading", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_selection::project_entry CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_selection", in_declaration: "project_entry", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.build_selection::project_field CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.build_selection", in_declaration: "project_field", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::assemble_cost CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "assemble_cost", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::price_available_candidate CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "price_available_candidate", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::price_candidate CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "price_candidate", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::price_over_horizon CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "price_over_horizon", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::price_screened_candidate CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "price_screened_candidate", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::priced_delay CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "priced_delay", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::priced_once CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "priced_once", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::priced_operating CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "priced_operating", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::priced_opportunity CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "priced_opportunity", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::priced_start CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "priced_start", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::priced_transition CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "priced_transition", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.selection::select_supply CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.selection", in_declaration: "select_supply", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: product.fabric.supply::offer_affordability_for CurrencyCode",
        subject: AdmissionSubject::Binding { module: "product.fabric.supply", in_declaration: "offer_affordability_for", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: std.effects::derive_effect_shape PathTemplate",
        subject: AdmissionSubject::Binding { module: "std.effects", in_declaration: "derive_effect_shape", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: std.effects::derive_effect_shape last_path_param",
        subject: AdmissionSubject::Binding { module: "std.effects", in_declaration: "derive_effect_shape", spelling: "last_path_param" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: std.effects::derive_op_effect PathTemplate",
        subject: AdmissionSubject::Binding { module: "std.effects", in_declaration: "derive_op_effect", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.acquisition_mechanism_witness::indivisible_scenario Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.acquisition_mechanism_witness", in_declaration: "indivisible_scenario", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.acquisition_mechanism_witness::two_usd_per_click Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.acquisition_mechanism_witness", in_declaration: "two_usd_per_click", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.acquisition_mechanism_witness::w_red_zero_entry_unit_cost_refuses Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.acquisition_mechanism_witness", in_declaration: "w_red_zero_entry_unit_cost_refuses", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::firmware_intent_match_crosses_as_typed_disposition BmcEndpoint",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "firmware_intent_match_crosses_as_typed_disposition", spelling: "BmcEndpoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::firmware_intent_match_crosses_as_typed_disposition OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "firmware_intent_match_crosses_as_typed_disposition", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::firmware_intent_match_crosses_as_typed_disposition Redfish",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "firmware_intent_match_crosses_as_typed_disposition", spelling: "Redfish" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::firmware_wire_version_is_parsed_before_track_matching OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "firmware_wire_version_is_parsed_before_track_matching", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::nbd_proxy_vm_capable_row_solves OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "nbd_proxy_vm_capable_row_solves", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::node_with_virtual_media_solves_direct OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "node_with_virtual_media_solves_direct", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::redfish_vm_preferred_over_nbd_proxy OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "redfish_vm_preferred_over_nbd_proxy", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::update_only_row_without_vm_catalog_falls_to_pxe OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "update_only_row_without_vm_catalog_falls_to_pxe", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_capability_solve::vm_capable_firmware_row_flips_to_update_path OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_capability_solve", in_declaration: "vm_capable_firmware_row_flips_to_update_path", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_fan_converge_witness::backup_manifest_identity_is_derived_from_every_manifest_field OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_fan_converge_witness", in_declaration: "backup_manifest_identity_is_derived_from_every_manifest_field", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_firmware_evidence_install_mechanism::uncatalogued_firmware OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_firmware_evidence_install_mechanism", in_declaration: "uncatalogued_firmware", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_firmware_evidence_install_mechanism::unknown_endpoint BmcEndpoint",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_firmware_evidence_install_mechanism", in_declaration: "unknown_endpoint", spelling: "BmcEndpoint" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_firmware_evidence_install_mechanism::unknown_endpoint Redfish",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_firmware_evidence_install_mechanism", in_declaration: "unknown_endpoint", spelling: "Redfish" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_firmware_thermal_witness::an_uncatalogued_image_refuses_to_derive_a_surface OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_firmware_thermal_witness", in_declaration: "an_uncatalogued_image_refuses_to_derive_a_surface", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.bmc_firmware_thermal_witness::fixture_pinned_track3_intent OpenBmc",
        subject: AdmissionSubject::Binding { module: "test.claim.bmc_firmware_thermal_witness", in_declaration: "fixture_pinned_track3_intent", spelling: "OpenBmc" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::usd_micros Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "usd_micros", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_a_build_with_no_purchases_reads_its_own_arm_not_a_zero Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_a_build_with_no_purchases_reads_its_own_arm_not_a_zero", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_an_unfillable_line_refuses_the_cash_answer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_an_unfillable_line_refuses_the_cash_answer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_basis_across_lots_in_two_currencies_refuses Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_basis_across_lots_in_two_currencies_refuses", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_basis_across_two_lots_in_one_currency_sums Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_basis_across_two_lots_in_one_currency_sums", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_basis_is_exact_under_a_named_policy Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_basis_is_exact_under_a_named_policy", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_cash_counts_purchases_only_and_owned_draws_report_zero_cash Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_cash_counts_purchases_only_and_owned_draws_report_zero_cash", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_cross_currency_purchase_refuses_rather_than_summing Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_cross_currency_purchase_refuses_rather_than_summing", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_fulfillment_witness::w_cross_currency_purchase_refuses_rather_than_summing Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_fulfillment_witness", in_declaration: "w_cross_currency_purchase_refuses_rather_than_summing", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::usd_micros Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "usd_micros", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_a_candidate_in_an_undeclared_currency_does_not_rank Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_a_candidate_in_an_undeclared_currency_does_not_rank", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_a_candidate_in_an_undeclared_currency_does_not_rank Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_a_candidate_in_an_undeclared_currency_does_not_rank", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_a_fully_owned_build_ranks_at_zero_cash Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_a_fully_owned_build_ranks_at_zero_cash", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_a_pending_candidate_neither_wins_nor_poisons_the_field Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_a_pending_candidate_neither_wins_nor_poisons_the_field", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_a_refused_cash_answer_makes_the_candidate_incomparable Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_a_refused_cash_answer_makes_the_candidate_incomparable", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_a_strictly_worse_candidate_is_dominated Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_a_strictly_worse_candidate_is_dominated", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_a_tradeoff_is_not_a_domination Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_a_tradeoff_is_not_a_domination", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_an_unbounded_ceiling_is_declared_missing_not_read_as_zero Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_an_unbounded_ceiling_is_declared_missing_not_read_as_zero", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_an_unread_compatibility_is_pending_evidence_not_a_winner Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_an_unread_compatibility_is_pending_evidence_not_a_winner", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_duplicate_candidate_identity_refuses_the_selection Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_duplicate_candidate_identity_refuses_the_selection", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_the_better_candidate_stands_on_the_frontier Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_the_better_candidate_stands_on_the_frontier", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.build_selection_witness::w_the_projected_entry_is_keyed_by_declaration_identity Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.build_selection_witness", in_declaration: "w_the_projected_entry_is_keyed_by_declaration_identity", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.econ_wedge_witness::github_hetzner_wedge_floor_usd_per_minute Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.econ_wedge_witness", in_declaration: "github_hetzner_wedge_floor_usd_per_minute", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.econ_wedge_witness::github_hetzner_wedge_spread_usd_per_minute Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.econ_wedge_witness", in_declaration: "github_hetzner_wedge_spread_usd_per_minute", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.econ_wedge_witness::github_tiny_umbrella_fixture Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.econ_wedge_witness", in_declaration: "github_tiny_umbrella_fixture", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.econ_wedge_witness::hetzner_ax41_usd_floor_fixture Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.econ_wedge_witness", in_declaration: "hetzner_ax41_usd_floor_fixture", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.effects_witness_test::derived_op PathTemplate",
        subject: AdmissionSubject::Binding { module: "test.claim.effects_witness_test", in_declaration: "derived_op", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.effects_witness_test::effects_path_parse_witnesses has_path_params",
        subject: AdmissionSubject::Binding { module: "test.claim.effects_witness_test", in_declaration: "effects_path_parse_witnesses", spelling: "has_path_params" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.effects_witness_test::effects_path_parse_witnesses last_path_param",
        subject: AdmissionSubject::Binding { module: "test.claim.effects_witness_test", in_declaration: "effects_path_parse_witnesses", spelling: "last_path_param" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.extdeps_round2_grounding_witness::w_served_static_route_content_type_media_type PathTemplate",
        subject: AdmissionSubject::Binding { module: "test.claim.extdeps_round2_grounding_witness", in_declaration: "w_served_static_route_content_type_media_type", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_budget_encumbrance_witness::a_foreign_currency_reservation_refuses_and_never_converts Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_budget_encumbrance_witness", in_declaration: "a_foreign_currency_reservation_refuses_and_never_converts", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_budget_encumbrance_witness::reserve_on Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_budget_encumbrance_witness", in_declaration: "reserve_on", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_budget_encumbrance_witness::settle_on Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_budget_encumbrance_witness", in_declaration: "settle_on", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_budget_encumbrance_witness::the_currency_refusal_is_distinct_from_a_ceiling_refusal Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_budget_encumbrance_witness", in_declaration: "the_currency_refusal_is_distinct_from_a_ceiling_refusal", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_budget_encumbrance_witness::the_currency_refusal_is_distinct_from_a_ceiling_refusal Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_budget_encumbrance_witness", in_declaration: "the_currency_refusal_is_distinct_from_a_ceiling_refusal", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_budget_encumbrance_witness::usd_account Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_budget_encumbrance_witness", in_declaration: "usd_account", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_contention_witness_test::c_demand Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_contention_witness_test", in_declaration: "c_demand", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_contention_witness_test::continuous_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_contention_witness_test", in_declaration: "continuous_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_contention_witness_test::flat_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_contention_witness_test", in_declaration: "flat_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_contention_witness_test::hourly_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_contention_witness_test", in_declaration: "hourly_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_contention_witness_test::paid_through_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_contention_witness_test", in_declaration: "paid_through_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_contention_witness_test::per_second_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_contention_witness_test", in_declaration: "per_second_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_control_plane_witness::an_offer_quoted_in_another_currency_is_refused_at_the_budget_seam Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_control_plane_witness", in_declaration: "an_offer_quoted_in_another_currency_is_refused_at_the_budget_seam", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_control_plane_witness::an_underfunded_budget_refuses_with_a_ledger_cause Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_control_plane_witness", in_declaration: "an_underfunded_budget_refuses_with_a_ledger_cause", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_control_plane_witness::cp_account CurrencyCode",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_control_plane_witness", in_declaration: "cp_account", spelling: "CurrencyCode" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_control_plane_witness::cp_broker Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_control_plane_witness", in_declaration: "cp_broker", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_control_plane_witness::cp_candidate_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_control_plane_witness", in_declaration: "cp_candidate_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_control_plane_witness::cp_offer_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_control_plane_witness", in_declaration: "cp_offer_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_control_plane_witness::the_encumbered_liability_is_the_full_selection_cost Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_control_plane_witness", in_declaration: "the_encumbered_liability_is_the_full_selection_cost", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_demand_effect_witness_test::running_grant Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_demand_effect_witness_test", in_declaration: "running_grant", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_demand_effect_witness_test::second_running_grant Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_demand_effect_witness_test", in_declaration: "second_running_grant", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_demand_effect_witness_test::terms Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_demand_effect_witness_test", in_declaration: "terms", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_floor_dispatch_witness::d_bound Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_floor_dispatch_witness", in_declaration: "d_bound", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_floor_dispatch_witness::d_no_delay_valuation Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_floor_dispatch_witness", in_declaration: "d_no_delay_valuation", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_floor_dispatch_witness::d_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_floor_dispatch_witness", in_declaration: "d_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::an_alternative_use_displaced_prices_a_paid_through_hour_above_zero Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "an_alternative_use_displaced_prices_a_paid_through_hour_above_zero", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::t_inputs Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "t_inputs", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::t_no_delay_valuation Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "t_no_delay_valuation", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::t_owned_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "t_owned_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::t_owned_operating Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "t_owned_operating", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::t_rented_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "t_rented_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::t_select_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "t_select_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_selection_witness::valued_delay_enters_the_ranking_and_not_the_reservation Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_selection_witness", in_declaration: "valued_delay_enters_the_ranking_and_not_the_reservation", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_terminal_contract_witness::buy_terms Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_terminal_contract_witness", in_declaration: "buy_terms", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_terminal_contract_witness::owned_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_terminal_contract_witness", in_declaration: "owned_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_terminal_contract_witness::owned_supply_states_zero_rather_than_absent Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_terminal_contract_witness", in_declaration: "owned_supply_states_zero_rather_than_absent", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.fabric_terminal_contract_witness::rented_offer Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.fabric_terminal_contract_witness", in_declaration: "rented_offer", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.http_serve_route_witness::matcher_accepts PathTemplate",
        subject: AdmissionSubject::Binding { module: "test.claim.http_serve_route_witness", in_declaration: "matcher_accepts", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.http_serve_route_witness::tpl PathTemplate",
        subject: AdmissionSubject::Binding { module: "test.claim.http_serve_route_witness", in_declaration: "tpl", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.inventory_ledger_witness::usd_micros Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.inventory_ledger_witness", in_declaration: "usd_micros", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.inventory_ledger_witness::w_landed_cost_sums_within_one_currency_and_refuses_across Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.inventory_ledger_witness", in_declaration: "w_landed_cost_sums_within_one_currency_and_refuses_across", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.inventory_ledger_witness::w_landed_cost_sums_within_one_currency_and_refuses_across Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.inventory_ledger_witness", in_declaration: "w_landed_cost_sums_within_one_currency_and_refuses_across", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.inventory_ledger_witness::w_summing_no_lots_is_empty_not_refused Eur",
        subject: AdmissionSubject::Binding { module: "test.claim.inventory_ledger_witness", in_declaration: "w_summing_no_lots_is_empty_not_refused", spelling: "Eur" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.inventory_ledger_witness::w_summing_no_lots_is_empty_not_refused Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.inventory_ledger_witness", in_declaration: "w_summing_no_lots_is_empty_not_refused", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.pricing_surface_witness::wrong_github_2core_per_minute_fixture Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.pricing_surface_witness", in_declaration: "wrong_github_2core_per_minute_fixture", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.roadmap_serve_witness::witness_dispatch_param_bound path_param_value",
        subject: AdmissionSubject::Binding { module: "test.claim.roadmap_serve_witness", in_declaration: "witness_dispatch_param_bound", spelling: "path_param_value" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_attempt_graph_witness::demand_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_attempt_graph_witness", in_declaration: "demand_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_checkpoint_witness::demand_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_checkpoint_witness", in_declaration: "demand_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_execution_control_witness::demand_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_execution_control_witness", in_declaration: "demand_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_execution_control_witness::grant_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_execution_control_witness", in_declaration: "grant_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_observation_cut_witness::demand_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_observation_cut_witness", in_declaration: "demand_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_observation_cut_witness::subject_from Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_observation_cut_witness", in_declaration: "subject_from", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_readback_witness::demand_keyed Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_readback_witness", in_declaration: "demand_keyed", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_reconciliation_witness::demand_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_reconciliation_witness", in_declaration: "demand_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_reconciliation_witness::grant_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_reconciliation_witness", in_declaration: "grant_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.training_resolution_witness::demand_for Usd",
        subject: AdmissionSubject::Binding { module: "test.claim.training_resolution_witness", in_declaration: "demand_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: test.claim.uri_path_match_witness::tpl PathTemplate",
        subject: AdmissionSubject::Binding { module: "test.claim.uri_path_match_witness", in_declaration: "tpl", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: tools.fabric_control_plane_live_probe::cp_live_broker_reserves_the_cell_the_market_chose Usd",
        subject: AdmissionSubject::Binding { module: "tools.fabric_control_plane_live_probe", in_declaration: "cp_live_broker_reserves_the_cell_the_market_chose", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: tools.fabric_control_plane_live_probe::cp_live_nothing_admissible_touches_no_store Usd",
        subject: AdmissionSubject::Binding { module: "tools.fabric_control_plane_live_probe", in_declaration: "cp_live_nothing_admissible_touches_no_store", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: tools.fabric_control_plane_live_probe::cp_probe_account Usd",
        subject: AdmissionSubject::Binding { module: "tools.fabric_control_plane_live_probe", in_declaration: "cp_probe_account", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: tools.fabric_control_plane_live_probe::cp_probe_offer_for Usd",
        subject: AdmissionSubject::Binding { module: "tools.fabric_control_plane_live_probe", in_declaration: "cp_probe_offer_for", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: tools.fabric_control_plane_live_probe::cp_probe_roster Usd",
        subject: AdmissionSubject::Binding { module: "tools.fabric_control_plane_live_probe", in_declaration: "cp_probe_roster", spelling: "Usd" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: v1.compiler.effect_derivation::re_export_derive_op_effect PathTemplate",
        subject: AdmissionSubject::Binding { module: "v1.compiler.effect_derivation", in_declaration: "re_export_derive_op_effect", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: v1.compiler.effect_derivation::re_export_has_path_params PathTemplate",
        subject: AdmissionSubject::Binding { module: "v1.compiler.effect_derivation", in_declaration: "re_export_has_path_params", spelling: "PathTemplate" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "std->extdeps consolidation 2026-08-28: v1.compiler.effect_derivation::re_export_has_path_params has_path_params",
        subject: AdmissionSubject::Binding { module: "v1.compiler.effect_derivation", in_declaration: "re_export_has_path_params", spelling: "has_path_params" },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
];

/// The denominators a green must name (DESIGN §5). A run that cannot say what it covered is
/// an instrument failure wearing coverage's clothes.
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
/// BOTH CHANNELS, ON BOTH SIDES, is what makes the derivation survive the cut. Before Step 1
/// the import claims carry most of it; after Step 1 they carry none of it and the reference
/// channel carries all of it. The FUNCTION does not change, so a base measured before the cut
/// and a head measured after it are measured by one instrument.
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
    for (_, spelling) in &record.referenced {
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

/// The declaring modules a spelling admits inside one module — the candidate POPULATION, not
/// a winner. An empty set is unresolved-at-this-grain; two or more is ambiguity.
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
    if record.declared.contains(spelling) || record.variants.contains(spelling) {
        out.insert(record.module_path.clone());
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

/// Where a name reached through `module` is actually DECLARED. An import member may be a
/// re-export, and a citation naming a re-export names the wrong authority (DESIGN §3 — a
/// fact's home is the module that declares it), so the chain is followed to the declarer.
/// Bounded by a visited set, so a re-export cycle terminates at the last module reached
/// rather than spinning.
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
/// WHY THE KEY IS THE LEAF AND NOT THE SPELLING, and this is a measured correction rather than
/// a preference. Keyed on the spelling, `widget` and `probe.home.widget` are two rows, so
/// QUALIFYING A REFERENCE reads as one row losing its declaration and an unrelated row
/// appearing — `NewUnresolvedness`, refused. Requalification is the namespace program's own
/// core motion (its projection P(B) inserts qualifier segments), so a wall keyed on spellings
/// would refuse the program in its entirety and be repaired by weakening it, which is the
/// failure mode the ruling's `SameDeclarationIdentityRebind` auto-admission exists to prevent.
/// The fixture that found this is
/// `dropping_an_import_for_a_qualified_spelling_keeps_the_declarer_and_is_admitted`.
///
/// THE LEAF IS THE PART THAT NAMES THE DECLARATION; the segments before it name the ROUTE. So
/// keying on the leaf and valuing on the declaring set is exactly the ruling's distinction
/// between a rebind (route moved, identity held) and a target change (identity moved), read
/// off the structure rather than asserted by an author.
///
/// AND IT IS AN INVARIANT OF THE OPERATION THE CUT PERFORMS, not merely a key that happens to
/// survive it. A requalification wave prepends the declarer's path and leaves the declarer
/// fixed, so it changes the spelling and leaves the last segment unchanged BY CONSTRUCTION.
/// The reduction is not coined here: `v1.05_emit_rust` `rust_fn_sig_leaf_name_dotted_note`
/// names `qualified_last_segment` as the single authority for taking an authored spelling to
/// its last segment. The converse is the wall working and is not to be softened: if a cut
/// repoints a reference to a DIFFERENT declaration whose leaf differs, the key moves and the
/// wall refuses.
///
/// THE UNION IS THE CEILING, STATED WHERE IT IS TAKEN: two references to one leaf inside one
/// declaration share a row, so one of them being requalified while the other is not is not
/// observable here. That is the same ceiling the module header names, arriving through the key
/// instead of through the shadowing case, and it has the same next rung.
fn binding_rows(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
) -> BTreeMap<(String, String), BTreeSet<String>> {
    let mut out: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for (in_declaration, spelling) in &record.referenced {
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
/// TAKES TWO INDEXES AND NO GIT. The production caller reconstructs the base index from the
/// diff; a fixture authors both sides directly. That boundary is what makes this wall's RED
/// authorable — DESIGN §4b judges reachability against what a FIXTURE may author, and a
/// wall reachable only through a repository history has no fixture at all.
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

    // CLOSURE MOTION, MEASURED FIRST AND ATTRIBUTED BELOW. A module whose own membership did
    // not move can still have its subject moved by a dependency's membership; that is the
    // blast radius a refusal must name, and it is why closure is computed over the whole
    // graph rather than per changed file.
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
        // ONLY ROWS PRESENT ON BOTH SIDES. A name authored by this change has no prior
        // denotation, so it cannot have changed one; a name this change deleted has no present
        // denotation to protect. The wall's subject is what an EXISTING reference denotes, and
        // widening it to new authorship would make every ordinary pull request an
        // adjudication — the tax that gets a wall weakened rather than obeyed.
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
                closure_blast_radius: 0,
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
                // An edge every one of whose names still denotes what it denoted is motion
                // that has been evaluated and found to change no meaning. An edge nothing in
                // the module reaches is motion nobody has accounted for.
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
                closure_blast_radius: blast_radius(&closure_moved_for, target),
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
                // A removed edge nothing bound through is unused membership going away. A
                // removed edge whose names all still denote the same declarations is a route
                // change with the identity intact — which is exactly the rebind the ruling
                // auto-admits. Anything else is refused HERE only when the binding channel
                // did not already refuse it, so one motion is never charged twice.
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
                closure_blast_radius: blast_radius(&closure_moved_for, target),
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
    let stale_admissions = admissions
        .iter()
        .enumerate()
        .filter(|(i, _)| !used.contains(i))
        .map(|(_, a)| {
            format!(
                "{} ({} {}) matches no delta in this run",
                a.label,
                disposition_label(a.disposition),
                admission_subject_render(&a.subject)
            )
        })
        .collect();

    WaveAdmissionReport {
        population,
        deltas,
        stale_admissions,
    }
}

/// Which disposition a changed candidate SET carries.
///
/// EVERY ARM IS OVER SETS, NOT WINNERS. `1 -> 0` is a name that stopped denoting anything;
/// `1 -> 2` is a name that now admits two declarations, which the namespace authority refuses
/// at the reference rather than resolving by nearness. Anything else with a non-empty pair is
/// a changed target.
///
/// `0 -> 1` IS TWO STATES, NOT ONE, AND THE SET PAIR CANNOT TELL THEM APART. An earlier
/// revision read every `0 -> 1` as resolution arriving from a pool — a name that began
/// denoting something WITHOUT ANYONE AUTHORING A REFERENCE, which is the coincidence the
/// containment rule exists to remove. That is one of the two, and it is the one whose cause
/// lives in ANOTHER module: the target grew a name this module was already reaching for. The
/// other is this module's own author writing the import that resolves a name the module was
/// already spelling — the exact repair the wall was built to want, and the state the wall
/// refused. Opposite owners, opposite repairs, one symbol: DESIGN's state-space conflation.
///
/// THE DISCRIMINATOR IS THE MODULE'S OWN SOURCE, and it is available for free — the
/// membership arm one function over already consults authorship (`membership_supported`),
/// which is what admitted the membership edge of the very change this arm was refusing. So
/// `authored_here` is passed in rather than re-derived: see `locally_authored_claim_added`.
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
/// `DeltaSubject::Membership` asks whether `module` gained `target` as a DIRECT DEPENDENCY,
/// and `import <target> { .. }` is exactly that dependency stated in authored syntax. The
/// reference set is a DOWNSTREAM PROXY for the same fact and a measurably lossy one: a name
/// used ONLY as a match-arm pattern head is unreachable from `for_each_node` IN PRINCIPLE,
/// because `MatchPattern::VariantPattern.name` is a `String` and never a `Node`. So a module
/// importing a coproduct purely to name its variants in patterns declared the dependency and
/// had it refused as `UnexplainedSubjectMotion`. Reading the weaker of two representations of
/// one fact is DESIGN 3, not a leniency question.
///
/// THE SPLIT FROM `membership_bound_through` IS THE FINDING, NOT A TIDY-UP. One predicate
/// served both directions until an executed RED showed the two ask OPPOSITE questions of the
/// same data. Add asks *does this module depend on target*, which an import claim answers
/// outright. Removal asks *was anything actually bound through it*, which an import claim
/// CANNOT answer -- an unused import is precisely one that is declared and bound through by
/// nothing. Widening the SHARED predicate therefore made every unused-import removal report
/// `SameDeclarationIdentityRebind` instead of `UnusedSubjectMembershipRemoved`; that is the
/// state-space conflation DESIGN names, one symbol answering two questions whose arms have
/// opposite owners and opposite repairs. It was caught by the sibling test going red, not by
/// review, which is why the fixture below is enrolled rather than described.
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
/// member-list import and a self-declaration name the leaf in the source, so they answer from
/// the two `ModuleDeclarationRecord`s alone. A blanket `import m` names no leaf at all, so a
/// first revision of this function admitted authorship whenever ANY blanket target was new —
/// which meant an unrelated new blanket import elsewhere in the same module auto-admitted a
/// genuine pool coincidence, a fail-open widening in the one direction this function must
/// never move (review 56882, on gunbc#9495, before it merged).
///
/// SO THE BLANKET ARM IS A CONJUNCTION, AND THE ORDER OF ITS TWO HALVES IS THE WHOLE POINT.
/// The claim must be NEW IN THIS MODULE'S SOURCE **and** the head target must actually supply
/// the leaf. Asking the second question alone is exactly the conflation this function exists
/// to discriminate — an unchanged blanket import whose target grew `leaf` would read as
/// authorship, which is the pool coincidence. Gating it behind the first question makes the
/// index consultation safe rather than forbidden: a claim the author did not write cannot
/// reach the surface check at all, whatever the target did.
///
/// FALSE IS THE FAIL-CLOSED ANSWER: it leaves the delta on the refusing arm, where a human
/// adjudicates. A target absent from the index answers false for the same reason.
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
    // THE UNION OF BOTH AUTHORED-REFERENCE CHANNELS, and the two are peers rather than one
    // widened set for the reason `authored_type_references` states: they have different
    // authorities. `referenced` is the index's own walk over the final tree; the other is the
    // parser's stamped answer, which reaches a declared type parked in `inferred` that no walk
    // over the tree can see. A consumer asking "is anything here bound through that import"
    // wants every authored reference regardless of which channel could observe it, so it takes
    // both -- and asking only the walk is what let this predicate report a live import as
    // unused.
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
    format!(
        "{} {} — {}{} [closure blast radius: {} module(s)]",
        disposition_label(delta.disposition),
        delta_subject_render(&delta.subject),
        delta.detail,
        admitted,
        delta.closure_blast_radius
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
/// `NotEvaluated` IS A REFUSAL AND NOT A SKIP, and the distinction is the whole reason it is a
/// variant rather than an `Err` folded in with a spawn failure. "I could not see what changed"
/// and "nothing changed" are different states with different remedies (DESIGN §5, the
/// empty-observation narrow), and the ruling puts `NotEvaluated` on the refusing side of the
/// admission partition explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaveAdmissionOutcome {
    /// The baseline resolves to the head, so this run has no diff to adjudicate. A push to
    /// `main` after a squash merge is the whole population of this state. It is NOT an
    /// admission: nothing was compared, and the phase reports it under its own name.
    NoSubject { head: String },
    /// The baseline could not be observed. Refuses.
    NotEvaluated { reason: String },
    Adjudicated {
        base: String,
        head: String,
        report: WaveAdmissionReport,
    },
}

/// Run git in the workspace and return stdout with TRAILING whitespace removed, or a refusal
/// naming the command.
///
/// `trim_end`, not `trim`, and the asymmetry is load-bearing rather than incidental: one caller
/// asks for the CONTENT OF A FILE at a ref (`git show <ref>:<path>`), and a `.dag` module whose
/// first line is indented would arrive with that indentation eaten -- a different file from the
/// one committed, compared against a head side read from disk intact. Every other caller here
/// (`rev-parse`, `merge-base`, `diff --name-only`, `status --porcelain`) produces output that
/// either carries no leading whitespace or is re-trimmed by the caller, so nothing pays for the
/// asymmetry. `status --porcelain` is the one to notice: its lines BEGIN with the two-column XY
/// code, so a leading `trim` would have corrupted a status read rather than tidied it.
///
/// THE ONE COPY. `claim_executor` carried a byte-identical private copy, and this module's first
/// draft carried a second — three spellings of one concept would have been the §3 fork this wall
/// exists to refuse elsewhere. It is `pub` here rather than private there because the wall is a
/// library fold and the bin is one of its callers; the bin's copy is deleted and its five call
/// sites read this one.
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

/// Whether a repository path is one the head sweep would have parsed. The two exclusions
/// mirror `run_dag_parse_sweep`'s own: build output, and the parser's deliberately malformed
/// fixtures. A base-side file the head sweep would not have read must not enter the base
/// index, or the two sides are measured by different instruments.
fn in_sweep_scope(rel: &str) -> bool {
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
/// reports a detected rename as ONE path -- the destination -- because rename detection is on by
/// default. An earlier revision read that single list as both sides at once, so a renamed module
/// lost its base side entirely: the source path was never re-read (it is not in the list) and the
/// destination path is absent from the base tree, so every declaration in the module read as
/// newly added and the wall could refuse an ordinary `.dag` rename over a delta it had invented.
/// Review 56471 was right to reject it.
///
/// So the diff is read rename-aware, `--name-status -z -M`, and each entry contributes to the two
/// sides SEPARATELY: a rename contributes its destination to the head side and its source to the
/// base side, an addition contributes only a head path, a deletion only a base path, and an
/// ordinary modification the same path to both. Scope is applied per side, because a rename may
/// cross the sweep boundary in either direction.
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
    head_touched.retain(|p| in_sweep_scope(p));
    base_side.retain(|p| in_sweep_scope(p));
    (head_touched, base_side)
}

/// One base-side source parsed into records, or a refusal naming what could not be read.
///
/// A BASE FILE THAT DOES NOT PARSE IS AN UNOBSERVABLE BASELINE, NOT AN EMPTY ONE, and the
/// difference decides the verdict. An earlier revision returned an empty vec here and argued
/// that this was "the conservative direction" because it could only make the wall quieter.
/// That argument is exactly backwards, and review 56449 was right to reject it: rows with no
/// base side are not deltas, so every binding and membership row that file would have carried
/// silently STOPS BEING COMPARED while the run still answers `Adjudicated`. Quieter is not
/// safer here — it is the empty-observation narrow DESIGN names, ⊥-as-answer conflated with
/// ⊥-as-ignorance, and it is strictly worse than the widen §5 already forbids: a widen is
/// merely expensive, a narrow is silently uncovered.
///
/// The head side is measured by a sweep that refuses on diagnostics, so refusing here is also
/// what keeps both sides measured by ONE instrument. History is still not this pull request's
/// to repair — but "I cannot see the baseline" is a refusal to state, not a fact to assume.
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
/// THE BASE INDEX IS THE HEAD INDEX WITH THE DIFF APPLIED IN REVERSE, at file grain, and that
/// is the construction rather than an optimisation. A file the diff did not touch has the same
/// text on both sides, so its record is the same record — deriving it twice would produce the
/// same answer at twice the cost, and a second corpus walk is the cost shape DESIGN §6 names.
/// So only the changed files are parsed again, from their base blobs, and substituted. The
/// closure and the bindings are then recomputed over BOTH whole graphs, because a module whose
/// own text did not move can still have its subject or its bindings moved by one that did.
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

    let mut base_index = DeclarationIndex::default();
    for record in index_records(head_index) {
        if !head_touched.iter().any(|c| c == &record.rel_path) {
            crate::cli_run::declaration_index::index_insert(&mut base_index, record.clone());
        }
    }
    // ABSENCE AT THE BASE IS ESTABLISHED FROM AN AUTHORITATIVE LISTING, NEVER INFERRED FROM A
    // FAILURE. An earlier revision read `git show <base>:<path>` and treated ANY error as proof
    // the change had ADDED that path — so a read fault, a corrupt object or a permission problem
    // all rendered as "new file", and the module's entire base side vanished from the comparison
    // while the run still answered `Adjudicated`. Review 56449 was right to reject it: only ONE
    // cause means added, and the rest are ignorance wearing its clothes.
    //
    // `ls-tree` separates the two: it answers what the base tree CONTAINS, so a path missing from
    // its output is genuinely absent, and a failure to obtain the listing at all is a refusal
    // rather than an empty answer.
    let base_paths = git_stdout(&workspace, &["ls-tree", "-r", "--name-only", &base])?;
    let base_paths: std::collections::BTreeSet<String> =
        base_paths.lines().map(|l| l.trim().to_string()).collect();
    for rel in &base_side {
        if !base_paths.contains(rel) {
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

    let report = adjudicate(&base_index, head_index, NAMESPACE_TRANSITION_ADMISSIONS);
    Ok(WaveAdmissionOutcome::Adjudicated { base, head, report })
}
