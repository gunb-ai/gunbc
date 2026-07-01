//! Route-A cargo-green witness: the faithful `--emit-fresh` assembled crate must
//! `cargo build` with 0 errors (debug + release). Receipt for roadmap `5-cargo-green`
//! (#5777/#5873); `regen_stage0 --verify` alone only proves byte-identical seed, not rustc-green.

fn regen_stage0_bin() -> std::path::PathBuf {
    let ws = crate::helpers::workspace_root();
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| ws.join("target"));
    let bin = target_dir.join("release/regen_stage0");
    if !bin.exists() {
        let build = std::process::Command::new("cargo")
            .current_dir(&ws)
            .args([
                "build",
                "-p",
                "v1-compiler",
                "--release",
                "--bin",
                "regen_stage0",
            ])
            .output()
            .expect("failed to build regen_stage0");
        assert!(
            build.status.success(),
            "regen_stage0 build failed:\n{}",
            String::from_utf8_lossy(&build.stderr)
        );
    }
    bin
}

fn emit_fresh_dir() -> std::path::PathBuf {
    let unique = format!(
        "route-a-emit-fresh-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    std::env::temp_dir().join(unique)
}

fn assemble_emit_fresh_crate(regen: &std::path::Path, fresh_dir: &std::path::Path) {
    let ws = crate::helpers::workspace_root();
    let output = std::process::Command::new(regen)
        .current_dir(&ws)
        .args(["--emit-fresh"])
        .arg(fresh_dir)
        .output()
        .expect("failed to run regen_stage0 --emit-fresh");
    assert!(
        output.status.success(),
        "regen_stage0 --emit-fresh failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cargo_build_in(dir: &std::path::Path, release: bool) {
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(dir).arg("build");
    if release {
        cmd.arg("--release");
    }
    let output = cmd.output().expect("failed to run cargo build");
    assert!(
        output.status.success(),
        "cargo build{} in {} failed:\n{}",
        if release { " --release" } else { "" },
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "Expensive: regen_stage0 --emit-fresh + cargo build debug+release (~3-5min)"]
fn emit_fresh_crate_cargo_builds_green() {
    let regen = regen_stage0_bin();
    let fresh = emit_fresh_dir();
    assemble_emit_fresh_crate(&regen, &fresh);
    cargo_build_in(&fresh, false);
    cargo_build_in(&fresh, true);
    let _ = std::fs::remove_dir_all(&fresh);
}
