//! Shared git diff transport for CI affected-set and Wave 3 shadow receipt emit.

use std::process::Command;

pub const WAVE3_LIVE_EVAL_DEBT: &str = "node://adhoc-331899f9-19a";

/// Diff range for the two supported GitHub event names. Returns `None` for any other event —
/// the event is a CLOSED boundary (`pull_request | push`), so an unknown/misspelled name must NOT
/// silently fall back to the push range and compute a plausible-but-wrong affected set (INVARIANTS
/// P3 — missing/unsupported input fails closed, never a fabricated default).
pub fn diff_range_for_event(event_name: &str) -> Option<&'static str> {
    match event_name {
        "pull_request" => Some("origin/main...HEAD"),
        "push" => Some("HEAD~1..HEAD"),
        _ => None,
    }
}

pub enum GitChangedPathsRead {
    Ok {
        range: String,
        paths: Vec<String>,
    },
    /// `detail` carries the structured reason for fail-closed so the binary
    /// entrypoint can surface it (library crates do not print — see
    /// `clippy::disallowed_macros`).
    FailClosed {
        range: String,
        detail: String,
    },
}

pub fn git_read_changed_paths_for_event(event_name: &str) -> GitChangedPathsRead {
    let range = match diff_range_for_event(event_name) {
        Some(r) => r.to_string(),
        // Unsupported event — fail closed (P3), do NOT guess a diff range.
        None => {
            return GitChangedPathsRead::FailClosed {
                range: format!("<unsupported event:{event_name}>"),
                detail: format!(
                    "unsupported event_name '{event_name}' (expected pull_request|push); fail-closed (all components affected)"
                ),
            };
        }
    };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_events_have_ranges() {
        assert_eq!(
            diff_range_for_event("pull_request"),
            Some("origin/main...HEAD")
        );
        assert_eq!(diff_range_for_event("push"), Some("HEAD~1..HEAD"));
    }

    #[test]
    fn unsupported_event_has_no_range_and_fails_closed() {
        // Closed boundary: unknown/misspelled events must not borrow the push range (P3).
        assert_eq!(diff_range_for_event("workflow_dispatch"), None);
        assert_eq!(diff_range_for_event("Pull_Request"), None);
        assert_eq!(diff_range_for_event(""), None);
        match git_read_changed_paths_for_event("workflow_dispatch") {
            GitChangedPathsRead::FailClosed { detail, .. } => {
                assert!(detail.contains("unsupported event_name 'workflow_dispatch'"));
            }
            GitChangedPathsRead::Ok { .. } => {
                panic!("unsupported event must fail closed, not read a guessed diff range")
            }
        }
    }
}
