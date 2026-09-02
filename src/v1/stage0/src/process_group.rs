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

/// ONE PROCESS AS /proc REPORTS IT. The pid and its process-group id are read from
/// `/proc/<pid>/stat`, which is what makes group membership decidable at all: a signal cannot ask
/// "who is in this group", it can only act on everyone who is.
struct ProcEntry {
    pid: i32,
    state: char,
}

/// Every process whose process-group id equals `pgid`, read from /proc.
///
/// The `comm` field may itself contain spaces and parentheses, so the fields after it are located
/// from the LAST ')' rather than by splitting the whole line -- splitting naively misreads any
/// process whose name contains a space, which is precisely the kind of quiet misparse that would
/// make this instrument report an empty group.
fn process_group_members(pgid: u32) -> Result<Vec<ProcEntry>, String> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir("/proc").map_err(|e| format!("reading /proc: {e}"))?;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let pid: i32 = match name.to_string_lossy().parse() {
            Ok(pid) => pid,
            Err(_) => continue,
        };
        // A process that exits between the readdir and the read is not an error: it is simply not
        // a member any more.
        let stat = match std::fs::read_to_string(entry.path().join("stat")) {
            Ok(stat) => stat,
            Err(_) => continue,
        };
        let tail = match stat.rfind(')') {
            Some(at) => &stat[at + 1..],
            None => continue,
        };
        let fields: Vec<&str> = tail.split_whitespace().collect();
        // After the ')' the fields are: state, ppid, pgrp, ...
        let (state, pgrp) = match (fields.first(), fields.get(2)) {
            (Some(state), Some(pgrp)) => (*state, *pgrp),
            _ => continue,
        };
        if pgrp.parse::<u32>().ok() == Some(pgid) {
            found.push(ProcEntry {
                pid,
                state: state.chars().next().unwrap_or('?'),
            });
        }
    }
    Ok(found)
}

/// WHAT REMAINED OF THE GROUP besides its leader. The leader is excluded because it is adjudicated
/// separately, by its exit status; this answers the different question of whether the server's
/// HELPERS outlived it -- the ones that would otherwise still hold the port when the next run of
/// this instrument tries to bind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessGroupResidue {
    /// No process other than the leader remains in the group.
    GroupAbsent,
    /// Members were still present when the observation budget ran out.
    GroupPresent { after: Duration, pids: Vec<i32> },
    /// The observation itself could not be made. This is NOT absence: an instrument that cannot
    /// see the group has not established that it is gone.
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
    /// Teardown succeeded only when nothing but the leader remained. A leader that exited cleanly
    /// while a child still holds the port is a FAILED teardown, and this is the single place that
    /// says so.
    pub(crate) fn group_is_gone(&self) -> bool {
        matches!(self.residue, ProcessGroupResidue::GroupAbsent)
    }
}

/// TERM the group, escalate to KILL WHILE THE IDENTITY IS STILL PINNED, observe what survived, and
/// only then reap the leader.
///
/// THE ORDER IS THE WHOLE SAFETY ARGUMENT, and an earlier version of this function got it wrong in
/// a way worth recording. It reaped the leader first and then signalled the numerically matching
/// group if anything appeared present. Once the leader is reaped its pid is free for reuse, so that
/// signal could land on an unrelated group that merely inherited the number -- wrong-subject
/// actuation, not a safe over-approximation. The annotation beside it claimed reuse "fails in the
/// safe direction", which was true of REFUSING on presence and false of the signalling the code
/// actually did.
///
/// An unreaped leader -- running or zombie -- keeps its pid allocated, and therefore keeps the
/// group identity pinned. So every signal this function sends happens before the reap, and after
/// the reap it neither signals nor observes: a surviving group is REFUSED to the caller instead.
pub(crate) fn terminate_process_group(
    child: &mut Child,
    pid: u32,
    grace: Duration,
) -> ProcessGroupTermination {
    signal_process_group(pid, libc::SIGTERM);

    // Membership is polled rather than waited on, because the question is about DESCENDANTS, and
    // there is no wait primitive for "the processes my child spawned".
    let survivors = |budget: Duration| -> Result<(Vec<i32>, bool), String> {
        let deadline = Instant::now() + budget;
        loop {
            let members = process_group_members(pid)?;
            let others: Vec<i32> = members
                .iter()
                .filter(|entry| entry.pid != pid as i32)
                .map(|entry| entry.pid)
                .collect();
            let leader_running = members
                .iter()
                .any(|entry| entry.pid == pid as i32 && entry.state != 'Z');
            if (others.is_empty() && !leader_running) || Instant::now() >= deadline {
                return Ok((others, leader_running));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    };

    let mut escalated_to_kill = false;
    let residue = match survivors(grace) {
        Err(detail) => ProcessGroupResidue::ResidueObservationFailed { detail },
        Ok((others, leader_running)) => {
            if others.is_empty() && !leader_running {
                ProcessGroupResidue::GroupAbsent
            } else {
                // Still pinned: the leader has not been reaped, so this number is still ours.
                signal_process_group(pid, libc::SIGKILL);
                escalated_to_kill = true;
                match survivors(grace) {
                    Err(detail) => ProcessGroupResidue::ResidueObservationFailed { detail },
                    Ok((others, _)) => {
                        if others.is_empty() {
                            ProcessGroupResidue::GroupAbsent
                        } else {
                            ProcessGroupResidue::GroupPresent {
                                after: grace,
                                pids: others,
                            }
                        }
                    }
                }
            }
        }
    };

    // The reap is LAST. Nothing below it may signal or observe the group by number.
    let leader = wait_for_exit(child, grace);
    ProcessGroupTermination {
        leader,
        residue,
        escalated_to_kill,
    }
}
