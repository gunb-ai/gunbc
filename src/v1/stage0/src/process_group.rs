//! GENERIC PROCESS-GROUP MECHANICS, SHARED BY THE TWO CONSUMERS THAT ACTUALLY HAVE THEM.
//!
//! WHY THIS IS AN EXTRACTION AND NOT A NEW CAPABILITY. `codex_app_server_stdio_session` already
//! knew how to put a child in its own group and signal that group; the evaluation-budget falsifier
//! needs the same three facts. Copying them would be the second authoring of one concept, and
//! calling the Codex driver as a generic supervisor would be worse: its control flow joins the
//! stream readers and WAITS FOR THE CHILD before signalling, which is correct only because its
//! protocol makes Codex exit. A server that stays alive would deadlock there forever. So the
//! generic half moves here and each caller keeps its own protocol.
//!
//! WHY A GROUP AND NOT A PID. A server spawns helpers; signalling only the parent leaves the
//! children holding the port, and the next run of the same instrument then fails to bind for a
//! reason that has nothing to do with its subject.
//!
//! THIS IS NOT A MODELED PROCESS SESSION. The `.dag` substrate has no vocabulary for a live child
//! -- `extdeps.shell.exec` runs one command to completion and `std.process_termination` describes a
//! process that ENDED -- so this is host mechanism sitting where no model exists yet, not a bypass
//! of one that does. Its dissolution trigger is in `gunbc.seed_growth_admission`.

use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// WHAT A WAIT ACTUALLY OBSERVED, kept as three states because collapsing them is the exact defect
/// this repository keeps paying for: a timed-out wait and a process that exited nonzero are
/// different facts, and a classifier that returns a bare code for both cannot tell "the subject
/// refused" from "the instrument gave up".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessGroupWait {
    Exited { code: i32 },
    Signaled { signal: i32 },
    TimedOut,
    WaitFailed { detail: String },
}

/// Spawn with the child as leader of a NEW process group, so a later signal reaches its descendants
/// too. The `pre_exec` runs in the forked child before `exec`.
pub(crate) fn spawn_in_new_process_group(cmd: &mut Command) -> std::io::Result<Child> {
    use std::os::unix::process::CommandExt;
    unsafe {
        cmd.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    cmd.spawn()
}

/// Signal the leader AND the group. Both are sent because a child that never reached `pre_exec`
/// is not a group leader, so the group signal alone can miss it.
pub(crate) fn signal_process_group(pid: u32, signal: i32) {
    unsafe {
        let _ = libc::kill(pid as i32, signal);
        let _ = libc::kill(-(pid as i32), signal);
    }
}

/// Poll for termination up to a deadline WITHOUT blocking forever. `try_wait` is used rather than
/// `wait` precisely so a server that ignores the signal produces `TimedOut` -- an adjudicable
/// state -- instead of hanging the instrument that was supposed to judge it.
pub(crate) fn wait_for_exit(child: &mut Child, budget: Duration) -> ProcessGroupWait {
    let deadline = Instant::now() + budget;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return match status.code() {
                    Some(code) => ProcessGroupWait::Exited { code },
                    None => {
                        use std::os::unix::process::ExitStatusExt;
                        match status.signal() {
                            Some(signal) => ProcessGroupWait::Signaled { signal },
                            None => ProcessGroupWait::WaitFailed {
                                detail: "status carried neither an exit code nor a signal"
                                    .to_string(),
                            },
                        }
                    }
                }
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    return ProcessGroupWait::TimedOut;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return ProcessGroupWait::WaitFailed {
                    detail: format!("try_wait failed: {e}"),
                }
            }
        }
    }
}

/// TERM, then wait, then KILL, then wait again. The escalation exists because a polite signal a
/// process declines to honour must not become an instrument that never returns; the second wait's
/// verdict is reported as-is, so a group that survives SIGKILL is visible rather than assumed away.
pub(crate) fn terminate_process_group(
    child: &mut Child,
    pid: u32,
    grace: Duration,
) -> ProcessGroupWait {
    signal_process_group(pid, libc::SIGTERM);
    match wait_for_exit(child, grace) {
        ProcessGroupWait::TimedOut => {
            signal_process_group(pid, libc::SIGKILL);
            wait_for_exit(child, grace)
        }
        settled => settled,
    }
}
