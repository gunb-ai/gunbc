//! v2 candidate-B host-compile bridge — live `v1-compiler compile` transport row.
//!
//! **Modeled authority (future):** `src/v2/compiler/compile_host.dag` (`run_compile_host_v2`).
//! Substrate `.dag` body stays `transport_not_wired` until eval intercept lands (operator HOLD).
//!
//! **Host boundary (DESIGN.md §3):** outcomes are typed carriers — setup failure is
//! `HostExitOutcome::Rejected(HostSetupFailure)`, logical compile outcome is
//! `HostExitOutcome::Accepted(ExitWitness::Holds|Violates)`. No free-form `String` authority.

use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Wall-clock bound for `v1-compiler compile`.
pub const HOST_COMPILE_TIMEOUT: Duration = Duration::from_secs(300);
/// Per-stream capture cap (stdout and stderr each).
pub const HOST_STREAM_BYTE_CAP: usize = 1 << 20;

/// Typed exit: success only when the compiler child exited 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitOk {
    pub code: i32,
}

/// Host-process phase for compile transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostPhase {
    Compile,
}

/// Child stream identifier for setup failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStream {
    Stdout,
    Stderr,
}

/// Harness / transport setup failure — distinct from logical compile outcome (DESIGN.md §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostSetupFailure {
    CompilerBinaryMissing {
        path: String,
    },
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
    EmptySourceRoots,
    EmptyOutputDir,
    EmptyCompileTarget,
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

/// Mirrors future `v2.std.host_run.HostExit` — typed setup vs logical exit separation.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildLog {
    pub lines: Vec<String>,
}

/// Parsed `compiled: N files emitted, M diagnostics` receipt line from v2 compile stdout/stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledReceipt {
    pub files_emitted: u32,
    pub diagnostic_count: u32,
}

/// Phase-1 transport pins: source roots + output dir + compile target string (`rust`, `dag`, `rust+dag`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileHostTransportInputs {
    pub source_roots: Vec<String>,
    pub output_dir: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileHostRunReceipt {
    pub exit: HostExit,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
    pub build_log: BuildLog,
    pub compiled_receipt: Option<CompiledReceipt>,
}

/// Fail-closed acceptance: process exit 0 **and** parsed receipt with zero diagnostics.
pub fn compile_accepted(receipt: &CompileHostRunReceipt) -> bool {
    receipt.exit.exit_holds()
        && receipt
            .compiled_receipt
            .as_ref()
            .is_some_and(|r| r.diagnostic_count == 0)
}

pub fn validate_compile_host_transport_inputs(
    inputs: &CompileHostTransportInputs,
) -> Result<(), HostSetupFailure> {
    if inputs.source_roots.is_empty() {
        return Err(HostSetupFailure::EmptySourceRoots);
    }
    if inputs.source_roots.iter().any(|root| root.is_empty()) {
        return Err(HostSetupFailure::EmptySourceRoots);
    }
    if inputs.output_dir.is_empty() {
        return Err(HostSetupFailure::EmptyOutputDir);
    }
    if inputs.target.is_empty() {
        return Err(HostSetupFailure::EmptyCompileTarget);
    }
    Ok(())
}

/// Parse v2 compile receipt from combined compiler output (M1 probe pattern).
pub fn parse_compiled_receipt(combined_output: &[u8]) -> Option<CompiledReceipt> {
    for line in String::from_utf8_lossy(combined_output).lines().rev() {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix("compiled: ")?;
        let (files, diagnostics) = rest.split_once(" files emitted, ")?;
        let diagnostic_count = diagnostics.strip_suffix(" diagnostics")?;
        let files_emitted = files.parse().ok()?;
        let diagnostic_count = diagnostic_count.parse().ok()?;
        return Some(CompiledReceipt {
            files_emitted,
            diagnostic_count,
        });
    }
    None
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
    thread::spawn(move || {
        let result = read_stream_bounded(stdout_pipe, cap, phase, HostStream::Stdout);
        let _ = tx_out.send(result);
    });
    thread::spawn(move || {
        let result = read_stream_bounded(stderr_pipe, cap, phase, HostStream::Stderr);
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

pub fn default_work_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}_{}", std::process::id()))
}

/// Run live `v1-compiler compile` (gunbc binary) and return a typed receipt.
pub fn run_compile_host_v2(
    compiler_bin: &Path,
    inputs: &CompileHostTransportInputs,
    work_dir: &Path,
) -> Result<CompileHostRunReceipt, HostSetupFailure> {
    validate_compile_host_transport_inputs(inputs)?;
    if !compiler_bin.is_file() {
        return Err(HostSetupFailure::CompilerBinaryMissing {
            path: compiler_bin.display().to_string(),
        });
    }
    std::fs::create_dir_all(work_dir).map_err(|e| HostSetupFailure::SpawnFailed {
        phase: HostPhase::Compile,
        source: format!("work_dir create: {e}"),
    })?;
    std::fs::create_dir_all(&inputs.output_dir).map_err(|e| HostSetupFailure::SpawnFailed {
        phase: HostPhase::Compile,
        source: format!("output_dir create: {e}"),
    })?;

    let mut cmd = Command::new(compiler_bin);
    cmd.arg("compile");
    for root in &inputs.source_roots {
        cmd.arg("--source-root").arg(root);
    }
    cmd.arg("--output-dir")
        .arg(&inputs.output_dir)
        .arg("--target")
        .arg(&inputs.target);

    let output = run_command_bounded(cmd, HOST_COMPILE_TIMEOUT, HostPhase::Compile)?;
    let build_log = bounded_output_to_log(&output, "compile");
    let mut combined = output.stdout.clone();
    combined.extend_from_slice(&output.stderr);
    let compiled_receipt = parse_compiled_receipt(&combined);
    Ok(CompileHostRunReceipt {
        exit: host_exit_from_bounded(&output, HostPhase::Compile),
        stdout_bytes: output.stdout,
        stderr_bytes: output.stderr,
        build_log,
        compiled_receipt,
    })
}

impl fmt::Display for HostSetupFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostSetupFailure::CompilerBinaryMissing { path } => {
                write!(f, "compiler binary missing: {path}")
            }
            HostSetupFailure::SpawnFailed { phase, source } => {
                write!(f, "spawn failed ({phase:?}): {source}")
            }
            HostSetupFailure::StdoutPipeMissing { phase } => {
                write!(f, "stdout pipe missing ({phase:?})")
            }
            HostSetupFailure::StderrPipeMissing { phase } => {
                write!(f, "stderr pipe missing ({phase:?})")
            }
            HostSetupFailure::TryWaitFailed { phase, source } => {
                write!(f, "try_wait failed ({phase:?}): {source}")
            }
            HostSetupFailure::StreamReadFailed {
                phase,
                stream,
                source,
            } => write!(f, "stream read failed ({phase:?} {stream:?}): {source}"),
            HostSetupFailure::EmptySourceRoots => write!(f, "empty source_roots"),
            HostSetupFailure::EmptyOutputDir => write!(f, "empty output_dir"),
            HostSetupFailure::EmptyCompileTarget => write!(f, "empty compile target"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_compiled_receipt_reads_trailing_line() {
        let log = b"noise\ncompiled: 3 files emitted, 0 diagnostics\n";
        let receipt = parse_compiled_receipt(log).expect("receipt");
        assert_eq!(receipt.files_emitted, 3);
        assert_eq!(receipt.diagnostic_count, 0);
    }

    #[test]
    fn compile_accepted_requires_zero_diagnostics() {
        let holds = CompileHostRunReceipt {
            exit: HostExit::holds(0),
            stdout_bytes: Vec::new(),
            stderr_bytes: b"compiled: 1 files emitted, 0 diagnostics\n".to_vec(),
            build_log: BuildLog { lines: vec![] },
            compiled_receipt: Some(CompiledReceipt {
                files_emitted: 1,
                diagnostic_count: 0,
            }),
        };
        assert!(compile_accepted(&holds));

        let rejects = CompileHostRunReceipt {
            compiled_receipt: Some(CompiledReceipt {
                files_emitted: 1,
                diagnostic_count: 2,
            }),
            ..holds
        };
        assert!(!compile_accepted(&rejects));
    }

    #[test]
    fn validate_compile_host_transport_inputs_rejects_empty_roots() {
        assert!(matches!(
            validate_compile_host_transport_inputs(&CompileHostTransportInputs {
                source_roots: vec![],
                output_dir: "out".into(),
                target: "rust".into(),
            }),
            Err(HostSetupFailure::EmptySourceRoots)
        ));
    }

    #[test]
    fn host_exit_separates_setup_from_logical() {
        let setup = HostExit::setup_rejected(HostSetupFailure::EmptyOutputDir);
        assert!(!setup.exit_holds());
        let logical = HostExit::logical_violation(HostLogicalFailure::ExitedNonzero {
            phase: HostPhase::Compile,
            code: Some(1),
        });
        assert!(!logical.exit_holds());
    }
}
