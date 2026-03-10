//! DAG-native system modeling primitives.
//!
//! This module models external systems/services as typed behavioral catalogs
//! that map directly onto `TypeId` / `Dag<TypeOp>` contracts.

use crate::dag::{Edge, Port};
use crate::node::Node;
use crate::type_registry::TypeExprError;
use crate::types::{parse_unary_generic_type_id, ValueBacking};
use crate::InvocationContract;
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
    /// Convention-level system (e.g., TCP socket layer).
    Convention,
    /// Protocol-level system (e.g., HTTP on top of TCP).
    Protocol,
}

/// Invocation style for a behavior.
pub type Invocation = InvocationContract;

/// Behavior properties relevant to contract/test generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    /// Declares that the behavior requires JSON content-type encoding.
    JsonContentType,
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
    System(SystemDependencyId),
    /// Depends on an external secret/resource.
    Secret(SecretDependencyId),
}

/// Typed system-dependency identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SystemDependencyId(pub String);

impl SystemDependencyId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for SystemDependencyId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SystemDependencyId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Typed secret-dependency identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SecretDependencyId(pub String);

impl SecretDependencyId {
    pub fn new(value: impl Into<String>) -> Self {
        let v = value.into();
        debug_assert!(
            !v.starts_with("secret:"),
            "SecretDependencyId should contain the bare name, not the 'secret:' prefix: {v}"
        );
        Self(v)
    }

    /// Create a secret dependency ID for an environment variable.
    pub fn env_var(name: impl Into<String>) -> Self {
        Self::new(name)
    }
}

impl From<&str> for SecretDependencyId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SecretDependencyId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A dependency edge from one system model to another system/resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub kind: DependencyKind,
}

impl Dependency {
    pub fn system(id: impl Into<SystemDependencyId>) -> Self {
        Self {
            kind: DependencyKind::System(id.into()),
        }
    }

    pub fn secret(id: impl Into<SecretDependencyId>) -> Self {
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
    /// System model IDs this model depends on (protocol stack layering).
    ///
    /// This is a convenience field for declaring system-level dependencies
    /// using plain IDs. It is merged with `dependencies` during graph
    /// validation — entries here are equivalent to `Dependency::system(id)`.
    #[serde(default)]
    pub depends_on: Vec<String>,
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
            depends_on: Vec::new(),
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

    pub fn with_depends_on(mut self, depends_on: &[&str]) -> Self {
        self.depends_on = depends_on.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Return all system model IDs this model depends on, merging both
    /// `depends_on` and system-typed entries from `dependencies`.
    pub fn all_system_deps(&self) -> BTreeSet<String> {
        let mut deps = BTreeSet::new();
        for id in &self.depends_on {
            deps.insert(id.clone());
        }
        for dep in &self.dependencies {
            if let DependencyKind::System(sys_id) = &dep.kind {
                deps.insert(sys_id.0.clone());
            }
        }
        deps
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
///
/// Considers both the `dependencies` (system-typed) and `depends_on` fields.
pub fn validate_dependency_graph_acyclic(models: &[SystemModel]) -> Result<(), String> {
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut outgoing = BTreeMap::<String, Vec<String>>::new();

    for model in models {
        indegree.entry(model.id.clone()).or_insert(0);
        outgoing.entry(model.id.clone()).or_default();
    }

    for model in models {
        // Collect all system deps from both fields.
        let all_deps = model.all_system_deps();
        for target_id in all_deps {
            if indegree.contains_key(&target_id) {
                *indegree
                    .get_mut(&target_id)
                    .expect("target indegree exists") += 1;
                outgoing
                    .get_mut(&model.id)
                    .expect("source entry exists")
                    .push(target_id);
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

/// Validate that all `depends_on` entries reference valid system model IDs
/// and that no model depends on itself.
pub fn validate_depends_on_references(models: &[SystemModel]) -> Result<(), String> {
    let known_ids: BTreeSet<&str> = models.iter().map(|m| m.id.as_str()).collect();
    for model in models {
        for dep_id in &model.depends_on {
            if dep_id == &model.id {
                return Err(format!(
                    "system model '{}' depends on itself via depends_on",
                    model.id
                ));
            }
            if !known_ids.contains(dep_id.as_str()) {
                return Err(format!(
                    "system model '{}' depends_on unknown model '{}'",
                    model.id, dep_id
                ));
            }
        }
        // Also check self-reference in system dependencies.
        for dep in &model.dependencies {
            if let DependencyKind::System(sys_id) = &dep.kind {
                if sys_id.0 == model.id {
                    return Err(format!(
                        "system model '{}' depends on itself via dependencies",
                        model.id
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Collect the transitive set of properties inherited through the dependency
/// chain for a given model.
///
/// Properties are gathered from the model itself and all models it transitively
/// depends on (via `depends_on` and system-typed `dependencies`).
pub fn collect_inherited_properties(model_id: &str, models: &[SystemModel]) -> BTreeSet<Property> {
    let model_map: BTreeMap<String, &SystemModel> =
        models.iter().map(|m| (m.id.clone(), m)).collect();
    let mut visited = BTreeSet::new();
    let mut properties = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(model_id.to_string());
    while let Some(current_id) = queue.pop_front() {
        if !visited.insert(current_id.clone()) {
            continue;
        }
        if let Some(model) = model_map.get(&current_id) {
            for behavior in &model.behaviors {
                for prop in &behavior.properties {
                    properties.insert(*prop);
                }
            }
            for dep_id in model.all_system_deps() {
                if !visited.contains(&dep_id) {
                    queue.push_back(dep_id);
                }
            }
        }
    }
    properties
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
        if let crate::node::NodeBody::Opaque(op) = &node.body {
            let marker = match op {
                TypeOp::Validate(crate::type_op::Predicate::Meta(
                    crate::SystemModelMeta::Property(raw),
                )) => Some(raw.as_str()),
                _ => None,
            };
            if let Some(raw) = marker {
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
        "JsonContentType" => Some(Property::JsonContentType),
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
/// - system + behavior metadata as `TypeOp::Validate(Predicate::Meta(...)` nodes
/// - behavior properties as `TypeOp::Validate(Predicate::Meta(SystemModelMeta::Property(...))` nodes
/// - input/output contracts as `TypeOp::Validate(Predicate::Meta(...)` nodes
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

    append_meta_step(
        &mut dag,
        &mut prev,
        &mut idx,
        crate::SystemModelMeta::SystemId(model.id.clone()),
    );
    append_meta_step(
        &mut dag,
        &mut prev,
        &mut idx,
        crate::SystemModelMeta::SystemKind(format!("{:?}", model.kind)),
    );
    append_meta_step(
        &mut dag,
        &mut prev,
        &mut idx,
        crate::SystemModelMeta::BehaviorId(behavior.id.clone()),
    );
    append_meta_step(
        &mut dag,
        &mut prev,
        &mut idx,
        crate::SystemModelMeta::Invocation(invocation_tag(&behavior.invocation)),
    );

    for property in &behavior.properties {
        append_meta_step(
            &mut dag,
            &mut prev,
            &mut idx,
            crate::SystemModelMeta::Property(format!("{property:?}")),
        );
    }

    for input in &behavior.inputs {
        validate_type_ref("input", &input.name, input.input_type.type_id(), registry)?;
        append_meta_step(
            &mut dag,
            &mut prev,
            &mut idx,
            crate::SystemModelMeta::InputContract {
                name: sanitize_ident(&input.name),
                type_id: input.input_type.type_id().0.clone(),
                required: input.required,
            },
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
        append_meta_step(
            &mut dag,
            &mut prev,
            &mut idx,
            crate::SystemModelMeta::OutputContract {
                name: sanitize_ident(&output.name),
                type_id: output.output_type.type_id().0.clone(),
            },
        );
    }

    dag.add_node(Node::opaque(
        "behavior_output",
        vec![Port::scalar("in", "Json")],
        vec![Port::scalar("out", "Json")],
        TypeOp::Identity,
    ));
    dag.add_edge(Edge::new(prev.as_str(), "out", "behavior_output", "in"));
    validate_no_metadata_validate_custom(&dag)?;
    Ok(dag)
}

fn append_meta_step(
    dag: &mut Dag<TypeOp>,
    prev: &mut String,
    idx: &mut usize,
    payload: crate::SystemModelMeta,
) {
    let marker = match &payload {
        crate::SystemModelMeta::SystemId(value) => format!("system_id_{}", sanitize_ident(value)),
        crate::SystemModelMeta::SystemKind(value) => {
            format!("system_kind_{}", sanitize_ident(value))
        }
        crate::SystemModelMeta::BehaviorId(value) => {
            format!("behavior_id_{}", sanitize_ident(value))
        }
        crate::SystemModelMeta::Invocation(value) => {
            format!("invocation_{}", sanitize_ident(value))
        }
        crate::SystemModelMeta::Property(value) => format!("property_{}", sanitize_ident(value)),
        crate::SystemModelMeta::InputContract { name, type_id, .. } => {
            format!("input_{}_{}", sanitize_ident(name), sanitize_ident(type_id))
        }
        crate::SystemModelMeta::OutputContract { name, type_id } => {
            format!(
                "output_{}_{}",
                sanitize_ident(name),
                sanitize_ident(type_id)
            )
        }
    };
    let node_id = format!("step_{}_{}", idx, sanitize_ident(&marker));
    dag.add_node(Node::opaque(
        node_id.as_str(),
        vec![Port::scalar("in", "Json")],
        vec![Port::scalar("out", "Json")],
        TypeOp::Validate(crate::type_op::Predicate::Meta(payload)),
    ));
    dag.add_edge(Edge::new(prev.as_str(), "out", node_id.as_str(), "in"));
    *prev = node_id;
    *idx += 1;
}

fn validate_no_metadata_validate_custom(dag: &Dag<TypeOp>) -> Result<(), String> {
    for node in &dag.nodes {
        if let crate::node::NodeBody::Opaque(TypeOp::Validate(Predicate::Custom(marker))) =
            &node.body
        {
            if marker.starts_with("meta:") || marker.starts_with("property:") {
                return Err(format!(
                    "metadata marker encoded in Validate(Custom(...)) is forbidden in strict mode: node='{}', marker='{}'",
                    node.id.0, marker
                ));
            }
        }
    }
    Ok(())
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

fn rust_type_for_type_id(type_id: &TypeId) -> String {
    let raw = &type_id.0;

    // "Any" maps to our Value type.
    if raw == "Any" {
        return "gunbc_ir::Value".to_string();
    }

    // Domain types with known Rust paths.
    match raw.as_str() {
        "FileResponse" => return "gunbc_ir::transport::FileResponse".to_string(),
        "ShellResponse" => return "gunbc_ir::transport::ShellResponse".to_string(),
        "RestResponse" => return "gunbc_ir::transport::RestResponse".to_string(),
        "HttpResponse" => return "gunbc_ir::transport::HttpResponse".to_string(),
        _ => {}
    }

    // List<T> → Vec<T>
    if let Some(inner) = parse_unary_generic_type_id(raw, "List") {
        let inner_rust = rust_type_for_type_id(&TypeId::from(inner));
        return format!("Vec<{inner_rust}>");
    }

    // Use ValueBacking for structural mapping.
    // Unknown types fall back to serde_json::Value.
    match crate::types::value_backing_for_type_id(raw).unwrap_or(ValueBacking::Json) {
        ValueBacking::String => "String".to_string(),
        ValueBacking::Secret => "String".to_string(),
        ValueBacking::Bool => "bool".to_string(),
        ValueBacking::Int => "i64".to_string(),
        ValueBacking::Float => "f64".to_string(),
        ValueBacking::Bytes => "Vec<u8>".to_string(),
        ValueBacking::Unit => "()".to_string(),
        ValueBacking::List => "Vec<serde_json::Value>".to_string(),
        ValueBacking::Set => "Vec<serde_json::Value>".to_string(),
        ValueBacking::Json | ValueBacking::Map => "serde_json::Value".to_string(),
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
            format!(
                "{}: {}",
                sanitize_ident(&input.name),
                rust_type_for_type_id(type_id)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = if spec.outputs.len() == 1 {
        let type_id = spec.outputs[0].output_type.type_id();
        rust_type_for_type_id(type_id)
    } else {
        format!(
            "({})",
            spec.outputs
                .iter()
                .map(|out| {
                    let type_id = out.output_type.type_id();
                    rust_type_for_type_id(type_id)
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
                TypeOp::Validate(crate::type_op::Predicate::Meta(
                    crate::SystemModelMeta::Property(raw),
                )) => {
                    properties.push(raw.to_string());
                }
                TypeOp::Validate(crate::type_op::Predicate::Meta(
                    crate::SystemModelMeta::InputContract {
                        name,
                        type_id,
                        required,
                    },
                )) => {
                    inputs.push((name.clone(), type_id.clone(), *required));
                }
                TypeOp::Validate(crate::type_op::Predicate::Meta(
                    crate::SystemModelMeta::OutputContract { name, type_id },
                )) => {
                    outputs.push((name.clone(), type_id.clone()));
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

/// Built-in system models discovered via inventory registration.
///
/// The remaining Rust-owned inventory surface is transport/runtime
/// infrastructure. Domain/provider modeling is moving to `.dag` extdeps.
/// This function collects all linked inventory registrations.
pub fn default_system_models() -> Vec<SystemModel> {
    iter_registered_system_models().collect()
}

// Remaining model data in Rust inventory:
// - lib/transport/src/system_models.rs
//   (transport.file, transport.shell, transport.tcp, transport.http, transport.rest)

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
    fn dependency_roundtrip_preserves_typed_kind() {
        let model = SystemModel::new(
            "test.dep_roundtrip",
            "Test Dep Roundtrip",
            SystemKind::Sdk,
            "v1",
            "roundtrip test",
        )
        .with_behaviors(vec![Behavior::new(
            "op",
            "Op",
            Invocation::Sdk {
                function: "op".to_string(),
                docs: "test".to_string(),
            },
        )
        .with_outputs(vec![BehaviorOutput::new(
            "ok",
            OutputType::TypeId(TypeId::from("Bool")),
        )])])
        .with_dependencies(vec![
            Dependency::system("other.system"),
            Dependency::secret(SecretDependencyId::env_var("MY_SECRET_KEY")),
        ]);

        let json = serde_json::to_string(&model).expect("serialize");
        let parsed: SystemModel = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.dependencies.len(), 2);
        assert!(matches!(
            &parsed.dependencies[0].kind,
            DependencyKind::System(id) if id.0 == "other.system"
        ));
        assert!(matches!(
            &parsed.dependencies[1].kind,
            DependencyKind::Secret(id) if id.0 == "MY_SECRET_KEY"
        ));

        // No string-prefix parsing exists in validation
        validate_dependency_graph_acyclic(&[parsed]).expect("acyclic");
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

    #[test]
    fn behavior_contract_dag_erasure_invariance() {
        let model = SystemModel::new(
            "provider.erasure",
            "Provider Erasure",
            SystemKind::Sdk,
            "v1",
            "test provider",
        )
        .with_behaviors(vec![Behavior::new(
            "query",
            "Query operation",
            Invocation::Sdk {
                function: "query".to_string(),
                docs: "query docs".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required(
            "id",
            InputType::TypeId(TypeId::from("String")),
        )])
        .with_outputs(vec![BehaviorOutput::new(
            "result",
            OutputType::TypeId(TypeId::from("Json")),
        )])
        .with_properties(&[Property::ReadOnly, Property::Deterministic])]);

        let mut registry = TypeRegistry::with_core_types();
        let dag = build_behavior_contract_dag(&model, &model.behaviors[0], &registry)
            .expect("should build contract DAG");

        // Strip all Meta nodes — remaining structure should still be valid
        // (Identity bookends + Optional wraps only).
        let non_meta_nodes: Vec<_> = dag
            .nodes
            .iter()
            .filter(|n| {
                !matches!(
                    &n.body,
                    crate::node::NodeBody::Opaque(TypeOp::Validate(
                        crate::type_op::Predicate::Meta(_)
                    ))
                )
            })
            .collect();

        // Must have at least the two Identity bookends.
        assert!(
            non_meta_nodes.len() >= 2,
            "expected at least 2 non-Meta nodes (bookends), got {}",
            non_meta_nodes.len()
        );

        // Bookend nodes must be Identity.
        let first = &non_meta_nodes[0];
        let last = &non_meta_nodes[non_meta_nodes.len() - 1];
        assert!(
            matches!(&first.body, crate::node::NodeBody::Opaque(TypeOp::Identity)),
            "first non-Meta node should be Identity"
        );
        assert!(
            matches!(&last.body, crate::node::NodeBody::Opaque(TypeOp::Identity)),
            "last non-Meta node should be Identity"
        );

        // No legacy Validate(Custom) nodes should exist.
        let has_validate_custom = dag.nodes.iter().any(|n| {
            matches!(
                &n.body,
                crate::node::NodeBody::Opaque(TypeOp::Validate(Predicate::Custom(_)))
            )
        });
        assert!(
            !has_validate_custom,
            "contract DAG must not contain legacy Validate(Custom) nodes"
        );

        // Registration with the DAG should succeed.
        register_system_behavior_type_dags(&mut registry, &[model])
            .expect("registration with clean contract DAG should succeed");
    }

    // --- depends_on layering tests ---

    fn make_layered_models() -> Vec<SystemModel> {
        let mk_behavior = |id: &str| {
            Behavior::new(
                id,
                format!("{id} behavior"),
                Invocation::Sdk {
                    function: id.to_string(),
                    docs: "docs".to_string(),
                },
            )
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from("Bool")),
            )])
        };

        vec![
            SystemModel::new(
                "layer.tcp",
                "TCP",
                SystemKind::Convention,
                "v1",
                "tcp layer",
            )
            .with_behaviors(vec![
                mk_behavior("connect").with_properties(&[Property::WritesWorld])
            ])
            .with_depends_on(&[]),
            SystemModel::new(
                "layer.http",
                "HTTP",
                SystemKind::Protocol,
                "v1",
                "http layer",
            )
            .with_behaviors(vec![
                mk_behavior("get").with_properties(&[Property::ReadOnly, Property::Deterministic]),
                mk_behavior("post").with_properties(&[Property::WritesWorld, Property::Idempotent]),
            ])
            .with_depends_on(&["layer.tcp"]),
            SystemModel::new(
                "layer.rest",
                "REST",
                SystemKind::Protocol,
                "v1",
                "rest layer",
            )
            .with_behaviors(vec![
                mk_behavior("get").with_properties(&[
                    Property::ReadOnly,
                    Property::Deterministic,
                    Property::JsonContentType,
                ]),
                mk_behavior("post").with_properties(&[
                    Property::WritesWorld,
                    Property::Idempotent,
                    Property::JsonContentType,
                ]),
            ])
            .with_depends_on(&["layer.http"]),
        ]
    }

    #[test]
    fn depends_on_references_must_be_valid() {
        let models = make_layered_models();
        validate_depends_on_references(&models).expect("layered models should validate");
    }

    #[test]
    fn depends_on_rejects_unknown_reference() {
        let mk_behavior = |id: &str| {
            Behavior::new(
                id,
                format!("{id} behavior"),
                Invocation::Sdk {
                    function: id.to_string(),
                    docs: "docs".to_string(),
                },
            )
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from("Bool")),
            )])
        };

        let models = vec![SystemModel::new("x.a", "A", SystemKind::Sdk, "v1", "a")
            .with_behaviors(vec![mk_behavior("op")])
            .with_depends_on(&["x.nonexistent"])];

        let err = validate_depends_on_references(&models)
            .expect_err("unknown depends_on target should fail");
        assert!(err.contains("x.nonexistent"), "unexpected error: {err}");
    }

    #[test]
    fn depends_on_rejects_self_reference() {
        let mk_behavior = |id: &str| {
            Behavior::new(
                id,
                format!("{id} behavior"),
                Invocation::Sdk {
                    function: id.to_string(),
                    docs: "docs".to_string(),
                },
            )
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from("Bool")),
            )])
        };

        let models = vec![
            SystemModel::new("x.self", "Self", SystemKind::Sdk, "v1", "self ref")
                .with_behaviors(vec![mk_behavior("op")])
                .with_depends_on(&["x.self"]),
        ];

        let err = validate_depends_on_references(&models)
            .expect_err("self-referencing depends_on should fail");
        assert!(err.contains("depends on itself"), "unexpected error: {err}");
    }

    #[test]
    fn layered_models_are_acyclic() {
        let models = make_layered_models();
        validate_dependency_graph_acyclic(&models).expect("TCP -> HTTP -> REST should be acyclic");
    }

    #[test]
    fn layered_models_detect_cycle_via_depends_on() {
        let mk_behavior = |id: &str| {
            Behavior::new(
                id,
                format!("{id} behavior"),
                Invocation::Sdk {
                    function: id.to_string(),
                    docs: "docs".to_string(),
                },
            )
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from("Bool")),
            )])
        };

        let models = vec![
            SystemModel::new("cycle.a", "A", SystemKind::Sdk, "v1", "a")
                .with_behaviors(vec![mk_behavior("op")])
                .with_depends_on(&["cycle.b"]),
            SystemModel::new("cycle.b", "B", SystemKind::Sdk, "v1", "b")
                .with_behaviors(vec![mk_behavior("op")])
                .with_depends_on(&["cycle.a"]),
        ];

        let err = validate_dependency_graph_acyclic(&models).expect_err("cycle should be detected");
        assert!(err.contains("cycle"), "unexpected error: {err}");
    }

    #[test]
    fn inherited_properties_flow_through_layered_chain() {
        let models = make_layered_models();

        // TCP layer only has WritesWorld
        let tcp_props = collect_inherited_properties("layer.tcp", &models);
        assert!(tcp_props.contains(&Property::WritesWorld));
        assert!(!tcp_props.contains(&Property::JsonContentType));

        // HTTP layer has its own properties + TCP's WritesWorld
        let http_props = collect_inherited_properties("layer.http", &models);
        assert!(http_props.contains(&Property::WritesWorld));
        assert!(http_props.contains(&Property::ReadOnly));
        assert!(http_props.contains(&Property::Deterministic));
        assert!(http_props.contains(&Property::Idempotent));
        assert!(!http_props.contains(&Property::JsonContentType));

        // REST layer inherits everything from HTTP + TCP, plus JsonContentType
        let rest_props = collect_inherited_properties("layer.rest", &models);
        assert!(rest_props.contains(&Property::WritesWorld));
        assert!(rest_props.contains(&Property::ReadOnly));
        assert!(rest_props.contains(&Property::Deterministic));
        assert!(rest_props.contains(&Property::Idempotent));
        assert!(rest_props.contains(&Property::JsonContentType));
    }

    #[test]
    fn all_system_deps_merges_both_fields() {
        let model = SystemModel::new("test.merge", "Merge", SystemKind::Sdk, "v1", "merge test")
            .with_behaviors(vec![Behavior::new(
                "op",
                "Op",
                Invocation::Sdk {
                    function: "op".to_string(),
                    docs: "test".to_string(),
                },
            )
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from("Bool")),
            )])])
            .with_dependencies(vec![Dependency::system("dep.via_dependency")])
            .with_depends_on(&["dep.via_depends_on"]);

        let all_deps = model.all_system_deps();
        assert!(all_deps.contains("dep.via_dependency"));
        assert!(all_deps.contains("dep.via_depends_on"));
        assert_eq!(all_deps.len(), 2);
    }

    #[test]
    fn depends_on_defaults_to_empty_on_deserialization() {
        let json = r#"{"id":"test.serde","name":"Serde Test","kind":"Sdk","version":"v1","docs":"serde test","behaviors":[{"id":"op","description":"Op","invocation":{"Sdk":{"function":"op","docs":"test"}},"inputs":[],"outputs":[{"name":"ok","output_type":{"TypeId":"Bool"}}],"properties":[]}],"dependencies":[]}"#;
        let model: SystemModel = serde_json::from_str(json).expect("deserialize model");
        assert!(
            model.depends_on.is_empty(),
            "depends_on should default to empty when missing from JSON"
        );
    }

    #[test]
    fn depends_on_roundtrips_through_serde() {
        let model = SystemModel::new("test.roundtrip", "Roundtrip", SystemKind::Sdk, "v1", "rt")
            .with_behaviors(vec![Behavior::new(
                "op",
                "Op",
                Invocation::Sdk {
                    function: "op".to_string(),
                    docs: "test".to_string(),
                },
            )
            .with_outputs(vec![BehaviorOutput::new(
                "ok",
                OutputType::TypeId(TypeId::from("Bool")),
            )])])
            .with_depends_on(&["other.model"]);

        let json = serde_json::to_string(&model).expect("serialize");
        let parsed: SystemModel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.depends_on, vec!["other.model".to_string()]);
    }

    // Model-specific behavior tests for the remaining Rust inventory surface
    // live in the owning transport crate.
    // Cross-cutting tests (contract specs, store mapping) moved to gunbc-tests.
}
