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

use crate::process_group::{spawn_in_new_process_group, terminate_process_group, ProcessGroupWait};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// WHAT A SUBPROCESS DID, kept as four states. A nonzero exit and a timeout are different facts,
/// and this distinction is here because the loop that first measured this receipt by hand wrapped
/// each run in `timeout` and reported ANY nonzero exit as a failed verdict -- so a slow run and a
/// false witness were the same observation. Two rows reported red and were green.
#[derive(Debug, Clone)]
pub(crate) enum CommandObservation {
    Completed {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },
    TimedOut {
        after: Duration,
    },
    Signaled {
        signal: i32,
    },
    SpawnRefused {
        detail: String,
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
    ServeSpawnFailed {
        detail: String,
    },
    ServeExitedBeforeReadiness {
        wait: ProcessGroupWait,
    },
    ReadinessTimedOut {
        after: Duration,
    },
    RequestFailed {
        detail: String,
    },
    ResponseDisagreed {
        expectation: String,
        observed: String,
    },
    ProcessTerminationFailed {
        wait: ProcessGroupWait,
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
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let out_handle = std::thread::spawn(move || {
        let mut buf: String = String::new();
        if let Some(pipe) = stdout.as_mut() {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });
    let err_handle = std::thread::spawn(move || {
        let mut buf: String = String::new();
        if let Some(pipe) = stderr.as_mut() {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });
    let pid = child.id();
    let wait = crate::process_group::wait_for_exit(&mut child, budget);
    let observation = match wait {
        ProcessGroupWait::Exited { code } => {
            let stdout: String = out_handle.join().unwrap_or_default();
            let stderr: String = err_handle.join().unwrap_or_default();
            CommandObservation::Completed {
                exit_code: code,
                stdout,
                stderr,
            }
        }
        ProcessGroupWait::Signaled { signal } => CommandObservation::Signaled { signal },
        ProcessGroupWait::TimedOut => {
            // A command that overran its budget is terminated here rather than left running, so a
            // timeout cannot leak a process into the next step's environment.
            let _ = terminate_process_group(&mut child, pid, Duration::from_secs(10));
            CommandObservation::TimedOut { after: budget }
        }
        ProcessGroupWait::WaitFailed { detail } => CommandObservation::SpawnRefused { detail },
    };
    observation
}

fn completed_zero(observation: &CommandObservation) -> bool {
    matches!(
        observation,
        CommandObservation::Completed { exit_code: 0, .. }
    )
}

fn git(workdir: &Path, args: &[&str], budget: Duration) -> CommandObservation {
    let mut command = Command::new("git");
    command.args(args).current_dir(workdir);
    observe(command, budget)
}

fn stdout_of(observation: &CommandObservation) -> String {
    match observation {
        CommandObservation::Completed { stdout, .. } => stdout.clone(),
        _ => String::new(),
    }
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
            let _ = terminate_process_group(&mut child, self.pid, Duration::from_secs(10));
        }
    }
}

/// Readiness is OBSERVED, never inferred from "the child has not exited yet". A process that is
/// alive but not listening would otherwise be read as ready, and the request that follows would
/// fail for a reason the transaction would then attribute to the subject.
fn await_listening(port: u16, budget: Duration) -> bool {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    false
}

/// One HTTP request over a raw socket. `curl` is deliberately not spawned: the response has to be
/// DECODED and judged field by field, and a shell string would put the adjudication back inside an
/// opaque command.
fn post_for_breach(port: u16, budget: Duration) -> Result<(u16, String), String> {
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
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| {
            format!(
                "no status line in response: {}",
                &text[..text.len().min(200)]
            )
        })?;
    let payload = text
        .split_once("\r\n\r\n")
        .map(|(_, rest)| rest.to_string())
        .unwrap_or_default();
    Ok((status, payload))
}

fn json_number(payload: &str, key: &str) -> Option<u128> {
    let needle = format!("\"{key}\":");
    let start = payload.find(&needle)? + needle.len();
    let rest = &payload[start..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse::<u128>().ok()
}

fn json_string_field(payload: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = payload.find(&needle)? + needle.len();
    let rest = &payload[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
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
    if !completed_zero(&status) || !stdout_of(&status).trim().is_empty() {
        return EvaluationBudgetConsequenceFalsifierOutcome::Refused(
            EvaluationBudgetConsequenceRefusal::SourceCheckoutNotClean {
                status: stdout_of(&status),
            },
        );
    }
    let head = stdout_of(&git(repo_root, &["rev-parse", "HEAD"], short))
        .trim()
        .to_string();
    let tree = stdout_of(&git(repo_root, &["rev-parse", "HEAD^{tree}"], short))
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
            return Some(EvaluationBudgetConsequenceRefusal::WorktreeCleanupFailed {
                observation: removed,
            });
        }
    }
    let _ = git(repo_root, &["worktree", "prune"], short);
    if worktree.exists() {
        return Some(EvaluationBudgetConsequenceRefusal::WorktreeCleanupFailed {
            observation: CommandObservation::Completed {
                exit_code: 0,
                stdout: format!(
                    "{} still present after remove and prune",
                    worktree.display()
                ),
                stderr: String::new(),
            },
        });
    }
    // THE PARENT IS RE-VERIFIED, not assumed. An instrument that mutates a copy must prove it did
    // not mutate the original, and "we intended not to" is not that proof.
    let head_after = stdout_of(&git(repo_root, &["rev-parse", "HEAD"], short))
        .trim()
        .to_string();
    let tree_after = stdout_of(&git(repo_root, &["rev-parse", "HEAD^{tree}"], short))
        .trim()
        .to_string();
    let status_after = stdout_of(&git(repo_root, &["status", "--porcelain"], short));
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
    let inventory_after = stdout_of(&git(repo_root, &["worktree", "list"], short));
    if inventory_after.contains(worktree_str) {
        return Some(EvaluationBudgetConsequenceRefusal::WorktreeCleanupFailed {
            observation: CommandObservation::Completed {
                exit_code: 0,
                stdout: format!("{worktree_str} still registered after remove and prune"),
                stderr: String::new(),
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
    let mut command = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()));
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

    // 2. BUILD THE UNPERTURBED CANDIDATE TOOL, before anything moves. Its digest is the answer to
    // "which producer noticed the drift", and an ambient binary on PATH is never that answer.
    let built = cargo_in(
        worktree,
        &target_dir,
        &["build", "--release", "--bin", "gunbc"],
        long,
    );
    if !completed_zero(&built) {
        return Outcome::Refused(Refusal::CandidateProductBuildFailed { observation: built });
    }

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
    let changed = stdout_of(&git(worktree, &["status", "--porcelain"], short));
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
        } => (*exit_code, stdout.clone(), stderr.clone()),
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
        return Outcome::Refused(Refusal::CandidateProductBuildFailed {
            observation: subject_build,
        });
    }

    // 7. SUPERVISE THE REAL SERVER. Loopback, its own process group, an ephemeral-ish port derived
    // from the pid so two concurrent runs cannot collide on one listener.
    let port: u16 = 8300 + (std::process::id() % 400) as u16;
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
            &port.to_string(),
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
    let mut guard = ServeGuard {
        child: Some(child),
        pid,
    };

    // Readiness is a SUCCESSFUL CONNECT, not a live pid. A process that is alive but not listening
    // would otherwise be called ready and the failed request blamed on the subject.
    let ready_budget = Duration::from_secs(900);
    if !await_listening(port, ready_budget) {
        if let Some(mut child) = guard.child.take() {
            let wait = terminate_process_group(&mut child, pid, Duration::from_secs(10));
            let refusal = match wait {
                ProcessGroupWait::TimedOut => Refusal::ReadinessTimedOut {
                    after: ready_budget,
                },
                settled => Refusal::ServeExitedBeforeReadiness { wait: settled },
            };
            return Outcome::Refused(refusal);
        }
        return Outcome::Refused(Refusal::ReadinessTimedOut {
            after: ready_budget,
        });
    }

    let response = post_for_breach(port, Duration::from_secs(600));

    // 9. TERMINATE AND WAIT, before judging. A verdict reached while the subject is still running
    // would leave the port held whichever way the judgement went.
    let teardown = match guard.child.take() {
        Some(mut child) => terminate_process_group(&mut child, pid, Duration::from_secs(30)),
        None => ProcessGroupWait::WaitFailed {
            detail: "serve child was already taken".to_string(),
        },
    };

    // 8. JUDGE THE RESPONSE.
    let judged = match response {
        Err(detail) => Err(Refusal::RequestFailed { detail }),
        Ok((status, payload)) => {
            let moved_occurrences = payload.matches(&format!("\"{moved_value}\"")).count();
            let former_occurrences = payload.matches(&format!("\"{original_value}\"")).count();
            let code = json_string_field(&payload, "code").unwrap_or_default();
            let entry = json_string_field(&payload, "entry").unwrap_or_default();
            let clock = json_string_field(&payload, "clock").unwrap_or_default();
            let limit_ms = json_number(&payload, "limit_ms").unwrap_or_default();
            let elapsed_nanos = json_number(&payload, "elapsed_ns").unwrap_or_default();
            let disagreement = if status != 500 {
                Some(format!("status {status}"))
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
            } else {
                None
            };
            match disagreement {
                Some(observed) => Err(Refusal::ResponseDisagreed {
                    expectation: format!(
                        "500, code={moved_value}, moved=1, former=0, entry={BREACH_FUNCTION}, clock=thread_cpu, limit_ms={CPU_LIMIT_MS}, elapsed_ns>{}",
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
                }),
            }
        }
    };

    // A teardown that did not settle is terminal even when the response agreed: an instrument that
    // reports a green while leaving a process group alive has not finished its transaction.
    if !matches!(
        teardown,
        ProcessGroupWait::Exited { .. } | ProcessGroupWait::Signaled { .. }
    ) {
        return Outcome::Refused(Refusal::ProcessTerminationFailed { wait: teardown });
    }

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
    let residue = stdout_of(&git(worktree, &["status", "--porcelain"], short));
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
