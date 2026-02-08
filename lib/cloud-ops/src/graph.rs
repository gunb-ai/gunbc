//! Provider-neutral cloud credential DAGs.

use crate::ops::CloudOps;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::build::{optional, port};
use gunbc_ir::transport::cloud::{CloudProviderKind, CloudRuntimeKind, CloudSecretConfig};
use gunbc_ir::{Dag, DagBuilder, Node, NodeBody, Value};
use gunbc_lib_aws_ops::{
    build_aws_secrets_manager_credential_graph, build_aws_secrets_manager_upsert_graph,
    AwsSecretManagerGraphOp,
};
use gunbc_lib_azure_ops::{
    build_azure_key_vault_credential_graph, build_azure_key_vault_upsert_graph,
    AzureKeyVaultGraphOp,
};
use gunbc_lib_gcp_ops::{
    build_gcp_secret_manager_credential_graph_github,
    build_gcp_secret_manager_credential_graph_local,
    build_gcp_secret_manager_credential_graph_metadata,
    build_gcp_secret_manager_upsert_graph_github,
    build_gcp_secret_manager_upsert_graph_local,
    build_gcp_secret_manager_upsert_graph_metadata, GcpSecretManagerGraphOp,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CloudSecretManagerGraphOp {
    Cloud(CloudOps),
    Gcp(GcpSecretManagerGraphOp),
    Aws(AwsSecretManagerGraphOp),
    Azure(AzureKeyVaultGraphOp),
}

impl Executable for CloudSecretManagerGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CloudSecretManagerGraphOp::Cloud(op) => op.execute(inputs),
            CloudSecretManagerGraphOp::Gcp(op) => op.execute(inputs),
            CloudSecretManagerGraphOp::Aws(op) => op.execute(inputs),
            CloudSecretManagerGraphOp::Azure(op) => op.execute(inputs),
        }
    }
}

// ---------------------------------------------------------------------------
// Public builders
// ---------------------------------------------------------------------------

/// Build a cloud credential graph based on a concrete config.
pub fn build_cloud_secret_manager_credential_graph_from_config(
    config: &CloudSecretConfig,
) -> Dag<CloudSecretManagerGraphOp> {
    match config.provider {
        CloudProviderKind::Gcp => match config.runtime {
            CloudRuntimeKind::GitHubActions => build_cloud_secret_manager_credential_graph_gcp_github(),
            CloudRuntimeKind::CloudMetadata => build_cloud_secret_manager_credential_graph_gcp_metadata(),
            CloudRuntimeKind::LocalDev => build_cloud_secret_manager_credential_graph_gcp_local(),
        },
        CloudProviderKind::Aws => build_cloud_secret_manager_credential_graph_aws_stub(),
        CloudProviderKind::Azure => build_cloud_secret_manager_credential_graph_azure_stub(),
    }
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-credential-gcp-github",
    builder = "build_cloud_secret_manager_credential_graph_gcp_github()",
)]
pub fn build_cloud_secret_manager_credential_graph_gcp_github() -> Dag<CloudSecretManagerGraphOp> {
    build_cloud_secret_manager_credential_graph_gcp(CloudRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-credential-gcp-metadata",
    builder = "build_cloud_secret_manager_credential_graph_gcp_metadata()",
)]
pub fn build_cloud_secret_manager_credential_graph_gcp_metadata() -> Dag<CloudSecretManagerGraphOp> {
    build_cloud_secret_manager_credential_graph_gcp(CloudRuntimeKind::CloudMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-credential-gcp-local",
    builder = "build_cloud_secret_manager_credential_graph_gcp_local()",
)]
pub fn build_cloud_secret_manager_credential_graph_gcp_local() -> Dag<CloudSecretManagerGraphOp> {
    build_cloud_secret_manager_credential_graph_gcp(CloudRuntimeKind::LocalDev)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-credential-aws-stub",
    builder = "build_cloud_secret_manager_credential_graph_aws_stub()",
)]
pub fn build_cloud_secret_manager_credential_graph_aws_stub() -> Dag<CloudSecretManagerGraphOp> {
    lift_aws(build_aws_secrets_manager_credential_graph())
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-credential-azure-stub",
    builder = "build_cloud_secret_manager_credential_graph_azure_stub()",
)]
pub fn build_cloud_secret_manager_credential_graph_azure_stub() -> Dag<CloudSecretManagerGraphOp> {
    lift_azure(build_azure_key_vault_credential_graph())
}

/// Build a cloud secret upsert graph based on a concrete config.
pub fn build_cloud_secret_manager_upsert_graph_from_config(
    config: &CloudSecretConfig,
) -> Dag<CloudSecretManagerGraphOp> {
    match config.provider {
        CloudProviderKind::Gcp => match config.runtime {
            CloudRuntimeKind::GitHubActions => build_cloud_secret_manager_upsert_graph_gcp_github(),
            CloudRuntimeKind::CloudMetadata => build_cloud_secret_manager_upsert_graph_gcp_metadata(),
            CloudRuntimeKind::LocalDev => build_cloud_secret_manager_upsert_graph_gcp_local(),
        },
        CloudProviderKind::Aws => build_cloud_secret_manager_upsert_graph_aws_stub(),
        CloudProviderKind::Azure => build_cloud_secret_manager_upsert_graph_azure_stub(),
    }
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-upsert-gcp-github",
    builder = "build_cloud_secret_manager_upsert_graph_gcp_github()",
)]
pub fn build_cloud_secret_manager_upsert_graph_gcp_github() -> Dag<CloudSecretManagerGraphOp> {
    build_cloud_secret_manager_upsert_graph_gcp(CloudRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-upsert-gcp-metadata",
    builder = "build_cloud_secret_manager_upsert_graph_gcp_metadata()",
)]
pub fn build_cloud_secret_manager_upsert_graph_gcp_metadata() -> Dag<CloudSecretManagerGraphOp> {
    build_cloud_secret_manager_upsert_graph_gcp(CloudRuntimeKind::CloudMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-upsert-gcp-local",
    builder = "build_cloud_secret_manager_upsert_graph_gcp_local()",
)]
pub fn build_cloud_secret_manager_upsert_graph_gcp_local() -> Dag<CloudSecretManagerGraphOp> {
    build_cloud_secret_manager_upsert_graph_gcp(CloudRuntimeKind::LocalDev)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-upsert-aws-stub",
    builder = "build_cloud_secret_manager_upsert_graph_aws_stub()",
)]
pub fn build_cloud_secret_manager_upsert_graph_aws_stub() -> Dag<CloudSecretManagerGraphOp> {
    lift_aws(build_aws_secrets_manager_upsert_graph())
}

#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "cloud-secret-upsert-azure-stub",
    builder = "build_cloud_secret_manager_upsert_graph_azure_stub()",
)]
pub fn build_cloud_secret_manager_upsert_graph_azure_stub() -> Dag<CloudSecretManagerGraphOp> {
    lift_azure(build_azure_key_vault_upsert_graph())
}

// ---------------------------------------------------------------------------
// Internal builders
// ---------------------------------------------------------------------------

fn build_cloud_secret_manager_credential_graph_gcp(
    runtime: CloudRuntimeKind,
) -> Dag<CloudSecretManagerGraphOp> {
    let gcp_dag = match runtime {
        CloudRuntimeKind::GitHubActions => build_gcp_secret_manager_credential_graph_github(),
        CloudRuntimeKind::CloudMetadata => build_gcp_secret_manager_credential_graph_metadata(),
        CloudRuntimeKind::LocalDev => build_gcp_secret_manager_credential_graph_local(),
    };
    let gcp_subdag = lift_gcp(gcp_dag);

    let mut builder: DagBuilder<CloudSecretManagerGraphOp> = DagBuilder::new();

    let resolve_config = builder
        .add_root_node(Node::opaque(
            "resolve_config",
            vec![port("config", "CloudSecretConfig")],
            vec![
                port("provider", "String"),
                port("runtime", "String"),
                port("audience", "String"),
                port("project_or_account", "String"),
                port("secret", "String"),
                optional("version", "OptionalString"),
                optional("service_account_or_role", "OptionalString"),
                optional("impersonate_account_or_role", "OptionalString"),
            ],
            CloudSecretManagerGraphOp::Cloud(CloudOps::ResolveConfig),
        ))
        .expect("resolve_config");

    let map_inputs = builder
        .add_node_after(
            Node::opaque(
                "map_gcp_inputs",
                vec![
                    port("provider", "String"),
                    port("runtime", "String"),
                    port("audience", "String"),
                    port("project_or_account", "String"),
                    port("secret", "String"),
                    optional("version", "OptionalString"),
                    optional("service_account_or_role", "OptionalString"),
                    optional("impersonate_account_or_role", "OptionalString"),
                    // Pass-through inputs for the GCP graph.
                    port("scheme", "String"),
                    optional("header_name", "OptionalString"),
                    port("source_id", "String"),
                    optional("lifetime_seconds", "OptionalInt"),
                    optional("request_url", "OptionalString"),
                    optional("request_token", "OptionalString"),
                ],
                vec![
                    port("audience", "String"),
                    port("project", "String"),
                    port("secret", "String"),
                    optional("version", "OptionalString"),
                    port("service_account", "String"),
                    port("scheme", "String"),
                    optional("header_name", "OptionalString"),
                    port("source_id", "String"),
                    optional("lifetime_seconds", "OptionalInt"),
                    optional("request_url", "OptionalString"),
                    optional("request_token", "OptionalString"),
                ],
                CloudSecretManagerGraphOp::Cloud(CloudOps::MapToGcpInputs { runtime }),
            ),
            &resolve_config,
        )
        .expect("map_gcp_inputs");

    // Wire resolved config → map node.
    builder
        .add_edge(resolve_config.out("provider"), map_inputs.in_port("provider"))
        .expect("resolve_config.provider -> map_gcp_inputs.provider");
    builder
        .add_edge(resolve_config.out("runtime"), map_inputs.in_port("runtime"))
        .expect("resolve_config.runtime -> map_gcp_inputs.runtime");
    builder
        .add_edge(resolve_config.out("audience"), map_inputs.in_port("audience"))
        .expect("resolve_config.audience -> map_gcp_inputs.audience");
    builder
        .add_edge(
            resolve_config.out("project_or_account"),
            map_inputs.in_port("project_or_account"),
        )
        .expect("resolve_config.project_or_account -> map_gcp_inputs.project_or_account");
    builder
        .add_edge(resolve_config.out("secret"), map_inputs.in_port("secret"))
        .expect("resolve_config.secret -> map_gcp_inputs.secret");
    builder
        .add_edge(resolve_config.out("version"), map_inputs.in_port("version"))
        .expect("resolve_config.version -> map_gcp_inputs.version");
    builder
        .add_edge(
            resolve_config.out("service_account_or_role"),
            map_inputs.in_port("service_account_or_role"),
        )
        .expect("resolve_config.service_account_or_role -> map_gcp_inputs.service_account_or_role");
    builder
        .add_edge(
            resolve_config.out("impersonate_account_or_role"),
            map_inputs.in_port("impersonate_account_or_role"),
        )
        .expect("resolve_config.impersonate_account_or_role -> map_gcp_inputs.impersonate_account_or_role");

    // GCP subdag.
    let gcp_node = builder
        .add_node_after(Node::subdag("gcp_wif_secret", gcp_subdag), &map_inputs)
        .expect("gcp_wif_secret");

    // Wire map outputs → GCP graph inputs.
    builder
        .add_edge(map_inputs.out("audience"), gcp_node.in_port("audience"))
        .expect("map_gcp_inputs.audience -> gcp_wif_secret.audience");
    builder
        .add_edge(map_inputs.out("project"), gcp_node.in_port("project"))
        .expect("map_gcp_inputs.project -> gcp_wif_secret.project");
    builder
        .add_edge(map_inputs.out("secret"), gcp_node.in_port("secret"))
        .expect("map_gcp_inputs.secret -> gcp_wif_secret.secret");
    builder
        .add_edge(map_inputs.out("version"), gcp_node.in_port("version"))
        .expect("map_gcp_inputs.version -> gcp_wif_secret.version");
    builder
        .add_edge(
            map_inputs.out("service_account"),
            gcp_node.in_port("service_account"),
        )
        .expect("map_gcp_inputs.service_account -> gcp_wif_secret.service_account");
    builder
        .add_edge(map_inputs.out("scheme"), gcp_node.in_port("scheme"))
        .expect("map_gcp_inputs.scheme -> gcp_wif_secret.scheme");
    builder
        .add_edge(map_inputs.out("header_name"), gcp_node.in_port("header_name"))
        .expect("map_gcp_inputs.header_name -> gcp_wif_secret.header_name");
    builder
        .add_edge(map_inputs.out("source_id"), gcp_node.in_port("source_id"))
        .expect("map_gcp_inputs.source_id -> gcp_wif_secret.source_id");
    builder
        .add_edge(
            map_inputs.out("lifetime_seconds"),
            gcp_node.in_port("lifetime_seconds"),
        )
        .expect("map_gcp_inputs.lifetime_seconds -> gcp_wif_secret.lifetime_seconds");

    if matches!(runtime, CloudRuntimeKind::GitHubActions) {
        builder
            .add_edge(map_inputs.out("request_url"), gcp_node.in_port("request_url"))
            .expect("map_gcp_inputs.request_url -> gcp_wif_secret.request_url");
        builder
            .add_edge(
                map_inputs.out("request_token"),
                gcp_node.in_port("request_token"),
            )
            .expect("map_gcp_inputs.request_token -> gcp_wif_secret.request_token");
    }

    builder.build()
}

fn build_cloud_secret_manager_upsert_graph_gcp(
    runtime: CloudRuntimeKind,
) -> Dag<CloudSecretManagerGraphOp> {
    let gcp_dag = match runtime {
        CloudRuntimeKind::GitHubActions => build_gcp_secret_manager_upsert_graph_github(),
        CloudRuntimeKind::CloudMetadata => build_gcp_secret_manager_upsert_graph_metadata(),
        CloudRuntimeKind::LocalDev => build_gcp_secret_manager_upsert_graph_local(),
    };
    let gcp_subdag = lift_gcp(gcp_dag);

    let mut builder: DagBuilder<CloudSecretManagerGraphOp> = DagBuilder::new();

    let resolve_config = builder
        .add_root_node(Node::opaque(
            "resolve_config",
            vec![port("config", "CloudSecretConfig")],
            vec![
                port("provider", "String"),
                port("runtime", "String"),
                port("audience", "String"),
                port("project_or_account", "String"),
                port("secret", "String"),
                optional("version", "OptionalString"),
                optional("service_account_or_role", "OptionalString"),
                optional("impersonate_account_or_role", "OptionalString"),
            ],
            CloudSecretManagerGraphOp::Cloud(CloudOps::ResolveConfig),
        ))
        .expect("resolve_config");

    let map_inputs = builder
        .add_node_after(
            Node::opaque(
                "map_gcp_secret_inputs",
                vec![
                    port("provider", "String"),
                    port("runtime", "String"),
                    port("audience", "String"),
                    port("project_or_account", "String"),
                    port("secret", "String"),
                    optional("version", "OptionalString"),
                    optional("service_account_or_role", "OptionalString"),
                    optional("impersonate_account_or_role", "OptionalString"),
                    optional("lifetime_seconds", "OptionalInt"),
                    optional("request_url", "OptionalString"),
                    optional("request_token", "OptionalString"),
                ],
                vec![
                    port("audience", "String"),
                    port("project", "String"),
                    port("secret", "String"),
                    port("service_account", "String"),
                    optional("version", "OptionalString"),
                    optional("lifetime_seconds", "OptionalInt"),
                    optional("request_url", "OptionalString"),
                    optional("request_token", "OptionalString"),
                ],
                CloudSecretManagerGraphOp::Cloud(CloudOps::MapToGcpSecretInputs { runtime }),
            ),
            &resolve_config,
        )
        .expect("map_gcp_secret_inputs");

    builder
        .add_edge(resolve_config.out("provider"), map_inputs.in_port("provider"))
        .expect("resolve_config.provider -> map_gcp_secret_inputs.provider");
    builder
        .add_edge(resolve_config.out("runtime"), map_inputs.in_port("runtime"))
        .expect("resolve_config.runtime -> map_gcp_secret_inputs.runtime");
    builder
        .add_edge(resolve_config.out("audience"), map_inputs.in_port("audience"))
        .expect("resolve_config.audience -> map_gcp_secret_inputs.audience");
    builder
        .add_edge(
            resolve_config.out("project_or_account"),
            map_inputs.in_port("project_or_account"),
        )
        .expect("resolve_config.project_or_account -> map_gcp_secret_inputs.project_or_account");
    builder
        .add_edge(resolve_config.out("secret"), map_inputs.in_port("secret"))
        .expect("resolve_config.secret -> map_gcp_secret_inputs.secret");
    builder
        .add_edge(resolve_config.out("version"), map_inputs.in_port("version"))
        .expect("resolve_config.version -> map_gcp_secret_inputs.version");
    builder
        .add_edge(
            resolve_config.out("service_account_or_role"),
            map_inputs.in_port("service_account_or_role"),
        )
        .expect("resolve_config.service_account_or_role -> map_gcp_secret_inputs.service_account_or_role");
    builder
        .add_edge(
            resolve_config.out("impersonate_account_or_role"),
            map_inputs.in_port("impersonate_account_or_role"),
        )
        .expect(
            "resolve_config.impersonate_account_or_role -> map_gcp_secret_inputs.impersonate_account_or_role",
        );

    let gcp_node = builder
        .add_node_after(Node::subdag("gcp_wif_secret_upsert", gcp_subdag), &map_inputs)
        .expect("gcp_wif_secret_upsert");

    builder
        .add_edge(
            map_inputs.out("audience"),
            gcp_node.in_port("audience"),
        )
        .expect("map_gcp_secret_inputs.audience -> gcp_wif_secret_upsert.audience");
    builder
        .add_edge(map_inputs.out("project"), gcp_node.in_port("project"))
        .expect("map_gcp_secret_inputs.project -> gcp_wif_secret_upsert.project");
    builder
        .add_edge(map_inputs.out("secret"), gcp_node.in_port("secret"))
        .expect("map_gcp_secret_inputs.secret -> gcp_wif_secret_upsert.secret");
    builder
        .add_edge(
            map_inputs.out("service_account"),
            gcp_node.in_port("service_account"),
        )
        .expect("map_gcp_secret_inputs.service_account -> gcp_wif_secret_upsert.service_account");
    builder
        .add_edge(
            map_inputs.out("lifetime_seconds"),
            gcp_node.in_port("lifetime_seconds"),
        )
        .expect(
            "map_gcp_secret_inputs.lifetime_seconds -> gcp_wif_secret_upsert.lifetime_seconds",
        );

    if matches!(runtime, CloudRuntimeKind::GitHubActions) {
        builder
            .add_edge(
                map_inputs.out("request_url"),
                gcp_node.in_port("request_url"),
            )
            .expect("map_gcp_secret_inputs.request_url -> gcp_wif_secret_upsert.request_url");
        builder
            .add_edge(
                map_inputs.out("request_token"),
                gcp_node.in_port("request_token"),
            )
            .expect("map_gcp_secret_inputs.request_token -> gcp_wif_secret_upsert.request_token");
    }

    builder.build()
}

// ---------------------------------------------------------------------------
// DAG lifting helpers (provider op → cloud op)
// ---------------------------------------------------------------------------

fn lift_gcp(dag: Dag<GcpSecretManagerGraphOp>) -> Dag<CloudSecretManagerGraphOp> {
    let mut lift = |op| CloudSecretManagerGraphOp::Gcp(op);
    map_dag_ops(dag, &mut lift)
}

fn lift_aws(dag: Dag<AwsSecretManagerGraphOp>) -> Dag<CloudSecretManagerGraphOp> {
    let mut lift = |op| CloudSecretManagerGraphOp::Aws(op);
    map_dag_ops(dag, &mut lift)
}

fn lift_azure(dag: Dag<AzureKeyVaultGraphOp>) -> Dag<CloudSecretManagerGraphOp> {
    let mut lift = |op| CloudSecretManagerGraphOp::Azure(op);
    map_dag_ops(dag, &mut lift)
}

fn map_dag_ops<T, U, F>(dag: Dag<T>, f: &mut F) -> Dag<U>
where
    T: Clone,
    U: Clone,
    F: FnMut(T) -> U,
{
    let mut out = Dag::new();
    out.edges = dag.edges.clone();
    out.nodes = dag
        .nodes
        .into_iter()
        .map(|node| map_node_ops(node, f))
        .collect();
    out
}

fn map_node_ops<T, U, F>(node: Node<T>, f: &mut F) -> Node<U>
where
    T: Clone,
    U: Clone,
    F: FnMut(T) -> U,
{
    let Node {
        id,
        inputs,
        outputs,
        body,
        examples,
    } = node;
    let body = match body {
        NodeBody::Opaque(op) => NodeBody::Opaque(f(op)),
        NodeBody::SubDag(subdag) => NodeBody::SubDag(map_dag_ops(subdag, f)),
    };
    Node {
        id,
        inputs,
        outputs,
        body,
        examples,
    }
}
