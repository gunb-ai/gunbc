//! v4 `extdeps/runtimes/emit_host.dag` Rust host row — compile + execute emitted artifacts.
//!
//! **Modeled authority:** `run_emit_host_rust`, `runtime_value_parse_rust` in
//! `src/v4/extdeps/runtimes/emit_host.dag`. Substrate `.dag` dispatch is fail-closed
//! (`transport_not_wired`) until W3 wires this crate into `run_emit_host_rust`; this crate is
//! the executable host-process boundary exercised by `v4_emit_host_harness_test.rs`.
//!
//! **Host boundary:** child processes use wall-clock timeouts and per-stream byte caps so a
//! buggy emitted program cannot hang `cargo test` or exhaust memory.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostExit {
    Ok(ExitOk),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildLog {
    pub lines: Vec<String>,
}

// W3 reconciliation: modeled `EmitHostRunReceipt` in `host_run.dag` uses
// `HostExit { outcome: Outcome<Witness<ExitOk>> }` and `logical_run: Outcome<HostLogicalRun>`
// (stdout only when exit outcome Holds). This Rust transport row keeps a flat
// `HostExit::Ok|Err` + `stdout_bytes` until W3 wiring maps host-process results into the `.dag`
// carrier without merging diverging shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitHostRunReceipt {
    pub source_text: String,
    pub exit: HostExit,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
    pub build_log: BuildLog,
}

/// MVP-2 / eval_runtime_mvp alignment: five stdout bytes denote runtime value `5`.
pub fn runtime_value_parse_rust(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() == 5 {
        Ok(())
    } else {
        Err(format!(
            "runtime_value_parse_rust: expected 5 stdout bytes, got {}",
            bytes.len()
        ))
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

fn read_stream_bounded<R: Read>(mut reader: R, cap: usize) -> (Vec<u8>, bool) {
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
                if out.is_empty() {
                    out.extend_from_slice(format!("read error: {e}").as_bytes());
                }
                break;
            }
        };
        out.extend_from_slice(&buf[..n]);
    }
    (out, truncated)
}

fn run_command_bounded(mut cmd: Command, timeout: Duration) -> Result<BoundedChildOutput, String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
    let stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| "stdout pipe missing".to_string())?;
    let stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| "stderr pipe missing".to_string())?;

    let cap = HOST_STREAM_BYTE_CAP;
    let (tx_out, rx_out) = mpsc::channel();
    let (tx_err, rx_err) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx_out.send(read_stream_bounded(stdout_pipe, cap));
    });
    thread::spawn(move || {
        let _ = tx_err.send(read_stream_bounded(stderr_pipe, cap));
    });

    let deadline = Instant::now() + timeout;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait().map_err(|e| format!("try_wait: {e}"))? {
            Some(status) => break Some(status),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break None
            }
            None => thread::sleep(Duration::from_millis(50)),
        }
    };

    let recv_timeout = Duration::from_secs(5);
    let (stdout, stdout_truncated) = rx_out
        .recv_timeout(recv_timeout)
        .unwrap_or((Vec::new(), false));
    let (stderr, stderr_truncated) = rx_err
        .recv_timeout(recv_timeout)
        .unwrap_or((Vec::new(), false));

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

fn host_exit_from_bounded(output: &BoundedChildOutput, ok_label: &str) -> HostExit {
    if output.timed_out {
        return HostExit::Err(format!("{ok_label}: timed out"));
    }
    let Some(status) = output.status else {
        return HostExit::Err(format!("{ok_label}: no exit status"));
    };
    if status.success() {
        HostExit::Ok(ExitOk {
            code: status.code().unwrap_or(0),
        })
    } else {
        HostExit::Err(format!("{ok_label} failed: {status}"))
    }
}

/// Compile `source` as a Rust binary crate in `work_dir`, run it, capture stdout/stderr.
pub fn run_emit_host_rust(source: &str, work_dir: &Path) -> Result<EmitHostRunReceipt, String> {
    fs::create_dir_all(work_dir).map_err(|e| format!("create work_dir: {e}"))?;
    let src_dir = work_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create src: {e}"))?;

    let cargo_toml = work_dir.join("Cargo.toml");
    let target_dir = work_dir.join("target");
    let manifest = "[package]\nname = \"emit_host_fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"fixture\"\npath = \"src/main.rs\"\n";
    let mut f = fs::File::create(&cargo_toml).map_err(|e| format!("create Cargo.toml: {e}"))?;
    f.write_all(manifest.as_bytes())
        .map_err(|e| format!("write Cargo.toml: {e}"))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, source).map_err(|e| format!("write main.rs: {e}"))?;

    let mut build_cmd = Command::new("cargo");
    build_cmd
        .args(["build", "--quiet", "--manifest-path"])
        .arg(&cargo_toml)
        .env("CARGO_TARGET_DIR", &target_dir);
    let build = run_command_bounded(build_cmd, HOST_BUILD_TIMEOUT)?;
    let build_log = bounded_output_to_log(&build, "build");
    if !matches!(build.status, Some(s) if s.success()) {
        return Ok(EmitHostRunReceipt {
            source_text: source.to_string(),
            exit: host_exit_from_bounded(&build, "cargo build"),
            stdout_bytes: build.stdout,
            stderr_bytes: build.stderr,
            build_log,
        });
    }

    let bin_path = target_dir.join("debug/fixture");
    let run = run_command_bounded(Command::new(&bin_path), HOST_RUN_TIMEOUT)?;
    let mut lines = build_log.lines;
    lines.extend(bounded_output_to_log(&run, "run").lines);
    Ok(EmitHostRunReceipt {
        source_text: source.to_string(),
        exit: host_exit_from_bounded(&run, "fixture run"),
        stdout_bytes: run.stdout,
        stderr_bytes: run.stderr,
        build_log: BuildLog { lines },
    })
}

/// Default temp directory under `std::env::temp_dir()`.
pub fn default_work_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(prefix)
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
    fn run_command_bounded_times_out() {
        let mut sleep_cmd = Command::new("sleep");
        sleep_cmd.arg("60");
        let out = run_command_bounded(sleep_cmd, Duration::from_millis(200))
        .expect("spawn sleep");
        assert!(out.timed_out, "expected timeout");
    }
}
