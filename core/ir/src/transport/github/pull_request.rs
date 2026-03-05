//! GitHub Pull Request operations via `gh` CLI.
//!
//! Provides request builders for PR creation, commenting, and merging
//! using the `gh pr` subcommand family.

use crate::transport::agent::{PullRequestResult, PullRequestSpec};
use crate::transport::{Hermeticity, ShellRequest};

use super::cli::GH_TOOL;

/// Build a `gh pr create` shell request from a `PullRequestSpec`.
pub fn build_pr_create_request(spec: &PullRequestSpec) -> ShellRequest {
    let mut args = vec![
        "pr".to_string(),
        "create".to_string(),
        "--repo".to_string(),
        format!("{}/{}", spec.owner, spec.repo),
        "--head".to_string(),
        spec.head_branch.clone(),
        "--base".to_string(),
        spec.base_branch.clone(),
        "--title".to_string(),
        spec.title.clone(),
        "--body".to_string(),
        spec.body.clone(),
    ];
    if spec.draft {
        args.push("--draft".to_string());
    }
    ShellRequest::new(GH_TOOL.command)
        .args(args)
        .with_semantics("github.pr.create", Hermeticity::External)
}

/// Build a `gh pr comment` shell request.
pub fn build_pr_comment_request(
    owner: &str,
    repo: &str,
    pr_number: u64,
    body: &str,
) -> ShellRequest {
    ShellRequest::new(GH_TOOL.command)
        .args([
            "pr",
            "comment",
            &pr_number.to_string(),
            "--repo",
            &format!("{owner}/{repo}"),
            "--body",
            body,
        ])
        .with_semantics("github.pr.comment", Hermeticity::External)
}

/// Build a `gh pr merge` shell request.
pub fn build_pr_merge_request(
    owner: &str,
    repo: &str,
    pr_number: u64,
    squash: bool,
) -> ShellRequest {
    let mut req = ShellRequest::new(GH_TOOL.command).args([
        "pr",
        "merge",
        &pr_number.to_string(),
        "--repo",
        &format!("{owner}/{repo}"),
        "--delete-branch",
    ]);
    if squash {
        req = req.arg("--squash");
    } else {
        req = req.arg("--merge");
    }
    req.with_semantics("github.pr.merge", Hermeticity::External)
}

/// Parse the stdout of `gh pr create` into a `PullRequestResult`.
///
/// `gh pr create` outputs the PR URL on success, e.g.:
/// `https://github.com/owner/repo/pull/42`
pub fn parse_pr_create_response(
    stdout: &str,
    head_branch: &str,
) -> Result<PullRequestResult, String> {
    let url = stdout.trim().to_string();
    if url.is_empty() {
        return Err("gh pr create returned empty output".to_string());
    }
    let number = url
        .rsplit('/')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| format!("cannot parse PR number from URL: {url}"))?;
    Ok(PullRequestResult {
        number,
        url,
        head_branch: head_branch.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_spec() -> PullRequestSpec {
        PullRequestSpec {
            owner: "test-org".into(),
            repo: "test-repo".into(),
            head_branch: "feature/my-feature".into(),
            base_branch: "main".into(),
            title: "Add my feature".into(),
            body: "Closes #42\n\nImplements the design.".into(),
            issue_number: 42,
            draft: false,
        }
    }

    #[test]
    fn pr_create_request_has_correct_args() {
        let req = build_pr_create_request(&test_spec());
        assert_eq!(req.command, "gh");
        assert!(req.args.contains(&"pr".to_string()));
        assert!(req.args.contains(&"create".to_string()));
        assert!(req.args.contains(&"--head".to_string()));
        assert!(req.args.contains(&"feature/my-feature".to_string()));
        assert!(req.args.contains(&"--base".to_string()));
        assert!(req.args.contains(&"main".to_string()));
        assert!(!req.args.contains(&"--draft".to_string()));
    }

    #[test]
    fn pr_create_request_includes_draft_flag() {
        let mut spec = test_spec();
        spec.draft = true;
        let req = build_pr_create_request(&spec);
        assert!(req.args.contains(&"--draft".to_string()));
    }

    #[test]
    fn pr_comment_request_has_correct_args() {
        let req = build_pr_comment_request("org", "repo", 42, "LGTM");
        assert_eq!(req.command, "gh");
        assert!(req.args.contains(&"comment".to_string()));
        assert!(req.args.contains(&"42".to_string()));
        assert!(req.args.contains(&"LGTM".to_string()));
    }

    #[test]
    fn pr_merge_squash_request() {
        let req = build_pr_merge_request("org", "repo", 42, true);
        assert!(req.args.contains(&"merge".to_string()));
        assert!(req.args.contains(&"--squash".to_string()));
        assert!(req.args.contains(&"--delete-branch".to_string()));
    }

    #[test]
    fn parse_pr_url_extracts_number() {
        let result = parse_pr_create_response(
            "https://github.com/test-org/test-repo/pull/42\n",
            "feature/test",
        )
        .expect("parse should succeed");
        assert_eq!(result.number, 42);
        assert_eq!(result.url, "https://github.com/test-org/test-repo/pull/42");
        assert_eq!(result.head_branch, "feature/test");
    }

    #[test]
    fn parse_pr_url_fails_on_empty() {
        let err = parse_pr_create_response("", "b").unwrap_err();
        assert!(err.contains("empty output"));
    }

    #[test]
    fn parse_pr_url_fails_on_bad_format() {
        let err = parse_pr_create_response("not-a-url", "b").unwrap_err();
        assert!(err.contains("cannot parse PR number"));
    }
}
