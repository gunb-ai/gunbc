//! Host realization for `v2.workflow.required_regen` — committed seed vs fresh emit.

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
use crate::v1_compiler_artifact::RenderTarget;
use crate::v1_compiler_compile::{
    compile_sources, stage0_self_compile_refusal_message, SourceFile,
};
use crate::v1_rt;
use bootstrap_stage0_crate_layout_generated::{
    EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES, HAND_MAINTAINED_STAGE0_DIRS,
    HAND_MAINTAINED_STAGE0_FILES,
};

/// Bumped from `.v1` when the flat eight-field record split into the two-variant carrier below.
///
/// THE BUMP IS LOAD-BEARING ONLY BECAUSE `read_receipt` COMPARES IT. An earlier revision of this
/// comment claimed a stale `.v1` receipt "fails to deserialize into the new shape". That was
/// FALSE, and review caught it: serde ignores unknown fields by default, and a v1 record carries
/// every field the reader requires plus the removed `fixed_point_equal`, so it parsed cleanly.
/// The version string was written three times and read zero times -- decoration, not a version.
/// Two things now make the claim true rather than aspirational: `deny_unknown_fields` on the
/// carrier, and an explicit equality check in `read_receipt`. Writing a version nobody compares is
/// the same class of defect as the impersonation this module exists to close -- an artifact
/// asserting a property that nothing establishes.
const RECEIPT_SCHEMA: &str = "gunbc.regen_receipt.v2";

/// Evidence produced by the FIRST regen pass, referenced by the second.
///
/// It carries the `commit_sha` it was measured at so that a consumer READS which tree these facts
/// describe instead of inferring it from the receipt that quotes them. A reference that did not
/// name its subject would be the impersonation this type exists to end, wearing better vocabulary.
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
/// This was one flat eight-field record, which forced the second pass to populate fields it had
/// not measured. The only source available was the receipt the first pass left on disk, so four of
/// six were copied through verbatim: `committed_generated_digest`, `first_generation_equal`,
/// `changed_paths`, `candidate_artifact`. The product is stamped with the SECOND pass `commit_sha`
/// while carrying the FIRST pass answers -- internally consistent, schema-valid, and silent about
/// which tree four of its fields describe. Validating stayed impossible because nothing in the
/// artifact recorded the provenance that would have been validated.
///
/// The split makes the fabrication UNWRITABLE rather than detectable: `FixedPoint` has no
/// `first_generation_equal` field to fill in, so there is no value to copy and no check to pass
/// (DESIGN 4b structural impossibility, one rung above the validation that would otherwise sit
/// here). Computing all six in the second pass was the alternative and is worse -- it would make
/// pass two re-derive `first_generation_equal` against the committed tree, which is pass one's
/// question, fusing two authorities into one row (DESIGN 3).
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
    },
    /// A POPULATION REFUSAL HAS NO MEASUREMENT, AND THIS VARIANT HAS NOWHERE TO WRITE ONE.
    ///
    /// The refusal happens BEFORE any content comparison, so there is no first generation to have
    /// a digest of and no comparison to have an answer to. Written as a `FirstGeneration` receipt
    /// this forced three fabrications into a persisted artifact: two `refused:population` strings
    /// standing in digest positions, and `first_generation_equal: false` standing where "not
    /// asked" belonged. The Bool was the load-bearing one, because unlike the string sentinels it
    /// is INDISTINGUISHABLE from a real answer -- `false` is exactly what an honest unequal
    /// comparison writes.
    ///
    /// It was not merely misleading in place, it CROSSED THE PROCESS BOUNDARY. The receipt file is
    /// the only carrier between the two passes, `read_receipt` returned the fabricated `false` as
    /// a `PriorMeasurement`, `PriorReceiptRef` copied it into the fixed-point receipt, and
    /// `claim_executor` printed it as `referenced_first_generation_equal=false`. A standalone
    /// `--required-regen-fixed-point` at the same commit passes the cross-tree guard precisely
    /// BECAUSE the refusal happened at this commit, so the guard admits it rather than catching
    /// it. The run still reds -- pass 2's real digest cannot equal `refused:population` -- so this
    /// was diagnostic harm, not fail-open: the operator was told the first generation compared
    /// unequal when it never compared at all.
    ///
    /// "Not asked" was already modelled correctly THREE times in this same file --
    /// `FirstGeneration::NotMeasured`, the `first_generation_equal()` accessor's documented
    /// `Option`, and `fixed_point_equal()`'s mirror of it -- so the Bool was a fourth
    /// representation disagreeing with three correct neighbours, not a missing state (DESIGN §3).
    /// The fix follows the move this file already made when the flat eight-field record split:
    /// the variant has no `first_generation_equal` field and no digest fields, so the fabrication
    /// is UNWRITABLE rather than detectable (DESIGN §4b, structural impossibility). What it
    /// carries instead is the one thing a refusal actually knows -- why it refused.
    #[serde(rename = "refused")]
    Refused {
        schema: String,
        commit_sha: String,
        authority_digest: String,
        reason: String,
        candidate_artifact: String,
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

impl RegenReceipt {
    /// The tree THIS pass ran against.
    pub fn commit_sha(&self) -> &str {
        match self {
            RegenReceipt::FirstGeneration { commit_sha, .. } => commit_sha,
            RegenReceipt::Refused { commit_sha, .. } => commit_sha,
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
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    /// `Some` only where the pass measured it -- the mirror of the above.
    pub fn fixed_point_equal(&self) -> Option<bool> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
            RegenReceipt::Refused { .. } => None,
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
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    /// The referenced first-pass evidence, present only on `FixedPoint`.
    /// The digest of the tree THIS pass emitted.
    ///
    /// `Some` exactly where a candidate digest was measured. This accessor was TOTAL while the
    /// only two variants both measured one; `Refused` measures none, so the argument that made it
    /// total no longer holds and the honest signature follows the same rule its Option-returning
    /// siblings already state -- Option because the other variant genuinely does not measure that
    /// fact. Returning a sentinel string here instead would put the fabrication straight back,
    /// one accessor lower than where it was removed.
    ///
    /// Its consumer is the composed `--required-ci` run, which hands pass 1's digest to the
    /// fixed-point pass IN MEMORY rather than having it re-read the receipt file the previous
    /// process wrote. `run_required_regen_fixed_point` has always taken `pass1_digest:
    /// Option<String>`; before the phases shared a process there was no way to supply it.
    pub fn candidate_generated_digest(&self) -> Option<&str> {
        match self {
            RegenReceipt::FirstGeneration {
                candidate_generated_digest,
                ..
            } => Some(candidate_generated_digest),
            RegenReceipt::Refused { .. } => None,
            RegenReceipt::FixedPoint {
                candidate_generated_digest,
                ..
            } => Some(candidate_generated_digest),
        }
    }

    /// Why the pass refused, present only on `Refused`. The refusal reason is the one fact a
    /// refusal genuinely measured, and carrying it on the receipt is what lets the next pass
    /// refuse with the ORIGINAL cause instead of inventing a comparison result to stand for it.
    pub fn refusal_reason(&self) -> Option<&str> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
            RegenReceipt::Refused { reason, .. } => Some(reason),
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    pub fn prior(&self) -> Option<&PriorReceiptRef> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
            RegenReceipt::Refused { .. } => None,
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
    /// generation to have a digest OF. This field carries that distinction for the IN-PROCESS
    /// handoff; `RegenReceipt::Refused` now carries it across the process boundary, so neither
    /// route has a digest position for a refusal to fill. The receipt previously did have one and
    /// filled it with the sentinel `refused:population`, which is why this field was introduced:
    /// handing the receipt to the fixed-point phase would have compared a real pass-two digest
    /// against sentinel prose and reported a determinism failure nobody measured -- a fabricated
    /// plausible output (DESIGN §5), and a convincing one, since the message names two digests
    /// and looks exactly like a genuine mismatch. Both carriers now model it; this one is not
    /// redundant with the receipt, because the in-process handoff must not have to go through a
    /// file at all.
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

pub fn run_required_regen(
    candidate_dir_rel: &str,
    receipt_rel: &str,
) -> Result<RequiredRegenOutcome, String> {
    let workspace = workspace_root();
    let candidate_dir = workspace.join(candidate_dir_rel);
    let receipt_path = workspace.join(receipt_rel);
    let stage0_src = workspace.join("src/v1/stage0/src");
    let run_started = Instant::now();

    // ADMISSION, AND IT IS THE FIRST THING THIS PHASE DOES. Every normalize below needs the
    // formatter, so an absent one is decidable here -- before an emit, a digest or a comparison
    // has been paid for. It used to be discovered at the first spawn, ~50 minutes in.
    let formatter = ResolvedFormatter::admit()?;

    let commit_sha = git_head_sha(&workspace)?;
    // PHASE MARKS. Every boundary below is stamped through `v1_rt::trace_mark`, the same
    // instrument compile.dag already uses for its stages, so `--regen-round-cost` reads one
    // ledger for the whole round rather than a second set of clocks kept beside the first.
    // The labels are `gunbc.regen_round_cost` `regen_round_phase_label`, verbatim.
    v1_rt::trace_mark("regen.corpus_load.begin".to_string());
    let sources = super::regen_input_sources(&workspace)?;
    v1_rt::trace_mark("regen.corpus_load.done".to_string());
    let authority_digest = authority_digest_from_sources(&sources)?;
    let formatter = formatter.with_normalize_cache(&workspace)?;

    // PRODUCTION, THEN ADJUDICATION -- and the order is the whole repair.
    //
    // Every refusal below used to return BEFORE the candidate tree was written, so the run that
    // refuses was exactly the run that destroyed the artifact needed to close it. The population
    // arm is not a hypothetical: adding a module to the v1 seed closure emits a mirror the
    // committed tree does not have, which is `emitted_not_committed` by construction on that
    // module's first commit, and the author's only route to the file they are being told to
    // commit is this tree.
    //
    // THIS IS WHY THE SHARED MEASUREMENT IS SPLIT IN TWO rather than called whole. The extraction
    // main landed is right -- one producer of the drift fact, shared by the drift gate, the
    // behavioural receipt and this path -- but it fused emit with adjudication, and a fused
    // measurement can only refuse before anything is written. `emit_generated_surface` and
    // `adjudicate_generated_surface` are the two halves; `measure_generated_surface` is still
    // their composition and still the single entry for every caller that wants the whole answer,
    // so nothing gained a second producer. This path is the one caller that needs to act BETWEEN
    // them.
    //
    // Writing first is not a relaxation. The gate refuses exactly the same populations it refused
    // before, with the same typed causes; what changes is that the emitter's product survives the
    // refusal, because it is emit's output and not a reward for agreeing with the committed tree.
    // Authority for the ordering: `v2.workflow.required_regen` `required_regen_run`, whose verdict
    // arms cannot be spelled without the tree they judged.
    let (emitted, emitted_basenames) = match emit_generated_surface(&sources)? {
        // EMIT PRODUCED NOTHING IS NOT A VERDICT ABOUT A TREE. Writing a receipt here would name a
        // `candidate_artifact` no pass had written, which is the impersonation the receipt split
        // above exists to end, one field over. `CandidateTreeUnproduced` in
        // `v2.workflow.required_regen` is the modeled arm and it carries no tree; the host
        // spelling of an outcome with no tree and no verdict is a refusal of the run itself.
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
    write_emitted_tree(&formatter, &fresh_src, &emitted)?;
    copy_hand_maintained_support(&stage0_src, &fresh_src)?;
    v1_rt::trace_mark("regen.mirror_write.done".to_string());
    // Verified against what EMIT produced, not against what is committed. Those two populations
    // are equal on a clean tree and differ in precisely the case this ordering exists to serve, so
    // checking the committed population here would re-impose the refusal one line after the write
    // and fail the producer on the run that needs it. The invariant a producer owes is that its
    // own product landed whole.
    v1_rt::trace_mark("regen.candidate_verify.begin".to_string());
    verify_candidate_tree(&fresh_src, &emitted_basenames)?;
    v1_rt::trace_mark("regen.candidate_verify.done".to_string());

    v1_rt::trace_mark("regen.adjudicate.begin".to_string());
    let adjudicated =
        adjudicate_generated_surface(&formatter, &stage0_src, &emitted, &emitted_basenames)?;
    v1_rt::trace_mark("regen.adjudicate.done".to_string());
    let (committed_basenames, sync) = match adjudicated {
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
        GeneratedSurfaceAdjudicated::Measured { committed, sync } => (committed, sync),
    };

    v1_rt::trace_mark("regen.hand_verify.begin".to_string());
    let hand = verify_hand_maintained(&formatter, &emitted, &stage0_src, &candidate_dir)?;
    v1_rt::trace_mark("regen.hand_verify.done".to_string());

    v1_rt::trace_mark("regen.digest.begin".to_string());
    let committed_digest =
        tree_digest_for_basenames(&formatter, &stage0_src, &committed_basenames, "committed")?;
    let candidate_digest = tree_digest_from_map(&formatter, &emitted, &committed_basenames)?;
    v1_rt::trace_mark("regen.digest.done".to_string());

    let first_generation_equal = sync.matches && hand.unverifiable.is_empty();
    let changed_paths = sync.drifted_paths.clone();

    // Every field here was measured by THIS pass against THIS tree. The old shape also carried
    // `fixed_point_equal: false`, which was not a measurement at all -- the first pass never asks
    // that question, so a literal `false` asserted a negative answer where the honest content was
    // "not asked". The variant has no such field, so the placeholder is now unwritable.
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

    eprintln!(
        "required-regen: elapsed_ms={} first_generation_equal={} planned={} executed={} \
         declared_divergent={} [{}]",
        run_started.elapsed().as_millis(),
        first_generation_equal,
        committed_basenames.len(),
        emitted_basenames.len(),
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
/// TWO SOURCES FOR ONE FACT, RECONCILED BY REFUSAL RATHER THAN BY PRECEDENCE. The receipt file
/// is read unconditionally — the cross-tree refusal and the `PriorReceiptRef` are provenance
/// facts only the file carries — so whenever a caller ALSO supplies the digest in memory it
/// exists twice. The previous form was `pass1_digest.unwrap_or(prior)`, which silently preferred
/// the argument: two representations of one fact with a precedence rule, so a disagreement
/// decided nothing and reported nothing (DESIGN §3).
///
/// WHO MADE IT REACHABLE, stated because it changes whose defect this is: until the phases
/// shared a process every caller passed `None`, so the file was the only source. The composed
/// `--required-ci` run is what supplies the argument, so the change that creates the second
/// source is the change that closes it.
///
/// AND WHAT IT IS *NOT*: this does not guard an active defect on the composed path. There,
/// `run_required_regen` writes the receipt and returns the same digest in one pass, so the two
/// agree by construction and this arm is unreachable. It guards the FUNCTION's contract, for a
/// caller that supplies a digest against a receipt written by some other run at this commit —
/// a rebuild between passes, a mutated `target/`, or a first pass that refused after writing.
/// Extracted from the call site precisely so that claim can be tested without running a
/// seven-minute emit to reach it.
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
    // under `target/`, and nothing requires the first to have run in this process, at this commit,
    // or at all. Without this arm a developer iterating on the determinism half alone -- the
    // ordinary thing to do -- over a `target/` warm from an earlier commit produces a receipt
    // stamped with TODAY's `commit_sha` carrying YESTERDAY's `changed_paths` and
    // `first_generation_equal`. In CI the arm is currently unreachable because actions/checkout's
    // default clean removes the ignored `target/` each run (measured: two consecutive main runs
    // each compiled 105 crates starting at proc-macro2, where a warm tree compiles zero) -- but
    // that is a property of a checkout default nobody declared, one cache-reuse change from live
    // on a required path.
    //
    // It refuses rather than recomputing: silently re-running the first pass here would fuse the
    // two authorities, and silently proceeding is the fabrication. The `PriorReceiptRef` shape
    // makes the impersonation unwritable; this makes referencing the WRONG tree loud.
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
    let emitted = compile_stage0(&sources)?;
    let committed_basenames = committed_generated_basenames(&workspace.join("src/v1/stage0/src"))?;
    if emitted.is_empty() {
        return Err("refusal: fixed-point emit produced zero files".to_string());
    }
    let emitted_basenames = generated_basenames_from_emit(&emitted);
    let hand_dir_shadows = hand_maintained_dir_shadows(&workspace.join("src/v1/stage0/src"))?;
    if let Some(reason) =
        validate_compared_populations(&committed_basenames, &emitted_basenames, &hand_dir_shadows)
    {
        return Err(reason);
    }
    let pass2 = tree_digest_from_map(&formatter, &emitted, &committed_basenames)?;
    let fixed_point_equal = pass1 == pass2;

    // `commit_sha` is what THIS pass ran against; `prior` names the tree its referenced evidence
    // came from. They are checked for equality above and a mismatch refuses, so a receipt reaching
    // this point never quotes another tree -- but the field is carried regardless, because a
    // reference whose subject is only guaranteed by an upstream check is one refactor away from
    // being a reference that does not name its subject.
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
/// This exists because an earlier revision of this file had two producers of a single fact —
/// which mirrors drifted. `measure_generated_drift` re-typed the same five calls that
/// `run_required_regen` performs, and nothing kept the copies in step. The receipt is on the
/// record: #8618 repaired a defect INSIDE `compare_generated_surfaces` (the committed side was
/// being normalized, so the comparison was `normalize(normalize(x))` against `normalize(x)` — a
/// false-positive drift with no reachable green). A repair landing in one of two copies of this
/// sequence leaves the other answering the old way, and the copies agreeing on the day they are
/// written is exactly what makes the duplication easy to leave in place.
///
/// The two callers genuinely differ, but they differ in their FAILURE POLICY, not in the
/// measurement: `run_required_regen` routes a refusal to `regen_refusal_outcome`, which writes a
/// receipt and returns `Ok` carrying failures, while the drift gate wants `Err`. So the refusal
/// is returned as a value and each caller applies its own policy — one `match` at the call site
/// rather than a second copy of the five calls above it.
enum GeneratedSurfaceMeasured {
    /// The comparison was taken. `emitted` and `committed` are returned because the regen path
    /// needs them for the candidate tree and its digests, and recomputing them would mean running
    /// the whole emit twice.
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
/// The half of the measurement that exists on its own because one caller must act between the two:
/// `run_required_regen` writes the candidate tree here, so that a later adjudication refusal
/// leaves the author holding the mirror it tells them to commit instead of destroying it.
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
        committed: Vec<String>,
        sync: SyncReport,
    },
    /// Ignorance, never "no drift" -- same refusal semantics as `GeneratedSurfaceMeasured`.
    Refused { reason: String },
}

fn emit_generated_surface(sources: &[(String, String)]) -> Result<GeneratedSurfaceEmit, String> {
    let emitted = compile_stage0(sources)?;
    if emitted.is_empty() {
        return Ok(GeneratedSurfaceEmit::EmitRefused {
            reason: "refusal: emit produced zero files".to_string(),
        });
    }
    let emitted_basenames = generated_basenames_from_emit(&emitted);
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
) -> Result<GeneratedSurfaceAdjudicated, String> {
    let committed = committed_generated_basenames(stage0_src)?;
    let hand_dir_shadows = hand_maintained_dir_shadows(stage0_src)?;
    if let Some(reason) =
        validate_compared_populations(&committed, emitted_basenames, &hand_dir_shadows)
    {
        return Ok(GeneratedSurfaceAdjudicated::Refused { reason });
    }
    let sync = compare_generated_surfaces(formatter, stage0_src, emitted, &committed)?;
    Ok(GeneratedSurfaceAdjudicated::Measured { committed, sync })
}

/// THE COMPOSITION, and still the single entry for every caller that wants the whole answer.
///
/// Splitting the two halves above did not create a second producer of anything: emit happens in
/// exactly one place, adjudication in exactly one place, and this function is their sequence. The
/// drift gate and the behavioural receipt call it unchanged; only the regen path, which has to
/// write the candidate BETWEEN them, reaches for the halves.
fn measure_generated_surface(
    formatter: &ResolvedFormatter,
    sources: &[(String, String)],
    stage0_src: &Path,
) -> Result<GeneratedSurfaceMeasured, String> {
    let (emitted, emitted_basenames) = match emit_generated_surface(sources)? {
        GeneratedSurfaceEmit::EmitRefused { reason } => {
            return Ok(GeneratedSurfaceMeasured::Refused { reason })
        }
        GeneratedSurfaceEmit::Emitted {
            emitted,
            emitted_basenames,
        } => (emitted, emitted_basenames),
    };
    match adjudicate_generated_surface(formatter, stage0_src, &emitted, &emitted_basenames)? {
        GeneratedSurfaceAdjudicated::Refused { reason } => {
            Ok(GeneratedSurfaceMeasured::Refused { reason })
        }
        GeneratedSurfaceAdjudicated::Measured { committed, sync } => {
            Ok(GeneratedSurfaceMeasured::Measured {
                emitted,
                committed,
                emitted_basenames,
                sync,
            })
        }
    }
}

/// The emitted generated surface, keyed by basename.
///
/// Routed through the SAME `measure_generated_surface` the drift gate and the regen path use, so
/// the bytes a behavioural receipt compiles are the bytes the drift gate compared. A second emit
/// here would be a second producer of the candidate itself -- the one fact a receipt absolutely
/// cannot afford to have two of.
pub fn emitted_generated_sources() -> Result<HashMap<String, String>, String> {
    let workspace = workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");
    // A THIRD ADMISSION SITE, AND THE PARAMETER IS WHAT FOUND IT. This entry serves the
    // behavioural receipt rather than the regen phase, and reading the code for "who spawns
    // rustfmt" would not have named it -- it is three calls above the spawn. Threading the
    // resolved formatter as an argument made the omission a compile error instead of a fourth
    // way to discover an absent formatter fifty minutes in.
    let formatter = ResolvedFormatter::admit()?.with_normalize_cache(&workspace)?;
    let sources = super::regen_input_sources(&workspace)?;
    let emitted = match measure_generated_surface(&formatter, &sources, &stage0_src)? {
        GeneratedSurfaceMeasured::Refused { reason } => return Err(reason),
        GeneratedSurfaceMeasured::Measured { emitted, .. } => emitted,
    };
    // KEYED BY BASENAME, and the conversion happens HERE rather than at the call site.
    //
    // Emit keys carry a `src/` prefix; everything that joins against a committed mirror keys on
    // `file_name()`. `generated_basenames_from_emit` already carries the warning that comparing
    // the two key spaces "made every file mismatch in both directions" -- and a caller of this
    // function walked straight into it anyway, looking up `std_pareto.rs` in a map keyed by emit
    // path and getting nothing. It refused rather than reporting equivalence, which is the design
    // working, but the refusal was about the key space rather than about the candidate.
    //
    // Returning the raw map invites that mistake from every future caller. Doing the derivation
    // once, through the same `emit_path_basename` the population census uses, removes it.
    let mut out: HashMap<String, String> = HashMap::new();
    for (path, content) in emitted {
        if !path.ends_with(".rs") || is_hand_maintained_path(&path) {
            continue;
        }
        let base = emit_path_basename(&path).to_string();
        // A collision would silently drop one candidate and compare the wrong bytes. The flat
        // generated surface makes basenames unique, so a duplicate means that assumption has
        // stopped holding, and a receipt built on a stale assumption is worse than no receipt.
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
    /// drop holding as declared. Reported and counted, never a failure -- that is what "declared"
    /// buys, and the count is the whole point: a suppression nobody counts has a frequency of zero
    /// by construction and can never rank for repair.
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
/// itself, so every regen walked and read the corpus twice -- once for the authority digest
/// and once here -- and the receipt priced the second walk at a quarter of the first phase
/// (`regen.corpus_reload`, measured 2026-08-30: 24 s on srv1, 17 s on BuildBuddy). The
/// sources are one fact; the caller reads them once and both consumers take that one value.
fn compile_stage0(sources: &[(String, String)]) -> Result<HashMap<String, String>, String> {
    let source_files: Vec<Rc<SourceFile>> = sources
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect();
    let result = compile_sources(Rc::new(source_files.into()), RenderTarget::Rust);
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
// about two different populations -- the emit and the committed tree -- and each carried its own
// copy of the membership rule. That is the fork this whole lane keeps finding: two readers of one
// fact, which agree until they do not, and here they sat inside the very file whose job is to
// detect that class. They now share this predicate, so a file cannot be compared on one side and
// skipped on the other.
//
// THE EMITTED-POPULATION MANIFEST NEEDS NO EXCEPTION HERE, AND THAT IS BY CONSTRUCTION RATHER
// THAN BY LUCK. `emitted_population.rs` (v1.compiler.emit_rust, `emit_emitted_population_manifest`)
// is the emitter's declaration of what it produced, and it is emitted as a `.rs` of comment lines
// precisely so that this rule already admits it: it enters the compared population, the drift
// comparison, the tree digest and candidate verification with nothing named and nothing added.
// An earlier revision made it a `.txt` and named it in this predicate as an exception. That was
// refused by execution, not by review -- every compared member is rustfmt-normalized before its
// digest is taken, so the `.txt` failed to parse as Rust ("expected one of `!` or `::`, found
// `.`"). The refusal was the right one: the compared population is Rust-shaped end to end, and an
// artifact that wants its gating has to be Rust.
fn is_compared_generated_basename(basename: &str) -> bool {
    basename.ends_with(".rs")
}

fn generated_basenames_from_emit(emitted: &HashMap<String, String>) -> Vec<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in emitted.keys() {
        if is_compared_generated_basename(emit_path_basename(path))
            && !is_hand_maintained_path(path)
        {
            // Basename, not the emit key: `committed_generated_basenames` keys on
            // `file_name()`, and emit keys carry a `src/` prefix. Comparing the two
            // key spaces made every file mismatch in both directions.
            names.insert(emit_path_basename(path).to_string());
        }
    }
    names.into_iter().collect()
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
/// Both compared populations key on basename, and the committed walk enumerates only the top level
/// of the stage0 crate, so a file living inside a hand-maintained DIRECTORY is invisible to it. An
/// emitted basename equal to one of those files therefore lands in `emitted_not_committed`, where
/// the two remedies on offer -- install the produced mirror, or investigate an emitter that
/// invented a surface -- BOTH destroy hand-authored code: installing overwrites it with generated
/// bytes, deleting removes it. This map lets that population be named under its own cause with the
/// remedy that actually applies (de-collide the module name).
///
/// IT DOES NOT EXCLUDE, AND THAT IS THE WHOLE DESIGN. Teaching `is_hand_maintained_path` about the
/// directory list would drop the colliding path out of the compared population entirely, which
/// silently stops comparing a genuinely generated surface -- a wrong refusal traded for no
/// refusal. Same failure as making the committed walk descend, entered from the other side.
///
/// ZERO EXPOSURE, stated so this is not read as repairing a live defect. Emitted Rust filenames are
/// flat by construction (`v1.compiler.emit_core_support` `module_to_filename` is split(".") joined
/// with "_" under `rust_source_root()`), so the emitter cannot write into a hand-maintained
/// directory; the collision is reachable only by authoring a module whose bare name equals one of
/// those files. Measured 2026-08-22: zero collisions in the corpus, required-regen green on main
/// at 90986d19469, and no upstream guard was found that would refuse such a module name (the
/// nearest candidates, `gunbc.stage0_rust_source_lifecycle_scaffold`
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
    // NEITHER IS ADMITTED, and the first one is where that matters. "An author introduced a module"
    // and "the emitter invented a surface nobody authored" produce the SAME population, and the
    // second is what this check exists to catch, so no arm computed from the populations can tell
    // them apart -- admitting the first would be this same conflation pointing the other way. The
    // refusal therefore names the fork and leaves the decision with the author; what changed is
    // that the install branch is now actionable, because the ordering above wrote the bytes before
    // this check ran instead of discarding them.
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
        // `v2.workflow.required_regen` `MirrorMissingShadowsHandMaintainedSource`. The refusal is
        // not softer than the one above -- the line still stops -- it is the same stop pointed at
        // the move that actually resolves the state.
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
    // A population refusal happens BEFORE any content comparison, so there is nothing here that a
    // digest or an equality could be ABOUT. The previous shape wrote a `FirstGeneration` receipt
    // and filled the three unmeasured positions with `refused:population`, `refused:population`
    // and `false`. `Refused` has none of those three fields, so the placeholders are unwritable
    // rather than merely discouraged, and the receipt carries the refusal's actual content --
    // its cause. See the variant's own comment for the route the fabricated Bool took off this
    // machine.
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
        // It previously normalized the committed side too, which made the comparison
        // `normalize(normalize(emitted))` vs `normalize(emitted)` once a candidate had been
        // installed. That is only an identity if rustfmt is idempotent, and it is not:
        // measured 2026-08-20, `v1_compiler_infer.rs` reformats on a second pass (a
        // `let ... = if (long_receiver_chain)` splits differently), so the fold reported the
        // same single file as drifted at generation 2, 3 and 4 with the candidate on disk
        // BYTE-IDENTICAL to the committed file it was compared against. No number of
        // generations could clear it: the check had no reachable green, and the only way to
        // silence it was to hand-edit the mirror -- validation standing where construction was
        // available (DESIGN 5). Comparing against the written artifact makes "install the
        // candidate" a guaranteed remedy by construction, and makes the two derivations of the
        // candidate one fact rather than two (DESIGN 3).
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
            // Not in the emitted population at all. For 35 of the 36 entries (measured by
            // execution 2026-08-21) this is the ordinary case and there is nothing to compare:
            // the emitter does not produce the file, so excluding it from the comparison costs
            // nothing. For a DECLARED row it is a defect in the declaration, not in the tree.
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
                        // THE MEASUREMENT ABOVE USED TO BE DISCARDED HERE. The divergent branch was
                        // an empty block under a comment saying drift is expected on a clean tree,
                        // which is the absorbing fallback (DESIGN section 5) in its authoring form:
                        // the comparison ran, found a real divergence between the authority and the
                        // committed artifact, and produced no typed, located, countable output -- so
                        // the deficit's frequency was zero by construction and it could never rank for
                        // repair. Membership in HAND_MAINTAINED_STAGE0_FILES was doing the silencing
                        // while claiming only to describe what the emitter does not produce.
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

fn write_emitted_tree(
    formatter: &ResolvedFormatter,
    dest_src: &Path,
    emitted: &HashMap<String, String>,
) -> Result<(), String> {
    if dest_src.exists() {
        fs::remove_dir_all(dest_src).map_err(|e| format!("remove {}: {e}", dest_src.display()))?;
    }
    fs::create_dir_all(dest_src).map_err(|e| format!("create {}: {e}", dest_src.display()))?;
    for (path, content) in emitted {
        let out_path = dest_src.join(emit_path_basename(path));
        // Only `.rs` surfaces are the generated-Rust population this comparator reasons
        // about (see committed_generated_basenames / generated_basenames_from_emit); a
        // non-Rust emitted artifact (e.g. Cargo.toml from the crate-layout emit) is not
        // rustfmt-normalizable and is written through verbatim.
        let normalized = if emit_path_basename(path).ends_with(".rs") {
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
/// `let x = if (long.receiver.chain)` re-splits on a second pass. A single pass therefore puts
/// this repository's two gates in direct contradiction on such a file, because they consume
/// different passes of the same formatter:
///
///   * `cargo fmt --all --check` (pre-commit, and the fmt gate) demands pass N+1 of whatever is
///     committed -- it re-formats the file in place;
///   * `write_emitted_tree` wrote pass 1 of the emitted bytes, and `compare_generated_surfaces`
///     compares against exactly those bytes.
///
/// Satisfying either one broke the other, in a loop with no exit: install the candidate and fmt
/// rewrites it; run fmt and regen reports drift. The only state satisfying both simultaneously is
/// a FIXED POINT of rustfmt, so that is what the emitted artifact must be -- then `cargo fmt` is a
/// no-op on it by definition, and byte-comparing the committed file against it is exact.
///
/// This is construction rather than validation (DESIGN 5): the disagreement is not detected and
/// reported, it is made unrepresentable, because the artifact is written in the one form both
/// consumers agree on. Iterating here rather than teaching the comparator to tolerate a second
/// pass is deliberate -- tolerance would have to be granted to the fmt gate too, and a tolerance
/// shared by two gates is a hole in both.
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
/// Both normalize paths used to spawn a BARE `rustfmt`, so the program was resolved from
/// ambient PATH at the moment of use -- which in a required run is 45-50 minutes deep. A run on
/// 2026-08-24 (run 32693719649, srv2-05) failed there with `spawn rustfmt: No such file or
/// directory` after the job's own toolchain probe had printed `/home/ghrunner/.cargo/bin/rustfmt`
/// 48 minutes earlier in the same shell environment. The cost of that shape is not the ENOENT, it
/// is WHERE it lands: a question about PATH became a fifty-minute-deep failure in a phase that
/// has nothing to do with PATH.
///
/// WHAT THIS FIXES AND WHAT IT DOES NOT, stated because the distinction is the whole honesty of
/// the row. It fixes the class where the formatter is ABSENT AT ADMISSION: that now refuses
/// immediately, naming the PATH searched, instead of running most of an hour first. It does NOT
/// prevent a formatter that is present at admission and gone at spawn -- resolving a path is not
/// holding a file open, and nothing here can make a concurrent process stop replacing a shim.
/// What it does for that case is make the failure ATTRIBUTABLE: the spawn error names the exact
/// program resolved at admission and states that it existed then, which turns an ambiguous ENOENT
/// into direct evidence that the environment mutated under the run -- the question the workflow's
/// toolchain probe is currently collecting baselines for, and which no run has yet answered.
///
/// AUTHORITY, AND THE RUNG THIS SITS AT. The refusal this type produces is named by the carrier
/// as `v2.workflow.required_regen` `RequiredRegenRefusal`'s `FormatterUnavailable`, added in the
/// same change. It is modeled there rather than only here because this host is a MIRROR of that
/// carrier -- DESIGN.md records `run_required_regen`'s hand-written ordering at *mitigatable*,
/// with derivation from the carrier as its next-rung trigger -- and a refusal the host can
/// produce that the carrier cannot name is precisely the divergence that trigger exists to
/// close. WHAT IS NOT CLAIMED: this host reports refusals as `String`, as every other refusal on
/// this path does, so the correspondence is held by review and by this citation, NOT by the type
/// system; introducing a typed carrier for this one refusal alone would be a second
/// representation of a vocabulary the carrier already owns. The class therefore stays at
/// *mitigatable* and inherits the host's existing next-rung trigger rather than adding a new one.
///
/// NO FALLBACK ARM EXISTS AND NONE MAY BE ADDED. There is no retry, no tolerated absence, no
/// "skip normalization and compare raw" path. The emitted artifact must be a fixed point of the
/// formatter or the comparison is meaningless (see `normalize_generated_source`), so a run
/// without a formatter has nothing to say and must stop. The line still stops; it stops at
/// admission rather than at minute fifty.
#[derive(Debug, Clone)]
pub struct ResolvedFormatter {
    program: PathBuf,
    /// The PATH the program was found on, kept so a later failure can say what was searched
    /// rather than making the reader reconstruct it from the environment they are not in.
    searched: String,
    /// NORMALIZE ONCE PER DISTINCT INPUT. Every regen pass normalized the whole emitted
    /// population THREE times -- `write_emitted_tree`, `compare_generated_surfaces`,
    /// `tree_digest_from_map` each called `normalize_generated_source` on the same bytes --
    /// and each normalize is at least two rustfmt spawns (the fixed-point seek). Measured
    /// 2026-08-30 by `--regen-round-cost`: mirror_write + adjudicate + digest = 67 s on the
    /// BuildBuddy runner, 97 s on srv1, for one population. The memo is keyed by the RAW
    /// input bytes, so a hit is exact by construction; clones share it so the three
    /// consumers see one table.
    memo: Rc<std::cell::RefCell<HashMap<String, String>>>,
    /// The on-disk half, keyed by rustfmt's own version string, so a fixed-point run on a tree
    /// whose emit did not change pays ZERO rustfmt spawns. Each entry stores the raw input
    /// beside the normalized output and is honoured only when the stored raw is byte-equal to
    /// the request -- a digest-named file is a LOCATION, never the identity, so a hash
    /// collision cannot serve another input's formatting (DESIGN section 5).
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

    /// PATH RESOLUTION REQUIRES THE EXECUTE BIT, not merely a file. The OS skips a
    /// non-executable entry and keeps searching, so a resolver keyed on `is_file` would ADMIT a
    /// file the kernel would have stepped over -- shadowing a real formatter further down PATH
    /// and failing where a bare `Command::new("rustfmt")` would have succeeded. Matching the
    /// kernel's own predicate is what keeps this resolver a faithful model of the lookup it
    /// replaces rather than a second, more permissive one.
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

    /// EXECUTING THE PROGRAM ONCE IS THE ADMISSION, because the metadata above is a proxy and
    /// this is the fact itself. A mode bit does not establish that the file runs: a broken shim,
    /// a wrong-architecture binary, or a dangling interpreter line all carry the bit and all die
    /// at the first real normalize -- which is the ~50-minute failure this row exists to move.
    /// `--version` is chosen because it is total, cheap and reads nothing from the tree.
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

    /// The spawn-failure message, which is the attributable half described on the type. It says
    /// the program EXISTED at admission, because that is the fact a reader cannot recover later
    /// and the one that discriminates "never installed" from "replaced under the run".
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
/// This was a separate `RegenReceiptStored` struct mirroring the carrier's field list -- a second
/// representation of one fact (DESIGN 3), and the place the false fail-closed claim hid: it
/// silently accepted any JSON containing its fields, so it read a v1 record as happily as a v2
/// one. It is gone. `read_receipt` now deserializes the REAL carrier and destructures it, so the
/// reader cannot drift from the writer -- there is only one shape.
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
/// The third was already true by construction (a `fixed_point` record lacks four required fields),
/// but it is stated as an explicit arm rather than left to a missing-field parse error, because a
/// parse error would report the symptom -- a missing field name -- instead of the cause.
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
        // refusal left a `FirstGeneration` receipt on disk and this arm read it as an ordinary
        // measurement: `first_generation_equal: false` became a `PriorMeasurement`, then a
        // `PriorReceiptRef`, then `referenced_first_generation_equal=false` on the operator's
        // terminal -- a comparison result reported for a comparison that never ran. The
        // cross-tree guard above cannot catch it, because the refusal happened AT this commit,
        // which is exactly the condition that guard admits. Now the variant carries no such
        // field, and this arm refuses with the ORIGINAL cause rather than a derived one: what
        // the operator has to fix is the refusal, not the fixed point.
        RegenReceipt::Refused { reason, .. } => Err(format!(
            "refusal: the first-generation pass at this commit REFUSED ({reason}) — there is no \
             first-generation measurement for the fixed-point pass to reference. Close that \
             refusal and re-run `claim_executor --required-regen` before asking for the fixed \
             point. Receipt: {}",
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

    /// BOTH REFUSALS ARE THE PRODUCT OF THIS ROW, so their TEXT is under test, not just their
    /// substrings. A multi-line non-raw literal silently absorbs the indentation of its
    /// continuation lines, and `cargo fmt` reflows such literals on its own -- so a message can
    /// go garbled without anyone editing it, and every `contains` assertion still passes because
    /// the runs of spaces sit BETWEEN the fragments being matched. This guard is what makes that
    /// regression loud; it went red on the form that shipped in this PR's first revision.
    fn assert_message_is_not_reflowed(message: &str) {
        assert!(
            !message.contains("  "),
            "a diagnostic must not carry runs of literal whitespace from source indentation \
             -- use `concat!` rather than splitting a literal across lines: {message:?}"
        );
    }

    /// THE REFUSAL NAMES WHAT WAS SEARCHED, AND THE PAIR IS THE ASSERTION.
    ///
    /// RED against the pre-change code: there was no admission at all, so the absent-formatter
    /// case had no return value to assert on -- it surfaced as `spawn rustfmt: No such file or
    /// directory` from whichever normalize happened to run first, ~50 minutes into a required
    /// run. The empty-PATH arm is the discriminating input; the real-PATH arm is the control
    /// without which a resolver that refused everything would also pass.
    ///
    /// `from_path_var` takes the PATH rather than reading the environment precisely so this test
    /// does not mutate a process-global that every other test in this binary shares -- a
    /// `set_var("PATH", "")` here would make unrelated tests fail depending on thread order,
    /// which is the flake class that costs more to diagnose than the check is worth.
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
    /// This is the RED for review 55506's first finding. Against the `is_file`-keyed resolver
    /// this test FAILS: the unusable file is admitted, the real formatter below it is never
    /// reached, and the run then dies at the first normalize -- the exact ~50-minute failure
    /// this row exists to move, reintroduced by the row itself. The kernel skips such an entry
    /// when resolving PATH, so the old resolver was strictly more permissive than the lookup it
    /// replaced.
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

    /// THE SPAWN REFUSAL DISTINGUISHES "NEVER INSTALLED" FROM "REPLACED UNDER THE RUN", which is
    /// the fact the run that motivated this row could not report. It is asserted on the message
    /// rather than by staging a mid-run deletion: nothing here can portably make a live process
    /// lose a file it already resolved, and faking it by pointing at a path that never existed
    /// would assert the message while proving nothing about the case it describes.
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
    // The defect this pins, found in review of gunbc#8647: a population refusal returns `Ok`, and
    // the receipt it left behind held `refused:population` in both digest positions and
    // `first_generation_equal: false` where "not asked" belonged. Two carriers cross out of that
    // function -- the in-process `RequiredRegenOutcome` and the on-disk receipt -- and the typed
    // `FirstGeneration` closed only the first. The receipt is the one that crosses the PROCESS
    // boundary, so the fabricated Bool reached a standalone `--required-regen-fixed-point` run
    // through `read_receipt` and printed as `referenced_first_generation_equal=false`.
    //
    // Both halves are asserted here. The in-memory half is the original test, unchanged in
    // substance: `pass1_digest_for_fixed_point` yields `None` for a refusal, and making it answer
    // from the receipt fails it. The on-disk half is new and is what this pass adds: the refusal
    // variant has no Bool and no digest to read, and `read_receipt` refuses a refused prior with
    // its ORIGINAL cause rather than passing a derived answer along.
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

        // THE HALF THAT USED TO BE THE GAP. The old test asserted the OPPOSITE of this line --
        // that the sentinel "IS still sitting in the receipt ... the wrong answer is right there
        // to be read" -- and called the coproduct load-bearing for keeping the outcome clean
        // while the receipt stayed fabricated. There is now no digest to read and no equality to
        // misread; both are absent from the variant rather than absent by convention.
        assert_eq!(refused.receipt.candidate_generated_digest(), None);
        assert_eq!(refused.receipt.first_generation_equal(), None);
        assert_eq!(
            refused.receipt.refusal_reason(),
            Some("refusal: emit produced zero files")
        );

        // POSITIVE CONTROL: ordinary drift is not a refusal. Pass one emitted, the comparison
        // disagreed, and the fixed point still has a subject -- skipping it there would lose a
        // determinism signal exactly when drift makes it interesting. Note this control now
        // carries a REAL digest and a REAL `false`, which is the distinction the whole change is
        // about: `false` here is an answer, and there is no longer any receipt on which it is not.
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

    // THE ON-DISK HALF, THROUGH THE REAL READER. `read_receipt` is the function that turned the
    // fabricated Bool into a `PriorMeasurement`, so the refusal has to be asserted THROUGH it and
    // not against the variant alone.
    //
    // RED: give `RegenReceipt::Refused` a `first_generation_equal: bool` field and let this arm
    // build a `PriorMeasurement` from it, and this test fails -- which is the whole defect
    // restored. The positive control beside it is what keeps the refusal from being satisfied by
    // a reader that refuses everything.
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
            },
        )
        .expect("write measured receipt");

        let prior = read_receipt(&path).expect("a measured prior reads");
        assert_eq!(prior.candidate_generated_digest, "real-digest");
        assert!(!prior.first_generation_equal);

        let _ = fs::remove_dir_all(&tmp);
    }

    // LOCAL RUST RED CONTROL FOR THE DUAL-INPUT REFUSAL — local, NOT enrolled. The Rust suite
    // has been out of CI since the 2026-07-11 operator ruling, so this executes for whoever runs
    // it and for no gate. Said plainly because the previous heading claimed "ENROLLED", which is
    // the rung inflation DESIGN §4b calls worse than sitting low: an unenrolled control that
    // says it is enrolled never ranks for enrolling.
    //
    // The arm it guards is UNREACHABLE from the
    // composed `--required-ci` path — there `run_required_regen` writes the receipt and returns
    // the same digest in one pass, so the two agree by construction — which is exactly why the
    // decision was extracted from its call site: reaching it through the real function would
    // require a seven-minute emit, and a wall no test can reach is a wall nobody knows works.
    //
    // RED: restoring `pass1_digest.unwrap_or(prior)` makes the disagreement case return Ok and
    // fails the first assertion. The None and agreeing cases are the positive controls, without
    // which a function that refused everything would also pass.
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
// what drifted, rebuild from the installed seed. This driver runs exactly that sequence ONCE,
// through the same `run_required_regen` the required phase runs (never a second emit path),
// with `v1_rt::trace_mark` armed so every phase boundary lands in one ledger, and renders the
// receipt through `gunbc.regen_round_cost` `regen_round_cost_render` via the interpreter — there
// is no Rust copy of the receipt format to drift from the model.
//
// IT MUTATES THE TREE, AND SAYS SO. Installing the drifted mirrors into src/v1/stage0/src is
// the step an author performs by hand today, and the round's rebuild is only a measurement of
// "rebuild from the installed seed" if the seed was actually installed. A read-only variant
// would price a round nobody runs. The receipt names the tree (HEAD sha, dirty flag) and lists
// every installed path by identity, so what the round changed is on the record.
//
// THE SEED THAT EMITS IS THE SEED THAT WAS BUILT, OR THE RUN REFUSES. The emit runs in THIS
// process, whose binary is the product of the seed build only if that build was a no-op on the
// running executable. The executable's bytes are hashed before and after the build; a changed
// hash means the round measured a stale seed's emit, and the only honest answer is to stop and
// say so rather than report a candidate the built seed never produced.
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
}

const REGEN_CONVERGENCE_SCHEMA: &str = "gunbc.regen_convergence_transaction.v1";
const REGEN_CONVERGENCE_BOUND: usize = 8;
const REGEN_CONVERGENCE_JOURNAL_REL: &str = "target/regen-convergence-journal";
const REGEN_CONVERGENCE_RECEIPT_REL: &str = "target/regen-convergence-transaction.json";

#[derive(Debug, Serialize, serde::Deserialize)]
struct RegenConvergenceJournalEntry {
    relative_path: String,
    backup_name: String,
    existed: bool,
    pre_stage_digest: String,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct RegenConvergenceJournal {
    schema: String,
    starting_commit: String,
    entries: Vec<RegenConvergenceJournalEntry>,
}

#[derive(Debug, Serialize)]
struct RegenConvergenceSurfaceReceipt {
    declaring_module: String,
    projected_path: String,
    pre_stage_digest: String,
    candidate_digest: String,
    installed_digest: String,
    planned: bool,
    executed: bool,
    terminal: bool,
    passed: bool,
}

#[derive(Debug, Clone, Serialize)]
enum RegenConvergenceStageKindReceipt {
    PromoteGenerationInputs,
    InstallSeedCompatibilityCut,
    PublishNonSeedOutputs,
}

#[derive(Debug, Serialize)]
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

fn regen_convergence_journal_path(workspace: &Path) -> PathBuf {
    workspace.join(REGEN_CONVERGENCE_JOURNAL_REL)
}

fn restore_regen_convergence_journal(workspace: &Path) -> Result<(), String> {
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
    for entry in &manifest.entries {
        let destination = workspace.join(&entry.relative_path);
        if entry.existed {
            let backup = root.join(&entry.backup_name);
            fs::copy(&backup, &destination).map_err(|e| {
                format!(
                    "rollback/restore refuses: copy {} -> {}: {e}",
                    backup.display(),
                    destination.display()
                )
            })?;
            let restored = path_digest(&destination)?;
            if restored != entry.pre_stage_digest {
                return Err(format!(
                    "rollback/restore refuses: {} restored as {} instead of {}",
                    entry.relative_path, restored, entry.pre_stage_digest
                ));
            }
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

fn journal_stage0_paths(
    workspace: &Path,
    stage0_src: &Path,
    basenames: &[String],
) -> Result<(), String> {
    let root = regen_convergence_journal_path(workspace);
    fs::create_dir_all(&root).map_err(|e| format!("create {}: {e}", root.display()))?;
    let manifest_path = root.join("journal.json");
    let mut journal = if manifest_path.is_file() {
        serde_json::from_slice::<RegenConvergenceJournal>(
            &fs::read(&manifest_path)
                .map_err(|e| format!("read {}: {e}", manifest_path.display()))?,
        )
        .map_err(|e| format!("parse {}: {e}", manifest_path.display()))?
    } else {
        RegenConvergenceJournal {
            schema: REGEN_CONVERGENCE_SCHEMA.to_string(),
            starting_commit: git_head_sha(workspace)?,
            entries: Vec::new(),
        }
    };
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
        let existed = source.is_file();
        let pre_stage_digest = if existed {
            fs::copy(&source, root.join(&backup_name))
                .map_err(|e| format!("snapshot {}: {e}", source.display()))?;
            path_digest(&source)?
        } else {
            "absent".to_string()
        };
        journal.entries.push(RegenConvergenceJournalEntry {
            relative_path,
            backup_name,
            existed,
            pre_stage_digest,
        });
    }
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
    let compiled_crates = stderr
        .lines()
        .filter(|l| l.trim_start().starts_with("Compiling "))
        .count() as u64;
    Ok(CargoBuildObservation { compiled_crates })
}

/// The seed binary ON DISK at the path this process was started from. After a cargo build
/// replaces that file, Linux reports the running image as `<path> (deleted)`; the digest is
/// taken from the path itself so the comparison is "what is installed there now" against "what
/// was installed there before the build", which is the question the refusal asks.
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

/// Install the candidate's drifted mirrors into the committed seed — the manual step of the
/// recipe, performed from the SAME candidate tree the regen just wrote and judged.
fn install_candidate_paths(
    candidate_src: &Path,
    stage0_src: &Path,
    basenames: &[String],
) -> Result<(), String> {
    for basename in basenames {
        let from = candidate_src.join(basename);
        let to = stage0_src.join(basename);
        fs::copy(&from, &to)
            .map_err(|e| format!("install {} -> {}: {e}", from.display(), to.display()))?;
    }
    Ok(())
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

fn read_first_generation_receipt(
    path: &Path,
) -> Result<(String, String, String, Vec<String>, String), String> {
    let receipt: RegenReceipt = serde_json::from_slice(
        &fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", path.display()))?;
    match receipt {
        RegenReceipt::FirstGeneration {
            candidate_generated_digest,
            committed_generated_digest,
            changed_paths,
            candidate_artifact,
            authority_digest,
            ..
        } => Ok((
            candidate_generated_digest,
            committed_generated_digest,
            candidate_artifact,
            changed_paths,
            authority_digest,
        )),
        RegenReceipt::Refused { reason, .. } => Err(format!(
            "regen refused before convergence planning: {reason}"
        )),
        RegenReceipt::FixedPoint { .. } => Err(
            "regen convergence expected a first-generation receipt, found fixed-point".to_string(),
        ),
    }
}

fn run_built_seed_regen(
    workspace: &Path,
    candidate_dir_rel: &str,
    receipt_rel: &str,
    source_roots: &[String],
) -> Result<(String, String, String, Vec<String>, String), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let shown = exe.to_string_lossy().into_owned();
    let on_disk = PathBuf::from(shown.strip_suffix(" (deleted)").unwrap_or(&shown));
    let mut command = Command::new(&on_disk);
    command
        .arg("--required-regen")
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
    let receipt = read_first_generation_receipt(&workspace.join(receipt_rel));
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
    ),
    String,
> {
    let (edges, modules) = regen_module_edges(workspace)?;
    // Ownership includes emitted-not-committed modules. Joining through the committed mirror
    // population would erase exactly the new-module case the convergence transaction must
    // install, so derive the projection from the complete module index instead.
    let basename_to_module = modules
        .iter()
        .map(|module| (format!("{}.rs", module.replace('.', "_")), module.clone()))
        .collect();
    let (generation, bootstrap_sources, bootstrap_products) =
        regen_generation_role_population(source_roots, &edges, &modules)?;
    Ok((
        basename_to_module,
        generation,
        bootstrap_sources,
        bootstrap_products,
    ))
}

fn convergence_plan_from_model(
    source_roots: &[String],
    ordinal: usize,
    generation_id: &str,
    candidate_tree_id: &str,
    candidate_tree_digest: &str,
    drifted: &[String],
    basename_to_module: &HashMap<String, String>,
    generation_modules: &BTreeSet<String>,
    bootstrap_sources: &BTreeSet<String>,
    bootstrap_products: &BTreeSet<String>,
) -> Result<(String, Vec<String>), String> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = source_roots
        .iter()
        .map(|root| Path::new(root).join("workflow/regen_convergence_transaction.dag"))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            "refusal: v2.workflow.regen_convergence_transaction is outside source roots".to_string()
        })?;
    let entry = entry.to_string_lossy().into_owned();
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| format!("refusal: convergence planner did not resolve: {e}"))?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let changed = drifted
        .iter()
        .map(|basename| {
            let module = basename_to_module
                .get(basename)
                .cloned()
                .unwrap_or_else(|| format!("v1.compiler.emit_rust.product.{basename}"));
            let role = if bootstrap_products.contains(basename) {
                "BootstrapSourceMirror"
            } else if generation_modules.contains(&module) {
                "GenerationInput"
            } else if bootstrap_sources.contains(&module) {
                "BootstrapSourceMirror"
            } else {
                "GenerationSubject"
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
                        str_value(format!("bound-by-host:{basename}")),
                    ),
                ]),
            };
            Value::Record {
                type_name: ctx.sym("RegenChangedSurface"),
                fields: Rc::new(vec![
                    (ctx.sym("identity"), identity),
                    (
                        ctx.sym("generation_role"),
                        Value::Variant {
                            type_name: ctx.sym("RegenGenerationRole"),
                            variant_name: ctx.sym(role),
                            fields: Rc::new(vec![]),
                        },
                    ),
                    (
                        ctx.sym("seed_membership"),
                        Value::Variant {
                            type_name: ctx.sym("RegenSeedMembership"),
                            variant_name: ctx.sym("SeedEmbedded"),
                            fields: Rc::new(vec![]),
                        },
                    ),
                    (
                        ctx.sym("dependency_closure_id"),
                        str_value("complete-seed-build"),
                    ),
                    (ctx.sym("pre_stage_digest"), str_value("observed-by-host")),
                    (ctx.sym("candidate"), candidate),
                ]),
            }
        })
        .collect::<Vec<_>>();
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
        (
            Some("changed".to_string()),
            Value::List(Rc::new(changed.into())),
        ),
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
        Value::Str(value) => value.to_string(),
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
    if kind == "Refused" || kind == "Terminal" {
        return Err(format!(
            "convergence planner returned {kind} for non-empty drift {drifted:?}"
        ));
    }
    Ok((kind, paths))
}

fn install_convergence_stage(
    workspace: &Path,
    stage0_src: &Path,
    candidate_src: &Path,
    basenames: &[String],
    basename_to_module: &HashMap<String, String>,
    ordinal: usize,
    kind: RegenConvergenceStageKindReceipt,
    seed_before: &str,
    generation_id: &str,
    candidate_tree_id: &str,
    candidate_tree_digest: &str,
    authority_digest: &str,
) -> Result<RegenConvergenceStageReceipt, String> {
    journal_stage0_paths(workspace, stage0_src, basenames)?;
    let mut surfaces = Vec::new();
    for basename in basenames {
        let destination = stage0_src.join(basename);
        let candidate = candidate_src.join(basename);
        let pre = if destination.is_file() {
            path_digest(&destination)?
        } else {
            "absent".to_string()
        };
        let candidate_digest = path_digest(&candidate)?;
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
            declaring_module: basename_to_module
                .get(basename)
                .cloned()
                .unwrap_or_else(|| format!("bootstrap-product:{basename}")),
            projected_path: format!("src/v1/stage0/src/{basename}"),
            pre_stage_digest: pre,
            candidate_digest,
            installed_digest: installed,
            planned: true,
            executed: true,
            terminal: false,
            passed: false,
        });
    }
    let build = seed_cargo_build(workspace, "round.rebuild_from_installed")?;
    let _ = build.compiled_crates;
    let seed_after = current_exe_digest()?;
    for surface in &mut surfaces {
        surface.terminal = true;
        surface.passed = true;
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
        dependency_closure_id: format!("authority:{authority_digest}"),
        build_target: "claim_executor".to_string(),
        build_invocation: "cargo build --release --bin claim_executor".to_string(),
        build_terminal: RegenConvergenceBuildTerminalReceipt::Passed,
        output_seed_digest: seed_after,
        next_generation_receipt_id: format!("generation-{}", ordinal + 1),
    })
}

pub fn run_regen_round_cost(
    candidate_dir_rel: &str,
    receipt_rel: &str,
    source_roots: &[String],
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

    let (basename_to_module, generation_modules, bootstrap_sources, bootstrap_products) =
        convergence_surface_roles(&workspace, source_roots)?;
    let regen = run_required_regen(candidate_dir_rel, receipt_rel)?;
    let (
        mut candidate_digest,
        starting_surface_digest,
        mut candidate_tree_id,
        mut drifted,
        mut authority_digest,
    ) = read_first_generation_receipt(&workspace.join(receipt_rel))?;
    let initial_seed_digest = current_exe_digest()?;
    let mut current_seed_digest = initial_seed_digest.clone();
    let mut stages = Vec::new();
    let mut seen_states = BTreeSet::new();
    let mut generation_ordinal = 0usize;
    let mut round_failures = Vec::new();
    if matches!(regen.receipt, RegenReceipt::Refused { .. }) {
        round_failures.extend(
            regen
                .failures
                .iter()
                .map(|failure| format!("regen: {failure}")),
        );
    }

    while !drifted.is_empty() {
        if generation_ordinal >= REGEN_CONVERGENCE_BOUND {
            let failure = format!("generation bound exceeded: {}", REGEN_CONVERGENCE_BOUND);
            restore_regen_convergence_journal(&workspace)?;
            return Err(failure);
        }
        let state = format!("{current_seed_digest}:{candidate_digest}");
        if !seen_states.insert(state.clone()) {
            restore_regen_convergence_journal(&workspace)?;
            return Err(format!("generation cycle detected at {state}"));
        }
        let (kind, install_set) = convergence_plan_from_model(
            source_roots,
            generation_ordinal + 1,
            &format!("generation-{generation_ordinal}"),
            &candidate_tree_id,
            &candidate_digest,
            &drifted,
            &basename_to_module,
            &generation_modules,
            &bootstrap_sources,
            &bootstrap_products,
        )?;
        let install_population: BTreeSet<String> = install_set.iter().cloned().collect();
        let deferred = drifted
            .iter()
            .filter(|path| !install_population.contains(*path))
            .cloned()
            .collect::<Vec<_>>();
        if install_set.is_empty() {
            restore_regen_convergence_journal(&workspace)?;
            return Err(format!(
                "generation made no progress with drift {drifted:?}"
            ));
        }
        v1_rt::trace_mark("round.install.begin".to_string());
        let stage_result = install_convergence_stage(
            &workspace,
            &stage0_src,
            &candidate_src,
            &install_set,
            &basename_to_module,
            generation_ordinal + 1,
            &kind,
            &current_seed_digest,
            &format!("generation-{generation_ordinal}"),
            &candidate_tree_id,
            &candidate_digest,
            &authority_digest,
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
        let next = run_built_seed_regen(&workspace, candidate_dir_rel, receipt_rel, source_roots);
        match next {
            Ok((next_candidate, _committed, next_tree, next_drifted, next_authority)) => {
                candidate_digest = next_candidate;
                candidate_tree_id = next_tree;
                drifted = next_drifted;
                authority_digest = next_authority;
            }
            Err(failure) => {
                restore_regen_convergence_journal(&workspace)?;
                return Err(format!(
                    "next generation refused; checkpoint restored: {failure}"
                ));
            }
        }
    }

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
        fixed_point_verdict: "RegenFixedPointReached".to_string(),
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

    let rebuild_compiled_crates = 0;
    let convergence_stage_receipt_ids = ordered_stage_receipt_ids;

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
             regen-round-cost: convergence_stages=1 [stage-1]\n"
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

// ===========================================================================================
// THE AFFECTED SET OF ONE EDIT -- host realization of `gunbc.regen_affected_set`.
//
// The host does three things the model cannot: read the edit (the floor's git diff range), read
// the tree (module names from the edited files, the seed's closure edge index, the committed
// mirror population), and take the reverse walk over the full edge index at native cost. The
// VERDICT -- which arm, which members -- is the model's: the host hands `regen_affected_set` the
// edited modules, the unlocatable paths, the edges among the modules its own walk reached, the
// compared rows, and the declared bootstrap rows, and prints what the model answers. The host's
// walk is then held to the model's answer on every run (`lockstep` below): a disagreement is a
// refusal, never a silently preferred side.
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
/// as a Rust prefix test. The model starts from its existing generation-input roots and follows
/// the same complete dependency edge population forward, so compiler substrate modules outside
/// the root prefixes retain their real role.
pub fn regen_generation_role_population(
    source_roots: &[String],
    edges: &[(String, String)],
    modules: &[String],
) -> Result<(BTreeSet<String>, BTreeSet<String>, BTreeSet<String>), String> {
    use crate::v1_interpreter::{self, str_value, ExecutionMode, Value};
    let entry = affected_set_entry(source_roots)?;
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| format!("refusal: {entry} did not resolve for generation roles: {e}"))?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let edge_values: Vec<Value> = edges
        .iter()
        .map(|(from, to)| Value::Record {
            type_name: ctx.sym("DependencyEdge"),
            fields: Rc::new(vec![
                (ctx.sym("from"), str_value(from.clone())),
                (ctx.sym("to"), str_value(to.clone())),
            ]),
        })
        .collect();
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
        vec![
            (
                Some("edges".to_string()),
                Value::List(Rc::new(edge_values.into())),
            ),
            (Some("modules".to_string()), strings(modules)),
        ],
    )?;
    let bootstrap_sources = list_result("regen_bootstrap_source_modules", vec![])?;
    let bootstrap_products = list_result("regen_bootstrap_product_paths", vec![])?;
    Ok((generation, bootstrap_sources, bootstrap_products))
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
