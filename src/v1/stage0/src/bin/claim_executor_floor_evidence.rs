//! Attempt-scoped floor-evidence sink (operator ruling on #7785; commit 4A closes the
//! evidence boundary: exact-attempt artifact upload, no broken legacy symlink, a
//! cross-language fingerprint file, fail-closed required writes, atomic per-receipt
//! observation fragments, and an honest manifest body digest).
//!
//! Seed-retained scaffold: registry rows mirror `gunbc.ci_floor_population_receipt_registry`
//! and MUST stay fingerprint-paired with `ci_floor_population_receipt_registry_fingerprint`
//! (the pairing now runs against the canonical bytes at
//! `dag/gunbc/ci_floor_population_receipt_registry.fingerprint`, included at compile time —
//! not a second hand-copied literal).
//! Dissolve-on: generated registry projection + write_floor_receipt observation fragments
//! replace this hand mirror (`finalize_floor_evidence_seed_deferral`).

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub const SCHEMA: &str = "gunbc.ci_floor_population_receipt_manifest.v4";
pub const EVIDENCE_ROOT_PREFIX: &str = "target/floor-evidence";
pub const OBSERVATIONS_DIR: &str = "observations";
pub const MANIFEST_REL: &str = "manifest.tsv";
pub const PHASE_JOURNAL_REL: &str = "phase-journal.tsv";

/// Cross-language registry binding (#7785 commit 4A). The canonical fingerprint bytes
/// are generated from `gunbc.ci_floor_population_receipt_registry`'s
/// `ci_floor_population_receipt_registry_fingerprint()` and committed at this path —
/// ONE authority both languages read, rather than a Rust literal that can drift from a
/// `.dag` edit unnoticed. A `.dag` registry edit that does not update the file reds
/// `registry_fingerprint_pairs_const` (this module) and the `.dag`-side pin witness.
pub const FLOOR_EVIDENCE_REGISTRY_FINGERPRINT: &str =
    include_str!("../../../../../dag/gunbc/ci_floor_population_receipt_registry.fingerprint");

#[derive(Clone, Copy)]
enum Locator {
    Exact(&'static str),
    Family {
        dir: &'static str,
        prefix: &'static str,
        suffix: &'static str,
    },
}

struct RegistryRow {
    kind: &'static str,
    locator: Locator,
    phase: &'static str,
    journal_phase: &'static str,
    required_for_settlement: bool,
    optional_worker: bool,
}

/// Mirrors `ci_floor_population_receipt_registry()` — fingerprint-paired, not rediscovery authority.
const ROWS: &[RegistryRow] = &[
    RegistryRow {
        kind: "PopulationReceiptManifest",
        locator: Locator::Exact("manifest.tsv"),
        phase: "AfterFloorAlways",
        journal_phase: "AfterFloorAlways",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "PhaseJournal",
        locator: Locator::Exact("phase-journal.tsv"),
        phase: "DuringFloor",
        journal_phase: "DuringFloor",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "BatchWall",
        locator: Locator::Exact("receipts/batch-wall.txt"),
        phase: "DuringFloor",
        journal_phase: "batch-wall-receipt",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "CompileCleanWall",
        locator: Locator::Exact("receipts/compile-clean-wall.txt"),
        phase: "DuringFloor",
        journal_phase: "compile-clean-wall-receipt",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "Component",
        locator: Locator::Exact("receipts/component.json"),
        phase: "DuringFloor",
        journal_phase: "floor-component-receipt",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "NativeTransition",
        locator: Locator::Exact("receipts/native-transition.tsv"),
        phase: "DuringFloor",
        journal_phase: "native-transition-receipt",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "OnSuccessStage1",
        locator: Locator::Exact("stages/1/receipt.tsv"),
        phase: "OnSuccessStage",
        journal_phase: "on-success-stage-1",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "OnSuccessStage2",
        locator: Locator::Exact("stages/2/receipt.tsv"),
        phase: "OnSuccessStage",
        journal_phase: "on-success-stage-2",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "OnSuccessMaterialization",
        locator: Locator::Exact("receipts/on-success-materialization.txt"),
        phase: "OnSuccessStage",
        journal_phase: "on-success-materialization",
        required_for_settlement: true,
        optional_worker: false,
    },
    RegistryRow {
        kind: "WorkerObservation",
        locator: Locator::Exact("workers/observation.tsv"),
        phase: "OptionalWorker",
        journal_phase: "coordinator-observation",
        required_for_settlement: false,
        optional_worker: true,
    },
    RegistryRow {
        kind: "WorkerTerminal",
        locator: Locator::Family {
            dir: "workers",
            prefix: "terminal-",
            suffix: ".tsv",
        },
        phase: "OptionalWorker",
        journal_phase: "OptionalWorker",
        required_for_settlement: false,
        optional_worker: true,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseStandingExact {
    PhaseNotReached,
    PhaseEntered,
    PhaseCompleted,
    PhaseRefused,
    PhaseInterrupted,
    PhaseStandingUnknown,
}

fn pattern_of(locator: Locator) -> String {
    match locator {
        Locator::Exact(p) => p.to_string(),
        Locator::Family {
            dir,
            prefix,
            suffix,
        } => format!("{dir}/{prefix}*{suffix}"),
    }
}

fn fingerprint_from_rows() -> String {
    ROWS.iter()
        .map(|r| {
            format!(
                "{}|{}|{}|{}|{}",
                r.kind,
                pattern_of(r.locator),
                r.phase,
                r.journal_phase,
                if r.required_for_settlement { "1" } else { "0" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn find_row(kind: &str) -> Option<&'static RegistryRow> {
    ROWS.iter().find(|r| r.kind == kind)
}

fn is_family_kind(kind: &str) -> bool {
    matches!(
        find_row(kind).map(|r| r.locator),
        Some(Locator::Family { .. })
    )
}

pub fn floor_evidence_root(attempt: &str) -> PathBuf {
    PathBuf::from(EVIDENCE_ROOT_PREFIX).join(attempt)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn sha256_file(path: &Path) -> Result<(u64, String), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok((bytes.len() as u64, sha256_bytes(&bytes)))
}

fn relative_path_escapes_root(relative_path: &str) -> bool {
    if relative_path.is_empty()
        || relative_path.starts_with('/')
        || relative_path.contains("..")
        || Path::new(relative_path).is_absolute()
    {
        return true;
    }
    for c in Path::new(relative_path).components() {
        if matches!(
            c,
            std::path::Component::ParentDir | std::path::Component::RootDir
        ) {
            return true;
        }
    }
    false
}

/// Observation-fragment filename for one receipt (blocker 5, #7785 commit 4A). A
/// singleton kind (any `Locator::Exact` row) owns exactly one fragment file named after
/// the kind, so a second write for the same kind is a filesystem-visible collision
/// rather than a silently-appended row; a family kind (`WorkerTerminal`) is keyed by the
/// receipt's own basename so N workers get N fragments.
fn observation_fragment_relpath(kind: &str, relative_path: &str) -> String {
    if is_family_kind(kind) {
        let basename = Path::new(relative_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| relative_path.to_string());
        format!("{OBSERVATIONS_DIR}/{kind}-{basename}.obs.tsv")
    } else {
        format!("{OBSERVATIONS_DIR}/{kind}.obs.tsv")
    }
}

/// Atomic write under `target/floor-evidence/<attempt>/` plus an atomic per-receipt
/// observation fragment (blocker 5, #7785 commit 4A): write receipt tmp → write obs tmp
/// → rename receipt → rename obs — a best-effort atomic PAIR, not a single fsync unit.
/// A crash between the two renames leaves a published receipt with no observation
/// fragment, which the finalizer reads as `MissingAfterProducerReached` (a real, typed
/// state) rather than as a false `ObservedPresent` — it never claims presence it cannot
/// prove, and it never partially-writes a shared file that a concurrent worker could
/// also be appending to (the defect this fragment split replaces).
pub fn write_floor_receipt(
    attempt: &str,
    kind: &str,
    relative_path: &str,
    body: &[u8],
    producer_phase: &str,
) -> Result<PathBuf, String> {
    if relative_path_escapes_root(relative_path) {
        return Err(format!(
            "write_floor_receipt refused unsafe relative path {relative_path:?}"
        ));
    }
    let root = floor_evidence_root(attempt);
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let receipt_tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&receipt_tmp, body).map_err(|e| format!("write {}: {e}", receipt_tmp.display()))?;

    let digest = sha256_bytes(body);
    let obs_rel = observation_fragment_relpath(kind, relative_path);
    let obs_path = root.join(&obs_rel);
    if let Some(parent) = obs_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            let _ = fs::remove_file(&receipt_tmp);
            return Err(format!("mkdir {}: {e}", parent.display()));
        }
    }
    let obs_body = format!(
        "{kind}\t{relative_path}\t{}\t{digest}\t{producer_phase}\n",
        body.len()
    );
    let obs_tmp = obs_path.with_extension(format!("tmp-{}", std::process::id()));
    if let Err(e) = fs::write(&obs_tmp, obs_body.as_bytes()) {
        let _ = fs::remove_file(&receipt_tmp);
        return Err(format!("write {}: {e}", obs_tmp.display()));
    }

    if let Err(e) = fs::rename(&receipt_tmp, &path) {
        let _ = fs::remove_file(&receipt_tmp);
        let _ = fs::remove_file(&obs_tmp);
        return Err(format!("publish {}: {e}", path.display()));
    }
    if let Err(e) = fs::rename(&obs_tmp, &obs_path) {
        let _ = fs::remove_file(&obs_tmp);
        return Err(format!("publish {}: {e}", obs_path.display()));
    }
    Ok(path)
}

pub fn install_floor_evidence_root(attempt: &str) -> Result<PathBuf, String> {
    let root = floor_evidence_root(attempt);
    fs::create_dir_all(&root).map_err(|e| format!("mkdir {}: {e}", root.display()))?;
    let journal = root.join(PHASE_JOURNAL_REL);
    if let Some(parent) = journal.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    // Touch so the dump step can distinguish absent vs empty when read.
    if !journal.exists() {
        fs::write(&journal, "").map_err(|e| format!("touch {}: {e}", journal.display()))?;
    }
    // No legacy `target/floor-phase-journal.tsv` symlink/copy (#7785 commit 4A, blocker 2):
    // the workflow composes the SAME attempt-scoped path directly at the GitHub Actions
    // expression level (v2.workflow.ci_floor_peak_emit `ci_floor_phase_journal_path`), so
    // every consumer — this process, the peak-post step, the falsifier step — reads the
    // one real file rather than a second alias that a stale symlink target could break.
    let journal_for_env = journal.canonicalize().unwrap_or_else(|_| journal.clone());
    std::env::set_var("GUNBC_FLOOR_PHASE_JOURNAL", &journal_for_env);
    Ok(root)
}

fn parse_state(state: &str) -> PhaseStandingExact {
    let s = state.trim().to_ascii_lowercase();
    if s.contains("interrupt") || s.contains("killed") || s.contains("abort") || s == "interrupted"
    {
        PhaseStandingExact::PhaseInterrupted
    } else if s == "completed" || s == "passed" {
        PhaseStandingExact::PhaseCompleted
    } else if s == "refused" || s == "failed" {
        PhaseStandingExact::PhaseRefused
    } else if s == "started" || s == "running" || s == "entered" {
        PhaseStandingExact::PhaseEntered
    } else {
        PhaseStandingExact::PhaseStandingUnknown
    }
}

fn merge_standing(cur: PhaseStandingExact, next: PhaseStandingExact) -> PhaseStandingExact {
    use PhaseStandingExact::*;
    match (cur, next) {
        (_, PhaseInterrupted) | (PhaseInterrupted, _) => PhaseInterrupted,
        (_, PhaseRefused) | (PhaseRefused, _) => PhaseRefused,
        (_, PhaseCompleted) | (PhaseCompleted, _) => PhaseCompleted,
        (_, PhaseEntered) | (PhaseEntered, _) => PhaseEntered,
        (PhaseStandingUnknown, other) | (other, PhaseStandingUnknown) => other,
        (PhaseNotReached, other) => other,
    }
}

pub fn parse_phase_journal(body: &str) -> HashMap<String, PhaseStandingExact> {
    let mut map = HashMap::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        // unix_millis pid phase state detail
        if parts.len() < 4 {
            continue;
        }
        let phase = parts[2].to_string();
        let standing = parse_state(parts[3]);
        let entry = map
            .entry(phase)
            .or_insert(PhaseStandingExact::PhaseNotReached);
        *entry = merge_standing(*entry, standing);
    }
    map
}

fn standing_label_missing(optional_worker: bool, phase: PhaseStandingExact) -> &'static str {
    match phase {
        PhaseStandingExact::PhaseNotReached | PhaseStandingExact::PhaseStandingUnknown => {
            if optional_worker {
                "NotApplicable"
            } else {
                "MissingBeforeProducerReached"
            }
        }
        PhaseStandingExact::PhaseEntered
        | PhaseStandingExact::PhaseCompleted
        | PhaseStandingExact::PhaseRefused => "MissingAfterProducerReached",
        PhaseStandingExact::PhaseInterrupted => "ProducerInterrupted",
    }
}

#[derive(Clone, Debug)]
struct Observation {
    kind: String,
    relative_path: String,
    size: u64,
    digest: String,
    #[allow(dead_code)]
    producer_phase: String,
}

/// Enumerate `observations/*.obs.tsv` fragments only (blocker 5, #7785 commit 4A) — never
/// a shared append-log. Refuses loudly (never silently ignores) on: an unknown kind, a
/// path that escapes `FloorEvidenceRoot`, a second fragment for a singleton kind, a
/// duplicate `(kind, path)` pair, or a fragment whose declared `producer_phase` disagrees
/// with the registry row it names.
fn load_observation_fragments(root: &Path) -> Result<Vec<Observation>, String> {
    let dir = root.join(OBSERVATIONS_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut fragment_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().ends_with(".obs.tsv"))
                .unwrap_or(false)
        })
        .collect();
    // Deterministic order so a duplicate refusal names the same pair on every run.
    fragment_paths.sort();

    let mut out: Vec<Observation> = Vec::new();
    for frag_path in &fragment_paths {
        let body = fs::read_to_string(frag_path)
            .map_err(|e| format!("read {}: {e}", frag_path.display()))?;
        let lines: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.len() != 1 {
            return Err(format!(
                "observation fragment {} must carry exactly one row, found {}",
                frag_path.display(),
                lines.len()
            ));
        }
        let parts: Vec<&str> = lines[0].split('\t').collect();
        if parts.len() != 5 {
            return Err(format!(
                "malformed observation fragment row in {}: {:?}",
                frag_path.display(),
                lines[0]
            ));
        }
        let kind = parts[0].to_string();
        let rel = parts[1].to_string();
        let size: u64 = parts[2].parse().map_err(|_| {
            format!(
                "bad size in observation fragment {}: {:?}",
                frag_path.display(),
                lines[0]
            )
        })?;
        let digest = parts[3].to_string();
        let producer_phase = parts[4].to_string();

        let Some(row) = find_row(&kind) else {
            return Err(format!(
                "observation fragment {} names unknown kind {kind:?} — refused rather than ignored",
                frag_path.display()
            ));
        };

        if relative_path_escapes_root(&rel) {
            return Err(format!(
                "observation fragment {} path escapes FloorEvidenceRoot: {rel:?}",
                frag_path.display()
            ));
        }

        if producer_phase != row.phase {
            return Err(format!(
                "observation fragment {} declares producer_phase={producer_phase:?} but registry row {kind} expects {:?}",
                frag_path.display(),
                row.phase
            ));
        }

        out.push(Observation {
            kind,
            relative_path: rel,
            size,
            digest,
            producer_phase,
        });
    }

    for row in ROWS {
        if matches!(row.locator, Locator::Family { .. }) {
            continue;
        }
        let count = out.iter().filter(|o| o.kind == row.kind).count();
        if count > 1 {
            return Err(format!(
                "duplicate observation fragments for singleton kind {} ({count} fragments; expected at most one)",
                row.kind
            ));
        }
    }

    for i in 0..out.len() {
        for j in (i + 1)..out.len() {
            if out[i].kind == out[j].kind && out[i].relative_path == out[j].relative_path {
                return Err(format!(
                    "duplicate observation for (kind={}, path={})",
                    out[i].kind, out[i].relative_path
                ));
            }
        }
    }

    Ok(out)
}

fn env_or_empty(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

fn is_git_object_hex(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64) && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn walk_attempt_segment_ok(raw: &str) -> bool {
    !(raw.is_empty()
        || raw == "."
        || raw == ".."
        || raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\n')
        || raw.contains('\r')
        || raw.contains('\0'))
}

enum SubjectOutcome {
    Observed {
        run_id: String,
        run_attempt: String,
        job_key: String,
        head: String,
        tree: String,
        walk: String,
    },
    Refused {
        cause: String,
        run_id: String,
        run_attempt: String,
        job_key: String,
        head: String,
        tree: String,
        walk: String,
    },
}

fn observe_local_subject() -> SubjectOutcome {
    let run_id = env_or_empty("GITHUB_RUN_ID");
    let run_attempt = env_or_empty("GITHUB_RUN_ATTEMPT");
    let job_key = env_or_empty("GITHUB_JOB");
    let head = env_or_empty("GITHUB_SHA");
    let mut walk = env_or_empty("GUNBC_WALK_ATTEMPT_ID");
    if walk.is_empty() && !run_id.is_empty() && !run_attempt.is_empty() && !job_key.is_empty() {
        walk = format!("{run_id}-{run_attempt}-{job_key}");
    }
    let mut tree = env_or_empty("GUNBC_TESTED_TREE");
    if tree.is_empty() && !head.is_empty() {
        if let Ok(out) = Command::new("git")
            .args(["log", "-1", "--format=%T", &head])
            .output()
        {
            if out.status.success() {
                tree = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
    }

    let refuse = |cause: &str| SubjectOutcome::Refused {
        cause: cause.to_string(),
        run_id: run_id.clone(),
        run_attempt: run_attempt.clone(),
        job_key: job_key.clone(),
        head: head.clone(),
        tree: tree.clone(),
        walk: walk.clone(),
    };

    if run_id.is_empty() {
        return refuse("missing-github_run_id");
    }
    if run_attempt.is_empty() {
        return refuse("missing-github_run_attempt");
    }
    if job_key.is_empty() {
        return refuse("missing-github_job_key");
    }
    if head.is_empty() {
        return refuse("missing-head_commit");
    }
    if !is_git_object_hex(&head) {
        return refuse("invalid-head_commit");
    }
    if tree.is_empty() {
        return refuse("missing-tested_tree");
    }
    if !is_git_object_hex(&tree) {
        return refuse("invalid-tested_tree");
    }
    if walk.is_empty() {
        return refuse("missing-walk_attempt_id");
    }
    if !walk_attempt_segment_ok(&walk) {
        return refuse("invalid-walk_attempt_id");
    }
    SubjectOutcome::Observed {
        run_id,
        run_attempt,
        job_key,
        head,
        tree,
        walk,
    }
}

fn observations_for_row<'a>(
    row: &RegistryRow,
    observations: &'a [Observation],
) -> Vec<&'a Observation> {
    match row.locator {
        Locator::Exact(p) => observations
            .iter()
            .filter(|o| o.kind == row.kind && o.relative_path == p)
            .collect(),
        Locator::Family {
            dir,
            prefix,
            suffix,
        } => {
            let dir_prefix = format!("{dir}/");
            observations
                .iter()
                .filter(|o| {
                    o.kind == row.kind
                        && o.relative_path.starts_with(&dir_prefix)
                        && o.relative_path[dir_prefix.len()..].starts_with(prefix)
                        && o.relative_path.ends_with(suffix)
                })
                .collect()
        }
    }
}

/// Writes `floor_evidence_root` and `walk_attempt` to `$GITHUB_OUTPUT` (blocker 1, #7785
/// commit 4A) so the downstream `always()` upload step can address the EXACT attempt
/// root (`${{ steps.finalize_floor_evidence.outputs.floor_evidence_root }}`) instead of
/// a `target/floor-evidence/*` wildcard that would sweep in every sibling attempt a
/// reused self-hosted `target/` happens to still be carrying. A no-op (never an error)
/// when `GITHUB_OUTPUT` is unset — local/dev invocations have nothing to write into.
fn write_github_output_kv(pairs: &[(&str, &str)]) -> Result<(), String> {
    let path = match std::env::var("GITHUB_OUTPUT") {
        Ok(p) if !p.is_empty() => p,
        _ => return Ok(()),
    };
    let mut body = String::new();
    for (k, v) in pairs {
        body.push_str(&format!("{k}={v}\n"));
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open GITHUB_OUTPUT {path}: {e}"))?;
    file.write_all(body.as_bytes())
        .map_err(|e| format!("write GITHUB_OUTPUT {path}: {e}"))?;
    Ok(())
}

/// Finalize one attempt root from observation fragments + exact journal phase standing.
/// Refuses incomplete subject (non-zero exit). Never scrapes sibling attempt roots.
pub fn finalize_floor_evidence() -> Result<ExitCode, ExitCode> {
    let computed = fingerprint_from_rows();
    if computed.trim() != FLOOR_EVIDENCE_REGISTRY_FINGERPRINT.trim() {
        eprintln!(
            "::error::finalize-floor-evidence: seed registry fingerprint drifted from dag/gunbc/ci_floor_population_receipt_registry.fingerprint"
        );
        return Err(ExitCode::from(1));
    }

    let subject = observe_local_subject();
    let (walk, subject_ok, subject_lines, refuse_cause) = match &subject {
        SubjectOutcome::Observed {
            run_id,
            run_attempt,
            job_key,
            head,
            tree,
            walk,
        } => (
            walk.clone(),
            true,
            format!(
                "subject_status=Observed\nrun_id={run_id}\nrun_attempt={run_attempt}\njob_key={job_key}\nhead_commit={head}\ntested_tree={tree}\nwalk_attempt={walk}\n"
            ),
            None,
        ),
        SubjectOutcome::Refused {
            cause,
            run_id,
            run_attempt,
            job_key,
            head,
            tree,
            walk,
        } => (
            walk.clone(),
            false,
            format!(
                "subject_status=Refused\nsubject_refuse_cause={cause}\nrun_id={run_id}\nrun_attempt={run_attempt}\njob_key={job_key}\nhead_commit={head}\ntested_tree={tree}\nwalk_attempt={walk}\n"
            ),
            Some(cause.clone()),
        ),
    };

    if walk.is_empty() || !walk_attempt_segment_ok(&walk) {
        eprintln!(
            "::error::finalize-floor-evidence: walk_attempt required to select FloorEvidenceRoot (cause={})",
            refuse_cause.as_deref().unwrap_or("missing-or-invalid-walk_attempt")
        );
        return Err(ExitCode::from(1));
    }

    let root = floor_evidence_root(&walk);
    if let Err(e) = fs::create_dir_all(&root) {
        eprintln!(
            "::error::finalize-floor-evidence: mkdir {}: {e}",
            root.display()
        );
        return Err(ExitCode::from(1));
    }

    // Write the exact attempt outputs BEFORE anything below can fail (blocker 1): the
    // upload step downstream runs `if: always()`, so even a later refusal in this same
    // finalize must leave a valid exact `floor_evidence_root` output behind it.
    if let Err(e) = write_github_output_kv(&[
        ("floor_evidence_root", &root.display().to_string()),
        ("walk_attempt", &walk),
    ]) {
        eprintln!("::error::finalize-floor-evidence: {e}");
        return Err(ExitCode::from(1));
    }

    let observations = match load_observation_fragments(&root) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("::error::finalize-floor-evidence: {e}");
            return Err(ExitCode::from(1));
        }
    };

    let journal_path = root.join(PHASE_JOURNAL_REL);
    let journal_map = match fs::read_to_string(&journal_path) {
        Ok(body) => parse_phase_journal(&body),
        Err(_) => HashMap::new(),
    };

    // PhaseJournal itself is present when the journal file exists (even if empty).
    let journal_file_present = journal_path.is_file();

    let mut out = String::new();
    out.push_str(SCHEMA);
    out.push('\n');
    // Honest header (blocker 6, #7785 commit 4A): a SHA-256 of the canonical fingerprint
    // bytes, not the raw multiline blob — the field name says exactly what it is.
    let registry_fingerprint_sha256 =
        sha256_bytes(FLOOR_EVIDENCE_REGISTRY_FINGERPRINT.trim().as_bytes());
    out.push_str(&format!(
        "registry_fingerprint_sha256={registry_fingerprint_sha256}\n"
    ));
    out.push_str(&subject_lines);
    out.push_str(
        "kind\texpected_pattern\tobserved_path\tstanding\tproducer_phase\tsize_bytes\tcontent_digest\n",
    );

    let mut row_count = 0usize;
    for row in ROWS {
        if row.kind == "PopulationReceiptManifest" {
            // The manifest cannot honestly claim a content_digest of itself as a data
            // row (blocker 6): its presence is the file existing at all, and its
            // integrity is `manifest_body_digest` below, computed over the exact bytes
            // that precede it — never a row inside the body it describes.
            continue;
        }
        let pattern = pattern_of(row.locator);
        let phase_standing = journal_map
            .get(row.journal_phase)
            .copied()
            .unwrap_or(PhaseStandingExact::PhaseNotReached);

        if row.kind == "PhaseJournal" {
            if journal_file_present {
                match sha256_file(&journal_path) {
                    Ok((size, digest)) => {
                        out.push_str(&format!(
                            "{}\t{}\t{}\tObservedPresent\t{}\t{}\t{}\n",
                            row.kind,
                            pattern,
                            journal_path.display(),
                            row.phase,
                            size,
                            digest
                        ));
                        row_count += 1;
                    }
                    Err(e) => {
                        eprintln!("::error::finalize-floor-evidence: {e}");
                        return Err(ExitCode::from(1));
                    }
                }
            } else {
                let standing = standing_label_missing(row.optional_worker, phase_standing);
                out.push_str(&format!(
                    "{}\t{}\t\t{}\t{}\t0\t\n",
                    row.kind, pattern, standing, row.phase
                ));
                row_count += 1;
            }
            continue;
        }

        let matches = observations_for_row(row, &observations);
        if matches.is_empty() {
            let standing = standing_label_missing(row.optional_worker, phase_standing);
            out.push_str(&format!(
                "{}\t{}\t\t{}\t{}\t0\t\n",
                row.kind, pattern, standing, row.phase
            ));
            row_count += 1;
            continue;
        }
        for obs in matches {
            // Verify digest against bytes under this root only — refuse foreign paths.
            let abs = root.join(&obs.relative_path);
            match sha256_file(&abs) {
                Ok((size, digest)) => {
                    if digest != obs.digest {
                        eprintln!(
                            "::error::finalize-floor-evidence: observation digest drift for {} (obs={} file={})",
                            abs.display(),
                            obs.digest,
                            digest
                        );
                        return Err(ExitCode::from(1));
                    }
                    if size != obs.size {
                        eprintln!(
                            "::error::finalize-floor-evidence: observation size drift for {}",
                            abs.display()
                        );
                        return Err(ExitCode::from(1));
                    }
                    out.push_str(&format!(
                        "{}\t{}\t{}\tObservedPresent\t{}\t{}\t{}\n",
                        row.kind,
                        pattern,
                        abs.display(),
                        row.phase,
                        size,
                        digest
                    ));
                    row_count += 1;
                }
                Err(e) => {
                    eprintln!("::error::finalize-floor-evidence: {e}");
                    return Err(ExitCode::from(1));
                }
            }
        }
    }

    // Honest self-digest (blocker 6): computed over exactly the bytes written above —
    // schema, header, subject, column header, and every data row — BEFORE this trailer
    // line is appended, so a reader can reproduce it by hashing the file with its last
    // line stripped. Never claimed as a `content_digest` path-digest row inside the
    // body it describes.
    let manifest_body_digest = sha256_bytes(out.as_bytes());
    out.push_str(&format!("manifest_body_digest={manifest_body_digest}\n"));

    let manifest_path = root.join(MANIFEST_REL);
    let mut file = match fs::File::create(&manifest_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "::error::finalize-floor-evidence: create {}: {e}",
                manifest_path.display()
            );
            return Err(ExitCode::from(1));
        }
    };
    if let Err(e) = file.write_all(out.as_bytes()).and_then(|_| file.sync_all()) {
        eprintln!(
            "::error::finalize-floor-evidence: write {}: {e}",
            manifest_path.display()
        );
        return Err(ExitCode::from(1));
    }

    eprintln!(
        "[floor-population-evidence] wrote {} schema={} rows={} subject_ok={subject_ok} attempt={walk}",
        manifest_path.display(),
        SCHEMA,
        row_count
    );

    if !subject_ok {
        eprintln!(
            "::error::finalize-floor-evidence: LocalSubjectRefused cause={}",
            refuse_cause.unwrap_or_else(|| "unknown".to_string())
        );
        return Err(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_cwd<F: FnOnce(&Path)>(f: F) {
        let _guard = CWD_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!(
            "floor-evidence-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        f(&tmp);
        std::env::set_current_dir(&prev).unwrap();
        let _ = fs::remove_dir_all(&tmp);
    }

    fn clear_subject_env() {
        for k in [
            "GITHUB_RUN_ID",
            "GITHUB_RUN_ATTEMPT",
            "GITHUB_JOB",
            "GITHUB_SHA",
            "GUNBC_WALK_ATTEMPT_ID",
            "GUNBC_TESTED_TREE",
            "GITHUB_OUTPUT",
        ] {
            std::env::remove_var(k);
        }
    }

    fn set_full_subject(walk: &str) {
        std::env::set_var("GITHUB_RUN_ID", "1");
        std::env::set_var("GITHUB_RUN_ATTEMPT", "1");
        std::env::set_var("GITHUB_JOB", "ci");
        std::env::set_var("GITHUB_SHA", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        std::env::set_var(
            "GUNBC_TESTED_TREE",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        );
        std::env::set_var("GUNBC_WALK_ATTEMPT_ID", walk);
    }

    #[test]
    fn registry_fingerprint_pairs_const() {
        assert_eq!(
            fingerprint_from_rows().trim(),
            FLOOR_EVIDENCE_REGISTRY_FINGERPRINT.trim()
        );
    }

    #[test]
    fn phase_journal_exact_states() {
        let body = "1\t2\ton-success-stage-2\tstarted\t\n3\t2\ton-success-stage-2\tcompleted\t\n4\t2\tbatch-wall-receipt\tinterrupted\tkilled\n";
        let map = parse_phase_journal(body);
        assert_eq!(
            map.get("on-success-stage-2"),
            Some(&PhaseStandingExact::PhaseCompleted)
        );
        assert_eq!(
            map.get("batch-wall-receipt"),
            Some(&PhaseStandingExact::PhaseInterrupted)
        );
        assert_eq!(map.get("missing"), None);
    }

    #[test]
    fn two_attempt_isolation_zero_cross_bytes() {
        with_temp_cwd(|_| {
            clear_subject_env();
            write_floor_receipt(
                "attempt-a",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"from-a\n",
                "DuringFloor",
            )
            .unwrap();
            write_floor_receipt(
                "attempt-b",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"from-b\n",
                "DuringFloor",
            )
            .unwrap();
            let b_root = floor_evidence_root("attempt-b");
            let a_root = floor_evidence_root("attempt-a");
            let b_body = fs::read_to_string(b_root.join("receipts/batch-wall.txt")).unwrap();
            assert_eq!(b_body, "from-b\n");
            assert!(!b_body.contains("from-a"));
            // No path under B references A.
            let b_obs = fs::read_to_string(b_root.join("observations/BatchWall.obs.tsv")).unwrap();
            assert!(!b_obs.contains("attempt-a"));
            assert!(!b_obs.contains(a_root.to_string_lossy().as_ref()));
            let a_obs = fs::read_to_string(a_root.join("observations/BatchWall.obs.tsv")).unwrap();
            assert!(a_obs.contains("from-a") || a_obs.contains("BatchWall"));
        });
    }

    #[test]
    fn two_worker_terminals_two_observation_fragments() {
        with_temp_cwd(|_| {
            write_floor_receipt(
                "w",
                "WorkerTerminal",
                "workers/terminal-0.tsv",
                b"ok\n",
                "OptionalWorker",
            )
            .unwrap();
            write_floor_receipt(
                "w",
                "WorkerTerminal",
                "workers/terminal-1.tsv",
                b"ok\n",
                "OptionalWorker",
            )
            .unwrap();
            let root = floor_evidence_root("w");
            assert!(root
                .join("observations/WorkerTerminal-terminal-0.tsv.obs.tsv")
                .is_file());
            assert!(root
                .join("observations/WorkerTerminal-terminal-1.tsv.obs.tsv")
                .is_file());
            let obs = load_observation_fragments(&root).unwrap();
            let terminals: Vec<_> = obs.iter().filter(|o| o.kind == "WorkerTerminal").collect();
            assert_eq!(terminals.len(), 2);
        });
    }

    #[test]
    fn install_floor_evidence_root_does_not_create_legacy_symlink() {
        with_temp_cwd(|_| {
            install_floor_evidence_root("no-symlink").unwrap();
            assert!(
                !Path::new("target/floor-phase-journal.tsv").exists(),
                "legacy target/floor-phase-journal.tsv must not be created (#7785 commit 4A blocker 2)"
            );
            assert!(floor_evidence_root("no-symlink")
                .join(PHASE_JOURNAL_REL)
                .is_file());
        });
    }

    #[test]
    fn unknown_observation_kind_refuses() {
        with_temp_cwd(|_| {
            let root = floor_evidence_root("unknown-kind");
            let dir = root.join(OBSERVATIONS_DIR);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("TotallyUnknownKind.obs.tsv"),
                "TotallyUnknownKind\treceipts/foo.txt\t3\tabc\tDuringFloor\n",
            )
            .unwrap();
            let err = load_observation_fragments(&root).unwrap_err();
            assert!(err.contains("unknown kind"), "got: {err}");
        });
    }

    #[test]
    fn duplicate_singleton_observation_refuses() {
        with_temp_cwd(|_| {
            let root = floor_evidence_root("dup-singleton");
            let dir = root.join(OBSERVATIONS_DIR);
            fs::create_dir_all(&dir).unwrap();
            // Two fragments both claiming the singleton BatchWall kind — even under two
            // different filenames, this must refuse (a real crash-injected race, or a
            // second producer that should never exist for a singleton kind).
            fs::write(
                dir.join("BatchWall.obs.tsv"),
                "BatchWall\treceipts/batch-wall.txt\t3\tabc\tDuringFloor\n",
            )
            .unwrap();
            fs::write(
                dir.join("BatchWall-2.obs.tsv"),
                "BatchWall\treceipts/batch-wall-2.txt\t3\tdef\tDuringFloor\n",
            )
            .unwrap();
            let err = load_observation_fragments(&root).unwrap_err();
            assert!(err.contains("duplicate"), "got: {err}");
        });
    }

    #[test]
    fn duplicate_kind_path_pair_refuses() {
        with_temp_cwd(|_| {
            let root = floor_evidence_root("dup-pair");
            let dir = root.join(OBSERVATIONS_DIR);
            fs::create_dir_all(&dir).unwrap();
            let row = "WorkerTerminal\tworkers/terminal-0.tsv\t2\tok\tOptionalWorker\n";
            fs::write(dir.join("WorkerTerminal-terminal-0.tsv.obs.tsv"), row).unwrap();
            fs::write(dir.join("WorkerTerminal-terminal-0.tsv.dup.obs.tsv"), row).unwrap();
            let err = load_observation_fragments(&root).unwrap_err();
            assert!(err.contains("duplicate"), "got: {err}");
        });
    }

    #[test]
    fn phase_mismatch_refuses() {
        with_temp_cwd(|_| {
            let root = floor_evidence_root("phase-mismatch");
            let dir = root.join(OBSERVATIONS_DIR);
            fs::create_dir_all(&dir).unwrap();
            // BatchWall's registry phase is DuringFloor; declare AfterFloorAlways instead.
            fs::write(
                dir.join("BatchWall.obs.tsv"),
                "BatchWall\treceipts/batch-wall.txt\t3\tabc\tAfterFloorAlways\n",
            )
            .unwrap();
            let err = load_observation_fragments(&root).unwrap_err();
            assert!(
                err.contains("phase mismatch") || err.contains("expects"),
                "got: {err}"
            );
        });
    }

    #[test]
    fn path_escape_observation_refuses() {
        with_temp_cwd(|_| {
            let root = floor_evidence_root("escape");
            let dir = root.join(OBSERVATIONS_DIR);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("BatchWall.obs.tsv"),
                "BatchWall\t../escape.txt\t3\tabc\tDuringFloor\n",
            )
            .unwrap();
            let err = load_observation_fragments(&root).unwrap_err();
            assert!(err.contains("escapes"), "got: {err}");
        });
    }

    #[test]
    fn receipt_byte_mutation_changes_sha256() {
        with_temp_cwd(|_| {
            write_floor_receipt(
                "m",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"v1",
                "DuringFloor",
            )
            .unwrap();
            let (_, d1) =
                sha256_file(&floor_evidence_root("m").join("receipts/batch-wall.txt")).unwrap();
            write_floor_receipt(
                "m",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"v2",
                "DuringFloor",
            )
            .unwrap();
            let (_, d2) =
                sha256_file(&floor_evidence_root("m").join("receipts/batch-wall.txt")).unwrap();
            assert_ne!(d1, d2);
        });
    }

    #[test]
    fn write_floor_receipt_refuses_parent_escape() {
        with_temp_cwd(|_| {
            let err =
                write_floor_receipt("x", "BatchWall", "../escape.txt", b"nope", "DuringFloor")
                    .unwrap_err();
            assert!(err.contains("unsafe") || err.contains("escapes"));
        });
    }

    #[test]
    fn incomplete_subject_refuses_finalize() {
        with_temp_cwd(|_| {
            clear_subject_env();
            std::env::set_var("GUNBC_WALK_ATTEMPT_ID", "local-only");
            let code = finalize_floor_evidence();
            assert!(code.is_err());
            clear_subject_env();
        });
    }

    #[test]
    fn finalize_b_contains_zero_a_paths() {
        with_temp_cwd(|_| {
            clear_subject_env();
            write_floor_receipt(
                "attempt-a",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"secret-a\n",
                "DuringFloor",
            )
            .unwrap();
            write_floor_receipt(
                "attempt-b",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"only-b\n",
                "DuringFloor",
            )
            .unwrap();
            fs::write(
                floor_evidence_root("attempt-b").join(PHASE_JOURNAL_REL),
                "1\t1\tbatch-wall-receipt\tcompleted\t\n",
            )
            .unwrap();
            set_full_subject("attempt-b");
            assert!(finalize_floor_evidence().is_ok());
            let manifest =
                fs::read_to_string(floor_evidence_root("attempt-b").join(MANIFEST_REL)).unwrap();
            assert!(!manifest.contains("attempt-a"));
            assert!(!manifest.contains("secret-a"));
            assert!(manifest.contains("only-b") || manifest.contains("ObservedPresent"));
            assert!(manifest.contains("BatchWall"));
            assert!(manifest.contains("registry_fingerprint_sha256="));
            assert!(manifest.contains("manifest_body_digest="));
            assert!(!manifest.contains("PopulationReceiptManifest\tmanifest.tsv"));
            clear_subject_env();
        });
    }

    #[test]
    fn manifest_body_digest_matches_bytes_before_trailer() {
        with_temp_cwd(|_| {
            clear_subject_env();
            write_floor_receipt(
                "digest-check",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"payload\n",
                "DuringFloor",
            )
            .unwrap();
            fs::write(
                floor_evidence_root("digest-check").join(PHASE_JOURNAL_REL),
                "1\t1\tbatch-wall-receipt\tcompleted\t\n",
            )
            .unwrap();
            set_full_subject("digest-check");
            assert!(finalize_floor_evidence().is_ok());
            let manifest =
                fs::read_to_string(floor_evidence_root("digest-check").join(MANIFEST_REL)).unwrap();
            let mut lines: Vec<&str> = manifest.lines().collect();
            let trailer = lines.pop().unwrap();
            assert!(trailer.starts_with("manifest_body_digest="));
            let claimed = trailer.trim_start_matches("manifest_body_digest=");
            let body_before_trailer = format!("{}\n", lines.join("\n"));
            let actual = sha256_bytes(body_before_trailer.as_bytes());
            assert_eq!(claimed, actual);
            clear_subject_env();
        });
    }

    #[test]
    fn github_output_writes_exact_root_and_walk_attempt() {
        with_temp_cwd(|dir| {
            clear_subject_env();
            let output_path = dir.join("github_output.txt");
            fs::write(&output_path, "").unwrap();
            std::env::set_var("GITHUB_OUTPUT", &output_path);
            fs::write(
                floor_evidence_root("gha-out")
                    .join(PHASE_JOURNAL_REL)
                    .parent()
                    .unwrap()
                    .join("placeholder-mkdir-guard"),
                "",
            )
            .ok();
            set_full_subject("gha-out");
            let _ = finalize_floor_evidence();
            let contents = fs::read_to_string(&output_path).unwrap();
            let expected_root = floor_evidence_root("gha-out");
            assert!(
                contents.contains(&format!("floor_evidence_root={}", expected_root.display())),
                "got: {contents}"
            );
            assert!(contents.contains("walk_attempt=gha-out"), "got: {contents}");
            clear_subject_env();
            std::env::remove_var("GITHUB_OUTPUT");
        });
    }

    #[test]
    fn phase_entered_missing_vs_never_reached() {
        with_temp_cwd(|_| {
            clear_subject_env();
            fs::create_dir_all(floor_evidence_root("ph")).unwrap();
            fs::write(
                floor_evidence_root("ph").join(PHASE_JOURNAL_REL),
                "1\t1\ton-success-stage-2\tstarted\t\n1\t1\tbatch-wall-receipt\tcompleted\t\n",
            )
            .unwrap();
            write_floor_receipt(
                "ph",
                "BatchWall",
                "receipts/batch-wall.txt",
                b"wall\n",
                "DuringFloor",
            )
            .unwrap();
            set_full_subject("ph");
            assert!(finalize_floor_evidence().is_ok());
            let manifest =
                fs::read_to_string(floor_evidence_root("ph").join(MANIFEST_REL)).unwrap();
            assert!(manifest
                .contains("OnSuccessStage2\tstages/2/receipt.tsv\t\tMissingAfterProducerReached"));
            assert!(manifest
                .contains("OnSuccessStage1\tstages/1/receipt.tsv\t\tMissingBeforeProducerReached"));
            clear_subject_env();
        });
    }

    #[test]
    fn phase_entered_missing_receipt_is_after() {
        assert_eq!(
            standing_label_missing(false, PhaseStandingExact::PhaseEntered),
            "MissingAfterProducerReached"
        );
        assert_eq!(
            standing_label_missing(false, PhaseStandingExact::PhaseNotReached),
            "MissingBeforeProducerReached"
        );
        assert_eq!(
            standing_label_missing(false, PhaseStandingExact::PhaseInterrupted),
            "ProducerInterrupted"
        );
    }
}
