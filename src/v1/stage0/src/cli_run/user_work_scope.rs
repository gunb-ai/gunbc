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
    match observed_cgroup_bound(&cg) {
        CgroupBound::Bounded => return UserWorkScopeStanding::BoundedByObservedCgroup,
        fallthrough => eprintln!(
            "user_work_scope: observed-bound arm did not admit ({})",
            cgroup_bound_fallthrough_wire(fallthrough)
        ),
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
    let mut swap_forbidden = false;
    let mut current = path.trim().trim_end_matches('/').to_string();
    while !current.is_empty() {
        let dir = format!("{}{current}", sysfs_root.display());
        if let Ok(max) = std::fs::read_to_string(format!("{dir}/memory.max")) {
            any_read = true;
            if max.trim().parse::<u64>().is_ok() {
                binding_limit = true;
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
    if binding_limit && swap_forbidden {
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct FixtureRoot(std::path::PathBuf);

    impl FixtureRoot {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "gunbc-cgroup-bound-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&dir).expect("fixture root");
            FixtureRoot(dir)
        }
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for FixtureRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn tree(levels: &[(&str, &str, &str)]) -> FixtureRoot {
        let root = FixtureRoot::new();
        for (path, max, swap) in levels {
            let dir = root.path().join(path.trim_start_matches('/'));
            std::fs::create_dir_all(&dir).expect("mkdir");
            std::fs::write(dir.join("memory.max"), format!("{max}\n")).expect("max");
            std::fs::write(dir.join("memory.swap.max"), format!("{swap}\n")).expect("swap");
        }
        root
    }

    fn verdict(root: &FixtureRoot, leaf: &str) -> &'static str {
        match observed_cgroup_bound_in(&format!("0::{leaf}\n"), root.path()) {
            CgroupBound::Bounded => "bounded",
            CgroupBound::Unbounded => "unbounded",
            CgroupBound::Unread => "unread",
        }
    }

    #[test]
    fn a_finite_max_with_no_swap_on_the_unit_bounds_it() {
        let t = tree(&[
            ("/system.slice/job.service", "21474836480", "0"),
            ("/system.slice", "max", "max"),
        ]);
        assert_eq!(verdict(&t, "/system.slice/job.service"), "bounded");
    }

    #[test]
    fn a_limit_on_one_level_and_no_swap_on_another_bounds_the_leaf() {
        let t = tree(&[
            ("/system.slice/job.service", "max", "0"),
            ("/system.slice", "8589934592", "max"),
        ]);
        assert_eq!(verdict(&t, "/system.slice/job.service"), "bounded");
    }

    #[test]
    fn a_bound_on_an_ancestor_bounds_the_leaf() {
        let t = tree(&[
            ("/system.slice/job.service", "max", "max"),
            ("/system.slice", "8589934592", "0"),
        ]);
        assert_eq!(verdict(&t, "/system.slice/job.service"), "bounded");
    }

    #[test]
    fn unlimited_max_or_allowed_swap_or_out_of_range_never_bounds() {
        let unlimited = tree(&[("/system.slice/job.service", "max", "0")]);
        assert_eq!(
            verdict(&unlimited, "/system.slice/job.service"),
            "unbounded"
        );
        let swap = tree(&[("/system.slice/job.service", "21474836480", "max")]);
        assert_eq!(verdict(&swap, "/system.slice/job.service"), "unbounded");
        let huge = tree(&[("/system.slice/job.service", "99999999999999999999999", "0")]);
        assert_eq!(verdict(&huge, "/system.slice/job.service"), "unbounded");
    }

    #[test]
    fn an_unreadable_chain_or_missing_line_is_unread_not_bounded() {
        let empty = FixtureRoot::new();
        assert_eq!(verdict(&empty, "/system.slice/job.service"), "unread");
        assert!(matches!(
            observed_cgroup_bound_in("1:name=systemd:/x\n", empty.path()),
            CgroupBound::Unread
        ));
    }
}
