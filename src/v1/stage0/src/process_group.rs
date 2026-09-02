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

/// WHAT REMAINED OF THE GROUP once teardown finished. This exists because the leader's exit status
/// answers a DIFFERENT question than the one teardown is asked: a server spawns helpers, and a
/// reaped leader beside a surviving child is exactly the state that holds the port and makes the
/// NEXT run of this instrument fail to bind for a reason unrelated to its subject. Absence is
/// observed with a null signal to the GROUP, never inferred from the leader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessGroupResidue {
    /// `kill(-pgid, 0)` reported ESRCH: no process remains in the group.
    GroupAbsent,
    /// Members were still present when the observation budget ran out.
    GroupPresent { after: Duration },
    /// The observation itself could not be made -- EPERM, or any errno that is not ESRCH. This is
    /// NOT absence: an instrument that cannot see the group has not established that it is gone.
    ResidueObservationFailed { detail: String },
}

/// The full teardown verdict: what the leader did, what survived it, and whether SIGKILL was
/// needed. Kept as three fields rather than one enum because a caller adjudicating cleanup and a
/// caller adjudicating the subject's exit are asking different questions of the same event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessGroupTermination {
    pub(crate) leader: ProcessGroupWait,
    pub(crate) residue: ProcessGroupResidue,
    pub(crate) escalated_to_kill: bool,
}

impl ProcessGroupTermination {
    /// Teardown succeeded only when nothing remains. A leader that exited cleanly while a child
    /// still holds the port is a FAILED teardown, and this is the single place that says so.
    pub(crate) fn group_is_gone(&self) -> bool {
        matches!(self.residue, ProcessGroupResidue::GroupAbsent)
    }
}

/// Observe whether ANY process remains in the group, by sending signal 0 to the negated pgid.
/// ESRCH is the only answer that establishes absence; every other errno is reported as an
/// observation failure rather than folded into presence, so "cannot tell" never reads as "gone".
fn observe_group_residue(pid: u32, budget: Duration) -> ProcessGroupResidue {
    let deadline = Instant::now() + budget;
    loop {
        let rc = unsafe { libc::kill(-(pid as i32), 0) };
        if rc == 0 {
            if Instant::now() >= deadline {
                return ProcessGroupResidue::GroupPresent { after: budget };
            }
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }
        let err = std::io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ESRCH) => ProcessGroupResidue::GroupAbsent,
            _ => ProcessGroupResidue::ResidueObservationFailed {
                detail: format!("kill(-{pid}, 0) failed: {err}"),
            },
        };
    }
}

/// TERM the group, reap the leader, then look for SURVIVORS -- and escalate to KILL on the group
/// whenever any remain, INCLUDING when the leader itself exited cleanly. The leader is waited for
/// first because an unreaped zombie is still a group member, so polling before the reap would
/// report presence for a process that is already dead.
///
/// PID REUSE IS A KNOWN, SAFE-DIRECTION RACE. Once the leader is reaped its pid may be recycled,
/// and a recycled pgid would be observed as presence. That mislabels a torn-down group as
/// surviving, which REFUSES; it can never report a surviving group as absent.
///
/// A group still standing after SIGKILL is returned as such. It is not retried and not assumed
/// away: the caller adjudicates it, because the only honest thing an instrument can say about a
/// process that survived SIGKILL is that it did.
pub(crate) fn terminate_process_group(
    child: &mut Child,
    pid: u32,
    grace: Duration,
) -> ProcessGroupTermination {
    signal_process_group(pid, libc::SIGTERM);
    let mut leader = wait_for_exit(child, grace);
    let mut residue = observe_group_residue(pid, grace);

    if matches!(residue, ProcessGroupResidue::GroupAbsent) {
        return ProcessGroupTermination {
            leader,
            residue,
            escalated_to_kill: false,
        };
    }

    signal_process_group(pid, libc::SIGKILL);
    if matches!(leader, ProcessGroupWait::TimedOut) {
        leader = wait_for_exit(child, grace);
    }
    residue = observe_group_residue(pid, grace);
    ProcessGroupTermination {
        leader,
        residue,
        escalated_to_kill: true,
    }
}
