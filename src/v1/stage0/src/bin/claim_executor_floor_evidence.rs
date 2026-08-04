//! Attempt-scoped floor-evidence sink (operator ruling on #7785).
//!
//! Seed-retained scaffold: registry rows mirror `gunbc.ci_floor_population_receipt_registry`
//! and MUST stay fingerprint-paired with `ci_floor_population_receipt_registry_fingerprint`.
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
pub const OBSERVATIONS_REL: &str = "observations.tsv";
pub const MANIFEST_REL: &str = "manifest.tsv";
pub const PHASE_JOURNAL_REL: &str = "phase-journal.tsv";

/// Exact pairing with `gunbc.ci_floor_population_receipt_registry`
/// `ci_floor_population_receipt_registry_fingerprint`. A `.dag` registry edit that does not
/// update this const (and the ROWS table) must red the seed-pairing control.
pub const FLOOR_EVIDENCE_REGISTRY_FINGERPRINT: &str = "\
PopulationReceiptManifest|manifest.tsv|AfterFloorAlways|AfterFloorAlways|1\n\
PhaseJournal|phase-journal.tsv|DuringFloor|DuringFloor|1\n\
BatchWall|receipts/batch-wall.txt|DuringFloor|batch-wall-receipt|1\n\
CompileCleanWall|receipts/compile-clean-wall.txt|DuringFloor|compile-clean-wall-receipt|1\n\
Component|receipts/component.json|DuringFloor|floor-component-receipt|1\n\
NativeTransition|receipts/native-transition.tsv|DuringFloor|native-transition-receipt|1\n\
OnSuccessStage1|stages/1/receipt.tsv|OnSuccessStage|on-success-stage-1|1\n\
OnSuccessStage2|stages/2/receipt.tsv|OnSuccessStage|on-success-stage-2|1\n\
OnSuccessMaterialization|receipts/on-success-materialization.txt|OnSuccessStage|on-success-materialization|1\n\
WorkerObservation|workers/observation.tsv|OptionalWorker|coordinator-observation|0\n\
WorkerTerminal|workers/terminal-*.tsv|OptionalWorker|OptionalWorker|0";

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

pub fn floor_evidence_root(attempt: &str) -> PathBuf {
    PathBuf::from(EVIDENCE_ROOT_PREFIX).join(attempt)
}

fn path_is_under_root(root: &Path, candidate: &Path) -> bool {
    let Ok(root) = root
        .canonicalize()
        .or_else(|_| Ok::<_, std::io::Error>(root.to_path_buf()))
    else {
        return false;
    };
    let Ok(cand) = candidate
        .canonicalize()
        .or_else(|_| Ok::<_, std::io::Error>(candidate.to_path_buf()))
    else {
        return false;
    };
    cand.starts_with(&root)
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

/// Atomic write under `target/floor-evidence/<attempt>/` plus observation append.
/// Refuses paths that escape the attempt root.
pub fn write_floor_receipt(
    attempt: &str,
    kind: &str,
    relative_path: &str,
    body: &[u8],
    producer_phase: &str,
) -> Result<PathBuf, String> {
    if relative_path.is_empty()
        || relative_path.starts_with('/')
        || relative_path.contains("..")
        || Path::new(relative_path).is_absolute()
    {
        return Err(format!(
            "write_floor_receipt refused unsafe relative path {relative_path:?}"
        ));
    }
    let root = floor_evidence_root(attempt);
    let path = root.join(relative_path);
    if !path_is_under_root(&root, &path) && path.parent() != Some(root.as_path()) {
        // Before create: ensure joined path stays under root by components.
        let mut ok = true;
        for c in Path::new(relative_path).components() {
            if matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::RootDir
            ) {
                ok = false;
                break;
            }
        }
        if !ok {
            return Err(format!(
                "write_floor_receipt path escapes FloorEvidenceRoot: {relative_path}"
            ));
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&tmp, body).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, &path).map_err(|e| format!("publish {}: {e}", path.display()))?;
    let digest = sha256_bytes(body);
    append_observation(
        attempt,
        kind,
        relative_path,
        body.len() as u64,
        &digest,
        producer_phase,
    )?;
    Ok(path)
}

fn append_observation(
    attempt: &str,
    kind: &str,
    relative_path: &str,
    size: u64,
    digest: &str,
    producer_phase: &str,
) -> Result<(), String> {
    let root = floor_evidence_root(attempt);
    fs::create_dir_all(&root).map_err(|e| format!("mkdir {}: {e}", root.display()))?;
    let obs = root.join(OBSERVATIONS_REL);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&obs)
        .map_err(|e| format!("open {}: {e}", obs.display()))?;
    writeln!(
        file,
        "{kind}\t{relative_path}\t{size}\t{digest}\t{producer_phase}"
    )
    .map_err(|e| format!("append observation: {e}"))?;
    file.sync_data()
        .map_err(|e| format!("sync observation: {e}"))?;
    Ok(())
}

pub fn install_floor_evidence_root(attempt: &str) -> Result<PathBuf, String> {
    let root = floor_evidence_root(attempt);
    fs::create_dir_all(&root).map_err(|e| format!("mkdir {}: {e}", root.display()))?;
    let journal = root.join(PHASE_JOURNAL_REL);
    if let Some(parent) = journal.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    // Touch so the dump step can distinguish absent vs empty when linked.
    if !journal.exists() {
        fs::write(&journal, "").map_err(|e| format!("touch {}: {e}", journal.display()))?;
    }
    std::env::set_var("GUNBC_FLOOR_PHASE_JOURNAL", &journal);
    // Convenience alias for the peak-post dump step (still keyed on the legacy path in
    // v2.workflow.ci_floor_peak_emit). Replaced each install so the alias tracks the
    // current attempt without cross-attempt byte copies into sibling evidence roots.
    let legacy = PathBuf::from("target/floor-phase-journal.tsv");
    if let Some(parent) = legacy.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let _ = fs::remove_file(&legacy);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&journal, &legacy)
            .map_err(|e| format!("symlink {} -> {}: {e}", legacy.display(), journal.display()))?;
    }
    #[cfg(not(unix))]
    {
        fs::copy(&journal, &legacy)
            .map_err(|e| format!("copy journal alias {}: {e}", legacy.display()))?;
    }
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

struct Observation {
    kind: String,
    relative_path: String,
    size: u64,
    digest: String,
}

fn load_observations(root: &Path) -> Result<Vec<Observation>, String> {
    let path = root.join(OBSERVATIONS_REL);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let body = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut out = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            return Err(format!("malformed observation row: {line}"));
        }
        let rel = parts[1].to_string();
        if rel.contains("..") || Path::new(&rel).is_absolute() {
            return Err(format!("observation path escapes root: {rel}"));
        }
        let abs = root.join(&rel);
        if !path_is_under_root(root, &abs) {
            // component check already done; still verify file is under root once created
            let mut ok = true;
            for c in Path::new(&rel).components() {
                if matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::RootDir
                ) {
                    ok = false;
                }
            }
            if !ok {
                return Err(format!("observation path escapes root: {rel}"));
            }
        }
        out.push(Observation {
            kind: parts[0].to_string(),
            relative_path: rel,
            size: parts[2]
                .parse()
                .map_err(|_| format!("bad size in observation: {line}"))?,
            digest: parts[3].to_string(),
        });
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

/// Finalize one attempt root from observation fragments + exact journal phase standing.
/// Refuses incomplete subject (non-zero exit). Never scrapes sibling attempt roots.
pub fn finalize_floor_evidence() -> Result<ExitCode, ExitCode> {
    let computed = fingerprint_from_rows();
    if computed != FLOOR_EVIDENCE_REGISTRY_FINGERPRINT {
        eprintln!(
            "::error::finalize-floor-evidence: seed registry fingerprint drifted from FLOOR_EVIDENCE_REGISTRY_FINGERPRINT"
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

    let observations = match load_observations(&root) {
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
    out.push_str(&format!(
        "registry_fingerprint={FLOOR_EVIDENCE_REGISTRY_FINGERPRINT}\n"
    ));
    out.push_str(&subject_lines);
    out.push_str(
        "kind\texpected_pattern\tobserved_path\tstanding\tproducer_phase\tsize_bytes\tcontent_digest\n",
    );

    let mut row_count = 0usize;
    for row in ROWS {
        if row.kind == "PopulationReceiptManifest" {
            // Written by this finalizer; mark as pending construction then overwrite after.
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
    // Include manifest row after we know bytes — rewrite with ObservedPresent for manifest.
    let mut with_manifest = out.clone();
    // Placeholder: compute digest of body without manifest row, then append manifest row.
    // Simpler: write body, then append manifest self-row by rewriting.
    let body_wo_manifest = out.clone();
    let (size, digest) = {
        let bytes = body_wo_manifest.as_bytes();
        (bytes.len() as u64, sha256_bytes(bytes))
    };
    with_manifest.push_str(&format!(
        "PopulationReceiptManifest\tmanifest.tsv\t{}\tObservedPresent\tAfterFloorAlways\t{}\t{}\n",
        manifest_path.display(),
        size,
        digest
    ));
    // Recompute: the manifest content includes its own row, so digest the final bytes.
    let final_bytes = {
        // Two-pass: write preliminary, then set digest of full content including the row
        // with a zero digest then replace — avoid self-reference by digesting rows excluding
        // the manifest self-row (declared: content_digest covers body before self-row).
        with_manifest.as_bytes().to_vec()
    };
    if let Err(e) = file.write_all(&final_bytes).and_then(|_| file.sync_all()) {
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
        row_count + 1
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
        assert_eq!(fingerprint_from_rows(), FLOOR_EVIDENCE_REGISTRY_FINGERPRINT);
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
            let obs = fs::read_to_string(b_root.join(OBSERVATIONS_REL)).unwrap();
            assert!(!obs.contains("attempt-a"));
            assert!(!obs.contains(a_root.to_string_lossy().as_ref()));
            let a_obs = fs::read_to_string(a_root.join(OBSERVATIONS_REL)).unwrap();
            assert!(a_obs.contains("from-a") || a_obs.contains("BatchWall"));
        });
    }

    #[test]
    fn two_worker_terminals_two_observation_rows() {
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
            let obs = load_observations(&floor_evidence_root("w")).unwrap();
            let terminals: Vec<_> = obs.iter().filter(|o| o.kind == "WorkerTerminal").collect();
            assert_eq!(terminals.len(), 2);
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
            clear_subject_env();
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
