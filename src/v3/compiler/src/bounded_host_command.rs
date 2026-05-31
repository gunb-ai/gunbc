//! Bounded host subprocess execution (P2 process boundary / P4 wall-clock cap).
//!
//! Shared by [`crate::post_emit_verifier::run_post_emit_verifier`] and the W1 /
//! L5 harness in [`crate::test_runner`]. Fail-closed vs unbounded
//! [`std::process::Command::output`] and unbounded `read_to_end` on verbose children.
//!
//! Wall budget covers **child wait and pipe drain**: if the direct child exits while
//! process-group descendants still hold pipe write ends, drain threads get EOF only
//! after the group is killed or the overall deadline trips.

use std::io::Read;
use std::process::{Child, Command, Output, Stdio};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Default wall-clock for post-emit verifiers and W1 host children (matches
/// [`crate::test_runner::EXECUTE_COMMAND_WALL_TIMEOUT`]).
pub const DEFAULT_WALL_TIMEOUT: Duration = Duration::from_secs(30);

const WAIT_POLL: Duration = Duration::from_millis(20);

const DRAIN_JOIN_GRACE_POLLS: u32 = 50;

const DRAIN_CHUNK_BYTES: usize = 8192;

/// Max bytes retained from a host child's **stdout**; drain to EOF after the cap.
pub const CAPTURE_MAX_STDOUT_BYTES: usize = 16 * 1024;

/// Max bytes retained from a host child's **stderr**; drain to EOF after the cap.
pub const CAPTURE_MAX_STDERR_BYTES: usize = 256 * 1024;

/// Host child ready to spawn: stdin null, stdout/stderr piped, process group on Unix.
///
/// Construct only via [`prepare_host_command`]; [`host_command_output`] rejects plain
/// [`Command`] so P2 boundary shape cannot depend on caller call-order convention.
#[must_use]
pub struct PreparedHostCommand(Command);

#[derive(Debug)]
struct BoundedPipeCapture {
    bytes: Vec<u8>,
    truncated: bool,
}

fn drain_reader_bounded(
    mut r: impl Read,
    max_stored: usize,
) -> std::io::Result<BoundedPipeCapture> {
    let mut buf = Vec::new();
    let mut truncated = false;
    let mut scratch = [0u8; DRAIN_CHUNK_BYTES];
    loop {
        let n = r.read(&mut scratch)?;
        if n == 0 {
            break;
        }
        if buf.len() >= max_stored {
            truncated = true;
            continue;
        }
        let room = max_stored - buf.len();
        if n <= room {
            buf.extend_from_slice(&scratch[..n]);
        } else {
            buf.extend_from_slice(&scratch[..room]);
            truncated = true;
        }
    }
    Ok(BoundedPipeCapture {
        bytes: buf,
        truncated,
    })
}

fn apply_host_process_boundary(cmd: &mut Command) {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
}

/// Apply P2/P4 host-child boundary (null stdin, piped stdout/stderr, process group).
pub fn prepare_host_command(cmd: Command) -> PreparedHostCommand {
    let mut cmd = cmd;
    apply_host_process_boundary(&mut cmd);
    PreparedHostCommand(cmd)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChildWaitFail {
    WallTimeout { wall_time: Duration },
    Io(String),
}

#[cfg(unix)]
fn kill_process_group(pgid: i32) {
    use libc::{kill, SIGKILL};
    if pgid > 0 && unsafe { kill(-pgid, SIGKILL) } < 0 {
        // Best-effort: group may already be gone.
    }
}

#[cfg(not(unix))]
fn kill_process_group(_pgid: i32) {}

fn reap_host_child(child: &mut Child, pgid: i32) {
    kill_process_group(pgid);
    let _ = child.kill();
    let _ = child.wait();
}

fn child_wait_until(
    child: &mut Child,
    deadline: Instant,
    wall_time: Duration,
) -> Result<std::process::ExitStatus, ChildWaitFail> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    reap_host_child(child, child.id() as i32);
                    return Err(ChildWaitFail::WallTimeout { wall_time });
                }
                std::thread::sleep(WAIT_POLL);
            }
            Err(err) => return Err(ChildWaitFail::Io(format!("{err}"))),
        }
    }
}

fn join_pipe_capture(
    handle: JoinHandle<std::io::Result<BoundedPipeCapture>>,
    deadline: Instant,
    pgid: i32,
    stream: &str,
    label: &str,
) -> Result<BoundedPipeCapture, String> {
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            kill_process_group(pgid);
            for _ in 0..DRAIN_JOIN_GRACE_POLLS {
                if handle.is_finished() {
                    break;
                }
                std::thread::sleep(WAIT_POLL);
            }
            if !handle.is_finished() {
                return Err(format!(
                    "{label}: {stream} drain exceeded wall-clock budget (process group {pgid} killed, fail-closed)"
                ));
            }
        }
        std::thread::sleep(WAIT_POLL);
    }
    handle
        .join()
        .map_err(|_| format!("{label}: {stream} capture thread panicked"))?
        .map_err(|e| format!("{label}: read {stream} failed: {e}"))
}

/// `spawn` + wall-bounded wait + bounded stdout/stderr capture.
pub fn host_command_output(
    label: &str,
    wall: Duration,
    prepared: PreparedHostCommand,
) -> Result<Output, String> {
    let PreparedHostCommand(mut cmd) = prepared;
    let started = Instant::now();
    let deadline = started + wall;

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{label}: failed to spawn host child: {e}"))?;
    let pgid = child.id() as i32;

    let Some(mut stdout) = child.stdout.take() else {
        reap_host_child(&mut child, pgid);
        return Err(format!(
            "{label}: internal error: stdout not piped after prepare_host_command"
        ));
    };
    let Some(mut stderr) = child.stderr.take() else {
        reap_host_child(&mut child, pgid);
        return Err(format!(
            "{label}: internal error: stderr not piped after prepare_host_command"
        ));
    };

    let stdout_handle =
        std::thread::spawn(move || drain_reader_bounded(&mut stdout, CAPTURE_MAX_STDOUT_BYTES));
    let stderr_handle =
        std::thread::spawn(move || drain_reader_bounded(&mut stderr, CAPTURE_MAX_STDERR_BYTES));

    let status = match child_wait_until(&mut child, deadline, wall) {
        Ok(s) => s,
        Err(ChildWaitFail::WallTimeout { wall_time }) => {
            let _ = join_pipe_capture(stdout_handle, deadline, pgid, "stdout", label);
            let _ = join_pipe_capture(stderr_handle, deadline, pgid, "stderr", label);
            return Err(format!(
                "{label}: exceeded {:.2}s wall-clock limit (process group killed, fail-closed)",
                wall_time.as_secs_f64()
            ));
        }
        Err(ChildWaitFail::Io(err)) => {
            reap_host_child(&mut child, pgid);
            let _ = join_pipe_capture(stdout_handle, deadline, pgid, "stdout", label);
            let _ = join_pipe_capture(stderr_handle, deadline, pgid, "stderr", label);
            return Err(format!("{label}: wait on host child failed: {err}"));
        }
    };

    let stdout_cap = join_pipe_capture(stdout_handle, deadline, pgid, "stdout", label)?;
    let stderr_cap = join_pipe_capture(stderr_handle, deadline, pgid, "stderr", label)?;

    if stdout_cap.truncated || stderr_cap.truncated {
        return Err(format!(
            "{label}: bounded host I/O exceeded (stdout cap {} B, stderr cap {} B; stdout_trunc={} stderr_trunc={}); child exit={:?}",
            CAPTURE_MAX_STDOUT_BYTES,
            CAPTURE_MAX_STDERR_BYTES,
            stdout_cap.truncated,
            stderr_cap.truncated,
            status.code(),
        ));
    }

    Ok(Output {
        status,
        stdout: stdout_cap.bytes,
        stderr: stderr_cap.bytes,
    })
}
