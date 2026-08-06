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
    // Re-run when git HEAD moves; also when this script or the survey bin change so
    // BUILD_DIRTY is recomputed after local source edits (defense-in-depth — survey
    // time ensure_clean_tree and verify_build_provenance still gate the run).
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bin/frontier_probe_survey.rs");

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
