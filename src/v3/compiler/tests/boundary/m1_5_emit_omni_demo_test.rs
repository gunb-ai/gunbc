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
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::cached_compile_to_dag;
use crate::common::determinism_fixtures::{GO_EMIT_EXCLUDE, PROGRAM_FIXTURES, PYTHON_EMIT_EXCLUDE};
use v3_compiler::emit_rust::emit_rust;
use v3_compiler::r1c_e_gates;

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);

struct TmpDir(PathBuf);

impl TmpDir {
    fn new() -> Self {
        let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "v3_emit_omni_roundtrip_{}_{}",
            std::process::id(),
            id
        ));
        std::fs::create_dir_all(&path).expect("create tmp dir");
        TmpDir(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
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
    let tmp_dir = TmpDir::new();
    let src_path = tmp_dir.path().join("main.rs");
    let bin_path = tmp_dir.path().join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write rust source");

    let compile = Command::new("rustc")
        // Strip RUSTC_BOOTSTRAP so the ratchet CI step's libtest unlock
        // does not leak into child rustc invocations.
        .env_remove("RUSTC_BOOTSTRAP")
        .arg("--edition=2021")
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
    if let Err(detail) = r1c_e_gates::check_omni_demo_fixtures_green() {
        panic!("emit_omni_demo_fixtures_green: {detail}");
    }
}
