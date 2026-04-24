//! **Layer:** boundary (TESTING.md § test layers — class-5 multi-target
//! toolchain roundtrip).
//!
//! Two tests live here:
//!
//! * `emit_omni_demo_rust_roundtrip` — non-ignored CI gate.  Emits all
//!   omni-set fixtures to Rust, compiles via rustc, and checks stdout.
//!   Rustc is always available so this runs unconditionally.
//!
//! * `emit_omni_demo_fixtures_green` — T-Emit lane closure receipt.
//!   Marked `#[ignore]` because it requires Go **and** Python toolchains
//!   that are not present in CI.  When run with `--ignored` it asserts
//!   both toolchains are reachable and fails hard if either is absent;
//!   a missing toolchain is an unmet receipt, not a skip.
//!
//! Run the full three-way proof locally:
//!
//!   cargo test -p v3-compiler --test integration \
//!       emit_omni_demo_fixtures_green -- --ignored --nocapture

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::cached_compile_to_dag;
use crate::common::determinism_fixtures::{GO_EMIT_EXCLUDE, PROGRAM_FIXTURES, PYTHON_EMIT_EXCLUDE};
use v3_compiler::emit::{emit, EmitTarget};
use v3_compiler::emit_rust::emit_rust;

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);

fn next_roundtrip_dir() -> PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_emit_omni_roundtrip_{}_{}",
        std::process::id(),
        id
    ))
}

fn omni_fixtures() -> Vec<&'static crate::common::determinism_fixtures::ProgramFixture> {
    PROGRAM_FIXTURES
        .iter()
        .filter(|f| !GO_EMIT_EXCLUDE.contains(&f.name))
        .filter(|f| !PYTHON_EMIT_EXCLUDE.contains(&f.name))
        .collect()
}

/// Emit `source` as a Rust program, compile via rustc, run, and return
/// trimmed stdout.  Panics if rustc compilation or execution fails.
fn rust_stdout(source: &str) -> String {
    let dag = cached_compile_to_dag(source, "omni_parity.v3");
    let rendered = emit_rust(&dag).expect("Rust emit succeeded");
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write rust source");

    let compile = Command::new("rustc")
        // Strip RUSTC_BOOTSTRAP so the ratchet CI step's libtest unlock
        // does not leak into child rustc invocations.
        .env_remove("RUSTC_BOOTSTRAP")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc");
    assert!(
        compile.success(),
        "rustc failed on emitted source:\n{rendered}"
    );

    let run = Command::new(&bin_path).output().expect("run rust binary");
    assert!(run.status.success(), "compiled rust binary exited non-zero");
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

/// Emit `source` as a Go program, run via `go run`, and return trimmed stdout.
/// Panics (not `None`) — callers must only invoke this when `go` is confirmed present.
fn go_stdout(fixture_name: &str, source: &str) -> String {
    let dag = cached_compile_to_dag(source, "omni_parity.v3");
    let rendered = emit(&dag, EmitTarget::Go)
        .unwrap_or_else(|e| panic!("Go emit failed for fixture `{fixture_name}`: {e:?}"))
        .text;
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.go");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write go source");

    let run = Command::new("go")
        .arg("run")
        .arg(&src_path)
        .current_dir(&tmp_dir)
        .output()
        .expect("invoke go run");
    assert!(
        run.status.success(),
        "go run failed for fixture `{fixture_name}`:\nsource:\n{rendered}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

/// Emit `source` as a Python program, run via `python3`, and return trimmed stdout.
/// Panics (not `None`) — callers must only invoke this when `python3` is confirmed present.
fn python_stdout(fixture_name: &str, source: &str) -> String {
    let dag = cached_compile_to_dag(source, "omni_parity.v3");
    let rendered = emit(&dag, EmitTarget::Python)
        .unwrap_or_else(|e| panic!("Python emit failed for fixture `{fixture_name}`: {e:?}"))
        .text;
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.py");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write python source");

    let run = Command::new("python3")
        .arg(&src_path)
        .output()
        .expect("invoke python3");
    assert!(
        run.status.success(),
        "python3 failed for fixture `{fixture_name}`:\nsource:\n{rendered}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stderr),
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn toolchain_available(cmd: &str, probe_arg: &str) -> bool {
    Command::new(cmd)
        .arg(probe_arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|s| s.success())
}

/// CI gate: Rust-only roundtrip over all omni fixtures.
///
/// Rustc is always available; this test runs unconditionally.  It proves
/// that the omni fixture set emits valid, executable Rust — the minimum bar
/// for every commit.
#[test]
fn emit_omni_demo_rust_roundtrip() {
    let fixtures = omni_fixtures();
    assert!(
        !fixtures.is_empty(),
        "omni fixture set must not be empty — check exclude lists"
    );
    for fixture in &fixtures {
        rust_stdout(fixture.source);
    }
}

/// T-Emit lane closure receipt: all three targets produce identical stdout.
///
/// Marked `#[ignore]` because `go` and `python3` are not present in CI.
/// Run locally with `--ignored` to execute the full three-way proof.  The
/// test asserts both toolchains are reachable and **fails hard** if either
/// is absent — a missing toolchain is an unmet receipt, not a skip.
#[test]
#[ignore = "requires go and python3 toolchains; run locally: cargo test ... -- --ignored --nocapture"]
fn emit_omni_demo_fixtures_green() {
    assert!(
        toolchain_available("go", "version"),
        "go toolchain not found — this test requires go to be on PATH"
    );
    assert!(
        toolchain_available("python3", "--version"),
        "python3 toolchain not found — this test requires python3 to be on PATH"
    );

    let fixtures = omni_fixtures();
    assert!(
        !fixtures.is_empty(),
        "omni fixture set must not be empty — check exclude lists"
    );

    for fixture in &fixtures {
        let rust = rust_stdout(fixture.source);

        let go = go_stdout(fixture.name, fixture.source);
        assert_eq!(
            go, rust,
            "Go output diverged from Rust baseline for fixture `{}`",
            fixture.name
        );

        let py = python_stdout(fixture.name, fixture.source);
        assert_eq!(
            py, rust,
            "Python output diverged from Rust baseline for fixture `{}`",
            fixture.name
        );
    }
}
