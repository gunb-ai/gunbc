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
    FailClosed { range: String },
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
        Ok(out) => {
            eprintln!(
                "error: git diff --name-only {range} exited {}; fail-closed",
                out.status
            );
            GitChangedPathsRead::FailClosed { range }
        }
        Err(e) => {
            eprintln!("error: git diff failed ({e}); fail-closed");
            GitChangedPathsRead::FailClosed { range }
        }
    }
}
