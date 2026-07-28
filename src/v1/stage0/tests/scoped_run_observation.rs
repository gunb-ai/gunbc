use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn primary_fixture() -> (PathBuf, PathBuf) {
    let root = repo_root().join("target").join(format!(
        "gunbc-scoped-run-primary-{}-{}",
        std::process::id(),
        FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).expect("create primary fixture root");
    let entry = root.join("scoped_run_observation.dag");
    std::fs::write(
        &entry,
        "module test.fixtures.scoped_run_observation\n\
         fn claim_true() -> Bool { true }\n\
         fn claim_false() -> Bool { false }\n\
         fn claim_non_bool() -> String { \"not a claim verdict\" }\n\
         type ProcessExit = ExitSuccess | ExitFailure { code: Int, reason: String }\n\
         fn exit_failure() -> ProcessExit { ExitFailure { code: 7, reason: \"legacy failure reason\" } }\n",
    )
    .expect("write primary fixture");
    (root, entry)
}

fn gunbc_args(root: &Path, entry: &Path, function: &str) -> Vec<String> {
    vec![
        "run".to_string(),
        "--source-root".to_string(),
        root.to_string_lossy().into_owned(),
        "--entry".to_string(),
        entry.to_string_lossy().into_owned(),
        "--function".to_string(),
        function.to_string(),
        "--claim-run".to_string(),
    ]
}

fn run_pipe(function: &str) -> Output {
    let (root, entry) = primary_fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args(gunbc_args(&root, &entry, function))
        .output()
        .expect("run scoped gunbc pipe capture");
    let _ = std::fs::remove_dir_all(root);
    output
}

fn run_pipe_with_diagnostics(function: &str) -> Output {
    let (root, entry) = primary_fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args(gunbc_args(&root, &entry, function))
        .env("GUNBC_RECOMPUTE_TRACE", "1")
        // Zero timeout is a deterministic fast-vs-timed control: once capture
        // arms the producer, recv_timeout(0) can fire before the joined end dump.
        .env("GUNBC_FLATTEN_SITE_DUMP_SECS", "0")
        .env("GUNBC_SCOPED_PERIODIC_WITNESS", "1")
        .env("GUNBC_SCOPED_OBSERVATION_RECEIPT", "1")
        .output()
        .expect("run scoped gunbc deferred-diagnostic capture");
    let _ = std::fs::remove_dir_all(root);
    output
}

fn run_unscoped(function: &str, claim_run: bool) -> Output {
    let (root, _) = primary_fixture();
    let relative_root = root
        .strip_prefix(repo_root())
        .expect("fixture is under repo root")
        .to_string_lossy()
        .into_owned();
    let mut args = vec![
        "run".to_string(),
        "--source-root".to_string(),
        relative_root,
        "--function".to_string(),
        function.to_string(),
    ];
    if claim_run {
        args.push("--claim-run".to_string());
    }
    let output = Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("run unscoped gunbc control");
    let _ = std::fs::remove_dir_all(root);
    output
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
        !stderr.contains("scoped-periodic-ticks="),
        "fast control armed a timed producer: {stderr}"
    );
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
fn scoped_non_bool_claim_is_refused_persistent_and_nonzero() {
    let output = run_pipe("claim_non_bool");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(output.stdout, b"not a claim verdict\n");
    assert!(!output.stderr.contains(&b'\r'), "{:?}", output.stderr);
    assert!(!output.stderr.contains(&0x1b), "{:?}", output.stderr);
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains(
            "⛔ decl=test.fixtures.scoped_run_observation::claim_non_bool#whole refused:"
        ),
        "{stderr}"
    );
}

#[test]
fn scoped_deferred_diagnostics_are_preserved_before_final() {
    let output = run_pipe_with_diagnostics("claim_true");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"true\n");
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    let recompute = stderr
        .find("[recompute-trace]")
        .expect("recompute trace kept");
    let periodic = stderr
        .find("--- free_monoid_to_vec by call site")
        .expect("periodic/end instrumentation kept");
    let final_line = stderr
        .rfind("finished with 0 problems shown")
        .expect("modeled Final kept");
    assert!(recompute < final_line, "{stderr}");
    assert!(periodic < final_line, "{stderr}");
    assert!(
        !stderr[final_line..].contains("[recompute-trace]")
            && !stderr[final_line..].contains("--- free_monoid_to_vec"),
        "Final must remain the last stderr diagnostic: {stderr}"
    );
    let ticks = stderr
        .split("[scoped-periodic-ticks=")
        .nth(1)
        .and_then(|tail| tail.split(']').next())
        .and_then(|count| count.parse::<usize>().ok())
        .expect("deterministic periodic tick receipt");
    assert!(
        ticks > 0,
        "timed producer did not fire under zero-timeout control: {stderr}"
    );
    let ticks_marker = stderr.find("[scoped-periodic-ticks=").expect("tick marker");
    assert!(
        ticks_marker < final_line,
        "timed receipt must precede Final: {stderr}"
    );
    let receipt = stderr
        .split("[scoped-observation-receipt wall_ns=")
        .nth(1)
        .and_then(|tail| tail.split(']').next())
        .expect("exact Nanosecond production receipt");
    let mut receipt_parts = receipt.split(" seed_resolution_ns=");
    let wall_ns = receipt_parts
        .next()
        .and_then(|value| value.parse::<u128>().ok())
        .expect("wall Nanosecond receipt");
    let seed_ns = receipt_parts
        .next()
        .and_then(|value| value.parse::<u128>().ok())
        .expect("seed-resolution overhead receipt");
    assert!(seed_ns > 0 && wall_ns >= seed_ns, "{stderr}");
    assert!(
        wall_ns % 1_000_000 != 0,
        "Nanosecond receipt was truncated to milliseconds: {stderr}"
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
    let (root, entry) = primary_fixture();
    let command = std::iter::once(env!("CARGO_BIN_EXE_gunbc"))
        .map(str::to_string)
        .chain(gunbc_args(&root, &entry, "claim_true"))
        .collect::<Vec<_>>()
        .join(" ");
    let status = Command::new("script")
        .current_dir(repo_root())
        // TERM=dumb downgrades emoji capability while the PTY remains
        // addressable; exact CR+EL2 proves cursor capability is independent.
        .env("TERM", "dumb")
        .args(["-qefc", &command])
        .arg(&transcript)
        .status()
        .expect("run scoped gunbc PTY capture");
    assert!(status.success());
    let bytes = std::fs::read(&transcript).expect("read PTY transcript");
    let _ = std::fs::remove_file(&transcript);
    let _ = std::fs::remove_dir_all(root);
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
    assert!(!exit_failure.status.success(), "{exit_failure:?}");
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

#[test]
fn scoped_declaration_identity_follows_the_selected_function_node() {
    let fixture_root = repo_root()
        .join("target")
        .join(format!("gunbc-scoped-run-identity-{}", std::process::id()));
    std::fs::create_dir_all(&fixture_root).expect("create identity fixture root");
    let imported = fixture_root.join("imported.dag");
    let entry = fixture_root.join("entry.dag");
    std::fs::write(
        &imported,
        "module test.scoped_identity.imported\n\nfn imported_claim() -> Bool { true }\n",
    )
    .expect("write imported fixture");
    std::fs::write(
        &entry,
        "module test.scoped_identity.entry\n\nimport test.scoped_identity.imported { imported_claim }\n",
    )
    .expect("write entry fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args([
            "run",
            "--source-root",
            fixture_root.to_str().expect("fixture root utf8"),
            "--entry",
            entry.to_str().expect("entry utf8"),
            "--function",
            "imported_claim",
            "--claim-run",
        ])
        .output()
        .expect("run imported scoped function");
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("decl=test.scoped_identity.imported::imported_claim#whole"),
        "{stderr}"
    );

    let qualified_output = Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args([
            "run",
            "--source-root",
            fixture_root.to_str().expect("fixture root utf8"),
            "--entry",
            entry.to_str().expect("entry utf8"),
            "--function",
            "test.scoped_identity.imported.imported_claim",
            "--claim-run",
        ])
        .output()
        .expect("run qualified imported scoped function");
    assert!(qualified_output.status.success(), "{qualified_output:?}");
    let qualified_stderr = String::from_utf8(qualified_output.stderr).expect("utf8 stderr");
    assert!(
        qualified_stderr.contains("decl=test.scoped_identity.imported::imported_claim#whole"),
        "qualified selection must carry the selected node identity: {qualified_stderr}"
    );

    let left = fixture_root.join("left.dag");
    let right = fixture_root.join("right.dag");
    let ambiguous = fixture_root.join("ambiguous.dag");
    std::fs::write(
        &left,
        "module test.scoped_identity.left\n\nfn same_claim() -> Bool { true }\n",
    )
    .expect("write left fixture");
    std::fs::write(
        &right,
        "module test.scoped_identity.right\n\nfn same_claim() -> Bool { true }\n",
    )
    .expect("write right fixture");
    std::fs::write(
        &ambiguous,
        "module test.scoped_identity.ambiguous\n\nimport test.scoped_identity.left { same_claim }\nimport test.scoped_identity.right { same_claim }\n",
    )
    .expect("write ambiguous fixture");
    let ambiguous_output = Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args([
            "run",
            "--source-root",
            fixture_root.to_str().expect("fixture root utf8"),
            "--entry",
            ambiguous.to_str().expect("ambiguous entry utf8"),
            "--function",
            "same_claim",
            "--claim-run",
        ])
        .output()
        .expect("run ambiguous scoped function");
    assert!(!ambiguous_output.status.success(), "{ambiguous_output:?}");
    assert!(
        !String::from_utf8_lossy(&ambiguous_output.stderr).contains("started decl="),
        "an ambiguous function must not mint an observation identity: {ambiguous_output:?}"
    );
    let _ = std::fs::remove_dir_all(&fixture_root);
}
