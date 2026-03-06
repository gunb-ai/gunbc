use gunbc_app::extern_ops::gunbc_runtime_bindings;
use gunbc_app::sdlc_workflow_spec;
use gunbc_exec::{execute_dag, lower, BoundaryMocks, DynOp, ExecuteConfig, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Dag, Value};
use gunbc_resolve::{builder::build_dsl_graph, BuildOpts};
use gunbc_test::auto_mock_spec;
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

fn ensure_test_profile_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| unsafe {
        std::env::set_var("GITHUB_TOKEN", "test-github-token");
    });
}

fn build_graph(relative_module: &str) -> Dag<DynOp> {
    ensure_test_profile_env();
    build_dsl_graph(
        relative_module,
        gunbc_runtime_bindings(),
        BuildOpts {
            profile: Some("local"),
            ..BuildOpts::default()
        },
    )
    .map(|result| result.dag)
    .unwrap_or_else(|e| panic!("`{relative_module}` should resolve: {e}"))
}

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
    subdag
        .edges
        .retain(|edge| visited.contains(&edge.from_node.0) && visited.contains(&edge.to_node.0));
    subdag
}

#[test]
fn builds_sdlc_worker_dsl_graph() {
    let dag = build_graph("funcs/sdlc_worker.dag");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn builds_sdlc_stages_dsl_graph() {
    let dag = build_graph("funcs/sdlc_stages.dag");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn builds_sdlc_workflow_dsl_graph() {
    let dag = build_graph("workflows/sdlc.dag");
    assert!(!dag.nodes.is_empty());
}

#[test]
fn sdlc_worker_uses_provider_auth_modules_and_not_legacy_bindings() {
    let dag = build_graph("funcs/sdlc_worker.dag");
    let node_ids: Vec<String> = dag.nodes.iter().map(|node| node.id.0.clone()).collect();

    assert!(
        node_ids.iter().any(|id| id.contains("github_token")),
        "worker graph should route GitHub auth through extdeps.github.auth::github_token"
    );
    assert!(
        node_ids.iter().any(|id| id.contains("llm_api_key")),
        "worker graph should route LLM auth through extdeps.llm.auth::llm_api_key"
    );

    let deleted_legacy_binding_prefixes = [
        "execute_transport_extdeps_sdlc_providers_stub_providers_stub_IssueProvider_",
        "execute_transport_extdeps_sdlc_providers_stub_providers_stub_ClaimStore_",
        "execute_transport_extdeps_sdlc_providers_stub_providers_stub_OutcomeLedger_",
        "execute_transport_extdeps_sdlc_providers_stub_providers_stub_AgentProvider_",
        "execute_transport_extdeps_sdlc_providers_gcp_credential_provider_GcpWifCredentialProvider_",
        "resolve_github_token",
        "resolve_llm_api_key",
    ];

    for deleted_prefix in &deleted_legacy_binding_prefixes {
        assert!(
            !has_node_with_prefix(node_ids.iter(), deleted_prefix),
            "worker graph should not contain deleted legacy provider wiring with prefix `{deleted_prefix}`"
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
fn dispatch_sdlc_dry_run_completes_without_legacy_bindings() {
    let dag = build_graph("funcs/sdlc_worker.dag");
    let dag = connected_subdag(&dag, "funcs.sdlc_worker::dispatch_sdlc");
    let mock_spec = auto_mock_spec(&dag, "sdlc-worker");
    let mut dry_run_mocks = mock_spec.to_dry_run_mocks();
    let boundary = mock_spec.to_boundary_mocks();

    // Keep prompt helpers explicitly mocked when generated mock specs omit them.
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

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::DryRun(dry_run_mocks),
            input_mocks: Some(&input_mocks),
            ..Default::default()
        },
    )
    .expect("sdlc worker dry-run should succeed");

    assert!(
        log.entries
            .iter()
            .any(|entry| entry.node_id == "funcs.sdlc_worker::dispatch_sdlc"),
        "dry-run should execute funcs.sdlc_worker::dispatch_sdlc"
    );
}
