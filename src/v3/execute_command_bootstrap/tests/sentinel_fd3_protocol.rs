#![cfg(target_os = "linux")]

//! Hermetic receipts for the fd3 sentinel stream (`s` / `e` / `f`) and CLOEXEC-before-exec
//! behavior. The parent simulates the runner by wiring the write end of a pipe to fd 3 in the
//! child before exec.

use std::io::Read;
use std::os::unix::io::IntoRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;

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
            if libc::dup2(write_fd, 3) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if write_fd != 3 {
                libc::close(write_fd);
            }
            libc::close(read_fd);
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
fn exec_true_emits_se_then_eof_on_sentinel_fd() {
    let (r, w) = pipe();
    let bin = env!("CARGO_BIN_EXE_gunbc_execute_command_bootstrap");
    let mut child = spawn_bootstrap_with_sentinel_fd(bin, r, w, &["true"]);
    unsafe {
        libc::close(w);
    }
    let buf = read_pipe_to_end(r);
    let status = child.wait().expect("wait");
    assert!(status.success(), "status={status:?}");
    assert_eq!(buf.as_slice(), b"se", "got={buf:?}");
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
