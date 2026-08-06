use std::path::{Path, PathBuf};
use std::process::Command;

fn git_output(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// Resolve the checkout's git metadata directory (follows worktree `gitdir:` files).
fn resolve_git_dir(mut dir: &Path) -> Option<PathBuf> {
    loop {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        }
        if dot_git.is_file() {
            let content = std::fs::read_to_string(&dot_git).ok()?;
            let gitdir = content.strip_prefix("gitdir:")?.trim();
            return Some(PathBuf::from(gitdir));
        }
        dir = dir.parent()?;
    }
}

fn rerun_if_changed(path: &Path) {
    if path.exists() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn main() {
    // Re-run when git HEAD moves; also when this script or the survey bin change so
    // BUILD_DIRTY is recomputed after local source edits (defense-in-depth — survey
    // time ensure_clean_tree and verify_build_provenance still gate the run).
    //
    // Paths must be absolute/explicit: `.git/HEAD` is package-relative and does not
    // exist (the checkout root is several ancestors up; worktrees use a `gitdir:` file).
    // A missing `rerun-if-changed` target makes every subsequent cargo invocation treat
    // the build script as stale, forcing a second v1-compiler compile on the CI gate.
    if let Some(git_dir) = resolve_git_dir(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        rerun_if_changed(&git_dir.join("HEAD"));
        rerun_if_changed(&git_dir.join("refs/heads"));
    }
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
