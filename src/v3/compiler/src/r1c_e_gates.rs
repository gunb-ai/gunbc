//! R1C-E — emit-gate check functions shared by the host `#[test]` harness and
//! the `r1c_e_emit_gates` `bin` (the `ExecuteCommand` logical child for the
//! T-Emit `.dag` `TestClaim` wrappers; the `.dag` source is spliced into a
//! `Dag` at integration-test compile time via `env!("CARGO_BIN_EXE_…")` —
//! see the integration-test driver for the on-disk path of the template).
//!
//! Each `check_*` returns `Ok(())` when the gate holds, or `Err(String)` with a
//! human-readable failure detail. The `bin` maps `Ok` → exit 0 / `Err` → exit 1
//! (no stdout/stderr capture by `ExecuteCommand` — exit code is the receipt).
//! `#[test]` callers panic with the detail to preserve the original failure
//! message.
//!
//! **Single source of truth.** The `#[test]` harness and the `bin` both call
//! these functions; do not duplicate the assertion bodies into either caller.
//!
//! **Public surface (R1 close scaffold).** The module is `pub` only so the
//! single bin in this crate can call it (Cargo bins compile against the public
//! lib API). Downstream crates must not depend on `r1c_e_gates::*`; this is a
//! scaffold that dissolves at R1 close together with the wrappers themselves.

use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use crate::compile_to_dag;
use crate::emit::emit;
use crate::emit::EmitTarget;
use crate::emit_rust::{emit_rust, emit_rust_module};
use crate::emit_rust_roundtrip_fixtures::{
    ProgramFixture, ReflectedExpected, GO_EMIT_EXCLUDE, PROGRAM_FIXTURES, PYTHON_EMIT_EXCLUDE,
    REFLECTED_FIXTURES,
};

/// `emit_generic_bounds_survive` (host receipt: `m1_3_emit_rust_test::emit_generic_bounds_survive`,
/// PR #650 post-mortem).
///
/// Pins the **Rust type line** for callable parameters: `impl Fn(...) -> ... + Clone`,
/// not `&impl Fn`. Body avoids higher-order `f(...)` calls — those are a separate
/// emit seam; this receipt only pins the parameter type spelling.
pub fn check_generic_bounds_survive() -> Result<(), String> {
    let src = "fn twice(f: fn(Int) -> Int) -> Int = 0\n";
    let dag = compile_to_dag(src, "r1c_e_generic_bounds.v3")
        .map_err(|e| format!("compile failed: {e:?}"))?;
    let out = emit_rust_module(&dag).map_err(|e| format!("emit failed: {e:?}"))?;

    let sig = "fn twice(p0: impl Fn(i64) -> i64 + Clone) -> i64";
    if !out.contains(sig) {
        return Err(format!(
            "callable param should carry synthesized + Clone (downstream rustc / stage0 contract); got:\n{out}"
        ));
    }
    if out.contains("&impl Fn") {
        return Err(format!(
            "borrowed callable param type must not be spelled as &impl Fn; got:\n{out}"
        ));
    }
    Ok(())
}

// === emit_rust_fixtures_rustc_green (batched rustc program + reflected harness) ===

/// Directory containing `libv3_compiler-*.rlib` for the current build. Test
/// executables sit in `…/target/debug/deps/`; the `r1c_e_emit_gates` bin is in
/// `…/target/debug/` and links against the same `dependency=…/deps` layout.
fn rustc_deps_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("current exe");
    let parent = exe.parent().expect("parent of current exe");
    if parent.file_name() == Some(std::ffi::OsStr::new("deps")) {
        parent.to_path_buf()
    } else {
        parent.join("deps")
    }
}

/// Resolves `lib{crate}-*.rlib` in the deps directory for the running executable.
///
/// Picks the file with the newest `modified()` time; on equal mtimes, uses
/// [`Path`] order so the choice is deterministic (avoids flake from `read_dir`
/// order). **Assumes** a single Cargo build is writing this `target` tree —
/// concurrent `cargo` invocations racing the same `deps/` dir are unsupported
/// and may select the wrong artifact.
fn find_current_rlib(crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let deps = rustc_deps_dir();
    let mut matches: Vec<PathBuf> = std::fs::read_dir(&deps)
        .unwrap_or_else(|e| panic!("read {}: {e}", deps.display()))
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
    fn mtime(path: &Path) -> Option<std::time::SystemTime> {
        std::fs::metadata(path).and_then(|m| m.modified()).ok()
    }
    matches.sort_by(|a, b| {
        mtime(a)
            .cmp(&mtime(b))
            .then_with(|| a.as_os_str().cmp(b.as_os_str()))
    });
    matches.pop().unwrap_or_else(|| {
        panic!(
            "no `{prefix}*.rlib` in {} (build `v3-compiler` for this target first)",
            deps.display()
        )
    })
}

/// How the rustc harness should link: standalone programs vs `v3_compiler` rlib
/// (reflected fixtures import `v3_compiler::dag` at runtime).
enum HarnessLinkMode {
    Standalone,
    WithV3Compiler,
}

struct R1cERustcHarness {
    scratch_dir: PathBuf,
    child_index: AtomicUsize,
}

impl R1cERustcHarness {
    fn new(scope: &str) -> Self {
        let pid = std::process::id();
        let scratch_dir = std::env::temp_dir().join(format!("r1c_e_{scope}_{pid}"));
        Self {
            scratch_dir,
            child_index: AtomicUsize::new(0),
        }
    }

    fn next_child_dir(&self) -> PathBuf {
        let id = self.child_index.fetch_add(1, Ordering::Relaxed);
        let path = self.scratch_dir.join(format!("c{id}"));
        std::fs::create_dir_all(&path).expect("create harness child dir");
        path
    }

    fn compile(&self, rust_source: &str, bin_name: &str, mode: HarnessLinkMode) -> PathBuf {
        let tmp_dir = self.next_child_dir();
        let src_path = tmp_dir.join("main.rs");
        let bin_path = tmp_dir.join(bin_name);
        std::fs::File::create(&src_path)
            .and_then(|mut f| f.write_all(rust_source.as_bytes()))
            .expect("write harness source");

        let mut cmd = Command::new("rustc");
        cmd.env_remove("RUSTC_BOOTSTRAP")
            .arg("--edition=2021")
            .arg(&src_path)
            .arg("-o")
            .arg(&bin_path);

        if let HarnessLinkMode::WithV3Compiler = mode {
            let deps = rustc_deps_dir();
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
            .expect("invoke rustc — install a rust toolchain to run this gate");
        assert!(status.success(), "rustc failed on harness source");
        bin_path
    }

    fn run(bin: &Path, args: &[&str]) -> String {
        let output = Command::new(bin)
            .args(args)
            .output()
            .expect("run compiled harness");
        if !output.status.success() {
            panic!(
                "compiled harness failed for args {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}

static EMIT_RUST_HARNESS: OnceLock<R1cERustcHarness> = OnceLock::new();
static PROGRAM_HARNESS_BIN: OnceLock<PathBuf> = OnceLock::new();
static REFLECTED_HARNESS_BIN: OnceLock<PathBuf> = OnceLock::new();

fn emit_rust_harness() -> &'static R1cERustcHarness {
    EMIT_RUST_HARNESS.get_or_init(|| R1cERustcHarness::new("emit_rust_r1c_e"))
}

fn build_program_harness() -> PathBuf {
    let mut body = String::new();
    for fixture in PROGRAM_FIXTURES {
        let emitted = emit_rust(
            &compile_to_dag(fixture.source, "program_fixture.v3").expect("compiles to dag"),
        )
        .expect("emit_rust");
        let emitted_pub_main = emitted.replace("fn main()", "pub fn main()");
        body.push_str(&format!(
            "#[allow(warnings, clippy::all)] pub mod {name} {{ {emitted} }}\n",
            name = fixture.name,
            emitted = emitted_pub_main,
        ));
    }
    body.push_str(
        "fn main() { \
           let name = std::env::args().nth(1).expect(\"program fixture name\"); \
           match name.as_str() { \
        ",
    );
    for fixture in PROGRAM_FIXTURES {
        body.push_str(&format!("\"{0}\" => {0}::main(), ", fixture.name));
    }
    body.push_str(
        "other => panic!(\"unknown program fixture: {other}\"), \
         } \
         }\n",
    );
    emit_rust_harness().compile(&body, "main_bin", HarnessLinkMode::Standalone)
}

fn program_harness_bin() -> &'static Path {
    PROGRAM_HARNESS_BIN
        .get_or_init(build_program_harness)
        .as_path()
}

/// Run the batched program fixture harness (same binary as `m1_3_emit_rust_test` host path).
pub fn run_rust_program_fixture(name: &str) -> String {
    R1cERustcHarness::run(program_harness_bin(), &[name])
}

fn build_reflected_harness() -> PathBuf {
    let mut body = String::new();
    for fixture in REFLECTED_FIXTURES {
        let module = emit_rust_module(
            &compile_to_dag(fixture.module.source, "reflected_fixture.v3").expect("compiles"),
        )
        .expect("emits");
        body.push_str(&format!(
            "#[allow(warnings, clippy::all)] \
             pub mod {name} {{ \
               use v3_compiler::dag::*; \
               use v3_compiler::diagnostics::*; \
               {module} \
               pub fn run() -> i64 {{ {wrapper} }} \
             }}\n",
            name = fixture.module.name,
            wrapper = fixture.wrapper_body,
        ));
    }
    body.push_str(
        "fn main() { \
           let name = std::env::args().nth(1).expect(\"test name arg\"); \
           let value: i64 = match name.as_str() { \
        ",
    );
    for fixture in REFLECTED_FIXTURES {
        body.push_str(&format!("\"{0}\" => {0}::run(), ", fixture.module.name));
    }
    body.push_str(
        "other => panic!(\"unknown reflected harness test: {other}\"), \
         }; \
         println!(\"{value}\"); \
         }\n",
    );
    emit_rust_harness().compile(&body, "reflected_bin", HarnessLinkMode::WithV3Compiler)
}

fn reflected_harness_bin() -> &'static Path {
    REFLECTED_HARNESS_BIN
        .get_or_init(build_reflected_harness)
        .as_path()
}

/// Run the batched reflected-module fixture harness.
pub fn run_rust_reflected_fixture(name: &str) -> String {
    R1cERustcHarness::run(reflected_harness_bin(), &[name])
}

/// `emit_rust_fixtures_rustc_green` — full matrix: all program + reflected
/// roundtrip expectations.
pub fn check_emit_rust_fixtures_rustc_green() -> Result<(), String> {
    let mut failures: Vec<String> = Vec::new();
    for fixture in PROGRAM_FIXTURES {
        let stdout = run_rust_program_fixture(fixture.name);
        if stdout != fixture.expected_stdout {
            failures.push(format!(
                "program {:?}: expected {:?}, got {stdout:?}",
                fixture.name, fixture.expected_stdout,
            ));
        }
    }
    for fixture in REFLECTED_FIXTURES {
        let stdout = run_rust_reflected_fixture(fixture.module.name);
        let (ok, label) = match &fixture.expected_stdout {
            ReflectedExpected::Exact(expected) => (stdout == *expected, format!("{expected:?}")),
            ReflectedExpected::PositiveInt => (
                stdout.parse::<i64>().is_ok_and(|n| n > 0),
                "positive integer".to_owned(),
            ),
        };
        if !ok {
            failures.push(format!(
                "reflected {:?}: expected {label}, got {stdout:?}",
                fixture.module.name,
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} fixture(s) failed:\n{}",
            failures.len(),
            failures.join("\n")
        ))
    }
}

// === emit_omni_demo_fixtures_green (multi-target) ===

struct OmniTmpDir(PathBuf);

impl OmniTmpDir {
    fn new(tag: u64) -> Self {
        let path = std::env::temp_dir().join(format!("v3_r1c_e_omni_{tag}_{}", std::process::id()));
        std::fs::create_dir_all(&path).expect("omni tmp dir");
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for OmniTmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Monotonic tag for each `OmniTmpDir` (`v3_r1c_e_omni_{tag}_{pid}`). Rust, Go, and
/// Python paths each `fetch_add` so concurrent scratch dirs never share one `tag`.
static OMNI_TMP_TAG: AtomicUsize = AtomicUsize::new(0);

fn omni_fixtures() -> Vec<&'static ProgramFixture> {
    PROGRAM_FIXTURES
        .iter()
        .filter(|f| !GO_EMIT_EXCLUDE.contains(&f.name))
        .filter(|f| !PYTHON_EMIT_EXCLUDE.contains(&f.name))
        .collect()
}

fn omni_rust_stdout(source: &str) -> Result<String, String> {
    let id = OMNI_TMP_TAG.fetch_add(1, Ordering::Relaxed) as u64;
    let dag =
        compile_to_dag(source, "omni_parity_r1c_e.v3").map_err(|e| format!("compile: {e:?}"))?;
    let rendered = emit_rust(&dag).map_err(|e| format!("Rust emit: {e:?}"))?;
    let tmp = OmniTmpDir::new(id);
    let src_path = tmp.path().join("main.rs");
    let bin_path = tmp.path().join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .map_err(|e| format!("write rust: {e}"))?;
    let compile = Command::new("rustc")
        .env_remove("RUSTC_BOOTSTRAP")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("invoke rustc: {e}"))?;
    if !compile.success() {
        return Err(format!("rustc failed on emitted source:\n{rendered}"));
    }
    let run = Command::new(&bin_path)
        .output()
        .map_err(|e| format!("run binary: {e}"))?;
    if !run.status.success() {
        return Err("compiled rust binary exited non-zero".to_string());
    }
    Ok(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

fn omni_go_stdout(fixture_name: &str, source: &str) -> Result<String, String> {
    let dag = compile_to_dag(source, "omni_parity_r1c_e.v3")
        .map_err(|e| format!("compile `{fixture_name}`: {e:?}"))?;
    let rendered = emit(&dag, EmitTarget::Go)
        .map_err(|e| format!("Go emit `{fixture_name}`: {e:?}"))?
        .text;
    let id = OMNI_TMP_TAG.fetch_add(1, Ordering::Relaxed) as u64;
    let tmp = OmniTmpDir::new(id);
    let src_path = tmp.path().join("main.go");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .map_err(|e| format!("write go: {e}"))?;
    let run = Command::new("go")
        .arg("run")
        .arg(&src_path)
        .current_dir(tmp.path())
        .output()
        .map_err(|e| format!("go run `{fixture_name}`: {e}"))?;
    if !run.status.success() {
        return Err(format!(
            "go run failed for `{fixture_name}`:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

fn omni_python_stdout(fixture_name: &str, source: &str) -> Result<String, String> {
    let dag = compile_to_dag(source, "omni_parity_r1c_e.v3")
        .map_err(|e| format!("compile `{fixture_name}`: {e:?}"))?;
    let rendered = emit(&dag, EmitTarget::Python)
        .map_err(|e| format!("Python emit `{fixture_name}`: {e:?}"))?
        .text;
    let id = OMNI_TMP_TAG.fetch_add(1, Ordering::Relaxed) as u64;
    let tmp = OmniTmpDir::new(id);
    let src_path = tmp.path().join("main.py");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .map_err(|e| format!("write py: {e}"))?;
    let run = Command::new("python3")
        .arg(&src_path)
        .output()
        .map_err(|e| format!("python3 `{fixture_name}`: {e}"))?;
    if !run.status.success() {
        return Err(format!(
            "python3 failed for `{fixture_name}`:\nstderr:\n{}",
            String::from_utf8_lossy(&run.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

fn omni_toolchain_available(cmd: &str, probe_arg: &str) -> bool {
    Command::new(cmd)
        .arg(probe_arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|s| s.success())
}

/// `emit_omni_demo_fixtures_green` — all omni targets agree with Rust baselines
/// (host: `m1_5_emit_omni_demo_test::emit_omni_demo_fixtures_green`).
pub fn check_omni_demo_fixtures_green() -> Result<(), String> {
    if !omni_toolchain_available("go", "version") {
        return Err("go toolchain not found — this gate requires go on PATH".to_string());
    }
    if !omni_toolchain_available("python3", "--version") {
        return Err("python3 toolchain not found — this gate requires python3 on PATH".to_string());
    }
    let fixtures = omni_fixtures();
    if fixtures.is_empty() {
        return Err("omni fixture set is empty after excludes".to_string());
    }
    for fixture in fixtures {
        let rust = omni_rust_stdout(fixture.source)
            .map_err(|e| format!("omni rust `{}`: {e}", fixture.name))?;
        let go = omni_go_stdout(fixture.name, fixture.source)?;
        if go != rust {
            return Err(format!(
                "Go output diverged from Rust for `{}` (go={go:?} rust={rust:?})",
                fixture.name
            ));
        }
        let py = omni_python_stdout(fixture.name, fixture.source)?;
        if py != rust {
            return Err(format!(
                "Python output diverged from Rust for `{}` (py={py:?} rust={rust:?})",
                fixture.name
            ));
        }
    }
    Ok(())
}

/// R1C-E `[[bin]]` entry: emitted `r1c_e_emit_gates_generated.rs` calls this (PB-0 cycle-6
/// `EXPECTED_HAND_AUTHORED_NON_TEST` retirement).
pub fn run_emit_gates_host_binary() -> std::process::ExitCode {
    use std::io::Write as _;
    use std::process::ExitCode;

    fn usage_stderr() -> ! {
        let _ = writeln!(
            std::io::stderr(),
            "usage: r1c_e_emit_gates <subcommand>\n\
             subcommands: generic-bounds | rust-fixtures | omni-demo"
        );
        std::process::exit(2);
    }

    let mut args = std::env::args().skip(1);
    let sub = args.next().unwrap_or_else(|| usage_stderr());
    if args.next().is_some() {
        usage_stderr();
    }

    let result = match sub.as_str() {
        "generic-bounds" => check_generic_bounds_survive(),
        "rust-fixtures" => check_emit_rust_fixtures_rustc_green(),
        "omni-demo" => check_omni_demo_fixtures_green(),
        _ => usage_stderr(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(detail) => {
            let _ = writeln!(std::io::stderr(), "r1c_e_emit_gates {sub}: {detail}");
            ExitCode::FAILURE
        }
    }
}
