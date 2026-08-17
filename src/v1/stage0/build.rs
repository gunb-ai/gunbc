use std::process::Command;

fn git_output(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn main() {
    // Re-run when this script or the survey bin change so BUILD_DIRTY is recomputed
    // after local source edits (defense-in-depth — survey time ensure_clean_tree and
    // verify_build_provenance still gate the run).
    //
    // Do NOT declare cargo:rerun-if-changed on .git/HEAD here. Package-relative paths
    // like `.git/HEAD` do not exist (checkout .git lives at workspace root; worktrees
    // use a gitdir: pointer file). A missing rerun-if-changed target makes every
    // subsequent cargo invocation treat the build script as stale, forcing a second
    // v1-compiler compile when alternating cargo build and cargo test. Pointing at
    // the real HEAD path would be worse: every commit would invalidate the shared lib.
    println!("cargo:rerun-if-changed=build.rs");

    let commit = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    let tree = git_output(&["rev-parse", "HEAD^{tree}"]).unwrap_or_default();
    let dirty = git_output(&["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(true);
    println!("cargo:rustc-env=FRONTIER_PROBE_SURVEY_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=FRONTIER_PROBE_SURVEY_BUILD_TREE={tree}");
    println!(
        "cargo:rustc-env=FRONTIER_PROBE_SURVEY_BUILD_DIRTY={}",
        if dirty { "1" } else { "0" }
    );
}
