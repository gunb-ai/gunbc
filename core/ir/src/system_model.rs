//! DAG-native system modeling primitives.
//!
//! This module models external systems/services as typed behavioral catalogs
//! that map directly onto `TypeId` / `Dag<TypeOp>` contracts.

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

fn ty(id: &str) -> InputType {
    InputType::TypeId(TypeId::from(id))
}

fn out_ty(id: &str) -> OutputType {
    OutputType::TypeId(TypeId::from(id))
}

/// Built-in transport + GCP system models.
pub fn default_system_models() -> Vec<SystemModel> {
    vec![
        SystemModel::new(
            "gcp.secret_manager",
            "GCP Secret Manager",
            SystemKind::SecretProvider,
            "v1",
            "Secret Manager access/list/upsert behaviors",
        )
        .with_behaviors(vec![
            Behavior::new(
                "access_secret_version",
                "Access one secret version payload",
                Invocation::Rest {
                    method: "GET".to_string(),
                    path: "/v1/projects/*/secrets/*/versions/*:access".to_string(),
                    docs: "https://cloud.google.com/secret-manager/docs".to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("secret_id", ty("String")),
                BehaviorInput::required("version", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new(
                "payload",
                out_ty("GcpSecretPayload"),
            )])
            .with_properties(&[Property::ReadOnly, Property::Deterministic]),
            Behavior::new(
                "list_secrets",
                "List available secrets in a project",
                Invocation::Rest {
                    method: "GET".to_string(),
                    path: "/v1/projects/*/secrets".to_string(),
                    docs: "https://cloud.google.com/secret-manager/docs".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("project_id", ty("String"))])
            .with_outputs(vec![BehaviorOutput::new("secrets", out_ty("JsonList"))])
            .with_properties(&[Property::ReadOnly, Property::Deterministic]),
            Behavior::new(
                "upsert_secret",
                "Create or update a secret payload",
                Invocation::Rest {
                    method: "POST".to_string(),
                    path: "/v1/projects/*/secrets".to_string(),
                    docs: "https://cloud.google.com/secret-manager/docs".to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("secret_id", ty("String")),
                BehaviorInput::required("payload", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("written", out_ty("Bool"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        ])
        .with_dependencies(vec![Dependency::secret(
            "secret:GOOGLE_APPLICATION_CREDENTIALS",
        )]),
        SystemModel::new(
            "gcp.iam",
            "GCP IAM",
            SystemKind::IdentityProvider,
            "v1",
            "Service account and IAM binding management",
        )
        .with_behaviors(vec![
            Behavior::new(
                "service_account_upsert",
                "Create/update service account",
                Invocation::Cli {
                    command: "gcloud iam service-accounts".to_string(),
                    docs: "https://cloud.google.com/iam/docs/service-accounts".to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("account_id", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("service_account", out_ty("Json"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "binding_upsert",
                "Create/update IAM binding",
                Invocation::Cli {
                    command: "gcloud projects add-iam-policy-binding".to_string(),
                    docs: "https://cloud.google.com/iam/docs/granting-changing-revoking-access"
                        .to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("member", ty("String")),
                BehaviorInput::required("role", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("binding", out_ty("Json"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "service_account_delete",
                "Delete service account",
                Invocation::Cli {
                    command: "gcloud iam service-accounts delete".to_string(),
                    docs: "https://cloud.google.com/iam/docs/service-accounts-delete".to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("account_email", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("deleted", out_ty("Bool"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "binding_remove",
                "Remove IAM binding",
                Invocation::Cli {
                    command: "gcloud projects remove-iam-policy-binding".to_string(),
                    docs: "https://cloud.google.com/iam/docs/granting-changing-revoking-access"
                        .to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("member", ty("String")),
                BehaviorInput::required("role", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("binding_removed", out_ty("Bool"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "wif_pool_upsert",
                "Create or update WIF pool",
                Invocation::Cli {
                    command: "gcloud iam workload-identity-pools".to_string(),
                    docs: "https://cloud.google.com/iam/docs/workload-identity-federation"
                        .to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("pool_id", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("pool", out_ty("Json"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "wif_provider_upsert",
                "Create or update WIF provider",
                Invocation::Cli {
                    command: "gcloud iam workload-identity-pools providers".to_string(),
                    docs: "https://cloud.google.com/iam/docs/workload-identity-federation"
                        .to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("project_id", ty("String")),
                BehaviorInput::required("pool_id", ty("String")),
                BehaviorInput::required("provider_id", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("provider", out_ty("Json"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        ])
        .with_dependencies(vec![Dependency::secret(
            "secret:GOOGLE_APPLICATION_CREDENTIALS",
        )]),
        SystemModel::new(
            "gcp.gcs",
            "GCP Cloud Storage",
            SystemKind::StorageProvider,
            "v1",
            "Object get/put/list/delete behaviors",
        )
        .with_behaviors(vec![
            Behavior::new(
                "get_object",
                "Read one object from bucket",
                Invocation::Rest {
                    method: "GET".to_string(),
                    path: "/storage/v1/b/*/o/*".to_string(),
                    docs: "https://cloud.google.com/storage/docs/json_api".to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("bucket", ty("String")),
                BehaviorInput::required("object", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("content", out_ty("String"))])
            .with_properties(&[Property::ReadOnly, Property::Deterministic]),
            Behavior::new(
                "put_object",
                "Write object to bucket",
                Invocation::Rest {
                    method: "PUT".to_string(),
                    path: "/upload/storage/v1/b/*/o".to_string(),
                    docs: "https://cloud.google.com/storage/docs/json_api/v1/how-tos/upload"
                        .to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("bucket", ty("String")),
                BehaviorInput::required("object", ty("String")),
                BehaviorInput::required("content", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("written", out_ty("Bool"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "list_objects",
                "List bucket objects",
                Invocation::Rest {
                    method: "GET".to_string(),
                    path: "/storage/v1/b/*/o".to_string(),
                    docs: "https://cloud.google.com/storage/docs/json_api".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("bucket", ty("String"))])
            .with_outputs(vec![BehaviorOutput::new("objects", out_ty("JsonList"))])
            .with_properties(&[Property::ReadOnly, Property::Deterministic]),
            Behavior::new(
                "delete_object",
                "Delete object from bucket",
                Invocation::Rest {
                    method: "DELETE".to_string(),
                    path: "/storage/v1/b/*/o/*".to_string(),
                    docs: "https://cloud.google.com/storage/docs/json_api".to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("bucket", ty("String")),
                BehaviorInput::required("object", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new("deleted", out_ty("Bool"))])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        ])
        .with_dependencies(vec![Dependency::secret(
            "secret:GOOGLE_APPLICATION_CREDENTIALS",
        )]),
        SystemModel::new(
            "transport.file",
            "File Transport",
            SystemKind::Transport,
            "v1",
            "File read/write/exists/delete transport behaviors",
        )
        .with_behaviors(vec![
            Behavior::new(
                "read",
                "Read a file path",
                Invocation::Protocol {
                    protocol: "file".to_string(),
                    docs: "gunbc file transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("path", ty("String"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("FileResponse"),
            )])
            .with_properties(&[Property::ReadOnly, Property::Deterministic]),
            Behavior::new(
                "write",
                "Write file content",
                Invocation::Protocol {
                    protocol: "file".to_string(),
                    docs: "gunbc file transport".to_string(),
                },
            )
            .with_inputs(vec![
                BehaviorInput::required("path", ty("String")),
                BehaviorInput::required("content", ty("String")),
            ])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("FileResponse"),
            )])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "exists",
                "Check if a file exists",
                Invocation::Protocol {
                    protocol: "file".to_string(),
                    docs: "gunbc file transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("path", ty("String"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("FileResponse"),
            )])
            .with_properties(&[Property::ReadOnly, Property::Deterministic]),
            Behavior::new(
                "delete",
                "Delete file path",
                Invocation::Protocol {
                    protocol: "file".to_string(),
                    docs: "gunbc file transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("path", ty("String"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("FileResponse"),
            )])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        ]),
        SystemModel::new(
            "transport.shell",
            "Shell Transport",
            SystemKind::Transport,
            "v1",
            "Shell execution transport behavior",
        )
        .with_behaviors(vec![Behavior::new(
            "exec",
            "Execute shell command",
            Invocation::Protocol {
                protocol: "shell".to_string(),
                docs: "gunbc shell transport".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("command", ty("String")),
            BehaviorInput::optional("args", ty("StringList")),
            BehaviorInput::optional("cwd", ty("OptionalString")),
            BehaviorInput::optional("env", ty("Json")),
            BehaviorInput::optional("timeout_ms", ty("OptionalInt")),
        ])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("ShellResponse"),
        )])
        .with_properties(&[Property::WritesWorld])]),
        SystemModel::new(
            "transport.http_rest",
            "HTTP/REST Transport",
            SystemKind::Transport,
            "v1",
            "HTTP+REST request behaviors",
        )
        .with_behaviors(vec![
            Behavior::new(
                "http_get",
                "HTTP GET request",
                Invocation::Protocol {
                    protocol: "http".to_string(),
                    docs: "gunbc http transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("HttpResponse"),
            )])
            .with_properties(&[Property::ReadOnly]),
            Behavior::new(
                "rest_post",
                "REST POST request",
                Invocation::Protocol {
                    protocol: "rest".to_string(),
                    docs: "gunbc rest transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("RestResponse"),
            )])
            .with_properties(&[Property::WritesWorld]),
            Behavior::new(
                "http_post",
                "HTTP POST request",
                Invocation::Protocol {
                    protocol: "http".to_string(),
                    docs: "gunbc http transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("HttpResponse"),
            )])
            .with_properties(&[Property::WritesWorld]),
            Behavior::new(
                "http_put",
                "HTTP PUT request",
                Invocation::Protocol {
                    protocol: "http".to_string(),
                    docs: "gunbc http transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("HttpResponse"),
            )])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "http_delete",
                "HTTP DELETE request",
                Invocation::Protocol {
                    protocol: "http".to_string(),
                    docs: "gunbc http transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("HttpResponse"),
            )])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "rest_get",
                "REST GET request",
                Invocation::Protocol {
                    protocol: "rest".to_string(),
                    docs: "gunbc rest transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("RestResponse"),
            )])
            .with_properties(&[Property::ReadOnly]),
            Behavior::new(
                "rest_put",
                "REST PUT request",
                Invocation::Protocol {
                    protocol: "rest".to_string(),
                    docs: "gunbc rest transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("RestResponse"),
            )])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
            Behavior::new(
                "rest_delete",
                "REST DELETE request",
                Invocation::Protocol {
                    protocol: "rest".to_string(),
                    docs: "gunbc rest transport".to_string(),
                },
            )
            .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
            .with_outputs(vec![BehaviorOutput::new(
                "response",
                out_ty("RestResponse"),
            )])
            .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        ]),
    ]
}

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
    fn gcp_secret_manager_model_declares_adc_secret_dependency() {
        let models = default_system_models();
        let model = models
            .iter()
            .find(|m| m.id == "gcp.secret_manager")
            .expect("gcp secret manager model should exist");
        assert!(model.dependencies.iter().any(|dep| {
            dep.kind == DependencyKind::Secret("secret:GOOGLE_APPLICATION_CREDENTIALS".to_string())
        }));
    }

    #[test]
    fn gcp_models_expose_expected_behavior_sets() {
        let models = default_system_models();
        let secret_manager = models
            .iter()
            .find(|m| m.id == "gcp.secret_manager")
            .expect("gcp secret manager model");
        let secret_ops: BTreeSet<_> = secret_manager
            .behaviors
            .iter()
            .map(|b| b.id.as_str())
            .collect();
        assert!(secret_ops.contains("access_secret_version"));
        assert!(secret_ops.contains("list_secrets"));
        assert!(secret_ops.contains("upsert_secret"));

        let iam = models
            .iter()
            .find(|m| m.id == "gcp.iam")
            .expect("gcp iam model");
        let iam_ops: BTreeSet<_> = iam.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert!(iam_ops.contains("service_account_upsert"));
        assert!(iam_ops.contains("service_account_delete"));
        assert!(iam_ops.contains("binding_upsert"));
        assert!(iam_ops.contains("binding_remove"));
        assert!(iam_ops.contains("wif_pool_upsert"));
        assert!(iam_ops.contains("wif_provider_upsert"));

        let gcs = models
            .iter()
            .find(|m| m.id == "gcp.gcs")
            .expect("gcp gcs model");
        let gcs_ops: BTreeSet<_> = gcs.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert!(gcs_ops.contains("get_object"));
        assert!(gcs_ops.contains("put_object"));
        assert!(gcs_ops.contains("list_objects"));
        assert!(gcs_ops.contains("delete_object"));
    }

    #[test]
    fn transport_models_expose_expected_behavior_sets() {
        let models = default_system_models();
        let file_model = models
            .iter()
            .find(|m| m.id == "transport.file")
            .expect("transport.file model");
        let file_ops: BTreeSet<_> = file_model.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(
            file_ops,
            BTreeSet::from(["read", "write", "exists", "delete"])
        );

        let shell_model = models
            .iter()
            .find(|m| m.id == "transport.shell")
            .expect("transport.shell model");
        let exec = shell_model
            .behaviors
            .iter()
            .find(|b| b.id == "exec")
            .expect("shell exec behavior should exist");
        let shell_inputs: BTreeSet<_> = exec.inputs.iter().map(|i| i.name.as_str()).collect();
        assert!(shell_inputs.contains("command"));
        assert!(shell_inputs.contains("args"));
        assert!(shell_inputs.contains("env"));
        assert!(shell_inputs.contains("cwd"));
        assert!(shell_inputs.contains("timeout_ms"));

        let http_rest_model = models
            .iter()
            .find(|m| m.id == "transport.http_rest")
            .expect("transport.http_rest model");
        let ops: BTreeSet<_> = http_rest_model
            .behaviors
            .iter()
            .map(|b| b.id.as_str())
            .collect();
        for op in [
            "http_get",
            "http_post",
            "http_put",
            "http_delete",
            "rest_get",
            "rest_post",
            "rest_put",
            "rest_delete",
        ] {
            assert!(ops.contains(op), "missing http/rest operation {op}");
        }
    }
}
