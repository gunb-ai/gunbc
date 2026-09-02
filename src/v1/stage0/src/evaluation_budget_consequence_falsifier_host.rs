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
    spawn_in_new_process_group, terminate_process_group, ProcessGroupTermination, ProcessGroupWait,
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
        stdout: String,
        stderr: String,
    },
    /// Overran its budget AND its process group is gone. The teardown verdict travels WITH the
    /// timeout because "the command overran" and "the instrument left it running" are different
    /// facts about the same event, and the second one invalidates every later step.
    TimedOut {
        after: Duration,
        termination: ProcessGroupTermination,
        stdout: String,
        stderr: String,
    },
    /// Overran its budget and something SURVIVED teardown. Separate from `TimedOut` because this
    /// arm may never be treated as a mere non-zero result: a surviving group holds resources the
    /// next step assumes are free.
    TimedOutWithTerminationFailure {
        after: Duration,
        termination: ProcessGroupTermination,
        stdout: String,
        stderr: String,
    },
    Signaled {
        signal: i32,
        stdout: String,
        stderr: String,
    },
    /// The command never started. Reserved for spawn itself failing.
    SpawnRefused { detail: String },
    /// The command started and the WAIT failed. Previously folded into `SpawnRefused`, which said
    /// the opposite of what happened -- a process that ran and could not be observed was reported
    /// as one that never ran, and its remedy is the reverse.
    WaitFailed {
        detail: String,
        stdout: String,
        stderr: String,
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
    /// WHICH BINARIES ACTUALLY ANSWERED. The source has always CLAIMED that the pre-perturbation
    /// candidate's digest is what identifies the producer that noticed the drift; until now it
    /// claimed it in prose while the receipt carried no digest at all, so a reader could not tell
    /// the binary that produced the dry red from the one rebuilt in its place -- they occupy the
    /// same path, and the second overwrites the first.
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

/// SHA-256 of a file's bytes. Used to give the receipt an identity for each artifact it depends
/// on, so "the tool that noticed" and "the tool that served" are distinguishable after the fact
/// rather than only assertable at the time.
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

    // THE PIPE THREADS ARE JOINED ON EVERY ARM, not only the happy one. They were previously
    // dropped on three of four arms, which detached them and threw away the child's own account of
    // why it failed -- exactly the output an operator needs when the failure is the interesting
    // case. Joining is safe here because the writer has either exited or been torn down below.
    let drain = |out: std::thread::JoinHandle<String>, err: std::thread::JoinHandle<String>| {
        (
            out.join().unwrap_or_default(),
            err.join().unwrap_or_default(),
        )
    };

    match wait {
        ProcessGroupWait::Exited { code } => {
            let (stdout, stderr) = drain(out_handle, err_handle);
            CommandObservation::Completed {
                exit_code: code,
                stdout,
                stderr,
            }
        }
        ProcessGroupWait::Signaled { signal } => {
            let (stdout, stderr) = drain(out_handle, err_handle);
            CommandObservation::Signaled {
                signal,
                stdout,
                stderr,
            }
        }
        ProcessGroupWait::TimedOut => {
            // A command that overran its budget is terminated here rather than left running, so a
            // timeout cannot leak a process into the next step's environment -- and whether that
            // termination actually SUCCEEDED is carried in the observation rather than discarded.
            let termination = terminate_process_group(&mut child, pid, Duration::from_secs(10));
            let (stdout, stderr) = drain(out_handle, err_handle);
            if termination.group_is_gone() {
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
            // The child may still be running: a wait that failed established nothing about it.
            let termination = terminate_process_group(&mut child, pid, Duration::from_secs(10));
            let (stdout, stderr) = drain(out_handle, err_handle);
            CommandObservation::WaitFailed {
                detail: format!("{detail}; teardown: {termination:?}"),
                stdout,
                stderr,
            }
        }
    }
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

/// A PIPE THAT IS DRAINED WHILE THE CHILD RUNS. The falsifier needs the server's output for two
/// different purposes at two different times -- the listening announcement DURING readiness, and
/// the breach diagnostic AFTER the request -- so the reader cannot be a thread whose result is
/// only available once the pipe closes. It appends into a shared buffer that either purpose can
/// read at any moment, and blocking is impossible because the reader never stops.
struct PipeDrain {
    text: Arc<Mutex<String>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl PipeDrain {
    fn spawn<R: Read + Send + 'static>(pipe: Option<R>) -> PipeDrain {
        let text = Arc::new(Mutex::new(String::new()));
        let sink = Arc::clone(&text);
        let handle = std::thread::spawn(move || {
            if let Some(pipe) = pipe {
                let mut reader = std::io::BufReader::new(pipe);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => return,
                        Ok(_) => {
                            if let Ok(mut held) = sink.lock() {
                                held.push_str(&line);
                            }
                        }
                    }
                }
            }
        });
        PipeDrain {
            text,
            handle: Some(handle),
        }
    }

    fn snapshot(&self) -> String {
        self.text.lock().map(|t| t.clone()).unwrap_or_default()
    }

    /// Join the reader, so the returned text is the child's COMPLETE output rather than whatever
    /// had arrived when the caller happened to look.
    fn finish(&mut self) -> String {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.snapshot()
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
        match child.try_wait() {
            Ok(Some(_)) => {
                let termination = terminate_process_group(child, pid, Duration::from_secs(10));
                return ServeReadiness::ExitedBeforeReady {
                    termination,
                    stdout: out.snapshot(),
                    stderr: err.snapshot(),
                };
            }
            Ok(None) => {}
            Err(e) => {
                return ServeReadiness::ReadinessObservationFailed {
                    cause: format!("try_wait during readiness: {e}"),
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
fn parse_flat_json_object(text: &str) -> Result<JsonObject, String> {
    let mut chars = text.trim().chars().peekable();
    if chars.next() != Some('{') {
        return Err(format!(
            "body is not a JSON object: {}",
            &text[..text.len().min(200)]
        ));
    }
    let mut object = JsonObject::default();
    loop {
        while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
            chars.next();
        }
        match chars.peek() {
            Some('}') => {
                chars.next();
                break;
            }
            Some(',') => {
                chars.next();
                continue;
            }
            Some('"') => {}
            other => return Err(format!("expected a member name, found {other:?}")),
        }
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
                object.strings.insert(key, value);
            }
            Some(c) if c.is_ascii_digit() => {
                let mut digits = String::new();
                while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
                    digits.push(chars.next().unwrap_or_default());
                }
                let value = digits
                    .parse::<u128>()
                    .map_err(|e| format!("member {key} is not an unsigned integer: {e}"))?;
                object.numbers.insert(key, value);
            }
            other => {
                return Err(format!(
                    "member {key} carries {other:?}, which this decoder does not model"
                ))
            }
        }
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
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| format!("no header/body boundary: {}", &text[..text.len().min(200)]))?;
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

    // The digest is taken HERE, while this binary is still the only one that has existed at this
    // path. Taken later it would be the rebuilt one, and the receipt would name the wrong producer
    // for the dry red with no way for a reader to notice.
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
                Some(mut child) => {
                    terminate_process_group(&mut child, pid, Duration::from_secs(10))
                }
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
                Some(mut child) => {
                    terminate_process_group(&mut child, pid, Duration::from_secs(10))
                }
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
        Some(mut child) => terminate_process_group(&mut child, pid, Duration::from_secs(30)),
        None => {
            return Outcome::Refused(Refusal::ProcessTerminationUnobservable {
                detail: "serve child was already taken before teardown".to_string(),
            })
        }
    };

    // The server's own stderr, complete. The breach diagnostic is the SUBJECT PROCESS'S account of
    // the same event the body describes, and joining the two is what distinguishes a server that
    // refused from one that merely rendered a refusal-shaped body.
    let serve_stdout = out_drain.finish();
    let serve_stderr = err_drain.finish();

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
            let code = fields.string("code").unwrap_or_default().to_string();
            let entry = fields.string("entry").unwrap_or_default().to_string();
            let clock = fields.string("clock").unwrap_or_default().to_string();
            let limit_ms = fields.number("limit_ms").unwrap_or_default();
            let elapsed_nanos = fields.number("elapsed_ns").unwrap_or_default();

            // The diagnostic the subject printed for THIS breach, reconstructed from the body's own
            // fields. If the two disagree, the body is not an account of what the server did.
            let expected_diagnostic =
                format!("serve: refused {entry} on {clock} clock: elapsed_ns={elapsed_nanos} limit_ms={limit_ms}");

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

    // A teardown that did not settle is terminal even when the response agreed: an instrument that
    // reports a green while leaving a process group alive has not finished its transaction.
    // The question is GROUP ABSENCE, not leader exit. A leader that exited cleanly while a helper
    // still holds the port is a failed teardown, and reading the leader's status alone is what
    // would call it a success.
    if !teardown.group_is_gone() {
        return Outcome::Refused(Refusal::ProcessTerminationFailed {
            termination: teardown,
        });
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
