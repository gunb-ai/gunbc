//! Helpers for resilient graph-mock specs on DSL-backed DAGs.

use gunbc_exec::{execute_single_node, Executable, ExecutionMode};
use gunbc_ir::transport::{
    cloud::{CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef},
    FileOp, FileResponse, RestResponse, ShellRequest, ShellResponse, TransportRequest,
    TransportResponse,
};
use gunbc_ir::{
    detect_boundaries, value_backing_for_type_id, Dag, NodeId, PortName, Value, ValueBacking,
};
use gunbc_primitives::filename;
use gunbc_test::extract_mock_requirements;
use gunbc_test::MockSpec;
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
    dedupe_boundary_mocks_keep_last(&mut spec);
    spec
}
