//! Auto-mock specification builder.
//!
//! Builds a complete `MockSpec` from a DAG by probing its structure:
//! type-compatible default values are generated for every boundary slot,
//! entrypoint port, and terminal node. The probing algorithm trial-executes
//! downstream consumer nodes to select the best response variant.

use gunbc_exec::{execute_single_node, Executable, ExecutionMode};
use gunbc_ir::transport::{
    FileOp, FileResponse, RestResponse, ShellRequest, ShellResponse, TransportRequest,
    TransportResponse,
};
use gunbc_ir::{
    detect_boundaries, detect_entrypoints, parse_map_type_id,
    variant_witnesses, Cardinality, Dag, NodeBody, NodeId, PortName, TypeId, TypeOp, TypeRegistry,
    Value, ValueBacking,
};
use gunbc_ir::filename;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    extract_mock_requirements, MockProvider, MockResponseSynthesis, MockSpec, NodeExample,
    OutputMatcher,
};

fn default_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

fn merged_registry(dsl_registry: Option<&TypeRegistry>) -> TypeRegistry {
    let mut registry = TypeRegistry::with_core_types();
    if let Some(dsl) = dsl_registry {
        registry.merge_dsl_types(dsl);
    }
    registry
}

fn parse_unary_generic_type_id<'a>(type_id: &'a str, wrapper: &str) -> Option<&'a str> {
    let rest = type_id.strip_prefix(wrapper)?;
    let inner = rest.strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

fn optional_inner_type_id(type_id: &str) -> Option<&str> {
    if let Some(inner) = type_id.strip_suffix('?') {
        let inner = inner.trim();
        if !inner.is_empty() {
            return Some(inner);
        }
    }
    if let Some(inner) = parse_unary_generic_type_id(type_id, "Optional") {
        return Some(inner);
    }
    let inner = type_id.strip_prefix("Optional")?;
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

fn parse_container_alias_inner<'a>(type_id: &'a str, suffix: &str) -> Option<&'a str> {
    let inner = type_id.strip_suffix(suffix)?;
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

fn is_placeholder_witness(value: &Value) -> bool {
    matches!(value, Value::Str(s) if s.starts_with('<') && s.ends_with('>'))
}

/// Generate a structurally correct witness value for a type using the registry.
///
/// Returns `Some` if the type can be resolved to a concrete value shape
/// (product types → Map, sum types → first variant, containers → wrapped inner).
/// Returns `None` for primitives that should use simpler fallback logic.
pub fn typed_witness_value(type_id: &str, registry: &TypeRegistry) -> Option<Value> {
    typed_witness_value_depth(type_id, registry, 0)
}

/// Max recursion depth for product field witness generation.
/// Prevents infinite recursion on recursive types.
const MAX_WITNESS_DEPTH: u8 = 4;

fn typed_witness_value_depth(type_id: &str, registry: &TypeRegistry, depth: u8) -> Option<Value> {
    if depth > MAX_WITNESS_DEPTH {
        return None;
    }

    if let Some(inner) = optional_inner_type_id(type_id) {
        let _ = inner;
        return Some(Value::Unit);
    }

    if let Some(inner) = parse_unary_generic_type_id(type_id, "List")
        .or_else(|| parse_container_alias_inner(type_id, "List"))
    {
        return typed_witness_value_depth(inner, registry, depth + 1)
            .map(|value| Value::List(vec![value]));
    }

    if let Some(inner) = parse_unary_generic_type_id(type_id, "Set")
        .or_else(|| parse_container_alias_inner(type_id, "Set"))
    {
        return typed_witness_value_depth(inner, registry, depth + 1)
            .map(|value| Value::Set(vec![value]));
    }

    if let Some((key_type, value_type)) = parse_map_type_id(type_id) {
        if key_type == "String" {
            let value = typed_witness_value_depth(&value_type, registry, depth + 1)?;
            let mut map = BTreeMap::new();
            map.insert("mock_key".to_string(), value);
            return Some(Value::Map(map));
        }
    }

    if let Some((_, value)) = variant_witnesses(type_id, registry).into_iter().next() {
        return Some(value);
    }

    let type_dag = registry.resolve_type(&TypeId::from(type_id))?;

    // Try standard witness generation first.
    let standard = gunbc_ir::contract::witnesses_checked(&type_dag)
        .ok()
        .and_then(|ws| {
            ws.into_iter()
                .map(|w| w.value)
                .find(|v| !matches!(v, Value::Unit) && !is_placeholder_witness(v))
        });
    if standard.is_some() {
        return standard;
    }

    // Standard witnesses failed (product types produce placeholder strings).
    // Check if this is a product type and build a structural Value::Map.
    product_witness(&type_dag, registry, depth)
}

/// Build a `Value::Map` witness for a product (record) type DAG.
///
/// Walks the type DAG for `Product(field_names)` nodes, extracts each field's
/// type from the corresponding `field_{name}` SubDag child, and recursively
/// generates witness values for each field.
fn product_witness(
    type_dag: &Dag<TypeOp>,
    registry: &TypeRegistry,
    depth: u8,
) -> Option<Value> {
    // Find the Product node and extract field names.
    let field_names: Vec<String> = type_dag.nodes.iter().find_map(|node| {
        if let NodeBody::Opaque(TypeOp::Product(fields)) = &node.body {
            Some(fields.clone())
        } else {
            None
        }
    })?;

    let mut map = BTreeMap::new();
    for field_name in &field_names {
        let child_id = format!("field_{field_name}");
        // Find the SubDag node for this field.
        let field_dag = type_dag.nodes.iter().find_map(|node| {
            if node.id.0 == child_id {
                if let NodeBody::SubDag(dag, _) = &node.body {
                    return Some(dag);
                }
            }
            None
        });
        let value = if let Some(fdag) = field_dag {
            // Extract the base type from the field's type DAG and recurse.
            let field_base = gunbc_ir::contract::base_type(fdag);
            let inner = field_base
                .as_deref()
                .and_then(|ft| typed_witness_value_depth(ft, registry, depth + 1))
                .unwrap_or_else(|| Value::Str("mock".to_string()));
            // base_type strips container wrappers — re-wrap if the field
            // has a List/Set/Optional wrapper so the mock matches the
            // expected runtime shape.
            match gunbc_ir::contract::wrapper_kind(fdag) {
                Some(gunbc_ir::WrapperKind::List | gunbc_ir::WrapperKind::NonEmptyList) => {
                    Value::List(vec![inner])
                }
                Some(gunbc_ir::WrapperKind::Set | gunbc_ir::WrapperKind::NonEmptySet) => {
                    Value::Set(vec![inner])
                }
                Some(gunbc_ir::WrapperKind::Optional) => Value::Unit,
                Some(gunbc_ir::WrapperKind::Map) => {
                    let mut m = BTreeMap::new();
                    m.insert("mock_key".to_string(), inner);
                    Value::Map(m)
                }
                None => inner,
            }
        } else {
            Value::Str("mock".to_string())
        };
        map.insert(field_name.clone(), value);
    }

    Some(Value::Map(map))
}

fn default_value_for_type(type_id: &str, registry: &TypeRegistry) -> Value {
    match type_id {
        "TransportResponse" => default_shell_response(),
        "TransportRequest" => Value::Request(TransportRequest::Shell(ShellRequest::new("true"))),
        "Secret" => Value::Secret(gunbc_ir::SecretString::new("mock")),
        "FilesystemHandle" => default_fs_handle(),
        _ => {
            if let Some(value) = typed_witness_value(type_id, registry) {
                return value;
            }
            // Use the passed-in registry (which includes DSL types) instead
            // of the static core-only registry from value_backing_for_type_id().
            match registry.value_backing(&gunbc_ir::TypeId::from(type_id)).unwrap_or(ValueBacking::Json) {
                ValueBacking::String => Value::Str("mock".to_string()),
                ValueBacking::Secret => Value::Secret(gunbc_ir::SecretString::new("mock")),
                ValueBacking::Bool => Value::Bool(true),
                ValueBacking::Int | ValueBacking::Float => Value::Int(1),
                ValueBacking::Json => Value::Json(serde_json::json!({"mock": true})),
                ValueBacking::Map => Value::Map(BTreeMap::new()),
                ValueBacking::List => Value::List(vec![Value::Str("mock".to_string())]),
                ValueBacking::Set => Value::Set(vec![Value::Str("mock".to_string())]),
                ValueBacking::Unit => Value::Unit,
                ValueBacking::Bytes => Value::List(vec![Value::Int(0)]),
            }
        }
    }
}

fn default_value_for_port(type_id: &str, cardinality: Cardinality, registry: &TypeRegistry) -> Value {
    if type_id == "Any" && cardinality.max != Some(1) {
        let count = cardinality.min.max(1) as usize;
        return Value::List(vec![Value::Str("mock".to_string()); count]);
    }

    let base = default_value_for_type(type_id, registry);
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

/// Resolve MockProvider from a node's OperationKey metadata.
///
/// Uses the typed `OperationKey` instead of parsing node ID strings.
/// Falls back to node ID heuristic only when no operation_key is present
/// (e.g., nodes not stamped by the lowerer).
fn resolve_mock_provider<T>(dag: &Dag<T>, node_id: &str) -> MockProvider {
    if let Some(node) = dag.get_node(&NodeId::from(node_id)) {
        if let Some(op_key) = &node.operation_key {
            return operation_key_to_mock_provider(op_key);
        }
    }
    // Fallback for nodes without operation_key (legacy or non-service nodes).
    infer_provider_from_node_id(node_id)
}

/// Resolve MockProvider from an OperationKey's full service path.
///
/// Handles LLM sub-providers: "llm.Anthropic" → Anthropic, "llm.OpenAI" → OpenAi.
fn operation_key_to_mock_provider(op_key: &gunbc_ir::OperationKey) -> MockProvider {
    let service_lower = op_key.service.to_lowercase();
    if service_lower.starts_with("github") {
        MockProvider::GitHub
    } else if service_lower.starts_with("gcp") {
        MockProvider::Gcp
    } else if service_lower.contains("anthropic") {
        MockProvider::Anthropic
    } else if service_lower.contains("openai") {
        MockProvider::OpenAi
    } else {
        MockProvider::Generic
    }
}

/// Legacy fallback: infer the mock provider from a node ID string.
/// Only used when no OperationKey metadata is available on the node.
fn infer_provider_from_node_id(node_id: &str) -> MockProvider {
    let lower = node_id.to_lowercase();
    if lower.starts_with("github.") || lower.contains("/github.") || lower.contains("gist") {
        MockProvider::GitHub
    } else if lower.starts_with("gcp.") || lower.contains("/gcp.") || lower.contains("secret") {
        MockProvider::Gcp
    } else if lower.starts_with("llm.anthropic") || lower.contains("anthropic") {
        MockProvider::Anthropic
    } else if lower.starts_with("llm.openai") || lower.contains("openai") {
        MockProvider::OpenAi
    } else {
        MockProvider::Generic
    }
}

/// Provider-aware REST response synthesis.
///
/// Uses `MockResponseSynthesis` from `gunbc_test::mock_synthesis` to generate
/// provider-specific response shapes instead of the kitchen sink blob.
fn rest_response_for_provider(provider: MockProvider) -> Value {
    let spec = MockResponseSynthesis::success(provider);
    let response = crate::synthesize_rest_response(&spec);
    Value::Response(TransportResponse::Rest(response))
}

/// Default REST response for unknown providers (kitchen sink fallback).
///
/// This fallback includes shapes from all providers so parse nodes can extract
/// fields regardless of which provider the operation targets. Once all callers
/// use `rest_response_for_provider`, this can be removed.
fn default_rest_response() -> Value {
    // Synthesize a merged response that satisfies all provider parse nodes.
    // This is the "kitchen sink" fallback for untyped contexts.
    let mut response = RestResponse::new(
        200,
        serde_json::json!({
            // OAuth/token fields (shared across auth flows)
            "access_token": "mock-access-token",
            "accessToken": "mock-access-token",
            "expires_in": 3600,
            "token_type": "Bearer",
            // GitHub fields
            "html_url": "https://gist.github.com/mock",
            "id": "mock-id",
            "state": "open",
            // GCP fields
            "raw": "mock-token",
            "payload": { "data": "bW9jaw==" },
            "bindings": [],
            "etag": "mock-etag",
            "name": "projects/mock-project/secrets/mock-secret/versions/1",
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
    registry: &TypeRegistry,
) -> Value {
    if type_id != "TransportResponse" {
        return default_value_for_port(type_id, cardinality, registry);
    }

    // Try provider-specific REST response first (using OperationKey metadata).
    let provider = resolve_mock_provider(dag, node_id);
    let provider_rest = rest_response_for_provider(provider);
    if response_candidate_satisfies_consumers(dag, node_id, port_name, &provider_rest, registry) {
        return provider_rest;
    }

    // Fall back to trying all response variants (REST-first since most
    // service operations are REST-based).
    let candidates = [
        default_rest_response(),
        default_shell_response(),
        default_file_response(),
    ];
    for candidate in candidates {
        if response_candidate_satisfies_consumers(dag, node_id, port_name, &candidate, registry) {
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
    registry: &TypeRegistry,
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
                default_value_for_port(input.type_id.0.as_str(), input.cardinality, registry)
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
///
/// When `dsl_registry` is provided, DSL-defined types (records, sum types) get
/// structurally correct mock values instead of falling back to `{"mock": true}`.
pub fn auto_mock_spec<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    name: &str,
    dsl_registry: Option<&TypeRegistry>,
) -> MockSpec {
    let registry = merged_registry(dsl_registry);
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
                &registry,
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
                &registry,
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
        let value = default_value_for_port(type_id.0.as_str(), cardinality, &registry);
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

        // Skip literal source nodes — these evaluate fn body expressions
        // that may produce empty values (e.g., comparison results).
        // Their output is unpredictable with passthrough inputs.
        if node.id.0.starts_with("literal_source_") {
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
            if input_port.type_id.0 == "TransportResponse" {
                deferred_response_ports.push(input_port.name.0.as_str());
                continue;
            }
            let value = if input_port.name.0 == "skip" && input_port.type_id.0 == "Bool" {
                Value::Bool(false)
            } else {
                default_value_for_port(input_port.type_id.0.as_str(), input_port.cardinality, &registry)
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
                match registry.value_backing(&port.type_id).unwrap_or(ValueBacking::Json) {
                    ValueBacking::Bool => OutputMatcher::IsBool,
                    ValueBacking::Int | ValueBacking::Float => OutputMatcher::IsInt,
                    ValueBacking::String => OutputMatcher::IsString,
                    ValueBacking::Secret => OutputMatcher::IsSecret,
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
                default_value_for_port(output.type_id.0.as_str(), output.cardinality, &registry)
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
///
/// Probe order: provider-specific REST → generic REST → Shell → File.
/// REST is tried first because the majority of service operations are
/// REST-based (GitHub, OpenAI, Anthropic, GCP).
#[allow(clippy::disallowed_macros)]
fn probe_best_response<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    node_id: &str,
    partial_example: &NodeExample,
) -> Value {
    // Try provider-specific REST response first (using OperationKey metadata).
    let provider = resolve_mock_provider(dag, node_id);
    let provider_rest = rest_response_for_provider(provider);
    {
        let mut inputs = partial_example.inputs.clone();
        inputs.insert("response".to_string(), provider_rest.clone());
        if execute_single_node(dag, node_id, inputs, ExecutionMode::Real).is_ok() {
            return provider_rest;
        }
    }

    // Fall back to trying all response variants (REST-first since most
    // service operations are REST-based).
    let candidates = [
        default_rest_response(),
        default_shell_response(),
        default_file_response(),
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

/// A failure-variant mock spec derived from a success spec.
#[derive(Debug, Clone)]
pub struct FailureVariant {
    /// Human-readable tag for the failure scenario (e.g. "shell_exit_1", "rest_401_auth").
    pub tag: String,
    /// The modified MockSpec with one transport mock replaced by a failure response.
    pub spec: MockSpec,
}

/// Generate failure-variant MockSpecs from a success-path MockSpec.
///
/// For each transport boundary mock in the spec, produces one or more
/// failure variants:
/// - Shell responses: exit code 1
/// - REST responses: 401 (auth) and 500 (server error)
///
/// Each variant replaces exactly one boundary mock, keeping all others
/// at their success values. This tests that the DAG handles individual
/// transport failures correctly.
pub fn auto_mock_failure_variants(success_spec: &MockSpec) -> Vec<FailureVariant> {
    let mut variants = Vec::new();

    for (idx, mock) in success_spec.boundary_mocks.iter().enumerate() {
        let failure_values = failure_values_for_mock(&mock.value, &mock.node);
        for (tag_suffix, failure_value) in failure_values {
            let mut spec = success_spec.clone();
            spec.boundary_mocks[idx].value = failure_value;
            // Clear node examples — success-path matchers won't hold for failure paths.
            spec.node_examples = Vec::new();
            let tag = format!("{}_{}", mock.node.replace("::", "__"), tag_suffix);
            variants.push(FailureVariant { tag, spec });
        }
    }

    variants
}

/// Check if a MockSpec contains any transport boundary mocks that would
/// produce failure variants.
pub fn has_transport_boundaries(spec: &MockSpec) -> bool {
    spec.boundary_mocks
        .iter()
        .any(|m| matches!(&m.value, Value::Response(_)))
}

fn failure_values_for_mock(value: &Value, node_id: &str) -> Vec<(&'static str, Value)> {
    match value {
        Value::Response(TransportResponse::Shell(_)) => {
            vec![(
                "shell_exit_1",
                Value::Response(TransportResponse::Shell(ShellResponse {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: "mock: command failed".to_string(),
                })),
            )]
        }
        Value::Response(TransportResponse::Rest(_)) => {
            let provider = infer_provider_from_node_id(node_id);
            vec![
                (
                    "rest_401_auth",
                    Value::Response(TransportResponse::Rest(crate::synthesize_rest_response(
                        &MockResponseSynthesis::failure(provider, 401, "auth"),
                    ))),
                ),
                (
                    "rest_500_server",
                    Value::Response(TransportResponse::Rest(crate::synthesize_rest_response(
                        &MockResponseSynthesis::failure(provider, 500, "server"),
                    ))),
                ),
            ]
        }
        Value::Response(TransportResponse::File(_)) => {
            vec![(
                "file_not_found",
                Value::Response(TransportResponse::File(FileResponse {
                    path: "mock.txt".to_string(),
                    operation: FileOp::Read,
                    success: false,
                    content: None,
                    bytes: None,
                    exists: Some(false),
                    error: Some("No such file or directory".to_string()),
                })),
            )]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BoundaryMock;

    fn mock_spec_with_shell() -> MockSpec {
        MockSpec {
            name: "test".to_string(),
            boundary_mocks: vec![BoundaryMock {
                node: "execute_shell".to_string(),
                port: "response".to_string(),
                value: Value::Response(TransportResponse::Shell(ShellResponse::ok(
                    "ok".to_string(),
                ))),
                sequence: None,
            }],
            input_expectations: vec![],
            resource_mocks: Default::default(),
            transport_mocks: vec![],
            expected_outputs: vec![],
            live_expected_outputs: vec![],
            node_examples: vec![],
            skipped_node_examples: vec![],
            input_mocks: vec![],
        }
    }

    fn mock_spec_with_rest() -> MockSpec {
        MockSpec {
            name: "test".to_string(),
            boundary_mocks: vec![BoundaryMock {
                node: "github.gists.create".to_string(),
                port: "response".to_string(),
                value: Value::Response(TransportResponse::Rest(RestResponse::new(
                    200,
                    serde_json::json!({"ok": true}),
                ))),
                sequence: None,
            }],
            input_expectations: vec![],
            resource_mocks: Default::default(),
            transport_mocks: vec![],
            expected_outputs: vec![],
            live_expected_outputs: vec![],
            node_examples: vec![],
            skipped_node_examples: vec![],
            input_mocks: vec![],
        }
    }

    #[test]
    fn shell_failure_variant_has_exit_code_1() {
        let variants = auto_mock_failure_variants(&mock_spec_with_shell());
        assert_eq!(variants.len(), 1);
        assert!(variants[0].tag.contains("shell_exit_1"));
        match &variants[0].spec.boundary_mocks[0].value {
            Value::Response(TransportResponse::Shell(resp)) => {
                assert_eq!(resp.exit_code, 1);
            }
            other => panic!("expected shell response, got {:?}", other),
        }
    }

    #[test]
    fn rest_failure_variants_include_401_and_500() {
        let variants = auto_mock_failure_variants(&mock_spec_with_rest());
        assert_eq!(variants.len(), 2);
        let tags: Vec<&str> = variants.iter().map(|v| v.tag.as_str()).collect();
        assert!(tags.iter().any(|t| t.contains("rest_401_auth")));
        assert!(tags.iter().any(|t| t.contains("rest_500_server")));
        for v in &variants {
            match &v.spec.boundary_mocks[0].value {
                Value::Response(TransportResponse::Rest(resp)) => {
                    assert!(resp.status == 401 || resp.status == 500);
                }
                other => panic!("expected rest response, got {:?}", other),
            }
        }
    }

    #[test]
    fn non_transport_mocks_produce_no_variants() {
        let spec = MockSpec {
            name: "test".to_string(),
            boundary_mocks: vec![BoundaryMock {
                node: "some_node".to_string(),
                port: "output".to_string(),
                value: Value::Str("hello".to_string()),
                sequence: None,
            }],
            input_expectations: vec![],
            resource_mocks: Default::default(),
            transport_mocks: vec![],
            expected_outputs: vec![],
            live_expected_outputs: vec![],
            node_examples: vec![],
            skipped_node_examples: vec![],
            input_mocks: vec![],
        };
        assert!(auto_mock_failure_variants(&spec).is_empty());
    }

    #[test]
    fn failure_variants_clear_node_examples() {
        let mut spec = mock_spec_with_shell();
        spec.node_examples.push(crate::NodeExample::new("terminal"));
        let variants = auto_mock_failure_variants(&spec);
        for v in &variants {
            assert!(
                v.spec.node_examples.is_empty(),
                "failure variants should clear success-path node examples"
            );
        }
    }

    #[test]
    fn default_value_for_cloud_runtime_uses_variant_witness() {
        let registry = TypeRegistry::with_core_types();
        assert_eq!(
            default_value_for_type("CloudRuntime", &registry),
            Value::Str("GitHubActions".to_string())
        );
    }

    #[test]
    fn default_value_for_list_of_fermi_depth_uses_variant_elements() {
        let registry = TypeRegistry::with_core_types();
        assert_eq!(
            default_value_for_type("List<FermiDepth>", &registry),
            Value::List(vec![Value::Str("Xs".to_string())])
        );
    }

    #[test]
    fn default_value_for_platform_uses_first_variant() {
        let registry = TypeRegistry::with_core_types();
        assert_eq!(
            default_value_for_type("Platform", &registry),
            Value::Str("Linux".to_string())
        );
    }

    #[test]
    fn default_value_for_optional_prefers_absent() {
        let registry = TypeRegistry::with_core_types();
        assert_eq!(default_value_for_type("Optional<String>", &registry), Value::Unit);
        assert_eq!(default_value_for_type("String?", &registry), Value::Unit);
    }
}
