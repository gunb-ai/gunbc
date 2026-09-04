use std::path::Path;
use std::process::Command;

const MATERIALIZED_TREE_IDENTITY_ENV: &str = "GUNBC_MATERIALIZED_TREE_IDENTITY";

fn validate_materialized_tree_identity(identity: &str) -> bool {
    let hex = identity
        .strip_prefix("tree:sha1:")
        .filter(|hex| hex.len() == 40)
        .or_else(|| {
            identity
                .strip_prefix("tree:sha256:")
                .filter(|hex| hex.len() == 64)
        });
    hex.is_some_and(|hex| {
        hex.bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn git_output(args: &[&str]) -> Option<String> {
    // NO OPTIONAL LOCKS: `git status` refreshes `.git/index` as a side effect, and this script
    // watches that path (`rerun-if-changed` below). Refreshing it DURING the script's own run
    // made the NEXT build see a changed input, rerun the script and recompile the whole crate
    // with nothing changed -- measured 2026-08-30 on srv1 (tree fce29f50, quiescent worktree):
    // cold 444 s, then a no-change build 382 s, then 0 s only once the index had settled; and
    // inside every `--regen-round-cost` round, seed_build and rebuild-from-installed each
    // recompiled the crate at changed_paths=0. `--no-optional-locks` makes the read a read.
    let out = Command::new("git")
        .arg("--no-optional-locks")
        .args(args)
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn watch_git_path(path: &str) {
    if Path::new(path).exists() {
        println!("cargo:rerun-if-changed={path}");
    }
}

fn main() {
    // Re-run when the binary's Rust inputs change so a clean build cannot keep its
    // identity after those inputs become dirty. Watching the repository root would
    // include `target/` and make build output invalidate its own build script.
    //
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed={MATERIALIZED_TREE_IDENTITY_ENV}");

    // Ask Git for its real paths: a linked worktree's `.git` is a pointer file, and a branch's
    // HEAD file contains only a stable symbolic-ref name. Watch both the worktree HEAD and its
    // resolved loose ref so every commit invalidates the identity. When the ref is packed, watch
    // its parent directory as well: the next commit creates the loose ref there. `packed-refs`
    // covers repacks, and the index covers staged/unstaged transitions whose source bytes do not
    // change. Existing paths only are enrolled because Cargo treats a missing watched path as
    // perpetually stale.
    if let Some(head_path) = git_output(&["rev-parse", "--git-path", "HEAD"]) {
        watch_git_path(&head_path);
    }
    if let Some(head_ref) = git_output(&["symbolic-ref", "-q", "HEAD"]) {
        if let Some(ref_path) = git_output(&["rev-parse", "--git-path", &head_ref]) {
            if Path::new(&ref_path).exists() {
                watch_git_path(&ref_path);
            } else if let Some(parent) = Path::new(&ref_path).parent().and_then(Path::to_str) {
                watch_git_path(parent);
            }
        }
    }
    for git_name in ["packed-refs", "index"] {
        if let Some(path) = git_output(&["rev-parse", "--git-path", git_name]) {
            watch_git_path(&path);
        }
    }

    let identity = match std::env::var(MATERIALIZED_TREE_IDENTITY_ENV) {
        Ok(materialized) => {
            assert!(
                validate_materialized_tree_identity(&materialized),
                "gunbc build received an invalid materialized-tree identity; expected tree:sha1:<40 lowercase hex> or tree:sha256:<64 lowercase hex>"
            );
            materialized
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("gunbc build received a non-Unicode materialized-tree identity")
        }
        Err(std::env::VarError::NotPresent) => {
            let commit = git_output(&["rev-parse", "HEAD"]).expect(
                "gunbc build cannot observe its source commit: `git rev-parse HEAD` failed or Git is unavailable",
            );
            assert!(
                commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "gunbc build received a non-40-hex source commit from `git rev-parse HEAD`"
            );
            let dirty = git_output(&["status", "--porcelain"])
                .map(|s| !s.is_empty())
                .unwrap_or(true);
            if dirty {
                format!("{commit}-dirty")
            } else {
                commit
            }
        }
    };
    println!("cargo:rustc-env=GUNBC_BUILD_IDENTITY={identity}");
}
