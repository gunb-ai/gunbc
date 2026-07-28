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

fn run_unscoped(function: &str, claim_run: bool) -> Output {
    let mut args = vec![
        "run",
        "--source-root",
        "src/v1/stage0/tests/fixtures",
        "--function",
        function,
    ];
    if claim_run {
        args.push("--claim-run");
    }
    Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("run unscoped gunbc control")
}

#[test]
fn scoped_run_pipe_keeps_stdout_for_value_and_stderr_control_free() {
    let output = run_pipe("claim_true");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"true\n");
    assert!(!output.stderr.contains(&b'\r'), "{:?}", output.stderr);
    assert!(!output.stderr.contains(&0x1b), "{:?}", output.stderr);
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("◐ started decl=test.fixtures.scoped_run_observation::claim_true#whole\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "✦ decl=test.fixtures.scoped_run_observation::claim_true#whole finished with 0 problems shown"
        ),
        "{stderr}"
    );
}

#[test]
fn scoped_negative_claim_is_failed_persistent_and_nonzero() {
    let output = run_pipe("claim_false");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(output.stdout, b"false\n");
    assert!(!output.stderr.contains(&b'\r'), "{:?}", output.stderr);
    assert!(!output.stderr.contains(&0x1b), "{:?}", output.stderr);
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("✗ decl=test.fixtures.scoped_run_observation::claim_false#whole failed: claim returned false"),
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
            .windows(b"\r\x1b[2K\xe2\x97\x90 started decl=test.fixtures.scoped_run_observation::claim_true#whole".len())
            .any(|w| w == b"\r\x1b[2K\xe2\x97\x90 started decl=test.fixtures.scoped_run_observation::claim_true#whole"),
        "missing exact projected Begin wire in PTY transcript"
    );
    assert!(
        bytes
            .windows(b"\r\x1b[2K\xe2\x9c\xa6 decl=test.fixtures.scoped_run_observation::claim_true#whole finished".len())
            .any(|w| w == b"\r\x1b[2K\xe2\x9c\xa6 decl=test.fixtures.scoped_run_observation::claim_true#whole finished"),
        "clean Final did not close the dynamic line through projected append"
    );
}

#[test]
fn unscoped_diagnostics_remain_byte_stable() {
    let claim_without_entry = run_unscoped("claim_true", true);
    assert_eq!(claim_without_entry.status.code(), Some(1));
    assert_eq!(
        claim_without_entry.stderr,
        b"error: --claim-run requires --entry <file.dag> (scoped import closure; loading the whole --source-root tree is too large for witness runs)\n"
    );

    let exit_failure = run_unscoped("exit_failure", false);
    assert_eq!(exit_failure.status.code(), Some(7));
    assert!(
        String::from_utf8_lossy(&exit_failure.stderr).contains("legacy failure reason\n"),
        "{exit_failure:?}"
    );

    let wrong_return = run_unscoped("claim_true", false);
    assert_eq!(wrong_return.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&wrong_return.stderr)
            .contains("not `ProcessExit`. Functions invoked via `dag run`"),
        "{wrong_return:?}"
    );

    let runtime_error = run_unscoped("missing_function", false);
    assert_eq!(runtime_error.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&runtime_error.stderr).contains("runtime error:"),
        "{runtime_error:?}"
    );
}
