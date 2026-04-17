// Each test binary under `tests/` includes this module via
// `mod common;`. A given binary may exercise only a subset of the API
// surface (e.g. tests that only compile reflected-module harnesses
// never touch `HarnessLinkMode::Standalone`). Suppress dead-code
// warnings per-binary rather than carry a separate "which features
// does this binary use" declaration.
#![allow(dead_code)]

//! Shared test harness for rustc-roundtrip integration tests.
//!
//! Lazy DI / RAII pattern. A `RustcHarness` is constructed once per
//! test binary, holds the compiled rlib discovery state, and hands out
//! temp-dir scratch space keyed by an atomic counter so tests running
//! in parallel do not collide.
//!
//! Individual test files use `std::sync::OnceLock<PathBuf>` to compile
//! their batched harness binaries on first access; once compiled, the
//! binary is reused across every `#[test]` in that file. This is the
//! concrete amortization win — without it, each `#[test]` that
//! spawned rustc paid ~3-5s of cold codegen on CI.
//!
//! The harness does not auto-clean its scratch directory on Drop
//! because the OS eventually reaps `/tmp` and because panics in tests
//! benefit from leaving artifacts visible.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Handle into the current test binary's `target/debug/deps` directory.
/// All `--extern v3_compiler=...` flags resolve against this.
pub fn deps_dir() -> PathBuf {
    std::env::current_exe()
        .expect("current test binary path")
        .parent()
        .expect("deps dir")
        .to_path_buf()
}

/// Most recently built `.rlib` for `crate_name` in the deps directory.
/// Picks the newest file so concurrent builds don't mismatch against
/// stale rlibs from earlier test runs.
pub fn find_current_rlib(crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let mut matches: Vec<PathBuf> = std::fs::read_dir(deps_dir())
        .expect("read deps dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let file_name = path.file_name()?.to_str()?;
            if file_name.starts_with(&prefix) && file_name.ends_with(".rlib") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    matches.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    });
    matches
        .into_iter()
        .last()
        .expect("compiled rlib for current crate")
}

/// Harness for spawning rustc against generated-Rust harnesses.
/// Holds the per-test-binary scratch root and a counter so parallel
/// tests within the same binary get distinct temp dirs.
pub struct RustcHarness {
    scratch_dir: PathBuf,
    child_index: AtomicUsize,
}

/// How the harness should link the compiled Rust:
/// - `Standalone`: plain `rustc` invocation. Use for fully self-
///   contained programs (emit_rust output with no external deps).
/// - `WithV3Compiler`: pass `-L dependency=...` and `--extern
///   v3_compiler=...` so the harness can import `v3_compiler::dag::*`
///   at runtime (module-mode emissions, reflected-DAG walkers).
pub enum HarnessLinkMode {
    Standalone,
    WithV3Compiler,
}

impl RustcHarness {
    /// Construct a new harness rooted at a scratch dir named after
    /// the caller's test binary pid. Two different test files can
    /// safely construct their own harnesses — they will not share
    /// scratch space.
    pub fn new(scope: &str) -> Self {
        let pid = std::process::id();
        let scratch_dir = std::env::temp_dir().join(format!("v3_{scope}_{pid}"));
        Self {
            scratch_dir,
            child_index: AtomicUsize::new(0),
        }
    }

    /// Allocate a fresh scratch subdir for a compilation. Used
    /// internally by `compile`; exposed so callers that need to
    /// inject additional files (fixture data, test inputs) can
    /// share the same directory.
    pub fn next_child_dir(&self) -> PathBuf {
        let id = self.child_index.fetch_add(1, Ordering::Relaxed);
        let path = self.scratch_dir.join(format!("c{id}"));
        std::fs::create_dir_all(&path).expect("create harness child dir");
        path
    }

    /// Compile `rust_source` to a binary under `bin_name`, returning
    /// the binary path. Caller is responsible for caching the path
    /// (e.g. in a `OnceLock<PathBuf>`) if multiple tests should
    /// share one compilation.
    pub fn compile(&self, rust_source: &str, bin_name: &str, mode: HarnessLinkMode) -> PathBuf {
        let tmp_dir = self.next_child_dir();
        let src_path = tmp_dir.join("main.rs");
        let bin_path = tmp_dir.join(bin_name);
        std::fs::File::create(&src_path)
            .and_then(|mut f| f.write_all(rust_source.as_bytes()))
            .expect("write harness source");

        let mut cmd = Command::new("rustc");
        cmd.arg("--edition=2021")
            .arg(&src_path)
            .arg("-o")
            .arg(&bin_path);

        if let HarnessLinkMode::WithV3Compiler = mode {
            let deps = deps_dir();
            let rlib = find_current_rlib("v3_compiler");
            cmd.arg("-L")
                .arg(format!("dependency={}", deps.display()))
                .arg("--extern")
                .arg(format!("v3_compiler={}", rlib.display()));
        }

        let status = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("invoke rustc — install a rust toolchain to run this test");
        assert!(status.success(), "rustc failed on harness source");
        bin_path
    }

    /// Run a compiled harness binary with `args`, asserting success
    /// and returning trimmed stdout. On failure the stderr is
    /// surfaced in the panic message so CI logs identify which
    /// dispatch case broke.
    pub fn run(bin: &Path, args: &[&str]) -> String {
        let output = Command::new(bin)
            .args(args)
            .output()
            .expect("run compiled harness");
        assert!(
            output.status.success(),
            "compiled harness failed for args {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}
