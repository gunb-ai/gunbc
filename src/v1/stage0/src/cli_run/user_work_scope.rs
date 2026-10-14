//! Seed trampoline for `gunbc.compute.unit_bounds` `gunbc_user_work_scope_command`.
//! The prefix argv is owned by `user_work_scope_wrap_prefix_argv`; this file cannot
//! eval that fold (wrap runs before corpus resolve). The join is
//! `test.claim.live_deploy.user_work_slice_witness`
//! `the_seed_trampoline_mirrors_the_declared_wrap_prefix`.
//! Retention: `v2.compiler.self_host.seed_retention_frontier` `retained_support_dir_member`
//! (`src/cli_run/user_work_scope.rs`); trigger: Test/Run wrap is emitted, this file deletes.

use std::os::unix::process::CommandExt;
use std::process::Command;

// SEED MIRROR of `gunbc.live_deploy.slice_bounds user_work_slice_unit_name`
const USER_WORK_SLICE: &str = "gunbc-user-work.slice";
// SEED MIRROR of `gunbc.live_deploy.slice_bounds user_work_slice_memory_swap_max`
const USER_WORK_SLICE_MEMORY_SWAP_MAX_BYTES: u64 = 0;

#[derive(Clone, Copy)]
enum UserWorkScopeStanding {
    AlreadyInSlice,
    RunnerSlotExempt,
    CiExempt,
    UnitTestExempt,
    UserSessionAbsent,
    UserManagerOwed,
    CgroupUnreadable,
    UserManagerProbeFailed,
}

pub fn reexec_into_user_work_slice_if_owed() {
    match user_work_scope_standing() {
        UserWorkScopeStanding::UserManagerOwed => wrap_now(),
        UserWorkScopeStanding::CgroupUnreadable => {
            eprintln!(
                "error: /proc/self/cgroup unreadable; cannot decide wrap into {USER_WORK_SLICE}"
            );
            std::process::exit(2);
        }
        UserWorkScopeStanding::UserManagerProbeFailed => {
            eprintln!(
                "error: systemctl --user show-environment failed with XDG_RUNTIME_DIR set; cannot wrap into {USER_WORK_SLICE}"
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
    if std::env::var_os("GITHUB_ACTIONS").is_some() {
        return UserWorkScopeStanding::CiExempt;
    }
    let cg = match std::fs::read_to_string("/proc/self/cgroup") {
        Ok(s) => s,
        Err(_) => return UserWorkScopeStanding::CgroupUnreadable,
    };
    if cg.contains(USER_WORK_SLICE) {
        return UserWorkScopeStanding::AlreadyInSlice;
    }
    if cg.contains("actions-runner@") {
        return UserWorkScopeStanding::RunnerSlotExempt;
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
    } else if std::env::var_os("XDG_RUNTIME_DIR").is_some() {
        UserWorkScopeStanding::UserManagerProbeFailed
    } else {
        UserWorkScopeStanding::UserSessionAbsent
    }
}
