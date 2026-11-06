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

pub(crate) enum UserWorkScopeStanding {
    AlreadyInSlice,
    BoundedByObservedCgroup,
    RunnerSlotExempt,
    CiExempt,
    UnitTestExempt,
    ExemptSystemUnit {
        unit: String,
        slice: String,
        bound: u64,
    },
    UncappedSystemSlice {
        unit: String,
        slice: String,
    },
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
        UserWorkScopeStanding::BoundedByObservedCgroup => "BoundedByObservedCgroup".to_string(),
        UserWorkScopeStanding::RunnerSlotExempt => "RunnerSlotExempt".to_string(),
        UserWorkScopeStanding::CiExempt => "CiExempt".to_string(),
        UserWorkScopeStanding::UnitTestExempt => "UnitTestExempt".to_string(),
        UserWorkScopeStanding::ExemptSystemUnit { unit, slice, bound } => {
            format!("ExemptSystemUnit:{slice}:{unit}:{bound}")
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
        RootMemoryRead::Capped(n) if n > 0 => UserWorkScopeStanding::ExemptSystemUnit {
            unit,
            slice,
            bound: n,
        },
        RootMemoryRead::Capped(_) | RootMemoryRead::Unbounded | RootMemoryRead::Unreadable => {
            UserWorkScopeStanding::UncappedSystemSlice { unit, slice }
        }
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
    bound: CgroupBound,
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
    match bound {
        CgroupBound::Bounded => return UserWorkScopeStanding::BoundedByObservedCgroup,
        fallthrough => eprintln!(
            "user_work_scope: observed-bound arm did not admit ({})",
            cgroup_bound_fallthrough_wire(fallthrough)
        ),
    }
    if let Some((unit, slice)) = system_service_unit_from_cgroup(cg) {
        if RECOGNIZED_SYSTEM_SLICES.contains(&slice.as_str()) {
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

// SEED MIRROR of `gunbc.compute.unit_bounds observed_cgroup_bound`, which reads the chain through
// `extdeps.linux.cgroup_v2_memory`: the process's unified-hierarchy cgroup (`0::<path>`) and every
// ancestor below the hierarchy root. Bounded when some level carries a finite memory.max (decimal,
// within u64 -- `parse_cgroup_memory_limit`'s "Limited") AND some level sets memory.swap.max to 0:
// any ancestor's limit binds and a swap ceiling of 0 anywhere forbids swap below it. A chain with
// neither file readable is unread; anything else is unbounded.
// RETAINED, with this file: the reason is that the trampoline must decide before the corpus is
// resolved, so it cannot evaluate the `.dag` fold; the migration trigger is the file's own (Test/Run
// wrap emitted natively, this file deletes). Executed by the unit tests below over fixture trees, not
// only by the `.dag` witness's text join.
#[derive(Clone, Copy)]
enum CgroupBound {
    Bounded,
    Unbounded,
    Unread,
}

// SEED MIRROR of `gunbc.compute.unit_bounds cgroup_bound_fallthrough_wire`
fn cgroup_bound_fallthrough_wire(b: CgroupBound) -> &'static str {
    match b {
        CgroupBound::Bounded => "chain-bounded",
        CgroupBound::Unbounded => "chain-unbounded",
        CgroupBound::Unread => "chain-unread",
    }
}

fn observed_cgroup_bound(proc_self_cgroup: &str) -> CgroupBound {
    observed_cgroup_bound_in(proc_self_cgroup, std::path::Path::new("/sys/fs/cgroup"))
}

fn observed_cgroup_bound_in(proc_self_cgroup: &str, sysfs_root: &std::path::Path) -> CgroupBound {
    let Some(path) = proc_self_cgroup.lines().find_map(|l| l.strip_prefix("0::")) else {
        return CgroupBound::Unread;
    };
    let mut any_read = false;
    let mut binding_limit = false;
    let mut zero_limit = false;
    let mut swap_forbidden = false;
    let mut current = path.trim().trim_end_matches('/').to_string();
    while !current.is_empty() {
        let dir = format!("{}{current}", sysfs_root.display());
        if let Ok(max) = std::fs::read_to_string(format!("{dir}/memory.max")) {
            any_read = true;
            match max.trim().parse::<u64>() {
                Ok(0) => zero_limit = true,
                Ok(_) => binding_limit = true,
                Err(_) => {}
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
    if binding_limit && !zero_limit && swap_forbidden {
        CgroupBound::Bounded
    } else if any_read {
        CgroupBound::Unbounded
    } else {
        CgroupBound::Unread
    }
}

#[cfg(test)]
mod observed_cgroup_bound_tests {
    use super::{observed_cgroup_bound_in, CgroupBound};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    // Plain functions rather than a struct with impls: an impl method is not citable as a
    // DeclarationRef, and this module's items are enumerated in
    // gunbc.user_work_observed_bound_seed_growth.
    fn fixture_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gunbc-cgroup-bound-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).expect("fixture root");
        dir
    }

    fn tree(levels: &[(&str, &str, &str)]) -> PathBuf {
        let root = fixture_root();
        for (path, max, swap) in levels {
            let dir = root.join(path.trim_start_matches('/'));
            std::fs::create_dir_all(&dir).expect("mkdir");
            std::fs::write(dir.join("memory.max"), format!("{max}\n")).expect("max");
            std::fs::write(dir.join("memory.swap.max"), format!("{swap}\n")).expect("swap");
        }
        root
    }

    // Reads the verdict and removes the fixture, so each tree is read exactly once.
    fn verdict(root: PathBuf, leaf: &str) -> &'static str {
        let v = match observed_cgroup_bound_in(&format!("0::{leaf}\n"), &root) {
            CgroupBound::Bounded => "bounded",
            CgroupBound::Unbounded => "unbounded",
            CgroupBound::Unread => "unread",
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
        assert_eq!(verdict(t, "/system.slice/job.service"), "bounded");
    }

    #[test]
    fn a_limit_on_one_level_and_no_swap_on_another_bounds_the_leaf() {
        let t = tree(&[
            ("/system.slice/job.service", "max", "0"),
            ("/system.slice", "8589934592", "max"),
        ]);
        assert_eq!(verdict(t, "/system.slice/job.service"), "bounded");
    }

    #[test]
    fn a_bound_on_an_ancestor_bounds_the_leaf() {
        let t = tree(&[
            ("/system.slice/job.service", "max", "max"),
            ("/system.slice", "8589934592", "0"),
        ]);
        assert_eq!(verdict(t, "/system.slice/job.service"), "bounded");
    }

    #[test]
    fn unlimited_max_or_allowed_swap_or_out_of_range_never_bounds() {
        let unlimited = tree(&[("/system.slice/job.service", "max", "0")]);
        assert_eq!(verdict(unlimited, "/system.slice/job.service"), "unbounded");
        let swap = tree(&[("/system.slice/job.service", "21474836480", "max")]);
        assert_eq!(verdict(swap, "/system.slice/job.service"), "unbounded");
        let huge = tree(&[("/system.slice/job.service", "99999999999999999999999", "0")]);
        assert_eq!(verdict(huge, "/system.slice/job.service"), "unbounded");
        let zero = tree(&[("/system.slice/job.service", "0", "0")]);
        assert_eq!(verdict(zero, "/system.slice/job.service"), "unbounded");
        let zero_ancestor = tree(&[
            ("/system.slice/job.service", "21474836480", "0"),
            ("/system.slice", "0", "max"),
        ]);
        assert_eq!(
            verdict(zero_ancestor, "/system.slice/job.service"),
            "unbounded"
        );
    }

    #[test]
    fn an_unreadable_chain_or_missing_line_is_unread_not_bounded() {
        let empty = fixture_root();
        assert!(matches!(
            observed_cgroup_bound_in("1:name=systemd:/x\n", &empty),
            CgroupBound::Unread
        ));
        assert_eq!(verdict(empty, "/system.slice/job.service"), "unread");
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
    let modeled = cg.and_then(innermost_modeled_capped_slice);
    let service_slice = cg.and_then(system_service_unit_from_cgroup).map(|(_, s)| s);
    let slice_for_live = modeled
        .or_else(|| service_slice.filter(|s| RECOGNIZED_SYSTEM_SLICES.contains(&s.as_str())));
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
        cg.map(observed_cgroup_bound).unwrap_or(CgroupBound::Unread),
    )
}

#[cfg(test)]
mod standing_fixture_tests {
    use super::*;

    // The observed-bound arm sits after the modeled-slice arm and BEFORE the system-unit refusal:
    // a guest's job unit under an unmodeled system.slice is admitted only by its observed chain.
    #[test]
    fn a_system_service_with_an_observed_bound_chain_admits_and_without_one_refuses() {
        let cg = "0::/system.slice/gunbc-revision-job.service";
        let run = |bound| {
            standing_wire(&standing_from_supplied(
                false,
                Some(cg),
                true,
                false,
                false,
                RootMemoryRead::Unreadable,
                RootMemoryRead::Unreadable,
                None,
                bound,
            ))
        };
        assert_eq!(run(CgroupBound::Bounded), "BoundedByObservedCgroup");
        assert_eq!(
            run(CgroupBound::Unbounded),
            "UncappedSystemSlice:system.slice:gunbc-revision-job.service"
        );
        assert_eq!(
            run(CgroupBound::Unread),
            "UncappedSystemSlice:system.slice:gunbc-revision-job.service"
        );
    }

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
            CgroupBound::Unread,
        ))
    }

    #[test]
    fn sessions_slice_scope_refuses_as_uncapped_system_slice() {
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
        assert_eq!(
            got,
            "UncappedSystemSlice:sessions.slice:docker-deadbeef.scope"
        );
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
    fn namespaced_root_unbounded_memory_max_refuses_even_with_a_buildbuddy_run_id() {
        let got = wire(
            false,
            Some("0::/"),
            false,
            false,
            false,
            RootMemoryRead::Unbounded,
            RootMemoryRead::Unreadable,
            Some("measured-rbe-run"),
        );
        assert_eq!(got, "BoundedContainerUncapped");
    }

    #[test]
    fn namespaced_root_zero_memory_max_refuses_even_with_a_buildbuddy_run_id() {
        let got = wire(
            false,
            Some("0::/"),
            false,
            false,
            false,
            RootMemoryRead::Capped(0),
            RootMemoryRead::Unreadable,
            Some("measured-rbe-run"),
        );
        assert_eq!(got, "BoundedContainerUncapped");
    }

    #[test]
    fn namespaced_root_unreadable_memory_max_with_buildbuddy_run_id_is_externally_bounded() {
        let got = wire(
            false,
            Some("0::/"),
            false,
            false,
            false,
            RootMemoryRead::Unreadable,
            RootMemoryRead::Unreadable,
            Some("measured-rbe-run"),
        );
        assert_eq!(got, "ExternallyBoundedExecutor:measured-rbe-run");
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
            "ExemptSystemUnit:gunbc-repository-convergence.slice:gunbc-tree-sync.service:1"
        );
    }

    #[test]
    fn modeled_slice_with_zero_memory_max_refuses() {
        let cg = "0::/gunbc-repository-convergence.slice/gunbc-tree-sync.service";
        let got = wire(
            false,
            Some(cg),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            RootMemoryRead::Capped(0),
            None,
        );
        assert_eq!(
            got,
            "UncappedSystemSlice:gunbc-repository-convergence.slice:gunbc-tree-sync.service"
        );
    }

    #[test]
    fn sessions_slice_with_finite_local_memory_max_skips() {
        let cg = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/sessions.slice/docker-deadbeef.scope";
        let got = wire(
            false,
            Some(cg),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            RootMemoryRead::Capped(137438953472),
            None,
        );
        assert_eq!(
            got,
            "ExemptSystemUnit:sessions.slice:docker-deadbeef.scope:137438953472"
        );
    }

    #[test]
    fn sessions_slice_with_unbounded_local_memory_max_refuses() {
        let cg = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/sessions.slice/docker-deadbeef.scope";
        let got = wire(
            false,
            Some(cg),
            true,
            false,
            false,
            RootMemoryRead::Unreadable,
            RootMemoryRead::Unbounded,
            None,
        );
        assert_eq!(
            got,
            "UncappedSystemSlice:sessions.slice:docker-deadbeef.scope"
        );
    }

    #[test]
    fn sessions_slice_with_unreadable_local_memory_max_refuses() {
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
        assert_eq!(
            got,
            "UncappedSystemSlice:sessions.slice:docker-deadbeef.scope"
        );
    }
}
