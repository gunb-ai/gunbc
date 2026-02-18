//! DAG-native system modeling primitives.
//!
//! This module models external systems/services as typed behavioral catalogs
//! that map directly onto `TypeId` / `Dag<TypeOp>` contracts.

use crate::port_type::PortType;
use crate::{Dag, TypeId, TypeOp, TypeRegistry};
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
pub fn registered_system_model_map() -> BTreeMap<String, SystemModel> {
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
    }
    Ok(())
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
    let mut specs = Vec::new();
    for model in models {
        for behavior in &model.behaviors {
            let has = |property: Property| behavior.properties.contains(&property);

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

fn rust_type_for_type_id(type_id: &TypeId) -> String {
    let port_type = PortType::from(type_id);
    rust_type_for_port_type(&port_type, type_id)
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
            format!(
                "{}: {}",
                sanitize_ident(&input.name),
                rust_type_for_type_id(input.input_type.type_id())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = if spec.outputs.len() == 1 {
        rust_type_for_type_id(spec.outputs[0].output_type.type_id()).to_string()
    } else {
        format!(
            "({})",
            spec.outputs
                .iter()
                .map(|out| rust_type_for_type_id(out.output_type.type_id()))
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
    for provider in ["gcp.gcs", "aws.s3"] {
        let model = models
            .iter()
            .find(|m| m.id == provider)
            .ok_or_else(|| format!("missing storage provider model '{provider}'"))?;
        let ops: BTreeSet<&str> = model.behaviors.iter().map(|b| b.id.as_str()).collect();
        if !required.is_subset(&ops) {
            return Err(format!(
                "storage provider '{}' missing required store operations: {:?}",
                provider,
                required.difference(&ops).copied().collect::<Vec<_>>()
            ));
        }
    }
    Ok(())
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

    // Model-specific behavior tests (GCP, AWS, transport) moved to owning crates:
    // - lib/gcp-ops/src/system_models.rs
    // - lib/aws-ops/src/system_models.rs
    // - lib/transport/src/system_models.rs
    // Cross-cutting tests (contract specs, store mapping) moved to gunbc-dag.
}
