//! Bounded host subprocess execution (P2 process boundary / P4 wall-clock cap).
//!
//! Shared by [`crate::post_emit_verifier::run_post_emit_verifier`] and the W1 /
//! L5 harness in [`crate::test_runner`]. Fail-closed vs unbounded
//! [`std::process::Command::output`] and unbounded `read_to_end` on verbose children.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Default wall-clock for post-emit verifiers and W1 host children (matches
/// [`crate::test_runner::EXECUTE_COMMAND_WALL_TIMEOUT`]).
pub const DEFAULT_WALL_TIMEOUT: Duration = Duration::from_secs(30);

const WAIT_POLL: Duration = Duration::from_millis(20);

const DRAIN_CHUNK_BYTES: usize = 8192;

/// Max bytes retained from a host child's **stdout**; drain to EOF after the cap.
pub const CAPTURE_MAX_STDOUT_BYTES: usize = 16 * 1024;

/// Max bytes retained from a host child's **stderr**; drain to EOF after the cap.
pub const CAPTURE_MAX_STDERR_BYTES: usize = 256 * 1024;

#[derive(Debug)]
struct BoundedPipeCapture {
    bytes: Vec<u8>,
    truncated: bool,
}

fn drain_reader_bounded(mut r: impl Read, max_stored: usize) -> std::io::Result<BoundedPipeCapture> {
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

/// New process group for the host child so timeout teardown can signal the group.
pub fn prepare_host_command(cmd: &mut Command) {
    cmd.stdin(Stdio::null());
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChildWaitFail {
    WallTimeout { wall_time: Duration },
    Io(String),
}

#[cfg(unix)]
fn kill_process_group_on_timeout(child: &mut std::process::Child) {
    use libc::{kill, SIGKILL};
    let p = child.id() as i32;
    if p != 0 {
        if unsafe { kill(-p, SIGKILL) } < 0 {
            let _ = child.kill();
        }
    } else {
        let _ = child.kill();
    }
}

#[cfg(not(unix))]
fn kill_process_group_on_timeout(child: &mut std::process::Child) {
    let _ = child.kill();
}

fn child_wait_bounded(
    child: &mut std::process::Child,
    wall_time: Duration,
) -> Result<std::process::ExitStatus, ChildWaitFail> {
    let deadline = Instant::now() + wall_time;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    kill_process_group_on_timeout(child);
                    let _ = child.wait();
                    return Err(ChildWaitFail::WallTimeout { wall_time });
                }
                std::thread::sleep(WAIT_POLL);
            }
            Err(err) => return Err(ChildWaitFail::Io(format!("{err}"))),
        }
    }
}

/// `spawn` + wall-bounded wait + bounded stdout/stderr capture.
pub fn host_command_output(
    label: &str,
    wall: Duration,
    mut cmd: Command,
) -> Result<std::process::Output, String> {
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{label}: failed to spawn host child: {e}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("{label}: internal error: stdout not piped"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("{label}: internal error: stderr not piped"))?;

    let stdout_handle = std::thread::spawn(move || {
        drain_reader_bounded(&mut stdout, CAPTURE_MAX_STDOUT_BYTES)
    });
    let stderr_handle = std::thread::spawn(move || {
        drain_reader_bounded(&mut stderr, CAPTURE_MAX_STDERR_BYTES)
    });

    let status = match child_wait_bounded(&mut child, wall) {
        Ok(s) => s,
        Err(ChildWaitFail::WallTimeout { wall_time }) => {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            return Err(format!(
                "{label}: exceeded {:.2}s wall-clock limit (process group killed, fail-closed)",
                wall_time.as_secs_f64()
            ));
        }
        Err(ChildWaitFail::Io(err)) => {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            return Err(format!("{label}: wait on host child failed: {err}"));
        }
    };

    let stdout_cap = stdout_handle
        .join()
        .map_err(|_| format!("{label}: stdout capture thread panicked"))?
        .map_err(|e| format!("{label}: read stdout failed: {e}"))?;
    let stderr_cap = stderr_handle
        .join()
        .map_err(|_| format!("{label}: stderr capture thread panicked"))?
        .map_err(|e| format!("{label}: read stderr failed: {e}"))?;

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

    Ok(std::process::Output {
        status,
        stdout: stdout_cap.bytes,
        stderr: stderr_cap.bytes,
    })
}
