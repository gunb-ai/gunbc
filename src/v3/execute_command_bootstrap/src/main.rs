//! `gunbc_execute_command_bootstrap` — Linux-only helper invoked under `unshare(1)` instead of a
//! pure `sh -c` bootstrap so **fd 3** can be marked `FD_CLOEXEC` immediately before `execvp(3)`,
//! closing the sentinel channel atomically on successful exec (POSIX `sh` cannot do this between
//! sentinel writes and `exec`).
//!
//! **Protocol on fd 3** (write end held by this process until `exec` or terminal failure):
//! - `s` — helper started
//! - `e` — about to `execvp` the logical user command
//! - `f` — `execvp` returned (final exec failed); parent may observe `sef` vs `se` + EOF
//!
//! **Invocation:** `argv[1]` is the logical program (`execvp` file), `argv[2..]` are its arguments.
//! This matches the prior `sh -c 'exec …' "$0" "$@"'` shape (`$0` / `$@` after the script).

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("gunbc_execute_command_bootstrap: Linux only (build artifact for non-Linux CI).");
    std::process::exit(2);
}

#[cfg(target_os = "linux")]
fn main() {
    linux::main();
}

#[cfg(target_os = "linux")]
mod linux {
    use std::ffi::{CString, OsStr};
    use std::io::ErrorKind;
    use std::os::unix::ffi::OsStrExt;
    use std::process::exit;

    const SENTINEL_FD: i32 = 3;

    fn write_all(fd: i32, bytes: &[u8]) -> std::io::Result<()> {
        let mut off = 0usize;
        while off < bytes.len() {
            let n = unsafe {
                libc::write(
                    fd,
                    bytes[off..].as_ptr().cast::<libc::c_void>(),
                    bytes.len() - off,
                )
            };
            if n < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if n == 0 {
                return Err(std::io::Error::new(
                    ErrorKind::WriteZero,
                    "write(2) returned 0 on sentinel fd",
                ));
            }
            off += n as usize;
        }
        Ok(())
    }

    fn set_cloexec_on_sentinel_fd() -> std::io::Result<()> {
        let flags = unsafe { libc::fcntl(SENTINEL_FD, libc::F_GETFD) };
        if flags < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let rc = unsafe { libc::fcntl(SENTINEL_FD, libc::F_SETFD, flags | libc::FD_CLOEXEC) };
        if rc < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    fn cstrings_from_user_argv() -> Result<Vec<CString>, ()> {
        let mut out = Vec::new();
        for arg in std::env::args_os().skip(1) {
            let Some(cs) = cstring_from_os_str(&arg) else {
                return Err(());
            };
            out.push(cs);
        }
        Ok(out)
    }

    fn cstring_from_os_str(s: &OsStr) -> Option<CString> {
        CString::new(s.as_bytes()).ok()
    }

    pub(super) fn main() {
        let user_argv = match cstrings_from_user_argv() {
            Ok(v) if !v.is_empty() => v,
            Ok(_) => {
                // No logical command — invalid invocation; do not emit partial sentinel stream.
                exit(127);
            }
            Err(()) => {
                // Cannot represent argv for `execvp` (e.g. interior NUL) — same `sef` surface as a
                // failed exec so the parent can fail-closed.
                let _ = write_all(SENTINEL_FD, b"s");
                let _ = write_all(SENTINEL_FD, b"e");
                let _ = write_all(SENTINEL_FD, b"f");
                exit(125);
            }
        };

        if let Err(_) = write_all(SENTINEL_FD, b"s") {
            exit(124);
        }

        let mut argv: Vec<*const libc::c_char> = user_argv
            .iter()
            .map(|s| s.as_ptr().cast::<libc::c_char>())
            .collect();
        argv.push(std::ptr::null());

        if let Err(_) = write_all(SENTINEL_FD, b"e") {
            let _ = write_all(SENTINEL_FD, b"f");
            exit(123);
        }

        if let Err(_) = set_cloexec_on_sentinel_fd() {
            let _ = write_all(SENTINEL_FD, b"f");
            exit(122);
        }

        unsafe {
            libc::execvp(argv[0], argv.as_ptr());
        }

        let _ = write_all(SENTINEL_FD, b"f");
        let code = if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            127
        } else {
            126
        };
        exit(code);
    }
}
