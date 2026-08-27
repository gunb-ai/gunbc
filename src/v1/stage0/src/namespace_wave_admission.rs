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
use std::sync::LazyLock;

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
    pub subject: DeltaSubject,
    pub disposition: NamespaceDeltaDisposition,
}

/// Exact admissions for #9400's owner-qualified call-target cut. Each row was measured by the
/// required floor against the merge base after the namespace wall landed; no disposition-wide
/// predicate is admitted. A row that no longer matches is itself a finding
/// (`stale_admissions`), so this temporary transition roster must shrink with its subject.
pub static NAMESPACE_TRANSITION_ADMISSIONS: LazyLock<Vec<TransitionAdmission>> =
    LazyLock::new(|| {
        vec![
    TransitionAdmission {
        label: "owner-qualified-call-target-01",
        subject: DeltaSubject::Binding {
            module: "gunbc.host_effect_realize".to_string(),
            in_declaration: "srv3_realize_os_install_actuator_toolchain_ensure_body".to_string(),
            spelling: "list_map".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-02",
        subject: DeltaSubject::Binding {
            module: "gunbc.host_effect_realize".to_string(),
            in_declaration: "srv3_toolchain_receipt".to_string(),
            spelling: "list_map".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-03",
        subject: DeltaSubject::Binding {
            module: "gunbc.host_effect_realize".to_string(),
            in_declaration: "websocat_trace_labels".to_string(),
            spelling: "list_map".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-04",
        subject: DeltaSubject::Binding {
            module: "gunbc.roadmap_sandbox".to_string(),
            in_declaration: "sandbox_page".to_string(),
            spelling: "workspace_band_paints".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-05",
        subject: DeltaSubject::Binding {
            module: "gunbc.roadmap_style".to_string(),
            in_declaration: "audition_sets".to_string(),
            spelling: "List".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-06",
        subject: DeltaSubject::Binding {
            module: "gunbc.roadmap_style".to_string(),
            in_declaration: "band_var_decls".to_string(),
            spelling: "List".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-07",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "admit_spark_readback_version".to_string(),
            spelling: "NonEmptyStr".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-08",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "compare_spark_readback".to_string(),
            spelling: "NonEmptyStr".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-09",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "derive_install_scope".to_string(),
            spelling: "NonEmptyStr".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-10",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "no_access_demand".to_string(),
            spelling: "List".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-11",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "spark_cell_in".to_string(),
            spelling: "List".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-12",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "spark_demand_for_selected".to_string(),
            spelling: "NonEmptyStr".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-13",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "spark_iam_delta".to_string(),
            spelling: "List".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-14",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "spark_legacy_containers".to_string(),
            spelling: "List".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-15",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "spark_plan_demand".to_string(),
            spelling: "List".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-16",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "spark_scope_admits".to_string(),
            spelling: "NonEmptyStr".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-17",
        subject: DeltaSubject::Binding {
            module: "gunbc.spark.bootstrap_provision".to_string(),
            in_declaration: "srv5_privilege_observation".to_string(),
            spelling: "NonEmptyStr".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-18",
        subject: DeltaSubject::Binding {
            module: "test.claim.citation_cit1_consumer_witness".to_string(),
            in_declaration: "x86_64_abi_syscall_citation_selector_conforms_to_rfc_8118".to_string(),
            spelling: "extdeps_external_authority_anchor".to_string(),
        },
        disposition: NamespaceDeltaDisposition::TargetChanged,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-20",
        subject: DeltaSubject::Binding {
            module: "test.claim.serving_privilege_derivation_witness_test".to_string(),
            in_declaration: "red_control_non_escalating_effects_contribute_no_grant_line"
                .to_string(),
            spelling: "Empty".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-21",
        subject: DeltaSubject::Binding {
            module: "test.claim.serving_privilege_derivation_witness_test".to_string(),
            in_declaration: "red_control_non_escalating_effects_contribute_no_grant_line"
                .to_string(),
            spelling: "length".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-22",
        subject: DeltaSubject::Binding {
            module: "test.claim.serving_privilege_derivation_witness_test".to_string(),
            in_declaration: "w_a_realization_with_fewer_escalating_effects_gets_a_smaller_grant"
                .to_string(),
            spelling: "length".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-23",
        subject: DeltaSubject::Binding {
            module: "test.claim.serving_privilege_derivation_witness_test".to_string(),
            in_declaration:
                "w_the_same_realization_needs_no_grant_when_the_actor_owns_what_it_mutates"
                    .to_string(),
            spelling: "length".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-24",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_absent_creates_and_empty_container_converges".to_string(),
            spelling: "classify_spark_container".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-25",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_bad_readback_refuses".to_string(),
            spelling: "decide_spark_materialization".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-26",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_confirmed_receipt_is_noop_at_exact_version".to_string(),
            spelling: "decide_spark_materialization".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-27",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_container_with_existing_version_conflicts".to_string(),
            spelling: "classify_spark_container".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-28",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_effective_access_is_only_partially_observed".to_string(),
            spelling: "spark_effective_access_standing".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-29",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_factory_hotspot_cannot_be_generated".to_string(),
            spelling: "admit_spark_material_source".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-30",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_first_write_requires_a_journaled_attempt".to_string(),
            spelling: "decide_spark_materialization".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-31",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_issued_unconfirmed_observes_never_rewrites".to_string(),
            spelling: "decide_spark_materialization".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-32",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_receipt_version_must_match_its_readback".to_string(),
            spelling: "decide_spark_materialization".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-33",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_unallocated_occurrence_refuses_observation".to_string(),
            spelling: "classify_spark_container".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-34",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_unreadable_container_refuses".to_string(),
            spelling: "classify_spark_container".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-35",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_version_address_is_not_a_payload_match".to_string(),
            spelling: "admit_spark_readback_version".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-36",
        subject: DeltaSubject::Binding {
            module: "test.claim.spark_bootstrap_provision_witness".to_string(),
            in_declaration: "spark_version_address_is_not_a_payload_match".to_string(),
            spelling: "compare_spark_readback".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-37",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration:
                "witness_srv3_nbd_proxy_apply_emit_artifact_transport_observe_refuses_fail_closed"
                    .to_string(),
            spelling: "host_effect_nbd_proxy_serve_session_binding".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-38",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration:
                "witness_srv3_nbd_proxy_apply_emit_artifact_transport_observe_refuses_fail_closed"
                    .to_string(),
            spelling: "srv3_nbd_proxy_serve_host_effect".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-39",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_foreign_port_refuses_at_apply_time"
                .to_string(),
            spelling: "PortObserved".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-40",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_foreign_port_refuses_at_apply_time"
                .to_string(),
            spelling: "host_effect_nbd_proxy_serve_apply_with_observation".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-41",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_foreign_port_refuses_at_apply_time"
                .to_string(),
            spelling: "host_effect_nbd_proxy_serve_session_binding".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-42",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_foreign_port_refuses_at_apply_time"
                .to_string(),
            spelling: "srv3_nbd_proxy_serve_host_effect".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-43",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_stale_matching_drains_not_foreign_kill"
                .to_string(),
            spelling: "PortObserved".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-44",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_stale_matching_drains_not_foreign_kill"
                .to_string(),
            spelling: "PortObservedStaleExpected".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-45",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_stale_matching_drains_not_foreign_kill"
                .to_string(),
            spelling: "host_effect_nbd_proxy_serve_apply_drains_stale_matching".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-46",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_stale_matching_drains_not_foreign_kill"
                .to_string(),
            spelling: "host_effect_nbd_proxy_serve_apply_refuses_foreign_port".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-47",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_stale_matching_drains_not_foreign_kill"
                .to_string(),
            spelling: "host_effect_nbd_proxy_serve_session_binding".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-48",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_apply_stale_matching_drains_not_foreign_kill"
                .to_string(),
            spelling: "srv3_nbd_proxy_serve_host_effect".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-49",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_on_bmc_refuses_typed".to_string(),
            spelling: "srv3_nbd_proxy_serve_host_effect".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-50",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_serve_session_binding_carries_lease"
                .to_string(),
            spelling: "host_effect_nbd_proxy_serve_session_binding".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-51",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_serve_session_binding_carries_lease"
                .to_string(),
            spelling: "host_effect_nbd_proxy_serve_session_policy_is_held_lease".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-52",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_serve_session_binding_carries_lease"
                .to_string(),
            spelling: "srv3_nbd_proxy_lease_key".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-53",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration: "witness_srv3_nbd_proxy_serve_session_binding_carries_lease"
                .to_string(),
            spelling: "srv3_nbd_proxy_serve_host_effect".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
    TransitionAdmission {
        label: "owner-qualified-call-target-54",
        subject: DeltaSubject::Binding {
            module: "test.claim.srv3_host_effect_apply_witness".to_string(),
            in_declaration:
                "witness_srv3_nbd_proxy_websocat_argv_materializes_token_not_shell_env_ref"
                    .to_string(),
            spelling: "srv3_nbd_proxy_serve_host_effect".to_string(),
        },
        disposition: NamespaceDeltaDisposition::NewPoolCoincidenceResolution,
    },
        ]
    });

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
            if admission.subject == delta.subject && admission.disposition == delta.disposition {
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
                delta_subject_render(&a.subject)
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
    record.referenced.iter().any(|(_, spelling)| {
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
        .map(|module| record_from_module(module, &source_indices, rel))
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

    let report = adjudicate(
        &base_index,
        head_index,
        NAMESPACE_TRANSITION_ADMISSIONS.as_slice(),
    );
    Ok(WaveAdmissionOutcome::Adjudicated { base, head, report })
}
