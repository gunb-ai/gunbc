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
// This teardown vocabulary is production-reachable through `codex_app_server_stdio_session`. It
// was originally gated with its falsifier-only consumer; the production wiring deliberately
// removes that gate rather than duplicating a weaker session-specific teardown path.
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
        // ONLY DISAPPEARANCE MAY BE SKIPPED. A process that exits between the readdir and the read
        // is genuinely no longer a member, and NotFound is how that is spelled. EVERY OTHER failure
        // means this numeric process COULD be in the target group and we could not tell -- skipping
        // it would let an unreadable /proc manufacture an empty vector and then `GroupAbsent`,
        // which is the instrument reporting "the group is gone" on evidence it never collected.
        let stat = match std::fs::read_to_string(entry.path().join("stat")) {
            Ok(stat) => stat,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("reading /proc/{pid}/stat: {e}")),
        };
        let tail = match stat.rfind(')') {
            Some(at) => &stat[at + 1..],
            None => return Err(format!("/proc/{pid}/stat has no comm terminator")),
        };
        let fields: Vec<&str> = tail.split_whitespace().collect();
        // After the ')' the fields are: state, ppid, pgrp, ...
        let (state, pgrp) = match (fields.first(), fields.get(2)) {
            (Some(state), Some(pgrp)) => (*state, *pgrp),
            _ => return Err(format!("/proc/{pid}/stat has no state and pgrp fields")),
        };
        let pgrp: u32 = match pgrp.parse() {
            Ok(pgrp) => pgrp,
            Err(e) => return Err(format!("/proc/{pid}/stat pgrp {pgrp:?}: {e}")),
        };
        if pgrp == pgid {
            let state = match state.chars().next() {
                Some(state) => state,
                None => return Err(format!("/proc/{pid}/stat has an empty state field")),
            };
            found.push(ProcEntry { pid, state });
        }
    }
    Ok(found)
}

/// WHAT THE LEADER IS DOING, OBSERVED WITHOUT REAPING IT.
///
/// `Child::try_wait` cannot answer this question, because asking it CONSUMES the exit status: after
/// it returns `Some`, the pid is released and any later signal or membership read addresses a number
/// that may already belong to somebody else. A caller that must tear the group down after noticing
/// the leader died therefore cannot use it. /proc can be read as often as we like and reaps nothing,
/// and a zombie is the positive observation -- exited, but still owning the pid, so the group
/// identity stays pinned for the teardown that follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LeaderObservation {
    Running,
    /// Exited and NOT yet reaped, so its pid still pins the group.
    ExitedUnreaped,
    /// The pid is not in /proc at all. For an unreaped child of this process this should not happen,
    /// and it is NOT reported as absence-of-group: something else released the identity.
    LeaderVanished,
    ObservationFailed {
        detail: String,
    },
}

pub(crate) fn observe_leader_without_reaping(pid: u32) -> LeaderObservation {
    let members = match process_group_members(pid) {
        Ok(members) => members,
        Err(detail) => return LeaderObservation::ObservationFailed { detail },
    };
    match members.iter().find(|entry| entry.pid == pid as i32) {
        Some(entry) if entry.state == 'Z' => LeaderObservation::ExitedUnreaped,
        Some(_) => LeaderObservation::Running,
        None => LeaderObservation::LeaderVanished,
    }
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
pub(crate) enum ProcessGroupTermination {
    /// EVERY REQUIRED FACT IS PRESENT, and this arm cannot be built without them. The leader is
    /// adjudicated -- exited or signalled, never timed out or unobserved -- AND nothing else in the
    /// group survived a COMPLETE observation.
    Settled {
        leader: SettledLeader,
        escalated_to_kill: bool,
    },
    /// Anything else, carrying what was actually seen.
    Unsettled {
        leader: ProcessGroupWait,
        residue: ProcessGroupResidue,
        escalated_to_kill: bool,
    },
}

/// The leader outcomes that can appear in a SETTLED teardown. `TimedOut` and `WaitFailed` are
/// deliberately absent: they are the states in which we do not know what the leader is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SettledLeader {
    Exited { code: i32 },
    Signaled { signal: i32 },
}

impl ProcessGroupTermination {
    /// THE SUCCESS QUESTION HAS ONE ANSWER AND IT IS THE CONSTRUCTOR. The previous shape was a
    /// struct plus a `group_is_gone()` predicate that consulted only `residue`, so a leader that had
    /// TIMED OUT -- still running, for all we knew -- could coexist with a "successful" teardown
    /// boolean, and every caller had to REMEMBER that the predicate omitted the leader. Making
    /// `Settled` unconstructible without an adjudicated leader removes the thing to remember.
    pub(crate) fn is_settled(&self) -> bool {
        matches!(self, ProcessGroupTermination::Settled { .. })
    }

    fn build(
        leader: ProcessGroupWait,
        residue: ProcessGroupResidue,
        escalated_to_kill: bool,
    ) -> ProcessGroupTermination {
        match (&leader, &residue) {
            (ProcessGroupWait::Exited { code }, ProcessGroupResidue::GroupAbsent) => {
                ProcessGroupTermination::Settled {
                    leader: SettledLeader::Exited { code: *code },
                    escalated_to_kill,
                }
            }
            (ProcessGroupWait::Signaled { signal }, ProcessGroupResidue::GroupAbsent) => {
                ProcessGroupTermination::Settled {
                    leader: SettledLeader::Signaled { signal: *signal },
                    escalated_to_kill,
                }
            }
            _ => ProcessGroupTermination::Unsettled {
                leader,
                residue,
                escalated_to_kill,
            },
        }
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
    ProcessGroupTermination::build(leader, residue, escalated_to_kill)
}

/// THE TEARDOWN TERMINAL'S FAILURE ARMS, on the ordinary merge path.
#[cfg(test)]
mod termination_terminal {
    use super::*;

    /// An unadjudicated leader may not appear in a SETTLED teardown, no matter how clean the
    /// residue looks. This is the case the old `group_is_gone()` predicate answered `true` for:
    /// it consulted only the residue, so a leader that was still running for all we knew coexisted
    /// with a "successful" teardown.
    #[test]
    fn an_unadjudicated_leader_cannot_construct_settled() {
        for leader in [
            ProcessGroupWait::TimedOut,
            ProcessGroupWait::WaitFailed {
                detail: "boom".to_string(),
            },
        ] {
            let terminal = ProcessGroupTermination::build(
                leader.clone(),
                ProcessGroupResidue::GroupAbsent,
                false,
            );
            assert!(
                !terminal.is_settled(),
                "an absent residue promoted {leader:?} to settled"
            );
        }

        // Positive controls: an adjudicated leader with an absent residue IS settled, so the
        // assertions above are not passing because nothing can ever settle.
        assert!(ProcessGroupTermination::build(
            ProcessGroupWait::Exited { code: 0 },
            ProcessGroupResidue::GroupAbsent,
            false
        )
        .is_settled());
        assert!(ProcessGroupTermination::build(
            ProcessGroupWait::Signaled { signal: 15 },
            ProcessGroupResidue::GroupAbsent,
            true
        )
        .is_settled());
    }

    /// Nor may an adjudicated leader settle over a residue that was never completely observed. An
    /// instrument that could not see the group has not established that the group is gone.
    #[test]
    fn an_unobserved_or_surviving_residue_cannot_construct_settled() {
        for residue in [
            ProcessGroupResidue::ResidueObservationFailed {
                detail: "/proc unreadable".to_string(),
            },
            ProcessGroupResidue::GroupPresent {
                after: Duration::from_secs(1),
                pids: vec![4242],
            },
        ] {
            assert!(!ProcessGroupTermination::build(
                ProcessGroupWait::Exited { code: 0 },
                residue.clone(),
                false
            )
            .is_settled());
        }
    }

    /// The readiness path must be able to notice a dead leader WITHOUT reaping it, because it tears
    /// the group down afterwards and a reaped pid may already belong to someone else.
    #[test]
    fn a_dead_leader_is_observed_as_exited_while_its_pid_is_still_pinned() {
        let mut command = Command::new("sh");
        command.arg("-c").arg("exit 7");
        let mut child = spawn_in_new_process_group(&mut command).expect("spawn");
        let pid = child.id();

        // Poll for the exit through /proc rather than through try_wait, which would reap it.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match observe_leader_without_reaping(pid) {
                LeaderObservation::ExitedUnreaped => break,
                LeaderObservation::Running if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20))
                }
                other => panic!("expected an unreaped exit, observed {other:?}"),
            }
        }
        // Still ours: the number can be addressed because nothing has reaped it. Replacing the
        // observation with Child::wait makes this assertion red.
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, 0);

        let terminal = terminate_process_group(&mut child, pid, Duration::from_secs(10));
        assert_eq!(
            terminal,
            ProcessGroupTermination::Settled {
                leader: SettledLeader::Exited { code: 7 },
                escalated_to_kill: false
            }
        );
    }
}
