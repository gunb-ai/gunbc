//! v4 `compiler/emit_host.dag` Rust host row — compile + execute emitted artifacts.
//!
//! **Modeled authority:** `run_emit_host_rust`, `runtime_value_parse_rust` in
//! `src/v4/compiler/emit_host.dag`. Executable host-process boundary for W3; substrate `.dag`
//! models receipt assembly while `emit_host_bridge.rs` dispatches this crate until T-22 eval
//! wires host transport directly.
//!
//! **Host boundary (INVARIANTS §P2):** outcomes are typed carriers — setup failure is
//! `HostExitOutcome::Rejected(HostSetupFailure)`, logical child outcome is
//! `HostExitOutcome::Accepted(ExitWitness::Holds|Violates)`. No free-form `String` authority.
//! **W3 dissolution:** map `HostExit` / `HostLogicalRun` into `v4.std.host_run` carriers when
//! `run_emit_host_rust` transport lands (dissolves `emit_host_transport_not_wired`).
//!
//! Child processes use wall-clock timeouts and per-stream byte caps so a buggy emitted program
//! cannot hang `cargo test` or exhaust memory.

use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

static WORK_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

/// Wall-clock bound for `cargo build` of the ephemeral fixture crate.
pub const HOST_BUILD_TIMEOUT: Duration = Duration::from_secs(300);
/// Wall-clock bound for running the compiled fixture binary.
pub const HOST_RUN_TIMEOUT: Duration = Duration::from_secs(30);
/// Per-stream capture cap (stdout and stderr each).
pub const HOST_STREAM_BYTE_CAP: usize = 1 << 20;

/// Typed exit: success only when the child process exited 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitOk {
    pub code: i32,
}

/// Which host-process phase produced an outcome (setup vs logical child).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostPhase {
    Build,
    FixtureRun,
}

/// Child stream identifier for setup failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStream {
    Stdout,
    Stderr,
}

/// Harness / transport setup failure — distinct from logical child exit (INVARIANTS §P2(c)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostSetupFailure {
    SpawnFailed {
        phase: HostPhase,
        source: String,
    },
    StdoutPipeMissing {
        phase: HostPhase,
    },
    StderrPipeMissing {
        phase: HostPhase,
    },
    TryWaitFailed {
        phase: HostPhase,
        source: String,
    },
    StreamReadFailed {
        phase: HostPhase,
        stream: HostStream,
        source: String,
    },
    WorkDirCreateFailed {
        source: String,
    },
    ManifestWriteFailed {
        source: String,
    },
    SourceWriteFailed {
        source: String,
    },
    EmptyClaimInputRoot,
    EmptyExpectedEvalRoot,
}

/// Logical child ran but did not satisfy the exit witness (timeout, nonzero, missing status).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostLogicalFailure {
    TimedOut { phase: HostPhase },
    NoExitStatus { phase: HostPhase },
    ExitedNonzero { phase: HostPhase, code: Option<i32> },
}

/// Mirrors `.dag` `Witness<ExitOk>` at the Rust transport row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitWitness {
    Holds(ExitOk),
    Violates(HostLogicalFailure),
}

/// Mirrors `.dag` `Outcome<Witness<ExitOk>>` at the Rust transport row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostExitOutcome {
    Accepted(ExitWitness),
    Rejected(HostSetupFailure),
}

/// Mirrors `v4.std.host_run.HostExit` — typed setup vs logical exit separation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostExit {
    pub outcome: HostExitOutcome,
}

impl HostExit {
    pub fn holds(code: i32) -> Self {
        Self {
            outcome: HostExitOutcome::Accepted(ExitWitness::Holds(ExitOk { code })),
        }
    }

    pub fn logical_violation(failure: HostLogicalFailure) -> Self {
        Self {
            outcome: HostExitOutcome::Accepted(ExitWitness::Violates(failure)),
        }
    }

    pub fn setup_rejected(failure: HostSetupFailure) -> Self {
        Self {
            outcome: HostExitOutcome::Rejected(failure),
        }
    }

    pub fn exit_holds(&self) -> bool {
        matches!(
            self.outcome,
            HostExitOutcome::Accepted(ExitWitness::Holds(_))
        )
    }
}

/// Phase-typed logical-run stdout — only `Some` when exit witness Holds (P2 boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostLogicalRun {
    pub stdout_bytes: Vec<u8>,
}

/// Project exit + captured stdout into logical-run carrier (mirrors `host_logical_run_from_exit`).
pub fn host_logical_run_from_exit(
    exit: &HostExit,
    stdout_bytes: Vec<u8>,
) -> Option<HostLogicalRun> {
    match &exit.outcome {
        HostExitOutcome::Accepted(ExitWitness::Holds(_)) => Some(HostLogicalRun { stdout_bytes }),
        HostExitOutcome::Accepted(ExitWitness::Violates(_)) | HostExitOutcome::Rejected(_) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildLog {
    pub lines: Vec<String>,
}

/// Claim-only pins at the `run_emit_host_*` substrate transport boundary (`Inputs.root` only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitHostTransportInputs {
    pub claim_input_root: String,
}

/// Full emit-vs-eval fixture pins — claim + expected eval root (both required facts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitHostFixtureInputs {
    pub claim_input_root: String,
    pub expected_eval_root: String,
}

impl EmitHostFixtureInputs {
    pub fn transport(&self) -> EmitHostTransportInputs {
        EmitHostTransportInputs {
            claim_input_root: self.claim_input_root.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitHostRunReceipt {
    pub source_text: String,
    pub exit: HostExit,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
    pub build_log: BuildLog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeValueParseFailure {
    pub expected_len: usize,
    pub actual_len: usize,
}

impl fmt::Display for RuntimeValueParseFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "runtime_value_parse_rust: expected {} stdout bytes, got {}",
            self.expected_len, self.actual_len
        )
    }
}

/// MVP-2 / eval_runtime_mvp alignment: five stdout bytes denote runtime value `5` (shared rust/python row).
/// Tranche-1: byte contract matches rust; `emit_host.dag` `runtime_value_parse` branches on authority pin
/// so python-specific parsing can land without changing transport callers.
pub fn runtime_value_parse_python(bytes: &[u8]) -> Result<(), RuntimeValueParseFailure> {
    runtime_value_parse_rust(bytes)
}

/// W3.3: same MVP-2 five-byte stdout contract as rust/python rows.
pub fn runtime_value_parse_go(bytes: &[u8]) -> Result<(), RuntimeValueParseFailure> {
    runtime_value_parse_rust(bytes)
}

/// Brief T / typescript host row: same MVP-2 five-byte stdout contract as rust/python/go rows.
pub fn runtime_value_parse_typescript(bytes: &[u8]) -> Result<(), RuntimeValueParseFailure> {
    runtime_value_parse_rust(bytes)
}

/// MVP-2 / eval_runtime_mvp alignment: five stdout bytes denote runtime value `5`.
pub fn runtime_value_parse_rust(bytes: &[u8]) -> Result<(), RuntimeValueParseFailure> {
    const EXPECTED: usize = 5;
    if bytes.len() == EXPECTED {
        Ok(())
    } else {
        Err(RuntimeValueParseFailure {
            expected_len: EXPECTED,
            actual_len: bytes.len(),
        })
    }
}

struct BoundedChildOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: Option<ExitStatus>,
    timed_out: bool,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

fn read_stream_bounded<R: Read>(
    mut reader: R,
    cap: usize,
    phase: HostPhase,
    stream: HostStream,
) -> Result<(Vec<u8>, bool), HostSetupFailure> {
    let mut buf = [0u8; 8192];
    let mut out = Vec::new();
    let mut truncated = false;
    loop {
        if out.len() >= cap {
            truncated = true;
            break;
        }
        let room = cap - out.len();
        let chunk = room.min(buf.len());
        let n = match reader.read(&mut buf[..chunk]) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                return Err(HostSetupFailure::StreamReadFailed {
                    phase,
                    stream,
                    source: e.to_string(),
                });
            }
        };
        out.extend_from_slice(&buf[..n]);
    }
    Ok((out, truncated))
}

fn run_command_bounded(
    mut cmd: Command,
    timeout: Duration,
    phase: HostPhase,
) -> Result<BoundedChildOutput, HostSetupFailure> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| HostSetupFailure::SpawnFailed {
        phase,
        source: e.to_string(),
    })?;
    let stdout_pipe = child
        .stdout
        .take()
        .ok_or(HostSetupFailure::StdoutPipeMissing { phase })?;
    let stderr_pipe = child
        .stderr
        .take()
        .ok_or(HostSetupFailure::StderrPipeMissing { phase })?;

    let cap = HOST_STREAM_BYTE_CAP;
    let (tx_out, rx_out) = mpsc::channel();
    let (tx_err, rx_err) = mpsc::channel();
    let phase_out = phase;
    let phase_err = phase;
    thread::spawn(move || {
        let result = read_stream_bounded(stdout_pipe, cap, phase_out, HostStream::Stdout);
        let _ = tx_out.send(result);
    });
    thread::spawn(move || {
        let result = read_stream_bounded(stderr_pipe, cap, phase_err, HostStream::Stderr);
        let _ = tx_err.send(result);
    });

    let deadline = Instant::now() + timeout;
    let mut timed_out = false;
    let status = loop {
        match child
            .try_wait()
            .map_err(|e| HostSetupFailure::TryWaitFailed {
                phase,
                source: e.to_string(),
            })? {
            Some(status) => break Some(status),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break None;
            }
            None => thread::sleep(Duration::from_millis(50)),
        }
    };

    let recv_timeout = Duration::from_secs(5);
    let (stdout, stdout_truncated) =
        rx_out
            .recv_timeout(recv_timeout)
            .map_err(|e| HostSetupFailure::StreamReadFailed {
                phase,
                stream: HostStream::Stdout,
                source: format!("recv: {e}"),
            })??;
    let (stderr, stderr_truncated) =
        rx_err
            .recv_timeout(recv_timeout)
            .map_err(|e| HostSetupFailure::StreamReadFailed {
                phase,
                stream: HostStream::Stderr,
                source: format!("recv: {e}"),
            })??;

    Ok(BoundedChildOutput {
        stdout,
        stderr,
        status,
        timed_out,
        stdout_truncated,
        stderr_truncated,
    })
}

fn bounded_output_to_log(output: &BoundedChildOutput, label: &str) -> BuildLog {
    let mut lines = Vec::new();
    if !output.stdout.is_empty() {
        lines.push(format!(
            "{label} stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    if !output.stderr.is_empty() {
        lines.push(format!(
            "{label} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout_truncated {
        lines.push(format!(
            "{label} stdout: truncated at {} bytes",
            HOST_STREAM_BYTE_CAP
        ));
    }
    if output.stderr_truncated {
        lines.push(format!(
            "{label} stderr: truncated at {} bytes",
            HOST_STREAM_BYTE_CAP
        ));
    }
    if output.timed_out {
        lines.push(format!("{label} status: timed out"));
    } else if let Some(status) = output.status {
        lines.push(format!("{label} status: {status}"));
    } else {
        lines.push(format!("{label} status: unknown"));
    }
    BuildLog { lines }
}

fn host_exit_from_bounded(output: &BoundedChildOutput, phase: HostPhase) -> HostExit {
    if output.timed_out {
        return HostExit::logical_violation(HostLogicalFailure::TimedOut { phase });
    }
    let Some(status) = output.status else {
        return HostExit::logical_violation(HostLogicalFailure::NoExitStatus { phase });
    };
    if status.success() {
        HostExit::holds(status.code().unwrap_or(0))
    } else {
        HostExit::logical_violation(HostLogicalFailure::ExitedNonzero {
            phase,
            code: status.code(),
        })
    }
}

/// Fail-closed when `run_emit_host_*` transport is invoked without a claim input pin.
///
/// At the substrate `run_emit_host_*` boundary only `claim_input_root` is modeled in
/// `Inputs.root` (`emit_host.dag` `run_test_claim_emit_vs_eval_for_claim` passes
/// `fixture_inputs: Inputs { root: claim_input_root }`). `expected_eval_root` is evaluated
/// separately and is not in scope on this call path — do not require it here.
pub fn validate_emit_host_transport_inputs(
    inputs: &EmitHostTransportInputs,
) -> Result<(), HostSetupFailure> {
    if inputs.claim_input_root.is_empty() {
        return Err(HostSetupFailure::EmptyClaimInputRoot);
    }
    Ok(())
}

/// Fail-closed when emit-vs-eval / harness callers omit either pin (both must be distinct facts).
///
/// Production callers: `emit_host_bridge` transport and cross-target parity entrypoints.
pub fn validate_emit_host_fixture_inputs(
    inputs: &EmitHostFixtureInputs,
) -> Result<(), HostSetupFailure> {
    validate_emit_host_transport_inputs(&inputs.transport())?;
    if inputs.expected_eval_root.is_empty() {
        return Err(HostSetupFailure::EmptyExpectedEvalRoot);
    }
    Ok(())
}

/// Compile `source` as a Rust binary crate in `work_dir`, run it, capture stdout/stderr.
pub fn run_emit_host_rust(
    source: &str,
    inputs: &EmitHostTransportInputs,
    work_dir: &Path,
) -> Result<EmitHostRunReceipt, HostSetupFailure> {
    validate_emit_host_transport_inputs(inputs)?;
    fs::create_dir_all(work_dir).map_err(|e| HostSetupFailure::WorkDirCreateFailed {
        source: e.to_string(),
    })?;
    let src_dir = work_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| HostSetupFailure::WorkDirCreateFailed {
        source: e.to_string(),
    })?;

    let cargo_toml = work_dir.join("Cargo.toml");
    let target_dir = work_dir.join("target");
    let manifest = "[package]\nname = \"emit_host_fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"fixture\"\npath = \"src/main.rs\"\n";
    let mut f =
        fs::File::create(&cargo_toml).map_err(|e| HostSetupFailure::ManifestWriteFailed {
            source: e.to_string(),
        })?;
    f.write_all(manifest.as_bytes())
        .map_err(|e| HostSetupFailure::ManifestWriteFailed {
            source: e.to_string(),
        })?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, source).map_err(|e| HostSetupFailure::SourceWriteFailed {
        source: e.to_string(),
    })?;

    let mut build_cmd = Command::new("cargo");
    build_cmd
        .args(["build", "--quiet", "--manifest-path"])
        .arg(&cargo_toml)
        .env("CARGO_TARGET_DIR", &target_dir);
    let build = run_command_bounded(build_cmd, HOST_BUILD_TIMEOUT, HostPhase::Build)?;
    let build_log = bounded_output_to_log(&build, "build");
    if !matches!(build.status, Some(s) if s.success()) {
        return Ok(EmitHostRunReceipt {
            source_text: source.to_string(),
            exit: host_exit_from_bounded(&build, HostPhase::Build),
            stdout_bytes: build.stdout,
            stderr_bytes: build.stderr,
            build_log,
        });
    }

    let bin_path = target_dir.join("debug/fixture");
    let run = run_command_bounded(
        Command::new(&bin_path),
        HOST_RUN_TIMEOUT,
        HostPhase::FixtureRun,
    )?;
    let mut lines = build_log.lines;
    lines.extend(bounded_output_to_log(&run, "run").lines);
    Ok(EmitHostRunReceipt {
        source_text: source.to_string(),
        exit: host_exit_from_bounded(&run, HostPhase::FixtureRun),
        stdout_bytes: run.stdout,
        stderr_bytes: run.stderr,
        build_log: BuildLog { lines },
    })
}

/// Run `source` as a Go program in `work_dir` via `go run`, capture stdout/stderr.
pub fn run_emit_host_go(
    source: &str,
    inputs: &EmitHostTransportInputs,
    work_dir: &Path,
) -> Result<EmitHostRunReceipt, HostSetupFailure> {
    validate_emit_host_transport_inputs(inputs)?;
    fs::create_dir_all(work_dir).map_err(|e| HostSetupFailure::WorkDirCreateFailed {
        source: e.to_string(),
    })?;

    let main_go = work_dir.join("main.go");
    fs::write(&main_go, source).map_err(|e| HostSetupFailure::SourceWriteFailed {
        source: e.to_string(),
    })?;

    let mut run_cmd = Command::new("go");
    run_cmd.arg("run").arg(&main_go);
    let run = run_command_bounded(run_cmd, HOST_RUN_TIMEOUT, HostPhase::FixtureRun)?;
    let build_log = bounded_output_to_log(&run, "run");
    Ok(EmitHostRunReceipt {
        source_text: source.to_string(),
        exit: host_exit_from_bounded(&run, HostPhase::FixtureRun),
        stdout_bytes: run.stdout,
        stderr_bytes: run.stderr,
        build_log,
    })
}

/// Host python interpreter — override via `GUNBC_PYTHON` or `V4_PHASE1_NAT_SEMIRING_PYTHON`.
pub fn python3_binary() -> String {
    std::env::var("GUNBC_PYTHON")
        .or_else(|_| std::env::var("V4_PHASE1_NAT_SEMIRING_PYTHON"))
        .unwrap_or_else(|_| "python3".to_string())
}

/// Node binary for typescript host row — override via `GUNBC_NODE`.
pub fn node_binary() -> String {
    std::env::var("GUNBC_NODE").unwrap_or_else(|_| "node".to_string())
}

/// Aligns with `emit_host_typescript_authority_pin` / `ts_mvp1_source_text` (`typescript.dag`).
pub const EMIT_HOST_TYPESCRIPT_AUTHORITY_PIN: &str =
    "function add(x: number, y: number): number { return x + y; }";

/// `npx -p` package pin for the typescript host row (matches M0 probe).
pub const EMIT_HOST_TYPESCRIPT_TSC_PACKAGE: &str = "typescript@5.9.2";

/// Ambient Node globals for `tsc` without requiring `@types/node` in the ephemeral work dir.
const TYPESCRIPT_NODE_AMBIENT_DECLS: &str = r#"
declare const process: {
  exit(code?: number): never;
  stdout: { write(chunk: Uint8Array): boolean };
};
declare const Buffer: { alloc(size: number): Uint8Array };
"#;

/// Transport harness appended after gunbc-emitted mvp1 add-fn source (not part of emitted text).
const TYPESCRIPT_MVP1_ADD_EXECUTE_HARNESS: &str = r#"
const __gunbc_add_probe = add(2, 3);
if (__gunbc_add_probe !== 5) {
  process.exit(1);
}
process.stdout.write(Buffer.alloc(5));
"#;

/// True only for the modeled `emit_host_typescript_authority_pin` (exact match — no prefix heuristic).
pub fn typescript_uses_mvp1_add_execute_harness(source: &str) -> bool {
    source == EMIT_HOST_TYPESCRIPT_AUTHORITY_PIN
}

fn typescript_needs_node_ambients(source: &str) -> bool {
    source.contains("process.") || source.contains("Buffer.")
}

fn typescript_fixture_source(source: &str) -> String {
    let body = if typescript_uses_mvp1_add_execute_harness(source) {
        format!("{source}{TYPESCRIPT_MVP1_ADD_EXECUTE_HARNESS}")
    } else {
        source.to_string()
    };
    if typescript_needs_node_ambients(&body) {
        format!("{TYPESCRIPT_NODE_AMBIENT_DECLS}{body}")
    } else {
        body
    }
}

/// Compile `source` with `tsc`, run emitted JS on Node, capture stdout/stderr.
///
/// For the mvp1 add-fn authority pin, transport appends a Node harness that calls `add(2, 3)`
/// and writes the MVP-2 five-byte stdout contract on success — the gunbc-emitted function text
/// is passed through unchanged.
pub fn run_emit_host_typescript(
    source: &str,
    inputs: &EmitHostTransportInputs,
    work_dir: &Path,
) -> Result<EmitHostRunReceipt, HostSetupFailure> {
    validate_emit_host_transport_inputs(inputs)?;
    fs::create_dir_all(work_dir).map_err(|e| HostSetupFailure::WorkDirCreateFailed {
        source: e.to_string(),
    })?;

    let fixture_source = typescript_fixture_source(source);
    let fixture_ts = work_dir.join("fixture.ts");
    fs::write(&fixture_ts, &fixture_source).map_err(|e| HostSetupFailure::SourceWriteFailed {
        source: e.to_string(),
    })?;

    let out_dir = work_dir.join("tsc-out");
    fs::create_dir_all(&out_dir).map_err(|e| HostSetupFailure::WorkDirCreateFailed {
        source: e.to_string(),
    })?;

    let mut build_cmd = Command::new("npx");
    build_cmd.args([
        "-y",
        "-p",
        EMIT_HOST_TYPESCRIPT_TSC_PACKAGE,
        "tsc",
        "--target",
        "ES2022",
        "--module",
        "commonjs",
        "--outDir",
    ]);
    build_cmd.arg(&out_dir).arg(&fixture_ts);
    let build = run_command_bounded(build_cmd, HOST_BUILD_TIMEOUT, HostPhase::Build)?;
    let build_log = bounded_output_to_log(&build, "tsc");
    if !matches!(build.status, Some(s) if s.success()) {
        return Ok(EmitHostRunReceipt {
            source_text: source.to_string(),
            exit: host_exit_from_bounded(&build, HostPhase::Build),
            stdout_bytes: build.stdout,
            stderr_bytes: build.stderr,
            build_log,
        });
    }

    let js_path = out_dir.join("fixture.js");
    let mut run_cmd = Command::new(node_binary());
    run_cmd.arg(&js_path);
    let run = run_command_bounded(run_cmd, HOST_RUN_TIMEOUT, HostPhase::FixtureRun)?;
    let mut lines = build_log.lines;
    lines.extend(bounded_output_to_log(&run, "node").lines);
    Ok(EmitHostRunReceipt {
        source_text: source.to_string(),
        exit: host_exit_from_bounded(&run, HostPhase::FixtureRun),
        stdout_bytes: run.stdout,
        stderr_bytes: run.stderr,
        build_log: BuildLog { lines },
    })
}

/// Run `source` as a Python script in `work_dir`, capture stdout/stderr.
pub fn run_emit_host_python(
    source: &str,
    inputs: &EmitHostTransportInputs,
    work_dir: &Path,
) -> Result<EmitHostRunReceipt, HostSetupFailure> {
    validate_emit_host_transport_inputs(inputs)?;
    fs::create_dir_all(work_dir).map_err(|e| HostSetupFailure::WorkDirCreateFailed {
        source: e.to_string(),
    })?;

    let script_path = work_dir.join("fixture.py");
    fs::write(&script_path, source).map_err(|e| HostSetupFailure::SourceWriteFailed {
        source: e.to_string(),
    })?;

    let mut run_cmd = Command::new(python3_binary());
    run_cmd.arg(&script_path);
    let run = run_command_bounded(run_cmd, HOST_RUN_TIMEOUT, HostPhase::FixtureRun)?;
    let build_log = bounded_output_to_log(&run, "run");
    Ok(EmitHostRunReceipt {
        source_text: source.to_string(),
        exit: host_exit_from_bounded(&run, HostPhase::FixtureRun),
        stdout_bytes: run.stdout,
        stderr_bytes: run.stderr,
        build_log,
    })
}

/// Default temp directory under `std::env::temp_dir()`.
pub fn default_work_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(prefix)
}

/// Hermetic work directory unique per process and per call (concurrent eval / parallel tests safe).
pub fn unique_work_dir(prefix: &str) -> PathBuf {
    let seq = WORK_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
    default_work_dir(&format!("{prefix}_{}_{seq}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_value_parse_rust_accepts_five_bytes() {
        assert!(runtime_value_parse_rust(&[0, 0, 0, 0, 0]).is_ok());
        assert!(runtime_value_parse_rust(&[1, 2, 3]).is_err());
    }

    #[test]
    fn unique_work_dir_differs_per_call_in_process() {
        let a = unique_work_dir("gunbc_emit_host_unique_test");
        let b = unique_work_dir("gunbc_emit_host_unique_test");
        assert_ne!(
            a, b,
            "concurrent eval calls must not share a work directory"
        );
    }

    #[test]
    fn validate_emit_host_transport_inputs_accepts_claim_only() {
        assert!(
            validate_emit_host_transport_inputs(&EmitHostTransportInputs {
                claim_input_root: "claim".into(),
            })
            .is_ok()
        );
    }

    #[test]
    fn validate_emit_host_fixture_inputs_rejects_empty_pins() {
        assert!(matches!(
            validate_emit_host_fixture_inputs(&EmitHostFixtureInputs {
                claim_input_root: String::new(),
                expected_eval_root: "x".into(),
            }),
            Err(HostSetupFailure::EmptyClaimInputRoot)
        ));
    }

    #[test]
    fn host_exit_separates_setup_from_logical() {
        let setup = HostExit::setup_rejected(HostSetupFailure::SpawnFailed {
            phase: HostPhase::Build,
            source: "e".into(),
        });
        assert!(!setup.exit_holds());
        assert!(matches!(
            setup.outcome,
            HostExitOutcome::Rejected(HostSetupFailure::SpawnFailed { .. })
        ));

        let logical = HostExit::logical_violation(HostLogicalFailure::TimedOut {
            phase: HostPhase::FixtureRun,
        });
        assert!(!logical.exit_holds());
        assert!(matches!(
            logical.outcome,
            HostExitOutcome::Accepted(ExitWitness::Violates(_))
        ));
    }

    #[test]
    fn host_logical_run_only_on_holds() {
        let exit = HostExit::holds(0);
        let run = host_logical_run_from_exit(&exit, vec![1, 2, 3]).expect("holds");
        assert_eq!(run.stdout_bytes, vec![1, 2, 3]);

        let violated = HostExit::logical_violation(HostLogicalFailure::ExitedNonzero {
            phase: HostPhase::FixtureRun,
            code: Some(1),
        });
        assert!(host_logical_run_from_exit(&violated, vec![]).is_none());
    }

    #[test]
    fn run_command_bounded_times_out() {
        let mut sleep_cmd = Command::new("sleep");
        sleep_cmd.arg("60");
        let out = run_command_bounded(sleep_cmd, Duration::from_millis(200), HostPhase::FixtureRun)
            .expect("spawn sleep");
        assert!(out.timed_out, "expected timeout");
        let exit = host_exit_from_bounded(&out, HostPhase::FixtureRun);
        assert!(matches!(
            exit.outcome,
            HostExitOutcome::Accepted(ExitWitness::Violates(HostLogicalFailure::TimedOut {
                phase: HostPhase::FixtureRun
            }))
        ));
    }

    #[test]
    fn typescript_mvp1_harness_only_on_exact_authority_pin() {
        assert!(typescript_uses_mvp1_add_execute_harness(
            EMIT_HOST_TYPESCRIPT_AUTHORITY_PIN
        ));
        assert!(!typescript_uses_mvp1_add_execute_harness(
            "function add(x: number, y: number): number { return x - y; }"
        ));
        assert!(!typescript_uses_mvp1_add_execute_harness(
            "function add(x: number, y: number): number { return x + y; }"
        ));
    }

    /// Brief T execution gate — requires `node` + `npx` on PATH (CI: v4-mvp1-typescript-emit-host-probe).
    #[test]
    fn typescript_mvp1_authority_pin_executes_on_node() {
        if Command::new("node").output().is_err() || Command::new("npx").output().is_err() {
            eprintln!("skip typescript_mvp1_authority_pin_executes_on_node: node/npx unavailable");
            return;
        }
        let work_dir = unique_work_dir("gunbc_emit_host_ts_mvp1_gate");
        let inputs = EmitHostTransportInputs {
            claim_input_root: "mvp1_ts_add_claim_input".into(),
        };
        let receipt = run_emit_host_typescript(
            EMIT_HOST_TYPESCRIPT_AUTHORITY_PIN,
            &inputs,
            &work_dir,
        )
        .expect("run_emit_host_typescript");
        assert!(receipt.exit.exit_holds(), "mvp1 add-fn must exit 0 on Node");
        runtime_value_parse_typescript(&receipt.stdout_bytes).expect("MVP-2 stdout");
    }

    #[test]
    fn typescript_non_authority_pin_skips_execute_harness() {
        assert!(!typescript_uses_mvp1_add_execute_harness(
            "function add(x: number, y: number): number { return x - y; }"
        ));
    }
}
