#![cfg(target_os = "linux")]

//! Hermetic receipts for the fd3 sentinel stream (`s` / `e` / `f`) and CLOEXEC-before-exec
//! behavior. The parent simulates the runner by wiring the write end of a pipe to fd 3 in the
//! child before exec.
//!
//! The successful-exec receipt uses a **long-running** logical child (`/usr/bin/sleep`): if fd 3
//! were not `FD_CLOEXEC`’d before `execvp`, the write end would stay open until that child exits,
//! and a full-pipe read would block for the sleep duration. Observing **`se` + EOF** while the
//! child is still running proves the write end was dropped at **exec** time.

use std::io::Read;
use std::os::fd::FromRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn pipe() -> (i32, i32) {
    let mut fds = [0_i32; 2];
    unsafe {
        assert_eq!(libc::pipe(fds.as_mut_ptr()), 0, "pipe(2)");
    }
    (fds[0], fds[1])
}

fn spawn_bootstrap_with_sentinel_fd(
    bin: &str,
    read_fd: i32,
    write_fd: i32,
    args: &[&str],
) -> std::process::Child {
    let mut cmd = Command::new(bin);
    for a in args {
        cmd.arg(a);
    }
    unsafe {
        cmd.pre_exec(move || {
            // Close the child's read end **before** `dup2(write, 3)`. If `read_fd == 3` (common
            // right after stdio), `dup2` reuses fd 3 for the write end — a later `close(read_fd)`
            // would then close the sentinel channel the bootstrap writes on (exit 124 in the
            // helper).
            libc::close(read_fd);
            if libc::dup2(write_fd, 3) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if write_fd != 3 {
                libc::close(write_fd);
            }
            Ok(())
        });
    }
    cmd.spawn().expect("spawn bootstrap")
}

fn read_pipe_to_end(read_fd: i32) -> Vec<u8> {
    unsafe {
        let mut file = std::fs::File::from_raw_fd(read_fd);
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("read sentinel pipe");
        buf
    }
}

#[test]
fn successful_exec_eof_before_child_exit_proves_fd3_cloexec_before_execvp() {
    let (r, w) = pipe();
    let bin = env!("CARGO_BIN_EXE_gunbc_execute_command_bootstrap");
    // Long enough that a leaked fd 3 (no CLOEXEC) would keep the pipe write end open for seconds.
    let mut child = spawn_bootstrap_with_sentinel_fd(bin, r, w, &["/usr/bin/sleep", "60"]);
    unsafe {
        libc::close(w);
    }

    let (tx, rx) = mpsc::sync_channel(1);
    let reader = thread::spawn(move || {
        let buf = read_pipe_to_end(r);
        let _ = tx.send(buf);
    });

    let fast_eof = Duration::from_secs(5);
    let buf = match rx.recv_timeout(fast_eof) {
        Ok(b) => b,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "sentinel pipe read blocked >{fast_eof:?}; \
                 if fd 3 were not FD_CLOEXEC before execvp, `/usr/bin/sleep` would inherit the pipe \
                 write end and EOF would not arrive until sleep exits"
            );
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("reader thread exited before sending pipe bytes");
        }
    };

    assert_eq!(buf.as_slice(), b"se", "got={buf:?}");

    let _ = child.kill();
    let _ = child.wait().expect("reap child");
    reader.join().expect("join reader");
}

#[test]
fn exec_missing_file_emits_sef() {
    let (r, w) = pipe();
    let bin = env!("CARGO_BIN_EXE_gunbc_execute_command_bootstrap");
    let missing = "/nonexistent/gunbc_execute_command_bootstrap_probe_9f2a1c";
    let mut child = spawn_bootstrap_with_sentinel_fd(bin, r, w, &[missing]);
    unsafe {
        libc::close(w);
    }
    let buf = read_pipe_to_end(r);
    let status = child.wait().expect("wait");
    assert_eq!(status.code(), Some(127));
    assert_eq!(buf.as_slice(), b"sef", "got={buf:?}");
}
