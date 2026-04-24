//! **Layer:** boundary (TESTING.md § test layers — class-5 multi-target
//! toolchain roundtrip).
//!
//! T-Emit lane closure test: `emit_omni_demo_fixtures_green`.
//!
//! Verifies that the "omni demo" fixtures — the subset of `PROGRAM_FIXTURES`
//! whose lowering is supported by Rust, Go, **and** Python — produce
//! identical stdout under each target's native toolchain.
//!
//! Rust always executes (rustc is always available in the build
//! environment).  Go and Python execute when their toolchains are
//! present, and are silently skipped otherwise.  The test therefore
//! passes in CI (rustc-only) and performs the full three-way
//! comparison when run locally with all toolchains installed.
//!
//! Run locally with all toolchains:
//!
//!   cargo test -p v3-compiler --test integration \
//!       emit_omni_demo_fixtures_green -- --nocapture

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

/// Emit `source` as a Go program, run via `go run`, and return trimmed
/// stdout.  Returns `None` when the Go toolchain is not available.
fn go_stdout(fixture_name: &str, source: &str) -> Option<String> {
    let go_available = Command::new("go")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|s| s.success());
    if !go_available {
        return None;
    }

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
    Some(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

/// Emit `source` as a Python program, run via `python3`, and return
/// trimmed stdout.  Returns `None` when python3 is not available.
fn python_stdout(fixture_name: &str, source: &str) -> Option<String> {
    let python_available = Command::new("python3")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|s| s.success());
    if !python_available {
        return None;
    }

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
    Some(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

/// T-Emit lane closure test.
///
/// For each fixture in the "omni" set (those not excluded from any
/// target), emit to Rust, Go, and Python; execute under the target's
/// toolchain; and assert all three outputs agree.
///
/// The omni set is derived automatically from `PROGRAM_FIXTURES` minus
/// `GO_EMIT_EXCLUDE` minus `PYTHON_EMIT_EXCLUDE` — no hard-coded list
/// to keep in sync.
#[test]
fn emit_omni_demo_fixtures_green() {
    let omni_fixtures: Vec<_> = PROGRAM_FIXTURES
        .iter()
        .filter(|f| !GO_EMIT_EXCLUDE.contains(&f.name))
        .filter(|f| !PYTHON_EMIT_EXCLUDE.contains(&f.name))
        .collect();

    assert!(
        !omni_fixtures.is_empty(),
        "omni fixture set must not be empty — check exclude lists"
    );

    for fixture in &omni_fixtures {
        let rust = rust_stdout(fixture.source);

        if let Some(go) = go_stdout(fixture.name, fixture.source) {
            assert_eq!(
                go, rust,
                "Go output diverged from Rust baseline for fixture `{}`",
                fixture.name
            );
        }

        if let Some(py) = python_stdout(fixture.name, fixture.source) {
            assert_eq!(
                py, rust,
                "Python output diverged from Rust baseline for fixture `{}`",
                fixture.name
            );
        }
    }
}
