//! WIF bootstrap DAG builder with idempotent upsert stages.

use crate::infra_spec::InfraSpec;
use gunbc_delegate_macros::DelegateExecutable;
use gunbc_exec::{require_bool, require_str, ExecError, Executable, OutputMap};
use gunbc_ir::build::port;
use gunbc_ir::transport::TransportResponse;
use gunbc_ir::{
    AuthScheme, BuilderError, Credential, Dag, DagBuilder, Node, NodeRef, Secret, Value,
};
use gunbc_lib_gcp_ops::services::iam::{IamRest, IamService};
use gunbc_lib_gcp_ops::services::resource_manager::{ResourceManagerRest, ResourceManagerService};
use gunbc_lib_gcp_ops::services::workload_identity::{
    WifProviderConfig, WorkloadIdentityRest, WorkloadIdentityService,
};
use gunbc_lib_transport::TransportOps;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

const ACTION_NOOP: &str = "noop";
const ACTION_CREATE: &str = "create";
const ACTION_UPDATE: &str = "update";

#[derive(Debug, Clone, DelegateExecutable)]
pub enum InfraBootstrapGraphOp {
    Bootstrap(InfraBootstrapOps),
    Transport(TransportOps),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InfraBootstrapOps {
    PassAccessToken,
    EnableApis {
        project: String,
        services: Vec<String>,
    },
    PrepareEnsureWifPool {
        project_number: String,
        pool_id: String,
        display_name: String,
    },
    CheckAndPrepareWifPool {
        project_number: String,
        pool_id: String,
        display_name: String,
    },
    ParseEnsureWifPool,
    PrepareEnsureWifProvider {
        project_number: String,
        pool_id: String,
        provider_id: String,
        oidc_issuer_uri: String,
        attribute_mapping: BTreeMap<String, String>,
        attribute_condition: Option<String>,
    },
    CheckAndPrepareWifProvider {
        project_number: String,
        pool_id: String,
        provider_id: String,
        oidc_issuer_uri: String,
        attribute_mapping: BTreeMap<String, String>,
        attribute_condition: Option<String>,
    },
    ParseEnsureWifProvider,
    PrepareEnsureServiceAccount {
        project: String,
        account_id: String,
        email: String,
        display_name: String,
    },
    CheckAndPrepareServiceAccount {
        project: String,
        account_id: String,
        email: String,
        display_name: String,
    },
    ParseEnsureServiceAccount,
    PrepareEnsureProjectRoleBinding {
        project: String,
        role: String,
        service_account: String,
    },
    CheckAndPrepareProjectRoleBinding {
        project: String,
        role: String,
        service_account: String,
    },
    ParseEnsureProjectRoleBinding,
    PrepareEnsureSaWifBinding {
        project: String,
        service_account: String,
        member: String,
    },
    CheckAndPrepareSaWifBinding {
        project: String,
        service_account: String,
        member: String,
    },
    ParseEnsureSaWifBinding,
    SummarizeBootstrap {
        environment: String,
        project: String,
    },
}

impl Executable for InfraBootstrapOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            InfraBootstrapOps::PassAccessToken => {
                let access_token = require_str(&inputs, "access_token")?;
                OutputMap::new().str("access_token", access_token).ok()
            }
            InfraBootstrapOps::EnableApis { project, services } => {
                let _access_token = require_str(&inputs, "access_token")?;
                let message = format!(
                    "API enablement staged for project {}: {}",
                    project,
                    services.join(", ")
                );
                OutputMap::new().bool("ok", true).str("note", message).ok()
            }
            InfraBootstrapOps::PrepareEnsureWifPool { .. } => {
                let _prev_ok = require_bool(&inputs, "prev_ok")?;
                self.prepare_get_pool(inputs)
            }
            InfraBootstrapOps::CheckAndPrepareWifPool { .. } => self.check_prepare_pool(inputs),
            InfraBootstrapOps::ParseEnsureWifPool => self.parse_stage(inputs, "wif_pool"),
            InfraBootstrapOps::PrepareEnsureWifProvider { .. } => {
                let _prev_ok = require_bool(&inputs, "prev_ok")?;
                self.prepare_get_provider(inputs)
            }
            InfraBootstrapOps::CheckAndPrepareWifProvider { .. } => {
                self.check_prepare_provider(inputs)
            }
            InfraBootstrapOps::ParseEnsureWifProvider => self.parse_stage(inputs, "wif_provider"),
            InfraBootstrapOps::PrepareEnsureServiceAccount { .. } => {
                let _prev_ok = require_bool(&inputs, "prev_ok")?;
                self.prepare_get_service_account(inputs)
            }
            InfraBootstrapOps::CheckAndPrepareServiceAccount { .. } => {
                self.check_prepare_service_account(inputs)
            }
            InfraBootstrapOps::ParseEnsureServiceAccount => {
                self.parse_stage(inputs, "service_account")
            }
            InfraBootstrapOps::PrepareEnsureProjectRoleBinding { .. } => {
                let _prev_ok = require_bool(&inputs, "prev_ok")?;
                self.prepare_get_project_role_binding(inputs)
            }
            InfraBootstrapOps::CheckAndPrepareProjectRoleBinding { .. } => {
                self.check_prepare_project_role_binding(inputs)
            }
            InfraBootstrapOps::ParseEnsureProjectRoleBinding => self.parse_stage(inputs, "role"),
            InfraBootstrapOps::PrepareEnsureSaWifBinding { .. } => {
                let _prev_ok = require_bool(&inputs, "prev_ok")?;
                self.prepare_get_sa_wif_binding(inputs)
            }
            InfraBootstrapOps::CheckAndPrepareSaWifBinding { .. } => {
                self.check_prepare_sa_wif_binding(inputs)
            }
            InfraBootstrapOps::ParseEnsureSaWifBinding => self.parse_stage(inputs, "sa_wif"),
            InfraBootstrapOps::SummarizeBootstrap {
                environment,
                project,
            } => {
                let _prev_ok = require_bool(&inputs, "prev_ok")?;
                OutputMap::new()
                    .bool("ok", true)
                    .str(
                        "report",
                        format!(
                            "WIF bootstrap flow completed for env '{}' (project '{}')",
                            environment, project
                        ),
                    )
                    .ok()
            }
        }
    }
}

impl InfraBootstrapOps {
    fn prepare_get_pool(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project_number, pool_id, _display_name) = match self {
            InfraBootstrapOps::PrepareEnsureWifPool {
                project_number,
                pool_id,
                display_name,
            } => (project_number, pool_id, display_name),
            _ => return Err(ExecError::new("invalid op variant for prepare_get_pool")),
        };
        let svc = workload_identity_service(access_token);
        let req = svc.get_pool(project_number, pool_id);
        OutputMap::new()
            .request("request", req.into())
            .bool("skip", false)
            .ok()
    }

    fn check_prepare_pool(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project_number, pool_id, display_name) = match self {
            InfraBootstrapOps::CheckAndPrepareWifPool {
                project_number,
                pool_id,
                display_name,
            } => (project_number, pool_id, display_name),
            _ => return Err(ExecError::new("invalid op variant for check_prepare_pool")),
        };

        let response = required_response(&inputs)?;
        let Some(response) = response else {
            return skipped_stage(ACTION_NOOP);
        };
        let rest = as_rest_response(response)?;
        match rest.status {
            200..=299 => skipped_stage(ACTION_NOOP),
            404 => {
                let svc = workload_identity_service(access_token);
                let req = svc.create_pool(project_number, pool_id, display_name);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .str("action", ACTION_CREATE)
                    .ok()
            }
            status => Err(ExecError::new(format!(
                "wif pool read failed (status {}): {}",
                status,
                body_summary(&rest.body)
            ))),
        }
    }

    fn prepare_get_provider(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project_number, pool_id, provider_id, ..) = match self {
            InfraBootstrapOps::PrepareEnsureWifProvider {
                project_number,
                pool_id,
                provider_id,
                oidc_issuer_uri,
                attribute_mapping,
                attribute_condition,
            } => (
                project_number,
                pool_id,
                provider_id,
                oidc_issuer_uri,
                attribute_mapping,
                attribute_condition,
            ),
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for prepare_get_provider",
                ))
            }
        };
        let svc = workload_identity_service(access_token);
        let req = svc.get_provider(project_number, pool_id, provider_id);
        OutputMap::new()
            .request("request", req.into())
            .bool("skip", false)
            .ok()
    }

    fn check_prepare_provider(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (
            project_number,
            pool_id,
            provider_id,
            oidc_issuer_uri,
            attribute_mapping,
            attribute_condition,
        ) = match self {
            InfraBootstrapOps::CheckAndPrepareWifProvider {
                project_number,
                pool_id,
                provider_id,
                oidc_issuer_uri,
                attribute_mapping,
                attribute_condition,
            } => (
                project_number,
                pool_id,
                provider_id,
                oidc_issuer_uri,
                attribute_mapping,
                attribute_condition,
            ),
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for check_prepare_provider",
                ))
            }
        };
        let response = required_response(&inputs)?;
        let Some(response) = response else {
            return skipped_stage(ACTION_NOOP);
        };
        let rest = as_rest_response(response)?;
        let config = to_provider_config(
            oidc_issuer_uri.clone(),
            attribute_mapping.clone(),
            attribute_condition.clone(),
        );
        let svc = workload_identity_service(access_token);

        match rest.status {
            200..=299 => {
                let req = svc.update_provider(project_number, pool_id, provider_id, &config);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .str("action", ACTION_UPDATE)
                    .ok()
            }
            404 => {
                let req = svc.create_provider(project_number, pool_id, provider_id, &config);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .str("action", ACTION_CREATE)
                    .ok()
            }
            status => Err(ExecError::new(format!(
                "wif provider read failed (status {}): {}",
                status,
                body_summary(&rest.body)
            ))),
        }
    }

    fn prepare_get_service_account(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project, _account_id, email, _display_name) = match self {
            InfraBootstrapOps::PrepareEnsureServiceAccount {
                project,
                account_id,
                email,
                display_name,
            } => (project, account_id, email, display_name),
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for prepare_get_service_account",
                ))
            }
        };
        let svc = iam_service(access_token);
        let req = svc.get_service_account(project, email);
        OutputMap::new()
            .request("request", req.into())
            .bool("skip", false)
            .ok()
    }

    fn check_prepare_service_account(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project, account_id, email, display_name) = match self {
            InfraBootstrapOps::CheckAndPrepareServiceAccount {
                project,
                account_id,
                email,
                display_name,
            } => (project, account_id, email, display_name),
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for check_prepare_service_account",
                ))
            }
        };
        let response = required_response(&inputs)?;
        let Some(response) = response else {
            return skipped_stage(ACTION_NOOP);
        };
        let rest = as_rest_response(response)?;
        let svc = iam_service(access_token);

        match rest.status {
            200..=299 => {
                let existing_display = rest
                    .body
                    .get("displayName")
                    .and_then(serde_json::Value::as_str);
                if existing_display == Some(display_name.as_str()) {
                    return skipped_stage(ACTION_NOOP);
                }
                let req = svc.update_service_account(project, email, display_name);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .str("action", ACTION_UPDATE)
                    .ok()
            }
            404 => {
                let req = svc.create_service_account(project, account_id, display_name);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .str("action", ACTION_CREATE)
                    .ok()
            }
            status => Err(ExecError::new(format!(
                "service account read failed (status {}): {}",
                status,
                body_summary(&rest.body)
            ))),
        }
    }

    fn prepare_get_project_role_binding(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let project = match self {
            InfraBootstrapOps::PrepareEnsureProjectRoleBinding { project, .. } => project,
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for prepare_get_project_role_binding",
                ))
            }
        };
        let svc = resource_manager_service(access_token);
        let req = svc.get_iam_policy(project);
        OutputMap::new()
            .request("request", req.into())
            .bool("skip", false)
            .ok()
    }

    fn check_prepare_project_role_binding(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project, role, service_account) = match self {
            InfraBootstrapOps::CheckAndPrepareProjectRoleBinding {
                project,
                role,
                service_account,
            } => (project, role, service_account),
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for check_prepare_project_role_binding",
                ))
            }
        };
        let response = required_response(&inputs)?;
        let Some(response) = response else {
            return skipped_stage(ACTION_NOOP);
        };
        let rest = as_rest_response(response)?;
        if !rest.is_success() {
            if rest.status == 403 || is_permission_denied(&rest.body) {
                return skipped_stage(ACTION_NOOP);
            }
            return Err(ExecError::new(format!(
                "project IAM read failed (status {}): {}",
                rest.status,
                body_summary(&rest.body)
            )));
        }

        let policy = rest
            .body
            .get("policy")
            .cloned()
            .unwrap_or_else(|| rest.body.clone());
        let member = format!("serviceAccount:{service_account}");
        if binding_exists(&policy, role, &member) {
            return skipped_stage(ACTION_NOOP);
        }
        let updated = policy_with_binding(policy, role, &member);
        let svc = resource_manager_service(access_token);
        let req = svc.set_iam_policy(project, updated);
        OutputMap::new()
            .request("request", req.into())
            .bool("skip", false)
            .str("action", ACTION_UPDATE)
            .ok()
    }

    fn prepare_get_sa_wif_binding(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project, service_account, _member) = match self {
            InfraBootstrapOps::PrepareEnsureSaWifBinding {
                project,
                service_account,
                member,
            } => (project, service_account, member),
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for prepare_get_sa_wif_binding",
                ))
            }
        };
        let svc = iam_service(access_token);
        let req = svc.get_service_account_iam_policy(project, service_account);
        OutputMap::new()
            .request("request", req.into())
            .bool("skip", false)
            .ok()
    }

    fn check_prepare_sa_wif_binding(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let access_token = require_str(&inputs, "access_token")?;
        let (project, service_account, member) = match self {
            InfraBootstrapOps::CheckAndPrepareSaWifBinding {
                project,
                service_account,
                member,
            } => (project, service_account, member),
            _ => {
                return Err(ExecError::new(
                    "invalid op variant for check_prepare_sa_wif_binding",
                ))
            }
        };
        let response = required_response(&inputs)?;
        let Some(response) = response else {
            return skipped_stage(ACTION_NOOP);
        };
        let rest = as_rest_response(response)?;
        if !rest.is_success() {
            if rest.status == 403 || is_permission_denied(&rest.body) {
                return skipped_stage(ACTION_NOOP);
            }
            return Err(ExecError::new(format!(
                "service-account IAM read failed (status {}): {}",
                rest.status,
                body_summary(&rest.body)
            )));
        }

        let policy = rest
            .body
            .get("policy")
            .cloned()
            .unwrap_or_else(|| rest.body.clone());
        let role = "roles/iam.workloadIdentityUser";
        if binding_exists(&policy, role, member) {
            return skipped_stage(ACTION_NOOP);
        }
        let updated = policy_with_binding(policy, role, member);
        let svc = iam_service(access_token);
        let req = svc.set_service_account_iam_policy(project, service_account, updated);
        OutputMap::new()
            .request("request", req.into())
            .bool("skip", false)
            .str("action", ACTION_UPDATE)
            .ok()
    }

    fn parse_stage(
        &self,
        inputs: HashMap<String, Value>,
        stage_name: &str,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let action = require_str(&inputs, "action")?;
        let response = required_response(&inputs)?;
        let Some(response) = response else {
            return OutputMap::new().bool("ok", true).str("action", action).ok();
        };
        let rest = as_rest_response(response)?;
        if rest.is_success() || (action == ACTION_CREATE && rest.status == 409) {
            return OutputMap::new().bool("ok", true).str("action", action).ok();
        }
        if rest.status == 403 || is_permission_denied(&rest.body) {
            return OutputMap::new().bool("ok", true).str("action", action).ok();
        }
        Err(ExecError::new(format!(
            "{} apply failed (status {}): {}",
            stage_name,
            rest.status,
            body_summary(&rest.body)
        )))
    }
}

fn workload_identity_service(access_token: &str) -> WorkloadIdentityRest {
    let cred = bearer_credential(access_token);
    WorkloadIdentityRest::new(cred)
}

fn iam_service(access_token: &str) -> IamRest {
    let cred = bearer_credential(access_token);
    IamRest::new(cred)
}

fn resource_manager_service(access_token: &str) -> ResourceManagerRest {
    let cred = bearer_credential(access_token);
    ResourceManagerRest::new(cred)
}

fn bearer_credential(access_token: &str) -> Credential {
    Credential::new(Secret::static_value(access_token), AuthScheme::Bearer)
}

fn to_provider_config(
    oidc_issuer_uri: String,
    attribute_mapping: BTreeMap<String, String>,
    attribute_condition: Option<String>,
) -> WifProviderConfig {
    WifProviderConfig {
        oidc_issuer_uri,
        attribute_mapping: attribute_mapping.into_iter().collect(),
        attribute_condition,
    }
}

fn skipped_stage(action: &str) -> Result<HashMap<String, Value>, ExecError> {
    OutputMap::new()
        .value("request", skipped_request())
        .bool("skip", true)
        .str("action", action)
        .ok()
}

fn skipped_request() -> Value {
    Value::Skipped
}

fn required_response(
    inputs: &HashMap<String, Value>,
) -> Result<Option<&TransportResponse>, ExecError> {
    match inputs.get("response") {
        Some(Value::Response(response)) => Ok(Some(response)),
        Some(Value::Skipped) => Ok(None),
        _ => Err(ExecError::new("missing or invalid 'response' input")),
    }
}

fn as_rest_response(
    response: &TransportResponse,
) -> Result<&gunbc_ir::transport::rest::RestResponse, ExecError> {
    match response {
        TransportResponse::Rest(rest) => Ok(rest),
        other => Err(ExecError::new(format!(
            "expected REST response, got {:?}",
            other
        ))),
    }
}

fn body_summary(body: &serde_json::Value) -> String {
    body.get("error")
        .and_then(serde_json::Value::as_str)
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| body.to_string())
}

fn is_permission_denied(body: &serde_json::Value) -> bool {
    let text = body.to_string();
    text.contains("PERMISSION_DENIED")
        || text.contains("permission denied")
        || text.contains("does not have")
}

fn binding_exists(policy: &serde_json::Value, role: &str, member: &str) -> bool {
    policy
        .get("bindings")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|bindings| {
            bindings.iter().any(|binding| {
                binding.get("role").and_then(serde_json::Value::as_str) == Some(role)
                    && binding
                        .get("members")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|members| {
                            members.iter().any(|entry| entry.as_str() == Some(member))
                        })
            })
        })
}

fn policy_with_binding(
    mut policy: serde_json::Value,
    role: &str,
    member: &str,
) -> serde_json::Value {
    let existing = policy
        .get("bindings")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut bindings = existing;
    let mut found_role = false;

    for binding in &mut bindings {
        if binding.get("role").and_then(serde_json::Value::as_str) == Some(role) {
            if let Some(members) = binding
                .get_mut("members")
                .and_then(serde_json::Value::as_array_mut)
            {
                members.push(serde_json::Value::String(member.to_string()));
            }
            found_role = true;
            break;
        }
    }
    if !found_role {
        bindings.push(serde_json::json!({
            "role": role,
            "members": [member],
        }));
    }

    policy["bindings"] = serde_json::Value::Array(bindings);
    policy
}

type StageNode = NodeRef<InfraBootstrapGraphOp>;

fn add_idempotent_stage(
    builder: &mut DagBuilder<InfraBootstrapGraphOp>,
    previous: &StageNode,
    context: &StageNode,
    stage_name: &str,
    prepare_op: InfraBootstrapOps,
    check_op: InfraBootstrapOps,
    parse_op: InfraBootstrapOps,
) -> Result<StageNode, BuilderError> {
    let prepare = builder.add_node_after(
        Node::opaque(
            format!("prepare_{}", stage_name).as_str(),
            vec![port("prev_ok", "Bool"), port("access_token", "String")],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            InfraBootstrapGraphOp::Bootstrap(prepare_op),
        ),
        previous,
    )?;
    builder.add_edge(previous.out("ok"), prepare.in_port("prev_ok"))?;
    builder.add_edge(context.out("access_token"), prepare.in_port("access_token"))?;

    let execute_get = builder.add_node_after(
        Node::opaque(
            format!("execute_get_{}", stage_name).as_str(),
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            InfraBootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare,
    )?;
    builder.add_edge(prepare.out("request"), execute_get.in_port("request"))?;
    builder.add_edge(prepare.out("skip"), execute_get.in_port("skip"))?;

    let check = builder.add_node_after(
        Node::opaque(
            format!("check_{}", stage_name).as_str(),
            vec![
                port("response", "TransportResponse"),
                port("access_token", "String"),
            ],
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                port("action", "String"),
            ],
            InfraBootstrapGraphOp::Bootstrap(check_op),
        ),
        &execute_get,
    )?;
    builder.add_edge(execute_get.out("response"), check.in_port("response"))?;
    builder.add_edge(context.out("access_token"), check.in_port("access_token"))?;

    let execute_apply = builder.add_node_after(
        Node::opaque(
            format!("execute_apply_{}", stage_name).as_str(),
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            InfraBootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &check,
    )?;
    builder.add_edge(check.out("request"), execute_apply.in_port("request"))?;
    builder.add_edge(check.out("skip"), execute_apply.in_port("skip"))?;

    let parse = builder.add_node_after(
        Node::opaque(
            format!("parse_{}", stage_name).as_str(),
            vec![
                port("response", "TransportResponse"),
                port("action", "String"),
            ],
            vec![port("ok", "Bool"), port("action", "String")],
            InfraBootstrapGraphOp::Bootstrap(parse_op),
        ),
        &execute_apply,
    )?;
    builder.add_edge(execute_apply.out("response"), parse.in_port("response"))?;
    builder.add_edge(check.out("action"), parse.in_port("action"))?;

    Ok(parse)
}

fn sanitize_node_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

/// Build the WIF bootstrap DAG for one infra environment.
///
/// Entrypoint:
/// - `access_token`: bearer token with IAM/Resource Manager permissions.
pub fn build_wif_bootstrap_dag(
    infra_spec: &InfraSpec,
) -> Result<Dag<InfraBootstrapGraphOp>, String> {
    infra_spec.validate()?;

    let mut builder: DagBuilder<InfraBootstrapGraphOp> = DagBuilder::new();
    let context = builder
        .add_root_node(Node::opaque(
            "context",
            vec![port("access_token", "String")],
            vec![port("access_token", "String")],
            InfraBootstrapGraphOp::Bootstrap(InfraBootstrapOps::PassAccessToken),
        ))
        .map_err(|err| format!("failed to add context node: {err}"))?;

    let enable_apis = builder
        .add_node_after(
            Node::opaque(
                "enable_apis",
                vec![port("access_token", "String")],
                vec![port("ok", "Bool"), port("note", "String")],
                InfraBootstrapGraphOp::Bootstrap(InfraBootstrapOps::EnableApis {
                    project: infra_spec.config.secrets_project.to_string(),
                    services: vec![
                        "iam.googleapis.com".to_string(),
                        "iamcredentials.googleapis.com".to_string(),
                        "cloudresourcemanager.googleapis.com".to_string(),
                    ],
                }),
            ),
            &context,
        )
        .map_err(|err| format!("failed to add enable_apis node: {err}"))?;
    builder
        .add_edge(
            context.out("access_token"),
            enable_apis.in_port("access_token"),
        )
        .map_err(|err| format!("failed to wire enable_apis: {err}"))?;

    let mut tail = add_idempotent_stage(
        &mut builder,
        &enable_apis,
        &context,
        "wif_pool",
        InfraBootstrapOps::PrepareEnsureWifPool {
            project_number: infra_spec.wif.project_number.to_string(),
            pool_id: infra_spec.wif.pool_id.to_string(),
            display_name: format!("{} pool", infra_spec.wif.pool_id),
        },
        InfraBootstrapOps::CheckAndPrepareWifPool {
            project_number: infra_spec.wif.project_number.to_string(),
            pool_id: infra_spec.wif.pool_id.to_string(),
            display_name: format!("{} pool", infra_spec.wif.pool_id),
        },
        InfraBootstrapOps::ParseEnsureWifPool,
    )
    .map_err(|err| format!("failed to add wif_pool stage: {err}"))?;

    let mapping: BTreeMap<String, String> = infra_spec
        .wif
        .attribute_mapping
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    tail = add_idempotent_stage(
        &mut builder,
        &tail,
        &context,
        "wif_provider",
        InfraBootstrapOps::PrepareEnsureWifProvider {
            project_number: infra_spec.wif.project_number.to_string(),
            pool_id: infra_spec.wif.pool_id.to_string(),
            provider_id: infra_spec.wif.provider_id.to_string(),
            oidc_issuer_uri: infra_spec.wif.oidc_issuer_uri.to_string(),
            attribute_mapping: mapping.clone(),
            attribute_condition: infra_spec
                .wif
                .attribute_condition
                .map(std::string::ToString::to_string),
        },
        InfraBootstrapOps::CheckAndPrepareWifProvider {
            project_number: infra_spec.wif.project_number.to_string(),
            pool_id: infra_spec.wif.pool_id.to_string(),
            provider_id: infra_spec.wif.provider_id.to_string(),
            oidc_issuer_uri: infra_spec.wif.oidc_issuer_uri.to_string(),
            attribute_mapping: mapping,
            attribute_condition: infra_spec
                .wif
                .attribute_condition
                .map(std::string::ToString::to_string),
        },
        InfraBootstrapOps::ParseEnsureWifProvider,
    )
    .map_err(|err| format!("failed to add wif_provider stage: {err}"))?;

    let project = infra_spec.config.secrets_project.to_string();
    for service_account in infra_spec.service_accounts {
        let sa_id = sanitize_node_id(service_account.name);
        let sa_email = service_account.email(infra_spec.config.secrets_project);

        tail = add_idempotent_stage(
            &mut builder,
            &tail,
            &context,
            format!("sa_{}", sa_id).as_str(),
            InfraBootstrapOps::PrepareEnsureServiceAccount {
                project: project.clone(),
                account_id: service_account.name.to_string(),
                email: sa_email.clone(),
                display_name: service_account.display_name.to_string(),
            },
            InfraBootstrapOps::CheckAndPrepareServiceAccount {
                project: project.clone(),
                account_id: service_account.name.to_string(),
                email: sa_email.clone(),
                display_name: service_account.display_name.to_string(),
            },
            InfraBootstrapOps::ParseEnsureServiceAccount,
        )
        .map_err(|err| format!("failed to add service account stage: {err}"))?;

        for (index, role) in service_account.self_roles.iter().enumerate() {
            tail = add_idempotent_stage(
                &mut builder,
                &tail,
                &context,
                format!("role_{}_{}", sa_id, index).as_str(),
                InfraBootstrapOps::PrepareEnsureProjectRoleBinding {
                    project: project.clone(),
                    role: role.to_string(),
                    service_account: sa_email.clone(),
                },
                InfraBootstrapOps::CheckAndPrepareProjectRoleBinding {
                    project: project.clone(),
                    role: role.to_string(),
                    service_account: sa_email.clone(),
                },
                InfraBootstrapOps::ParseEnsureProjectRoleBinding,
            )
            .map_err(|err| format!("failed to add project role stage: {err}"))?;
        }

        for (index, member) in service_account.wif_bindings.iter().enumerate() {
            tail = add_idempotent_stage(
                &mut builder,
                &tail,
                &context,
                format!("wif_{}_{}", sa_id, index).as_str(),
                InfraBootstrapOps::PrepareEnsureSaWifBinding {
                    project: project.clone(),
                    service_account: sa_email.clone(),
                    member: member.to_string(),
                },
                InfraBootstrapOps::CheckAndPrepareSaWifBinding {
                    project: project.clone(),
                    service_account: sa_email.clone(),
                    member: member.to_string(),
                },
                InfraBootstrapOps::ParseEnsureSaWifBinding,
            )
            .map_err(|err| format!("failed to add SA WIF binding stage: {err}"))?;
        }
    }

    let summary = builder
        .add_node_after(
            Node::opaque(
                "bootstrap_summary",
                vec![port("prev_ok", "Bool")],
                vec![port("ok", "Bool"), port("report", "String")],
                InfraBootstrapGraphOp::Bootstrap(InfraBootstrapOps::SummarizeBootstrap {
                    environment: infra_spec.environment.to_string(),
                    project: project.clone(),
                }),
            ),
            &tail,
        )
        .map_err(|err| format!("failed to add summary stage: {err}"))?;
    builder
        .add_edge(tail.out("ok"), summary.in_port("prev_ok"))
        .map_err(|err| format!("failed to wire summary stage: {err}"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra_spec::DEV_SPEC;
    use gunbc_ir::transport::rest::RestResponse;

    fn rest_value(status: u16, body: serde_json::Value) -> Value {
        Value::Response(TransportResponse::Rest(RestResponse::new(status, body)))
    }

    #[test]
    fn bootstrap_graph_contains_core_stages() {
        let dag = build_wif_bootstrap_dag(&DEV_SPEC).expect("bootstrap dag should build");
        let ids: std::collections::HashSet<&str> =
            dag.nodes.iter().map(|node| node.id.0.as_str()).collect();

        assert!(ids.contains("enable_apis"));
        assert!(ids.contains("parse_wif_pool"));
        assert!(ids.contains("parse_wif_provider"));
        assert!(ids.contains("bootstrap_summary"));
        assert!(ids.contains("parse_sa_gunbai_dev_secrets"));
        assert!(ids.contains("parse_role_gunbai_dev_secrets_0"));
        assert!(ids.contains("parse_wif_gunbai_dev_secrets_0"));
    }

    #[test]
    fn check_pool_requests_create_when_missing() {
        let op = InfraBootstrapOps::CheckAndPrepareWifPool {
            project_number: "123".to_string(),
            pool_id: "github-pool".to_string(),
            display_name: "GitHub Pool".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("access_token".to_string(), Value::Str("tok".to_string()));
        inputs.insert(
            "response".to_string(),
            rest_value(404, serde_json::json!({"error": "not found"})),
        );

        let out = op.execute(inputs).expect("check pool should succeed");
        assert_eq!(out.get("skip"), Some(&Value::Bool(false)));
        assert_eq!(
            out.get("action").and_then(Value::as_str),
            Some(ACTION_CREATE)
        );
        let req = out
            .get("request")
            .and_then(Value::as_request)
            .expect("request should be emitted");
        let rest = match req {
            gunbc_ir::transport::TransportRequest::Rest(rest) => rest,
            other => panic!("expected REST request, got {:?}", other),
        };
        assert!(
            rest.url.contains("workloadIdentityPoolId=github-pool"),
            "create pool request should include pool id"
        );
    }

    #[test]
    fn check_provider_updates_when_present() {
        let op = InfraBootstrapOps::CheckAndPrepareWifProvider {
            project_number: "123".to_string(),
            pool_id: "github-pool".to_string(),
            provider_id: "github".to_string(),
            oidc_issuer_uri: "https://token.actions.githubusercontent.com".to_string(),
            attribute_mapping: BTreeMap::from([
                ("google.subject".to_string(), "assertion.sub".to_string()),
                (
                    "attribute.repository".to_string(),
                    "assertion.repository".to_string(),
                ),
            ]),
            attribute_condition: Some("assertion.repository == \"gunb-ai/gunbc\"".to_string()),
        };
        let mut inputs = HashMap::new();
        inputs.insert("access_token".to_string(), Value::Str("tok".to_string()));
        inputs.insert(
            "response".to_string(),
            rest_value(200, serde_json::json!({})),
        );

        let out = op.execute(inputs).expect("check provider should succeed");
        assert_eq!(
            out.get("action").and_then(Value::as_str),
            Some(ACTION_UPDATE)
        );
        assert_eq!(out.get("skip"), Some(&Value::Bool(false)));
    }

    #[test]
    fn check_service_account_skips_when_display_name_matches() {
        let op = InfraBootstrapOps::CheckAndPrepareServiceAccount {
            project: "gunbai-secrets".to_string(),
            account_id: "gunbai-dev-secrets".to_string(),
            email: "gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string(),
            display_name: "gunbc dev secrets".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("access_token".to_string(), Value::Str("tok".to_string()));
        inputs.insert(
            "response".to_string(),
            rest_value(
                200,
                serde_json::json!({
                    "displayName": "gunbc dev secrets"
                }),
            ),
        );

        let out = op.execute(inputs).expect("check sa should succeed");
        assert_eq!(out.get("skip"), Some(&Value::Bool(true)));
        assert_eq!(out.get("action").and_then(Value::as_str), Some(ACTION_NOOP));
    }
}
