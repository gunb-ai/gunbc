//! Shared `git diff --name-only` host transport for the affected-set CI bins.
//!
//! Both `detect-ci-affected-components` and `emit-affected-set-ci-receipt` read the same PR diff;
//! this module is the single source for the diff range and the fail-closed read so the two bins
//! can never drift on what "changed paths" means.

use std::process::Command;

/// Outcome of reading the PR diff. `FailClosed` selects the fail-closed superset (all components).
pub enum GitDiffRead {
    Ok(Vec<String>),
    FailClosed,
}

/// Diff range for a GitHub Actions event: PRs diff against `origin/main`, pushes against the prior
/// commit.
pub fn diff_range(event_name: &str) -> &'static str {
    if event_name == "pull_request" {
        "origin/main...HEAD"
    } else {
        "HEAD~1..HEAD"
    }
}

/// Run `git diff --name-only <range>`; fail closed (no paths, caller selects superset) on any error.
pub fn read_changed_paths(range: &str) -> GitDiffRead {
    let output = Command::new("git")
        .args(["diff", "--name-only", range])
        .output();
    match output {
        Ok(out) if out.status.success() => GitDiffRead::Ok(
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
        ),
        Ok(out) => {
            eprintln!(
                "error: git diff --name-only {range} exited {}; fail-closed (all components affected)",
                out.status
            );
            GitDiffRead::FailClosed
        }
        Err(e) => {
            eprintln!("error: git diff failed ({e}); fail-closed (all components affected)");
            GitDiffRead::FailClosed
        }
    }
}
