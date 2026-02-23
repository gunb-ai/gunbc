//! Provider-neutral cloud credential DAGs.

use crate::ops::CloudOps;
use gunbc_exec::DynOp;
use gunbc_ir::build::{list, optional, port};
use gunbc_ir::transport::cloud::{CloudProviderKind, CloudRuntimeKind, CloudSecretConfig};
use gunbc_ir::{BuilderError, Dag, DagBuilder, Node};
use gunbc_lib_aws_ops::{
    build_aws_secrets_manager_credential_graph, build_aws_secrets_manager_upsert_graph,
};
use gunbc_lib_azure_ops::{
    build_azure_key_vault_credential_graph, build_azure_key_vault_upsert_graph,
};
use gunbc_lib_gcp_ops::{
    build_gcp_secret_manager_credential_graph_github,
    build_gcp_secret_manager_credential_graph_local,
    build_gcp_secret_manager_credential_graph_metadata,
    build_gcp_secret_manager_upsert_graph_github, build_gcp_secret_manager_upsert_graph_local,
    build_gcp_secret_manager_upsert_graph_metadata,
};

pub type CloudSecretManagerGraphOp = DynOp;

// ---------------------------------------------------------------------------
// Public builders
// ---------------------------------------------------------------------------

/// Build a cloud credential graph based on a concrete config.
pub fn build_cloud_secret_manager_credential_graph_from_config(
    config: &CloudSecretConfig,
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    match config.provider {
        CloudProviderKind::Gcp => match config.runtime {
            CloudRuntimeKind::GitHubActions => {
                build_cloud_secret_manager_credential_graph_gcp_github()
            }
            CloudRuntimeKind::CloudMetadata => {
                build_cloud_secret_manager_credential_graph_gcp_metadata()
            }
            CloudRuntimeKind::LocalDev => build_cloud_secret_manager_credential_graph_gcp_local(),
        },
        CloudProviderKind::Aws => build_cloud_secret_manager_credential_graph_aws_stub(),
        CloudProviderKind::Azure => build_cloud_secret_manager_credential_graph_azure_stub(),
    }
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-credential-gcp-github",
    builder = "build_cloud_secret_manager_credential_graph_gcp_github()",
    returns_result
)]
pub fn build_cloud_secret_manager_credential_graph_gcp_github(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    build_cloud_secret_manager_credential_graph_gcp(CloudRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-credential-gcp-metadata",
    builder = "build_cloud_secret_manager_credential_graph_gcp_metadata()",
    returns_result
)]
pub fn build_cloud_secret_manager_credential_graph_gcp_metadata(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    build_cloud_secret_manager_credential_graph_gcp(CloudRuntimeKind::CloudMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-credential-gcp-local",
    builder = "build_cloud_secret_manager_credential_graph_gcp_local()",
    returns_result
)]
pub fn build_cloud_secret_manager_credential_graph_gcp_local(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    build_cloud_secret_manager_credential_graph_gcp(CloudRuntimeKind::LocalDev)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-credential-aws-stub",
    builder = "build_cloud_secret_manager_credential_graph_aws_stub()",
    returns_result
)]
pub fn build_cloud_secret_manager_credential_graph_aws_stub(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    Ok(lift_aws(build_aws_secrets_manager_credential_graph()?))
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-credential-azure-stub",
    builder = "build_cloud_secret_manager_credential_graph_azure_stub()",
    returns_result
)]
pub fn build_cloud_secret_manager_credential_graph_azure_stub(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    Ok(lift_azure(build_azure_key_vault_credential_graph()?))
}

/// Build a cloud secret upsert graph based on a concrete config.
pub fn build_cloud_secret_manager_upsert_graph_from_config(
    config: &CloudSecretConfig,
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    match config.provider {
        CloudProviderKind::Gcp => match config.runtime {
            CloudRuntimeKind::GitHubActions => build_cloud_secret_manager_upsert_graph_gcp_github(),
            CloudRuntimeKind::CloudMetadata => {
                build_cloud_secret_manager_upsert_graph_gcp_metadata()
            }
            CloudRuntimeKind::LocalDev => build_cloud_secret_manager_upsert_graph_gcp_local(),
        },
        CloudProviderKind::Aws => build_cloud_secret_manager_upsert_graph_aws_stub(),
        CloudProviderKind::Azure => build_cloud_secret_manager_upsert_graph_azure_stub(),
    }
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-upsert-gcp-github",
    builder = "build_cloud_secret_manager_upsert_graph_gcp_github()",
    returns_result
)]
pub fn build_cloud_secret_manager_upsert_graph_gcp_github(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    build_cloud_secret_manager_upsert_graph_gcp(CloudRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-upsert-gcp-metadata",
    builder = "build_cloud_secret_manager_upsert_graph_gcp_metadata()",
    returns_result
)]
pub fn build_cloud_secret_manager_upsert_graph_gcp_metadata(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    build_cloud_secret_manager_upsert_graph_gcp(CloudRuntimeKind::CloudMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-upsert-gcp-local",
    builder = "build_cloud_secret_manager_upsert_graph_gcp_local()",
    returns_result
)]
pub fn build_cloud_secret_manager_upsert_graph_gcp_local(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    build_cloud_secret_manager_upsert_graph_gcp(CloudRuntimeKind::LocalDev)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-upsert-aws-stub",
    builder = "build_cloud_secret_manager_upsert_graph_aws_stub()",
    returns_result
)]
pub fn build_cloud_secret_manager_upsert_graph_aws_stub(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    Ok(lift_aws(build_aws_secrets_manager_upsert_graph()?))
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "cloud-secret-upsert-azure-stub",
    builder = "build_cloud_secret_manager_upsert_graph_azure_stub()",
    returns_result
)]
pub fn build_cloud_secret_manager_upsert_graph_azure_stub(
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    Ok(lift_azure(build_azure_key_vault_upsert_graph()?))
}

// ---------------------------------------------------------------------------
// Internal builders
// ---------------------------------------------------------------------------

fn build_cloud_secret_manager_credential_graph_gcp(
    runtime: CloudRuntimeKind,
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    let gcp_dag = match runtime {
        CloudRuntimeKind::GitHubActions => build_gcp_secret_manager_credential_graph_github()?,
        CloudRuntimeKind::CloudMetadata => build_gcp_secret_manager_credential_graph_metadata()?,
        CloudRuntimeKind::LocalDev => build_gcp_secret_manager_credential_graph_local()?,
    };
    let gcp_subdag = lift_gcp(gcp_dag);

    let mut builder: DagBuilder<CloudSecretManagerGraphOp> = DagBuilder::new();

    let resolve_config = builder.add_root_node(Node::opaque(
        "resolve_config",
        vec![port("config", "CloudSecretConfig")],
        vec![
            port("provider", "Platform"),
            port("runtime", "String"),
            port("audience", "NonEmptyString"),
            port("project_or_account", "String"),
            port("secret", "String"),
            optional("version", "OptionalString"),
            optional("service_account_or_role", "OptionalString"),
            optional("impersonate_account_or_role", "OptionalString"),
        ],
        DynOp::new(CloudOps::ResolveConfig),
    ))?;

    let mut map_outputs = vec![
        port("project", "GcpProjectId"),
        port("secret", "String"),
        optional("version", "OptionalString"),
        port("service_account", "GcpServiceAccountEmail"),
        port("scheme", "String"),
        optional("header_name", "OptionalString"),
        port("source_id", "String"),
        list("required_scopes", "String"),
        optional("allow_impersonation", "OptionalBool"),
        optional("lifetime_seconds", "OptionalInt"),
    ];
    // interactive_allowed is only needed for non-local runtimes where
    // it gets wired to the GCP sub-DAG. For LocalDev (ADC-based auth),
    // it's unused — excluding it prevents a dangling output that would
    // propagate up through nested sub-DAGs.
    if !matches!(runtime, CloudRuntimeKind::LocalDev) {
        map_outputs.push(optional("interactive_allowed", "OptionalBool"));
        map_outputs.push(port("audience", "NonEmptyString"));
    }
    if matches!(runtime, CloudRuntimeKind::GitHubActions) {
        map_outputs.push(optional("request_url", "OptionalString"));
        map_outputs.push(optional("request_token", "OptionalString"));
    }

    let map_inputs = builder.add_node_after(
        Node::opaque(
            "map_gcp_inputs",
            vec![
                port("provider", "Platform"),
                port("runtime", "String"),
                port("audience", "NonEmptyString"),
                port("project_or_account", "String"),
                port("secret", "String"),
                optional("version", "OptionalString"),
                optional("service_account_or_role", "OptionalString"),
                optional("impersonate_account_or_role", "OptionalString"),
                // Pass-through inputs for the GCP graph.
                port("scheme", "String"),
                optional("header_name", "OptionalString"),
                port("source_id", "String"),
                list("required_scopes", "String"),
                optional("allow_impersonation", "OptionalBool"),
                optional("lifetime_seconds", "OptionalInt"),
                // interactive_allowed is accepted as input for all runtimes
                // (parent graphs always wire it), but only OUTPUT for
                // non-LocalDev runtimes where it gets wired to the GCP sub-DAG.
                optional("interactive_allowed", "OptionalBool"),
                optional("request_url", "OptionalString"),
                optional("request_token", "OptionalString"),
            ],
            map_outputs,
            DynOp::new(CloudOps::MapToGcpInputs { runtime }),
        ),
        &resolve_config,
    )?;

    // Wire resolved config → map node.
    builder.add_edge(
        resolve_config.out("provider"),
        map_inputs.in_port("provider"),
    )?;
    builder.add_edge(resolve_config.out("runtime"), map_inputs.in_port("runtime"))?;
    builder.add_edge(
        resolve_config.out("audience"),
        map_inputs.in_port("audience"),
    )?;
    builder.add_edge(
        resolve_config.out("project_or_account"),
        map_inputs.in_port("project_or_account"),
    )?;
    builder.add_edge(resolve_config.out("secret"), map_inputs.in_port("secret"))?;
    builder.add_edge(resolve_config.out("version"), map_inputs.in_port("version"))?;
    builder.add_edge(
        resolve_config.out("service_account_or_role"),
        map_inputs.in_port("service_account_or_role"),
    )?;
    builder.add_edge(
        resolve_config.out("impersonate_account_or_role"),
        map_inputs.in_port("impersonate_account_or_role"),
    )?;

    // GCP subdag.
    let gcp_node =
        builder.add_node_after(Node::subdag("gcp_wif_secret", gcp_subdag), &map_inputs)?;

    // Wire map outputs → GCP graph inputs.
    if !matches!(runtime, CloudRuntimeKind::LocalDev) {
        builder.add_edge(map_inputs.out("audience"), gcp_node.in_port("audience"))?;
    }
    builder.add_edge(map_inputs.out("project"), gcp_node.in_port("project"))?;
    builder.add_edge(map_inputs.out("secret"), gcp_node.in_port("secret"))?;
    builder.add_edge(map_inputs.out("version"), gcp_node.in_port("version"))?;
    builder.add_edge(
        map_inputs.out("service_account"),
        gcp_node.in_port("service_account"),
    )?;
    builder.add_edge(map_inputs.out("scheme"), gcp_node.in_port("scheme"))?;
    builder.add_edge(
        map_inputs.out("header_name"),
        gcp_node.in_port("header_name"),
    )?;
    builder.add_edge(map_inputs.out("source_id"), gcp_node.in_port("source_id"))?;
    builder.add_edge(
        map_inputs.out("required_scopes"),
        gcp_node.in_port("required_scopes"),
    )?;
    builder.add_edge(
        map_inputs.out("allow_impersonation"),
        gcp_node.in_port("allow_impersonation"),
    )?;
    builder.add_edge(
        map_inputs.out("lifetime_seconds"),
        gcp_node.in_port("lifetime_seconds"),
    )?;
    // Note: interactive_allowed is no longer wired to the GCP subdag for LocalDev
    // because the new ADC-based local auth flow doesn't use it.

    if matches!(runtime, CloudRuntimeKind::GitHubActions) {
        builder.add_edge(
            map_inputs.out("request_url"),
            gcp_node.in_port("request_url"),
        )?;
        builder.add_edge(
            map_inputs.out("request_token"),
            gcp_node.in_port("request_token"),
        )?;
    }

    Ok(builder.build())
}

fn build_cloud_secret_manager_upsert_graph_gcp(
    runtime: CloudRuntimeKind,
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    let gcp_dag = match runtime {
        CloudRuntimeKind::GitHubActions => build_gcp_secret_manager_upsert_graph_github()?,
        CloudRuntimeKind::CloudMetadata => build_gcp_secret_manager_upsert_graph_metadata()?,
        CloudRuntimeKind::LocalDev => build_gcp_secret_manager_upsert_graph_local()?,
    };
    let gcp_subdag = lift_gcp(gcp_dag);

    let mut builder: DagBuilder<CloudSecretManagerGraphOp> = DagBuilder::new();

    let resolve_config = builder.add_root_node(Node::opaque(
        "resolve_config",
        vec![port("config", "CloudSecretConfig")],
        vec![
            port("provider", "Platform"),
            port("runtime", "String"),
            port("audience", "NonEmptyString"),
            port("project_or_account", "String"),
            port("secret", "String"),
            optional("version", "OptionalString"),
            optional("service_account_or_role", "OptionalString"),
            optional("impersonate_account_or_role", "OptionalString"),
        ],
        DynOp::new(CloudOps::ResolveConfig),
    ))?;

    let mut map_outputs = vec![
        port("project", "GcpProjectId"),
        port("secret", "String"),
        port("service_account", "GcpServiceAccountEmail"),
        optional("version", "OptionalString"),
        optional("allow_impersonation", "OptionalBool"),
        optional("lifetime_seconds", "OptionalInt"),
    ];
    // interactive_allowed only needed for non-local runtimes (see credential graph).
    if !matches!(runtime, CloudRuntimeKind::LocalDev) {
        map_outputs.push(optional("interactive_allowed", "OptionalBool"));
        map_outputs.push(port("audience", "NonEmptyString"));
    }
    if matches!(runtime, CloudRuntimeKind::GitHubActions) {
        map_outputs.push(optional("request_url", "OptionalString"));
        map_outputs.push(optional("request_token", "OptionalString"));
    }

    let map_inputs = builder.add_node_after(
        Node::opaque(
            "map_gcp_secret_inputs",
            vec![
                port("provider", "Platform"),
                port("runtime", "String"),
                port("audience", "NonEmptyString"),
                port("project_or_account", "String"),
                port("secret", "String"),
                optional("version", "OptionalString"),
                optional("service_account_or_role", "OptionalString"),
                optional("impersonate_account_or_role", "OptionalString"),
                optional("allow_impersonation", "OptionalBool"),
                optional("lifetime_seconds", "OptionalInt"),
                optional("interactive_allowed", "OptionalBool"),
                optional("request_url", "OptionalString"),
                optional("request_token", "OptionalString"),
            ],
            map_outputs,
            DynOp::new(CloudOps::MapToGcpSecretInputs { runtime }),
        ),
        &resolve_config,
    )?;

    builder.add_edge(
        resolve_config.out("provider"),
        map_inputs.in_port("provider"),
    )?;
    builder.add_edge(resolve_config.out("runtime"), map_inputs.in_port("runtime"))?;
    builder.add_edge(
        resolve_config.out("audience"),
        map_inputs.in_port("audience"),
    )?;
    builder.add_edge(
        resolve_config.out("project_or_account"),
        map_inputs.in_port("project_or_account"),
    )?;
    builder.add_edge(resolve_config.out("secret"), map_inputs.in_port("secret"))?;
    builder.add_edge(resolve_config.out("version"), map_inputs.in_port("version"))?;
    builder.add_edge(
        resolve_config.out("service_account_or_role"),
        map_inputs.in_port("service_account_or_role"),
    )?;
    builder.add_edge(
        resolve_config.out("impersonate_account_or_role"),
        map_inputs.in_port("impersonate_account_or_role"),
    )?;

    let gcp_node = builder.add_node_after(
        Node::subdag("gcp_wif_secret_upsert", gcp_subdag),
        &map_inputs,
    )?;

    if !matches!(runtime, CloudRuntimeKind::LocalDev) {
        builder.add_edge(map_inputs.out("audience"), gcp_node.in_port("audience"))?;
    }
    builder.add_edge(map_inputs.out("project"), gcp_node.in_port("project"))?;
    builder.add_edge(map_inputs.out("secret"), gcp_node.in_port("secret"))?;
    builder.add_edge(
        map_inputs.out("service_account"),
        gcp_node.in_port("service_account"),
    )?;
    builder.add_edge(
        map_inputs.out("allow_impersonation"),
        gcp_node.in_port("allow_impersonation"),
    )?;
    builder.add_edge(
        map_inputs.out("lifetime_seconds"),
        gcp_node.in_port("lifetime_seconds"),
    )?;
    // Note: interactive_allowed is no longer wired to the GCP subdag for LocalDev
    // because the new ADC-based local auth flow doesn't use it.

    if matches!(runtime, CloudRuntimeKind::GitHubActions) {
        builder.add_edge(
            map_inputs.out("request_url"),
            gcp_node.in_port("request_url"),
        )?;
        builder.add_edge(
            map_inputs.out("request_token"),
            gcp_node.in_port("request_token"),
        )?;
    }

    Ok(builder.build())
}

// ---------------------------------------------------------------------------
// DAG lifting helpers — identity since all provider types are now DynOp
// ---------------------------------------------------------------------------

fn lift_gcp(dag: Dag<DynOp>) -> Dag<CloudSecretManagerGraphOp> {
    dag
}

fn lift_aws(dag: Dag<DynOp>) -> Dag<CloudSecretManagerGraphOp> {
    dag
}

fn lift_azure(dag: Dag<DynOp>) -> Dag<CloudSecretManagerGraphOp> {
    dag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_local_credential_graph_does_not_panic() {
        let _dag = build_cloud_secret_manager_credential_graph_gcp_local().unwrap();
    }

    #[test]
    fn build_local_upsert_graph_does_not_panic() {
        let _dag = build_cloud_secret_manager_upsert_graph_gcp_local().unwrap();
    }

    #[test]
    fn local_cloud_credential_exposes_expires_in() {
        let dag = build_cloud_secret_manager_credential_graph_gcp_local().unwrap();
        let node = dag
            .get_node(&"gcp_wif_secret".into())
            .expect("gcp_wif_secret node should exist");
        assert!(
            node.outputs.iter().any(|port| port.name.0 == "expires_in"),
            "local cloud credential graph must expose expires_in for runtime-uniform contracts",
        );
    }
}
