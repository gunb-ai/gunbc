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
#[derive(Debug)]
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
    process_group_members_at(std::path::Path::new("/proc"), pgid)
}

/// The production wrapper's scan, against an arbitrary root.
fn process_group_members_at(root: &std::path::Path, pgid: u32) -> Result<Vec<ProcEntry>, String> {
    let entries = std::fs::read_dir(root)
        .map_err(|e| format!("reading {}: {e}", root.display()))?
        .map(|entry| {
            entry.map(|entry| {
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    entry.path(),
                )
            })
        });
    scan_process_group_members(entries, pgid)
}

/// THE SCAN'S TESTABLE CORE, over a FALLIBLE entry source.
///
/// The source is a parameter for one reason: `Err(_) => continue` on the directory iterator was a
/// live defect -- an entry that failed to enumerate might have named a member, so skipping it let an
/// incomplete scan report an empty group and hence `GroupAbsent`. A real `/proc` cannot be made to
/// yield an iterator error on demand, so with the iterator hard-wired the fix could only be asserted
/// by reading the source. DESIGN §4b: where no harness can express the subject, the missing harness
/// is the next-rung trigger -- so the harness is what gets built.
fn scan_process_group_members(
    entries: impl Iterator<Item = std::io::Result<(String, std::path::PathBuf)>>,
    pgid: u32,
) -> Result<Vec<ProcEntry>, String> {
    let mut found = Vec::new();
    for entry in entries {
        // AN UNREADABLE DIRECTORY ENTRY IS NOT AN ABSENT PROCESS.
        let (name, path) = match entry {
            Ok(entry) => entry,
            Err(e) => {
                return Err(format!(
                    "enumerating the process table: {e} -- the scan is incomplete, so absence is not established"
                ))
            }
        };
        // A NON-NUMERIC ENTRY IS NOT A PROCESS AT ALL. /proc carries `self`, `meminfo` and friends;
        // ignoring them is not skipping evidence, because they never named a process.
        let pid: i32 = match name.parse() {
            Ok(pid) => pid,
            Err(_) => continue,
        };
        // ONLY DISAPPEARANCE MAY BE SKIPPED. A process that exits between the readdir and the read
        // is genuinely no longer a member, and NotFound is how that is spelled. EVERY OTHER failure
        // means this numeric process COULD be in the target group and we could not tell.
        let stat = match std::fs::read_to_string(path.join("stat")) {
            Ok(stat) => stat,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("reading the stat of process {pid}: {e}")),
        };
        if let Some(entry) = parse_proc_stat(pid, &stat, pgid)? {
            found.push(entry);
        }
    }
    Ok(found)
}

/// PARSE ONE `stat` LINE, STRICTLY, and say whether it names a member of `pgid`.
///
/// Separated so the malformed forms have one place to be stated and one place to be tested. It is
/// strict about the fields it uses and about the identity it was handed: a line whose own pid prefix
/// disagrees with the directory it came from is not a record this scan may interpret.
fn parse_proc_stat(pid: i32, stat: &str, pgid: u32) -> Result<Option<ProcEntry>, String> {
    // The comm field is parenthesised and may itself contain spaces and parentheses, so the fields
    // after it are located from the LAST ')' rather than by splitting the whole line.
    let close = match stat.rfind(')') {
        Some(at) => at,
        None => return Err(format!("the stat of process {pid} has no comm terminator")),
    };
    let head = &stat[..close];
    let declared = head.split_whitespace().next().unwrap_or("");
    match declared.parse::<i32>() {
        Ok(declared) if declared == pid => {}
        Ok(declared) => {
            return Err(format!(
                "the stat of process {pid} declares pid {declared}; the record does not describe the process it was read from"
            ))
        }
        Err(e) => {
            return Err(format!(
                "the stat of process {pid} has an unparsable pid prefix {declared:?}: {e}"
            ))
        }
    }
    if !head.contains('(') {
        return Err(format!("the stat of process {pid} has no comm opener"));
    }
    let fields: Vec<&str> = stat[close + 1..].split_whitespace().collect();
    // After the ')' the fields are: state, ppid, pgrp, ...
    let (state, pgrp) = match (fields.first(), fields.get(2)) {
        (Some(state), Some(pgrp)) => (*state, *pgrp),
        _ => {
            return Err(format!(
                "the stat of process {pid} has no state and pgrp fields"
            ))
        }
    };
    let mut state_chars = state.chars();
    let (state, extra) = (state_chars.next(), state_chars.next());
    let state = match (state, extra) {
        (Some(state), None) => state,
        _ => {
            return Err(format!(
                "the stat of process {pid} has a state field {state:?} that is not one character"
            ))
        }
    };
    let pgrp: u32 = pgrp
        .parse()
        .map_err(|e| format!("the stat of process {pid} has pgrp {pgrp:?}: {e}"))?;
    if pgrp == pgid {
        Ok(Some(ProcEntry { pid, state }))
    } else {
        Ok(None)
    }
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

/// PROOF THAT A GROUP IDENTITY IS STILL OURS TO ADDRESS.
///
/// A bare `u32` cannot carry this fact, and that is the whole reason this type exists. A pid is
/// just a number: it says nothing about whether the process it named is still the one we started,
/// and after a reap the kernel may hand the same number to anybody. Every caller that wanted to
/// signal therefore had to REMEMBER to check first, and the review found two that did not -- a
/// readiness path that had observed the leader LEAVE /proc, and a generic arm whose own comment
/// said the failed wait had established nothing about the child, both of which then actuated on
/// the identity they had just failed to establish.
///
/// So the check moves into the type. This value can only be produced by an observation that found
/// the process still present and unreaped, and `terminate_process_group` cannot be called without
/// one. Losing the identity now fails to COMPILE into a signal rather than being a rule to follow.
pub(crate) struct PinnedProcessGroupIdentity {
    pid: u32,
}

impl PinnedProcessGroupIdentity {
    pub(crate) fn pid(&self) -> u32 {
        self.pid
    }
}

/// Establish that `pid` is still allocated and unreaped, and hand back the proof if it is.
///
/// `Err` carries what was observed instead, so a caller can report WHY it may not signal. It never
/// returns a proof for a vanished or unobservable process: those are exactly the states in which a
/// signal would be aimed at a number that may belong to somebody else.
pub(crate) fn pin_process_group_identity(
    pid: u32,
) -> Result<PinnedProcessGroupIdentity, LeaderObservation> {
    match observe_leader_without_reaping(pid) {
        LeaderObservation::Running | LeaderObservation::ExitedUnreaped => {
            Ok(PinnedProcessGroupIdentity { pid })
        }
        lost => Err(lost),
    }
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
/// needed. A caller adjudicating cleanup and a caller adjudicating the subject's exit ask different
/// questions of the same event, and both are answered from the state below.
/// THE TEARDOWN VERDICT, SEALED. The state is private, so no module outside this one can write a
/// `Settled` -- which the review correctly pointed out was still possible while the enum itself was
/// `pub(crate)`: the builder's join could simply be bypassed, and my own positive test did exactly
/// that. `Settled` is now reachable ONLY through `build`, which requires both an adjudicated leader
/// and a completely observed absence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessGroupTermination {
    state: ProcessGroupTerminationState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessGroupTerminationState {
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
        matches!(self.state, ProcessGroupTerminationState::Settled { .. })
    }

    /// What the teardown saw, for a caller that must report it. Read-only: there is no path from
    /// this back to a constructor.
    pub(crate) fn state_debug(&self) -> String {
        format!("{:?}", self.state)
    }

    /// The adjudicated leader outcome, available only when the sealed terminal settled.
    pub(crate) fn settled_leader(&self) -> Option<&SettledLeader> {
        match &self.state {
            ProcessGroupTerminationState::Settled { leader, .. } => Some(leader),
            ProcessGroupTerminationState::Unsettled { .. } => None,
        }
    }

    /// THE TEARDOWN THAT DID NOT HAPPEN, because the identity could not be proved still ours. It is
    /// an ordinary UNSETTLED verdict: nothing was signalled, nothing was observed, and no caller may
    /// read it as a clean group.
    #[cfg(test)]
    pub(crate) fn identity_lost() -> ProcessGroupTermination {
        ProcessGroupTermination {
            state: ProcessGroupTerminationState::Unsettled {
                leader: ProcessGroupWait::WaitFailed {
                    detail: "process-group identity could not be proved still pinned; no signal was sent".to_string(),
                },
                residue: ProcessGroupResidue::ResidueObservationFailed {
                    detail: "the group was never observed, because it was never safely addressable".to_string(),
                },
                escalated_to_kill: false,
            },
        }
    }

    fn build(
        leader: ProcessGroupWait,
        residue: ProcessGroupResidue,
        escalated_to_kill: bool,
    ) -> ProcessGroupTermination {
        let state = match (&leader, &residue) {
            (ProcessGroupWait::Exited { code }, ProcessGroupResidue::GroupAbsent) => {
                ProcessGroupTerminationState::Settled {
                    leader: SettledLeader::Exited { code: *code },
                    escalated_to_kill,
                }
            }
            (ProcessGroupWait::Signaled { signal }, ProcessGroupResidue::GroupAbsent) => {
                ProcessGroupTerminationState::Settled {
                    leader: SettledLeader::Signaled { signal: *signal },
                    escalated_to_kill,
                }
            }
            _ => ProcessGroupTerminationState::Unsettled {
                leader,
                residue,
                escalated_to_kill,
            },
        };
        ProcessGroupTermination { state }
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
    identity: &PinnedProcessGroupIdentity,
    grace: Duration,
) -> ProcessGroupTermination {
    let pid = identity.pid();
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

        let identity = pin_process_group_identity(pid).expect("the unreaped leader is still ours");
        let terminal = terminate_process_group(&mut child, &identity, Duration::from_secs(10));
        assert!(terminal.is_settled(), "observed {}", terminal.state_debug());
        assert!(terminal.state_debug().contains("Exited { code: 7 }"));
    }
}

/// THE /proc SCAN'S REFUSAL ARMS, made authorable by the fixture root.
///
/// I told the reviewing authority this RED could not be written, because the real `/proc` cannot be
/// made to return a malformed `stat`. That was a statement about the FUNCTION'S SIGNATURE, not about
/// the class -- the missing harness was the next-rung trigger, and adding a root parameter is what
/// discharges it (DESIGN §4b: ask whether the check's red is authorable BEFORE concluding it is not).
#[cfg(test)]
mod proc_scan_refusals {
    use super::*;

    fn fixture(entries: &[(&str, &str)]) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "gunbc-proc-fixture-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        std::fs::create_dir_all(&root).expect("fixture root");
        for (pid, stat) in entries {
            let dir = root.join(pid);
            std::fs::create_dir_all(&dir).expect("fixture pid dir");
            std::fs::write(dir.join("stat"), stat).expect("fixture stat");
        }
        root
    }

    /// The positive control. Without it every refusal below would also pass on a scan that refused
    /// everything, and the fixture's own shape would go unverified.
    #[test]
    fn a_well_formed_fixture_scan_finds_its_members() {
        let root = fixture(&[
            ("41", "41 (a name with spaces) S 1 41 41 0 -1 0"),
            ("42", "42 (other) Z 1 41 41 0 -1 0"),
            ("43", "43 (elsewhere) S 1 99 99 0 -1 0"),
        ]);
        let mut found = process_group_members_at(&root, 41).expect("a clean scan");
        found.sort_by_key(|entry| entry.pid);
        assert_eq!(found.len(), 2, "expected pids 41 and 42, got {found:?}");
        assert_eq!(found[0].pid, 41);
        assert_eq!(found[0].state, 'S');
        assert_eq!(found[1].pid, 42);
        // The zombie is what `observe_leader_without_reaping` reads as an unreaped exit.
        assert_eq!(found[1].state, 'Z');
        let _ = std::fs::remove_dir_all(&root);
    }

    /// THE ITERATOR-ERROR ARM. This is the one the `Err(_) => continue` defect lived on, and it is
    /// only reachable because the scan takes its entry source as a parameter.
    #[test]
    fn a_failed_directory_entry_refuses_rather_than_shortening_the_scan() {
        let entries = vec![
            Ok((
                "41".to_string(),
                std::path::PathBuf::from("/nonexistent-fixture/41"),
            )),
            Err(std::io::Error::other("the directory stream broke")),
        ];
        let scanned = scan_process_group_members(entries.into_iter(), 41);
        assert!(
            scanned.is_err(),
            "a failed entry was skipped, so an incomplete scan could report absence"
        );
    }

    /// THE POSITIVE CONTROL THE RULING NAMED: a group with no failures and no members really is
    /// absent. Without it, a scanner that refused every empty result would pass every RED here.
    #[test]
    fn a_clean_scan_with_no_members_reports_an_empty_group() {
        let root = fixture(&[("41", "41 (name) S 1 99 99 0 -1 0")]);
        let found = process_group_members_at(&root, 7).expect("a clean scan");
        assert!(
            found.is_empty(),
            "expected no members of group 7, got {found:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A non-numeric entry never named a process, so ignoring it is not skipping evidence. /proc is
    /// full of them.
    #[test]
    fn nonnumeric_entries_are_ignored_rather_than_refused() {
        let root = fixture(&[("41", "41 (name) S 1 41 41 0 -1 0")]);
        std::fs::write(root.join("meminfo"), "MemTotal: 1 kB").expect("a nonnumeric file");
        std::fs::create_dir_all(root.join("self")).expect("a nonnumeric directory");
        let found = process_group_members_at(&root, 41).expect("nonnumeric entries are not errors");
        assert_eq!(found.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A stat that fails to read for a reason OTHER than disappearance must refuse. A directory in
    /// place of the file produces exactly such an error and is not NotFound.
    #[test]
    fn a_non_notfound_stat_read_failure_refuses() {
        let root = fixture(&[("41", "41 (name) S 1 41 41 0 -1 0")]);
        let awkward = root.join("42");
        std::fs::create_dir_all(awkward.join("stat")).expect("a stat that is a directory");
        let scanned = process_group_members_at(&root, 41);
        assert!(
            scanned.is_err(),
            "an unreadable stat was skipped, so the scan could report absence"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A record whose own pid prefix disagrees with the directory it came from does not describe the
    /// process it was read from, so it may not be interpreted as that process's state.
    #[test]
    fn a_stat_declaring_a_different_pid_refuses() {
        let root = fixture(&[("41", "999 (name) S 1 41 41 0 -1 0")]);
        assert!(process_group_members_at(&root, 41).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A process we could not parse MIGHT be in the target group, so every malformed form refuses
    /// rather than being skipped into an empty -- and therefore "absent" -- result.
    #[test]
    fn an_unparsable_entry_refuses_rather_than_reporting_absence() {
        for (label, stat) in [
            ("no comm terminator", "41 unterminated S 1 41 41"),
            ("too few fields", "41 (name) S"),
            ("unparsable pgrp", "41 (name) S 1 not-a-number 41"),
            ("multi-character state", "41 (name) SS 1 41 41"),
            ("no comm opener", "41 name) S 1 41 41"),
        ] {
            let root = fixture(&[("41", stat)]);
            let scanned = process_group_members_at(&root, 41);
            assert!(
                scanned.is_err(),
                "{label}: an unparsable entry was skipped, so the scan could report absence"
            );
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    /// The ONE arm that may still be skipped: a process that genuinely vanished mid-scan. Its stat
    /// is NotFound, which is exactly how "it exited between the readdir and the read" is spelled.
    #[test]
    fn a_vanished_process_is_skipped_rather_than_refusing() {
        let root = fixture(&[("41", "41 (name) S 1 41 41 0 -1 0")]);
        std::fs::create_dir_all(root.join("42")).expect("a pid dir with no stat at all");
        let found = process_group_members_at(&root, 41).expect("a vanished entry is not an error");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].pid, 41);
        let _ = std::fs::remove_dir_all(&root);
    }
}
