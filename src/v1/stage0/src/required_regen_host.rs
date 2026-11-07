//! Host realization for `v2.workflow.required_regen` — committed seed vs fresh emit.

// CLIPPY ROSTER -- 16 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::disallowed_macros,  // 3
    clippy::enum_variant_names,  // 1
    clippy::only_used_in_recursion,  // 1
    clippy::redundant_closure,  // 2
    clippy::too_many_arguments,  // 3
    clippy::type_complexity,  // 3
    dead_code,  // 2
    unused_variables,  // 1
)]

use crate::v1_interpreter::bounded_shell_host_drain::{
    default_shell_stderr_capture_policy, drain_stream, StreamCapturePolicy,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::Instant;

#[path = "bootstrap_seed_retention_frontier_generated.rs"]
mod bootstrap_seed_retention_frontier_generated;
#[path = "bootstrap_stage0_crate_layout_generated.rs"]
mod bootstrap_stage0_crate_layout_generated;
use super::workspace_root;
use crate::gunbc_stage0_stage0_emitted_population_manifest::{
    emitted_population_manifest_basename, emitted_population_manifest_line_prefix,
    emitted_population_manifest_line_separator,
};
use crate::v1_compiler_artifact::{RenderTarget, RustModuleRenderSelection};
use crate::v1_compiler_compile::{
    compile_sources_selected, compile_to_resolved, emittable_graph,
    stage0_self_compile_refusal_message, SourceFile,
};
use crate::v1_compiler_emit_rust::rust_module_emit_path;
use crate::v1_rt;
use bootstrap_seed_retention_frontier_generated::SEED_RETENTION_FRONTIER_TOP_LEVEL_SRC_BASENAMES;
use bootstrap_stage0_crate_layout_generated::{
    EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES, HAND_MAINTAINED_STAGE0_DIRS,
    HAND_MAINTAINED_STAGE0_FILES,
};

/// Bumped from `.v1` when the flat eight-field record split into the two-variant carrier below.
///
/// Its only reader, the deleted second-generation pass (operator ruling 2026-10-04), compared it
/// before building on a prior receipt; what still rejects a stale record is `deny_unknown_fields`
/// on the carrier, which refuses the `.v1` shape's `fixed_point_equal` and the deleted
/// `fixed_point` variant's fields alike.
///
/// Bumped from `.v2` when `FirstGeneration` gained the REQUIRED `surface_roles` (generation 0
/// carries the round's surface roles to the orchestrator): a `.v2` first-generation receipt no
/// longer parses, so the shape it names is a different carrier and gets a different name.
const RECEIPT_SCHEMA: &str = "gunbc.regen_receipt.v3";

/// EACH VARIANT CARRIES ONLY WHAT ITS PASS MEASURED (operator ruling 2026-08-20). There is one pass
/// now: the second-generation pass and its `FixedPoint` variant, which referenced the first pass's
/// evidence without impersonating it, were deleted under the operator ruling of 2026-10-04 ("we
/// are not that particular about determinism in v1 anymore") -- a same-binary re-emit could only
/// ever see nondeterminism. The declared drop is `gunbc.rung_drop`
/// `floor_cut_regen_second_generation_agreement`.
///
/// THE SURFACE-ROLE FACTS a convergence round plans from (`gunbc.regen_receipt` `RegenSurfaceRoles`),
/// carried by the generation that derived them. Building them loads the module index and its
/// per-file heads readings, so they are generation work under generation-state lifetime isolation
/// (`v2.workflow.regen_convergence_transaction` `regen_admit_generation_begin`): generation 0's
/// child computes them while its corpus is resident anyway and writes them into its receipt, and
/// the orchestrator plans from the receipt instead of loading the corpus into its own lifetime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConvergenceSurfaceRoles {
    pub basename_to_module: Vec<SurfaceOwnerRow>,
    pub generation_modules: Vec<String>,
    pub bootstrap_sources: Vec<String>,
    pub bootstrap_products: Vec<String>,
    pub seed_embedded_basenames: Vec<String>,
    pub generated_product_roles: Vec<GeneratedProductRoleRow>,
}

/// `gunbc.regen_receipt` `RegenSurfaceOwner`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceOwnerRow {
    pub basename: String,
    pub module: String,
}

/// `gunbc.regen_receipt` `RegenGeneratedProductRole`: the two role facts are the closed coproducts of
/// `gunbc.regen_affected_set`, serialized under the host-boundary labels that module projects
/// (`regen_generated_product_generation_role_labels`, `regen_generated_product_seed_membership_labels`),
/// so an unknown label is unwritable on the way in and does not deserialize on the way back.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedProductRoleRow {
    pub product: String,
    pub generation_role: GeneratedProductGenerationRole,
    pub seed_membership: GeneratedProductSeedMembership,
}

/// `gunbc.regen_affected_set` `GeneratedAggregateProductGenerationRole`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
pub enum GeneratedProductGenerationRole {
    GenerationInput,
    GenerationSubject,
    NonSeedGeneratedOutput,
}

/// `gunbc.regen_affected_set` `GeneratedAggregateProductSeedMembership`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
pub enum GeneratedProductSeedMembership {
    SeedEmbedded,
    OutsideSeed,
}

impl GeneratedProductGenerationRole {
    fn from_label(label: &str) -> Result<Self, String> {
        match label {
            "GenerationInput" => Ok(Self::GenerationInput),
            "GenerationSubject" => Ok(Self::GenerationSubject),
            "NonSeedGeneratedOutput" => Ok(Self::NonSeedGeneratedOutput),
            other => Err(format!(
                "SurfaceRolesReceiptUnknownRole: generation role {other:?}"
            )),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::GenerationInput => "GenerationInput",
            Self::GenerationSubject => "GenerationSubject",
            Self::NonSeedGeneratedOutput => "NonSeedGeneratedOutput",
        }
    }
}

impl GeneratedProductSeedMembership {
    fn from_label(label: &str) -> Result<Self, String> {
        match label {
            "SeedEmbedded" => Ok(Self::SeedEmbedded),
            "OutsideSeed" => Ok(Self::OutsideSeed),
            other => Err(format!(
                "SurfaceRolesReceiptUnknownRole: seed membership {other:?}"
            )),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::SeedEmbedded => "SeedEmbedded",
            Self::OutsideSeed => "OutsideSeed",
        }
    }
}

/// The planning shape the convergence loop reads.
type SurfaceRolesForPlanning = (
    HashMap<String, String>,
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeSet<String>,
    HashMap<String, (String, String)>,
);

impl ConvergenceSurfaceRoles {
    /// The receipt rows, sorted, so one tree's roles always serialize to the same bytes. A role
    /// label outside the closed coproducts refuses.
    fn from_planning(roles: SurfaceRolesForPlanning) -> Result<Self, String> {
        let (owners, generation, sources, products, embedded, product_roles) = roles;
        let mut basename_to_module: Vec<SurfaceOwnerRow> = owners
            .into_iter()
            .map(|(basename, module)| SurfaceOwnerRow { basename, module })
            .collect();
        basename_to_module.sort();
        let mut generated_product_roles = product_roles
            .into_iter()
            .map(|(product, (generation_role, seed_membership))| {
                Ok(GeneratedProductRoleRow {
                    product,
                    generation_role: GeneratedProductGenerationRole::from_label(&generation_role)?,
                    seed_membership: GeneratedProductSeedMembership::from_label(&seed_membership)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        generated_product_roles.sort();
        Ok(ConvergenceSurfaceRoles {
            basename_to_module,
            generation_modules: generation.into_iter().collect(),
            bootstrap_sources: sources.into_iter().collect(),
            bootstrap_products: products.into_iter().collect(),
            seed_embedded_basenames: embedded.into_iter().collect(),
            generated_product_roles,
        })
    }

    /// Back to the planning shape. A basename or product listed twice is refused rather than
    /// resolved by whichever row came last.
    fn into_planning(self) -> Result<SurfaceRolesForPlanning, String> {
        let mut owners = HashMap::new();
        for row in self.basename_to_module {
            if owners.insert(row.basename.clone(), row.module).is_some() {
                return Err(format!(
                    "SurfaceRolesReceiptDuplicated: basename {} has two owner rows",
                    row.basename
                ));
            }
        }
        let mut product_roles = HashMap::new();
        for row in self.generated_product_roles {
            let labels = (
                row.generation_role.label().to_string(),
                row.seed_membership.label().to_string(),
            );
            if product_roles.insert(row.product.clone(), labels).is_some() {
                return Err(format!(
                    "SurfaceRolesReceiptDuplicated: product {} has two role rows",
                    row.product
                ));
            }
        }
        Ok((
            owners,
            self.generation_modules.into_iter().collect(),
            self.bootstrap_sources.into_iter().collect(),
            self.bootstrap_products.into_iter().collect(),
            self.seed_embedded_basenames.into_iter().collect(),
            product_roles,
        ))
    }
}

/// Authority: `gunbc.regen_receipt`.
#[derive(Debug, Serialize, serde::Deserialize)]
#[serde(tag = "pass", deny_unknown_fields)]
pub enum RegenReceipt {
    #[serde(rename = "first_generation")]
    FirstGeneration {
        schema: String,
        commit_sha: String,
        authority_digest: String,
        committed_generated_digest: String,
        candidate_generated_digest: String,
        first_generation_equal: bool,
        changed_paths: Vec<String>,
        candidate_artifact: String,
        candidate_manifest: RegenCandidateManifest,
        /// Required: a receipt without them does not parse, so an orchestrator can never plan a
        /// round from roles nobody derived.
        surface_roles: ConvergenceSurfaceRoles,
    },
    /// A POPULATION REFUSAL HAS NO MEASUREMENT, AND THIS VARIANT HAS NOWHERE TO WRITE ONE.
    ///
    /// The refusal happens BEFORE any content comparison, so there is no first generation to
    /// digest and no comparison to answer. Written as a `FirstGeneration` receipt this forced
    /// three fabrications into a persisted artifact: two `refused:population` strings in digest
    /// positions, and `first_generation_equal: false` where "not asked" belonged. The Bool was
    /// load-bearing: unlike the string sentinels it is INDISTINGUISHABLE from a real answer --
    /// `false` is what an honest unequal comparison writes.
    ///
    /// It CROSSED THE PROCESS BOUNDARY: the receipt file was then read back by the (since deleted)
    /// second-generation pass as a measurement, and printed as a first-generation answer for a
    /// comparison that never ran.
    ///
    /// "Not asked" was already modelled correctly by the `first_generation_equal()` accessor's
    /// documented `Option`, so the Bool was a second representation disagreeing with a correct
    /// neighbour, not a missing state (DESIGN §3). The variant has no `first_generation_equal` and no digest
    /// fields, so the fabrication is UNWRITABLE (DESIGN §4b, structural impossibility). It carries
    /// the one thing a refusal knows -- why it refused.
    #[serde(rename = "refused")]
    Refused {
        schema: String,
        commit_sha: String,
        authority_digest: String,
        reason: String,
        candidate_artifact: String,
    },
    /// THE EDIT AFFECTS NO COMPARED MIRROR, so this round adjudicated nothing.
    ///
    /// Reached only under an affected scope whose selection is empty -- an edit confined to `.dag`
    /// modules that no mirror is emitted from, which is an ordinary state and not a defect. This
    /// PR's own edit set is one: `v2.workflow.required_regen` and `gunbc.regen_round_cost` are read
    /// through the interpreter and have no mirror in `src/v1/stage0/src`.
    ///
    /// IT CARRIES NO DIGESTS, for exactly the reason `Refused` carries none: nothing was compared,
    /// so there is no first generation to have a digest of. Routing this through `FirstGeneration`
    /// would have put an empty-population digest in a field read as evidence.
    /// The three empty-population refusals in this file (`verify_candidate_tree`,
    /// `tree_digest_for_basenames`, `tree_digest_from_map`) are RIGHT and stay: a digest over
    /// nothing is not evidence of anything, and for the whole-population round an empty population
    /// means the tree is broken. This variant is how the scoped round avoids asking them a
    /// question they correctly refuse, rather than weakening them so they answer it.
    #[serde(rename = "no_affected_mirrors")]
    NoAffectedMirrors {
        schema: String,
        commit_sha: String,
        authority_digest: String,
        /// The scope line that selected nothing, so the receipt says WHY it adjudicated nothing.
        scope: String,
    },
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub enum RegenCandidateManifestSurfaceRole {
    GeneratedSurface,
    BootstrapSourceMirror,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct RegenCandidateManifestSurface {
    pub declaring_module: String,
    pub projected_path: String,
    pub content_digest: String,
    pub role: RegenCandidateManifestSurfaceRole,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct RegenCandidateManifest {
    pub producer_seed_digest: String,
    pub generation_id: String,
    pub candidate_tree_id: String,
    pub candidate_tree_digest: String,
    pub surfaces: Vec<RegenCandidateManifestSurface>,
    pub aggregate_digest: String,
}

impl RegenReceipt {
    /// The tree THIS pass ran against.
    pub fn commit_sha(&self) -> &str {
        match self {
            RegenReceipt::FirstGeneration { commit_sha, .. } => commit_sha,
            RegenReceipt::Refused { commit_sha, .. } => commit_sha,
            RegenReceipt::NoAffectedMirrors { commit_sha, .. } => commit_sha,
        }
    }

    /// `Some` only where the pass measured it: "not asked" and "measured as false" are different
    /// states and a `bool` cannot hold both.
    pub fn first_generation_equal(&self) -> Option<bool> {
        match self {
            RegenReceipt::FirstGeneration {
                first_generation_equal,
                ..
            } => Some(*first_generation_equal),
            RegenReceipt::Refused { .. } => None,
            // Nothing was compared, so there is no answer -- not `Some(true)`, which would report a
            // clean comparison that never happened.
            RegenReceipt::NoAffectedMirrors { .. } => None,
        }
    }

    /// The candidate artifact path, measured only by the first pass.
    pub fn candidate_artifact(&self) -> Option<&str> {
        match self {
            RegenReceipt::FirstGeneration {
                candidate_artifact, ..
            } => Some(candidate_artifact),
            RegenReceipt::Refused {
                candidate_artifact, ..
            } => Some(candidate_artifact),
            // No candidate tree was written, because no emit was run.
            RegenReceipt::NoAffectedMirrors { .. } => None,
        }
    }

    /// Why the pass refused, present only on `Refused` -- the one fact a refusal measured, carried
    /// so the next pass refuses with the ORIGINAL cause instead of an invented comparison result.
    pub fn refusal_reason(&self) -> Option<&str> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
            RegenReceipt::Refused { reason, .. } => Some(reason),
            // Not a refusal: the round ran and correctly had nothing to do.
            RegenReceipt::NoAffectedMirrors { .. } => None,
        }
    }
}

#[derive(Debug)]
pub struct RequiredRegenOutcome {
    pub receipt: RegenReceipt,
    pub failures: Vec<String>,
}

/// THE UNSCOPED ROUND, unchanged: every committed mirror's bytes adjudicated, written and
/// digested. This is what the required CI phase and `--required-regen` run, and it is the
/// measurement that establishes the fixed-point precondition a scoped round relies on.
pub fn run_required_regen(
    candidate_dir_rel: &str,
    receipt_rel: &str,
) -> Result<RequiredRegenOutcome, String> {
    run_required_regen_scoped(
        candidate_dir_rel,
        receipt_rel,
        &RegenEmissionScope::WholePopulation,
    )
}

/// The same round with the affected-set bound consumed. See `RegenEmissionScope` and, for the
/// reasoning, `v2.workflow.required_regen` `RegenEmissionScope`.
pub fn run_required_regen_scoped(
    candidate_dir_rel: &str,
    receipt_rel: &str,
    scope: &RegenEmissionScope,
) -> Result<RequiredRegenOutcome, String> {
    let workspace = workspace_root();
    let candidate_dir = workspace.join(candidate_dir_rel);
    let receipt_path = workspace.join(receipt_rel);
    let stage0_src = workspace.join("src/v1/stage0/src");
    let run_started = Instant::now();

    // ADMISSION, FIRST. Every normalize below needs the formatter, so an absent one is decidable
    // here, before any emit, digest or comparison is paid for; it used to surface at the first
    // spawn, ~50 minutes in.
    let formatter = ResolvedFormatter::admit()?;

    // THE SCOPE ANSWERS BEFORE THE EMIT IS PAID FOR, for the same reason the formatter is
    // admitted here: a round that cannot say which mirrors it is regenerating has nothing to do
    // with the ~7 minutes it is about to spend. `scope_selection` over the committed roster is
    // the refusal arm; it is the one call that can fail, and it fails loudly rather than
    // widening to the population.
    let selected: BTreeSet<String> =
        scope_selection(scope, &committed_generated_basenames(&stage0_src)?)?
            .into_iter()
            .collect();
    eprintln!("{}", scope.line());

    let commit_sha = git_head_sha(&workspace)?;
    // PHASE MARKS. Every boundary below is stamped through `v1_rt::trace_mark`, the instrument
    // compile.dag uses for its stages, so `--regen-round-cost` reads one ledger for the round.
    // Labels: `gunbc.regen_round_cost` `regen_round_phase_label`, verbatim.
    v1_rt::trace_mark("regen.corpus_load.begin".to_string());
    let sources = super::regen_input_sources(&workspace)?;
    v1_rt::trace_mark("regen.corpus_load.done".to_string());
    let authority_digest = authority_digest_from_sources(&sources)?;
    let formatter = formatter.with_normalize_cache(&workspace)?;

    // AN EMPTY SELECTION ENDS THE ROUND HERE, SUCCESSFULLY, BEFORE THE EMIT.
    //
    // Finding from codex/gpt-5.6-sol on review 57625, and it was right: the scoped round drove an
    // empty selection into `verify_candidate_tree` and both digest functions, all three of which
    // correctly refuse an empty population -- so the state `v2.workflow.required_regen`
    // `regen_scope_line` documents as an ordinary answer ("an edit that touches no module in the
    // compared population ... adjudicates nothing and installs nothing") was a hard refusal in the
    // host. The model said one thing and the realization did another, which is the fork this whole
    // file exists to catch, sitting inside it.
    //
    // The repair is not to relax those three refusals. They are right: a digest over nothing is
    // evidence of nothing, and for a whole-population round an empty population means the tree is
    // broken. The repair is for the scoped round not to ask them a question they should refuse.
    //
    // ENDING BEFORE THE EMIT IS THE POINT, not an optimization detail. An edit that can change no
    // mirror has no reason to pay the emit (163 s) or the rebuild (230 s) that follow, so this is
    // the one place in the round where the affected-set bound removes work proportional to the
    // corpus rather than to the change. The whole-population arm cannot reach it -- an empty
    // committed population there is `EmptyCommittedPopulation`, a different refusal about a
    // different subject.
    if selected.is_empty() && !matches!(scope, RegenEmissionScope::WholePopulation) {
        let receipt = RegenReceipt::NoAffectedMirrors {
            schema: RECEIPT_SCHEMA.to_string(),
            commit_sha: git_head_sha(&workspace)?,
            authority_digest,
            scope: scope.line(),
        };
        write_receipt(&receipt_path, &receipt)?;
        eprintln!(
            "required-regen: the affected-set bound selects no compared mirror for this edit; \
             nothing adjudicated, nothing installed, no emit run ({})",
            scope.line()
        );
        return Ok(RequiredRegenOutcome {
            receipt,
            failures: Vec::new(),
        });
    }

    // PRODUCTION, THEN ADJUDICATION -- and the order is the whole repair.
    //
    // Every refusal below used to return BEFORE the candidate tree was written, so the refusing
    // run destroyed the artifact needed to close it. The population arm is real: adding a module
    // to the v1 seed closure emits a mirror the committed tree lacks -- `emitted_not_committed` by
    // construction on that module's first commit -- and this tree is the author's only route to
    // the file they are told to commit.
    //
    // THIS IS WHY THE SHARED MEASUREMENT IS SPLIT IN TWO. The extraction main landed is right --
    // one producer of the drift fact for the drift gate, the behavioural receipt and this path --
    // but it fused emit with adjudication, so it could only refuse before anything was written.
    // `emit_generated_surface` and `adjudicate_generated_surface` are the halves;
    // `measure_generated_surface` is still their composition and the single entry for every caller
    // wanting the whole answer, so nothing gained a second producer. This path alone acts BETWEEN.
    //
    // Writing first is not a relaxation: the gate refuses the same populations with the same typed
    // causes; the emitter's product merely survives the refusal, being emit's output and not a
    // reward for agreeing with the committed tree. Authority for the ordering:
    // `v2.workflow.required_regen` `required_regen_run`, whose verdict arms cannot be spelled
    // without the tree they judged.
    // THE EMISSION IS DENOMINATED IN THE CHANGE. `selected` is the intersection of the
    // affected-set bound with the committed roster -- the exact derived closure of this edit, not
    // "the module that changed" -- and it is what the emitter renders. Everything the emitter
    // declares about the whole population (the crate module list, the manifest, the closure-stub
    // decision) is derived from paths and from the resolved graph, so it is unchanged by this.
    let render_selection = Rc::new(match scope {
        RegenEmissionScope::WholePopulation => RustModuleRenderSelection::RenderEveryModule,
        _ => RustModuleRenderSelection::RenderSelectedMirrors {
            basenames: Rc::new(selected.iter().cloned().collect()),
        },
    });
    let (emitted, emitted_basenames) = match emit_generated_surface(&sources, &render_selection)? {
        // EMIT PRODUCED NOTHING IS NOT A VERDICT ABOUT A TREE. A receipt here would name a
        // `candidate_artifact` no pass wrote -- the impersonation the receipt split ends, one field
        // over. `CandidateTreeUnproduced` in `v2.workflow.required_regen` is the modeled arm and
        // carries no tree; the host spelling of no tree and no verdict is a refusal of the run.
        GeneratedSurfaceEmit::EmitRefused { reason } => {
            return Err(format!(
                "{reason} — no candidate tree produced, nothing to compare"
            ));
        }
        GeneratedSurfaceEmit::Emitted {
            emitted,
            emitted_basenames,
        } => (emitted, emitted_basenames),
    };

    if candidate_dir.exists() {
        fs::remove_dir_all(&candidate_dir)
            .map_err(|e| format!("remove {}: {e}", candidate_dir.display()))?;
    }
    let fresh_src = candidate_dir.join("src");
    v1_rt::trace_mark("regen.mirror_write.begin".to_string());
    let restrict = match scope {
        RegenEmissionScope::WholePopulation => None,
        _ => Some(&selected),
    };
    write_emitted_tree(&formatter, &fresh_src, &emitted, restrict)?;
    copy_hand_maintained_support(&stage0_src, &fresh_src)?;
    v1_rt::trace_mark("regen.mirror_write.done".to_string());
    // Verified against what EMIT produced, not what is committed: the two populations are equal on
    // a clean tree and differ in precisely the case this ordering serves, so checking the committed
    // population here would re-impose the refusal one line after the write. A producer owes only
    // that its own product landed whole.
    v1_rt::trace_mark("regen.candidate_verify.begin".to_string());
    // Verified against what THIS round wrote: the whole emitted population when unscoped, the
    // selection when scoped. Checking the emitted roster under a scope would refuse the round for
    // the absence of files it deliberately did not write.
    let written: Vec<String> = match scope {
        RegenEmissionScope::WholePopulation => emitted_basenames.clone(),
        _ => selected.iter().cloned().collect(),
    };
    verify_candidate_tree(&fresh_src, &written)?;
    v1_rt::trace_mark("regen.candidate_verify.done".to_string());

    v1_rt::trace_mark("regen.adjudicate.begin".to_string());
    let adjudicated =
        adjudicate_generated_surface(&formatter, &stage0_src, &emitted, &emitted_basenames, scope)?;
    v1_rt::trace_mark("regen.adjudicate.done".to_string());
    let (committed_basenames, selected_basenames, sync) = match adjudicated {
        GeneratedSurfaceAdjudicated::Refused { reason } => {
            return regen_refusal_outcome(
                &workspace,
                candidate_dir_rel,
                receipt_rel,
                commit_sha,
                authority_digest,
                format!(
                    "{reason} — the produced candidate tree is at {}",
                    fresh_src.display()
                ),
            );
        }
        GeneratedSurfaceAdjudicated::Measured {
            committed,
            selected,
            sync,
        } => (committed, selected, sync),
    };

    v1_rt::trace_mark("regen.hand_verify.begin".to_string());
    let hand = verify_hand_maintained(&formatter, &emitted, &stage0_src, &candidate_dir)?;
    v1_rt::trace_mark("regen.hand_verify.done".to_string());

    v1_rt::trace_mark("regen.digest.begin".to_string());
    // BOTH DIGESTS ARE DENOMINATED IN THE SELECTION, and equal to the old whole-population
    // digests when unscoped, because the selection IS the whole population there.
    //
    // A digest over a scoped population is not comparable to one over another population, and
    // that is fail-closed rather than a hazard: the payload interleaves each member's NAME with
    // its content digest, so two different selections produce two different digests and a
    // fixed-point comparison across them goes red. It cannot quietly agree.
    let committed_digest =
        tree_digest_for_basenames(&formatter, &stage0_src, &selected_basenames, "committed")?;
    let candidate_digest = tree_digest_from_map(&formatter, &emitted, &selected_basenames)?;
    v1_rt::trace_mark("regen.digest.done".to_string());
    report_cgroup_memory_diagnostic("child.after_digest");

    let first_generation_equal = sync.matches && hand.unverifiable.is_empty();
    let changed_paths = sync.drifted_paths.clone();
    let convergence_roots = [workspace.join("dag"), workspace.join("src/v2")]
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let (
        basename_to_module,
        generation_modules,
        bootstrap_sources,
        bootstrap_products,
        seed_embedded_basenames,
        generated_product_roles,
    ) = convergence_surface_roles(&workspace, &convergence_roots)?;
    report_cgroup_memory_diagnostic("child.after_surface_roles");
    let producer_seed_digest = current_exe_digest()?;
    let candidate_tree_id = format!("{candidate_dir_rel}:{candidate_digest}");
    // This is the writer's own exact artifact population, including non-Rust products such as
    // the crate manifest.  It is deliberately distinct from `selected_basenames`, whose authority
    // is the compared generated-Rust population and therefore cannot name these products.
    let emitted_artifact_paths = emitted
        .keys()
        .map(|path| candidate_relative_emit_path(path))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let candidate_manifest = produce_candidate_manifest(
        &fresh_src,
        &selected_basenames,
        &emitted_artifact_paths,
        &basename_to_module,
        &producer_seed_digest,
        &candidate_tree_id,
    )?;

    // Every field here was measured by THIS pass against THIS tree.
    let receipt = RegenReceipt::FirstGeneration {
        schema: RECEIPT_SCHEMA.to_string(),
        commit_sha,
        authority_digest,
        committed_generated_digest: committed_digest,
        candidate_generated_digest: candidate_digest,
        first_generation_equal,
        changed_paths: changed_paths.clone(),
        candidate_artifact: candidate_dir_rel.to_string(),
        candidate_manifest,
        surface_roles: ConvergenceSurfaceRoles::from_planning((
            basename_to_module,
            generation_modules,
            bootstrap_sources,
            bootstrap_products,
            seed_embedded_basenames,
            generated_product_roles,
        ))?,
    };
    write_receipt(&receipt_path, &receipt)?;

    let mut failures = Vec::new();
    if !sync.matches {
        failures.push(format!(
            "generated surface drift: {}",
            changed_paths.join(", ")
        ));
    }
    for (name, reason) in &hand.unverifiable {
        failures.push(format!("hand unverifiable {name}: {reason}"));
    }
    for name in &hand.undeclared_divergent {
        failures.push(format!(
            "refusal: {name} is excluded from the generated-surface comparison but the emitter \
             produces it and its output DIVERGES from the committed file, and it is not declared \
             in EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES. Declare it as an \
             EmitterProducedDivergentRegistration row in \
             v2.compiler.self_host.stage0_crate_layout, with a reason and a restoration trigger, \
             or close the divergence"
        ));
    }
    for name in &hand.dissolved_declarations {
        failures.push(format!(
            "refusal: {name} is declared in EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES but the \
             emitter's output now MATCHES the committed file. The rung drop has dissolved: delete \
             its EmitterProducedDivergentRegistration row in \
             v2.compiler.self_host.stage0_crate_layout and regenerate"
        ));
    }
    for name in &hand.unproduced_declarations {
        failures.push(format!(
            "refusal: {name} is declared in EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES but is absent \
             from the emitted population, so there is nothing for the declaration to excuse. \
             Delete its EmitterProducedDivergentRegistration row in \
             v2.compiler.self_host.stage0_crate_layout and regenerate"
        ));
    }

    // `planned`/`executed` stay the WHOLE rosters: they are the population identity join's two
    // sides, and that join is never scoped. `adjudicated` is the scoped count beside them, so a
    // reader can see both without either standing in for the other.
    eprintln!(
        "required-regen: elapsed_ms={} first_generation_equal={} planned={} executed={} \
         adjudicated={} declared_divergent={} [{}]",
        run_started.elapsed().as_millis(),
        first_generation_equal,
        committed_basenames.len(),
        emitted_basenames.len(),
        selected_basenames.len(),
        hand.declared_divergent.len(),
        hand.declared_divergent.join(", ")
    );
    // WHY, PRINTED AT THE COMPARISON POINT. `first_generation_equal` is one bool on the line
    // above, and the round driver forwards `failures` only when the receipt is `Refused` (see
    // `round_failures` in the convergence loop), so on this `FirstGeneration` path a false bool
    // reached a reader with no reason attached: heal's own job log carried
    // `first_generation_equal=false` and nothing else, on a run whose summary was otherwise
    // identical to a run that printed `true`. The bool cannot tell "the generated surface moved"
    // from "a hand-maintained file could not be verified", which are different work for the
    // reader. These are the SAME strings the caller receives, printed from the one place that
    // builds them, so there is no second spelling of the drift.
    if !first_generation_equal {
        let reasons = if failures.is_empty() {
            // Cannot happen today -- the flag is false only when one of the loops above pushed a
            // reason. If that ever stops holding, the reader sees this instead of silence.
            "required-regen: first_generation_equal=false with no reason recorded".to_string()
        } else {
            failures
                .iter()
                .map(|failure| {
                    format!("required-regen: first_generation_equal=false because {failure}")
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        eprintln!("{reasons}");
    }

    report_cgroup_memory_diagnostic("child.end");
    Ok(RequiredRegenOutcome { receipt, failures })
}

/// The emit-and-compare sequence, performed ONCE and in ONE place.
///
/// An earlier revision had two producers of one fact — which mirrors drifted:
/// `measure_generated_drift` re-typed the five calls `run_required_regen` performs, and nothing
/// kept them in step. On record: #8618 repaired a defect INSIDE `compare_generated_surfaces` (the
/// committed side was normalized, so the comparison was `normalize(normalize(x))` against
/// `normalize(x)` — a false-positive drift with no reachable green). A repair in one copy leaves
/// the other answering the old way; copies agreeing on the day they are written is what makes the
/// duplication easy to leave.
///
/// The callers differ in FAILURE POLICY, not measurement: `run_required_regen` routes a refusal to
/// `regen_refusal_outcome`, which writes a receipt and returns `Ok` carrying failures; the drift
/// gate wants `Err`. So the refusal is returned as a value and each caller applies its policy —
/// one `match` at the call site, not a second copy of the five calls.
enum GeneratedSurfaceMeasured {
    /// The comparison was taken. `emitted` and `committed` are returned because the regen path
    /// needs them for the candidate tree and its digests; recomputing means emitting twice.
    Measured {
        emitted: HashMap<String, String>,
        committed: Vec<String>,
        /// Returned rather than recomputed by the caller: it is part of THIS measurement, and a
        /// caller deriving it again would be a second producer of the same fact one level down.
        emitted_basenames: Vec<String>,
        sync: SyncReport,
    },
    /// The comparison could NOT be taken. This is ignorance, never "no drift" — see the refusal
    /// note on `measure_generated_drift`.
    Refused { reason: String },
}

/// What the emitter PRODUCED, before anything has been compared to it.
///
/// Exists on its own because one caller must act between the halves: `run_required_regen` writes
/// the candidate tree here, so a later adjudication refusal leaves the author holding the mirror
/// it tells them to commit.
enum GeneratedSurfaceEmit {
    Emitted {
        emitted: HashMap<String, String>,
        emitted_basenames: Vec<String>,
    },
    /// Emit itself produced nothing. Distinct from a comparison that could not be taken: there is
    /// no candidate to have an opinion about, rather than an opinion that could not be formed.
    EmitRefused { reason: String },
}

/// The verdict half: what the committed tree says about a candidate that already exists.
enum GeneratedSurfaceAdjudicated {
    Measured {
        /// The WHOLE committed roster, which the population identity join ran over.
        committed: Vec<String>,
        /// The subset whose BYTES were compared — equal to `committed` under
        /// `RegenEmissionScope::WholePopulation`, and the affected-set bound's intersection with
        /// it otherwise. Both digests and the candidate write are denominated in this.
        selected: Vec<String>,
        sync: SyncReport,
    },
    /// Ignorance, never "no drift" -- same refusal semantics as `GeneratedSurfaceMeasured`.
    Refused { reason: String },
}

/// WHICH MIRRORS' BYTES THIS ROUND ADJUDICATES — the host side of
/// `v2.workflow.required_regen` `RegenEmissionScope`, whose note carries the reasoning this
/// realization must not restate.
///
/// The one thing worth repeating at the seam, because it is what the code below has to keep true:
/// a scope bounds which mirrors' BYTES are read, normalized, compared, digested and written. It
/// never bounds which mirrors EXIST — `validate_compared_populations` runs over the whole
/// emitted and committed rosters under every scope, so a newly emitted surface and a committed
/// mirror the emitter has stopped producing are found exactly as they were before.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegenEmissionScope {
    /// Every committed mirror. The required CI phase runs this and only this, and it is what
    /// establishes the fixed-point precondition a later scoped author round relies on.
    WholePopulation,
    /// The affected-set bound's members, by committed basename (mirrors plus declared bootstrap
    /// products). Over-approximate by construction; never under-approximate.
    Affected { members: Vec<String> },
    /// The bound refused: an edited path could not be named as a module. The round refuses with
    /// it. It does NOT widen to `WholePopulation` — see the model's note.
    Unlocatable { paths: Vec<String>, reason: String },
}

impl RegenEmissionScope {
    /// The receipt line, matching `regen_scope_line` in the model.
    pub fn line(&self) -> String {
        match self {
            RegenEmissionScope::WholePopulation => "regen-scope: WholePopulationScope".to_string(),
            RegenEmissionScope::Affected { members } => {
                format!("regen-scope: AffectedScope members={}", members.len())
            }
            RegenEmissionScope::Unlocatable { paths, reason } => format!(
                "regen-scope: ScopeUnlocatable paths={} reason={reason}",
                paths.len()
            ),
        }
    }
}

/// THE ONE PRODUCER OF THE SELECTION, read by the write, the comparison and both digests.
///
/// An affected scope selects by INTERSECTION with the committed population rather than by taking
/// the bound's list as the population: the bound names mirrors derived from the module graph and
/// the declared bootstrap edges, and a member the tree does not carry would otherwise enter the
/// compared set, where it reads as a missing file instead of as what it is.
///
/// The refusal arm returns `Err`, which is this file's spelling of "the round does not run".
fn scope_selection(
    scope: &RegenEmissionScope,
    committed: &[String],
) -> Result<Vec<String>, String> {
    match scope {
        RegenEmissionScope::WholePopulation => Ok(committed.to_vec()),
        RegenEmissionScope::Unlocatable { paths, reason } => Err(format!(
            "refusal: the affected-set bound could not locate {} edited path(s) as modules, so \
             this round has no selection and does not widen to the whole population: {reason} \
             [{}]",
            paths.len(),
            paths.join(", ")
        )),
        RegenEmissionScope::Affected { members } => {
            let member_set: BTreeSet<&str> = members.iter().map(String::as_str).collect();
            Ok(committed
                .iter()
                .filter(|name| member_set.contains(name.as_str()))
                .cloned()
                .collect())
        }
    }
}

/// `selection` bounds which modules are RENDERED, and nothing else. The emitted-population
/// roster below is read from the manifest, which the emitter derives from paths, so it stays
/// whole under every selection -- see `generated_basenames_from_emit`.
fn emit_generated_surface(
    sources: &[(String, String)],
    selection: &Rc<RustModuleRenderSelection>,
) -> Result<GeneratedSurfaceEmit, String> {
    let emitted = compile_stage0(sources, selection)?;
    if emitted.is_empty() {
        return Ok(GeneratedSurfaceEmit::EmitRefused {
            reason: "refusal: emit produced zero files".to_string(),
        });
    }
    let emitted_basenames = generated_basenames_from_emit(&emitted)?;
    Ok(GeneratedSurfaceEmit::Emitted {
        emitted,
        emitted_basenames,
    })
}

fn adjudicate_generated_surface(
    formatter: &ResolvedFormatter,
    stage0_src: &Path,
    emitted: &HashMap<String, String>,
    emitted_basenames: &[String],
    scope: &RegenEmissionScope,
) -> Result<GeneratedSurfaceAdjudicated, String> {
    let committed = committed_generated_basenames(stage0_src)?;
    let hand_dir_shadows = hand_maintained_dir_shadows(stage0_src)?;
    // POPULATION IDENTITY IS NEVER SCOPED. Both rosters, whole, under every scope: this is the
    // join that finds a surface the emitter produces and the tree does not carry, and a mirror
    // the tree carries and the emitter no longer produces. It reads no bytes, so there is
    // nothing here for a selection to save and everything for one to hide.
    if let Some(reason) = validate_compared_populations(
        &committed,
        emitted_basenames,
        &hand_dir_shadows,
        SEED_RETENTION_FRONTIER_TOP_LEVEL_SRC_BASENAMES,
    ) {
        return Ok(GeneratedSurfaceAdjudicated::Refused { reason });
    }
    // BYTE ADJUDICATION IS SCOPED. This is the expensive half — a read, a normalization and a
    // comparison per member — and it is the half the affected-set bound is a bound ON.
    let selected = scope_selection(scope, &committed)?;
    let sync = compare_generated_surfaces(formatter, stage0_src, emitted, &selected)?;
    Ok(GeneratedSurfaceAdjudicated::Measured {
        committed,
        selected,
        sync,
    })
}

/// THE COMPOSITION, and still the single entry for every caller that wants the whole answer.
///
/// The split created no second producer: emit happens in one place, adjudication in one, and this
/// is their sequence. The drift gate and behavioural receipt call it unchanged; only the regen
/// path, which writes the candidate BETWEEN them, reaches for the halves.
fn measure_generated_surface(
    formatter: &ResolvedFormatter,
    sources: &[(String, String)],
    stage0_src: &Path,
) -> Result<GeneratedSurfaceMeasured, String> {
    let (emitted, emitted_basenames) = match emit_generated_surface(
        sources,
        &Rc::new(RustModuleRenderSelection::RenderEveryModule),
    )? {
        GeneratedSurfaceEmit::EmitRefused { reason } => {
            return Ok(GeneratedSurfaceMeasured::Refused { reason })
        }
        GeneratedSurfaceEmit::Emitted {
            emitted,
            emitted_basenames,
        } => (emitted, emitted_basenames),
    };
    // THE DRIFT GATE IS NEVER SCOPED. Its callers are the required CI phase and the behavioural
    // receipt, and a scoped answer there would be a gate that stopped looking at most of the tree.
    // The selection exists for the AUTHOR'S round, over a tree this gate has already verified.
    match adjudicate_generated_surface(
        formatter,
        stage0_src,
        &emitted,
        &emitted_basenames,
        &RegenEmissionScope::WholePopulation,
    )? {
        GeneratedSurfaceAdjudicated::Refused { reason } => {
            Ok(GeneratedSurfaceMeasured::Refused { reason })
        }
        GeneratedSurfaceAdjudicated::Measured {
            committed,
            selected: _,
            sync,
        } => Ok(GeneratedSurfaceMeasured::Measured {
            emitted,
            committed,
            emitted_basenames,
            sync,
        }),
    }
}

/// The emitted generated surface, keyed by basename.
///
/// Routed through the SAME `measure_generated_surface` the drift gate and regen path use, so the
/// bytes a behavioural receipt compiles are the bytes the drift gate compared; a second emit here
/// would be a second producer of the candidate itself.
pub fn emitted_generated_sources() -> Result<HashMap<String, String>, String> {
    let workspace = workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");
    // A THIRD ADMISSION SITE, FOUND BY THE PARAMETER. This entry serves the behavioural receipt,
    // three calls above the rustfmt spawn, so reading for "who spawns rustfmt" would not name it.
    // Threading the resolved formatter as an argument made the omission a compile error instead
    // of a fourth way to discover an absent formatter fifty minutes in.
    let formatter = ResolvedFormatter::admit()?.with_normalize_cache(&workspace)?;
    let sources = super::regen_input_sources(&workspace)?;
    let emitted = match measure_generated_surface(&formatter, &sources, &stage0_src)? {
        GeneratedSurfaceMeasured::Refused { reason } => return Err(reason),
        GeneratedSurfaceMeasured::Measured { emitted, .. } => emitted,
    };
    // KEYED BY BASENAME, and the conversion happens HERE rather than at the call site.
    //
    // Emit keys carry a `src/` prefix; everything joining against a committed mirror keys on
    // `file_name()`. `generated_basenames_from_emit` already warns that comparing the two key
    // spaces "made every file mismatch in both directions" -- and a caller walked into it anyway,
    // looking up `std_pareto.rs` in a map keyed by emit path and getting nothing. It refused
    // rather than reporting equivalence, but about the key space, not the candidate. Returning the
    // raw map invites that from every caller; deriving once through the same `emit_path_basename`
    // the population census uses removes it.
    let mut out: HashMap<String, String> = HashMap::new();
    for (path, content) in emitted {
        if !path.ends_with(".rs") || is_hand_maintained_path(&path) {
            continue;
        }
        let base = emit_path_basename(&path).to_string();
        // A collision would silently drop one candidate and compare the wrong bytes. The flat
        // generated surface makes basenames unique; a duplicate means that assumption broke.
        if let Some(prior) = out.insert(base.clone(), content) {
            let _ = prior;
            return Err(format!(
                "refusal: two emitted paths share the basename {base}; the generated surface \
                 is no longer flat and a basename join would compare the wrong candidate"
            ));
        }
    }
    Ok(out)
}

struct SyncReport {
    matches: bool,
    drifted_paths: Vec<String>,
}

struct HandVerifyReport {
    unverifiable: Vec<(String, String)>,
    /// Declared in `EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES` and measured divergent: the rung
    /// drop holding as declared. Reported and counted, never a failure -- a suppression nobody
    /// counts has frequency zero by construction and never ranks for repair.
    declared_divergent: Vec<String>,
    /// Emitted, divergent, and NOT declared. A failure: a new divergence cannot be hidden by
    /// adding a basename to the exclusion list, because the exclusion list is not the authority
    /// on what may diverge.
    undeclared_divergent: Vec<String>,
    /// Declared divergent but measured IDENTICAL. A failure, and the arm that makes each row's
    /// restoration trigger executable rather than prose: the moment the emitter can produce the
    /// committed bytes, the line stops until the row is deleted.
    dissolved_declarations: Vec<String>,
    /// Declared divergent but absent from the emitted population entirely. A failure: a row
    /// cannot outlive the producer whose output it excuses, or it silently becomes an ordinary
    /// unexplained exclusion wearing a rung-drop's name.
    unproduced_declarations: Vec<String>,
}

fn is_declared_divergent(file_name: &str) -> bool {
    EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES.contains(&file_name)
}

/// ONE READ OF THE CLOSURE, SUPPLIED BY THE CALLER. This used to call `regen_input_sources`
/// itself, so every regen read the corpus twice -- authority digest and here -- and the receipt
/// priced the second walk at a quarter of the first phase (`regen.corpus_reload`, measured
/// 2026-08-30: 24 s on srv1, 17 s on BuildBuddy). The caller reads once; both consumers share it.
fn compile_stage0(
    sources: &[(String, String)],
    selection: &Rc<RustModuleRenderSelection>,
) -> Result<HashMap<String, String>, String> {
    // This is the one emission whose population IS the stage0 seed, so the host shell's module
    // roster is read from its lib.rs and the package graph from the emitted tree's own partition.
    let declared_shell_modules = super::emitted_closure_compile_host::closure_modules(
        &workspace_root().join("src/v1/stage0/src/lib.rs"),
    )?;
    compile_stage0_admitted(
        sources,
        selection,
        Rc::new(declared_shell_modules.into_iter().collect()),
        emitted_tree_packages(&workspace_root())?,
    )
}

/// THE EMISSION AND ITS EDGE ADMISSION, OVER A SUPPLIED PACKAGE GRAPH. `compile_stage0` supplies
/// the seed tree's host shell and partition; a witness over a fixture population supplies the rows
/// that population is built under, rather than being judged against the seed's (DESIGN §3, a
/// witness supplies its inputs). The admission itself is not relaxed for either.
fn compile_stage0_admitted(
    sources: &[(String, String)],
    selection: &Rc<RustModuleRenderSelection>,
    host_shell_modules: Rc<im::Vector<String>>,
    packages: Rc<crate::gunbc_stage0_stage0_emitted_edge_admission::Stage0EmittedTreePackages>,
) -> Result<HashMap<String, String>, String> {
    let source_files: Vec<Rc<SourceFile>> = sources
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect();
    let result = compile_sources_selected(
        Rc::new(source_files.into()),
        RenderTarget::Rust,
        selection.clone(),
    );
    if let Some(message) =
        stage0_self_compile_refusal_message("v2 self-compile".to_string(), result.clone())
    {
        return Err(message);
    }
    // EVERY EDGE THE EMITTER WROTE MUST RESOLVE IN THE CRATE ITS MODULE LANDS IN. The emitter
    // returns its use-line and prelude edges beside the text (`gunbc.rust_emitted_edge`), and
    // `gunbc.stage0.stage0_emitted_edge_admission` maps both ends through the supplied partition rows
    // and package graph; a module in neither the rows nor the shell refuses as not covered rather
    // than passing.
    let admission =
        crate::gunbc_stage0_stage0_emitted_edge_admission::stage0_emitted_edge_admission(
            result.emitted_edges.clone(),
            host_shell_modules,
            packages,
        );
    if let crate::gunbc_stage0_stage0_emitted_edge_admission::Stage0EmittedEdgeAdmission::Stage0EmittedEdgesAdmitted {
        edge_count,
    } = &*admission
    {
        eprintln!("required-regen: emitted-edge admission admitted edges={edge_count}");
    }
    if let Some(message) =
        crate::gunbc_stage0_stage0_emitted_edge_admission::stage0_emitted_edge_admission_refusal(
            admission,
        )
    {
        return Err(message);
    }
    let mut out = HashMap::new();
    for file in result.files.iter() {
        out.insert(file.path.clone(), file.content.clone());
    }
    Ok(out)
}

/// THE PACKAGE GRAPH THE EMITTED TREE WILL BE BUILT UNDER, READ FROM THAT TREE: its partition rows
/// and the host shell node (`gunbc.stage0.stage0_executable_assembly_generated`) above them.
///
/// The admission above maps each emitted edge through partition rows. The rows it must use are the
/// ones this emission's own corpus declares -- `gunbc.stage0.stage0_crate_partition_generated`
/// `generated_partition_crate_rows`, read from the same source roots `regen_input_sources` walks --
/// because those are the rows the candidate mirror is rendered from and the crate layout it will be
/// built under. The rows compiled into THIS binary are the predecessor's: reading them admitted a
/// candidate against the partition it was replacing, so a change adding a seed module refused
/// NotCovered until a regen had already installed the row it was refusing to produce (#12171's
/// bootstrap deadlock, broken on #12185 only by hand-seeding the mirror). An unreadable or
/// ill-shaped row set refuses; it never falls back to the compiled rows.
fn emitted_tree_packages(
    workspace: &Path,
) -> Result<Rc<crate::gunbc_stage0_stage0_emitted_edge_admission::Stage0EmittedTreePackages>, String>
{
    use crate::gunbc_stage0_stage0_crate_partition_generated::GeneratedPartitionCrateRow;
    use crate::gunbc_stage0_stage0_emitted_edge_admission::Stage0EmittedTreePackages;
    use crate::v1_interpreter::{self, ExecutionMode};
    let roots: Vec<String> = super::regen_source_roots()
        .all()
        .iter()
        .map(|root| {
            workspace
                .join(root.repo_relative_path())
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let entry = roots
        .iter()
        .map(|root| Path::new(root).join("gunbc/stage0/stage0_emitted_edge_admission.dag"))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            format!(
                "refusal: gunbc.stage0.stage0_emitted_edge_admission is under no regen source root \
                 {roots:?}, so the emitted tree's package graph cannot be read"
            )
        })?;
    // ONE ROUTE, THE CHEAP ONE. This read resolves ONE entry, and it used to build the whole
    // regen tree's index to do it -- `build_multi_entry_index`, the route this file measures at
    // 169 s and 10.13 GB for a 26-module closure (see `resolve_orchestrator_model`), paid inside
    // the orchestrator between generations. That is what `GenerationStateRetainedByOrchestrator`
    // caught: the peak is the orchestrator's own, so the ~8 GB stayed resident after the read
    // ended and the next generation was refused. Every other orchestrator-side model load goes
    // through `resolve_orchestrator_model` and pays ~150 MB; this was the one site that forked to
    // the whole-tree route (DESIGN section 6b: a cost's costliest route avoided by one caller and
    // paid by another is a fork, and the rule is one route at the owed cost). The child that
    // answers the partition-move question took the same route and exited -- "process exit frees
    // the index" -- which is why the round peaked in a child while the orchestrator alone was
    // charged for the memory.
    report_cgroup_memory_diagnostic("parent.emitted_tree.resolve.begin");
    let (graph, indices) = resolve_orchestrator_model(&roots, &entry.to_string_lossy())
        .map_err(|e| format!("refusal: the emitted tree's package graph did not resolve: {e}"))?;
    report_cgroup_memory_diagnostic("parent.emitted_tree.resolve.end");
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    // DECODE IS NOT HAND-WRITTEN: `Value` -> `value_to_wire_json` -> `serde_json::from_value` into
    // the mirror types, the decode authority `namespace_baseline` `decode_environment_value` uses.
    // The wire encoder's tag policy and the mirror's `#[serde(...)]` attributes come from the same
    // emitter, so the row shape and the kind variants are not restated here, and a row or kind this
    // binary's mirror cannot represent refuses as undecodable rather than being guessed.
    fn decode<T: serde::de::DeserializeOwned>(
        ctx: &v1_interpreter::InterpContext,
        name: &str,
    ) -> Result<T, String> {
        let value = v1_interpreter::with_active_context(ctx, || {
            v1_interpreter::run_in_context_with_args(ctx, name, &[], false)
        })
        .map_err(|e| format!("refusal: {name} did not evaluate: {e}"))?;
        let wire = super::value_to_wire_json(&value, ctx)
            .map_err(|e| format!("refusal: {name} did not wire-encode: {e}"))?;
        serde_json::from_value(wire).map_err(|e| format!("refusal: {name} did not decode: {e}"))
    }
    let rows: im::Vector<Rc<GeneratedPartitionCrateRow>> =
        decode(&ctx, "generated_partition_crate_rows")?;
    report_cgroup_memory_diagnostic("parent.emitted_tree.decode.rows");
    let host_shell_package_name: String = decode(&ctx, "generated_host_shell_package_name")?;
    let host_shell_dependencies: im::Vector<String> =
        decode(&ctx, "generated_host_shell_partition_dependencies")?;
    report_cgroup_memory_diagnostic("parent.emitted_tree.decode.end");
    Ok(Rc::new(Stage0EmittedTreePackages {
        rows: Rc::new(rows),
        host_shell_package_name,
        host_shell_dependencies: Rc::new(host_shell_dependencies),
    }))
}

// ONE AUTHORITY FOR "WHAT THE REGEN COMPARES", read from both sides.
//
// `generated_basenames_from_emit` and `committed_generated_basenames` answer the same question
// about two populations -- the emit and the committed tree -- and each carried its own copy of the
// membership rule: two readers of one fact, inside the very file whose job is to detect that
// class. They now share this predicate, so a file cannot be compared on one side and skipped on
// the other.
//
// THE EMITTED-POPULATION MANIFEST NEEDS NO EXCEPTION HERE, BY CONSTRUCTION. `emitted_population.rs`
// (v1.compiler.emit_rust, `emit_emitted_population_manifest`) is the emitter's declaration of what
// it produced, emitted as a `.rs` of comment lines precisely so this rule admits it: it enters the
// compared population, drift comparison, tree digest and candidate verification with nothing
// added. An earlier revision made it a `.txt` named here as an exception; execution refused it --
// every compared member is rustfmt-normalized before its digest, so the `.txt` failed to parse as
// Rust ("expected one of `!` or `::`, found `.`"). Rightly: the compared population is Rust-shaped
// end to end, and an artifact wanting its gating has to be Rust.
fn is_compared_generated_basename(basename: &str) -> bool {
    basename.ends_with(".rs")
}

/// WHAT MAY BE WRITTEN INTO THE COMMITTED SEED, ASKED AT THE MUTATION BOUNDARY.
///
/// The installer used to ask nothing. It copied `candidate_src/<basename>` to
/// `stage0_src/<basename>` for whatever roster it was handed, and the only thing keeping a
/// non-Rust artifact out of that roster was `is_compared_generated_basename` — a predicate
/// upstream, in a different function, answering a DIFFERENT question ("what does the drift
/// comparison denominate?"). Install admissibility had no authority of its own; it was a
/// consequence of the comparator's denominator, which is exactly the shape DESIGN section 3
/// forbids: one fact with no home, inferred from another fact that is free to move.
///
/// THIS GUARD DELIBERATELY DOES NOT ASK WHETHER THE ARTIFACT IS RUST, and the omission is the
/// considered part. An earlier revision refused every non-`.rs` install target on the reasoning
/// that non-Rust emitted artifacts are not installable into the seed. That reasoning is wrong in
/// the direction that matters: the emitted `Cargo.toml` is stage0's OWN package manifest emitted
/// incompletely (17 lines carrying `[package] name = "v1_compiler"` against the committed
/// 172-line `v1-compiler` manifest), standing in the same relation to its committed file as the
/// emitted `main.rs` does. `v2.compiler.self_host.stage0_crate_layout`
/// `emitter_produced_divergent_note` settles that relation: the reachable end state is that the
/// emitter produces the committed bytes and the divergent family is EMPTY. So an extension arm
/// would refuse the correct end state by construction, writing an accidental comparison
/// denominator into a second place as deliberate policy — the dual of widening the compared
/// population, and the same error.
///
/// What keeps a non-Rust artifact out of the install roster is therefore left where it already
/// is, and the deficit it rests on is named rather than re-implemented here: destinations are
/// addressed as BARE BASENAMES under `stage0_src`, `produce_candidate_manifest` has already
/// discarded the package-root distinction, and so a declared non-Rust `GeneratedSurface` is
/// invisible to the fixed point instead of dispositioned. That is the subject of the projection
/// identity work, not of this boundary.
///
/// So the boundary asks only what it can answer on its own, and refuses rather than widening
/// (DESIGN section 5) — both arms kind-agnostic:
///
/// - a path that is not a bare basename cannot address anything outside `stage0_src`;
/// - a hand-maintained mirror is authored, never installed — the remedy for an emitted/committed
///   divergence there is `verify_hand_maintained`'s declared-divergence roster, not a copy over
///   the author's bytes.
///
/// REFUSES NOTHING TODAY, BY MEASUREMENT AND BY CONSTRUCTION. Every install roster is a subset
/// of `drifted`, which is computed by `compare_generated_surfaces` over
/// `committed_generated_basenames`/`generated_basenames_from_emit` — both already `.rs`-only and
/// both already hand-maintained-excluded. The live corpus is this wall's positive control; its
/// discriminating RED is authored in
/// `install_admission_refuses_unaddressable_and_hand_maintained`.
///
/// RUNG, HONESTLY: mechanically preventable. The invalid state stays writable — the plan
/// projection hands this function a `Vec<String>`, so a non-installable path is expressible right
/// up to the call — and safety depends on this admission executing. The attainable ceiling is
/// structural impossibility: membership is decidable and fully modeled, so the state has no
/// constructor once `v2.workflow.regen_convergence_transaction` `regen_stage_plan_surface_paths`
/// projects a typed surface identity (declaring module + projected path, as the same module's
/// `RegenSurfaceIdentity` already carries for the candidate manifest) instead of a
/// `List<String>`, and the installer takes that type rather than `&[String]`. That
/// projection is the next-rung trigger; this admission dissolves with it, while the RED below
/// stays enrolled as the regression control the climb does not retire (DESIGN 4b(4)).
fn admit_install_target(basename: &str) -> Result<(), String> {
    if basename.is_empty() || emit_path_basename(basename) != basename {
        return Err(format!(
            "InstallTargetNotABareBasename: {basename} is not a bare stage0 basename, so the \
             install destination is not inside the committed generated surface"
        ));
    }
    if HAND_MAINTAINED_STAGE0_FILES.contains(&basename) {
        return Err(format!(
            "InstallTargetHandMaintained: {basename} is a hand-maintained stage0 mirror. An \
             emitted/committed divergence there is adjudicated by verify_hand_maintained, never \
             by installing over the authored bytes"
        ));
    }
    Ok(())
}

/// THE NAME IS NOT THE DESTINATION: the lexical admission above proves only that the basename
/// SPELLS a bare stage0 entry, never where that name RESOLVES. Git tracks symlinks (mode 120000),
/// so a committed stage0 entry can be one, and `fs::copy` follows the DESTINATION link -- the
/// bytes land wherever it points, outside the committed surface, with the lexical check fully
/// satisfied and nothing refused. That indirection is the second route to the harm the failure-mode
/// row under-enumerated: it reaches a path outside `stage0_src` without the join ever changing.
///
/// RUNG, HONESTLY: this is a BOUNDARY OBSERVATION, not a construction, and deliberately not on the
/// ladder (DESIGN 4b, "outside the modeled guarantee"). The filesystem is external reality that no
/// modeled type can make impossible, so the only honest arms are LOOK and REFUSE. It refuses on
/// unreadable as well as on symlink: an unreadable destination is ignorance about containment, and
/// answering ignorance with "proceed" is the absorbing fallback DESIGN 5 forbids.
fn admit_install_destination(stage0_src: &Path, basename: &str) -> Result<(), String> {
    let destination = stage0_src.join(basename);
    // symlink_metadata, never metadata: metadata FOLLOWS the link and would report the target's
    // kind, so the one question being asked would be answered about the wrong file.
    match fs::symlink_metadata(&destination) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "InstallDestinationNotARegularFile: {basename} is a symlink. fs::copy follows the \
             destination link, so the installed bytes would land outside the committed generated \
             surface"
        )),
        Ok(metadata) if !metadata.file_type().is_file() => Err(format!(
            "InstallDestinationNotARegularFile: {basename} exists and is not a regular file, so \
             the install destination is not a committed generated artifact"
        )),
        Ok(_) => Ok(()),
        // A first-time emitted artifact legitimately has no committed destination yet. A DANGLING
        // symlink is not this arm: symlink_metadata reports it Ok and is_symlink, so it refuses
        // above rather than reading as absent here.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "InstallDestinationUnreadable: {basename}: {e}. Containment is unknown, and unknown \
             containment refuses rather than proceeds"
        )),
    }
}

/// THE EMITTED ROSTER IS READ FROM THE EMITTER'S DECLARATION, NOT FROM WHAT IT HANDED BACK.
///
/// This used to walk the keys of the returned map -- the files this emit RENDERED. That was the
/// same population under the only emission that existed, and it stops being so the moment an
/// emission is denominated in a change: under `RenderSelectedMirrors` the keys ARE the selection,
/// so every unselected mirror would read as `committed_not_emitted` and the population identity
/// join would be scoped by accident. That join is the one thing `v2.workflow.required_regen`
/// says may never be scoped, because it reads no bytes and so has nothing to save and everything
/// to hide.
///
/// `emit_emitted_population_manifest` (`v1.compiler.emit_rust`) declares every path the emit
/// produced, derived from PATHS rather than from rendered content, so it is total under every
/// selection and identical to the rendered set when there is none. Reading it here is the
/// structural inverse of that write, taking both literals from the one authority the writer reads
/// (`gunbc.stage0.stage0_emitted_population_manifest`); the same inverse is spelled in the model at
/// `gunbc.stage0.stage0_rust_host_observation` `emitted_population_paths_from_manifest`, over the
/// committed artifact rather than an in-memory emission, and in `cssl_seed_linked_closure_assembly`
/// `declared_emitted_paths`, over a file on disk. Neither is reachable from here: the first is not
/// in the seed closure and the second lives in a separate binary's private module.
///
/// A manifest the emit did not produce, or one carrying no declared line, REFUSES. It cannot be
/// silently replaced by the rendered keys: that fallback is exactly the widening this function
/// exists to remove, and it would be invisible because the two agree whenever the selection is
/// whole.
fn generated_basenames_from_emit(emitted: &HashMap<String, String>) -> Result<Vec<String>, String> {
    let manifest_key = format!("src/{}", emitted_population_manifest_basename());
    let manifest = emitted.get(&manifest_key).ok_or_else(|| {
        format!(
            "refusal: emit declared no population -- {manifest_key} is absent from the emitted              files, so the roster the population identity join needs does not exist"
        )
    })?;
    let prefix = emitted_population_manifest_line_prefix();
    let separator = emitted_population_manifest_line_separator();
    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut declared = 0usize;
    for line in manifest.split(separator.as_str()) {
        let Some(path) = line.strip_prefix(prefix.as_str()) else {
            continue;
        };
        declared += 1;
        // Basename, not the declared path: `committed_generated_basenames` keys on
        // `file_name()`, and declared paths carry a `src/` prefix. Comparing the two
        // key spaces made every file mismatch in both directions.
        if is_compared_generated_basename(emit_path_basename(path))
            && !is_hand_maintained_path(path)
        {
            names.insert(emit_path_basename(path).to_string());
        }
    }
    if declared == 0 {
        return Err(format!(
            "refusal: {manifest_key} carries no declared path line -- the emitted population              cannot be read from it"
        ));
    }
    Ok(names.into_iter().collect())
}

// Emit keys are the target-relative artifact path (e.g. "src/cli_run.rs" for
// every Rust module — see rust_source_root()); committed-tree comparisons key
// on the bare basename. This is the single place that bridges the two, so
// every consumer below compares/looks up on equal footing instead of each
// re-deriving its own normalization (or, as before, silently comparing
// "src/x.rs" against "x.rs" as unequal strings for the whole corpus).
fn emit_path_basename(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
}

fn lookup_emitted<'a>(emitted: &'a HashMap<String, String>, basename: &str) -> Option<&'a String> {
    emitted
        .get(&format!("src/{basename}"))
        .or_else(|| emitted.get(basename))
}

fn committed_generated_basenames(stage0_src: &Path) -> Result<Vec<String>, String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(stage0_src)
        .map_err(|e| format!("read committed stage0 src {}: {e}", stage0_src.display()))?
    {
        let entry = entry.map_err(|e| format!("read committed stage0 entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let basename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if is_compared_generated_basename(basename)
            && !HAND_MAINTAINED_STAGE0_FILES.contains(&basename)
        {
            names.insert(basename.to_string());
        }
    }
    if names.is_empty() {
        return Err("refusal: committed generated population is empty".to_string());
    }
    Ok(names.into_iter().collect())
}

/// The only seam where `HAND_MAINTAINED_STAGE0_DIRS` reaches the population comparison, and it
/// CLASSIFIES a refusal rather than excluding anything from one.
///
/// Both compared populations key on basename, and the committed walk enumerates only the stage0
/// crate's top level, so a file inside a hand-maintained DIRECTORY is invisible to it. An emitted
/// basename equal to one of those files lands in `emitted_not_committed`, whose two remedies --
/// install the produced mirror, or investigate an emitter that invented a surface -- BOTH destroy
/// hand-authored code (installing overwrites it, deleting removes it). This map names that
/// population under its own cause with the remedy that applies (de-collide the module name).
///
/// IT DOES NOT EXCLUDE. Teaching `is_hand_maintained_path` the directory list would drop the
/// colliding path from the compared population, silently ceasing to compare a genuinely generated
/// surface -- a wrong refusal traded for none. Same failure as making the committed walk descend,
/// from the other side.
///
/// ZERO EXPOSURE, so this is not read as repairing a live defect. Emitted Rust filenames are flat
/// by construction (`v1.compiler.emit_core_support` `module_to_filename` is split(".") joined with
/// "_" under `rust_source_root()`), so the emitter cannot write into a hand-maintained directory;
/// the collision needs a module whose bare name equals one of those files. Measured 2026-08-22:
/// zero collisions in the corpus, required-regen green on main at 90986d19469, and no upstream
/// guard refuses such a module name (the nearest, `gunbc.stage0.stage0_rust_source_lifecycle_scaffold`
/// `classified_residue_disjoint_holds` and the generated/hand disposition joins beside it, are
/// scoped to top-level stage0 paths and cannot see a subdirectory file).
fn hand_maintained_dir_shadows(stage0_src: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut shadows = BTreeMap::new();
    for dir_name in HAND_MAINTAINED_STAGE0_DIRS {
        let dir = stage0_src.join(dir_name);
        if !dir.is_dir() {
            continue;
        }
        collect_dir_shadows(&dir, dir_name, &mut shadows)?;
    }
    // Sorted so the refusal text is a function of the collision, not of the roster's authoring
    // order: two clones must print the same remedy for the same tree.
    for homes in shadows.values_mut() {
        homes.sort();
    }
    Ok(shadows)
}

fn collect_dir_shadows(
    dir: &Path,
    home: &str,
    shadows: &mut BTreeMap<String, Vec<String>>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("read dir {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| format!("read dir entry under {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir_shadows(&path, home, shadows)?;
            continue;
        }
        if let Some(basename) = path.file_name().and_then(|n| n.to_str()) {
            if is_compared_generated_basename(basename) {
                // EVERY HOME, NOT THE LAST ONE SEEN. `insert` here was last-write-wins, so one
                // basename hand-authored under two hand-maintained directories -- `mod.rs` being
                // the obvious candidate -- refused while naming only one of them, sending the
                // author to the wrong file. Authority for the list shape:
                // `v2.workflow.required_regen` `EmittedNotCommittedShadowsHandMaintained`.
                let homes = shadows.entry(basename.to_string()).or_default();
                if !homes.iter().any(|h| h == home) {
                    homes.push(home.to_string());
                }
            }
        }
    }
    Ok(())
}

fn validate_compared_populations(
    committed: &[String],
    emitted: &[String],
    hand_dir_shadows: &BTreeMap<String, Vec<String>>,
    seed_retained_top_level_src: &[&str],
) -> Option<String> {
    if committed.is_empty() {
        return Some("refusal: committed generated population is empty".to_string());
    }
    if emitted.is_empty() {
        return Some("refusal: emit produced zero generated surfaces".to_string());
    }
    let committed_set: BTreeSet<&str> = committed.iter().map(String::as_str).collect();
    let emitted_set: BTreeSet<&str> = emitted.iter().map(String::as_str).collect();
    let mut emitted_not_committed = Vec::new();
    let mut shadowing_hand_maintained = Vec::new();
    for name in emitted {
        if committed_set.contains(name.as_str()) {
            continue;
        }
        match hand_dir_shadows.get(name.as_str()) {
            Some(homes) => shadowing_hand_maintained.push(format!(
                "{name} (hand-maintained under {})",
                homes.join(", ")
            )),
            None => emitted_not_committed.push(name.clone()),
        }
    }
    // A committed basename the emitter no longer produces is CommittedMirrorNoLongerEmitted
    // unless the seed-retention frontier already names it: then it is retained, not lost.
    //
    // RUNG, HONESTLY: this skip excludes a presented retained basename. HAND_MAINTAINED_STAGE0_FILES
    // currently agrees with it for std_logic.rs by coincidence (#10712 listed the file as
    // hand-maintained), so committed_generated_basenames never presents std_logic.rs to this loop
    // on main or at 0a811fc -- the class is not reachable on the merge path. Discrimination is
    // the off-path #[test] pair below (rust_unit_tests_off_the_merge_path). Do not author a
    // reaching fixture to make the live walk present a retained file; that would manufacture a
    // subject. The nonempty frontier witness is not evidence for this join.
    let seed_retained: BTreeSet<&str> = seed_retained_top_level_src.iter().copied().collect();
    let mut committed_not_emitted = Vec::new();
    for name in committed {
        if !emitted_set.contains(name.as_str()) && !seed_retained.contains(name.as_str()) {
            committed_not_emitted.push(name.clone());
        }
    }
    // TWO OPPOSITE STATES, TWO REFUSALS, TWO REMEDIES -- this refused on their union, so a module
    // being introduced was indistinguishable from the emitter having LOST a surface. Authority for
    // the split: `v2.workflow.required_regen` `MirrorMissingForEmittedSurface` and
    // `CommittedMirrorNoLongerEmitted`.
    //
    // NEITHER IS ADMITTED, and the first is where that matters. "An author introduced a module"
    // and "the emitter invented a surface nobody authored" produce the SAME population, and the
    // second is what this check catches, so no arm computed from the populations can tell them
    // apart -- admitting the first would be the same conflation the other way. The refusal names
    // the fork and leaves the decision with the author; the install branch is now actionable
    // because the ordering above wrote the bytes before this check ran.
    let mut reasons = Vec::new();
    if !emitted_not_committed.is_empty() {
        reasons.push(format!(
            "refusal: emitted surface has no committed mirror — {emitted_not_committed:?}; if you introduced these modules, install the produced mirror(s) named below and commit them; if you did not, the emitter produced a surface nobody authored and installing it would launder that"
        ));
    }
    if !committed_not_emitted.is_empty() {
        reasons.push(format!(
            "refusal: committed mirror is no longer emitted — {committed_not_emitted:?}; the emitter stopped producing these surfaces, so either the authority that emitted them was removed on purpose (delete the committed mirror) or it regressed (restore it) — do NOT install anything for this class"
        ));
    }
    if !shadowing_hand_maintained.is_empty() {
        // A THIRD CAUSE BECAUSE THE OTHER TWO REMEDIES DAMAGE THIS CLASS. Authority:
        // `v2.workflow.required_regen` `MirrorMissingShadowsHandMaintainedSource`. Not softer --
        // the line still stops -- but pointed at the move that resolves the state.
        reasons.push(format!(
            "refusal: emitted surface collides with hand-maintained source — {shadowing_hand_maintained:?}; the emitted basename addresses a file that is hand-authored under a hand-maintained directory, so BOTH remedies for a missing mirror would damage it (installing overwrites hand-authored code with generated bytes; deleting destroys it). Rename the emitting module so its basename no longer collides, or move the hand-maintained file — do NOT install anything for this class"
        ));
    }
    if reasons.is_empty() {
        None
    } else {
        Some(reasons.join(" | "))
    }
}

fn verify_candidate_tree(
    candidate_src: &Path,
    expected_basenames: &[String],
) -> Result<(), String> {
    if expected_basenames.is_empty() {
        return Err(
            "refusal: cannot verify candidate tree against empty expected population".to_string(),
        );
    }
    if !candidate_src.is_dir() {
        return Err(format!(
            "refusal: candidate src directory absent at {}",
            candidate_src.display()
        ));
    }
    let mut found = 0usize;
    for basename in expected_basenames {
        if candidate_src.join(basename).is_file() {
            found += 1;
        }
    }
    if found == 0 {
        return Err(format!(
            "refusal: candidate tree has zero generated files under {}",
            candidate_src.display()
        ));
    }
    if found != expected_basenames.len() {
        return Err(format!(
            "refusal: candidate tree incomplete — found {found} of {} expected generated files under {}",
            expected_basenames.len(),
            candidate_src.display()
        ));
    }
    Ok(())
}

fn regen_refusal_outcome(
    workspace: &Path,
    candidate_dir_rel: &str,
    receipt_rel: &str,
    commit_sha: String,
    authority_digest: String,
    reason: String,
) -> Result<RequiredRegenOutcome, String> {
    let receipt_path = workspace.join(receipt_rel);
    // A population refusal happens BEFORE any content comparison, so nothing here is what a digest
    // or equality could be ABOUT. The previous shape wrote a `FirstGeneration` receipt with
    // `refused:population`, `refused:population` and `false` in the three unmeasured positions.
    // `Refused` has none of those fields, so the placeholders are unwritable and the receipt
    // carries the refusal's cause. The variant's comment traces the fabricated Bool's route off
    // this machine.
    let receipt = RegenReceipt::Refused {
        schema: RECEIPT_SCHEMA.to_string(),
        commit_sha,
        authority_digest,
        reason: reason.clone(),
        candidate_artifact: candidate_dir_rel.to_string(),
    };
    write_receipt(&receipt_path, &receipt)?;
    Ok(RequiredRegenOutcome {
        receipt,
        failures: vec![reason.clone()],
    })
}

fn is_hand_maintained_path(path: &str) -> bool {
    HAND_MAINTAINED_STAGE0_FILES.contains(&emit_path_basename(path))
}

fn compare_generated_surfaces(
    formatter: &ResolvedFormatter,
    stage0_src: &Path,
    emitted: &HashMap<String, String>,
    generated_basenames: &[String],
) -> Result<SyncReport, String> {
    let mut drifted = Vec::new();
    for basename in generated_basenames {
        let committed_path = stage0_src.join(basename);
        // The committed side is read RAW and compared against exactly the bytes
        // `write_emitted_tree` puts in the candidate tree -- `normalize_generated_source(emitted)`.
        // Normalizing the committed side too made the comparison `normalize(normalize(emitted))`
        // vs `normalize(emitted)` once a candidate was installed -- an identity only if rustfmt is
        // idempotent, and it is not: measured 2026-08-20, `v1_compiler_infer.rs` reformats on a
        // second pass (a `let ... = if (long_receiver_chain)` splits differently), so the fold
        // reported the same file as drifted at generation 2, 3 and 4 with the candidate on disk
        // BYTE-IDENTICAL to the committed file. No reachable green; the only silence was
        // hand-editing the mirror -- validation where construction was available (DESIGN 5).
        // Comparing against the written artifact makes "install the candidate" a guaranteed
        // remedy by construction, and the two derivations of the candidate one fact (DESIGN 3).
        let committed = fs::read_to_string(&committed_path)
            .map_err(|e| format!("read committed {}: {e}", committed_path.display()))?;
        let candidate = lookup_emitted(emitted, basename)
            .ok_or_else(|| format!("emit missing generated file {basename}"))?;
        let candidate_norm = normalize_generated_source(formatter, candidate)
            .map_err(|e| format!("normalize candidate {basename}: {e}"))?;
        if committed != candidate_norm {
            drifted.push(basename.clone());
        }
    }
    Ok(SyncReport {
        matches: drifted.is_empty(),
        drifted_paths: drifted,
    })
}

fn verify_hand_maintained(
    formatter: &ResolvedFormatter,
    emitted: &HashMap<String, String>,
    stage0_src: &Path,
    work_dir: &Path,
) -> Result<HandVerifyReport, String> {
    let mut unverifiable = Vec::new();
    let mut declared_divergent = Vec::new();
    let mut undeclared_divergent = Vec::new();
    let mut dissolved_declarations = Vec::new();
    let mut unproduced_declarations = Vec::new();
    for file_name in HAND_MAINTAINED_STAGE0_FILES {
        let candidate = emitted
            .get(&format!("src/{file_name}"))
            .or_else(|| emitted.get(*file_name));
        let Some(candidate) = candidate else {
            // Not in the emitted population. For 35 of the 36 entries (measured by execution
            // 2026-08-21) this is ordinary and there is nothing to compare. For a DECLARED row it
            // is a defect in the declaration, not the tree.
            if is_declared_divergent(file_name) {
                unproduced_declarations.push((*file_name).to_string());
            }
            continue;
        };
        let committed_path = stage0_src.join(file_name);
        let committed = fs::read_to_string(&committed_path)
            .map_err(|e| format!("read committed hand file {}: {e}", committed_path.display()))?;
        match normalize_with_workdir(formatter, &committed, work_dir, "committed") {
            Ok(committed_norm) => {
                match normalize_with_workdir(formatter, candidate, work_dir, "candidate") {
                    Ok(candidate_norm) => {
                        // THE MEASUREMENT ABOVE USED TO BE DISCARDED HERE: the divergent branch was
                        // an empty block under a comment saying drift is expected on a clean tree
                        // -- the absorbing fallback (DESIGN section 5) in authoring form. A real
                        // divergence between authority and committed artifact produced no typed,
                        // located, countable output, so its frequency was zero by construction and
                        // never ranked for repair. Membership in HAND_MAINTAINED_STAGE0_FILES did
                        // the silencing while claiming only to describe what the emitter does not
                        // produce.
                        if committed_norm != candidate_norm {
                            if is_declared_divergent(file_name) {
                                declared_divergent.push((*file_name).to_string());
                            } else {
                                undeclared_divergent.push((*file_name).to_string());
                            }
                        } else if is_declared_divergent(file_name) {
                            dissolved_declarations.push((*file_name).to_string());
                        }
                    }
                    Err(reason) => unverifiable.push(((*file_name).to_string(), reason)),
                }
            }
            Err(reason) => unverifiable.push(((*file_name).to_string(), reason)),
        }
    }
    Ok(HandVerifyReport {
        unverifiable,
        declared_divergent,
        undeclared_divergent,
        dissolved_declarations,
        unproduced_declarations,
    })
}

/// `restrict` is the scoped round's selection: `None` writes the whole emitted population (the
/// unscoped round, byte for byte what it always wrote), `Some(set)` writes only those basenames.
///
/// A SCOPED CANDIDATE TREE IS A PARTIAL TREE, AND THAT IS WHAT IT IS FOR. The unscoped tree is a
/// usable crate — every emitted surface plus the copied hand-maintained support — and an author
/// can build it. A scoped tree holds only the mirrors the bound selected, because the one thing
/// the round does with it is install the drifted ones, and the drifted set is a subset of the
/// selection by construction (`compare_generated_surfaces` is run over exactly this set). It is
/// not a crate and is not offered as one.
fn candidate_relative_emit_path(emitted_path: &str) -> Result<String, String> {
    let path = Path::new(emitted_path);
    let relative = path.strip_prefix("src").unwrap_or(path);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "refusal: emitted artifact path {emitted_path} has no safe candidate-root-relative projection"
        ));
    }
    Ok(relative.to_string_lossy().into_owned())
}

fn write_emitted_tree(
    formatter: &ResolvedFormatter,
    dest_src: &Path,
    emitted: &HashMap<String, String>,
    restrict: Option<&BTreeSet<String>>,
) -> Result<(), String> {
    if dest_src.exists() {
        fs::remove_dir_all(dest_src).map_err(|e| format!("remove {}: {e}", dest_src.display()))?;
    }
    fs::create_dir_all(dest_src).map_err(|e| format!("create {}: {e}", dest_src.display()))?;
    let batch: Vec<&str> = emitted
        .iter()
        .filter(|(path, _)| path.ends_with(".rs"))
        .filter(|(path, _)| restrict.is_none_or(|s| s.contains(emit_path_basename(path))))
        .map(|(_, content)| content.as_str())
        .collect();
    normalize_population(formatter, &normalize_batch_dir(dest_src), &batch)?;
    for (path, content) in emitted {
        if let Some(selected) = restrict {
            if !selected.contains(emit_path_basename(path)) {
                continue;
            }
        }
        let relative_path = candidate_relative_emit_path(path)?;
        let out_path = dest_src.join(&relative_path);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create emitted parent {}: {e}", parent.display()))?;
        }
        // Only `.rs` surfaces are the generated-Rust population this comparator reasons
        // about (see committed_generated_basenames / generated_basenames_from_emit); a
        // non-Rust emitted artifact (e.g. Cargo.toml from the crate-layout emit) is not
        // rustfmt-normalizable and is written through verbatim.
        let normalized = if relative_path.ends_with(".rs") {
            normalize_generated_source(formatter, content)
                .map_err(|e| format!("normalize emitted {path}: {e}"))?
        } else {
            content.clone()
        };
        fs::write(&out_path, normalized)
            .map_err(|e| format!("write {}: {e}", out_path.display()))?;
    }
    Ok(())
}

fn copy_hand_maintained_support(stage0_src: &Path, dest_src: &Path) -> Result<(), String> {
    for file_name in HAND_MAINTAINED_STAGE0_FILES {
        let source = stage0_src.join(file_name);
        if source.exists() {
            fs::copy(&source, dest_src.join(file_name))
                .map_err(|e| format!("copy {}: {e}", source.display()))?;
        } else {
            return Err(format!(
                "declared hand-maintained stage0 file names no file on disk: row {file_name:?} \
                 resolves to {} which does not exist. The crate-layout authority \
                 (v2.compiler.self_host.stage0_crate_layout) declares this row; either the row is \
                 corrupt or the file was removed without retiring it.",
                source.display()
            ));
        }
    }
    for dir_name in HAND_MAINTAINED_STAGE0_DIRS {
        let source = stage0_src.join(dir_name);
        if source.is_dir() {
            copy_dir_recursive(&source, &dest_src.join(dir_name))?;
        } else {
            return Err(format!(
                "declared hand-maintained stage0 dir names no directory on disk: row {dir_name:?} \
                 resolves to {} which is not a directory. The crate-layout authority \
                 (v2.compiler.self_host.stage0_crate_layout) declares this row; either the row is \
                 corrupt or the directory was removed without retiring it.",
                source.display()
            ));
        }
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("create {}: {e}", dest.display()))?;
    for entry in fs::read_dir(source).map_err(|e| format!("read dir {}: {e}", source.display()))? {
        let entry = entry.map_err(|e| format!("read dir entry: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            copy_dir_recursive(&path, &dest.join(entry.file_name()))?;
        } else {
            fs::copy(&path, dest.join(entry.file_name()))
                .map_err(|e| format!("copy {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn authority_digest_from_sources(sources: &[(String, String)]) -> Result<String, String> {
    let mut payload = String::new();
    for (path, content) in sources {
        payload.push_str(path);
        payload.push('\0');
        payload.push_str(&bytes_digest(content.as_bytes()));
        payload.push('\n');
    }
    Ok(bytes_digest(payload.as_bytes()))
}

fn tree_digest_for_basenames(
    formatter: &ResolvedFormatter,
    src_dir: &Path,
    basenames: &[String],
    label: &str,
) -> Result<String, String> {
    if basenames.is_empty() {
        return Err(format!(
            "refusal: cannot compute {label} digest over empty population"
        ));
    }
    let contents = basenames
        .iter()
        .map(|name| {
            let path = src_dir.join(name);
            fs::read_to_string(&path).map_err(|e| format!("read {label} {}: {e}", path.display()))
        })
        .collect::<Result<Vec<String>, String>>()?;
    let batch: Vec<&str> = contents.iter().map(String::as_str).collect();
    normalize_population(formatter, &normalize_batch_dir(src_dir), &batch)?;
    let mut payload = String::new();
    for (name, content) in basenames.iter().zip(&contents) {
        let norm = normalize_generated_source(formatter, content)
            .map_err(|e| format!("normalize {label} {name}: {e}"))?;
        payload.push_str(name);
        payload.push('\0');
        payload.push_str(&bytes_digest(norm.as_bytes()));
        payload.push('\n');
    }
    Ok(bytes_digest(payload.as_bytes()))
}

fn tree_digest_from_map(
    formatter: &ResolvedFormatter,
    emitted: &HashMap<String, String>,
    basenames: &[String],
) -> Result<String, String> {
    if basenames.is_empty() {
        return Err("refusal: cannot compute candidate digest over empty population".to_string());
    }
    let mut payload = String::new();
    for name in basenames {
        let content = lookup_emitted(emitted, name)
            .ok_or_else(|| format!("emit missing {name} for digest"))?;
        let norm = normalize_generated_source(formatter, content)
            .map_err(|e| format!("normalize candidate {name}: {e}"))?;
        payload.push_str(name);
        payload.push('\0');
        payload.push_str(&bytes_digest(norm.as_bytes()));
        payload.push('\n');
    }
    Ok(bytes_digest(payload.as_bytes()))
}

/// Maximum rustfmt passes taken while seeking the formatter's fixed point. Exceeding it is a
/// typed refusal, never a silent "good enough" -- a widened failure arm here would be exactly the
/// absorbing fallback DESIGN 5 forbids, and it would restore the unclosable state below.
const NORMALIZE_FIXED_POINT_MAX_PASSES: usize = 8;

/// Run rustfmt to a FIXED POINT, not once.
///
/// rustfmt is not idempotent. Measured 2026-08-20 on `v1_compiler_infer.rs`: a
/// `let x = if (long.receiver.chain)` re-splits on a second pass. A single pass therefore puts the
/// repository's two gates in contradiction on such a file, since they consume different passes:
///
///   * `cargo fmt --all --check` (pre-commit, and the fmt gate) demands pass N+1 of whatever is
///     committed -- it re-formats the file in place;
///   * `write_emitted_tree` wrote pass 1 of the emitted bytes, and `compare_generated_surfaces`
///     compares against exactly those bytes.
///
/// Satisfying either broke the other, with no exit: install the candidate and fmt rewrites it; run
/// fmt and regen reports drift. The only state satisfying both is a FIXED POINT of rustfmt, so the
/// emitted artifact must be one -- then `cargo fmt` is a no-op on it by definition and the byte
/// comparison is exact.
///
/// Construction, not validation (DESIGN 5): the disagreement is made unrepresentable by writing the
/// artifact in the one form both consumers agree on. Iterating here rather than teaching the
/// comparator to tolerate a second pass is deliberate -- the fmt gate would need the tolerance
/// too, and a tolerance shared by two gates is a hole in both.
fn normalize_generated_source(
    formatter: &ResolvedFormatter,
    content: &str,
) -> Result<String, String> {
    if let Some(hit) = formatter.memo.borrow().get(content) {
        return Ok(hit.clone());
    }
    if let Some(hit) = normalize_cache_read(formatter, content) {
        memo_normalized(formatter, content, &hit);
        return Ok(hit);
    }
    let normalized = normalize_generated_source_uncached(formatter, content)?;
    memo_normalized(formatter, content, &normalized);
    normalize_cache_write(formatter, content, &normalized)?;
    Ok(normalized)
}

/// Record one normalization AND its fixed point.
///
/// `normalized` is returned only once rustfmt maps it to itself (the uncached seek stops on
/// `next == current`; the batched seek on an unchanged file), so `normalized -> normalized` is an
/// observed fact, not an assumption. Recording it is what lets the COMMITTED side of a round --
/// whose bytes are exactly a previous round's normalized output -- cost no spawn: before this,
/// every committed mirror paid its own two-spawn seek to rediscover a fixed point the emitted
/// side had just established (measured: 654 spawns for 163 mirrors, `regen-round-cost`, run
/// 37293307189).
fn memo_normalized(formatter: &ResolvedFormatter, raw: &str, normalized: &str) {
    let mut memo = formatter.memo.borrow_mut();
    memo.insert(raw.to_string(), normalized.to_string());
    memo.insert(normalized.to_string(), normalized.to_string());
}

/// NORMALIZE A POPULATION WITH ONE rustfmt SPAWN PER PASS, not two per member.
///
/// Every member not already memoized or on the disk cache is written to its own file under one
/// work directory, and each pass formats all still-moving files in a single invocation. A member
/// is settled when a pass leaves its bytes unchanged -- the same fixed-point definition
/// `normalize_generated_source_uncached` uses for one input, so the answer per member is the same
/// and only the spawn count changes. It fills the memo and the disk cache; the per-member callers
/// that follow then answer from the memo. A member still moving after
/// `NORMALIZE_FIXED_POINT_MAX_PASSES` refuses the whole population, as the single seek refuses.
fn normalize_population(
    formatter: &ResolvedFormatter,
    work_dir: &Path,
    contents: &[&str],
) -> Result<(), String> {
    let mut pending: Vec<String> = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for content in contents {
        if !seen.insert(content) || formatter.memo.borrow().contains_key(*content) {
            continue;
        }
        if let Some(hit) = normalize_cache_read(formatter, content) {
            memo_normalized(formatter, content, &hit);
            continue;
        }
        pending.push(content.to_string());
    }
    if pending.is_empty() {
        return Ok(());
    }
    if work_dir.exists() {
        fs::remove_dir_all(work_dir).map_err(|e| format!("remove {}: {e}", work_dir.display()))?;
    }
    fs::create_dir_all(work_dir).map_err(|e| format!("create {}: {e}", work_dir.display()))?;
    let paths: Vec<PathBuf> = (0..pending.len())
        .map(|i| work_dir.join(format!("m{i}.rs")))
        .collect();
    let mut current: Vec<String> = pending.clone();
    for (path, content) in paths.iter().zip(&current) {
        fs::write(path, content).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    // `moving[i]`: member i has not yet been observed unchanged by a pass AFTER its first. The
    // first pass maps raw -> pass 1 and settles nothing, exactly as the single seek's first
    // attempt does.
    let mut moving: Vec<usize> = (0..pending.len()).collect();
    let mut first = true;
    for _ in 0..NORMALIZE_FIXED_POINT_MAX_PASSES {
        if moving.is_empty() {
            break;
        }
        RUSTFMT_SPAWNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // `skip_children=true`: given a FILE, rustfmt follows its `mod` declarations to sibling
        // files, which stdin mode never does, so a mirror declaring a module refused the batch
        // with `failed to resolve mod` (seen on a remote round). Each member is formatted alone,
        // exactly as the single stdin seek formats it.
        let output = formatter
            .command()
            .arg("--edition")
            .arg("2021")
            .arg("--config")
            .arg("skip_children=true")
            .args(moving.iter().map(|i| paths[*i].as_os_str()))
            .output()
            .map_err(|e| formatter.spawn_refusal(e))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        let mut still = Vec::new();
        for i in moving {
            let next = fs::read_to_string(&paths[i])
                .map_err(|e| format!("read normalized {}: {e}", paths[i].display()))?;
            if first || next != current[i] {
                still.push(i);
            }
            current[i] = next;
        }
        moving = still;
        first = false;
    }
    if !moving.is_empty() {
        return Err(format!(
            "rustfmt did not reach a fixed point in {NORMALIZE_FIXED_POINT_MAX_PASSES} passes \
             for {} batched member(s)",
            moving.len()
        ));
    }
    for (raw, normalized) in pending.iter().zip(&current) {
        memo_normalized(formatter, raw, normalized);
        normalize_cache_write(formatter, raw, normalized)?;
    }
    let _ = fs::remove_dir_all(work_dir);
    Ok(())
}

/// The cache file layout: `<raw byte length>\n<raw bytes><normalized bytes>`. The raw bytes
/// are stored whole and compared whole on read, so the file name (a digest) only locates the
/// entry and never stands in for its identity.
fn normalize_cache_path(formatter: &ResolvedFormatter, content: &str) -> Option<PathBuf> {
    formatter
        .disk_cache
        .as_ref()
        .map(|dir| dir.join(v1_rt::bytes_identity_hash(content.as_bytes())))
}

fn normalize_cache_read(formatter: &ResolvedFormatter, content: &str) -> Option<String> {
    let path = normalize_cache_path(formatter, content)?;
    let bytes = fs::read(&path).ok()?;
    let newline = bytes.iter().position(|b| *b == b'\n')?;
    let raw_len: usize = std::str::from_utf8(&bytes[..newline]).ok()?.parse().ok()?;
    let raw_start = newline + 1;
    let raw_end = raw_start.checked_add(raw_len)?;
    if raw_end > bytes.len() || &bytes[raw_start..raw_end] != content.as_bytes() {
        return None;
    }
    String::from_utf8(bytes[raw_end..].to_vec()).ok()
}

fn normalize_cache_write(
    formatter: &ResolvedFormatter,
    content: &str,
    normalized: &str,
) -> Result<(), String> {
    let Some(path) = normalize_cache_path(formatter, content) else {
        return Ok(());
    };
    let mut bytes = format!("{}\n", content.len()).into_bytes();
    bytes.extend_from_slice(content.as_bytes());
    bytes.extend_from_slice(normalized.as_bytes());
    // Write-then-rename, so a reader never sees a half-written entry as a miss-with-garbage.
    let tmp = path.with_extension(format!("tmp{}", std::process::id()));
    fs::write(&tmp, &bytes).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))
}

fn normalize_generated_source_uncached(
    formatter: &ResolvedFormatter,
    content: &str,
) -> Result<String, String> {
    let mut current = normalize_generated_source_attempt(formatter, content)?;
    for _ in 1..NORMALIZE_FIXED_POINT_MAX_PASSES {
        let next = normalize_generated_source_attempt(formatter, &current)?;
        if next == current {
            return Ok(current);
        }
        current = next;
    }
    Err(format!(
        "rustfmt did not reach a fixed point in {NORMALIZE_FIXED_POINT_MAX_PASSES} passes"
    ))
}

/// THE FORMATTER, RESOLVED ONCE AT ADMISSION INSTEAD OF LOOKED UP AT EVERY SPAWN.
///
/// Both normalize paths used to spawn a BARE `rustfmt`, resolved from ambient PATH at the moment
/// of use -- 45-50 minutes into a required run. On 2026-08-24 (run 32693719649, srv2-05) that
/// failed with `spawn rustfmt: No such file or directory` after the job's own toolchain probe had
/// printed `/home/ghrunner/.cargo/bin/rustfmt` 48 minutes earlier in the same shell environment.
/// The cost is WHERE it lands: a PATH question became a fifty-minute-deep failure in a phase
/// unrelated to PATH.
///
/// WHAT THIS FIXES AND WHAT IT DOES NOT. It fixes the formatter ABSENT AT ADMISSION: that refuses
/// immediately, naming the PATH searched. It does NOT prevent a formatter present at admission and
/// gone at spawn -- resolving a path is not holding a file open, and nothing here stops a
/// concurrent process replacing a shim. For that case it makes the failure ATTRIBUTABLE: the spawn
/// error names the program resolved at admission and states it existed then, turning an ambiguous
/// ENOENT into evidence that the environment mutated under the run -- the question the workflow's
/// toolchain probe is collecting baselines for, which no run has yet answered.
///
/// AUTHORITY, AND THE RUNG. The refusal is named by the carrier as `v2.workflow.required_regen`
/// `RequiredRegenRefusal`'s `FormatterUnavailable`, added in the same change, because this host is
/// a MIRROR of that carrier -- DESIGN.md records `run_required_regen`'s hand-written ordering at
/// *mitigatable*, with derivation from the carrier as its next-rung trigger -- and a refusal the
/// host can produce that the carrier cannot name is the divergence that trigger closes. WHAT IS
/// NOT CLAIMED: this host reports refusals as `String` like every other refusal on this path, so
/// the correspondence is held by review and this citation, NOT the type system; a typed carrier
/// for this one refusal would be a second representation of a vocabulary the carrier owns. The
/// class stays at *mitigatable* and inherits the host's existing next-rung trigger.
///
/// NO FALLBACK ARM EXISTS AND NONE MAY BE ADDED: no retry, no tolerated absence, no "skip
/// normalization and compare raw". The emitted artifact must be a fixed point of the formatter or
/// the comparison is meaningless (see `normalize_generated_source`), so a run without one must
/// stop -- at admission rather than at minute fifty.
#[derive(Debug, Clone)]
pub struct ResolvedFormatter {
    program: PathBuf,
    /// The PATH the program was found on, kept so a later failure can say what was searched
    /// rather than making the reader reconstruct it from the environment they are not in.
    searched: String,
    /// NORMALIZE ONCE PER DISTINCT INPUT. Every regen pass normalized the whole emitted
    /// population THREE times -- `write_emitted_tree`, `compare_generated_surfaces`,
    /// `tree_digest_from_map` each called `normalize_generated_source` on the same bytes -- at
    /// two or more rustfmt spawns each (the fixed-point seek). Measured 2026-08-30 by
    /// `--regen-round-cost`: mirror_write + adjudicate + digest = 67 s on the BuildBuddy runner,
    /// 97 s on srv1, for one population. Keyed by the RAW input bytes, so a hit is exact by
    /// construction; clones share it so the three consumers see one table.
    memo: Rc<std::cell::RefCell<HashMap<String, String>>>,
    /// The on-disk half, keyed by rustfmt's own version string, so a fixed-point run on a tree
    /// whose emit did not change pays ZERO rustfmt spawns. Each entry stores the raw input beside
    /// the normalized output and is honoured only when the stored raw is byte-equal to the
    /// request -- a digest-named file is a LOCATION, never the identity, so a hash collision
    /// cannot serve another input's formatting (DESIGN section 5).
    disk_cache: Option<PathBuf>,
}

/// Every NORMALIZE spawn this process has made -- the `--version` probe in `with_normalize_cache`
/// is not counted, since the fixed-point claim is about normalizations. Read by `--regen-round-cost` before and after
/// the round so the receipt carries the count -- the fixed-point control's claim is that it
/// is ~0, and a claim about a count is checked against the count.
static RUSTFMT_SPAWNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn rustfmt_spawn_count() -> u64 {
    RUSTFMT_SPAWNS.load(std::sync::atomic::Ordering::Relaxed)
}

const NORMALIZE_CACHE_DIR_REL: &str = "target/stage0-regen-rustfmt-cache";

/// Where a batched normalization writes its members: beside the workspace's `target/`, never
/// inside the tree being normalized, so a batch over the committed mirrors cannot touch them.
fn normalize_batch_dir(anchor: &Path) -> PathBuf {
    let workspace = anchor
        .ancestors()
        .find(|a| a.join("Cargo.toml").is_file() && a.join("target").is_dir())
        .map(Path::to_path_buf)
        .unwrap_or_else(workspace_root);
    // Per process: a batch clears its directory before writing, so two rounds in one workspace
    // sharing a path would delete each other's members mid-format (review 77080).
    workspace.join(format!(
        "target/stage0-regen-rustfmt-batch-{}",
        std::process::id()
    ))
}

impl ResolvedFormatter {
    /// Resolve `rustfmt` against a supplied PATH string. Separate from the environment read so
    /// the refusal is testable without mutating a process-global that every other test shares.
    fn from_path_var(path_var: &str) -> Result<Self, String> {
        let entries: Vec<&str> = path_var.split(':').filter(|e| !e.is_empty()).collect();
        for dir in &entries {
            let candidate = Path::new(dir).join("rustfmt");
            if !Self::is_executable_file(&candidate) {
                continue;
            }
            let resolved = ResolvedFormatter {
                program: candidate,
                searched: path_var.to_string(),
                memo: Rc::new(std::cell::RefCell::new(HashMap::new())),
                disk_cache: None,
            };
            resolved.probe()?;
            return Ok(resolved);
        }
        Err(format!(
            concat!(
                "rustfmt is not on PATH: searched {} entr(ies) [{}]. ",
                "The required regen phase normalizes every emitted source through rustfmt to a ",
                "fixed point, so it cannot run without one -- install the component ",
                "(`rustup component add rustfmt`) or repair PATH. Refused at admission rather ",
                "than at the first normalize, which is ~50 minutes into the run."
            ),
            entries.len(),
            entries.join(", ")
        ))
    }

    /// PATH RESOLUTION REQUIRES THE EXECUTE BIT, not merely a file. The OS skips a non-executable
    /// entry and keeps searching, so an `is_file` resolver would ADMIT a file the kernel steps
    /// over -- shadowing a real formatter further down PATH and failing where a bare
    /// `Command::new("rustfmt")` succeeds. Matching the kernel's predicate keeps this a faithful
    /// model of the lookup it replaces, not a more permissive one.
    #[cfg(unix)]
    fn is_executable_file(candidate: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        match fs::metadata(candidate) {
            Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }

    /// The non-unix arm is a PLATFORM REALIZATION, not a failure arm: there is no mode bit to
    /// read, so existence is the strongest available predicate. The probe below still runs, so
    /// an unusable program is refused at admission on every platform.
    #[cfg(not(unix))]
    fn is_executable_file(candidate: &Path) -> bool {
        candidate.is_file()
    }

    /// EXECUTING THE PROGRAM ONCE IS THE ADMISSION; the metadata above is a proxy. A broken shim,
    /// a wrong-architecture binary, or a dangling interpreter line all carry the mode bit and die
    /// at the first real normalize -- the ~50-minute failure this row moves. `--version` is
    /// total, cheap and reads nothing from the tree.
    fn probe(&self) -> Result<(), String> {
        let observed = Command::new(&self.program).arg("--version").output();
        match observed {
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => Err(format!(
                concat!(
                    "rustfmt at {} is not usable: `--version` exited {}. It was found on PATH ",
                    "[{}] and is executable, so this is a broken or mis-targeted installation ",
                    "rather than a missing one -- repair it or remove it from PATH so a later ",
                    "entry can serve. Refused at admission rather than at the first normalize."
                ),
                self.program.display(),
                out.status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "by signal".to_string()),
                self.searched
            )),
            Err(cause) => Err(format!(
                concat!(
                    "rustfmt at {} could not be executed: {}. It was found on PATH [{}] with ",
                    "the execute bit set, so this is a broken installation rather than a ",
                    "missing one. Refused at admission rather than at the first normalize."
                ),
                self.program.display(),
                cause,
                self.searched
            )),
        }
    }

    fn admit() -> Result<Self, String> {
        Self::from_path_var(&std::env::var("PATH").unwrap_or_default())
    }

    /// Attach the on-disk normalize cache for this formatter's version under `target/`.
    /// The version is read from the program itself, so a toolchain change opens a new
    /// directory rather than serving another rustfmt's fixed points.
    fn with_normalize_cache(mut self, workspace: &Path) -> Result<Self, String> {
        let out = self
            .command()
            .arg("--version")
            .output()
            .map_err(|e| self.spawn_refusal(e))?;
        if !out.status.success() {
            return Err(format!(
                "rustfmt --version failed at {}: {}",
                self.program.display(),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        let version: String = String::from_utf8_lossy(&out.stdout)
            .trim()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        if version.is_empty() {
            return Err("rustfmt --version printed nothing; the cache has no key".to_string());
        }
        let dir = workspace.join(NORMALIZE_CACHE_DIR_REL).join(version);
        fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
        self.disk_cache = Some(dir);
        Ok(self)
    }

    fn command(&self) -> Command {
        Command::new(&self.program)
    }

    /// The spawn-failure message, the attributable half described on the type: it says the program
    /// EXISTED at admission -- the fact a reader cannot recover later, discriminating "never
    /// installed" from "replaced under the run".
    fn spawn_refusal(&self, cause: std::io::Error) -> String {
        format!(
            concat!(
                "spawn rustfmt: {} -- program {} was resolved at admission from PATH [{}], ",
                "and ran there: `--version` was executed successfully before this phase began. ",
                "So it has been removed, replaced or made unusable while this run was ",
                "executing; this is not a missing or broken installation"
            ),
            cause,
            self.program.display(),
            self.searched
        )
    }
}

fn normalize_generated_source_attempt(
    formatter: &ResolvedFormatter,
    content: &str,
) -> Result<String, String> {
    RUSTFMT_SPAWNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut child = formatter
        .command()
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| formatter.spawn_refusal(e))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "rustfmt stdin unavailable".to_string())?;
    let owned = content.to_string();
    let writer = std::thread::spawn(move || stdin.write_all(owned.as_bytes()));
    let out = child
        .wait_with_output()
        .map_err(|e| format!("wait rustfmt: {e}"))?;
    writer
        .join()
        .map_err(|_| "rustfmt stdin writer panicked".to_string())?
        .map_err(|e| format!("write rustfmt stdin: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

fn normalize_with_workdir(
    formatter: &ResolvedFormatter,
    content: &str,
    work_dir: &Path,
    label: &str,
) -> Result<String, String> {
    let path = work_dir.join(format!("{label}.rs"));
    fs::write(&path, content).map_err(|e| format!("write {}: {e}", path.display()))?;
    RUSTFMT_SPAWNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let output = formatter
        .command()
        .arg("--edition")
        .arg("2021")
        .arg(path.as_os_str())
        .output()
        .map_err(|e| formatter.spawn_refusal(e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    fs::read_to_string(&path).map_err(|e| format!("read normalized {label}: {e}"))
}

fn git_head_sha(workspace: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("git rev-parse HEAD: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn write_receipt(path: &Path, receipt: &RegenReceipt) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(receipt)
        .map_err(|e| format!("serialize regen receipt: {e}"))?;
    fs::write(path, json).map_err(|e| format!("write receipt {}: {e}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// The cgroup diagnostic reads the leaf and walks ancestors to the first level that binds, prints
    /// a file that does not exist as `absent` and never as a number, and does not walk past the mount
    /// root. The discriminator is the `absent` field: a default of `0` or `max` would turn it red.
    #[test]
    fn the_cgroup_diagnostic_prints_absent_files_and_stops_at_the_first_binding_ancestor() {
        let root = temp_dir("cgroup-diag");
        let slice = root.join("system.slice");
        let unit = slice.join("unit.service");
        fs::create_dir_all(&unit).expect("create cgroup tree");
        fs::write(unit.join("memory.current"), "100\n").unwrap();
        fs::write(unit.join("memory.peak"), "200\n").unwrap();
        fs::write(unit.join("memory.max"), "300\n").unwrap();
        fs::write(unit.join("memory.high"), "250\n").unwrap();
        fs::write(
            unit.join("memory.events"),
            "low 0\nhigh 7\nmax 0\noom 0\noom_kill 0\n",
        )
        .unwrap();
        fs::write(unit.join("memory.stat"), "anon 11\nfile 22\nswapcached 0\n").unwrap();
        // memory.swap.current and memory.swap.max are deliberately NOT written on the leaf.
        fs::write(slice.join("memory.max"), "max\n").unwrap();
        fs::write(slice.join("memory.high"), "max\n").unwrap();
        fs::write(slice.join("memory.swap.max"), "max\n").unwrap();
        fs::write(root.join("memory.max"), "999\n").unwrap();
        let line = cgroup_memory_diagnostic_line(&unit, &root, "test.phase", 42);
        for needle in [
            "phase=test.phase pid=42",
            "memory.current=100 memory.peak=200",
            "memory.swap.current=absent",
            "memory.events=low 0 high 7 max 0 oom 0 oom_kill 0",
            "anon=11 file=22",
            "kernel=absent",
            "[memory.max=300 memory.high=250 memory.swap.max=absent]",
            "[memory.max=max memory.high=max memory.swap.max=max]",
            "[memory.max=999 memory.high=absent memory.swap.max=absent]",
        ] {
            assert!(line.contains(needle), "missing `{needle}` in: {line}");
        }
        let above_root = cgroup_memory_diagnostic_line(&root, &root, "p", 1);
        assert_eq!(
            above_root.matches("memory.max=").count(),
            1,
            "the walk must stop at the mount root: {above_root}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&path).expect("create temp");
        path
    }

    /// THE CONTROL FOR "FORWARD THE CHILD'S OUTPUT, DO NOT BUFFER AND DROP IT".
    /// `run_built_seed_regen` used to spawn the rebuilt-seed child with `Command::output()`: its
    /// stdout and stderr went into two `String`s that nobody read unless a refusal fired, so the
    /// child's own per-phase `started …`/`… done in …` marks -- which come out of
    /// `v1_rt::trace_mark`, and that writes with `eprintln!`, i.e. on STDERR -- and every cargo and
    /// rustc diagnostic were dropped. That is how the one phase whose cost moves (5 minutes on one
    /// runner, 53 on another) printed nothing at all.
    ///
    /// The fixture runs the SAME function production calls, against a child that prints a line on
    /// stdout, prints a phase line on stderr, writes 20 more stderr lines, sleeps, then fails. Three
    /// legs, each pinning one way the deliverable can regress:
    /// ONE (streaming): an observer thread must see the child's phase line in the destination while
    ///   the child is still alive. A buffer-and-flush-at-exit implementation -- `output()` restored
    ///   -- cannot satisfy this leg, and nothing else in the tree reads that stream.
    /// TWO (policy): production's own choice must still be the job log. Move
    ///   `rebuilt_seed_regen_child_output_policy()` off it and this leg goes red.
    /// THREE (refusals byte-identical): the bounded tail is the last 12 lines in the child's own
    ///   order, both refusal texts are spelled exactly as they were before this change, no truncation
    ///   marker appears on an un-truncated stream, and neither text may fabricate the child's line.
    #[test]
    #[cfg(unix)]
    fn rebuilt_seed_child_output_reaches_the_job_log_before_the_refusal() {
        use std::os::unix::fs::PermissionsExt;
        let temp = temp_dir("rebuilt-seed-child-forwarding");
        let script = temp.join("child.sh");
        let stdout_path = temp.join("child-stdout.log");
        let stderr_path = temp.join("child-stderr.log");
        fs::write(
            &script,
            "#!/bin/sh\n\
             echo 'child stdout line'\n\
             echo 'started round.built_seed_regen child phase line' >&2\n\
             n=1\n\
             while [ $n -le 20 ]; do echo \"child stderr line $n\" >&2; n=$((n+1)); done\n\
             sleep 6\n\
             exit 7\n",
        )
        .expect("write the fixture child");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755))
            .expect("set the execute bit on the fixture child");

        assert!(
            matches!(
                rebuilt_seed_regen_child_output_policy(),
                RebuiltSeedChildOutput::ForwardToJobLog
            ),
            "production no longer hands the rebuilt-seed child's output to the job log; the phase \
             whose cost moves would print nothing again"
        );

        // The liveness model is orchestration state, loaded before the generation begins -- and
        // before the observer's clock below starts, so its load time is not charged to the child.
        let liveness = SupervisedSubjectModel::load(
            &["dag", "src/v2"]
                .iter()
                .map(|root| workspace_root().join(root).to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
        )
        .expect("load the liveness model");

        // ONE: read the destination from another thread while the child is still running. The child
        // sleeps 6s after printing, so a line seen within 3s of the call starting is strictly before
        // the child can have exited -- which is the whole claim: the reader sees the child working.
        let started = std::time::Instant::now();
        let watched = stderr_path.clone();
        let observer = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            loop {
                if fs::read_to_string(&watched)
                    .unwrap_or_default()
                    .contains("started round.built_seed_regen child phase line")
                {
                    return Some(started.elapsed());
                }
                if std::time::Instant::now() >= deadline {
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        });
        let run = spawn_rebuilt_seed_regen_child(
            &script,
            &[],
            RebuiltSeedChildOutput::ForwardToFiles {
                stdout: stdout_path.clone(),
                stderr: stderr_path.clone(),
            },
            &temp,
            GenerationSupervision::SlotStallVerdict {
                subject: "generation 1".to_string(),
                model: &liveness,
            },
        )
        .expect("spawn the fixture child");
        let observed_at = observer
            .join()
            .expect("join the destination observer")
            .expect("the child's phase line never reached the destination while it was alive");
        assert_eq!(
            run.status.code(),
            Some(7),
            "the fixture child must fail the way a refused regeneration pass fails"
        );
        assert!(
            observed_at.as_millis() < 6000,
            "the phase line only appeared after the child could have exited: {observed_at:?}"
        );

        // THREE: read before anything else can rewrite the destinations.
        let shown = fs::read_to_string(&stdout_path).expect("the child's stdout destination");
        assert!(
            shown.contains("child stdout line"),
            "the child's stdout never reached the destination the parent handed over: {shown}"
        );
        assert!(
            !run.stderr_truncated,
            "a 21-line stream must not report a truncated tail"
        );
        let expected_tail = (9..=20)
            .map(|line| format!("child stderr line {line}"))
            .collect::<Vec<_>>()
            .join(" | ");
        assert_eq!(
            run.stderr_tail, expected_tail,
            "the tail is not the last 12 lines in the child's own order"
        );
        let receipt_path = temp.join("target/stage0-regen-receipt.json");
        assert_eq!(
            rebuilt_seed_regen_missing_receipt_refusal(&receipt_path, &run),
            format!(
                "rebuilt seed produced no fresh generation receipt at {}; status=exit status: 7 stderr_tail={expected_tail}",
                receipt_path.display()
            )
        );
        assert_eq!(
            rebuilt_seed_regen_failed_refusal(&run),
            "rebuilt seed exited exit status: 7; expected a successful regeneration pass stderr_tail=child stderr line 9 | child stderr line 10 | child stderr line 11 | child stderr line 12 | child stderr line 13 | child stderr line 14 | child stderr line 15 | child stderr line 16 | child stderr line 17 | child stderr line 18 | child stderr line 19 | child stderr line 20"
        );
        assert!(
            !rebuilt_seed_regen_failed_refusal(&run).contains("phase line"),
            "a refusal must not fabricate the child's output; the child printed it, to the log"
        );
    }

    fn fixture_candidate_manifest() -> RegenCandidateManifest {
        let surfaces = Vec::new();
        RegenCandidateManifest {
            producer_seed_digest: "seed".to_string(),
            generation_id: "generation".to_string(),
            candidate_tree_id: "tree".to_string(),
            candidate_tree_digest: "tree-digest".to_string(),
            aggregate_digest: candidate_manifest_aggregate(
                "seed",
                "generation",
                "tree",
                "tree-digest",
                &surfaces,
            )
            .unwrap(),
            surfaces,
        }
    }

    /// BOTH REFUSALS ARE THE PRODUCT OF THIS ROW, so their TEXT is under test, not just substrings.
    /// A multi-line non-raw literal absorbs its continuation indentation, and `cargo fmt` reflows
    /// such literals -- so a message garbles with nobody editing it while every `contains` still
    /// passes, the runs of spaces sitting BETWEEN matched fragments. This guard went red on the
    /// form that shipped in this PR's first revision.
    fn assert_message_is_not_reflowed(message: &str) {
        assert!(
            !message.contains("  "),
            "a diagnostic must not carry runs of literal whitespace from source indentation \
             -- use `concat!` rather than splitting a literal across lines: {message:?}"
        );
    }

    /// THE REFUSAL NAMES WHAT WAS SEARCHED, AND THE PAIR IS THE ASSERTION.
    ///
    /// RED against the pre-change code: with no admission, the absent-formatter case had no return
    /// value to assert on -- it surfaced as `spawn rustfmt: No such file or directory` from
    /// whichever normalize ran first, ~50 minutes into a required run. The empty-PATH arm is the
    /// discriminating input; the real-PATH arm is the control without which a resolver refusing
    /// everything would pass.
    ///
    /// `from_path_var` takes the PATH rather than reading the environment so this test does not
    /// mutate a process-global shared by every test in this binary -- `set_var("PATH", "")` would
    /// fail unrelated tests by thread order, a flake class costing more to diagnose than the
    /// check is worth.
    #[test]
    fn an_absent_formatter_refuses_at_admission_and_names_the_path_searched() {
        let refused = ResolvedFormatter::from_path_var("/nonexistent/aa:/nonexistent/bb")
            .expect_err("no rustfmt exists under either entry");
        assert!(
            refused.contains("not on PATH"),
            "the refusal must say what is missing: {refused}"
        );
        assert!(
            refused.contains("/nonexistent/aa") && refused.contains("/nonexistent/bb"),
            "and must name every entry it searched, or the reader cannot act on it: {refused}"
        );
        assert!(
            refused.contains("admission"),
            "and must say WHERE it refused, since the whole point is that this is not the \
             spawn-time failure it replaces: {refused}"
        );
        assert_message_is_not_reflowed(&refused);

        let resolved = ResolvedFormatter::admit().expect(
            "the real PATH has a rustfmt; this arm is the control against a resolver \
                     that refuses everything",
        );
        assert!(
            resolved.program.is_file(),
            "a resolved formatter must name a real file, not a bare program name"
        );
    }

    /// A NON-EXECUTABLE FILE NAMED `rustfmt` MUST NOT SHADOW A REAL ONE FURTHER DOWN PATH.
    ///
    /// The RED for review 55506's first finding. Against the `is_file`-keyed resolver this FAILS:
    /// the unusable file is admitted, the real formatter below it never reached, and the run dies
    /// at the first normalize -- the ~50-minute failure this row moves, reintroduced by the row.
    /// The kernel skips such an entry, so the old resolver was strictly more permissive than the
    /// lookup it replaced.
    #[test]
    fn a_non_executable_file_does_not_shadow_a_real_formatter_later_on_path() {
        let shadow = temp_dir("formatter-shadow");
        fs::write(shadow.join("rustfmt"), "not a program\n").expect("plant the shadow");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(shadow.join("rustfmt"), fs::Permissions::from_mode(0o644))
                .expect("clear the execute bit");
        }

        let real = ResolvedFormatter::admit().expect("this host has a usable rustfmt");
        let real_dir = real
            .program
            .parent()
            .expect("a resolved program has a parent directory");

        let resolved = ResolvedFormatter::from_path_var(&format!(
            "{}:{}",
            shadow.display(),
            real_dir.display()
        ))
        .expect("the real formatter below the shadow must still be found");
        assert_eq!(
            resolved.program, real.program,
            "the unusable file must be stepped over, not admitted"
        );
    }

    /// AN EXECUTABLE THAT DOES NOT RUN IS REFUSED AT ADMISSION TOO. The mode bit is a proxy;
    /// executing `--version` is the fact. A broken shim carries the bit and dies at the first
    /// normalize, which is the failure being moved -- so admission probes rather than inspects.
    #[test]
    #[cfg(unix)]
    fn an_executable_that_fails_its_probe_is_refused_at_admission() {
        use std::os::unix::fs::PermissionsExt;
        let broken = temp_dir("formatter-broken");
        fs::write(broken.join("rustfmt"), "#!/bin/sh\nexit 3\n").expect("plant a broken shim");
        fs::set_permissions(broken.join("rustfmt"), fs::Permissions::from_mode(0o755))
            .expect("set the execute bit");

        let refused = ResolvedFormatter::from_path_var(&broken.display().to_string())
            .expect_err("a program that cannot run must not be admitted");
        assert!(
            refused.contains("not usable") && refused.contains("exited 3"),
            "the refusal must name the observed exit, not guess at a cause: {refused}"
        );
        assert!(
            refused.contains("rather than a missing one"),
            "and must separate broken from absent, since the remedies differ: {refused}"
        );
        assert_message_is_not_reflowed(&refused);
    }

    /// THE SPAWN REFUSAL DISTINGUISHES "NEVER INSTALLED" FROM "REPLACED UNDER THE RUN" -- the fact
    /// the motivating run could not report. Asserted on the message rather than by staging a
    /// mid-run deletion: nothing here can portably make a live process lose a resolved file, and
    /// pointing at a path that never existed would assert the message while proving nothing.
    #[test]
    fn the_spawn_refusal_says_the_program_existed_at_admission() {
        let formatter = ResolvedFormatter::admit().expect("rustfmt on PATH for this test");
        let message = formatter.spawn_refusal(std::io::Error::from_raw_os_error(2));
        assert!(
            message.contains("ran there") && message.contains("executed successfully"),
            "the message must carry the admission-time fact, which is now that the program RAN \
             and not merely that it existed: {message}"
        );
        assert!(
            message.contains("is not a missing or broken installation"),
            "and must rule out BOTH readings it is there to rule out -- absent, and present but \
             unusable -- since the probe now excludes the second as well: {message}"
        );
        assert!(
            message.contains(&formatter.program.display().to_string()),
            "and must name the exact program, not the bare name: {message}"
        );
        assert_message_is_not_reflowed(&message);
    }

    /// THE BATCH IS A SPAWN-COUNT CHANGE, NOT A FORMATTING CHANGE: every member's normalized bytes
    /// equal what the single-input seek produces for it, on fresh formatters with no memo or disk
    /// cache between them. RED: a batch that settled a member after ONE pass (skipping the confirm
    /// pass), or that formatted members in a different configuration than stdin, would diverge on
    /// the non-canonical inputs here. The fixed-point memo is asserted too: the normalized bytes
    /// answer from the memo as themselves, which is the committed side's zero-spawn path.
    #[test]
    fn batched_normalization_is_byte_identical_to_the_single_seek() {
        let inputs = [
            "fn  a( x:i32 )->i32{ x+1 }\n",
            "pub struct S{a:u8,b:String}\nimpl S{fn f(&self)->u8{self.a}}\n",
            "fn b() { let v = if true { vec![1,2,3].iter().map(|x| x * 2).collect::<Vec<_>>() } else { Vec::new() }; drop(v); }\n",
            "fn  a( x:i32 )->i32{ x+1 }\n",
            // A member declaring a module whose file does not exist beside it: stdin formats it,
            // and a batch that followed the declaration refused the whole population.
            "mod  no_such_sibling;\nfn c(){}\n",
        ];
        let single = ResolvedFormatter::admit().expect("rustfmt on PATH for this test");
        let expected: Vec<String> = inputs
            .iter()
            .map(|i| normalize_generated_source_uncached(&single, i).expect("single seek"))
            .collect();

        let batched = ResolvedFormatter::admit().expect("rustfmt on PATH for this test");
        let dir = temp_dir("required-regen-rustfmt-batch");
        normalize_population(&batched, &dir, &inputs).expect("batched seek");
        for (input, want) in inputs.iter().zip(&expected) {
            let got = batched.memo.borrow().get(*input).cloned();
            assert_eq!(
                got.as_deref(),
                Some(want.as_str()),
                "batch must equal the single seek"
            );
            assert_eq!(
                batched.memo.borrow().get(want.as_str()).map(String::as_str),
                Some(want.as_str()),
                "a normalized output must be memoized as its own fixed point"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    /// Plants every declared hand-maintained row on disk, then removes exactly one, so the
    /// accepted control and the refusal differ by a single row. `"rs"` — the corrupt row a
    /// merge produced on integration/namespace-cut — is inert without this wall, because the
    /// copy loop skipped a declared row naming nothing.
    // ------------------------------------------------------------------------------------
    // THE CHANGE-DENOMINATED EMISSION'S CONTROLS.
    //
    // Three modules, no imports, one selected. That is enough to discriminate every claim the
    // scoped emission makes, and small enough to run in the required unit step rather than in a
    // live-corpus lane -- the whole-tree version of the same comparison is the planted-edit
    // control the regen round runs.
    // ------------------------------------------------------------------------------------
    fn selection_fixture() -> Vec<(String, String)> {
        ["alpha", "beta", "gamma"]
            .iter()
            .map(|name| {
                (
                    format!("fx_{name}.dag"),
                    format!("module fx.{name}\nfn {name}_add(a: Int, b: Int) -> Int {{ a + b }}\n"),
                )
            })
            .collect()
    }

    /// THE PACKAGE GRAPH THE FIXTURE POPULATION IS BUILT UNDER, supplied rather than read from the
    /// seed tree, whose rows name none of the fixture modules and so would refuse every edge as not
    /// covered. It has the seed's shape at the scale of the fixture: the runtime prelude module
    /// (`gunbc.rust_emitted_edge` `rust_runtime_prelude_module`) in a foundation crate, and the
    /// three fixture modules in a layered core crate that re-exports it, so each prelude edge is
    /// admitted by the partition's own layering rule rather than by a hand-written edge.
    fn selection_fixture_packages(
    ) -> Rc<crate::gunbc_stage0_stage0_emitted_edge_admission::Stage0EmittedTreePackages> {
        use crate::gunbc_stage0_stage0_crate_partition_generated::{
            GeneratedPartitionCrateKind, GeneratedPartitionCrateRow,
        };
        let row = |package: &str,
                   kind: GeneratedPartitionCrateKind,
                   modules: im::Vector<String>,
                   reexports: im::Vector<String>| {
            Rc::new(GeneratedPartitionCrateRow {
                package_name: package.to_string(),
                crate_dir: package.replace('-', "_"),
                kind,
                modules: Rc::new(modules),
                reexport_packages: Rc::new(reexports),
                carries_non_empty_wrappers: false,
            })
        };
        Rc::new(
            crate::gunbc_stage0_stage0_emitted_edge_admission::Stage0EmittedTreePackages {
                rows: Rc::new(im::vector![
                    row(
                        "fx-runtime",
                        GeneratedPartitionCrateKind::GeneratedFoundationCrate,
                        im::vector![crate::gunbc_rust_emitted_edge::rust_runtime_prelude_module()],
                        im::Vector::new(),
                    ),
                    row(
                        "fx-core",
                        GeneratedPartitionCrateKind::GeneratedLayeredCoreCrate,
                        selection_fixture()
                            .iter()
                            .map(|(path, _)| path.trim_end_matches(".dag").to_string())
                            .collect(),
                        im::vector!["fx-runtime".to_string()],
                    ),
                ]),
                host_shell_package_name: "fx-host".to_string(),
                host_shell_dependencies: Rc::new(im::vector!["fx-core".to_string()]),
            },
        )
    }

    /// `selected_basenames` is `None` for the whole-closure arm and `Some(list)` for a selection.
    /// The `RustModuleRenderSelection` itself is built INSIDE the worker thread: the emitter's
    /// values are `Rc`-shaped and therefore not `Send`, so the selection cannot cross the thread
    /// boundary the deep-recursion stack requires. What crosses is owned, thread-safe data in and
    /// the emitted map out.
    fn emit_fixture(selected_basenames: Option<Vec<String>>) -> HashMap<String, String> {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let selection = Rc::new(match selected_basenames {
                    None => RustModuleRenderSelection::RenderEveryModule,
                    Some(names) => RustModuleRenderSelection::RenderSelectedMirrors {
                        basenames: Rc::new(names.into_iter().collect()),
                    },
                });
                compile_stage0_admitted(
                    &selection_fixture(),
                    &selection,
                    Rc::new(im::Vector::new()),
                    selection_fixture_packages(),
                )
                .expect("fixture emits clean")
            })
            .expect("spawn emit thread")
            .join()
            .expect("emit thread panicked")
    }

    /// THE ADMISSION IS NOT RELAXED FOR THE FIXTURE. The same population, judged against a package
    /// graph whose rows omit its modules, still refuses as not covered: the supplied rows are what
    /// admit it, and removing them reds.
    #[test]
    fn the_fixture_refuses_as_not_covered_without_its_partition_rows() {
        let refusal = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let bare = Rc::new(
                    crate::gunbc_stage0_stage0_emitted_edge_admission::Stage0EmittedTreePackages {
                        rows: Rc::new(im::Vector::new()),
                        ..(*selection_fixture_packages()).clone()
                    },
                );
                compile_stage0_admitted(
                    &selection_fixture(),
                    &Rc::new(RustModuleRenderSelection::RenderEveryModule),
                    Rc::new(im::Vector::new()),
                    bare,
                )
                .expect_err("a population no partition row covers must refuse")
            })
            .expect("spawn emit thread")
            .join()
            .expect("emit thread panicked");
        assert!(
            refusal.contains("Stage0EmittedEdgesNotCovered") && refusal.contains("fx_beta"),
            "the refusal must be the not-covered arm naming a fixture module: {refusal}"
        );
    }

    fn beta_only() -> Option<Vec<String>> {
        Some(vec!["fx_beta.rs".to_string()])
    }

    /// THE SELECTION IS THE RENDERED SET, and the rendered bytes are the whole round's bytes.
    /// This is the unit-grain form of the planted-edit control: what a scoped round writes for a
    /// selected mirror is byte-identical to what an unscoped round writes for it, and nothing
    /// outside the selection is written at all.
    #[test]
    fn a_scoped_emission_renders_the_selection_and_nothing_else() {
        let whole = emit_fixture(None);
        let scoped = emit_fixture(beta_only());
        assert!(
            whole.contains_key("src/fx_alpha.rs"),
            "positive control: the unscoped emission renders every module"
        );
        assert!(
            !scoped.contains_key("src/fx_alpha.rs") && !scoped.contains_key("src/fx_gamma.rs"),
            "a scoped emission must not render an unselected module"
        );
        assert_eq!(
            scoped.get("src/fx_beta.rs"),
            whole.get("src/fx_beta.rs"),
            "a selected mirror must be byte-identical to what the unscoped emission produced"
        );
    }

    /// CONTENT INDEPENDENCE, DEMONSTRATED RATHER THAN ASSERTED. `emit_lib_rs_from_paths` and
    /// `emit_emitted_population_manifest` take `List<String>`, so no rendered byte can reach
    /// them -- and this is what that buys: the two aggregates are the SAME BYTES under an
    /// emission that rendered one module and one that rendered three.
    ///
    /// It is discriminating in both directions. Were either aggregate derived from the rendered
    /// files, the scoped `lib.rs` would drop two `pub mod` lines and the manifest two paths, and
    /// both halves below would go red -- which is exactly the E0583 a change-denominated
    /// emission would otherwise ship silently, since the round never re-reads a mirror it did
    /// not select.
    #[test]
    fn the_crate_module_list_and_the_manifest_are_content_independent() {
        let whole = emit_fixture(None);
        let scoped = emit_fixture(beta_only());
        assert_eq!(
            scoped.get("src/lib.rs"),
            whole.get("src/lib.rs"),
            "lib.rs is derived from paths, so a selection cannot move it"
        );
        assert_eq!(
            scoped.get("src/emitted_population.rs"),
            whole.get("src/emitted_population.rs"),
            "the manifest is derived from paths, so a selection cannot move it"
        );
        let lib = scoped
            .get("src/lib.rs")
            .expect("scoped emission still writes lib.rs");
        assert!(
            lib.contains("pub mod fx_alpha;") && lib.contains("pub mod fx_gamma;"),
            "the crate must declare modules this emission did not render: {lib}"
        );
        let manifest = scoped
            .get("src/emitted_population.rs")
            .expect("scoped emission still writes the manifest");
        assert!(
            manifest.contains("// src/fx_alpha.rs") && manifest.contains("// src/fx_gamma.rs"),
            "the manifest must declare paths this emission did not render: {manifest}"
        );
    }

    /// POPULATION IDENTITY IS NEVER SCOPED, read at the seam where a scope could have leaked into
    /// it. `generated_basenames_from_emit` reads the emitter's declaration, so the roster a
    /// scoped round hands the identity join is the roster an unscoped round hands it.
    #[test]
    fn the_emitted_roster_is_whole_under_a_scope() {
        let whole = emit_fixture(None);
        let scoped = emit_fixture(beta_only());
        let whole_roster = generated_basenames_from_emit(&whole).expect("whole roster");
        let scoped_roster = generated_basenames_from_emit(&scoped).expect("scoped roster");
        assert!(
            whole_roster.contains(&"fx_alpha.rs".to_string()),
            "positive control: the roster names every emitted mirror"
        );
        assert_eq!(
            scoped_roster, whole_roster,
            "the roster the population identity join reads may not narrow with the selection"
        );
    }

    /// AN EMPTY SELECTION RENDERS NOTHING AND STILL DECLARES EVERYTHING. It is an ordinary answer
    /// -- an edit that touches no compared mirror -- and not a spelling of RenderEveryModule.
    #[test]
    fn an_empty_selection_renders_no_module_and_declares_them_all() {
        let whole = emit_fixture(None);
        let none = emit_fixture(Some(vec![]));
        assert!(
            !none.contains_key("src/fx_beta.rs"),
            "an empty selection renders no module"
        );
        assert_eq!(
            none.get("src/lib.rs"),
            whole.get("src/lib.rs"),
            "an empty selection still declares the whole crate"
        );
    }

    /// THE PRECONDITION `module_refusals` EXACTNESS RESTS ON, PUT ON THE EXECUTED PATH.
    ///
    /// A scoped emission observes refusals only for the modules it rendered. That is exact rather
    /// than partial because an emit carrying an error diagnostic returns NO FILES -- so a
    /// committed mirror is a file some clean emit produced, and the modules a selection declined
    /// to render had nothing to say. The construction is `final_files` in
    /// `v1.compiler.compile` `emit_resolved_for_target_selected`; this is its discriminating red.
    ///
    /// WHAT IT COVERS, STATED NARROWLY: the refusing class here is a module-filename collision,
    /// which `emit_rust` refuses by returning an empty file list of its own. It establishes the
    /// implication for a refusal reachable from a fixture, not for every refusing class -- an
    /// import refusal is authored by a cross-module export failure this fixture cannot express.
    #[test]
    fn an_emit_that_refuses_hands_back_no_files() {
        let (refused, files_empty) = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let sources: Vec<Rc<SourceFile>> = [
                    (
                        "fx_a.dag",
                        "module fx.alpha\nfn a_add(a: Int, b: Int) -> Int { a + b }\n",
                    ),
                    (
                        "fx_b.dag",
                        "module fx_alpha\nfn b_add(a: Int, b: Int) -> Int { a + b }\n",
                    ),
                ]
                .iter()
                .map(|(path, content)| {
                    Rc::new(SourceFile {
                        path: path.to_string(),
                        content: content.to_string(),
                    })
                })
                .collect();
                let result = compile_sources_selected(
                    Rc::new(sources.into()),
                    RenderTarget::Rust,
                    Rc::new(RustModuleRenderSelection::RenderEveryModule),
                );
                (
                    result
                        .diagnostics
                        .iter()
                        .any(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone())),
                    result.files.is_empty(),
                )
            })
            .expect("spawn refusal thread")
            .join()
            .expect("refusal thread panicked");
        assert!(
            refused,
            "the fixture must actually refuse, or this control proves nothing"
        );
        assert!(
            files_empty,
            "an emit carrying an error diagnostic must hand back no files at all"
        );
    }

    #[test]
    fn declared_hand_maintained_row_must_name_an_existing_path() {
        let root = temp_dir("declared-row-wall");
        let src = root.join("src");
        let dest = root.join("dest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");
        for file_name in HAND_MAINTAINED_STAGE0_FILES {
            fs::write(src.join(file_name), "// planted\n").expect("plant file");
        }
        for dir_name in HAND_MAINTAINED_STAGE0_DIRS {
            fs::create_dir_all(src.join(dir_name)).expect("plant dir");
        }

        copy_hand_maintained_support(&src, &dest).expect("complete population is accepted");

        let victim = HAND_MAINTAINED_STAGE0_FILES
            .first()
            .expect("roster is non-empty");
        fs::remove_file(src.join(victim)).expect("remove one declared file");
        let err = copy_hand_maintained_support(&src, &dest)
            .expect_err("a declared row naming no file must refuse");
        assert!(
            err.contains("names no file on disk") && err.contains(victim),
            "refusal must locate the row: {err}"
        );

        let victim_dir = HAND_MAINTAINED_STAGE0_DIRS
            .first()
            .expect("dir roster is non-empty");
        fs::write(src.join(victim), "// replanted\n").expect("restore file");
        fs::remove_dir_all(src.join(victim_dir)).expect("remove one declared dir");
        let err = copy_hand_maintained_support(&src, &dest)
            .expect_err("a declared dir row naming no directory must refuse");
        assert!(
            err.contains("names no directory on disk") && err.contains(victim_dir),
            "refusal must locate the dir row: {err}"
        );
    }

    #[test]
    fn empty_population_digest_refuses() {
        let fmt = ResolvedFormatter::admit().expect("rustfmt on PATH for this test");
        let err = tree_digest_for_basenames(&fmt, Path::new("/tmp"), &[], "committed").unwrap_err();
        assert!(err.contains("empty population"));
        let err = tree_digest_from_map(&fmt, &HashMap::new(), &[]).unwrap_err();
        assert!(err.contains("empty population"));
    }

    #[test]
    fn empty_emit_population_refuses_before_agreement() {
        let reason =
            validate_compared_populations(&["foo.rs".to_string()], &[], &BTreeMap::new(), &[])
                .expect("expected refusal");
        assert!(reason.contains("zero generated surfaces"));
    }

    #[test]
    fn empty_committed_population_refuses_before_agreement() {
        let reason =
            validate_compared_populations(&[], &["foo.rs".to_string()], &BTreeMap::new(), &[])
                .expect("expected refusal");
        assert!(reason.contains("committed generated population is empty"));
    }

    #[test]
    fn seed_retained_committed_not_emitted_is_not_that_refusal() {
        let reason = validate_compared_populations(
            &["retained.rs".to_string(), "still.rs".to_string()],
            &["still.rs".to_string()],
            &BTreeMap::new(),
            &["retained.rs"],
        );
        assert!(
            reason.is_none(),
            "a frontier-rostered basename that left emit is retained, not CommittedMirrorNoLongerEmitted: {reason:?}"
        );
    }

    #[test]
    fn unrostered_committed_not_emitted_still_refuses() {
        let reason = validate_compared_populations(
            &["lost.rs".to_string(), "still.rs".to_string()],
            &["still.rs".to_string()],
            &BTreeMap::new(),
            &["retained.rs"],
        )
        .expect("expected refusal");
        assert!(
            reason.contains("committed mirror is no longer emitted") && reason.contains("lost.rs"),
            "an unrostered drop must still refuse: {reason}"
        );
        assert!(
            !reason.contains("retained.rs"),
            "the planted retained name must not appear: {reason}"
        );
    }

    // OFF-PATH DISCRIMINATION, not merge-path reachability. These plant `std_logic.rs` in the
    // committed-not-emitted slot of `validate_compared_populations` itself because the live walk
    // never does: HAND_MAINTAINED_STAGE0_FILES already subtracts it (and layout-only hosts). A
    // no-drift round never enters the arm anyway (#10795). The tests sit under
    // rust_unit_tests_off_the_merge_path. The live projected roster is the join surface;
    // `generated_stage0_filenames` is not consulted (it would also drop required_regen_host.rs).
    #[test]
    fn live_frontier_projection_is_not_the_crate_layout_filename_union() {
        assert!(
            SEED_RETENTION_FRONTIER_TOP_LEVEL_SRC_BASENAMES.contains(&"std_logic.rs"),
            "the generated join surface must carry the deliberately retained oracle"
        );
        assert!(
            HAND_MAINTAINED_STAGE0_FILES.contains(&"target_invocation_host.rs"),
            "the cheap list includes layout-only hosts"
        );
        assert!(
            !SEED_RETENTION_FRONTIER_TOP_LEVEL_SRC_BASENAMES.contains(&"target_invocation_host.rs"),
            "joining the cheap list would over-exclude this class; the frontier must not"
        );
    }

    #[test]
    fn std_logic_committed_not_emitted_is_retained_by_live_frontier_join() {
        let reason = validate_compared_populations(
            &["std_logic.rs".to_string(), "still.rs".to_string()],
            &["still.rs".to_string()],
            &BTreeMap::new(),
            SEED_RETENTION_FRONTIER_TOP_LEVEL_SRC_BASENAMES,
        );
        assert!(
            reason.is_none(),
            "std_logic.rs is SeedRetainedIntrinsic on the frontier; committed-not-emitted must not fire: {reason:?}"
        );
    }

    #[test]
    fn unrostered_drop_still_refuses_beside_std_logic() {
        let reason = validate_compared_populations(
            &[
                "std_logic.rs".to_string(),
                "lost.rs".to_string(),
                "still.rs".to_string(),
            ],
            &["still.rs".to_string()],
            &BTreeMap::new(),
            SEED_RETENTION_FRONTIER_TOP_LEVEL_SRC_BASENAMES,
        )
        .expect("expected refusal");
        assert!(
            reason.contains("committed mirror is no longer emitted") && reason.contains("lost.rs"),
            "the unrostered drop must still reach the refusal: {reason}"
        );
        assert!(
            !reason.contains("std_logic.rs"),
            "the retained oracle must not be named in the refusal: {reason}"
        );
    }

    #[test]
    fn absent_candidate_tree_refuses() {
        let tmp = temp_dir("required-regen-absent");
        let missing = tmp.join("no-such-src");
        let err = verify_candidate_tree(&missing, &["foo.rs".to_string()]).unwrap_err();
        assert!(err.contains("candidate src directory absent"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn present_candidate_tree_reports_honestly() {
        let tmp = temp_dir("required-regen-present");
        let src = tmp.join("src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("foo.rs"), "fn foo() {}\n").expect("write foo");
        verify_candidate_tree(&src, &["foo.rs".to_string()]).expect("candidate present");
        let _ = fs::remove_dir_all(&tmp);
    }
}

// ---------------------------------------------------------------------------------------------
// ONE REGEN ROUND, PRICED — `claim_executor --regen-round-cost`.
//
// The round is the convergence recipe `gunbc.generated_artifact_merge_driver`
// `generated_artifact_merge_driver_repair_steps` prints: build the seed, emit from it, install
// what drifted, rebuild from the installed seed. This driver runs that sequence ONCE, through the
// same `run_required_regen` the required phase runs (never a second emit path), with
// `v1_rt::trace_mark` armed so every phase boundary lands in one ledger, and renders the receipt
// through `gunbc.regen_round_cost` `regen_round_cost_render` via the interpreter — no Rust copy
// of the receipt format to drift from the model.
//
// IT MUTATES THE TREE, AND SAYS SO. Installing the drifted mirrors into src/v1/stage0/src is the
// step an author performs by hand today, and the rebuild measures "rebuild from the installed
// seed" only if the seed was installed; a read-only variant would price a round nobody runs. The
// receipt names the tree (HEAD sha, dirty flag) and lists every installed path by identity.
//
// THE SEED THAT EMITS IS THE SEED THAT WAS BUILT, OR THE RUN REFUSES. The emit runs in THIS
// process, whose binary is the seed build's product only if that build was a no-op on the running
// executable. Its bytes are hashed before and after the build; a changed hash means a stale
// seed's emit was measured, and the honest answer is to stop rather than report a candidate the
// built seed never produced.
// ---------------------------------------------------------------------------------------------

/// THE ONE-MIRROR DISCRIMINATOR beside the round (`//gunbc/instruments:regen-round-cost`).
///
/// The question it answers is where `compile.emit` spends its time: rendering each module, or
/// re-deriving closure-wide facts per render. It runs the SAME producer the round runs
/// (`run_required_regen_scoped`, never a second emit path) over the same corpus with a selection
/// of exactly ONE mirror, and returns that run's drained trace ledger. If emit cost is per-module
/// it collapses to roughly the whole emit over the population count; if closure-wide work
/// dominates it stays near the whole emit. Either reading is an answer, so the probe reports the
/// rows and decides nothing. It writes only under `target/` (its own candidate dir and receipt),
/// never into `src/v1/stage0/src`.
pub fn run_regen_one_mirror_emit_probe(
    basename: &str,
) -> Result<Vec<v1_rt::TraceLedgerRow>, String> {
    let scope = RegenEmissionScope::Affected {
        members: vec![basename.to_string()],
    };
    v1_rt::trace_ledger_arm();
    let outcome = run_required_regen_scoped(
        "target/stage0-regen-one-mirror-probe",
        "target/stage0-regen-one-mirror-probe-receipt.json",
        &scope,
    );
    let rows = v1_rt::trace_ledger_drain().unwrap_or_default();
    let outcome = outcome?;
    if !outcome.failures.is_empty() {
        return Err(format!(
            "refusal: the one-mirror probe's regen reported failures, so its ledger is not a \
             measurement of a clean emit: {}",
            outcome.failures.join("; ")
        ));
    }
    if rows.is_empty() {
        return Err("refusal: the one-mirror probe's trace ledger is empty".to_string());
    }
    Ok(rows.into_iter().collect())
}

pub struct RegenRoundCostOutcome {
    /// The rendered receipt, already printed by the caller's contract to stderr.
    pub rendered: String,
    /// Where the same bytes were written, so a remote run can bring them back by path.
    pub receipt_path: PathBuf,
    /// Anything that stopped the round short of install/rebuild/diff, or that the regen
    /// reported as a failure. Non-empty means the round is not a clean round.
    pub round_failures: Vec<String>,
    /// The base commit round 1 seeded from, when this round was asked to bootstrap (DESIGN §5,
    /// RFM #14963). The receipt text names it too; carrying it lets a
    /// caller (and a test) assert WHICH commit the round seeded from without parsing that text.
    /// `None` is not "it failed": it means this round was never a heal round.
    pub base_mirror_commit: Option<String>,
}

pub const REGEN_ROUND_COST_RECEIPT_REL: &str = "target/stage0-regen-round-cost.txt";
/// Set on the single re-exec after the seed build replaced the running executable.
const REGEN_ROUND_COST_REEXEC_ENV: &str = "GUNBC_REGEN_ROUND_COST_REEXEC";
/// The heal round's base-mirror authority (DESIGN §5, RFM PR #14963): the base commit whose
/// COMMITTED stage0 mirror population bootstraps round 1's seed, carried as an env value so the
/// ref never reaches argv — the admit gate refuses any argv word containing `${{`.
/// THE ROUND'S BOOTSTRAP AUTHORITY, its own name: the pull request's base COMMIT (`base.sha`),
/// bound by `gunbc.heal_workflow` `heal_round_base_mirror_env`. Distinct from `HEAL_BASE_REF`,
/// which is the candidate step's branch NAME (`gunbc.heal_candidate` `heal_candidate_base_ref_env`)
/// and answers a different question.
const REGEN_ROUND_COST_BASE_MIRROR_ENV: &str = "HEAL_BASE_MIRROR_COMMIT";
const REGEN_ROUND_COST_PRODUCER: &str = "claim_executor --regen-round-cost";
const REGEN_ROUND_COST_ENTRY_UNDER_ROOT: &str = "gunbc/regen_round_cost.dag";

/// Workspace-relative directory holding the stage0 mirror population. The same literal the
/// journal, the diff, and the emitter's population walk use; the base bootstrap spells it once.
const STAGE0_SRC_REL: &str = "src/v1/stage0/src";

/// Workspace-relative directory holding the WHOLE SEED CRATE — `src`, `tests`, `testdata`,
/// `Cargo.toml`, `build.rs`. The round-1 bootstrap restores this, not the mirror directory alone:
/// what cargo compiles is the crate, so what round 1 must pin is the crate. See
/// `crate_population_at_commit`.
const STAGE0_CRATE_REL: &str = "src/v1/stage0";

struct CargoBuildObservation {
    compiled_crates: u64,
    compiled_packages: Vec<String>,
    /// cargo's own fingerprint log lines for this build (`CARGO_LOG=cargo::core::compiler::
    /// fingerprint=info`), which say WHY each unit was rebuilt. The scope guard can otherwise only
    /// name the packages it refused, not why they were dirty — the #14876 / #15310 / #15441 mtime
    /// mechanism had to be INFERRED from the guard's package list because no fingerprint line was
    /// ever captured. Kept in memory; surfaced only on a refusal.
    fingerprint_reasons: Vec<String>,
}

const REGEN_CONVERGENCE_SCHEMA: &str = "gunbc.regen_convergence_transaction.v1";
const REGEN_CONVERGENCE_JOURNAL_REL: &str = "target/regen-convergence-journal";
const REGEN_CONVERGENCE_RECEIPT_REL: &str = "target/regen-convergence-transaction.json";

#[derive(Debug, Serialize, serde::Deserialize)]
struct RegenConvergenceJournalEntry {
    relative_path: String,
    backup_name: String,
    pre_stage_state: RegenPreStageState,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
enum RegenPreStageState {
    PresentBeforeInstall { digest: String },
    AbsentBeforeInstall,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct RegenConvergenceJournal {
    schema: String,
    starting_commit: String,
    source_authority_digest: String,
    stage_plan_authority_digest: String,
    checkpoint_id: String,
    entries: Vec<RegenConvergenceJournalEntry>,
}

/// What a convergence checkpoint is about: the commit, the stage0 input closure's authority digest,
/// and the stage plan's. ONE value per round (`round_subject` in `run_regen_round_cost`): produced
/// once by the checkpoint-subject child, carried across the re-exec in `SeedHandoff`, and passed to
/// every journal write and restore of the round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegenConvergenceCheckpointSubject {
    starting_commit: String,
    source_authority_digest: String,
    stage_plan_authority_digest: String,
}

struct RegenConvergenceModel {
    graph: Rc<crate::v1_compiler_compile::ResolvedGraph>,
    indices: Rc<im::HashMap<String, Rc<crate::v1_compiler_compile::NewlineIndex>>>,
}

/// THE ORCHESTRATOR'S ONE WAY TO LOAD AN INTERPRETED MODEL: the entry's own import closure, acquired
/// through the source layout (`resolve_entry_graph_demand_acquired`, the route `gunbc run` takes), never
/// the process-wide whole-tree index. Measured on main c0a1ba43, release, in the order the re-exec'd
/// orchestrator runs: `RegenConvergenceModel::load` through `process_shared_index_or_refuse` took
/// 169 s and left 10.13 GB resident to resolve a 26-module closure that the demand-acquired route
/// resolves in 2.9 s at 150 MB -- the orchestrator's whole pre-generation footprint (heal runs read
/// 10.69-10.70 GB at generation 0). A whole-tree index serving a handful of small entries is the
/// duplication DESIGN section 2 orders removed before any cache is considered; every orchestrator-side
/// model load goes through here so none of them rebuilds it later, between generations.
fn resolve_orchestrator_model(
    source_roots: &[String],
    entry: &str,
) -> Result<
    (
        Rc<crate::v1_compiler_compile::ResolvedGraph>,
        Rc<im::HashMap<String, Rc<crate::v1_compiler_compile::NewlineIndex>>>,
    ),
    String,
> {
    super::resolve_entry_graph_demand_acquired(source_roots, entry)
}

impl RegenConvergenceModel {
    fn load(source_roots: &[String]) -> Result<Self, String> {
        let entry = source_roots
            .iter()
            .map(|root| Path::new(root).join("workflow/regen_convergence_transaction.dag"))
            .find(|path| path.is_file())
            .ok_or_else(|| {
                "refusal: convergence transaction model is outside source roots".to_string()
            })?;
        let (graph, indices) = resolve_orchestrator_model(source_roots, &entry.to_string_lossy())
            .map_err(|e| {
            format!("refusal: convergence transaction model did not resolve: {e}")
        })?;
        Ok(Self { graph, indices })
    }

    fn context(&self) -> crate::v1_interpreter::InterpContext {
        super::make_eval_context(
            &self.graph,
            self.indices.clone(),
            crate::v1_interpreter::ExecutionMode::Hermetic,
        )
    }
}

/// THE LIVENESS AUTHORITY, loaded once before generation 0 as orchestration state and asked at
/// every supervisor sample: `gunbc.host.memory_stall_refusal`. The seed supplies the slot readings
/// it observed and evaluates the authority's own window construction, verdict and texts -- the
/// same route `regen_admit_generation_begin` takes through `RegenConvergenceModel` -- so no
/// threshold, arm or rendering is restated in Rust.
pub(crate) struct SupervisedSubjectModel {
    graph: Rc<crate::v1_compiler_compile::ResolvedGraph>,
    indices: Rc<im::HashMap<String, Rc<crate::v1_compiler_compile::NewlineIndex>>>,
}

impl SupervisedSubjectModel {
    fn load(source_roots: &[String]) -> Result<Self, String> {
        let entry = source_roots
            .iter()
            .map(|root| Path::new(root).join("gunbc/host/memory_stall_refusal.dag"))
            .find(|path| path.is_file())
            .ok_or_else(|| {
                "refusal: the supervised-subject liveness model (gunbc.host.memory_stall_refusal) \
                 is outside source roots"
                    .to_string()
            })?;
        let (graph, indices) = resolve_orchestrator_model(source_roots, &entry.to_string_lossy())
            .map_err(|e| {
            format!("refusal: the supervised-subject liveness model did not resolve: {e}")
        })?;
        Ok(Self { graph, indices })
    }

    /// One evaluation of the authority over the window between two slot readings: the window's own
    /// line, and the refusal text when the verdict refuses. An authority that does not answer is a
    /// refusal of the generation, never an admission.
    fn judge(
        &self,
        base: &crate::memory_governor::SlotStallCounters,
        now: &crate::memory_governor::SlotStallCounters,
        wall_ms: u64,
    ) -> Result<(String, Option<String>), String> {
        use crate::v1_interpreter::{self, str_value, Value};
        let ctx = super::make_eval_context(
            &self.graph,
            self.indices.clone(),
            crate::v1_interpreter::ExecutionMode::Hermetic,
        );
        let measure = |count: u64| Value::Record {
            type_name: ctx.sym("Measure"),
            fields: Rc::new(vec![(ctx.sym("count"), Value::Int(count as i64))]),
        };
        let reading = |c: &crate::memory_governor::SlotStallCounters| Value::Record {
            type_name: ctx.sym("SupervisedSlotReading"),
            fields: Rc::new(vec![
                (ctx.sym("major_faults"), Value::Int(c.major_faults as i64)),
                (ctx.sym("user_cpu"), measure(c.user_usec / 1000)),
                (
                    ctx.sym("envelope_events"),
                    Value::Int(c.envelope_events as i64),
                ),
                (ctx.sym("pressure_body"), str_value(c.pressure_body.clone())),
            ]),
        };
        let call = |name: &str, args: &[(Option<String>, Value)]| {
            v1_interpreter::with_active_context(&ctx, || {
                v1_interpreter::run_in_context_with_args(&ctx, name, args, false)
            })
            .map_err(|e| {
                format!("refusal: gunbc.host.memory_stall_refusal {name} did not answer: {e}")
            })
        };
        let text = |v: Value| match &v {
            Value::Str(s) => Ok(s.as_ref().to_string()),
            other => Err(format!(
                "refusal: the liveness model returned {} where it renders text",
                other.type_label_public()
            )),
        };
        let window = call(
            "supervised_subject_window_between",
            &[
                (Some("base".to_string()), reading(base)),
                (Some("now".to_string()), reading(now)),
                (Some("wall".to_string()), measure(wall_ms)),
            ],
        )?;
        let line = text(call(
            "supervised_subject_window_text",
            &[(Some("w".to_string()), window.clone())],
        )?)?;
        let verdict = call(
            "supervised_subject_verdict",
            &[(Some("w".to_string()), window)],
        )?;
        let refuses = match call(
            "supervised_subject_verdict_refuses",
            &[(Some("v".to_string()), verdict.clone())],
        )? {
            Value::Bool(b) => b,
            other => {
                return Err(format!(
                    "refusal: supervised_subject_verdict_refuses returned {}",
                    other.type_label_public()
                ))
            }
        };
        if !refuses {
            return Ok((line, None));
        }
        let refusal = text(call(
            "supervised_subject_refusal_text",
            &[(Some("v".to_string()), verdict)],
        )?)?;
        Ok((line, Some(refusal)))
    }
}

#[derive(Debug, Serialize)]
struct RegenConvergenceSurfaceReceipt {
    declaring_module: String,
    projected_path: String,
    pre_stage_state: RegenPreStageState,
    candidate_digest: String,
    installed_digest: String,
    standing: RegenSurfaceExecutionStandingReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum RegenSurfaceExecutionStandingReceipt {
    Planned,
    Executed,
    TerminalPassed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum RegenConvergenceStageKindReceipt {
    PromoteGenerationInputs,
    InstallSeedCompatibilityCut,
    PublishNonSeedOutputs,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum RegenConvergenceDeferredReasonReceipt {
    AwaitingPromotedProducer,
    AwaitingSeedCompatibilityCut,
    AwaitingBuildableSeedGeneration,
}

#[derive(Debug, Serialize)]
struct RegenConvergenceDeferredSurfaceReceipt {
    projected_path: String,
    reason: RegenConvergenceDeferredReasonReceipt,
}

#[derive(Debug, Serialize)]
enum RegenConvergenceBuildTerminalReceipt {
    Passed,
}

#[derive(Debug, Serialize)]
enum RegenConvergenceFixedPointReceipt {
    Reached,
}

#[derive(Debug, Serialize)]
struct RegenConvergenceStageReceipt {
    receipt_id: String,
    ordinal: usize,
    kind: RegenConvergenceStageKindReceipt,
    input_seed_digest: String,
    input_candidate_tree_id: String,
    input_candidate_tree_digest: String,
    producer_generation_id: String,
    surfaces: Vec<RegenConvergenceSurfaceReceipt>,
    deferred_surfaces: Vec<RegenConvergenceDeferredSurfaceReceipt>,
    dependency_closure_id: String,
    build_target: String,
    build_invocation: String,
    build_terminal: RegenConvergenceBuildTerminalReceipt,
    build_compiled_crates: u64,
    build_packages: Vec<String>,
    assembly_equivalence_authority: String,
    output_seed_digest: String,
    next_generation_receipt_id: String,
}

#[derive(Debug, Serialize)]
struct RegenConvergenceReceipt {
    schema_version: String,
    starting_commit: String,
    source_authority_digest: String,
    starting_generated_surface_digest: String,
    stage_plan_authority_digest: String,
    generation_role_authority_digest: String,
    ownership_authority_digest: String,
    initial_seed_digest: String,
    ordered_stage_receipt_ids: Vec<String>,
    stages: Vec<RegenConvergenceStageReceipt>,
    terminal_seed_digest: String,
    terminal_surface_digest: String,
    fixed_point_verdict: RegenConvergenceFixedPointReceipt,
}

fn bytes_digest(bytes: &[u8]) -> String {
    format!("fnv1a64:{}", v1_rt::bytes_identity_hash(bytes))
}

fn path_digest(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| bytes_digest(&bytes))
        .map_err(|e| format!("read {} for digest: {e}", path.display()))
}

fn observe_complete_candidate_artifact_population_from_entries<I>(
    candidate_src: &Path,
    phase: &str,
    entries: I,
) -> Result<Vec<String>, String>
where
    I: IntoIterator<Item = Result<PathBuf, String>>,
{
    let mut population = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| {
            format!(
                "CandidateManifestPopulationUnreadable: candidate_path={} phase={phase} error={error}",
                candidate_src.display()
            )
        })?;
        let relative = path.strip_prefix(candidate_src).map_err(|error| {
            format!(
                "CandidateManifestPopulationUnreadable: candidate_path={} phase={phase} error=observed file {} is outside the candidate root: {error}",
                candidate_src.display(),
                path.display()
            )
        })?;
        population.push(
            relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
        );
    }
    population.sort();
    population.dedup();
    Ok(population)
}

fn observe_complete_candidate_artifact_population(
    candidate_src: &Path,
    phase: &str,
) -> Result<Vec<String>, String> {
    fn visit(candidate_src: &Path, directory: &Path, entries: &mut Vec<Result<PathBuf, String>>) {
        let directory_entries = match fs::read_dir(directory) {
            Ok(value) => value,
            Err(error) => {
                entries.push(Err(format!("read {}: {error}", directory.display())));
                return;
            }
        };
        for entry in directory_entries {
            let entry = match entry {
                Ok(value) => value,
                Err(error) => {
                    entries.push(Err(format!(
                        "read directory entry under {}: {error}",
                        directory.display()
                    )));
                    continue;
                }
            };
            let path = entry.path();
            match entry.file_type() {
                Ok(file_type) if file_type.is_dir() => visit(candidate_src, &path, entries),
                Ok(file_type) if file_type.is_file() => entries.push(Ok(path)),
                Ok(_) => entries.push(Err(format!(
                    "unsupported non-file candidate entry {}",
                    path.display()
                ))),
                Err(error) => entries.push(Err(format!(
                    "read candidate entry type {}: {error}",
                    path.display()
                ))),
            }
        }
    }

    let mut entries = Vec::new();
    visit(candidate_src, candidate_src, &mut entries);
    observe_complete_candidate_artifact_population_from_entries(candidate_src, phase, entries)
}

fn candidate_artifact_tree_digest(
    candidate_src: &Path,
    paths: &[String],
    label: &str,
) -> Result<String, String> {
    if paths.is_empty() {
        return Err(format!(
            "CandidateManifestPopulationMismatch: cannot compute {label} over an empty artifact"
        ));
    }
    let mut payload = String::new();
    for relative_path in paths {
        payload.push_str(relative_path);
        payload.push('\0');
        payload.push_str(&path_digest(&candidate_src.join(relative_path))?);
        payload.push('\n');
    }
    Ok(bytes_digest(payload.as_bytes()))
}

fn candidate_manifest_aggregate(
    producer_seed_digest: &str,
    generation_id: &str,
    candidate_tree_id: &str,
    candidate_tree_digest: &str,
    surfaces: &[RegenCandidateManifestSurface],
) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(
        REGEN_CONVERGENCE_SCHEMA,
        producer_seed_digest,
        generation_id,
        candidate_tree_id,
        candidate_tree_digest,
        surfaces,
    ))
    .map_err(|e| format!("encode candidate manifest identity: {e}"))?;
    Ok(bytes_digest(&bytes))
}

fn produce_candidate_manifest(
    candidate_src: &Path,
    selected_basenames: &[String],
    emitted_artifact_paths: &BTreeSet<String>,
    basename_to_module: &HashMap<String, String>,
    producer_seed_digest: &str,
    candidate_tree_id: &str,
) -> Result<RegenCandidateManifest, String> {
    let selected = selected_basenames.iter().cloned().collect::<BTreeSet<_>>();
    let bootstrap_mirrors = HAND_MAINTAINED_STAGE0_FILES
        .iter()
        .map(|name| (*name).to_string())
        .collect::<BTreeSet<_>>();
    let names =
        observe_complete_candidate_artifact_population(candidate_src, "manifest-production")?;
    let generation_id =
        bytes_digest(format!("{producer_seed_digest}:{candidate_tree_id}").as_bytes());
    let surfaces = names
        .iter()
        .map(|basename| {
            let bootstrap_directory = HAND_MAINTAINED_STAGE0_DIRS
                .iter()
                .find(|directory| {
                    basename
                        .strip_prefix(**directory)
                        .is_some_and(|suffix| suffix.starts_with('/'))
                });
            let (declaring_module, role) = if bootstrap_mirrors.contains(basename)
                || bootstrap_directory.is_some()
            {
                (
                    basename
                        .strip_suffix(".rs")
                        .unwrap_or(basename)
                        .replace('/', "::"),
                    RegenCandidateManifestSurfaceRole::BootstrapSourceMirror,
                )
            } else if selected.contains(basename) {
                (
                    basename_to_module.get(basename).cloned().ok_or_else(|| {
                        format!(
                            "SurfaceOwnershipUnresolved: candidate manifest surface {basename} has no \
                             declaring module"
                        )
                    })?,
                    RegenCandidateManifestSurfaceRole::GeneratedSurface,
                )
            } else if emitted_artifact_paths.contains(basename) {
                // Non-Rust aggregate products do not have a same-named DAG module. Their
                // authority is the emitter transaction that returned this exact path and whose
                // writer installed it into this candidate tree. A path merely observed on disk
                // cannot enter this arm.
                (
                    "v1.compiler.emit_rust".to_string(),
                    RegenCandidateManifestSurfaceRole::GeneratedSurface,
                )
            } else {
                return Err(format!(
                    "CandidateManifestPopulationMismatch: candidate artifact surface {basename} is neither a generated surface nor a modeled bootstrap-source mirror"
                ));
            };
            Ok(RegenCandidateManifestSurface {
                declaring_module,
                projected_path: basename.clone(),
                content_digest: path_digest(&candidate_src.join(basename))?,
                role,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let complete_candidate_tree_digest =
        candidate_artifact_tree_digest(candidate_src, &names, "complete candidate manifest")?;
    let aggregate_digest = candidate_manifest_aggregate(
        producer_seed_digest,
        &generation_id,
        candidate_tree_id,
        &complete_candidate_tree_digest,
        &surfaces,
    )?;
    Ok(RegenCandidateManifest {
        producer_seed_digest: producer_seed_digest.to_string(),
        generation_id,
        candidate_tree_id: candidate_tree_id.to_string(),
        candidate_tree_digest: complete_candidate_tree_digest,
        surfaces,
        aggregate_digest,
    })
}

fn admit_candidate_generation_from_model(
    model: &RegenConvergenceModel,
    manifest: &RegenCandidateManifest,
    current_seed_digest: &str,
) -> Result<(), String> {
    use crate::v1_interpreter::{self, str_value, Value};
    let ctx = model.context();
    let identities = manifest
        .surfaces
        .iter()
        .map(|surface| Value::Record {
            type_name: ctx.sym("RegenSurfaceIdentity"),
            fields: Rc::new(vec![
                (
                    ctx.sym("declaring_module"),
                    str_value(&surface.declaring_module),
                ),
                (
                    ctx.sym("projected_path"),
                    str_value(&surface.projected_path),
                ),
            ]),
        })
        .collect::<Vec<_>>();
    let observation = Value::Record {
        type_name: ctx.sym("RegenCandidateGenerationObservation"),
        fields: Rc::new(vec![
            (
                ctx.sym("planned_surfaces"),
                Value::List(Rc::new(identities.into())),
            ),
            (
                ctx.sym("current_seed_digest"),
                str_value(current_seed_digest),
            ),
            (
                ctx.sym("manifest_producer_seed_digest"),
                str_value(&manifest.producer_seed_digest),
            ),
            (
                ctx.sym("observed_candidate_digest"),
                str_value(&manifest.aggregate_digest),
            ),
        ]),
    };
    let admission = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_admit_candidate_generation",
            &[(Some("observation".to_string()), observation)],
            false,
        )
    })
    .map_err(|e| format!("refusal: candidate generation admission did not answer: {e}"))?;
    let label = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_candidate_generation_admission_label",
            &[(Some("admission".to_string()), admission)],
            false,
        )
    })
    .map_err(|e| format!("refusal: candidate generation admission label failed: {e}"))?;
    match label {
        Value::Str(ref label) if label.as_ref() == "Admitted" => Ok(()),
        Value::Str(ref label) => Err(format!("candidate generation admission {label}")),
        other => Err(format!(
            "refusal: candidate generation admission label returned {}",
            other.type_label_public()
        )),
    }
}

fn admit_candidate_manifest(
    model: &RegenConvergenceModel,
    candidate_src: &Path,
    manifest: &RegenCandidateManifest,
    expected_seed_digest: &str,
) -> Result<HashMap<String, RegenCandidateManifestSurface>, String> {
    admit_candidate_generation_from_model(model, manifest, expected_seed_digest)?;
    let aggregate = candidate_manifest_aggregate(
        &manifest.producer_seed_digest,
        &manifest.generation_id,
        &manifest.candidate_tree_id,
        &manifest.candidate_tree_digest,
        &manifest.surfaces,
    )?;
    if aggregate != manifest.aggregate_digest {
        return Err(format!(
            "CandidateManifestDigestMismatch: recorded {} observed {}",
            manifest.aggregate_digest, aggregate
        ));
    }
    let observed_population =
        observe_complete_candidate_artifact_population(candidate_src, "manifest-admission")?;
    let recorded_population = manifest
        .surfaces
        .iter()
        .map(|surface| surface.projected_path.clone())
        .collect::<Vec<_>>();
    if observed_population != recorded_population {
        return Err(format!(
            "CandidateManifestPopulationMismatch: recorded {recorded_population:?} observed {observed_population:?}"
        ));
    }
    let observed_tree_digest =
        candidate_artifact_tree_digest(candidate_src, &recorded_population, "candidate manifest")?;
    if observed_tree_digest != manifest.candidate_tree_digest {
        return Err(format!(
            "CandidateManifestTreeDigestMismatch: recorded {} observed {}",
            manifest.candidate_tree_digest, observed_tree_digest
        ));
    }
    let mut prior = "";
    let mut admitted = HashMap::new();
    for surface in &manifest.surfaces {
        if !prior.is_empty() && prior >= surface.projected_path.as_str() {
            return Err(format!(
                "CandidateManifestPopulationNotStrictlySorted: {} then {}",
                prior, surface.projected_path
            ));
        }
        prior = &surface.projected_path;
        let observed = path_digest(&candidate_src.join(&surface.projected_path))?;
        if observed != surface.content_digest {
            return Err(format!(
                "CandidateManifestSurfaceDigestMismatch: {} recorded {} observed {}",
                surface.projected_path, surface.content_digest, observed
            ));
        }
        admitted.insert(surface.projected_path.clone(), surface.clone());
    }
    Ok(admitted)
}

/// Evaluate one decision function of the rebuild-scope module. The host effects the answer; the
/// decision lives in `gunbc.stage0_partition_rebuild_scope`.
fn rebuild_scope_decision(
    source_roots: &[String],
    function: &str,
    args: Vec<(Option<String>, ModelValue)>,
) -> Result<ModelValue, String> {
    rebuild_scope_decision_with(source_roots, function, |_| Ok(args))
}

/// `rebuild_scope_decision` for arguments that are typed model records, which carry symbols of the
/// evaluation context and so can only be built once it exists.
fn rebuild_scope_decision_with(
    source_roots: &[String],
    function: &str,
    args: impl FnOnce(
        &crate::v1_interpreter::InterpContext,
    ) -> Result<Vec<(Option<String>, ModelValue)>, String>,
) -> Result<ModelValue, String> {
    use crate::v1_interpreter::{self, ExecutionMode};
    let entry = round_cost_entry(source_roots)?;
    let (graph, indices) = resolve_orchestrator_model(source_roots, &entry).map_err(|e| {
        format!("refusal: {entry} did not resolve, so {function} has no decider: {e}")
    })?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let args = args(&ctx)?;
    v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, function, &args, false)
    })
    .map_err(|e| format!("refusal: {function} did not evaluate: {e}"))
}

/// The modules the host shell package declares PLUS each of its crate roots, read from the files
/// that declare them: the rebuild-scope fold owns a mirror through this declaration alone, so a
/// root file is a fact this read supplies (its stem), not a spelling the fold recognises. A
/// package has one root per target: `lib.rs` AND the path of every `[[bin]]` in its manifest
/// (the generated `main.rs` is the `gunbc` bin's root and no `mod` declares it, so reading
/// `lib.rs` alone left it `MirrorHasNoOwningPackage`). `closure_modules` reads each root's
/// declarations transitively, so a hand file any root declares is owned too. An unreadable
/// manifest or bin root is returned, never rendered as an empty roster.
fn host_shell_scope_modules(lib_rs: &Path, manifest: &Path) -> Result<Vec<String>, String> {
    let package_dir = manifest
        .parent()
        .ok_or_else(|| format!("refusal: manifest {} has no directory", manifest.display()))?;
    let mut roots = vec![lib_rs.to_path_buf()];
    let text = std::fs::read_to_string(manifest)
        .map_err(|e| format!("refusal: shell manifest {}: {e}", manifest.display()))?;
    // ONE READER of the manifest's bin roster (`cli_run`), not a second TOML walk here.
    match super::parse_stage0_cargo_manifest_bin_paths(&text) {
        super::Stage0CargoManifestBinParse::Parsed {
            authored_relative_paths,
        } => {
            for path in authored_relative_paths {
                roots.push(package_dir.join(path));
            }
        }
        super::Stage0CargoManifestBinParse::Refused { detail } => {
            return Err(format!(
                "refusal: shell manifest {}: {detail}",
                manifest.display()
            ));
        }
    }
    let mut modules = Vec::new();
    for root in &roots {
        modules.extend(super::emitted_closure_compile_host::closure_modules(root)?);
        let stem = root
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("refusal: shell root {} has no file stem", root.display()))?;
        modules.push(stem.to_string());
    }
    Ok(modules)
}

fn stage_rebuild_is_whole_compiler(
    source_roots: &[String],
    boundary_files: &[String],
) -> Result<bool, String> {
    match rebuild_scope_decision(
        source_roots,
        "stage0_stage_rebuild_is_whole_compiler",
        vec![(
            Some("boundary_files".to_string()),
            model_string_list(boundary_files),
        )],
    )? {
        ModelValue::Bool(value) => Ok(value),
        other => Err(format!(
            "refusal: stage0_stage_rebuild_is_whole_compiler returned {} instead of Bool",
            other.type_label_public()
        )),
    }
}

fn regen_convergence_generation_bound(source_roots: &[String]) -> Result<i64, String> {
    match rebuild_scope_decision(
        source_roots,
        "regen_convergence_generation_bound",
        Vec::new(),
    )? {
        ModelValue::Int(value) => Ok(value),
        other => Err(format!(
            "refusal: regen_convergence_generation_bound returned {} instead of Int",
            other.type_label_public()
        )),
    }
}

fn regen_convergence_bound_exceeded(
    source_roots: &[String],
    generation_ordinal: usize,
) -> Result<bool, String> {
    match rebuild_scope_decision(
        source_roots,
        "regen_convergence_bound_exceeded",
        vec![(
            Some("generation_ordinal".to_string()),
            ModelValue::Int(generation_ordinal as i64),
        )],
    )? {
        ModelValue::Bool(value) => Ok(value),
        other => Err(format!(
            "refusal: regen_convergence_bound_exceeded returned {} instead of Bool",
            other.type_label_public()
        )),
    }
}

fn regen_convergence_journal_path(workspace: &Path) -> PathBuf {
    workspace.join(REGEN_CONVERGENCE_JOURNAL_REL)
}

/// THE ONE PRODUCER of a round's checkpoint subject. Its source digest is taken over the stage0 input
/// closure (`regen_input_sources`), which builds the whole-tree `build_multi_entry_index`, so only the
/// checkpoint-subject child calls it (`run_checkpoint_subject`); the orchestrator consumes the value.
fn current_convergence_checkpoint_subject(
    workspace: &Path,
) -> Result<RegenConvergenceCheckpointSubject, String> {
    let (starting_commit, stage_plan_authority_digest) = checkpoint_subject_live_facts(workspace)?;
    Ok(RegenConvergenceCheckpointSubject {
        starting_commit,
        source_authority_digest: authority_digest_from_sources(&super::regen_input_sources(
            workspace,
        )?)?,
        stage_plan_authority_digest,
    })
}

const STAGE_PLAN_AUTHORITY_REL: &str = "src/v2/workflow/regen_convergence_transaction.dag";

/// The checkpoint subject's two CHEAP facts, read live: the commit and the stage plan's digest. The
/// producer reads them here, and so does every journal write and restore
/// (`admit_subject_live_facts`), so the carried subject's cheap facts are re-observed at each
/// checkpoint boundary rather than assumed. The expensive fact, the source closure's digest, is
/// re-observed by every generation instead (`admit_generation_authority`).
fn checkpoint_subject_live_facts(workspace: &Path) -> Result<(String, String), String> {
    Ok((
        git_head_sha(workspace)?,
        path_digest(&workspace.join(STAGE_PLAN_AUTHORITY_REL))?,
    ))
}

/// A journal write or restore under the carried subject first re-reads its cheap facts. A HEAD that
/// moved, or a stage plan edited, during the round refuses: the carried subject no longer names the
/// round, and journalling or restoring under it would checkpoint the wrong transaction.
fn admit_subject_live_facts(
    workspace: &Path,
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<(), String> {
    let (commit, plan) = checkpoint_subject_live_facts(workspace)?;
    if commit == subject.starting_commit && plan == subject.stage_plan_authority_digest {
        return Ok(());
    }
    Err(format!(
        "CheckpointSubjectDriftedMidRound: at a checkpoint boundary the tree reads \
         (commit={commit}, plan={plan}), but the round's checkpoint subject carries \
         (commit={}, plan={}); the carried subject no longer names the round",
        subject.starting_commit, subject.stage_plan_authority_digest
    ))
}

fn convergence_checkpoint_id(journal: &RegenConvergenceJournal) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(
        &journal.schema,
        &journal.starting_commit,
        &journal.source_authority_digest,
        &journal.stage_plan_authority_digest,
        &journal.entries,
    ))
    .map_err(|e| format!("encode convergence checkpoint identity: {e}"))?;
    Ok(bytes_digest(&bytes))
}

/// Return every journalled path to the byte recorded before the round, and report WHICH were put
/// back and WHICH were removed (workspace-relative). The paths are returned rather than re-derived by
/// a caller so that the journal keeps exactly one reader (§3): a round that seeded the base snapshot
/// prints what its rollback did, and the partition-rebuild decision is fed the rewritten list — the
/// paths whose mtime this restore moved are exactly the ones cargo will see as changed. A removed
/// path is NOT in that second list: the head's commit does not track it, so it is not a mirror whose
/// new bytes round 2 must rebuild around.
/// WHAT THE JOURNAL RESTORE DID, in the two actions it can take per entry. A journalled path is
/// either put back by writing the bytes the round found there (`rewritten`), or taken away because
/// the round found NOTHING there and the round itself created the file (`removed`, the
/// `AbsentBeforeInstall` shape). They are kept apart because the receipt has to say which happened:
/// a path the head's commit does not track is removed, and reporting that as a rewrite would claim
/// the round restored a byte the head never had.
#[derive(Debug, Default, Clone)]
struct RegenJournalRestore {
    rewritten: Vec<String>,
    removed: Vec<String>,
}

fn restore_regen_convergence_journal_for_subject(
    workspace: &Path,
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<RegenJournalRestore, String> {
    let root = regen_convergence_journal_path(workspace);
    let manifest_path = root.join("journal.json");
    if !manifest_path.is_file() {
        return Ok(RegenJournalRestore::default());
    }
    admit_subject_live_facts(workspace, subject)?;
    let manifest: RegenConvergenceJournal = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|e| format!("read {}: {e}", manifest_path.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;
    if manifest.schema != REGEN_CONVERGENCE_SCHEMA {
        return Err(format!(
            "rollback/restore refuses: journal schema {} is not {}",
            manifest.schema, REGEN_CONVERGENCE_SCHEMA
        ));
    }
    if manifest.starting_commit != subject.starting_commit
        || manifest.source_authority_digest != subject.source_authority_digest
        || manifest.stage_plan_authority_digest != subject.stage_plan_authority_digest
        || convergence_checkpoint_id(&manifest)? != manifest.checkpoint_id
    {
        return Err(format!(
            "CheckpointSubjectMismatch: journal(commit={}, source={}, plan={}, checkpoint={}) \
             current(commit={}, source={}, plan={})",
            manifest.starting_commit,
            manifest.source_authority_digest,
            manifest.stage_plan_authority_digest,
            manifest.checkpoint_id,
            subject.starting_commit,
            subject.source_authority_digest,
            subject.stage_plan_authority_digest,
        ));
    }
    // Validate every backup before creating a temporary restore file or touching a destination.
    for entry in &manifest.entries {
        if let RegenPreStageState::PresentBeforeInstall { digest } = &entry.pre_stage_state {
            let backup = root.join(&entry.backup_name);
            let observed = path_digest(&backup)?;
            if observed != *digest {
                return Err(format!(
                    "CheckpointArtifactDigestMismatch: {} recorded {} observed {}",
                    entry.backup_name, digest, observed
                ));
            }
        }
    }
    let restore_tmp = root.join("validated-restore");
    if restore_tmp.exists() {
        fs::remove_dir_all(&restore_tmp)
            .map_err(|e| format!("clear restore temp {}: {e}", restore_tmp.display()))?;
    }
    fs::create_dir_all(&restore_tmp)
        .map_err(|e| format!("create restore temp {}: {e}", restore_tmp.display()))?;
    for entry in &manifest.entries {
        if matches!(
            entry.pre_stage_state,
            RegenPreStageState::PresentBeforeInstall { .. }
        ) {
            let backup = root.join(&entry.backup_name);
            fs::copy(&backup, restore_tmp.join(&entry.backup_name))
                .map_err(|e| format!("prepare validated restore {}: {e}", backup.display()))?;
        }
    }
    let mut restored = RegenJournalRestore::default();
    for entry in &manifest.entries {
        let destination = workspace.join(&entry.relative_path);
        if matches!(
            entry.pre_stage_state,
            RegenPreStageState::PresentBeforeInstall { .. }
        ) {
            let prepared = restore_tmp.join(&entry.backup_name);
            // SKIP A PATH THAT WAS NEVER MOVED. If the destination already holds the bytes the
            // journal backed up, the rename is redundant work, and it bumps an mtime that cargo's
            // path-dependency fingerprint reads as a change — the same defect the round-1 seed now
            // avoids (#14876 specimen: an excluded crate recompiled because a restore rewrote a file
            // whose bytes had never changed). DESIGN §2.
            if destination_bytes_equal(&destination, &prepared)? {
                continue;
            }
            fs::rename(&prepared, &destination).map_err(|e| {
                format!(
                    "rollback/restore refuses: atomic rename {} -> {}: {e}",
                    prepared.display(),
                    destination.display()
                )
            })?;
            restored.rewritten.push(entry.relative_path.clone());
        } else if destination.exists() {
            fs::remove_file(&destination).map_err(|e| {
                format!(
                    "rollback/restore refuses: remove new {}: {e}",
                    destination.display()
                )
            })?;
            // THE PATH IS NAMED IN THE RETURN, NOT ONLY REMOVED. A journalled
            // `AbsentBeforeInstall` entry is the shape a round-1 bootstrap leaves when the PR DELETED
            // a path: the restore's action on it is removal, not a rewrite, and a teardown line that
            // counted it among "paths rewritten to the head's own bytes" would describe an action
            // that did not happen.
            restored.removed.push(entry.relative_path.clone());
        }
    }
    fs::remove_dir_all(&root)
        .map_err(|e| format!("remove restored journal {}: {e}", root.display()))?;
    Ok(restored)
}

/// True when both paths exist and hold the same bytes. An unreadable side is NOT equal: the caller
/// then writes, so a comparison failure can never turn into a skipped restore.
fn destination_bytes_equal(destination: &Path, prepared: &Path) -> Result<bool, String> {
    let (Ok(current), Ok(wanted)) = (fs::read(destination), fs::read(prepared)) else {
        return Ok(false);
    };
    Ok(current == wanted)
}

fn journal_stage0_paths_for_subject(
    workspace: &Path,
    stage0_src: &Path,
    basenames: &[String],
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<(), String> {
    let entries = basenames
        .iter()
        .map(|basename| {
            (
                format!("src/v1/stage0/src/{basename}"),
                stage0_src.join(basename),
            )
        })
        .collect::<Vec<_>>();
    journal_paths_for_subject(workspace, &entries, subject)
}

/// THE CHECKPOINT IS KEYED BY WORKSPACE-RELATIVE PATH, so a partition boundary file (outside the
/// seed's `src/`) is restored by the same journal a mirror is: one rollback, not a second one beside it.
fn journal_paths_for_subject(
    workspace: &Path,
    paths: &[(String, PathBuf)],
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<(), String> {
    admit_subject_live_facts(workspace, subject)?;
    let root = regen_convergence_journal_path(workspace);
    fs::create_dir_all(&root).map_err(|e| format!("create {}: {e}", root.display()))?;
    let manifest_path = root.join("journal.json");
    let existing_journal = manifest_path.is_file();
    let mut journal = if existing_journal {
        serde_json::from_slice::<RegenConvergenceJournal>(
            &fs::read(&manifest_path)
                .map_err(|e| format!("read {}: {e}", manifest_path.display()))?,
        )
        .map_err(|e| format!("parse {}: {e}", manifest_path.display()))?
    } else {
        RegenConvergenceJournal {
            schema: REGEN_CONVERGENCE_SCHEMA.to_string(),
            starting_commit: subject.starting_commit.clone(),
            source_authority_digest: subject.source_authority_digest.clone(),
            stage_plan_authority_digest: subject.stage_plan_authority_digest.clone(),
            checkpoint_id: String::new(),
            entries: Vec::new(),
        }
    };
    if existing_journal
        && (journal.schema != REGEN_CONVERGENCE_SCHEMA
            || journal.starting_commit != subject.starting_commit
            || journal.source_authority_digest != subject.source_authority_digest
            || journal.stage_plan_authority_digest != subject.stage_plan_authority_digest
            || convergence_checkpoint_id(&journal)? != journal.checkpoint_id)
    {
        return Err(format!(
            "CheckpointSubjectMismatch: existing journal cannot be extended for commit={} source={} plan={}",
            subject.starting_commit,
            subject.source_authority_digest,
            subject.stage_plan_authority_digest
        ));
    }
    for entry in &journal.entries {
        if let RegenPreStageState::PresentBeforeInstall { digest } = &entry.pre_stage_state {
            let observed = path_digest(&root.join(&entry.backup_name))?;
            if observed != *digest {
                return Err(format!(
                    "CheckpointArtifactDigestMismatch: {} recorded {} observed {}",
                    entry.backup_name, digest, observed
                ));
            }
        }
    }
    for (relative_path, source) in paths {
        let relative_path = relative_path.clone();
        if journal
            .entries
            .iter()
            .any(|entry| entry.relative_path == relative_path)
        {
            continue;
        }
        let source = source.clone();
        let backup_name = format!("surface-{}.bak", journal.entries.len());
        let pre_stage_state = if source.is_file() {
            fs::copy(&source, root.join(&backup_name))
                .map_err(|e| format!("snapshot {}: {e}", source.display()))?;
            RegenPreStageState::PresentBeforeInstall {
                digest: path_digest(&source)?,
            }
        } else {
            RegenPreStageState::AbsentBeforeInstall
        };
        journal.entries.push(RegenConvergenceJournalEntry {
            relative_path,
            backup_name,
            pre_stage_state,
        });
    }
    journal.checkpoint_id = convergence_checkpoint_id(&journal)?;
    let bytes = serde_json::to_vec_pretty(&journal).map_err(|e| format!("encode journal: {e}"))?;
    fs::write(&manifest_path, bytes).map_err(|e| format!("write {}: {e}", manifest_path.display()))
}

/// THE SEED CRATE'S OWN SOURCE POPULATION as committed at ONE COMMIT: every path git tracks under
/// `src/v1/stage0`, keyed by its repo-relative path — mirrors at any depth, the hand-maintained
/// files, and the hand-maintained directories. Called for the BASE commit by the bootstrap authority
/// and for the HEAD commit by the same round, because the delta between those two populations is the
/// fact the receipt has to state; one git spelling for both (DESIGN §3).
///
/// There is deliberately NO filter here, and the two that used to be are the bug this replaces.
/// `!rest.contains('/')` skipped every nested mirror (`cli_run/compile_clean.rs`,
/// `bin/fixture_main.rs`), and the hand roster skip left every hand-maintained file at the head.
/// Round 1 then compiled a CRATE THAT IS NEITHER BASE NOR HEAD: the head's hand-written caller
/// against base's generated definitions, which refuses with its own E0599 while the head's `.dag`
/// edit that would repair it sits in the tree, unreachable (`v1-stage0-std-core` includes those
/// mirrors through `#[path = "../../stage0/src/<basename>.rs"]`, and `#[path]` recurses, so nested
/// partitions compile too). Specimens: PR #15175 at `432809a3ed4b` (`variant `Dag` is
/// constructed, but its only constructor \`MirrorRow::Dag\` lives in the head mirror the round never
/// read`), PR #14876 (heal run 40739785316, 26 × E0308/E0282 in `v1-stage0-std-core`), heal run
/// 40637527543 (the same shape with the nested file). See
/// `gunbc.recurring_failure_mode.heal_round_one_bootstraps_from_the_mirror_the_change_must_repair`.
///
/// "Always base, never a fallback" (root ruling) has a precise meaning: base is what the round
/// COMPILES, and the head's authored `.dag`/`.rs` authorities are what it REGENERATES — round 2
/// installs the regeneration over the whole base surface, so a base byte here is transient and can
/// never be published. What a base checkout can no longer do is hand the round half of each.
fn crate_population_at_commit(
    workspace: &Path,
    commit: &str,
) -> Result<BTreeMap<String, String>, String> {
    let output = Command::new("git")
        .args([
            "ls-tree",
            "-r",
            "--name-only",
            commit,
            "--",
            STAGE0_CRATE_REL,
        ])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("git ls-tree {commit}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-tree {commit} -- {STAGE0_CRATE_REL}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut population = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let path = line.trim_end_matches('\r');
        if path.is_empty() {
            continue;
        }
        population.insert(path.to_string(), path.to_string());
    }
    Ok(population)
}

/// The same population as it CURRENTLY SITS in the worktree, recursively. Read from the filesystem
/// rather than from git, so an uncommitted generated or hand edit is part of the subject: what the
/// round overwrote is exactly what the journal must return.
fn worktree_crate_population(stage0_crate: &Path) -> Result<BTreeSet<String>, String> {
    let mut present = BTreeSet::new();
    let mut stack = vec![stage0_crate.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .map_err(|e| format!("enumerate stage0 crate {}: {e}", dir.display()))?
        {
            let entry = entry.map_err(|e| format!("read stage0 crate entry: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(rest) = path.strip_prefix(stage0_crate) else {
                continue;
            };
            let Some(text) = rest.to_str() else {
                continue;
            };
            present.insert(format!("{STAGE0_CRATE_REL}/{text}"));
        }
    }
    Ok(present)
}

/// THE BASE/HEAD PATH-SET DELTA, computed in ONE place from the three populations a round can see.
///
/// THREE SETS, and the round must not confuse them:
/// - THE BASE POPULATION: every path the base commit tracks under the seed crate. That is the
///   validated bootstrap input, and round 1 compiles ALL of it, with no filter of any kind. Every
///   filter that used to sit here is the defect this replaces: the last one, `from_base ∩ worktree`,
///   dropped the paths a PR had DELETED while still restoring base's `lib.rs`, which declares them,
///   so round 1 compiled a crate that was neither base nor head. The two retained specimens, read
///   from the heal logs and the object store (commands and counts in
///   `gunbc.recurring_failure_mode.heal_round_one_bootstraps_from_the_mirror_the_change_must_repair`):
///   PR #15418 heal run 41605027842 job 125813087856, base `5ff414e66015` tracks 303 crate paths and
///   head `5b7b2bc2` tracks 300, the receipt printed `paths=300`, and rustc printed
///   `error[E0583]: file not found for module v1_tests_claim_interpolation_hole_census` and the same
///   for `v1_tests_claim_shared_type_leaf_collision_census`; PR #15540 heal run 41591786053 job
///   125769944667, base `6e1c65a36bcb` 303 and head `3be90ce5363e` 302, receipt `paths=302`,
///   `error[E0583]: file not found for module required_lane_resolution_census` at the NESTED path
///   `src/v1/stage0/src/cli_run/required_lane_resolution_census.rs`.
/// - THE HEAD'S TRACKED POPULATION, read from the head COMMIT. "The head deleted this path" is a
///   fact about a commit; the filesystem cannot tell a deleted path from an untracked one.
/// - THE WORKTREE'S ON-DISK POPULATION, a filesystem walk, which also sees UNTRACKED files. It is
///   not an authority about the head. What it is good for is exactly one thing here: telling a truly
///   absent path from a path some other process left behind, so the round never guesses.
///
/// `head_deleted` is base minus head-tracked. Those paths are restored for the compile and journalled
/// `AbsentBeforeInstall`, so the round's own teardown removes them again: absence is the head's
/// authority, and the final regenerated output never carries a module the head deleted.
///
/// A `head_deleted` path that is nevertheless PRESENT ON DISK is not a deleted path — it is an
/// untracked file sitting at a path base names. Overwriting it would clobber bytes the head never
/// tracked; deleting it would clobber them by inference. So the round REFUSES, typed and located, and
/// moves nothing.
///
/// INTERRUPTION IS THE SAME JOURNAL, NOT A NEW ONE. A round killed after the write leaves the
/// resurrected path on disk with the journal naming it `AbsentBeforeInstall`; the next round restores
/// that journal BEFORE it bootstraps (`restore_regen_convergence_journal` at the head of the
/// convergence loop), which removes it. If the subject has moved since the kill, that recovery refuses
/// with `CheckpointSubjectMismatch` instead of guessing which on-disk files "look like" base's.
struct BasePopulationDelta {
    /// The exact validated bootstrap input: every path the base commit tracks under the crate.
    restore: BTreeSet<String>,
    /// Base tracks it, the head does not: restored for the compile, removed after it.
    head_deleted: BTreeSet<String>,
    /// The head tracks it, base does not: it stays at the head, journalled, and counted.
    head_added: BTreeSet<String>,
}

fn base_population_delta(
    from_base: &BTreeMap<String, String>,
    head_tracked: &BTreeSet<String>,
    in_worktree: &BTreeSet<String>,
    base_commit: &str,
    head_commit: &str,
) -> Result<BasePopulationDelta, String> {
    let base: BTreeSet<String> = from_base.keys().cloned().collect();
    let head_deleted: BTreeSet<String> = base.difference(head_tracked).cloned().collect();
    if let Some(path) = head_deleted.iter().find(|path| in_worktree.contains(*path)) {
        return Err(format!(
            "BootstrapBaseMirrorUntrackedCollision: {path} is tracked by the base commit \
             {base_commit}, is not tracked by the head {head_commit}, and is present in the \
             worktree; the round will neither overwrite it with base's bytes nor remove it by \
             inference"
        ));
    }
    Ok(BasePopulationDelta {
        restore: base.clone(),
        head_deleted,
        head_added: head_tracked.difference(&base).cloned().collect(),
    })
}

/// What the round-1 bootstrap did, in the receipt's vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BaseRestore {
    /// The base commit, exactly as the step named it. The receipt NAMES it (sleek-owl-397
    /// caution, RFM #14963).
    base_commit: String,
    /// How many crate paths the round-1 seed actually built from the base commit's bytes.
    paths_restored: usize,
    /// How many of those paths the seed actually wrote. A path whose base bytes already equal the
    /// worktree's is NOT written (see `bootstrap_round_one_from_base_mirror`): rewriting identical
    /// bytes bumps an mtime cargo reads as a change and drags excluded crates into the rebuild.
    /// Zero is the shape of a PR that touches no stage0 file at all.
    paths_rewritten: usize,
    /// How many paths the HEAD tracks that base does not have at all: they stay at the head, are
    /// journaled before the round touches them, and are counted rather than silent — a head-added
    /// support file is exactly where a new hand-written call to a new generated method lives, and
    /// the compile is still whole-crate base because the generated side of that call is base's.
    paths_absent_at_base: usize,
    /// How many paths the BASE tracks that the HEAD does not: the PR deleted them. Round 1 restores
    /// them so the crate it compiles is base's crate and not a chimera, and the journal's
    /// `AbsentBeforeInstall` entries remove them again before anything is published. Counted, never
    /// silent: this is the delta the receipt could not previously show, and the two specimens in
    /// `base_population_delta` are what hiding it cost.
    head_deleted_paths: usize,
}

/// The receipt's base-mirror arm, in the shape its renderer takes. Split out so the HOST half of
/// the line — which commit and which three counts reach the renderer — is checkable without loading a
/// corpus. The RENDERER's half (the line's text, and its absence for a round that was never asked
/// to bootstrap) is asserted in `dag/test/claim/self_host_regen_round_cost_witness.dag`
/// `a_bootstrapped_receipt_names_the_base_commit_after_the_header` and
/// `a_non_bootstrapped_receipt_renders_no_base_mirror_line`, which reach a verdict on a required
/// run. A Rust control that renders through the interpreter pays a whole corpus load per call —
/// measured at about 50 minutes of libtest time on srv1 at `b339f6fa0e6f`, the same front-end cost
/// this repository's native-execution lane exists to remove — so rendering does not belong here.
fn base_mirror_receipt_authority(base: &BaseRestore) -> (String, usize, usize, usize) {
    (
        base.base_commit.clone(),
        base.paths_restored,
        base.paths_absent_at_base,
        base.head_deleted_paths,
    )
}

/// The same arm, DERIVED READ-ONLY from the authority the bootstrap uses, for the image that renders
/// the receipt without having seeded: the re-exec'd handoff child. Under the handoff rule the child
/// seeds nothing, so `base_bootstrap` is `None` there, and passing that `None` to the renderer would
/// select the `SeedBuiltFromHead` arm and print a receipt claiming round 1 built from the HEAD —
/// fabricating a plausible output for the one run whose whole purpose is to say which bytes round 1
/// compiled. So the child re-derives the commit and all three counts through the SAME
/// `base_population_delta` the seeding image used, rather than a second copy of the set arithmetic:
/// two spellings of that arithmetic is how the parent and the child could disagree about a delta
/// neither one admits. No fallback and no `.ok()`: a base commit that cannot be named refuses the
/// round with the typed `BootstrapBaseMirrorCommitUnresolved` diagnostic instead of dropping the line,
/// and so does an untracked file sitting at a base-only path, which is the one case the delta refuses.
fn round_one_receipt_base_authority(
    workspace: &Path,
    base_commit_env: Option<&str>,
) -> Result<(String, usize, usize, usize), String> {
    let (base_commit, from_base) = resolve_base_mirror_authority(workspace, base_commit_env)?;
    let in_worktree = worktree_crate_population(&workspace.join(STAGE0_CRATE_REL))?;
    let head_commit = git_head_sha(workspace)?;
    let head_tracked: BTreeSet<String> = crate_population_at_commit(workspace, &head_commit)?
        .keys()
        .cloned()
        .collect();
    let delta = base_population_delta(
        &from_base,
        &head_tracked,
        &in_worktree,
        &base_commit,
        &head_commit,
    )?;
    Ok((
        base_commit,
        delta.restore.len(),
        delta.head_added.len(),
        delta.head_deleted.len(),
    ))
}

/// THE HEAL ROUND'S BOOTSTRAP AUTHORITY — DESIGN §5, `gunbc.recurring_failure_mode`
/// `heal_round_one_bootstraps_from_the_mirror_the_change_must_repair` (RFM #14963).
///
/// Round 1 of the heal round compiles the seed against the BASE ref's committed stage0 mirror,
/// never against the mirror carried by the pull request's own head. The head's mirror is precisely
/// what the PR's `.dag` edits exist to repair: when it already carries a refusal the fix removes,
/// the installed compiler refuses in round 1 before the regenerated seed can produce the
/// correction — the deadlock hit on PR #13936 (run 40064940799, job 120786085841, step 7:
/// `regen-round-cost: refused: stage seed build refused … cargo build
/// (round.rebuild_from_installed) failed with exit status: 101`, five rows of
/// `error: shared layer refused for 'Nat': TypeDeclNotDeclared — wrap/deref/Copy of a user type
/// requires one declaration identity`, each pointing at `../../stage0/src/std_measure.rs:650:34`,
/// `:652:90`, `:1389:12` and `../../stage0/src/std_checked_arithmetic.rs:316:28`, `:320:70` — the
/// committed mirror, compiled as `src/v1/stage0_std_core/src/../../stage0/src/…`. Those
/// `compile_error!` rows are what the PR's `.dag` edits remove, so the round was refused by the
/// artifact its own repair regenerates).
///
/// THIS IS NOT A FALLBACK AND IS NEVER TAKEN AFTER A FAILURE. The heal round step passes
/// `--base-mirror` unconditionally and the commit arrives as `HEAL_BASE_MIRROR_COMMIT`; a round
/// that is not a
/// heal round simply does not ask for it. One path runs per invocation — there is no second
/// attempt, no retry, and nothing to fall back TO (§5).
///
/// Scope, and why it is exactly this wide:
/// - Round 1's seed build ONLY. Rounds 2+ regenerate, install and re-measure from what the PR's own
///   `.dag` produces, so the transaction still converges to the PR seed's own fixed point and the
///   published candidate carries no base byte (`sleek-owl-397` caution, RFM #14963).
/// - THE WHOLE CRATE, NESTED INCLUDED. What cargo compiles is `src/v1/stage0` — `src/**` at every
///   depth, `tests/**`, `testdata/**`, `Cargo.toml`, `build.rs` — so that is what round 1 pins.
///   Restoring only top-level generated mirrors left a CRATE THAT IS NEITHER BASE NOR HEAD, and
///   that chimera refuses with its own E0599/E0308 while the fix sits unreachable in the tree:
///   `crate_population_at_commit` names the specimens.
/// - THE BASE SNAPSHOT EXISTS ONLY FOR THE COMPILE. `restore_regen_convergence_journal` — the round's
///   own teardown, the one reader of the journal (§3) — puts the head's own bytes back as soon as
///   round 1's seed build is no longer reading them.
///   It is NOT enough to rely on regeneration: the generator regenerates GENERATED files, so the
///   hand-maintained paths the whole-crate population now includes — `required_regen_host.rs`
///   itself, `bin/**`, `tests/**`, `Cargo.toml`, `build.rs` — would keep base's bytes through a
///   SUCCESSFUL round, and a green heal round would then publish a commit that reverts the pull
///   request's own hand-written Rust (review 87490 on #15339; `gunbc.heal_candidate_stage0`
///   `mirror_paths_of` filters by directory depth and suffix only, so such a file enters the
///   candidate). Base is what round 1 COMPILES and nothing else.
/// - Only a path whose bytes ACTUALLY DIFFER is written, and only those paths are journalled. A
///   rewrite of identical bytes is not a no-op: it bumps the file's mtime, and cargo fingerprints a
///   path dependency by mtime, so the seed build and the measured partition rebuild recompile crates
///   the round's own derived scope had EXCLUDED, and the scope guard refuses a round that did nothing
///   wrong. Specimens: PR #14876 (heal run 41061363628, job 124081920078 — a `.dag`-only PR, so all
///   306 crate paths are byte-identical to base, and `compiled excluded packages
///   ["v1-stage0-runtime"]` with `package_closure=[std-core, v1-infer, std-surface,
///   extdeps-languages, v1-artifact, emit-core, v1-compiler]`) and PR #15310 (heal run 41098380134 —
///   six excluded partition packages, `changed_mirrors=[v1_compiler_emit_rust.rs]`,
///   `closure=[v1-compiler]`). The guard is right and is NOT widened; the writer was doing redundant
///   work (DESIGN §2), and mtimes are not restored either — a restored mtime is a second lie on the
///   same file. `a_stage0_untouched_pr_moves_no_mtime_in_the_seed_or_the_restore` controls both
///   halves, with a discriminator so the mtime assertion cannot pass vacuously.
/// - Every path it DOES overwrite is journalled FIRST, under the same checkpoint subject the install
///   path uses: the snapshot holds the tree's OWN bytes, so the transaction's single existing
///   restore returns the worktree to the head state on every exit path. This is the same rollback,
///   not a second one beside it. A head-added path the seed never writes is journalled by the
///   install that overwrites it, not twice over.
///
/// Idempotent: the heal round step runs once per image (a seed-handoff re-exec re-runs it), and a
/// re-run finds the bytes already equal to base's, so it writes nothing and moves no mtime.
fn bootstrap_round_one_from_base_mirror(
    workspace: &Path,
    base_commit_env: Option<&str>,
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<BaseRestore, String> {
    let (base_commit, from_base) = resolve_base_mirror_authority(workspace, base_commit_env)?;
    let stage0_crate = workspace.join(STAGE0_CRATE_REL);
    let in_worktree = worktree_crate_population(&stage0_crate)?;
    // RESTORE THE EXACT VALIDATED BASE POPULATION: every path the base commit tracks under the
    // crate, with NO filter of any kind. The rule used to be `from_base ∩ worktree`, and that
    // intersection was a filter in the one place it mattered — it dropped exactly the paths the PR
    // had DELETED while base's `lib.rs` went on declaring them, so round 1 compiled a crate that was
    // neither base nor head and rustc refused with E0583. `base_population_delta` states the three
    // sets, the two retained specimens, and the one case that still refuses: a base-only path
    // present on disk as an untracked file, which the round neither overwrites nor removes.
    //
    // WHAT THIS DOES NOT DO IS RESURRECT A DELETION. A restored head-deleted path is journalled
    // `AbsentBeforeInstall` by the same journal that snapshots every other staged path, so the
    // round's own teardown removes it before the candidate is compared or published, and the
    // emitter's population walk never sees a base byte for it.
    let head_commit = git_head_sha(workspace)?;
    let head_tracked: BTreeSet<String> = crate_population_at_commit(workspace, &head_commit)?
        .keys()
        .cloned()
        .collect();
    let delta = base_population_delta(
        &from_base,
        &head_tracked,
        &in_worktree,
        &base_commit,
        &head_commit,
    )?;
    let restore = delta.restore;
    let head_deleted_paths = delta.head_deleted.len();
    let paths_absent_at_base = delta.head_added.len();

    // FETCH EVERY BASE BLOB ONCE AND WRITE ONLY THE PATHS WHOSE BYTES ACTUALLY DIFFER.
    //
    // A path whose base bytes already equal the worktree's needs no write, and writing it anyway is
    // not harmless: it bumps that file's mtime, and cargo's fingerprint for a path dependency reads
    // mtimes — so every crate that reads a rewritten file is rebuilt, including crates the round's
    // derived scope had correctly EXCLUDED. That is the #14876 specimen of this failure mode
    // (run 41061363628, job 124081920078): the PR touches no stage0 or runtime file, so all 306
    // crate paths are byte-identical to base, the seed and the restore still rewrote all 306, and
    // the round-2 rebuild recompiled `v1-stage0-runtime` — which the guard's own receipt had derived
    // as out of scope (`changed_mirrors=[std_algebra.rs, v1_compiler_infer_{method,resolve,types}.rs]
    // owning_packages=[v1-stage0-std-core, v1-stage0-v1-infer]`, closure = reverse deps, runtime not
    // in it), so the scope guard refused a round that had done nothing wrong.
    //
    // The guard is right and stays exactly as it is; the writer is what was wrong. DESIGN §2: no
    // redundant work. mtimes are not restored either — a restored mtime would be a second, stranger
    // lie on the same file.
    let mut base_bytes: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut to_write: BTreeSet<String> = BTreeSet::new();
    for path in &restore {
        let output = Command::new("git")
            .args(["show", &format!("{}:{}", base_commit, path)])
            .current_dir(workspace)
            .output()
            .map_err(|e| format!("git show {}:{}: {e}", base_commit, path))?;
        if !output.status.success() {
            return Err(format!(
                "BootstrapBaseMirrorUnreadable: git show {}:{}: {}",
                base_commit,
                path,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let current = fs::read(workspace.join(path)).ok();
        base_bytes.insert(path.clone(), output.stdout.clone());
        if current.as_deref() != Some(output.stdout.as_slice()) {
            to_write.insert(path.clone());
        }
    }

    // JOURNAL THE TREE'S OWN BYTES BEFORE ANY WRITE, under THE SAME SUBJECT the install path uses.
    // One rollback, not a second one beside it: the pre-stage snapshot holds the head's bytes, and
    // the transaction's existing restore returns every path the bootstrap touched. A path the
    // install journals later keeps this first snapshot — the head's bytes, never base's.
    // ONLY THE PATHES THIS SEED ACTUALLY WRITES are journalled: a path the seed leaves alone has
    // nothing to undo, and the round's own install journals whatever IT overwrites (head-added
    // support files included), so journalling an untouched path would be redundant work and would
    // make the journal's population stop meaning "what the seed moved".
    let journal_paths: Vec<String> = to_write.iter().cloned().collect();
    let entries = journal_paths
        .iter()
        .map(|path| (path.clone(), workspace.join(path)))
        .collect::<Vec<_>>();
    journal_paths_for_subject(workspace, &entries, subject).map_err(|e| {
        format!("BootstrapBaseMirrorJournal: journal the head bytes before the base restore: {e}")
    })?;

    for path in &to_write {
        let destination = workspace.join(path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        fs::write(&destination, &base_bytes[path])
            .map_err(|e| format!("write {}: {e}", destination.display()))?;
    }
    Ok(BaseRestore {
        base_commit,
        // The population the round holds at base bytes — which for a PR that touches no stage0 file
        // is the whole crate, even though nothing had to be written to get there.
        paths_restored: restore.len(),
        // How many of those the seed actually moved on disk. Zero means base's bytes and the head's
        // were already the same bytes, which is the #14876 shape and the reason no mtime moved.
        paths_rewritten: to_write.len(),
        paths_absent_at_base,
        head_deleted_paths,
    })
}

/// Read the base commit's generated mirror population, from THE ONE AUTHORITY the heal round step
/// binds: `HEAL_BASE_MIRROR_COMMIT`, the pull request's immutable `base.sha`.
///
/// It is a COMMIT ID, not a name, and this function never looks one up. A branch is a moving name
/// that a checkout's local branch and its remote-tracking ref can disagree about (a local `main`
/// can sit behind `origin/main`); seeding the round from whichever one happens to exist, and then
/// recording the pick as "the base commit", would fabricate a fact the round never established —
/// DESIGN §3 (each fact lives in exactly one place) and §5 (no plausible output is fabricated).
/// GitHub has already resolved this pull request's base to a commit; that commit is the input.
///
/// Every refusal is typed and located, and none of them builds from the head mirror instead:
/// - `BootstrapBaseMirrorCommitUnresolved` — the step did not supply an authority at all (unset, or
///   present but blank: the two causes are named differently, so a mis-wired step and an empty
///   string are distinguishable in the log),
/// - `BootstrapBaseMirrorCommitMalformed` — a name or a short prefix rather than a full commit id,
/// - `BootstrapBaseMirrorCommitUnreachable` — a well-formed id this checkout does not contain, or
///   one that is not a commit (heal's checkout is `fetch-depth: 0`, so the base is present),
/// - `BootstrapBaseMirrorUnreadable` — the commit is present but carries no mirror population.
///
/// No case falls through to a build: the caller refuses the round.
fn resolve_base_mirror_authority(
    workspace: &Path,
    raw_commit: Option<&str>,
) -> Result<(String, BTreeMap<String, String>), String> {
    let env = REGEN_ROUND_COST_BASE_MIRROR_ENV;
    let raw = match raw_commit {
        None => {
            return Err(format!(
                "BootstrapBaseMirrorCommitUnresolved: {env} is UNSET while --base-mirror asked \
                 round 1 to bootstrap from the base mirror; the heal round step must bind it to \
                 the pull request's base commit"
            ))
        }
        Some(value) => value,
    };
    let base_commit = raw.trim();
    if base_commit.is_empty() {
        return Err(format!(
            "BootstrapBaseMirrorCommitUnresolved: {env} is set but {} while --base-mirror asked \
             round 1 to bootstrap from the base mirror; the heal round step must bind it to the \
             pull request's base commit",
            if raw.is_empty() {
                "EMPTY"
            } else {
                "only whitespace"
            }
        ));
    }
    // A FULL commit id, so the authority is not a name that could be re-resolved to something else
    // between the step's read and this build.
    if base_commit.len() != 40
        || !base_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte.to_ascii_lowercase()))
    {
        return Err(format!(
            "BootstrapBaseMirrorCommitMalformed: {env}={base_commit} is not a 40-hex commit id. \
             The round's bootstrap authority is the pull request's base COMMIT, never a branch \
             name or an abbreviated id: a name resolves to whichever commit the checkout happens \
             to hold under it, which is a different authority from the one the receipt would name."
        ));
    }
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            &format!("{base_commit}^{{commit}}"),
        ])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("git rev-parse {base_commit}: {e}"))?;
    let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() || resolved != base_commit {
        return Err(format!(
            "BootstrapBaseMirrorCommitUnreachable: {env}={base_commit} is not a commit this \
             checkout contains ({})",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let population = crate_population_at_commit(workspace, base_commit)
        .map_err(|e| format!("BootstrapBaseMirrorUnreadable: {e}"))?;
    if population.is_empty() {
        return Err(format!(
            "BootstrapBaseMirrorUnreadable: {env}={base_commit} carries no generated mirror \
             population under {STAGE0_SRC_REL}; a round cannot bootstrap from a mirror that does \
             not exist"
        ));
    }
    Ok((resolved, population))
}

/// The seed build's cost as observed by one process image.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SeedBuildCost {
    compiled_crates: u64,
    wall_ms: u64,
    /// `None` is `gunbc.regen_round_cost` `CpuUnreadable`, never zero.
    cpu_ms: Option<u64>,
}

/// What the replaced image hands to the built one: its seed-build cost, the source identity it
/// built (HEAD plus the digest of the uncommitted tracked diff: a dirty tree stays legal, a
/// same-commit source change is visible), and the digest of the artifact it built.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SeedHandoff {
    cost: SeedBuildCost,
    source: String,
    built_exe: String,
    /// The round's checkpoint subject if the image that owns the round established one (a
    /// base-mirror round always has). Encoded explicitly as present or absent: a handoff without
    /// the field is `HandoffPredatesRoundSubject`, never a cue to recompute.
    round_subject: Option<RegenConvergenceCheckpointSubject>,
}

/// Why the seed handoff refuses. Standing: runtime mitigation (guarantee-ladder rung 1) -- a
/// typed refusal at the point the invalid state is observed; the invalid state stays writable.
#[derive(Clone, Debug, PartialEq, Eq)]
enum SeedHandoffRefusal {
    /// The built image was replaced by another path replacement after the handoff. Claims
    /// nothing about idempotence: only that the executable changed again.
    ExecutableReplacedAfterHandoff {
        before: String,
        after: String,
    },
    /// The source the first image built is not the source this image sees (another commit, or
    /// the same commit with a different uncommitted tracked diff).
    SourceChangedAcrossHandoff {
        carried: String,
        observed: String,
    },
    /// The descriptor opened for the exec holds other bytes than the build produced.
    OpenedTargetNotBuildOutput {
        built: String,
        opened: String,
    },
    /// This image is not the artifact the first image built.
    WrongArtifactAcrossHandoff {
        built: String,
        running: String,
    },
    /// The composed ledger holds other than exactly one `round.seed_build` measurement, so the
    /// carried cost cannot be added exactly once.
    SeedBuildMeasurementCount {
        found: usize,
    },
    /// The carried observation does not decode.
    HandoffUndecodable {
        value: String,
    },
    /// The carried observation is well formed up to the point where the round-subject field is
    /// due and ends there: the image that wrote it predates that field (first written by 0256037,
    /// #15855). Refused, never read as an absent subject. Distinct from `HandoffUndecodable` so it
    /// can be counted: its remedy is known (merge the base into the head), an undecodable value's
    /// is not.
    HandoffPredatesRoundSubject {
        missing_field: &'static str,
        value: String,
    },
    ReExecFailed {
        path: String,
        cause: String,
    },
}

impl std::fmt::Display for SeedHandoffRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = REGEN_ROUND_COST_PRODUCER;
        match self {
            Self::ExecutableReplacedAfterHandoff { before, after } => write!(
                f,
                "refusal: ExecutableReplacedAfterHandoff: {p}: the seed build replaced the \
                 running executable again after the one handoff ({before} -> {after}); the tree \
                 is the carried one, so another path is replacing the executable"
            ),
            Self::SourceChangedAcrossHandoff { carried, observed } => write!(
                f,
                "refusal: SourceChangedAcrossHandoff: {p}: first image built source {carried}, \
                 this image sees {observed}"
            ),
            Self::OpenedTargetNotBuildOutput { built, opened } => write!(
                f,
                "refusal: OpenedTargetNotBuildOutput: {p}: the build produced {built} but the \
                 descriptor opened for the exec holds {opened}"
            ),
            Self::WrongArtifactAcrossHandoff { built, running } => write!(
                f,
                "refusal: WrongArtifactAcrossHandoff: {p}: first image built {built}, this \
                 image is {running}"
            ),
            Self::SeedBuildMeasurementCount { found } => write!(
                f,
                "refusal: SeedBuildMeasurementCount: {p}: expected exactly one round.seed_build \
                 measurement to receive the carried cost, found {found}"
            ),
            Self::HandoffUndecodable { value } => write!(
                f,
                "refusal: HandoffUndecodable: {p}: {REGEN_ROUND_COST_REEXEC_ENV}={value:?}"
            ),
            Self::HandoffPredatesRoundSubject {
                missing_field,
                value,
            } => write!(
                f,
                "refusal: HandoffPredatesRoundSubject: {p}: the pre-exec image predates the \
                 round-subject handoff (base 0256037): its handoff ends where `{missing_field}` \
                 is due ({REGEN_ROUND_COST_REEXEC_ENV}={value:?}); merge the base into the head"
            ),
            Self::ReExecFailed { path, cause } => write!(
                f,
                "refusal: ReExecFailed: {p}: the seed build replaced the running executable and \
                 re-exec of {path} failed: {cause}"
            ),
        }
    }
}

enum SeedHandoffDecision {
    Proceed,
    ReExec(SeedHandoff),
}

impl SeedHandoff {
    fn encode(&self) -> String {
        let cpu = self
            .cost
            .cpu_ms
            .map_or("unreadable".to_string(), |c| c.to_string());
        let subject = match &self.round_subject {
            Some(s) => format!(
                "subject=present;subject_commit={};subject_source={};subject_plan={}",
                s.starting_commit, s.source_authority_digest, s.stage_plan_authority_digest
            ),
            None => "subject=absent".to_string(),
        };
        format!(
            "crates={};wall_ms={};cpu_ms={};source={};built_exe={};{subject}",
            self.cost.compiled_crates, self.cost.wall_ms, cpu, self.source, self.built_exe
        )
    }

    fn decode(value: &str) -> Result<Self, SeedHandoffRefusal> {
        let bad = || SeedHandoffRefusal::HandoffUndecodable {
            value: value.to_string(),
        };
        let mut it = value.split(';').map(|kv| kv.split_once('='));
        let mut field = |key: &str| -> Result<String, SeedHandoffRefusal> {
            match it.next() {
                Some(Some((k, v))) if k == key => Ok(v.to_string()),
                // The input ends exactly where the subject field is due: the previous encoding.
                None if key == "subject" => Err(SeedHandoffRefusal::HandoffPredatesRoundSubject {
                    missing_field: "subject",
                    value: value.to_string(),
                }),
                _ => Err(bad()),
            }
        };
        let compiled_crates = field("crates")?.parse::<u64>().map_err(|_| bad())?;
        let wall_ms = field("wall_ms")?.parse::<u64>().map_err(|_| bad())?;
        let cpu = field("cpu_ms")?;
        let cpu_ms = if cpu == "unreadable" {
            None
        } else {
            Some(cpu.parse::<u64>().map_err(|_| bad())?)
        };
        let source = field("source")?;
        let built_exe = field("built_exe")?;
        let round_subject = match field("subject")?.as_str() {
            "present" => Some(RegenConvergenceCheckpointSubject {
                starting_commit: field("subject_commit")?,
                source_authority_digest: field("subject_source")?,
                stage_plan_authority_digest: field("subject_plan")?,
            }),
            "absent" => None,
            _ => return Err(bad()),
        };
        if it.next().is_some() {
            return Err(bad());
        }
        Ok(SeedHandoff {
            cost: SeedBuildCost {
                compiled_crates,
                wall_ms,
                cpu_ms,
            },
            source,
            built_exe,
            round_subject,
        })
    }

    fn from_env() -> Result<Option<Self>, SeedHandoffRefusal> {
        match std::env::var(REGEN_ROUND_COST_REEXEC_ENV) {
            Ok(v) => Self::decode(&v).map(Some),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(v)) => Err(SeedHandoffRefusal::HandoffUndecodable {
                value: v.to_string_lossy().into_owned(),
            }),
        }
    }
}

fn decide_seed_handoff(
    incoming: Option<&SeedHandoff>,
    exe_before: &str,
    exe_after: &str,
    source: &str,
    cost: &SeedBuildCost,
    round_subject: Option<&RegenConvergenceCheckpointSubject>,
) -> Result<SeedHandoffDecision, SeedHandoffRefusal> {
    if let Some(h) = incoming {
        if h.source != source {
            return Err(SeedHandoffRefusal::SourceChangedAcrossHandoff {
                carried: h.source.clone(),
                observed: source.to_string(),
            });
        }
        if h.built_exe != exe_before {
            return Err(SeedHandoffRefusal::WrongArtifactAcrossHandoff {
                built: h.built_exe.clone(),
                running: exe_before.to_string(),
            });
        }
        if exe_before != exe_after {
            return Err(SeedHandoffRefusal::ExecutableReplacedAfterHandoff {
                before: exe_before.to_string(),
                after: exe_after.to_string(),
            });
        }
        return Ok(SeedHandoffDecision::Proceed);
    }
    if exe_before == exe_after {
        return Ok(SeedHandoffDecision::Proceed);
    }
    Ok(SeedHandoffDecision::ReExec(SeedHandoff {
        cost: cost.clone(),
        source: source.to_string(),
        built_exe: exe_after.to_string(),
        round_subject: round_subject.cloned(),
    }))
}

/// Compose the carried pre-handoff seed build into this image's observation: crate count and
/// the `round.seed_build` ledger row, summed exactly once. A carried CPU that is unreadable
/// makes the composed CPU unreadable, never zero.
fn compose_seed_build_cost(
    carried: Option<&SeedBuildCost>,
    this_image_crates: u64,
    mut marks: std::vec::Vec<v1_rt::TraceLedgerRow>,
) -> Result<(u64, std::vec::Vec<v1_rt::TraceLedgerRow>), SeedHandoffRefusal> {
    let Some(c) = carried else {
        return Ok((this_image_crates, marks));
    };
    let found = marks
        .iter()
        .filter(|r| r.label == "round.seed_build")
        .count();
    if found != 1 {
        return Err(SeedHandoffRefusal::SeedBuildMeasurementCount { found });
    }
    for row in marks.iter_mut().filter(|r| r.label == "round.seed_build") {
        row.wall_ms += c.wall_ms;
        row.cpu_ms = match (row.cpu_ms, c.cpu_ms) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        };
    }
    Ok((this_image_crates + c.compiled_crates, marks))
}

fn seed_cargo_build(workspace: &Path, label: &str) -> Result<CargoBuildObservation, String> {
    v1_rt::trace_mark(format!("{label}.begin"));
    let output = Command::new("cargo")
        // `--color never`: the compiled-crate count below parses cargo's `Compiling` lines, and an
        // inherited CARGO_TERM_COLOR=always (every workflow sets it) prefixes them with escape
        // codes, so the count read 0 while the build burned 655 s of CPU (run 37293307189).
        .args([
            "build",
            "--release",
            "--color",
            "never",
            "--bin",
            "claim_executor",
        ])
        // Ask cargo WHY it rebuilt each unit. The guard below reads `Compiling` lines and can name
        // the packages a scoped rebuild touched, but a refusal that names only them leaves the
        // mechanism to be inferred: three heal refusals (#14876 job 124081920078, #15310 run
        // 41098380134, #15441 run 41121984158) carried no evidence of WHY the excluded packages
        // were dirty. Logging only — it changes no route and no decision (DESIGN section 5) — and
        // the lines reach a refusal's text, never a digest.
        .env("CARGO_LOG", "cargo::core::compiler::fingerprint=info")
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn cargo build ({label}): {e}"))?;
    v1_rt::trace_mark(format!("{label}.done"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        // The fingerprint log is noise in a FAILURE tail: the compiler's own diagnostics must stay
        // the last thing a reader sees, not a `Dirty` line (§5: a refusal names the error).
        let kept: Vec<&str> = stderr
            .lines()
            .filter(|line| !line.contains("fingerprint"))
            .rev()
            .take(40)
            .collect();
        let tail: Vec<&str> = kept.into_iter().rev().collect();
        return Err(format!(
            "refusal: cargo build ({label}) failed with {} — last lines:\n{}",
            output.status,
            tail.join("\n")
        ));
    }
    let compiled_packages = stderr
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("Compiling "))
        .filter_map(|rest| rest.split_whitespace().next())
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let compiled_crates = compiled_packages.len() as u64;
    let fingerprint_reasons: Vec<String> = stderr
        .lines()
        .filter(|line| line.contains("fingerprint"))
        .map(|line| line.trim().to_string())
        .collect();
    Ok(CargoBuildObservation {
        compiled_crates,
        compiled_packages,
        fingerprint_reasons,
    })
}

/// The seed binary ON DISK at the path this process started from. After a cargo build replaces
/// that file, Linux reports the running image as `<path> (deleted)`; digesting the path itself
/// compares "installed there now" against "installed there before the build" -- the refusal's
/// question.
fn current_exe_on_disk() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let shown = exe.to_string_lossy().into_owned();
    Ok(PathBuf::from(
        shown.strip_suffix(" (deleted)").unwrap_or(&shown),
    ))
}

/// Open the path ONCE and digest the bytes read through that descriptor. The returned file is
/// the exec target: `/proc/self/fd/N` names this inode, so a later replacement of the path
/// cannot make the executed image differ from the hashed one.
fn open_exec_target(path: &Path) -> Result<(fs::File, String), String> {
    let mut file = fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let digest = digest_open_file(&mut file)?;
    Ok((file, digest))
}

/// Admit the exec target: the carry keeps the BUILD-OUTPUT digest, and the opened descriptor's
/// digest must equal it. Never overwrites the carry from the open.
fn admit_exec_target(
    carry: SeedHandoff,
    opened_digest: &str,
) -> Result<SeedHandoff, SeedHandoffRefusal> {
    if carry.built_exe != opened_digest {
        return Err(SeedHandoffRefusal::OpenedTargetNotBuildOutput {
            built: carry.built_exe,
            opened: opened_digest.to_string(),
        });
    }
    Ok(carry)
}

/// Digest of the bytes of an open file, read from its start.
fn digest_open_file(file: &mut fs::File) -> Result<String, String> {
    use std::io::{Read, Seek};
    file.rewind().map_err(|e| format!("rewind: {e}"))?;
    let mut bytes = std::vec::Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("read: {e}"))?;
    Ok(bytes_digest(&bytes))
}

/// The RUNNING image: `/proc/self/exe` opened directly, no installed-pathname reopen and no
/// ` (deleted)` strip. Three identities stay separate: the build output (`current_exe_digest`,
/// the installed path), the opened exec target (`open_exec_target`, carried as `built_exe`),
/// and this one.
fn running_image_digest() -> Result<String, String> {
    let mut file =
        fs::File::open("/proc/self/exe").map_err(|e| format!("open /proc/self/exe: {e}"))?;
    digest_open_file(&mut file)
}

fn current_exe_digest() -> Result<String, String> {
    let on_disk = current_exe_on_disk()?;
    let bytes = fs::read(&on_disk).map_err(|e| format!("read {}: {e}", on_disk.display()))?;
    Ok(bytes_digest(&bytes))
}

/// HEAD, the digest of the uncommitted tracked diff, and path+content digests of untracked,
/// non-ignored files (a new module or source the build consumes).
fn source_identity(workspace: &Path, head: &str) -> Result<String, String> {
    let run = |args: &[&str]| -> Result<Vec<u8>, String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(workspace)
            .output()
            .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(output.stdout)
    };
    let diff = run(&["diff", "HEAD", "--binary"])?;
    let listing = run(&["ls-files", "--others", "--exclude-standard", "-z"])?;
    let mut untracked = std::vec::Vec::new();
    for rel in listing.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        use std::os::unix::ffi::OsStrExt;
        let full = workspace.join(std::ffi::OsStr::from_bytes(rel));
        let content = fs::read(&full).map_err(|e| format!("read {}: {e}", full.display()))?;
        untracked.push((rel.to_vec(), content));
    }
    Ok(compose_source_identity(head, &diff, &mut untracked))
}

fn compose_source_identity(
    head: &str,
    tracked_diff: &[u8],
    untracked: &mut [(std::vec::Vec<u8>, std::vec::Vec<u8>)],
) -> String {
    untracked.sort_by(|a, b| a.0.cmp(&b.0));
    // Length-prefixed raw path bytes: no filename byte (`=`, newline, NUL) can move a
    // boundary between entries.
    let mut payload = std::vec::Vec::new();
    for (path, content) in untracked.iter() {
        payload.extend_from_slice(&(path.len() as u64).to_le_bytes());
        payload.extend_from_slice(path);
        payload.extend_from_slice(bytes_digest(content).as_bytes());
    }
    format!(
        "{head}+{}+{}",
        bytes_digest(tracked_diff),
        bytes_digest(&payload)
    )
}

fn git_tree_dirty(workspace: &Path) -> Result<bool, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("git status: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(!output.stdout.is_empty())
}

fn git_changed_stage0_paths(workspace: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--", "src/v1/stage0/src"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("git diff --name-only: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

fn host_name() -> String {
    // `/proc/sys/kernel/hostname` first (no libc buffer sizing), then the POSIX call; a host
    // that answers neither is reported as unreadable rather than as an empty name.
    if let Ok(name) = fs::read_to_string("/proc/sys/kernel/hostname") {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let mut buf = [0u8; 256];
    // SAFETY: `buf` is a live, fully-owned buffer; `gethostname` writes at most `buf.len()`
    // bytes into it and nothing else.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if rc == 0 {
        let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
        let name = String::from_utf8_lossy(&buf[..end]).to_string();
        if !name.is_empty() {
            return name;
        }
    }
    "unreadable".to_string()
}

fn round_cost_entry(source_roots: &[String]) -> Result<String, String> {
    source_roots
        .iter()
        .map(|root| Path::new(root).join(REGEN_ROUND_COST_ENTRY_UNDER_ROOT))
        .find(|candidate| candidate.is_file())
        .map(|found| found.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "refusal: {REGEN_ROUND_COST_ENTRY_UNDER_ROOT} is not under any declared source \
                 root {source_roots:?}, so the receipt has no renderer"
            )
        })
}

fn render_round_cost_receipt(
    source_roots: &[String],
    host: &str,
    tree: &str,
    tree_dirty: bool,
    seed_build_compiled_crates: u64,
    rebuild_compiled_crates: u64,
    rustfmt_spawns: u64,
    marks: &[v1_rt::TraceLedgerRow],
    changed_paths: &[String],
    convergence_stage_receipt_ids: &[String],
    installed_mirrors: &[String],
    host_shell_modules: &[String],
    rebuild_packages: &[String],
    executable_digest: &str,
    second_generation_candidate_digest: &str,
    base_mirror_authority: Option<(&str, usize, usize, usize)>,
) -> Result<String, String> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = round_cost_entry(source_roots)?;
    let (graph, indices) = resolve_orchestrator_model(source_roots, &entry).map_err(|e| {
        format!("refusal: {entry} did not resolve, so the receipt cannot render: {e}")
    })?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    // The model's carriers, built in the model's vocabulary: a duration is a
    // `std.measure` Nanosecond (`Measure { count }`) inside `std.observation` Measured, on a
    // `TimedMeasurement` row that names its clock. Milliseconds from the ledger become
    // nanoseconds here so no unit lives in a field name past this boundary.
    let nanosecond = |ms: u64| Value::Record {
        type_name: ctx.sym("Measure"),
        fields: Rc::new(vec![(
            ctx.sym("count"),
            Value::Int((ms * 1_000_000) as i64),
        )]),
    };
    let measured = |ms: Option<u64>| match ms {
        Some(ms) => Value::Variant {
            type_name: ctx.sym("Measured"),
            variant_name: ctx.sym("MeasuredValue"),
            fields: Rc::new(vec![(ctx.sym("value"), nanosecond(ms))]),
        },
        None => Value::Variant {
            type_name: ctx.sym("Measured"),
            variant_name: ctx.sym("MeasuredUnavailable"),
            fields: Rc::new(vec![(
                ctx.sym("cause"),
                str_value("process accounting unreadable"),
            )]),
        },
    };
    let timed = |basis: &str, ms: Option<u64>| Value::Record {
        type_name: ctx.sym("TimedMeasurement"),
        fields: Rc::new(vec![
            (
                ctx.sym("basis"),
                Value::Variant {
                    type_name: ctx.sym("ClockBasis"),
                    variant_name: ctx.sym(basis),
                    fields: Rc::new(vec![]),
                },
            ),
            (ctx.sym("elapsed"), measured(ms)),
        ]),
    };
    let mark_values: Vec<Value> = marks
        .iter()
        .map(|row| Value::Record {
            type_name: ctx.sym("TraceMark"),
            fields: Rc::new(vec![
                (ctx.sym("label"), str_value(row.label.clone())),
                (
                    ctx.sym("durations"),
                    Value::List(Rc::new(
                        vec![
                            timed("WallClock", Some(row.wall_ms)),
                            timed("CpuClock", row.cpu_ms),
                        ]
                        .into(),
                    )),
                ),
            ]),
        })
        .collect();
    let path_values: Vec<Value> = changed_paths.iter().map(str_value).collect();
    let stage_values: Vec<Value> = convergence_stage_receipt_ids
        .iter()
        .map(str_value)
        .collect();
    let installed_values: Vec<Value> = installed_mirrors.iter().map(str_value).collect();
    let receipt = Value::Record {
        type_name: ctx.sym("RegenRoundCostReceipt"),
        fields: Rc::new(vec![
            (ctx.sym("producer"), str_value(REGEN_ROUND_COST_PRODUCER)),
            (ctx.sym("host"), str_value(host)),
            (ctx.sym("tree"), str_value(tree)),
            (ctx.sym("tree_dirty"), Value::Bool(tree_dirty)),
            (
                ctx.sym("seed_build_compiled_crates"),
                Value::Int(seed_build_compiled_crates as i64),
            ),
            (
                ctx.sym("rebuild_compiled_crates"),
                Value::Int(rebuild_compiled_crates as i64),
            ),
            (ctx.sym("rustfmt_spawns"), Value::Int(rustfmt_spawns as i64)),
            (ctx.sym("marks"), Value::List(Rc::new(mark_values.into()))),
            (
                ctx.sym("changed_paths"),
                Value::List(Rc::new(path_values.into())),
            ),
            (
                ctx.sym("convergence_stage_receipt_ids"),
                Value::List(Rc::new(stage_values.into())),
            ),
            (
                ctx.sym("installed_mirrors"),
                Value::List(Rc::new(installed_values.into())),
            ),
            (
                ctx.sym("host_shell_modules"),
                model_string_list(host_shell_modules),
            ),
            (
                ctx.sym("rebuild_packages"),
                Value::List(Rc::new(
                    rebuild_packages
                        .iter()
                        .map(str_value)
                        .collect::<Vec<_>>()
                        .into(),
                )),
            ),
            (ctx.sym("executable_digest"), str_value(executable_digest)),
            (
                ctx.sym("second_generation_candidate_digest"),
                str_value(second_generation_candidate_digest),
            ),
            // WHICH MIRROR ROUND 1 COMPILED, as the model's own carrier. A heal round names the
            // base commit it resolved; any other round is `SeedBuiltFromHead`, which renders no
            // line at all. The commit -- never the ref -- is what is recorded: the ref is a
            // mutable name and this line is a record of a past round.
            (
                ctx.sym("base_mirror_authority"),
                match base_mirror_authority {
                    Some((commit, crate_paths, head_added_paths_kept, head_deleted_paths)) => {
                        Value::Variant {
                            type_name: ctx.sym("RoundOneMirrorAuthority"),
                            variant_name: ctx.sym("FromBaseMirror"),
                            fields: Rc::new(vec![
                                (ctx.sym("commit"), str_value(commit)),
                                (ctx.sym("crate_paths"), Value::Int(crate_paths as i64)),
                                (
                                    ctx.sym("head_added_paths_kept"),
                                    Value::Int(head_added_paths_kept as i64),
                                ),
                                (
                                    ctx.sym("head_deleted_paths"),
                                    Value::Int(head_deleted_paths as i64),
                                ),
                            ]),
                        }
                    }
                    None => Value::Variant {
                        type_name: ctx.sym("RoundOneMirrorAuthority"),
                        variant_name: ctx.sym("SeedBuiltFromHead"),
                        fields: Rc::new(vec![]),
                    },
                },
            ),
        ]),
    };
    let args = vec![(Some("receipt".to_string()), receipt)];
    let rendered = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "regen_round_cost_render", &args, false)
    })
    .map_err(|e| format!("refusal: regen_round_cost_render did not render: {e}"))?;
    match rendered {
        Value::Str(ref s) => Ok(s.to_string()),
        other => Err(format!(
            "refusal: regen_round_cost_render returned {} where a String was expected",
            other.type_label_public()
        )),
    }
}

struct PartitionRebuildActuation {
    actuatable: bool,
    package_closure: Vec<String>,
    excluded_packages: Vec<String>,
    decision_line: String,
    /// The model's ReleaseScopeEmpty arm: every changed mirror is excluded from the release
    /// build by construction (e.g. a `#[cfg(test)]`-gated module), so the correct package
    /// closure IS the empty set and the build runs purely as verification of that claim.
    release_scope_empty: bool,
    /// The decision is the positively modeled partition-generation class (the partition mirror
    /// moved). Whether that move is EXPLAINED is derived by the host, never read from here.
    partition_generation_move: bool,
    /// Projection of WholeCompilerRebuildRequired, including shared build inputs.
    whole_compiler_required: bool,
}

type ModelValue = crate::v1_interpreter::Value;

/// A CLAIM THAT RESOLVES THE LIVE CORPUS CANNOT BE MEASURED WITHOUT A READABLE HOST MEMORY BOUND,
/// and says so at its own boundary before it starts. Resolution refuses `HostBudgetUnreadable` on
/// such a host anyway (`cli_run::entry_resolve`); this states that precondition where the claim is
/// declared, as a red. It never returns early: an unmeasured claim that reports a pass is the
/// vacuous-pass class (`gunbc.recurring_failure_mode`
/// `missing_precondition_reported_as_a_pass`).
#[cfg(test)]
fn require_measurable_host_budget() {
    let budget = crate::memory_governor::read_host_budget_resolution();
    if budget.bytes().is_none() {
        panic!(
            "NOT MEASURED: host budget unreadable ({}). This claim resolves the live corpus and \
             cannot run without a readable memory bound. This is a red, not a pass.",
            budget.label()
        );
    }
}

fn model_string_list(xs: &[String]) -> ModelValue {
    use crate::v1_interpreter::str_value;
    ModelValue::List(Rc::new(
        xs.iter().map(str_value).collect::<Vec<ModelValue>>().into(),
    ))
}

fn model_value_to_string_list(value: &ModelValue, what: &str) -> Result<Vec<String>, String> {
    match value {
        ModelValue::List(items) => items
            .iter()
            .map(|item| match item {
                ModelValue::Str(s) => Ok(s.to_string()),
                other => Err(format!(
                    "refusal: {what} returned a {} where a String was expected",
                    other.type_label_public()
                )),
            })
            .collect(),
        other => Err(format!(
            "refusal: {what} returned {} where a List was expected",
            other.type_label_public()
        )),
    }
}

fn partition_rebuild_actuation(
    source_roots: &[String],
    installed_mirrors: &[String],
    host_shell_modules: &[String],
) -> Result<PartitionRebuildActuation, String> {
    use crate::v1_interpreter::{self, ExecutionMode};
    let entry = round_cost_entry(source_roots)?;
    let (graph, indices) = resolve_orchestrator_model(source_roots, &entry).map_err(|e| {
        format!("refusal: {entry} did not resolve, so the rebuild scope has no decider: {e}")
    })?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let call = |function: &str| -> Result<ModelValue, String> {
        let args = vec![
            (
                Some("changed_mirrors".to_string()),
                model_string_list(installed_mirrors),
            ),
            (Some("unlocatable".to_string()), model_string_list(&[])),
            (
                Some("host_shell_modules".to_string()),
                model_string_list(host_shell_modules),
            ),
        ];
        v1_interpreter::with_active_context(&ctx, || {
            v1_interpreter::run_in_context_with_args(&ctx, function, &args, false)
        })
        .map_err(|e| format!("refusal: {function} did not evaluate: {e}"))
    };
    let actuatable = match call("stage0_partition_rebuild_is_actuatable_today")? {
        ModelValue::Bool(value) => value,
        other => {
            return Err(format!(
                "refusal: stage0_partition_rebuild_is_actuatable_today returned {} instead of Bool",
                other.type_label_public()
            ))
        }
    };
    let package_closure = model_value_to_string_list(
        &call("stage0_partition_rebuild_packages_today")?,
        "stage0_partition_rebuild_packages_today",
    )?;
    let excluded_packages = model_value_to_string_list(
        &call("stage0_partition_rebuild_excluded_today")?,
        "stage0_partition_rebuild_excluded_today",
    )?;
    let decision_line = match call("stage0_partition_rebuild_decision_line_today")? {
        ModelValue::Str(ref value) => value.to_string(),
        other => {
            return Err(format!(
            "refusal: stage0_partition_rebuild_decision_line_today returned {} instead of String",
            other.type_label_public()
        ))
        }
    };
    let release_scope_empty = match call("stage0_partition_rebuild_release_scope_empty_today")? {
        ModelValue::Bool(value) => value,
        other => {
            return Err(format!(
                "refusal: stage0_partition_rebuild_release_scope_empty_today returned {} instead of Bool",
                other.type_label_public()
            ))
        }
    };
    let partition_generation_move =
        match call("stage0_partition_rebuild_is_partition_generation_move_today")? {
            ModelValue::Bool(value) => value,
            other => {
                return Err(format!(
                    "refusal: stage0_partition_rebuild_is_partition_generation_move_today returned {} instead of Bool",
                    other.type_label_public()
                ))
            }
        };
    let whole_compiler_required =
        match call("stage0_partition_rebuild_requires_whole_compiler_today")? {
            ModelValue::Bool(value) => value,
            other => {
                return Err(format!(
                    "refusal: stage0_partition_rebuild_requires_whole_compiler_today returned {} instead of Bool",
                    other.type_label_public()
                ))
            }
        };
    Ok(PartitionRebuildActuation {
        actuatable,
        package_closure,
        excluded_packages,
        decision_line,
        release_scope_empty,
        partition_generation_move,
        whole_compiler_required,
    })
}

/// A partition move is ADMITTED only when it is DERIVED, never asserted (no flag, no input, no
/// allow-list). Two conjuncts, both read from the tree and the change, neither from the mirror:
///   1. a changed input of the partition-generation mirrors is among THIS change's edits: the
///      decision is `stage0_partition_move_explained_by_affected_inputs`, over the arm and members
///      of the `gunbc.regen_affected_set` bound for the diff (plus the authority path), so the
///      mirror moved because an input moved and not because something drifted;
///   2. the boundary files the move needs are RENDERED FROM THE CANDIDATE TREE'S ROWS, not the
///      predecessor's compiled-in ones.
/// The converse equality (the built seed's own compiled-in rows project to these same bytes) is
/// checked after each build by `built_seed_boundary_drift`, whose answer is another loop population. Anything else keeps the
/// typed `WholeCompilerRebuildRequired` refusal.
/// THE DECISION ITSELF, in whichever process runs it: whether this change's edits explain the moved
/// partition mirror. It is the one step of a partition move that demands the whole-corpus closure
/// edge index, so the orchestrator never calls it -- `partition_move_explanation_child` runs it in a
/// child (`claim_executor --regen-partition-move-explanation`) and reads the answer back.
fn partition_move_explained_by_this_change(
    source_roots: &[String],
    workspace: &Path,
) -> Result<bool, String> {
    let (changed, _departed) = super::required_floor_runner::floor_git_diff_name_status_range()
        .map_err(|e| {
            format!(
                "PartitionMoveUnexplained: WholeCompilerRebuildRequired stands; the change's diff \
                 could not be read, so the move cannot be derived from it: {e}"
            )
        })?;
    let diff_text = super::required_floor_runner::floor_git_diff_range().map_err(|e| {
        format!(
            "PartitionMoveUnexplained: WholeCompilerRebuildRequired stands; the change's diff \
             could not be read, so the move cannot be derived from it: {e}"
        )
    })?;
    let unexplainable = |e: String| {
        format!(
            "PartitionMoveUnexplained: WholeCompilerRebuildRequired stands; the affected-set \
             bound could not be derived, so the move cannot be explained: {e}"
        )
    };
    let population = edited_population_from_diff(workspace, &diff_text);
    let (edges, modules) = regen_module_edges(workspace).map_err(unexplainable)?;
    let compared = compared_mirror_rows(&workspace.join("src/v1/stage0/src"), &modules)
        .map_err(unexplainable)?;
    match rebuild_scope_decision_with(
        source_roots,
        "stage0_partition_move_explained_by_affected_inputs",
        |ctx| {
            let mut args = affected_set_model_args(
                ctx,
                &population.edited_modules,
                &population.unlocatable,
                &edges,
                &compared,
            );
            args.push((
                Some("changed_paths".to_string()),
                model_string_list(&changed),
            ));
            Ok(args)
        },
    )? {
        ModelValue::Bool(value) => Ok(value),
        other => Err(format!(
            "refusal: stage0_partition_move_explained_by_affected_inputs returned {} instead of Bool",
            other.type_label_public()
        )),
    }
}

/// A partition move is ADMITTED only when it is DERIVED, never asserted (no flag, no input, no
/// allow-list): this change's edits explain the moved mirror (`partition_move_explained_by_this_change`,
/// answered by a child process), and the boundary files the move needs are rendered from the
/// candidate tree's rows. Anything else keeps the typed `WholeCompilerRebuildRequired` refusal.
fn admit_derived_partition_move(
    workspace: &Path,
    source_roots: &[String],
    liveness: &SupervisedSubjectModel,
) -> Result<crate::v1_compiler_stage0_crates::Stage0CrateBoundaryEmitOutcome, String> {
    match partition_move_explanation_child(workspace, source_roots, liveness)? {
        PartitionMoveExplanationReceipt::Explained { .. } => {}
        PartitionMoveExplanationReceipt::Unexplained { .. } => return Err(
            "PartitionMoveUnexplained: WholeCompilerRebuildRequired stands; the partition mirror \
                 moved but no changed input of it (gunbc.regen_affected_set) explains it"
                .to_string(),
        ),
        PartitionMoveExplanationReceipt::Refused { reason, .. } => return Err(reason),
    }
    let packages = emitted_tree_packages(workspace)?;
    let outcome = crate::v1_compiler_stage0_crates::partition_crate_boundary_emit_outcome_over(
        packages.rows.clone(),
        crate::v1_compiler_stage0_crates::stage0_foundation_runtime_dependencies(),
    );
    report_cgroup_memory_diagnostic("parent.partition_move.boundary_fold.end");
    Ok((*outcome).clone())
}

/// THE ONE ANSWER THE EXPLANATION CHILD HANDS BACK, and this enum is its only definition: it is a
/// host process-boundary carrier, encoded and decoded here and nowhere else, so no `.dag` type stands
/// beside it (review 90466: a model with no executing route would be a second, drifting authority).
/// The decision it carries is the model's -- `stage0_partition_move_explained_by_affected_inputs` --
/// and the three arms keep its outcomes apart: explained, unexplained (the decision answered no),
/// and refused (the decision could not be taken, with its cause), which is neither answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "answer", deny_unknown_fields)]
pub enum PartitionMoveExplanationReceipt {
    #[serde(rename = "explained")]
    Explained { schema: String },
    #[serde(rename = "unexplained")]
    Unexplained { schema: String },
    #[serde(rename = "refused")]
    Refused { schema: String, reason: String },
}

const PARTITION_MOVE_EXPLANATION_SCHEMA: &str = "gunbc.regen_partition_move_explanation.v1";
const PARTITION_MOVE_EXPLANATION_RECEIPT_REL: &str = "target/stage0-regen-partition-move.json";

/// The child's entry (`claim_executor --regen-partition-move-explanation`): take the decision with
/// the whole-corpus index resident in THIS process, write the answer, and exit -- process exit
/// frees the index. Every outcome, the refusal included, is written, so the orchestrator always
/// reads an answer rather than inferring one from an exit code.
pub fn run_partition_move_explanation(
    receipt_rel: &str,
    source_roots: &[String],
) -> Result<PartitionMoveExplanationReceipt, String> {
    let workspace = workspace_root();
    let schema = PARTITION_MOVE_EXPLANATION_SCHEMA.to_string();
    let receipt = match partition_move_explained_by_this_change(source_roots, &workspace) {
        Ok(true) => PartitionMoveExplanationReceipt::Explained { schema },
        Ok(false) => PartitionMoveExplanationReceipt::Unexplained { schema },
        Err(reason) => PartitionMoveExplanationReceipt::Refused { schema, reason },
    };
    write_child_answer(&workspace, receipt_rel, &receipt)?;
    Ok(receipt)
}

/// The orchestrator's side of the partition-move decision: one `answer_in_child` call.
fn partition_move_explanation_child(
    workspace: &Path,
    source_roots: &[String],
    liveness: &SupervisedSubjectModel,
) -> Result<PartitionMoveExplanationReceipt, String> {
    let mut mode = vec!["--regen-partition-move-explanation".to_string()];
    for root in source_roots {
        mode.push("--source-root".to_string());
        mode.push(root.clone());
    }
    answer_in_child(
        workspace,
        &mode,
        PARTITION_MOVE_EXPLANATION_RECEIPT_REL,
        liveness,
    )
}

/// ONE TYPED ANSWER HANDED ACROSS A PROCESS BOUNDARY by a child of this image. The answer's type is
/// its only definition (a host carrier, encoded and decoded here and nowhere else), and its schema
/// names the version both ends must agree on.
trait ChildAnswer: Serialize + serde::de::DeserializeOwned {
    /// The schema the child writes and the readback demands.
    const SCHEMA: &'static str;
    /// The question's name, used in the supervisor's heartbeat and in every refusal.
    const SUBJECT: &'static str;
    fn schema(&self) -> &str;
}

impl ChildAnswer for PartitionMoveExplanationReceipt {
    const SCHEMA: &'static str = PARTITION_MOVE_EXPLANATION_SCHEMA;
    const SUBJECT: &'static str = "partition-move explanation";
    fn schema(&self) -> &str {
        match self {
            Self::Explained { schema }
            | Self::Unexplained { schema }
            | Self::Refused { schema, .. } => schema,
        }
    }
}

/// THE ONE ROUTE BY WHICH THE ORCHESTRATOR ASKS A QUESTION WHOSE WORK IT MUST NOT HOLD (a
/// whole-corpus index, today): run it in a child of THIS image (`/proc/self/exe`, so the child is
/// exactly the orchestrator's own code even after a stage build replaced the executable on disk),
/// under the same liveness supervisor as a generation, and read back its one typed answer. Process
/// exit is the destructor for the child's work. A prior answer is retired first, and a missing,
/// unparseable or other-schema answer refuses, so nothing is ever inferred from an exit code or a
/// stale file.
fn answer_in_child<R: ChildAnswer>(
    workspace: &Path,
    mode_argv: &[String],
    receipt_rel: &str,
    liveness: &SupervisedSubjectModel,
) -> Result<R, String> {
    let subject = R::SUBJECT;
    let receipt_path = workspace.join(receipt_rel);
    if receipt_path.exists() {
        fs::remove_file(&receipt_path).map_err(|e| {
            format!(
                "refusal: cannot retire a prior {subject} answer {}: {e}",
                receipt_path.display()
            )
        })?;
    }
    let mut argv = mode_argv.to_vec();
    argv.push("--regen-receipt".to_string());
    argv.push(receipt_rel.to_string());
    report_cgroup_memory_diagnostic(&format!("parent.begin {subject}"));
    let run = spawn_rebuilt_seed_regen_child(
        Path::new("/proc/self/exe"),
        &argv,
        rebuilt_seed_regen_child_output_policy(),
        workspace,
        GenerationSupervision::SlotStallVerdict {
            subject: subject.to_string(),
            model: liveness,
        },
    );
    report_cgroup_memory_diagnostic(&format!("parent.end {subject}"));
    let run = run?;
    read_child_answer::<R>(&receipt_path).map_err(|reason| {
        format!(
            "{reason}; status={} stderr_tail={}{}",
            run.status,
            run.stderr_tail,
            rebuilt_seed_regen_truncation_note(run.stderr_truncated)
        )
    })
}

/// The child's side: write its one answer, whatever it is, so the orchestrator always reads one.
fn write_child_answer<R: ChildAnswer>(
    workspace: &Path,
    receipt_rel: &str,
    answer: &R,
) -> Result<(), String> {
    let path = workspace.join(receipt_rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    fs::write(
        &path,
        serde_json::to_vec(answer).map_err(|e| format!("encode {}: {e}", path.display()))?,
    )
    .map_err(|e| format!("write {}: {e}", path.display()))
}

/// Read a child's answer back: an absent, unparseable or other-schema answer refuses.
fn read_child_answer<R: ChildAnswer>(path: &Path) -> Result<R, String> {
    let subject = R::SUBJECT;
    let bytes = fs::read(path).map_err(|e| {
        format!(
            "refusal: the {subject} child wrote no answer at {}: {e}",
            path.display()
        )
    })?;
    let answer: R = serde_json::from_slice(&bytes)
        .map_err(|e| format!("refusal: parse {}: {e}", path.display()))?;
    if answer.schema() != R::SCHEMA {
        return Err(format!(
            "refusal: the {subject} answered in {}, not {}",
            answer.schema(),
            R::SCHEMA
        ));
    }
    Ok(answer)
}

/// THE ROUND'S CHECKPOINT SUBJECT, AS THE ONE ANSWER ITS CHILD HANDS BACK
/// (`claim_executor --regen-checkpoint-subject`). Two arms: the subject, or the reason it could not
/// be produced, which is not a subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "answer", deny_unknown_fields)]
pub enum CheckpointSubjectReceipt {
    #[serde(rename = "answered")]
    Answered {
        schema: String,
        subject: RegenConvergenceCheckpointSubject,
    },
    #[serde(rename = "refused")]
    Refused { schema: String, reason: String },
}

const CHECKPOINT_SUBJECT_SCHEMA: &str = "gunbc.regen_checkpoint_subject.v1";
const CHECKPOINT_SUBJECT_RECEIPT_REL: &str = "target/stage0-regen-checkpoint-subject.json";

impl ChildAnswer for CheckpointSubjectReceipt {
    const SCHEMA: &'static str = CHECKPOINT_SUBJECT_SCHEMA;
    const SUBJECT: &'static str = "checkpoint subject";
    fn schema(&self) -> &str {
        match self {
            Self::Answered { schema, .. } | Self::Refused { schema, .. } => schema,
        }
    }
}

/// The child's entry (`claim_executor --regen-checkpoint-subject`): produce the round's checkpoint
/// subject with the whole-tree index resident in THIS process, write it, and exit.
pub fn run_checkpoint_subject(receipt_rel: &str) -> Result<CheckpointSubjectReceipt, String> {
    let workspace = workspace_root();
    let schema = CHECKPOINT_SUBJECT_SCHEMA.to_string();
    let receipt = match current_convergence_checkpoint_subject(&workspace) {
        Ok(subject) => CheckpointSubjectReceipt::Answered { schema, subject },
        Err(reason) => CheckpointSubjectReceipt::Refused { schema, reason },
    };
    write_child_answer(&workspace, receipt_rel, &receipt)?;
    Ok(receipt)
}

/// The orchestrator's side: the round's checkpoint subject, from its child.
fn checkpoint_subject_child(
    workspace: &Path,
    liveness: &SupervisedSubjectModel,
) -> Result<RegenConvergenceCheckpointSubject, String> {
    match answer_in_child::<CheckpointSubjectReceipt>(
        workspace,
        &["--regen-checkpoint-subject".to_string()],
        CHECKPOINT_SUBJECT_RECEIPT_REL,
        liveness,
    )? {
        CheckpointSubjectReceipt::Answered { subject, .. } => Ok(subject),
        CheckpointSubjectReceipt::Refused { reason, .. } => {
            Err(format!("CheckpointSubjectRefused: {reason}"))
        }
    }
}

/// THE ROUND SUBJECT'S ONE PRODUCTION: if the slot is empty, ask `produce` once (the child) and
/// keep the answer; if it holds a value (established earlier in this image, or carried across the
/// re-exec), return it and produce nothing. `latest_generation` is the generation current at the
/// moment of a late establishment, whose own reading of the closure must agree with the produced
/// subject.
fn establish_round_subject<'s>(
    slot: &'s mut Option<RegenConvergenceCheckpointSubject>,
    latest_generation: Option<(usize, &str)>,
    produce: impl FnOnce() -> Result<RegenConvergenceCheckpointSubject, String>,
) -> Result<&'s RegenConvergenceCheckpointSubject, String> {
    if slot.is_none() {
        let produced = produce()?;
        if let Some((ordinal, observed)) = latest_generation {
            admit_generation_authority(&produced, ordinal, observed)?;
        }
        *slot = Some(produced);
    }
    slot.as_ref()
        .ok_or_else(|| "refusal: the round subject was not established".to_string())
}

fn regen_convergence_journal_exists(workspace: &Path) -> bool {
    regen_convergence_journal_path(workspace)
        .join("journal.json")
        .is_file()
}

/// THE ROUND'S ROLLBACK. A journal never exists without the round's subject: every journal write
/// is preceded by `establish_round_subject`. So no journal restores nothing, and a journal with no
/// established subject is a refusal, never a cue to produce one at teardown.
fn restore_round_journal(
    workspace: &Path,
    established: Option<&RegenConvergenceCheckpointSubject>,
) -> Result<RegenJournalRestore, String> {
    match established {
        Some(subject) => restore_regen_convergence_journal_for_subject(workspace, subject),
        None if !regen_convergence_journal_exists(workspace) => Ok(RegenJournalRestore::default()),
        None => Err(
            "CheckpointJournalWithoutRoundSubject: a convergence journal exists but this round \
             established no checkpoint subject; it was not written by this round, and the round \
             refuses rather than restore under a subject it did not produce"
                .to_string(),
        ),
    }
}

/// THE CARRIED SUBJECT IS CHECKED, NOT ASSUMED. Every generation loads the stage0 input closure
/// itself and reports its authority digest, so each one is a free reading of the fact the round
/// carries. A difference means the closure changed during the round (a `.dag` edit, or a rebuilt
/// seed that computes the closure differently) and the carried subject no longer names it, so the
/// round refuses rather than journal or restore under a stale subject.
fn admit_generation_authority(
    round_subject: &RegenConvergenceCheckpointSubject,
    ordinal: usize,
    observed: &str,
) -> Result<(), String> {
    if observed == round_subject.source_authority_digest {
        return Ok(());
    }
    Err(format!(
        "CheckpointSubjectDriftedMidRound: generation {ordinal} read the stage0 input closure as \
         {observed}, but the round's checkpoint subject carries {}; the carried subject no longer \
         names the round's sources",
        round_subject.source_authority_digest
    ))
}

/// The whole-compiler rebuild the partition-generation class requires: the closure is NOT shown
/// complete for a partition move, so the correct scope is everything. The boundary files are
/// journalled (the one rollback) and installed BEFORE the build that needs them.
fn partition_move_rebuild(
    workspace: &Path,
    subject: &RegenConvergenceCheckpointSubject,
    outcome: &crate::v1_compiler_stage0_crates::Stage0CrateBoundaryEmitOutcome,
) -> Result<CargoBuildObservation, String> {
    let files = match outcome {
        crate::v1_compiler_stage0_crates::Stage0CrateBoundaryEmitOutcome::Stage0CrateBoundaryEmitOk { files } => files,
        crate::v1_compiler_stage0_crates::Stage0CrateBoundaryEmitOutcome::Stage0CrateBoundaryEmitRefused { cause } => {
            return Err(format!(
                "PartitionBoundaryCarrierRefused: the candidate tree's partition does not render: {cause:?}"
            ))
        }
    };
    let entries = files
        .iter()
        .map(|f| (f.path.clone(), workspace.join(&f.path)))
        .collect::<Vec<_>>();
    journal_paths_for_subject(workspace, &entries, subject)?;
    match super::partition_crate_boundary_host::run_partition_crate_boundary_over(outcome, true) {
        super::partition_crate_boundary_host::PartitionCrateBoundaryOutcome::CarrierRefused {
            cause,
        } => return Err(format!("PartitionBoundaryWriteRefused: {cause}")),
        super::partition_crate_boundary_host::PartitionCrateBoundaryOutcome::Rendered {
            ..
        } => {}
    }
    seed_cargo_build(workspace, "round.rebuild_from_installed")
}

/// THE BOUNDARY POPULATION OF THE FIXED POINT: the built seed projects, from its OWN compiled-in
/// rows and renderer, the partition boundary files. `Ok(vec![])` is agreement; a nonempty answer
/// names the workspace-relative files the rebuilt producer would change, which are one more
/// drifted population of the loop (another install and rebuild), never a terminal refusal. A
/// producer that exits nonzero WITHOUT naming a drifted file did not observe, and refuses.
fn built_seed_boundary_drift(workspace: &Path) -> Result<Vec<String>, String> {
    let on_disk = current_exe_on_disk()?;
    let output = Command::new(&on_disk)
        .arg("--emit-partition-crates")
        .current_dir(workspace)
        .output()
        .map_err(|e| {
            format!(
                "PartitionProjectionUnobserved: spawn {}: {e}",
                on_disk.display()
            )
        })?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let drifted = boundary_drift_from_producer_report(&output.stdout).ok_or_else(|| {
        format!(
            "PartitionProjectionUnobserved: the built seed returned no typed boundary report: {stderr}"
        )
    })?;
    if output.status.success() != drifted.is_empty() {
        return Err(format!(
            "PartitionProjectionUnobserved: exit status and typed report disagree: {stderr}"
        ));
    }
    Ok(drifted)
}

fn boundary_drift_from_producer_report(stdout: &[u8]) -> Option<Vec<String>> {
    let report: serde_json::Value = serde_json::from_slice(stdout).ok()?;
    if report.get("schema")?.as_str()? != "gunbc.partition_boundary_report.v1" {
        return None;
    }
    report
        .get("divergent")?
        .as_array()?
        .iter()
        .map(|p| p.as_str().map(str::to_string))
        .collect()
}

/// Install the rebuilt producer's boundary output: journal the files first (the one rollback),
/// then let the built seed write them.
/// Install the built seed's boundary files, journalled under the round's established subject (a
/// boundary install always follows a stage install, which established it; the journal write refuses
/// if the journal on disk names another subject).
fn install_built_seed_boundary(
    workspace: &Path,
    drifted: &[String],
    subject: Option<&RegenConvergenceCheckpointSubject>,
) -> Result<(), String> {
    let subject = subject.ok_or_else(|| {
        "CheckpointJournalWithoutRoundSubject: a boundary install was asked to journal before any \
         stage install established the round's checkpoint subject"
            .to_string()
    })?;
    let entries = drifted
        .iter()
        .map(|path| (path.clone(), workspace.join(path)))
        .collect::<Vec<_>>();
    journal_paths_for_subject(workspace, &entries, subject)?;
    let on_disk = current_exe_on_disk()?;
    let status = Command::new(&on_disk)
        .args(["--emit-partition-crates", "--write"])
        .current_dir(workspace)
        .output()
        .map_err(|e| {
            format!(
                "PartitionBoundaryWriteRefused: spawn {}: {e}",
                on_disk.display()
            )
        })?;
    if !status.status.success() {
        return Err(format!(
            "PartitionBoundaryWriteRefused: {}",
            String::from_utf8_lossy(&status.stderr)
        ));
    }
    Ok(())
}

fn partitioned_rebuild_from_installed(
    workspace: &Path,
    actuation: &PartitionRebuildActuation,
) -> Result<CargoBuildObservation, String> {
    if !actuation.actuatable {
        return Err(format!(
            "StageSeedBuildRefused: no partitioned build was attempted and no full-build fallback ran -- {}",
            actuation.decision_line
        ));
    }
    if actuation.package_closure.is_empty() && !actuation.release_scope_empty {
        return Err(format!(
            "StageSeedBuildRefused: actuation admitted an empty package closure -- {}",
            actuation.decision_line
        ));
    }
    let observation = seed_cargo_build(workspace, "round.rebuild_from_installed")?;
    let widened = observation
        .compiled_packages
        .iter()
        .filter(|package| actuation.excluded_packages.contains(package))
        .collect::<Vec<_>>();
    if !widened.is_empty() {
        // The guard names what it refused; cargo knows WHY it rebuilt it. Prefer the lines that
        // mention a refused package, and fall back to the whole capture so a refusal of this shape
        // never arrives without the reason attached.
        let mut reasons: Vec<&String> = observation
            .fingerprint_reasons
            .iter()
            .filter(|line| {
                widened
                    .iter()
                    .any(|package| line.contains(package.as_str()))
            })
            .collect();
        if reasons.is_empty() {
            reasons = observation.fingerprint_reasons.iter().collect();
        }
        let shown: Vec<&str> = reasons
            .iter()
            .take(24)
            .map(|reason| reason.as_str())
            .collect();
        return Err(format!(
            "StageSeedBuildRefused: compiled excluded packages {widened:?} -- {} — cargo fingerprint \
             reasons for this build ({} captured, {} shown): [{}]",
            actuation.decision_line,
            observation.fingerprint_reasons.len(),
            shown.len(),
            shown.join(" | ")
        ));
    }
    Ok(observation)
}

fn next_pass_executable_digest(workspace: &Path) -> Result<String, String> {
    let executable = workspace.join("target/release/claim_executor");
    fs::read(&executable)
        .map(|bytes| bytes_digest(&bytes))
        .map_err(|e| {
            format!(
                "StageOutputExecutableUnbound: read {}: {e}",
                executable.display()
            )
        })
}

struct FirstGenerationConvergenceObservation {
    committed_digest: String,
    drifted: Vec<String>,
    authority_digest: String,
    manifest: RegenCandidateManifest,
    surface_roles: ConvergenceSurfaceRoles,
}

fn read_first_generation_receipt(
    path: &Path,
) -> Result<FirstGenerationConvergenceObservation, String> {
    let receipt: RegenReceipt = serde_json::from_slice(
        &fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", path.display()))?;
    match receipt {
        RegenReceipt::FirstGeneration {
            candidate_generated_digest: _,
            committed_generated_digest,
            changed_paths,
            candidate_artifact: _,
            authority_digest,
            candidate_manifest,
            surface_roles,
            ..
        } => Ok(FirstGenerationConvergenceObservation {
            committed_digest: committed_generated_digest,
            drifted: changed_paths,
            authority_digest,
            manifest: candidate_manifest,
            surface_roles,
        }),
        RegenReceipt::Refused { reason, .. } => Err(format!(
            "regen refused before convergence planning: {reason}"
        )),
        RegenReceipt::NoAffectedMirrors { scope, .. } => Err(format!(
            "regen convergence has no affected mirrors for {scope}"
        )),
    }
}

fn run_built_seed_regen(
    workspace: &Path,
    candidate_dir_rel: &str,
    receipt_rel: &str,
    source_roots: &[String],
    affected_scope: bool,
    admitted_executable_digest: &str,
    ordinal: usize,
    liveness: &SupervisedSubjectModel,
    round_subject: Option<&RegenConvergenceCheckpointSubject>,
) -> Result<FirstGenerationConvergenceObservation, String> {
    let receipt_path = workspace.join(receipt_rel);
    if receipt_path.exists() {
        fs::remove_file(&receipt_path).map_err(|e| {
            format!(
                "refusal: cannot retire prior generation receipt {} before rebuilt-seed emit: {e}",
                receipt_path.display()
            )
        })?;
    }
    let on_disk = current_exe_on_disk()?;
    // The admitted side carries the executable-digest spelling (`current_exe_digest`,
    // `next_pass_executable_digest`); the tagged file digest (`path_digest`) spells the same
    // bytes differently, and this gate compared them verbatim -- unpassable since #9771.
    let observed_executable_digest = current_exe_digest()?;
    if observed_executable_digest != admitted_executable_digest {
        return Err(format!(
            "CandidateGeneratedByDifferentSeed: stage admitted executable {} but next generation would run {} at {}",
            admitted_executable_digest,
            observed_executable_digest,
            on_disk.display()
        ));
    }
    let mut argv: Vec<String> = vec!["--required-regen".to_string()];
    if affected_scope {
        argv.push("--regen-affected-scope".to_string());
    }
    argv.push("--regen-candidate-dir".to_string());
    argv.push(candidate_dir_rel.to_string());
    argv.push("--regen-receipt".to_string());
    argv.push(receipt_rel.to_string());
    for root in source_roots {
        argv.push("--source-root".to_string());
        argv.push(root.clone());
    }
    // THE CHILD'S OUTPUT IS FORWARDED, NOT BUFFERED-AND-DROPPED. This used to run on
    // `Command::output()`, which buffers stdout AND stderr in memory and hands them to nobody
    // unless a refusal fires. The child is another `gunbc --required-regen`, and its per-phase
    // `started …`/`… done in …` marks come out of `v1_rt::trace_mark` on STDERR -- `eprintln!`,
    // verified at `src/v1/stage0/src/v1_rt.rs` `trace_mark`, not assumed -- along with every cargo
    // and rustc diagnostic. Under `output()` all of it sat in two `String`s that only the refusal
    // arms read (12 stderr lines), and everything else was dropped. That is how a 53-minute phase
    // could sit completely silent in the job log while the child worked: the lines existed on the
    // child's stderr and we were throwing that stream away. Both ends are now piped and forwarded
    // to OUR stderr and stdout AS THEY ARRIVE, through one `drain_stream` call each from
    // `gunbc.bounded_shell_host_drain` -- the crate's single authority for capture semantics -- so
    // nothing is buffered on our side, no scratch file is left behind, and the bounded tail the
    // refusal arms need comes from the same authority's ring rather than a second implementation.
    let run = spawn_rebuilt_seed_regen_child(
        &on_disk,
        &argv,
        rebuilt_seed_regen_child_output_policy(),
        workspace,
        GenerationSupervision::SlotStallVerdict {
            subject: format!("generation {ordinal}"),
            model: liveness,
        },
    )?;
    // Drift makes --required-regen exit one after it has written the candidate and receipt. The
    // receipt, not the process code, distinguishes ordinary drift from a production refusal.
    if !receipt_path.is_file() {
        return Err(rebuilt_seed_regen_missing_receipt_refusal(
            &receipt_path,
            &run,
        ));
    }
    let receipt = read_first_generation_receipt(&receipt_path);
    report_cgroup_memory_diagnostic("parent.readback");
    match receipt {
        Ok(value) => {
            // Checked against the carried subject whenever one is established; a subject
            // established later is checked against the generation current at that moment
            // (`establish_round_subject`).
            if let Some(subject) = round_subject {
                admit_generation_authority(subject, ordinal, &value.authority_digest)?;
            }
            Ok(value)
        }
        Err(reason) => Err(format!(
            "{reason}; {}",
            rebuilt_seed_regen_failed_refusal(&run)
        )),
    }
}
/// Where the rebuilt-seed child's two output ends go.
enum RebuiltSeedChildOutput {
    /// Production: every byte the child writes is forwarded to OUR stdout and stderr the moment it
    /// arrives, which is the job log. This is the deliverable -- the child's phase marks live on its
    /// stderr (`v1_rt::trace_mark` uses `eprintln!`) and were being read by nobody.
    ForwardToJobLog,
    /// Test seam: the same forwarding, pointed at two files, so a fixture can assert what the
    /// parent would have shown and WHEN it showed it. The destination is the only difference.
    #[cfg(test)]
    ForwardToFiles { stdout: PathBuf, stderr: PathBuf },
}

/// What the rebuilt-seed child's output is handed. Named as a function so the fixture test can
/// assert the CHOICE and not only that the forwarding path works: the two ways this deliverable
/// regresses are (a) the policy moved off the job log, and (b) the forwarding dropped back to a
/// buffer nobody reads. One leg of the test goes red for each.
fn rebuilt_seed_regen_child_output_policy() -> RebuiltSeedChildOutput {
    RebuiltSeedChildOutput::ForwardToJobLog
}

/// Read from `source`, write to `sink`, hand the bytes on untouched. A sink that cannot be written
/// (a log pipe that closed) must not fail the regeneration pass, so write errors are dropped: the
/// caller is paid to regenerate the seed, not to guarantee a log.
struct TeeReader<R: std::io::Read> {
    source: R,
    sink: Box<dyn std::io::Write + Send>,
}

impl<R: std::io::Read> std::io::Read for TeeReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.source.read(buf)?;
        if read > 0 {
            let _ = self.sink.write_all(&buf[..read]);
            let _ = self.sink.flush();
        }
        Ok(read)
    }
}

/// What one rebuilt-seed regeneration pass left behind: the child's exit status, and the bounded
/// stderr evidence its refusals quote.
struct RebuiltSeedChildRun {
    status: std::process::ExitStatus,
    stderr_tail: String,
    stderr_truncated: bool,
}

/// Spawn the rebuilt-seed regeneration child, forward both of its output ends as they arrive, and
/// wait for it. One function owns both ends, so the fixture test exercises the production call
/// rather than a copy of it, and the capture semantics come from `bounded_shell_host_drain` -- the
/// crate's single authority -- rather than a second drain written here.
fn spawn_rebuilt_seed_regen_child(
    program: &Path,
    argv: &[String],
    output_policy: RebuiltSeedChildOutput,
    working_dir: &Path,
    supervision: GenerationSupervision<'_>,
) -> Result<RebuiltSeedChildRun, String> {
    let (stdout_sink, stderr_sink): (
        Box<dyn std::io::Write + Send>,
        Box<dyn std::io::Write + Send>,
    ) = match output_policy {
        RebuiltSeedChildOutput::ForwardToJobLog => {
            (Box::new(std::io::stdout()), Box::new(std::io::stderr()))
        }
        #[cfg(test)]
        RebuiltSeedChildOutput::ForwardToFiles { stdout, stderr } => (
            Box::new(
                fs::File::create(&stdout)
                    .map_err(|error| format!("create {}: {error}", stdout.display()))?,
            ),
            Box::new(
                fs::File::create(&stderr)
                    .map_err(|error| format!("create {}: {error}", stderr.display()))?,
            ),
        ),
    };
    let mut command = Command::new(program);
    command
        .args(argv)
        .current_dir(working_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    // THE GENERATION IS ONE PROCESS GROUP, so a refusal ends the whole generation -- the child and
    // every process it spawned -- in one signal, rather than the direct child alone with its
    // descendants left holding the slot's memory (`crate::process_group`, the crate's one authority
    // for group lifetime).
    let mut child = crate::process_group::spawn_in_new_process_group(&mut command)
        .map_err(|error| format!("spawn rebuilt seed {}: {error}", program.display()))?;
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let stderr_sink: Box<dyn std::io::Write + Send> = Box::new(ProgressMarkScanner {
        sink: stderr_sink,
        partial: Vec::new(),
        marks: progress.clone(),
    });
    // Drain on our side while the child runs: a pipe nobody reads would eventually block it, and
    // reading here is also what makes the forwarding live rather than post-mortem.
    let stdout_handle = stdout_pipe.map(|pipe| {
        std::thread::spawn(move || {
            drain_stream(
                TeeReader {
                    source: pipe,
                    sink: stdout_sink,
                },
                StreamCapturePolicy::Discarded,
            )
        })
    });
    let stderr_handle = stderr_pipe.map(|pipe| {
        std::thread::spawn(move || {
            drain_stream(
                TeeReader {
                    source: pipe,
                    sink: stderr_sink,
                },
                default_shell_stderr_capture_policy(),
            )
        })
    });
    let supervised = supervise_generation(&mut child, &progress, supervision);
    let status = match supervised {
        GenerationEnding::Exited(status) => status,
        GenerationEnding::WaitFailed(detail) => {
            return Err(format!(
                "wait for rebuilt seed {}: {detail}",
                program.display()
            ))
        }
        GenerationEnding::Refused {
            refusal,
            teardown,
            settled,
        } => {
            // A SETTLED teardown means no member of the group survives, so both pipes are at EOF
            // and the drains end. An UNSETTLED one may have left a descendant holding a pipe, and
            // joining its drain would hang the orchestrator at exactly the moment it has a refusal
            // to report -- so the drains are left behind and the refusal says the group may live.
            if settled {
                if let Some(handle) = stdout_handle {
                    let _ = handle.join();
                }
                if let Some(handle) = stderr_handle {
                    let _ = handle.join();
                }
            }
            return Err(format!("{refusal} teardown={teardown}"));
        }
    };
    if let Some(handle) = stdout_handle {
        handle
            .join()
            .map_err(|_| "the rebuilt-seed child's stdout drain panicked".to_string())?
            .map_err(|error| format!("drain the rebuilt-seed child's stdout: {error}"))?;
    }
    let observation = stderr_handle
        .map(|handle| {
            handle
                .join()
                .map_err(|_| "the rebuilt-seed child's stderr drain panicked".to_string())?
                .map_err(|error| format!("drain the rebuilt-seed child's stderr: {error}"))
        })
        .transpose()?
        .ok_or("the rebuilt-seed child's stderr pipe is missing")?;
    Ok(RebuiltSeedChildRun {
        status,
        stderr_tail: rebuilt_seed_regen_stderr_tail(&observation),
        stderr_truncated: observation.truncated,
    })
}

/// HOW A GENERATION'S CHILD IS WATCHED WHILE IT RUNS.
///
/// The authority is `gunbc.host.memory_stall_refusal` `supervised_subject_verdict`, evaluated
/// through `SupervisedSubjectModel` over readings `memory_governor` takes: a generation that emits no progress mark for the declared window while its
/// slot shows memory-envelope evidence is refused and its process group ended, instead of holding
/// under memory.high until the step timeout -- the srv3 heal ending, where a swapless slot produced
/// neither an OOM kill nor swap for the exit-time envelope verdict to read. The supervisor is named
/// for the subject it watches (a generation), not for heal's round 2, so another subject step under
/// the same slot contract enrolls by supplying the same window.
pub(crate) enum GenerationSupervision<'a> {
    /// Production: read the slot counters from this process's own cgroup (the child inherits it).
    SlotStallVerdict {
        /// What is watched, as the heartbeat and the refusal name it: `generation N`, or the
        /// partition-move explanation child.
        subject: String,
        model: &'a SupervisedSubjectModel,
    },
    /// Test seam: refuse after a fixed wall, so the teardown route is executed for real without a
    /// memory-starved slot.
    #[cfg(test)]
    RefuseAfter { after: std::time::Duration },
}

/// How often the supervisor samples the slot, and how often it prints a heartbeat. The heartbeat
/// is observability only: it is not progress, and the verdict never reads it.
const GENERATION_SUPERVISOR_SAMPLE: std::time::Duration = std::time::Duration::from_secs(10);
const GENERATION_SUPERVISOR_HEARTBEAT: std::time::Duration = std::time::Duration::from_secs(120);
const GENERATION_TEARDOWN_GRACE: std::time::Duration = std::time::Duration::from_secs(10);

/// A child stderr line is a PROGRESS MARK exactly when it is a phase-boundary line `trace_mark`
/// renders -- recognised through the renderer mirrors themselves (`v1_rt`
/// `render_phase_begin_line_mirror` / `render_phase_concluded_line_mirror`, authority
/// `gunbc.observation_seed_render`), never through a second spelling of their glyphs. Ordinary
/// output, cargo lines and the heartbeat are not progress: semantic advancement is a phase boundary.
fn is_generation_progress_mark(line: &str) -> bool {
    let line = line.trim_end_matches(['\r', '\n']);
    let phase_shaped = |phase: &str| !phase.is_empty() && !phase.contains(char::is_whitespace);
    for emoji in [true, false] {
        let begin_prefix = v1_rt::render_phase_begin_line_mirror("", emoji);
        if let Some(phase) = line.strip_prefix(begin_prefix.as_str()) {
            if phase_shaped(phase) && v1_rt::render_phase_begin_line_mirror(phase, emoji) == line {
                return true;
            }
        }
        let probe = v1_rt::render_phase_concluded_line_mirror("\u{0}", 0, emoji);
        let Some((head, tail)) = probe.split_once('\u{0}') else {
            continue;
        };
        let Some(between) = tail.strip_suffix(v1_rt::obs_human_duration(0).as_str()) else {
            continue;
        };
        if let Some(rest) = line.strip_prefix(head) {
            if let Some((phase, _elapsed)) = rest.split_once(between) {
                if phase_shaped(phase) {
                    return true;
                }
            }
        }
    }
    false
}

/// Forwards every byte to the job log untouched and counts the progress marks among the complete
/// lines it carries. A line longer than the buffer is not a mark (marks are short), so the buffer
/// is bounded and an overlong line is simply never matched.
struct ProgressMarkScanner {
    sink: Box<dyn std::io::Write + Send>,
    partial: Vec<u8>,
    marks: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

const PROGRESS_MARK_LINE_LIMIT: usize = 4096;

impl std::io::Write for ProgressMarkScanner {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = self.sink.write_all(buf);
        for &byte in buf {
            if byte == b'\n' {
                if self.partial.len() <= PROGRESS_MARK_LINE_LIMIT
                    && is_generation_progress_mark(&String::from_utf8_lossy(&self.partial))
                {
                    self.marks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
                self.partial.clear();
            } else if self.partial.len() <= PROGRESS_MARK_LINE_LIMIT {
                self.partial.push(byte);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.sink.flush()
    }
}

/// The supervisor's decision per sample, pure over (time, mark count, slot reading) so the arms are
/// driven by a test without a process or a slot.
struct GenerationSupervisor {
    subject: String,
    begun: Instant,
    marks_seen: u64,
    /// The slot reading the current window is measured from: taken when the last progress mark was
    /// observed (or at the generation's start). Every counter is cumulative and slot-wide, so the
    /// verdict reads differences from here, never absolute values.
    baseline: Option<(Instant, crate::memory_governor::SlotStallCounters)>,
    last_heartbeat: Instant,
}

enum SupervisorStep {
    Continue { heartbeat: Option<String> },
    Refuse { refusal: String },
}

impl GenerationSupervisor {
    fn new(subject: String, now: Instant) -> Self {
        GenerationSupervisor {
            subject,
            begun: now,
            marks_seen: 0,
            baseline: None,
            last_heartbeat: now,
        }
    }

    fn observe(
        &mut self,
        model: &SupervisedSubjectModel,
        now: Instant,
        marks: u64,
        reading: crate::memory_governor::SlotStallReading,
    ) -> SupervisorStep {
        use crate::memory_governor::SlotStallReading;
        let heartbeat_due =
            now.saturating_duration_since(self.last_heartbeat) >= GENERATION_SUPERVISOR_HEARTBEAT;
        let elapsed_s = now.saturating_duration_since(self.begun).as_secs();
        let counters = match reading {
            SlotStallReading::Read(counters) => counters,
            // NOT A ZERO. Without the slot's counters there is no memory evidence to judge, so the
            // generation runs unsupervised and every heartbeat says so -- a typed, printed
            // degradation, never a fabricated reading that admits.
            SlotStallReading::Unreadable { cause } => {
                self.baseline = None;
                return SupervisorStep::Continue {
                    heartbeat: heartbeat_due.then(|| {
                        self.last_heartbeat = now;
                        format!(
                            "required-regen: {} supervisor elapsed_s={elapsed_s} \
                             progress_marks={marks} supervision=unavailable cause={cause}",
                            self.subject
                        )
                    }),
                };
            }
        };
        if marks != self.marks_seen || self.baseline.is_none() {
            self.marks_seen = marks;
            self.baseline = Some((now, counters.clone()));
        }
        let (since, base) = self.baseline.as_ref().expect("baseline set above");
        let wall_ms = now.saturating_duration_since(*since).as_millis() as u64;
        let memory_current = counters
            .memory_current
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".to_string());
        let (window_line, refusal) = match model.judge(base, &counters, wall_ms) {
            Ok(judged) => judged,
            Err(failure) => {
                return SupervisorStep::Refuse {
                    refusal: format!("{failure} [{} after {elapsed_s} s]", self.subject),
                }
            }
        };
        let receipt =
            format!("progress_marks={marks} slot_memory_current={memory_current} {window_line}");
        if let Some(refusal) = refusal {
            return SupervisorStep::Refuse {
                refusal: format!(
                    "{refusal} [{} after {elapsed_s} s; {receipt}]",
                    self.subject
                ),
            };
        }
        SupervisorStep::Continue {
            heartbeat: heartbeat_due.then(|| {
                self.last_heartbeat = now;
                format!(
                    "required-regen: {} supervisor elapsed_s={elapsed_s} {receipt}",
                    self.subject
                )
            }),
        }
    }
}

enum GenerationEnding {
    Exited(std::process::ExitStatus),
    WaitFailed(String),
    Refused {
        refusal: String,
        teardown: String,
        settled: bool,
    },
}

/// Wait for the generation, sampling the slot between polls. On a refusal the group is ended while
/// the child is still unreaped -- `try_wait` returning `None` is what proves the pid, and so the
/// group identity, is still ours -- through `process_group::terminate_process_group`, which signals
/// before it reaps.
fn supervise_generation(
    child: &mut std::process::Child,
    progress: &std::sync::atomic::AtomicU64,
    supervision: GenerationSupervision<'_>,
) -> GenerationEnding {
    let begun = Instant::now();
    let slot_dir = crate::memory_governor::leaf_cgroup_dir();
    let mut supervisor = match &supervision {
        GenerationSupervision::SlotStallVerdict { subject, .. } => {
            Some(GenerationSupervisor::new(subject.clone(), begun))
        }
        #[cfg(test)]
        GenerationSupervision::RefuseAfter { .. } => None,
    };
    let mut next_sample = begun;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return GenerationEnding::Exited(status),
            Ok(None) => {}
            Err(error) => return GenerationEnding::WaitFailed(error.to_string()),
        }
        let now = Instant::now();
        let refusal = if now < next_sample {
            None
        } else {
            next_sample = now + GENERATION_SUPERVISOR_SAMPLE;
            match (&supervision, supervisor.as_mut()) {
                (GenerationSupervision::SlotStallVerdict { model, .. }, Some(supervisor)) => {
                    let reading = match slot_dir.as_deref() {
                        Some(dir) => crate::memory_governor::read_slot_stall_counters(dir),
                        None => crate::memory_governor::SlotStallReading::Unreadable {
                            cause: "no cgroup v2 path in /proc/self/cgroup".to_string(),
                        },
                    };
                    match supervisor.observe(
                        model,
                        now,
                        progress.load(std::sync::atomic::Ordering::SeqCst),
                        reading,
                    ) {
                        SupervisorStep::Continue { heartbeat } => {
                            if let Some(line) = heartbeat {
                                eprintln!("{line}");
                            }
                            None
                        }
                        SupervisorStep::Refuse { refusal } => Some(refusal),
                    }
                }
                #[cfg(test)]
                (GenerationSupervision::RefuseAfter { after }, _) => {
                    (now.saturating_duration_since(begun) >= *after)
                        .then(|| "fixture refusal".to_string())
                }
                #[allow(unreachable_patterns)]
                _ => None,
            }
        };
        if let Some(refusal) = refusal {
            let (teardown, settled) =
                match crate::process_group::pin_process_group_identity(child.id()) {
                    Ok(identity) => {
                        let terminal = crate::process_group::terminate_process_group(
                            child,
                            &identity,
                            GENERATION_TEARDOWN_GRACE,
                        );
                        (terminal.state_debug(), terminal.is_settled())
                    }
                    Err(lost) => (format!("not signalled: {lost:?}"), false),
                };
            return GenerationEnding::Refused {
                refusal,
                teardown,
                settled,
            };
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

/// The bounded tail both refusal arms render, kept as ONE construction so a refusal reads the same
/// whichever arm fired: the LAST 12 lines the authority's ring retained, in the child's own order.
/// The double reverse is the spelling the text has always carried and the fixture test pins it. The
/// ring keeps `DEFAULT_SHELL_STDERR_TAIL_BYTES`; if the tail was cut, `stderr_truncated` says so in
/// the refusal instead of the reader finding out by counting lines.
fn rebuilt_seed_regen_stderr_tail(
    observation: &crate::v1_interpreter::bounded_shell_host_drain::StreamCaptureObservation,
) -> String {
    observation
        .retained_utf8_lossy_trimmed()
        .lines()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The truncation marker, and only when the authority's ring actually overflowed -- so the ordinary
/// refusal text stays byte-for-byte what it has always been.
fn rebuilt_seed_regen_truncation_note(truncated: bool) -> &'static str {
    if truncated {
        " stderr_truncated=true"
    } else {
        ""
    }
}

fn rebuilt_seed_regen_missing_receipt_refusal(
    receipt_path: &Path,
    run: &RebuiltSeedChildRun,
) -> String {
    format!(
        "rebuilt seed produced no fresh generation receipt at {}; status={} stderr_tail={}{}",
        receipt_path.display(),
        run.status,
        run.stderr_tail,
        rebuilt_seed_regen_truncation_note(run.stderr_truncated)
    )
}

fn rebuilt_seed_regen_failed_refusal(run: &RebuiltSeedChildRun) -> String {
    format!(
        "rebuilt seed exited {}; expected a successful regeneration pass stderr_tail={}{}",
        run.status,
        run.stderr_tail,
        rebuilt_seed_regen_truncation_note(run.stderr_truncated)
    )
}

/// `memory.stat` keys the diagnostic prints: the anonymous/file split and the swap-relevant lines.
/// A key the kernel does not report is printed `absent`, never defaulted.
const CGROUP_DIAGNOSTIC_STAT_KEYS: &[&str] = &[
    "anon",
    "file",
    "kernel",
    "slab",
    "shmem",
    "file_mapped",
    "file_dirty",
    "file_writeback",
    "swapcached",
    "inactive_anon",
    "active_anon",
    "inactive_file",
    "active_file",
    "unevictable",
];

fn cgroup_stat_field(stat_body: &str, key: &str) -> String {
    stat_body
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next()) {
                (Some(name), Some(value)) if name == key => Some(value.to_string()),
                _ => None,
            }
        })
        .unwrap_or_else(|| "absent".to_string())
}

/// The effective limits of one cgroup directory, as one bracketed group.
fn cgroup_limits_group(dir: &Path) -> String {
    format!(
        "{}[memory.max={} memory.high={} memory.swap.max={}]",
        dir.display(),
        crate::memory_governor::cgroup_file_or_absent(dir, "memory.max"),
        crate::memory_governor::cgroup_file_or_absent(dir, "memory.high"),
        crate::memory_governor::cgroup_file_or_absent(dir, "memory.swap.max"),
    )
}

/// The diagnostic line for the cgroup at `dir`, under `mount_root`. Pure over the filesystem it is
/// pointed at, so the fixture test drives it over a temporary tree. Limits are printed for `dir`
/// and then for each ancestor up to and including the first one whose `memory.max` or `memory.high`
/// is not `max` (the first level that can bind or throttle), or the mount root.
fn cgroup_memory_diagnostic_line(dir: &Path, mount_root: &Path, phase: &str, pid: u32) -> String {
    let stat_body = crate::memory_governor::cgroup_file_or_absent(dir, "memory.stat");
    let stat = CGROUP_DIAGNOSTIC_STAT_KEYS
        .iter()
        .map(|key| format!("{key}={}", cgroup_stat_field(&stat_body, key)))
        .collect::<Vec<_>>()
        .join(" ");
    let events = crate::memory_governor::cgroup_file_or_absent(dir, "memory.events")
        .lines()
        .collect::<Vec<_>>()
        .join(" ");
    let mut limits = vec![cgroup_limits_group(dir)];
    let mut level = dir.parent();
    while let Some(ancestor) = level {
        if !ancestor.starts_with(mount_root) {
            break;
        }
        limits.push(cgroup_limits_group(ancestor));
        let binds = |file: &str| {
            let value = crate::memory_governor::cgroup_file_or_absent(ancestor, file);
            value != "max" && value != "absent"
        };
        if binds("memory.max") || binds("memory.high") || ancestor == mount_root {
            break;
        }
        level = ancestor.parent();
    }
    format!(
        "required-regen: cgroup-diag phase={phase} pid={pid} cgroup={} memory.current={} memory.peak={} memory.swap.current={} memory.events={events} memory.stat[{stat}] limits={}",
        dir.display(),
        crate::memory_governor::cgroup_file_or_absent(dir, "memory.current"),
        crate::memory_governor::cgroup_file_or_absent(dir, "memory.peak"),
        crate::memory_governor::cgroup_file_or_absent(dir, "memory.swap.current"),
        limits.join(" > "),
    )
}

/// This process's own `/proc/self/status` memory lines, in the same field shape the residency
/// gate reads.
///
/// THE TWO HALVES OF ONE QUESTION, PRINTED TOGETHER. `admit_generation_begin` reports the
/// ORCHESTRATOR (`VmRSS`); `cgroup_memory_diagnostic_line` reports the SLOT (`memory.current`).
/// They were printed by separate lines that never appeared at the same instant, so a reader
/// comparing a generation-begin `VmRSS` against the last slot reading two steps earlier saw a
/// contradiction that was only a gap in the log. Every diag line now carries both, and the
/// high-water mark beside the current value, so a peak the allocator did not return is
/// distinguishable from state the process still holds.
fn status_memory_fields_from(status: &str) -> String {
    let mut fields = Vec::new();
    for name in ["VmRSS", "VmHWM", "VmSize"] {
        // THE KEY AND THE VALUE ARE ONE WHITESPACE-FREE TOKEN APART: a status line is
        // "VmRSS:\t  216968 kB", so the key arrives WITH its colon and the number is the second
        // field. Stripping the bare name and taking the next token yields ":" and every number on
        // the line reads as `VmRSS=:kB` -- an instrumentation bug that reports itself as a real
        // reading, which is the one thing a measurement line must not do. `admit_generation_begin`'s
        // own probe reads the same line by field position; this parses it the same way.
        let key = format!("{name}:");
        let value = status
            .lines()
            .find(|line| line.split_whitespace().next() == Some(key.as_str()))
            .and_then(|line| line.split_whitespace().nth(1))
            .filter(|value| !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()));
        if let Some(value) = value {
            fields.push(format!("{name}={value}kB"));
        }
    }
    fields.join(" ")
}

fn self_status_memory_fields() -> String {
    status_memory_fields_from(&std::fs::read_to_string("/proc/self/status").unwrap_or_default())
}

/// OBSERVATION ONLY: prints the cgroup this process runs in, at a named point of the round, beside
/// this process's own memory lines. The parent and the rebuilt-seed child call it with their own
/// `phase` labels; a child spawned without a cgroup move shares its parent's cgroup, so the two
/// share a `cgroup=` value and the `pid=` field says which process read it. Nothing reads the line
/// back, no cap moves, nothing refuses.
///
/// IT NAMES THE BINARY THAT PRINTED IT, because a round that cannot say which executable ran it is
/// not evidence. A srv1 pair of rounds was run at two heads and refused identically to the unpatched
/// reproduction, with none of the phases that had just been added to the source firing -- including a
/// diag line sitting four lines above one that did fire, in the same straight-line function. That is
/// only possible if the image that ran was not the tree that was checked out, and nothing in the log
/// could say so: the `pid` and `cgroup` fields identify the process, never the file it was launched
/// from. So every diag line now carries the resolved `exe=` path and that file's modification time.
/// Render the executing image's identity for a diagnostic line.
///
/// The path is always printed; the modification time is printed only when it
/// was actually read. An unreadable timestamp must not appear as `0`, because
/// this field exists to tell two builds of the same-named file apart, and a
/// placeholder that looks like a reading is how a stale binary gets credited
/// as the one under test. Absent is rendered as absent, matching the
/// `/proc/self/status` parser, which omits an unreadable field rather than
/// printing it as zero.
fn self_exe_field() -> String {
    let mtime = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    let path = std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unresolved".to_string());
    self_exe_field_with(path, mtime)
}

fn self_exe_field_with(path: String, mtime: Option<u64>) -> String {
    match mtime {
        Some(secs) => format!("exe={path} exe_mtime={secs}"),
        None => format!("exe={path}"),
    }
}

fn report_cgroup_memory_diagnostic(phase: &str) {
    let process = self_status_memory_fields();
    let exe = self_exe_field();
    let line = match crate::memory_governor::leaf_cgroup_dir() {
        Some(dir) => format!(
            "{} {process} {exe}",
            cgroup_memory_diagnostic_line(
                &dir,
                Path::new("/sys/fs/cgroup"),
                phase,
                std::process::id()
            )
        ),
        None => format!(
            "required-regen: cgroup-diag phase={phase} pid={} cgroup=absent {process} {exe}",
            std::process::id()
        ),
    };
    eprintln!("{line}");
}

fn convergence_surface_roles(
    workspace: &Path,
    source_roots: &[String],
) -> Result<
    (
        HashMap<String, String>,
        BTreeSet<String>,
        BTreeSet<String>,
        BTreeSet<String>,
        BTreeSet<String>,
        HashMap<String, (String, String)>,
    ),
    String,
> {
    // THE MODULE NAMES, NOT THE EDGES. This used to call `regen_module_edges`, which builds the
    // whole-pool closure edge index (2065857 edges over 8098 modules on main 3637b4f651a; about
    // 100 s and 2.5 GB of transient heap on a release build) and returns that index's module keys
    // beside it -- and this function bound the edges and never read them. `regen_module_names` is
    // the identical `source_files` key set of the identical index, so the answer is unchanged by
    // construction and the undemanded edge build is gone (DESIGN section 2). This call runs in the
    // generation child right after `regen.digest`: the point where the srv3 children went silent
    // at memory.high.
    let modules = regen_module_names(workspace);
    // Ownership includes emitted-not-committed modules. Joining through the committed mirror
    // population would erase exactly the new-module case the convergence transaction must
    // install, so derive the projection from the complete module index instead.
    let basename_to_module = modules
        .iter()
        .map(|module| {
            (
                emit_path_basename(&rust_module_emit_path(module.clone())).to_string(),
                module.clone(),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut basename_to_module = basename_to_module;
    let (
        generation,
        bootstrap_sources,
        bootstrap_products,
        generated_product_owners,
        generated_product_roles,
    ) = regen_generation_role_population(source_roots, &modules)?;
    for (product, source_module) in generated_product_owners {
        match basename_to_module.insert(product.clone(), source_module.clone()) {
            Some(existing) if existing != source_module => {
                return Err(format!(
                    "SurfaceOwnershipAmbiguous: bootstrap product {product} is owned by both {existing} and {source_module}"
                ));
            }
            _ => {}
        }
    }
    let seed_binary_modules = super::emitted_closure_compile_host::binary_modules(
        &workspace.join("src/v1/stage0/src/lib.rs"),
    )?
    .into_iter()
    .collect::<BTreeSet<_>>();
    let partition_rows =
        crate::gunbc_stage0_stage0_crate_partition_generated::generated_partition_crate_rows();
    let seed_modules = assembled_seed_modules(seed_binary_modules, partition_rows.as_ref());
    let seed_embedded_basenames = seed_modules
        .into_iter()
        .map(|module| format!("{module}.rs"))
        .filter(|basename| !generated_product_roles.contains_key(basename))
        .collect::<BTreeSet<_>>();
    Ok((
        basename_to_module,
        generation,
        bootstrap_sources,
        bootstrap_products,
        seed_embedded_basenames,
        generated_product_roles,
    ))
}

fn assembled_seed_modules(
    mut seed_binary_modules: BTreeSet<String>,
    partition_rows: &crate::std_types::List<
        Rc<crate::gunbc_stage0_stage0_crate_partition_generated::GeneratedPartitionCrateRow>,
    >,
) -> BTreeSet<String> {
    use crate::gunbc_stage0_stage0_crate_partition_generated::GeneratedPartitionCrateKind::*;
    for row in partition_rows.iter() {
        match row.kind {
            GeneratedFoundationCrate | GeneratedLayeredCoreCrate => {
                seed_binary_modules.extend(row.modules.iter().cloned());
            }
            // Emit-core and facade crates are consumers/re-exports, not module-bearing assembly owners.
            GeneratedEmitCoreCrate | GeneratedFacadeCrate => {}
        }
    }
    seed_binary_modules
}

/// THE ORCHESTRATOR'S SIDE OF GENERATION-STATE LIFETIME ISOLATION, asked before every generation
/// begins. The rule is the model's (`v2.workflow.regen_convergence_transaction`
/// `regen_admit_generation_begin`); this reads the one input only the host has -- this process's
/// resident set -- and prints it at every transition, so the first rounds under the rule are also
/// the measurement an absolute ceiling would be set from. Returns the resident set read.
fn admit_generation_begin(
    model: &RegenConvergenceModel,
    ordinal: usize,
    resident_at_first: Option<u64>,
) -> Result<u64, String> {
    let now = crate::memory_governor::self_resident_bytes().ok_or_else(|| {
        format!(
            "OrchestratorResidentUnobservable: /proc/self/status VmRSS could not be read before \
             generation {ordinal}; the lifetime-isolation check has no input and refuses"
        )
    })?;
    let first = resident_at_first.unwrap_or(now);
    // THE INSTANT THE GATE DECIDES, in both currencies. This is the only place the orchestrator's
    // own `VmRSS` and the slot's `memory.current` are read at the same instant. The gate printed
    // only the process and the periodic diag only the slot, never at the instant that matters, so
    // a reader comparing a generation-begin `VmRSS` with the last slot reading (taken steps
    // earlier) could mistake a log gap for a contradiction -- which is how a correct refusal came
    // to be argued about instead of read.
    report_cgroup_memory_diagnostic(&format!(
        "parent.generation_begin_admission ordinal={ordinal} orchestrator_resident={now}"
    ));
    eprintln!(
        "required-regen: generation {ordinal} begins; orchestrator resident={now} \
         (at generation 0: {first})"
    );
    admit_generation_begin_readings(model, ordinal, first, now).map(|()| now)
}

/// The bridge itself, over supplied readings, so a test drives the REAL model through both its
/// admission and its refusal: the model's witnesses prove the rule, this proves the host calls it
/// with the shape it declares (`resident_at_first` / `resident_now` as `std.measure` `ByteSize`).
fn admit_generation_begin_readings(
    model: &RegenConvergenceModel,
    ordinal: usize,
    resident_at_first: u64,
    resident_now: u64,
) -> Result<(), String> {
    use crate::v1_interpreter::{self, Value};
    let ctx = model.context();
    let byte_size = |count: u64| Value::Record {
        type_name: ctx.sym("Measure"),
        fields: Rc::new(vec![(ctx.sym("count"), Value::Int(count as i64))]),
    };
    let admission = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_admit_generation_begin",
            &[
                (Some("ordinal".to_string()), Value::Int(ordinal as i64)),
                (
                    Some("resident_at_first".to_string()),
                    byte_size(resident_at_first),
                ),
                (Some("resident_now".to_string()), byte_size(resident_now)),
            ],
            false,
        )
    })
    .map_err(|e| format!("refusal: generation begin admission did not answer: {e}"))?;
    let label = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_generation_begin_admission_label",
            &[(Some("admission".to_string()), admission)],
            false,
        )
    })
    .map_err(|e| format!("refusal: generation begin admission projection failed: {e}"))?;
    match label {
        Value::Str(ref label) if label.as_ref() == "Admitted" => Ok(()),
        Value::Str(ref label) => Err(format!("generation begin {label}")),
        other => Err(format!(
            "refusal: generation begin admission projection returned {}",
            other.type_label_public()
        )),
    }
}

fn convergence_plan_from_model(
    model: &RegenConvergenceModel,
    ordinal: usize,
    generation_id: &str,
    candidate_tree_id: &str,
    candidate_tree_digest: &str,
    drifted: &[String],
    admitted_manifest: &HashMap<String, RegenCandidateManifestSurface>,
    stage0_src: &Path,
    basename_to_module: &HashMap<String, String>,
    generation_modules: &BTreeSet<String>,
    bootstrap_sources: &BTreeSet<String>,
    bootstrap_products: &BTreeSet<String>,
    seed_embedded_basenames: &BTreeSet<String>,
    generated_product_roles: &HashMap<String, (String, String)>,
    affected_bound: &RegenEmissionScope,
    seen_state_keys: &[String],
    seed_digest: &str,
    generation_bound: i64,
) -> Result<(RegenConvergenceStageKindReceipt, Vec<String>, String), String> {
    use crate::v1_interpreter::{self, str_value, Value};
    let ctx = model.context();
    let progress_args = vec![
        (
            Some("seen_state_keys".to_string()),
            Value::List(Rc::new(
                seen_state_keys
                    .iter()
                    .map(|key| str_value(key))
                    .collect::<Vec<_>>()
                    .into(),
            )),
        ),
        (Some("seed_digest".to_string()), str_value(seed_digest)),
        (
            Some("candidate_tree_digest".to_string()),
            str_value(candidate_tree_digest),
        ),
        (
            Some("ordinal".to_string()),
            Value::Int((ordinal - 1) as i64),
        ),
        (Some("bound".to_string()), Value::Int(generation_bound)),
    ];
    let progress = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_admit_generation_progress",
            &progress_args,
            false,
        )
    })
    .map_err(|e| format!("refusal: generation progress admission did not answer: {e}"))?;
    let progress_label = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_generation_progress_admission_label",
            &[(Some("admission".to_string()), progress)],
            false,
        )
    })
    .map_err(|e| format!("refusal: generation progress projection failed: {e}"))?;
    match progress_label {
        Value::Str(ref label) if label.as_ref() == "Admitted" => {}
        Value::Str(ref label) => return Err(format!("generation progress {label}")),
        other => {
            return Err(format!(
                "refusal: generation progress projection returned {}",
                other.type_label_public()
            ))
        }
    }
    let changed = drifted
        .iter()
        .map(|basename| {
            let manifest_surface = admitted_manifest.get(basename).ok_or_else(|| {
                format!("CandidateManifestPopulationMismatch: drifted {basename} is absent")
            })?;
            let module = basename_to_module.get(basename).cloned().ok_or_else(|| {
                format!(
                    "SurfaceOwnershipUnresolved: changed surface {basename} has no declaring \
                         module in the emitted-surface ownership authority"
                )
            })?;
            if module != manifest_surface.declaring_module {
                return Err(format!(
                    "CandidateManifestOwnershipMismatch: {basename} manifest={} authority={module}",
                    manifest_surface.declaring_module
                ));
            }
            let role = if bootstrap_products.contains(basename) {
                "BootstrapSourceMirror"
            } else if let Some((role, _)) = generated_product_roles.get(basename) {
                role.as_str()
            } else if generation_modules.contains(&module) {
                "GenerationInput"
            } else if bootstrap_sources.contains(&module) {
                "BootstrapSourceMirror"
            } else {
                "GenerationSubject"
            };
            let aggregate_membership = generated_product_roles
                .get(basename)
                .map(|(_, membership)| membership.as_str());
            let seed_embedded = seed_embedded_basenames.contains(basename);
            let role_value = Value::Variant {
                type_name: ctx.sym("RegenGenerationRole"),
                variant_name: ctx.sym(role),
                fields: Rc::new(vec![]),
            };
            // An aggregate product's membership is read from its declared row ALONE: the basename
            // heuristic below never mints it, and a product with no row is refused upstream.
            let membership_value = if let Some(membership) = aggregate_membership {
                Value::Variant {
                    type_name: ctx.sym("RegenSeedMembership"),
                    variant_name: ctx.sym(membership),
                    fields: Rc::new(vec![]),
                }
            } else if seed_embedded {
                Value::Variant {
                    type_name: ctx.sym("RegenSeedMembership"),
                    variant_name: ctx.sym("SeedEmbedded"),
                    fields: Rc::new(vec![]),
                }
            } else {
                Value::Variant {
                    type_name: ctx.sym("RegenSeedMembership"),
                    variant_name: ctx.sym("UnresolvedSeedMembership"),
                    fields: Rc::new(vec![(
                        ctx.sym("reason"),
                        str_value(format!(
                            "{basename} has no owner in generated_stage0_crate_partition"
                        )),
                    )]),
                }
            };
            let closure_disposition = v1_interpreter::with_active_context(&ctx, || {
                v1_interpreter::run_in_context_with_args(
                    &ctx,
                    "regen_dependency_closure_disposition",
                    &[
                        (Some("role".to_string()), role_value.clone()),
                        (Some("membership".to_string()), membership_value.clone()),
                    ],
                    false,
                )
            })
            .map_err(|e| format!("dependency closure authority did not answer: {e}"))?;
            let closure_ids = v1_interpreter::with_active_context(&ctx, || {
                v1_interpreter::run_in_context_with_args(
                    &ctx,
                    "regen_dependency_closure_ids",
                    &[(Some("disposition".to_string()), closure_disposition)],
                    false,
                )
            })
            .map_err(|e| format!("dependency closure projection did not answer: {e}"))?;
            let dependency_closure_id = match closure_ids {
                Value::List(ref ids) if ids.len() == 1 => match &ids[0] {
                    Value::Str(id) => id.to_string(),
                    other => {
                        return Err(format!(
                            "StageDependencyClosureIncomplete: model returned {} closure member",
                            other.type_label_public()
                        ))
                    }
                },
                Value::List(ref ids) => {
                    return Err(format!(
                        "StageDependencyClosureIncomplete: model returned {} closure identities for {basename}",
                        ids.len()
                    ))
                }
                other => {
                    return Err(format!(
                        "StageDependencyClosureIncomplete: model returned {} instead of List",
                        other.type_label_public()
                    ))
                }
            };
            let pre_stage_state = if stage0_src.join(basename).is_file() {
                Value::Variant {
                    type_name: ctx.sym("RegenPreStageState"),
                    variant_name: ctx.sym("PresentBeforeInstall"),
                    fields: Rc::new(vec![(
                        ctx.sym("digest"),
                        str_value(path_digest(&stage0_src.join(basename))?),
                    )]),
                }
            } else {
                Value::Variant {
                    type_name: ctx.sym("RegenPreStageState"),
                    variant_name: ctx.sym("AbsentBeforeInstall"),
                    fields: Rc::new(vec![]),
                }
            };
            let identity = Value::Record {
                type_name: ctx.sym("RegenSurfaceIdentity"),
                fields: Rc::new(vec![
                    (ctx.sym("declaring_module"), str_value(module)),
                    (ctx.sym("projected_path"), str_value(basename.clone())),
                ]),
            };
            let candidate = Value::Record {
                type_name: ctx.sym("RegenCandidateIdentity"),
                fields: Rc::new(vec![
                    (ctx.sym("producer_generation_id"), str_value(generation_id)),
                    (ctx.sym("candidate_tree_id"), str_value(candidate_tree_id)),
                    (
                        ctx.sym("candidate_tree_digest"),
                        str_value(candidate_tree_digest),
                    ),
                    (ctx.sym("surface"), identity.clone()),
                    (
                        ctx.sym("candidate_digest"),
                        str_value(&manifest_surface.content_digest),
                    ),
                ]),
            };
            Ok::<Value, String>(Value::Record {
                type_name: ctx.sym("RegenChangedSurface"),
                fields: Rc::new(vec![
                    (ctx.sym("identity"), identity),
                    (
                        ctx.sym("generation_role"),
                        role_value,
                    ),
                    (ctx.sym("seed_membership"), membership_value),
                    (
                        ctx.sym("dependency_closure_id"),
                        str_value(dependency_closure_id),
                    ),
                    (ctx.sym("pre_stage_state"), pre_stage_state),
                    (ctx.sym("candidate"), candidate),
                ]),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let strings = |items: &[String]| {
        Value::List(Rc::new(
            items
                .iter()
                .map(|item| str_value(item))
                .collect::<Vec<_>>()
                .into(),
        ))
    };
    let changed_value = Value::List(Rc::new(changed.into()));
    let bound_value = match affected_bound {
        RegenEmissionScope::WholePopulation => Value::Variant {
            type_name: ctx.sym("RegenAffectedBound"),
            variant_name: ctx.sym("CompleteCandidatePopulation"),
            fields: Rc::new(vec![]),
        },
        RegenEmissionScope::Affected { members } => Value::Variant {
            type_name: ctx.sym("RegenAffectedBound"),
            variant_name: ctx.sym("AffectedCandidatePopulation"),
            fields: Rc::new(vec![(ctx.sym("projected_paths"), strings(members))]),
        },
        RegenEmissionScope::Unlocatable { reason, .. } => Value::Variant {
            type_name: ctx.sym("RegenAffectedBound"),
            variant_name: ctx.sym("AffectedCandidatePopulationRefused"),
            fields: Rc::new(vec![(ctx.sym("reason"), str_value(reason))]),
        },
    };
    let bound_args = vec![
        (Some("bound".to_string()), bound_value),
        (Some("changed".to_string()), changed_value.clone()),
    ];
    let bound_admission = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_admit_affected_bound",
            &bound_args,
            false,
        )
    })
    .map_err(|e| format!("refusal: affected bound admission did not answer: {e}"))?;
    let bound_label = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_affected_bound_admission_label",
            &[(Some("admission".to_string()), bound_admission)],
            false,
        )
    })
    .map_err(|e| format!("refusal: affected bound admission projection failed: {e}"))?;
    match bound_label {
        Value::Str(ref label) if label.as_ref() == "Admitted" => {}
        Value::Str(ref label) => {
            return Err(format!(
                "affected-set bound and convergence stage population disagree: {label}"
            ))
        }
        other => {
            return Err(format!(
                "refusal: affected bound admission returned {}",
                other.type_label_public()
            ))
        }
    }
    let args = vec![
        (Some("ordinal".to_string()), Value::Int(ordinal as i64)),
        (
            Some("producer_generation_id".to_string()),
            str_value(generation_id),
        ),
        (
            Some("candidate_tree_id".to_string()),
            str_value(candidate_tree_id),
        ),
        (
            Some("candidate_tree_digest".to_string()),
            str_value(candidate_tree_digest),
        ),
        (Some("changed".to_string()), changed_value),
        (
            Some("build_target".to_string()),
            str_value("claim_executor"),
        ),
        (
            Some("build_invocation".to_string()),
            str_value("cargo build --release --bin claim_executor"),
        ),
    ];
    let outcome = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "regen_plan_next_stage", &args, false)
    })
    .map_err(|e| format!("refusal: convergence planner did not answer: {e}"))?;
    let outcome_arg = vec![(Some("outcome".to_string()), outcome)];
    let kind = match v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_stage_plan_kind_label",
            &outcome_arg,
            false,
        )
    })
    .map_err(|e| format!("refusal: convergence kind projection failed: {e}"))?
    {
        Value::Str(ref value) if value.as_ref() == "PromoteGenerationInputs" => {
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs
        }
        Value::Str(ref value) if value.as_ref() == "InstallSeedCompatibilityCut" => {
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut
        }
        Value::Str(ref value) if value.as_ref() == "PublishNonSeedOutputs" => {
            RegenConvergenceStageKindReceipt::PublishNonSeedOutputs
        }
        Value::Str(ref value) => {
            return Err(format!(
                "refusal: convergence kind projection returned unknown closed variant {value}"
            ))
        }
        other => {
            return Err(format!(
                "refusal: convergence kind projection returned {}",
                other.type_label_public()
            ))
        }
    };
    let paths = match v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_stage_plan_surface_paths",
            &outcome_arg,
            false,
        )
    })
    .map_err(|e| format!("refusal: convergence surface projection failed: {e}"))?
    {
        Value::List(ref values) => values
            .iter()
            .map(|value| match value {
                Value::Str(path) => Ok(path.to_string()),
                other => Err(format!(
                    "refusal: convergence surface projection member is {}",
                    other.type_label_public()
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        other => {
            return Err(format!(
                "refusal: convergence surface projection returned {}",
                other.type_label_public()
            ))
        }
    };
    let closure_id = match v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_stage_plan_dependency_closure_ids",
            &outcome_arg,
            false,
        )
    })
    .map_err(|e| format!("refusal: convergence closure projection failed: {e}"))?
    {
        Value::List(ref ids) if ids.len() == 1 => match &ids[0] {
            Value::Str(id) => id.to_string(),
            other => {
                return Err(format!(
                    "refusal: convergence closure projection member is {}",
                    other.type_label_public()
                ))
            }
        },
        Value::List(ref ids) => {
            return Err(format!(
                "refusal: planned convergence stage has {} admitted closure identities",
                ids.len()
            ))
        }
        other => {
            return Err(format!(
                "refusal: convergence closure projection returned {}",
                other.type_label_public()
            ))
        }
    };
    Ok((kind, paths, closure_id))
}

fn install_convergence_stage(
    model: &RegenConvergenceModel,
    source_roots: &[String],
    workspace: &Path,
    stage0_src: &Path,
    candidate_src: &Path,
    basenames: &[String],
    base_rewritten_mirrors: &[String],
    admitted_manifest: &HashMap<String, RegenCandidateManifestSurface>,
    basename_to_module: &HashMap<String, String>,
    ordinal: usize,
    kind: RegenConvergenceStageKindReceipt,
    seed_before: &str,
    generation_id: &str,
    candidate_tree_id: &str,
    candidate_tree_digest: &str,
    dependency_closure_id: &str,
    whole_compiler: bool,
    liveness: &SupervisedSubjectModel,
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<(RegenConvergenceStageReceipt, bool), String> {
    // The generated shell declares the emitted modules not owned by a partition.
    // Read the same declaration surface convergence_surface_roles uses for seed
    // membership; missing partition ownership alone must never imply shell ownership.
    let declared_shell_modules = host_shell_scope_modules(
        &stage0_src.join("lib.rs"),
        &stage0_src.join("../Cargo.toml"),
    )?;
    let changed_mirrors: Vec<String> = basenames
        .iter()
        .chain(base_rewritten_mirrors)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let actuation =
        partition_rebuild_actuation(source_roots, &changed_mirrors, &declared_shell_modules)?;
    let partition_move = if actuation.partition_generation_move {
        Some(admit_derived_partition_move(
            workspace,
            source_roots,
            liveness,
        )?)
    } else {
        None
    };
    let receipt = install_convergence_stage_with_backend(
        model,
        workspace,
        stage0_src,
        candidate_src,
        basenames,
        admitted_manifest,
        basename_to_module,
        ordinal,
        kind,
        seed_before,
        generation_id,
        candidate_tree_id,
        candidate_tree_digest,
        dependency_closure_id,
        &subject,
        |workspace| match &partition_move {
            Some(outcome) => partition_move_rebuild(workspace, &subject, outcome),
            // v1 maintenance purpose: realize the model's shared-input whole build so v2
            // regeneration can converge. Partition moves retain their separate admission above.
            None if whole_compiler || actuation.whole_compiler_required => {
                println!(
                    "partition-rebuild: selected whole-compiler build; boundary_files_changed={whole_compiler}; {}",
                    actuation.decision_line
                );
                seed_cargo_build(workspace, "round.rebuild_from_installed")
            }
            None => partitioned_rebuild_from_installed(workspace, &actuation),
        },
        || next_pass_executable_digest(workspace),
    )?;
    Ok((receipt, partition_move.is_some()))
}

#[allow(clippy::too_many_arguments)]
fn install_convergence_stage_with_backend<Build, SeedDigest>(
    model: &RegenConvergenceModel,
    workspace: &Path,
    stage0_src: &Path,
    candidate_src: &Path,
    basenames: &[String],
    admitted_manifest: &HashMap<String, RegenCandidateManifestSurface>,
    basename_to_module: &HashMap<String, String>,
    ordinal: usize,
    kind: RegenConvergenceStageKindReceipt,
    seed_before: &str,
    generation_id: &str,
    candidate_tree_id: &str,
    candidate_tree_digest: &str,
    dependency_closure_id: &str,
    checkpoint_subject: &RegenConvergenceCheckpointSubject,
    mut build_seed: Build,
    mut seed_digest: SeedDigest,
) -> Result<RegenConvergenceStageReceipt, String>
where
    Build: FnMut(&Path) -> Result<CargoBuildObservation, String>,
    SeedDigest: FnMut() -> Result<String, String>,
{
    // THE FIRST THING THE INSTALL BOUNDARY DOES, over the WHOLE roster, before a digest is read
    // or a journal is written. A per-file check inside the copy loop would refuse the fourth
    // entry with three already installed -- a partial mutation whose only remedy is the
    // checkpoint restore, i.e. a widen dressed as a refusal.
    for basename in basenames {
        admit_install_target(basename)?;
        admit_install_destination(stage0_src, basename)?;
    }
    let changed_before = git_changed_stage0_paths(workspace)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    // Re-admit every planned candidate immediately before the journal/mutation boundary. The
    // manifest was produced by the generation; hashes observed here cannot become their own
    // expectations.
    for basename in basenames {
        let expected = admitted_manifest.get(basename).ok_or_else(|| {
            format!("CandidateManifestPopulationMismatch: planned {basename} is absent")
        })?;
        let observed = path_digest(&candidate_src.join(basename))?;
        if observed != expected.content_digest {
            return Err(format!(
                "CandidateManifestSurfaceDigestMismatch: {basename} recorded {} observed {}",
                expected.content_digest, observed
            ));
        }
    }
    // Snapshot the complete authoritative generated population once. A build is not expected to
    // mutate source, but if it does the unplanned-path refusal must still be able to restore the
    // checkpoint it claims to preserve. Planned new paths are added beside the committed roster.
    let mut checkpoint_basenames = committed_generated_basenames(stage0_src)?;
    checkpoint_basenames.extend(basenames.iter().cloned());
    checkpoint_basenames.sort();
    checkpoint_basenames.dedup();
    journal_stage0_paths_for_subject(
        workspace,
        stage0_src,
        &checkpoint_basenames,
        checkpoint_subject,
    )?;
    // THE PRE-STAGE HALF OF THE INDEPENDENT EFFECT OBSERVATION, over the WHOLE journalled
    // roster rather than the planned subset. It is taken here, before the first copy, so the
    // delta below denominates in the population the checkpoint restores and not in the plan.
    let pre_stage_population =
        observe_generated_population_state(stage0_src, &checkpoint_basenames)?;
    let mut surfaces = Vec::new();
    for basename in basenames {
        let destination = stage0_src.join(basename);
        let candidate = candidate_src.join(basename);
        let pre_stage_state = if destination.is_file() {
            RegenPreStageState::PresentBeforeInstall {
                digest: path_digest(&destination)?,
            }
        } else {
            RegenPreStageState::AbsentBeforeInstall
        };
        let expected = admitted_manifest.get(basename).ok_or_else(|| {
            format!("CandidateManifestPopulationMismatch: installing {basename} is absent")
        })?;
        let candidate_digest = path_digest(&candidate)?;
        if candidate_digest != expected.content_digest {
            return Err(format!(
                "CandidateManifestSurfaceDigestMismatch immediately before copy: {basename} \
                 recorded {} observed {}",
                expected.content_digest, candidate_digest
            ));
        }
        fs::copy(&candidate, &destination).map_err(|e| {
            format!(
                "install {} -> {}: {e}",
                candidate.display(),
                destination.display()
            )
        })?;
        let installed = path_digest(&destination)?;
        if installed != candidate_digest {
            return Err(format!(
                "installed digest mismatch for {basename}: planned {candidate_digest}, observed {installed}"
            ));
        }
        surfaces.push(RegenConvergenceSurfaceReceipt {
            declaring_module: basename_to_module.get(basename).cloned().ok_or_else(|| {
                format!(
                    "SurfaceOwnershipUnresolved: planned surface {basename} has no declaring \
                     module before installation"
                )
            })?,
            projected_path: format!("src/v1/stage0/src/{basename}"),
            pre_stage_state,
            candidate_digest,
            installed_digest: installed,
            standing: RegenSurfaceExecutionStandingReceipt::Executed,
        });
    }
    let build = build_seed(workspace)?;
    let seed_after = seed_digest()?;
    if seed_after.is_empty() {
        return Err("stage output executable absent or unbound after successful build".to_string());
    }
    let allowed_after = changed_before
        .iter()
        .cloned()
        .chain(
            basenames
                .iter()
                .map(|basename| format!("src/v1/stage0/src/{basename}")),
        )
        .collect::<BTreeSet<_>>();
    let unplanned = git_changed_stage0_paths(workspace)?
        .into_iter()
        .filter(|path| !allowed_after.contains(path))
        .collect::<Vec<_>>();
    if !unplanned.is_empty() {
        return Err(format!("UnplannedPathMutated: {unplanned:?}"));
    }
    let mut observed_population = Vec::new();
    for surface in &surfaces {
        let observed_digest = path_digest(&workspace.join(&surface.projected_path))?;
        observed_population.push((surface.projected_path.clone(), observed_digest));
    }
    // The post-stage half. `ObservedEffectPopulation` is the only thing the model sees as
    // `executed`, and it is a digest delta over the roster — never a projection of `surfaces`.
    let post_stage_population =
        observe_generated_population_state(stage0_src, &checkpoint_basenames)?;
    let executed = ObservedEffectPopulation::from_stage_delta(
        &pre_stage_population,
        &post_stage_population,
        basename_to_module,
    )?;
    admit_stage_execution_from_model(model, &surfaces, &observed_population, &executed)?;
    for surface in &mut surfaces {
        surface.standing = RegenSurfaceExecutionStandingReceipt::TerminalPassed;
    }
    Ok(RegenConvergenceStageReceipt {
        receipt_id: format!("stage-{ordinal}"),
        ordinal,
        kind,
        input_seed_digest: seed_before.to_string(),
        input_candidate_tree_id: candidate_tree_id.to_string(),
        input_candidate_tree_digest: candidate_tree_digest.to_string(),
        producer_generation_id: generation_id.to_string(),
        surfaces,
        deferred_surfaces: Vec::new(),
        dependency_closure_id: dependency_closure_id.to_string(),
        build_target: "claim_executor".to_string(),
        build_invocation: "cargo build --release --bin claim_executor".to_string(),
        build_terminal: RegenConvergenceBuildTerminalReceipt::Passed,
        build_compiled_crates: build.compiled_crates,
        build_packages: build.compiled_packages,
        assembly_equivalence_authority: "v2.compiler.self_host.stage0_executable_assembly;v2.test.claim.self_host.stage0_executable_assembly".to_string(),
        output_seed_digest: seed_after,
        next_generation_receipt_id: format!("generation-{}", ordinal + 1),
    })
}

/// THE INDEPENDENTLY OBSERVED EFFECT POPULATION OF ONE STAGE — the `executed` side of the
/// planned/executed join, and a distinct TYPE precisely so it cannot be the planned side.
///
/// Before this existed, `admit_stage_execution_from_model` built one `Value::List` from the
/// stage plan and passed it as BOTH `planned` and `executed`, so the model's
/// `regen_identity_population_eq` conjunct was `x == x.clone()`: executed, reached, gated on,
/// and incapable of ever going red (DESIGN `executed_conjunct_discriminates_nothing`), while
/// `StagePlannedExecutedMismatch` stayed authorable in the `.dag` witness and unreachable from
/// the host. A permanently-green check standing where a wall is claimed is rung inflation.
///
/// The repair is a SECOND PRODUCER, not a second read of the first. This population is a
/// digest delta over the whole generated roster the checkpoint already journals — pre-stage
/// digests taken before the copy loop, post-stage digests taken after the build — so it is
/// derived from the filesystem and knows nothing about which surfaces were planned. The
/// denominator is the roster, not the plan.
///
/// IT IS A NEWTYPE AND THAT IS THE POINT. `admit_stage_execution_from_model` takes
/// `&[RegenConvergenceSurfaceReceipt]` for planned and `&ObservedEffectPopulation` for
/// executed; there is no conversion from the former to the latter, so a later refactor cannot
/// re-collapse the two sides into one value without deleting this type. A reviewer noticing is
/// not what keeps them apart.
///
/// THE OBSERVATION IS AN ENUMERATION, NOT A ROSTER LOOKUP, and the first draft got this wrong.
/// It named `checkpoint_basenames` as the population and declared that a file CREATED outside
/// that roster was covered by the `UnplannedPathMutated` git observation. That declaration was
/// false: `git_changed_stage0_paths` runs `git diff --name-only`, which reports tracked
/// modifications and says nothing about untracked files. A build creating a new file in the seed
/// source directory was therefore absent from the roster, absent from git, and absent from the
/// restoration journal — invisible to every producer at once. So both halves walk the DIRECTORY,
/// with the roster as a floor rather than as the population, and a path present after the stage
/// that was absent before is an effect like any other.
///
/// EVERY top-level regular file, not only the `.rs` generated ones. A build has no business
/// writing anything into `stage0_src`, and a file whose basename resolves to no declaring module
/// is refused as `SurfaceOwnershipUnresolved` — typed and located, naming the path. Files that
/// merely EXIST unchanged never reach that lookup, so the hand-maintained population costs
/// nothing here and a build MUTATING one refuses whether or not git tracks it.
struct ObservedEffectPopulation {
    identities: Vec<(String, String)>,
}

/// Absent is a state, not a missing digest: a surface that did not exist before the stage and
/// one whose bytes did not move are different observations, and collapsing them would make an
/// install onto a fresh path indistinguishable from a no-op.
#[derive(PartialEq, Eq)]
enum GeneratedSurfaceState {
    Absent,
    Present { digest: String },
}

fn observe_generated_population_state(
    stage0_src: &Path,
    roster: &[String],
) -> Result<BTreeMap<String, GeneratedSurfaceState>, String> {
    let mut basenames = roster.iter().cloned().collect::<BTreeSet<_>>();
    for entry in fs::read_dir(stage0_src)
        .map_err(|e| format!("enumerate stage0 src {}: {e}", stage0_src.display()))?
    {
        let entry = entry.map_err(|e| format!("read stage0 src entry: {e}"))?;
        if !entry.path().is_file() {
            continue;
        }
        let basename = entry
            .file_name()
            .to_str()
            .ok_or_else(|| {
                format!(
                    "InstallObservationPathNotUtf8: {} holds a non-UTF-8 entry name",
                    stage0_src.display()
                )
            })?
            .to_string();
        basenames.insert(basename);
    }
    let mut observed = BTreeMap::new();
    for basename in basenames {
        let path = stage0_src.join(&basename);
        let state = if path.is_file() {
            GeneratedSurfaceState::Present {
                digest: path_digest(&path)?,
            }
        } else {
            GeneratedSurfaceState::Absent
        };
        observed.insert(basename, state);
    }
    Ok(observed)
}

impl ObservedEffectPopulation {
    /// The ONLY constructor. It takes two observations of the roster and never the plan.
    fn from_stage_delta(
        before: &BTreeMap<String, GeneratedSurfaceState>,
        after: &BTreeMap<String, GeneratedSurfaceState>,
        basename_to_module: &HashMap<String, String>,
    ) -> Result<Self, String> {
        let mut identities = Vec::new();
        for (basename, after_state) in after {
            let before_state = before
                .get(basename)
                .unwrap_or(&GeneratedSurfaceState::Absent);
            if before_state == after_state {
                continue;
            }
            let declaring_module = basename_to_module.get(basename).ok_or_else(|| {
                format!(
                    "SurfaceOwnershipUnresolved: observed effect on {basename} has no declaring \
                     module"
                )
            })?;
            identities.push((
                declaring_module.clone(),
                format!("src/v1/stage0/src/{basename}"),
            ));
        }
        Ok(Self { identities })
    }
}

/// Build the model's `RegenSurfaceIdentity` list from basenames, refusing an unowned one rather
/// than fabricating a module. The population joins compare IDENTITIES, so a fabricated module
/// would make two different surfaces compare equal and quietly satisfy the join it exists to test.
fn convergence_identity_values(
    ctx: &crate::v1_interpreter::InterpContext,
    basenames: &[String],
    basename_to_module: &HashMap<String, String>,
) -> Result<crate::v1_interpreter::Value, String> {
    use crate::v1_interpreter::{str_value, Value};
    let mut rows = Vec::new();
    for basename in basenames {
        let module = basename_to_module.get(basename).ok_or_else(|| {
            format!("SurfaceOwnershipUnresolved: {basename} has no declaring module")
        })?;
        rows.push(Value::Record {
            type_name: ctx.sym("RegenSurfaceIdentity"),
            fields: Rc::new(vec![
                (ctx.sym("declaring_module"), str_value(module)),
                (ctx.sym("projected_path"), str_value(basename)),
            ]),
        });
    }
    Ok(Value::List(Rc::new(rows.into())))
}

/// Run one population admission and report its label AND detail. The label alone would name the
/// arm and drop the residues, which is the whole defect these joins exist to close.
fn population_admission_verdict(
    model: &RegenConvergenceModel,
    admission: crate::v1_interpreter::Value,
) -> Result<(), String> {
    use crate::v1_interpreter::{self, Value};
    let ctx = model.context();
    let for_detail = admission.clone();
    let label = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_population_admission_label",
            &[(Some("admission".to_string()), admission)],
            false,
        )
    })
    .map_err(|e| format!("refusal: population admission label failed: {e}"))?;
    match label {
        Value::Str(ref label) if label.as_ref() == "Admitted" => Ok(()),
        Value::Str(ref label) => {
            let detail = match v1_interpreter::with_active_context(&ctx, || {
                v1_interpreter::run_in_context_with_args(
                    &ctx,
                    "regen_population_admission_detail",
                    &[(Some("admission".to_string()), for_detail)],
                    false,
                )
            }) {
                Ok(Value::Str(ref detail)) => detail.to_string(),
                Ok(other) => format!("<detail returned {}>", other.type_label_public()),
                Err(e) => format!("<detail refused: {e}>"),
            };
            Err(format!("{label}: {detail}"))
        }
        other => Err(format!(
            "refusal: population admission label returned {}",
            other.type_label_public()
        )),
    }
}

/// JOIN 1 -- the producer's changed population equals planned UNION deferred, by identity.
///
/// The host computes `deferred` as `drifted` minus the install set, so today the two sides agree
/// by construction. That is exactly why the join is worth executing rather than assuming: the
/// construction is one edit away from being wrong, and the previous form of this boundary was
/// self-consistent for the same reason -- both sides inherited one narrowing and the receipt
/// proved nothing about the population that entered.
fn admit_install_boundary_population_from_model(
    model: &RegenConvergenceModel,
    admitted: &[String],
    planned: &[String],
    deferred: &[String],
    basename_to_module: &HashMap<String, String>,
) -> Result<(), String> {
    use crate::v1_interpreter::Value;
    let ctx = model.context();
    let observation = Value::Record {
        type_name: ctx.sym("RegenInstallBoundaryObservation"),
        fields: Rc::new(vec![
            (
                ctx.sym("admitted"),
                convergence_identity_values(&ctx, admitted, basename_to_module)?,
            ),
            (
                ctx.sym("planned"),
                convergence_identity_values(&ctx, planned, basename_to_module)?,
            ),
            (
                ctx.sym("deferred"),
                convergence_identity_values(&ctx, deferred, basename_to_module)?,
            ),
        ]),
    };
    let admission = crate::v1_interpreter::with_active_context(&ctx, || {
        crate::v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_admit_install_boundary_population",
            &[(Some("observation".to_string()), observation)],
            false,
        )
    })
    .map_err(|e| format!("refusal: install boundary population admission did not answer: {e}"))?;
    population_admission_verdict(model, admission)
}

/// One surface's terminal disposition, accumulated across the transaction's generations.
///
/// `Deferred` is deliberately representable even though a converged transaction never ends in one:
/// its presence is what lets the lineage join REFUSE an unfinished transaction rather than report a
/// fixed point it did not reach. Dropping the arm because the happy path cannot produce it would
/// make the terminality conjunct true by construction.
enum ConvergenceDisposition {
    Applied { installed_digest: String },
    Superseded { by_generation_id: String },
    Deferred,
}

/// JOIN 3 -- the population admitted across the transaction equals the terminal lineage, both
/// directions, and every lineage row reached a terminal disposition.
///
/// THE LINEAGE IS BUILT FROM THE STAGE RECEIPTS, NEVER FROM THE ADMITTED LIST. Deriving it from
/// the admitted population would make `admitted_without_lineage` empty by construction and the
/// join would restate its own input -- the same `x == x.clone()` shape this lane was opened to
/// remove, one level up. A surface the stage loop drops appears in no receipt, so it is absent
/// from the lineage and the join names it.
fn admit_transaction_lineage_from_model(
    model: &RegenConvergenceModel,
    admitted: &BTreeSet<String>,
    lineage: &BTreeMap<String, ConvergenceDisposition>,
    basename_to_module: &HashMap<String, String>,
) -> Result<(), String> {
    use crate::v1_interpreter::{str_value, Value};
    let ctx = model.context();
    let admitted_rows = admitted.iter().cloned().collect::<Vec<_>>();
    let mut rows = Vec::new();
    for (basename, disposition) in lineage {
        let module = basename_to_module.get(basename).ok_or_else(|| {
            format!("SurfaceOwnershipUnresolved: lineage row {basename} has no declaring module")
        })?;
        let disposition_value = match disposition {
            ConvergenceDisposition::Applied { installed_digest } => Value::Variant {
                type_name: ctx.sym("RegenSurfaceDisposition"),
                variant_name: ctx.sym("SurfaceApplied"),
                fields: Rc::new(vec![(
                    ctx.sym("installed_digest"),
                    str_value(installed_digest),
                )]),
            },
            ConvergenceDisposition::Superseded { by_generation_id } => Value::Variant {
                type_name: ctx.sym("RegenSurfaceDisposition"),
                variant_name: ctx.sym("SurfaceSuperseded"),
                fields: Rc::new(vec![(
                    ctx.sym("by_generation_id"),
                    str_value(by_generation_id),
                )]),
            },
            ConvergenceDisposition::Deferred => Value::Variant {
                type_name: ctx.sym("RegenSurfaceDisposition"),
                variant_name: ctx.sym("SurfaceDeferred"),
                fields: Rc::new(vec![(
                    ctx.sym("reason"),
                    Value::Variant {
                        type_name: ctx.sym("RegenDeferredReason"),
                        variant_name: ctx.sym("AwaitingBuildableSeedGeneration"),
                        fields: Rc::new(vec![]),
                    },
                )]),
            },
        };
        rows.push(Value::Record {
            type_name: ctx.sym("RegenSurfaceLineage"),
            fields: Rc::new(vec![
                (
                    ctx.sym("identity"),
                    Value::Record {
                        type_name: ctx.sym("RegenSurfaceIdentity"),
                        fields: Rc::new(vec![
                            (ctx.sym("declaring_module"), str_value(module)),
                            (ctx.sym("projected_path"), str_value(basename)),
                        ]),
                    },
                ),
                (ctx.sym("disposition"), disposition_value),
            ]),
        });
    }
    let admitted_values = convergence_identity_values(&ctx, &admitted_rows, basename_to_module)?;
    let lineage_values = Value::List(Rc::new(rows.into()));
    let admission = crate::v1_interpreter::with_active_context(&ctx, || {
        crate::v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_admit_transaction_lineage",
            &[
                (Some("initial_admitted".to_string()), admitted_values),
                (Some("lineage".to_string()), lineage_values),
            ],
            false,
        )
    })
    .map_err(|e| format!("refusal: transaction lineage admission did not answer: {e}"))?;
    population_admission_verdict(model, admission)
}

fn admit_stage_execution_from_model(
    model: &RegenConvergenceModel,
    planned: &[RegenConvergenceSurfaceReceipt],
    observed: &[(String, String)],
    executed: &ObservedEffectPopulation,
) -> Result<(), String> {
    use crate::v1_interpreter::{self, str_value, Value};
    let ctx = model.context();
    let identities = planned
        .iter()
        .map(|surface| Value::Record {
            type_name: ctx.sym("RegenSurfaceIdentity"),
            fields: Rc::new(vec![
                (
                    ctx.sym("declaring_module"),
                    str_value(&surface.declaring_module),
                ),
                (
                    ctx.sym("projected_path"),
                    str_value(&surface.projected_path),
                ),
            ]),
        })
        .collect::<Vec<_>>();
    let expected_digest = bytes_digest(
        &serde_json::to_vec(
            &planned
                .iter()
                .map(|surface| (&surface.projected_path, &surface.candidate_digest))
                .collect::<Vec<_>>(),
        )
        .map_err(|e| format!("encode planned stage population: {e}"))?,
    );
    let observed_digest = bytes_digest(
        &serde_json::to_vec(observed)
            .map_err(|e| format!("encode observed stage population: {e}"))?,
    );
    let executed_identities = executed
        .identities
        .iter()
        .map(|(declaring_module, projected_path)| Value::Record {
            type_name: ctx.sym("RegenSurfaceIdentity"),
            fields: Rc::new(vec![
                (ctx.sym("declaring_module"), str_value(declaring_module)),
                (ctx.sym("projected_path"), str_value(projected_path)),
            ]),
        })
        .collect::<Vec<_>>();
    let observation = Value::Record {
        type_name: ctx.sym("RegenStageExecutionObservation"),
        fields: Rc::new(vec![
            (ctx.sym("planned"), Value::List(Rc::new(identities.into()))),
            (
                ctx.sym("executed"),
                Value::List(Rc::new(executed_identities.into())),
            ),
            (
                ctx.sym("expected_candidate_digest"),
                str_value(expected_digest),
            ),
            (
                ctx.sym("observed_candidate_digest"),
                str_value(observed_digest),
            ),
            (
                ctx.sym("build_terminal"),
                Value::Variant {
                    type_name: ctx.sym("RegenStageBuildTerminal"),
                    variant_name: ctx.sym("RegenStageBuildPassed"),
                    fields: Rc::new(vec![]),
                },
            ),
        ]),
    };
    let admission = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_admit_stage_execution",
            &[(Some("observation".to_string()), observation)],
            false,
        )
    })
    .map_err(|e| format!("refusal: stage execution admission did not answer: {e}"))?;
    let admission_for_detail = admission.clone();
    let label = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_stage_execution_admission_label",
            &[(Some("admission".to_string()), admission)],
            false,
        )
    })
    .map_err(|e| format!("refusal: stage execution admission label failed: {e}"))?;
    match label {
        Value::Str(ref label) if label.as_ref() == "Admitted" => Ok(()),
        Value::Str(ref label) => {
            // The label NAMES the arm; the detail LOCATES it. Rendering only the name collapsed
            // every unlabelled arm to one word and discarded the populations that caused a
            // population verdict -- which is what a reader needs and what §5 asks a typed
            // diagnostic to carry. A detail that itself refuses is reported rather than dropped,
            // because a silent renderer here would reintroduce exactly the silence it repairs.
            let detail = match v1_interpreter::with_active_context(&ctx, || {
                v1_interpreter::run_in_context_with_args(
                    &ctx,
                    "regen_stage_execution_admission_detail",
                    &[(Some("admission".to_string()), admission_for_detail)],
                    false,
                )
            }) {
                Ok(Value::Str(ref detail)) => detail.to_string(),
                Ok(other) => format!("<detail returned {}>", other.type_label_public()),
                Err(e) => format!("<detail refused: {e}>"),
            };
            Err(format!("stage execution admission {label}: {detail}"))
        }
        other => Err(format!(
            "refusal: stage execution admission label returned {}",
            other.type_label_public()
        )),
    }
}

/// `affected_scope` consumes the affected-set bound for this round: the selection is derived
/// from the SAME edited population `--regen-affected-set` reports (the floor's own diff range),
/// so the report and the round read one edit rather than two. `false` is the whole-population
/// round, byte for byte what it was.
pub fn run_regen_round_cost(
    candidate_dir_rel: &str,
    receipt_rel: &str,
    source_roots: &[String],
    affected_scope: bool,
    base_mirror: bool,
) -> Result<RegenRoundCostOutcome, String> {
    let workspace = workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");
    let candidate_src = workspace.join(candidate_dir_rel).join("src");
    let host = host_name();
    let exe_before = running_image_digest()?;

    // READ ONCE, BEFORE ANYTHING TOUCHES THE TREE: this value decides whether this image owns the
    // round-1 seeding at all (a carried tree means it does not), and it is the same value the
    // handoff decision compares below, so the two questions cannot be answered from different
    // observations.
    let handoff_in = SeedHandoff::from_env().map_err(|cause| cause.to_string())?;

    // THE ROUND'S CHECKPOINT SUBJECT: AT MOST ONE PRODUCER PER ROUND, INVOKED AT THE FIRST DEMAND,
    // THEN CARRIED (`establish_round_subject`). The demands are a journal to restore at startup, the
    // base-mirror bootstrap's journal, and the first install; a round that never journals never
    // produces it, and a journal never exists without it (`restore_round_journal`). Every journal
    // write and restore below is about this one value; the stage0 input closure it digests does not change
    // during a round (installs and the base snapshot write `.rs`, never `.dag`), and every
    // generation receipt checks that (`admit_generation_authority`). It used to be recomputed at
    // each restore, in each image, and at each install stage, and each recompute built the
    // whole-tree index in THIS process (run 41899288377: 7.57 GB resident at generation 0). Now the
    // image that owns the round asks a child for it, so the index lives and dies there, and the
    // re-exec'd image takes the carried value. The liveness model supervises that child, so it is
    // loaded first.
    let liveness_model = SupervisedSubjectModel::load(source_roots)?;
    let mut round_subject: Option<RegenConvergenceCheckpointSubject> = handoff_in
        .as_ref()
        .and_then(|carried| carried.round_subject.clone());

    // THE HANDOFF CHILD IS NOT A SECOND OWNER OF THE TREE (manager ruling 2026-10-29, #15339).
    // The parent seeded the base snapshot, journaled the head's bytes, and compiled that tree; the
    // child was exec'd to report on THAT build. Restoring the journal and seeding again is redundant
    // work (§2), and it is what made the two images' populations disagree: the child restored every
    // journaled path to the head's bytes and then re-seeded under ITS OWN rule, so a round whose
    // parent and child disagree about the population leaves the child compiling a tree its parent
    // never compiled, cargo rebuilds, the relink trips `ExecutableReplacedAfterHandoff`, and the
    // refusal is correct — the measurement would otherwise describe bytes the emitting image never
    // built. The child trusts the carried tree instead. `SourceChangedAcrossHandoff` is the guard
    // that makes trusting safe: it compares this image's observation against the one the parent
    // carried, so a child that found a different tree refuses rather than reporting on it.
    if handoff_in.is_none() || !base_mirror {
        // Recover an interrupted transaction before attempting to build or observe a new subject.
        // THE BASE-MIRROR CHILD SKIPS THIS DELIBERATELY (manager ruling 2026-10-29, #15339): its
        // parent journalled the head's bytes and then built the seed from the seeded tree, so a
        // restore here would replace the very bytes the child was exec'd to report on, and the
        // build below would compile something its parent never compiled. Every other round keeps
        // the behaviour main has always had, including the recovery of an interrupted round.
        if regen_convergence_journal_exists(&workspace) {
            let subject = establish_round_subject(&mut round_subject, None, || {
                checkpoint_subject_child(&workspace, &liveness_model)
            })?;
            restore_regen_convergence_journal_for_subject(&workspace, subject)?;
        }
    }

    // THE SUBJECT IS READ AFTER THE RESTORE, which is what that comment has always claimed and
    // what the round-1 bootstrap makes load-bearing rather than incidental. Read before the
    // restore, the first image would see the committed head and the second would see whatever the
    // previous image had written, and the round would refuse for a defect in the observation order
    // rather than in the source.
    //
    // `tree` and `tree_dirty` describe the COMMITTED state and stay here: a heal round writes base's
    // bytes into the worktree, and the round is still a round on one commit with the pull request's
    // own diff, not on a dirty tree of someone else's bytes.
    let tree = git_head_sha(&workspace)?;
    let tree_dirty = git_tree_dirty(&workspace)?;

    // THE ROUND-1 BOOTSTRAP (DESIGN §5, RFM #14963): when the caller is the heal round, the seed
    // build below compiles the BASE ref's committed mirror instead of the head's. Unconditional
    // for a heal round — never a step taken after a failure — and scoped to round 1: everything
    // after the first seed build runs on bytes this PR's own `.dag` regenerates, and the journal
    // restore returns the head's bytes before any install or publication.
    //
    // ONLY THE IMAGE THAT OWNS THE ROUND SEEDS (manager ruling 2026-10-29, #15339). The re-exec'd
    // child reports on a build that consumed THIS tree; seeding again would restore the head's
    // bytes and rewrite a population under the child's own rule, and when parent and child
    // disagree about that population the child compiles bytes its parent never compiled, cargo
    // rebuilds, and the relink trips `ExecutableReplacedAfterHandoff` — a refusal that is correct,
    // because the measurement would otherwise describe a tree the emitting image never built.
    // THE ROUND'S SUBJECT, READ ONCE. The bootstrap journals the head's bytes under it and the
    // teardown returns them under it; `restore_regen_convergence_journal_for_subject` REFUSES on a
    // subject mismatch, so a second read at teardown time would turn any mid-round drift into a new
    // refusal instead of a rollback. One read, both ends.
    // That one read is `round_subject`, above: one producer per round, carried across the re-exec.
    let base_bootstrap = if base_mirror && handoff_in.is_none() {
        v1_rt::trace_mark("round.base_mirror_bootstrap.begin".to_string());
        // The ref is read HERE, from the environment, and passed as a value: a branch name is
        // pull-request-controlled text and the workflow admit gate refuses any argv word
        // containing `${{`, so the ref never reaches argv. The subject is the one the install path
        // reads later, so the journal this bootstrap creates is the journal the round restores.
        let base_commit_env = std::env::var(REGEN_ROUND_COST_BASE_MIRROR_ENV).ok();
        let subject = establish_round_subject(&mut round_subject, None, || {
            checkpoint_subject_child(&workspace, &liveness_model)
        })?;
        let restored =
            bootstrap_round_one_from_base_mirror(&workspace, base_commit_env.as_deref(), subject)?;
        v1_rt::trace_mark("round.base_mirror_bootstrap.done".to_string());
        // THE RECEIPT NAMES ITS SOURCE (sleek-owl-397, RFM #14963): the commit, the population size,
        // and the whole-crate scope, so a reader can tell WHICH bytes the round compiled and can
        // distinguish a base byte from a head byte without re-running anything.
        println!(
            "regen-round-cost: round 1 builds the whole {STAGE0_CRATE_REL} crate from the base \
             mirror commit={} paths={} paths_rewritten={} head_added_paths_kept={} \
             head_deleted_paths={}",
            restored.base_commit,
            restored.paths_restored,
            restored.paths_rewritten,
            restored.paths_absent_at_base,
            restored.head_deleted_paths
        );
        Some(restored)
    } else {
        None
    };

    // THE SOURCE THE ROUND BUILT IS THE SEEDED TREE, NOT THE COMMITTED ONE, so the identity is read
    // AFTER the bootstrap (manager ruling 2026-10-29, #15339). Read before it, the parent would
    // carry the identity of a clean committed tree while the tree the seed build actually consumed
    // holds base's bytes, and the child — which restores and seeds nothing, because it trusts what
    // it was exec'd into — would observe the seeded tree and refuse with
    // `SourceChangedAcrossHandoff`. Reading here makes the carried value name the bytes that were
    // compiled, which is the only thing the guard can usefully compare: both images of one round
    // observe the same bytes, and any third path that moves the tree between them still refuses.
    let source_identity = source_identity(&workspace, &tree)?;

    v1_rt::trace_ledger_arm();
    let rustfmt_spawns_before = rustfmt_spawn_count();
    let seed_cpu_before = v1_rt::trace_process_tree_cpu_ms();
    let seed_wall_before = std::time::Instant::now();
    let seed_build = seed_cargo_build(&workspace, "round.seed_build")?;
    let seed_build_wall_ms = seed_wall_before.elapsed().as_millis() as u64;
    let seed_build_cpu_ms = match (seed_cpu_before, v1_rt::trace_process_tree_cpu_ms()) {
        (Some(b), Some(a)) => Some(a.saturating_sub(b)),
        _ => None,
    };
    let exe_after = current_exe_digest()?;
    let seed_build_cost = SeedBuildCost {
        compiled_crates: seed_build.compiled_crates,
        wall_ms: seed_build_wall_ms,
        cpu_ms: seed_build_cpu_ms,
    };
    match decide_seed_handoff(
        handoff_in.as_ref(),
        &exe_before,
        &exe_after,
        &source_identity,
        &seed_build_cost,
        round_subject.as_ref(),
    )
    .map_err(|cause| cause.to_string())?
    {
        SeedHandoffDecision::Proceed => {}
        SeedHandoffDecision::ReExec(carry) => {
            // ROOT CAUSE: the build's output path IS this process's image path, so a build that
            // relinks replaces the executable that is running it. The emit below runs in this
            // process, so it must be the built seed: hand the observation to the built binary,
            // which composes it into the receipt exactly once.
            use std::os::fd::AsRawFd;
            use std::os::unix::process::CommandExt;
            let on_disk = current_exe_on_disk()?;
            // `carry.built_exe` is the build-output digest; the descriptor we exec must match
            // it before the exec, and the carry is never re-identified from the open.
            let (target, target_digest) = open_exec_target(&on_disk)?;
            let carry =
                admit_exec_target(carry, &target_digest).map_err(|cause| cause.to_string())?;
            let err = Command::new(format!("/proc/self/fd/{}", target.as_raw_fd()))
                .arg0(&on_disk)
                .args(std::env::args_os().skip(1))
                .env(REGEN_ROUND_COST_REEXEC_ENV, carry.encode())
                .exec();
            return Err(SeedHandoffRefusal::ReExecFailed {
                path: on_disk.display().to_string(),
                cause: err.to_string(),
            }
            .to_string());
        }
    }
    // THE IMAGE THAT PROCEEDS ENDS THE BASE SNAPSHOT before anything is observed, published, or
    // regenerated (review 87490 on #15339). Nothing downstream may see a base byte: the candidate
    // population is read from the worktree right after this, the publisher takes whatever differs
    // from the head, and `gunbc.heal_candidate_stage0` `mirror_paths_of` filters only by directory
    // depth and suffix — so a hand-maintained file left at base's bytes would be published as one of
    // the pull request's own edits, i.e. a bot commit REVERTING the change under review. This PR is
    // the specimen: it edits the very file that implements the bootstrap.
    //
    // THE ROLLBACK IS THE AUTHORITY, not a second copy of it (`restore_regen_convergence_journal`
    // is the one reader of `journal.json`, §3): it returns every journalled path to the byte
    // recorded before the round and deletes the journal. On the pre-fix success path that call never
    // happened — the journal was simply deleted at the end of the round, so base bytes survived into
    // the candidate. Generated mirrors do not depend on this: the generator rewrites them anyway.
    // Hand-maintained files, the crate manifest and the build script do, and the round-1 population
    // now includes all three, which is why the whole-crate scope and this call are one change.
    //
    // It runs HERE, in the image that Proceeds, rather than before the handoff exec: the re-exec'd
    // image is the seed this build produced FROM the base tree, so reverting before the exec would
    // make that image's build differ from its parent's and relink — the refusal
    // `ExecutableReplacedAfterHandoff` exists to raise. The base tree must survive the exec; it must
    // not survive the round. The child reaches here with the parent's journal still on disk (the
    // child restores nothing at startup, and only the end of a round deletes it), so one call
    // unwinds what the parent seeded.
    let base_rewritten_mirrors: Vec<String> = if base_mirror {
        v1_rt::trace_mark("round.base_snapshot_revert.begin".to_string());
        let revert = restore_round_journal(&workspace, round_subject.as_ref())?;
        let (rewritten, removed) = (revert.rewritten, revert.removed);
        v1_rt::trace_mark("round.base_snapshot_revert.done".to_string());
        // THE RECEIPT NAMES WHAT IT PUT BACK, so a reader can tell a round that returned the head's
        // bytes from one that quietly kept compiling base's into the candidate. The count is the
        // paths the restore actually rewrote: a journalled path the seed never moved already holds
        // the head's bytes, and rewriting it would be mtime churn cargo reads as a change.
        println!(
            "regen-round-cost: round 1 ended the base snapshot; {} journalled paths were rewritten \
             to the head's own bytes, {} were removed because the head's commit does not track them \
             (the rest already held them)",
            rewritten.len(),
            removed.len()
        );
        // THE CHANGED SET THE REBUILD DECISION READS INCLUDES THESE. Round 1 compiled base's
        // bytes; this restore just moved the head's differing mirrors under the built artifacts, so
        // cargo sees exactly these paths as changed in round 2. They are taken from the restore's own
        // return, never inferred from mtimes (RFM
        // `heal_partition_guard_infers_changed_mirrors_from_mtimes_not_rewritten_paths`).
        let src_prefix = format!("{STAGE0_CRATE_REL}/src/");
        rewritten
            .iter()
            .filter(|path| path.starts_with(&src_prefix) && path.ends_with(".rs"))
            .map(|path| emit_path_basename(path).to_string())
            .collect()
    } else {
        Vec::new()
    };

    // Pre-handoff cost joins this image's no-op rebuild: summed once, here, before the
    // ledger rows and the crate count reach the receipt.
    let carried_cost = handoff_in.as_ref().map(|h| h.cost.clone());

    // The surface roles are NOT computed here. They are corpus-derived generation work
    // (`ConvergenceSurfaceRoles`), carried by generation 0's receipt and bound once it is read.
    let scope = if affected_scope {
        regen_emission_scope_for_diff(&workspace, source_roots)?
    } else {
        RegenEmissionScope::WholePopulation
    };
    // GENERATION-STATE LIFETIME ISOLATION (`v2.workflow.regen_convergence_transaction`
    // `regen_admit_generation_begin`): at most one generation's state is resident at a time. Every
    // generation, THIS FIRST ONE INCLUDED, runs as its own rebuilt-seed child, and process exit is
    // the destructor for what it loaded. Generation 0 used to run in this process
    // (`run_required_regen_scoped`), so its corpus had the round's lifetime while every later
    // generation's had its own, and the two composed: 15.6 GB still charged to the slot when
    // generation 1 began (srv4), which a declared 22/21/0 slot cannot hold beside a second
    // generation. The orchestration state -- the surface roles above and the convergence model
    // below -- is loaded BEFORE generation 0, so the orchestrator's footprint at generation 0 is
    // the baseline every later transition is held to.
    let convergence_model = RegenConvergenceModel::load(source_roots)?;
    let initial_seed_digest = current_exe_digest()?;
    let orchestrator_resident_at_first = admit_generation_begin(&convergence_model, 0, None)?;
    v1_rt::trace_mark("round.built_seed_regen.begin".to_string());
    report_cgroup_memory_diagnostic("parent.begin generation=0");
    let first = run_built_seed_regen(
        &workspace,
        candidate_dir_rel,
        receipt_rel,
        source_roots,
        affected_scope,
        &initial_seed_digest,
        0,
        &liveness_model,
        round_subject.as_ref(),
    );
    report_cgroup_memory_diagnostic("parent.end generation=0");
    v1_rt::trace_mark("round.built_seed_regen.done".to_string());
    let first = first?;
    let (
        basename_to_module,
        generation_modules,
        bootstrap_sources,
        bootstrap_products,
        seed_embedded_basenames,
        generated_product_roles,
    ) = first.surface_roles.clone().into_planning()?;
    let mut candidate_digest = first.manifest.candidate_tree_digest.clone();
    let starting_surface_digest = first.committed_digest;
    let mut candidate_tree_id = first.manifest.candidate_tree_id.clone();
    let mut drifted = first.drifted;
    let mut authority_digest = first.authority_digest;
    let mut candidate_manifest = first.manifest;
    let mut current_seed_digest = initial_seed_digest.clone();
    let mut stages = Vec::new();
    let mut seen_states = BTreeSet::new();
    let mut generation_ordinal = 0usize;
    let round_failures: Vec<String> = Vec::new();
    // THE DENOMINATOR, ACCUMULATED RATHER THAN SAMPLED. Every generation's drifted population
    // joins this set, because installing generation inputs can make the NEXT generation emit a
    // surface the first one did not -- so the transaction's admitted population is the union, and
    // taking only the first generation's would refuse a correct run.
    let mut transaction_admitted: BTreeSet<String> = BTreeSet::new();
    let mut transaction_lineage: BTreeMap<String, ConvergenceDisposition> = BTreeMap::new();

    let generation_bound = regen_convergence_generation_bound(source_roots)?;
    let mut partition_moved = false;
    let mut round_records: Vec<String> = Vec::new();
    loop {
        if drifted.is_empty() {
            if !partition_moved {
                break;
            }
            let boundary_drift = match built_seed_boundary_drift(&workspace) {
                Ok(paths) => paths,
                Err(failure) => {
                    restore_round_journal(&workspace, round_subject.as_ref())?;
                    return Err(format!("{failure}; checkpoint restored"));
                }
            };
            if boundary_drift.is_empty() {
                break;
            }
            round_records.push(format!(
                "generation {generation_ordinal}: mirrors converged, boundary drift under the \
                 rebuilt producer {boundary_drift:?}"
            ));
            if regen_convergence_bound_exceeded(source_roots, generation_ordinal)? {
                restore_round_journal(&workspace, round_subject.as_ref())?;
                return Err(format!(
                    "RegenConvergenceBoundExceeded: boundary files did not converge within the \
                     declared bound (regen_convergence_max_generations) at generation \
                     {generation_ordinal}; checkpoint restored; nothing is published -- {}",
                    round_records.join("; ")
                ));
            }
            let rebuilt =
                install_built_seed_boundary(&workspace, &boundary_drift, round_subject.as_ref())
                    .and_then(|()| seed_cargo_build(&workspace, "round.rebuild_from_boundary"))
                    .and_then(|_| next_pass_executable_digest(&workspace));
            match rebuilt {
                Ok(digest) => current_seed_digest = digest,
                Err(failure) => {
                    restore_round_journal(&workspace, round_subject.as_ref())?;
                    return Err(format!(
                        "boundary stage refused; checkpoint restored: {failure}"
                    ));
                }
            }
            generation_ordinal += 1;
            if let Err(failure) = admit_generation_begin(
                &convergence_model,
                generation_ordinal,
                Some(orchestrator_resident_at_first),
            ) {
                restore_round_journal(&workspace, round_subject.as_ref())?;
                return Err(format!(
                    "next generation refused; checkpoint restored: {failure}"
                ));
            }
            match run_built_seed_regen(
                &workspace,
                candidate_dir_rel,
                receipt_rel,
                source_roots,
                affected_scope,
                &current_seed_digest,
                generation_ordinal,
                &liveness_model,
                round_subject.as_ref(),
            ) {
                Ok(next) => {
                    candidate_digest = next.manifest.candidate_tree_digest.clone();
                    candidate_tree_id = next.manifest.candidate_tree_id.clone();
                    drifted = next.drifted;
                    authority_digest = next.authority_digest;
                    candidate_manifest = next.manifest;
                }
                Err(failure) => {
                    restore_round_journal(&workspace, round_subject.as_ref())?;
                    return Err(format!(
                        "next generation refused; checkpoint restored: {failure}"
                    ));
                }
            }
            continue;
        }
        round_records.push(format!(
            "generation {generation_ordinal}: first_generation_equal=false drifted={drifted:?}"
        ));
        if regen_convergence_bound_exceeded(source_roots, generation_ordinal)? {
            restore_round_journal(&workspace, round_subject.as_ref())?;
            return Err(format!(
                "RegenConvergenceBoundExceeded: no fixed point within the declared bound \
                 (regen_convergence_max_generations) at generation {generation_ordinal}; checkpoint restored; nothing is published -- {}",
                round_records.join("; ")
            ));
        }
        let admitted_manifest = admit_candidate_manifest(
            &convergence_model,
            &candidate_src,
            &candidate_manifest,
            &current_seed_digest,
        )?;
        let state = format!("{current_seed_digest}:{candidate_digest}");
        let seen_state_keys = seen_states.iter().cloned().collect::<Vec<_>>();
        let (kind, install_set, dependency_closure_id) = convergence_plan_from_model(
            &convergence_model,
            generation_ordinal + 1,
            &candidate_manifest.generation_id,
            &candidate_tree_id,
            &candidate_digest,
            &drifted,
            &admitted_manifest,
            &stage0_src,
            &basename_to_module,
            &generation_modules,
            &bootstrap_sources,
            &bootstrap_products,
            &seed_embedded_basenames,
            &generated_product_roles,
            &scope,
            &seen_state_keys,
            &current_seed_digest,
            generation_bound,
        )?;
        seen_states.insert(state);
        let stage_kind = kind;
        let install_population: BTreeSet<String> = install_set.iter().cloned().collect();
        let deferred_reason = match stage_kind {
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs => {
                RegenConvergenceDeferredReasonReceipt::AwaitingPromotedProducer
            }
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut => {
                RegenConvergenceDeferredReasonReceipt::AwaitingSeedCompatibilityCut
            }
            RegenConvergenceStageKindReceipt::PublishNonSeedOutputs => {
                RegenConvergenceDeferredReasonReceipt::AwaitingBuildableSeedGeneration
            }
        };
        let deferred = drifted
            .iter()
            .filter(|path| !install_population.contains(*path))
            .map(|path| RegenConvergenceDeferredSurfaceReceipt {
                projected_path: path.clone(),
                reason: deferred_reason,
            })
            .collect::<Vec<_>>();
        if install_set.is_empty() {
            restore_round_journal(&workspace, round_subject.as_ref())?;
            return Err(format!(
                "generation made no progress with drift {drifted:?}"
            ));
        }
        transaction_admitted.extend(drifted.iter().cloned());
        // JOIN 1, before any byte moves: the population this generation reported changed must be
        // exactly the population the stage plans plus the population it postpones.
        admit_install_boundary_population_from_model(
            &convergence_model,
            &drifted,
            &install_set,
            &deferred
                .iter()
                .map(|row| row.projected_path.clone())
                .collect::<Vec<_>>(),
            &basename_to_module,
        )
        .map_err(|failure| {
            format!("install boundary population disagrees with the stage partition: {failure}")
        })?;
        // THE BOUNDARY POPULATION JOINS THE STAGE THAT IS ALREADY PENDING. The post-install
        // binary's own producer names the boundary files it would change; when a mirror stage is
        // pending anyway they install in THE SAME install, so that stage's one rebuild covers
        // both and no separate boundary rebuild or regeneration runs. Only boundary drift that
        // first shows after the mirrors converged takes the tail arm above.
        let stage_boundary = if partition_moved {
            match built_seed_boundary_drift(&workspace) {
                Ok(paths) => paths,
                Err(failure) => {
                    restore_round_journal(&workspace, round_subject.as_ref())?;
                    return Err(format!("{failure}; checkpoint restored"));
                }
            }
        } else {
            Vec::new()
        };
        let stage_whole_compiler = stage_rebuild_is_whole_compiler(source_roots, &stage_boundary)?;
        if !stage_boundary.is_empty() {
            round_records.push(format!(
                "generation {generation_ordinal}: boundary drift under the post-install producer \
                 {stage_boundary:?} installs with the pending mirrors"
            ));
            if let Err(failure) =
                install_built_seed_boundary(&workspace, &stage_boundary, round_subject.as_ref())
            {
                restore_round_journal(&workspace, round_subject.as_ref())?;
                return Err(format!(
                    "boundary install refused; checkpoint restored: {failure}"
                ));
            }
        }
        let drifted_entering_stage = drifted.clone();
        // THE FIRST INSTALL IS A DEMAND: the stage journals what it overwrites, under the round's
        // subject. Produced here if nothing demanded it earlier, and checked against this
        // generation's own reading of the closure.
        let subject = establish_round_subject(
            &mut round_subject,
            Some((generation_ordinal, &authority_digest)),
            || checkpoint_subject_child(&workspace, &liveness_model),
        )?
        .clone();
        v1_rt::trace_mark("round.install.begin".to_string());
        let stage_result = install_convergence_stage(
            &convergence_model,
            source_roots,
            &workspace,
            &stage0_src,
            &candidate_src,
            &install_set,
            &base_rewritten_mirrors,
            &admitted_manifest,
            &basename_to_module,
            generation_ordinal + 1,
            stage_kind,
            &current_seed_digest,
            &candidate_manifest.generation_id,
            &candidate_tree_id,
            &candidate_digest,
            &dependency_closure_id,
            stage_whole_compiler,
            &liveness_model,
            &subject,
        );
        v1_rt::trace_mark("round.install.done".to_string());
        let mut stage = match stage_result {
            Ok((stage, moved)) => {
                partition_moved |= moved;
                stage
            }
            Err(failure) => {
                restore_round_journal(&workspace, round_subject.as_ref())?;
                return Err(format!(
                    "stage seed build refused; checkpoint restored: {failure}"
                ));
            }
        };
        stage.deferred_surfaces = deferred;
        current_seed_digest = stage.output_seed_digest.clone();
        stages.push(stage);
        generation_ordinal += 1;
        if let Err(failure) = admit_generation_begin(
            &convergence_model,
            generation_ordinal,
            Some(orchestrator_resident_at_first),
        ) {
            restore_round_journal(&workspace, round_subject.as_ref())?;
            return Err(format!(
                "next generation refused; checkpoint restored: {failure}"
            ));
        }
        v1_rt::trace_mark("round.built_seed_regen.begin".to_string());
        report_cgroup_memory_diagnostic(&format!("parent.begin generation={generation_ordinal}"));
        let next = run_built_seed_regen(
            &workspace,
            candidate_dir_rel,
            receipt_rel,
            source_roots,
            affected_scope,
            &current_seed_digest,
            generation_ordinal,
            &liveness_model,
            round_subject.as_ref(),
        );
        report_cgroup_memory_diagnostic(&format!("parent.end generation={generation_ordinal}"));
        v1_rt::trace_mark("round.built_seed_regen.done".to_string());
        match next {
            Ok(next) => {
                // APPLIED comes from the stage RECEIPT, never from the plan: a surface the loop
                // dropped leaves no receipt row, so the lineage join can see the hole.
                for surface in &stages
                    .last()
                    .expect("a stage receipt was just pushed")
                    .surfaces
                {
                    if let Some(basename) = surface.projected_path.rsplit('/').next() {
                        transaction_lineage.insert(
                            basename.to_string(),
                            ConvergenceDisposition::Applied {
                                installed_digest: surface.installed_digest.clone(),
                            },
                        );
                    }
                }
                // SUPERSEDED is the honest name for a surface that entered the generation drifted
                // and left it undrifted without being installed: a later candidate replaced the
                // one that was pending, so its lineage ENDS -- it is not deferred, and calling it
                // deferred would report an unfinished transaction as finished.
                let next_drift: BTreeSet<&String> = next.drifted.iter().collect();
                for basename in &drifted_entering_stage {
                    if !next_drift.contains(basename) && !transaction_lineage.contains_key(basename)
                    {
                        transaction_lineage.insert(
                            basename.clone(),
                            ConvergenceDisposition::Superseded {
                                by_generation_id: next.manifest.generation_id.clone(),
                            },
                        );
                    }
                }
                candidate_digest = next.manifest.candidate_tree_digest.clone();
                candidate_tree_id = next.manifest.candidate_tree_id.clone();
                drifted = next.drifted;
                authority_digest = next.authority_digest;
                candidate_manifest = next.manifest;
            }
            Err(failure) => {
                restore_round_journal(&workspace, round_subject.as_ref())?;
                return Err(format!(
                    "next generation refused; checkpoint restored: {failure}"
                ));
            }
        }
    }

    // Anything still drifting at the terminal is a PROMISE, not an outcome, and the lineage join
    // refuses it. The loop exits on an empty drift so a converged run records none -- the arm
    // exists because a fixed point that was never reached must not be able to render as one.
    for basename in &drifted {
        transaction_lineage
            .entry(basename.clone())
            .or_insert(ConvergenceDisposition::Deferred);
    }
    // JOIN 3: what was admitted across the whole transaction, against what the receipts account
    // for, by identity in both directions and with every row required to have terminated.
    admit_transaction_lineage_from_model(
        &convergence_model,
        &transaction_admitted,
        &transaction_lineage,
        &basename_to_module,
    )
    .map_err(|failure| {
        format!("the admitted population did not survive to the terminal lineage: {failure}")
    })?;
    let changed_paths = git_changed_stage0_paths(&workspace)?;
    let terminal_surface_digest = candidate_digest.clone();
    let ordered_stage_receipt_ids = stages
        .iter()
        .map(|stage| stage.receipt_id.clone())
        .collect::<Vec<_>>();
    let transaction_receipt = RegenConvergenceReceipt {
        schema_version: REGEN_CONVERGENCE_SCHEMA.to_string(),
        starting_commit: tree.clone(),
        source_authority_digest: authority_digest.clone(),
        starting_generated_surface_digest: starting_surface_digest,
        stage_plan_authority_digest: path_digest(
            &workspace.join("src/v2/workflow/regen_convergence_transaction.dag"),
        )?,
        generation_role_authority_digest: path_digest(
            &workspace.join("dag/gunbc/regen_affected_set.dag"),
        )?,
        ownership_authority_digest: path_digest(&workspace.join("src/v1/05_emit_rust.dag"))?,
        initial_seed_digest,
        ordered_stage_receipt_ids: ordered_stage_receipt_ids.clone(),
        stages,
        terminal_seed_digest: current_seed_digest,
        terminal_surface_digest,
        fixed_point_verdict: RegenConvergenceFixedPointReceipt::Reached,
    };
    fs::write(
        workspace.join(REGEN_CONVERGENCE_RECEIPT_REL),
        serde_json::to_vec_pretty(&transaction_receipt)
            .map_err(|e| format!("encode convergence receipt: {e}"))?,
    )
    .map_err(|e| format!("write convergence receipt: {e}"))?;
    let completed_journal = regen_convergence_journal_path(&workspace);
    if completed_journal.exists() {
        fs::remove_dir_all(&completed_journal)
            .map_err(|e| format!("remove completed convergence journal: {e}"))?;
    }

    let rebuild_compiled_crates = transaction_receipt
        .stages
        .iter()
        .map(|stage| stage.build_compiled_crates)
        .sum();
    // The receipt field is named in the model's vocabulary: a "mirror" is the basename the
    // partition rows and rosters key on. The stages carry projected PATHS, so project through
    // the single bridge -- feeding paths to the decision model rendered a spurious
    // MirrorHasNoOwningPackage line on every drifted round's receipt.
    let installed_mirrors = transaction_receipt
        .stages
        .iter()
        .flat_map(|stage| stage.surfaces.iter())
        .map(|surface| emit_path_basename(&surface.projected_path).to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let convergence_stage_receipt_ids = ordered_stage_receipt_ids;
    let rebuild_packages = transaction_receipt
        .stages
        .iter()
        .flat_map(|stage| stage.build_packages.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let executable_digest = transaction_receipt.terminal_seed_digest.clone();
    let second_generation_candidate_digest = transaction_receipt.terminal_surface_digest.clone();

    // The runtime's `Vec` is the persistent vector its emitted programs use; the receipt
    // renderer takes a slice, so the rows are collected once here.
    let drained_marks: std::vec::Vec<v1_rt::TraceLedgerRow> = v1_rt::trace_ledger_drain()
        .ok_or_else(|| {
            "refusal: the trace ledger was not armed, so no phase was recorded".to_string()
        })?
        .iter()
        .cloned()
        .collect();
    let (seed_build_crates_total, marks) = compose_seed_build_cost(
        carried_cost.as_ref(),
        seed_build.compiled_crates,
        drained_marks,
    )
    .map_err(|cause| cause.to_string())?;
    let declared_shell_modules = host_shell_scope_modules(
        &stage0_src.join("lib.rs"),
        &stage0_src.join("../Cargo.toml"),
    )?;

    // ONE AUTHORITY FOR THE RECEIPT'S BASE ARM, read by whichever image reports. The parent holds
    // what its own seed did. The handoff child seeds nothing (the handoff rule), so it DERIVES the
    // same arm from the same read-only authority; passing `None` there would select the renderer's
    // `SeedBuiltFromHead` arm and print a receipt claiming round 1 compiled the HEAD — a fabricated
    // output in the one run whose purpose is to record which bytes round 1 compiled. No `.ok()`
    // anywhere: a base commit that cannot be named refuses the round with its typed
    // `BootstrapBaseMirrorCommitUnresolved` diagnostic rather than dropping the line silently.
    let receipt_base_authority: Option<(String, usize, usize, usize)> =
        match base_bootstrap.as_ref() {
            Some(restored) => Some(base_mirror_receipt_authority(restored)),
            None if base_mirror => Some(round_one_receipt_base_authority(
                &workspace,
                std::env::var(REGEN_ROUND_COST_BASE_MIRROR_ENV)
                    .ok()
                    .as_deref(),
            )?),
            None => None,
        };
    let rendered = render_round_cost_receipt(
        source_roots,
        &host,
        &tree,
        tree_dirty,
        seed_build_crates_total,
        rebuild_compiled_crates,
        rustfmt_spawn_count() - rustfmt_spawns_before,
        &marks,
        &changed_paths,
        &convergence_stage_receipt_ids,
        &installed_mirrors,
        &declared_shell_modules,
        &rebuild_packages,
        &executable_digest,
        &second_generation_candidate_digest,
        receipt_base_authority
            .as_ref()
            .map(|(commit, seeded, head_added, head_deleted)| {
                (commit.as_str(), *seeded, *head_added, *head_deleted)
            }),
    )?;
    let receipt_path = workspace.join(REGEN_ROUND_COST_RECEIPT_REL);
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    fs::write(&receipt_path, &rendered)
        .map_err(|e| format!("write {}: {e}", receipt_path.display()))?;
    Ok(RegenRoundCostOutcome {
        rendered,
        receipt_path,
        round_failures,
        // THE RECEIPT NAMES ITS SOURCE IN EVERY IMAGE OF THE ROUND, not only in the one that did
        // the seeding, and both readers of that fact read the SAME value: the rendered line and
        // this field come from `receipt_base_authority`. Under the handoff rule the image that
        // reports is the child, which seeds nothing; if the commit came only from `base_bootstrap`,
        // this PR's own authority — "which bytes did round 1 compile" — would disappear from
        // exactly the run that answers it, and the rendered receipt would claim the head instead.
        base_mirror_commit: receipt_base_authority
            .as_ref()
            .map(|(commit, _, _, _)| commit.clone()),
    })
}

/// THE ROUND-1 BOOTSTRAP — DESIGN §5, `gunbc.recurring_failure_mode`
/// `heal_round_one_bootstraps_from_the_mirror_the_change_must_repair` (RFM #14963).
///
/// These tests are the discriminating evidence for the heal round-1 deadlock hit on PR #13936
/// (heal run 40064940799, job 120786085841, step 7 "Converge the stage0 mirrors to the seed's own
/// fixed point"): the round refused with `stage seed build refused … cargo build
/// (round.rebuild_from_installed) failed with exit status: 101`, and the build's own output was
/// five rows of `error: shared layer refused for 'Nat': TypeDeclNotDeclared — wrap/deref/Copy of a
/// user type requires one declaration identity`, at `../../stage0/src/std_measure.rs:650:34`,
/// `:652:90`, `:1389:12` and `../../stage0/src/std_checked_arithmetic.rs:316:28`, `:320:70`. Those
/// are rows of the COMMITTED MIRROR, compiled into `v1-stage0-std-core` through its
/// `#[path = "../../stage0/src/std_measure.rs"]` include — and they are exactly what the PR's
/// `.dag` edits remove. The round was refused by the artifact its own repair regenerates, so no
/// amount of regenerating could reach the fix: the head's mirror was the thing blocking the fix.
///
/// The fixture is a real git repo with a real base commit and a real PR-head commit, because the
/// authority under test IS a commit. A string fake would test plumbing and prove nothing about
/// which bytes a round compiles.
#[cfg(test)]
mod heal_round_one_base_mirror_tests {
    use super::*;

    /// `(root, stage0 src, base commit id)`. BASE holds a generated mirror, a hand-maintained
    /// file, and a hand-maintained directory. HEAD carries the damage the PR came to repair
    /// (the `Nat` refusal spelled in the generated mirror), its own hand-authored repair, and a
    /// mirror base has no form of.
    /// THE FIXTURE, parameterized by the one shape the bootstrap authority had never seen: a module
    /// the BASE tracks and the HEAD DELETES. `deleted` is a path relative to the crate's `src`
    /// directory; when it is `Some`, base carries that file and the head removes it in its own commit,
    /// which is exactly the PR shape that made round 1 compile a base/head chimera (PR #15418, PR
    /// #15540). When it is `None` the fixture is byte-for-byte the one every existing test uses.
    fn base_mirror_fixture_with_deleted(deleted: Option<&str>) -> (PathBuf, PathBuf, String) {
        let root = std::env::temp_dir().join(format!(
            "gunbc-heal-base-mirror-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap_or_else(|e| panic!("fixture root must be fresh: {e}"));
        let stage0 = root.join(STAGE0_SRC_REL);
        fs::create_dir_all(&stage0).unwrap();
        fs::create_dir_all(stage0.join("bin")).unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(&root)
                .env("GIT_AUTHOR_NAME", "heal fixture")
                .env("GIT_AUTHOR_EMAIL", "heal@example.invalid")
                .env("GIT_COMMITTER_NAME", "heal fixture")
                .env("GIT_COMMITTER_EMAIL", "heal@example.invalid")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        let head_sha = || {
            String::from_utf8_lossy(
                &Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .current_dir(&root)
                    .output()
                    .unwrap()
                    .stdout,
            )
            .trim()
            .to_string()
        };

        // BASE — the ref the PR was opened against.
        fs::write(
            stage0.join("fixture_emitted.rs"),
            "// base mirror bytes -- accepted by the installed compiler\n",
        )
        .unwrap();
        fs::write(
            stage0.join("std_logic.rs"),
            "// base hand-maintained bytes\n",
        )
        .unwrap();
        fs::write(
            stage0.join("bin").join("fixture_main.rs"),
            "// base hand-maintained main\n",
        )
        .unwrap();
        // A NESTED GENERATED PARTITION: the shape the old filter skipped. In the real repo this is
        // `src/v1/stage0/src/cli_run/compile_clean.rs`, which `v1-stage0-std-core` compiles through
        // a recursive `#[path]` include, so a base-skipped nested file is a base/head chimera in the
        // very crate the seed build refuses in (PR #15175, PR #14876).
        fs::create_dir_all(stage0.join("cli_run")).unwrap();
        fs::write(
            stage0.join("cli_run").join("fixture_partition.rs"),
            "// base nested partition bytes -- defines MirrorRow::Dag\n",
        )
        .unwrap();
        fs::write(
            root.join(STAGE0_CRATE_REL).join("build.rs"),
            "// base build identity source\n",
        )
        .unwrap();
        // A MODULE THE HEAD WILL DELETE, in a NESTED directory: the shape of PR #15540, whose lost
        // path was `src/v1/stage0/src/cli_run/required_lane_resolution_census.rs`. Base's crate is
        // incomplete without it, and the head's tree is incomplete with it -- which is the whole
        // difficulty the authority has to get right in both directions.
        if let Some(retired) = deleted {
            fs::write(
                stage0.join(retired),
                "// base bytes for a module the head deletes\n",
            )
            .unwrap();
            // A SECOND NESTED MODULE THE HEAD KEEPS UNEDITED: the survivor is the control that the
            // round's journal and restore leave an untouched file alone, bytes AND mtime. A module
            // the head edits cannot show that, because the round rewrites it in both directions.
            fs::write(
                stage0.join("cli_run").join("fixture_survivor.rs"),
                "// base bytes for a module the head keeps exactly as they are\n",
            )
            .unwrap();
        }
        // The `.dag` sources are OUTSIDE the crate the authority owns: they are what the round
        // regenerates FROM, so the pull request's edits there must survive the bootstrap.
        fs::create_dir_all(root.join("dag")).unwrap();
        fs::write(
            root.join("dag").join("fixture_authority.dag"),
            "// base .dag authority\n",
        )
        .unwrap();
        git(&["init", "-q", "--initial-branch=main"]);
        git(&["add", "-A"]);
        git(&["commit", "-qm", "base"]);
        let base_commit = head_sha();

        // HEAD — the PR. Its committed mirror carries the refusal its `.dag` edits remove.
        git(&["checkout", "-q", "-b", "pr"]);
        fs::write(
            stage0.join("fixture_emitted.rs"),
            "// head mirror bytes -- shared layer refused for 'Nat'\n",
        )
        .unwrap();
        fs::write(
            stage0.join("std_logic.rs"),
            "// head hand-maintained bytes -- part of the repair\n",
        )
        .unwrap();
        fs::write(
            stage0.join("bin").join("fixture_main.rs"),
            "// head hand-maintained main -- part of the repair\n",
        )
        .unwrap();
        fs::write(
            stage0.join("fixture_added_by_pr.rs"),
            "// a mirror with no base form\n",
        )
        .unwrap();
        // The head's damage in the NESTED partition: this is where the E0599 lived, because the
        // round read the head's damaged nested file against base's top-level definitions.
        fs::write(
            stage0.join("cli_run").join("fixture_partition.rs"),
            "// head nested partition bytes -- variant `Dag` constructed, constructor missing\n",
        )
        .unwrap();
        // A HEAD-ADDED file in a hand-maintained directory: base has no form of it, and it is
        // exactly where a new hand-written call to a new generated method would live.
        fs::write(
            stage0.join("bin").join("fixture_added_by_pr_main.rs"),
            "// head-only hand-authored caller; base has no form of this path\n",
        )
        .unwrap();
        // The PR's own repair, written in the authority the round regenerates from. If the bootstrap
        // reached outside its crate it would revert this and the round would regenerate the damage.
        fs::write(
            root.join("dag").join("fixture_authority.dag"),
            "// head .dag authority -- the repair this PR authored\n",
        )
        .unwrap();
        write_fixture_stage_plan(&root);
        git(&["add", "-A"]);
        git(&["commit", "-qm", "head"]);
        if let Some(retired) = deleted {
            git(&["rm", "-q", &format!("{STAGE0_SRC_REL}/{retired}")]);
            git(&["commit", "-qm", "head deletes a module"]);
        }
        (root, stage0, base_commit)
    }

    /// The fixture as every existing test has always used it: no head-deleted module.
    fn base_mirror_fixture() -> (PathBuf, PathBuf, String) {
        base_mirror_fixture_with_deleted(None)
    }

    /// The fixture WITH a head-deleted nested module: base tracks it, the head removes it.
    fn base_mirror_deletion_fixture() -> (PathBuf, PathBuf, String) {
        base_mirror_fixture_with_deleted(Some("cli_run/fixture_retired.rs"))
    }

    /// The crate's tracked population AT A COMMIT, read straight from git. A test that reused the
    /// set the code under test computes could only prove the writer agrees with itself; this reads
    /// the commit, which is the thing the authority is supposed to agree with.
    fn fixture_crate_paths_at_commit(root: &Path, commit: &str) -> Vec<String> {
        let output = Command::new("git")
            .current_dir(root)
            .args([
                "ls-tree",
                "-r",
                "--name-only",
                commit,
                "--",
                STAGE0_CRATE_REL,
            ])
            .output()
            .expect("git ls-tree runs");
        assert!(
            output.status.success(),
            "git ls-tree {commit} -- {STAGE0_CRATE_REL}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("path list is utf8")
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// The blob a commit holds for one path, read straight from git.
    fn fixture_blob_at_commit(root: &Path, commit: &str, path: &str) -> String {
        let output = Command::new("git")
            .current_dir(root)
            .args(["show", &format!("{commit}:{path}")])
            .output()
            .expect("git show runs");
        assert!(
            output.status.success(),
            "git show {commit}:{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("blob is utf8")
    }

    /// THE DELETION SHAPE, ASSERTED IN BOTH DIRECTIONS. Base tracks a nested module the head
    /// DELETES, plus a second nested module the head keeps byte-for-byte. Round 1 must compile
    /// BASE's crate — every path base tracks present with base's bytes, the deleted one included —
    /// and the round's own teardown must put the tree back to HEAD: the deleted module absent
    /// again, the survivor untouched down to its mtime.
    ///
    /// THIS TEST IS ALSO THE CONTROL FOR THE RULE IT REPLACES, and it was written and run against
    /// that rule BEFORE the rule was changed: there it fails at the first assertion, because the
    /// old authority restored only the paths base and the worktree shared, a path the head deleted
    /// is by definition absent from the worktree, so it was never written, and base's restored
    /// `lib.rs` went on declaring a module with no file. That is the E0583 in the heal runs of PR
    /// #15418 (run 41605027842, head 5b7b2bc2, base 5ff414e6, `lib.rs` lines 755 and 832) and PR
    /// #15540 (run 41591786053, head 3be90ce5, base 6e1c65a3, the lost path nested under `cli_run/`).
    /// The old rule is NOT re-implemented here: a second copy of it would be a second authority,
    /// and the red comes from the run at the pre-fix revision instead.
    #[test]
    fn a_head_deleted_module_is_restored_for_round_one_and_removed_after_it() {
        let (root, stage0, base_commit) = base_mirror_deletion_fixture();
        let retired = stage0.join("cli_run").join("fixture_retired.rs");
        let survivor = stage0.join("cli_run").join("fixture_survivor.rs");
        let survivor_bytes = read(&survivor);
        let survivor_mtime = fs::metadata(&survivor)
            .expect("survivor metadata")
            .modified()
            .expect("survivor mtime");
        assert!(
            !retired.exists(),
            "the fixture's head must not carry the module its commit deleted"
        );

        bootstrap_round_one_from_base_mirror(&root, Some(&base_commit), &fixture_subject(&root))
            .expect("the round-1 bootstrap runs");

        // ROUND 1 SEES BASE'S CRATE, PATH FOR PATH.
        for path in fixture_crate_paths_at_commit(&root, &base_commit) {
            let on_disk = root.join(&path);
            assert!(
                on_disk.is_file(),
                "round 1 left {path} absent, so the crate it compiled was neither base's nor                  head's: base declares this path and the tree does not have it"
            );
            assert_eq!(
                read(&on_disk),
                fixture_blob_at_commit(&root, &base_commit, &path),
                "round 1 left {path} short of the base commit it claims to compile"
            );
        }
        assert!(
            retired.is_file(),
            "round 1 did not restore the module the head deleted"
        );

        // THE ROUND'S OWN TEARDOWN PUTS THE TREE BACK TO HEAD.
        let put_back_paths =
            restore_regen_convergence_journal_for_subject(&root, &fixture_subject(&root))
                .expect("the round ends its snapshot");
        assert!(
            !retired.exists(),
            "a module the head deleted survived the round into the tree the emitter walks:              {put_back_paths:?}"
        );
        assert!(
            put_back_paths
                .removed
                .iter()
                .any(|path| path.starts_with(STAGE0_CRATE_REL)
                    && path.ends_with("/fixture_retired.rs")),
            "the restore did not report the resurrected path among the paths it REMOVED: \
             {put_back_paths:?}"
        );
        assert!(
            put_back_paths
                .removed
                .iter()
                .all(|path| path.ends_with("/fixture_retired.rs")),
            "a path other than the head-deleted one was removed, which would mean the round \
             deleted something the head tracks: {put_back_paths:?}"
        );
        assert_eq!(
            read(&survivor),
            survivor_bytes,
            "the round changed a module neither base nor the head touched"
        );
        assert_eq!(
            fs::metadata(&survivor)
                .expect("survivor metadata")
                .modified()
                .expect("survivor mtime"),
            survivor_mtime,
            "the round rewrote a module neither side touched, bumping an mtime cargo's              path-dependency fingerprint reads as a change (PR #14876): {put_back_paths:?}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// The subject a round journals under. The three digests are fixed for one fixture: what the
    /// tests exercise is that the bootstrap journalled under THE SUBJECT THE ROUND RESTORES, and
    /// the identity formula has its own tests. `git` is left to the code under test, which is the
    /// only thing here that must read a real commit.
    fn fixture_subject(root: &Path) -> RegenConvergenceCheckpointSubject {
        let (starting_commit, stage_plan_authority_digest) =
            checkpoint_subject_live_facts(root).expect("fixture commit and stage plan");
        RegenConvergenceCheckpointSubject {
            starting_commit,
            source_authority_digest: "fixture-source-authority".to_string(),
            stage_plan_authority_digest,
        }
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    /// THE CONTROL THE MANAGER ASKED FOR, for the class that stranded PR #15175 (E0599
    /// `variant ... does not exist`) and PR #14876 (26 E0308/E0282 in `v1-stage0-std-core`, heal run
    /// 40739785316): a PR whose damage is in a NESTED partition and whose repair reaches a file in a
    /// hand-maintained directory. Before the whole-crate authority, the restore population was the
    /// top-level generated name list, so the nested partition kept its damaged head bytes while every
    /// top-level file held base bytes — a base/head chimera — and the hand-authored call the PR added
    /// was left beside definitions it was never written against. Both arms are asserted here: the
    /// nested file and the hand file hold BASE bytes after the bootstrap (red before the fix, where
    /// they held head bytes), and the journal covers both, so the rollback puts the PR's own bytes
    /// back rather than stranding the working tree.
    #[test]
    fn the_whole_crate_authority_repairs_the_nested_and_hand_directory_chimera() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        let nested = stage0.join("cli_run").join("fixture_partition.rs");
        let hand_dir_file = stage0.join("bin").join("fixture_main.rs");
        let head_added_bin = stage0.join("bin").join("fixture_added_by_pr_main.rs");
        assert!(
            read(&nested).contains("variant `Dag` constructed, constructor missing"),
            "the fixture's head must be the damaged nested state the class started in"
        );

        let restored = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect("the round-1 bootstrap runs");
        assert!(
            read(&nested).contains("defines MirrorRow::Dag"),
            "the nested partition kept the head's damaged bytes: {}",
            read(&nested)
        );
        assert!(
            read(&hand_dir_file).contains("base hand-maintained"),
            "the nested-directory hand file was left at head: {}",
            read(&hand_dir_file)
        );
        // The population the receipt names is every path the authority holds, not a top-level slice.
        assert_eq!(
            restored.paths_absent_at_base, 2,
            "the receipt must count the paths only the head has"
        );
        assert!(
            read(&head_added_bin).contains("head-only hand-authored caller"),
            "a path base has never contained was reverted instead of kept"
        );

        // The rollback covers the same population: a round that fails returns the PR's bytes in the
        // nested and hand-directory files too, so the working tree is never left on base.
        restore_regen_convergence_journal_for_subject(&root, &fixture_subject(&root))
            .expect("the round rolls back");
        assert!(
            read(&nested).contains("variant `Dag` constructed"),
            "the rollback left base bytes in the nested partition"
        );
        assert!(
            read(&hand_dir_file).contains("head hand-maintained"),
            "the rollback left base bytes in a hand-maintained file"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE SUCCESS PATH IS THE ONE THAT PUBLISHES, SO IT IS THE PATH THIS CONTROL OWNS (review 87490
    /// on #15339). A round that SUCCEEDS never reaches a refusal path, and before this change no
    /// refusal path ran either: the round deleted the convergence journal at the end and left every
    /// seeded byte in place. `gunbc.heal_candidate_stage0` `mirror_paths_of` takes any top-level `.rs`
    /// in the mirror directory, filtered by depth and suffix alone, so a hand-maintained file still
    /// holding base's bytes entered the candidate and the bot commit reverted the pull request's own
    /// Rust — on a green round. This PR is the specimen: `required_regen_host.rs`, the file that
    /// implements the bootstrap, is itself in that population.
    ///
    /// The round's own teardown is the single authority that ends the snapshot (the rollback, not a
    /// second journal reader), so the control drives exactly that call and asserts three things a
    /// silent fallback cannot satisfy: it reports the number it put back, every file the head
    /// rewrote reads the HEAD's bytes afterwards, and the journal is gone so no later round can
    /// restore over the regeneration. Only paths the HEAD actually rewrote are asserted: a file
    /// committed identically on both sides (the crate's build script) would pass either way.
    #[test]
    fn a_succeeded_round_leaves_no_base_byte_for_the_candidate_to_publish() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        let hand_file = stage0.join("std_logic.rs");
        let nested = stage0.join("cli_run").join("fixture_partition.rs");
        let hand_dir_file = stage0.join("bin").join("fixture_main.rs");

        let restored = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect("the round-1 bootstrap runs");
        for (path, what) in [
            (&hand_file, "hand-maintained file"),
            (&hand_dir_file, "file inside a hand-maintained directory"),
        ] {
            assert!(
                read(path).contains("base "),
                "the fixture's bootstrap did not seed the {what}, so this control would pass \
                 without exercising the hazard: {}",
                read(path)
            );
        }

        // What a SUCCEEDED round now does before anything observes or publishes the worktree.
        let put_back_paths =
            restore_regen_convergence_journal_for_subject(&root, &fixture_subject(&root))
                .expect("the round ends its snapshot");
        let put_back = put_back_paths.rewritten.len() + put_back_paths.removed.len();
        // THE DECISION'S CHANGED SET IS THESE PATHS, NOT AN MTIME SCAN (RFM
        // `heal_partition_guard_infers_changed_mirrors_from_mtimes_not_rewritten_paths`): the restore
        // names each file it moved under the built artifacts, workspace-relative, so the rebuild
        // scope can list them. A count alone cannot feed it, and a path the restore skipped (equal
        // bytes) must not appear.
        assert!(
            put_back_paths
                .rewritten
                .iter()
                .any(|path| path.starts_with(STAGE0_CRATE_REL) && path.ends_with("/std_logic.rs")),
            "the restore did not name the hand-maintained file it moved: {put_back_paths:?}"
        );
        assert!(
            put_back_paths.rewritten.iter().all(|path| !path.ends_with("/build.rs")),
            "the restore named a path it never moved (its bytes were identical): {put_back_paths:?}"
        );
        // The journal covers exactly what the seed MOVED. A path whose base bytes already equal the
        // head's is never written (a rewrite would bump an mtime cargo misreads as a change — the
        // #14876 / #15310 specimen), so it is never journalled and has nothing to put back; a
        // head-added path is journalled by the install that overwrites it, not by the seed. So the
        // teardown's count is the seed's rewrite count, and the byte assertions below are what prove
        // no base byte survives where the candidate reads.
        assert_eq!(
            put_back, restored.paths_rewritten,
            "the teardown did not return every path the seed moved ({}), so a base byte is still on \
             disk where the candidate reads",
            restored.paths_rewritten
        );
        assert_eq!(
            (restored.paths_restored, restored.paths_rewritten, restored.paths_absent_at_base),
            (5, 4, 2),
            "the fixture's population moved: base carries 5 paths under {STAGE0_CRATE_REL}, the head \
             rewrites 4 of them (its build script is committed identically, so the seed must skip \
             it), and the head adds 2 — this control's arithmetic reads those numbers"
        );
        assert!(put_back > 0, "the teardown reported restoring nothing");
        for (path, needle, what) in [
            (
                hand_file.clone(),
                "head hand-maintained",
                "hand-maintained file",
            ),
            (
                nested.clone(),
                "variant `Dag` constructed",
                "nested partition",
            ),
            (
                hand_dir_file.clone(),
                "head hand-maintained main",
                "file inside a hand-maintained directory",
            ),
        ] {
            assert!(
                read(&path).contains(needle),
                "a succeeded round left the {what} at base's bytes; the candidate would publish \
                 a revert of the pull request's own edit: {}",
                read(&path)
            );
        }
        assert!(
            !regen_convergence_journal_path(&root).exists(),
            "the journal survived a succeeded round, so a later round could restore over the \
             regeneration"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE HANDOFF CHILD IS NOT A SECOND OWNER OF THE TREE (manager ruling 2026-10-29).
    ///
    /// A re-exec'd image was exec'd to report on a build that consumed the tree it is handed. The
    /// old behaviour was to restore the journal and seed again on arrival, and each half of that is
    /// fatal in a way this control pins separately, with real bytes rather than with a flag:
    ///
    ///  * the restore puts the HEAD's bytes back, so the image reports on a tree its parent did not
    ///    compile — cargo rebuilds, the relink trips `ExecutableReplacedAfterHandoff`, and the
    ///    refusal is correct, because the measurement would otherwise describe bytes the emitting
    ///    image never built;
    ///  * the restore also DELETES the journal, so the round's teardown has nothing left to return
    ///    the head's bytes from, and the base snapshot becomes unrecoverable — which is the
    ///    publication hazard this PR fixes, reintroduced by a child that touched the tree.
    ///
    /// Both arms are asserted, then the child's actual (empty) arrival is run and the parent's tree
    /// and journal are asserted intact: the two things the round depends on, still true at the
    /// moment the child starts work.
    #[test]
    fn a_handoff_child_arrives_on_the_tree_it_was_exec_d_into() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        let hand_file = stage0.join("std_logic.rs");
        // A path the whole-crate population seeds that the generator never rewrites: the load-bearing
        // assertion in the last arm, because a generated mirror would be rewritten anyway.
        let build_script = root.join(STAGE0_CRATE_REL).join("build.rs");
        let subject = fixture_subject(&root);

        // The parent's act: seed the base snapshot, journal the head's bytes, build from that tree.
        let restored = bootstrap_round_one_from_base_mirror(&root, Some(&base_commit), &subject)
            .expect("the round-1 bootstrap runs");
        assert!(
            read(&hand_file).contains("base hand-maintained")
                && read(&build_script).contains("base build identity source"),
            "the parent did not seed the tree it would build from, so this control proves nothing"
        );
        let journal = regen_convergence_journal_path(&root);
        assert!(journal.exists(), "the parent journalled nothing");

        // ARM 1 — WHAT THE CHILD USED TO DO ON ARRIVAL. Restoring the journal is what made the two
        // images' trees disagree, and it destroyed the record the teardown needs. If a future edit
        // hands the startup restore back to the child, this arm is the one that goes red.
        {
            let (throwaway, throwaway_stage0, throwaway_base) = base_mirror_fixture();
            let throwaway_hand = throwaway_stage0.join("std_logic.rs");
            bootstrap_round_one_from_base_mirror(
                &throwaway,
                Some(&throwaway_base),
                &fixture_subject(&throwaway),
            )
            .expect("the parent's bootstrap runs in the throwaway tree");
            restore_regen_convergence_journal_for_subject(&throwaway, &fixture_subject(&throwaway))
                .expect("the old child's startup restore runs");
            assert!(
                read(&throwaway_hand).contains("head hand-maintained"),
                "the startup restore no longer moves the tree off what the parent compiled, so the \
                 handoff rule below is dead code and should be deleted with this arm"
            );
            assert!(
                !regen_convergence_journal_path(&throwaway).exists(),
                "the startup restore leaves a journal behind, so the teardown hazard is gone and \
                 the handoff rule below is guarding nothing"
            );
            let _ = fs::remove_dir_all(&throwaway);
        }

        // ARM 2 — WHAT THE CHILD DOES NOW: nothing. The carried tree and the journal the teardown
        // reads are exactly what the parent left.
        assert!(
            read(&hand_file).contains("base hand-maintained")
                && read(&build_script).contains("base build identity source"),
            "the child's arrival moved the tree it was exec'd into"
        );
        assert!(
            journal.exists(),
            "the child's arrival destroyed the journal the round's teardown restores from"
        );
        // THE CHILD'S RECEIPT ARM MATCHES WHAT THE PARENT SEEDED. The child seeded nothing under the
        // handoff rule, so its arm is DERIVED from the same authority; a `None`, or a different
        // count, would send the renderer down its `SeedBuiltFromHead` arm and print a receipt
        // claiming round 1 compiled the HEAD — the opposite of what the heal round records, in the
        // one image that reports.
        let child_arm = round_one_receipt_base_authority(&root, Some(base_commit.as_str()))
            .expect("the child can ask the same read-only authority the parent used");
        assert_eq!(
            (child_arm.0.as_str(), child_arm.1, child_arm.2),
            (
                base_commit.as_str(),
                restored.paths_restored,
                restored.paths_absent_at_base
            ),
            "the child's receipt arm disagrees with what the parent seeded, so the receipt the \
             child renders names the wrong bytes for round 1"
        );
        assert_eq!(
            restore_regen_convergence_journal_for_subject(&root, &subject)
                .expect("the round's teardown restores what the parent seeded")
                .rewritten
                .len(),
            restored.paths_rewritten,
            "the teardown did not return every path the parent seed moved ({}), so the child touched \
             the tree or the journal",
            restored.paths_rewritten
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE CHILD'S RECEIPT ARM HAS NO FALLBACK PATH. An unresolvable base commit refuses with the
    /// typed diagnostic; it does not yield `None`, which is what let the rendered receipt fall to the
    /// head arm and the struct field drop the commit (review 88497, second finding).
    #[test]
    fn the_childs_receipt_arm_refuses_rather_than_dropping_the_line() {
        let (root, _stage0, _base_commit) = base_mirror_fixture();
        let error = round_one_receipt_base_authority(&root, Some("   "))
            .expect_err("a base commit that is not a commit id must refuse the round");
        assert!(
            error.starts_with("BootstrapBaseMirrorCommitUnresolved"),
            "the child's receipt arm refused with an untyped diagnostic: {error}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE AUTHORITY IS NEVER A FALLBACK. If the journal cannot be written, the round refuses BEFORE
    /// touching a file — a partial restore is what produced the 26-error `v1-stage0-std-core` refusal
    /// in heal run 40739785316, where some files held base bytes and others held head bytes.
    #[test]
    fn a_refusal_before_the_bootstrap_moves_no_bytes() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        // Make the per-subject journal unwritable: the directory the journal lives in is a plain
        // file, so neither the directory nor the per-subject file can be created.
        let journal_root = root
            .join(REGEN_CONVERGENCE_JOURNAL_REL)
            .parent()
            .expect("the journal path has a parent")
            .to_path_buf();
        let _ = fs::remove_dir_all(&journal_root);
        fs::create_dir_all(&journal_root).expect("journal root");
        fs::write(
            journal_root.join("regen-convergence-journal"),
            "not a directory\n",
        )
        .expect("block the journal");
        let error = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect_err("an unwritable journal must refuse the round, not half-seed it");
        assert!(
            error.starts_with("BootstrapBaseMirrorJournal"),
            "journal refusal: {error}"
        );
        assert!(
            read(&stage0.join("cli_run").join("fixture_partition.rs"))
                .contains("variant `Dag` constructed"),
            "a refusal still moved bytes"
        );
        assert!(
            read(&stage0.join("fixture_emitted.rs")).contains("refused for 'Nat'"),
            "a refusal still moved bytes"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE DEFECT, DISCRIMINATED: at the PR head the mirror the seed compiles carries the
    /// refusal (that is the state run 40064940799 built), and after the bootstrap the same
    /// position holds the BASE commit's accepted bytes. Nothing else about the round moves.
    #[test]
    fn round_one_compiles_the_base_commit_and_not_the_heads_refusal() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        let mirror = stage0.join("fixture_emitted.rs");
        assert!(
            read(&mirror).contains("refused for 'Nat'"),
            "the fixture's head must be the state that deadlocked: the head's own mirror"
        );
        let restored = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect("the round-1 bootstrap runs");
        assert_eq!(restored.base_commit, base_commit);
        // THE WHOLE CRATE, NESTED INCLUDED: base's bytes for the top-level mirror AND for the
        // nested partition. `paths_restored` counts every crate path base has that the worktree
        // has: all five the fixture commits (src/fixture_emitted.rs, src/std_logic.rs,
        // src/bin/fixture_main.rs, src/cli_run/fixture_partition.rs, build.rs) of the seven the
        // worktree tracks — not the four top-level `src/*.rs` files the old population returned.
        assert_eq!(
            restored.paths_restored, 5,
            "the bootstrap claimed a different number of crate paths than base committed"
        );
        assert_eq!(
            restored.paths_absent_at_base, 2,
            "the head-added paths (a new mirror, a new hand-authored bin) were not counted \
             where a reader can see them"
        );
        assert!(
            read(&mirror).contains("base mirror bytes"),
            "round 1 still compiled the head's damaged mirror"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE AUTHORITY IS THE CRATE, NOT THE REPOSITORY AND NOT A HAND LIST. Round 1 builds base's
    /// bytes for every path under `src/v1/stage0` — generated mirrors, NESTED partitions, and the
    /// hand-maintained files inside the crate alike (root ruling: the whole crate, always base, never
    /// a fallback), because a base mirror beside a head hand-file is the chimera that stranded
    /// #15175 with E0599. What the authority does NOT reach is anything outside the crate: the
    /// `.dag` sources and the host shell are what the round REGENERATES from, so the pull request's
    /// own edits there survive the bootstrap untouched, and a path base never contained is kept.
    #[test]
    fn hand_maintained_files_and_directories_survive_the_bootstrap() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        bootstrap_round_one_from_base_mirror(&root, Some(&base_commit), &fixture_subject(&root))
            .expect("the round-1 bootstrap runs");
        // Inside the crate, base is the only thing round 1 compiles — hand-maintained files
        // included. The old filter left these at head, which is how a base mirror ended up beside a
        // head consumer of a variant only head declared.
        assert!(
            read(&stage0.join("std_logic.rs")).contains("base hand-maintained"),
            "a hand-maintained file inside the crate was left at head: {}",
            read(&stage0.join("std_logic.rs"))
        );
        assert!(
            read(&stage0.join("bin").join("fixture_main.rs")).contains("base hand-maintained main"),
            "a hand-maintained file in a nested hand directory was left at head: {}",
            read(&stage0.join("bin").join("fixture_main.rs"))
        );
        // Outside the crate, the pull request's own bytes are the authority and are never touched.
        assert!(
            read(&root.join("dag").join("fixture_authority.dag")).contains("head .dag authority"),
            "the bootstrap reached outside the crate it owns and reverted an authored authority"
        );
        // A path base has no form of is kept, not deleted: it is the PR's own new partition.
        assert!(
            read(&stage0.join("fixture_added_by_pr.rs")).contains("no base form"),
            "the bootstrap damaged a mirror the PR added"
        );
        // The population is every path base committed under the crate, keyed repo-relative, with no
        // hand-maintained exclusion and no depth filter.
        let (_, population) =
            resolve_base_mirror_authority(&root, Some(&base_commit)).expect("base resolves");
        let expected: BTreeSet<String> = [
            "src/v1/stage0/src/fixture_emitted.rs",
            "src/v1/stage0/src/std_logic.rs",
            "src/v1/stage0/src/bin/fixture_main.rs",
            "src/v1/stage0/src/cli_run/fixture_partition.rs",
            "src/v1/stage0/build.rs",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        assert_eq!(
            population.keys().cloned().collect::<BTreeSet<_>>(),
            expected,
            "the base population is not the whole crate the round must compile"
        );
        assert!(
            !population.contains_key("src/v1/stage0/src/fixture_added_by_pr.rs"),
            "base reported a population it never committed"
        );
        assert!(
            !population.contains_key("src/v1/stage0/src/bin/fixture_added_by_pr_main.rs"),
            "base reported a head-only path as its own"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE HEAD STATE IS RETURNED ON EVERY PATH, by the ONE journal the install path already
    /// uses. The snapshot holds the head's own bytes even though the bootstrap wrote base bytes
    /// first, so no base byte can reach an install, a candidate, or a publication.
    #[test]
    fn the_one_journal_snapshots_head_and_returns_it_after_the_bootstrap() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        let subject = fixture_subject(&root);
        bootstrap_round_one_from_base_mirror(&root, Some(&base_commit), &subject)
            .expect("the round-1 bootstrap runs");
        assert!(read(&stage0.join("fixture_emitted.rs")).contains("base mirror bytes"));
        restore_regen_convergence_journal_for_subject(&root, &subject)
            .expect("the same restore the install path runs");
        assert!(
            read(&stage0.join("fixture_emitted.rs")).contains("refused for 'Nat'"),
            "the journal snapshotted base bytes instead of the head's, so a round could publish them"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// EVERY MISSING OR MALFORMED AUTHORITY IS A TYPED, LOCATED REFUSAL, and none of them writes a
    /// byte. Falling back to the head mirror when base could not be read would be the forbidden
    /// second path wearing a different name, so the round refuses instead.
    #[test]
    fn every_unusable_base_authority_refuses_typed_and_writes_nothing() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        let subject = fixture_subject(&root);
        let head_bytes = read(&stage0.join("fixture_emitted.rs"));
        let cases: [(&str, Option<&str>, &str); 5] = [
            ("unset", None, "BootstrapBaseMirrorCommitUnresolved"),
            ("empty", Some(""), "BootstrapBaseMirrorCommitUnresolved"),
            (
                "whitespace",
                Some("   "),
                "BootstrapBaseMirrorCommitUnresolved",
            ),
            // A BRANCH NAME is not an authority: it resolves to whatever the checkout holds under
            // it, which is a different commit from the one the receipt would go on naming.
            (
                "branch name",
                Some("main"),
                "BootstrapBaseMirrorCommitMalformed",
            ),
            (
                "abbreviated id",
                Some(&base_commit[..8]),
                "BootstrapBaseMirrorCommitMalformed",
            ),
        ];
        for (cause, value, refusal) in cases {
            let error = bootstrap_round_one_from_base_mirror(&root, value, &subject)
                .expect_err(format!("{cause} must refuse").as_str());
            assert!(
                error.starts_with(refusal),
                "{cause}: expected {refusal}, got {error}"
            );
            assert!(
                error.contains(REGEN_ROUND_COST_BASE_MIRROR_ENV),
                "{cause}: the refusal does not name the env it refused: {error}"
            );
            assert_eq!(
                read(&stage0.join("fixture_emitted.rs")),
                head_bytes,
                "{cause}: a refused bootstrap still moved bytes"
            );
        }
        // UNSET and EMPTY are distinguishable, so a mis-wired step is not read as an empty string.
        let unset =
            bootstrap_round_one_from_base_mirror(&root, None, &subject).expect_err("unset refuses");
        let empty = bootstrap_round_one_from_base_mirror(&root, Some(""), &subject)
            .expect_err("empty refuses");
        assert!(
            unset.contains("UNSET") && !empty.contains("UNSET") && empty.contains("EMPTY"),
            "the two causes collapsed: {unset} | {empty}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// A WELL-FORMED COMMIT THE CHECKOUT DOES NOT CONTAIN REFUSES, and does not quietly resolve to
    /// the commit that branch, tag, or remote-tracking name happens to point at. This is the drift
    /// the reviewer named: a local `main` behind `origin/main` is a DIFFERENT AUTHORITY, and the
    /// round must refuse rather than seed from whichever one exists and call it the base.
    #[test]
    fn a_commit_the_checkout_lacks_refuses_rather_than_re_resolving_a_name() {
        let (root, stage0, _) = base_mirror_fixture();
        let subject = fixture_subject(&root);
        // A real 40-hex id that this repository has never contained.
        let foreign = "0000000000000000000000000000000000000000";
        let error = bootstrap_round_one_from_base_mirror(&root, Some(foreign), &subject)
            .expect_err("an absent commit must refuse");
        assert!(
            error.starts_with("BootstrapBaseMirrorCommitUnreachable"),
            "unreachable refusal: {error}"
        );
        assert_eq!(
            read(&stage0.join("fixture_emitted.rs")),
            "// head mirror bytes -- shared layer refused for 'Nat'\n",
            "a refusal still moved bytes"
        );
        // A name that DOES exist is still refused: existing does not make it the authority.
        let named = Command::new("git")
            .args(["branch", "stale-base", "main"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            named.status.success(),
            "fixture branch: {}",
            String::from_utf8_lossy(&named.stderr)
        );
        let error = bootstrap_round_one_from_base_mirror(&root, Some("stale-base"), &subject)
            .expect_err("a name is never the authority, stale or not");
        assert!(
            error.starts_with("BootstrapBaseMirrorCommitMalformed"),
            "name-as-authority must be malformed: {error}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE BOOTSTRAP IS IDEMPOTENT and the journal keeps the HEAD snapshot across repeats, so a
    /// re-executed image cannot turn the rollback into a base-byte restore.
    #[test]
    fn a_second_bootstrap_is_the_same_authority_and_keeps_the_head_snapshot() {
        let (root, stage0, base_commit) = base_mirror_fixture();
        let subject = fixture_subject(&root);
        let first = bootstrap_round_one_from_base_mirror(&root, Some(&base_commit), &subject)
            .expect("first");
        let seeded = read(&stage0.join("fixture_emitted.rs"));
        let second = bootstrap_round_one_from_base_mirror(&root, Some(&base_commit), &subject)
            .expect("second");
        assert_eq!(first.base_commit, second.base_commit);
        assert_eq!(second.base_commit, base_commit);
        assert_eq!(read(&stage0.join("fixture_emitted.rs")), seeded);
        restore_regen_convergence_journal_for_subject(&root, &subject).expect("restore");
        assert!(
            read(&stage0.join("fixture_emitted.rs")).contains("refused for 'Nat'"),
            "a repeat bootstrap replaced the head snapshot with base bytes"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE REAL CI SHAPE: heal checks out the PR HEAD commit and the base needs no branch name at
    /// all. The step hands over `base.sha`, the object is present in the `fetch-depth: 0`
    /// checkout, and the round seeds from it — while a name that would resolve elsewhere is
    /// refused outright. This is what makes the commit (not the name) the right thing to bind.
    #[test]
    fn a_head_only_checkout_seeds_from_the_base_commit_with_no_branch_name() {
        let (root, _, base_commit) = base_mirror_fixture();
        // A clone that contains BOTH commits but names only the PR branch: no local `main`, and
        // the base's remote-tracking name deleted, exactly as a head-sha checkout can present.
        let clone = std::env::temp_dir().join(format!(
            "{}-head-only",
            root.file_name().unwrap().to_str().unwrap()
        ));
        let output = Command::new("git")
            .args([
                "clone",
                "-q",
                "--no-checkout",
                &root.to_string_lossy(),
                &clone.to_string_lossy(),
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "clone: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(&clone)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        git(&["checkout", "-q", "pr"]);
        // No NAME for the base anywhere: neither a local branch nor a remote-tracking ref.
        git(&["update-ref", "-d", "refs/heads/main"]);
        git(&["update-ref", "-d", "refs/remotes/origin/main"]);
        let cloned_stage0 = clone.join(STAGE0_SRC_REL);
        assert!(
            read(&cloned_stage0.join("fixture_emitted.rs")).contains("refused for 'Nat'"),
            "the clone must sit on the damaged head"
        );
        // NO NAME RESOLVES ANYWHERE, yet the commit is an object in the checkout.
        let name_error = Command::new("git")
            .args(["rev-parse", "--verify", "main"])
            .current_dir(&clone)
            .output()
            .unwrap();
        assert!(
            !name_error.status.success(),
            "the fixture must have no base NAME to fall back to"
        );
        let subject = fixture_subject(&clone);
        let restored = bootstrap_round_one_from_base_mirror(&clone, Some(&base_commit), &subject)
            .expect("the base COMMIT is enough; no name is consulted");
        assert_eq!(restored.base_commit, base_commit);
        assert!(
            read(&cloned_stage0.join("fixture_emitted.rs")).contains("base mirror bytes"),
            "the head's damaged mirror survived a base-commit bootstrap"
        );
        // THE NESTED FILE TOO: a head-only checkout is the CI shape, and the nested partition is
        // where the #15175 chimera lived. If the population were still the top-level name list,
        // this assert is the one that goes red.
        assert!(
            read(&cloned_stage0.join("cli_run").join("fixture_partition.rs"))
                .contains("defines MirrorRow::Dag"),
            "a head-only checkout left the nested partition at the head's damaged bytes"
        );
        let _ = fs::remove_dir_all(&clone);
        let _ = fs::remove_dir_all(&root);
    }

    /// THE RECEIPT'S HOST ARM CARRIES THE COMMIT AND BOTH COUNTS. This is the host half only, and
    /// it is deliberately corpus-free: rendering the receipt here would resolve its `.dag` renderer
    /// through the interpreter, which loads the whole corpus per call (about 50 minutes of libtest
    /// time on srv1 at `b339f6fa0e6f`). The renderer's own arms — the line's text, and its absence
    /// for a round never asked to bootstrap — are
    /// `dag/test/claim/self_host_regen_round_cost_witness.dag`
    /// `a_bootstrapped_receipt_names_the_base_commit_after_the_header` and
    /// `a_non_bootstrapped_receipt_renders_no_base_mirror_line`.
    #[test]
    fn the_receipts_host_arm_carries_the_commit_and_both_counts() {
        let (root, _stage0, base_commit) = base_mirror_fixture();
        let restored = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect("the bootstrap runs");
        let (commit, seeded, head_added, head_deleted) = base_mirror_receipt_authority(&restored);
        assert_eq!(
            commit,
            base_commit.as_str(),
            "the receipt would name something other than the commit the step named, so the \
             round's stated source is not the authority it actually used"
        );
        assert_eq!(
            (seeded, head_added, head_deleted),
            (5, 2, 0),
            "the receipt's counts moved off the fixture's population, so the line it feeds no \
             longer describes what the seed built. The third count is the new one: this fixture has \
             no head-deleted path, so it must read zero rather than borrow another fixture's number."
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// THE PATH-SET DELTA IS IN THE RECEIPT, not hidden in the writer's arithmetic. The #15418 heal
    /// log printed `paths=300` while the base commit it named tracked 303 crate paths, and nothing in
    /// that line separated the three paths the head had deleted from paths that were never in base at
    /// all -- the two cases print identically until a count of the deletions exists next to the count
    /// of head additions. The fixture's population is exact, so the row is checked against it.
    #[test]
    fn the_receipt_names_the_paths_the_head_deleted() {
        let (root, _stage0, base_commit) = base_mirror_deletion_fixture();
        let restored = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect("the bootstrap runs");
        let (commit, seeded, head_added, head_deleted) = base_mirror_receipt_authority(&restored);
        assert_eq!(
            commit,
            base_commit.as_str(),
            "the receipt would name something other than the commit the step named"
        );
        assert_eq!(
            (seeded, head_added, head_deleted),
            (7, 2, 1),
            "base tracks seven crate paths here, the head tracks eight of which two are new and one \
             is base-only; a receipt that does not carry that delta cannot distinguish a deleted \
             module from a path base never had"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// AN UNTRACKED FILE AT A BASE-ONLY PATH REFUSES THE ROUND AND MOVES NOTHING. The path is in
    /// base's population, absent from the head's tracked set, and PRESENT on disk. The round cannot
    /// tell that from a head deletion whose bytes some other process left behind, and both readings
    /// are destructive in different ways: overwriting the file clobbers bytes the head never tracked,
    /// deleting it clobbers them by inference. So the round refuses, typed and located, before it
    /// journals or writes anything -- which is also why the refusal carries both commit ids: the
    /// reader has to be able to see which side of the delta the path sits on.
    #[test]
    fn an_untracked_file_at_a_base_only_path_refuses_and_moves_nothing() {
        let (root, stage0, base_commit) = base_mirror_deletion_fixture();
        let retired = stage0.join("cli_run").join("fixture_retired.rs");
        let untracked_bytes = "// not base's bytes, and not tracked by either commit\n";
        fs::write(&retired, untracked_bytes).expect("plant the untracked file");
        let before = mtimes_under(&root.join(STAGE0_CRATE_REL));
        let error = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect_err("a base-only path present on disk as an untracked file must refuse the round");
        assert!(
            error.contains("BootstrapBaseMirrorUntrackedCollision")
                && error.contains("cli_run/fixture_retired.rs"),
            "the refusal is not the typed, located one: {error}"
        );
        assert!(
            error.contains(&base_commit) && error.contains(&git_head_sha(&root).expect("head sha")),
            "the refusal names the path but not the two commits whose delta put it on the \
             deleted side: {error}"
        );
        assert_eq!(
            read(&retired),
            untracked_bytes,
            "the round overwrote a file the head does not track"
        );
        assert_eq!(
            mtimes_under(&root.join(STAGE0_CRATE_REL)),
            before,
            "the round moved a file before refusing"
        );
        assert!(
            !regen_convergence_journal_path(&root).exists(),
            "the round journalled before refusing, so a later round would restore a state that \
             never existed"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// CLEANUP AFTER A FORCED FAILURE. `std_logic.rs` is made unwritable, so the bootstrap writes
    /// some base bytes and then refuses mid-round: by that point the head-deleted module is back on
    /// disk, which is exactly the state a failure leaves. Every failure arm of the round ends in the
    /// same journal restore this test calls, so the assertion is about the journal's lifecycle and
    /// not about one error arm: absence comes back, and the head's own bytes come back for every file
    /// the round managed to rewrite. The file that refused is never touched by the restore either --
    /// it still holds the head's bytes, and the mode the fixture gave it survives.
    #[test]
    fn a_forced_failure_after_the_bootstrap_restores_absence_and_the_heads_bytes() {
        use std::os::unix::fs::PermissionsExt;
        let (root, stage0, base_commit) = base_mirror_deletion_fixture();
        let retired = stage0.join("cli_run").join("fixture_retired.rs");
        let rewritten = stage0.join("cli_run").join("fixture_partition.rs");
        let std_logic = stage0.join("std_logic.rs");
        let head_partition = read(&rewritten);
        let head_std_logic = read(&std_logic);
        fs::set_permissions(&std_logic, fs::Permissions::from_mode(0o444))
            .expect("make one base-tracked file unwritable");

        let error = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect_err("an unwritable base-tracked file must refuse the bootstrap");
        // THE REFUSAL IS LOCATED, NOT TYPED: the write arm spells `write {path}: {cause}` (an existing
        // property of this function, not widened or narrowed by this change), so the assertion names
        // the path and the cause rather than a prefix that does not exist.
        assert!(
            error.contains(&format!("write {}:", std_logic.display()))
                && error.contains("Permission denied"),
            "the refusal is not the located write refusal this fixture forces: {error}"
        );
        assert!(
            retired.is_file(),
            "the fixture did not reach the state a mid-round failure leaves, so the cleanup below \
             would be asserting nothing: {error}"
        );

        let put_back_paths =
            restore_regen_convergence_journal_for_subject(&root, &fixture_subject(&root))
                .expect("the failure path restores the journal");
        assert!(
            !retired.exists(),
            "a module the head deleted survived a FAILED round into the tree the next stage walks: \
             {put_back_paths:?}"
        );
        assert_eq!(
            read(&rewritten),
            head_partition,
            "a file the round rewrote before failing did not come back to the head's bytes"
        );
        assert_eq!(
            read(&std_logic),
            head_std_logic,
            "the file whose write refused was changed anyway"
        );
        assert_eq!(
            fs::metadata(&std_logic)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o222,
            0,
            "the restore wrote through a mode the head had, so the next round sees a tree the head \
             never had"
        );
        assert!(
            put_back_paths
                .removed
                .iter()
                .any(|path| path.ends_with("/cli_run/fixture_retired.rs")),
            "the restore did not report the resurrected path among the paths it REMOVED: \
             {put_back_paths:?}"
        );
        assert!(
            put_back_paths
                .rewritten
                .iter()
                .any(|path| path.ends_with("/cli_run/fixture_partition.rs")),
            "the file the round rewrote before failing is not among the paths the restore put \
             back: {put_back_paths:?}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// INTERRUPTION AND RECOVERY. A process that dies after the bootstrap and before the round's own
    /// teardown leaves the journal behind, and the next round begins by restoring it -- the head of
    /// the round body calls `restore_regen_convergence_journal` before anything else. That recovery
    /// can only match if the bootstrap left the journal's subject re-derivable, and it does: the
    /// subject is the starting commit, the digest of the `.dag` sources under the entry root, and the
    /// digest of the transaction plan, none of which a base-mirror bootstrap touches, because it
    /// writes only inside `src/v1/stage0`. This test calls the subject-carrying form because the
    /// fixture's subject is fixed rather than computed; the production call is the same function with
    /// the subject the next round would carry.
    #[test]
    fn a_round_that_died_after_the_bootstrap_leaves_a_journal_the_next_round_can_recover() {
        let (root, stage0, base_commit) = base_mirror_deletion_fixture();
        let retired = stage0.join("cli_run").join("fixture_retired.rs");
        bootstrap_round_one_from_base_mirror(&root, Some(&base_commit), &fixture_subject(&root))
            .expect("the bootstrap runs");
        assert!(
            retired.is_file(),
            "the round did not reach the state a crash would interrupt"
        );
        assert!(
            regen_convergence_journal_path(&root).exists(),
            "no journal was left behind, so there is no interruption to recover from"
        );

        let put_back_paths =
            restore_regen_convergence_journal_for_subject(&root, &fixture_subject(&root))
                .expect("the next round's head restores the stale journal");
        assert!(
            !retired.exists(),
            "the resurrected module survived the interruption into the tree the next round builds: \
             {put_back_paths:?}"
        );
        assert!(
            !regen_convergence_journal_path(&root).exists(),
            "the recovery left its journal behind, so a third round would restore a state that no \
             longer exists"
        );
        let _ = fs::remove_dir_all(&root);
    }
    /// Every mtime under the crate, for the control that asserts the seed and the restore leave an
    /// untouched tree alone. `.git` is skipped: the round's own `git show` calls do not write there,
    /// and a control that watched it would be measuring git, not the writer.
    fn mtimes_under(dir: &Path) -> BTreeMap<PathBuf, std::time::SystemTime> {
        let mut out = BTreeMap::new();
        let mut walk = std::collections::VecDeque::new();
        walk.push_back(dir.to_path_buf());
        while let Some(dir) = walk.pop_front() {
            for entry in
                fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
            {
                let path = entry.unwrap().path();
                if path.file_name() == Some(std::ffi::OsStr::new(".git")) {
                    continue;
                }
                if path.is_dir() {
                    walk.push_back(path);
                } else {
                    out.insert(
                        path.clone(),
                        fs::metadata(&path)
                            .unwrap_or_else(|e| panic!("stat {}: {e}", path.display()))
                            .modified()
                            .unwrap_or_else(|e| panic!("mtime {}: {e}", path.display())),
                    );
                }
            }
        }
        out
    }

    /// THE CONTROL FOR THE #14876 / #15310 SPECIMENS: a PR that touches no stage0 file must not move
    /// a single mtime, in the seed or in the restore.
    ///
    /// Round 1 seeds base's bytes over the whole base crate population because it cannot know which
    /// paths the PR edited — and for a `.dag`-only PR that population is 306 paths of byte-identical
    /// content. Writing identical bytes is not a no-op: it bumps the mtime, and cargo fingerprints a
    /// path dependency by mtime, so the seed build and then the measured partition rebuild recompile
    /// crates the round's own derived scope had EXCLUDED. The scope guard refuses correctly; the
    /// writer was doing redundant work (DESIGN §2). Heal run 41061363628 / job 124081920078 on #14876
    /// (`compiled excluded packages ["v1-stage0-runtime"]`, closure `[std-core, v1-infer, std-surface,
    /// extdeps-languages, v1-artifact, emit-core, v1-compiler]`) and run 41098380134 on #15310
    /// (six excluded partition packages, `changed_mirrors=[v1_compiler_emit_rust.rs]`, closure
    /// `[v1-compiler]`) are the two specimens.
    ///
    /// The second half is the DISCRIMINATOR: the same measurement on a tree that DOES differ must
    /// show the differing paths' mtimes move and the identical path's stay put. Without it, a fixture
    /// that never wrote anything, or a filesystem with a coarse clock, would pass the first half for
    /// the wrong reason.
    #[test]
    fn a_stage0_untouched_pr_moves_no_mtime_in_the_seed_or_the_restore() {
        // ARM 1 — THE CI SHAPE THAT WAS REFUSED: the PR edits only `.dag` authorities, so every crate
        // path in the worktree already holds base's bytes.
        let (root, _stage0, base_commit) = base_mirror_fixture();
        let checkout = Command::new("git")
            .args(["checkout", &base_commit, "--", STAGE0_CRATE_REL])
            .current_dir(&root)
            .output()
            .expect("git checkout of the crate from base");
        assert!(
            checkout.status.success(),
            "the fixture could not reset the crate to base's bytes: {}",
            String::from_utf8_lossy(&checkout.stderr)
        );
        let crate_dir = root.join(STAGE0_CRATE_REL);
        let before = mtimes_under(&crate_dir);
        assert!(!before.is_empty(), "the fixture's crate is empty");

        let restored = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect("the round-1 bootstrap runs");
        assert_eq!(
            (restored.paths_restored, restored.paths_rewritten),
            (5, 0),
            "a tree identical to base must hold the whole population as base's bytes (5) while the \
             seed writes NONE of them; the seed wrote {}, which is the mtime churn that drags \
             excluded crates into the rebuild",
            restored.paths_rewritten
        );
        assert_eq!(
            restored.paths_absent_at_base, 2,
            "the head-added paths are still head-added, and the install journals them when it \
             overwrites them — the seed must not journal them"
        );
        assert_eq!(
            mtimes_under(&crate_dir),
            before,
            "the seed rewrote a path whose bytes already equalled base's"
        );

        // The restore of a journal that moved nothing must also touch nothing.
        assert_eq!(
            restore_regen_convergence_journal_for_subject(&root, &fixture_subject(&root))
                .expect("the round's teardown runs")
                .rewritten
                .len(),
            0,
            "the teardown wrote back paths the seed never moved"
        );
        assert_eq!(
            mtimes_under(&crate_dir),
            before,
            "the restore rewrote a path it had never moved"
        );
        let _ = fs::remove_dir_all(&root);

        // ARM 2 — THE DISCRIMINATOR: on a tree that differs, the differing paths' mtimes MUST move
        // and the one path committed identically on both sides (the crate's build script) must not.
        let (root, stage0, base_commit) = base_mirror_fixture();
        let crate_dir = root.join(STAGE0_CRATE_REL);
        let build_script = crate_dir.join("build.rs");
        let emitted = stage0.join("fixture_emitted.rs");
        let before = mtimes_under(&crate_dir);
        let restored = bootstrap_round_one_from_base_mirror(
            &root,
            Some(&base_commit),
            &fixture_subject(&root),
        )
        .expect("the round-1 bootstrap runs on a differing tree");
        assert_eq!(
            restored.paths_rewritten, 4,
            "the fixture's head edits 4 crate paths"
        );
        let after = mtimes_under(&crate_dir);
        assert_ne!(
            after[&emitted], before[&emitted],
            "a path whose bytes differ was not rewritten, so arm 1 above proves nothing"
        );
        assert_eq!(
            after[&build_script], before[&build_script],
            "the seed rewrote the one crate path committed identically on both sides"
        );
        let _ = fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod regen_round_cost_tests {
    use super::*;

    fn handoff_subject() -> RegenConvergenceCheckpointSubject {
        RegenConvergenceCheckpointSubject {
            starting_commit: "c0ffee".to_string(),
            source_authority_digest: "sha256:source".to_string(),
            stage_plan_authority_digest: "sha256:plan".to_string(),
        }
    }

    fn cost(crates: u64, wall: u64, cpu: Option<u64>) -> SeedBuildCost {
        SeedBuildCost {
            compiled_crates: crates,
            wall_ms: wall,
            cpu_ms: cpu,
        }
    }

    fn row(label: &str, wall: u64, cpu: Option<u64>) -> v1_rt::TraceLedgerRow {
        v1_rt::TraceLedgerRow {
            label: label.to_string(),
            wall_ms: wall,
            cpu_ms: cpu,
        }
    }

    /// P1 discriminator: a supplied nonzero pre-handoff cost and crate count survive the
    /// production composition and the production render, once.
    #[test]
    fn pre_handoff_seed_build_cost_is_composed_exactly_once_into_the_receipt() {
        let carried = cost(655, 180_000, Some(170_000));
        let marks = vec![
            row("round.seed_build", 96, Some(100)),
            row("corpus_load", 7, Some(7)),
        ];
        let (crates, composed) = compose_seed_build_cost(Some(&carried), 0, marks.clone()).unwrap();
        assert_eq!(crates, 655);
        assert_eq!(composed[0].wall_ms, 180_096);
        assert_eq!(composed[0].cpu_ms, Some(170_100));
        assert_eq!(composed[1].wall_ms, 7, "other phases are untouched");
        // no carried cost: identity (no omission invented, nothing added)
        let (c0, m0) = compose_seed_build_cost(None, 3, marks.clone()).unwrap();
        assert_eq!((c0, m0[0].wall_ms), (3, 96));
        // unreadable carried CPU is unreadable, never zero
        let (_, un) = compose_seed_build_cost(Some(&cost(1, 1, None)), 0, marks.clone()).unwrap();
        assert_eq!(un[0].cpu_ms, None);
        // zero rows: the carried duration would vanish while the crates stayed
        assert!(matches!(
            compose_seed_build_cost(Some(&carried), 0, vec![row("corpus_load", 7, Some(7))]),
            Err(SeedHandoffRefusal::SeedBuildMeasurementCount { found: 0 })
        ));
        // two rows: the carried duration would be counted twice
        let two = vec![
            row("round.seed_build", 96, Some(1)),
            row("round.seed_build", 4, Some(1)),
        ];
        assert!(matches!(
            compose_seed_build_cost(Some(&carried), 0, two),
            Err(SeedHandoffRefusal::SeedBuildMeasurementCount { found: 2 })
        ));

        let workspace = workspace_root();
        let roots = vec![
            workspace.join("dag").display().to_string(),
            workspace.join("src/v2").display().to_string(),
        ];
        let rendered = render_round_cost_receipt(
            &roots,
            "h",
            "t",
            false,
            crates,
            0,
            0,
            &composed,
            &[],
            &[],
            &[],
            &[],
            &[],
            "e",
            "c",
            // A probe round: it was never asked to bootstrap from a base mirror, so the receipt
            // carries the head authority and renders no base-mirror line at all.
            None,
        )
        .expect("the production renderer accepts the composed rows");
        assert!(
            rendered.contains("seed_build_compiled_crates=655"),
            "{rendered}"
        );
        assert!(
            rendered.contains("phase=seed_build wall_ms=180096"),
            "{rendered}"
        );
        assert_eq!(rendered.matches("seed_build_compiled_crates=").count(), 1);
    }

    #[test]
    fn source_identity_sees_untracked_files() {
        let mut none: Vec<(Vec<u8>, Vec<u8>)> = vec![];
        let base = compose_source_identity("H", b"d", &mut none);
        let mut added = vec![(b"new.dag".to_vec(), b"x".to_vec())];
        let with_new = compose_source_identity("H", b"d", &mut added);
        let mut edited = vec![(b"new.dag".to_vec(), b"y".to_vec())];
        let edited_id = compose_source_identity("H", b"d", &mut edited);
        assert_ne!(
            base, with_new,
            "an added untracked file changes the identity"
        );
        assert_ne!(
            with_new, edited_id,
            "an edited untracked file changes the identity"
        );
        let c = cost(1, 1, None);
        let carry =
            match decide_seed_handoff(None, "a", "b", &with_new, &c, Some(&handoff_subject()))
                .unwrap()
            {
                SeedHandoffDecision::ReExec(h) => h,
                SeedHandoffDecision::Proceed => panic!(),
            };
        assert!(matches!(
            decide_seed_handoff(
                Some(&carry),
                "b",
                "b",
                &edited_id,
                &c,
                Some(&handoff_subject())
            ),
            Err(SeedHandoffRefusal::SourceChangedAcrossHandoff { .. })
        ));
    }

    #[test]
    fn untracked_framing_distinguishes_ambiguous_populations() {
        let c = cost(1, 1, None);
        // old framing: path '=' digest '\n'. Two files vs one file whose NAME embeds the
        // first file's entry.
        let mut two = vec![
            (b"a.dag".to_vec(), b"x".to_vec()),
            (b"b.dag".to_vec(), b"y".to_vec()),
        ];
        let forged = format!("a.dag={}\nb.dag", bytes_digest(b"x")).into_bytes();
        let mut one = vec![(forged, b"y".to_vec())];
        let id_two = compose_source_identity("H", b"d", &mut two);
        let id_one = compose_source_identity("H", b"d", &mut one);
        assert_ne!(id_two, id_one);
        let carry = match decide_seed_handoff(None, "a", "b", &id_two, &c, Some(&handoff_subject()))
            .unwrap()
        {
            SeedHandoffDecision::ReExec(h) => h,
            SeedHandoffDecision::Proceed => panic!(),
        };
        assert!(matches!(
            decide_seed_handoff(
                Some(&carry),
                "b",
                "b",
                &id_one,
                &c,
                Some(&handoff_subject())
            ),
            Err(SeedHandoffRefusal::SourceChangedAcrossHandoff { .. })
        ));
        assert!(matches!(
            decide_seed_handoff(
                Some(&carry),
                "b",
                "b",
                &id_two,
                &c,
                Some(&handoff_subject())
            ),
            Ok(SeedHandoffDecision::Proceed)
        ));
    }

    #[test]
    fn exec_target_digest_is_of_the_opened_inode_not_the_path() {
        let dir = std::env::temp_dir().join(format!("rrc-exec-target-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("img");
        fs::write(&path, b"built").unwrap();
        let (file, digest) = open_exec_target(&path).unwrap();
        // the path is replaced after the open: the descriptor still names the hashed bytes
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"replaced").unwrap();
        use std::io::{Read, Seek};
        let mut f = file;
        f.rewind().unwrap();
        let mut got = std::vec::Vec::new();
        f.read_to_end(&mut got).unwrap();
        assert_eq!(got, b"built");
        assert_eq!(digest, bytes_digest(b"built"));
        assert_ne!(digest, current_digest_of(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    fn current_digest_of(p: &Path) -> String {
        bytes_digest(&fs::read(p).unwrap())
    }

    #[test]
    fn exec_target_admission_compares_with_the_build_output_and_never_overwrites() {
        let c = cost(1, 1, None);
        // production decision: build output B is what the carry names
        let carry = match decide_seed_handoff(None, "old", "B", "S", &c, Some(&handoff_subject()))
            .unwrap()
        {
            SeedHandoffDecision::ReExec(h) => h,
            SeedHandoffDecision::Proceed => panic!(),
        };
        assert_eq!(carry.built_exe, "B");
        // build=B, install replaced with C before the open: opened=C refuses
        assert!(matches!(
            admit_exec_target(carry.clone(), "C"),
            Err(SeedHandoffRefusal::OpenedTargetNotBuildOutput { .. })
        ));
        // build=B, opened=B (a byte-identical replacement included): proceeds, carry intact
        let admitted = admit_exec_target(carry.clone(), "B").unwrap();
        assert_eq!(admitted, carry);
        // and the receiver still proceeds on the admitted carry
        assert!(matches!(
            decide_seed_handoff(Some(&admitted), "B", "B", "S", &c, Some(&handoff_subject())),
            Ok(SeedHandoffDecision::Proceed)
        ));
    }

    #[test]
    fn running_image_is_read_from_proc_self_exe_not_the_installed_path() {
        let direct = bytes_digest(&fs::read("/proc/self/exe").unwrap());
        assert_eq!(running_image_digest().unwrap(), direct);
    }

    #[test]
    fn modify_after_verify_and_installed_running_mismatch_refuse() {
        let c = cost(1, 1, None);
        // the first image hashed inode I = B and carried it; another writer then rewrote I
        let dir = std::env::temp_dir().join(format!("rrc-mav-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("img");
        fs::write(&path, b"B").unwrap();
        let (_target, built) = open_exec_target(&path).unwrap();
        let carry = SeedHandoff {
            cost: c.clone(),
            source: "S".into(),
            built_exe: built.clone(),
            round_subject: Some(handoff_subject()),
        };
        // rewrite the same inode, then put B back at the installed path
        let mut rewritten = fs::OpenOptions::new().write(true).open(&path).unwrap();
        use std::io::Write as _;
        rewritten.write_all(b"C").unwrap();
        let running = digest_open_file(&mut fs::File::open(&path).unwrap()).unwrap();
        assert_ne!(
            running, built,
            "the rewrite is visible through the running inode"
        );
        // receiver: running image C, installed path B again -> the old path-reopen said Proceed
        assert!(matches!(
            decide_seed_handoff(
                Some(&carry),
                &running,
                &built,
                "S",
                &c,
                Some(&handoff_subject())
            ),
            Err(SeedHandoffRefusal::WrongArtifactAcrossHandoff { .. })
        ));
        // installed/running mismatch with the carried artifact running
        assert!(matches!(
            decide_seed_handoff(
                Some(&carry),
                &built,
                &running,
                "S",
                &c,
                Some(&handoff_subject())
            ),
            Err(SeedHandoffRefusal::ExecutableReplacedAfterHandoff { .. })
        ));
        // same artifact everywhere proceeds
        assert!(matches!(
            decide_seed_handoff(
                Some(&carry),
                &built,
                &built,
                "S",
                &c,
                Some(&handoff_subject())
            ),
            Ok(SeedHandoffDecision::Proceed)
        ));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn handoff_decisions_are_typed_and_round_trip() {
        let c = cost(4, 5, Some(6));
        let carry =
            match decide_seed_handoff(None, "a", "b", "T", &c, Some(&handoff_subject())).unwrap() {
                SeedHandoffDecision::ReExec(h) => h,
                SeedHandoffDecision::Proceed => panic!("replaced exe must hand off"),
            };
        assert_eq!(carry.built_exe, "b", "the carry names the built artifact");
        assert_eq!(SeedHandoff::decode(&carry.encode()).unwrap(), carry);
        assert!(matches!(
            decide_seed_handoff(None, "a", "a", "T", &c, Some(&handoff_subject())),
            Ok(SeedHandoffDecision::Proceed)
        ));
        // positive: the re-exec'd image is the built artifact, same source, build is a no-op
        assert!(matches!(
            decide_seed_handoff(Some(&carry), "b", "b", "T", &c, Some(&handoff_subject())),
            Ok(SeedHandoffDecision::Proceed)
        ));
        assert!(matches!(
            decide_seed_handoff(Some(&carry), "b", "c", "T", &c, Some(&handoff_subject())),
            Err(SeedHandoffRefusal::ExecutableReplacedAfterHandoff { .. })
        ));
        // same commit, different uncommitted source
        assert!(matches!(
            decide_seed_handoff(
                Some(&carry),
                "b",
                "b",
                "T-other-diff",
                &c,
                Some(&handoff_subject())
            ),
            Err(SeedHandoffRefusal::SourceChangedAcrossHandoff { .. })
        ));
        // wrong artifact: running image is not the one the first image built
        assert!(matches!(
            decide_seed_handoff(Some(&carry), "z", "z", "T", &c, Some(&handoff_subject())),
            Err(SeedHandoffRefusal::WrongArtifactAcrossHandoff { .. })
        ));
        assert!(matches!(
            SeedHandoff::decode("garbage"),
            Err(SeedHandoffRefusal::HandoffUndecodable { .. })
        ));
    }

    #[test]
    fn assembled_seed_includes_foundation_and_layered_partitions_not_emit_consumer() {
        let workspace = workspace_root();
        let seed_binary_modules = super::super::emitted_closure_compile_host::binary_modules(
            &workspace.join("src/v1/stage0/src/lib.rs"),
        )
        .expect("the executable assembly manifest resolves")
        .into_iter()
        .collect::<BTreeSet<_>>();
        let rows =
            crate::gunbc_stage0_stage0_crate_partition_generated::generated_partition_crate_rows();
        let assembled = assembled_seed_modules(seed_binary_modules, rows.as_ref());
        assert!(assembled.contains("v1_rt"));
        assert!(assembled.contains("std_content_hash"));
        assert!(assembled.contains("std_measure"));
        assert!(assembled.contains("v1_compiler_infer_service"));
    }

    /// The process-tree clock reads on this host, and it is monotone across work.
    #[test]
    fn process_tree_cpu_reads_and_does_not_go_backwards() {
        let before = v1_rt::trace_process_tree_cpu_ms();
        let mut sink = 0u64;
        for i in 0..2_000_000u64 {
            sink = sink.wrapping_mul(31).wrapping_add(i);
        }
        assert_ne!(sink, 1);
        let after = v1_rt::trace_process_tree_cpu_ms();
        if let (Some(b), Some(a)) = (before, after) {
            assert!(a >= b, "cpu went backwards: {b} -> {a}");
        }
    }
}

#[cfg(test)]
mod regen_convergence_host_instrument_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    /// The fixture's root, removed when the guard drops -- on success AND on unwind, so a test
    /// that panics after creating its fixture leaves nothing behind in the runner's temp dir.
    struct FixtureRoot(PathBuf);

    impl Drop for FixtureRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    impl std::ops::Deref for FixtureRoot {
        type Target = Path;
        fn deref(&self) -> &Path {
            &self.0
        }
    }

    impl AsRef<Path> for FixtureRoot {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    fn fixture_workspace() -> (
        FixtureRoot,
        PathBuf,
        PathBuf,
        RegenConvergenceCheckpointSubject,
    ) {
        // THE FIXTURE OWNS A FRESH ROOT. Process id and counter alone repeat across runs (a
        // later process can draw the same id), so the name also carries the wall-clock nanos,
        // and the root is created with
        // `create_dir`: a root that already exists refuses here, located, rather than being
        // written into with a previous run's files and permissions still in it.
        let root = std::env::temp_dir().join(format!(
            "gunbc-regen-convergence-host-{}-{}-{}",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        fs::create_dir(&root)
            .unwrap_or_else(|e| panic!("fixture root {} must be fresh: {e}", root.display()));
        // Guard taken before any further fallible step, so a panic inside the fixture also cleans up.
        let root = FixtureRoot(root);
        let stage0 = root.join("src/v1/stage0/src");
        let candidate = root.join("candidate/src");
        fs::create_dir_all(&stage0).unwrap();
        fs::create_dir_all(&candidate).unwrap();
        for (name, bytes) in [
            ("fixture_producer.rs", "// old producer\n"),
            ("fixture_subject.rs", "// old subject\n"),
            ("fixture_dependent.rs", "// old dependent\n"),
            ("fixture_unplanned.rs", "// stable\n"),
        ] {
            fs::write(stage0.join(name), bytes).unwrap();
        }
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(&*root)
                .env("GIT_AUTHOR_NAME", "regen fixture")
                .env("GIT_AUTHOR_EMAIL", "regen@example.invalid")
                .env("GIT_COMMITTER_NAME", "regen fixture")
                .env("GIT_COMMITTER_EMAIL", "regen@example.invalid")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        write_fixture_stage_plan(&root);
        git(&["init", "-q", "--initial-branch=main"]);
        git(&["add", "."]);
        git(&["commit", "-qm", "fixture"]);
        let (starting_commit, stage_plan_authority_digest) =
            checkpoint_subject_live_facts(&root).expect("fixture commit and stage plan");
        let subject = RegenConvergenceCheckpointSubject {
            starting_commit,
            source_authority_digest: "fixture-authority".to_string(),
            stage_plan_authority_digest,
        };
        (root, stage0, candidate, subject)
    }

    fn fixture_manifest(
        candidate: &Path,
        rows: &[(&str, &str, &str)],
    ) -> (
        RegenCandidateManifest,
        HashMap<String, RegenCandidateManifestSurface>,
    ) {
        for entry in fs::read_dir(candidate).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                fs::remove_file(path).unwrap();
            }
        }
        for (path, _, bytes) in rows {
            fs::write(candidate.join(path), bytes).unwrap();
        }
        let mut surfaces = rows
            .iter()
            .map(|(path, module, _)| RegenCandidateManifestSurface {
                declaring_module: (*module).to_string(),
                projected_path: (*path).to_string(),
                content_digest: path_digest(&candidate.join(path)).unwrap(),
                role: RegenCandidateManifestSurfaceRole::GeneratedSurface,
            })
            .collect::<Vec<_>>();
        surfaces.sort_by(|left, right| left.projected_path.cmp(&right.projected_path));
        let population = surfaces
            .iter()
            .map(|surface| surface.projected_path.clone())
            .collect::<Vec<_>>();
        let candidate_tree_digest =
            candidate_artifact_tree_digest(candidate, &population, "fixture candidate").unwrap();
        let aggregate_digest = candidate_manifest_aggregate(
            "seed-0",
            "generation-0",
            "tree-0",
            &candidate_tree_digest,
            &surfaces,
        )
        .unwrap();
        let manifest = RegenCandidateManifest {
            producer_seed_digest: "seed-0".to_string(),
            generation_id: "generation-0".to_string(),
            candidate_tree_id: "tree-0".to_string(),
            candidate_tree_digest,
            surfaces,
            aggregate_digest,
        };
        let model = RegenConvergenceModel::load(&fixture_roots()).unwrap();
        let admitted = admit_candidate_manifest(&model, candidate, &manifest, "seed-0").unwrap();
        (manifest, admitted)
    }

    fn fixture_roots() -> Vec<String> {
        ["dag", "src/v2"]
            .iter()
            .map(|root| workspace_root().join(root).to_string_lossy().into_owned())
            .collect()
    }

    fn fixture_modules(rows: &[(&str, &str, &str)]) -> HashMap<String, String> {
        rows.iter()
            .map(|(path, module, _)| ((*path).to_string(), (*module).to_string()))
            .collect()
    }

    #[test]
    fn a_test_that_panics_after_creating_the_fixture_leaves_no_root_behind() {
        let (root_tx, root_rx) = std::sync::mpsc::channel();
        let unwound = std::panic::catch_unwind(move || {
            let (workspace, _, _, _) = fixture_workspace();
            root_tx.send(workspace.to_path_buf()).unwrap();
            assert!(
                workspace.exists(),
                "fixture root must exist before the panic"
            );
            panic!("planted failure after fixture creation");
        });
        assert!(unwound.is_err(), "the planted panic must fire");
        let root = root_rx
            .recv()
            .expect("fixture reported its root before panicking");
        assert!(!root.exists(), "leaked fixture root {}", root.display());
    }

    #[test]
    fn complete_candidate_population_refuses_an_unreadable_entry_and_sorts_successes() {
        let candidate = PathBuf::from("fixture-candidate");
        let refused = observe_complete_candidate_artifact_population_from_entries(
            &candidate,
            "fixture-observation",
            vec![
                Ok(candidate.join("a.rs")),
                Err("injected read_dir entry failure".to_string()),
                Ok(candidate.join("b.rs")),
            ],
        )
        .unwrap_err();
        assert_eq!(
            refused,
            "CandidateManifestPopulationUnreadable: candidate_path=fixture-candidate phase=fixture-observation error=injected read_dir entry failure"
        );

        let observed = observe_complete_candidate_artifact_population_from_entries(
            &candidate,
            "fixture-observation",
            vec![Ok(candidate.join("b.rs")), Ok(candidate.join("a.rs"))],
        )
        .unwrap();
        assert_eq!(observed, vec!["a.rs".to_string(), "b.rs".to_string()]);
    }
}

// ===========================================================================================
// THE AFFECTED SET OF ONE EDIT -- host realization of `gunbc.regen_affected_set`.
//
// The host does what the model cannot: read the edit (the floor's git diff range), read the tree
// (module names from the edited files, the seed's closure edge index, the committed mirror
// population), and take the reverse walk over the full edge index at native cost. The VERDICT --
// which arm, which members -- is the model's: the host hands `regen_affected_set` the edited
// modules, the unlocatable paths, the edges among the modules its walk reached, the compared rows,
// and the declared bootstrap rows, and prints the answer. The host's walk is held to the model's
// answer on every run (`lockstep` below): a disagreement is a refusal, never a preferred side.
// ===========================================================================================

const REGEN_AFFECTED_SET_PRODUCER: &str = "claim_executor --regen-affected-set";
const REGEN_AFFECTED_SET_ENTRY_UNDER_ROOT: &str = "gunbc/regen_affected_set.dag";

pub struct RegenAffectedSetOutcome {
    /// Provenance, the edited population, the bound line, and one `member` line per mirror.
    pub rendered: String,
    /// The model's arm name (`AffectedMirrors` | `WholePopulation` | `EditedSetUnlocatable`).
    pub arm: String,
    /// The mirrors the bound names, by committed basename; empty for the two non-selecting arms.
    pub members: Vec<String>,
}

/// The model's answer for one edited population, as the host consumes it.
pub struct AffectedSetBound {
    pub line: String,
    pub arm: String,
    pub members: Vec<String>,
}

/// The edited population, classified. `unlocatable` is every `.dag` path the diff names that the
/// tree cannot name as a module -- departed, unreadable, or without a `module` line -- and a
/// non-empty list is the model's refusal arm. Paths that are not `.dag` are not the selection's
/// subject (a hand edit to a mirror is what the regen's own diff catches) and are only counted.
#[derive(Debug, PartialEq, Eq)]
pub struct EditedPopulation {
    pub edited_modules: Vec<String>,
    pub unlocatable: Vec<String>,
    pub non_dag_paths: Vec<String>,
}

pub fn edited_population_from_diff(workspace: &Path, diff_text: &str) -> EditedPopulation {
    let departed = super::parse_unified_diff_departed_paths(diff_text);
    let mut paths: BTreeSet<String> = super::parse_unified_diff_changed_new_lines(diff_text)
        .keys()
        .cloned()
        .collect();
    paths.extend(super::parse_unified_diff_added_paths(diff_text));
    paths.extend(departed.iter().cloned());
    let mut edited_modules: BTreeSet<String> = BTreeSet::new();
    let mut unlocatable = Vec::new();
    let mut non_dag_paths = Vec::new();
    for path in paths {
        if !path.ends_with(".dag") {
            non_dag_paths.push(path);
            continue;
        }
        if departed.contains(&path) {
            unlocatable.push(format!(
                "{path} (departed: no module line remains in the tree)"
            ));
            continue;
        }
        match fs::read_to_string(workspace.join(&path)) {
            Ok(content) => match super::extract_module_path_public(&content) {
                Some(module) => {
                    edited_modules.insert(module);
                }
                None => unlocatable.push(format!("{path} (no module line)")),
            },
            Err(e) => unlocatable.push(format!("{path} (unreadable: {e})")),
        }
    }
    EditedPopulation {
        edited_modules: edited_modules.into_iter().collect(),
        unlocatable,
        non_dag_paths,
    }
}

/// The seed's closure edges, module to module, off the SAME edge index the regen's closure walk
/// uses (`whole_pool_closure_edge_index`: dotted references, which include every import line, plus bare
/// references) -- one authority for "what pulls what", read here in reverse. A file the index
/// names but cannot map to a module is a refusal: an edge dropped silently would shrink the bound.
/// Every module the regen source roots declare: the `source_files` keys of the module index over
/// `regen_source_roots()`, sorted. The edge-free half of `regen_module_edges`, for a caller that
/// demands the names and not the dependency edges.
pub fn regen_module_names(workspace: &Path) -> Vec<String> {
    let abs_roots: Vec<String> = super::regen_source_roots()
        .all()
        .iter()
        .map(|root| {
            workspace
                .join(root.repo_relative_path())
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let index = super::build_multi_entry_index(&abs_roots);
    index
        .source_files
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn regen_module_edges(
    workspace: &Path,
) -> Result<(Vec<(String, String)>, Vec<String>), String> {
    let abs_roots: Vec<String> = super::regen_source_roots()
        .all()
        .iter()
        .map(|root| {
            workspace
                .join(root.repo_relative_path())
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let index = super::build_multi_entry_index(&abs_roots);
    let edge_index = super::whole_pool_closure_edge_index(&index)?;
    let mut module_of_path: HashMap<String, String> = HashMap::new();
    let mut modules: BTreeSet<String> = BTreeSet::new();
    for (module, source) in index.source_files.iter() {
        module_of_path.insert(
            super::workspace_relative_repo_path(&source.path),
            module.clone(),
        );
        modules.insert(module.clone());
    }
    let mut edges: BTreeSet<(String, String)> = BTreeSet::new();
    let mut unmapped: BTreeSet<String> = BTreeSet::new();
    for table in [&edge_index.ref_out, &edge_index.bare_out] {
        for (from_path, to_paths) in table {
            let from_key = super::workspace_relative_repo_path(from_path);
            let Some(from) = module_of_path.get(&from_key) else {
                unmapped.insert(from_key);
                continue;
            };
            for to_path in to_paths {
                let to_key = super::workspace_relative_repo_path(to_path);
                match module_of_path.get(&to_key) {
                    Some(to) if to != from => {
                        edges.insert((from.clone(), to.clone()));
                    }
                    Some(_) => {}
                    None => {
                        unmapped.insert(to_key);
                    }
                }
            }
        }
    }
    if !unmapped.is_empty() {
        return Err(format!(
            "refusal: {} closure edge endpoint(s) name no module in the index, so the reverse \
             walk would be missing edges: {:?}",
            unmapped.len(),
            unmapped.iter().take(8).collect::<Vec<_>>()
        ));
    }
    Ok((edges.into_iter().collect(), modules.into_iter().collect()))
}

/// The host's reverse walk: every module from which an edited module is reachable. The model's
/// `regen_reverse_closure` is the same relation as a bounded fold; `lockstep` holds them equal.
pub fn regen_reverse_closure_host(
    edited: &[String],
    edges: &[(String, String)],
) -> BTreeSet<String> {
    let mut dependents_of: HashMap<&str, Vec<&str>> = HashMap::new();
    for (from, to) in edges {
        dependents_of
            .entry(to.as_str())
            .or_default()
            .push(from.as_str());
    }
    let mut reached: BTreeSet<String> = edited.iter().cloned().collect();
    let mut frontier: Vec<String> = edited.to_vec();
    while let Some(module) = frontier.pop() {
        if let Some(dependents) = dependents_of.get(module.as_str()) {
            for dependent in dependents {
                if reached.insert((*dependent).to_string()) {
                    frontier.push((*dependent).to_string());
                }
            }
        }
    }
    reached
}

/// The committed mirror population as `(module, basename)` rows: a module whose mirror basename
/// (`a.b.c` -> `a_b_c.rs`) is a committed generated file. Rows are the join of two authorities the
/// regen already owns -- the module index and the committed generated population -- never a list.
pub fn compared_mirror_rows(
    stage0_src: &Path,
    modules: &[String],
) -> Result<Vec<(String, String)>, String> {
    let committed: BTreeSet<String> = committed_generated_basenames(stage0_src)?
        .into_iter()
        .collect();
    Ok(modules
        .iter()
        .filter_map(|module| {
            let basename = format!("{}.rs", module.replace('.', "_"));
            committed
                .contains(&basename)
                .then(|| (module.clone(), basename))
        })
        .collect())
}

/// The generation-role population, asked of `gunbc.regen_affected_set` rather than reproduced
/// as a Rust prefix test. The model's explicit generation-input/bootstrap-source roster is the
/// sole producer; dependency edges remain the separate affectedness authority and are not cited
/// as provenance for an answer they cannot change.
pub fn regen_generation_role_population(
    source_roots: &[String],
    modules: &[String],
) -> Result<
    (
        BTreeSet<String>,
        BTreeSet<String>,
        BTreeSet<String>,
        HashMap<String, String>,
        HashMap<String, (String, String)>,
    ),
    String,
> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = affected_set_entry(source_roots)?;
    let index = super::process_shared_index_or_refuse(source_roots);
    let (graph, indices) = super::resolve_entry_with_index(&index, &entry)
        .map_err(|e| format!("refusal: {entry} did not resolve for generation roles: {e}"))?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let strings = |items: &[String]| {
        Value::List(Rc::new(
            items.iter().map(str_value).collect::<Vec<_>>().into(),
        ))
    };
    let list_result = |function: &str, args: Vec<(Option<String>, Value)>| {
        match v1_interpreter::with_active_context(&ctx, || {
            v1_interpreter::run_in_context_with_args(&ctx, function, &args, false)
        })
        .map_err(|e| format!("refusal: {function} did not answer: {e}"))?
        {
            Value::List(ref items) => items
                .iter()
                .map(|item| match item {
                    Value::Str(s) => Ok(s.to_string()),
                    other => Err(format!(
                        "refusal: {function} returned a {} list member",
                        other.type_label_public()
                    )),
                })
                .collect::<Result<BTreeSet<_>, _>>(),
            other => Err(format!(
                "refusal: {function} returned {} instead of List",
                other.type_label_public()
            )),
        }
    };
    let generation = list_result(
        "regen_generation_role_modules",
        vec![(Some("modules".to_string()), strings(modules))],
    )?;
    let bootstrap_sources = list_result("regen_bootstrap_source_modules", vec![])?;
    let bootstrap_products = list_result("regen_bootstrap_product_paths", vec![])?;
    let owned_products = list_result("regen_generated_product_owner_paths", vec![])?;
    let mut generated_product_owners = HashMap::new();
    let mut generated_product_roles = HashMap::new();
    for product in &owned_products {
        let owners = list_result(
            "regen_generated_product_source_modules",
            vec![(Some("product".to_string()), str_value(product))],
        )?;
        if owners.len() != 1 {
            return Err(format!(
                "SurfaceOwnershipUnresolved: bootstrap product {product} has {} declared source modules",
                owners.len()
            ));
        }
        generated_product_owners.insert(product.clone(), owners.into_iter().next().unwrap());
        let roles = list_result(
            "regen_generated_product_generation_role_labels",
            vec![(Some("product".to_string()), str_value(product))],
        )?;
        if bootstrap_products.contains(product) {
            if !roles.is_empty() {
                return Err(format!(
                    "SurfaceInIncompatibleStageRoles: bootstrap product {product} also has aggregate roles {roles:?}"
                ));
            }
        } else if roles.len() != 1 {
            return Err(format!(
                "GenerationRoleUnresolved: aggregate product {product} has {} declared roles",
                roles.len()
            ));
        } else {
            let memberships = list_result(
                "regen_generated_product_seed_membership_labels",
                vec![(Some("product".to_string()), str_value(product))],
            )?;
            if memberships.len() != 1 {
                return Err(format!(
                    "SeedMembershipUnresolved: aggregate product {product} has {} declared seed memberships",
                    memberships.len()
                ));
            }
            generated_product_roles.insert(
                product.clone(),
                (
                    roles.into_iter().next().unwrap(),
                    memberships.into_iter().next().unwrap(),
                ),
            );
        }
    }
    Ok((
        generation,
        bootstrap_sources,
        bootstrap_products,
        generated_product_owners,
        generated_product_roles,
    ))
}

fn affected_set_entry(source_roots: &[String]) -> Result<String, String> {
    source_roots
        .iter()
        .map(|root| Path::new(root).join(REGEN_AFFECTED_SET_ENTRY_UNDER_ROOT))
        .find(|candidate| candidate.is_file())
        .map(|found| found.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "refusal: {REGEN_AFFECTED_SET_ENTRY_UNDER_ROOT} is not under any declared source \
                 root {source_roots:?}, so the bound has no authority to answer from"
            )
        })
}

/// The arguments `gunbc.regen_affected_set regen_affected_set` is asked with, one construction for
/// the bound's own run and for the partition-move decision that consumes the typed bound.
fn affected_set_model_args(
    ctx: &crate::v1_interpreter::InterpContext,
    edited: &[String],
    unlocatable: &[String],
    edges: &[(String, String)],
    compared: &[(String, String)],
) -> Vec<(Option<String>, crate::v1_interpreter::Value)> {
    use crate::v1_interpreter::{str_value, Value};
    let reached = regen_reverse_closure_host(edited, edges);
    let edge_values: Vec<Value> = edges
        .iter()
        .filter(|(_, to)| reached.contains(to))
        .map(|(from, to)| Value::Record {
            type_name: ctx.sym("DependencyEdge"),
            fields: Rc::new(vec![
                (ctx.sym("from"), str_value(from.clone())),
                (ctx.sym("to"), str_value(to.clone())),
            ]),
        })
        .collect();
    let compared_values: Vec<Value> = compared
        .iter()
        .map(|(module, basename)| Value::Record {
            type_name: ctx.sym("MirrorRow"),
            fields: Rc::new(vec![
                (ctx.sym("module"), str_value(module.clone())),
                (ctx.sym("basename"), str_value(basename.clone())),
            ]),
        })
        .collect();
    let strs = |items: &[String]| {
        let values: Vec<Value> = items.iter().map(str_value).collect();
        Value::List(Rc::new(values.into()))
    };
    let reached_list: Vec<String> = reached.iter().cloned().collect();
    let args = vec![
        (Some("edited".to_string()), strs(edited)),
        (Some("unlocatable".to_string()), strs(unlocatable)),
        (
            Some("edges".to_string()),
            Value::List(Rc::new(edge_values.into())),
        ),
        (
            Some("compared".to_string()),
            Value::List(Rc::new(compared_values.into())),
        ),
        (Some("modules".to_string()), strs(&reached_list)),
    ];
    args
}

/// Ask the model. The edges handed over are those whose target the host's walk reached -- the
/// subgraph on which the model's bounded fold re-derives the same closure at interpreter cost --
/// and `modules` is that reached set, which bounds the fold (a path among n modules has at most
/// n-1 edges). The bootstrap rows are the model's own declaration; nothing is passed for them.
pub fn render_affected_set_bound(
    source_roots: &[String],
    edited: &[String],
    unlocatable: &[String],
    edges: &[(String, String)],
    compared: &[(String, String)],
) -> Result<AffectedSetBound, String> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = affected_set_entry(source_roots)?;
    let index = super::process_shared_index_or_refuse(source_roots);
    let (graph, indices) = super::resolve_entry_with_index(&index, &entry).map_err(|e| {
        format!("refusal: {entry} did not resolve, so the bound cannot answer: {e}")
    })?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);

    let args = affected_set_model_args(&ctx, edited, unlocatable, edges, compared);
    let bound = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "regen_affected_set", &args, false)
    })
    .map_err(|e| format!("refusal: regen_affected_set did not answer: {e}"))?;
    let bound_arg = vec![(Some("bound".to_string()), bound)];
    let line = match v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_affected_set_bound_line",
            &bound_arg,
            false,
        )
    })
    .map_err(|e| format!("refusal: regen_affected_set_bound_line did not render: {e}"))?
    {
        Value::Str(ref s) => s.to_string(),
        other => {
            return Err(format!(
                "refusal: regen_affected_set_bound_line returned {} where a String was expected",
                other.type_label_public()
            ))
        }
    };
    let members: Vec<String> = match v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_affected_set_members",
            &bound_arg,
            false,
        )
    })
    .map_err(|e| format!("refusal: regen_affected_set_members did not answer: {e}"))?
    {
        Value::List(ref items) => items
            .iter()
            .map(|item| match item {
                Value::Str(s) => Ok(s.to_string()),
                other => Err(format!(
                    "refusal: regen_affected_set_members holds a {} where a String was expected",
                    other.type_label_public()
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        other => {
            return Err(format!(
                "refusal: regen_affected_set_members returned {} where a List was expected",
                other.type_label_public()
            ))
        }
    };
    let arm = line
        .strip_prefix("regen-affected-set: ")
        .and_then(|rest| rest.split_whitespace().next())
        .ok_or_else(|| format!("refusal: bound line has no arm: {line}"))?
        .to_string();
    // LOCKSTEP, every run: on the selecting arm the model's mirrors must be exactly the host's
    // reached modules joined to the compared rows. Either side alone could be wrong; agreement is
    // the evidence, and disagreement stops the line rather than electing a side.
    if arm == "AffectedMirrors" {
        let reached = regen_reverse_closure_host(edited, edges);
        let host_mirrors: BTreeSet<String> = compared
            .iter()
            .filter(|(module, _)| reached.contains(module))
            .map(|(_, basename)| basename.clone())
            .collect();
        let model_mirrors: BTreeSet<String> = members
            .iter()
            .filter(|m| host_mirrors.contains(*m) || compared.iter().any(|(_, b)| b == *m))
            .cloned()
            .collect();
        if host_mirrors != model_mirrors {
            return Err(format!(
                "refusal: lockstep disagreement -- host reverse walk names {} mirror(s), the model's \
                 regen_affected_set names {}; host-only {:?}, model-only {:?}",
                host_mirrors.len(),
                model_mirrors.len(),
                host_mirrors.difference(&model_mirrors).take(8).collect::<Vec<_>>(),
                model_mirrors.difference(&host_mirrors).take(8).collect::<Vec<_>>()
            ));
        }
    }
    Ok(AffectedSetBound { line, arm, members })
}

/// The bound for an edited population against the live tree: edges and compared rows from the
/// tree, verdict from the model.
pub fn affected_set_bound_for(
    workspace: &Path,
    source_roots: &[String],
    edited: &[String],
    unlocatable: &[String],
) -> Result<AffectedSetBound, String> {
    let (edges, modules) = regen_module_edges(workspace)?;
    let compared = compared_mirror_rows(&workspace.join("src/v1/stage0/src"), &modules)?;
    render_affected_set_bound(source_roots, edited, unlocatable, &edges, &compared)
}

/// The model's own answer for one scope, over one committed roster.
///
/// `v2.workflow.required_regen` `regen_scope_select` is the authority for what a scope selects;
/// this runs it and returns the members it names, so the host's `scope_selection` can be held to
/// it. Neither side is trusted over the other: agreement is the evidence, and disagreement stops
/// the line rather than electing a winner.
pub fn render_scope_selection(
    source_roots: &[String],
    scope: &RegenEmissionScope,
    committed: &[String],
) -> Result<Vec<String>, String> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = required_regen_scope_entry(source_roots)?;
    let index = super::process_shared_index_or_refuse(source_roots);
    let (graph, indices) = super::resolve_entry_with_index(&index, &entry).map_err(|e| {
        format!("refusal: {entry} did not resolve, so the scope cannot answer: {e}")
    })?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let strs = |items: &[String]| {
        let values: Vec<Value> = items.iter().map(str_value).collect();
        Value::List(Rc::new(values.into()))
    };
    let scope_value = match scope {
        RegenEmissionScope::WholePopulation => Value::Variant {
            type_name: ctx.sym("RegenEmissionScope"),
            variant_name: ctx.sym("WholePopulationScope"),
            fields: Rc::new(vec![]),
        },
        RegenEmissionScope::Affected { members } => Value::Variant {
            type_name: ctx.sym("RegenEmissionScope"),
            variant_name: ctx.sym("AffectedScope"),
            fields: Rc::new(vec![(ctx.sym("members"), strs(members))]),
        },
        RegenEmissionScope::Unlocatable { paths, reason } => Value::Variant {
            type_name: ctx.sym("RegenEmissionScope"),
            variant_name: ctx.sym("ScopeUnlocatable"),
            fields: Rc::new(vec![
                (ctx.sym("paths"), strs(paths)),
                (ctx.sym("reason"), str_value(reason.clone())),
            ]),
        },
    };
    let args = vec![
        (Some("scope".to_string()), scope_value),
        (Some("committed".to_string()), strs(committed)),
    ];
    let selection = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "regen_scope_select", &args, false)
    })
    .map_err(|e| format!("refusal: regen_scope_select did not answer: {e}"))?;
    let selection_arg = vec![(Some("sel".to_string()), selection)];
    match v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "regen_scope_selection_members",
            &selection_arg,
            false,
        )
    })
    .map_err(|e| format!("refusal: regen_scope_selection_members did not answer: {e}"))?
    {
        Value::List(ref items) => items
            .iter()
            .map(|item| match item {
                Value::Str(s) => Ok(s.to_string()),
                other => Err(format!(
                    "refusal: regen_scope_selection_members holds a {} where a String was \
                     expected",
                    other.type_label_public()
                )),
            })
            .collect::<Result<Vec<_>, _>>(),
        other => Err(format!(
            "refusal: regen_scope_selection_members returned {} where a List was expected",
            other.type_label_public()
        )),
    }
}

/// Run `v2.workflow.regen_scope_worktree` `regen_scope_observe_worktree_refusal` against the
/// real working tree. An empty answer admits the scoped round; any other answer is its refusal.
fn regen_scope_worktree_admitted(source_roots: &[String]) -> Result<(), String> {
    use crate::v1_interpreter::{self, ExecutionMode, Value};
    let entry = source_roots
        .iter()
        .map(|root| Path::new(root).join("workflow/regen_scope_worktree.dag"))
        .find(|candidate| candidate.is_file())
        .map(|found| found.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "refusal: workflow/regen_scope_worktree.dag is not under any declared source \
                 root {source_roots:?}, so the scoped round cannot establish that its edit \
                 observation sees every edit"
            )
        })?;
    let index = super::process_shared_index_or_refuse(source_roots);
    let (graph, indices) = super::resolve_entry_with_index(&index, &entry)
        .map_err(|e| format!("refusal: {entry} did not resolve: {e}"))?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Wet);
    let answer = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context(&ctx, "regen_scope_observe_worktree_refusal", false)
    })
    .map_err(|e| format!("refusal: regen_scope_observe_worktree_refusal did not answer: {e}"))?;
    match &answer {
        Value::Str(s) if s.is_empty() => Ok(()),
        Value::Str(s) => Err(s.to_string()),
        other => Err(format!(
            "refusal: regen_scope_observe_worktree_refusal returned {} where a String was expected",
            other.type_label_public()
        )),
    }
}

/// Where `v2.workflow.required_regen` lives under the declared source roots.
fn required_regen_scope_entry(source_roots: &[String]) -> Result<String, String> {
    source_roots
        .iter()
        .map(|root| Path::new(root).join("workflow/required_regen.dag"))
        .find(|candidate| candidate.is_file())
        .map(|found| found.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "refusal: workflow/required_regen.dag is not under any declared source root \
                 {source_roots:?}, so the scope has no authority"
            )
        })
}

/// THE BOUND AS A SCOPE, derived from the same edited population `run_regen_affected_set`
/// reports — one producer of the selection, read by the report and by the round.
///
/// The three arms map one to one onto the model's, and the third is the point: `EditedSetUnlocatable`
/// becomes `RegenEmissionScope::Unlocatable`, which REFUSES the round. It does not become
/// `WholePopulation`. "Regenerate everything" and "the selection could not answer" are different
/// states, and a fallback that widened here would be denominated in the corpus rather than in the
/// change — the absorbing fallback DESIGN section 5 forbids, in the one place where its cost is
/// unbounded.
pub fn regen_emission_scope_for_diff(
    workspace: &Path,
    source_roots: &[String],
) -> Result<RegenEmissionScope, String> {
    // THE EDIT OBSERVATION SEES COMMITS ONLY, so a tree carrying an uncommitted .dag edit
    // refuses before the diff is read rather than being scoped by an observation blind to it.
    // `v2.workflow.regen_scope_worktree` owns the predicate and the refusal text.
    regen_scope_worktree_admitted(source_roots)?;
    let diff_text = super::required_floor_runner::floor_git_diff_range()?;
    let population = edited_population_from_diff(workspace, &diff_text);
    let bound = affected_set_bound_for(
        workspace,
        source_roots,
        &population.edited_modules,
        &population.unlocatable,
    )?;
    let scope = match bound.arm.as_str() {
        "AffectedMirrors" => RegenEmissionScope::Affected {
            members: bound.members,
        },
        "WholePopulation" => RegenEmissionScope::WholePopulation,
        "EditedSetUnlocatable" => RegenEmissionScope::Unlocatable {
            paths: population.unlocatable.clone(),
            reason: bound.line.clone(),
        },
        // An arm this host does not know is not a scope it may guess at. The model owns the
        // vocabulary; a new arm arriving here refuses rather than picking the nearest neighbour.
        other => {
            return Err(format!(
                "refusal: the affected-set bound answered an arm this round cannot consume: \
                 {other} ({}). Teach `regen_emission_scope_for_diff` the arm or fix the model.",
                bound.line
            ))
        }
    };
    // LOCKSTEP, ON EVERY SCOPED ROUND, over the real committed roster this round will use. The
    // model's fold and the host's filter answer the same question, and a round runs only when
    // they answer it the same way.
    let committed = committed_generated_basenames(&workspace.join("src/v1/stage0/src"))?;
    if !matches!(scope, RegenEmissionScope::Unlocatable { .. }) {
        let host: BTreeSet<String> = scope_selection(&scope, &committed)?.into_iter().collect();
        let model: BTreeSet<String> = render_scope_selection(source_roots, &scope, &committed)?
            .into_iter()
            .collect();
        if host != model {
            return Err(format!(
                "refusal: scope lockstep disagreement -- the host selects {} mirror(s), \
                 v2.workflow.required_regen regen_scope_select selects {}; host-only {:?}, \
                 model-only {:?}",
                host.len(),
                model.len(),
                host.difference(&model).take(8).collect::<Vec<_>>(),
                model.difference(&host).take(8).collect::<Vec<_>>()
            ));
        }
    }
    Ok(scope)
}

/// `claim_executor --regen-affected-set`: the edited population is the floor's own diff range
/// (the same "what changed" the required floor selects witnesses from), so the selection and the
/// gate read one edit.
pub fn run_regen_affected_set(source_roots: &[String]) -> Result<RegenAffectedSetOutcome, String> {
    let workspace = workspace_root();
    let tree = git_head_sha(&workspace)?;
    let tree_dirty = git_tree_dirty(&workspace)?;
    let diff_text = super::required_floor_runner::floor_git_diff_range()?;
    let population = edited_population_from_diff(&workspace, &diff_text);
    let bound = affected_set_bound_for(
        &workspace,
        source_roots,
        &population.edited_modules,
        &population.unlocatable,
    )?;
    let mut rendered = format!(
        "regen-affected-set: producer={REGEN_AFFECTED_SET_PRODUCER} host={} tree={tree} tree_dirty={tree_dirty}\n",
        host_name()
    );
    for module in &population.edited_modules {
        rendered.push_str(&format!("regen-affected-set: edited {module}\n"));
    }
    for path in &population.unlocatable {
        rendered.push_str(&format!("regen-affected-set: unlocatable {path}\n"));
    }
    rendered.push_str(&format!(
        "regen-affected-set: non_dag_paths={}\n",
        population.non_dag_paths.len()
    ));
    rendered.push_str(&bound.line);
    rendered.push('\n');
    for member in &bound.members {
        rendered.push_str(&format!("regen-affected-set: member {member}\n"));
    }
    Ok(RegenAffectedSetOutcome {
        rendered,
        arm: bound.arm,
        members: bound.members,
    })
}

/// THE SCOPE'S OWN EVIDENCE. Each of these has a discriminating RED: the selection is wrong if
/// it selects too much, selects something the tree does not carry, or answers at all on the
/// refusal arm.
#[cfg(test)]
mod regen_emission_scope_tests {
    use super::*;

    fn roots() -> Vec<String> {
        ["dag", "src/v2"]
            .iter()
            .map(|r| workspace_root().join(r).to_string_lossy().into_owned())
            .collect()
    }

    fn s(x: &str) -> String {
        x.to_string()
    }

    fn committed() -> Vec<String> {
        ["std_a.rs", "std_b.rs", "gunbc_c.rs", "v1_rt.rs"]
            .iter()
            .map(|n| s(n))
            .collect()
    }

    #[test]
    fn the_whole_population_scope_selects_the_whole_population() {
        assert_eq!(
            scope_selection(&RegenEmissionScope::WholePopulation, &committed()).unwrap(),
            committed()
        );
    }

    /// The selection is the INTERSECTION with the committed roster, not the bound's list. The red
    /// this discriminates: `not_in_the_tree.rs` is named by the bound and absent from the tree, and
    /// a selection that took the bound verbatim would carry it into the compared set, where it
    /// reads as a missing file rather than as a member outside the tree.
    #[test]
    fn an_affected_scope_selects_the_intersection_with_the_committed_population() {
        let scope = RegenEmissionScope::Affected {
            members: vec![s("std_b.rs"), s("v1_rt.rs"), s("not_in_the_tree.rs")],
        };
        assert_eq!(
            scope_selection(&scope, &committed()).unwrap(),
            vec![s("std_b.rs"), s("v1_rt.rs")]
        );
    }

    /// An edit touching no mirror selects nothing, and that is an answer rather than a refusal:
    /// the honest round for it adjudicates nothing and installs nothing.
    #[test]
    fn an_affected_scope_naming_no_committed_mirror_selects_nothing() {
        let scope = RegenEmissionScope::Affected {
            members: vec![s("not_in_the_tree.rs")],
        };
        assert!(scope_selection(&scope, &committed()).unwrap().is_empty());
    }

    /// THE REFUSAL, AND THE DIRECTION THAT MATTERS. The failure arm must refuse, never widen: an
    /// unlocatable scope produces an `Err`, and the assertion below is that it did NOT produce the
    /// committed population. A regression to `WholePopulation` here would be green on every other
    /// check in this file -- the round would run, converge and pass -- while the only signal that
    /// the module locator has a deficit was gone (DESIGN section 5, the absorbing fallback).
    #[test]
    fn an_unlocatable_scope_refuses_and_does_not_widen_to_the_population() {
        let scope = RegenEmissionScope::Unlocatable {
            paths: vec![s("dag/std/departed.dag")],
            reason: s("regen-affected-set: EditedSetUnlocatable unlocatable=1"),
        };
        let answer = scope_selection(&scope, &committed());
        let message = answer.expect_err("an unlocatable scope has no selection");
        assert!(
            message.contains("does not widen") && message.contains("dag/std/departed.dag"),
            "the refusal names the unlocatable path and its direction: {message}"
        );
    }

    /// THE ARM `review 57625` FOUND, and the red it discriminates.
    ///
    /// An empty affected selection is an ordinary answer, and the round for it must END, not
    /// refuse. Before the repair the host drove the empty selection into `verify_candidate_tree`
    /// and both digest functions, all three of which correctly refuse an empty population, so the
    /// documented no-op was a hard error. The three assertions below are the three properties that
    /// were false: the empty selection is not an `Err`; the round's receipt carries no digest to
    /// have fabricated; and the receipt is NOT a refusal, so a caller cannot read "nothing to do"
    /// as "something is wrong".
    ///
    /// The empty-population refusals themselves are asserted to still stand, because the wrong
    /// repair here is to relax them -- that would let a whole-population round with a broken tree
    /// digest nothing and report a fixed point.
    #[test]
    fn an_empty_affected_selection_is_an_answer_and_the_empty_population_walls_still_stand() {
        let scope = RegenEmissionScope::Affected {
            members: vec![s("not_in_the_tree.rs")],
        };
        let selection = scope_selection(&scope, &committed()).expect("an answer, not a refusal");
        assert!(selection.is_empty());

        let receipt = RegenReceipt::NoAffectedMirrors {
            schema: RECEIPT_SCHEMA.to_string(),
            commit_sha: s("0000000000000000000000000000000000000000"),
            authority_digest: s("sha256:test"),
            scope: scope.line(),
        };
        assert_eq!(receipt.first_generation_equal(), None);
        assert_eq!(receipt.candidate_artifact(), None);
        assert_eq!(
            receipt.refusal_reason(),
            None,
            "an empty selection is not a refusal; reporting one would make an ordinary edit look \
             like a broken tree"
        );

        // The walls the repair deliberately did NOT touch.
        let tmp = std::env::temp_dir();
        assert!(verify_candidate_tree(&tmp, &[]).is_err());
        let formatter = match ResolvedFormatter::admit() {
            Ok(f) => f,
            // The formatter is a boundary fact; where it is absent this half of the control is
            // unobservable, and an unobserved control is a red, never an early-return pass.
            Err(e) => panic!(
                "NOT MEASURED: no admissible formatter ({e}), so the tree-digest walls cannot be \
                 observed. This is a red, not a pass."
            ),
        };
        assert!(tree_digest_from_map(&formatter, &HashMap::new(), &[]).is_err());
        assert!(tree_digest_for_basenames(&formatter, &tmp, &[], "committed").is_err());
    }
}

#[cfg(test)]
mod regen_affected_set_tests {
    use super::*;

    fn roots() -> Vec<String> {
        ["dag", "src/v2"]
            .iter()
            .map(|r| workspace_root().join(r).to_string_lossy().into_owned())
            .collect()
    }

    fn s(x: &str) -> String {
        x.to_string()
    }

    /// The witness's fixture graph, so the host walk and the model fold are held to one answer on
    /// the same edges the .dag witness asserts against.
    fn fixture_edges() -> Vec<(String, String)> {
        vec![
            (s("std.b"), s("std.a")),
            (s("gunbc.c"), s("std.b")),
            (s("gunbc.d"), s("std.x")),
            (s("v1.compiler.emit_rust"), s("std.a")),
        ]
    }

    fn fixture_compared() -> Vec<(String, String)> {
        [
            "std.a",
            "std.b",
            "gunbc.c",
            "gunbc.d",
            "std.x",
            "v1.compiler.emit_rust",
        ]
        .iter()
        .map(|m| (s(m), format!("{}.rs", m.replace('.', "_"))))
        .collect()
    }

    /// The edit reader on a synthetic diff: an existing module is named, a departed `.dag` and an
    /// added `.dag` that is not in the tree are unlocatable, and a `.rs` is only counted.
    #[test]
    fn edited_population_classifies_named_departed_and_missing_paths() {
        let diff = "\
diff --git a/dag/std/content_hash.dag b/dag/std/content_hash.dag
--- a/dag/std/content_hash.dag
+++ b/dag/std/content_hash.dag
@@ -1,1 +1,2 @@
 module std.content_hash
+data planted: Int = 1
diff --git a/dag/std/gone.dag b/dag/std/gone.dag
deleted file mode 100644
--- a/dag/std/gone.dag
+++ /dev/null
@@ -1,1 +0,0 @@
-module std.gone
diff --git a/dag/std/never_written.dag b/dag/std/never_written.dag
new file mode 100644
--- /dev/null
+++ b/dag/std/never_written.dag
@@ -0,0 +1,1 @@
+module std.never_written
diff --git a/src/v1/stage0/src/v1_rt.rs b/src/v1/stage0/src/v1_rt.rs
--- a/src/v1/stage0/src/v1_rt.rs
+++ b/src/v1/stage0/src/v1_rt.rs
@@ -1,1 +1,2 @@
 // x
+// y
";
        let population = edited_population_from_diff(&workspace_root(), diff);
        assert_eq!(population.edited_modules, vec![s("std.content_hash")]);
        assert_eq!(
            population.unlocatable.len(),
            2,
            "{:?}",
            population.unlocatable
        );
        assert!(population.unlocatable[0].starts_with("dag/std/gone.dag (departed"));
        assert!(population.unlocatable[1].starts_with("dag/std/never_written.dag (unreadable"));
        assert_eq!(
            population.non_dag_paths,
            vec![s("src/v1/stage0/src/v1_rt.rs")]
        );
    }
}

// ---------------------------------------------------------------------------
// THE EMITTED `dag-artifact.json`'S OWN IDENTITY, ASKED THE ONLY WAY THAT ANSWERS IT.
//
// WHY IT LIVES BESIDE THE REGEN FIXED POINT AND NOT IN A PHASE OF ITS OWN. The question here is
// the fixed point's question one artifact over: does re-running the producer over an unchanged
// tree produce the same bytes? The regen fixed point asks it of the stage0 Rust surface; nothing
// asked it of `dag-artifact.json`, and the answer on main was no. It is a RIDER on an existing
// required phase rather than a sixth phase because the roster of required jobs is closed to
// growth (`gunbc.witness.witness_floor_workflow` `witness_floor_lane_jobs`), and this check belongs to a
// phase that already exists.
//
// WHAT THE DEFECT WAS, so the control's shape is legible. `v1.compile` `serialize_typed_module`
// and `emit_dag_artifact` fed `map_keys` of an item registry -- a host-ordered `HashMap` --
// straight into the artifact. Two runs of one binary over one tree emitted the identical graph
// with `item_registry_keys` in two orders, which is a byte difference with no semantic
// difference, and therefore a difference every byte-equality oracle downstream reports as
// "changed". Both sites now read `sorted_map_keys`, whose `PrimitiveTraversalOrderFact` is
// `CanonicalOrder` (`std.primitive_identity`). THE FIX IS AT THE KEY, NOT AT THE TEXT: nothing
// here sorts emitted output.
//
// WHY TWO IN-PROCESS EMISSIONS DISCRIMINATE, and it is not obvious. `im::HashMap`'s default
// `RandomState` is seeded PER INSTANCE from a thread-local counter, not once per process, so two
// independently built registries in one process already disagree on iteration order. The control
// therefore does not need two subprocesses to see the class -- and the class it sees is the same
// one two subprocesses saw (gunbc#11072).
//
// THE TWO HALVES ARE BOTH REQUIRED. Equality alone is satisfied by an emitter that emits the same
// bytes for every input, so the perturbed arm asserts that ONE added declaration still changes the
// artifact. And both are refused rather than passed when the fixture stops carrying the subject:
// a specimen whose artifact holds fewer than two registry keys cannot distinguish an ordered
// emitter from an unordered one, and a control that cannot go red is worse than absent.

pub const DAG_ARTIFACT_IDENTITY_SUBJECT_ROOT: &str = "fixtures/dag_artifact_identity/subject";
pub const DAG_ARTIFACT_IDENTITY_PERTURBED_ROOT: &str = "fixtures/dag_artifact_identity/perturbed";
pub const DAG_ARTIFACT_IDENTITY_SPECIMEN_BASENAME: &str = "registry_order_specimen.dag";
/// The declaration the perturbed arm adds and the subject does not have. The positive control
/// asserts THIS KEY's presence and absence, rather than inferring the added declaration from the
/// two artifacts merely differing.
pub const DAG_ARTIFACT_IDENTITY_ADDED_DECLARATION: &str = "item_quebec";
pub const DAG_ARTIFACT_BASENAME: &str = "dag-artifact.json";

/// The minimum number of registry keys the subject artifact must carry for the equality half to
/// mean anything. Two keys agree by chance half the time; the committed specimen carries
/// sixteen, and this floor exists so a specimen edited down to nothing refuses instead of
/// greening.
pub const DAG_ARTIFACT_IDENTITY_MIN_REGISTRY_KEYS: usize = 8;

#[derive(Debug, Clone)]
pub struct DagArtifactIdentityOutcome {
    pub subject_root: String,
    pub perturbed_root: String,
    pub first_digest: String,
    pub second_digest: String,
    pub perturbed_digest: String,
    pub registry_keys_observed: usize,
    pub findings: Vec<String>,
}

impl DagArtifactIdentityOutcome {
    pub fn passed(&self) -> bool {
        self.findings.is_empty()
    }

    /// A green NAMES ITS DENOMINATOR: the key count the equality was taken over, so a subject
    /// that narrowed is legible from the log rather than only from the verdict.
    pub fn summary(&self) -> String {
        format!(
            "registry_keys={} first={} second={} perturbed={} findings={}",
            self.registry_keys_observed,
            self.first_digest,
            self.second_digest,
            self.perturbed_digest,
            self.findings.len()
        )
    }
}

/// One emission of one fixture root, returning the `dag-artifact.json` TEXT.
///
/// It goes through `cli_run::compile_emission` -- the same transaction `gunbc compile` runs --
/// rather than reaching `emit_dag_artifact` directly, because the subject is what the CLI
/// produces. Nothing is written to disk: the bytes compared are the bytes the writer would have
/// written, so no filesystem state can make two unequal emissions look equal.
///
/// THE SUBJECT IS AN `Entry`, NOT A `PrimaryRoot`, AND THAT IS NOT A SPELLING PREFERENCE. A
/// primary-root subject is a WHOLE-CORPUS compile and is asked `whole_corpus_compile_admission`
/// before anything is indexed; on a host exposing no cgroup memory limit that arm refuses with
/// `WholeCorpusCompileBudgetUnreadable` — correctly, since an unbounded resolve on an unbounded
/// host is the SIGKILL it exists to prevent. Measured on a BuildBuddy runner: the root form
/// refused four runs out of four and emitted nothing. This control's subject is one specimen
/// file, so `Entry` is also what it actually MEANS; borrowing the corpus subject would have made
/// the control's availability a fact about the host's cgroup rather than about the emitter.
fn emit_dag_artifact_text(root_rel: &str) -> Result<String, String> {
    // ANCHORED ON `workspace_root()`, NOT LEFT RELATIVE. A relative subject is resolved against
    // the PROCESS's workspace root, which is not the same path under every consumer: the same
    // spelling that `gunbc compile` resolved refused under the test harness with
    // `entry file does not exist or is not a file`. The fixture's location is a fact about the
    // repository, so it is addressed as one.
    let root = workspace_root().join(root_rel);
    let entry = root.join(DAG_ARTIFACT_IDENTITY_SPECIMEN_BASENAME);
    // A FIXTURE POOL, COMPILED ONCE AND READ BY NOTHING AFTER: it owns its index, so the
    // subject and the perturbed root never sit beside the run's pool in the shared memo.
    let run = super::compile_emission_over(
        &super::CompileRequest {
            subject: super::CompileSubject::Entry(entry.to_string_lossy().to_string()),
            root_demand: super::RootDemandDeclaration::default(),
            source_roots: vec![root.to_string_lossy().to_string()],
            primary_precedence: false,
            render_targets: vec![RenderTarget::Dag],
        },
        super::IndexResidency::OwnedByThisCompile,
    );
    match &run.disposition {
        super::CompileDisposition::Completed { .. } => {}
        super::CompileDisposition::Refused { phase, cause } => {
            return Err(format!("{root_rel}: compile refused at {phase}: {cause}"));
        }
        super::CompileDisposition::NotExecuted {
            earlier_phase,
            cause,
        } => {
            return Err(format!(
                "{root_rel}: compile not executed at {earlier_phase}: {cause}"
            ));
        }
    }
    for emission in &run.emissions {
        for file in emission.result.files.iter() {
            if file.path.as_str().ends_with(DAG_ARTIFACT_BASENAME) {
                return Ok(file.content.clone());
            }
        }
    }
    Err(format!(
        "{root_rel}: the dag emission carries no {DAG_ARTIFACT_BASENAME} -- the control's subject \
         is absent, which is a refusal and not a passing comparison over nothing"
    ))
}

/// How many `item_registry_keys` entries the artifact actually carries, summed over every list it
/// holds. This is the control's DENOMINATOR, and it is measured rather than assumed: the fixture
/// could be edited, the field could be renamed, and either would leave an equality that holds
/// trivially.
fn registry_keys_in_artifact(artifact: &str) -> usize {
    let marker = "\"item_registry_keys\": [";
    let mut total = 0usize;
    let mut rest = artifact;
    while let Some(at) = rest.find(marker) {
        rest = &rest[at + marker.len()..];
        match rest.find(']') {
            Some(end) => {
                let body = &rest[..end];
                if !body.trim().is_empty() {
                    total += body.matches(',').count() + 1;
                }
                rest = &rest[end..];
            }
            None => break,
        }
    }
    total
}

/// The two-run byte-equality control over `dag-artifact.json`, with its positive control.
pub fn run_dag_artifact_identity() -> Result<DagArtifactIdentityOutcome, String> {
    let first = emit_dag_artifact_text(DAG_ARTIFACT_IDENTITY_SUBJECT_ROOT)?;
    let second = emit_dag_artifact_text(DAG_ARTIFACT_IDENTITY_SUBJECT_ROOT)?;
    let perturbed = emit_dag_artifact_text(DAG_ARTIFACT_IDENTITY_PERTURBED_ROOT)?;

    let registry_keys_observed = registry_keys_in_artifact(&first);
    let mut findings = Vec::new();

    if registry_keys_observed < DAG_ARTIFACT_IDENTITY_MIN_REGISTRY_KEYS {
        findings.push(format!(
            "the subject artifact carries {registry_keys_observed} item_registry_keys entries, \
             below the {DAG_ARTIFACT_IDENTITY_MIN_REGISTRY_KEYS} this control needs to \
             distinguish an ordered emitter from an unordered one -- the specimen at {} no \
             longer carries the subject",
            DAG_ARTIFACT_IDENTITY_SUBJECT_ROOT
        ));
    }

    if first != second {
        let at = first
            .char_indices()
            .zip(second.char_indices())
            .find(|((_, a), (_, b))| a != b)
            .map(|((i, _), _)| i);
        findings.push(format!(
            "two emissions of {} over one unchanged tree produced different \
             {DAG_ARTIFACT_BASENAME} bytes (first differing char offset {}) -- the emitted \
             artifact's identity is unstable, so every byte-equality oracle over it reports \
             reordering as change",
            DAG_ARTIFACT_IDENTITY_SUBJECT_ROOT,
            at.map(|i| i.to_string())
                .unwrap_or_else(|| "n/a (length differs)".to_string()),
        ));
    }

    if first == perturbed {
        findings.push(format!(
            "the perturbed specimen at {} emitted the SAME {DAG_ARTIFACT_BASENAME} bytes as the \
             subject -- the equality half above is then satisfied by an emitter blind to its \
             input, which is a control that cannot go red",
            DAG_ARTIFACT_IDENTITY_PERTURBED_ROOT
        ));
    }

    // THE ADDED DECLARATION IS ASSERTED, NOT INFERRED FROM THE BYTES DIFFERING. The two arms are
    // two files, so they differ in their module name as well as in the declaration, and
    // `serialize_node_record` emits names and spans -- which means byte inequality alone would
    // hold even if `item_quebec` never reached the artifact at all, and the control would claim a
    // sensitivity it had not established (review 64007, gunbc#11084, and the objection is
    // correct). These two arms name the declaration and check both directions of its membership,
    // so the positive control is discriminating for the thing it claims to protect rather than
    // for any difference whatsoever.
    if !perturbed.contains(DAG_ARTIFACT_IDENTITY_ADDED_DECLARATION) {
        findings.push(format!(
            "the perturbed artifact does not carry `{DAG_ARTIFACT_IDENTITY_ADDED_DECLARATION}` -- \
             the declaration the perturbed arm adds never reached the emitted bytes, so the two \
             artifacts differing says nothing about the emitter's sensitivity to a real change"
        ));
    }
    if first.contains(DAG_ARTIFACT_IDENTITY_ADDED_DECLARATION) {
        findings.push(format!(
            "the SUBJECT artifact already carries `{DAG_ARTIFACT_IDENTITY_ADDED_DECLARATION}` -- \
             the two arms are then not one declaration apart, and the positive control's subject \
             is not what it says it is"
        ));
    }

    Ok(DagArtifactIdentityOutcome {
        subject_root: DAG_ARTIFACT_IDENTITY_SUBJECT_ROOT.to_string(),
        perturbed_root: DAG_ARTIFACT_IDENTITY_PERTURBED_ROOT.to_string(),
        first_digest: bytes_digest(first.as_bytes()),
        second_digest: bytes_digest(second.as_bytes()),
        perturbed_digest: bytes_digest(perturbed.as_bytes()),
        registry_keys_observed,
        findings,
    })
}

#[cfg(test)]
/// The residency observation the generation gate is argued from, read the way the gate reads it.
///
/// WHY THIS TEST EXISTS: these lines are the evidence a correct refusal is judged on, and the one
/// failure mode that makes them worse than nothing is a field that parses to something plausible
/// and wrong. Stripping the bare name from `"VmRSS:\t 216968 kB"` leaves `":"` as the next token,
/// which prints as `VmRSS=:kB` -- a number-shaped field that is not a number, in the one line a
/// reviewer is reading to decide whether a retention is real.
mod orchestrator_residency_observation_tests {
    use super::status_memory_fields_from;

    #[test]
    fn status_memory_fields_parse_the_number_not_the_colon() {
        // Real `/proc/self/status` padding: a tab after the key, then the value right-aligned.
        let status = "Name:\tclaim_executor\nVmHWM:\t  8272424 kB\nVmSize:\t 123456789 kB\n\
                      VmRSS:\t   216968 kB\nUid:\t1000\t1000\t1000\t1000\n";
        let line = status_memory_fields_from(status);
        assert_eq!(
            line, "VmRSS=216968kB VmHWM=8272424kB VmSize=123456789kB",
            "the fields a reader judges a refusal by must carry the numbers, not the key's colon"
        );
        // A name that is a PREFIX of another key's line must not be confused with it: `VmRSS` and
        // `VmHWM` are distinct keys, and `Name:` starts with no field we read.
        assert!(
            !line.contains("Name"),
            "only the memory fields are printed: {line}"
        );
    }

    #[test]
    fn an_unreadable_field_is_omitted_rather_than_printed_as_zero() {
        // Absent, not zero: a diag line claiming `VmRSS=0kB` would read as an orchestrator that
        // released everything, which is the exact claim this whole observation is asked about.
        let line = status_memory_fields_from("Name:\tgum\nThreads:\t1\n");
        assert_eq!(line, "");
        let partial = status_memory_fields_from("VmRSS:\t 12 kB\nVmHWM:\t junk kB\n");
        assert_eq!(partial, "VmRSS=12kB");
    }

    #[test]
    fn an_unreadable_exe_mtime_is_omitted_rather_than_printed_as_a_reading() {
        // The same discipline on the field whose entire job is to tell two builds of the
        // same-named file apart. `exe_mtime=0` is a value, and a value is read as an observation:
        // an unreadable timestamp printed as zero is how a stale image gets credited as the one
        // under test, which is the failure this branch was diagnosed under.
        let known = self_exe_field_with("/tmp/claim_executor".to_string(), Some(1_730_000_000));
        assert_eq!(known, "exe=/tmp/claim_executor exe_mtime=1730000000");
        let unknown = self_exe_field_with("/tmp/claim_executor".to_string(), None);
        assert_eq!(unknown, "exe=/tmp/claim_executor");
        assert!(
            !unknown.contains("exe_mtime"),
            "an unreadable timestamp must not print at all: {unknown}"
        );
    }
}

#[cfg(test)]
mod dag_artifact_identity_tests {
    use super::{run_dag_artifact_identity, DAG_ARTIFACT_IDENTITY_MIN_REGISTRY_KEYS};
    /// THE LOCAL ARM OF THE REQUIRED RIDER, so the control can be run against a tree without
    /// driving a whole required lane. It is not the enrolled consumer -- that is the
    /// `dag-artifact-identity` rider inside `RequiredCiPhase::GeneratedArtifact`, which is what
    /// executes on the merge path (`cargo test -p v1-compiler --lib` is off it, a declared drop).
    ///
    /// Measured on this control's own two arms: with `v1.compile` emitting `map_keys` of the item
    /// registry it reports the two-emission finding, and with `sorted_map_keys` it is clean.
    ///
    /// THE TEST PATH IS `cli_run::required_regen_host::…`, NOT `required_regen_host::…`, because
    /// this file is wired in by `#[path] mod` INSIDE `cli_run`. Recorded because the wrong
    /// spelling does not error: `cargo test -- --exact <wrong path>` reports
    /// `0 passed; 0 failed; 970 filtered out` and exits 0, which is a green from a test that never
    /// ran -- measured, on this very test, before the spelling was corrected.
    #[test]
    #[ignore = "live-corpus: compiles a committed fixture source root off disk"]
    fn dag_artifact_identity_control_holds() {
        let outcome =
            run_dag_artifact_identity().expect("the control's subject must be obtainable");
        assert!(
            outcome.registry_keys_observed >= DAG_ARTIFACT_IDENTITY_MIN_REGISTRY_KEYS,
            "the fixture stopped carrying the subject: {}",
            outcome.summary()
        );
        assert!(
            outcome.passed(),
            "{}\n{}",
            outcome.summary(),
            outcome.findings.join("\n")
        );
    }
}

#[cfg(test)]
mod boundary_population_tests {
    use super::*;

    #[test]
    fn producer_report_is_read_as_a_typed_record() {
        let report =
            br#"{"schema":"gunbc.partition_boundary_report.v1","divergent":["src/b/lib.rs"]}"#;
        assert_eq!(
            boundary_drift_from_producer_report(report),
            Some(vec!["src/b/lib.rs".to_string()])
        );
        assert_eq!(
            boundary_drift_from_producer_report(b"emit-partition-crates: drifted x/y"),
            None
        );
        assert_eq!(
            boundary_drift_from_producer_report(br#"{"schema":"other","divergent":[]}"#),
            None
        );
    }
}

#[cfg(test)]
mod host_shell_scope_modules_tests {
    use super::host_shell_scope_modules;

    // DISCRIMINATING: `compile_clean` is declared by `cli_run`, not by lib.rs, and `drain` is
    // reached by a `#[path]` attribute. Reading lib.rs alone left both unowned, so a hand edit to
    // either refused `MirrorHasNoOwningPackage`. `stray.rs` sits in the same directory but is
    // declared nowhere, so it must stay unowned (the refusal for a path with no owning package).
    #[test]
    fn nested_and_path_attributed_modules_are_owned_and_undeclared_files_are_not() {
        let dir = std::env::temp_dir().join(format!("host_shell_scope_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("cli_run")).unwrap();
        std::fs::write(dir.join("lib.rs"), "mod cli_run;\nmod plain;\n").unwrap();
        std::fs::write(dir.join("plain.rs"), "").unwrap();
        std::fs::write(
            dir.join("cli_run.rs"),
            "mod compile_clean;\n#[path = \"drain_file.rs\"]\nmod drain;\n",
        )
        .unwrap();
        std::fs::write(dir.join("cli_run/compile_clean.rs"), "").unwrap();
        std::fs::write(
            dir.join("drain_file.rs"),
            "mod deeper;\n#[path = \"sibling.rs\"]\nmod via_sibling;\n",
        )
        .unwrap();
        std::fs::write(dir.join("deeper.rs"), "").unwrap();
        std::fs::write(dir.join("sibling.rs"), "").unwrap();
        std::fs::write(dir.join("stray.rs"), "").unwrap();
        // A second target root: `main.rs` is the `[[bin]]` root no `mod` declares, and it declares
        // `bin_child` itself. Reading lib.rs alone owns neither.
        std::fs::write(dir.join("main.rs"), "mod bin_child;\n").unwrap();
        std::fs::write(dir.join("bin_child.rs"), "").unwrap();
        // A second `[[bin]]` under `bin/`, as `infer_semantics_witness` is in the seed: a changed
        // file there refused `MirrorHasNoOwningPackage` (#15175 heal, run 41934097748).
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::write(dir.join("bin/witness.rs"), "").unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[[bin]]\nname = \"x\"\npath = \"main.rs\"\n\n[[bin]]\nname = \"witness\"\npath = \"bin/witness.rs\"\n",
        )
        .unwrap();
        let modules =
            host_shell_scope_modules(&dir.join("lib.rs"), &dir.join("Cargo.toml")).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        for owned in [
            "cli_run",
            "plain",
            "compile_clean",
            "drain_file",
            "deeper",
            "sibling",
            "lib",
            "main",
            "bin_child",
            "witness",
        ] {
            assert!(
                modules.iter().any(|m| m == owned),
                "{owned} unowned: {modules:?}"
            );
        }
        assert!(!modules.iter().any(|m| m == "stray"), "{modules:?}");
    }
}

/// THE GENERATION SUPERVISOR (`gunbc.host.memory_stall_refusal` `supervised_subject_verdict`, seed
/// mirror in `memory_governor`), exercised at its three host seams: which child lines count as
/// progress, how the per-sample decision moves its window, and the real teardown of a whole
/// generation's process group.
#[cfg(test)]
mod generation_supervisor_tests {
    use super::*;
    use crate::memory_governor::{SlotStallCounters, SlotStallReading};
    use std::time::Duration;

    fn liveness_model() -> SupervisedSubjectModel {
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|root| workspace_root().join(root).to_string_lossy().into_owned())
            .collect();
        SupervisedSubjectModel::load(&roots).expect("load gunbc.host.memory_stall_refusal")
    }

    #[test]
    fn a_progress_mark_is_exactly_what_trace_mark_renders_for_a_phase_boundary() {
        for emoji in [true, false] {
            for phase in ["regen.digest", "compile.emit", "round.built_seed_regen"] {
                assert!(is_generation_progress_mark(
                    &v1_rt::render_phase_begin_line_mirror(phase, emoji)
                ));
                for ms in [0, 785, 68_000, 311_000] {
                    assert!(is_generation_progress_mark(
                        &v1_rt::render_phase_concluded_line_mirror(phase, ms, emoji)
                    ));
                }
            }
        }
        for not_a_mark in [
            "",
            "required-regen: generation 1 supervisor elapsed_s=120 progress_marks=3",
            "   Compiling v1_compiler v0.1.0",
            "started regen.digest",
            "regen.digest done in 5 minutes",
            "required-regen: first_generation_equal=false because the generated surface moved",
        ] {
            assert!(
                !is_generation_progress_mark(not_a_mark),
                "{not_a_mark:?} was read as a progress mark; output is not progress"
            );
        }
    }

    fn counters(
        faults: u64,
        user_ms: u64,
        events: u64,
        psi_full_ms: Option<u64>,
    ) -> SlotStallReading {
        SlotStallReading::Read(SlotStallCounters {
            major_faults: faults,
            user_usec: user_ms * 1000,
            envelope_events: events,
            pressure_body: psi_full_ms
                .map(|ms| {
                    format!(
                        "some avg10=0.00 avg60=0.00 avg300=0.00 total={}\n\
                         full avg10=0.00 avg60=0.00 avg300=0.00 total={}\n",
                        ms * 1000 + 5,
                        ms * 1000
                    )
                })
                .unwrap_or_default(),
            memory_current: Some(22_000_000_000),
        })
    }

    /// THE HOST BRIDGE TO `regen_admit_generation_begin`, against the REAL convergence model: an
    /// orchestrator that did not grow is admitted, and one still holding a generation (the srv4
    /// reading) is refused with the model's own typed refusal. A stale argument name or a bare Int
    /// where the model declares `ByteSize` fails both legs.
    #[test]
    fn the_generation_begin_bridge_admits_and_refuses_through_the_real_model() {
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|root| workspace_root().join(root).to_string_lossy().into_owned())
            .collect();
        let model = RegenConvergenceModel::load(&roots).expect("load the convergence model");
        assert_eq!(
            admit_generation_begin_readings(&model, 1, 900_000_000, 900_000_000),
            Ok(())
        );
        assert_eq!(
            admit_generation_begin_readings(&model, 1, 900_000_000, 400_000_000),
            Ok(())
        );
        let refused = admit_generation_begin_readings(&model, 2, 900_000_000, 15_636_525_056)
            .expect_err("an orchestrator holding a generation was admitted");
        assert!(
            refused.contains("GenerationStateRetainedByOrchestrator ordinal=2")
                && refused.contains("orchestrator_resident_bytes_now=15636525056"),
            "{refused}"
        );
    }

    /// The srv3 ending, driven through the per-sample decision: a generation that stops marking
    /// while its slot loses most of its wall to memory and crosses memory.high is refused once the
    /// window has elapsed -- and not one sample before.
    #[test]
    fn a_silent_generation_under_the_envelope_throttle_is_refused_after_the_window() {
        let t0 = Instant::now();
        let model = liveness_model();
        let mut supervisor = GenerationSupervisor::new("generation 1".to_string(), t0);
        // A mark arrives at 60 s: the window is measured from here.
        assert!(matches!(
            supervisor.observe(&model, t0, 0, counters(0, 0, 0, Some(0))),
            SupervisorStep::Continue { .. }
        ));
        let mark_at = t0 + Duration::from_secs(60);
        assert!(matches!(
            supervisor.observe(&model, mark_at, 1, counters(10, 50_000, 0, Some(1_000))),
            SupervisorStep::Continue { .. }
        ));
        // The authority's window (gunbc.host.memory_stall_refusal
        // supervised_subject_no_progress_window_minimum_wall), supplied here as the reading it
        // declares: fifteen minutes.
        let window = Duration::from_secs(15 * 60);
        let wall_ms = window.as_millis() as u64;
        // One second short of the window: still progressing, however bad the counters.
        assert!(matches!(
            supervisor.observe(
                &model,
                mark_at + window - Duration::from_secs(1),
                1,
                counters(10, 50_000, 900, Some(1_000 + wall_ms))
            ),
            SupervisorStep::Continue { .. }
        ));
        match supervisor.observe(
            &model,
            mark_at + window,
            1,
            counters(10, 50_000, 412, Some(1_000 + wall_ms * 8 / 10)),
        ) {
            SupervisorStep::Refuse { refusal } => {
                assert!(
                    refusal.starts_with("MemoryStallRefusedEnvelopeThrottle"),
                    "{refusal}"
                );
                assert!(refusal.contains("[generation 1 after"), "{refusal}");
                assert!(refusal.contains("psi_full_share_bp=8000"), "{refusal}");
                assert!(
                    refusal.contains("envelope_events_in_window=412"),
                    "{refusal}"
                );
            }
            SupervisorStep::Continue { .. } => panic!("the throttle ending was admitted"),
        }
    }

    /// A NEW MARK RESETS THE WINDOW: the same pressure over the same wall admits when the
    /// generation kept marking, because semantic progress outranks every counter.
    #[test]
    fn a_generation_that_keeps_marking_is_never_judged_on_its_counters() {
        let t0 = Instant::now();
        let model = liveness_model();
        let mut supervisor = GenerationSupervisor::new("generation 2".to_string(), t0);
        let _ = supervisor.observe(&model, t0, 0, counters(0, 0, 0, Some(0)));
        for minute in 1..=40u64 {
            let at = t0 + Duration::from_secs(minute * 60);
            let step = supervisor.observe(
                &model,
                at,
                minute / 5,
                counters(minute * 100_000, 0, minute * 1000, Some(minute * 60_000)),
            );
            assert!(
                matches!(step, SupervisorStep::Continue { .. }),
                "refused at minute {minute} although a mark arrived every five minutes"
            );
        }
    }

    /// AN UNREADABLE SLOT IS NOT A ZERO: no refusal can be issued on readings that do not exist,
    /// and the heartbeat says the generation is unsupervised rather than looking healthy.
    #[test]
    fn an_unreadable_slot_is_reported_unsupervised_not_admitted_as_healthy() {
        let t0 = Instant::now();
        let model = liveness_model();
        let mut supervisor = GenerationSupervisor::new("generation 1".to_string(), t0);
        let unreadable = || SlotStallReading::Unreadable {
            cause: "read /sys/fs/cgroup/x/memory.stat: No such file or directory".to_string(),
        };
        assert!(matches!(
            supervisor.observe(&model, t0, 0, unreadable()),
            SupervisorStep::Continue { heartbeat: None }
        ));
        match supervisor.observe(&model, t0 + Duration::from_secs(3600), 0, unreadable()) {
            SupervisorStep::Continue {
                heartbeat: Some(line),
            } => assert!(line.contains("supervision=unavailable"), "{line}"),
            _ => panic!("an unreadable slot must heartbeat as unsupervised"),
        }
    }

    /// THE REAL TEARDOWN, end to end through the production spawn: a generation whose leader has a
    /// live descendant is refused, and NEITHER survives -- killing the direct child alone would
    /// leave the descendant holding the slot.
    #[test]
    fn a_refused_generation_is_ended_as_a_whole_process_group() {
        use std::os::unix::fs::PermissionsExt;
        let temp =
            std::env::temp_dir().join(format!("gunbc-generation-teardown-{}", std::process::id()));
        fs::create_dir_all(&temp).unwrap();
        let pid_file = temp.join("descendant.pid");
        let script = temp.join("generation.sh");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\nsleep 300 &\necho $! > {}\nwait\n",
                pid_file.display()
            ),
        )
        .unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        let started = Instant::now();
        let result = spawn_rebuilt_seed_regen_child(
            &script,
            &[],
            RebuiltSeedChildOutput::ForwardToFiles {
                stdout: temp.join("stdout"),
                stderr: temp.join("stderr"),
            },
            &temp,
            GenerationSupervision::RefuseAfter {
                after: Duration::from_secs(1),
            },
        );
        // BOUNDED: the descendant sleeps 300 s and holds the stderr pipe, so a teardown that missed
        // it returns only when it exits on its own -- and would then pass the liveness check below
        // by waiting it out. The refusal must come back within the teardown's own grace.
        let returned_after = started.elapsed();
        assert!(
            returned_after
                < Duration::from_secs(1) + GENERATION_TEARDOWN_GRACE * 2 + Duration::from_secs(5),
            "the refusal took {returned_after:?}: something outlived the teardown and held a pipe"
        );
        let refusal = match result {
            Err(refusal) => refusal,
            Ok(_) => panic!("the fixture generation was not refused"),
        };
        assert!(
            refusal.starts_with("fixture refusal teardown=Settled"),
            "{refusal}"
        );
        let descendant: i32 = fs::read_to_string(&pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let alive = std::path::Path::new(&format!("/proc/{descendant}")).exists()
            && !fs::read_to_string(format!("/proc/{descendant}/stat"))
                .unwrap_or_default()
                .contains(") Z ");
        assert!(
            !alive,
            "the generation's descendant {descendant} survived the refusal"
        );
        let _ = fs::remove_dir_all(&temp);
    }

    /// INHABITANCE: the production reader against this process's own cgroup. On a cgroup v2 host
    /// whose leaf carries the memory controller's files the reading must be complete; anywhere
    /// else it must say WHICH file it could not read rather than return numbers.
    #[test]
    fn the_slot_reader_inhabits_the_live_cgroup_or_names_what_it_could_not_read() {
        let Some(dir) = crate::memory_governor::leaf_cgroup_dir() else {
            return;
        };
        let reading = crate::memory_governor::read_slot_stall_counters(&dir);
        let complete = dir.join("memory.stat").is_file()
            && dir.join("cpu.stat").is_file()
            && dir.join("memory.events").is_file();
        match reading {
            SlotStallReading::Read(c) => {
                assert!(
                    complete,
                    "a reading was produced without the files it reads"
                );
                assert!(
                    c.user_usec > 0,
                    "this test process has run user code: {c:?}"
                );
            }
            SlotStallReading::Unreadable { cause } => {
                assert!(!complete, "the files exist but the reader refused: {cause}");
                assert!(cause.contains(&dir.display().to_string()), "{cause}");
            }
        }
    }
}

/// THE SURFACE ROLES TRAVEL BY RECEIPT (generation-state lifetime isolation): generation 0's child
/// derives them and its receipt carries them; the orchestrator never loads the corpus to derive them
/// itself. These pin the carrier, not the derivation (`convergence_surface_roles` is unchanged
/// except for the edge build it never read).
#[cfg(test)]
mod surface_roles_receipt_tests {
    use super::*;

    fn roles() -> SurfaceRolesForPlanning {
        (
            [
                ("std_measure.rs".to_string(), "std.measure".to_string()),
                (
                    "v1_compiler_parse.rs".to_string(),
                    "v1.compiler.parse".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            ["std.measure".to_string()].into_iter().collect(),
            BTreeSet::new(),
            ["Cargo.toml".to_string()].into_iter().collect(),
            ["std_measure.rs".to_string()].into_iter().collect(),
            [(
                "Cargo.toml".to_string(),
                ("GenerationInput".to_string(), "OutsideSeed".to_string()),
            )]
            .into_iter()
            .collect(),
        )
    }

    #[test]
    fn the_roles_round_trip_through_the_receipt_rows() {
        let carried = ConvergenceSurfaceRoles::from_planning(roles()).unwrap();
        let json = serde_json::to_string(&carried).unwrap();
        let back: ConvergenceSurfaceRoles = serde_json::from_str(&json).unwrap();
        assert_eq!(back.clone().into_planning().unwrap(), roles());
        // Sorted rows: one tree's roles serialize to one byte string whatever the map order was.
        assert_eq!(
            serde_json::to_string(&ConvergenceSurfaceRoles::from_planning(roles()).unwrap())
                .unwrap(),
            json
        );
    }

    /// RED: a first-generation receipt with no surface roles does not parse, so the orchestrator
    /// refuses the round instead of planning from roles nobody derived.
    #[test]
    fn a_receipt_without_surface_roles_refuses_the_round() {
        let dir = std::env::temp_dir().join(format!("gunbc-roles-receipt-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("receipt.json");
        let mut receipt = serde_json::to_value(RegenReceipt::FirstGeneration {
            schema: RECEIPT_SCHEMA.to_string(),
            commit_sha: "sha".to_string(),
            authority_digest: "a".to_string(),
            committed_generated_digest: "c".to_string(),
            candidate_generated_digest: "d".to_string(),
            first_generation_equal: false,
            changed_paths: vec!["std_measure.rs".to_string()],
            candidate_artifact: "target/stage0-regen-candidate".to_string(),
            candidate_manifest: RegenCandidateManifest {
                producer_seed_digest: "seed".to_string(),
                generation_id: "generation".to_string(),
                candidate_tree_id: "tree".to_string(),
                candidate_tree_digest: "tree-digest".to_string(),
                aggregate_digest: "aggregate".to_string(),
                surfaces: Vec::new(),
            },
            surface_roles: ConvergenceSurfaceRoles::from_planning(roles()).unwrap(),
        })
        .unwrap();
        // The carrier with required surface roles is the v3 shape; a v2 receipt is another carrier.
        assert_eq!(receipt["schema"], "gunbc.regen_receipt.v3");
        fs::write(&path, serde_json::to_vec(&receipt).unwrap()).unwrap();
        let read = read_first_generation_receipt(&path).expect("a complete receipt is read");
        assert_eq!(read.surface_roles.into_planning().unwrap(), roles());
        receipt.as_object_mut().unwrap().remove("surface_roles");
        fs::write(&path, serde_json::to_vec(&receipt).unwrap()).unwrap();
        let refused = read_first_generation_receipt(&path)
            .err()
            .expect("a receipt without surface roles was accepted");
        assert!(refused.contains("surface_roles"), "{refused}");
        let _ = fs::remove_dir_all(&dir);
    }

    /// The role facts are closed: an unknown label refuses on the way in and does not deserialize
    /// on the way back.
    #[test]
    fn an_unknown_role_label_is_unwritable_in_the_receipt() {
        let mut bad = roles();
        bad.5.insert(
            "x.rs".to_string(),
            ("GenerationInputs".to_string(), "OutsideSeed".to_string()),
        );
        let refused =
            ConvergenceSurfaceRoles::from_planning(bad).expect_err("a misspelt role was written");
        assert!(
            refused.contains("SurfaceRolesReceiptUnknownRole"),
            "{refused}"
        );
        let json = serde_json::to_string(&ConvergenceSurfaceRoles::from_planning(roles()).unwrap())
            .unwrap()
            .replace("\"GenerationInput\"", "\"GenerationInputs\"");
        assert!(serde_json::from_str::<ConvergenceSurfaceRoles>(&json).is_err());
    }

    #[test]
    fn a_duplicated_owner_row_is_refused_not_last_write_wins() {
        let mut carried = ConvergenceSurfaceRoles::from_planning(roles()).unwrap();
        carried.basename_to_module.push(SurfaceOwnerRow {
            basename: "std_measure.rs".to_string(),
            module: "std.other".to_string(),
        });
        let refused = carried
            .into_planning()
            .expect_err("a duplicate owner was resolved");
        assert!(
            refused.contains("SurfaceRolesReceiptDuplicated"),
            "{refused}"
        );
    }

    /// The footprint this moves, measured: the module names alone (the identical key set of the
    /// identical index) without the edge build. Run with --ignored --nocapture on a release build;
    /// prints the orchestrator-shaped VmRSS readings the RFM row cites.
    #[test]
    #[ignore]
    fn measure_module_names_versus_module_edges() {
        let ws = workspace_root();
        let rss = || {
            fs::read_to_string("/proc/self/status")
                .unwrap()
                .lines()
                .find(|l| l.starts_with("VmRSS"))
                .unwrap()
                .to_string()
        };
        let t = Instant::now();
        let names = regen_module_names(&ws);
        eprintln!("MEASURE names={} {:?} {}", names.len(), t.elapsed(), rss());
        let t = Instant::now();
        let (_edges, modules) = regen_module_edges(&ws).unwrap();
        eprintln!(
            "MEASURE edges path modules={} {:?} {}",
            modules.len(),
            t.elapsed(),
            rss()
        );
        assert_eq!(
            names, modules,
            "the edge-free module set differs from the edge path's"
        );
    }
}

/// THE PARTITION-MOVE EXPLANATION CROSSES A PROCESS BOUNDARY AS ONE TYPED ANSWER
/// (`PartitionMoveExplanationReceipt`, its only definition). The spawn itself needs the real
/// `claim_executor` binary and is exercised by the heal round; these pin the carrier, the readback's
/// refusals, and the child entry's write.
#[cfg(test)]
mod partition_move_explanation_tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gunbc-partition-move-{name}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.join("answer.json")
    }

    #[test]
    fn the_three_answers_round_trip_and_stay_distinct() {
        let schema = PARTITION_MOVE_EXPLANATION_SCHEMA.to_string();
        for answer in [
            PartitionMoveExplanationReceipt::Explained {
                schema: schema.clone(),
            },
            PartitionMoveExplanationReceipt::Unexplained {
                schema: schema.clone(),
            },
            PartitionMoveExplanationReceipt::Refused {
                schema: schema.clone(),
                reason: "diff range unreadable".to_string(),
            },
        ] {
            let path = scratch("roundtrip");
            fs::write(&path, serde_json::to_vec(&answer).unwrap()).unwrap();
            assert_eq!(
                read_child_answer::<PartitionMoveExplanationReceipt>(&path).unwrap(),
                answer
            );
        }
    }

    /// RED: no answer, an unparseable answer, and an answer in another schema all refuse.
    #[test]
    fn an_absent_garbled_or_foreign_answer_refuses() {
        let path = scratch("refusals");
        let _ = fs::remove_file(&path);
        assert!(read_child_answer::<PartitionMoveExplanationReceipt>(&path)
            .unwrap_err()
            .contains("wrote no answer"));
        fs::write(&path, b"{\"answer\":\"maybe\"}").unwrap();
        assert!(read_child_answer::<PartitionMoveExplanationReceipt>(&path)
            .unwrap_err()
            .contains("refusal: parse"));
        fs::write(
            &path,
            serde_json::to_vec(&PartitionMoveExplanationReceipt::Explained {
                schema: "gunbc.regen_partition_move_explanation.v0".to_string(),
            })
            .unwrap(),
        )
        .unwrap();
        assert!(read_child_answer::<PartitionMoveExplanationReceipt>(&path)
            .unwrap_err()
            .contains("not gunbc.regen_partition_move_explanation.v1"));
    }

    /// The child's real entry writes whatever it answered -- here, with no diff range bound in a test
    /// process, a typed refusal -- and the readback returns exactly that.
    #[test]
    fn the_child_entry_writes_the_answer_it_returns() {
        let rel = format!(
            "target/test-partition-move-explanation-{}.json",
            std::process::id()
        );
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| workspace_root().join(r).to_string_lossy().into_owned())
            .collect();
        let answered = run_partition_move_explanation(&rel, &roots).unwrap();
        let path = workspace_root().join(&rel);
        assert_eq!(
            read_child_answer::<PartitionMoveExplanationReceipt>(&path).unwrap(),
            answered
        );
        let _ = fs::remove_file(&path);
    }
}

/// THE ORCHESTRATOR LOADS ITS MODELS FROM THEIR OWN CLOSURES, never through the whole-tree shared
/// index. The discriminator is the thread's shared-index memo (`PROCESS_RESOLVE_INDEX` is
/// thread-local and each test runs on its own thread): replaying the re-exec'd orchestrator's loads
/// in PRODUCTION ORDER -- the liveness model, the convergence model, the round-cost decider -- must
/// leave it empty, and a shared-index build over even a tiny root must not.
#[cfg(test)]
mod orchestrator_model_load_tests {
    use super::*;

    fn roots() -> Vec<String> {
        ["dag", "src/v2"]
            .iter()
            .map(|r| workspace_root().join(r).to_string_lossy().into_owned())
            .collect()
    }

    /// Replace every `OccurrenceId { value: N }` by its order of first appearance in `text`.
    fn canonical_occurrences(text: &str) -> String {
        const MARK: &str = "OccurrenceId { value: ";
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(at) = rest.find(MARK) {
            out.push_str(&rest[..at + MARK.len()]);
            rest = &rest[at + MARK.len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            let next = seen.len();
            let canonical = *seen.entry(digits.clone()).or_insert(next);
            out.push_str(&format!("#{canonical}"));
            rest = &rest[digits.len()..];
        }
        out.push_str(rest);
        out
    }

    fn first_divergence(a: &str, b: &str) -> String {
        let at = a
            .bytes()
            .zip(b.bytes())
            .position(|(x, y)| x != y)
            .unwrap_or(a.len().min(b.len()));
        let window = |s: &str| s[at.saturating_sub(80)..(at + 120).min(s.len())].to_string();
        format!("byte {at}: {:?} vs {:?}", window(a), window(b))
    }

    /// The renumbering is a bijection test, not a blur: two texts that differ in which occurrences
    /// are the SAME still compare unequal.
    #[test]
    fn occurrence_canonicalization_keeps_sharing_and_forgets_only_numbering() {
        let a = "x OccurrenceId { value: 6877 } y OccurrenceId { value: 6877 } z OccurrenceId { value: 9 }";
        let renumbered = "x OccurrenceId { value: 1276332 } y OccurrenceId { value: 1276332 } z OccurrenceId { value: 5 }";
        let unshared =
            "x OccurrenceId { value: 1 } y OccurrenceId { value: 2 } z OccurrenceId { value: 3 }";
        assert_eq!(canonical_occurrences(a), canonical_occurrences(renumbered));
        assert_ne!(canonical_occurrences(a), canonical_occurrences(unshared));
    }

    fn carried_subject() -> RegenConvergenceCheckpointSubject {
        RegenConvergenceCheckpointSubject {
            starting_commit: "c0ffee".to_string(),
            source_authority_digest: "sha256:source".to_string(),
            stage_plan_authority_digest: "sha256:plan".to_string(),
        }
    }

    /// EVERYTHING THE ORCHESTRATOR DOES IN-PROCESS BEFORE GENERATION 0, in production order
    /// (`run_regen_round_cost`): the liveness model, the round subject, the convergence model, the
    /// round-cost decider, then the generation-begin admission. None of it may start a whole-pool
    /// index build on this thread. The previous version of this replay listed only the model loads
    /// and missed the checkpoint subject, which run 41899288377 then measured in production (7.57
    /// GB at generation 0). The subject child itself needs the real `claim_executor` and runs in
    /// the heal round; here the ask is counted, and both images' routes are driven.
    #[test]
    fn nothing_the_orchestrator_does_before_generation_0_builds_a_whole_pool_index() {
        let roots = roots();
        let builds_before = super::super::entry_resolve::whole_pool_index_builds_on_this_thread();
        let _liveness = SupervisedSubjectModel::load(&roots).expect("liveness model");
        // The image that owns a base-mirror round establishes the subject (the child, counted)...
        let mut owned_slot = None;
        let mut asks = 0;
        let owned = establish_round_subject(&mut owned_slot, None, || {
            asks += 1;
            Ok(carried_subject())
        })
        .unwrap()
        .clone();
        let _ = establish_round_subject(&mut owned_slot, None, || {
            panic!("an established subject is never produced again")
        })
        .unwrap();
        assert_eq!(
            asks, 1,
            "the owning image produces the subject exactly once"
        );
        // ...and the re-exec'd image takes the carried value and asks nobody.
        let carry = SeedHandoff {
            cost: SeedBuildCost {
                compiled_crates: 1,
                wall_ms: 1,
                cpu_ms: None,
            },
            source: "S".to_string(),
            built_exe: "B".to_string(),
            round_subject: Some(owned.clone()),
        };
        let mut reexecd_slot = SeedHandoff::decode(&carry.encode()).unwrap().round_subject;
        let reexecd = establish_round_subject(&mut reexecd_slot, None, || {
            panic!("the re-exec'd image must not produce the subject again")
        })
        .unwrap()
        .clone();
        assert_eq!(reexecd, owned);
        let convergence = RegenConvergenceModel::load(&roots).expect("convergence model");
        let entry = round_cost_entry(&roots).unwrap();
        let _decider = resolve_orchestrator_model(&roots, &entry).expect("round-cost decider");
        // The loaded model evaluates through the same bridge production uses.
        assert_eq!(
            admit_generation_begin_readings(&convergence, 1, 900_000_000, 900_000_000),
            Ok(())
        );
        assert_eq!(
            super::super::entry_resolve::whole_pool_index_builds_on_this_thread() - builds_before,
            0,
            "something the orchestrator does before generation 0 built a whole-pool index"
        );
        assert_eq!(
            super::super::entry_resolve::shared_index_residency_control(),
            Ok(0),
            "an orchestrator model load built the whole-tree shared index"
        );
    }

    /// RED for the discriminator, on the SUBJECT PRODUCER'S OWN ROUTE: `regen_input_sources`, which
    /// the checkpoint subject's digest calls unconditionally, is seen as a whole-pool build -- here
    /// over a one-module fixture root, because the real corpus costs about 7.5 GB.
    #[test]
    fn the_subject_producers_closure_build_is_seen_by_the_counter() {
        let root = workspace_root().join(format!(
            "target/gunbc-subject-producer-red-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("probe")).unwrap();
        fs::write(
            root.join("probe/fixture.dag"),
            "module probe.fixture\n\nimport std.types { Int }\n\ndata fixture_value: Int = 1\n",
        )
        .unwrap();
        let before = super::super::entry_resolve::whole_pool_index_builds_on_this_thread();
        let sources = super::super::regen_input_sources_over_roots(
            &root,
            &[root.to_string_lossy().into_owned()],
        );
        let builds = super::super::entry_resolve::whole_pool_index_builds_on_this_thread() - before;
        let _ = fs::remove_dir_all(&root);
        assert!(sources.is_ok(), "the fixture closure resolves: {sources:?}");
        assert!(
            builds >= 1,
            "the counter did not see the subject producer's whole-pool build"
        );
    }

    /// RED for the discriminator: a shared-index build, here over a one-module fixture root, is seen.
    #[test]
    fn a_shared_index_build_is_seen_by_the_residency_read() {
        // Under the workspace: a shared-index root must be one, or key normalization refuses it.
        let root = workspace_root().join(format!(
            "target/gunbc-shared-index-red-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("probe")).unwrap();
        fs::write(
            root.join("probe/fixture.dag"),
            "module probe.fixture\n\nimport std.types { Int }\n\ndata fixture_value: Int = 1\n",
        )
        .unwrap();
        let fixture_roots = vec![root.to_string_lossy().into_owned()];
        let before = super::super::entry_resolve::whole_pool_index_builds_on_this_thread();
        let _ = super::super::entry_resolve::try_process_shared_index(&fixture_roots);
        let builds = super::super::entry_resolve::whole_pool_index_builds_on_this_thread() - before;
        let resident = super::super::entry_resolve::shared_index_residency_control();
        let _ = fs::remove_dir_all(&root);
        assert!(
            matches!(resident, Ok(n) if n >= 1),
            "the residency read did not see a shared-index build: {resident:?}"
        );
        assert!(builds >= 1, "the counter did not see a shared-index build");
    }

    /// EQUIVALENCE, PER SITE: the demand-acquired route and the shared-index route resolve each
    /// orchestrator entry to the SAME graph -- the same modules, the same item registry with every
    /// item's resolved content, the same diagnostics -- and the model functions the five sites call
    /// answer identically on fixed inputs. Two resolutions that could diverge silently would be two
    /// authorities for one model (DESIGN section 3). The shared-index side costs about 169 s and
    /// 10 GB, so this runs on demand: --ignored --test-threads=1 on a release build.
    #[test]
    #[ignore]
    fn both_routes_resolve_every_orchestrator_entry_identically() {
        use crate::v1_interpreter::{self, ExecutionMode, Value};
        let roots = roots();
        let entries = [
            Path::new(&roots[1]).join("workflow/regen_convergence_transaction.dag"),
            Path::new(&roots[0]).join("gunbc/host/memory_stall_refusal.dag"),
            PathBuf::from(round_cost_entry(&roots).unwrap()),
            // THE SITE THAT FORKED. This entry resolved over the whole-tree route while every
            // site above was on the demand-acquired one, and it ran inside the orchestrator
            // between generations -- ~10 GB, which is why a round that needed a second generation
            // was refused by GenerationStateRetainedByOrchestrator. Its rows feed the partition
            // boundary emit and reach generated bytes, so a divergence between the routes here
            // would be a divergence in the seed, not only in a log line.
            Path::new(&roots[0]).join("gunbc/stage0/stage0_emitted_edge_admission.dag"),
        ];
        let shared = super::super::process_shared_index_or_refuse(&roots);
        for entry in &entries {
            let entry = entry.to_string_lossy().into_owned();
            let (demand, demand_si) = resolve_orchestrator_model(&roots, &entry).unwrap();
            let (whole, whole_si) =
                super::super::resolve_entry_with_index(&shared, &entry).unwrap();
            // EQUAL UP TO OCCURRENCE NUMBERING. Occurrence identities are minted from a per-process
            // counter (std.occurrence_identity), so the route that resolved a whole tree first numbers
            // the same occurrences from a different base (measured: 6877 against 1276332 for one
            // binder). Both graphs are therefore compared after one consistent renumbering -- each
            // distinct id replaced by its order of first appearance across the WHOLE rendering -- which
            // holds exactly when a single bijection maps one graph's occurrences onto the other's.
            let registry = |g: &crate::v1_compiler_compile::ResolvedGraph| {
                let mut keys: Vec<&String> = g.item_registry.keys().collect();
                keys.sort();
                canonical_occurrences(
                    &keys
                        .iter()
                        .map(|k| format!("{k} => {:?}", g.item_registry[*k]))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            };
            let (demand_items, whole_items) = (registry(&demand), registry(&whole));
            assert!(
                demand_items == whole_items,
                "{entry}: the two routes registered different items (first divergence: {})",
                first_divergence(&demand_items, &whole_items)
            );
            let modules = |g: &crate::v1_compiler_compile::ResolvedGraph| {
                let mut rendered: Vec<String> = g
                    .modules
                    .iter()
                    .map(|m| canonical_occurrences(&format!("{m:?}")))
                    .collect();
                rendered.sort();
                rendered
            };
            assert!(
                modules(&demand) == modules(&whole),
                "{entry}: the two routes resolved different modules"
            );
            let diags = |g: &crate::v1_compiler_compile::ResolvedGraph| {
                let mut d: Vec<String> = g
                    .diagnostics
                    .iter()
                    .map(|x| canonical_occurrences(&format!("{x:?}")))
                    .collect();
                d.sort();
                d
            };
            assert!(
                diags(&demand) == diags(&whole),
                "{entry}: diagnostics differ"
            );
            // Behaviour on fixed inputs: the functions the sites call, both contexts.
            let answer =
                |g: &crate::v1_compiler_compile::ResolvedGraph,
                 si: Rc<im::HashMap<String, Rc<crate::v1_compiler_compile::NewlineIndex>>>,
                 function: &str,
                 args: &dyn Fn(&v1_interpreter::InterpContext) -> Vec<(Option<String>, Value)>|
                 -> String {
                    let ctx = super::super::make_eval_context(g, si, ExecutionMode::Hermetic);
                    let args = args(&ctx);
                    format!(
                        "{:?}",
                        v1_interpreter::with_active_context(&ctx, || {
                            v1_interpreter::run_in_context_with_args(&ctx, function, &args, false)
                        })
                    )
                };
            let measure = |ctx: &v1_interpreter::InterpContext, n: i64| Value::Record {
                type_name: ctx.sym("Measure"),
                fields: Rc::new(vec![(ctx.sym("count"), Value::Int(n))]),
            };
            let probes: Vec<(
                &str,
                Box<dyn Fn(&v1_interpreter::InterpContext) -> Vec<(Option<String>, Value)>>,
            )> = if entry.ends_with("regen_convergence_transaction.dag") {
                vec![
                    (
                        "regen_admit_generation_begin",
                        Box::new(move |ctx| {
                            vec![
                                (Some("ordinal".to_string()), Value::Int(2)),
                                (
                                    Some("resident_at_first".to_string()),
                                    measure(ctx, 900_000_000),
                                ),
                                (
                                    Some("resident_now".to_string()),
                                    measure(ctx, 15_636_525_056),
                                ),
                            ]
                        }),
                    ),
                    (
                        "regen_admit_generation_begin",
                        Box::new(move |ctx| {
                            vec![
                                (Some("ordinal".to_string()), Value::Int(1)),
                                (
                                    Some("resident_at_first".to_string()),
                                    measure(ctx, 10_692_333_568),
                                ),
                                (
                                    Some("resident_now".to_string()),
                                    measure(ctx, 12_602_740_736),
                                ),
                            ]
                        }),
                    ),
                ]
            } else if entry.ends_with("memory_stall_refusal.dag") {
                let reading = move |ctx: &v1_interpreter::InterpContext,
                                    faults: i64,
                                    cpu: i64,
                                    events: i64,
                                    full: i64| {
                    Value::Record {
                        type_name: ctx.sym("SupervisedSlotReading"),
                        fields: Rc::new(vec![
                            (ctx.sym("major_faults"), Value::Int(faults)),
                            (ctx.sym("user_cpu"), measure(ctx, cpu)),
                            (ctx.sym("envelope_events"), Value::Int(events)),
                            (ctx.sym("pressure_body"), v1_interpreter::str_value(format!(
                                "some avg10=0.00 avg60=0.00 avg300=0.00 total={}\nfull avg10=0.00 avg60=0.00 avg300=0.00 total={full}\n",
                                full + 5
                            ))),
                        ]),
                    }
                };
                vec![(
                    "supervised_subject_window_between",
                    Box::new(move |ctx| {
                        vec![
                            (
                                Some("base".to_string()),
                                reading(ctx, 100, 2000, 7, 1_000_000),
                            ),
                            (
                                Some("now".to_string()),
                                reading(ctx, 160, 2500, 419, 1_000_000 + 900_001 * 800),
                            ),
                            (Some("wall".to_string()), measure(ctx, 900_001)),
                        ]
                    }),
                )]
            } else if entry.ends_with("stage0_emitted_edge_admission.dag") {
                // NO ARGUMENTS: the rows are the emitted tree's own. Compared RAW, un-renumbered,
                // like the other arms -- these are the rows the boundary emit digests into
                // generated bytes, so an occurrence identity reaching one would change the seed
                // between routes and must fail this claim rather than be renumbered away.
                vec![("generated_partition_crate_rows", Box::new(|_ctx| vec![]))]
            } else {
                vec![]
            };
            // RAW equality here, deliberately NOT renumbered: these are the values the five sites hand
            // on (labels, verdict and refusal texts, decisions) and some reach persisted receipts, so
            // an occurrence identity leaking into one would change those bytes between routes and
            // must fail this claim rather than be renumbered away.
            for (function, args) in &probes {
                let (on_demand, on_whole) = (
                    answer(&demand, demand_si.clone(), function, args.as_ref()),
                    answer(&whole, whole_si.clone(), function, args.as_ref()),
                );
                assert!(
                    !on_demand.contains("OccurrenceId"),
                    "{entry}: {function} hands on an occurrence identity: {on_demand}"
                );
                assert_eq!(
                    on_demand, on_whole,
                    "{entry}: {function} answered differently on the two routes"
                );
            }
            eprintln!(
                "EQUIVALENT (up to occurrence numbering) {entry}: {} modules, {} items, {} diagnostics, {} behavioural probes",
                demand.modules.len(),
                demand.item_registry.len(),
                demand.diagnostics.len(),
                probes.len()
            );
        }
    }

    /// The footprint, in production order, on a release build in a fresh process: run alone with
    /// --ignored --nocapture --test-threads=1. Prints the readings the RFM row cites.
    #[test]
    #[ignore]
    fn measure_the_orchestrators_model_loads_in_production_order() {
        let rss = || {
            fs::read_to_string("/proc/self/status")
                .unwrap()
                .lines()
                .find(|l| l.starts_with("VmRSS"))
                .unwrap()
                .to_string()
        };
        let roots = roots();
        eprintln!("MEASURE start {}", rss());
        let t = Instant::now();
        let _liveness = SupervisedSubjectModel::load(&roots).unwrap();
        let _convergence = RegenConvergenceModel::load(&roots).unwrap();
        let entry = round_cost_entry(&roots).unwrap();
        let _decider = resolve_orchestrator_model(&roots, &entry).unwrap();
        eprintln!(
            "MEASURE after the three orchestrator loads {:?} {}",
            t.elapsed(),
            rss()
        );
    }
}

/// THE ROUND'S CHECKPOINT SUBJECT IS ONE CARRIED VALUE: it crosses the re-exec in the handoff (and
/// an old handoff without it refuses), every generation receipt is checked against it, and its
/// child's answer refuses when absent, garbled or foreign.
/// A fixture tree's stage plan: the checkpoint subject's cheap facts are read live at every journal
/// boundary, so a fixture that journals needs a real plan file at the production path.
#[cfg(test)]
fn write_fixture_stage_plan(root: &Path) {
    let plan = root.join(STAGE_PLAN_AUTHORITY_REL);
    fs::create_dir_all(plan.parent().unwrap()).unwrap();
    fs::write(&plan, "module v2.workflow.regen_convergence_transaction\n").unwrap();
}

#[cfg(test)]
mod round_checkpoint_subject_tests {
    use super::*;

    fn git_fixture(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("gunbc-live-facts-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        write_fixture_stage_plan(&root);
        fs::write(root.join("file.rs"), "before\n").unwrap();
        for args in [
            vec!["init", "-q", "--initial-branch=main"],
            vec!["add", "-A"],
            vec![
                "-c",
                "user.email=f@f",
                "-c",
                "user.name=f",
                "commit",
                "-qm",
                "fixture",
            ],
        ] {
            let out = Command::new("git")
                .args(&args)
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        root
    }

    fn live_subject(root: &Path) -> RegenConvergenceCheckpointSubject {
        let (starting_commit, stage_plan_authority_digest) =
            checkpoint_subject_live_facts(root).unwrap();
        RegenConvergenceCheckpointSubject {
            starting_commit,
            source_authority_digest: "sha256:source".to_string(),
            stage_plan_authority_digest,
        }
    }

    /// ALL THREE CARRIED FACTS ARE RE-OBSERVED. The cheap two at every journal write and restore:
    /// the live subject journals; a moved HEAD and an edited stage plan each refuse, at the write
    /// and at the restore. (The source digest is the generation check, below.)
    #[test]
    fn a_moved_head_or_an_edited_stage_plan_refuses_at_journal_write_and_restore() {
        let root = git_fixture("cheap");
        let subject = live_subject(&root);
        let entries = vec![("file.rs".to_string(), root.join("file.rs"))];
        journal_paths_for_subject(&root, &entries, &subject).expect("the live subject journals");

        let moved = RegenConvergenceCheckpointSubject {
            starting_commit: "0000000000000000000000000000000000000000".to_string(),
            ..subject.clone()
        };
        let edited = RegenConvergenceCheckpointSubject {
            stage_plan_authority_digest: "sha256:another-plan".to_string(),
            ..subject.clone()
        };
        for stale in [&moved, &edited] {
            let write = journal_paths_for_subject(&root, &entries, stale).unwrap_err();
            assert!(
                write.starts_with("CheckpointSubjectDriftedMidRound: at a checkpoint boundary"),
                "{write}"
            );
            let restore = restore_regen_convergence_journal_for_subject(&root, stale).unwrap_err();
            assert!(
                restore.starts_with("CheckpointSubjectDriftedMidRound: at a checkpoint boundary"),
                "{restore}"
            );
        }
        // Positive control: the live subject still restores its own journal.
        restore_regen_convergence_journal_for_subject(&root, &subject)
            .expect("the live subject restores");
        let _ = fs::remove_dir_all(&root);
    }

    fn subject(source: &str) -> RegenConvergenceCheckpointSubject {
        RegenConvergenceCheckpointSubject {
            starting_commit: "c0ffee".to_string(),
            source_authority_digest: source.to_string(),
            stage_plan_authority_digest: "sha256:plan".to_string(),
        }
    }

    fn handoff() -> SeedHandoff {
        SeedHandoff {
            cost: SeedBuildCost {
                compiled_crates: 3,
                wall_ms: 4,
                cpu_ms: Some(5),
            },
            source: "T".to_string(),
            built_exe: "B".to_string(),
            round_subject: Some(subject("sha256:source")),
        }
    }

    #[test]
    fn the_handoff_carries_the_subject_or_its_absence_and_round_trips() {
        let carry = handoff();
        assert_eq!(SeedHandoff::decode(&carry.encode()).unwrap(), carry);
        let none = SeedHandoff {
            round_subject: None,
            ..handoff()
        };
        assert_eq!(SeedHandoff::decode(&none.encode()).unwrap(), none);
    }

    /// AT MOST ONE PRODUCTION, AT THE FIRST DEMAND: a round that never demands it produces nothing;
    /// a late establishment is checked against the generation current at that moment.
    #[test]
    fn the_subject_is_produced_once_at_first_demand_and_checked_when_late() {
        let mut slot = None;
        assert!(slot.is_none(), "nothing is produced before a demand");
        let mut asks = 0;
        let produced = establish_round_subject(&mut slot, Some((2, "sha256:source")), || {
            asks += 1;
            Ok(subject("sha256:source"))
        })
        .unwrap()
        .clone();
        assert_eq!(produced, subject("sha256:source"));
        let again = establish_round_subject(&mut slot, Some((3, "sha256:source")), || {
            asks += 1;
            Ok(subject("sha256:other"))
        })
        .unwrap()
        .clone();
        assert_eq!((asks, again), (1, produced));
        // RED: produced late, disagreeing with the current generation's reading.
        let mut late = None;
        let refused = establish_round_subject(&mut late, Some((2, "sha256:edited")), || {
            Ok(subject("sha256:source"))
        })
        .unwrap_err();
        assert!(
            refused.starts_with("CheckpointSubjectDriftedMidRound: generation 2 "),
            "{refused}"
        );
        assert!(late.is_none(), "a refused production establishes nothing");
    }

    /// No journal restores nothing without a subject; a journal with no established subject refuses.
    #[test]
    fn a_journal_without_an_established_subject_refuses_at_restore() {
        let root = std::env::temp_dir().join(format!("gunbc-round-journal-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let nothing = restore_round_journal(&root, None).expect("no journal restores nothing");
        assert!(nothing.rewritten.is_empty() && nothing.removed.is_empty());
        let journal = regen_convergence_journal_path(&root);
        fs::create_dir_all(&journal).unwrap();
        fs::write(journal.join("journal.json"), b"{}").unwrap();
        let refused = restore_round_journal(&root, None).unwrap_err();
        let _ = fs::remove_dir_all(&root);
        assert!(
            refused.starts_with("CheckpointJournalWithoutRoundSubject"),
            "{refused}"
        );
    }

    /// RED: a handoff in the previous encoding (no subject) refuses at decode with its own typed
    /// arm naming the missing field and the remedy; a truncated subject and a trailing field stay
    /// `HandoffUndecodable`. None is read as a cue to recompute.
    #[test]
    fn a_handoff_without_the_subject_refuses() {
        let old = "crates=3;wall_ms=4;cpu_ms=5;source=T;built_exe=B";
        let refused = SeedHandoff::decode(old)
            .err()
            .expect("the old encoding must refuse");
        assert_eq!(
            refused,
            SeedHandoffRefusal::HandoffPredatesRoundSubject {
                missing_field: "subject",
                value: old.to_string(),
            }
        );
        let rendered = refused.to_string();
        assert!(
            rendered.starts_with("refusal: HandoffPredatesRoundSubject: ")
                && rendered.contains("predates the round-subject handoff")
                && rendered.contains("merge the base into the head"),
            "{rendered}"
        );
        // An old encoding that carried an unreadable cpu reading is the same arm.
        assert!(matches!(
            SeedHandoff::decode("crates=3;wall_ms=4;cpu_ms=unreadable;source=T;built_exe=B"),
            Err(SeedHandoffRefusal::HandoffPredatesRoundSubject { .. })
        ));
        // Ending earlier than the subject field is not this arm: it is not a well-formed old handoff.
        assert!(matches!(
            SeedHandoff::decode("crates=3;wall_ms=4;cpu_ms=5;source=T"),
            Err(SeedHandoffRefusal::HandoffUndecodable { .. })
        ));
        let truncated = "crates=3;wall_ms=4;cpu_ms=5;source=T;built_exe=B;subject=present";
        assert!(matches!(
            SeedHandoff::decode(truncated),
            Err(SeedHandoffRefusal::HandoffUndecodable { .. })
        ));
        let extra = format!("{};extra=1", handoff().encode());
        assert!(matches!(
            SeedHandoff::decode(&extra),
            Err(SeedHandoffRefusal::HandoffUndecodable { .. })
        ));
    }

    /// The check runs for EVERY generation (one call site, inside `run_built_seed_regen`): equal
    /// digests admit at any ordinal, and a drifted digest refuses at generation 0 and at a later one.
    #[test]
    fn every_generation_receipt_is_checked_against_the_carried_subject() {
        let carried = subject("sha256:source");
        for ordinal in [0usize, 1, 3] {
            assert_eq!(
                admit_generation_authority(&carried, ordinal, "sha256:source"),
                Ok(())
            );
        }
        for ordinal in [0usize, 3] {
            let refused =
                admit_generation_authority(&carried, ordinal, "sha256:edited").unwrap_err();
            assert!(
                refused.starts_with("CheckpointSubjectDriftedMidRound: generation ")
                    && refused.contains(&format!("generation {ordinal} "))
                    && refused.contains("sha256:edited")
                    && refused.contains("sha256:source"),
                "{refused}"
            );
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gunbc-checkpoint-subject-{name}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.join("answer.json")
    }

    #[test]
    fn the_subject_answer_round_trips_and_its_refusal_stays_distinct() {
        let schema = CHECKPOINT_SUBJECT_SCHEMA.to_string();
        for answer in [
            CheckpointSubjectReceipt::Answered {
                schema: schema.clone(),
                subject: subject("sha256:source"),
            },
            CheckpointSubjectReceipt::Refused {
                schema: schema.clone(),
                reason: "corpus unreadable".to_string(),
            },
        ] {
            let path = scratch("roundtrip");
            fs::write(&path, serde_json::to_vec(&answer).unwrap()).unwrap();
            assert_eq!(
                read_child_answer::<CheckpointSubjectReceipt>(&path).unwrap(),
                answer
            );
        }
    }

    /// RED: no answer, an unparseable answer, an answer missing a subject field, and an answer in
    /// another schema all refuse.
    #[test]
    fn an_absent_garbled_partial_or_foreign_subject_answer_refuses() {
        let path = scratch("refusals");
        let _ = fs::remove_file(&path);
        assert!(read_child_answer::<CheckpointSubjectReceipt>(&path)
            .unwrap_err()
            .contains("checkpoint subject child wrote no answer"));
        fs::write(&path, b"{\"answer\":\"maybe\"}").unwrap();
        assert!(read_child_answer::<CheckpointSubjectReceipt>(&path)
            .unwrap_err()
            .contains("refusal: parse"));
        fs::write(
            &path,
            br#"{"answer":"answered","schema":"gunbc.regen_checkpoint_subject.v1","subject":{"starting_commit":"c","source_authority_digest":"s"}}"#,
        )
        .unwrap();
        assert!(read_child_answer::<CheckpointSubjectReceipt>(&path)
            .unwrap_err()
            .contains("refusal: parse"));
        fs::write(
            &path,
            serde_json::to_vec(&CheckpointSubjectReceipt::Answered {
                schema: "gunbc.regen_checkpoint_subject.v0".to_string(),
                subject: subject("sha256:source"),
            })
            .unwrap(),
        )
        .unwrap();
        assert!(read_child_answer::<CheckpointSubjectReceipt>(&path)
            .unwrap_err()
            .contains("not gunbc.regen_checkpoint_subject.v1"));
    }
}
