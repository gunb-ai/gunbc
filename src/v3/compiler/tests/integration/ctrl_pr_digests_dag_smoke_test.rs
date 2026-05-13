//! **Layer:** integration
//!
//! Lexer + structural ratchet for Wave-1 catalog #8 `dsl/ctrl/pr_digests.dag`.
//!
//! `compile_to_dag` today parses the expression/program surface; authored
//! `module … { … service … }` carrier files (same shape as `dsl/ctrl/review_verdict.dag`
//! and `dsl/extdeps/github/*.dag`) are not lowered through that entrypoint. This
//! test still guards the file against lexer drift and documents the intended
//! service surface for ctrl catalog #8.

use std::path::PathBuf;

use v3_compiler::tokenize_for_test;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("expected src/v3/compiler -> workspace root")
        .to_path_buf()
}

#[test]
fn ctrl_pr_digests_dag_tokenizes_and_matches_expected_surface() {
    let path = workspace_root().join("dsl/ctrl/pr_digests.dag");
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    tokenize_for_test(&source, "dsl/ctrl/pr_digests.dag").unwrap_or_else(|diag| {
        panic!("dsl/ctrl/pr_digests.dag should tokenize cleanly: {diag:?}");
    });

    for needle in [
        "module ctrl.pr_digests",
        "import extdeps.github.pulls { PullRequest, PullReview }",
        "import std.errors { GitHubErrorShape }",
        "import std.types { Url }",
        "type MergeReadinessVerdict",
        "type AttachedUrl",
        "type AttachedUrlContainer",
        "type AttachedUrlTextContext",
        "type RestFallbackReason",
        "service ctrl.PrDigests",
        "operation ExtractAttachedUrls",
        "operation RenderPrSummaryLine",
        "operation JudgeMergeReadiness",
        "operation ClassifyRestFallback",
        "readonly",
    ] {
        assert!(
            source.contains(needle),
            "dsl/ctrl/pr_digests.dag must contain `{needle}` (structural contract drift?)"
        );
    }
}
