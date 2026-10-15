//! Seed trampoline for `gunbc.compute.unit_bounds` `gunbc_user_work_scope_command`.
//! Prefix argv: `user_work_scope_wrap_prefix_argv`. Standing: `user_work_scope_standing` /
//! `user_work_scope_standing_effect`. Seed cannot eval those folds at wrap time; the join is
//! `the_seed_trampoline_mirrors_the_declared_wrap_prefix` and
//! `the_seed_trampoline_mirrors_the_declared_wrap_standing`.
//! Retention: `v2.compiler.self_host.seed_retention_frontier` `retained_support_dir_member`
//! (`src/cli_run/user_work_scope.rs`); trigger: Test/Run wrap is emitted, this file deletes.

use std::os::unix::process::CommandExt;
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

#[derive(Clone, Copy)]
enum UserWorkScopeStanding {
    AlreadyInSlice,
    BoundedByObservedCgroup,
    RunnerSlotExempt,
    CiExempt,
    UnitTestExempt,
    UserSessionAbsent,
    UserManagerOwed,
    CgroupUnreadable,
    UserManagerProbeFailed,
}

pub fn reexec_into_user_work_slice_if_owed() {
    let standing = user_work_scope_standing();
    match standing {
        UserWorkScopeStanding::UserManagerOwed => wrap_now(),
        UserWorkScopeStanding::CgroupUnreadable
        | UserWorkScopeStanding::UserManagerProbeFailed
        | UserWorkScopeStanding::UserSessionAbsent => {
            eprintln!(
                "error: cannot wrap into {USER_WORK_SLICE} ({})",
                standing_wire(standing)
            );
            std::process::exit(2);
        }
        UserWorkScopeStanding::AlreadyInSlice => {}
        UserWorkScopeStanding::UnitTestExempt => {}
        standing => {
            eprintln!(
                "user_work_scope: skip wrap into {USER_WORK_SLICE} ({})",
                standing_wire(standing)
            );
        }
    }
}

fn standing_wire(s: UserWorkScopeStanding) -> &'static str {
    match s {
        UserWorkScopeStanding::AlreadyInSlice => "AlreadyInSlice",
        UserWorkScopeStanding::BoundedByObservedCgroup => "BoundedByObservedCgroup",
        UserWorkScopeStanding::RunnerSlotExempt => "RunnerSlotExempt",
        UserWorkScopeStanding::CiExempt => "CiExempt",
        UserWorkScopeStanding::UnitTestExempt => "UnitTestExempt",
        UserWorkScopeStanding::UserSessionAbsent => "UserSessionAbsent",
        UserWorkScopeStanding::UserManagerOwed => "UserManagerOwed",
        UserWorkScopeStanding::CgroupUnreadable => "CgroupUnreadable",
        UserWorkScopeStanding::UserManagerProbeFailed => "UserManagerProbeFailed",
    }
}

fn wrap_now() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: current_exe unreadable; cannot wrap into {USER_WORK_SLICE}: {e}");
            std::process::exit(2);
        }
    };
    let mut cmd = Command::new("systemd-run");
    cmd.args([
        "--user",
        "--scope",
        &format!("--property=Slice={USER_WORK_SLICE}"),
        &format!("--property=MemorySwapMax={USER_WORK_SLICE_MEMORY_SWAP_MAX_BYTES}"),
        "--",
    ]);
    cmd.arg(&exe);
    cmd.args(std::env::args().skip(1));
    let err = cmd.exec();
    eprintln!("error: systemd-run exec into {USER_WORK_SLICE} failed: {err}");
    std::process::exit(1);
}

fn user_work_scope_standing() -> UserWorkScopeStanding {
    if cfg!(test) {
        return UserWorkScopeStanding::UnitTestExempt;
    }
    if std::env::var_os(USER_WORK_SCOPE_CI_ENV).is_some() {
        return UserWorkScopeStanding::CiExempt;
    }
    let cg = match std::fs::read_to_string("/proc/self/cgroup") {
        Ok(s) => s,
        Err(_) => return UserWorkScopeStanding::CgroupUnreadable,
    };
    if cg.contains(USER_WORK_SLICE) {
        return UserWorkScopeStanding::AlreadyInSlice;
    }
    if cg.contains(RUNNER_SLOT_CGROUP_NEEDLE) {
        return UserWorkScopeStanding::RunnerSlotExempt;
    }
    if observed_cgroup_bound(&cg) {
        return UserWorkScopeStanding::BoundedByObservedCgroup;
    }
    let probe_ok = Command::new("systemctl")
        .args(["--user", "show-environment"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if probe_ok {
        UserWorkScopeStanding::UserManagerOwed
    } else if std::env::var_os(USER_WORK_SCOPE_XDG_RUNTIME_DIR_ENV).is_some() {
        UserWorkScopeStanding::UserManagerProbeFailed
    } else {
        UserWorkScopeStanding::UserSessionAbsent
    }
}

// SEED MIRROR of `gunbc.compute.unit_bounds observed_cgroup_bound`: the process's unified-hierarchy
// cgroup (`0::<path>`) and each ancestor to the root; bounded when one carries a finite memory.max
// AND memory.swap.max = 0. An unreadable chain is not a bound.
fn observed_cgroup_bound(proc_self_cgroup: &str) -> bool {
    let Some(path) = proc_self_cgroup.lines().find_map(|l| l.strip_prefix("0::")) else {
        return false;
    };
    let mut current = path.trim().trim_end_matches('/').to_string();
    loop {
        let dir = format!("/sys/fs/cgroup{current}");
        let max =
            std::fs::read_to_string(format!("{dir}/memory.max")).map(|s| s.trim().to_string());
        let swap =
            std::fs::read_to_string(format!("{dir}/memory.swap.max")).map(|s| s.trim().to_string());
        if let (Ok(max), Ok(swap)) = (max, swap) {
            if swap == "0" && !max.is_empty() && max.bytes().all(|b| b.is_ascii_digit()) {
                return true;
            }
        }
        match current.rfind('/') {
            Some(0) | None => return false,
            Some(i) => current.truncate(i),
        }
    }
}
