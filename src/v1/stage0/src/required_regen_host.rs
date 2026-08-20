//! Host realization for `v2.workflow.required_regen` — committed seed vs fresh emit.

use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::Path;
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
    HAND_MAINTAINED_STAGE0_DIRS, HAND_MAINTAINED_STAGE0_FILES,
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
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    /// `Some` only where the pass measured it -- the mirror of the above.
    pub fn fixed_point_equal(&self) -> Option<bool> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
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
            RegenReceipt::FixedPoint { .. } => None,
        }
    }

    /// The referenced first-pass evidence, present only on `FixedPoint`.
    /// The digest of the tree THIS pass emitted.
    ///
    /// TOTAL, unlike the accessors above, and the difference is the point: both variants
    /// measure a candidate digest, so there is no arm that has none and no `Option` to
    /// misread as "unmeasured". The Option-returning siblings are Option because the other
    /// variant genuinely does not measure that fact.
    ///
    /// Its consumer is the composed `--required-ci` run, which hands pass 1's digest to the
    /// fixed-point pass IN MEMORY rather than having it re-read the receipt file the previous
    /// process wrote. `run_required_regen_fixed_point` has always taken `pass1_digest:
    /// Option<String>`; before the phases shared a process there was no way to supply it.
    pub fn candidate_generated_digest(&self) -> &str {
        match self {
            RegenReceipt::FirstGeneration {
                candidate_generated_digest,
                ..
            } => candidate_generated_digest,
            RegenReceipt::FixedPoint {
                candidate_generated_digest,
                ..
            } => candidate_generated_digest,
        }
    }

    pub fn prior(&self) -> Option<&PriorReceiptRef> {
        match self {
            RegenReceipt::FirstGeneration { .. } => None,
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
    /// generation to have a digest OF. The receipt still has to carry a `String` in that
    /// position, and it carries the sentinel `refused:population` — which is why the fixed-point
    /// handoff must not read the receipt. Reading it there would compare a real pass-two digest
    /// against sentinel prose and report a determinism failure nobody measured: a fabricated
    /// plausible output (DESIGN §5), and a convincing one, since the message names two digests
    /// and looks exactly like a genuine mismatch.
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

    let commit_sha = git_head_sha(&workspace)?;
    let sources = super::regen_input_sources(&workspace)?;
    let authority_digest = authority_digest_from_sources(&sources)?;

    // ONE producer of the drift fact, shared with `measure_generated_drift`. What differs between
    // the two callers is only what a refusal MEANS here — a receipt plus an `Ok` carrying
    // failures, rather than an `Err` — so the policy is applied at this call site and the
    // measurement is not re-typed.
    let (emitted, committed_basenames, emitted_basenames, sync) =
        match measure_generated_surface(&workspace, &stage0_src)? {
            GeneratedSurfaceMeasured::Refused { reason } => {
                return regen_refusal_outcome(
                    &workspace,
                    candidate_dir_rel,
                    receipt_rel,
                    commit_sha,
                    authority_digest,
                    reason,
                );
            }
            GeneratedSurfaceMeasured::Measured {
                emitted,
                committed,
                emitted_basenames,
                sync,
            } => (emitted, committed, emitted_basenames, sync),
        };
    // verify_hand_maintained writes scratch normalize files into candidate_dir; on a clean
    // tree nothing has created that directory yet (write_emitted_tree does so later), so it
    // must exist before this call.
    fs::create_dir_all(&candidate_dir)
        .map_err(|e| format!("create {}: {e}", candidate_dir.display()))?;
    let hand = verify_hand_maintained(&emitted, &stage0_src, &candidate_dir)?;

    let committed_digest =
        tree_digest_for_basenames(&stage0_src, &committed_basenames, "committed")?;
    let candidate_digest = tree_digest_from_map(&emitted, &committed_basenames)?;

    let first_generation_equal = sync.matches && hand.unverifiable.is_empty();
    let changed_paths = sync.drifted_paths.clone();

    if candidate_dir.exists() {
        fs::remove_dir_all(&candidate_dir)
            .map_err(|e| format!("remove {}: {e}", candidate_dir.display()))?;
    }
    let fresh_src = candidate_dir.join("src");
    write_emitted_tree(&fresh_src, &emitted)?;
    copy_hand_maintained_support(&stage0_src, &fresh_src)?;
    verify_candidate_tree(&fresh_src, &committed_basenames)?;

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

    eprintln!(
        "required-regen: elapsed_ms={} first_generation_equal={} planned={} executed={}",
        run_started.elapsed().as_millis(),
        first_generation_equal,
        committed_basenames.len(),
        emitted_basenames.len()
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
    let sources = super::regen_input_sources(&workspace)?;
    let authority_digest = authority_digest_from_sources(&sources)?;
    let emitted = compile_stage0(&workspace)?;
    let committed_basenames = committed_generated_basenames(&workspace.join("src/v1/stage0/src"))?;
    if emitted.is_empty() {
        return Err("refusal: fixed-point emit produced zero files".to_string());
    }
    let emitted_basenames = generated_basenames_from_emit(&emitted);
    if let Some(reason) = validate_compared_populations(&committed_basenames, &emitted_basenames) {
        return Err(reason);
    }
    let pass2 = tree_digest_from_map(&emitted, &committed_basenames)?;
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

fn measure_generated_surface(
    workspace: &Path,
    stage0_src: &Path,
) -> Result<GeneratedSurfaceMeasured, String> {
    let emitted = compile_stage0(workspace)?;
    if emitted.is_empty() {
        return Ok(GeneratedSurfaceMeasured::Refused {
            reason: "refusal: emit produced zero files".to_string(),
        });
    }
    let committed = committed_generated_basenames(stage0_src)?;
    let emitted_basenames = generated_basenames_from_emit(&emitted);
    if let Some(reason) = validate_compared_populations(&committed, &emitted_basenames) {
        return Ok(GeneratedSurfaceMeasured::Refused { reason });
    }
    let sync = compare_generated_surfaces(stage0_src, &emitted, &committed)?;
    Ok(GeneratedSurfaceMeasured::Measured {
        emitted,
        committed,
        emitted_basenames,
        sync,
    })
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
    let emitted = match measure_generated_surface(&workspace, &stage0_src)? {
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
}

fn compile_stage0(workspace: &Path) -> Result<HashMap<String, String>, String> {
    let sources = super::regen_input_sources(workspace)?;
    let source_files: Vec<Rc<SourceFile>> = sources
        .into_iter()
        .map(|(path, content)| Rc::new(SourceFile { path, content }))
        .collect();
    let result = compile_sources(Rc::new(source_files.into()), RenderTarget::Rust);
    if let Some(message) = stage0_self_compile_refusal_message(result.clone()) {
        return Err(message);
    }
    let mut out = HashMap::new();
    for file in result.files.iter() {
        out.insert(file.path.clone(), file.content.clone());
    }
    Ok(out)
}

fn generated_basenames_from_emit(emitted: &HashMap<String, String>) -> Vec<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in emitted.keys() {
        if path.ends_with(".rs") && !is_hand_maintained_path(path) {
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
        if basename.ends_with(".rs") && !HAND_MAINTAINED_STAGE0_FILES.contains(&basename) {
            names.insert(basename.to_string());
        }
    }
    if names.is_empty() {
        return Err("refusal: committed generated population is empty".to_string());
    }
    Ok(names.into_iter().collect())
}

fn validate_compared_populations(committed: &[String], emitted: &[String]) -> Option<String> {
    if committed.is_empty() {
        return Some("refusal: committed generated population is empty".to_string());
    }
    if emitted.is_empty() {
        return Some("refusal: emit produced zero generated surfaces".to_string());
    }
    let committed_set: BTreeSet<&str> = committed.iter().map(String::as_str).collect();
    let emitted_set: BTreeSet<&str> = emitted.iter().map(String::as_str).collect();
    let mut emitted_not_committed = Vec::new();
    for name in emitted {
        if !committed_set.contains(name.as_str()) {
            emitted_not_committed.push(name.clone());
        }
    }
    let mut committed_not_emitted = Vec::new();
    for name in committed {
        if !emitted_set.contains(name.as_str()) {
            committed_not_emitted.push(name.clone());
        }
    }
    if !emitted_not_committed.is_empty() || !committed_not_emitted.is_empty() {
        return Some(format!(
            "refusal: surface population mismatch — emitted_not_committed={:?} committed_not_emitted={:?}",
            emitted_not_committed,
            committed_not_emitted
        ));
    }
    None
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
    // A population refusal happens BEFORE any content comparison, so the digests are not
    // "refused" values of a measurement -- there was no measurement. The sentinel survives in the
    // RECEIPT, whose fields are `String`, and `first_generation_equal: false` below is likewise
    // "not asked" rather than "asked and answered no". What no longer survives is the sentinel's
    // route OUT: the outcome carries `FirstGeneration::NotMeasured`, so the composed coordinator
    // cannot hand this string to the fixed-point phase.
    let receipt = RegenReceipt::FirstGeneration {
        schema: RECEIPT_SCHEMA.to_string(),
        commit_sha,
        authority_digest,
        committed_generated_digest: "refused:population".to_string(),
        candidate_generated_digest: "refused:population".to_string(),
        first_generation_equal: false,
        changed_paths: Vec::new(),
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
        let candidate_norm = normalize_generated_source(candidate)
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
    emitted: &HashMap<String, String>,
    stage0_src: &Path,
    work_dir: &Path,
) -> Result<HandVerifyReport, String> {
    let mut unverifiable = Vec::new();
    for file_name in HAND_MAINTAINED_STAGE0_FILES {
        let candidate = emitted
            .get(&format!("src/{file_name}"))
            .or_else(|| emitted.get(*file_name));
        let Some(candidate) = candidate else {
            continue;
        };
        let committed_path = stage0_src.join(file_name);
        let committed = fs::read_to_string(&committed_path)
            .map_err(|e| format!("read committed hand file {}: {e}", committed_path.display()))?;
        match normalize_with_workdir(&committed, work_dir, "committed") {
            Ok(committed_norm) => match normalize_with_workdir(candidate, work_dir, "candidate") {
                Ok(candidate_norm) => {
                    if committed_norm != candidate_norm {
                        // drift expected on clean tree for some hand files; not a sync refusal.
                    }
                }
                Err(reason) => unverifiable.push(((*file_name).to_string(), reason)),
            },
            Err(reason) => unverifiable.push(((*file_name).to_string(), reason)),
        }
    }
    Ok(HandVerifyReport { unverifiable })
}

fn write_emitted_tree(dest_src: &Path, emitted: &HashMap<String, String>) -> Result<(), String> {
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
            normalize_generated_source(content)
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
        }
    }
    for dir_name in HAND_MAINTAINED_STAGE0_DIRS {
        let source = stage0_src.join(dir_name);
        if source.is_dir() {
            copy_dir_recursive(&source, &dest_src.join(dir_name))?;
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
        let norm = normalize_generated_source(&content)
            .map_err(|e| format!("normalize {label} {name}: {e}"))?;
        payload.push_str(name);
        payload.push('\0');
        payload.push_str(&digest_label(norm.as_bytes()));
        payload.push('\n');
    }
    Ok(digest_label(payload.as_bytes()))
}

fn tree_digest_from_map(
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
        let norm = normalize_generated_source(content)
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
fn normalize_generated_source(content: &str) -> Result<String, String> {
    let mut current = normalize_generated_source_attempt(content)?;
    for _ in 1..NORMALIZE_FIXED_POINT_MAX_PASSES {
        let next = normalize_generated_source_attempt(&current)?;
        if next == current {
            return Ok(current);
        }
        current = next;
    }
    Err(format!(
        "rustfmt did not reach a fixed point in {NORMALIZE_FIXED_POINT_MAX_PASSES} passes"
    ))
}

fn normalize_generated_source_attempt(content: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
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

fn normalize_with_workdir(content: &str, work_dir: &Path, label: &str) -> Result<String, String> {
    let path = work_dir.join(format!("{label}.rs"));
    fs::write(&path, content).map_err(|e| format!("write {}: {e}", path.display()))?;
    let output = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(path.as_os_str())
        .output()
        .map_err(|e| format!("rustfmt {label}: {e}"))?;
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

    #[test]
    fn empty_population_digest_refuses() {
        let err = tree_digest_for_basenames(Path::new("/tmp"), &[], "committed").unwrap_err();
        assert!(err.contains("empty population"));
        let err = tree_digest_from_map(&HashMap::new(), &[]).unwrap_err();
        assert!(err.contains("empty population"));
    }

    #[test]
    fn empty_emit_population_refuses_before_agreement() {
        let reason =
            validate_compared_populations(&["foo.rs".to_string()], &[]).expect("expected refusal");
        assert!(reason.contains("zero generated surfaces"));
    }

    #[test]
    fn empty_committed_population_refuses_before_agreement() {
        let reason =
            validate_compared_populations(&[], &["foo.rs".to_string()]).expect("expected refusal");
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

    // THE SENTINEL HAS NO ROUTE TO THE FIXED-POINT PHASE.
    //
    // The defect this pins, found in review of gunbc#8647: a population refusal returns `Ok`
    // with a receipt whose digest fields hold `refused:population`. Before the typed
    // `FirstGeneration`, the composed coordinator read the receipt, so a refusal handed that
    // string to phase three, which compared it against a real pass-two digest and reported
    // `fixed-point refused: pass-1 digest refused:population != pass-2 digest <real>` -- a
    // determinism failure nobody measured, wearing the exact shape of a real one.
    //
    // RED, stated at what this test can actually reach: making `pass1_digest_for_fixed_point`
    // answer from the receipt -- `Some(outcome.receipt.candidate_generated_digest())` -- fails
    // it. What it does NOT reach is the coordinator choosing to bypass the accessor and read the
    // receipt itself; that is one call site in `claim_executor.rs`, guarded by the comment there
    // and by review, not by this test. Naming the gap rather than implying the test closes it.
    #[test]
    fn a_refused_first_generation_hands_no_digest_to_the_fixed_point() {
        let sentinel_receipt = || RegenReceipt::FirstGeneration {
            schema: RECEIPT_SCHEMA.to_string(),
            commit_sha: "sha".to_string(),
            authority_digest: "auth".to_string(),
            committed_generated_digest: "refused:population".to_string(),
            candidate_generated_digest: "refused:population".to_string(),
            first_generation_equal: false,
            changed_paths: Vec::new(),
            candidate_artifact: "cand".to_string(),
        };

        let refused = RequiredRegenOutcome {
            receipt: sentinel_receipt(),
            failures: vec!["refusal: emit produced zero files".to_string()],
            first_generation: FirstGeneration::NotMeasured(
                "refusal: emit produced zero files".to_string(),
            ),
        };
        assert_eq!(pass1_digest_for_fixed_point(&refused), None);
        // And the sentinel IS still sitting in the receipt, which is what makes the coproduct
        // load-bearing rather than decorative: the wrong answer is right there to be read.
        assert_eq!(
            refused.receipt.candidate_generated_digest(),
            "refused:population"
        );

        // POSITIVE CONTROL: ordinary drift is not a refusal. Pass one emitted, the comparison
        // disagreed, and the fixed point still has a subject -- skipping it there would lose a
        // determinism signal exactly when drift makes it interesting.
        let drifted = RequiredRegenOutcome {
            receipt: sentinel_receipt(),
            failures: vec!["17 file(s) drifted".to_string()],
            first_generation: FirstGeneration::Measured("real-digest".to_string()),
        };
        assert_eq!(pass1_digest_for_fixed_point(&drifted), Some("real-digest"));
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
