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
    // Re-run when the binary's Rust inputs change so a clean build cannot keep its
    // identity after those inputs become dirty. Watching the repository root would
    // include `target/` and make build output invalidate its own build script.
    //
    // Do NOT declare cargo:rerun-if-changed on .git/HEAD here. Package-relative paths
    // like `.git/HEAD` do not exist (checkout .git lives at workspace root; worktrees
    // use a gitdir: pointer file). A missing rerun-if-changed target makes every
    // subsequent cargo invocation treat the build script as stale, forcing a second
    // v1-compiler compile when alternating cargo build and cargo test. Pointing at
    // the real HEAD path would be worse: every commit would invalidate the shared lib.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let commit = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    assert!(
        commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "gunbc build requires an exact 40-hex source commit from `git rev-parse HEAD`"
    );
    let dirty = git_output(&["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(true);
    let identity = if dirty {
        format!("{commit}-dirty")
    } else {
        commit
    };
    println!("cargo:rustc-env=GUNBC_BUILD_IDENTITY={identity}");
}
