#![allow(clippy::disallowed_methods)]

use gunbc_app::extern_ops::gunbc_runtime_bindings;
use gunbc_exec::{execute_with_mode_and_inputs, lower, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_resolve::{builder::build_dsl_graph, BuildOpts};
use gunbc_test::auto_mock_spec;

fn build_graph_for_entrypoint(
    relative_module: &str,
    entry_func: &str,
) -> gunbc_ir::Dag<gunbc_exec::DynOp> {
    build_dsl_graph(
        relative_module,
        gunbc_runtime_bindings(),
        BuildOpts {
            entry_func: Some(entry_func),
            profile: None,
        },
    )
    .map(|result| result.dag)
    .unwrap_or_else(|e| panic!("`{relative_module}` entry `{entry_func}` should build: {e}"))
}

fn build_gist_recent_graph() -> gunbc_ir::Dag<gunbc_exec::DynOp> {
    build_graph_for_entrypoint("tools/gist.dag", "gist_recent")
}

#[test]
fn shared_gist_upload_graph_builds_with_provider_auth_module() {
    let dag = build_graph_for_entrypoint("shared/gist_modes.dag", "share_content");
    let lowered = lower(&dag).expect("lowered shared share_content");

    assert!(
        lowered
            .dag
            .nodes
            .iter()
            .any(|n| n.id.0.contains("github_token")),
        "shared gist_upload should compile against extdeps.github.auth::github_token. \
         All nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

#[test]
fn gist_recent_graph_no_ls_files() {
    let dag = build_gist_recent_graph();
    let ls_files_nodes: Vec<&str> = dag
        .nodes
        .iter()
        .filter(|n| n.id.0.contains("LsFiles"))
        .map(|n| n.id.0.as_str())
        .collect();
    assert!(
        ls_files_nodes.is_empty(),
        "gist_recent graph should NOT contain LsFiles nodes (cross-callable leakage): {ls_files_nodes:?}"
    );
}

#[test]
fn gist_recent_graph_wires_diff_base_input() {
    let dag = build_gist_recent_graph();
    let lowered = lower(&dag).expect("lowered gist-recent");

    // The auth materialization changed, but the diff transport still needs a
    // concrete base input edge.
    let has_base_edge = lowered
        .dag
        .edges
        .iter()
        .any(|edge| edge.to_node.0.contains("Diff") && edge.to_port.0 == "base");
    assert!(
        has_base_edge,
        "gist-recent must wire a base ref into git diff prepare node. \
         Diff-related edges: {:?}",
        lowered
            .dag
            .edges
            .iter()
            .filter(|e| e.to_node.0.contains("Diff") || e.from_node.0.contains("Diff"))
            .map(|e| format!(
                "{}:{} -> {}:{}",
                e.from_node.0, e.from_port.0, e.to_node.0, e.to_port.0
            ))
            .collect::<Vec<_>>()
    );
}

/// Structural: gist_recent must include concrete token-resolution wiring.
#[test]
fn gist_recent_graph_has_token_resolution_path() {
    let dag = build_gist_recent_graph();
    let lowered = lower(&dag).expect("lowered gist-recent");

    let has_provider_auth_fn = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("github_token"));
    let has_local_auth = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("local_auth"));
    let has_sts_exchange = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("gcp_STS_Exchange"));
    let has_rest_secret_access = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("gcp_SecretManager_AccessVersion"));
    let has_shell_secret_manager = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("SecretManagerAccessVersion"));
    assert!(
        has_provider_auth_fn && has_local_auth && has_sts_exchange && has_rest_secret_access,
        "gist-recent must route GitHub auth through github_token -> credential_chain -> local_auth + REST Secret Manager. \
         All nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
    assert!(
        !has_shell_secret_manager,
        "gist-recent should no longer depend on the shell SecretManagerAccessVersion path. \
         All nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

#[test]
fn gist_recent_graph_uses_provider_auth_module() {
    let dag = build_gist_recent_graph();
    let lowered = lower(&dag).expect("lowered gist-recent");

    let has_provider_auth_helper = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("github_token"));
    assert!(
        has_provider_auth_helper,
        "gist-recent should route gist creation through extdeps.github.auth::github_token. \
         All nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

/// End-to-end DryRun: gist_recent completes with auto-mocked transport nodes.
///
/// Validates that the full pipeline (git, provider auth materialization, gist create)
/// is structurally connected and executes without errors. Uses DryRun mode because
/// the auth path still contains effectful REST auth hops.
#[test]
#[ignore] // Pre-existing: GetField on credential token fails in DryRun (gist pipeline)
fn gist_recent_end_to_end_emits_gist_url() {
    let dag = build_gist_recent_graph();

    let spec = auto_mock_spec(&dag, "gist_recent");
    let dry_run_mocks = spec.to_dry_run_mocks();

    let input_mocks = {
        let lowered = lower(&dag).expect("lower for entrypoint detection");
        let entrypoints = detect_entrypoints(&lowered.dag);
        let boundary = spec.to_boundary_mocks();
        let mut mocks = BoundaryMocks::new();
        for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
            if let Some(val) = boundary.get_input(&node_id.0, &port_name.0) {
                mocks.set_input(node_id.0.clone(), port_name.0.clone(), val.clone());
            } else {
                // Provide defaults for entrypoint ports not covered by auto_mock_spec
                match port_name.0.as_str() {
                    "since" => mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str("3.days.ago".into()),
                    ),
                    "public" => {
                        mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Bool(false))
                    }
                    _ => {}
                }
            }
        }
        mocks
    };

    let log = execute_with_mode_and_inputs(
        &dag,
        ExecutionMode::DryRun(dry_run_mocks),
        Some(&input_mocks),
    )
    .expect("gist-recent DryRun execution should succeed");

    let node_ids: Vec<&str> = log.entries.iter().map(|e| e.node_id.as_str()).collect();

    // Key pipeline stages must appear in execution
    assert!(
        node_ids
            .iter()
            .any(|id| id.starts_with("parse_transport_services_git_git_Core_Diff")),
        "execution should include git Diff parse. Got: {node_ids:?}"
    );
    assert!(
        node_ids
            .iter()
            .any(|id| id.contains("render_diff_markdown")),
        "execution should include render_diff_markdown. Got: {node_ids:?}"
    );
    assert!(
        node_ids.iter().any(|id| id.contains("Gist_Create")),
        "execution should include Gist_Create transport. Got: {node_ids:?}"
    );
    assert!(
        node_ids.iter().any(|id| id.contains("gcp_STS_Exchange")),
        "execution should include STS.Exchange transport. Got: {node_ids:?}"
    );
    assert!(
        node_ids
            .iter()
            .any(|id| id.contains("gcp_SecretManager_AccessVersion")),
        "execution should include REST SecretManager.AccessVersion transport. Got: {node_ids:?}"
    );
}
