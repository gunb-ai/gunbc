//! v2-compiler `#[cfg(test)]` harness compile guard — closes the CI blind spot.
//!
//! `src/v2/stage0/src/compiler_tests.rs` is the stage0 self-test module (`cargo test -p
//! v2-compiler --lib`). CI's `ci_floor` job runs only ONE `v2-compiler-tests` invocation —
//! the `pipeline::dag_emit_from_resolved_matches_compile_sources_for_v4_slice` parity receipt
//! — so this harness is otherwise dormant. A standalone `#[test]` here would never be selected.
//!
//! **Always-runs without a new CI step.** These are `pub` helpers invoked by that always-on
//! parity test (same pattern as `r2_emit_add_named_test`). The guard runs
//! `cargo test -p v2-compiler --lib --no-run --release` so Rc call-site regressions in the
//! emitted `compiler_tests.rs` fail the parity receipt with zero `ci.yml` change. Release
//! profile matches the ci_floor_parity shared closure (debug would cold-compile serde under
//! sccache pressure and flake with EAGAIN on shared runners).

use crate::helpers::workspace_root;

fn cargo_binary() -> &'static str {
    if std::path::Path::new("/opt/cargo/bin/cargo").exists() {
        "/opt/cargo/bin/cargo"
    } else {
        "cargo"
    }
}

/// True when nested cargo failed from shared-runner sccache/EAGAIN pressure, not Rc regressions.
fn is_fleet_sccache_infra_failure(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}{stderr}");
    [
        "Resource temporarily unavailable",
        "failed to spawn helper thread",
        "failed to spawn coordinator thread",
        "failed to execute compile",
        "Failed to send data to or receive data from server",
        "Broken pipe",
    ]
    .iter()
    .any(|sig| combined.contains(sig))
}

fn run_v2_compiler_lib_test_compile(cold: bool) -> std::process::Output {
    let mut cmd = std::process::Command::new(cargo_binary());
    cmd.arg("test")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--lib")
        .arg("--no-run")
        .arg("--release")
        .arg("--quiet")
        .current_dir(workspace_root());
    if cold {
        // Nested --no-run compile inherits CI's RUSTC_WRAPPER=sccache; under fleet pressure
        // ring/cc-rs flakes (observed ci_floor_parity run 27581068975 / #4978).
        cmd.env_remove("RUSTC_WRAPPER");
        cmd.env("CARGO_BUILD_JOBS", "1");
    }
    cmd.output()
        .expect("failed to spawn cargo test -p v2-compiler --lib --no-run")
}

/// Compile the v2-compiler lib test harness (`compiler_tests` module). Invoked by the
/// always-on parity test so CI exercises the stage0 self-test surface.
pub fn assert_v2_compiler_lib_tests_compile() {
    let output = run_v2_compiler_lib_test_compile(false);
    if output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output = if is_fleet_sccache_infra_failure(&stdout, &stderr) {
        let retry = run_v2_compiler_lib_test_compile(true);
        if retry.status.success() {
            return;
        }
        retry
    } else {
        output
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    panic!(
        "v2-compiler lib test harness failed to compile (CI blind spot guard).\n\
         Fix Rc call-sites in compiler_tests_rust.dag and regen stage0.\n\
         --- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}
