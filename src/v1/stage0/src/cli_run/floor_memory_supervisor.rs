//! Supervised memory qualification for a required-floor run.
//!
//! WHY A SUPERVISOR AND NOT AN IN-PROCESS OBSERVER. A `Drop` guard, an exit hook or a final log
//! line covers ordinary returns and unwinding panics and nothing else. None of them runs on
//! `panic = abort`, `process::abort`, `process::exit` or SIGKILL, and Rust's allocation-error
//! handler normally ABORTS rather than unwinds. So allocation failure and the OOM kill — the two
//! terminations a memory instrument most exists to report — are exactly the ones an in-process
//! observer cannot report on. The kernel maintains `memory.peak` and `memory.events` regardless,
//! so a supervisor that OUTLIVED the child can still read them.
//!
//! WHY THE CHILD SHARES THE SUPERVISOR'S CGROUP RATHER THAN GETTING ITS OWN. Two arms were
//! measured on srv1 (2026-09-19) and both fail:
//!   * a systemd-managed unit REAPS its cgroup when the process exits, so `memory.peak` is gone
//!     before it can be read — even with `RemainAfterExit=yes`;
//!   * a self-created cgroup cannot be JOINED by an existing process across a delegation
//!     boundary — writing a pid from a session scope into a cgroup under `user@1000.service` is
//!     EPERM.
//! Running the child inside the cgroup the supervisor is ALREADY in avoids both: nothing has to
//! be moved, and nothing reaps the cgroup because the supervisor is still living in it. The
//! operator supplies the cgroup by wrapping the invocation in `systemd-run --scope`.
//!
//! THE SUPERVISOR'S OWN FOOTPRINT IS INSIDE THE READING, and that is stated rather than netted
//! out. It is a few MiB against a workload measured in tens of GiB; subtracting an estimate would
//! replace a measured number with an adjusted one.
//!
//! WHAT THIS REFUSES TO DO. Invoked where no ancestor cgroup carries a readable `memory.peak`,
//! it REFUSES rather than reporting a figure. A run that was never bounded and a run that fit
//! produce identical output from the workload itself, so the absence of a limit must be reported
//! by the harness or it is not reported at all.

use std::path::{Path, PathBuf};
use std::process::Command;

/// One cgroup's memory interface, read as text so an unparseable body stays distinguishable from
/// an absent file. Authority for the file names: `extdeps.linux.cgroup_v2_memory`
/// `cgroup_memory_interface_file_name`.
#[derive(Debug, Clone)]
pub struct CgroupMemoryRead {
    pub dir: String,
    pub peak: u64,
    pub limit_max: String,
    pub limit_high: String,
    pub high_events: u64,
    pub max_events: u64,
    pub oom_kills: u64,
}

/// WHY A READ CANNOT SILENTLY RETURN ZERO. Each arm names a distinct thing that went wrong, and
/// none of them may render as "it fit". Mirrors `gunbc.floor_memory_demand`
/// `DemandReadRefusalCause`.
#[derive(Debug, Clone)]
pub enum QualificationRefusal {
    NoCgroupWithMemoryPeak { searched_from: String },
    CgroupFileUnreadable { path: String, detail: String },
    CgroupValueUnparseable { file: String, body: String },
    ChildNotSpawned { detail: String },
    ChildNotWaited { detail: String },
}

impl QualificationRefusal {
    pub fn render(&self) -> String {
        match self {
            QualificationRefusal::NoCgroupWithMemoryPeak { searched_from } => format!(
                "NoCgroupWithMemoryPeak — no cgroup from {searched_from} up to the root carries a \
                 readable memory.peak, so this invocation has no enforceable limit to measure \
                 against. Wrap it: systemd-run --user --scope -p MemoryMax=<bytes> \
                 -p MemoryHigh=infinity -- gunbc test //gunbc/instruments:floor-memory-qualification. \
                 Note that MemoryHigh=max is the CGROUP spelling and systemd rejects it; the \
                 systemd spelling is infinity, and a dropped property yields a CONSTRAINED run \
                 reported as uncensored."
            ),
            QualificationRefusal::CgroupFileUnreadable { path, detail } => {
                format!("CgroupFileUnreadable — {path}: {detail}")
            }
            QualificationRefusal::CgroupValueUnparseable { file, body } => {
                format!("CgroupValueUnparseable — {file} held {body:?}")
            }
            QualificationRefusal::ChildNotSpawned { detail } => {
                format!("ChildNotSpawned — the measured process never started: {detail}")
            }
            QualificationRefusal::ChildNotWaited { detail } => format!(
                "ChildNotWaited — the measured process started and its termination was not \
                 observed, so no reading may be published: {detail}"
            ),
        }
    }
}

/// How the measured process ended, observed from OUTSIDE it. Mirrors
/// `gunbc.floor_memory_demand` `SupervisedTermination`.
#[derive(Debug, Clone)]
pub enum SupervisedTermination {
    Exited { code: i32 },
    Signalled { signal: i32 },
}

/// The leaf cgroup of THIS process, from `/proc/self/cgroup`.
fn leaf_cgroup_dir() -> Option<PathBuf> {
    let content = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let rel = content
        .lines()
        .find_map(|l| l.strip_prefix("0::"))
        .map(|p| p.trim().trim_start_matches('/').to_string())?;
    Some(Path::new("/sys/fs/cgroup").join(rel))
}

/// LEAF FIRST, THEN ANCESTORS. The nearest cgroup that actually carries `memory.peak` is the one
/// whose counter describes this process most closely; an ancestor on a shared host aggregates
/// every co-tenant, which is how a charge reading can exceed a scope peak by gigabytes. Measured
/// on srv1: scope peak 26.97 GiB against `user-1000.slice` 249.52 GiB at the same instant.
fn nearest_cgroup_with_peak(leaf: &Path) -> Option<PathBuf> {
    let mut dir = leaf.to_path_buf();
    loop {
        if dir.join("memory.peak").is_file() {
            return Some(dir);
        }
        if dir == Path::new("/sys/fs/cgroup") || !dir.pop() {
            return None;
        }
    }
}

fn read_file(dir: &Path, name: &str) -> Result<String, QualificationRefusal> {
    let path = dir.join(name);
    std::fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| QualificationRefusal::CgroupFileUnreadable {
            path: path.to_string_lossy().to_string(),
            detail: e.to_string(),
        })
}

fn read_count(dir: &Path, name: &str) -> Result<u64, QualificationRefusal> {
    let body = read_file(dir, name)?;
    body.parse::<u64>()
        .map_err(|_| QualificationRefusal::CgroupValueUnparseable {
            file: name.to_string(),
            body,
        })
}

/// `memory.events` is `key value` lines. A MISSING KEY IS NOT A ZERO: the file documents a fixed
/// key set, so a key that is not there means this kernel did not report it, and reporting that as
/// "no events occurred" is the substitution this whole instrument exists to refuse.
fn events_field(body: &str, field: &str) -> Result<u64, QualificationRefusal> {
    body.lines()
        .find_map(|l| {
            let mut it = l.split_whitespace();
            match (it.next(), it.next()) {
                (Some(k), Some(v)) if k == field => Some(v.parse::<u64>().ok()),
                _ => None,
            }
        })
        .flatten()
        .ok_or_else(|| QualificationRefusal::CgroupValueUnparseable {
            file: "memory.events".to_string(),
            body: format!("no parseable {field} key in {body:?}"),
        })
}

pub fn read_cgroup_memory(dir: &Path) -> Result<CgroupMemoryRead, QualificationRefusal> {
    let events_body = read_file(dir, "memory.events")?;
    Ok(CgroupMemoryRead {
        dir: dir.to_string_lossy().to_string(),
        peak: read_count(dir, "memory.peak")?,
        limit_max: read_file(dir, "memory.max")?,
        limit_high: read_file(dir, "memory.high")?,
        high_events: events_field(&events_body, "high")?,
        max_events: events_field(&events_body, "max")?,
        oom_kills: events_field(&events_body, "oom_kill")?,
    })
}

/// Locate the cgroup whose counters this invocation may honestly report, BEFORE the child starts.
/// Refusing here rather than after a 35-minute run is the same discipline the failure-mode row
/// `suppressed_precondition_failure_runs_the_workload_unconstrained` states: verify the
/// environment by readback before spending the workload, never infer it from a call's success.
pub fn resolve_measurement_cgroup() -> Result<PathBuf, QualificationRefusal> {
    let leaf = leaf_cgroup_dir().ok_or_else(|| QualificationRefusal::NoCgroupWithMemoryPeak {
        searched_from: "/proc/self/cgroup".to_string(),
    })?;
    nearest_cgroup_with_peak(&leaf).ok_or_else(|| QualificationRefusal::NoCgroupWithMemoryPeak {
        searched_from: leaf.to_string_lossy().to_string(),
    })
}

/// Run the child to completion IN THIS PROCESS'S OWN CGROUP and report how it ended. No cgroup
/// plumbing happens here on purpose: inheriting the supervisor's cgroup is what makes the
/// post-mortem read possible at all.
pub fn run_child_in_own_cgroup(
    program: &str,
    args: &[String],
) -> Result<SupervisedTermination, QualificationRefusal> {
    use std::os::unix::process::ExitStatusExt;
    let mut child = Command::new(program).args(args).spawn().map_err(|e| {
        QualificationRefusal::ChildNotSpawned {
            detail: format!("{program}: {e}"),
        }
    })?;
    let status = child
        .wait()
        .map_err(|e| QualificationRefusal::ChildNotWaited {
            detail: e.to_string(),
        })?;
    if let Some(signal) = status.signal() {
        return Ok(SupervisedTermination::Signalled { signal });
    }
    Ok(SupervisedTermination::Exited {
        code: status.code().unwrap_or(-1),
    })
}
