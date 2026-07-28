use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ENTRY: &str = "src/v1/stage0/tests/fixtures/scoped_run_observation.dag";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn gunbc_args(function: &str) -> Vec<&str> {
    vec![
        "run",
        "--source-root",
        "dag",
        "--source-root",
        "src/v2",
        "--source-root",
        "src/v1/stage0/tests/fixtures",
        "--entry",
        ENTRY,
        "--function",
        function,
        "--claim-run",
    ]
}

fn run_pipe(function: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args(gunbc_args(function))
        .output()
        .expect("run scoped gunbc pipe capture")
}

#[test]
fn scoped_run_pipe_keeps_stdout_for_value_and_stderr_control_free() {
    let output = run_pipe("claim_true");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"true\n");
    assert!(!output.stderr.contains(&b'\r'), "{:?}", output.stderr);
    assert!(!output.stderr.contains(&0x1b), "{:?}", output.stderr);
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("◐ started claim_true\n"), "{stderr}");
    assert!(
        stderr.contains("✦ claim_true finished with 0 problems shown"),
        "{stderr}"
    );
}

#[test]
fn scoped_claim_refusal_is_persistent_and_nonzero() {
    let output = run_pipe("claim_false");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(output.stdout, b"false\n");
    assert!(!output.stderr.contains(&b'\r'), "{:?}", output.stderr);
    assert!(!output.stderr.contains(&0x1b), "{:?}", output.stderr);
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("⛔ claim_false refused: claim returned false"),
        "{stderr}"
    );
}

#[test]
fn scoped_run_pty_begin_uses_the_projected_overwrite_wire() {
    let script = Command::new("script").arg("--version").output();
    if script.is_err() {
        eprintln!("skipping PTY capture: util-linux script is unavailable");
        return;
    }
    let transcript = std::env::temp_dir().join(format!(
        "gunbc-scoped-run-observation-{}-{}.typescript",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let command = std::iter::once(env!("CARGO_BIN_EXE_gunbc"))
        .chain(gunbc_args("claim_true"))
        .collect::<Vec<_>>()
        .join(" ");
    let status = Command::new("script")
        .current_dir(repo_root())
        .args(["-qefc", &command])
        .arg(&transcript)
        .status()
        .expect("run scoped gunbc PTY capture");
    assert!(status.success());
    let bytes = std::fs::read(&transcript).expect("read PTY transcript");
    let _ = std::fs::remove_file(&transcript);
    assert!(
        bytes
            .windows(b"\r\x1b[2K\xe2\x97\x90 started claim_true".len())
            .any(|w| w == b"\r\x1b[2K\xe2\x97\x90 started claim_true"),
        "missing exact projected Begin wire in PTY transcript"
    );
    assert!(
        bytes
            .windows(b"\r\x1b[2K\xe2\x9c\xa6 claim_true finished".len())
            .any(|w| w == b"\r\x1b[2K\xe2\x9c\xa6 claim_true finished"),
        "clean Final did not close the dynamic line through projected append"
    );
}
