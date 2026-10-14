//! Realization of `gunbc.compute.unit_bounds` `gunbc_user_work_scope_command` for the
//! retained `gunbc test` / `gunbc run` verbs. Shape: `systemd-run --user --scope`
//! `-p Slice=gunbc-user-work.slice` `-p MemorySwapMax=0` `--` argv.
//! Skip when already in that slice, already in an actions-runner unit, or the user
//! manager is unreachable (CI / GITHUB_ACTIONS).

use std::os::unix::process::CommandExt;
use std::process::Command;

const USER_WORK_SLICE: &str = "gunbc-user-work.slice";

pub fn reexec_into_user_work_slice_if_owed() {
    if !owes_user_work_scope() {
        return;
    }
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
        "--property=MemorySwapMax=0",
        "--",
    ]);
    cmd.arg(&exe);
    cmd.args(std::env::args().skip(1));
    let err = cmd.exec();
    eprintln!("error: systemd-run exec into {USER_WORK_SLICE} failed: {err}");
    std::process::exit(1);
}

fn owes_user_work_scope() -> bool {
    if cfg!(test) {
        return false;
    }
    if std::env::var_os("GITHUB_ACTIONS").is_some() {
        return false;
    }
    let cg = std::fs::read_to_string("/proc/self/cgroup").unwrap_or_default();
    if cg.contains(USER_WORK_SLICE) {
        return false;
    }
    if cg.contains("actions-runner@") {
        return false;
    }
    Command::new("systemctl")
        .args(["--user", "show-environment"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
