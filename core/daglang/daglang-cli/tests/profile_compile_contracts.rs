// Test infrastructure: filesystem/process access for CLI contract tests.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use gunbc_ir::WorkspaceLayout;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

fn workspace_root() -> PathBuf {
    static WORKSPACE_ROOT: OnceLock<PathBuf> = OnceLock::new();
    WORKSPACE_ROOT
        .get_or_init(|| {
            WorkspaceLayout::from_env_manifest_dir()
                .expect("resolve workspace layout")
                .workspace_root
        })
        .clone()
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

#[test]
fn compile_sdlc_worker_requires_profile_and_resolves_all_core_bindings() {
    let worker_path = "dsl/funcs/sdlc_worker.dag";

    let missing_profile = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["compile", worker_path])
        .output()
        .expect("invoke daglang compile without profile");
    assert!(
        !missing_profile.status.success(),
        "compile without profile should fail for interface-bound worker"
    );
    let missing_profile_stderr = String::from_utf8_lossy(&missing_profile.stderr);
    assert!(
        missing_profile_stderr.contains("compile with --profile <name>"),
        "missing-profile compile should report actionable profile error: {missing_profile_stderr}"
    );

    let unit_test = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["compile", "--profile", "unit_test", worker_path])
        .output()
        .expect("invoke daglang compile with unit_test profile");
    assert!(
        unit_test.status.success(),
        "unit_test profile should compile worker module: {}",
        String::from_utf8_lossy(&unit_test.stderr)
    );

    let local_missing_env = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["compile", "--profile", "local", worker_path])
        .env_remove("GITHUB_TOKEN")
        .env_remove("CODEX_API_KEY")
        .output()
        .expect("invoke daglang compile with local profile and no env");
    assert!(
        !local_missing_env.status.success(),
        "local profile should fail closed when required env vars are missing"
    );
    let local_missing_env_stderr = String::from_utf8_lossy(&local_missing_env.stderr);
    assert!(
        local_missing_env_stderr.contains("requires config `credential` from env var"),
        "missing local env vars should be reported explicitly: {local_missing_env_stderr}"
    );

    let local_with_env = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["compile", "--profile", "local", worker_path])
        .env("GITHUB_TOKEN", "fixture-github-token")
        .env("CODEX_API_KEY", "fixture-codex-key")
        .output()
        .expect("invoke daglang compile with local profile and env");
    assert!(
        local_with_env.status.success(),
        "local profile should compile worker module once env config is provided: {}",
        String::from_utf8_lossy(&local_with_env.stderr)
    );

    let cloud_run = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["compile", "--profile", "cloud_run", worker_path])
        .output()
        .expect("invoke daglang compile with cloud_run profile");
    assert!(
        cloud_run.status.success(),
        "cloud_run profile should compile worker module: {}",
        String::from_utf8_lossy(&cloud_run.stderr)
    );
}
