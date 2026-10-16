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
// SEED MIRROR of `gunbc.live_deploy.slice_bounds declared_system_slice_memory_maxes` unit names
// (rows already filtered by byte_size_count(memory_max) > 0 in modeled_system_slice_memory_max).
// Wrap cannot eval the DAG; this list is seed-retained with user_work_scope.rs. A census name
// missing here is a RED in `the_seed_trampoline_mirrors_the_declared_wrap_standing`.
const MODELED_CAPPED_SYSTEM_SLICES: &[&str] = &[
    "gunbc-repository-convergence.slice",
    "gunbc-approval-broker.slice",
    "gunbc-attempt-worker.slice",
];

pub(crate) enum UserWorkScopeStanding {
    AlreadyInSlice,
    RunnerSlotExempt,
    CiExempt,
    UnitTestExempt,
    ExemptSystemUnit { unit: String, slice: String },
    UncappedSystemSlice { unit: String, slice: String },
    UserSessionAbsent,
    UserManagerOwed,
    CgroupUnreadable,
    UserManagerProbeFailed,
    UserWorkSliceAbsent,
    UserWorkSliceUncapped,
    CallerEnvelopeUnreadable,
    ExemptBoundedContainer { bytes: u64 },
    BoundedContainerUncapped,
    BoundedContainerUnreadable,
    ExternallyBoundedExecutor { run_id: String },
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
        | UserWorkScopeStanding::UncappedSystemSlice { .. }
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
        UserWorkScopeStanding::AlreadyInSlice => {}
        UserWorkScopeStanding::UnitTestExempt => {}
        standing => {
            eprintln!(
                "user_work_scope: skip wrap into {USER_WORK_SLICE} ({})",
                standing_wire(&standing)
            );
        }
    }
}

pub(crate) fn standing_wire(s: &UserWorkScopeStanding) -> String {
    match s {
        UserWorkScopeStanding::AlreadyInSlice => "AlreadyInSlice".to_string(),
        UserWorkScopeStanding::RunnerSlotExempt => "RunnerSlotExempt".to_string(),
        UserWorkScopeStanding::CiExempt => "CiExempt".to_string(),
        UserWorkScopeStanding::UnitTestExempt => "UnitTestExempt".to_string(),
        UserWorkScopeStanding::ExemptSystemUnit { unit, slice } => {
            format!("ExemptSystemUnit:{slice}:{unit}")
        }
        UserWorkScopeStanding::UncappedSystemSlice { unit, slice } => {
            format!("UncappedSystemSlice:{slice}:{unit}")
        }
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
        MemoryBound::Capped(n) if n == 0 => return None,
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
        .filter(|seg| {
            if seg.is_empty() {
                return false;
            }
            let mut parts = seg.rsplitn(2, '.');
            let last = parts.next().unwrap_or("");
            parts.next().is_some() && format!(".{last}") == suffix
        })
        .last()
        .map(str::to_string)
}

fn cgroup_rel_through_segment(text: &str, segment: &str) -> Option<String> {
    let rel = cgroup_v2_rel_path(text)?;
    let mut acc = String::new();
    for seg in rel.split('/') {
        if seg.is_empty() {
            continue;
        }
        acc.push('/');
        acc.push_str(seg);
        if seg == segment {
            return Some(acc);
        }
    }
    None
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

// SEED MIRROR of `gunbc.compute.unit_bounds live_modeled_slice_standing`
fn live_modeled_slice_standing(
    unit: String,
    slice: String,
    slice_memory: RootMemoryRead,
) -> UserWorkScopeStanding {
    match slice_memory {
        RootMemoryRead::Capped(_) => UserWorkScopeStanding::ExemptSystemUnit { unit, slice },
        RootMemoryRead::Unbounded | RootMemoryRead::Unreadable => {
            UserWorkScopeStanding::UncappedSystemSlice { unit, slice }
        }
    }
}

// SEED MIRROR of `gunbc.compute.unit_bounds innermost_modeled_capped_slice`
fn innermost_modeled_capped_slice(text: &str) -> Option<String> {
    text.split(['/', '\n'])
        .map(str::trim)
        .filter(|seg| MODELED_CAPPED_SYSTEM_SLICES.contains(seg))
        .last()
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
        RootMemoryRead::Capped(n) => UserWorkScopeStanding::ExemptBoundedContainer { bytes: n },
        RootMemoryRead::Unbounded => UserWorkScopeStanding::BoundedContainerUncapped,
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

// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_standing`
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_standing_with_root_memory`
fn standing_from_supplied(
    github_actions: bool,
    cgroup: Option<&str>,
    systemd_manager_present: bool,
    user_manager_reachable: bool,
    xdg_runtime_dir: bool,
    root_memory: RootMemoryRead,
    slice_memory: RootMemoryRead,
    buildbuddy_run_id: Option<&str>,
) -> UserWorkScopeStanding {
    if cgroup.is_some_and(cgroup_text_is_root) && !systemd_manager_present {
        return namespaced_root_standing(root_memory, buildbuddy_run_id);
    }
    if github_actions {
        return UserWorkScopeStanding::CiExempt;
    }
    let Some(cg) = cgroup else {
        return UserWorkScopeStanding::CgroupUnreadable;
    };
    if cg.contains(USER_WORK_SLICE) {
        return UserWorkScopeStanding::AlreadyInSlice;
    }
    if cg.contains(RUNNER_SLOT_CGROUP_NEEDLE) {
        return UserWorkScopeStanding::RunnerSlotExempt;
    }
    if let Some(slice) = innermost_modeled_capped_slice(cg) {
        let unit = last_cgroup_unit_with_suffix(cg, SCOPE_UNIT_SUFFIX)
            .or_else(|| last_cgroup_unit_with_suffix(cg, SERVICE_UNIT_SUFFIX))
            .unwrap_or_default();
        return live_modeled_slice_standing(unit, slice, slice_memory);
    }
    if let Some((unit, slice)) = system_service_unit_from_cgroup(cg) {
        if MODELED_CAPPED_SYSTEM_SLICES.contains(&slice.as_str()) {
            return live_modeled_slice_standing(unit, slice, slice_memory);
        }
        return UserWorkScopeStanding::UncappedSystemSlice { unit, slice };
    }
    if user_manager_reachable {
        UserWorkScopeStanding::UserManagerOwed
    } else if xdg_runtime_dir {
        UserWorkScopeStanding::UserManagerProbeFailed
    } else {
        UserWorkScopeStanding::UserSessionAbsent
    }
}

fn live_slice_memory(cg: &str, slice: &str) -> RootMemoryRead {
    let rel = cgroup_rel_through_segment(cg, slice).unwrap_or_else(|| format!("/{slice}"));
    root_memory_from_file(read_cgroup_memory_file(&rel, "memory.max"))
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
    let xdg_runtime_dir = std::env::var_os(USER_WORK_SCOPE_XDG_RUNTIME_DIR_ENV).is_some();
    let buildbuddy_run_id = buildbuddy_run_id_from_env();
    let namespaced_root = cg.is_some_and(cgroup_text_is_root) && !systemd_manager;
    let root_memory = if namespaced_root {
        root_memory_from_file(read_cgroup_memory_file("/", "memory.max"))
    } else {
        RootMemoryRead::Unreadable
    };
    let modeled = cg.and_then(innermost_modeled_capped_slice);
    let service_slice = cg.and_then(system_service_unit_from_cgroup).map(|(_, s)| s);
    let slice_for_live = modeled
        .or_else(|| service_slice.filter(|s| MODELED_CAPPED_SYSTEM_SLICES.contains(&s.as_str())));
    let slice_memory = match (cg, slice_for_live.as_deref()) {
        (Some(text), Some(slice)) => live_slice_memory(text, slice),
        _ => RootMemoryRead::Unreadable,
    };
    let need_user_manager = !namespaced_root
        && !github_actions
        && cg.is_some_and(|text| {
            !text.contains(USER_WORK_SLICE)
                && !text.contains(RUNNER_SLOT_CGROUP_NEEDLE)
                && innermost_modeled_capped_slice(text).is_none()
                && system_service_unit_from_cgroup(text).is_none()
        });
    standing_from_supplied(
        github_actions,
        cg,
        systemd_manager,
        need_user_manager && user_manager_reachable(),
        xdg_runtime_dir,
        root_memory,
        slice_memory,
        buildbuddy_run_id.as_deref(),
    )
}

#[cfg(test)]
mod standing_fixture_tests {
    use super::*;

    fn wire(
        github_actions: bool,
        cgroup: Option<&str>,
        systemd_manager_present: bool,
        user_manager_reachable: bool,
        xdg_runtime_dir: bool,
        root_memory: RootMemoryRead,
        slice_memory: RootMemoryRead,
        buildbuddy_run_id: Option<&str>,
    ) -> String {
        standing_wire(&standing_from_supplied(
            github_actions,
            cgroup,
            systemd_manager_present,
            user_manager_reachable,
            xdg_runtime_dir,
            root_memory,
            slice_memory,
            buildbuddy_run_id,
        ))
    }

    #[test]
    fn sessions_slice_scope_refuses_as_user_session_absent() {
        let cg = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/sessions.slice/docker-deadbeef.scope";
        let got = wire(
            false,
            Some(cg),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            RootMemoryRead::Unreadable,
            None,
        );
        assert_eq!(got, "UserSessionAbsent");
    }

    #[test]
    fn system_slice_service_refuses_uncapped() {
        let cg = "0::/system.slice/gunbc-tree-sync.service";
        let got = wire(
            false,
            Some(cg),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            RootMemoryRead::Unreadable,
            None,
        );
        assert_eq!(
            got,
            "UncappedSystemSlice:system.slice:gunbc-tree-sync.service"
        );
    }

    #[test]
    fn namespaced_root_finite_memory_max_skips() {
        let got = wire(
            false,
            Some("0::/"),
            false,
            false,
            false,
            RootMemoryRead::Capped(25769803776),
            RootMemoryRead::Unreadable,
            None,
        );
        assert_eq!(got, "ExemptBoundedContainer:25769803776");
    }

    #[test]
    fn namespaced_root_unbounded_memory_max_refuses() {
        let got = wire(
            false,
            Some("0::/"),
            false,
            false,
            false,
            RootMemoryRead::Unbounded,
            RootMemoryRead::Unreadable,
            None,
        );
        assert_eq!(got, "BoundedContainerUncapped");
    }

    #[test]
    fn modeled_slice_with_finite_memory_max_skips() {
        let cg = "0::/gunbc-repository-convergence.slice/gunbc-tree-sync.service";
        let got = wire(
            false,
            Some(cg),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            RootMemoryRead::Capped(1),
            None,
        );
        assert_eq!(
            got,
            "ExemptSystemUnit:gunbc-repository-convergence.slice:gunbc-tree-sync.service"
        );
    }
}
