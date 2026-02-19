//! DAG-native system modeling primitives.
//!
//! This module models external systems/services as typed behavioral catalogs
//! that map directly onto `TypeId` / `Dag<TypeOp>` contracts.

use crate::dag::{Edge, Port};
use crate::node::Node;
use crate::port_type::PortType;
use crate::type_registry::TypeExprError;
use crate::Predicate;
use crate::{Dag, TypeId, TypeOp, TypeRegistry, WrapperKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub use inventory;

/// High-level system family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemKind {
    Cli,
    RestApi,
    LlmApi,
    Sdk,
    SecretProvider,
    StorageProvider,
    Transport,
    IdentityProvider,
}

/// Invocation style for a behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Invocation {
    Cli {
        command: String,
        docs: String,
    },
    Rest {
        method: String,
        path: String,
        docs: String,
    },
    Sdk {
        function: String,
        docs: String,
    },
    Protocol {
        protocol: String,
        docs: String,
    },
}

/// Behavior properties relevant to contract/test generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Property {
    ReadOnly,
    WritesWorld,
    Deterministic,
    Idempotent,
    IdempotentWithKey,
    FailsWhen,
    EdgeCase,
    Retryable,
    SecretScoped,
    PermissionScoped,
}

/// Input type mapping for behavior contracts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    /// Named type reference in registry.
    TypeId(TypeId),
    /// Explicit mapping to a type DAG via its registered `TypeId`.
    TypeDag(TypeId),
}

impl InputType {
    pub fn type_id(&self) -> &TypeId {
        match self {
            InputType::TypeId(id) | InputType::TypeDag(id) => id,
        }
    }

    pub fn resolve_dag(&self, registry: &TypeRegistry) -> Option<Dag<TypeOp>> {
        registry.resolve_type(self.type_id())
    }
}

/// Output type mapping for behavior contracts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputType {
    TypeId(TypeId),
    TypeDag(TypeId),
}

impl OutputType {
    pub fn type_id(&self) -> &TypeId {
        match self {
            OutputType::TypeId(id) | OutputType::TypeDag(id) => id,
        }
    }

    pub fn resolve_dag(&self, registry: &TypeRegistry) -> Option<Dag<TypeOp>> {
        registry.resolve_type(self.type_id())
    }
}

/// Input spec for a behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorInput {
    pub name: String,
    pub input_type: InputType,
    pub required: bool,
}

impl BehaviorInput {
    pub fn required(name: impl Into<String>, input_type: InputType) -> Self {
        Self {
            name: name.into(),
            input_type,
            required: true,
        }
    }

    pub fn optional(name: impl Into<String>, input_type: InputType) -> Self {
        Self {
            name: name.into(),
            input_type,
            required: false,
        }
    }
}

/// Output spec for a behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorOutput {
    pub name: String,
    pub output_type: OutputType,
}

impl BehaviorOutput {
    pub fn new(name: impl Into<String>, output_type: OutputType) -> Self {
        Self {
            name: name.into(),
            output_type,
        }
    }
}

/// Typed behavior contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Behavior {
    pub id: String,
    pub description: String,
    pub invocation: Invocation,
    pub inputs: Vec<BehaviorInput>,
    pub outputs: Vec<BehaviorOutput>,
    pub properties: Vec<Property>,
}

impl Behavior {
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        invocation: Invocation,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            invocation,
            inputs: Vec::new(),
            outputs: Vec::new(),
            properties: Vec::new(),
        }
    }

    pub fn with_inputs(mut self, inputs: Vec<BehaviorInput>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<BehaviorOutput>) -> Self {
        self.outputs = outputs;
        self
    }

    pub fn with_properties(mut self, properties: &[Property]) -> Self {
        self.properties = properties.to_vec();
        self
    }
}

/// Dependency kind for system models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    /// Depends on another system model id.
    System(String),
    /// Depends on an external secret/resource.
    Secret(String),
}

/// A dependency edge from one system model to another system/resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub kind: DependencyKind,
}

impl Dependency {
    pub fn system(id: impl Into<String>) -> Self {
        Self {
            kind: DependencyKind::System(id.into()),
        }
    }

    pub fn secret(id: impl Into<String>) -> Self {
        Self {
            kind: DependencyKind::Secret(id.into()),
        }
    }
}

/// Top-level DAG-native system model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemModel {
    pub id: String,
    pub name: String,
    pub kind: SystemKind,
    pub version: String,
    pub docs: String,
    pub behaviors: Vec<Behavior>,
    pub dependencies: Vec<Dependency>,
}

/// Upsert-oriented lifecycle phases used for contract tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpsertPhase {
    Check,
    Create,
    Resolve,
}

/// Contract-test specification derived from a behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractTestSpec {
    pub system_id: String,
    pub behavior_id: String,
    pub phase: UpsertPhase,
    pub required_all: Vec<Property>,
    pub required_any: Vec<Property>,
    pub inputs: Vec<BehaviorInput>,
    pub outputs: Vec<BehaviorOutput>,
}

impl ContractTestSpec {
    pub fn id(&self) -> String {
        format!("{}::{}::{:?}", self.system_id, self.behavior_id, self.phase)
    }
}

impl SystemModel {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: SystemKind,
        version: impl Into<String>,
        docs: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            version: version.into(),
            docs: docs.into(),
            behaviors: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_behaviors(mut self, behaviors: Vec<Behavior>) -> Self {
        self.behaviors = behaviors;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<Dependency>) -> Self {
        self.dependencies = dependencies;
        self
    }
}

/// Inventory registration entry.
#[derive(Debug)]
pub struct SystemModelDef {
    pub build: fn() -> SystemModel,
}

inventory::collect!(SystemModelDef);

/// Submit a system-model builder function into inventory.
#[macro_export]
macro_rules! submit_system_model {
    ($builder:path) => {
        $crate::system_model::inventory::submit! {
            $crate::system_model::SystemModelDef { build: $builder }
        }
    };
}

/// Iterate over all registered system models.
pub fn iter_registered_system_models() -> impl Iterator<Item = SystemModel> {
    inventory::iter::<SystemModelDef>
        .into_iter()
        .map(|def| (def.build)())
}

/// Collect registered models by id (last writer wins on duplicate ids).
fn registered_system_model_map() -> BTreeMap<String, SystemModel> {
    let mut map = BTreeMap::new();
    for model in iter_registered_system_models() {
        map.insert(model.id.clone(), model);
    }
    map
}

/// Get one registered model by id.
pub fn get_registered_system_model(id: &str) -> Option<SystemModel> {
    registered_system_model_map().remove(id)
}

/// Validate one system model for basic consistency.
pub fn validate_system_model(model: &SystemModel) -> Result<(), String> {
    if model.id.trim().is_empty() {
        return Err("system model id must not be empty".to_string());
    }
    if model.behaviors.is_empty() {
        return Err(format!("system model '{}' has no behaviors", model.id));
    }
    let mut behavior_ids = BTreeSet::new();
    for behavior in &model.behaviors {
        if behavior.id.trim().is_empty() {
            return Err(format!(
                "system model '{}' has behavior with empty id",
                model.id
            ));
        }
        if !behavior_ids.insert(behavior.id.clone()) {
            return Err(format!(
                "system model '{}' has duplicate behavior id '{}'",
                model.id, behavior.id
            ));
        }
        if behavior.outputs.is_empty() {
            return Err(format!(
                "system model '{}.{}' must declare at least one output",
                model.id, behavior.id
            ));
        }

        if let Invocation::Rest { path, .. } = &behavior.invocation {
            if path.contains('*') {
                return Err(format!(
                    "system model '{}.{}' REST path '{}' uses wildcard '*' segments; use named placeholders like '{{project_id}}'",
                    model.id, behavior.id, path
                ));
            }

            let placeholders = rest_path_placeholders(path).map_err(|error| {
                format!(
                    "system model '{}.{}' has invalid REST path '{}': {}",
                    model.id, behavior.id, path, error
                )
            })?;

            for placeholder in placeholders {
                let Some(input) = behavior
                    .inputs
                    .iter()
                    .find(|input| input.name == placeholder)
                else {
                    return Err(format!(
                        "system model '{}.{}' REST path placeholder '{}' has no matching behavior input",
                        model.id, behavior.id, placeholder
                    ));
                };
                if !input.required {
                    return Err(format!(
                        "system model '{}.{}' REST path placeholder '{}' must map to a required input",
                        model.id, behavior.id, placeholder
                    ));
                }
            }
        }
    }
    Ok(())
}

fn rest_path_placeholders(path: &str) -> Result<BTreeSet<String>, String> {
    let mut placeholders = BTreeSet::new();
    let mut i = 0usize;
    let bytes = path.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let Some(end) = path[i + 1..].find('}') else {
                return Err("missing closing '}' for placeholder".to_string());
            };
            let end_index = i + 1 + end;
            let raw_name = &path[i + 1..end_index];
            if raw_name.is_empty() {
                return Err("empty placeholder '{}' is not allowed".to_string());
            }
            if !raw_name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                return Err(format!(
                    "invalid placeholder '{raw_name}' (expected [A-Za-z0-9_]+)"
                ));
            }
            placeholders.insert(raw_name.to_string());
            i = end_index + 1;
            continue;
        }
        if bytes[i] == b'}' {
            return Err("unmatched closing '}' in path".to_string());
        }
        i += 1;
    }
    Ok(placeholders)
}

/// Ensure the system dependency graph is acyclic.
pub fn validate_dependency_graph_acyclic(models: &[SystemModel]) -> Result<(), String> {
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut outgoing = BTreeMap::<String, Vec<String>>::new();

    for model in models {
        indegree.entry(model.id.clone()).or_insert(0);
        outgoing.entry(model.id.clone()).or_default();
    }

    for model in models {
        for dep in &model.dependencies {
            if let DependencyKind::System(target) = &dep.kind {
                if indegree.contains_key(target) {
                    *indegree.get_mut(target).expect("target indegree exists") += 1;
                    outgoing
                        .get_mut(&model.id)
                        .expect("source entry exists")
                        .push(target.clone());
                }
            }
        }
    }

    let mut queue: VecDeque<String> = indegree
        .iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(id.clone()))
        .collect();
    let mut visited = 0usize;

    while let Some(id) = queue.pop_front() {
        visited += 1;
        if let Some(targets) = outgoing.get(&id) {
            for target in targets {
                let degree = indegree
                    .get_mut(target)
                    .expect("target indegree should exist");
                *degree = degree.saturating_sub(1);
                if *degree == 0 {
                    queue.push_back(target.clone());
                }
            }
        }
    }

    if visited != indegree.len() {
        return Err("system model dependency graph contains a cycle".to_string());
    }

    Ok(())
}

/// Derive contract-test specs from system models and behavior properties.
pub fn derive_contract_test_specs(models: &[SystemModel]) -> Vec<ContractTestSpec> {
    let mut behavior_type_registry = TypeRegistry::with_core_types();
    let registry_ready =
        register_system_behavior_type_dags(&mut behavior_type_registry, models).is_ok();

    let mut specs = Vec::new();
    for model in models {
        for behavior in &model.behaviors {
            let property_markers = if registry_ready {
                let type_id = system_behavior_type_id(&model.id, &behavior.id);
                behavior_properties_from_type_dag(&behavior_type_registry, &type_id)
                    .unwrap_or_else(|| behavior.properties.clone())
            } else {
                behavior.properties.clone()
            };

            let has = |property: Property| property_markers.contains(&property);

            if has(Property::ReadOnly) && has(Property::Deterministic) {
                specs.push(ContractTestSpec {
                    system_id: model.id.clone(),
                    behavior_id: behavior.id.clone(),
                    phase: UpsertPhase::Check,
                    required_all: vec![Property::ReadOnly, Property::Deterministic],
                    required_any: Vec::new(),
                    inputs: behavior.inputs.clone(),
                    outputs: behavior.outputs.clone(),
                });
            }

            if has(Property::WritesWorld)
                && (has(Property::Idempotent) || has(Property::IdempotentWithKey))
            {
                specs.push(ContractTestSpec {
                    system_id: model.id.clone(),
                    behavior_id: behavior.id.clone(),
                    phase: UpsertPhase::Create,
                    required_all: vec![Property::WritesWorld],
                    required_any: vec![Property::Idempotent, Property::IdempotentWithKey],
                    inputs: behavior.inputs.clone(),
                    outputs: behavior.outputs.clone(),
                });
            }

            if has(Property::ReadOnly) && has(Property::FailsWhen) {
                specs.push(ContractTestSpec {
                    system_id: model.id.clone(),
                    behavior_id: behavior.id.clone(),
                    phase: UpsertPhase::Resolve,
                    required_all: vec![Property::ReadOnly, Property::FailsWhen],
                    required_any: Vec::new(),
                    inputs: behavior.inputs.clone(),
                    outputs: behavior.outputs.clone(),
                });
            }
        }
    }
    specs
}

fn behavior_properties_from_type_dag(
    registry: &TypeRegistry,
    type_id: &TypeId,
) -> Option<Vec<Property>> {
    let dag = registry.get(type_id)?;
    let mut properties = Vec::new();
    for node in &dag.nodes {
        if let crate::node::NodeBody::Opaque(TypeOp::Validate(Predicate::Custom(marker))) =
            &node.body
        {
            if let Some(raw) = marker.strip_prefix("property:") {
                if let Some(property) = parse_property_marker(raw) {
                    if !properties.contains(&property) {
                        properties.push(property);
                    }
                }
            }
        }
    }
    Some(properties)
}

fn parse_property_marker(raw: &str) -> Option<Property> {
    match raw {
        "ReadOnly" => Some(Property::ReadOnly),
        "WritesWorld" => Some(Property::WritesWorld),
        "Deterministic" => Some(Property::Deterministic),
        "Idempotent" => Some(Property::Idempotent),
        "IdempotentWithKey" => Some(Property::IdempotentWithKey),
        "FailsWhen" => Some(Property::FailsWhen),
        "EdgeCase" => Some(Property::EdgeCase),
        "Retryable" => Some(Property::Retryable),
        "SecretScoped" => Some(Property::SecretScoped),
        "PermissionScoped" => Some(Property::PermissionScoped),
        _ => None,
    }
}

/// Canonical type id used to register a behavior contract DAG.
///
/// Format: `System::<system_id>::Behavior::<behavior_id>`.
pub fn system_behavior_type_id(system_id: &str, behavior_id: &str) -> TypeId {
    TypeId::new(format!(
        "System::{}::Behavior::{}",
        sanitize_ident(system_id),
        sanitize_ident(behavior_id)
    ))
}

/// Register per-behavior contract DAGs for all provided system models.
///
/// Each behavior is materialized as a deterministic `Dag<TypeOp>` descriptor
/// and registered into `TypeRegistry` under [`system_behavior_type_id`].
///
/// The descriptor encodes:
/// - system + behavior metadata as `Validate(Custom(...))` nodes
/// - behavior properties as `Validate(Custom("property:<...>"))` nodes
/// - input/output contracts as `Validate(Custom(...))` nodes
/// - optional inputs as explicit `TypeOp::Wrap(WrapperKind::Optional)` nodes
///
/// All referenced input/output `TypeId`s are validated against the current
/// registry before registration.
pub fn register_system_behavior_type_dags(
    registry: &mut TypeRegistry,
    models: &[SystemModel],
) -> Result<Vec<TypeId>, String> {
    let mut planned = Vec::new();
    for model in models {
        for behavior in &model.behaviors {
            let type_id = system_behavior_type_id(&model.id, &behavior.id);
            let dag = build_behavior_contract_dag(model, behavior, registry)?;
            planned.push((type_id, dag));
        }
    }

    let mut registered = Vec::with_capacity(planned.len());
    for (type_id, dag) in planned {
        registry.register(type_id.clone(), dag);
        registered.push(type_id);
    }
    Ok(registered)
}

fn build_behavior_contract_dag(
    model: &SystemModel,
    behavior: &Behavior,
    registry: &TypeRegistry,
) -> Result<Dag<TypeOp>, String> {
    let mut dag = Dag::new();
    dag.add_node(Node::opaque(
        "behavior_input",
        vec![Port::scalar("in", "Json")],
        vec![Port::scalar("out", "Json")],
        TypeOp::Identity,
    ));

    let mut prev = "behavior_input".to_string();
    let mut idx = 0usize;

    append_validate_step(
        &mut dag,
        &mut prev,
        &mut idx,
        format!("meta:system_id={}", model.id),
    );
    append_validate_step(
        &mut dag,
        &mut prev,
        &mut idx,
        format!("meta:system_kind={:?}", model.kind),
    );
    append_validate_step(
        &mut dag,
        &mut prev,
        &mut idx,
        format!("meta:behavior_id={}", behavior.id),
    );
    append_validate_step(
        &mut dag,
        &mut prev,
        &mut idx,
        format!("meta:invocation={}", invocation_tag(&behavior.invocation)),
    );

    for property in &behavior.properties {
        append_validate_step(
            &mut dag,
            &mut prev,
            &mut idx,
            format!("property:{property:?}"),
        );
    }

    for input in &behavior.inputs {
        validate_type_ref("input", &input.name, input.input_type.type_id(), registry)?;
        append_validate_step(
            &mut dag,
            &mut prev,
            &mut idx,
            format!(
                "input:{}:{}:required={}",
                sanitize_ident(&input.name),
                input.input_type.type_id().0,
                input.required
            ),
        );
        if !input.required {
            let wrap_node_id = format!("step_{idx}_optional_wrap");
            dag.add_node(Node::opaque(
                wrap_node_id.as_str(),
                vec![Port::scalar("in", "Json")],
                vec![Port::scalar("out", "Json")],
                TypeOp::Wrap(WrapperKind::Optional),
            ));
            dag.add_edge(Edge::new(prev.as_str(), "out", wrap_node_id.as_str(), "in"));
            prev = wrap_node_id;
            idx += 1;
        }
    }

    for output in &behavior.outputs {
        validate_type_ref(
            "output",
            &output.name,
            output.output_type.type_id(),
            registry,
        )?;
        append_validate_step(
            &mut dag,
            &mut prev,
            &mut idx,
            format!(
                "output:{}:{}",
                sanitize_ident(&output.name),
                output.output_type.type_id().0
            ),
        );
    }

    dag.add_node(Node::opaque(
        "behavior_output",
        vec![Port::scalar("in", "Json")],
        vec![Port::scalar("out", "Json")],
        TypeOp::Identity,
    ));
    dag.add_edge(Edge::new(prev.as_str(), "out", "behavior_output", "in"));
    Ok(dag)
}

fn append_validate_step(dag: &mut Dag<TypeOp>, prev: &mut String, idx: &mut usize, marker: String) {
    let node_id = format!("step_{}_{}", idx, sanitize_ident(&marker));
    dag.add_node(Node::opaque(
        node_id.as_str(),
        vec![Port::scalar("in", "Json")],
        vec![Port::scalar("out", "Json")],
        TypeOp::Validate(Predicate::Custom(marker)),
    ));
    dag.add_edge(Edge::new(prev.as_str(), "out", node_id.as_str(), "in"));
    *prev = node_id;
    *idx += 1;
}

fn validate_type_ref(
    role: &str,
    field_name: &str,
    type_id: &TypeId,
    registry: &TypeRegistry,
) -> Result<(), String> {
    match registry.resolve_type_checked(type_id) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(format!(
            "unregistered {role} type `{}` for `{}`",
            type_id.0, field_name
        )),
        Err(err) => Err(format!(
            "invalid {role} type expression `{}` for `{}`: {}",
            type_id.0,
            field_name,
            render_type_expr_error(&err)
        )),
    }
}

fn render_type_expr_error(error: &TypeExprError) -> String {
    error.to_string()
}

fn invocation_tag(invocation: &Invocation) -> String {
    match invocation {
        Invocation::Cli { command, .. } => format!("cli:{command}"),
        Invocation::Rest { method, path, .. } => format!("rest:{method}:{path}"),
        Invocation::Sdk { function, .. } => format!("sdk:{function}"),
        Invocation::Protocol { protocol, .. } => format!("protocol:{protocol}"),
    }
}

fn rust_type_for_port_type(port_type: &PortType, original_type_id: &TypeId) -> String {
    match port_type {
        PortType::String => "String".to_string(),
        PortType::Bool => "bool".to_string(),
        PortType::Int => "i64".to_string(),
        PortType::Float => "f64".to_string(),
        PortType::Bytes => "Vec<u8>".to_string(),
        PortType::Json => "serde_json::Value".to_string(),
        PortType::Secret => "String".to_string(),
        PortType::List(inner) => {
            let inner_type =
                rust_type_for_port_type(inner, &TypeId::new(inner.to_type_id().0.clone()));
            format!("Vec<{inner_type}>")
        }
        PortType::Any => {
            // Domain-specific types with known Rust paths in gunbc_ir::transport.
            match original_type_id.0.as_str() {
                "FileResponse" => "gunbc_ir::transport::FileResponse".to_string(),
                "ShellResponse" => "gunbc_ir::transport::ShellResponse".to_string(),
                "RestResponse" => "gunbc_ir::transport::RestResponse".to_string(),
                "HttpResponse" => "gunbc_ir::transport::HttpResponse".to_string(),
                _ => "gunbc_ir::Value".to_string(),
            }
        }
    }
}

fn sanitize_ident(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

/// Render a Rust harness signature for one contract-test spec.
pub fn render_contract_test_harness(spec: &ContractTestSpec) -> String {
    let fn_name = format!(
        "contract_{}_{}_{}",
        sanitize_ident(&spec.system_id),
        sanitize_ident(&spec.behavior_id),
        format!("{:?}", spec.phase).to_lowercase()
    );

    let args = spec
        .inputs
        .iter()
        .map(|input| {
            let type_id = input.input_type.type_id();
            let port_type = PortType::from(type_id);
            format!(
                "{}: {}",
                sanitize_ident(&input.name),
                rust_type_for_port_type(&port_type, type_id)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = if spec.outputs.len() == 1 {
        let type_id = spec.outputs[0].output_type.type_id();
        let port_type = PortType::from(type_id);
        rust_type_for_port_type(&port_type, type_id).to_string()
    } else {
        format!(
            "({})",
            spec.outputs
                .iter()
                .map(|out| {
                    let type_id = out.output_type.type_id();
                    let port_type = PortType::from(type_id);
                    rust_type_for_port_type(&port_type, type_id)
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    format!("fn {fn_name}({args}) -> {return_type} {{ unimplemented!(\"generated contract harness\") }}")
}

/// Render Rust harness signatures for all specs.
pub fn generate_contract_test_harnesses(specs: &[ContractTestSpec]) -> Vec<String> {
    specs.iter().map(render_contract_test_harness).collect()
}

/// Validate that provider models support the storage abstraction behavior set.
pub fn validate_store_behavior_mapping(models: &[SystemModel]) -> Result<(), String> {
    let required: BTreeSet<&str> =
        BTreeSet::from(["get_object", "put_object", "list_objects", "delete_object"]);
    let gcp = models
        .iter()
        .find(|m| m.id == "gcp.gcs")
        .ok_or_else(|| "missing storage provider model 'gcp.gcs'".to_string())?;
    let aws = models
        .iter()
        .find(|m| m.id == "aws.s3")
        .ok_or_else(|| "missing storage provider model 'aws.s3'".to_string())?;

    let gcp_ops: BTreeSet<&str> = gcp.behaviors.iter().map(|b| b.id.as_str()).collect();
    let aws_ops: BTreeSet<&str> = aws.behaviors.iter().map(|b| b.id.as_str()).collect();
    if !required.is_subset(&gcp_ops) {
        return Err(format!(
            "storage provider '{}' missing required store operations: {:?}",
            gcp.id,
            required.difference(&gcp_ops).copied().collect::<Vec<_>>()
        ));
    }
    if !required.is_subset(&aws_ops) {
        return Err(format!(
            "storage provider '{}' missing required store operations: {:?}",
            aws.id,
            required.difference(&aws_ops).copied().collect::<Vec<_>>()
        ));
    }

    let mut registry = TypeRegistry::with_core_types();
    register_system_behavior_type_dags(&mut registry, &[gcp.clone(), aws.clone()])?;

    for behavior_id in &required {
        let gcp_type = system_behavior_type_id(&gcp.id, behavior_id);
        let aws_type = system_behavior_type_id(&aws.id, behavior_id);
        let gcp_shape = behavior_contract_shape(&registry, &gcp_type).ok_or_else(|| {
            format!(
                "missing behavior DAG for '{}.{}' in registry",
                gcp.id, behavior_id
            )
        })?;
        let aws_shape = behavior_contract_shape(&registry, &aws_type).ok_or_else(|| {
            format!(
                "missing behavior DAG for '{}.{}' in registry",
                aws.id, behavior_id
            )
        })?;
        if gcp_shape != aws_shape {
            return Err(format!(
                "storage behavior contract mismatch for '{}': gcp={:?} aws={:?}",
                behavior_id, gcp_shape, aws_shape
            ));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BehaviorContractShape {
    properties: Vec<String>,
    inputs: Vec<(String, String, bool)>,
    outputs: Vec<(String, String)>,
    optional_wrap_count: usize,
}

fn behavior_contract_shape(
    registry: &TypeRegistry,
    type_id: &TypeId,
) -> Option<BehaviorContractShape> {
    let dag = registry.get(type_id)?;
    let mut properties = Vec::new();
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut optional_wrap_count = 0usize;

    for node in &dag.nodes {
        if let crate::node::NodeBody::Opaque(op) = &node.body {
            match op {
                TypeOp::Validate(Predicate::Custom(marker)) => {
                    if let Some(raw) = marker.strip_prefix("property:") {
                        properties.push(raw.to_string());
                    } else if let Some(parsed) = parse_input_marker(marker) {
                        inputs.push(parsed);
                    } else if let Some(parsed) = parse_output_marker(marker) {
                        outputs.push(parsed);
                    }
                }
                TypeOp::Wrap(WrapperKind::Optional) => {
                    optional_wrap_count += 1;
                }
                _ => {}
            }
        }
    }

    properties.sort();
    properties.dedup();
    inputs.sort();
    inputs.dedup();
    outputs.sort();
    outputs.dedup();

    Some(BehaviorContractShape {
        properties,
        inputs,
        outputs,
        optional_wrap_count,
    })
}

fn parse_input_marker(marker: &str) -> Option<(String, String, bool)> {
    let marker = marker.strip_prefix("input:")?;
    let (left, required_raw) = marker.rsplit_once(":required=")?;
    let required = match required_raw {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    let (name, type_id) = left.split_once(':')?;
    Some((name.to_string(), type_id.to_string(), required))
}

fn parse_output_marker(marker: &str) -> Option<(String, String)> {
    let marker = marker.strip_prefix("output:")?;
    let (name, type_id) = marker.split_once(':')?;
    Some((name.to_string(), type_id.to_string()))
}

/// Built-in system models discovered via inventory registration.
///
/// Each owning crate (gcp-ops, aws-ops, transport) registers its models via
/// `submit_system_model!`. This function collects them all. Consumers must
/// depend on the registering crates for the linker to include inventory symbols.
pub fn default_system_models() -> Vec<SystemModel> {
    iter_registered_system_models().collect()
}

// Model data distributed to owning crates:
// - lib/gcp-ops/src/system_models.rs (gcp.secret_manager, gcp.iam, gcp.gcs)
// - lib/aws-ops/src/system_models.rs (aws.secrets_manager, aws.iam, aws.s3)
// - lib/transport/src/system_models.rs (transport.file, transport.shell, transport.http_rest)

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_model_for_registry() -> SystemModel {
        SystemModel::new(
            "test.minimal_model",
            "Test Minimal Model",
            SystemKind::Sdk,
            "v0",
            "unit-test model",
        )
        .with_behaviors(vec![Behavior::new(
            "ping",
            "Ping behavior",
            Invocation::Sdk {
                function: "ping".to_string(),
                docs: "test".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required(
            "input",
            InputType::TypeId(TypeId::from("String")),
        )])
        .with_outputs(vec![BehaviorOutput::new(
            "output",
            OutputType::TypeId(TypeId::from("String")),
        )])
        .with_properties(&[Property::Deterministic, Property::ReadOnly])])
    }

    submit_system_model!(minimal_model_for_registry);

    #[test]
    fn registered_model_can_be_retrieved_by_id() {
        let model = get_registered_system_model("test.minimal_model")
            .expect("registered model should be discoverable");
        assert_eq!(model.name, "Test Minimal Model");
        assert_eq!(model.behaviors.len(), 1);
    }

    #[test]
    fn default_models_are_parseable_and_acyclic() {
        let models = default_system_models();
        for model in &models {
            validate_system_model(model).expect("default model should validate");
            let json = serde_json::to_string(model).expect("serialize model");
            let parsed: SystemModel = serde_json::from_str(&json).expect("parse model");
            assert_eq!(parsed.id, model.id);
        }
        validate_dependency_graph_acyclic(&models)
            .expect("default system model dependencies must be acyclic");
    }

    #[test]
    fn register_behavior_type_dags_adds_registry_entries() {
        let model = SystemModel::new(
            "provider.alpha",
            "Provider Alpha",
            SystemKind::Sdk,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "fetch_item",
            "Fetch one item",
            Invocation::Sdk {
                function: "fetch_item".to_string(),
                docs: "fetches item".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("id", InputType::TypeId(TypeId::from("String"))),
            BehaviorInput::optional("limit", InputType::TypeId(TypeId::from("Int"))),
        ])
        .with_outputs(vec![BehaviorOutput::new(
            "value",
            OutputType::TypeId(TypeId::from("Json")),
        )])
        .with_properties(&[Property::ReadOnly, Property::Deterministic])]);

        let mut registry = TypeRegistry::with_core_types();
        let registered = register_system_behavior_type_dags(&mut registry, &[model])
            .expect("behavior type DAG registration should succeed");

        assert_eq!(registered.len(), 1);
        let behavior_type = system_behavior_type_id("provider.alpha", "fetch_item");
        assert_eq!(registered[0], behavior_type);

        let dag = registry
            .get(&behavior_type)
            .expect("registered behavior type should be present");
        assert!(
            dag.nodes.iter().any(|node| matches!(
                node.body,
                crate::node::NodeBody::Opaque(TypeOp::Wrap(WrapperKind::Optional))
            )),
            "optional input should produce a WrapperKind::Optional node"
        );
    }

    #[test]
    fn register_behavior_type_dags_rejects_unknown_input_type() {
        let model = SystemModel::new(
            "provider.beta",
            "Provider Beta",
            SystemKind::Sdk,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "compute",
            "Compute result",
            Invocation::Sdk {
                function: "compute".to_string(),
                docs: "computes value".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required(
            "payload",
            InputType::TypeId(TypeId::from("NotRegisteredType")),
        )])
        .with_outputs(vec![BehaviorOutput::new(
            "ok",
            OutputType::TypeId(TypeId::from("Bool")),
        )])]);

        let mut registry = TypeRegistry::with_core_types();
        let err = register_system_behavior_type_dags(&mut registry, &[model])
            .expect_err("unregistered input types should fail");
        assert!(err.contains("NotRegisteredType"), "unexpected error: {err}");
    }

    #[test]
    fn derive_contract_specs_uses_property_markers_from_behavior_type_dag() {
        let model = SystemModel::new(
            "provider.gamma",
            "Provider Gamma",
            SystemKind::Sdk,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "lookup",
            "Lookup item",
            Invocation::Sdk {
                function: "lookup".to_string(),
                docs: "lookup docs".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required(
            "id",
            InputType::TypeId(TypeId::from("String")),
        )])
        .with_outputs(vec![BehaviorOutput::new(
            "value",
            OutputType::TypeId(TypeId::from("Json")),
        )])
        .with_properties(&[Property::ReadOnly, Property::Deterministic])]);

        let mut registry = TypeRegistry::with_core_types();
        register_system_behavior_type_dags(&mut registry, std::slice::from_ref(&model))
            .expect("registration should succeed");

        let type_id = system_behavior_type_id("provider.gamma", "lookup");
        let properties = behavior_properties_from_type_dag(&registry, &type_id)
            .expect("type DAG should be present");
        assert!(properties.contains(&Property::ReadOnly));
        assert!(properties.contains(&Property::Deterministic));

        let specs = derive_contract_test_specs(&[model]);
        assert!(specs.iter().any(|spec| {
            spec.behavior_id == "lookup"
                && spec.phase == UpsertPhase::Check
                && spec.required_all.contains(&Property::ReadOnly)
                && spec.required_all.contains(&Property::Deterministic)
        }));
    }

    #[test]
    fn validate_store_behavior_mapping_accepts_structurally_equivalent_models() {
        let mk_behavior = |id: &str| {
            Behavior::new(
                id,
                format!("{id} behavior"),
                Invocation::Sdk {
                    function: id.to_string(),
                    docs: "docs".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required(
                "key",
                InputType::TypeId(TypeId::from("String")),
            )])
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from("Bool")),
            )])
            .with_properties(&[Property::ReadOnly, Property::Deterministic])
        };

        let gcp = SystemModel::new("gcp.gcs", "GCS", SystemKind::StorageProvider, "v1", "gcp")
            .with_behaviors(vec![
                mk_behavior("get_object"),
                mk_behavior("put_object"),
                mk_behavior("list_objects"),
                mk_behavior("delete_object"),
            ]);
        let aws = SystemModel::new("aws.s3", "S3", SystemKind::StorageProvider, "v1", "aws")
            .with_behaviors(vec![
                mk_behavior("get_object"),
                mk_behavior("put_object"),
                mk_behavior("list_objects"),
                mk_behavior("delete_object"),
            ]);

        validate_store_behavior_mapping(&[gcp, aws])
            .expect("equivalent provider contracts should validate");
    }

    #[test]
    fn validate_store_behavior_mapping_rejects_structural_mismatch() {
        let mk_behavior = |id: &str, output_type: &str| {
            Behavior::new(
                id,
                format!("{id} behavior"),
                Invocation::Sdk {
                    function: id.to_string(),
                    docs: "docs".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required(
                "key",
                InputType::TypeId(TypeId::from("String")),
            )])
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from(output_type)),
            )])
            .with_properties(&[Property::ReadOnly, Property::Deterministic])
        };

        let gcp = SystemModel::new("gcp.gcs", "GCS", SystemKind::StorageProvider, "v1", "gcp")
            .with_behaviors(vec![
                mk_behavior("get_object", "Bool"),
                mk_behavior("put_object", "Bool"),
                mk_behavior("list_objects", "Bool"),
                mk_behavior("delete_object", "Bool"),
            ]);
        let aws = SystemModel::new("aws.s3", "S3", SystemKind::StorageProvider, "v1", "aws")
            .with_behaviors(vec![
                mk_behavior("get_object", "Json"), // mismatch on purpose
                mk_behavior("put_object", "Bool"),
                mk_behavior("list_objects", "Bool"),
                mk_behavior("delete_object", "Bool"),
            ]);

        let err = validate_store_behavior_mapping(&[gcp, aws])
            .expect_err("mismatched contracts should fail validation");
        assert!(
            err.contains("mismatch for 'get_object'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_system_model_rejects_rest_wildcard_paths() {
        let model = SystemModel::new(
            "provider.rest",
            "Provider Rest",
            SystemKind::RestApi,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "get_item",
            "Get item",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/v1/projects/*/items/{item_id}".to_string(),
                docs: "test".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", InputType::TypeId(TypeId::from("String"))),
            BehaviorInput::required("item_id", InputType::TypeId(TypeId::from("String"))),
        ])
        .with_outputs(vec![BehaviorOutput::new(
            "item",
            OutputType::TypeId(TypeId::from("Json")),
        )])]);

        let err = validate_system_model(&model).expect_err("wildcard path should fail");
        assert!(err.contains("wildcard"), "unexpected error: {err}");
    }

    #[test]
    fn validate_system_model_rejects_unbound_rest_placeholders() {
        let model = SystemModel::new(
            "provider.rest",
            "Provider Rest",
            SystemKind::RestApi,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "get_item",
            "Get item",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/v1/projects/{project_id}/items/{item_id}".to_string(),
                docs: "test".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required(
            "project_id",
            InputType::TypeId(TypeId::from("String")),
        )])
        .with_outputs(vec![BehaviorOutput::new(
            "item",
            OutputType::TypeId(TypeId::from("Json")),
        )])]);

        let err = validate_system_model(&model).expect_err("missing path input should fail");
        assert!(err.contains("item_id"), "unexpected error: {err}");
    }

    #[test]
    fn validate_system_model_rejects_optional_path_placeholders() {
        let model = SystemModel::new(
            "provider.rest",
            "Provider Rest",
            SystemKind::RestApi,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "get_item",
            "Get item",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/v1/projects/{project_id}/items/{item_id}".to_string(),
                docs: "test".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", InputType::TypeId(TypeId::from("String"))),
            BehaviorInput::optional("item_id", InputType::TypeId(TypeId::from("String"))),
        ])
        .with_outputs(vec![BehaviorOutput::new(
            "item",
            OutputType::TypeId(TypeId::from("Json")),
        )])]);

        let err = validate_system_model(&model).expect_err("optional path input should fail");
        assert!(err.contains("required input"), "unexpected error: {err}");
    }

    #[test]
    fn validate_system_model_accepts_bound_rest_placeholders() {
        let model = SystemModel::new(
            "provider.rest",
            "Provider Rest",
            SystemKind::RestApi,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "get_item",
            "Get item",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/v1/projects/{project_id}/items/{item_id}".to_string(),
                docs: "test".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", InputType::TypeId(TypeId::from("String"))),
            BehaviorInput::required("item_id", InputType::TypeId(TypeId::from("String"))),
            BehaviorInput::optional("verbose", InputType::TypeId(TypeId::from("Bool"))),
        ])
        .with_outputs(vec![BehaviorOutput::new(
            "item",
            OutputType::TypeId(TypeId::from("Json")),
        )])]);

        validate_system_model(&model).expect("placeholder-bound REST model should validate");
    }

    // Model-specific behavior tests (GCP, AWS, transport) moved to owning crates:
    // - lib/gcp-ops/src/system_models.rs
    // - lib/aws-ops/src/system_models.rs
    // - lib/transport/src/system_models.rs
    // Cross-cutting tests (contract specs, store mapping) moved to gunbc-dag.
}
