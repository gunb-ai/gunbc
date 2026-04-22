// Each test binary under `tests/` includes this module via
// `mod common;`. A given binary may exercise only a subset of the API
// surface (e.g. tests that only compile reflected-module harnesses
// never touch `HarnessLinkMode::Standalone`). Suppress dead-code
// warnings per-binary rather than carry a separate "which features
// does this binary use" declaration.
#![allow(dead_code, unused_imports)]

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

pub mod budgeted;
pub mod cached_compile;
pub mod determinism_fixtures;
pub mod substrate_receipts;

pub use cached_compile::{
    cached_compile_any, cached_compile_outcome, cached_compile_to_dag, CachedCompileOutcome,
};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum IntegrationRsScan {
    Code,
    LineComment,
    BlockComment(u32),
    String,
}

/// True when `needle` appears in `integration_rs` **outside** Rust line comments
/// (`//` … including `///` / `//!`), **nested** block comments (`/* … */`), and
/// normal `"…"` string literals.
///
/// Band-C wiring ratchets use this so `#[path = …]` / `mod …;` matches cannot
/// false-green on commented-out or string-embedded copies. Needles are ASCII
/// (`#[path`, `mod foo`); the scan is byte-oriented on UTF-8 boundaries.
///
/// **Not handled:** raw strings (`r#"…"#`), byte strings, or char literals — none
/// appear in today’s `tests/integration.rs` module list; extend if those surfaces
/// start carrying `#[path`-shaped text outside normal strings.
pub fn integration_rs_active_line_contains(integration_rs: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let bytes = integration_rs.as_bytes();
    let mut i = 0usize;
    let mut state = IntegrationRsScan::Code;

    while i < bytes.len() {
        match state {
            IntegrationRsScan::Code => {
                if bytes[i] == b'/' && i + 1 < bytes.len() {
                    if bytes[i + 1] == b'/' {
                        state = IntegrationRsScan::LineComment;
                        i += 2;
                        continue;
                    }
                    if bytes[i + 1] == b'*' {
                        state = IntegrationRsScan::BlockComment(1);
                        i += 2;
                        continue;
                    }
                }
                if bytes[i] == b'"' {
                    state = IntegrationRsScan::String;
                    i += 1;
                    continue;
                }
                if integration_rs[i..].starts_with(needle) {
                    return true;
                }
                i += 1;
            }
            IntegrationRsScan::LineComment => {
                if bytes[i] == b'\n' {
                    state = IntegrationRsScan::Code;
                }
                i += 1;
            }
            IntegrationRsScan::BlockComment(depth) => {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    state = IntegrationRsScan::BlockComment(depth + 1);
                    i += 2;
                    continue;
                }
                if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    let d = depth - 1;
                    i += 2;
                    state = if d == 0 {
                        IntegrationRsScan::Code
                    } else {
                        IntegrationRsScan::BlockComment(d)
                    };
                    continue;
                }
                i += 1;
            }
            IntegrationRsScan::String => match bytes[i] {
                b'\\' => {
                    i = (i + 2).min(bytes.len());
                }
                b'"' => {
                    i += 1;
                    state = IntegrationRsScan::Code;
                }
                _ => {
                    i += 1;
                }
            },
        }
    }
    false
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
        // Strip RUSTC_BOOTSTRAP before spawning rustc. The ratchet CI step sets
        // RUSTC_BOOTSTRAP=1 at the outer shell so libtest accepts its unstable
        // `--report-time` flag; without this removal the env var would leak to
        // every child rustc invocation, flipping them into bootstrap mode and
        // breaking the boundary-layer contract that these tests exercise the
        // **real** stable toolchain (TESTING.md § test layers).
        cmd.env_remove("RUSTC_BOOTSTRAP")
            .arg("--edition=2021")
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

// ────────────────────────────────────────────────────────────────────
// Shared cost-lookup helpers (DB-14 test-fixture invariant)
// ────────────────────────────────────────────────────────────────────
//
// Post-review-round 1b.5: the interpretation "for test fixtures, a
// named bind's cost must be `FoundCost` with a non-negative value"
// was duplicated across m1_3_lens_cost_test, thesis_validation_test,
// and m1_5_testgen_test — and m1_5 had already drifted to the weaker
// "only check MissingCost, don't validate sign" shape. These helpers
// are the single expression of the fixture-side invariant: all three
// test consumers route through them, and lens_testgen's own Option-
// returning variant in `src/v3/compiler/src/lens_testgen.rs:bind_cost_of`
// uses the same two panic paths (it diverges only in treating
// "bind not found" as `None` instead of panic, which is a legitimate
// API-shape difference, not an interpretation difference).
//
// Both failure modes are compiler invariant violations at the
// fixture boundary:
//   - `MissingCost` — the lens emits no cost for a named bind the
//     test explicitly constructed. Malformed fixture or lens regression.
//   - negative `FoundCost(c)` — the complexity algebra is non-negative
//     by construction; a negative value is an invariant violation
//     upstream of the test.
//
// `context` is a caller-provided string used in panic messages so
// the test failure names which bind/port/fixture tripped the assert.

use v3_compiler::lens_cost::CostLookup;

/// Extract a non-negative `i64` cost from a `CostLookup`, panicking
/// with `context` on either invariant violation. Use when the caller
/// wants an `i64` (e.g. passing to a comparison operator that takes
/// `i64 × i64 → bool`).
pub fn require_fixture_cost_i64(lookup: CostLookup, context: &str) -> i64 {
    match lookup {
        CostLookup::FoundCost { _0: cost } => {
            assert!(
                cost >= 0,
                "complexity lens emitted negative cost `{cost}` for {context} — \
                 the complexity algebra is non-negative by construction; a negative \
                 value is a compiler invariant violation upstream of this test."
            );
            cost
        }
        CostLookup::MissingCost => {
            panic!(
                "complexity lens returned MissingCost for {context} — malformed \
                 fixture or lens regression (the named bind exists in the Dag but \
                 has no cost entry)."
            );
        }
    }
}

/// `usize` variant of `require_fixture_cost_i64`. Use when the
/// caller indexes or compares as `usize` (the majority of cost-lens
/// consumers). The sign check is redundant with `usize::try_from`
/// but surfaces the "negative cost" diagnostic before the conversion
/// rather than via a generic `TryFromIntError`.
pub fn require_fixture_cost_usize(lookup: CostLookup, context: &str) -> usize {
    let cost = require_fixture_cost_i64(lookup, context);
    usize::try_from(cost).unwrap_or_else(|_| {
        // Unreachable after the i64 helper's sign assert, but keeps
        // the conversion-failure path explicit rather than silently
        // wrapping — single-authority claim holds if nobody reads it.
        panic!("complexity lens emitted cost that does not fit in usize for {context}: {cost}")
    })
}

// ────────────────────────────────────────────────────────────────────
// Per-test wall-clock budget macro (Layer 2 ratchet)
// ────────────────────────────────────────────────────────────────────
//
// `#[macro_export]` places this at the integration-test crate root so
// any `tests/*.rs` that declares `mod common;` can invoke
// `budgeted_test! { ... }` unqualified.

/// Per-test wall-clock budget (default 3s) for integration tests.
///
/// **Requires** `mod common;` at the top of the same `tests/*.rs` file: each
/// integration test binary is its own crate, and the macro expands to
/// `$crate::common::budgeted::with_budget_ms` / `DEFAULT_BUDGET_MS`, which only
/// exist when `common` is linked (`budgeted` is a public submodule).
///
/// Forms:
/// - `budgeted_test! { name, { ... } }` — default 3000 ms.
/// - `budgeted_test! { 5000, name, { ... } }` — custom budget in ms.
#[macro_export]
macro_rules! budgeted_test {
    ($ms:literal, $name:ident, $body:block) => {
        #[test]
        fn $name() {
            $crate::common::budgeted::with_budget_ms($ms, || $body);
        }
    };
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            $crate::common::budgeted::with_budget_ms(
                $crate::common::budgeted::DEFAULT_BUDGET_MS,
                || $body,
            );
        }
    };
}
