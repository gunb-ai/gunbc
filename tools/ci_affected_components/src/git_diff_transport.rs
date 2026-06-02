//! Shared git diff transport for CI affected-set and Wave 3 shadow receipt emit.

use std::process::Command;

pub const WAVE3_LIVE_EVAL_DEBT: &str = "node://adhoc-331899f9-19a";

pub fn diff_range_for_event(event_name: &str) -> &'static str {
    if event_name == "pull_request" {
        "origin/main...HEAD"
    } else {
        "HEAD~1..HEAD"
    }
}

pub enum GitChangedPathsRead {
    Ok { range: String, paths: Vec<String> },
    /// `detail` carries the structured reason for fail-closed so the binary
    /// entrypoint can surface it (library crates do not print — see
    /// `clippy::disallowed_macros`).
    FailClosed { range: String, detail: String },
}

pub fn git_read_changed_paths_for_event(event_name: &str) -> GitChangedPathsRead {
    let range = diff_range_for_event(event_name).to_string();
    match Command::new("git")
        .args(["diff", "--name-only", &range])
        .output()
    {
        Ok(out) if out.status.success() => GitChangedPathsRead::Ok {
            range,
            paths: String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
        },
        Ok(out) => GitChangedPathsRead::FailClosed {
            detail: format!("git diff --name-only {range} exited {}", out.status),
            range,
        },
        Err(e) => GitChangedPathsRead::FailClosed {
            detail: format!("git diff --name-only {range} failed ({e})"),
            range,
        },
    }
}
