//! THE DURABLE PRODUCT FALSIFIER FOR THE EVALUATION-BUDGET CONSEQUENCE BRIDGE.
//!
//! WHAT IT ESTABLISHES, and it is one thing: that a production `evaluation_budget_exceeded`
//! response carries its machine identity BECAUSE the generated projection of
//! `std.evaluation_budget evaluation_budget_refusal_code` carries it -- not because the serving
//! boundary independently chose the same text. Unperturbed, every check in this transaction is
//! green whether or not the seed reads the projection at all, since the value the boundary would
//! have chosen on its own is the same value. MOVING THE AUTHORITY IS THE WHOLE INSTRUMENT.
//!
//! WHY IT IS HOST RUST AND NOT A MODELED ACTUATOR. The `.dag` substrate has no vocabulary for a
//! live child process: `extdeps.shell.exec` runs one command to completion and returns its status,
//! and `std.process_termination` describes a process that already ended. There is no handle, no
//! readiness, no later termination. Writing this as a modeled actuator would mean either inventing
//! a managed-process substrate whose only consumer is this receipt -- substrate authored to make a
//! receipt look modeled -- or hiding start/readiness/request/kill inside one opaque command, which
//! puts the adjudication somewhere no fold can see it. The out-of-band-actuation tell is about
//! bypassing a modeled operation that EXISTS; here the missing fact is that it does not.
//! Dissolution trigger is in `gunbc.seed_growth_admission`: a real modeled consumer introduces a
//! process-session capability and this instrument migrates onto it.
//!
//! IT MUTATES SOURCE, SO IT MUTATES A DISPOSABLE COPY. The transaction perturbs an authority file,
//! regenerates, rebuilds and serves. None of that touches the originating checkout: it runs in a
//! detached worktree at the bound commit with its own CARGO_TARGET_DIR, and the parent's HEAD,
//! tree and status are re-verified at the end, and this instrument's own worktree is required to
//! be gone from the registration list. The WHOLE list is deliberately not compared: this repository
//! is shared, other sessions add and remove worktrees while the transaction runs, and a check whose
//! red is dominated by events outside its subject trains a reader to ignore the real one.
//!
//! EVERY OUTCOME IS TYPED. `Result<_, String>` at the adjudication boundary would make "the gate
//! refused for the wrong reason" and "the instrument could not run" the same value. Detail strings
//! live INSIDE the typed causes, never in place of them.

use crate::process_group::{
    pin_process_group_identity, spawn_in_new_process_group, terminate_process_group,
    ProcessGroupTermination, ProcessGroupWait,
};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// WHAT A SUBPROCESS DID, kept as four states. A nonzero exit and a timeout are different facts,
/// and this distinction is here because the loop that first measured this receipt by hand wrapped
/// each run in `timeout` and reported ANY nonzero exit as a failed verdict -- so a slow run and a
/// false witness were the same observation. Two rows reported red and were green.
#[derive(Debug, Clone)]
pub(crate) enum CommandObservation {
    Completed {
        exit_code: i32,
        stdout: StreamOutcome,
        stderr: StreamOutcome,
    },
    /// Overran its budget AND its process group is gone. The teardown verdict travels WITH the
    /// timeout because "the command overran" and "the instrument left it running" are different
    /// facts about the same event, and the second one invalidates every later step.
    TimedOut {
        after: Duration,
        termination: ProcessGroupTermination,
        stdout: StreamOutcome,
        stderr: StreamOutcome,
    },
    /// Overran its budget and something SURVIVED teardown. Separate from `TimedOut` because this
    /// arm may never be treated as a mere non-zero result: a surviving group holds resources the
    /// next step assumes are free.
    TimedOutWithTerminationFailure {
        after: Duration,
        termination: ProcessGroupTermination,
        stdout: StreamOutcome,
        stderr: StreamOutcome,
    },
    Signaled {
        signal: i32,
        stdout: StreamOutcome,
        stderr: StreamOutcome,
    },
    /// The command never started. Reserved for spawn itself failing.
    SpawnRefused { detail: String },
    /// The command started and the WAIT failed. Previously folded into `SpawnRefused`, which said
    /// the opposite of what happened -- a process that ran and could not be observed was reported
    /// as one that never ran, and its remedy is the reverse.
    WaitFailedAfterSpawn {
        detail: String,
        /// The teardown VERDICT, not a formatted rendering of it. The timeout arms preserve this
        /// algebra and this arm was throwing it away into a debug string, so a caller could not
        /// ask whether the group was actually gone without parsing prose.
        termination: ProcessGroupTermination,
        stdout: StreamOutcome,
        stderr: StreamOutcome,
    },
}

/// THE REFUSAL ALGEBRA. Each arm is a place this transaction can stop being about its subject, and
/// they are separate because the remedies differ: a dirty parent checkout is an operator problem, a
/// wrong drift population is a defect in the bridge, and a failed teardown is an instrument that
/// must not report a verdict at all.
#[derive(Debug, Clone)]
pub(crate) enum EvaluationBudgetConsequenceRefusal {
    SourceCheckoutNotClean {
        status: String,
    },
    WorktreeCreationFailed {
        observation: CommandObservation,
    },
    AuthorityOccurrenceNotExactlyOne {
        occurrences: usize,
    },
    UnexpectedChangedPath {
        paths: Vec<String>,
    },
    DryGateDidNotComplete {
        observation: CommandObservation,
    },
    DryGateDriftPopulationWrong {
        detail: String,
    },
    RegenerationFailed {
        generation: u8,
        observation: CommandObservation,
    },
    FixedPointNotReached {
        detail: String,
    },
    CandidateProductBuildFailed {
        observation: CommandObservation,
    },
    /// THE EXPERIMENT'S OWN WORKTREE STOPPED BEING ONE, mid-run, from outside.
    ///
    /// A linked worktree is a directory whose `.git` FILE points at an admin directory under the
    /// common `.git/worktrees/`. Any process sharing that repository can delete it -- `git worktree
    /// prune` does, routinely, and on a machine where many sessions share one clone it is not even
    /// unusual. The directory survives; only its registration dies. Every `git` call inside it then
    /// fails, and because the seed's build script hard-requires `git rev-parse HEAD` to stamp the
    /// source commit, the FIRST thing to fail is the candidate build -- so an event with no
    /// relation to the subject arrives wearing CandidateProductBuildFailed's name.
    ///
    /// This arm exists because that misattribution is the failure this whole instrument is against:
    /// charging the subject for something the subject did not do. Observed 2026-09-02 at
    /// 17f3808cba, where the admin directory was gone while the worktree directory remained.
    ExperimentWorktreeUnregistered {
        detail: String,
    },
    ServeSpawnFailed {
        detail: String,
    },
    ServeExitedBeforeReadiness {
        termination: ProcessGroupTermination,
        stdout: String,
        stderr: String,
    },
    ReadinessTimedOut {
        after: Duration,
        termination: ProcessGroupTermination,
        stdout: String,
        stderr: String,
    },
    /// A required digest could not be taken. An instrument that cannot identify its own artifacts
    /// does not get to describe what they did.
    DigestUnavailable {
        detail: String,
    },
    /// The rebuilt server binary is byte-identical to the one that produced the dry red. The
    /// generated consequence changed, so the binary embedding it MUST change; equal digests mean
    /// the rebuild did not take, and every later observation would be the old producer's.
    SubjectBinaryUnchanged {
        digest: String,
    },
    /// The server announced a contract this run did not arm, or its announcement could not be
    /// read. Distinct from a readiness TIMEOUT because a server saying something else and a server
    /// saying nothing have different causes and different remedies.
    ReadinessObservationFailed {
        cause: String,
        termination: ProcessGroupTermination,
        stdout: String,
        stderr: String,
    },
    RequestFailed {
        detail: String,
    },
    ResponseDisagreed {
        expectation: String,
        observed: String,
        /// The subject's own output for the exchange that disagreed. Retained because the first
        /// question asked of a disagreement is what the server thought it was doing, and a
        /// refusal that drops it sends the reader back to re-run a 30-minute transaction.
        serve_stdout: String,
        serve_stderr: String,
    },
    ProcessTerminationFailed {
        termination: ProcessGroupTermination,
    },
    /// There was no child left to tear down at the point teardown was due. This is an instrument
    /// defect, not a subject verdict, and it refuses rather than being read as a clean teardown.
    ProcessTerminationUnobservable {
        detail: String,
    },
    AuthorityRestorationFailed {
        detail: String,
    },
    RestoredTreeDisagreed {
        detail: String,
    },
    WorktreeCleanupFailed {
        observation: CommandObservation,
    },
    /// A GIT OBSERVATION THAT DID NOT COMPLETE, kept apart from every verdict derived from git's
    /// output. `stdout_of` used to answer `String::new()` for a timed-out or signalled command, so
    /// a git that never ran produced an empty HEAD which then compared unequal and was reported as
    /// ParentCheckoutChanged -- a located, confident, WRONG subject. The instrument must not name a
    /// culprit on evidence it failed to collect.
    GitObservationFailed {
        what: &'static str,
        observation: CommandObservation,
    },
    /// A MEMBER THE RESPONSE DID NOT CARRY. Absent is not empty: defaulting a missing `code` to ""
    /// and then comparing it merely reports a disagreement about the wrong thing, and defaulting a
    /// missing `limit_ms` to 0 invents a number the server never sent.
    /// A STREAM THAT DID NOT END. The serve process's own account of the breach is evidence, and a
    /// truncated prefix is not that account -- so an incomplete drain refuses rather than being
    /// compared as though it were the whole output.
    ServeStreamsIncomplete {
        stdout: StreamOutcome,
        stderr: StreamOutcome,
    },
    ResponseMemberAbsent {
        member: &'static str,
        body: String,
    },
    ParentCheckoutChanged {
        detail: String,
    },
}

/// The receipt a PASS carries. It names what was bound, so a green is auditable rather than a word.
#[derive(Debug, Clone)]
pub(crate) struct EvaluationBudgetConsequenceReceipt {
    pub(crate) subject_commit: String,
    pub(crate) subject_tree: String,
    pub(crate) moved_value: String,
    pub(crate) original_value: String,
    pub(crate) serve_status: u16,
    pub(crate) moved_occurrences: usize,
    pub(crate) former_occurrences: usize,
    pub(crate) elapsed_nanos: u128,
    /// WHICH BINARIES WERE ON DISK AT EACH STEP -- and deliberately NOT which image a process had
    /// loaded. `file_digest` hashes the file at a path; it does not inspect the executable mapped
    /// into the serve child. The earlier heading here said "which binaries actually answered",
    /// which claimed process-incarnation identity on evidence that only establishes on-disk
    /// artifact identity, and the gap is real: a process could in principle be running an image
    /// that no longer matches the file it was launched from.
    ///
    /// The narrower claim is the one this rung rests on and it is sufficient for it: the dry-gate
    /// and serving digests DIFFER, which establishes that the rebuild changed the artifact rather
    /// than the run having reused one binary throughout -- they occupy the same path, and the
    /// second overwrites the first. Binding the loaded image itself (via /proc/<pid>/exe, or an
    /// inherited executable handle) would strengthen this to process-incarnation identity and is
    /// not claimed here.
    pub(crate) orchestrator_digest: String,
    pub(crate) dry_gate_gunbc_digest: String,
    pub(crate) serving_gunbc_digest: String,
    pub(crate) generated_artifact_digest: String,
    /// The child's own listening announcement, which is what binds the connection this receipt
    /// describes to the process this run started rather than to whoever held the port.
    pub(crate) serve_announcement: String,
    /// The subject process's diagnostic for the breach, required to agree with the body.
    pub(crate) serve_diagnostic: String,
}

/// THE TERMINAL. A cleanup failure never overwrites the experiment's own cause: both are carried,
/// because "the subject was wrong" and "the instrument left a mess" are separately actionable and
/// the second must not be able to hide the first.
#[derive(Debug, Clone)]
pub(crate) enum EvaluationBudgetConsequenceFalsifierOutcome {
    Passed(EvaluationBudgetConsequenceReceipt),
    Refused(EvaluationBudgetConsequenceRefusal),
    RefusedWithCleanupFailure {
        primary: EvaluationBudgetConsequenceRefusal,
        cleanup: EvaluationBudgetConsequenceRefusal,
    },
    /// A PASSING TRANSACTION WHOSE CLEANUP DID NOT SETTLE. It is not a pass -- an instrument that
    /// leaves residue has not finished -- but the receipt is CARRIED rather than discarded. The
    /// first run of this instrument lost exactly that information: cleanup refused, the match arm
    /// replaced the outcome wholesale, and the terminal could no longer say whether the product
    /// transaction had succeeded. Two facts, two fields.
    PassedWithCleanupFailure {
        receipt: EvaluationBudgetConsequenceReceipt,
        cleanup: EvaluationBudgetConsequenceRefusal,
    },
}

/// THE DIGESTS TAKEN BEFORE THE PERTURBATION, carried forward as one value. They are grouped
/// rather than passed loose because they share a single property that a loose parameter list would
/// not express: both were taken while the tree was still unperturbed, and neither can be re-taken
/// afterwards -- the binary is overwritten in place and the orchestrator is the process running.
struct PrePerturbationDigests {
    orchestrator: String,
    dry_gate_gunbc: String,
}

/// Is the experiment's worktree still a registered worktree? Answered by resolving its `.git`
/// pointer, because that is the thing an outside prune removes -- not by asking git from inside,
/// which fails for this reason and several unrelated ones and so cannot discriminate.
fn worktree_admin_area_present(worktree: &Path) -> Result<(), String> {
    let pointer = worktree.join(".git");
    let text = std::fs::read_to_string(&pointer)
        .map_err(|e| format!("reading {}: {e}", pointer.display()))?;
    let gitdir = text
        .lines()
        .find_map(|line| line.strip_prefix("gitdir: "))
        .ok_or_else(|| format!("{} carries no gitdir pointer", pointer.display()))?
        .trim()
        .to_string();
    if Path::new(&gitdir).is_dir() {
        Ok(())
    } else {
        Err(format!(
            "the worktree's admin directory {gitdir} no longer exists, so {} is no longer a registered worktree; another process sharing this repository pruned it",
            worktree.display()
        ))
    }
}

/// SHA-256 of a file's bytes AT A PATH. Used to give the receipt an on-disk identity for each
/// artifact it depends on, so "the tool that noticed" and "the tool that served" are distinguishable
/// after the fact rather than only assertable at the time. It says nothing about the image any
/// process actually loaded -- see the receipt's digest fields for what is and is not claimed.
fn file_digest(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

const AUTHORITY_REL: &str = "dag/std/evaluation_budget.dag";
const GENERATED_REL: &str = "src/v1/stage0/src/evaluation_budget_consequence_generated.rs";
const GATE_ENTRY: &str = "dag/gunbc/instruments/generated_artifact_gate.dag";
const BREACH_FIXTURE: &str = "dag/test/fixture/serve_seam_probe.dag";
const BREACH_FUNCTION: &str = "serve_budget_breach_probe";
const CPU_LIMIT_MS: u64 = 50;

fn observe(mut command: Command, budget: Duration) -> CommandObservation {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match spawn_in_new_process_group(&mut command) {
        Ok(child) => child,
        Err(e) => {
            return CommandObservation::SpawnRefused {
                detail: format!("{e}"),
            }
        }
    };
    let pid = child.id();
    let mut out_drain = PipeDrain::spawn(child.stdout.take());
    let mut err_drain = PipeDrain::spawn(child.stderr.take());
    let wait = crate::process_group::wait_for_exit(&mut child, budget);

    // THE DRAIN IS BOUNDED ON EVERY ARM. It is read on all of them, because the child's own account
    // of a failure is exactly what an operator needs on the arms that are not the happy one -- and
    // it is bounded because on the arms where teardown FAILED, a surviving descendant still holds
    // the write end and EOF never comes. An unbounded join there would hang the instrument at
    // precisely the moment it had something to report.
    let drain_budget = Duration::from_secs(10);

    match wait {
        ProcessGroupWait::Exited { code } => CommandObservation::Completed {
            exit_code: code,
            stdout: out_drain.finish_within(drain_budget),
            stderr: err_drain.finish_within(drain_budget),
        },
        ProcessGroupWait::Signaled { signal } => CommandObservation::Signaled {
            signal,
            stdout: out_drain.finish_within(drain_budget),
            stderr: err_drain.finish_within(drain_budget),
        },
        ProcessGroupWait::TimedOut => {
            // A command that overran its budget is terminated here rather than left running, so a
            // timeout cannot leak a process into the next step's environment -- and whether that
            // termination actually SUCCEEDED is carried in the observation rather than discarded.
            let termination = match pin_process_group_identity(pid) {
                Ok(identity) => Some(terminate_process_group(
                    &mut child,
                    &identity,
                    Duration::from_secs(10),
                )),
                Err(_) => None,
            };
            let stdout = out_drain.finish_within(drain_budget);
            let stderr = err_drain.finish_within(drain_budget);
            // An identity we cannot prove is ours is not torn down at all. `None` therefore reports
            // an UNSETTLED teardown, never a successful one.
            let termination = match termination {
                Some(termination) => termination,
                None => {
                    return CommandObservation::TimedOutWithTerminationFailure {
                        after: budget,
                        termination: ProcessGroupTermination::identity_lost(),
                        stdout,
                        stderr,
                    }
                }
            };
            if termination.is_settled() {
                CommandObservation::TimedOut {
                    after: budget,
                    termination,
                    stdout,
                    stderr,
                }
            } else {
                CommandObservation::TimedOutWithTerminationFailure {
                    after: budget,
                    termination,
                    stdout,
                    stderr,
                }
            }
        }
        ProcessGroupWait::WaitFailed { detail } => {
            // The child may still be running: a wait that failed established nothing about it --
            // which is exactly why this arm may not actuate on the identity it failed to establish.
            // It re-establishes the identity first, and refuses to signal when it cannot.
            let termination = match pin_process_group_identity(pid) {
                Ok(identity) => {
                    terminate_process_group(&mut child, &identity, Duration::from_secs(10))
                }
                Err(_) => ProcessGroupTermination::identity_lost(),
            };
            CommandObservation::WaitFailedAfterSpawn {
                detail,
                termination,
                stdout: out_drain.finish_within(drain_budget),
                stderr: err_drain.finish_within(drain_budget),
            }
        }
    }
}

/// SUCCESS IS EXIT ZERO **AND** BOTH STREAMS COMPLETE. Zero exit alone was accepted here while the
/// stream terminals were ignored, so a command whose output was truncated -- or whose reader failed
/// -- still counted as a clean success, and its partial text was then read as the whole answer.
fn completed_zero(observation: &CommandObservation) -> bool {
    matches!(
        observation,
        CommandObservation::Completed { exit_code: 0, stdout, stderr }
            if stdout.is_complete() && stderr.is_complete()
    )
}

fn git(workdir: &Path, args: &[&str], budget: Duration) -> CommandObservation {
    let mut command = Command::new("git");
    command.args(args).current_dir(workdir);
    observe(command, budget)
}

/// THE ONE TEXT-BEARING SUCCESS EXTRACTOR. `stdout_of` used to stand here answering `String::new()`
/// for any non-`Completed` observation and the raw text for a `Completed` one whose streams were
/// truncated -- so a timed-out, signalled, or half-read command produced text that the next
/// comparison treated as the command's answer. It is DELETED rather than fixed in place: leaving the
/// unsafe spelling available beside a safe one is how call sites keep finding it.
///
/// `Err` carries the whole observation, so a refusal can say what actually happened.
fn completed_drained(observation: &CommandObservation) -> Result<(i32, String, String), ()> {
    match observation {
        CommandObservation::Completed {
            exit_code,
            stdout,
            stderr,
        } if stdout.is_complete() && stderr.is_complete() => Ok((
            *exit_code,
            stdout.text().to_string(),
            stderr.text().to_string(),
        )),
        _ => Err(()),
    }
}

/// READ GIT'S OUTPUT, OR REFUSE -- never both silently. Every caller that DERIVES a verdict from
/// git's stdout goes through here, so a command that timed out, was signalled, or could not be
/// waited on becomes `GitObservationFailed` naming what was being read, instead of an empty string
/// that the next comparison confidently misattributes. There is no unadjudicated spelling left to
/// reach for: `completed_drained` is the only extractor, and it refuses a truncated stream.
fn git_stdout(
    workdir: &Path,
    args: &[&str],
    budget: Duration,
    what: &'static str,
) -> Result<String, EvaluationBudgetConsequenceRefusal> {
    let observation = git(workdir, args, budget);
    // ALL THREE FACTS, JOINED. Accepting any `Completed` was the residue of this repair's first
    // attempt: a nonzero `rev-parse` with empty stdout, or a zero-exit command whose stdout was only
    // an unfinished prefix, still produced an apparently successful read that the next comparison
    // then attributed to an innocent subject -- the very failure this function was added to remove.
    if completed_zero(&observation) {
        if let CommandObservation::Completed { stdout, .. } = &observation {
            return Ok(stdout.text().to_string());
        }
    }
    Err(EvaluationBudgetConsequenceRefusal::GitObservationFailed { what, observation })
}

/// LAST-RESORT TEARDOWN ONLY. The explicit finalizer below produces the ADJUDICABLE cleanup
/// receipt; this exists so an unwind between spawn and teardown cannot leave a server holding the
/// port. A `Drop` that reported verdicts would be a second, invisible adjudication path.
struct ServeGuard {
    child: Option<std::process::Child>,
    pid: u32,
}

impl Drop for ServeGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // THE GUARD OBEYS THE SAME ALGEBRA AS THE EXPLICIT PATHS. Removing the signal from the
            // identity-lost arms above would be pointless if destruction then sent the same numeric
            // signal invisibly -- a drop cannot report anything, so an unprovable identity here
            // would be the quietest possible wrong-subject actuation. If the pin fails the child is
            // left unadjudicated, which is worse than a clean teardown and far better than
            // signalling whoever inherited the number.
            match pin_process_group_identity(self.pid) {
                Ok(identity) => {
                    let _ = terminate_process_group(&mut child, &identity, Duration::from_secs(10));
                }
                // Reap our own handle without signalling: the number is no longer provably ours.
                Err(_) => {
                    let _ = child.wait();
                }
            }
        }
    }
}

/// A PIPE THAT IS DRAINED WHILE THE CHILD RUNS. The falsifier needs the server's output for two
/// different purposes at two different times -- the listening announcement DURING readiness, and
/// the breach diagnostic AFTER the request -- so the reader cannot be a thread whose result is
/// only available once the pipe closes. It appends into a shared buffer that either purpose can
/// read at any moment, and blocking is impossible because the reader never stops.
struct PipeDrain {
    text: Arc<Mutex<String>>,
    /// HOW THE READER STOPPED, written by the reader itself. Without this the thread merely
    /// finishing is indistinguishable from EOF, so a read error became "complete output".
    end: Arc<Mutex<Option<Result<(), String>>>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl PipeDrain {
    fn spawn<R: Read + Send + 'static>(pipe: Option<R>) -> PipeDrain {
        let text = Arc::new(Mutex::new(String::new()));
        let end: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        let sink = Arc::clone(&text);
        let end_sink = Arc::clone(&end);
        let handle = std::thread::spawn(move || {
            // EOF AND FAILURE ARE RECORDED SEPARATELY. They used to share one `return`, so a reader
            // that died on a read error left a finished thread that the terminal below then called
            // `Closed` -- a truncated stream judged complete.
            let outcome = match pipe {
                None => Ok(()),
                Some(pipe) => {
                    let mut reader = std::io::BufReader::new(pipe);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line) {
                            Ok(0) => break Ok(()),
                            Err(e) => break Err(format!("reading child stream: {e}")),
                            Ok(_) => {
                                let mut held = match sink.lock() {
                                    Ok(held) => held,
                                    Err(poisoned) => poisoned.into_inner(),
                                };
                                held.push_str(&line);
                            }
                        }
                    }
                }
            };
            let mut slot = match end_sink.lock() {
                Ok(slot) => slot,
                Err(poisoned) => poisoned.into_inner(),
            };
            *slot = Some(outcome);
        });
        PipeDrain {
            text,
            end,
            handle: Some(handle),
        }
    }

    /// Wait for the reader to reach EOF, but only up to a budget, and report WHICH happened.
    ///
    /// An unbounded join is not safe here and that is not hypothetical: a descendant of a child
    /// that survived teardown still holds the write end of the pipe, so EOF never arrives and the
    /// join blocks forever -- the instrument hangs on exactly the arm where teardown already
    /// failed. The thread is left detached on the unfinished arm rather than killed, because a
    /// reader blocked on a pipe cannot be interrupted; what matters is that this function returns.
    fn finish_within(&mut self, budget: Duration) -> StreamOutcome {
        let deadline = Instant::now() + budget;
        loop {
            let finished = match &self.handle {
                None => true,
                Some(handle) => handle.is_finished(),
            };
            if finished {
                // JOINED, not merely observed finished. A panicking reader also finishes, and only
                // the join distinguishes it from one that reached EOF.
                if let Some(handle) = self.handle.take() {
                    if handle.join().is_err() {
                        return StreamOutcome::ReaderPanicked {
                            partial: self.snapshot(),
                        };
                    }
                }
                let end = {
                    let slot = match self.end.lock() {
                        Ok(slot) => slot,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    slot.clone()
                };
                return match end {
                    Some(Ok(())) => StreamOutcome::Closed {
                        text: self.snapshot(),
                    },
                    Some(Err(cause)) => StreamOutcome::ReadFailed {
                        partial: self.snapshot(),
                        cause,
                    },
                    // Finished and joined without recording an end: the reader did not run to its
                    // own terminal, which is not something to interpret as EOF.
                    None => StreamOutcome::ReaderPanicked {
                        partial: self.snapshot(),
                    },
                };
            }
            if Instant::now() >= deadline {
                return StreamOutcome::DeadlineExceeded {
                    partial: self.snapshot(),
                    after: budget,
                };
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    /// A POISONED LOCK IS NOT AN EMPTY STREAM. The previous `unwrap_or_default()` answered "" for a
    /// poisoned mutex, which is a silent widen in the one place the text is the evidence; the data
    /// behind a poisoned lock is still there and is returned.
    fn snapshot(&self) -> String {
        match self.text.lock() {
            Ok(text) => text.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

/// WHETHER A STREAM ACTUALLY ENDED, and if not, WHY. `Closed` is the only arm whose text is the
/// child's complete output; `ReadFailed`, `DeadlineExceeded` and `ReaderPanicked` each carry a
/// PREFIX and say what stopped the reader. Collapsing any of them into `Closed` would let a
/// truncated read be judged as a complete one -- which is precisely what a single `Unfinished` arm,
/// and before it a bare thread return, allowed.
#[derive(Debug, Clone)]
pub(crate) enum StreamOutcome {
    /// EOF was reached and the text is the child's COMPLETE output.
    Closed { text: String },
    /// The reader failed mid-stream; the text is a PREFIX.
    ReadFailed { partial: String, cause: String },
    /// Something still held the write end when the budget ran out; the text is a PREFIX.
    DeadlineExceeded { partial: String, after: Duration },
    /// The reader thread did not reach its own terminal; the text is whatever had arrived.
    ReaderPanicked { partial: String },
}

impl StreamOutcome {
    pub(crate) fn text(&self) -> &str {
        match self {
            StreamOutcome::Closed { text } => text,
            StreamOutcome::ReadFailed { partial, .. } => partial,
            StreamOutcome::DeadlineExceeded { partial, .. } => partial,
            StreamOutcome::ReaderPanicked { partial } => partial,
        }
    }

    /// Whether this text may be treated as the stream's WHOLE content. Only one arm may.
    pub(crate) fn is_complete(&self) -> bool {
        matches!(self, StreamOutcome::Closed { .. })
    }
}

/// WHAT READINESS ACTUALLY OBSERVED. A Boolean stood here, and it could not tell a server that
/// never listened from one that exited, nor either from a FOREIGN listener that answered on a port
/// this instrument had merely guessed. Those have different remedies, and only one of them is a
/// fact about the subject.
#[derive(Debug)]
enum ServeReadiness {
    /// The owned child announced its own listening address, is still running, and a connection to
    /// the ANNOUNCED port succeeded. All three, jointly.
    Ready { port: u16, announcement: String },
    ExitedBeforeReady {
        termination: ProcessGroupTermination,
        stdout: String,
        stderr: String,
    },
    ReadinessTimedOut {
        after: Duration,
        stdout: String,
        stderr: String,
    },
    /// The observation itself could not be made -- an announcement that did not match the contract
    /// this instrument armed, or a port field that did not parse. Never folded into "not ready":
    /// a server announcing something ELSE is a different fact from a server announcing nothing.
    ReadinessObservationFailed {
        cause: String,
        stdout: String,
        stderr: String,
    },
}

/// The announcement `gunbc serve` prints once it has bound. Every field is pinned except the port,
/// which is what the OS chose and what this instrument is trying to learn -- so matching the line
/// establishes that the listener belongs to the contract THIS run armed, and not merely that
/// something is listening somewhere.
fn announced_port(line: &str, prefix: &str, suffix: &str) -> Option<u16> {
    let rest = line.strip_prefix(prefix)?;
    let digits = rest.strip_suffix(suffix)?;
    digits.parse::<u16>().ok()
}

/// Readiness is a JOINT observation of three facts, none of which implies the others: the owned
/// child is still running, it emitted the exact announcement for the contract that was armed, and
/// a connection to the port IT named succeeds.
///
/// The port is not guessed. Asking the OS for one (`--port 0`) and learning it from the child's own
/// announcement is what makes the connection attributable: a guessed port can already be held by an
/// unrelated local listener, which would satisfy a connect-only readiness wall and send the
/// instrument on to blame the subject for a response the subject never sent.
fn await_serve_ready(
    child: &mut std::process::Child,
    pid: u32,
    out: &PipeDrain,
    err: &PipeDrain,
    prefix: &str,
    suffix: &str,
    budget: Duration,
) -> ServeReadiness {
    let deadline = Instant::now() + budget;
    loop {
        // The child's own state first: a process that has exited cannot become ready later, and
        // reading the pipes first would race a server that died between the two observations.
        // NOT `try_wait`. Asking it CONSUMES the exit status, and this arm must tear the group down
        // AFTER noticing the leader died -- so a reap here would release the pid and hand the
        // following signal a number that may already belong to somebody else. That is the exact
        // wrong-subject actuation `terminate_process_group` is ordered to avoid, and this call site
        // was violating its precondition. /proc reaps nothing, so the identity stays pinned until
        // `terminate_process_group` does the reap itself, last.
        match crate::process_group::observe_leader_without_reaping(pid) {
            crate::process_group::LeaderObservation::ExitedUnreaped => {
                // Still unreaped, so the identity is provably ours and the teardown may signal.
                let termination = match pin_process_group_identity(pid) {
                    Ok(identity) => {
                        terminate_process_group(child, &identity, Duration::from_secs(10))
                    }
                    Err(_) => ProcessGroupTermination::identity_lost(),
                };
                return ServeReadiness::ExitedBeforeReady {
                    termination,
                    stdout: out.snapshot(),
                    stderr: err.snapshot(),
                };
            }
            crate::process_group::LeaderObservation::Running => {}
            // The pid left /proc without us reaping it. We no longer own the identity, so this
            // refuses instead of signalling or inferring group membership from a released number.
            crate::process_group::LeaderObservation::LeaderVanished => {
                return ServeReadiness::ReadinessObservationFailed {
                    cause: format!("serve leader {pid} left /proc before it was reaped"),
                    stdout: out.snapshot(),
                    stderr: err.snapshot(),
                }
            }
            crate::process_group::LeaderObservation::ObservationFailed { detail } => {
                return ServeReadiness::ReadinessObservationFailed {
                    cause: format!("observing serve leader during readiness: {detail}"),
                    stdout: out.snapshot(),
                    stderr: err.snapshot(),
                }
            }
        }

        let stderr = err.snapshot();
        if let Some(line) = stderr
            .lines()
            .find(|line| line.starts_with("gunbc serve listening on"))
        {
            let port = match announced_port(line, prefix, suffix) {
                Some(port) => port,
                None => {
                    return ServeReadiness::ReadinessObservationFailed {
                        cause: format!(
                            "the server announced a contract this run did not arm.\n  announced: {line}\n  expected:  {prefix}<port>{suffix}"
                        ),
                        stdout: out.snapshot(),
                        stderr,
                    }
                }
            };
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return ServeReadiness::Ready {
                    port,
                    announcement: line.to_string(),
                };
            }
        }

        if Instant::now() >= deadline {
            return ServeReadiness::ReadinessTimedOut {
                after: budget,
                stdout: out.snapshot(),
                stderr,
            };
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// A DECODED HTTP RESPONSE. Status, headers and a PARSED body are three separate observations, and
/// the previous shape returned only the first and the raw third -- so a response that was not JSON
/// at all, or was JSON of some other shape, reached the field checks as a string in which
/// substrings happened or did not happen to appear.
struct BreachResponse {
    status: u16,
    content_type: Option<String>,
    body: String,
    fields: JsonObject,
}

/// The smallest object decoder that can answer this instrument's questions HONESTLY: it either
/// parses the whole top-level object or refuses. `find("\"code\":\"")` could not tell a field from
/// the same text appearing inside another string, and could not tell a missing field from a
/// malformed document.
#[derive(Debug, Default)]
struct JsonObject {
    strings: std::collections::BTreeMap<String, String>,
    numbers: std::collections::BTreeMap<String, u128>,
}

impl JsonObject {
    fn string(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(|s| s.as_str())
    }
    fn number(&self, key: &str) -> Option<u128> {
        self.numbers.get(key).copied()
    }
}

/// Parse a flat JSON object of string and unsigned-integer members. Nested values and arrays are
/// REFUSED rather than skipped: this instrument asserts about a document whose shape it knows, and
/// silently tolerating a shape it does not know is how a check stops discriminating.
/// A DIAGNOSTIC PREVIEW THAT CANNOT PANIC. Slicing by BYTE offset -- `&text[..text.len().min(200)]`
/// -- panics when the cut lands inside a multibyte character, so a malformed document containing
/// any non-ASCII text could kill the instrument while it was building the typed refusal that was
/// supposed to describe it. Counting CHARACTERS cannot land mid-character.
fn preview(text: &str) -> String {
    text.chars().take(200).collect()
}

/// DECODE RESPONSE BYTES, OR REFUSE. Factored out so the invalid-UTF-8 arm has an authorable red:
/// the response declares charset=utf-8, and `from_utf8_lossy` would repair broken bytes into U+FFFD
/// and let the instrument compare a body the server never sent.
fn decode_response_bytes(raw: Vec<u8>) -> Result<String, String> {
    String::from_utf8(raw).map_err(|e| format!("response is not valid UTF-8: {e}"))
}

/// WHERE THE DECODER IS INSIDE THE OBJECT, so that "a comma is allowed here" is a state rather than
/// a thing the loop re-decides from whatever character it happens to see.
enum JsonMemberPosition {
    First,
    AfterMember,
    AfterComma,
}

fn parse_flat_json_object(text: &str) -> Result<JsonObject, String> {
    let mut chars = text.trim().chars().peekable();
    if chars.next() != Some('{') {
        return Err(format!("body is not a JSON object: {}", preview(text)));
    }
    let mut object = JsonObject::default();
    // SEPARATORS ARE REQUIRED, EXACTLY ONCE, BETWEEN MEMBERS. The previous loop consumed a comma
    // wherever it found one and also accepted a fresh quoted key with no comma before it, so
    // `{,"a":1}`, `{"a":1,}`, `{"a":1,,"b":2}` and `{"a":1 "b":2}` were all read as well-formed.
    // A decoder that accepts documents JSON rejects is not establishing "the body is JSON"; it is
    // establishing "the body is something this function tolerates".
    let mut expect = JsonMemberPosition::First;
    loop {
        while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
            chars.next();
        }
        match (&expect, chars.peek()) {
            // Only an EMPTY object may close here; after a comma a close is a trailing separator.
            (JsonMemberPosition::First, Some('}'))
            | (JsonMemberPosition::AfterMember, Some('}')) => {
                chars.next();
                break;
            }
            (JsonMemberPosition::AfterMember, Some(',')) => {
                chars.next();
                expect = JsonMemberPosition::AfterComma;
                continue;
            }
            (JsonMemberPosition::First, Some('"'))
            | (JsonMemberPosition::AfterComma, Some('"')) => {}
            (JsonMemberPosition::AfterMember, other) => {
                return Err(format!(
                    "expected ',' or '}}' after a member, found {other:?}"
                ))
            }
            (_, other) => return Err(format!("expected a member name, found {other:?}")),
        }
        expect = JsonMemberPosition::AfterMember;
        let key = read_json_string(&mut chars)?;
        while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
            chars.next();
        }
        if chars.next() != Some(':') {
            return Err(format!("member {key} is not followed by ':'"));
        }
        while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
            chars.next();
        }
        match chars.peek() {
            Some('"') => {
                let value = read_json_string(&mut chars)?;
                // A DUPLICATE MEMBER IS REFUSED, not last-one-wins. A response carrying two `code`
                // members has two answers to a question this instrument treats as having one, and
                // silently taking either is the instrument choosing which claim to believe.
                if object.strings.contains_key(&key) || object.numbers.contains_key(&key) {
                    return Err(format!("member {key} appears more than once"));
                }
                object.strings.insert(key, value);
            }
            Some(c) if c.is_ascii_digit() => {
                let mut digits = String::new();
                while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
                    digits.push(chars.next().unwrap_or_default());
                }
                // JSON HAS NO LEADING ZEROS. `01` parses perfectly well as 1, which is exactly the
                // problem: the decoder would accept a document JSON rejects and then report the
                // number as though the body had been well formed.
                if digits.len() > 1 && digits.starts_with('0') {
                    return Err(format!(
                        "member {key} carries {digits}, which has a leading zero"
                    ));
                }
                let value = digits
                    .parse::<u128>()
                    .map_err(|e| format!("member {key} is not an unsigned integer: {e}"))?;
                if object.strings.contains_key(&key) || object.numbers.contains_key(&key) {
                    return Err(format!("member {key} appears more than once"));
                }
                object.numbers.insert(key, value);
            }
            other => {
                return Err(format!(
                    "member {key} carries {other:?}, which this decoder does not model"
                ))
            }
        }
    }
    // THE WHOLE DOCUMENT, not a prefix of it. Bytes after the closing brace mean the response is
    // not the object it appeared to be, and accepting a valid-looking prefix would let a body carry
    // anything at all after the fields this instrument checks.
    let trailing: String = chars.collect();
    if !trailing.trim().is_empty() {
        return Err(format!(
            "trailing bytes after the closing brace: {}",
            preview(&trailing)
        ));
    }
    Ok(object)
}

fn read_json_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, String> {
    if chars.next() != Some('"') {
        return Err("expected an opening quote".to_string());
    }
    let mut out = String::new();
    loop {
        match chars.next() {
            None => return Err("unterminated string".to_string()),
            Some('"') => return Ok(out),
            Some('\\') => match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(c @ ('"' | '\\' | '/')) => out.push(c),
                other => return Err(format!("unmodelled escape \\{other:?}")),
            },
            // RAW CONTROL CHARACTERS ARE NOT LEGAL IN A JSON STRING. Accepting U+0000..U+001F
            // unescaped means the decoder is admitting documents JSON rejects while the receipt
            // claims the body IS JSON.
            Some(c) if (c as u32) < 0x20 => {
                return Err(format!(
                    "unescaped control character U+{:04X} inside a string",
                    c as u32
                ))
            }
            Some(c) => out.push(c),
        }
    }
}

/// One HTTP request over a raw socket, decoded. `curl` is deliberately not spawned: the response
/// has to be judged field by field, and a shell string would put the adjudication back inside an
/// opaque command. Headers are retained rather than discarded, because a 500 carrying the right
/// text with the wrong content type is a different product than the one this receipt describes.
fn post_for_breach(port: u16, budget: Duration) -> Result<BreachResponse, String> {
    let mut stream =
        std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| format!("connect: {e}"))?;
    stream
        .set_read_timeout(Some(budget))
        .map_err(|e| format!("set_read_timeout: {e}"))?;
    let body = "receipt";
    let request = format!(
        "POST /burn HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read: {e}"))?;
    // NOT LOSSY. The response declares `charset=utf-8`, so invalid bytes are a broken response, and
    // replacing them with U+FFFD would let the instrument compare a repaired body against the
    // authority value and call the agreement real.
    let text = decode_response_bytes(raw)?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| format!("no header/body boundary: {}", preview(&text)))?;
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| {
            format!(
                "no status line in response: {}",
                &head[..head.len().min(200)]
            )
        })?;
    let content_type = lines.find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-type")
            .then(|| value.trim().to_string())
    });
    let fields = parse_flat_json_object(body)?;
    Ok(BreachResponse {
        status,
        content_type,
        body: body.to_string(),
        fields,
    })
}

/// THE TRANSACTION. Ten steps, each of which can only refuse in its own typed way.
///
/// STEP ORDER IS LOAD-BEARING IN ONE PLACE: the candidate tool is built BEFORE the authority is
/// perturbed, so the binary that produces the expected drift is known to predate the change it is
/// asked to notice. Built afterwards, a green would be consistent with a tool that had simply been
/// regenerated into agreement.
pub(crate) fn run_evaluation_budget_consequence_falsifier(
    repo_root: &Path,
    scratch: &Path,
) -> EvaluationBudgetConsequenceFalsifierOutcome {
    let long = Duration::from_secs(3600);
    let short = Duration::from_secs(120);

    // 1. BIND AND ISOLATE. A dirty parent means the "restored tree" comparison at the end would be
    // against a moving target, so it refuses here rather than producing an unfalsifiable green.
    let status = git(repo_root, &["status", "--porcelain"], short);
    // AN OBSERVATION FAILURE IS NOT A DIRTY CHECKOUT. This arm used to read `stdout_of`, so a git
    // timeout, wait failure, signal, or failed reader became `SourceCheckoutNotClean` -- usually
    // with an EMPTY status string, naming the source checkout as the culprit on evidence that was
    // never collected. It is the same class as the empty-HEAD defect, one function along.
    let status_text = match completed_drained(&status) {
        Ok((0, stdout, _)) => stdout,
        Ok(_) | Err(()) => {
            return EvaluationBudgetConsequenceFalsifierOutcome::Refused(
                EvaluationBudgetConsequenceRefusal::GitObservationFailed {
                    what: "parent status",
                    observation: status,
                },
            )
        }
    };
    if !status_text.trim().is_empty() {
        return EvaluationBudgetConsequenceFalsifierOutcome::Refused(
            EvaluationBudgetConsequenceRefusal::SourceCheckoutNotClean {
                status: status_text,
            },
        );
    }
    let head = match git_stdout(repo_root, &["rev-parse", "HEAD"], short, "parent HEAD") {
        Ok(v) => v,
        Err(refusal) => return EvaluationBudgetConsequenceFalsifierOutcome::Refused(refusal),
    }
    .trim()
    .to_string();
    let tree = match git_stdout(
        repo_root,
        &["rev-parse", "HEAD^{tree}"],
        short,
        "parent tree",
    ) {
        Ok(v) => v,
        Err(refusal) => return EvaluationBudgetConsequenceFalsifierOutcome::Refused(refusal),
    }
    .trim()
    .to_string();
    // DELIBERATELY NOT A WHOLE-INVENTORY SNAPSHOT. The first run of this instrument compared
    // `git worktree list` before and after and refused ParentCheckoutChanged -- correctly by its
    // own rule and wrongly as a fact, because this repository is shared: other sessions add and
    // remove worktrees while the transaction runs, and none of that is something this instrument
    // caused or can control. A check whose red is dominated by events outside its subject is not a
    // wall, it is a source of false refusals that would train a reader to ignore the real one. What
    // this transaction OWNS is its own worktree, so that is what is verified gone.

    let worktree = scratch.join(format!("ebc-falsifier-{}", std::process::id()));
    let worktree_str = worktree.to_string_lossy().into_owned();
    let created = git(
        repo_root,
        &["worktree", "add", "--detach", &worktree_str, &head],
        long,
    );
    if !completed_zero(&created) {
        return EvaluationBudgetConsequenceFalsifierOutcome::Refused(
            EvaluationBudgetConsequenceRefusal::WorktreeCreationFailed {
                observation: created,
            },
        );
    }

    let outcome = run_bound_transaction(&worktree, &head, &tree, long, short);

    // 10. RESTORE AND REMOVE, unconditionally. The primary cause survives a cleanup failure.
    let cleanup = finalize(
        repo_root,
        &worktree,
        &worktree_str,
        &head,
        &tree,
        long,
        short,
    );
    match (outcome, cleanup) {
        (out, None) => out,
        (EvaluationBudgetConsequenceFalsifierOutcome::Refused(primary), Some(cleanup)) => {
            EvaluationBudgetConsequenceFalsifierOutcome::RefusedWithCleanupFailure {
                primary,
                cleanup,
            }
        }
        (EvaluationBudgetConsequenceFalsifierOutcome::Passed(receipt), Some(cleanup)) => {
            EvaluationBudgetConsequenceFalsifierOutcome::PassedWithCleanupFailure {
                receipt,
                cleanup,
            }
        }
        (already_paired, Some(_)) => already_paired,
    }
}

#[allow(clippy::too_many_arguments)]
fn finalize(
    repo_root: &Path,
    worktree: &Path,
    worktree_str: &str,
    head: &str,
    tree: &str,
    long: Duration,
    short: Duration,
) -> Option<EvaluationBudgetConsequenceRefusal> {
    // The worktree is removed with --force because the transaction deliberately leaves it dirty on
    // failure; refusing to clean up a known-dirty disposable copy would strand the operator with a
    // directory whose only purpose was to be thrown away.
    if worktree.exists() {
        let removed = git(
            repo_root,
            &["worktree", "remove", "--force", worktree_str],
            long,
        );
        if !completed_zero(&removed) {
            // GIT DECLINING TO REMOVE IT IS NOT THE SAME FACT AS THE DIRECTORY SURVIVING, and this
            // repository is a SHARED checkout: a `git worktree prune` run by any other session
            // deregisters this instrument's worktree mid-transaction, after which `worktree remove`
            // answers "is not a working tree" for a directory that is still very much there. That is
            // an environmental event about the registry, not a fact about the subject -- the same
            // distinction `ExperimentWorktreeUnregistered` already draws on the build path, drawn
            // here on the teardown path rather than duplicated as a second notion of the same thing.
            //
            // The obligation this function actually owes is that OUR directory is gone and no
            // registration of ours survives. When git has already forgotten the path, discharging
            // that obligation is ours to finish, so we remove the tree directly. This does NOT widen
            // the refusal: every post-condition below still runs unchanged, and a directory or a
            // registration that survives this still refuses. The path is our own scratch directory
            // under a name this run generated, so nothing else can be addressed by it.
            let deregistered = matches!(
                &removed,
                CommandObservation::Completed { exit_code, stderr, .. }
                    // The stderr must be COMPLETE before its text decides anything: a truncated
                    // error stream that happens to contain this phrase, or one that lost it to
                    // truncation, would both mis-route the teardown.
                    if *exit_code != 0
                        && stderr.is_complete()
                        && stderr.text().contains("is not a working tree")
            );
            if deregistered {
                let _ = std::fs::remove_dir_all(worktree);
            } else {
                return Some(EvaluationBudgetConsequenceRefusal::WorktreeCleanupFailed {
                    observation: removed,
                });
            }
        }
    }
    let _ = git(repo_root, &["worktree", "prune"], short);
    if worktree.exists() {
        return Some(EvaluationBudgetConsequenceRefusal::WorktreeCleanupFailed {
            observation: CommandObservation::Completed {
                exit_code: 0,
                stdout: StreamOutcome::Closed {
                    text: format!(
                        "{} still present after remove and prune",
                        worktree.display()
                    ),
                },
                stderr: StreamOutcome::Closed {
                    text: String::new(),
                },
            },
        });
    }
    // THE PARENT IS RE-VERIFIED, not assumed. An instrument that mutates a copy must prove it did
    // not mutate the original, and "we intended not to" is not that proof.
    let head_after = match git_stdout(
        repo_root,
        &["rev-parse", "HEAD"],
        short,
        "parent HEAD after",
    ) {
        Ok(v) => v,
        Err(refusal) => return Some(refusal),
    }
    .trim()
    .to_string();
    let tree_after = match git_stdout(
        repo_root,
        &["rev-parse", "HEAD^{tree}"],
        short,
        "parent tree after",
    ) {
        Ok(v) => v,
        Err(refusal) => return Some(refusal),
    }
    .trim()
    .to_string();
    let status_after = match git_stdout(
        repo_root,
        &["status", "--porcelain"],
        short,
        "parent status after",
    ) {
        Ok(v) => v,
        Err(refusal) => return Some(refusal),
    };
    if head_after != head || tree_after != tree || !status_after.trim().is_empty() {
        return Some(EvaluationBudgetConsequenceRefusal::ParentCheckoutChanged {
            detail: format!(
                "head {head} -> {head_after}, tree {tree} -> {tree_after}, status {:?}",
                status_after
            ),
        });
    }
    // ITS OWN ROW, AND ONLY ITS OWN. A surviving registration for this instrument's worktree is a
    // leak it caused; every other row in that list belongs to somebody else.
    let inventory_after = match git_stdout(
        repo_root,
        &["worktree", "list"],
        short,
        "worktree inventory after",
    ) {
        Ok(v) => v,
        Err(refusal) => return Some(refusal),
    };
    if inventory_after.contains(worktree_str) {
        return Some(EvaluationBudgetConsequenceRefusal::WorktreeCleanupFailed {
            observation: CommandObservation::Completed {
                exit_code: 0,
                stdout: StreamOutcome::Closed {
                    text: format!("{worktree_str} still registered after remove and prune"),
                },
                stderr: StreamOutcome::Closed {
                    text: String::new(),
                },
            },
        });
    }
    None
}

fn cargo_in(
    worktree: &Path,
    target_dir: &Path,
    args: &[&str],
    budget: Duration,
) -> CommandObservation {
    // THE CARGO THAT INVOKED THIS TEST, NEVER WHATEVER PATH OFFERS. This instrument's whole claim
    // is same-identity evidence -- that the binary it judges was produced from the source it
    // perturbed -- so silently building with a DIFFERENT toolchain than the one running the test is
    // precisely the substitution it exists to detect. Cargo always sets CARGO for a test process;
    // its absence means this is not being run the way the receipt assumes, and that is a refusal
    // rather than a default.
    let cargo = match std::env::var("CARGO") {
        Ok(path) => path,
        Err(_) => {
            return CommandObservation::SpawnRefused {
                detail: "CARGO is unset: refusing to PATH-search a cargo whose identity is unknown"
                    .to_string(),
            }
        }
    };
    let mut command = Command::new(cargo);
    command
        .args(args)
        .current_dir(worktree)
        // ITS OWN TARGET DIRECTORY. Sharing the test process's target would let this instrument
        // overwrite the artifacts of the run that invoked it.
        .env("CARGO_TARGET_DIR", target_dir);
    observe(command, budget)
}

fn gunbc_run(
    worktree: &Path,
    binary: &Path,
    entry: &str,
    function: &str,
    budget: Duration,
) -> CommandObservation {
    let mut command = Command::new(binary);
    command
        .args([
            "run",
            "--source-root",
            "dag",
            "--source-root",
            "src/v2",
            "--entry",
            entry,
            "--function",
            function,
        ])
        .current_dir(worktree);
    observe(command, budget)
}

#[allow(clippy::too_many_lines)]
fn run_bound_transaction(
    worktree: &Path,
    head: &str,
    tree: &str,
    long: Duration,
    short: Duration,
) -> EvaluationBudgetConsequenceFalsifierOutcome {
    use EvaluationBudgetConsequenceFalsifierOutcome as Outcome;
    use EvaluationBudgetConsequenceRefusal as Refusal;

    let target_dir = worktree.join("target-falsifier");
    let gunbc = target_dir.join("release").join("gunbc");

    // 2. BUILD THE UNPERTURBED CANDIDATE TOOL, before anything moves. Its digest identifies the
    // ARTIFACT ON DISK that produced the dry red -- not the image any process loaded, which this
    // receipt does not claim -- and an ambient binary on PATH is never that artifact.
    let built = cargo_in(
        worktree,
        &target_dir,
        &["build", "--release", "--bin", "gunbc"],
        long,
    );
    if !completed_zero(&built) {
        // A build cannot be charged to the subject until the environment it ran in is known to
        // have still been intact. This ordering is the whole point: ask first, attribute second.
        if let Err(detail) = worktree_admin_area_present(worktree) {
            return Outcome::Refused(Refusal::ExperimentWorktreeUnregistered { detail });
        }
        return Outcome::Refused(Refusal::CandidateProductBuildFailed { observation: built });
    }

    // The digest is taken HERE, while this binary is still the only one that has existed at this
    // path. Taken later it would be the rebuilt one, and the receipt would carry the wrong on-disk
    // artifact for the dry red with no way for a reader to notice.
    let dry_gate_gunbc_digest = match file_digest(&gunbc) {
        Ok(digest) => digest,
        Err(detail) => return Outcome::Refused(Refusal::DigestUnavailable { detail }),
    };
    let orchestrator_digest = match std::env::current_exe()
        .map_err(|e| format!("locating the orchestrating test executable: {e}"))
        .and_then(|exe| file_digest(&exe))
    {
        Ok(digest) => digest,
        Err(detail) => return Outcome::Refused(Refusal::DigestUnavailable { detail }),
    };

    // 3. PERTURB EXACTLY ONE FACT, by locating the value rather than by running a blind sed and
    // reading exit zero as success.
    let authority_path = worktree.join(AUTHORITY_REL);
    let original_source = match std::fs::read_to_string(&authority_path) {
        Ok(text) => text,
        Err(e) => {
            return Outcome::Refused(Refusal::AuthorityRestorationFailed {
                detail: format!("reading {AUTHORITY_REL}: {e}"),
            })
        }
    };
    let original_value =
        match json_free_literal_after(&original_source, "fn evaluation_budget_refusal_code") {
            Some(value) => value,
            None => {
                return Outcome::Refused(Refusal::AuthorityOccurrenceNotExactlyOne {
                    occurrences: 0,
                })
            }
        };
    let quoted = format!("\"{original_value}\"");
    let occurrences = original_source.matches(&quoted).count();
    if occurrences != 1 {
        return Outcome::Refused(Refusal::AuthorityOccurrenceNotExactlyOne { occurrences });
    }
    let moved_value = format!("{original_value}_moved_probe_{}", std::process::id());
    let perturbed = original_source.replace(&quoted, &format!("\"{moved_value}\""));
    if let Err(e) = std::fs::write(&authority_path, &perturbed) {
        return Outcome::Refused(Refusal::AuthorityRestorationFailed {
            detail: format!("writing perturbed {AUTHORITY_REL}: {e}"),
        });
    }
    let changed = match git_stdout(
        worktree,
        &["status", "--porcelain"],
        short,
        "perturbed worktree status",
    ) {
        Ok(v) => v,
        Err(refusal) => return Outcome::Refused(refusal),
    };
    let changed_paths: Vec<String> = changed
        .lines()
        .map(|line| {
            line.split_whitespace()
                .last()
                .unwrap_or_default()
                .to_string()
        })
        .filter(|p| !p.is_empty() && !p.starts_with("target-falsifier"))
        .collect();
    if changed_paths != vec![AUTHORITY_REL.to_string()] {
        return Outcome::Refused(Refusal::UnexpectedChangedPath {
            paths: changed_paths,
        });
    }

    // 4. THE ATTRIBUTED DRY RED. A timeout, a signal or a refused spawn is NOT the expected red,
    // and neither is a drift naming some other artifact.
    let dry = gunbc_run(worktree, &gunbc, GATE_ENTRY, "main", long);
    let (dry_code, dry_stdout, dry_stderr) = match &dry {
        CommandObservation::Completed {
            exit_code,
            stdout,
            stderr,
        } if stdout.is_complete() && stderr.is_complete() => (
            *exit_code,
            stdout.text().to_string(),
            stderr.text().to_string(),
        ),
        // A COMPLETED GATE WHOSE OUTPUT WAS TRUNCATED IS NOT A GATE RESULT. The drift population is
        // counted from this text, so a prefix carrying one expected drift line would establish
        // "exactly one named drift" while the rest of the output was never seen.
        _ => return Outcome::Refused(Refusal::DryGateDidNotComplete { observation: dry }),
    };
    let dry_text = format!("{dry_stdout}{dry_stderr}");
    if dry_code == 0 {
        return Outcome::Refused(Refusal::DryGateDriftPopulationWrong {
            detail: "the gate accepted a perturbed authority with no regeneration".to_string(),
        });
    }
    let drift_lines: Vec<&str> = dry_text
        .lines()
        .filter(|line| line.contains("committed content differs from authority"))
        .collect();
    if drift_lines.len() != 1
        || !drift_lines[0].contains("evaluation_budget_consequence_generated.rs")
    {
        return Outcome::Refused(Refusal::DryGateDriftPopulationWrong {
            detail: format!("drift lines: {drift_lines:?}"),
        });
    }

    // 5. THE TWO-GENERATION CYCLE. One regeneration cannot stand in for it: the crate-layout
    // emitter reads the installed mirror whose bytes the first generation changes.
    let first = gunbc_run(worktree, &gunbc, GATE_ENTRY, "main_wet", long);
    if !completed_zero(&first) {
        return Outcome::Refused(Refusal::RegenerationFailed {
            generation: 1,
            observation: first,
        });
    }
    let generated = match std::fs::read_to_string(worktree.join(GENERATED_REL)) {
        Ok(text) => text,
        Err(e) => {
            return Outcome::Refused(Refusal::RegenerationFailed {
                generation: 1,
                observation: CommandObservation::SpawnRefused {
                    detail: format!("reading {GENERATED_REL}: {e}"),
                },
            })
        }
    };
    if !generated.contains(&moved_value) {
        return Outcome::Refused(Refusal::FixedPointNotReached {
            detail: format!("generated constant does not carry {moved_value}"),
        });
    }
    let rebuilt = cargo_in(
        worktree,
        &target_dir,
        &["build", "--release", "--bin", "gunbc"],
        long,
    );
    if !completed_zero(&rebuilt) {
        if let Err(detail) = worktree_admin_area_present(worktree) {
            return Outcome::Refused(Refusal::ExperimentWorktreeUnregistered { detail });
        }
        return Outcome::Refused(Refusal::CandidateProductBuildFailed {
            observation: rebuilt,
        });
    }
    let second = gunbc_run(worktree, &gunbc, GATE_ENTRY, "main_wet", long);
    if !completed_zero(&second) {
        return Outcome::Refused(Refusal::RegenerationFailed {
            generation: 2,
            observation: second,
        });
    }
    let settled = gunbc_run(worktree, &gunbc, GATE_ENTRY, "main", long);
    if !completed_zero(&settled) {
        return Outcome::Refused(Refusal::FixedPointNotReached {
            detail: "the gate still refuses after two generations".to_string(),
        });
    }

    serve_and_judge(
        worktree,
        &gunbc,
        PrePerturbationDigests {
            orchestrator: orchestrator_digest,
            dry_gate_gunbc: dry_gate_gunbc_digest,
        },
        head,
        tree,
        &original_value,
        &moved_value,
        &original_source,
        &authority_path,
        &target_dir,
        long,
        short,
    )
}

/// Locate the string literal a zero-argument authority function returns. Reading the value out of
/// the source is what makes step 3's "exactly one occurrence" check meaningful: a hardcoded
/// expectation here would silently stop matching the day the authority changed, and the instrument
/// would then perturb nothing while still reporting a green.
fn json_free_literal_after(source: &str, marker: &str) -> Option<String> {
    let start = source.find(marker)? + marker.len();
    let rest = &source[start..];
    let open = rest.find('"')? + 1;
    let tail = &rest[open..];
    let close = tail.find('"')?;
    Some(tail[..close].to_string())
}

#[allow(clippy::too_many_arguments)]
fn serve_and_judge(
    worktree: &Path,
    gunbc: &Path,
    pre: PrePerturbationDigests,
    head: &str,
    tree: &str,
    original_value: &str,
    moved_value: &str,
    original_source: &str,
    authority_path: &Path,
    target_dir: &Path,
    long: Duration,
    short: Duration,
) -> EvaluationBudgetConsequenceFalsifierOutcome {
    use EvaluationBudgetConsequenceFalsifierOutcome as Outcome;
    use EvaluationBudgetConsequenceRefusal as Refusal;

    // 6. BIND THE SUBJECT PRODUCT. This is the binary built FROM the moved authority; every step
    // below uses this exact path, never a PATH lookup.
    let subject_build = cargo_in(
        worktree,
        target_dir,
        &["build", "--release", "--bin", "gunbc"],
        long,
    );
    if !completed_zero(&subject_build) {
        if let Err(detail) = worktree_admin_area_present(worktree) {
            return Outcome::Refused(Refusal::ExperimentWorktreeUnregistered { detail });
        }
        return Outcome::Refused(Refusal::CandidateProductBuildFailed {
            observation: subject_build,
        });
    }

    // THE REBUILD MUST HAVE TAKEN. The generated consequence now carries the moved value, and the
    // binary embeds that generated file, so a byte-identical binary means the server about to be
    // started is the SAME producer that answered the dry gate -- and every field of the response
    // would then be evidence about the old value dressed as evidence about the new one.
    let serving_gunbc_digest = match file_digest(gunbc) {
        Ok(digest) => digest,
        Err(detail) => return Outcome::Refused(Refusal::DigestUnavailable { detail }),
    };
    if serving_gunbc_digest == pre.dry_gate_gunbc {
        return Outcome::Refused(Refusal::SubjectBinaryUnchanged {
            digest: serving_gunbc_digest,
        });
    }
    let generated_artifact_digest = match file_digest(&worktree.join(GENERATED_REL)) {
        Ok(digest) => digest,
        Err(detail) => return Outcome::Refused(Refusal::DigestUnavailable { detail }),
    };

    // 7. SUPERVISE THE REAL SERVER. Loopback, its own process group, and a port THE OS CHOOSES.
    //
    // A port was previously derived as `8300 + pid % 400`, with a comment claiming that prevented
    // concurrent collision. It did not: 400 slots collide across pids, and an unrelated local
    // listener may already hold the one drawn. That mattered here more than it usually would,
    // because readiness was a bare successful connect -- so a foreign listener could SATISFY the
    // readiness wall, and every later disagreement would then be charged to the subject.
    // `--port 0` removes the guess, and the child announces what it actually bound.
    let mut command = Command::new(gunbc);
    command
        .args([
            "serve",
            "--source-root",
            "dag",
            "--source-root",
            "src/v2",
            "--entry",
            BREACH_FIXTURE,
            "--function",
            BREACH_FUNCTION,
            "--release-revision",
            head,
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--eval-budget-cpu-ms",
            &CPU_LIMIT_MS.to_string(),
        ])
        .current_dir(worktree)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = match spawn_in_new_process_group(&mut command) {
        Ok(child) => child,
        Err(e) => {
            return Outcome::Refused(Refusal::ServeSpawnFailed {
                detail: format!("{e}"),
            })
        }
    };
    let pid = child.id();
    let mut child = child;
    let mut out_drain = PipeDrain::spawn(child.stdout.take());
    let mut err_drain = PipeDrain::spawn(child.stderr.take());

    // Every character of the announcement is pinned except the port. Matching it is what binds the
    // listener to the contract THIS run armed -- entry, release revision and both budget arms --
    // rather than establishing only that something, somewhere, is listening.
    let announce_prefix = "gunbc serve listening on 127.0.0.1:";
    let announce_suffix = format!(
        " -> {BREACH_FUNCTION}() release_revision={head} eval_budget_cpu_ms={CPU_LIMIT_MS} eval_budget_wall_ms=unset"
    );

    let ready_budget = Duration::from_secs(900);
    let readiness = await_serve_ready(
        &mut child,
        pid,
        &out_drain,
        &err_drain,
        announce_prefix,
        &announce_suffix,
        ready_budget,
    );

    let mut guard = ServeGuard {
        child: Some(child),
        pid,
    };

    let (port, announcement) = match readiness {
        ServeReadiness::Ready { port, announcement } => (port, announcement),
        ServeReadiness::ExitedBeforeReady {
            termination,
            stdout,
            stderr,
        } => {
            // The child is already reaped; nothing is left for the guard to tear down.
            guard.child = None;
            return Outcome::Refused(Refusal::ServeExitedBeforeReadiness {
                termination,
                stdout,
                stderr,
            });
        }
        ServeReadiness::ReadinessTimedOut {
            after,
            stdout,
            stderr,
        } => {
            let termination = match guard.child.take() {
                Some(mut child) => match pin_process_group_identity(pid) {
                    Ok(identity) => {
                        terminate_process_group(&mut child, &identity, Duration::from_secs(10))
                    }
                    Err(_) => ProcessGroupTermination::identity_lost(),
                },
                None => {
                    return Outcome::Refused(Refusal::ProcessTerminationUnobservable {
                        detail: "serve child vanished between readiness and teardown".to_string(),
                    })
                }
            };
            return Outcome::Refused(Refusal::ReadinessTimedOut {
                after,
                termination,
                stdout,
                stderr,
            });
        }
        ServeReadiness::ReadinessObservationFailed {
            cause,
            stdout,
            stderr,
        } => {
            let termination = match guard.child.take() {
                Some(mut child) => match pin_process_group_identity(pid) {
                    Ok(identity) => {
                        terminate_process_group(&mut child, &identity, Duration::from_secs(10))
                    }
                    Err(_) => ProcessGroupTermination::identity_lost(),
                },
                None => {
                    return Outcome::Refused(Refusal::ProcessTerminationUnobservable {
                        detail: "serve child vanished between readiness and teardown".to_string(),
                    })
                }
            };
            return Outcome::Refused(Refusal::ReadinessObservationFailed {
                cause,
                termination,
                stdout,
                stderr,
            });
        }
    };

    let response = post_for_breach(port, Duration::from_secs(600));

    // 9. TERMINATE AND WAIT, before judging. A verdict reached while the subject is still running
    // would leave the port held whichever way the judgement went.
    let teardown = match guard.child.take() {
        Some(mut child) => match pin_process_group_identity(pid) {
            Ok(identity) => terminate_process_group(&mut child, &identity, Duration::from_secs(30)),
            Err(_) => ProcessGroupTermination::identity_lost(),
        },
        None => {
            return Outcome::Refused(Refusal::ProcessTerminationUnobservable {
                detail: "serve child was already taken before teardown".to_string(),
            })
        }
    };

    // TEARDOWN IS ADJUDICATED BEFORE THE STREAMS ARE DRAINED. This order is the point: an unsettled
    // teardown means a descendant may still hold the pipe's write end, so draining first would block
    // forever on precisely the arm that was supposed to report the failure -- the instrument hanging
    // instead of refusing.
    if !teardown.is_settled() {
        return Outcome::Refused(Refusal::ProcessTerminationFailed {
            termination: teardown,
        });
    }

    // The server's own stderr, complete. The breach diagnostic is the SUBJECT PROCESS'S account of
    // the same event the body describes, and joining the two is what distinguishes a server that
    // refused from one that merely rendered a refusal-shaped body.
    //
    // BOUNDED even here, and the completeness is CHECKED rather than assumed: the unbounded join
    // this replaced could not fail, it could only hang.
    let serve_stdout_outcome = out_drain.finish_within(Duration::from_secs(30));
    let serve_stderr_outcome = err_drain.finish_within(Duration::from_secs(30));
    if !serve_stdout_outcome.is_complete() || !serve_stderr_outcome.is_complete() {
        return Outcome::Refused(Refusal::ServeStreamsIncomplete {
            stdout: serve_stdout_outcome,
            stderr: serve_stderr_outcome,
        });
    }
    let serve_stdout = serve_stdout_outcome.text().to_string();
    let serve_stderr = serve_stderr_outcome.text().to_string();

    // 8. JUDGE THE RESPONSE.
    let judged = match response {
        Err(detail) => Err(Refusal::RequestFailed { detail }),
        Ok(response) => {
            let BreachResponse {
                status,
                content_type,
                body,
                fields,
            } = response;
            let moved_occurrences = body.matches(&format!("\"{moved_value}\"")).count();
            let former_occurrences = body.matches(&format!("\"{original_value}\"")).count();
            // ABSENT IS NOT EMPTY. Defaulting a missing member and comparing it downstream reports
            // a disagreement about the WRONG thing -- "code " rather than "the body carried no
            // code" -- and a defaulted `limit_ms` of 0 invents a number the server never sent.
            macro_rules! member {
                ($getter:expr, $name:literal) => {
                    match $getter {
                        Some(value) => value,
                        None => {
                            return Outcome::Refused(Refusal::ResponseMemberAbsent {
                                member: $name,
                                body: body.clone(),
                            })
                        }
                    }
                };
            }
            let code = member!(fields.string("code"), "code").to_string();
            let entry = member!(fields.string("entry"), "entry").to_string();
            let clock = member!(fields.string("clock"), "clock").to_string();
            let limit_ms = member!(fields.number("limit_ms"), "limit_ms");
            let elapsed_nanos = member!(fields.number("elapsed_ns"), "elapsed_ns");

            // The diagnostic the subject printed for THIS breach, reconstructed from the body's own
            // fields. If the two disagree, the body is not an account of what the server did.
            let expected_diagnostic = super::serve_budget_refusal::budget_refusal_diagnostic_text(
                &entry,
                &clock,
                elapsed_nanos,
                limit_ms,
            );

            let disagreement = if status != 500 {
                Some(format!("status {status}"))
            } else if content_type.as_deref() != Some("application/json; charset=utf-8") {
                Some(format!("content-type {content_type:?}"))
            } else if code != moved_value {
                Some(format!("code {code}"))
            } else if moved_occurrences != 1 {
                Some(format!("moved occurrences {moved_occurrences}"))
            } else if former_occurrences != 0 {
                Some(format!("former occurrences {former_occurrences}"))
            } else if entry != BREACH_FUNCTION {
                Some(format!("entry {entry}"))
            } else if clock != "thread_cpu" {
                Some(format!("clock {clock}"))
            } else if limit_ms != u128::from(CPU_LIMIT_MS) {
                Some(format!("limit_ms {limit_ms}"))
            } else if elapsed_nanos <= u128::from(CPU_LIMIT_MS) * 1_000_000 {
                // The elapsed value is not pinned -- only required to exceed the limit it breached.
                Some(format!("elapsed_ns {elapsed_nanos}"))
            } else if !serve_stderr.contains(&expected_diagnostic) {
                Some(format!(
                    "the server printed no diagnostic agreeing with the body; expected {expected_diagnostic:?}"
                ))
            } else {
                None
            };
            match disagreement {
                Some(observed) => Err(Refusal::ResponseDisagreed {
                    serve_stdout: serve_stdout.clone(),
                    serve_stderr: serve_stderr.clone(),
                    expectation: format!(
                        "500, content-type=application/json; charset=utf-8, code={moved_value}, moved=1, former=0, entry={BREACH_FUNCTION}, clock=thread_cpu, limit_ms={CPU_LIMIT_MS}, elapsed_ns>{}, and a serve diagnostic agreeing with the body",
                        u128::from(CPU_LIMIT_MS) * 1_000_000
                    ),
                    observed,
                }),
                None => Ok(EvaluationBudgetConsequenceReceipt {
                    subject_commit: head.to_string(),
                    subject_tree: tree.to_string(),
                    moved_value: moved_value.to_string(),
                    original_value: original_value.to_string(),
                    serve_status: status,
                    moved_occurrences,
                    former_occurrences,
                    elapsed_nanos,
                    orchestrator_digest: pre.orchestrator.clone(),
                    dry_gate_gunbc_digest: pre.dry_gate_gunbc.clone(),
                    serving_gunbc_digest: serving_gunbc_digest.clone(),
                    generated_artifact_digest: generated_artifact_digest.clone(),
                    serve_announcement: announcement.clone(),
                    serve_diagnostic: expected_diagnostic,
                }),
            }
        }
    };

    // 10a. RESTORE THE AUTHORITY AND REGENERATE, inside the disposable copy, so the tree comparison
    // below is against the bound original rather than against whatever the experiment left.
    if let Err(e) = std::fs::write(authority_path, original_source) {
        return Outcome::Refused(Refusal::AuthorityRestorationFailed {
            detail: format!("restoring {AUTHORITY_REL}: {e}"),
        });
    }
    let restored = gunbc_run(worktree, gunbc, GATE_ENTRY, "main_wet", long);
    if !completed_zero(&restored) {
        return Outcome::Refused(Refusal::AuthorityRestorationFailed {
            detail: format!("regeneration after restore did not complete: {restored:?}"),
        });
    }
    let residue = match git_stdout(
        worktree,
        &["status", "--porcelain"],
        short,
        "restored worktree status",
    ) {
        Ok(v) => v,
        Err(refusal) => return Outcome::Refused(refusal),
    };
    let residual_paths: Vec<&str> = residue
        .lines()
        .map(|line| line.split_whitespace().last().unwrap_or_default())
        .filter(|p| !p.is_empty() && !p.starts_with("target-falsifier"))
        .collect();
    if !residual_paths.is_empty() {
        return Outcome::Refused(Refusal::RestoredTreeDisagreed {
            detail: format!("paths still differing after restore: {residual_paths:?}"),
        });
    }

    match judged {
        Ok(receipt) => Outcome::Passed(receipt),
        Err(refusal) => Outcome::Refused(refusal),
    }
}

/// THE DECODER'S WALLS, EXECUTED. These are ordinary `--lib` tests rather than part of the
/// `#[ignore]` transaction, deliberately: the transaction costs tens of minutes and cannot be the
/// only thing that establishes a refusal, and a wall whose RED is never authored anywhere is a
/// decoration that will later be cited as coverage. Every case below is a document a real HTTP
/// response could carry.
#[cfg(test)]
mod json_decoder_walls {
    use super::parse_flat_json_object;

    /// The positive control. Without it, every refusal below would also pass on a decoder that
    /// rejected everything.
    #[test]
    fn a_flat_object_of_strings_and_numbers_decodes() {
        let object = parse_flat_json_object(r#"{"code":"x","limit_ms":50}"#)
            .expect("a flat object must decode");
        assert_eq!(object.string("code"), Some("x"));
        assert_eq!(object.number("limit_ms"), Some(50));
    }

    /// THE DISCRIMINATING RED for duplicate members. Last-one-wins would silently answer "b" here,
    /// and this instrument treats `code` as having exactly one answer.
    #[test]
    fn a_duplicated_member_refuses_rather_than_letting_the_later_one_win() {
        let refusal = parse_flat_json_object(r#"{"code":"a","code":"b"}"#)
            .expect_err("a duplicated member must refuse");
        assert!(
            refusal.contains("code") && refusal.contains("more than once"),
            "the refusal must name the duplicated member: {refusal}"
        );
    }

    /// A duplicate that changes TYPE must refuse too, or the two maps would each hold one copy and
    /// neither would see a collision.
    #[test]
    fn a_member_duplicated_across_string_and_number_refuses() {
        let refusal = parse_flat_json_object(r#"{"limit_ms":50,"limit_ms":"50"}"#)
            .expect_err("a member duplicated across types must refuse");
        assert!(
            refusal.contains("limit_ms"),
            "the refusal must name the duplicated member: {refusal}"
        );
    }

    /// THE DISCRIMINATING RED for whole-document parsing. The prefix is a perfectly good object,
    /// so a decoder that stopped at the closing brace would accept this and never see the rest.
    #[test]
    fn trailing_bytes_after_the_closing_brace_refuse() {
        let refusal = parse_flat_json_object(r#"{"code":"x"} and then some"#)
            .expect_err("trailing bytes must refuse");
        assert!(
            refusal.contains("trailing"),
            "the refusal must say what it found: {refusal}"
        );
    }

    /// Whitespace after the object is not trailing CONTENT, and refusing it would make the wall
    /// fire on ordinary well-formed responses.
    #[test]
    fn trailing_whitespace_is_not_trailing_content() {
        assert!(parse_flat_json_object("{\"code\":\"x\"}\n").is_ok());
    }

    /// A shape this decoder does not model is refused rather than skipped, so a body that nests
    /// the fields under another object cannot read as one that carries them at the top level.
    #[test]
    fn an_unmodelled_member_shape_refuses_rather_than_being_skipped() {
        assert!(parse_flat_json_object(r#"{"nested":{"code":"x"}}"#).is_err());
        assert!(parse_flat_json_object(r#"{"list":[1,2]}"#).is_err());
    }
}

/// THE FAILURE ARMS OF THE OBSERVATION MECHANISM, ON THE ORDINARY MERGE PATH.
///
/// The product falsifier is `#[ignore]` and runs once per landing head, so it is the wrong place to
/// establish that these arms REFUSE: it exercises the happy path, where none of them fire. These are
/// `--lib` tests for that reason -- each one drives a failure the instrument must not paper over,
/// and each goes red if the corresponding widen is restored.
#[cfg(test)]
mod observation_failure_arms {
    use super::*;

    /// Review 59152/59127's separator finding, one case per malformed form. The valid object is
    /// asserted in `json_decoder_walls`, so these cannot all pass on a decoder that rejects
    /// everything.
    #[test]
    fn every_malformed_member_separator_refuses() {
        for body in [
            r#"{,"code":"x"}"#,
            r#"{"code":"x",}"#,
            r#"{"code":"x",,"entry":"y"}"#,
            r#"{"code":"x" "entry":"y"}"#,
        ] {
            assert!(
                parse_flat_json_object(body).is_err(),
                "decoder accepted malformed separators: {body}"
            );
        }
        // The positive control belongs beside them: a decoder that refused everything would pass
        // every assertion above.
        assert!(parse_flat_json_object(r#"{"code":"x","entry":"y"}"#).is_ok());
        assert!(parse_flat_json_object("{}").is_ok());
    }

    /// A truncated stream is not a complete one, and `completed_zero` is the join that decides
    /// whether a command's output may be read as its whole answer.
    #[test]
    fn zero_exit_with_an_unfinished_stream_is_not_a_successful_observation() {
        let drained = CommandObservation::Completed {
            exit_code: 0,
            stdout: StreamOutcome::Closed {
                text: "abc".to_string(),
            },
            stderr: StreamOutcome::Closed {
                text: String::new(),
            },
        };
        assert!(
            completed_zero(&drained),
            "positive control must be accepted"
        );

        for truncated in [
            StreamOutcome::DeadlineExceeded {
                partial: "ab".to_string(),
                after: Duration::from_millis(1),
            },
            StreamOutcome::ReadFailed {
                partial: "ab".to_string(),
                cause: "boom".to_string(),
            },
            StreamOutcome::ReaderPanicked {
                partial: "ab".to_string(),
            },
        ] {
            assert!(!truncated.is_complete());
            assert!(
                !completed_zero(&CommandObservation::Completed {
                    exit_code: 0,
                    stdout: truncated,
                    stderr: StreamOutcome::Closed {
                        text: String::new()
                    },
                }),
                "a zero exit with an incomplete stream was accepted as success"
            );
        }
    }

    /// A nonzero git with empty stdout must not become an apparently successful empty read -- that
    /// is the shape that produced an empty HEAD and then blamed an innocent subject.
    #[test]
    fn a_nonzero_git_does_not_become_an_empty_successful_read() {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let refused = git_stdout(
            &repo,
            &[
                "rev-parse",
                "--verify",
                "definitely-not-a-real-ref^{commit}",
            ],
            Duration::from_secs(30),
            "control",
        );
        match refused {
            Err(EvaluationBudgetConsequenceRefusal::GitObservationFailed { what, .. }) => {
                assert_eq!(what, "control")
            }
            other => panic!("expected GitObservationFailed, got {other:?}"),
        }
        // Positive control: an ordinary git read still succeeds, so the assertion above is not
        // passing because every git call refuses.
        assert!(git_stdout(
            &repo,
            &["rev-parse", "HEAD"],
            Duration::from_secs(30),
            "control"
        )
        .is_ok());
    }

    /// A reader that fails mid-stream must not be reported as EOF. Before the terminal recorded WHY
    /// the reader stopped, both spellings were a bare `return` and the thread merely finishing was
    /// read as `Closed`.
    #[test]
    fn a_failing_reader_does_not_become_a_closed_stream() {
        struct FailsAfterOneLine {
            sent: bool,
        }
        impl Read for FailsAfterOneLine {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.sent {
                    return Err(std::io::Error::other("stream broke"));
                }
                self.sent = true;
                let line = b"first\n";
                buf[..line.len()].copy_from_slice(line);
                Ok(line.len())
            }
        }
        let mut drain = PipeDrain::spawn(Some(FailsAfterOneLine { sent: false }));
        match drain.finish_within(Duration::from_secs(10)) {
            StreamOutcome::ReadFailed { partial, .. } => assert_eq!(partial, "first\n"),
            other => panic!("a failed read was reported as {other:?}"),
        }

        // Positive control: a reader that reaches EOF still reports Closed with its whole text.
        let mut clean = PipeDrain::spawn(Some(std::io::Cursor::new(b"only\n".to_vec())));
        match clean.finish_within(Duration::from_secs(10)) {
            StreamOutcome::Closed { text } => assert_eq!(text, "only\n"),
            other => panic!("a clean EOF was reported as {other:?}"),
        }
    }

    /// The reader-panic arm must be REACHED, not merely matched. Constructing
    /// `StreamOutcome::ReaderPanicked` by hand tests the consumer; it never executes
    /// `JoinHandle::join().is_err()`, which is the line that actually distinguishes a panicking
    /// reader from one that reached EOF.
    #[test]
    fn a_panicking_reader_is_not_reported_as_a_closed_stream() {
        struct PanicsWhenPolled;
        impl Read for PanicsWhenPolled {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                panic!("reader panics while draining");
            }
        }
        // The panic is expected and its message would otherwise be printed by the default hook.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let mut drain = PipeDrain::spawn(Some(PanicsWhenPolled));
        let outcome = drain.finish_within(Duration::from_secs(10));
        std::panic::set_hook(previous);
        match outcome {
            StreamOutcome::ReaderPanicked { .. } => {}
            other => panic!("a panicking reader was reported as {other:?}"),
        }
    }

    /// Invalid response bytes refuse. The strict `from_utf8` is the right implementation, but the
    /// source spelling is not executing evidence -- the same standard applied to the #[ignore]
    /// falsifier applies here.
    #[test]
    fn an_invalid_utf8_response_body_refuses_rather_than_being_repaired() {
        // A lone continuation byte is not valid UTF-8; `from_utf8_lossy` would answer U+FFFD.
        assert!(decode_response_bytes(vec![b'{', 0x80, b'}']).is_err());
        // Positive control: ordinary bytes still decode.
        assert_eq!(
            decode_response_bytes(b"{\"a\":1}".to_vec()).expect("valid utf-8"),
            "{\"a\":1}"
        );
    }

    /// The decoder claims the body is JSON, so forms JSON rejects must refuse -- not merely the
    /// separator forms found earlier.
    #[test]
    fn forms_that_json_rejects_are_refused() {
        // A raw control character inside a string.
        assert!(parse_flat_json_object("{\"code\":\"a\u{1}b\"}").is_err());
        // A leading zero in a number.
        assert!(parse_flat_json_object(r#"{"limit_ms":01}"#).is_err());
        // Positive controls: a MODELLED escape and an ordinary number are still accepted, so the
        // refusals above are not passing on a decoder that rejects every string. `\u` is not among
        // the modelled escapes -- a narrow parser may refuse valid forms it does not model, and
        // that refusal is deliberate rather than a gap this test should paper over.
        assert!(parse_flat_json_object(r#"{"code":"a\nb"}"#).is_ok());
        assert!(parse_flat_json_object(r#"{"limit_ms":10}"#).is_ok());
        assert!(parse_flat_json_object(r#"{"limit_ms":0}"#).is_ok());
    }

    /// A refusal must be constructible for a multibyte document. The previews used to slice by BYTE
    /// offset, so a malformed body containing any non-ASCII text panicked the instrument while it
    /// was building the diagnostic meant to describe it.
    #[test]
    fn a_multibyte_malformed_document_refuses_without_panicking() {
        let long_multibyte = "é".repeat(400);
        let malformed = format!("not an object at all {long_multibyte}");
        assert!(parse_flat_json_object(&malformed).is_err());
        let trailing = format!("{{\"code\":\"x\"}} {long_multibyte}");
        assert!(parse_flat_json_object(&trailing).is_err());
    }

    /// A descendant holding the write end must produce a BOUNDED refusal, not a hang. This is the
    /// arm the unbounded `finish()` could not reach: it had no failure outcome, only blocking.
    #[test]
    fn a_pipe_held_open_by_a_survivor_ends_the_drain_rather_than_hanging() {
        let mut command = Command::new("sh");
        // The shell exits immediately while its background child keeps stdout open.
        command
            .arg("-c")
            .arg("sleep 30 & exit 0")
            .stdout(std::process::Stdio::piped());
        let mut child = crate::process_group::spawn_in_new_process_group(&mut command)
            .expect("spawn a shell that leaves a descendant holding the pipe");
        let pid = child.id();
        let mut drain = PipeDrain::spawn(child.stdout.take());
        let outcome = drain.finish_within(Duration::from_millis(500));
        // Tear the group down before asserting, so a failure here cannot leak the sleeper.
        // Reaping is always safe; it is SIGNALLING an unprovable identity that is not. So the lost
        // arm still waits on our own handle, it simply sends nothing.
        match crate::process_group::pin_process_group_identity(pid) {
            Ok(identity) => {
                let _ = crate::process_group::terminate_process_group(
                    &mut child,
                    &identity,
                    Duration::from_secs(10),
                );
            }
            Err(_) => {
                let _ = child.wait();
            }
        }
        match outcome {
            StreamOutcome::DeadlineExceeded { .. } => {}
            other => panic!("expected a bounded deadline outcome, got {other:?}"),
        }
    }
}
