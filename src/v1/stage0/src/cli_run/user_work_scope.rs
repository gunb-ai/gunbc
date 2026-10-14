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
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_user_slice_cgroup_needle`
const USER_SLICE_CGROUP_NEEDLE: &str = "/user.slice/";
// SEED MIRROR of `gunbc.compute.unit_bounds user_work_scope_service_unit_suffix`
const SERVICE_UNIT_SUFFIX: &str = ".service";

enum UserWorkScopeStanding {
    AlreadyInSlice,
    RunnerSlotExempt,
    CiExempt,
    UnitTestExempt,
    ExemptSystemUnit { unit: String },
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

fn standing_wire(s: &UserWorkScopeStanding) -> String {
    match s {
        UserWorkScopeStanding::AlreadyInSlice => "AlreadyInSlice".to_string(),
        UserWorkScopeStanding::RunnerSlotExempt => "RunnerSlotExempt".to_string(),
        UserWorkScopeStanding::CiExempt => "CiExempt".to_string(),
        UserWorkScopeStanding::UnitTestExempt => "UnitTestExempt".to_string(),
        UserWorkScopeStanding::ExemptSystemUnit { unit } => {
            format!("ExemptSystemUnit:{unit}")
        }
        UserWorkScopeStanding::UserSessionAbsent => "UserSessionAbsent".to_string(),
        UserWorkScopeStanding::UserManagerOwed => "UserManagerOwed".to_string(),
        UserWorkScopeStanding::CgroupUnreadable => "CgroupUnreadable".to_string(),
        UserWorkScopeStanding::UserManagerProbeFailed => "UserManagerProbeFailed".to_string(),
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

// SEED MIRROR of `gunbc.compute.unit_bounds system_service_unit_from_cgroup`
fn system_service_unit_from_cgroup(text: &str) -> Option<String> {
    if text.contains(USER_SLICE_CGROUP_NEEDLE) {
        return None;
    }
    text.split(['/', '\n'])
        .map(str::trim)
        .filter(|seg| seg.ends_with(SERVICE_UNIT_SUFFIX) && !seg.is_empty())
        .last()
        .map(str::to_string)
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
    if let Some(unit) = system_service_unit_from_cgroup(&cg) {
        return UserWorkScopeStanding::ExemptSystemUnit { unit };
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
