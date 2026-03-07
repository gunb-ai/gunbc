use gunbc_dag::dsl_builder::{build_dsl_graph, build_dsl_graph_with_profile};
use gunbc_dag::sdlc_workflow_spec;
use gunbc_exec::{
    execute_with_mode_and_inputs, lower, BoundaryMocks, ExecutionMode,
};
use gunbc_ir::{detect_entrypoints, Dag, Value};
use gunbc_test::auto_mock_spec;
use std::collections::{HashSet, VecDeque};

fn has_node_with_prefix<'a, I>(mut node_ids: I, prefix: &str) -> bool
where
    I: Iterator<Item = &'a String>,
{
    node_ids.any(|id| id.starts_with(prefix))
}

fn connected_subdag<T: Clone>(dag: &Dag<T>, seed_prefix: &str) -> Dag<T> {
    let seed = dag
        .nodes
        .iter()
        .find(|node| node.id.0.starts_with(seed_prefix))
        .map(|node| node.id.0.clone())
        .unwrap_or_else(|| panic!("seed node prefix `{seed_prefix}` not found in DAG"));

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    visited.insert(seed.clone());
    queue.push_back(seed);

    while let Some(current) = queue.pop_front() {
        for edge in &dag.edges {
            let maybe_neighbor = if edge.from_node.0 == current {
                Some(edge.to_node.0.clone())
            } else if edge.to_node.0 == current {
                Some(edge.from_node.0.clone())
            } else {
                None
            };
            if let Some(neighbor) = maybe_neighbor {
                if visited.insert(neighbor.clone()) {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    let mut subdag = dag.clone();
    subdag.nodes.retain(|node| visited.contains(&node.id.0));
    subdag.edges.retain(|edge| {
        visited.contains(&edge.from_node.0) && visited.contains(&edge.to_node.0)
    });
    subdag
}

#[test]
fn builds_sdlc_worker_dsl_graph() {
    let dag = build_dsl_graph("funcs/sdlc_worker.dag")
        .expect("sdlc worker DSL graph should resolve");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn builds_sdlc_stages_dsl_graph() {
    let dag = build_dsl_graph("funcs/sdlc_stages.dag")
        .expect("sdlc stages DSL graph should resolve");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn builds_sdlc_workflow_dsl_graph() {
    let dag = build_dsl_graph("workflows/sdlc.dag")
        .expect("sdlc workflow DSL graph should resolve");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn builds_sdlc_worker_unit_test_profile_dsl_graph() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "profiles.sdlc.unit_test")
        .expect("sdlc worker unit_test profile graph should resolve");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn builds_sdlc_worker_local_profile_dsl_graph() {
    if std::env::var("GITHUB_TOKEN").is_err() || std::env::var("CODEX_API_KEY").is_err() {
        eprintln!(
            "skipping local profile compile test (requires GITHUB_TOKEN and CODEX_API_KEY)"
        );
        return;
    }
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "profiles.sdlc.local")
        .expect("sdlc worker local profile graph should resolve");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn builds_sdlc_worker_cloud_run_profile_dsl_graph() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "profiles.sdlc.cloud_run")
        .expect("sdlc worker cloud_run profile graph should resolve");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn sdlc_pipeline_unit_test_profile_resolves_stub_provider_nodes() {
    let dag = build_dsl_graph_with_profile("pipelines/sdlc.dag", "profiles.sdlc.unit_test")
        .expect("sdlc pipeline unit_test profile graph should resolve");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn sdlc_worker_unit_test_profile_binds_all_interfaces_to_stub_implementations() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "profiles.sdlc.unit_test")
        .expect("sdlc worker unit_test profile graph should resolve");
    let node_ids: Vec<String> = dag.nodes.iter().map(|node| node.id.0.clone()).collect();

    let expected_stub_execute_prefixes = [
        (
            "IssueProvider",
            "execute_transport_profiles_unit_test_StubIssueProvider_",
        ),
        (
            "ClaimStore",
            "execute_transport_profiles_unit_test_InMemoryClaimStore_",
        ),
        (
            "OutcomeLedger",
            "execute_transport_profiles_unit_test_InMemoryOutcomeLedger_",
        ),
        (
            "AgentProvider",
            "execute_transport_profiles_unit_test_StubAgentProvider_",
        ),
        (
            "SignalStore",
            "execute_transport_profiles_unit_test_InMemorySignalStore_",
        ),
        (
            "ArtifactStore",
            "execute_transport_profiles_unit_test_InMemoryArtifactStore_",
        ),
        (
            "CredentialProvider",
            "execute_transport_services_sdlc_providers_stub_credential_provider_StubCredentialProvider_",
        ),
    ];

    for (interface_name, expected_prefix) in &expected_stub_execute_prefixes {
        assert!(
            has_node_with_prefix(node_ids.iter(), expected_prefix),
            "unit_test profile should resolve {interface_name} to stub implementation nodes with prefix `{expected_prefix}`"
        );
    }

    let unresolved_interface_execute_prefixes = [
        "execute_transport_interfaces_issue_provider_IssueProvider_",
        "execute_transport_interfaces_claim_store_ClaimStore_",
        "execute_transport_interfaces_outcome_ledger_OutcomeLedger_",
        "execute_transport_interfaces_agent_provider_AgentProvider_",
        "execute_transport_interfaces_signal_store_SignalStore_",
        "execute_transport_interfaces_artifact_store_ArtifactStore_",
        "execute_transport_interfaces_credential_provider_CredentialProvider_",
    ];

    for unresolved_prefix in &unresolved_interface_execute_prefixes {
        assert!(
            !has_node_with_prefix(node_ids.iter(), unresolved_prefix),
            "unit_test profile graph should not contain unresolved interface transport nodes with prefix `{unresolved_prefix}`"
        );
    }
}

#[test]
fn builds_sdlc_workflow_spec() {
    let spec = sdlc_workflow_spec().expect("sdlc workflow spec should build");
    assert!(
        spec.dag.nodes.iter().any(|node| node.id.0 == "sdlc.worker"),
        "sdlc workflow spec should include worker stage"
    );
    assert!(
        spec.dag.nodes.iter().any(|node| node.id.0 == "sdlc.report"),
        "sdlc workflow spec should include report stage"
    );
}

#[test]
fn dispatch_sdlc_unit_test_profile_dry_run_completes() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "profiles.sdlc.unit_test")
        .expect("build sdlc worker graph with unit_test profile");
    let dag = connected_subdag(&dag, "funcs.sdlc_worker::dispatch_sdlc");
    let mock_spec = auto_mock_spec(&dag, "sdlc-worker");
    let mut dry_run_mocks = mock_spec.to_dry_run_mocks();
    let boundary = mock_spec.to_boundary_mocks();

    // Guardrail for passthrough-invariant execution in DryRun mode:
    // keep prompt helpers explicitly mocked even when generated mock specs
    // omit them.
    for node in &dag.nodes {
        if node.id.0.ends_with("code_review_system_prompt") {
            dry_run_mocks.set_value(
                node.id.0.clone(),
                "return",
                Value::Str("system prompt".to_string()),
            );
        }
        if node.id.0.ends_with("code_review_user_prompt") {
            dry_run_mocks.set_value(
                node.id.0.clone(),
                "return",
                Value::Str("user prompt".to_string()),
            );
        }
    }

    let lowered = lower(&dag).expect("lower graph for entrypoint detection");
    let entrypoints = detect_entrypoints(&lowered.dag);
    let mut input_mocks = BoundaryMocks::new();
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        if let Some(value) = boundary.get_input(&node_id.0, &port_name.0) {
            input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), value.clone());
            continue;
        }
        match port_name.0.as_str() {
            "auth_token" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("test-github-token".to_string()),
            ),
            "api_key" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("test-api-key".to_string()),
            ),
            "owner" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("gunb-ai".to_string()),
            ),
            "repo" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("gunbc".to_string()),
            ),
            "worker_id" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("test-worker".to_string()),
            ),
            "llm_provider" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("anthropic".to_string()),
            ),
            "llm_model" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("claude-sonnet-4-20250514".to_string()),
            ),
            _ => {}
        }
    }

    let log = execute_with_mode_and_inputs(
        &dag,
        ExecutionMode::DryRun(dry_run_mocks),
        Some(&input_mocks),
    )
    .expect("sdlc worker dry-run should succeed");

    assert!(
        log.entries
            .iter()
            .any(|entry| entry.node_id == "funcs.sdlc_worker::dispatch_sdlc"),
        "dry-run should execute funcs.sdlc_worker::dispatch_sdlc"
    );
}

#[test]
fn sdlc_pipeline_unit_test_profile_contains_all_stage_markers() {
    let dag = build_dsl_graph_with_profile("pipelines/sdlc.dag", "profiles.sdlc.unit_test")
        .expect("build sdlc pipeline graph with unit_test profile");
    let node_ids: Vec<String> = dag.nodes.iter().map(|node| node.id.0.clone()).collect();

    let expected_stage_markers = [
        (
            "fetch",
            "execute_transport_profiles_unit_test_StubIssueProvider_get",
        ),
        (
            "claim_design",
            "execute_transport_profiles_unit_test_InMemoryClaimStore_acquire",
        ),
        ("design", "tools.design::generate_design"),
        ("design_review", "tools.design::review_design"),
        (
            "record_design_outcome",
            "execute_transport_profiles_unit_test_InMemoryOutcomeLedger_upsert",
        ),
        (
            "accept_design",
            "execute_transport_profiles_unit_test_StubIssueProvider_set_labels",
        ),
        (
            "implementation",
            "execute_transport_profiles_unit_test_StubAgentProvider_spawn",
        ),
        (
            "code_review",
            "execute_transport_extdeps_github_pull_requests_github_PullRequest_ListFiles",
        ),
        ("acceptance", "execute_transport_extdeps_cargo_cargo_Build_Test"),
        (
            "close",
            "execute_transport_profiles_unit_test_StubIssueProvider_close",
        ),
        ("report", "shared.dag_util::format_report"),
    ];

    for (stage_name, marker_prefix) in &expected_stage_markers {
        assert!(
            has_node_with_prefix(node_ids.iter(), marker_prefix),
            "unit_test profile pipeline should include stage marker for `{stage_name}` with node prefix `{marker_prefix}`"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// Track B: Feedback types and interface compilation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn compiles_feedback_store_interface() {
    let dag = build_dsl_graph("interfaces/feedback_store.dag")
        .expect("feedback store interface should compile");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn compiles_feedback_ingestion() {
    let dag = build_dsl_graph("funcs/feedback_ingestion.dag")
        .expect("feedback ingestion should compile");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn compiles_feedback_response() {
    let dag = build_dsl_graph("funcs/feedback_response.dag")
        .expect("feedback response should compile");
    assert!(!dag.nodes.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Track C: Intellectual pipeline kernel compilation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn compiles_sdlc_kernel_mapping() {
    let dag = build_dsl_graph("funcs/sdlc_kernel_mapping.dag")
        .expect("SDLC kernel mapping should compile");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn compiles_ml_investigation_exemplar() {
    let dag = build_dsl_graph("exemplars/ml_investigation.dag")
        .expect("ML investigation exemplar should compile");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn compiles_intent_expansion() {
    let dag = build_dsl_graph("funcs/intent_expansion.dag")
        .expect("intent expansion should compile");
    assert!(!dag.nodes.is_empty());
}
