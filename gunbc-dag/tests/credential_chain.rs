//! Regression tests for the canonical credential chain.
//!
//! Every credentialed tool graph must follow the pattern:
//!   resolve_auth → cloud_env → bind_secret → cloud_credential → execute(res:credential)
//!
//! These tests enforce:
//! (a) No legacy `credential_env` nodes in production tool graphs.
//! (b) All `ScopeContract` implementations produce non-empty, valid scopes.
//! (c) The canonical chain shape holds for each credentialed tool graph.

use gunbc_ir::transport::gist::GistScopeContract;
use gunbc_ir::transport::llm::LlmScopeContract;
use gunbc_ir::transport::review::ReviewScopeContract;
use gunbc_ir::transport::scope::ScopeContract;

// ---------------------------------------------------------------------------
// Helper: verify the canonical credential chain exists in a DAG
// ---------------------------------------------------------------------------

fn assert_canonical_chain<T: std::fmt::Debug>(dag: &gunbc_ir::Dag<T>, graph_name: &str) {
    let canonical_nodes = [
        "resolve_auth",
        "cloud_env",
        "bind_secret",
        "cloud_credential",
    ];

    for node_id in &canonical_nodes {
        assert!(
            dag.get_node(&(*node_id).into()).is_some(),
            "{graph_name}: missing canonical credential chain node '{node_id}'"
        );
    }

    // Assert no legacy credential_env node
    assert!(
        dag.get_node(&"credential_env".into()).is_none(),
        "{graph_name}: contains legacy 'credential_env' node — use canonical chain instead"
    );
}

fn assert_chain_edges<T: std::fmt::Debug>(dag: &gunbc_ir::Dag<T>, graph_name: &str) {
    // cloud_env → bind_secret (config)
    let has_cloud_env_to_bind = dag.edges.iter().any(|e| {
        e.from_node.0 == "cloud_env"
            && e.to_node.0 == "bind_secret"
            && e.from_port.0 == "config"
            && e.to_port.0 == "config"
    });
    assert!(
        has_cloud_env_to_bind,
        "{graph_name}: missing edge cloud_env.config → bind_secret.config"
    );

    // resolve_auth → bind_secret (service)
    let has_resolve_to_bind = dag.edges.iter().any(|e| {
        e.from_node.0 == "resolve_auth"
            && e.to_node.0 == "bind_secret"
            && e.from_port.0 == "service"
            && e.to_port.0 == "service"
    });
    assert!(
        has_resolve_to_bind,
        "{graph_name}: missing edge resolve_auth.service → bind_secret.service"
    );

    // bind_secret → cloud_credential (config)
    let has_bind_to_cred = dag.edges.iter().any(|e| {
        e.from_node.0 == "bind_secret"
            && e.to_node.0 == "cloud_credential"
            && e.from_port.0 == "config"
            && e.to_port.0 == "config"
    });
    assert!(
        has_bind_to_cred,
        "{graph_name}: missing edge bind_secret.config → cloud_credential.config"
    );
}

// ---------------------------------------------------------------------------
// (a) No credential_env nodes in production tool graphs
// ---------------------------------------------------------------------------

#[test]
fn no_legacy_credential_env_in_gist_graphs() {
    use gunbc_gist::{build_gist_graph, GistMode};

    for mode in [
        GistMode::Snapshot,
        GistMode::Recent,
        GistMode::Diff {
            base_ref: "main".to_string(),
        },
    ] {
        let label = format!("gist({:?})", mode);
        let dag = build_gist_graph(mode, vec![], false).expect("gist graph should build");
        assert!(
            dag.get_node(&"credential_env".into()).is_none(),
            "{label}: contains legacy credential_env node"
        );
    }
}

#[test]
fn no_legacy_credential_env_in_llm_graph() {
    let dag = gunbc_lib_llm_ops::graph::build_chat_completion_graph();
    assert!(
        dag.get_node(&"credential_env".into()).is_none(),
        "llm: contains legacy credential_env node"
    );
}

#[test]
fn no_legacy_credential_env_in_review_graphs() {
    let dag = gunbc_lib_review::graph::build_inline_review_graph().unwrap();
    assert!(
        dag.get_node(&"credential_env".into()).is_none(),
        "review-inline: contains legacy credential_env node"
    );

    let dag = gunbc_lib_review::graph::build_diff_review_graph().unwrap();
    assert!(
        dag.get_node(&"credential_env".into()).is_none(),
        "review-diff: contains legacy credential_env node"
    );
}

#[test]
fn no_legacy_credential_env_in_github_credential_graph() {
    let dag = gunbc_lib_cloud_ops::build_github_credential_graph().unwrap();
    assert!(
        dag.get_node(&"credential_env".into()).is_none(),
        "github-credential: contains legacy credential_env node"
    );
}

// ---------------------------------------------------------------------------
// (b) All ScopeContract implementations produce valid, non-empty scopes
// ---------------------------------------------------------------------------

#[test]
fn gist_scope_contract_is_valid() {
    let intent = GistScopeContract.credential_intent();
    assert!(
        intent.validate().is_ok(),
        "GistScopeContract should produce a valid credential intent: {:?}",
        intent.validate()
    );
    assert!(
        !intent.required_scopes.is_empty(),
        "GistScopeContract must declare at least one required scope"
    );
}

#[test]
fn llm_scope_contract_openai_is_valid() {
    let intent = LlmScopeContract::openai().credential_intent();
    assert!(
        intent.validate().is_ok(),
        "LlmScopeContract::openai() should produce a valid credential intent: {:?}",
        intent.validate()
    );
    assert!(
        !intent.required_scopes.is_empty(),
        "LlmScopeContract::openai() must declare at least one required scope"
    );
}

#[test]
fn llm_scope_contract_anthropic_is_valid() {
    let intent = LlmScopeContract::anthropic().credential_intent();
    assert!(
        intent.validate().is_ok(),
        "LlmScopeContract::anthropic() should produce a valid credential intent: {:?}",
        intent.validate()
    );
    assert!(
        !intent.required_scopes.is_empty(),
        "LlmScopeContract::anthropic() must declare at least one required scope"
    );
}

#[test]
fn review_scope_contract_is_valid() {
    let intent = ReviewScopeContract::new("openai").credential_intent();
    assert!(
        intent.validate().is_ok(),
        "ReviewScopeContract should produce a valid credential intent: {:?}",
        intent.validate()
    );
    assert!(
        intent.required_scopes.len() >= 2,
        "ReviewScopeContract should include both LLM and review scopes, got: {:?}",
        intent.required_scopes
    );
}

// ---------------------------------------------------------------------------
// (c) Canonical chain shape: resolve_auth → cloud_env → bind_secret →
//     cloud_credential → execute(res:credential)
// ---------------------------------------------------------------------------

#[test]
fn gist_has_canonical_credential_chain() {
    use gunbc_gist::{build_gist_graph, GistMode};
    use gunbc_ir::NodeBody;

    for mode in [
        GistMode::Snapshot,
        GistMode::Recent,
        GistMode::Diff {
            base_ref: "main".to_string(),
        },
    ] {
        let label = format!("gist({:?})", mode);
        let dag = build_gist_graph(mode, vec![], false).expect("gist graph should build");

        // Credential chain is now inside the gist_upload SubDag
        let gist_upload = dag
            .get_node(&"gist_upload".into())
            .unwrap_or_else(|| panic!("{label}: missing gist_upload SubDag node"));

        match &gist_upload.body {
            NodeBody::SubDag(inner_dag) => {
                assert_canonical_chain(inner_dag, &format!("{label}/gist_upload"));
                assert_chain_edges(inner_dag, &format!("{label}/gist_upload"));
            }
            _ => panic!("{label}: gist_upload is not a SubDag"),
        }
    }
}

#[test]
fn llm_has_canonical_credential_chain() {
    let dag = gunbc_lib_llm_ops::graph::build_chat_completion_graph();
    assert_canonical_chain(&dag, "llm");
    assert_chain_edges(&dag, "llm");
}

#[test]
fn review_inline_has_canonical_credential_chain() {
    let dag = gunbc_lib_review::graph::build_inline_review_graph().unwrap();
    assert_canonical_chain(&dag, "review-inline");
    assert_chain_edges(&dag, "review-inline");
}

#[test]
fn review_diff_has_canonical_credential_chain() {
    let dag = gunbc_lib_review::graph::build_diff_review_graph().unwrap();
    assert_canonical_chain(&dag, "review-diff");
    assert_chain_edges(&dag, "review-diff");
}

#[test]
fn github_credential_has_canonical_chain() {
    let dag = gunbc_lib_cloud_ops::build_github_credential_graph().unwrap();
    assert_canonical_chain(&dag, "github-credential");
    assert_chain_edges(&dag, "github-credential");
}
