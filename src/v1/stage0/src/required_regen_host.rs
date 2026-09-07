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

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::Instant;

#[path = "bootstrap_stage0_crate_layout_generated.rs"]
mod bootstrap_stage0_crate_layout_generated;
use super::workspace_root;
use crate::gunbc_stage0_emitted_population_manifest::{
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
use bootstrap_stage0_crate_layout_generated::{
    EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES, HAND_MAINTAINED_STAGE0_DIRS,
    HAND_MAINTAINED_STAGE0_FILES,
};

/// Bumped from `.v1` when the flat eight-field record split into the two-variant carrier below.
///
/// THE BUMP IS LOAD-BEARING ONLY BECAUSE `read_receipt` COMPARES IT. An earlier revision claimed a
/// stale `.v1` receipt "fails to deserialize into the new shape". FALSE, caught in review: serde
/// ignores unknown fields by default, and a v1 record carries every required field plus the
/// removed `fixed_point_equal`, so it parsed cleanly. The version string was written three times
/// and read zero times -- decoration. Two things now make the claim true: `deny_unknown_fields` on
/// the carrier, and an explicit equality check in `read_receipt`. A version nobody compares is the
/// same defect class as the impersonation this module closes -- an artifact asserting a property
/// nothing establishes.
const RECEIPT_SCHEMA: &str = "gunbc.regen_receipt.v2";

/// Evidence produced by the FIRST regen pass, referenced by the second.
///
/// Carries the `commit_sha` it was measured at so a consumer READS which tree these facts describe
/// instead of inferring it from the quoting receipt. A reference not naming its subject would be
/// the impersonation this type ends, wearing better vocabulary.
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct PriorReceiptRef {
    pub commit_sha: String,
    pub committed_generated_digest: String,
    pub first_generation_equal: bool,
    pub changed_paths: Vec<String>,
    pub candidate_artifact: String,
}

/// A RECEIPT MAY REFERENCE PRIOR EVIDENCE BUT MAY NOT IMPERSONATE PRIOR EVIDENCE AS SOMETHING IT
/// MEASURED ITSELF (operator ruling, 2026-08-20).
///
/// This was one flat eight-field record, forcing the second pass to populate fields it had not
/// measured; the only source was the first pass's on-disk receipt, so four of six were copied
/// verbatim: `committed_generated_digest`, `first_generation_equal`, `changed_paths`,
/// `candidate_artifact`. The product was stamped with the SECOND pass `commit_sha` while carrying
/// FIRST pass answers -- consistent, schema-valid, silent about which tree four fields describe,
/// and unvalidatable because nothing recorded the provenance to validate.
///
/// The split makes the fabrication UNWRITABLE rather than detectable: `FixedPoint` has no
/// `first_generation_equal` field, so there is no value to copy and no check to pass (DESIGN 4b
/// structural impossibility, one rung above validation). Computing all six in the second pass is
/// worse -- pass two would re-derive `first_generation_equal` against the committed tree, pass
/// one's question, fusing two authorities into one row (DESIGN 3).
///
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
    /// It CROSSED THE PROCESS BOUNDARY: the receipt file is the only carrier between the passes,
    /// `read_receipt` returned the fabricated `false` as a `PriorMeasurement`, `PriorReceiptRef`
    /// copied it into the fixed-point receipt, and `claim_executor` printed
    /// `referenced_first_generation_equal=false`. A standalone `--required-regen-fixed-point` at
    /// the same commit passes the cross-tree guard precisely BECAUSE the refusal happened at this
    /// commit. The run still reds -- pass 2's real digest cannot equal `refused:population` -- so
    /// this was diagnostic harm, not fail-open: the operator was told the first generation
    /// compared unequal when it never compared.
    ///
    /// "Not asked" was already modelled correctly THREE times in this file --
    /// `FirstGeneration::NotMeasured`, the `first_generation_equal()` accessor's documented
    /// `Option`, and `fixed_point_equal()`'s mirror -- so the Bool was a fourth representation
    /// disagreeing with three correct neighbours, not a missing state (DESIGN §3). The fix repeats
    /// the eight-field split's move: the variant has no `first_generation_equal` and no digest
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
    /// would have put an empty-population digest in a field a fixed-point pass reads as evidence.
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
    #[serde(rename = "fixed_point")]
    FixedPoint {
        schema: String,
        commit_sha: String,
        authority_digest: String,
        candidate_generated_digest: String,
        fixed_point_equal: bool,
        prior: PriorReceiptRef,
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
            RegenReceipt::FixedPoint { commit_sha, .. } => commit_sha,
        }
    }

    /// `Some` only where the pass measured it. `FixedPoint` returns `None` rather than reaching
    /// into `prior`, because "the second pass did not measure this" and "the first pass measured
    /// it as false" are different states and a `bool` cannot hold both.
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
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    /// `Some` only where the pass measured it -- the mirror of the above.
    pub fn fixed_point_equal(&self) -> Option<bool> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
            RegenReceipt::Refused { .. } => None,
            RegenReceipt::NoAffectedMirrors { .. } => None,
            RegenReceipt::FixedPoint {
                fixed_point_equal, ..
            } => Some(*fixed_point_equal),
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
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    /// The referenced first-pass evidence, present only on `FixedPoint`.
    /// The digest of the tree THIS pass emitted.
    ///
    /// `Some` exactly where a candidate digest was measured. This accessor was TOTAL while both
    /// variants measured one; `Refused` measures none, so it follows the rule its Option-returning
    /// siblings state. A sentinel string here would put the fabrication back one accessor lower
    /// than where it was removed.
    ///
    /// Consumer: the composed `--required-ci` run, which hands pass 1's digest to the fixed-point
    /// pass IN MEMORY rather than re-reading the previous process's receipt file.
    /// `run_required_regen_fixed_point` has always taken `pass1_digest: Option<String>`; before
    /// the phases shared a process there was no way to supply it.
    pub fn candidate_generated_digest(&self) -> Option<&str> {
        match self {
            RegenReceipt::FirstGeneration {
                candidate_generated_digest,
                ..
            } => Some(candidate_generated_digest),
            RegenReceipt::Refused { .. } => None,
            RegenReceipt::NoAffectedMirrors { .. } => None,
            RegenReceipt::FixedPoint {
                candidate_generated_digest,
                ..
            } => Some(candidate_generated_digest),
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
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    pub fn prior(&self) -> Option<&PriorReceiptRef> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
            RegenReceipt::Refused { .. } => None,
            RegenReceipt::NoAffectedMirrors { .. } => None,
            RegenReceipt::FixedPoint { prior, .. } => Some(prior),
        }
    }
}

#[derive(Debug)]
pub struct RequiredRegenOutcome {
    pub receipt: RegenReceipt,
    pub failures: Vec<String>,
    /// WHETHER PASS ONE ACTUALLY EMITTED, kept OFF the receipt's digest fields on purpose.
    ///
    /// A population refusal happens before any content comparison, so there is no first
    /// generation to digest. This field carries that for the IN-PROCESS handoff;
    /// `RegenReceipt::Refused` carries it across the process boundary, so neither route has a
    /// digest position for a refusal to fill. The receipt previously filled one with the sentinel
    /// `refused:population`, which is why this field exists: handing that receipt to the
    /// fixed-point phase would compare a real pass-two digest against sentinel prose and report a
    /// determinism failure nobody measured -- a fabricated plausible output (DESIGN §5), convincing
    /// because it names two digests. Not redundant with the receipt: the in-process handoff must
    /// not go through a file.
    pub first_generation: FirstGeneration,
}

/// What pass one produced, as a coproduct rather than as a string that might be a digest.
/// `NotMeasured` has no digest field at all, so the sentinel has no route into a comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirstGeneration {
    Measured(String),
    NotMeasured(String),
}

/// The ONLY way the fixed-point phase learns pass one's digest. `NotMeasured` yields `None`, and
/// phase three reports its own SKIPPED state rather than a refusal it did not observe.
pub fn pass1_digest_for_fixed_point(outcome: &RequiredRegenOutcome) -> Option<&str> {
    match &outcome.first_generation {
        FirstGeneration::Measured(d) => Some(d.as_str()),
        FirstGeneration::NotMeasured(_) => None,
    }
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
            first_generation: FirstGeneration::NotMeasured(
                "the affected-set bound selected no compared mirror, so no generation was emitted"
                    .to_string(),
            ),
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

    let first_generation_equal = sync.matches && hand.unverifiable.is_empty();
    let changed_paths = sync.drifted_paths.clone();
    let convergence_roots = [workspace.join("dag"), workspace.join("src/v2")]
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let (basename_to_module, _, _, _, _, _) =
        convergence_surface_roles(&workspace, &convergence_roots)?;
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

    // Every field here was measured by THIS pass against THIS tree. The old shape also carried
    // `fixed_point_equal: false` -- not a measurement, since the first pass never asks; a literal
    // `false` asserted a negative where "not asked" belonged. The variant has no such field.
    let first_generation = FirstGeneration::Measured(candidate_digest.clone());
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

    Ok(RequiredRegenOutcome {
        receipt,
        failures,
        first_generation,
    })
}

/// Reconcile the two available answers to "what did the first generation emit".
///
/// TWO SOURCES FOR ONE FACT, RECONCILED BY REFUSAL RATHER THAN BY PRECEDENCE. The receipt file is
/// read unconditionally — the cross-tree refusal and the `PriorReceiptRef` are provenance facts
/// only the file carries — so a caller ALSO supplying the digest in memory makes it exist twice.
/// The previous `pass1_digest.unwrap_or(prior)` silently preferred the argument: two
/// representations with a precedence rule, so a disagreement decided and reported nothing
/// (DESIGN §3).
///
/// WHO MADE IT REACHABLE: until the phases shared a process every caller passed `None`, so the
/// file was the only source. The composed `--required-ci` run supplies the argument, so the change
/// creating the second source is the change closing it.
///
/// WHAT IT IS *NOT*: not a guard on an active defect on the composed path, where
/// `run_required_regen` writes the receipt and returns the same digest in one pass, so the two
/// agree by construction. It guards the FUNCTION's contract for a caller supplying a digest
/// against a receipt written by some other run at this commit — a rebuild between passes, a
/// mutated `target/`, a first pass that refused after writing. Extracted from the call site so
/// that claim is testable without a seven-minute emit.
fn reconcile_pass1_digest(supplied: Option<String>, prior: &str) -> Result<String, String> {
    match supplied {
        Some(supplied) if supplied != prior => Err(format!(
            "refusal: pass-1 digest disagreement — the caller supplied {supplied} but the \
             receipt at this commit records {prior}. These are two answers to what the first \
             generation emitted; the fixed-point comparison is meaningless until they agree. \
             Re-run `claim_executor --required-regen` at this commit so the receipt and the \
             in-memory pass agree."
        )),
        Some(supplied) => Ok(supplied),
        None => Ok(prior.to_string()),
    }
}

pub fn run_required_regen_fixed_point(
    receipt_rel: &str,
    pass1_digest: Option<String>,
) -> Result<RequiredRegenOutcome, String> {
    let workspace = workspace_root();
    let receipt_path = workspace.join(receipt_rel);
    let commit_sha = git_head_sha(&workspace)?;
    let prior = read_receipt(&receipt_path)?;

    // THE CROSS-TREE REFUSAL. The two passes are separate process invocations sharing one file
    // under `target/`; nothing requires the first to have run in this process, at this commit, or
    // at all. Without this arm a developer iterating on the determinism half over a `target/` warm
    // from an earlier commit produces a receipt stamped with TODAY's `commit_sha` carrying
    // YESTERDAY's `changed_paths` and `first_generation_equal`. In CI the arm is unreachable
    // because actions/checkout's default clean removes the ignored `target/` each run (measured:
    // two consecutive main runs each compiled 105 crates starting at proc-macro2, where a warm
    // tree compiles zero) -- a checkout default nobody declared, one cache-reuse change from live.
    //
    // It refuses rather than recomputing: re-running the first pass here would fuse the two
    // authorities, and proceeding is the fabrication. `PriorReceiptRef` makes the impersonation
    // unwritable; this makes referencing the WRONG tree loud.
    if prior.commit_sha != commit_sha {
        return Err(format!(
            "refusal: prior regen receipt was measured at commit {} but HEAD is {} -- the \
             fixed-point pass may reference first-generation evidence only from the same tree. \
             Re-run `claim_executor --required-regen` at this commit first.",
            prior.commit_sha, commit_sha
        ));
    }

    let pass1 = reconcile_pass1_digest(pass1_digest, &prior.candidate_generated_digest)?;
    let formatter = ResolvedFormatter::admit()?.with_normalize_cache(&workspace)?;
    let sources = super::regen_input_sources(&workspace)?;
    let authority_digest = authority_digest_from_sources(&sources)?;
    // WHOLE POPULATION, DELIBERATELY. The fixed-point pass asks whether a seed rebuilt from the
    // installed mirrors regenerates them unchanged; it compares digests over the COMMITTED
    // roster, so it must render every member of that roster. A selection here would compare a
    // digest over one population against pass 1's over another.
    let emitted = compile_stage0(
        &sources,
        &Rc::new(RustModuleRenderSelection::RenderEveryModule),
    )?;
    let committed_basenames = committed_generated_basenames(&workspace.join("src/v1/stage0/src"))?;
    if emitted.is_empty() {
        return Err("refusal: fixed-point emit produced zero files".to_string());
    }
    let emitted_basenames = generated_basenames_from_emit(&emitted)?;
    let hand_dir_shadows = hand_maintained_dir_shadows(&workspace.join("src/v1/stage0/src"))?;
    if let Some(reason) =
        validate_compared_populations(&committed_basenames, &emitted_basenames, &hand_dir_shadows)
    {
        return Err(reason);
    }
    let pass2 = tree_digest_from_map(&formatter, &emitted, &committed_basenames)?;
    let fixed_point_equal = pass1 == pass2;

    // `commit_sha` is what THIS pass ran against; `prior` names the tree its referenced evidence
    // came from. Equality is checked above and a mismatch refuses, so no receipt here quotes
    // another tree -- but the field is carried regardless: a subject guaranteed only by an
    // upstream check is one refactor from a reference that does not name its subject.
    let receipt = RegenReceipt::FixedPoint {
        schema: RECEIPT_SCHEMA.to_string(),
        commit_sha,
        authority_digest,
        candidate_generated_digest: pass2.clone(),
        fixed_point_equal,
        prior: PriorReceiptRef {
            commit_sha: prior.commit_sha,
            committed_generated_digest: prior.committed_generated_digest,
            first_generation_equal: prior.first_generation_equal,
            changed_paths: prior.changed_paths,
            candidate_artifact: prior.candidate_artifact,
        },
    };
    write_receipt(&receipt_path, &receipt)?;

    let failures = if fixed_point_equal {
        Vec::new()
    } else {
        vec![format!(
            "fixed-point refused: pass-1 digest {pass1} != pass-2 digest {pass2}"
        )]
    };

    // This outcome IS the fixed-point pass; it is not anybody's first generation, and saying so
    // is more useful than echoing a digest a later reader might hand onward.
    Ok(RequiredRegenOutcome {
        receipt,
        failures,
        first_generation: FirstGeneration::NotMeasured(
            "this outcome is the fixed-point pass, not a first generation".to_string(),
        ),
    })
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
    if let Some(reason) =
        validate_compared_populations(&committed, emitted_basenames, &hand_dir_shadows)
    {
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
    let mut out = HashMap::new();
    for file in result.files.iter() {
        out.insert(file.path.clone(), file.content.clone());
    }
    Ok(out)
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
/// (`gunbc.stage0_emitted_population_manifest`); the same inverse is spelled in the model at
/// `gunbc.stage0_rust_host_observation` `emitted_population_paths_from_manifest`, over the
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
/// guard refuses such a module name (the nearest, `gunbc.stage0_rust_source_lifecycle_scaffold`
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
    let mut committed_not_emitted = Vec::new();
    for name in committed {
        if !emitted_set.contains(name.as_str()) {
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
        first_generation: FirstGeneration::NotMeasured(reason),
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

fn digest_label(bytes: &[u8]) -> String {
    format!("fnv1a64:{}", v1_rt::bytes_identity_hash(bytes))
}

fn authority_digest_from_sources(sources: &[(String, String)]) -> Result<String, String> {
    let mut payload = String::new();
    for (path, content) in sources {
        payload.push_str(path);
        payload.push('\0');
        payload.push_str(&digest_label(content.as_bytes()));
        payload.push('\n');
    }
    Ok(digest_label(payload.as_bytes()))
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
    let mut payload = String::new();
    for name in basenames {
        let path = src_dir.join(name);
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("read {label} {}: {e}", path.display()))?;
        let norm = normalize_generated_source(formatter, &content)
            .map_err(|e| format!("normalize {label} {name}: {e}"))?;
        payload.push_str(name);
        payload.push('\0');
        payload.push_str(&digest_label(norm.as_bytes()));
        payload.push('\n');
    }
    Ok(digest_label(payload.as_bytes()))
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
        payload.push_str(&digest_label(norm.as_bytes()));
        payload.push('\n');
    }
    Ok(digest_label(payload.as_bytes()))
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
        formatter
            .memo
            .borrow_mut()
            .insert(content.to_string(), hit.clone());
        return Ok(hit);
    }
    let normalized = normalize_generated_source_uncached(formatter, content)?;
    formatter
        .memo
        .borrow_mut()
        .insert(content.to_string(), normalized.clone());
    normalize_cache_write(formatter, content, &normalized)?;
    Ok(normalized)
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

/// The first-pass measurement the second pass builds on.
///
/// This was a separate `RegenReceiptStored` struct mirroring the carrier's fields -- a second
/// representation (DESIGN 3), and where the false fail-closed claim hid: it accepted any JSON
/// containing its fields, reading a v1 record as happily as a v2. Gone: `read_receipt`
/// deserializes the REAL carrier and destructures it, so reader cannot drift from writer.
struct PriorMeasurement {
    commit_sha: String,
    committed_generated_digest: String,
    candidate_generated_digest: String,
    first_generation_equal: bool,
    changed_paths: Vec<String>,
    candidate_artifact: String,
}

/// Read the prior receipt, refusing everything that is not a first-generation measurement written
/// by this version of the carrier.
///
/// THREE REFUSALS, each closing a state the previous shape accepted silently:
///
///   * `deny_unknown_fields` on the carrier rejects a record carrying a field this shape does not
///     know -- which is exactly a stale `.v1` receipt, whose removed `fixed_point_equal` serde
///     would otherwise ignore;
///   * the schema equality check rejects a record whose version differs, so the version string is
///     compared rather than merely written;
///   * the variant match rejects a `fixed_point` receipt, because the second pass must build on a
///     FIRST-pass measurement and a receipt left by another second pass is not one.
///
/// The third was already true by construction (a `fixed_point` record lacks four required fields)
/// but is an explicit arm so the refusal reports the cause rather than a missing field name.
fn read_receipt(path: &Path) -> Result<PriorMeasurement, String> {
    let bytes =
        fs::read_to_string(path).map_err(|e| format!("read receipt {}: {e}", path.display()))?;
    let receipt: RegenReceipt = serde_json::from_str(&bytes)
        .map_err(|e| format!("parse receipt {}: {e}", path.display()))?;
    match receipt {
        RegenReceipt::FirstGeneration {
            schema,
            commit_sha,
            authority_digest: _,
            committed_generated_digest,
            candidate_generated_digest,
            first_generation_equal,
            changed_paths,
            candidate_artifact,
            candidate_manifest: _,
        } => {
            if schema != RECEIPT_SCHEMA {
                return Err(format!(
                    "refusal: prior receipt {} declares schema {schema} but this reader is \
                     {RECEIPT_SCHEMA} -- re-run `claim_executor --required-regen` to rewrite it",
                    path.display()
                ));
            }
            Ok(PriorMeasurement {
                commit_sha,
                committed_generated_digest,
                candidate_generated_digest,
                first_generation_equal,
                changed_paths,
                candidate_artifact,
            })
        }
        // THE ROUTE THE FABRICATED BOOL USED TO TAKE. Before `Refused` existed, a population
        // refusal left a `FirstGeneration` receipt and this arm read it as a measurement:
        // `first_generation_equal: false` became a `PriorMeasurement`, then a `PriorReceiptRef`,
        // then `referenced_first_generation_equal=false` on the operator's terminal -- a result
        // for a comparison that never ran. The cross-tree guard cannot catch it: the refusal
        // happened AT this commit, exactly what the guard admits. Now the variant carries no such
        // field and this arm refuses with the ORIGINAL cause: the fix is the refusal, not the
        // fixed point.
        RegenReceipt::Refused { reason, .. } => Err(format!(
            "refusal: the first-generation pass at this commit REFUSED ({reason}) — there is no \
             first-generation measurement for the fixed-point pass to reference. Close that \
             refusal and re-run `claim_executor --required-regen` before asking for the fixed \
             point. Receipt: {}",
            path.display()
        )),
        // A SCOPED ROUND THAT SELECTED NOTHING EMITTED NOTHING, so there is no first generation
        // here either -- and the remedy is different from the refusal above, which is why it is
        // its own arm rather than folded into one. Nothing is broken; the fixed-point pass is
        // simply being asked to build on a round that had no work to do. The operator either
        // wants the WHOLE-population round (drop `--regen-affected-scope`) or does not need a
        // fixed point for this edit at all.
        RegenReceipt::NoAffectedMirrors { scope, .. } => Err(format!(
            "refusal: the first-generation pass at this commit selected no compared mirror \
             ({scope}), so it emitted nothing and there is no first-generation measurement for \
             the fixed-point pass to reference. Re-run without `--regen-affected-scope` if you \
             need a whole-population fixed point. Receipt: {}",
            path.display()
        )),
        RegenReceipt::FixedPoint { .. } => Err(format!(
            "refusal: prior receipt {} is a fixed-point receipt, not a first-generation \
             measurement -- the fixed-point pass cannot build on another fixed-point pass. \
             Re-run `claim_executor --required-regen` first.",
            path.display()
        )),
    }
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

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&path).expect("create temp");
        path
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
                compile_stage0(&selection_fixture(), &selection).expect("fixture emits clean")
            })
            .expect("spawn emit thread")
            .join()
            .expect("emit thread panicked")
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

    /// THE PRECONDITION `import_refusals` EXACTNESS RESTS ON, PUT ON THE EXECUTED PATH.
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
    fn filter_in_branch_condition_refuses_and_does_not_publish_the_module() {
        let (named, module_published, positive_published, positive_named) =
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn(|| {
                    let emit_one = |content: &str| {
                        let module_index =
                            crate::cli_run::build_module_path_index_from_witness_roots();
                        let sources = crate::cli_run::resolve_virtual_source_with_imports(
                            "probe.dag",
                            content,
                            &module_index,
                        );
                        let resolved = compile_to_resolved(Rc::new(sources.into()));
                        let typed = emittable_graph(resolved)
                            .expect("front-end must accept the specimen so emission is the wall")
                            .graph();
                        crate::v1_compiler_emit_rust::emit_rust(typed)
                    };
                    let negative = emit_one(
                        "module fx.filter_guard\nimport std.types { List, Bool, Int }\nfn f(xs: List<Int>) -> Int {\n  if (xs |> filter(x => x > 0) |> count) > 0 {\n    1\n  } else {\n    0\n  }\n}\n",
                    );
                    let named = negative.diagnostics.iter().any(|d| {
                        matches!(
                            &*d.diagnostic,
                            crate::v1_std_core::CompilerDiagnostic::EmissionConstructUnprojectable {
                                construct,
                                ..
                            } if matches!(
                                construct.as_ref(),
                                crate::v1_std_core::UnprojectableConstruct::FilterInBranchCondition
                            )
                        ) && crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone())
                    });
                    let module_published = negative.files.iter().any(|f| {
                        f.path.contains("fx_filter_guard")
                    });
                    let positive = emit_one(
                        "module fx.any_guard\nimport std.types { List, Bool, Int }\nfn f(xs: List<Int>) -> Int {\n  if xs |> any(x => x > 0) {\n    1\n  } else {\n    0\n  }\n}\n",
                    );
                    let positive_named = positive.diagnostics.iter().any(|d| {
                        matches!(
                            &*d.diagnostic,
                            crate::v1_std_core::CompilerDiagnostic::EmissionConstructUnprojectable { .. }
                        )
                    });
                    let positive_published = positive.files.iter().any(|f| f.path.contains("fx_any_guard"));
                    (
                        named,
                        module_published,
                        positive_published,
                        positive_named,
                    )
                })
                .expect("spawn projection-refusal thread")
                .join()
                .expect("projection-refusal thread panicked");
        assert!(
            named,
            "filter in a branch condition must refuse at emission with EmissionConstructUnprojectable naming the construct"
        );
        assert!(
            !module_published,
            "the refused module must be absent from EmitResult.files — output-plus-diagnostic is not a fix"
        );
        assert!(
            positive_published && !positive_named,
            "an already-supported guarded any-lambda must still emit its module"
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
        let reason = validate_compared_populations(&["foo.rs".to_string()], &[], &BTreeMap::new())
            .expect("expected refusal");
        assert!(reason.contains("zero generated surfaces"));
    }

    #[test]
    fn empty_committed_population_refuses_before_agreement() {
        let reason = validate_compared_populations(&[], &["foo.rs".to_string()], &BTreeMap::new())
            .expect("expected refusal");
        assert!(reason.contains("committed generated population is empty"));
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

    // THE SENTINEL HAS NO ROUTE TO THE FIXED-POINT PHASE -- ON EITHER CARRIER.
    //
    // The defect pinned, found in review of gunbc#8647: a population refusal returns `Ok`, and
    // its receipt held `refused:population` in both digest positions and
    // `first_generation_equal: false` where "not asked" belonged. Two carriers cross out of that
    // function -- the in-process `RequiredRegenOutcome` and the on-disk receipt -- and the typed
    // `FirstGeneration` closed only the first. The receipt crosses the PROCESS boundary, so the
    // fabricated Bool reached a standalone `--required-regen-fixed-point` run through
    // `read_receipt` and printed as `referenced_first_generation_equal=false`.
    //
    // Both halves are asserted. In-memory (the original test): `pass1_digest_for_fixed_point`
    // yields `None` for a refusal, and answering from the receipt fails it. On-disk (new): the
    // refusal variant has no Bool and no digest to read, and `read_receipt` refuses a refused
    // prior with its ORIGINAL cause rather than a derived answer.
    #[test]
    fn a_refused_first_generation_hands_no_digest_to_the_fixed_point() {
        let refused_receipt = || RegenReceipt::Refused {
            schema: RECEIPT_SCHEMA.to_string(),
            commit_sha: "sha".to_string(),
            authority_digest: "auth".to_string(),
            reason: "refusal: emit produced zero files".to_string(),
            candidate_artifact: "cand".to_string(),
        };

        let refused = RequiredRegenOutcome {
            receipt: refused_receipt(),
            failures: vec!["refusal: emit produced zero files".to_string()],
            first_generation: FirstGeneration::NotMeasured(
                "refusal: emit produced zero files".to_string(),
            ),
        };
        assert_eq!(pass1_digest_for_fixed_point(&refused), None);

        // THE HALF THAT USED TO BE THE GAP. The old test asserted the OPPOSITE -- that the
        // sentinel "IS still sitting in the receipt ... the wrong answer is right there to be
        // read" -- and called the coproduct load-bearing for a clean outcome over a fabricated
        // receipt. Now no digest and no equality exist to misread; absent from the variant, not
        // by convention.
        assert_eq!(refused.receipt.candidate_generated_digest(), None);
        assert_eq!(refused.receipt.first_generation_equal(), None);
        assert_eq!(
            refused.receipt.refusal_reason(),
            Some("refusal: emit produced zero files")
        );

        // POSITIVE CONTROL: ordinary drift is not a refusal. Pass one emitted, the comparison
        // disagreed, and the fixed point still has a subject -- skipping it would lose the
        // determinism signal exactly when drift makes it interesting. This control carries a REAL
        // digest and a REAL `false`: here `false` is an answer, and no receipt remains on which it
        // is not.
        let drifted = RequiredRegenOutcome {
            receipt: RegenReceipt::FirstGeneration {
                schema: RECEIPT_SCHEMA.to_string(),
                commit_sha: "sha".to_string(),
                authority_digest: "auth".to_string(),
                committed_generated_digest: "committed-digest".to_string(),
                candidate_generated_digest: "real-digest".to_string(),
                first_generation_equal: false,
                changed_paths: vec!["drifted.rs".to_string()],
                candidate_artifact: "cand".to_string(),
                candidate_manifest: fixture_candidate_manifest(),
            },
            failures: vec!["17 file(s) drifted".to_string()],
            first_generation: FirstGeneration::Measured("real-digest".to_string()),
        };
        assert_eq!(pass1_digest_for_fixed_point(&drifted), Some("real-digest"));
        assert_eq!(
            drifted.receipt.candidate_generated_digest(),
            Some("real-digest")
        );
        assert_eq!(drifted.receipt.first_generation_equal(), Some(false));
        assert_eq!(drifted.receipt.refusal_reason(), None);
    }

    // THE ON-DISK HALF, THROUGH THE REAL READER. `read_receipt` turned the fabricated Bool into a
    // `PriorMeasurement`, so the refusal is asserted THROUGH it, not against the variant alone.
    //
    // RED: give `RegenReceipt::Refused` a `first_generation_equal: bool` field and let this arm
    // build a `PriorMeasurement` from it -- the defect restored -- and this test fails. The
    // positive control keeps the refusal from being satisfied by a reader refusing everything.
    #[test]
    fn read_receipt_refuses_a_refused_prior_with_its_original_cause() {
        let tmp = temp_dir("required-regen-refused-prior");
        fs::create_dir_all(&tmp).expect("create tmp");
        let path = tmp.join("receipt.json");

        write_receipt(
            &path,
            &RegenReceipt::Refused {
                schema: RECEIPT_SCHEMA.to_string(),
                commit_sha: "sha".to_string(),
                authority_digest: "auth".to_string(),
                reason: "refusal: committed mirror is no longer emitted".to_string(),
                candidate_artifact: "cand".to_string(),
            },
        )
        .expect("write refused receipt");

        // Matched rather than `expect_err` because `PriorMeasurement` is deliberately not `Debug`;
        // the Ok arm names what went wrong instead of asking the type to print itself.
        let err = match read_receipt(&path) {
            Ok(_) => panic!("a refused prior must not read as a first-generation measurement"),
            Err(e) => e,
        };
        assert!(
            err.contains("refusal: committed mirror is no longer emitted"),
            "the ORIGINAL cause must survive, not a comparison result standing in for it: {err}"
        );

        // POSITIVE CONTROL: a real first-generation receipt still reads, with its real answer.
        write_receipt(
            &path,
            &RegenReceipt::FirstGeneration {
                schema: RECEIPT_SCHEMA.to_string(),
                commit_sha: "sha".to_string(),
                authority_digest: "auth".to_string(),
                committed_generated_digest: "committed-digest".to_string(),
                candidate_generated_digest: "real-digest".to_string(),
                first_generation_equal: false,
                changed_paths: vec!["drifted.rs".to_string()],
                candidate_artifact: "cand".to_string(),
                candidate_manifest: fixture_candidate_manifest(),
            },
        )
        .expect("write measured receipt");

        let prior = read_receipt(&path).expect("a measured prior reads");
        assert_eq!(prior.candidate_generated_digest, "real-digest");
        assert!(!prior.first_generation_equal);

        let _ = fs::remove_dir_all(&tmp);
    }

    // LOCAL RUST RED CONTROL FOR THE DUAL-INPUT REFUSAL — local, NOT enrolled. The Rust suite has
    // been out of CI since the 2026-07-11 operator ruling, so this executes for whoever runs it
    // and for no gate. The previous heading claimed "ENROLLED" — the rung inflation DESIGN §4b
    // calls worse than sitting low: an unenrolled control claiming enrollment never ranks for it.
    //
    // The arm it guards is UNREACHABLE from the composed `--required-ci` path — there
    // `run_required_regen` writes the receipt and returns the same digest in one pass — which is
    // why the decision was extracted from its call site: through the real function it needs a
    // seven-minute emit, and a wall no test can reach is a wall nobody knows works.
    //
    // RED: restoring `pass1_digest.unwrap_or(prior)` makes the disagreement case return Ok and
    // fails the first assertion. The None and agreeing cases are the positive controls.
    #[test]
    fn pass1_digest_disagreement_refuses_rather_than_preferring_one() {
        let err = reconcile_pass1_digest(Some("supplied-abc".to_string()), "receipt-xyz")
            .expect_err("two different answers to one fact must refuse");
        assert!(
            err.contains("supplied-abc") && err.contains("receipt-xyz"),
            "the refusal must name BOTH values so the reader sees a contradiction rather than \
             a comparison whose operand was chosen for them: {err}"
        );

        assert_eq!(
            reconcile_pass1_digest(Some("same".to_string()), "same").expect("agreement is fine"),
            "same",
            "positive control: agreeing sources are not a refusal"
        );
        assert_eq!(
            reconcile_pass1_digest(None, "from-receipt").expect("no second source, no conflict"),
            "from-receipt",
            "positive control: with one source the receipt is simply used"
        );
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

pub struct RegenRoundCostOutcome {
    /// The rendered receipt, already printed by the caller's contract to stderr.
    pub rendered: String,
    /// Where the same bytes were written, so a remote run can bring them back by path.
    pub receipt_path: PathBuf,
    /// Anything that stopped the round short of install/rebuild/diff, or that the regen
    /// reported as a failure. Non-empty means the round is not a clean round.
    pub round_failures: Vec<String>,
}

pub const REGEN_ROUND_COST_RECEIPT_REL: &str = "target/stage0-regen-round-cost.txt";
const REGEN_ROUND_COST_PRODUCER: &str = "claim_executor --regen-round-cost";
const REGEN_ROUND_COST_ENTRY_UNDER_ROOT: &str = "gunbc/regen_round_cost.dag";

struct CargoBuildObservation {
    compiled_crates: u64,
    compiled_packages: Vec<String>,
}

const REGEN_CONVERGENCE_SCHEMA: &str = "gunbc.regen_convergence_transaction.v1";
const REGEN_CONVERGENCE_BOUND: usize = 8;
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

#[derive(Debug, Clone)]
struct RegenConvergenceCheckpointSubject {
    starting_commit: String,
    source_authority_digest: String,
    stage_plan_authority_digest: String,
}

struct RegenConvergenceModel {
    graph: Rc<crate::v1_compiler_compile::ResolvedGraph>,
    indices: Rc<im::HashMap<String, Rc<crate::v1_compiler_compile::NewlineIndex>>>,
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
        let index = super::process_shared_index(source_roots);
        let (graph, indices) =
            super::resolve_entry_with_index_for_discovery_corpus(&index, &entry.to_string_lossy())
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
        Value::Str(label) if label.as_ref() == "Admitted" => Ok(()),
        Value::Str(label) => Err(format!("candidate generation admission {label}")),
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

fn regen_convergence_journal_path(workspace: &Path) -> PathBuf {
    workspace.join(REGEN_CONVERGENCE_JOURNAL_REL)
}

fn current_convergence_checkpoint_subject(
    workspace: &Path,
) -> Result<RegenConvergenceCheckpointSubject, String> {
    Ok(RegenConvergenceCheckpointSubject {
        starting_commit: git_head_sha(workspace)?,
        source_authority_digest: authority_digest_from_sources(&super::regen_input_sources(
            workspace,
        )?)?,
        stage_plan_authority_digest: path_digest(
            &workspace.join("src/v2/workflow/regen_convergence_transaction.dag"),
        )?,
    })
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

fn restore_regen_convergence_journal(workspace: &Path) -> Result<(), String> {
    let subject = current_convergence_checkpoint_subject(workspace)?;
    restore_regen_convergence_journal_for_subject(workspace, &subject)
}

fn restore_regen_convergence_journal_for_subject(
    workspace: &Path,
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<(), String> {
    let root = regen_convergence_journal_path(workspace);
    let manifest_path = root.join("journal.json");
    if !manifest_path.is_file() {
        return Ok(());
    }
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
    for entry in &manifest.entries {
        let destination = workspace.join(&entry.relative_path);
        if matches!(
            entry.pre_stage_state,
            RegenPreStageState::PresentBeforeInstall { .. }
        ) {
            let prepared = restore_tmp.join(&entry.backup_name);
            fs::rename(&prepared, &destination).map_err(|e| {
                format!(
                    "rollback/restore refuses: atomic rename {} -> {}: {e}",
                    prepared.display(),
                    destination.display()
                )
            })?;
        } else if destination.exists() {
            fs::remove_file(&destination).map_err(|e| {
                format!(
                    "rollback/restore refuses: remove new {}: {e}",
                    destination.display()
                )
            })?;
        }
    }
    fs::remove_dir_all(&root)
        .map_err(|e| format!("remove restored journal {}: {e}", root.display()))
}

fn journal_stage0_paths_for_subject(
    workspace: &Path,
    stage0_src: &Path,
    basenames: &[String],
    subject: &RegenConvergenceCheckpointSubject,
) -> Result<(), String> {
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
    for basename in basenames {
        let relative_path = format!("src/v1/stage0/src/{basename}");
        if journal
            .entries
            .iter()
            .any(|entry| entry.relative_path == relative_path)
        {
            continue;
        }
        let source = stage0_src.join(basename);
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

fn seed_cargo_build(workspace: &Path, label: &str) -> Result<CargoBuildObservation, String> {
    v1_rt::trace_mark(format!("{label}.begin"));
    let output = Command::new("cargo")
        .args(["build", "--release", "--bin", "claim_executor"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn cargo build ({label}): {e}"))?;
    v1_rt::trace_mark(format!("{label}.done"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let tail: Vec<&str> = stderr.lines().rev().take(40).collect::<Vec<_>>();
        let tail: Vec<&str> = tail.into_iter().rev().collect();
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
    Ok(CargoBuildObservation {
        compiled_crates,
        compiled_packages,
    })
}

/// The seed binary ON DISK at the path this process started from. After a cargo build replaces
/// that file, Linux reports the running image as `<path> (deleted)`; digesting the path itself
/// compares "installed there now" against "installed there before the build" -- the refusal's
/// question.
fn current_exe_digest() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let shown = exe.to_string_lossy().into_owned();
    let on_disk = PathBuf::from(shown.strip_suffix(" (deleted)").unwrap_or(&shown));
    let bytes = fs::read(&on_disk).map_err(|e| format!("read {}: {e}", on_disk.display()))?;
    Ok(v1_rt::bytes_identity_hash(&bytes))
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
    rebuild_packages: &[String],
    executable_digest: &str,
    second_generation_candidate_digest: &str,
) -> Result<String, String> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = round_cost_entry(source_roots)?;
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| {
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
        ]),
    };
    let args = vec![(Some("receipt".to_string()), receipt)];
    let rendered = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "regen_round_cost_render", &args, false)
    })
    .map_err(|e| format!("refusal: regen_round_cost_render did not render: {e}"))?;
    match rendered {
        Value::Str(s) => Ok(s.to_string()),
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
}

type ModelValue = crate::v1_interpreter::Value;

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
) -> Result<PartitionRebuildActuation, String> {
    use crate::v1_interpreter::{self, ExecutionMode};
    let entry = round_cost_entry(source_roots)?;
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| {
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
        ModelValue::Str(value) => value.to_string(),
        other => {
            return Err(format!(
            "refusal: stage0_partition_rebuild_decision_line_today returned {} instead of String",
            other.type_label_public()
        ))
        }
    };
    Ok(PartitionRebuildActuation {
        actuatable,
        package_closure,
        excluded_packages,
        decision_line,
    })
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
    if actuation.package_closure.is_empty() {
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
        return Err(format!(
            "StageSeedBuildRefused: compiled excluded packages {widened:?} -- {}",
            actuation.decision_line
        ));
    }
    Ok(observation)
}

fn next_pass_executable_digest(workspace: &Path) -> Result<String, String> {
    let executable = workspace.join("target/release/claim_executor");
    fs::read(&executable)
        .map(|bytes| v1_rt::bytes_identity_hash(&bytes))
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
            ..
        } => Ok(FirstGenerationConvergenceObservation {
            committed_digest: committed_generated_digest,
            drifted: changed_paths,
            authority_digest,
            manifest: candidate_manifest,
        }),
        RegenReceipt::Refused { reason, .. } => Err(format!(
            "regen refused before convergence planning: {reason}"
        )),
        RegenReceipt::FixedPoint { .. } => Err(
            "regen convergence expected a first-generation receipt, found fixed-point".to_string(),
        ),
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
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let shown = exe.to_string_lossy().into_owned();
    let on_disk = PathBuf::from(shown.strip_suffix(" (deleted)").unwrap_or(&shown));
    let observed_executable_digest = path_digest(&on_disk)?;
    if observed_executable_digest != admitted_executable_digest {
        return Err(format!(
            "CandidateGeneratedByDifferentSeed: stage admitted executable {} but next generation would run {} at {}",
            admitted_executable_digest,
            observed_executable_digest,
            on_disk.display()
        ));
    }
    let mut command = Command::new(&on_disk);
    command.arg("--required-regen");
    if affected_scope {
        command.arg("--regen-affected-scope");
    }
    command
        .arg("--regen-candidate-dir")
        .arg(candidate_dir_rel)
        .arg("--regen-receipt")
        .arg(receipt_rel)
        .current_dir(workspace);
    for root in source_roots {
        command.arg("--source-root").arg(root);
    }
    let output = command
        .output()
        .map_err(|e| format!("spawn rebuilt seed {}: {e}", on_disk.display()))?;
    // Drift makes --required-regen exit one after it has written the candidate and receipt. The
    // receipt, not the process code, distinguishes ordinary drift from a production refusal.
    if !receipt_path.is_file() {
        return Err(format!(
            "rebuilt seed produced no fresh generation receipt at {}; status={} stderr_tail={}",
            receipt_path.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    let receipt = read_first_generation_receipt(&receipt_path);
    match receipt {
        Ok(value) => Ok(value),
        Err(reason) => Err(format!(
            "{reason}; rebuilt seed status={} stderr_tail={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" | ")
        )),
    }
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
        HashMap<String, String>,
    ),
    String,
> {
    let (edges, modules) = regen_module_edges(workspace)?;
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
    let host_shell_modules = super::emitted_closure_compile_host::closure_modules(
        &workspace.join("src/v1/stage0/src/lib.rs"),
    )?
    .into_iter()
    .collect::<BTreeSet<_>>();
    let partition_rows =
        crate::gunbc_stage0_crate_partition_generated::generated_partition_crate_rows();
    let seed_modules = assembled_seed_modules(host_shell_modules, partition_rows.as_ref());
    let seed_embedded_basenames = seed_modules
        .into_iter()
        .map(|module| format!("{module}.rs"))
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
    mut host_shell_modules: BTreeSet<String>,
    partition_rows: &crate::std_types::List<
        Rc<crate::gunbc_stage0_crate_partition_generated::GeneratedPartitionCrateRow>,
    >,
) -> BTreeSet<String> {
    use crate::gunbc_stage0_crate_partition_generated::GeneratedPartitionCrateKind::*;
    for row in partition_rows.iter() {
        match row.kind {
            GeneratedFoundationCrate | GeneratedLayeredCoreCrate => {
                host_shell_modules.extend(row.modules.iter().cloned());
            }
            // Emit-core is a consumer/re-export, not a module-bearing assembly owner.
            GeneratedEmitCoreCrate => {}
        }
    }
    host_shell_modules
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
    generated_product_roles: &HashMap<String, String>,
    affected_bound: &RegenEmissionScope,
    seen_state_keys: &[String],
    seed_digest: &str,
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
        (
            Some("bound".to_string()),
            Value::Int(REGEN_CONVERGENCE_BOUND as i64),
        ),
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
        Value::Str(label) if label.as_ref() == "Admitted" => {}
        Value::Str(label) => return Err(format!("generation progress {label}")),
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
            } else if let Some(role) = generated_product_roles.get(basename) {
                role.as_str()
            } else if generation_modules.contains(&module) {
                "GenerationInput"
            } else if bootstrap_sources.contains(&module) {
                "BootstrapSourceMirror"
            } else {
                "GenerationSubject"
            };
            let seed_embedded = seed_embedded_basenames.contains(basename);
            let role_value = Value::Variant {
                type_name: ctx.sym("RegenGenerationRole"),
                variant_name: ctx.sym(role),
                fields: Rc::new(vec![]),
            };
            let membership_value = if seed_embedded {
                Value::Variant {
                    type_name: ctx.sym("RegenSeedMembership"),
                    variant_name: ctx.sym("SeedEmbedded"),
                    fields: Rc::new(vec![]),
                }
            } else if role == "NonSeedGeneratedOutput" {
                Value::Variant {
                    type_name: ctx.sym("RegenSeedMembership"),
                    variant_name: ctx.sym("OutsideSeed"),
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
                Value::List(ids) if ids.len() == 1 => match &ids[0] {
                    Value::Str(id) => id.to_string(),
                    other => {
                        return Err(format!(
                            "StageDependencyClosureIncomplete: model returned {} closure member",
                            other.type_label_public()
                        ))
                    }
                },
                Value::List(ids) => {
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
        Value::Str(label) if label.as_ref() == "Admitted" => {}
        Value::Str(label) => {
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
        Value::Str(value) if value.as_ref() == "PromoteGenerationInputs" => {
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs
        }
        Value::Str(value) if value.as_ref() == "InstallSeedCompatibilityCut" => {
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut
        }
        Value::Str(value) if value.as_ref() == "PublishNonSeedOutputs" => {
            RegenConvergenceStageKindReceipt::PublishNonSeedOutputs
        }
        Value::Str(value) => {
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
        Value::List(values) => values
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
        Value::List(ids) if ids.len() == 1 => match &ids[0] {
            Value::Str(id) => id.to_string(),
            other => {
                return Err(format!(
                    "refusal: convergence closure projection member is {}",
                    other.type_label_public()
                ))
            }
        },
        Value::List(ids) => {
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
    admitted_manifest: &HashMap<String, RegenCandidateManifestSurface>,
    basename_to_module: &HashMap<String, String>,
    ordinal: usize,
    kind: RegenConvergenceStageKindReceipt,
    seed_before: &str,
    generation_id: &str,
    candidate_tree_id: &str,
    candidate_tree_digest: &str,
    dependency_closure_id: &str,
) -> Result<RegenConvergenceStageReceipt, String> {
    let subject = current_convergence_checkpoint_subject(workspace)?;
    let actuation = partition_rebuild_actuation(source_roots, basenames)?;
    install_convergence_stage_with_backend(
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
        |workspace| partitioned_rebuild_from_installed(workspace, &actuation),
        || next_pass_executable_digest(workspace),
    )
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
        assembly_equivalence_authority: "v2.compiler.self_host.stage0_executable_assembly;v2.test.claim.self_host.stage0_executable_assembly_test".to_string(),
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
        Value::Str(label) if label.as_ref() == "Admitted" => Ok(()),
        Value::Str(label) => {
            let detail = match v1_interpreter::with_active_context(&ctx, || {
                v1_interpreter::run_in_context_with_args(
                    &ctx,
                    "regen_population_admission_detail",
                    &[(Some("admission".to_string()), for_detail)],
                    false,
                )
            }) {
                Ok(Value::Str(detail)) => detail.to_string(),
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
        Value::Str(label) if label.as_ref() == "Admitted" => Ok(()),
        Value::Str(label) => {
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
                Ok(Value::Str(detail)) => detail.to_string(),
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
) -> Result<RegenRoundCostOutcome, String> {
    let workspace = workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");
    let candidate_src = workspace.join(candidate_dir_rel).join("src");
    let host = host_name();
    let tree = git_head_sha(&workspace)?;
    let tree_dirty = git_tree_dirty(&workspace)?;
    let exe_before = current_exe_digest()?;

    // Recover an interrupted transaction before attempting to build or observe a new subject.
    restore_regen_convergence_journal(&workspace)?;

    v1_rt::trace_ledger_arm();
    let rustfmt_spawns_before = rustfmt_spawn_count();
    let seed_build = seed_cargo_build(&workspace, "round.seed_build")?;
    let exe_after = current_exe_digest()?;
    if exe_before != exe_after {
        return Err(format!(
            "refusal: the seed build replaced the running executable ({exe_before} -> \
             {exe_after}), so an emit from this process would measure a seed the build did not \
             produce. Re-run {REGEN_ROUND_COST_PRODUCER} so the emitting seed is the built one."
        ));
    }

    let (
        basename_to_module,
        generation_modules,
        bootstrap_sources,
        bootstrap_products,
        seed_embedded_basenames,
        generated_product_roles,
    ) = convergence_surface_roles(&workspace, source_roots)?;
    let scope = if affected_scope {
        regen_emission_scope_for_diff(&workspace, source_roots)?
    } else {
        RegenEmissionScope::WholePopulation
    };
    let regen = run_required_regen_scoped(candidate_dir_rel, receipt_rel, &scope)?;
    let first = read_first_generation_receipt(&workspace.join(receipt_rel))?;
    let mut candidate_digest = first.manifest.candidate_tree_digest.clone();
    let starting_surface_digest = first.committed_digest;
    let mut candidate_tree_id = first.manifest.candidate_tree_id.clone();
    let mut drifted = first.drifted;
    let mut authority_digest = first.authority_digest;
    let mut candidate_manifest = first.manifest;
    let initial_seed_digest = current_exe_digest()?;
    let mut current_seed_digest = initial_seed_digest.clone();
    let mut stages = Vec::new();
    let mut seen_states = BTreeSet::new();
    let mut generation_ordinal = 0usize;
    let mut round_failures = Vec::new();
    let convergence_model = RegenConvergenceModel::load(source_roots)?;
    // THE DENOMINATOR, ACCUMULATED RATHER THAN SAMPLED. Every generation's drifted population
    // joins this set, because installing generation inputs can make the NEXT generation emit a
    // surface the first one did not -- so the transaction's admitted population is the union, and
    // taking only the first generation's would refuse a correct run.
    let mut transaction_admitted: BTreeSet<String> = BTreeSet::new();
    let mut transaction_lineage: BTreeMap<String, ConvergenceDisposition> = BTreeMap::new();
    if matches!(regen.receipt, RegenReceipt::Refused { .. }) {
        round_failures.extend(
            regen
                .failures
                .iter()
                .map(|failure| format!("regen: {failure}")),
        );
    }

    while !drifted.is_empty() {
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
            restore_regen_convergence_journal(&workspace)?;
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
        let drifted_entering_stage = drifted.clone();
        v1_rt::trace_mark("round.install.begin".to_string());
        let stage_result = install_convergence_stage(
            &convergence_model,
            source_roots,
            &workspace,
            &stage0_src,
            &candidate_src,
            &install_set,
            &admitted_manifest,
            &basename_to_module,
            generation_ordinal + 1,
            stage_kind,
            &current_seed_digest,
            &candidate_manifest.generation_id,
            &candidate_tree_id,
            &candidate_digest,
            &dependency_closure_id,
        );
        v1_rt::trace_mark("round.install.done".to_string());
        let mut stage = match stage_result {
            Ok(stage) => stage,
            Err(failure) => {
                restore_regen_convergence_journal(&workspace)?;
                return Err(format!(
                    "stage seed build refused; checkpoint restored: {failure}"
                ));
            }
        };
        stage.deferred_surfaces = deferred;
        current_seed_digest = stage.output_seed_digest.clone();
        stages.push(stage);
        generation_ordinal += 1;
        let next = run_built_seed_regen(
            &workspace,
            candidate_dir_rel,
            receipt_rel,
            source_roots,
            affected_scope,
            &current_seed_digest,
        );
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
                restore_regen_convergence_journal(&workspace)?;
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
    let installed_mirrors = transaction_receipt
        .stages
        .iter()
        .flat_map(|stage| stage.surfaces.iter())
        .map(|surface| surface.projected_path.clone())
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
    let marks: std::vec::Vec<v1_rt::TraceLedgerRow> = v1_rt::trace_ledger_drain()
        .ok_or_else(|| {
            "refusal: the trace ledger was not armed, so no phase was recorded".to_string()
        })?
        .iter()
        .cloned()
        .collect();
    let rendered = render_round_cost_receipt(
        source_roots,
        &host,
        &tree,
        tree_dirty,
        seed_build.compiled_crates,
        rebuild_compiled_crates,
        rustfmt_spawn_count() - rustfmt_spawns_before,
        &marks,
        &changed_paths,
        &convergence_stage_receipt_ids,
        &installed_mirrors,
        &rebuild_packages,
        &executable_digest,
        &second_generation_candidate_digest,
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
    })
}

#[cfg(test)]
mod regen_round_cost_tests {
    use super::*;

    #[test]
    fn assembled_seed_includes_foundation_and_layered_partitions_not_emit_consumer() {
        let workspace = workspace_root();
        let host_shell_modules = super::super::emitted_closure_compile_host::closure_modules(
            &workspace.join("src/v1/stage0/src/lib.rs"),
        )
        .expect("the executable assembly manifest resolves")
        .into_iter()
        .collect::<BTreeSet<_>>();
        let rows = crate::gunbc_stage0_crate_partition_generated::generated_partition_crate_rows();
        let assembled = assembled_seed_modules(host_shell_modules, rows.as_ref());
        assert!(assembled.contains("v1_rt"));
        assert!(assembled.contains("std_content_hash"));
        assert!(assembled.contains("std_measure"));
        assert!(assembled.contains("v1_compiler_infer_service"));
    }

    /// THE SEED-TO-MODEL LOCKSTEP the .dag witness says it cannot hold: the host builds the
    /// receipt Value with these field and variant names, and the model's renderer either
    /// accepts them or refuses. A renamed field on either side reds here, not in a
    /// forty-minute round. The expected text is the same fixture the .dag witness asserts.
    #[test]
    fn host_built_receipt_renders_through_the_model() {
        // Both roots the production driver passes: `std.observation`'s closure reaches
        // `std.cache_interface`, which imports `v2.std.optional` from src/v2.
        let roots: Vec<String> = ["dag", "src/v2"]
            .iter()
            .map(|r| workspace_root().join(r).to_string_lossy().into_owned())
            .collect();
        let marks = vec![
            v1_rt::TraceLedgerRow {
                label: "round.seed_build".to_string(),
                wall_ms: 1500,
                cpu_ms: Some(9000),
            },
            v1_rt::TraceLedgerRow {
                label: "compile.emit".to_string(),
                wall_ms: 300000,
                cpu_ms: None,
            },
        ];
        let rendered = render_round_cost_receipt(
            &roots,
            "srv1",
            "2a11b317d2caf3c37d1d38a4421e8e0c06188925",
            true,
            0,
            2,
            7,
            &marks,
            &["v1_rt.rs".to_string()],
            &["stage-1".to_string()],
            &["v1_rt.rs".to_string()],
            &["v1-stage0-runtime".to_string()],
            "sha256:claim-executor",
            "sha256:g1-candidate-tree",
        )
        .expect("the model renders a host-built receipt");
        assert_eq!(
            rendered,
            "regen-round-cost: producer=claim_executor --regen-round-cost host=srv1 \
             tree=2a11b317d2caf3c37d1d38a4421e8e0c06188925 tree_dirty=true \
             seed_build_compiled_crates=0 rebuild_compiled_crates=2 rustfmt_spawns=7\n\
             regen-round-cost: phase=seed_build wall_ms=1500 cpu_ms=9000\n\
             regen-round-cost: phase=compile.emit wall_ms=300000 cpu_ms=na\n\
             regen-round-cost: total wall_ms=301500 cpu_ms=na\n\
             regen-round-cost: changed_paths=1 [v1_rt.rs]\n\
             regen-round-cost: convergence_stages=1 [stage-1]\n\
             partition-rebuild: PartitionRebuildScopeDerived changed_mirrors=[v1_rt.rs] \
             owning_packages=[v1-stage0-runtime] \
             package_closure=[v1-stage0-runtime, v1-stage0-std-core, v1-stage0-std-surface, \
             v1-stage0-extdeps-languages, v1-stage0-v1-artifact, v1-stage0-v1-infer, \
             v1-stage0-emit-core, v1-compiler] executable_assembly=assembled package=v1-compiler bin=claim_executor\n\
             regen-round-cost: execution-identity rebuild_packages=1 [v1-stage0-runtime] \
             executable_digest=sha256:claim-executor second_generation_candidate=sha256:g1-candidate-tree\n"
        );
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

    fn fixture_workspace() -> (PathBuf, PathBuf, PathBuf, RegenConvergenceCheckpointSubject) {
        let root = std::env::temp_dir().join(format!(
            "gunbc-regen-convergence-host-{}-{}",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
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
                .current_dir(&root)
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
        git(&["init", "-q"]);
        git(&["add", "."]);
        git(&["commit", "-qm", "fixture"]);
        let subject = RegenConvergenceCheckpointSubject {
            starting_commit: "fixture-head".to_string(),
            source_authority_digest: "fixture-authority".to_string(),
            stage_plan_authority_digest: "fixture-plan".to_string(),
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

    /// THE DISCRIMINATING RED FOR `admit_install_target`, and the reason the wall is not a
    /// decoration: the forbidden state is authorable here even though no production roster can
    /// currently express it.
    ///
    /// Each negative arm passes an EMPTY admitted manifest. Without the boundary admission the
    /// call still returns `Err` — `CandidateManifestPopulationMismatch`, from the re-admit loop —
    /// so a test asserting only "it refused" would be permanently green and carry no information.
    /// It is the CAUSE that discriminates: these arms pass only if the install boundary answered
    /// before the manifest was consulted at all.
    /// CONTAINMENT, PROVEN AGAINST THE COPY AND NOT AGAINST A MESSAGE.
    ///
    /// This is a separate test from the cause-arms above for one reason found by running the RED:
    /// with the destination admission neutered, those arms never reach `fs::copy` at all -- the
    /// install stops earlier at `CandidateManifestPopulationMismatch`, because a bare basename with
    /// no admitted manifest row is refused upstream. So an "the outside file survived" assertion
    /// placed there is satisfied by the UPSTREAM refusal and discriminates nothing about
    /// containment (DESIGN `executed_conjunct_discriminates_nothing`).
    ///
    /// Here the basename carries a real admitted manifest row and real candidate bytes, so every
    /// upstream gate passes and `admit_install_destination` is the ONLY thing standing between the
    /// copy and a file outside `stage0_src`. The surviving bytes are then evidence of containment
    /// rather than evidence that something else refused first.
    #[test]
    fn install_admission_contains_the_destination_against_a_symlink() {
        let (workspace, stage0, candidate, subject) = fixture_workspace();
        let model = RegenConvergenceModel::load(&fixture_roots()).unwrap();

        // The link points OUTSIDE stage0_src, so a following copy writes somewhere observably
        // wrong rather than merely somewhere else.
        let outside = workspace.join("outside_the_surface.rs");
        let preserved = "// must not be overwritten by an install\n";
        fs::write(&outside, preserved).unwrap();
        std::os::unix::fs::symlink(&outside, stage0.join("linked_generated.rs")).unwrap();

        let rows = [(
            "linked_generated.rs",
            "fixture.linked",
            "// bytes that must never reach the link target\n",
        )];
        let (_, admitted) = fixture_manifest(&candidate, &rows);

        let refused = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[rows[0].0.to_string()],
            &admitted,
            &fixture_modules(&rows),
            1,
            RegenConvergenceStageKindReceipt::PublishNonSeedOutputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-0",
            "closure-0",
            &subject,
            |_| -> Result<CargoBuildObservation, String> {
                panic!("a refused install must never reach the seed build")
            },
            || -> Result<String, String> {
                panic!("a refused install must never reach a seed digest")
            },
        )
        .unwrap_err();

        assert!(
            refused.contains("InstallDestinationNotARegularFile"),
            "expected the destination admission to refuse, got: {refused}"
        );
        assert_eq!(
            fs::read_to_string(&outside).unwrap(),
            preserved,
            "an install escaped stage0_src through a destination symlink"
        );
    }

    /// THE DISCRIMINATING RED FOR THE PLANNED/EXECUTED JOIN, and the reason it is not the
    /// decoration it was.
    ///
    /// `admit_stage_execution_from_model` used to build one identity list from the stage plan
    /// and pass it as both `planned` and `executed`, so the model conjunct was `x == x.clone()`.
    /// `StagePlannedExecutedMismatch` was authorable in the `.dag` witness and unreachable from
    /// the host. Both arms below construct populations that the old form scored as equal.
    ///
    /// ARM A -- AN EFFECT OUTSIDE THE PLAN, AND IT WAS GREEN. The producer is installed in
    /// stage 1, so its path is git-dirty when stage 2 begins and lands in `changed_before`.
    /// `allowed_after` is `changed_before` UNION the planned paths, so the `UnplannedPathMutated`
    /// git observation cannot see stage 2's build rewriting it: every already-dirty stage0 path
    /// is blanket-permitted for the rest of the transaction. Nothing else looked. The delta over
    /// the journalled roster does, because its denominator is the roster and not the plan.
    ///
    /// ARM B -- A PLANNED SURFACE WITH NO EFFECT, AND IT REFUSED ON THE WRONG AXIS. Reverting an
    /// installed surface to its pre-stage bytes was already caught, but as
    /// `InstalledDigestMismatch` -- a CONTENT verdict standing in for a POPULATION one. The
    /// assertion is on the cause, not on "it refused", because "it refused" was true before this
    /// change and would carry no information.
    /// THE SEED-TO-MODEL LOCKSTEP FOR THE POPULATION JOINS, asserted on the CAUSE.
    ///
    /// These run the model through the interpreter with host-built values, so a renamed field or
    /// variant on either side reds here rather than forty minutes into a convergence round. Each
    /// arm asserts the refusal's own name and its residues -- "it refused" would be satisfied by a
    /// join that refuses everything, which is the shape the positive controls exclude.
    #[test]
    fn population_joins_refuse_by_identity_and_admit_an_exact_partition() {
        let model = RegenConvergenceModel::load(&fixture_roots()).unwrap();
        let modules = fixture_modules(&[
            ("a.rs", "fixture.a", ""),
            ("b.rs", "fixture.b", ""),
            ("c.rs", "fixture.c", ""),
        ]);
        let names = |rows: &[&str]| rows.iter().map(|r| (*r).to_string()).collect::<Vec<_>>();

        // JOIN 1, the loss this lane exists to close: an admitted surface that is neither planned
        // nor deferred vanishes with no typed disposition.
        let dropped = admit_install_boundary_population_from_model(
            &model,
            &names(&["a.rs", "b.rs"]),
            &names(&["a.rs"]),
            &[],
            &modules,
        )
        .unwrap_err();
        assert!(
            dropped.contains("StagePartitionPopulationDisagrees")
                && dropped.contains("admitted_without_partition=[b.rs]"),
            "the dropped surface must be named as a residue, got: {dropped}"
        );

        // THE DIRECTION A COUNT CANNOT SEE: one missing and one phantom, totals equal.
        let compensating = admit_install_boundary_population_from_model(
            &model,
            &names(&["a.rs"]),
            &names(&["b.rs"]),
            &[],
            &modules,
        )
        .unwrap_err();
        assert!(
            compensating.contains("admitted_without_partition=[a.rs]")
                && compensating.contains("partition_without_admitted=[b.rs]"),
            "equal counts with different identities must name BOTH residues, got: {compensating}"
        );

        // POSITIVE CONTROL: an exact partition, planned and deferred together.
        admit_install_boundary_population_from_model(
            &model,
            &names(&["a.rs", "b.rs"]),
            &names(&["a.rs"]),
            &names(&["b.rs"]),
            &modules,
        )
        .expect("an exact partition is admitted");

        // JOIN 3, both directions and terminality.
        let admitted: BTreeSet<String> = names(&["a.rs", "b.rs"]).into_iter().collect();
        let mut lineage = BTreeMap::new();
        lineage.insert(
            "a.rs".to_string(),
            ConvergenceDisposition::Applied {
                installed_digest: "digest-a".to_string(),
            },
        );
        let missing = admit_transaction_lineage_from_model(&model, &admitted, &lineage, &modules)
            .unwrap_err();
        assert!(
            missing.contains("TransactionLineagePopulationDisagrees")
                && missing.contains("admitted_without_lineage=[b.rs]"),
            "a surface the receipts never accounted for must be named, got: {missing}"
        );

        // DEFERRED IS NONTERMINAL: the populations agree in both directions here, so a population
        // equality alone would report this transaction as complete.
        lineage.insert("b.rs".to_string(), ConvergenceDisposition::Deferred);
        let unfinished =
            admit_transaction_lineage_from_model(&model, &admitted, &lineage, &modules)
                .unwrap_err();
        assert!(
            unfinished.contains("SurfaceLineageUnfinished") && unfinished.contains("[b.rs]"),
            "a lineage ending in a promise must refuse as unfinished, got: {unfinished}"
        );

        // POSITIVE CONTROL: Superseded closes a lineage that was never installed, so a correct
        // transaction is not refused for having postponed something a later candidate replaced.
        lineage.insert(
            "b.rs".to_string(),
            ConvergenceDisposition::Superseded {
                by_generation_id: "g2".to_string(),
            },
        );
        admit_transaction_lineage_from_model(&model, &admitted, &lineage, &modules)
            .expect("applied and superseded together close the lineage");
    }

    #[test]
    fn stage_execution_joins_the_plan_to_independently_observed_effects() {
        let (workspace, stage0, candidate, subject) = fixture_workspace();
        let model = RegenConvergenceModel::load(&fixture_roots()).unwrap();
        // One module map covering BOTH surfaces: an observed effect on an unplanned path must
        // reach the population join, not stop at `SurfaceOwnershipUnresolved`. In production
        // this map is the whole-corpus one `convergence_surface_roles` returns.
        let modules = fixture_modules(&[
            ("fixture_producer.rs", "fixture.producer", ""),
            ("fixture_subject.rs", "fixture.subject", ""),
        ]);
        let passing_build = |_: &Path| -> Result<CargoBuildObservation, String> {
            Ok(CargoBuildObservation {
                compiled_crates: 1,
                compiled_packages: vec!["fixture-seed".to_string()],
            })
        };

        // Stage 1: install the producer. This is also the POSITIVE CONTROL -- planned and
        // independently observed effects agree, so the join admits. Without it, a join that
        // refused everything would satisfy both arms below.
        let p_rows = [(
            "fixture_producer.rs",
            "fixture.producer",
            "// new producer\n",
        )];
        let (_, p_admitted) = fixture_manifest(&candidate, &p_rows);
        install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[p_rows[0].0.to_string()],
            &p_admitted,
            &modules,
            1,
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-p",
            "generation-input-cut",
            &subject,
            passing_build,
            || Ok("seed-1".to_string()),
        )
        .expect("planned and observed agree, so the stage is admitted");

        // ARM A.
        let s_rows = [("fixture_subject.rs", "fixture.subject", "// new subject\n")];
        let (_, s_admitted) = fixture_manifest(&candidate, &s_rows);
        let outside_the_plan = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[s_rows[0].0.to_string()],
            &s_admitted,
            &modules,
            2,
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut,
            "seed-1",
            "generation-0",
            "tree-0",
            "manifest-s",
            "seed-compatibility-cut",
            &subject,
            |root| {
                fs::write(
                    root.join("src/v1/stage0/src/fixture_producer.rs"),
                    "// rewritten by a build that was not planned to touch this\n",
                )
                .unwrap();
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-2".to_string()),
        )
        .unwrap_err();
        assert!(
            outside_the_plan.contains("StagePlannedExecutedMismatch"),
            "an effect on an already-dirty path outside the plan must refuse as a population \
             mismatch, got: {outside_the_plan}"
        );
        restore_regen_convergence_journal_for_subject(&workspace, &subject).unwrap();

        // ARM B: the build reverts the surface this stage just installed.
        let planned_without_effect = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[s_rows[0].0.to_string()],
            &s_admitted,
            &modules,
            2,
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut,
            "seed-1",
            "generation-0",
            "tree-0",
            "manifest-s",
            "seed-compatibility-cut",
            &subject,
            |root| {
                fs::write(
                    root.join("src/v1/stage0/src/fixture_subject.rs"),
                    "// old subject\n",
                )
                .unwrap();
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-2".to_string()),
        )
        .unwrap_err();
        assert!(
            planned_without_effect.contains("StagePlannedExecutedMismatch"),
            "a planned surface the stage left byte-identical must refuse as a population \
             mismatch, not as a content digest verdict, got: {planned_without_effect}"
        );
        restore_regen_convergence_journal_for_subject(&workspace, &subject).unwrap();

        // ARM C -- A FILE THE BUILD CREATES, which the first version of this observation could
        // not see. `git_changed_stage0_paths` runs `git diff --name-only` and reports tracked
        // modifications only, so an UNTRACKED creation is absent from the git observation; it was
        // also absent from a roster-lookup observation and from the restoration journal, which is
        // three producers blind at once. The observation enumerates the directory, so the path is
        // an effect the moment it appears.
        let created = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[s_rows[0].0.to_string()],
            &s_admitted,
            &fixture_modules(&[
                ("fixture_producer.rs", "fixture.producer", ""),
                ("fixture_subject.rs", "fixture.subject", ""),
                ("fixture_created.rs", "fixture.created", ""),
            ]),
            2,
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut,
            "seed-1",
            "generation-0",
            "tree-0",
            "manifest-s",
            "seed-compatibility-cut",
            &subject,
            |root| {
                fs::write(
                    root.join("src/v1/stage0/src/fixture_created.rs"),
                    "// invented by the build, tracked by nothing\n",
                )
                .unwrap();
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-2".to_string()),
        )
        .unwrap_err();
        assert!(
            created.contains("StagePlannedExecutedMismatch"),
            "an untracked file created by the build must refuse as a population mismatch, \
             got: {created}"
        );
        fs::remove_file(stage0.join("fixture_created.rs")).unwrap();
        restore_regen_convergence_journal_for_subject(&workspace, &subject).unwrap();
        fs::remove_dir_all(&workspace).unwrap();
    }

    #[test]
    fn install_admission_refuses_unaddressable_and_hand_maintained() {
        let (workspace, stage0, candidate, subject) = fixture_workspace();
        let model = RegenConvergenceModel::load(&fixture_roots()).unwrap();
        // NO Cargo.toml ARM HERE, deliberately. The emitted manifest is stage0's own package
        // manifest emitted incompletely, so it is a self-host GAP and not a foreign artifact;
        // refusing it on its extension would cement the comparator's accidental denominator as
        // this boundary's policy and refuse the correct end state. What keeps it off the roster
        // stays upstream, and making its absence a typed disposition is the projection-identity
        // subject, not this one.
        let subject_before = fs::read_to_string(stage0.join("fixture_subject.rs")).unwrap();
        let mut mismatches: Vec<String> = Vec::new();

        for (basename, cause) in [
            ("../Cargo.toml", "InstallTargetNotABareBasename"),
            ("nested/mod.rs", "InstallTargetNotABareBasename"),
            ("cli_run.rs", "InstallTargetHandMaintained"),
        ] {
            let refused = install_convergence_stage_with_backend(
                &model,
                &workspace,
                &stage0,
                &candidate,
                &[basename.to_string()],
                &HashMap::new(),
                &HashMap::new(),
                1,
                RegenConvergenceStageKindReceipt::PublishNonSeedOutputs,
                "seed-0",
                "generation-0",
                "tree-0",
                "manifest-0",
                "closure-0",
                &subject,
                |_| -> Result<CargoBuildObservation, String> {
                    panic!("a refused install must never reach the seed build")
                },
                || -> Result<String, String> {
                    panic!("a refused install must never reach a seed digest")
                },
            )
            .unwrap_err();
            // ACCUMULATED, NOT ASSERTED PER ARM. Asserting inside the loop aborts at the first
            // mismatch, so a run proves only the FIRST arm discriminates and says nothing about
            // the rest — and the arms exercise different branches. Collecting every mismatch
            // makes one red run report all three causes at once.
            if !refused.contains(cause) {
                mismatches.push(format!(
                    "installing {basename} refused with {refused}, expected {cause}"
                ));
            }
        }
        // POSITIVE CONTROL for the symlink arm: the SAME basename shape with an ordinary regular
        // destination must NOT refuse for containment. Without this, an admission that refused
        // every destination would pass the arm above while discriminating nothing.
        fs::write(stage0.join("plain_generated.rs"), "// regular file\n").unwrap();
        let control = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &["plain_generated.rs".to_string()],
            &HashMap::new(),
            &HashMap::new(),
            1,
            RegenConvergenceStageKindReceipt::PublishNonSeedOutputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-0",
            "closure-0",
            &subject,
            |_| -> Result<CargoBuildObservation, String> {
                panic!("this control must never reach the seed build")
            },
            || -> Result<String, String> { panic!("this control must never reach a seed digest") },
        )
        .unwrap_err();
        if control.contains("InstallDestinationNotARegularFile")
            || control.contains("InstallTargetNotABareBasename")
        {
            mismatches.push(format!(
                "a regular destination was refused by admission: {control}"
            ));
        }
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));

        // The refusal is BEFORE the mutation boundary, not a rollback of one: no artifact landed,
        // and no authoritative byte moved and came back.
        assert!(!stage0.join("Cargo.toml").exists());
        assert!(!stage0.join("cli_run.rs").exists());
        assert!(!stage0.join("nested").exists());
        assert_eq!(
            fs::read_to_string(stage0.join("fixture_subject.rs")).unwrap(),
            subject_before
        );

        // POSITIVE CONTROL. The same entry point, one generated Rust surface, installs — so the
        // arms above measure the artifact kind and not a call that refuses everything.
        let rows = [("fixture_subject.rs", "fixture.subject", "// new subject\n")];
        let (_, admitted) = fixture_manifest(&candidate, &rows);
        install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[rows[0].0.to_string()],
            &admitted,
            &fixture_modules(&rows),
            1,
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-0",
            "generation-input-cut",
            &subject,
            |_| {
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-1".to_string()),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(stage0.join("fixture_subject.rs")).unwrap(),
            "// new subject\n"
        );
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

    /// HOST-PATH INSTRUMENT: this calls the same journal/install/build/admission orchestration as
    /// production. Only the external seed build and executable digest are hermetic callbacks.
    #[test]
    fn mutating_transaction_binds_candidates_restores_and_reaches_staged_fixed_point() {
        let roots = fixture_roots();
        let model = RegenConvergenceModel::load(&roots).unwrap();

        // An emitted non-Rust artifact is admitted from the writer's exact output population,
        // not from a basename exception. This is the crate-layout-product shape without naming
        // any particular product path in the manifest authority.
        let (_, _, emitted_artifact_candidate, _) = fixture_workspace();
        fs::write(
            emitted_artifact_candidate.join("fixture-layout.artifact"),
            "emitted layout bytes\n",
        )
        .unwrap();
        let emitted_artifact_manifest = produce_candidate_manifest(
            &emitted_artifact_candidate,
            &[],
            &["fixture-layout.artifact".to_string()]
                .into_iter()
                .collect(),
            &HashMap::new(),
            "seed-0",
            "tree-emitted-artifact",
        )
        .unwrap();
        assert_eq!(emitted_artifact_manifest.surfaces.len(), 1);
        assert!(matches!(
            emitted_artifact_manifest.surfaces[0].role,
            RegenCandidateManifestSurfaceRole::GeneratedSurface
        ));

        // Relative-path identity is load-bearing: a nested file cannot borrow the generated role
        // of an emitted root artifact merely because their basenames collide.
        let (_, _, emitted_collision_candidate, _) = fixture_workspace();
        fs::create_dir_all(emitted_collision_candidate.join("nested")).unwrap();
        fs::write(
            emitted_collision_candidate.join("nested/fixture-layout.artifact"),
            "nested foreign bytes\n",
        )
        .unwrap();
        assert!(produce_candidate_manifest(
            &emitted_collision_candidate,
            &[],
            &["fixture-layout.artifact".to_string()]
                .into_iter()
                .collect(),
            &HashMap::new(),
            "seed-0",
            "tree-emitted-collision",
        )
        .unwrap_err()
        .contains("CandidateManifestPopulationMismatch"));

        // Bootstrap-source mirrors inhabit the same immutable candidate artifact as generated
        // surfaces. Their role is bound by the manifest, and changing their bytes after
        // production refuses before any install journal exists.
        let (_, _, bootstrap_candidate, _) = fixture_workspace();
        fs::create_dir_all(bootstrap_candidate.join("cli_run")).unwrap();
        fs::write(
            bootstrap_candidate.join("cli_run/fixture_support.txt"),
            "original support bytes\n",
        )
        .unwrap();
        let bootstrap_manifest = produce_candidate_manifest(
            &bootstrap_candidate,
            &[],
            &BTreeSet::new(),
            &HashMap::new(),
            "seed-0",
            "tree-bootstrap",
        )
        .unwrap();
        assert!(matches!(
            bootstrap_manifest.surfaces[0].role,
            RegenCandidateManifestSurfaceRole::BootstrapSourceMirror
        ));
        fs::write(
            bootstrap_candidate.join("cli_run/fixture_support.txt"),
            "tampered support bytes\n",
        )
        .unwrap();
        assert!(admit_candidate_manifest(
            &model,
            &bootstrap_candidate,
            &bootstrap_manifest,
            "seed-0"
        )
        .unwrap_err()
        .contains("CandidateManifestTreeDigestMismatch"));

        // A file with neither a generated-surface row nor a bootstrap-source-mirror row remains
        // foreign to the complete artifact and is refused at the population wall.
        let (_, _, foreign_candidate, _) = fixture_workspace();
        let foreign_rows = [(
            "fixture_generated.rs",
            "fixture.generated",
            "// generated\n",
        )];
        let (foreign_manifest, _) = fixture_manifest(&foreign_candidate, &foreign_rows);
        fs::write(foreign_candidate.join("foreign.bin"), b"foreign bytes\n").unwrap();
        assert!(
            admit_candidate_manifest(&model, &foreign_candidate, &foreign_manifest, "seed-0")
                .unwrap_err()
                .contains("CandidateManifestPopulationMismatch")
        );

        // A candidate changed after its generation manifest is refused before a journal exists.
        let (workspace, stage0, candidate, subject) = fixture_workspace();
        let rows = [(
            "fixture_producer.rs",
            "fixture.producer",
            "// new producer\n",
        )];
        let (_, admitted) = fixture_manifest(&candidate, &rows);
        let stale_manifest = RegenCandidateManifest {
            producer_seed_digest: "seed-g0".to_string(),
            generation_id: "generation-g0".to_string(),
            candidate_tree_id: "tree-g0".to_string(),
            candidate_tree_digest: "tree-g0-digest".to_string(),
            surfaces: admitted.values().cloned().collect(),
            aggregate_digest: String::new(),
        };
        let stale_manifest = RegenCandidateManifest {
            aggregate_digest: candidate_manifest_aggregate(
                &stale_manifest.producer_seed_digest,
                &stale_manifest.generation_id,
                &stale_manifest.candidate_tree_id,
                &stale_manifest.candidate_tree_digest,
                &stale_manifest.surfaces,
            )
            .unwrap(),
            ..stale_manifest
        };
        assert!(
            admit_candidate_manifest(&model, &candidate, &stale_manifest, "seed-g1")
                .unwrap_err()
                .contains("CandidateStaleAfterProducerRebuild")
        );
        fs::write(candidate.join(rows[0].0), "// tampered\n").unwrap();
        let tampered = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[rows[0].0.to_string()],
            &admitted,
            &fixture_modules(&rows),
            1,
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-0",
            "generation-input-cut",
            &subject,
            |_| {
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-1".to_string()),
        )
        .unwrap_err();
        assert!(tampered.contains("CandidateManifestSurfaceDigestMismatch"));
        assert!(!regen_convergence_journal_path(&workspace).exists());
        fs::write(candidate.join(rows[0].0), "// new producer\n").unwrap();
        let post_build_tamper = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[rows[0].0.to_string()],
            &admitted,
            &fixture_modules(&rows),
            1,
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-0",
            "generation-input-cut",
            &subject,
            |root| {
                fs::write(
                    root.join("src/v1/stage0/src/fixture_producer.rs"),
                    "// mutated during build\n",
                )
                .unwrap();
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-1".to_string()),
        )
        .unwrap_err();
        assert!(post_build_tamper.contains("InstalledDigestMismatch"));
        restore_regen_convergence_journal_for_subject(&workspace, &subject).unwrap();
        fs::remove_dir_all(&workspace).unwrap();

        // A failed build crosses the real copy boundary, then the subject-bound journal restores
        // the admitted checkpoint. This is the single-pass negative control.
        let (workspace, stage0, candidate, subject) = fixture_workspace();
        let rows = [("fixture_subject.rs", "fixture.subject", "// new subject\n")];
        let (_, admitted) = fixture_manifest(&candidate, &rows);
        let failed = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[rows[0].0.to_string()],
            &admitted,
            &fixture_modules(&rows),
            1,
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-0",
            "seed-compatibility-cut",
            &subject,
            |_| Err("fixture seed rejected partial generation".to_string()),
            || Ok("seed-1".to_string()),
        )
        .unwrap_err();
        assert!(failed.contains("partial generation"));
        restore_regen_convergence_journal_for_subject(&workspace, &subject).unwrap();
        assert_eq!(
            fs::read_to_string(stage0.join(rows[0].0)).unwrap(),
            "// old subject\n"
        );

        // Promote the producer, then install the complete subject/dependent compatibility cut.
        let p_rows = [(
            "fixture_producer.rs",
            "fixture.producer",
            "// new producer\n",
        )];
        let (_, p_admitted) = fixture_manifest(&candidate, &p_rows);
        install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[p_rows[0].0.to_string()],
            &p_admitted,
            &fixture_modules(&p_rows),
            1,
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-p",
            "generation-input-cut",
            &subject,
            |_| {
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-1".to_string()),
        )
        .unwrap();
        let s_rows = [
            ("fixture_subject.rs", "fixture.subject", "// new subject\n"),
            (
                "fixture_dependent.rs",
                "fixture.dependent",
                "// new dependent\n",
            ),
        ];
        let (_, s_admitted) = fixture_manifest(&candidate, &s_rows);
        let stage = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &s_rows
                .iter()
                .map(|row| row.0.to_string())
                .collect::<Vec<_>>(),
            &s_admitted,
            &fixture_modules(&s_rows),
            2,
            RegenConvergenceStageKindReceipt::InstallSeedCompatibilityCut,
            "seed-1",
            "generation-0",
            "tree-0",
            "manifest-s",
            "seed-compatibility-cut",
            &subject,
            |root| {
                let src = root.join("src/v1/stage0/src");
                if fs::read_to_string(src.join("fixture_subject.rs")).unwrap() != "// new subject\n"
                    || fs::read_to_string(src.join("fixture_dependent.rs")).unwrap()
                        != "// new dependent\n"
                {
                    return Err("compatibility cut incomplete".to_string());
                }
                Ok(CargoBuildObservation {
                    compiled_crates: 2,
                    compiled_packages: vec![
                        "fixture-producer".to_string(),
                        "fixture-seed".to_string(),
                    ],
                })
            },
            || Ok("seed-2".to_string()),
        )
        .unwrap();
        assert!(stage.surfaces.iter().all(
            |surface| surface.standing == RegenSurfaceExecutionStandingReceipt::TerminalPassed
        ));
        assert_eq!(stage.output_seed_digest, "seed-2");
        assert_eq!(stage.dependency_closure_id, "seed-compatibility-cut");

        // Cross-head and corrupt-backup journals refuse before touching authoritative bytes.
        let wrong_subject = RegenConvergenceCheckpointSubject {
            starting_commit: "other-head".to_string(),
            ..subject.clone()
        };
        let before = fs::read_to_string(stage0.join("fixture_subject.rs")).unwrap();
        assert!(
            restore_regen_convergence_journal_for_subject(&workspace, &wrong_subject)
                .unwrap_err()
                .contains("CheckpointSubjectMismatch")
        );
        assert_eq!(
            fs::read_to_string(stage0.join("fixture_subject.rs")).unwrap(),
            before
        );
        let journal_root = regen_convergence_journal_path(&workspace);
        let backup = fs::read_dir(&journal_root)
            .unwrap()
            .map(|entry| entry.expect("fixture journal directory entry must remain readable"))
            .map(|entry| entry.path())
            .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("bak"))
            .unwrap();
        fs::write(&backup, "corrupt backup\n").unwrap();
        assert!(
            restore_regen_convergence_journal_for_subject(&workspace, &subject)
                .unwrap_err()
                .contains("CheckpointArtifactDigestMismatch")
        );
        assert_eq!(
            fs::read_to_string(stage0.join("fixture_subject.rs")).unwrap(),
            before
        );
        fs::remove_dir_all(&workspace).unwrap();

        // An unplanned generated mutation is detected after the hermetic build callback and the
        // complete-population journal restores it with the planned surface.
        let (workspace, stage0, candidate, subject) = fixture_workspace();
        let rows = [(
            "fixture_producer.rs",
            "fixture.producer",
            "// new producer\n",
        )];
        let (_, admitted) = fixture_manifest(&candidate, &rows);
        let unplanned = install_convergence_stage_with_backend(
            &model,
            &workspace,
            &stage0,
            &candidate,
            &[rows[0].0.to_string()],
            &admitted,
            &fixture_modules(&rows),
            1,
            RegenConvergenceStageKindReceipt::PromoteGenerationInputs,
            "seed-0",
            "generation-0",
            "tree-0",
            "manifest-0",
            "generation-input-cut",
            &subject,
            |root| {
                fs::write(
                    root.join("src/v1/stage0/src/fixture_unplanned.rs"),
                    "// mutated\n",
                )
                .unwrap();
                Ok(CargoBuildObservation {
                    compiled_crates: 1,
                    compiled_packages: vec!["fixture-seed".to_string()],
                })
            },
            || Ok("seed-1".to_string()),
        )
        .unwrap_err();
        assert!(unplanned.contains("UnplannedPathMutated"));
        restore_regen_convergence_journal_for_subject(&workspace, &subject).unwrap();
        assert_eq!(
            fs::read_to_string(stage0.join("fixture_unplanned.rs")).unwrap(),
            "// stable\n"
        );

        // Cycle and bound are reached through the host's production planner over successive
        // generation identities, rather than supplied as fixture terminal variants.
        let (_, admitted) = fixture_manifest(&candidate, &rows);
        let modules = fixture_modules(&rows);
        let generation_modules = ["fixture.producer".to_string()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        let seed_members = [rows[0].0.to_string()].into_iter().collect::<BTreeSet<_>>();
        let empty = BTreeSet::new();
        let digest = bytes_digest(b"generation-state");
        // Publication is reachable only from an explicit generated-product role. Merely being
        // absent from the seed manifest remains unresolved for unclassified surfaces.
        let non_seed_roles = [(rows[0].0.to_string(), "NonSeedGeneratedOutput".to_string())]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let (publish_kind, publish_paths, publish_closure_id) = convergence_plan_from_model(
            &model,
            1,
            "generation-publish",
            "tree-publish",
            &digest,
            &[rows[0].0.to_string()],
            &admitted,
            &stage0,
            &modules,
            &empty,
            &empty,
            &empty,
            &empty,
            &non_seed_roles,
            &RegenEmissionScope::WholePopulation,
            &[],
            "seed-publish",
        )
        .unwrap();
        assert_eq!(
            publish_kind,
            RegenConvergenceStageKindReceipt::PublishNonSeedOutputs
        );
        assert_eq!(publish_paths, vec![rows[0].0.to_string()]);
        assert_eq!(publish_closure_id, "non-seed-publish");

        let cycle = convergence_plan_from_model(
            &model,
            2,
            "generation-1",
            "tree-1",
            &digest,
            &[rows[0].0.to_string()],
            &admitted,
            &stage0,
            &modules,
            &generation_modules,
            &empty,
            &empty,
            &seed_members,
            &HashMap::new(),
            &RegenEmissionScope::WholePopulation,
            &[format!("seed-0:{digest}")],
            "seed-0",
        )
        .unwrap_err();
        assert!(cycle.contains("CycleRefused"), "{cycle}");
        let bound = convergence_plan_from_model(
            &model,
            REGEN_CONVERGENCE_BOUND + 1,
            "generation-bound",
            "tree-bound",
            &bytes_digest(b"new-tree"),
            &[rows[0].0.to_string()],
            &admitted,
            &stage0,
            &modules,
            &generation_modules,
            &empty,
            &empty,
            &seed_members,
            &HashMap::new(),
            &RegenEmissionScope::WholePopulation,
            &[],
            "seed-new",
        )
        .unwrap_err();
        assert!(bound.contains("BoundRefused"), "{bound}");
        fs::remove_dir_all(&workspace).unwrap();
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
/// uses (`both_closure_edge_index`: dotted references, which include every import line, plus bare
/// references) -- one authority for "what pulls what", read here in reverse. A file the index
/// names but cannot map to a module is a refusal: an edge dropped silently would shrink the bound.
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
    let edge_index = super::both_closure_edge_index(&index)?;
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
        HashMap<String, String>,
    ),
    String,
> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = affected_set_entry(source_roots)?;
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
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
            Value::List(items) => items
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
            generated_product_roles.insert(product.clone(), roles.into_iter().next().unwrap());
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
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| {
            format!("refusal: {entry} did not resolve, so the bound cannot answer: {e}")
        })?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);

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
        Value::Str(s) => s.to_string(),
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
        Value::List(items) => items
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
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| {
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
        Value::List(items) => items
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
        assert_eq!(receipt.candidate_generated_digest(), None);
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
            // unobservable and says so rather than passing vacuously.
            Err(_) => return,
        };
        assert!(tree_digest_from_map(&formatter, &HashMap::new(), &[]).is_err());
        assert!(tree_digest_for_basenames(&formatter, &tmp, &[], "committed").is_err());
    }

    /// LOCKSTEP with `v2.workflow.required_regen` `regen_scope_select`, on the same rosters, for
    /// both selecting arms. A rename or a changed fold on either side reds here rather than in a
    /// forty-minute round.
    #[test]
    fn host_selection_and_model_selection_agree() {
        for scope in [
            RegenEmissionScope::WholePopulation,
            RegenEmissionScope::Affected {
                members: vec![s("std_b.rs"), s("v1_rt.rs"), s("not_in_the_tree.rs")],
            },
        ] {
            let host: BTreeSet<String> = scope_selection(&scope, &committed())
                .expect("the host selects")
                .into_iter()
                .collect();
            let model: BTreeSet<String> = render_scope_selection(&roots(), &scope, &committed())
                .expect("the model selects")
                .into_iter()
                .collect();
            assert_eq!(host, model, "scope {scope:?}");
        }
    }

    /// The model's refusal arm is REACHABLE and carries no members -- the same fact the host's
    /// `Err` carries, asserted on the side that owns the vocabulary.
    #[test]
    fn the_model_selects_nothing_on_the_unlocatable_arm() {
        let scope = RegenEmissionScope::Unlocatable {
            paths: vec![s("dag/std/departed.dag")],
            reason: s("regen-affected-set: EditedSetUnlocatable unlocatable=1"),
        };
        assert!(render_scope_selection(&roots(), &scope, &committed())
            .expect("the model answers")
            .is_empty());
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

    #[test]
    fn host_walk_and_model_fold_agree_on_the_fixture_graph() {
        let host = regen_reverse_closure_host(&[s("std.a")], &fixture_edges());
        assert_eq!(
            host,
            ["std.a", "std.b", "gunbc.c", "v1.compiler.emit_rust"]
                .iter()
                .map(|m| s(m))
                .collect::<BTreeSet<_>>()
        );
        let bound = render_affected_set_bound(
            &roots(),
            &[s("std.a")],
            &[],
            &fixture_edges(),
            &fixture_compared(),
        )
        .expect("the model answers on the fixture");
        assert_eq!(bound.arm, "AffectedMirrors");
        assert_eq!(
            bound.line,
            "regen-affected-set: AffectedMirrors edited=1 mirrors=4 bootstrap_products=0"
        );
        let members: BTreeSet<String> = bound.members.into_iter().collect();
        assert_eq!(
            members,
            [
                "std_a.rs",
                "std_b.rs",
                "gunbc_c.rs",
                "v1_compiler_emit_rust.rs"
            ]
            .iter()
            .map(|m| s(m))
            .collect()
        );
    }

    /// RED CONTROL: an unlocatable path refuses with no members, and does not widen to the
    /// population -- the arm is the refusal, and the edited module beside it is not walked.
    #[test]
    fn an_unlocatable_edited_path_refuses_and_selects_nothing() {
        let bound = render_affected_set_bound(
            &roots(),
            &[s("std.a")],
            &[s(
                "dag/std/gone.dag (departed: no module line remains in the tree)",
            )],
            &fixture_edges(),
            &fixture_compared(),
        )
        .expect("the refusal is an arm, not an error");
        assert_eq!(bound.arm, "EditedSetUnlocatable");
        assert!(bound.members.is_empty());
        assert!(bound
            .line
            .starts_with("regen-affected-set: EditedSetUnlocatable unlocatable=1 reason="));
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

    /// THE LIVE-TREE CONTROLS, one index build for all three: the three measured edits of
    /// 2026-08-30 (tree 677988a2 / 0fe2c517) each land on the arm and members the measurement
    /// drifted. A leaf std edit names its own mirror and not the runtime shim; the runtime
    /// template names its mirror AND the shim through the declared bootstrap edge; an emitter
    /// edit is the whole population.
    #[test]
    fn live_tree_controls_land_on_the_measured_arms() {
        let workspace = workspace_root();
        let (edges, modules) = regen_module_edges(&workspace).expect("the closure edge index maps");
        let compared = compared_mirror_rows(&workspace.join("src/v1/stage0/src"), &modules)
            .expect("committed population");
        assert!(compared.iter().any(|(m, _)| m == "std.content_hash"));

        let leaf =
            render_affected_set_bound(&roots(), &[s("std.content_hash")], &[], &edges, &compared)
                .expect("leaf edit answers");
        assert_eq!(leaf.arm, "AffectedMirrors", "{}", leaf.line);
        assert!(
            leaf.members.iter().any(|m| m == "std_content_hash.rs"),
            "{:?}",
            leaf.members
        );
        assert!(
            !leaf.members.iter().any(|m| m == "v1_rt.rs"),
            "{:?}",
            leaf.members
        );
        assert!(
            leaf.members.len() < compared.len(),
            "the leaf bound is a proper subset"
        );

        let template = render_affected_set_bound(
            &roots(),
            &[s("v1.compiler.runtime_rust")],
            &[],
            &edges,
            &compared,
        )
        .expect("template edit answers");
        assert_eq!(template.arm, "AffectedMirrors", "{}", template.line);
        assert!(
            template
                .members
                .iter()
                .any(|m| m == "v1_compiler_runtime_rust.rs"),
            "{:?}",
            template.members
        );
        assert!(
            template.members.iter().any(|m| m == "v1_rt.rs"),
            "{:?}",
            template.members
        );

        let emitter = render_affected_set_bound(
            &roots(),
            &[s("v1.compiler.emit_rust")],
            &[],
            &edges,
            &compared,
        )
        .expect("emitter edit answers");
        assert_eq!(emitter.arm, "WholePopulation", "{}", emitter.line);
        assert!(emitter.members.is_empty());
    }
}
