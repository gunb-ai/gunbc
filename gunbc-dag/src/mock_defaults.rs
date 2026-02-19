//! Helpers for resilient graph-mock specs on DSL-backed DAGs.

use gunbc_exec::{execute_single_node, Executable, ExecutionMode};
use gunbc_ir::transport::{
    cloud::{CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef},
    FileOp, FileResponse, RestResponse, ShellRequest, ShellResponse, TransportRequest,
    TransportResponse,
};
use gunbc_ir::{
    detect_boundaries, detect_entrypoints, value_backing_for_type_id, Dag, NodeId, PortName, Value,
    ValueBacking,
};
use gunbc_primitives::filename;
use gunbc_test::extract_mock_requirements;
use gunbc_test::{MockSpec, NodeExample, OutputMatcher};
use std::collections::{HashMap, HashSet};

fn default_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

fn default_value_for_type(type_id: &str) -> Value {
    match type_id {
        "TransportResponse" => default_shell_response(),
        "TransportRequest" => Value::Request(TransportRequest::Shell(ShellRequest::new("true"))),
        "CloudSecretConfig" => default_cloud_secret_config(),
        "Secret" => Value::Secret(gunbc_ir::SecretString::new("mock")),
        "FilesystemHandle" => default_fs_handle(),
        _ => match value_backing_for_type_id(type_id) {
            ValueBacking::String => Value::Str("mock".to_string()),
            ValueBacking::Bool => Value::Bool(true),
            ValueBacking::Int | ValueBacking::Float => Value::Int(1),
            ValueBacking::Json => Value::Json(serde_json::json!({"mock": true})),
            ValueBacking::Map => Value::Map(std::collections::BTreeMap::new()),
            ValueBacking::List => Value::List(vec![Value::Str("mock".to_string())]),
            ValueBacking::Set => Value::Set(vec![Value::Str("mock".to_string())]),
            ValueBacking::Unit => Value::Unit,
            ValueBacking::Bytes => Value::List(vec![Value::Int(0)]),
        },
    }
}

fn default_cloud_secret_config() -> Value {
    CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::LocalDev,
        audience: "mock-audience".to_string(),
        project_or_account: "mock-project".to_string(),
        secret: CloudSecretRef {
            prefix: "projects/mock-project/secrets/".to_string(),
            name: "mock-secret".to_string(),
            delimiter: String::new(),
            version: Some("latest".to_string()),
        },
        service_account_or_role: Some("mock-sa@mock-project.iam.gserviceaccount.com".to_string()),
        impersonate_account_or_role: None,
    }
    .into()
}

fn default_shell_response() -> Value {
    Value::Response(TransportResponse::Shell(ShellResponse::ok(String::new())))
}

fn default_file_response() -> Value {
    Value::Response(TransportResponse::File(FileResponse {
        path: "mock.txt".to_string(),
        operation: FileOp::Read,
        success: true,
        content: Some(
            r#"{"client_id":"mock-client","client_secret":"mock-secret","refresh_token":"mock-refresh","type":"authorized_user"}"#
                .to_string(),
        ),
        exists: Some(true),
        error: None,
    }))
}

fn default_rest_response() -> Value {
    let mut response = RestResponse::new(
        200,
        serde_json::json!({
            "access_token": "mock-access-token",
            "accessToken": "mock-access-token",
            "expires_in": 3600,
            "token_type": "Bearer",
            "html_url": "https://gist.github.com/mock",
            "id": "mock-id",
            "raw": "mock-token",
            "payload": { "data": "bW9jaw==" },
            "bindings": [],
            "etag": "mock-etag"
        }),
    );
    response
        .headers
        .insert("x-oauth-scopes".to_string(), "github:api".to_string());
    Value::Response(TransportResponse::Rest(response))
}

fn default_value_for_slot<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    node_id: &str,
    port_name: &str,
    type_id: &str,
) -> Value {
    if type_id != "TransportResponse" {
        return default_value_for_type(type_id);
    }

    // Pick a response variant compatible with downstream parse nodes.
    let candidates = [
        default_shell_response(),
        default_file_response(),
        default_rest_response(),
    ];
    for candidate in candidates {
        if response_candidate_satisfies_consumers(dag, node_id, port_name, &candidate) {
            return candidate;
        }
    }
    default_shell_response()
}

fn response_candidate_satisfies_consumers<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    node_id: &str,
    port_name: &str,
    candidate: &Value,
) -> bool {
    let source_node = NodeId::from(node_id);
    let source_port = PortName::from(port_name);

    let consumers: Vec<(NodeId, PortName)> = dag
        .edges
        .iter()
        .filter(|edge| edge.from_node == source_node && edge.from_port == source_port)
        .map(|edge| (edge.to_node.clone(), edge.to_port.clone()))
        .collect();

    if consumers.is_empty() {
        return false;
    }

    for (consumer_node_id, consumer_port) in consumers {
        let Some(consumer_node) = dag.get_node(&consumer_node_id) else {
            return false;
        };

        let mut probe_inputs: HashMap<String, Value> = HashMap::new();
        probe_inputs.insert(consumer_port.0.clone(), candidate.clone());

        for input in &consumer_node.inputs {
            if input.name == consumer_port {
                continue;
            }
            if !input.cardinality.requires_one() {
                continue;
            }
            probe_inputs
                .entry(input.name.0.clone())
                .or_insert_with(|| default_value_for_type(input.type_id.0.as_str()));
        }

        if execute_single_node(
            dag,
            consumer_node_id.0.as_str(),
            probe_inputs,
            ExecutionMode::Real,
        )
        .is_err()
        {
            return false;
        }
    }

    true
}

fn dedupe_boundary_mocks_keep_last(spec: &mut MockSpec) {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut deduped_rev = Vec::with_capacity(spec.boundary_mocks.len());
    for mock in spec.boundary_mocks.iter().rev() {
        let key = (mock.node.clone(), mock.port.clone());
        if seen.insert(key) {
            deduped_rev.push(mock.clone());
        }
    }
    deduped_rev.reverse();
    spec.boundary_mocks = deduped_rev;
}

/// Build a `MockSpec` by auto-filling required slots with type-compatible defaults.
pub fn auto_mock_spec<T: Executable + Clone + Send>(dag: &Dag<T>, name: &str) -> MockSpec {
    let mut reqs = extract_mock_requirements(dag, name);
    let lowered =
        gunbc_exec::lower(dag).unwrap_or_else(|error| panic!("failed to lower {name}: {error}"));
    for node in &lowered.dag.nodes {
        reqs = reqs.skip_node_example(node.id.0.as_str());
    }
    loop {
        let missing: Vec<(String, String, String)> = reqs
            .missing_slots()
            .into_iter()
            .map(|slot| {
                (
                    slot.node_id.0.clone(),
                    slot.port_name.0.clone(),
                    slot.type_id.0.clone(),
                )
            })
            .collect();
        if missing.is_empty() {
            break;
        }
        for (node, port, type_id) in missing {
            let value = default_value_for_slot(
                &lowered.dag,
                node.as_str(),
                port.as_str(),
                type_id.as_str(),
            );
            reqs = reqs
                .boundary(node.as_str(), port.as_str(), value)
                .unwrap_or_else(|error| {
                    panic!("failed to auto-fill mock slot {node}.{port} ({type_id}): {error}")
                });
        }
    }
    let mut spec = reqs.build_unchecked();
    let boundaries = detect_boundaries(&lowered.dag);
    for (node_id, port_name) in &boundaries.boundary_ports {
        let Some(node) = lowered.dag.get_node(node_id) else {
            continue;
        };
        let Some(port) = node.outputs.iter().find(|p| p.name == *port_name) else {
            continue;
        };
        spec = spec.boundary(
            node_id.0.as_str(),
            port_name.0.as_str(),
            default_value_for_slot(
                &lowered.dag,
                node_id.0.as_str(),
                port_name.0.as_str(),
                port.type_id.0.as_str(),
            ),
        );
    }
    // Auto-fill entrypoint input ports (input ports with no upstream edge).
    // These are DAG entry points that need values injected at runtime.
    let entrypoints = detect_entrypoints(&lowered.dag);
    for (node_id, port_name, type_id) in &entrypoints.entrypoint_ports {
        let value = default_value_for_type(type_id.0.as_str());
        spec = spec.input_mock(node_id.0.as_str(), port_name.0.as_str(), value);
    }

    dedupe_boundary_mocks_keep_last(&mut spec);

    // Auto-add OutputMatchers for terminal nodes (no outgoing edges) to satisfy
    // the observability invariant: every terminal reachable from a probe must
    // have an OutputMatcher.
    let has_outgoing: HashSet<&NodeId> = lowered.dag.edges.iter().map(|e| &e.from_node).collect();
    for node in &lowered.dag.nodes {
        if has_outgoing.contains(&node.id) {
            continue;
        }

        // Step 1: Collect required (non-passthrough) inputs.
        // Two-pass strategy: first populate all non-TransportResponse inputs,
        // then probe for the best response variant. This ensures the probe
        // has all required context (e.g. node_count, total_node_count) when
        // it trial-executes the node.
        let output_names: HashSet<&str> = node.outputs.iter().map(|o| o.name.0.as_str()).collect();
        let mut required_inputs: HashMap<String, Value> = HashMap::new();
        let mut deferred_response_ports: Vec<&str> = Vec::new();
        for input_port in &node.inputs {
            if output_names.contains(input_port.name.0.as_str()) {
                continue;
            }
            if input_port.cardinality.allows_empty() {
                continue;
            }
            if input_port.type_id.0 == "TransportResponse" {
                deferred_response_ports.push(input_port.name.0.as_str());
                continue;
            }
            let value = if input_port.name.0 == "skip" && input_port.type_id.0 == "Bool" {
                Value::Bool(false)
            } else {
                default_value_for_type(input_port.type_id.0.as_str())
            };
            required_inputs.insert(input_port.name.0.clone(), value);
        }

        // Step 2: Find "reliable" output ports by running a minimal trial
        // (required inputs only, no passthrough inputs). Ports that appear
        // here are always produced regardless of which chain provides inputs.
        // This prevents chain tests from failing on ports that only exist
        // when specific passthrough inputs are provided (e.g. IdentityCallableOp
        // only outputs what it receives).
        let mut minimal_inputs = required_inputs.clone();
        // Probe best response for deferred response ports in the minimal context.
        let mut minimal_example = NodeExample::new(node.id.0.as_str());
        for (k, v) in &minimal_inputs {
            minimal_example = minimal_example.input(k.as_str(), v.clone());
        }
        for port_name in &deferred_response_ports {
            let value = probe_best_response(&lowered.dag, &node.id.0, &minimal_example);
            minimal_inputs.insert(port_name.to_string(), value.clone());
            minimal_example = minimal_example.input(*port_name, value);
        }
        let reliable_ports: HashSet<String> = execute_single_node(
            &lowered.dag,
            node.id.0.as_str(),
            minimal_inputs.clone(),
            ExecutionMode::Real,
        )
        .map(|outputs| outputs.into_keys().collect())
        .unwrap_or_default();

        // Step 3: Build the NodeExample with matchers only for reliable ports
        // that also match declared output port names. Undeclared ports
        // (e.g. `stdout`, `stderr` from TransportOps::Execute) are internal
        // implementation details and should not have matchers.
        //
        // Fallback: if the minimal trial produced no reliable ports (typical for
        // DeferredCallableOp / identity passthroughs at SubDag boundaries), add
        // NonEmpty matchers for all declared output ports. These nodes forward
        // inputs as outputs, so the passthrough inputs added in Step 4 below
        // will produce the outputs. This ensures the observability invariant
        // (every terminal reachable from a probe has an OutputMatcher) holds.
        let use_fallback = reliable_ports.is_empty() && !node.outputs.is_empty();
        let mut example = NodeExample::new(node.id.0.as_str());
        for port in &node.outputs {
            if !use_fallback && !reliable_ports.contains(&port.name.0) {
                continue;
            }
            let matcher = if use_fallback {
                OutputMatcher::NonEmpty
            } else {
                match value_backing_for_type_id(port.type_id.0.as_str()) {
                    ValueBacking::Bool => OutputMatcher::IsBool,
                    ValueBacking::Int | ValueBacking::Float => OutputMatcher::IsInt,
                    ValueBacking::String => OutputMatcher::IsString,
                    _ => OutputMatcher::NonEmpty,
                }
            };
            example = example.output(port.name.0.as_str(), matcher);
        }

        // Step 4: Add passthrough inputs so the test_example test works.
        // Passthrough ops (IdentityCallableOp, DeferredCallableOp) forward
        // inputs as outputs, so providing these makes test_example succeed.
        for output in &node.outputs {
            let passthrough_value =
                if output.name.0 == "skip" && output.type_id.0 == "Bool" {
                    Value::Bool(false)
                } else {
                    default_value_for_type(output.type_id.0.as_str())
                };
            example = example.input(output.name.0.as_str(), passthrough_value);
        }
        // Add required non-passthrough inputs.
        for (k, v) in &required_inputs {
            example = example.input(k.as_str(), v.clone());
        }
        // Add deferred response inputs.
        for port_name in &deferred_response_ports {
            if let Some(v) = minimal_inputs.get(*port_name) {
                example = example.input(*port_name, v.clone());
            }
        }

        spec = spec.node_example(example);
    }

    // Also add OutputMatchers for terminal nodes in the *original* (pre-lowered)
    // DAG. SubDag boundary nodes (e.g., `tools.bootstrap::bootstrap`) are terminal
    // in the original DAG but get expanded away during lowering. The observability
    // invariant checks the original DAG's topology, so these need matchers too.
    let lowered_node_ids: HashSet<&str> =
        lowered.dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
    let original_has_outgoing: HashSet<&NodeId> =
        dag.edges.iter().map(|e| &e.from_node).collect();
    for node in &dag.nodes {
        if original_has_outgoing.contains(&node.id) {
            continue;
        }
        // Skip nodes that exist in the lowered DAG (already handled above).
        if lowered_node_ids.contains(node.id.0.as_str()) {
            continue;
        }
        let mut example = NodeExample::new(node.id.0.as_str());
        for port in &node.outputs {
            example = example.output(port.name.0.as_str(), OutputMatcher::NonEmpty);
            example = example.input(
                port.name.0.as_str(),
                default_value_for_type(port.type_id.0.as_str()),
            );
        }
        if !example.outputs.is_empty() {
            spec = spec.node_example(example);
        }
    }

    spec
}

/// Try executing a terminal node with each response variant and return the
/// first one that produces the expected outputs. Falls back to Shell.
fn probe_best_response<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    node_id: &str,
    partial_example: &NodeExample,
) -> Value {
    let candidates = [
        default_shell_response(),
        default_file_response(),
        default_rest_response(),
    ];

    for candidate in candidates {
        let mut inputs = partial_example.inputs.clone();
        inputs.insert("response".to_string(), candidate.clone());
        if execute_single_node(dag, node_id, inputs, ExecutionMode::Real).is_ok() {
            return candidate;
        }
    }

    default_shell_response()
}
