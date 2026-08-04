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
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    let commit = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    let tree = git_output(&["rev-parse", "HEAD^{tree}"]).unwrap_or_default();
    println!("cargo:rustc-env=FRONTIER_PROBE_SURVEY_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=FRONTIER_PROBE_SURVEY_BUILD_TREE={tree}");
}
