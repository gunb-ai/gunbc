//! Helpers for resilient graph-mock specs on DSL-backed DAGs.

use gunbc_exec::{execute_single_node, Executable, ExecutionMode};
use gunbc_ir::transport::{
    cloud::{CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef},
    FileOp, FileResponse, RestResponse, ShellRequest, ShellResponse, TransportRequest,
    TransportResponse,
};
use gunbc_ir::{
    detect_boundaries, detect_entrypoints, value_backing_for_type_id, Cardinality, Dag, NodeId,
    PortName, Value, ValueBacking,
};
use gunbc_primitives::filename;
use gunbc_test::extract_mock_requirements;
use gunbc_test::{MockSpec, NodeExample, OutputMatcher};
use std::collections::{HashMap, HashSet};

fn default_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GcpFieldKind {
    Audience,
    Project,
    Secret,
    SubjectToken,
    Version,
    ServiceAccount,
}

fn typed_gcp_field_kind(type_id: &str) -> Option<GcpFieldKind> {
    match type_id {
        // Preferred modeling path: refined type aliases encode intent directly.
        "OidcAudience" | "WifAudience" => Some(GcpFieldKind::Audience),
        "GcpProjectId" => Some(GcpFieldKind::Project),
        "GcpSecretId" => Some(GcpFieldKind::Secret),
        "GcpSubjectToken" | "OidcSubjectToken" => Some(GcpFieldKind::SubjectToken),
        "GcpSecretVersion" => Some(GcpFieldKind::Version),
        "GcpServiceAccountEmail" => Some(GcpFieldKind::ServiceAccount),
        _ => None,
    }
}

fn gcp_field_value_for_kind(kind: GcpFieldKind) -> Value {
    match kind {
        GcpFieldKind::Audience => Value::Str("mock-audience".to_string()),
        GcpFieldKind::Project => Value::Str("mock-project".to_string()),
        GcpFieldKind::Secret => Value::Str("mock-secret".to_string()),
        GcpFieldKind::SubjectToken => Value::Str("mock-subject-token".to_string()),
        GcpFieldKind::Version => Value::Str("latest".to_string()),
        GcpFieldKind::ServiceAccount => {
            Value::Str("mock-sa@mock-project.iam.gserviceaccount.com".to_string())
        }
    }
}

/// Legacy compatibility path while graph ports migrate to refined type aliases.
fn legacy_gcp_field_value_by_name(port_name: &str) -> Option<Value> {
    match port_name {
        "audience" => Some(gcp_field_value_for_kind(GcpFieldKind::Audience)),
        "project" => Some(gcp_field_value_for_kind(GcpFieldKind::Project)),
        "secret" | "secret_name" => Some(gcp_field_value_for_kind(GcpFieldKind::Secret)),
        "subject_token" => Some(gcp_field_value_for_kind(GcpFieldKind::SubjectToken)),
        "version" => Some(gcp_field_value_for_kind(GcpFieldKind::Version)),
        "service_account" | "service_account_or_role" => {
            Some(gcp_field_value_for_kind(GcpFieldKind::ServiceAccount))
        }
        _ => None,
    }
}

/// Returns a realistic GCP mock value when semantic intent is encoded in the
/// type ID (preferred) or, as a compatibility path, in legacy port names.
///
/// This avoids GCP prepare ops falling back to `"(unresolved)"` when their
/// inputs are optional or entrypoint ports filled with generic `"mock"`.
fn gcp_field_value(type_id: &str, port_name: &str) -> Option<Value> {
    typed_gcp_field_kind(type_id)
        .map(gcp_field_value_for_kind)
        .or_else(|| legacy_gcp_field_value_by_name(port_name))
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

fn default_value_for_port(type_id: &str, cardinality: Cardinality) -> Value {
    if type_id == "Any" && cardinality.max != Some(1) {
        let count = cardinality.min.max(1) as usize;
        return Value::List(vec![Value::Str("mock".to_string()); count]);
    }

    let base = default_value_for_type(type_id);
    if cardinality.max == Some(1) {
        return base;
    }

    match base {
        Value::List(_) => base,
        Value::Set(values) => Value::List(values),
        value => {
            let count = cardinality.min.max(1) as usize;
            Value::List(vec![value; count])
        }
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
        bytes: None,
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
            "etag": "mock-etag",
            // LLM response shapes so parse nodes extract non-empty content.
            // Anthropic Messages API: content/0/text
            "content": [{ "type": "text", "text": "Mock LLM response content." }],
            "stop_reason": "end_turn",
            // OpenAI Chat Completions API: choices/0/message/content
            "choices": [{ "message": { "content": "Mock LLM response content." }, "finish_reason": "stop" }],
            // OpenAI Responses API: output/0/content/0/text
            "output": [{ "content": [{ "text": "Mock LLM response content." }] }],
            // Shared usage shape (both providers)
            "usage": { "input_tokens": 100, "output_tokens": 200, "prompt_tokens": 100, "completion_tokens": 200 },
            "model": "mock-model"
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
    cardinality: Cardinality,
) -> Value {
    if type_id != "TransportResponse" {
        return default_value_for_port(type_id, cardinality);
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
            probe_inputs.entry(input.name.0.clone()).or_insert_with(|| {
                default_value_for_port(input.type_id.0.as_str(), input.cardinality)
            });
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
            let cardinality = lowered
                .dag
                .get_node(&NodeId::from(node.as_str()))
                .and_then(|candidate| {
                    candidate
                        .outputs
                        .iter()
                        .find(|output| output.name.0 == port)
                        .map(|output| output.cardinality)
                })
                .unwrap_or(Cardinality::ONE);
            let value = default_value_for_slot(
                &lowered.dag,
                node.as_str(),
                port.as_str(),
                type_id.as_str(),
                cardinality,
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
                port.cardinality,
            ),
        );
    }
    // Auto-fill entrypoint input ports (input ports with no upstream edge).
    // These are DAG entry points that need values injected at runtime.
    let entrypoints = detect_entrypoints(&lowered.dag);
    for (node_id, port_name, type_id) in &entrypoints.entrypoint_ports {
        let cardinality = lowered
            .dag
            .get_node(node_id)
            .and_then(|candidate| {
                candidate
                    .inputs
                    .iter()
                    .find(|input| input.name == *port_name)
                    .map(|input| input.cardinality)
            })
            .unwrap_or(Cardinality::ONE);
        let value = gcp_field_value(type_id.0.as_str(), port_name.0.as_str())
            .unwrap_or_else(|| default_value_for_port(type_id.0.as_str(), cardinality));
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
            // Inject known GCP field values even for optional inputs.
            if input_port.cardinality.allows_empty() {
                if let Some(value) =
                    gcp_field_value(input_port.type_id.0.as_str(), input_port.name.0.as_str())
                {
                    required_inputs.insert(input_port.name.0.clone(), value);
                }
                continue;
            }
            if input_port.type_id.0 == "TransportResponse" {
                deferred_response_ports.push(input_port.name.0.as_str());
                continue;
            }
            let value = if input_port.name.0 == "skip" && input_port.type_id.0 == "Bool" {
                Value::Bool(false)
            } else {
                gcp_field_value(input_port.type_id.0.as_str(), input_port.name.0.as_str())
                    .unwrap_or_else(|| {
                        default_value_for_port(
                            input_port.type_id.0.as_str(),
                            input_port.cardinality,
                        )
                    })
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
        // Fallback: if no reliable port matches a declared output port name
        // (typical for IdentityCallableOp passthroughs),
        // add NonEmpty matchers for all declared output ports. These nodes
        // forward inputs as outputs, so the passthrough inputs added in Step 4
        // below will produce the outputs. This ensures the observability invariant
        // (every terminal reachable from a probe has an OutputMatcher) holds.
        let has_matching_reliable = node
            .outputs
            .iter()
            .any(|p| reliable_ports.contains(&p.name.0));
        let use_fallback = !has_matching_reliable && !node.outputs.is_empty();
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
        // Passthrough ops (IdentityCallableOp) forward
        // inputs as outputs, so providing these makes test_example succeed.
        for output in &node.outputs {
            let passthrough_value = if output.name.0 == "skip" && output.type_id.0 == "Bool" {
                Value::Bool(false)
            } else {
                default_value_for_port(output.type_id.0.as_str(), output.cardinality)
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

    eprintln!(
        "mock warning: probe_best_response for `{node_id}` — all response \
         candidates failed, falling back to default_shell_response()"
    );
    default_shell_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcp_field_value_prefers_typed_hints_over_port_names() {
        assert_eq!(
            gcp_field_value("GcpProjectId", "renamed_project"),
            Some(Value::Str("mock-project".to_string()))
        );
        assert_eq!(
            gcp_field_value("GcpServiceAccountEmail", "identity"),
            Some(Value::Str(
                "mock-sa@mock-project.iam.gserviceaccount.com".to_string()
            ))
        );
        assert_eq!(
            gcp_field_value("GcpSubjectToken", "token"),
            Some(Value::Str("mock-subject-token".to_string()))
        );
    }

    #[test]
    fn gcp_field_value_keeps_legacy_name_fallback_for_string_ports() {
        assert_eq!(
            gcp_field_value("String", "project"),
            Some(Value::Str("mock-project".to_string()))
        );
        assert_eq!(gcp_field_value("String", "not_a_gcp_field"), None);
    }

    #[test]
    fn auto_mock_spec_ci_graph_produces_transport_mocks() {
        let dag = crate::ci::build_ci_graph().expect("build ci graph");
        let spec = auto_mock_spec(&dag, "ci");
        // CI graph should have transport mocks for service transport nodes.
        assert!(
            !spec.transport_mocks.is_empty(),
            "CI graph should have at least one transport mock"
        );
    }
}
