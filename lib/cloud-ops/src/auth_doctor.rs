//! Provider-neutral cloud auth doctor.
//!
//! Builds a diagnostic DAG that any credentialed tool can use to report
//! environment readiness. The tool passes its `CredentialIntent` to
//! parameterize the report.

use crate::env_requirements::{collect_missing_requirements, detect_cloud_env_requirements};
use crate::env_status::CloudEnvStatus;
use gunbc_exec::{require_str, ExecError, Executable, OutputMap};
use gunbc_ir::build::{list, port};
use gunbc_ir::transport::cloud::CloudRuntimeKind;
use gunbc_ir::transport::scope::CredentialIntent;
use gunbc_ir::{BuilderError, Dag, DagBuilder, Node, Value};
use std::collections::HashMap;

/// Op enum for the cloud auth doctor graph.
#[derive(Debug, Clone)]
pub enum CloudAuthDoctorOp {
    /// Environment status node.
    CloudStatus(CloudEnvStatus),
    /// Build the diagnostic report.
    BuildReport {
        intent: CredentialIntent,
        runtime_hint: Option<String>,
    },
}

impl Executable for CloudAuthDoctorOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CloudAuthDoctorOp::CloudStatus(op) => op.execute(inputs),
            CloudAuthDoctorOp::BuildReport {
                intent,
                runtime_hint,
            } => execute_build_report(inputs, intent, runtime_hint.as_deref()),
        }
    }
}

fn execute_build_report(
    inputs: HashMap<String, Value>,
    intent: &CredentialIntent,
    runtime_hint: Option<&str>,
) -> Result<HashMap<String, Value>, ExecError> {
    let status = require_str(&inputs, "status")?;

    let mut env_req = detect_cloud_env_requirements();
    if let Some(hint) = runtime_hint {
        let runtime = CloudRuntimeKind::parse(hint)
            .ok_or_else(|| ExecError::new(format!("invalid runtime_hint '{hint}'")))?;
        env_req = crate::env_requirements::requirements_for(env_req.provider, runtime);
    }

    intent
        .validate()
        .map_err(|e| ExecError::new(format!("invalid scope contract: {e}")))?;

    let required_env: Vec<String> = env_req.required.iter().map(|v| (*v).to_string()).collect();
    let missing = collect_missing_requirements(&env_req);
    let missing_env: Vec<String> = missing
        .missing_required
        .iter()
        .map(|v| (*v).to_string())
        .collect();

    let required_any_of: Vec<String> = env_req
        .required_any_of
        .iter()
        .map(|group| group.join(" | "))
        .collect();
    let missing_any_of: Vec<String> = missing
        .missing_any_of
        .iter()
        .map(|group| group.join(" | "))
        .collect();

    let secret_prefix =
        std::env::var("GCP_SECRETS_PREFIX").unwrap_or_else(|_| "<GCP_SECRETS_PREFIX>".to_string());
    let secret_name = format!("{secret_prefix}{}", intent.service);

    let local_setup = if matches!(env_req.runtime, CloudRuntimeKind::LocalDev) {
        "login-on-demand via credential upsert (manual fallback: gcloud auth login --update-adc)"
    } else {
        "n/a"
    };

    let ready = missing_env.is_empty() && missing_any_of.is_empty();

    OutputMap::new()
        .str("status", status)
        .str("provider", env_req.provider.as_str())
        .str("runtime", env_req.runtime.as_str())
        .str_list("required_env", required_env)
        .str_list("missing_env", missing_env)
        .str_list("required_any_of", required_any_of)
        .str_list("missing_any_of", missing_any_of)
        .str("service", &intent.service)
        .str_list("required_scopes", intent.required_scopes.clone())
        .str("secret_name", secret_name)
        .str(
            "acquisition_flow",
            "resolve_auth_contract -> cloud_env -> bind_secret -> cloud_credential(local: check_auth -> create_auth_if_needed -> resolve_token) -> execute(res:credential)",
        )
        .str("local_setup", local_setup)
        .bool("ready", ready)
        .ok()
}

/// Build a provider-neutral auth doctor graph for the given credential intent.
///
/// The `intent` specifies which service and scopes are being diagnosed.
/// The `runtime_hint` allows overriding the detected runtime (e.g., from CLI args).
pub fn build_cloud_auth_doctor_graph(
    intent: CredentialIntent,
    runtime_hint: Option<String>,
) -> Result<Dag<CloudAuthDoctorOp>, BuilderError> {
    let mut builder: DagBuilder<CloudAuthDoctorOp> = DagBuilder::new();

    let cloud_status = builder.add_root_node(Node::opaque(
        "cloud_status",
        vec![],
        vec![port("status", "String")],
        CloudAuthDoctorOp::CloudStatus(CloudEnvStatus::new()),
    ))?;

    let auth_report = builder.add_node_after(
        Node::opaque(
            "auth_report",
            vec![port("status", "String")],
            vec![
                port("status", "String"),
                port("provider", "String"),
                port("runtime", "String"),
                list("required_env", "String"),
                list("missing_env", "String"),
                list("required_any_of", "String"),
                list("missing_any_of", "String"),
                port("service", "String"),
                list("required_scopes", "String"),
                port("secret_name", "String"),
                port("acquisition_flow", "String"),
                port("local_setup", "String"),
                port("ready", "Bool"),
            ],
            CloudAuthDoctorOp::BuildReport {
                intent,
                runtime_hint,
            },
        ),
        &cloud_status,
    )?;

    builder.add_edge(cloud_status.out("status"), auth_report.in_port("status"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::{execute_with_mode, ExecutionMode};

    #[test]
    fn test_cloud_auth_doctor_runs_with_gist_intent() {
        let intent = CredentialIntent::new("github", "github", "bearer")
            .with_required_scopes(["gist:write"]);
        let dag = build_cloud_auth_doctor_graph(intent, None)
            .expect("auth doctor graph should build");
        let log = execute_with_mode(&dag, ExecutionMode::DryRun(Default::default()))
            .expect("auth doctor graph should execute");
        let report = log.get("auth_report").expect("auth_report should run");
        assert!(report.outputs.contains_key("required_scopes"));
        assert_eq!(
            report.outputs.get("service"),
            Some(&Value::Str("github".to_string()))
        );
    }

    #[test]
    fn test_cloud_auth_doctor_runs_with_llm_intent() {
        let intent = CredentialIntent::new("openai", "openai", "bearer")
            .with_required_scopes(["llm:chat_completion"]);
        let dag = build_cloud_auth_doctor_graph(intent, None)
            .expect("auth doctor graph should build");
        let log = execute_with_mode(&dag, ExecutionMode::DryRun(Default::default()))
            .expect("auth doctor graph should execute");
        let report = log.get("auth_report").expect("auth_report should run");
        assert_eq!(
            report.outputs.get("service"),
            Some(&Value::Str("openai".to_string()))
        );
    }
}
