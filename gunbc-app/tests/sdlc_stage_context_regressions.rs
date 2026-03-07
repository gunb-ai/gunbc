use std::fs;

const SDLC_STAGES_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../dsl/funcs/sdlc_stages.dag"
);

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn sdlc_stages_source() -> String {
    fs::read_to_string(SDLC_STAGES_PATH)
        .unwrap_or_else(|err| panic!("read `{SDLC_STAGES_PATH}`: {err}"))
}

#[test]
fn idea_stage_posts_generated_design_content() {
    let source = sdlc_stages_source();
    assert!(
        source.contains("design.content"),
        "idea stage comment should reference the generated design content"
    );
    assert!(
        source.contains("design.summary"),
        "idea stage comment should reference the design summary"
    );
    assert!(
        source.contains("\"design_content\": design.content"),
        "idea stage should persist generated design content in the stage payload"
    );
}

#[test]
fn design_review_stage_reuses_prior_design_payload() {
    let source = sdlc_stages_source();
    assert!(
        source.contains("prior_design = outcomes.get("),
        "design review stage should load the prior idea-stage outcome"
    );
    assert!(
        source.contains("design_body = prior_design.outcome.payload.design_content"),
        "design review stage should read generated design content from the prior outcome payload"
    );
    assert!(
        source.contains("design: design_body"),
        "design review LLM call should review the stored design artifact, not the issue body"
    );
}

#[test]
fn implementation_and_code_review_stages_thread_payload_context() {
    let source = sdlc_stages_source();
    assert!(
        source.contains("\"agent_run_id\": agent.run_id"),
        "accepted->implementing should persist the spawned agent run id"
    );
    assert!(
        source.contains("\"branch\": branch"),
        "accepted->implementing should persist the agent branch name"
    );
    assert!(
        source.contains("branch = prior_outcome.outcome.payload.branch"),
        "implementing->code-review should recover the branch from the prior outcome payload"
    );
    assert!(
        source.contains("\"pr_number\": pr.number"),
        "implementing->code-review should persist the created PR number"
    );
    assert!(
        source.contains("pr_number = implementation_outcome.outcome.payload.pr_number"),
        "code review should recover the PR number from the implementation outcome payload"
    );
    assert!(
        source.matches("number: pr_number").count() >= 3,
        "code review should use the recovered PR number for PR fetch, file list, and PR comment calls"
    );
}
