//! Seed trampoline for `gunbc.compute.unit_bounds` `gunbc_user_work_scope_command`.
//! Prefix argv: `user_work_scope_wrap_prefix_argv`. Standing: `user_work_scope_standing` /
//! `user_work_scope_standing_effect`. Seed cannot eval those folds at wrap time; the join is
//! `the_seed_trampoline_mirrors_the_declared_wrap_prefix` and
//! `the_seed_trampoline_mirrors_the_declared_wrap_standing`.
//! Retention: `v2.compiler.self_host.seed_retention_frontier` `retained_support_dir_member`
//! (`src/cli_run/user_work_scope.rs`); trigger: Test/Run wrap is emitted, this file deletes.

use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

// SEED MIRROR of `gunbc.live_deploy.slice_bounds user_work_slice_unit_name`
const USER_WORK_SLICE: &str = "gunbc-user-work.slice";
// SEED MIRROR of `gunbc.live_deploy.slice_bounds user_work_slice_memory_swap_max`
const USER_WORK_SLICE_MEMORY_SWAP_MAX_BYTES: u64 = 0;
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_ci_env_name`
const USER_WORK_SCOPE_CI_ENV: &str = "GITHUB_ACTIONS";
// SEED MIRROR of `gunbc.runner.runner_unit runner_unit_prefix`
const RUNNER_SLOT_CGROUP_NEEDLE: &str = "actions-runner@";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_xdg_runtime_dir_env_name`
const USER_WORK_SCOPE_XDG_RUNTIME_DIR_ENV: &str = "XDG_RUNTIME_DIR";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_systemd_manager_dir`
const SYSTEMD_MANAGER_DIR: &str = "/run/systemd/system";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_buildbuddy_run_id_env_name`
const BUILDBUDDY_RUN_ID_ENV: &str = "BUILDBUDDY_RUN_ID";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_user_slice_cgroup_needle`
const USER_SLICE_CGROUP_NEEDLE: &str = "/user.slice/";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_service_unit_suffix`
const SERVICE_UNIT_SUFFIX: &str = ".service";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_slice_unit_suffix`
const SLICE_UNIT_SUFFIX: &str = ".slice";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_scope_unit_suffix`
const SCOPE_UNIT_SUFFIX: &str = ".scope";
// SEED MIRROR of `gunbc.compute.unit_bounds recognized_system_slice`
// (declared_system_slice_memory_maxes unit names with memory_max > 0, plus
// `gunbc.oomd_install` `gunbc_oomd_sessions_slice`). Wrap cannot eval the DAG;
// this list is seed-retained with user_work_scope.rs. A census name missing here
// is a RED in `the_seed_trampoline_mirrors_the_declared_wrap_standing`.
const RECOGNIZED_SYSTEM_SLICES: &[&str] = &[
    "gunbc-repository-convergence.slice",
    "gunbc-approval-broker.slice",
    "gunbc-attempt-worker.slice",
    "gunbc-fci1-controls.slice",
    "sessions.slice",
];

// SEED MIRROR of `gunbc.compute.unit_bounds WrapExemptionPremise`: WHICH claim asked to be spared the
// wrap. A premise never skips by itself -- `live_enclosing_bound` below is what decides -- so no arm
// here may be reached by a path substring, an env var, or a declared cap alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WrapExemptionPremise {
    InCiRunnerEnvironment,
    InRunnerSlotCgroup,
    AlreadyInDeclaredUserWorkSlice,
    UnderModeledSlice { unit: String, slice: String },
    UnderUnmodeledSystemUnit { unit: String, slice: String },
}

// SEED MIRROR of `gunbc.compute.unit_bounds EnclosingBoundCause`: why no live finite enclosing
// memory.max was observed, as a shape rather than prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EnclosingBoundCause {
    ChainUnreadable { detail: String },
    NoFiniteMemoryMax,
    MemoryMaxZero { dir: String },
    SwapNotForbidden { dir: String, bytes: u64 },
}

// SEED MIRROR of `gunbc.compute.unit_bounds EnclosingBoundObservation`: the ONE answer of the ONE
// check every bound-premised exemption goes through.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EnclosingBoundObservation {
    LiveEnclosingBound {
        cgroup: String,
        memory_max: u64,
    },
    NoLiveEnclosingBound {
        cgroup: String,
        cause: EnclosingBoundCause,
    },
}

pub(crate) enum UserWorkScopeStanding {
    // THE ONE SKIP ARM: an exempt premise (or none) plus the observed live finite enclosing
    // memory.max that earned the skip. `memory_max` is the evidence; the premise only says who asked.
    ExemptUnderObservedBound {
        premise: Option<WrapExemptionPremise>,
        cgroup: String,
        memory_max: u64,
    },
    // THE ONE REFUSAL for a premise that claimed an enclosing bound and could not show one, located:
    // the cgroup consulted and the shape of what its memory.max said (unreadable / 0 / max / swap
    // still allowed). This is `UncappedSystemSlice` renamed and generalized -- the same cause, no
    // parallel one minted -- so the slice arm, the CI arm, the slot arm and the unit arm all refuse
    // through it (srv1 cap deploy run 38865847078: CiExempt skipped, then HostBudgetUnreadable).
    UncappedEnclosingCgroup {
        premise: WrapExemptionPremise,
        cgroup: String,
        cause: EnclosingBoundCause,
    },
    // NOT bound-premised, and deliberately outside `live_enclosing_bound`: the seed's own
    // #[cfg(test)] cargo-test binary claims no enclosing bound, only that the process is a test
    // harness (the cfg(test) gate is the check). Wrapping it would need a user manager cargo test
    // does not have, and would cap the harness rather than the work.
    UnitTestExempt,
    UserSessionAbsent,
    UserManagerOwed,
    CgroupUnreadable,
    UserManagerProbeFailed,
    UserWorkSliceAbsent,
    UserWorkSliceUncapped,
    CallerEnvelopeUnreadable,
    ExemptBoundedContainer {
        bytes: u64,
    },
    BoundedContainerUncapped,
    BoundedContainerUnreadable,
    ExternallyBoundedExecutor {
        run_id: String,
    },
    // DAG: gunbc.compute.unit_bounds OperatorHostWithoutCgroups (user_work_host_facility_standing).
    OperatorHostWithoutCgroups,
}

pub fn reexec_into_user_work_slice_if_owed() {
    let standing = user_work_scope_standing();
    match standing {
        UserWorkScopeStanding::UserManagerOwed => match standing_for_wrap() {
            UserWorkScopeStanding::UserManagerOwed => wrap_now(),
            gated => {
                eprintln!(
                    "error: cannot wrap into {USER_WORK_SLICE} ({})",
                    standing_wire(&gated)
                );
                std::process::exit(2);
            }
        },
        UserWorkScopeStanding::CgroupUnreadable
        | UserWorkScopeStanding::UserManagerProbeFailed
        | UserWorkScopeStanding::UserSessionAbsent
        | UserWorkScopeStanding::UncappedEnclosingCgroup { .. }
        | UserWorkScopeStanding::UserWorkSliceAbsent
        | UserWorkScopeStanding::UserWorkSliceUncapped
        | UserWorkScopeStanding::CallerEnvelopeUnreadable
        | UserWorkScopeStanding::BoundedContainerUncapped
        | UserWorkScopeStanding::BoundedContainerUnreadable => {
            eprintln!(
                "error: cannot wrap into {USER_WORK_SLICE} ({})",
                standing_wire(&standing)
            );
            std::process::exit(2);
        }
        UserWorkScopeStanding::UnitTestExempt => {}
        standing => {
            eprintln!(
                "user_work_scope: skip wrap into {USER_WORK_SLICE} ({})",
                standing_wire(&standing)
            );
        }
    }
}

// SEED MIRROR of `gunbc.compute.unit_bounds optional_premise_wire` / `wrap_exemption_premise_wire`.
fn premise_wire(premise: Option<&WrapExemptionPremise>) -> String {
    match premise {
        Some(WrapExemptionPremise::InCiRunnerEnvironment) => "ci-runner-environment".to_string(),
        Some(WrapExemptionPremise::InRunnerSlotCgroup) => "runner-slot-cgroup".to_string(),
        Some(WrapExemptionPremise::AlreadyInDeclaredUserWorkSlice) => {
            "already-in-user-work-slice".to_string()
        }
        Some(WrapExemptionPremise::UnderModeledSlice { unit, slice }) => {
            format!("modeled-slice:{slice}:{unit}")
        }
        Some(WrapExemptionPremise::UnderUnmodeledSystemUnit { unit, slice }) => {
            format!("unmodeled-system-unit:{slice}:{unit}")
        }
        None => "no-premise".to_string(),
    }
}

// SEED MIRROR of `gunbc.compute.unit_bounds enclosing_bound_cause_text`.
fn enclosing_bound_cause_text(cause: &EnclosingBoundCause) -> String {
    match cause {
        EnclosingBoundCause::ChainUnreadable { detail } => format!(
            "memory.max unreadable: {detail}; no level of the chain yielded a reading"
        ),
        EnclosingBoundCause::NoFiniteMemoryMax => {
            "no finite memory.max: every memory.max on the chain reads max".to_string()
        }
        EnclosingBoundCause::MemoryMaxZero { dir } => {
            format!("memory.max is 0 at {dir}, which systemd reads as no limit")
        }
        EnclosingBoundCause::SwapNotForbidden { dir, bytes } => format!(
            "tightest memory.max {bytes} at {dir} is not binding: no level on the chain sets memory.swap.max to 0"
        ),
    }
}

pub(crate) fn standing_wire(s: &UserWorkScopeStanding) -> String {
    match s {
        // SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_standing_wire`
        // One line for both verdicts, and it always names the CGROUP consulted and what its memory.max
        // said -- the srv1 next run reads one line and tells us which cgroup the job actually sits in
        // (plan run 38967145061), rather than the standing name alone.
        UserWorkScopeStanding::ExemptUnderObservedBound {
            premise,
            cgroup,
            memory_max,
        } => format!(
            "ExemptUnderObservedBound:{}:{cgroup}:{memory_max}",
            premise_wire(premise.as_ref())
        ),
        UserWorkScopeStanding::UncappedEnclosingCgroup {
            premise,
            cgroup,
            cause,
        } => format!(
            "UncappedEnclosingCgroup:{} @ {cgroup}: {}",
            premise_wire(Some(premise)),
            enclosing_bound_cause_text(cause)
        ),
        UserWorkScopeStanding::UnitTestExempt => "UnitTestExempt".to_string(),
        UserWorkScopeStanding::UserSessionAbsent => "UserSessionAbsent".to_string(),
        UserWorkScopeStanding::UserManagerOwed => "UserManagerOwed".to_string(),
        UserWorkScopeStanding::CgroupUnreadable => "CgroupUnreadable".to_string(),
        UserWorkScopeStanding::UserManagerProbeFailed => "UserManagerProbeFailed".to_string(),
        UserWorkScopeStanding::UserWorkSliceAbsent => "UserWorkSliceAbsent".to_string(),
        UserWorkScopeStanding::UserWorkSliceUncapped => "UserWorkSliceUncapped".to_string(),
        UserWorkScopeStanding::CallerEnvelopeUnreadable => "CallerEnvelopeUnreadable".to_string(),
        UserWorkScopeStanding::ExemptBoundedContainer { bytes } => {
            format!("ExemptBoundedContainer:{bytes}")
        }
        UserWorkScopeStanding::BoundedContainerUncapped => "BoundedContainerUncapped".to_string(),
        UserWorkScopeStanding::BoundedContainerUnreadable => {
            "BoundedContainerUnreadable".to_string()
        }
        UserWorkScopeStanding::ExternallyBoundedExecutor { run_id } => {
            format!("ExternallyBoundedExecutor:{run_id}")
        }
        UserWorkScopeStanding::OperatorHostWithoutCgroups => {
            "OperatorHostWithoutCgroups:no memory bound on this host".to_string()
        }
    }
}

#[derive(Clone, Copy)]
enum MemoryBound {
    Capped(u64),
    Unbounded,
}

struct SliceInstall {
    memory_max: u64,
    memory_high: MemoryBound,
}

enum SliceInstallRead {
    NotInstalled,
    Uncapped,
    Installed(SliceInstall),
}

struct CallerEnvelope {
    memory_max: MemoryBound,
    memory_high: MemoryBound,
}

fn parse_memory_bound(raw: &str) -> Option<MemoryBound> {
    let t = raw.trim();
    if t.is_empty() {
        return Some(MemoryBound::Unbounded);
    }
    if t == "max" || t == "infinity" || t == "Infinity" || t == "-1" {
        return Some(MemoryBound::Unbounded);
    }
    t.parse::<u64>().ok().map(MemoryBound::Capped)
}

fn tighter_bound(a: MemoryBound, b: MemoryBound) -> MemoryBound {
    match (a, b) {
        (MemoryBound::Unbounded, x) | (x, MemoryBound::Unbounded) => x,
        (MemoryBound::Capped(aa), MemoryBound::Capped(bb)) => MemoryBound::Capped(aa.min(bb)),
    }
}

fn systemctl_user_show(unit: &str, prop: &str) -> Option<String> {
    let out = Command::new("systemctl")
        .args(["--user", "show", "--value", "-p", prop, unit])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn read_slice_install() -> SliceInstallRead {
    let load = match systemctl_user_show(USER_WORK_SLICE, "LoadState") {
        Some(s) => s,
        None => return SliceInstallRead::NotInstalled,
    };
    if load == "not-found" || load == "stub" {
        return SliceInstallRead::NotInstalled;
    }
    match systemctl_user_show(USER_WORK_SLICE, "MemoryMax").and_then(|s| parse_memory_bound(&s)) {
        Some(MemoryBound::Capped(max)) if max > 0 => {
            match systemctl_user_show(USER_WORK_SLICE, "MemoryHigh")
                .and_then(|s| parse_memory_bound(&s))
            {
                Some(high) => SliceInstallRead::Installed(SliceInstall {
                    memory_max: max,
                    memory_high: high,
                }),
                None => SliceInstallRead::Uncapped,
            }
        }
        Some(MemoryBound::Capped(_)) | Some(MemoryBound::Unbounded) => SliceInstallRead::Uncapped,
        None => SliceInstallRead::NotInstalled,
    }
}

fn cgroup_v2_rel_path(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("0::") {
            return Some(rest.to_string());
        }
    }
    None
}

fn read_cgroup_memory_file(rel: &str, name: &str) -> Option<MemoryBound> {
    let path = format!("/sys/fs/cgroup{rel}/{name}");
    let raw = std::fs::read_to_string(&path).ok()?;
    parse_memory_bound(&raw)
}

fn read_caller_envelope() -> Option<CallerEnvelope> {
    let cg = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let rel = cgroup_v2_rel_path(&cg)?;
    let memory_max = read_cgroup_memory_file(&rel, "memory.max")?;
    let memory_high = read_cgroup_memory_file(&rel, "memory.high")?;
    Some(CallerEnvelope {
        memory_max,
        memory_high,
    })
}

// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_standing_for_wrap`
fn standing_for_wrap() -> UserWorkScopeStanding {
    match read_slice_install() {
        SliceInstallRead::NotInstalled => UserWorkScopeStanding::UserWorkSliceAbsent,
        SliceInstallRead::Uncapped => UserWorkScopeStanding::UserWorkSliceUncapped,
        SliceInstallRead::Installed(install) => match read_caller_envelope() {
            None => UserWorkScopeStanding::CallerEnvelopeUnreadable,
            Some(_) => {
                if install.memory_max == 0 {
                    UserWorkScopeStanding::UserWorkSliceUncapped
                } else {
                    UserWorkScopeStanding::UserManagerOwed
                }
            }
        },
    }
}

fn wrap_envelope_args(
    install: &SliceInstall,
    caller: &CallerEnvelope,
) -> Option<(u64, Option<u64>)> {
    let max = match tighter_bound(caller.memory_max, MemoryBound::Capped(install.memory_max)) {
        MemoryBound::Unbounded => return None,
        MemoryBound::Capped(0) => return None,
        MemoryBound::Capped(n) => n,
    };
    let high = match tighter_bound(caller.memory_high, install.memory_high) {
        MemoryBound::Unbounded => None,
        MemoryBound::Capped(h) => Some(h.min(max)),
    };
    Some((max, high))
}

fn wrap_now() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: current_exe unreadable; cannot wrap into {USER_WORK_SLICE}: {e}");
            std::process::exit(2);
        }
    };
    let (memory_max, memory_high) = match (read_slice_install(), read_caller_envelope()) {
        (SliceInstallRead::Installed(install), Some(caller)) => {
            match wrap_envelope_args(&install, &caller) {
                Some(pair) => pair,
                None => {
                    eprintln!(
                        "error: cannot wrap into {USER_WORK_SLICE} ({})",
                        standing_wire(&UserWorkScopeStanding::UserWorkSliceUncapped)
                    );
                    std::process::exit(2);
                }
            }
        }
        _ => {
            eprintln!(
                "error: cannot wrap into {USER_WORK_SLICE} ({})",
                standing_wire(&UserWorkScopeStanding::CallerEnvelopeUnreadable)
            );
            std::process::exit(2);
        }
    };
    let mut cmd = Command::new("systemd-run");
    cmd.args([
        "--user",
        "--scope",
        &format!("--property=Slice={USER_WORK_SLICE}"),
        &format!("--property=MemorySwapMax={USER_WORK_SLICE_MEMORY_SWAP_MAX_BYTES}"),
        &format!("--property=MemoryMax={memory_max}"),
    ]);
    if let Some(high) = memory_high {
        cmd.arg(format!("--property=MemoryHigh={high}"));
    }
    cmd.args(["--"]);
    cmd.arg(&exe);
    cmd.args(std::env::args().skip(1));
    let err = cmd.exec();
    eprintln!("error: systemd-run exec into {USER_WORK_SLICE} failed: {err}");
    std::process::exit(1);
}

// SEED MIRROR of `gunbc.compute.unit_bounds last_cgroup_unit_with_dot_suffix`
fn last_cgroup_unit_with_suffix(text: &str, suffix: &str) -> Option<String> {
    text.split(['/', '\n'])
        .map(str::trim)
        .rfind(|seg| {
            if seg.is_empty() {
                return false;
            }
            let mut parts = seg.rsplitn(2, '.');
            let last = parts.next().unwrap_or("");
            parts.next().is_some() && format!(".{last}") == suffix
        })
        .map(str::to_string)
}

enum RootMemoryRead {
    Capped(u64),
    Unbounded,
    Unreadable,
}

fn root_memory_from_file(bound: Option<MemoryBound>) -> RootMemoryRead {
    match bound {
        Some(MemoryBound::Capped(n)) if n > 0 => RootMemoryRead::Capped(n),
        Some(MemoryBound::Capped(_)) | Some(MemoryBound::Unbounded) => RootMemoryRead::Unbounded,
        None => RootMemoryRead::Unreadable,
    }
}

// SEED MIRROR of `gunbc.compute.unit_bounds innermost_modeled_capped_slice`
fn innermost_modeled_capped_slice(text: &str) -> Option<String> {
    text.split(['/', '\n'])
        .map(str::trim)
        .rfind(|seg| RECOGNIZED_SYSTEM_SLICES.contains(seg))
        .map(str::to_string)
}

// SEED MIRROR of `gunbc.compute.unit_bounds system_service_unit_from_cgroup`
fn system_service_unit_from_cgroup(text: &str) -> Option<(String, String)> {
    if text.contains(USER_SLICE_CGROUP_NEEDLE) {
        return None;
    }
    let unit = last_cgroup_unit_with_suffix(text, SERVICE_UNIT_SUFFIX)?;
    let slice = last_cgroup_unit_with_suffix(text, SLICE_UNIT_SUFFIX).unwrap_or_default();
    Some((unit, slice))
}

fn namespaced_root_standing(
    root_memory: RootMemoryRead,
    buildbuddy_run_id: Option<&str>,
) -> UserWorkScopeStanding {
    // SEED MIRROR of `gunbc.compute.unit_bounds namespaced_root_standing`
    match root_memory {
        RootMemoryRead::Capped(n) if n > 0 => {
            UserWorkScopeStanding::ExemptBoundedContainer { bytes: n }
        }
        RootMemoryRead::Capped(_) | RootMemoryRead::Unbounded => {
            UserWorkScopeStanding::BoundedContainerUncapped
        }
        RootMemoryRead::Unreadable => match buildbuddy_run_id {
            Some(id) if !id.is_empty() => UserWorkScopeStanding::ExternallyBoundedExecutor {
                // DAG: ExternallyBoundedExecutor.run_id is NonEmptyStr; empty is unwritable there.
                run_id: id.to_string(),
            },
            _ => UserWorkScopeStanding::BoundedContainerUnreadable,
        },
    }
}

fn cgroup_text_is_root(text: &str) -> bool {
    matches!(cgroup_v2_rel_path(text).as_deref(), Some("/") | Some(""))
}

// SEED MIRROR of `gunbc.compute.unit_bounds wrap_exemption_premise`
// Classifies WHICH claim asked to be spared the wrap; it never discharges one. The order is the
// declared order (slice, slot, CI env, modeled slice, unmodeled system unit).
fn wrap_exemption_premise(github_actions: bool, text: &str) -> Option<WrapExemptionPremise> {
    if text.contains(USER_WORK_SLICE) {
        return Some(WrapExemptionPremise::AlreadyInDeclaredUserWorkSlice);
    }
    if text.contains(RUNNER_SLOT_CGROUP_NEEDLE) {
        return Some(WrapExemptionPremise::InRunnerSlotCgroup);
    }
    if github_actions {
        return Some(WrapExemptionPremise::InCiRunnerEnvironment);
    }
    if let Some(slice) = innermost_modeled_capped_slice(text) {
        let unit = last_cgroup_unit_with_suffix(text, SCOPE_UNIT_SUFFIX)
            .or_else(|| last_cgroup_unit_with_suffix(text, SERVICE_UNIT_SUFFIX))
            .unwrap_or_default();
        return Some(WrapExemptionPremise::UnderModeledSlice { unit, slice });
    }
    if let Some((unit, slice)) = system_service_unit_from_cgroup(text) {
        return if RECOGNIZED_SYSTEM_SLICES.contains(&slice.as_str()) {
            Some(WrapExemptionPremise::UnderModeledSlice { unit, slice })
        } else {
            Some(WrapExemptionPremise::UnderUnmodeledSystemUnit { unit, slice })
        };
    }
    None
}

// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_standing`
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_standing_with_root_memory`
// A premise never skips on its own: `live_enclosing_bound` (below, via the chain reader) is called
// exactly once, here, and only a live finite enclosing memory.max skips -- for the CI arm, the slot
// arm, the slice arm and the unit arm alike. No arm has a private fast path.
#[allow(clippy::too_many_arguments)]
fn standing_from_supplied(
    github_actions: bool,
    cgroup: Option<&str>,
    systemd_manager_present: bool,
    user_manager_reachable: bool,
    xdg_runtime_dir: bool,
    root_memory: RootMemoryRead,
    buildbuddy_run_id: Option<&str>,
    bound: EnclosingBoundObservation,
) -> UserWorkScopeStanding {
    if cgroup.is_some_and(cgroup_text_is_root) && !systemd_manager_present {
        return namespaced_root_standing(root_memory, buildbuddy_run_id);
    }
    let Some(cg) = cgroup else {
        return UserWorkScopeStanding::CgroupUnreadable;
    };
    let premise = wrap_exemption_premise(github_actions, cg);
    match bound {
        // ONE skip for every premise: the bound admits, the premise only says who asked.
        EnclosingBoundObservation::LiveEnclosingBound { cgroup, memory_max } => {
            UserWorkScopeStanding::ExemptUnderObservedBound {
                premise,
                cgroup,
                memory_max,
            }
        }
        // No bound observed: a premise that claimed one refuses, located. With no premise the process
        // is ordinary user work and the user-manager arms decide.
        EnclosingBoundObservation::NoLiveEnclosingBound { cgroup, cause } => match premise {
            Some(premise) => UserWorkScopeStanding::UncappedEnclosingCgroup {
                premise,
                cgroup,
                cause,
            },
            None => {
                eprintln!(
                    "user_work_scope: observed-bound arm did not admit ({})",
                    cgroup_bound_fallthrough_wire(
                        &EnclosingBoundObservation::NoLiveEnclosingBound { cgroup, cause }
                    )
                );
                if user_manager_reachable {
                    UserWorkScopeStanding::UserManagerOwed
                } else if xdg_runtime_dir {
                    UserWorkScopeStanding::UserManagerProbeFailed
                } else {
                    UserWorkScopeStanding::UserSessionAbsent
                }
            }
        },
    }
}

// SEED MIRROR of `gunbc.compute.unit_bounds live_enclosing_bound`
// THE ONE CHECK, SEED SIDE. `gunbc.compute.unit_bounds` declares it as `live_enclosing_bound` over a
// supplied `CgroupBoundObservation` (the chain's memory.max / memory.swap.max levels, read through
// `extdeps.linux.cgroup_v2_memory`); the trampoline cannot evaluate that fold before the corpus is
// resolved, so it performs the walk itself and answers the SAME question: is there a live finite
// enclosing memory.max on the process's cgroup chain, and if not, what exactly said so. Bounded when
// some level carries a nonzero finite memory.max (decimal, within u64 -- `parse_cgroup_memory_limit`'s
// "Limited", the tightest wins because any ancestor's limit binds) AND some level sets memory.swap.max
// to 0 (a swap ceiling of 0 anywhere forbids swap below it). The negatives are the check's typed
// causes, naming the cgroup consulted: 0 is not a bound (systemd reads it as no limit), `max`/garbage
// everywhere is no finite max, a limit with swap still allowed is not binding, and a chain that yielded
// no reading at all is unread.
// RETAINED, with this file: the reason is that the trampoline must decide before the corpus is
// resolved; the migration trigger is the file's own (Test/Run wrap emitted natively, this file
// deletes). Executed by the unit tests below over fixture trees, not only by the `.dag` witness's
// text join.
fn live_enclosing_bound_in(
    proc_self_cgroup: &str,
    sysfs_root: &std::path::Path,
) -> EnclosingBoundObservation {
    let Some(path) = proc_self_cgroup.lines().find_map(|l| l.strip_prefix("0::")) else {
        return no_live_enclosing_bound(
            enclosing_cgroup_label(proc_self_cgroup),
            EnclosingBoundCause::ChainUnreadable {
                detail: "no unified-hierarchy (0::) line in /proc/self/cgroup".to_string(),
            },
        );
    };
    let label = enclosing_cgroup_label(proc_self_cgroup);
    let mut any_read = false;
    let mut tightest: Option<(String, u64)> = None;
    let mut swap_forbidden = false;
    let mut current = path.trim().trim_end_matches('/').to_string();
    while !current.is_empty() {
        let dir = format!("{}{current}", sysfs_root.display());
        if let Ok(max) = std::fs::read_to_string(format!("{dir}/memory.max")) {
            any_read = true;
            // A non-numeric reading (`max`, or out of u64 range) is not a limit and is skipped; 0
            // PARSES and so stays the tightest, which is how the check gets to name it MemoryMaxZero
            // rather than treating it as no limit.
            if let Ok(n) = max.trim().parse::<u64>() {
                match &tightest {
                    Some((_, best)) if *best <= n => {}
                    _ => tightest = Some((dir.clone(), n)),
                }
            }
        }
        if let Ok(swap) = std::fs::read_to_string(format!("{dir}/memory.swap.max")) {
            any_read = true;
            if swap.trim().parse::<u64>() == Ok(0) {
                swap_forbidden = true;
            }
        }
        match current.rfind('/') {
            Some(i) => current.truncate(i),
            None => current.clear(),
        }
    }
    if !any_read {
        let detail =
            format!("no memory.max or memory.swap.max readable at {label} or any ancestor of it");
        return no_live_enclosing_bound(label, EnclosingBoundCause::ChainUnreadable { detail });
    }
    match tightest {
        None => no_live_enclosing_bound(label, EnclosingBoundCause::NoFiniteMemoryMax),
        Some((dir, 0)) => {
            no_live_enclosing_bound(dir.clone(), EnclosingBoundCause::MemoryMaxZero { dir })
        }
        Some((dir, bytes)) if !swap_forbidden => no_live_enclosing_bound(
            dir.clone(),
            EnclosingBoundCause::SwapNotForbidden { dir, bytes },
        ),
        Some((dir, bytes)) => EnclosingBoundObservation::LiveEnclosingBound {
            cgroup: dir,
            memory_max: bytes,
        },
    }
}

fn no_live_enclosing_bound(
    cgroup: String,
    cause: EnclosingBoundCause,
) -> EnclosingBoundObservation {
    EnclosingBoundObservation::NoLiveEnclosingBound { cgroup, cause }
}

// THE CGROUP A REFUSAL NAMES WHEN NO LEVEL COULD BE READ AT ALL: the path read from
// /proc/self/cgroup, flattened to one line.
fn enclosing_cgroup_label(text: &str) -> String {
    text.replace('\n', " ").trim().to_string()
}

// SEED MIRROR of `gunbc.compute.unit_bounds cgroup_bound_fallthrough_wire`
fn cgroup_bound_fallthrough_wire(b: &EnclosingBoundObservation) -> &'static str {
    match b {
        EnclosingBoundObservation::LiveEnclosingBound { .. } => "chain-bounded",
        EnclosingBoundObservation::NoLiveEnclosingBound {
            cause: EnclosingBoundCause::ChainUnreadable { .. },
            ..
        } => "chain-unread",
        EnclosingBoundObservation::NoLiveEnclosingBound { .. } => "chain-unbounded",
    }
}

fn live_enclosing_bound(proc_self_cgroup: &str) -> EnclosingBoundObservation {
    live_enclosing_bound_in(proc_self_cgroup, std::path::Path::new("/sys/fs/cgroup"))
}

#[cfg(test)]
pub(crate) mod observed_cgroup_bound_tests {
    use super::{live_enclosing_bound_in, EnclosingBoundCause, EnclosingBoundObservation};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    // Plain functions rather than a struct with impls: an impl method is not citable as a
    // DeclarationRef, and this module's items are enumerated in
    // gunbc.user_work_observed_bound_seed_growth.
    pub(crate) fn fixture_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gunbc-cgroup-bound-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).expect("fixture root");
        dir
    }

    pub(crate) fn tree(levels: &[(&str, &str, &str)]) -> PathBuf {
        let root = fixture_root();
        for (path, max, swap) in levels {
            let dir = root.join(path.trim_start_matches('/'));
            std::fs::create_dir_all(&dir).expect("mkdir");
            std::fs::write(dir.join("memory.max"), format!("{max}\n")).expect("max");
            std::fs::write(dir.join("memory.swap.max"), format!("{swap}\n")).expect("swap");
        }
        root
    }

    // Reads the verdict and removes the fixture, so each tree is read exactly once. The verdict is
    // the ONE check's answer, named -- which shape fired, and the CGROUP that answered it (fixture
    // root stripped, so the assertion reads as the cgroup path a host refusal would print).
    fn verdict(root: PathBuf, leaf: &str) -> String {
        let prefix = root.display().to_string();
        let strip = |p: &str| p.trim_start_matches(prefix.as_str()).to_string();
        let v = match live_enclosing_bound_in(&format!("0::{leaf}\n"), &root) {
            EnclosingBoundObservation::LiveEnclosingBound { cgroup, memory_max } => {
                format!("bounded@{}:{memory_max}", strip(&cgroup))
            }
            EnclosingBoundObservation::NoLiveEnclosingBound { cgroup, cause } => match cause {
                EnclosingBoundCause::ChainUnreadable { .. } => format!("unread@{cgroup}"),
                EnclosingBoundCause::NoFiniteMemoryMax => {
                    format!("no-finite-max@{}", strip(&cgroup))
                }
                EnclosingBoundCause::MemoryMaxZero { dir } => format!("zero@{}", strip(&dir)),
                EnclosingBoundCause::SwapNotForbidden { dir, bytes } => {
                    format!("swap-allowed@{}:{bytes}", strip(&dir))
                }
            },
        };
        let _ = std::fs::remove_dir_all(&root);
        v
    }
    #[test]
    fn a_finite_max_with_no_swap_on_the_unit_bounds_it() {
        let t = tree(&[
            ("/system.slice/job.service", "21474836480", "0"),
            ("/system.slice", "max", "max"),
        ]);
        assert_eq!(
            verdict(t, "/system.slice/job.service"),
            "bounded@/system.slice/job.service:21474836480"
        );
    }

    #[test]
    fn a_limit_on_one_level_and_no_swap_on_another_bounds_the_leaf() {
        let t = tree(&[
            ("/system.slice/job.service", "max", "0"),
            ("/system.slice", "8589934592", "max"),
        ]);
        assert_eq!(
            verdict(t, "/system.slice/job.service"),
            "bounded@/system.slice:8589934592"
        );
    }

    #[test]
    fn a_bound_on_an_ancestor_bounds_the_leaf() {
        let t = tree(&[
            ("/system.slice/job.service", "max", "max"),
            ("/system.slice", "8589934592", "0"),
        ]);
        assert_eq!(
            verdict(t, "/system.slice/job.service"),
            "bounded@/system.slice:8589934592"
        );
    }

    #[test]
    fn unlimited_max_or_allowed_swap_or_out_of_range_never_bounds() {
        // None of these is a live finite bound, and each names its own shape -- the refusal is the
        // same for all of them, the located cause is not.
        let unlimited = tree(&[("/system.slice/job.service", "max", "0")]);
        assert_eq!(
            verdict(unlimited, "/system.slice/job.service"),
            "no-finite-max@0::/system.slice/job.service"
        );
        let swap = tree(&[("/system.slice/job.service", "21474836480", "max")]);
        assert_eq!(
            verdict(swap, "/system.slice/job.service"),
            "swap-allowed@/system.slice/job.service:21474836480"
        );
        let huge = tree(&[("/system.slice/job.service", "99999999999999999999999", "0")]);
        assert_eq!(
            verdict(huge, "/system.slice/job.service"),
            "no-finite-max@0::/system.slice/job.service"
        );
        let zero = tree(&[("/system.slice/job.service", "0", "0")]);
        assert_eq!(
            verdict(zero, "/system.slice/job.service"),
            "zero@/system.slice/job.service"
        );
        let zero_ancestor = tree(&[
            ("/system.slice/job.service", "21474836480", "0"),
            ("/system.slice", "0", "max"),
        ]);
        assert_eq!(
            verdict(zero_ancestor, "/system.slice/job.service"),
            "zero@/system.slice"
        );
    }

    #[test]
    fn an_unreadable_chain_or_missing_line_is_unread_not_bounded() {
        let empty = fixture_root();
        assert!(matches!(
            live_enclosing_bound_in("1:name=systemd:/x\n", &empty),
            EnclosingBoundObservation::NoLiveEnclosingBound {
                cause: EnclosingBoundCause::ChainUnreadable { .. },
                ..
            }
        ));
        let _ = std::fs::remove_dir_all(&empty);
        assert_eq!(
            verdict(fixture_root(), "/system.slice/job.service"),
            "unread@0::/system.slice/job.service"
        );
    }
}

fn user_manager_reachable() -> bool {
    Command::new("systemctl")
        .args(["--user", "show-environment"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub(crate) fn buildbuddy_run_id_from_env() -> Option<String> {
    std::env::var(BUILDBUDDY_RUN_ID_ENV)
        .ok()
        .filter(|id| !id.is_empty())
}

pub(crate) fn user_work_scope_standing() -> UserWorkScopeStanding {
    if cfg!(test) {
        return UserWorkScopeStanding::UnitTestExempt;
    }
    let cg_read = std::fs::read_to_string("/proc/self/cgroup");
    let cg = cg_read.as_deref().ok();
    let systemd_manager = Path::new(SYSTEMD_MANAGER_DIR).exists();
    let github_actions = std::env::var_os(USER_WORK_SCOPE_CI_ENV).is_some();
    // SEED MIRROR of `gunbc.compute.unit_bounds user_work_host_facility_standing`.
    if cfg!(target_os = "macos") {
        return if !github_actions && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            UserWorkScopeStanding::OperatorHostWithoutCgroups
        } else {
            UserWorkScopeStanding::CgroupUnreadable
        };
    }
    let xdg_runtime_dir = std::env::var_os(USER_WORK_SCOPE_XDG_RUNTIME_DIR_ENV).is_some();
    let buildbuddy_run_id = buildbuddy_run_id_from_env();
    let namespaced_root = cg.is_some_and(cgroup_text_is_root) && !systemd_manager;
    let root_memory = if namespaced_root {
        root_memory_from_file(read_cgroup_memory_file("/", "memory.max"))
    } else {
        RootMemoryRead::Unreadable
    };
    // ONE check, called once, for every exempt premise: the chain of the cgroup this process actually
    // sits in. No premise -- CI env, slot path, slice path, unit path -- gets its own reading or its
    // own fast path any more.
    let bound = cg.map(live_enclosing_bound).unwrap_or_else(|| {
        no_live_enclosing_bound(
            String::new(),
            EnclosingBoundCause::ChainUnreadable {
                detail: "/proc/self/cgroup unreadable".to_string(),
            },
        )
    });
    let need_user_manager = !namespaced_root
        && cg.is_some_and(|text| wrap_exemption_premise(github_actions, text).is_none());
    standing_from_supplied(
        github_actions,
        cg,
        systemd_manager,
        need_user_manager && user_manager_reachable(),
        xdg_runtime_dir,
        root_memory,
        buildbuddy_run_id.as_deref(),
        bound,
    )
}

#[cfg(test)]
mod standing_fixture_tests {
    use super::*;
    use std::path::PathBuf;

    // A runner slot as CI actually lays it out on srv1: an `actions-runner@<host>.service` unit under
    // system.slice, which is what the declared needle (`runner_unit_prefix`) matches. A name invented
    // for the test would exercise the classifier against a path no host produces -- and quietly pass
    // through the unmodeled-system-unit premise instead.
    const SLOT: &str =
        r"0::/system.slice/system-actions\x2drunner.slice/actions-runner@srv1-03.service";
    const SLOT_LEAF: &str =
        r"/system.slice/system-actions\x2drunner.slice/actions-runner@srv1-03.service";

    // THE ONE CHECK, as the fold sees it: the observation is supplied to `standing_from_supplied`, so
    // each row says what the chain reading answered and what the fold made of it. The chain reader's
    // own verdicts (fixture trees on disk) are covered in `observed_cgroup_bound_tests` above; the
    // four rows at the bottom of this module pin the RED pairs end to end -- observation built from a
    // fixture tree, fold, then the printed line.
    fn wire_with_bound(
        github_actions: bool,
        cgroup: Option<&str>,
        bound: EnclosingBoundObservation,
    ) -> String {
        standing_wire(&standing_from_supplied(
            github_actions,
            cgroup,
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            None,
            bound,
        ))
    }

    fn live(cgroup: &str, bytes: u64) -> EnclosingBoundObservation {
        EnclosingBoundObservation::LiveEnclosingBound {
            cgroup: cgroup.to_string(),
            memory_max: bytes,
        }
    }

    // A chain that yielded no reading: the check names the CGROUP IT READ (the /proc/self/cgroup path,
    // flattened), not a directory, so rows that pin an unread refusal pass that path in.
    fn unread_at(cgroup: &str, detail: &str) -> EnclosingBoundObservation {
        missing(
            cgroup,
            EnclosingBoundCause::ChainUnreadable {
                detail: detail.to_string(),
            },
        )
    }

    fn missing(cgroup: &str, cause: EnclosingBoundCause) -> EnclosingBoundObservation {
        EnclosingBoundObservation::NoLiveEnclosingBound {
            cgroup: cgroup.to_string(),
            cause,
        }
    }

    fn no_finite_max(cgroup: &str) -> EnclosingBoundObservation {
        missing(cgroup, EnclosingBoundCause::NoFiniteMemoryMax)
    }

    // A premise that claims an enclosing bound refuses, located, when the check found none -- and the
    // line names the cgroup consulted and what its memory.max said. THE CI ARM: GITHUB_ACTIONS set is
    // no longer a skip (srv1 cap deploy run 38865847078: `CiExempt`, then HostBudgetUnreadable).
    #[test]
    fn ci_environment_without_a_live_bound_refuses_naming_the_cgroup_and_the_reading() {
        let cg = "0::/system.slice/gunbc-cap-deploy.service";
        let got = wire_with_bound(true, Some(cg), no_finite_max(cg));
        assert_eq!(
            got,
            "UncappedEnclosingCgroup:ci-runner-environment @ 0::/system.slice/gunbc-cap-deploy.service: no finite memory.max: every memory.max on the chain reads max"
        );
    }

    // THE RUNNER SLOT ARM, same pair. A slot path is not a bound; an observed finite one is.
    #[test]
    fn runner_slot_without_a_live_bound_refuses_and_with_one_skips() {
        // Literals, not format!: the `.dag` mirror join compares the DECLARED wire of these
        // standings against the seed's text, so the expected line must be readable there.
        let cg = SLOT;
        assert_eq!(
            wire_with_bound(false, Some(cg), no_finite_max(cg)),
            r"UncappedEnclosingCgroup:runner-slot-cgroup @ 0::/system.slice/system-actions\x2drunner.slice/actions-runner@srv1-03.service: no finite memory.max: every memory.max on the chain reads max"
        );
        assert_eq!(
            wire_with_bound(false, Some(cg), live(cg, 21474836480)),
            r"ExemptUnderObservedBound:runner-slot-cgroup:0::/system.slice/system-actions\x2drunner.slice/actions-runner@srv1-03.service:21474836480"
        );
    }

    // THE CI ARM'S POSITIVE CONTROL: the same GITHUB_ACTIONS process skips once a finite
    // memory.max is actually observed.
    #[test]
    fn ci_environment_with_a_live_bound_skips() {
        let cg = "0::/system.slice/gunbc-cap-deploy.service";
        assert_eq!(
            wire_with_bound(true, Some(cg), live(cg, 22548578304)),
            "ExemptUnderObservedBound:ci-runner-environment:0::/system.slice/gunbc-cap-deploy.service:22548578304"
        );
    }

    // THE ALREADY-IN-SLICE ARM, same pair: even a path that is the user-work slice itself is exempt
    // only while a finite bound is observed (its own slice memory.max, normally).
    #[test]
    fn already_in_the_user_work_slice_requires_the_live_bound_too() {
        let cg = format!("0::/{USER_WORK_SLICE}/gunbc-run.scope");
        assert_eq!(
            wire_with_bound(false, Some(&cg), no_finite_max(&cg)),
            format!("UncappedEnclosingCgroup:already-in-user-work-slice @ {cg}: no finite memory.max: every memory.max on the chain reads max")
        );
        assert_eq!(
            wire_with_bound(false, Some(&cg), live(&cg, 1073741824)),
            format!("ExemptUnderObservedBound:already-in-user-work-slice:{cg}:1073741824")
        );
    }

    // Every premise refuses through the ONE located cause, whatever its shape: unreadable, zero, or
    // swap-still-allowed are three names for "no live finite enclosing bound".
    #[test]
    fn every_negative_shape_of_the_check_refuses_the_same_way_with_its_own_cause() {
        let cg = "0::/gunbc-repository-convergence.slice/gunbc-tree-sync.service";
        let premise = "modeled-slice:gunbc-repository-convergence.slice:gunbc-tree-sync.service";
        let cases = [
            (
                missing(cg, EnclosingBoundCause::ChainUnreadable { detail: "EACCES".into() }),
                "memory.max unreadable: EACCES; no level of the chain yielded a reading",
            ),
            (
                missing(cg, EnclosingBoundCause::MemoryMaxZero { dir: "/sys/fs/cgroup/gunbc-repository-convergence.slice".into() }),
                "memory.max is 0 at /sys/fs/cgroup/gunbc-repository-convergence.slice, which systemd reads as no limit",
            ),
            (
                missing(cg, EnclosingBoundCause::SwapNotForbidden { dir: "/sys/fs/cgroup/x".into(), bytes: 4096 }),
                "tightest memory.max 4096 at /sys/fs/cgroup/x is not binding: no level on the chain sets memory.swap.max to 0",
            ),
        ];
        for (bound, cause) in cases {
            let got = wire_with_bound(false, Some(cg), bound);
            assert_eq!(
                got,
                format!("UncappedEnclosingCgroup:{premise} @ {cg}: {cause}"),
                "cause {cause:?}"
            );
        }
    }

    // The arm BoundedByObservedCgroup used to be: no premise at all, but a finite bound observed,
    // still skips. With no premise AND no bound, the user-manager arms decide.
    #[test]
    fn a_session_cgroup_skips_on_an_observed_bound_and_else_reaches_the_user_manager_arms() {
        let cg = "0::/user.slice/user-1000.slice/gunbc-session.scope";
        assert_eq!(
            wire_with_bound(false, Some(cg), live(cg, 4096)),
            "ExemptUnderObservedBound:no-premise:0::/user.slice/user-1000.slice/gunbc-session.scope:4096"
        );
        assert_eq!(
            standing_wire(&standing_from_supplied(
                false,
                Some(cg),
                true,
                true,
                false,
                RootMemoryRead::Unreadable,
                None,
                no_finite_max(cg),
            )),
            "UserManagerOwed"
        );
        assert_eq!(
            standing_wire(&standing_from_supplied(
                false,
                Some(cg),
                true,
                false,
                true,
                RootMemoryRead::Unreadable,
                None,
                no_finite_max(cg),
            )),
            "UserManagerProbeFailed"
        );
        assert_eq!(
            standing_wire(&standing_from_supplied(
                false,
                Some(cg),
                true,
                false,
                false,
                RootMemoryRead::Unreadable,
                None,
                no_finite_max(cg),
            )),
            "UserSessionAbsent"
        );
    }

    // The unmodeled-system-unit arm (a guest job unit under system.slice): admitted only by its
    // observed chain; without one it refuses, located.
    #[test]
    fn a_system_service_admits_only_on_an_observed_bound() {
        let cg = "0::/system.slice/gunbc-revision-job.service";
        assert_eq!(
            wire_with_bound(false, Some(cg), live(cg, 8589934592)),
            "ExemptUnderObservedBound:unmodeled-system-unit:system.slice:gunbc-revision-job.service:0::/system.slice/gunbc-revision-job.service:8589934592"
        );
        assert_eq!(
            wire_with_bound(false, Some(cg), no_finite_max(cg)),
            "UncappedEnclosingCgroup:unmodeled-system-unit:system.slice:gunbc-revision-job.service @ 0::/system.slice/gunbc-revision-job.service: no finite memory.max: every memory.max on the chain reads max"
        );
    }

    // The wire words the declared fallthrough fold declares, printed when a premise-free process
    // reaches the user-manager arms.
    #[test]
    fn the_fallthrough_wire_names_which_answer_the_check_gave() {
        assert_eq!(
            cgroup_bound_fallthrough_wire(&live("x", 1)),
            "chain-bounded"
        );
        assert_eq!(
            cgroup_bound_fallthrough_wire(&missing("x", EnclosingBoundCause::NoFiniteMemoryMax)),
            "chain-unbounded"
        );
        assert_eq!(
            cgroup_bound_fallthrough_wire(&missing(
                "x",
                EnclosingBoundCause::ChainUnreadable { detail: "x".into() }
            )),
            "chain-unread"
        );
    }

    // UNCHANGED BEHAVIOUR, pinned across the cut: the namespaced-root pair reads the container root's
    // memory.max, not the chain, so it never goes through the one check. Its subject is different, and
    // it was already live-bound (review 80587).
    fn root_wire(root_memory: RootMemoryRead, buildbuddy_run_id: Option<&str>) -> String {
        standing_wire(&standing_from_supplied(
            false,
            Some("0::/"),
            false,
            false,
            false,
            root_memory,
            buildbuddy_run_id,
            no_finite_max("/"),
        ))
    }

    #[test]
    fn namespaced_root_finite_memory_max_skips() {
        assert_eq!(
            root_wire(RootMemoryRead::Capped(25769803776), None),
            "ExemptBoundedContainer:25769803776"
        );
    }

    #[test]
    fn namespaced_root_unbounded_memory_max_refuses() {
        assert_eq!(
            root_wire(RootMemoryRead::Unbounded, None),
            "BoundedContainerUncapped"
        );
    }

    #[test]
    fn namespaced_root_unbounded_memory_max_refuses_even_with_a_buildbuddy_run_id() {
        assert_eq!(
            root_wire(RootMemoryRead::Unbounded, Some("measured-rbe-run")),
            "BoundedContainerUncapped"
        );
    }

    #[test]
    fn namespaced_root_zero_memory_max_refuses_even_with_a_buildbuddy_run_id() {
        assert_eq!(
            root_wire(RootMemoryRead::Capped(0), Some("measured-rbe-run")),
            "BoundedContainerUncapped"
        );
    }

    #[test]
    fn namespaced_root_unreadable_memory_max_with_buildbuddy_run_id_is_externally_bounded() {
        assert_eq!(
            root_wire(RootMemoryRead::Unreadable, Some("measured-rbe-run")),
            "ExternallyBoundedExecutor:measured-rbe-run"
        );
    }

    // UNIT_TEST_EXEMPT is the one exempt arm that claims no enclosing bound, so it is the one arm
    // reached without the check being consulted at all (its gate is cfg(test), not a memory.max).
    #[test]
    fn the_unit_test_arm_skips_without_any_bound_observed() {
        assert_eq!(
            standing_wire(&UserWorkScopeStanding::UnitTestExempt),
            "UnitTestExempt"
        );
        assert!(matches!(
            user_work_scope_standing(),
            UserWorkScopeStanding::UnitTestExempt
        ));
    }

    // THE DECLARED FIXTURE CGROUPS, spelled as `.dag` spells them, so the mirror join can compare the
    // declared fold's wire with the seed's printed line for the SAME cgroup. These three paths are the
    // ones `test.claim.live_deploy.user_work_slice_witness` composes from the declared needles.
    const SESSIONS_SCOPE: &str =
        "0::/user.slice/user-1000.slice/user@1000.service/app.slice/sessions.slice/docker-deadbeef.scope";
    const SYSTEM_TREE_SYNC: &str = "0::/system.slice/gunbc-tree-sync.service";
    const MODELED_TREE_SYNC: &str =
        "0::/gunbc-repository-convergence.slice/gunbc-tree-sync.service";

    // sessions.slice is a recognized slice (gunbc.oomd_install), so its premise names it, but a wrap
    // into gunbc-user-work.slice is not a fallback for a slice that already carries a named memory
    // policy: an unreadable or unbounded local memory.max refuses (review 80751).
    #[test]
    fn sessions_slice_scope_refuses_as_uncapped_system_slice() {
        assert_eq!(
            wire_with_bound(false, Some(SESSIONS_SCOPE), unread_at(SESSIONS_SCOPE, "fixture")),
            "UncappedEnclosingCgroup:modeled-slice:sessions.slice:docker-deadbeef.scope @ 0::/user.slice/user-1000.slice/user@1000.service/app.slice/sessions.slice/docker-deadbeef.scope: memory.max unreadable: fixture; no level of the chain yielded a reading"
        );
    }

    #[test]
    fn sessions_slice_with_finite_local_memory_max_skips() {
        assert_eq!(
            wire_with_bound(false, Some(SESSIONS_SCOPE), live(SESSIONS_SCOPE, 137438953472)),
            "ExemptUnderObservedBound:modeled-slice:sessions.slice:docker-deadbeef.scope:0::/user.slice/user-1000.slice/user@1000.service/app.slice/sessions.slice/docker-deadbeef.scope:137438953472"
        );
    }

    #[test]
    fn a_system_slice_named_service_cgroup_refuses_uncapped() {
        assert_eq!(
            wire_with_bound(false, Some(SYSTEM_TREE_SYNC), unread_at(SYSTEM_TREE_SYNC, "fixture")),
            "UncappedEnclosingCgroup:unmodeled-system-unit:system.slice:gunbc-tree-sync.service @ 0::/system.slice/gunbc-tree-sync.service: memory.max unreadable: fixture; no level of the chain yielded a reading"
        );
    }

    #[test]
    fn modeled_slice_with_zero_memory_max_refuses() {
        assert_eq!(
            wire_with_bound(
                false,
                Some(MODELED_TREE_SYNC),
                missing(
                    "/sys/fs/cgroup/gunbc-repository-convergence.slice",
                    EnclosingBoundCause::MemoryMaxZero {
                        dir: "/sys/fs/cgroup/gunbc-repository-convergence.slice".to_string(),
                    },
                ),
            ),
            "UncappedEnclosingCgroup:modeled-slice:gunbc-repository-convergence.slice:gunbc-tree-sync.service @ /sys/fs/cgroup/gunbc-repository-convergence.slice: memory.max is 0 at /sys/fs/cgroup/gunbc-repository-convergence.slice, which systemd reads as no limit"
        );
        // The same path with no reading at all is the same refusal, with the unread cause.
        assert_eq!(
            wire_with_bound(false, Some(MODELED_TREE_SYNC), unread_at(MODELED_TREE_SYNC, "fixture")),
            "UncappedEnclosingCgroup:modeled-slice:gunbc-repository-convergence.slice:gunbc-tree-sync.service @ 0::/gunbc-repository-convergence.slice/gunbc-tree-sync.service: memory.max unreadable: fixture; no level of the chain yielded a reading"
        );
    }

    // ---- RED pairs, end to end: a fixture cgroup tree -> the ONE check -> the fold -> the line. ----
    use super::observed_cgroup_bound_tests::{fixture_root, tree};

    fn chain_verdict(leaf: &str, max: &str, swap: &str) -> (PathBuf, EnclosingBoundObservation) {
        let root = tree(&[(leaf, max, swap)]);
        let obs = live_enclosing_bound_in(&format!("0::{leaf}\n"), &root);
        (root, obs)
    }

    // RED 1 (CI arm): GITHUB_ACTIONS set, and the process's chain has memory.max = max -> refuse, and
    // the refusal prints the /proc/self/cgroup path plus what memory.max said. The positive control is
    // `ci_environment_with_a_live_bound_skips` plus this one: same tree, finite max -> skip.
    #[test]
    fn a_ci_runner_on_a_chain_whose_memory_max_is_max_refuses_and_names_the_cgroup() {
        let leaf = "/system.slice/gunbc-cap-deploy.service";
        let (root, obs) = chain_verdict(leaf, "max", "0");
        assert!(matches!(
            obs,
            EnclosingBoundObservation::NoLiveEnclosingBound {
                cause: EnclosingBoundCause::NoFiniteMemoryMax,
                ..
            }
        ));
        let got = standing_wire(&standing_from_supplied(
            true,
            Some(&format!("0::{leaf}")),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            None,
            obs,
        ));
        assert!(
            got.starts_with("UncappedEnclosingCgroup:ci-runner-environment @ 0::/system.slice/gunbc-cap-deploy.service:"),
            "{got}"
        );
        assert!(
            got.contains("every memory.max on the chain reads max"),
            "{got}"
        );
        let _ = std::fs::remove_dir_all(&root);

        let (root, obs) = chain_verdict(leaf, "22548578304", "0");
        let got = standing_wire(&standing_from_supplied(
            true,
            Some(&format!("0::{leaf}")),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            None,
            obs,
        ));
        assert!(
            got.starts_with("ExemptUnderObservedBound:ci-runner-environment:"),
            "{got}"
        );
        assert!(got.ends_with(":22548578304"), "{got}");
        let _ = std::fs::remove_dir_all(&root);
    }

    // RED 2 (slot arm): a runner-slot path with no finite max refuses; the positive control -- the same
    // slot tree with a finite memory.max -- skips.
    #[test]
    fn a_runner_slot_without_a_finite_memory_max_refuses_and_with_one_skips() {
        let (root, obs) = chain_verdict(SLOT_LEAF, "max", "max");
        let got = standing_wire(&standing_from_supplied(
            false,
            Some(SLOT),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            None,
            obs,
        ));
        assert!(
            got.starts_with(&format!(
                "UncappedEnclosingCgroup:runner-slot-cgroup @ {SLOT}:"
            )),
            "{got}"
        );
        assert!(got.contains(SLOT_LEAF), "{got}");
        let _ = std::fs::remove_dir_all(&root);

        let (root, obs) = chain_verdict(SLOT_LEAF, "21474836480", "0");
        let got = standing_wire(&standing_from_supplied(
            false,
            Some(SLOT),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            None,
            obs,
        ));
        assert!(
            got.starts_with("ExemptUnderObservedBound:runner-slot-cgroup:"),
            "{got}"
        );
        // The bounded cgroup named is the deepest finite level of the fixture tree, i.e. the slot
        // directory itself (prefixed by the fixture root the walk was read from).
        assert!(got.contains(SLOT_LEAF.trim_start_matches('/')), "{got}");
        assert!(got.ends_with(":21474836480"), "{got}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
