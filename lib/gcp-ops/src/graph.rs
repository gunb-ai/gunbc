//! DAGs for GCP WIF + Secret Manager.

use crate::ops::{GcpOps, GcpRuntimeKind};
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::build::{list, optional, port, resource, AccessMode};
use gunbc_ir::{Dag, DagBuilder, Edge, Node, NodeRef, Value};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::NetEnv;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum GcpSecretManagerGraphOp {
    Gcp(GcpOps),
    NetEnv(NetEnv),
    Transport(TransportOps),
}

impl Executable for GcpSecretManagerGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GcpSecretManagerGraphOp::Gcp(op) => op.execute(inputs),
            GcpSecretManagerGraphOp::NetEnv(op) => op.execute(inputs),
            GcpSecretManagerGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build a GCP Secret Manager credential acquisition graph for the given runtime.
///
/// Entrypoints:
/// - `audience`: WIF provider audience (GitHub/metadata runtimes only)
/// - `request_url`: GitHub OIDC request URL (GitHub runtime only)
/// - `request_token`: GitHub OIDC request token (GitHub runtime only)
/// - `interactive_allowed`: allow interactive local auth upsert (local runtime only)
/// - `service_account`: SA email for impersonation
/// - `lifetime_seconds`: optional SA token lifetime (default: 3600s)
/// - `project`: GCP project ID for Secret Manager
/// - `secret`: secret name (no prefixing baked in)
/// - `version`: secret version (default: "latest")
/// - `scheme`: "bearer" | "header"
/// - `header_name`: header name when scheme=header
/// - `source_id`: stable provider ID for SecretSource::Exchange
///
/// Outputs:
/// - `credential`: Credential capability
pub fn build_gcp_secret_manager_credential_graph(
    runtime: GcpRuntimeKind,
) -> Dag<GcpSecretManagerGraphOp> {
    let mut builder: DagBuilder<GcpSecretManagerGraphOp> = DagBuilder::new();

    let net_env = builder
        .add_root_node(Node::opaque(
            "net_env",
            vec![],
            vec![port("net", "NetworkHandle")],
            GcpSecretManagerGraphOp::NetEnv(NetEnv),
        ))
        .expect("net_env");

    // ---------------------------------------------------------------------
    // Base access token acquisition
    // ---------------------------------------------------------------------

    let access_token_node = match runtime {
        GcpRuntimeKind::GitHubActions | GcpRuntimeKind::GcpMetadata => {
            // OIDC subject token acquisition
            let subject_token_node = match runtime {
                GcpRuntimeKind::GitHubActions => {
                    let prepare = builder
                        .add_root_node(Node::opaque(
                            "prepare_github_oidc",
                            vec![
                                port("audience", "String"),
                                optional("request_url", "OptionalString"),
                                optional("request_token", "OptionalString"),
                            ],
                            vec![port("request", "TransportRequest"), port("skip", "Bool")],
                            GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareGitHubOidcRequest),
                        ))
                        .expect("prepare_github_oidc");

                    let execute = builder
                        .add_node_after(
                            Node::opaque(
                                "execute_github_oidc",
                                vec![
                                    port("request", "TransportRequest"),
                                    port("skip", "Bool"),
                                    resource("net", "NetworkHandle", AccessMode::Read),
                                ],
                                vec![port("response", "TransportResponse")],
                                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                            ),
                            &prepare,
                        )
                        .expect("execute_github_oidc");

                    let parse = builder
                        .add_node_after(
                            Node::opaque(
                                "parse_github_oidc",
                                vec![port("response", "TransportResponse")],
                                vec![port("subject_token", "String")],
                                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseGitHubOidcResponse),
                            ),
                            &execute,
                        )
                        .expect("parse_github_oidc");

                    builder
                        .add_edge(prepare.out("request"), execute.in_port("request"))
                        .expect("prepare_github_oidc.request -> execute_github_oidc.request");
                    builder
                        .add_edge(prepare.out("skip"), execute.in_port("skip"))
                        .expect("prepare_github_oidc.skip -> execute_github_oidc.skip");
                    builder
                        .add_edge(net_env.out("net"), execute.in_port("res:net"))
                        .expect("net_env -> execute_github_oidc.res:net");
                    builder
                        .add_edge(execute.out("response"), parse.in_port("response"))
                        .expect("execute_github_oidc.response -> parse_github_oidc.response");

                    parse
                }
                GcpRuntimeKind::GcpMetadata => {
                    let prepare = builder
                        .add_root_node(Node::opaque(
                            "prepare_metadata_oidc",
                            vec![port("audience", "String")],
                            vec![port("request", "TransportRequest"), port("skip", "Bool")],
                            GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareMetadataOidcRequest),
                        ))
                        .expect("prepare_metadata_oidc");

                    let execute = builder
                        .add_node_after(
                            Node::opaque(
                                "execute_metadata_oidc",
                                vec![
                                    port("request", "TransportRequest"),
                                    port("skip", "Bool"),
                                    resource("net", "NetworkHandle", AccessMode::Read),
                                ],
                                vec![port("response", "TransportResponse")],
                                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                            ),
                            &prepare,
                        )
                        .expect("execute_metadata_oidc");

                    let parse = builder
                        .add_node_after(
                            Node::opaque(
                                "parse_metadata_oidc",
                                vec![port("response", "TransportResponse")],
                                vec![port("subject_token", "String")],
                                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseMetadataOidcResponse),
                            ),
                            &execute,
                        )
                        .expect("parse_metadata_oidc");

                    builder
                        .add_edge(prepare.out("request"), execute.in_port("request"))
                        .expect("prepare_metadata_oidc.request -> execute_metadata_oidc.request");
                    builder
                        .add_edge(prepare.out("skip"), execute.in_port("skip"))
                        .expect("prepare_metadata_oidc.skip -> execute_metadata_oidc.skip");
                    builder
                        .add_edge(net_env.out("net"), execute.in_port("res:net"))
                        .expect("net_env -> execute_metadata_oidc.res:net");
                    builder
                        .add_edge(execute.out("response"), parse.in_port("response"))
                        .expect("execute_metadata_oidc.response -> parse_metadata_oidc.response");

                    parse
                }
                GcpRuntimeKind::LocalDev => unreachable!(),
            };

            // STS exchange (subject_token -> access_token)
            let prepare_sts = builder
                .add_node_after(
                    Node::opaque(
                        "prepare_sts",
                        vec![port("audience", "String"), port("subject_token", "String")],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareStsExchange),
                    ),
                    &subject_token_node,
                )
                .expect("prepare_sts");

            let execute_sts = builder
                .add_node_after(
                    Node::opaque(
                        "execute_sts",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_sts,
                )
                .expect("execute_sts");

            let parse_sts = builder
                .add_node_after(
                    Node::opaque(
                        "parse_sts",
                        vec![port("response", "TransportResponse")],
                        vec![port("access_token", "String"), port("expires_in", "Int")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseStsExchange),
                    ),
                    &execute_sts,
                )
                .expect("parse_sts");

            builder
                .add_edge(
                    subject_token_node.out("subject_token"),
                    prepare_sts.in_port("subject_token"),
                )
                .expect("subject_token -> prepare_sts.subject_token");
            builder
                .add_edge(prepare_sts.out("request"), execute_sts.in_port("request"))
                .expect("prepare_sts.request -> execute_sts.request");
            builder
                .add_edge(prepare_sts.out("skip"), execute_sts.in_port("skip"))
                .expect("prepare_sts.skip -> execute_sts.skip");
            builder
                .add_edge(net_env.out("net"), execute_sts.in_port("res:net"))
                .expect("net_env -> execute_sts.res:net");
            builder
                .add_edge(execute_sts.out("response"), parse_sts.in_port("response"))
                .expect("execute_sts.response -> parse_sts.response");

            parse_sts
        }
        GcpRuntimeKind::LocalDev => {
            // Use the canonical upsert sub-DAG for local auth
            // (check -> create[guarded] -> resolve)

            builder
                .add_root_node(Node::subdag(
                    "local_auth_upsert",
                    build_local_auth_upsert_dag(),
                ))
                .expect("local_auth_upsert")
        }
    };

    // Ensure SA has required IAM roles before impersonation (local dev only).
    add_ensure_iam_nodes(&mut builder, &net_env, &access_token_node, runtime);

    // ---------------------------------------------------------------------
    // Service Account impersonation
    // ---------------------------------------------------------------------

    let should_impersonate = builder
        .add_node_after(
            Node::opaque(
                "should_impersonate",
                vec![port("service_account", "String")],
                vec![port("should", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ShouldImpersonate),
            ),
            &access_token_node,
        )
        .expect("should_impersonate");

    let prepare_impersonate = builder
        .add_node_after(
            Node::opaque(
                "prepare_impersonate",
                vec![
                    port("access_token", "String"),
                    port("service_account", "String"),
                    optional("lifetime_seconds", "OptionalInt"),
                    optional("should_impersonate", "OptionalBool"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareImpersonate),
            ),
            &should_impersonate,
        )
        .expect("prepare_impersonate");

    let execute_impersonate = builder
        .add_node_after(
            Node::opaque(
                "execute_impersonate",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_impersonate,
        )
        .expect("execute_impersonate");

    let parse_impersonate = builder
        .add_node_after(
            Node::opaque(
                "parse_impersonate",
                vec![
                    port("response", "TransportResponse"),
                    optional("base_access_token", "OptionalString"),
                ],
                vec![port("access_token", "String")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseImpersonate),
            ),
            &execute_impersonate,
        )
        .expect("parse_impersonate");

    builder
        .add_edge(
            access_token_node.out("access_token"),
            prepare_impersonate.in_port("access_token"),
        )
        .expect("access_token_node.access_token -> prepare_impersonate.access_token");
    builder
        .add_edge(
            should_impersonate.out("should"),
            prepare_impersonate.in_port("should_impersonate"),
        )
        .expect("should_impersonate.should -> prepare_impersonate.should_impersonate");
    builder
        .add_edge(
            prepare_impersonate.out("request"),
            execute_impersonate.in_port("request"),
        )
        .expect("prepare_impersonate.request -> execute_impersonate.request");
    builder
        .add_edge(
            prepare_impersonate.out("skip"),
            execute_impersonate.in_port("skip"),
        )
        .expect("prepare_impersonate.skip -> execute_impersonate.skip");
    builder
        .add_edge(net_env.out("net"), execute_impersonate.in_port("res:net"))
        .expect("net_env -> execute_impersonate.res:net");
    builder
        .add_edge(
            execute_impersonate.out("response"),
            parse_impersonate.in_port("response"),
        )
        .expect("execute_impersonate.response -> parse_impersonate.response");
    builder
        .add_edge(
            access_token_node.out("access_token"),
            parse_impersonate.in_port("base_access_token"),
        )
        .expect("access_token_node.access_token -> parse_impersonate.base_access_token");

    // ---------------------------------------------------------------------
    // Secret Manager access
    // ---------------------------------------------------------------------

    let prepare_secret = builder
        .add_node_after(
            Node::opaque(
                "prepare_secret_access",
                vec![
                    port("access_token", "String"),
                    port("project", "String"),
                    port("secret", "String"),
                    optional("version", "OptionalString"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareSecretAccess),
            ),
            &parse_impersonate,
        )
        .expect("prepare_secret_access");

    let execute_secret = builder
        .add_node_after(
            Node::opaque(
                "execute_secret_access",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_secret,
        )
        .expect("execute_secret_access");

    let parse_secret = builder
        .add_node_after(
            Node::opaque(
                "parse_secret_access",
                vec![port("response", "TransportResponse")],
                vec![port("secret", "String")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseSecretAccess),
            ),
            &execute_secret,
        )
        .expect("parse_secret_access");

    builder
        .add_edge(
            parse_impersonate.out("access_token"),
            prepare_secret.in_port("access_token"),
        )
        .expect("parse_impersonate.access_token -> prepare_secret.access_token");
    builder
        .add_edge(
            prepare_secret.out("request"),
            execute_secret.in_port("request"),
        )
        .expect("prepare_secret.request -> execute_secret.request");
    builder
        .add_edge(prepare_secret.out("skip"), execute_secret.in_port("skip"))
        .expect("prepare_secret.skip -> execute_secret.skip");
    builder
        .add_edge(net_env.out("net"), execute_secret.in_port("res:net"))
        .expect("net_env -> execute_secret_access.res:net");
    builder
        .add_edge(
            execute_secret.out("response"),
            parse_secret.in_port("response"),
        )
        .expect("execute_secret.response -> parse_secret.response");

    // ---------------------------------------------------------------------
    // Credential assembly
    // ---------------------------------------------------------------------

    let build_credential = builder
        .add_node_after(
            Node::opaque(
                "build_credential",
                vec![
                    port("secret", "String"),
                    port("scheme", "String"),
                    optional("header_name", "OptionalString"),
                    port("source_id", "String"),
                    list("required_scopes", "String"),
                ],
                vec![port("credential", "Credential")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::BuildCredential),
            ),
            &parse_secret,
        )
        .expect("build_credential");

    builder
        .add_edge(
            parse_secret.out("secret"),
            build_credential.in_port("secret"),
        )
        .expect("parse_secret.secret -> build_credential.secret");

    builder.build()
}

pub fn build_gcp_secret_manager_credential_graph_github() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_credential_graph(GcpRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-metadata",
    builder = "build_gcp_secret_manager_credential_graph_metadata()"
)]
pub fn build_gcp_secret_manager_credential_graph_metadata() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_credential_graph(GcpRuntimeKind::GcpMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-local",
    builder = "build_gcp_secret_manager_credential_graph_local()"
)]
pub fn build_gcp_secret_manager_credential_graph_local() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_credential_graph(GcpRuntimeKind::LocalDev)
}

/// Build a GCP Secret Manager upsert graph for the given runtime.
///
/// Entrypoints:
/// - `audience`: WIF provider audience (GitHub/metadata runtimes only)
/// - `request_url`: GitHub OIDC request URL (GitHub runtime only)
/// - `request_token`: GitHub OIDC request token (GitHub runtime only)
/// - `interactive_allowed`: allow interactive local auth upsert (local runtime only)
/// - `service_account`: SA email for impersonation
/// - `lifetime_seconds`: optional SA token lifetime (default: 3600s)
/// - `project`: GCP project ID for Secret Manager
/// - `secret`: secret name (no prefixing baked in)
/// - `secret_value`: Secret payload to store as a new version
///
/// Outputs:
/// - `version`: created secret version name
pub fn build_gcp_secret_manager_upsert_graph(
    runtime: GcpRuntimeKind,
) -> Dag<GcpSecretManagerGraphOp> {
    let mut builder: DagBuilder<GcpSecretManagerGraphOp> = DagBuilder::new();

    let net_env = builder
        .add_root_node(Node::opaque(
            "net_env",
            vec![],
            vec![port("net", "NetworkHandle")],
            GcpSecretManagerGraphOp::NetEnv(NetEnv),
        ))
        .expect("net_env");

    // ---------------------------------------------------------------------
    // Base access token acquisition
    // ---------------------------------------------------------------------

    let access_token_node = match runtime {
        GcpRuntimeKind::GitHubActions | GcpRuntimeKind::GcpMetadata => {
            // OIDC subject token acquisition
            let subject_token_node = match runtime {
                GcpRuntimeKind::GitHubActions => {
                    let prepare = builder
                        .add_root_node(Node::opaque(
                            "prepare_github_oidc",
                            vec![
                                port("audience", "String"),
                                optional("request_url", "OptionalString"),
                                optional("request_token", "OptionalString"),
                            ],
                            vec![port("request", "TransportRequest"), port("skip", "Bool")],
                            GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareGitHubOidcRequest),
                        ))
                        .expect("prepare_github_oidc");

                    let execute = builder
                        .add_node_after(
                            Node::opaque(
                                "execute_github_oidc",
                                vec![
                                    port("request", "TransportRequest"),
                                    port("skip", "Bool"),
                                    resource("net", "NetworkHandle", AccessMode::Read),
                                ],
                                vec![port("response", "TransportResponse")],
                                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                            ),
                            &prepare,
                        )
                        .expect("execute_github_oidc");

                    let parse = builder
                        .add_node_after(
                            Node::opaque(
                                "parse_github_oidc",
                                vec![port("response", "TransportResponse")],
                                vec![port("subject_token", "String")],
                                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseGitHubOidcResponse),
                            ),
                            &execute,
                        )
                        .expect("parse_github_oidc");

                    builder
                        .add_edge(prepare.out("request"), execute.in_port("request"))
                        .expect("prepare_github_oidc.request -> execute_github_oidc.request");
                    builder
                        .add_edge(prepare.out("skip"), execute.in_port("skip"))
                        .expect("prepare_github_oidc.skip -> execute_github_oidc.skip");
                    builder
                        .add_edge(net_env.out("net"), execute.in_port("res:net"))
                        .expect("net_env -> execute_github_oidc.res:net");
                    builder
                        .add_edge(execute.out("response"), parse.in_port("response"))
                        .expect("execute_github_oidc.response -> parse_github_oidc.response");

                    parse
                }
                GcpRuntimeKind::GcpMetadata => {
                    let prepare = builder
                        .add_root_node(Node::opaque(
                            "prepare_metadata_oidc",
                            vec![port("audience", "String")],
                            vec![port("request", "TransportRequest"), port("skip", "Bool")],
                            GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareMetadataOidcRequest),
                        ))
                        .expect("prepare_metadata_oidc");

                    let execute = builder
                        .add_node_after(
                            Node::opaque(
                                "execute_metadata_oidc",
                                vec![
                                    port("request", "TransportRequest"),
                                    port("skip", "Bool"),
                                    resource("net", "NetworkHandle", AccessMode::Read),
                                ],
                                vec![port("response", "TransportResponse")],
                                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                            ),
                            &prepare,
                        )
                        .expect("execute_metadata_oidc");

                    let parse = builder
                        .add_node_after(
                            Node::opaque(
                                "parse_metadata_oidc",
                                vec![port("response", "TransportResponse")],
                                vec![port("subject_token", "String")],
                                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseMetadataOidcResponse),
                            ),
                            &execute,
                        )
                        .expect("parse_metadata_oidc");

                    builder
                        .add_edge(prepare.out("request"), execute.in_port("request"))
                        .expect("prepare_metadata_oidc.request -> execute_metadata_oidc.request");
                    builder
                        .add_edge(prepare.out("skip"), execute.in_port("skip"))
                        .expect("prepare_metadata_oidc.skip -> execute_metadata_oidc.skip");
                    builder
                        .add_edge(net_env.out("net"), execute.in_port("res:net"))
                        .expect("net_env -> execute_metadata_oidc.res:net");
                    builder
                        .add_edge(execute.out("response"), parse.in_port("response"))
                        .expect("execute_metadata_oidc.response -> parse_metadata_oidc.response");

                    parse
                }
                GcpRuntimeKind::LocalDev => unreachable!(),
            };

            // STS exchange (subject_token -> access_token)
            let prepare_sts = builder
                .add_node_after(
                    Node::opaque(
                        "prepare_sts",
                        vec![port("audience", "String"), port("subject_token", "String")],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareStsExchange),
                    ),
                    &subject_token_node,
                )
                .expect("prepare_sts");

            let execute_sts = builder
                .add_node_after(
                    Node::opaque(
                        "execute_sts",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_sts,
                )
                .expect("execute_sts");

            let parse_sts = builder
                .add_node_after(
                    Node::opaque(
                        "parse_sts",
                        vec![port("response", "TransportResponse")],
                        vec![port("access_token", "String"), port("expires_in", "Int")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseStsExchange),
                    ),
                    &execute_sts,
                )
                .expect("parse_sts");

            builder
                .add_edge(
                    subject_token_node.out("subject_token"),
                    prepare_sts.in_port("subject_token"),
                )
                .expect("subject_token -> prepare_sts.subject_token");
            builder
                .add_edge(prepare_sts.out("request"), execute_sts.in_port("request"))
                .expect("prepare_sts.request -> execute_sts.request");
            builder
                .add_edge(prepare_sts.out("skip"), execute_sts.in_port("skip"))
                .expect("prepare_sts.skip -> execute_sts.skip");
            builder
                .add_edge(net_env.out("net"), execute_sts.in_port("res:net"))
                .expect("net_env -> execute_sts.res:net");
            builder
                .add_edge(execute_sts.out("response"), parse_sts.in_port("response"))
                .expect("execute_sts.response -> parse_sts.response");

            parse_sts
        }
        GcpRuntimeKind::LocalDev => {
            // Use the canonical upsert sub-DAG for local auth
            // (check -> create[guarded] -> resolve)

            builder
                .add_root_node(Node::subdag(
                    "local_auth_upsert",
                    build_local_auth_upsert_dag(),
                ))
                .expect("local_auth_upsert")
        }
    };

    // Ensure SA has required IAM roles before impersonation (local dev only).
    add_ensure_iam_nodes(&mut builder, &net_env, &access_token_node, runtime);

    // ---------------------------------------------------------------------
    // Service Account impersonation
    // ---------------------------------------------------------------------

    let should_impersonate = builder
        .add_node_after(
            Node::opaque(
                "should_impersonate",
                vec![port("service_account", "String")],
                vec![port("should", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ShouldImpersonate),
            ),
            &access_token_node,
        )
        .expect("should_impersonate");

    let prepare_impersonate = builder
        .add_node_after(
            Node::opaque(
                "prepare_impersonate",
                vec![
                    port("access_token", "String"),
                    port("service_account", "String"),
                    optional("lifetime_seconds", "OptionalInt"),
                    optional("should_impersonate", "OptionalBool"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareImpersonate),
            ),
            &should_impersonate,
        )
        .expect("prepare_impersonate");

    let execute_impersonate = builder
        .add_node_after(
            Node::opaque(
                "execute_impersonate",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_impersonate,
        )
        .expect("execute_impersonate");

    let parse_impersonate = builder
        .add_node_after(
            Node::opaque(
                "parse_impersonate",
                vec![
                    port("response", "TransportResponse"),
                    optional("base_access_token", "OptionalString"),
                ],
                vec![port("access_token", "String")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseImpersonate),
            ),
            &execute_impersonate,
        )
        .expect("parse_impersonate");

    builder
        .add_edge(
            access_token_node.out("access_token"),
            prepare_impersonate.in_port("access_token"),
        )
        .expect("access_token_node.access_token -> prepare_impersonate.access_token");
    builder
        .add_edge(
            should_impersonate.out("should"),
            prepare_impersonate.in_port("should_impersonate"),
        )
        .expect("should_impersonate.should -> prepare_impersonate.should_impersonate");
    builder
        .add_edge(
            prepare_impersonate.out("request"),
            execute_impersonate.in_port("request"),
        )
        .expect("prepare_impersonate.request -> execute_impersonate.request");
    builder
        .add_edge(
            prepare_impersonate.out("skip"),
            execute_impersonate.in_port("skip"),
        )
        .expect("prepare_impersonate.skip -> execute_impersonate.skip");
    builder
        .add_edge(net_env.out("net"), execute_impersonate.in_port("res:net"))
        .expect("net_env -> execute_impersonate.res:net");
    builder
        .add_edge(
            execute_impersonate.out("response"),
            parse_impersonate.in_port("response"),
        )
        .expect("execute_impersonate.response -> parse_impersonate.response");
    builder
        .add_edge(
            access_token_node.out("access_token"),
            parse_impersonate.in_port("base_access_token"),
        )
        .expect("access_token_node.access_token -> parse_impersonate.base_access_token");

    // ---------------------------------------------------------------------
    // Secret Manager upsert: check -> create -> addVersion
    // ---------------------------------------------------------------------

    let prepare_get = builder
        .add_node_after(
            Node::opaque(
                "prepare_secret_get",
                vec![
                    port("access_token", "String"),
                    port("project", "String"),
                    port("secret", "String"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareSecretGet),
            ),
            &parse_impersonate,
        )
        .expect("prepare_secret_get");

    let execute_get = builder
        .add_node_after(
            Node::opaque(
                "execute_secret_get",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_get,
        )
        .expect("execute_secret_get");

    let parse_get = builder
        .add_node_after(
            Node::opaque(
                "parse_secret_get",
                vec![port("response", "TransportResponse")],
                vec![port("exists", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseSecretGet),
            ),
            &execute_get,
        )
        .expect("parse_secret_get");

    builder
        .add_edge(
            parse_impersonate.out("access_token"),
            prepare_get.in_port("access_token"),
        )
        .expect("parse_impersonate.access_token -> prepare_secret_get.access_token");
    builder
        .add_edge(prepare_get.out("request"), execute_get.in_port("request"))
        .expect("prepare_secret_get.request -> execute_secret_get.request");
    builder
        .add_edge(prepare_get.out("skip"), execute_get.in_port("skip"))
        .expect("prepare_secret_get.skip -> execute_secret_get.skip");
    builder
        .add_edge(net_env.out("net"), execute_get.in_port("res:net"))
        .expect("net_env -> execute_secret_get.res:net");
    builder
        .add_edge(execute_get.out("response"), parse_get.in_port("response"))
        .expect("execute_secret_get.response -> parse_secret_get.response");

    let prepare_create = builder
        .add_node_after(
            Node::opaque(
                "prepare_secret_create",
                vec![
                    port("access_token", "String"),
                    port("project", "String"),
                    port("secret", "String"),
                    port("exists", "Bool"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareSecretCreate),
            ),
            &parse_get,
        )
        .expect("prepare_secret_create");

    let execute_create = builder
        .add_node_after(
            Node::opaque(
                "execute_secret_create",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_create,
        )
        .expect("execute_secret_create");

    builder
        .add_edge(
            parse_impersonate.out("access_token"),
            prepare_create.in_port("access_token"),
        )
        .expect("parse_impersonate.access_token -> prepare_secret_create.access_token");
    builder
        .add_edge(parse_get.out("exists"), prepare_create.in_port("exists"))
        .expect("parse_secret_get.exists -> prepare_secret_create.exists");
    builder
        .add_edge(
            prepare_create.out("request"),
            execute_create.in_port("request"),
        )
        .expect("prepare_secret_create.request -> execute_secret_create.request");
    builder
        .add_edge(prepare_create.out("skip"), execute_create.in_port("skip"))
        .expect("prepare_secret_create.skip -> execute_secret_create.skip");
    builder
        .add_edge(net_env.out("net"), execute_create.in_port("res:net"))
        .expect("net_env -> execute_secret_create.res:net");

    let prepare_add = builder
        .add_node_after(
            Node::opaque(
                "prepare_secret_add_version",
                vec![
                    port("access_token", "String"),
                    port("project", "String"),
                    port("secret", "String"),
                    port("secret_value", "Secret"),
                    optional("create_done", "OptionalBool"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareSecretAddVersion),
            ),
            &execute_create,
        )
        .expect("prepare_secret_add_version");

    let execute_add = builder
        .add_node_after(
            Node::opaque(
                "execute_secret_add_version",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_add,
        )
        .expect("execute_secret_add_version");

    let parse_add = builder
        .add_node_after(
            Node::opaque(
                "parse_secret_add_version",
                vec![port("response", "TransportResponse")],
                vec![port("version", "String")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseSecretAddVersion),
            ),
            &execute_add,
        )
        .expect("parse_secret_add_version");

    builder
        .add_edge(
            parse_impersonate.out("access_token"),
            prepare_add.in_port("access_token"),
        )
        .expect("parse_impersonate.access_token -> prepare_secret_add_version.access_token");
    builder
        .add_edge(
            execute_create.out("skip"),
            prepare_add.in_port("create_done"),
        )
        .expect("execute_secret_create.skip -> prepare_secret_add_version.create_done");
    builder
        .add_edge(prepare_add.out("request"), execute_add.in_port("request"))
        .expect("prepare_secret_add_version.request -> execute_secret_add_version.request");
    builder
        .add_edge(prepare_add.out("skip"), execute_add.in_port("skip"))
        .expect("prepare_secret_add_version.skip -> execute_secret_add_version.skip");
    builder
        .add_edge(net_env.out("net"), execute_add.in_port("res:net"))
        .expect("net_env -> execute_secret_add_version.res:net");
    builder
        .add_edge(execute_add.out("response"), parse_add.in_port("response"))
        .expect("execute_secret_add_version.response -> parse_secret_add_version.response");

    builder.build()
}

pub fn build_gcp_secret_manager_upsert_graph_github() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_upsert_graph(GcpRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-upsert-metadata",
    builder = "build_gcp_secret_manager_upsert_graph_metadata()"
)]
pub fn build_gcp_secret_manager_upsert_graph_metadata() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_upsert_graph(GcpRuntimeKind::GcpMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-upsert-local",
    builder = "build_gcp_secret_manager_upsert_graph_local()"
)]
pub fn build_gcp_secret_manager_upsert_graph_local() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_upsert_graph(GcpRuntimeKind::LocalDev)
}

// ---------------------------------------------------------------------------
// Local auth upsert sub-DAG (shared by credential and upsert graphs)
// ---------------------------------------------------------------------------

/// Public accessor for the local auth upsert sub-DAG (used by discovery_graph).
pub fn build_local_auth_upsert_dag_pub() -> Dag<GcpSecretManagerGraphOp> {
    build_local_auth_upsert_dag()
}

/// Add IAM ensure nodes to a graph builder (local dev only).
///
/// Uses REST API (getIamPolicy + setIamPolicy) to ensure the SA has
/// `roles/secretmanager.secretAccessor` on the secrets project.
/// Fast in the common case (binding exists = single REST call, ~1s).
///
/// Flow:
/// 1. `prepare_ensure_iam` — builds getIamPolicy REST request
/// 2. `execute_get_iam` — executes getIamPolicy
/// 3. `check_iam_binding` — checks policy, outputs setIamPolicy request if missing
/// 4. `execute_set_iam` — executes setIamPolicy (skipped if binding exists)
/// 5. `parse_set_iam` — validates result
///
/// Tolerates PERMISSION_DENIED gracefully.
fn add_ensure_iam_nodes(
    builder: &mut DagBuilder<GcpSecretManagerGraphOp>,
    net_env: &NodeRef<GcpSecretManagerGraphOp>,
    access_token_node: &NodeRef<GcpSecretManagerGraphOp>,
    runtime: GcpRuntimeKind,
) {
    if !matches!(runtime, GcpRuntimeKind::LocalDev) {
        return;
    }

    // Step 1: Prepare getIamPolicy request
    let prepare_ensure_iam = builder
        .add_node_after(
            Node::opaque(
                "prepare_ensure_iam",
                vec![
                    port("access_token", "String"),
                    port("project", "String"),
                    port("service_account", "String"),
                ],
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    port("service_account", "String"),
                    port("project", "String"),
                ],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareEnsureIamBinding),
            ),
            access_token_node,
        )
        .expect("prepare_ensure_iam");

    // Step 2: Execute getIamPolicy
    let execute_get_iam = builder
        .add_node_after(
            Node::opaque(
                "execute_get_iam",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_ensure_iam,
        )
        .expect("execute_get_iam");

    // Step 3: Check binding and prepare setIamPolicy if needed
    let check_iam = builder
        .add_node_after(
            Node::opaque(
                "check_iam_binding",
                vec![
                    port("response", "TransportResponse"),
                    port("access_token", "String"),
                    port("project", "String"),
                    port("service_account", "String"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::CheckAndPrepareIamBinding),
            ),
            &execute_get_iam,
        )
        .expect("check_iam_binding");

    // Step 4: Execute setIamPolicy (skipped if binding already exists)
    let execute_set_iam = builder
        .add_node_after(
            Node::opaque(
                "execute_set_iam",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("net", "NetworkHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
            ),
            &check_iam,
        )
        .expect("execute_set_iam");

    // Step 5: Parse setIamPolicy result
    let parse_set_iam = builder
        .add_node_after(
            Node::opaque(
                "parse_set_iam",
                vec![port("response", "TransportResponse")],
                vec![port("ok", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::ParseSetIamBinding),
            ),
            &execute_set_iam,
        )
        .expect("parse_set_iam");

    // Wire: prepare -> execute_get_iam
    builder
        .add_edge(
            prepare_ensure_iam.out("request"),
            execute_get_iam.in_port("request"),
        )
        .expect("prepare_ensure_iam.request -> execute_get_iam.request");
    builder
        .add_edge(
            prepare_ensure_iam.out("skip"),
            execute_get_iam.in_port("skip"),
        )
        .expect("prepare_ensure_iam.skip -> execute_get_iam.skip");
    builder
        .add_edge(net_env.out("net"), execute_get_iam.in_port("res:net"))
        .expect("net_env -> execute_get_iam.res:net");

    // Wire: execute_get_iam -> check_iam_binding
    builder
        .add_edge(
            execute_get_iam.out("response"),
            check_iam.in_port("response"),
        )
        .expect("execute_get_iam.response -> check_iam_binding.response");
    // Pass through access_token, project, service_account
    builder
        .add_edge(
            prepare_ensure_iam.out("service_account"),
            check_iam.in_port("service_account"),
        )
        .expect("prepare_ensure_iam.sa -> check_iam_binding.sa");
    builder
        .add_edge(
            prepare_ensure_iam.out("project"),
            check_iam.in_port("project"),
        )
        .expect("prepare_ensure_iam.project -> check_iam_binding.project");

    // Wire: check_iam_binding -> execute_set_iam
    builder
        .add_edge(check_iam.out("request"), execute_set_iam.in_port("request"))
        .expect("check_iam_binding.request -> execute_set_iam.request");
    builder
        .add_edge(check_iam.out("skip"), execute_set_iam.in_port("skip"))
        .expect("check_iam_binding.skip -> execute_set_iam.skip");
    builder
        .add_edge(net_env.out("net"), execute_set_iam.in_port("res:net"))
        .expect("net_env -> execute_set_iam.res:net");

    // Wire: execute_set_iam -> parse_set_iam
    builder
        .add_edge(
            execute_set_iam.out("response"),
            parse_set_iam.in_port("response"),
        )
        .expect("execute_set_iam.response -> parse_set_iam.response");

    // Wire access_token from the auth step to the IAM ensure nodes
    builder
        .add_edge(
            access_token_node.out("access_token"),
            prepare_ensure_iam.in_port("access_token"),
        )
        .expect("access_token_node -> prepare_ensure_iam.access_token");
    builder
        .add_edge(
            access_token_node.out("access_token"),
            check_iam.in_port("access_token"),
        )
        .expect("access_token_node -> check_iam_binding.access_token");
}

/// Build the local auth upsert sub-DAG using ADC + OAuth2 REST.
///
/// Implements the canonical upsert pattern (check -> create[guarded] -> resolve)
/// for local developer authentication via Application Default Credentials.
///
/// Instead of shelling out to `gcloud auth print-access-token`, this:
/// 1. **Check**: Tests if `~/.config/gcloud/application_default_credentials.json` exists
/// 2. **Create**: If missing, reports an error with `gcloud auth application-default login` instructions
/// 3. **Resolve**: Reads ADC file, extracts refresh_token, POSTs to oauth2.googleapis.com/token
///
/// Entrypoints:
/// - `interactive_allowed`: OptionalBool — (legacy, kept for interface compat)
///
/// Boundaries (outputs):
/// - `access_token`: String — the resolved GCP access token
/// - `expires_in`: Int — token lifetime in seconds
///
/// Internal structure:
/// ```text
/// [check: prepare_check_adc -> execute -> parse(exists)]
/// [create: guarded(exists==false) -> error with instructions]
/// [resolve: read_adc -> parse_adc -> prepare_oauth2 -> execute_oauth2 -> parse_oauth2(access_token)]
/// ```
fn build_local_auth_upsert_dag() -> Dag<GcpSecretManagerGraphOp> {
    let mut dag = Dag::new();

    // Network environment (needed for OAuth2 REST calls)
    dag.add_node(Node::opaque(
        "net_env",
        vec![],
        vec![port("net", "NetworkHandle")],
        GcpSecretManagerGraphOp::NetEnv(NetEnv),
    ));

    // ========================================================================
    // Check phase: does ADC file exist?
    // ========================================================================

    dag.add_node(Node::opaque(
        "prepare_check",
        vec![],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareCheckAdc),
    ));

    dag.add_node(Node::opaque(
        "execute_check",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_check",
        vec![port("response", "TransportResponse")],
        vec![port("exists", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseCheckAdc),
    ));

    // Check edges
    dag.add_edge(Edge::new(
        "prepare_check",
        "request",
        "execute_check",
        "request",
    ));
    dag.add_edge(Edge::new("prepare_check", "skip", "execute_check", "skip"));
    dag.add_edge(Edge::new("net_env", "net", "execute_check", "res:net"));
    dag.add_edge(Edge::new(
        "execute_check",
        "response",
        "parse_check",
        "response",
    ));

    // ========================================================================
    // Try-refresh phase: read ADC -> parse -> OAuth2 refresh -> try parse
    // ========================================================================

    // Step 1: Read ADC file
    dag.add_node(Node::opaque(
        "prepare_read_adc",
        vec![port("exists", "Bool")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareReadAdc),
    ));

    dag.add_node(Node::opaque(
        "execute_read_adc",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ));

    // Step 2: Parse ADC credentials
    dag.add_node(Node::opaque(
        "parse_adc",
        vec![port("response", "TransportResponse")],
        vec![
            port("client_id", "String"),
            port("client_secret", "String"),
            port("refresh_token", "String"),
        ],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseAdcCredentials),
    ));

    // Step 3: Prepare OAuth2 token refresh
    dag.add_node(Node::opaque(
        "prepare_oauth2",
        vec![
            port("client_id", "String"),
            port("client_secret", "String"),
            port("refresh_token", "String"),
        ],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareOAuth2Refresh),
    ));

    // Step 4: Execute OAuth2 refresh
    dag.add_node(Node::opaque(
        "execute_oauth2",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ));

    // Step 5: Try-parse — catches auth errors as needs_reauth instead of failing
    dag.add_node(Node::opaque(
        "parse_try_refresh",
        vec![port("response", "TransportResponse")],
        vec![
            port("needs_reauth", "Bool"),
            optional("access_token", "OptionalString"),
            optional("expires_in", "OptionalInt"),
        ],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseTryRefresh),
    ));

    // Try-refresh edges
    dag.add_edge(Edge::new(
        "parse_check",
        "exists",
        "prepare_read_adc",
        "exists",
    ));
    dag.add_edge(Edge::new(
        "prepare_read_adc",
        "request",
        "execute_read_adc",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_read_adc",
        "skip",
        "execute_read_adc",
        "skip",
    ));
    dag.add_edge(Edge::new("net_env", "net", "execute_read_adc", "res:net"));
    dag.add_edge(Edge::new(
        "execute_read_adc",
        "response",
        "parse_adc",
        "response",
    ));

    dag.add_edge(Edge::new(
        "parse_adc",
        "client_id",
        "prepare_oauth2",
        "client_id",
    ));
    dag.add_edge(Edge::new(
        "parse_adc",
        "client_secret",
        "prepare_oauth2",
        "client_secret",
    ));
    dag.add_edge(Edge::new(
        "parse_adc",
        "refresh_token",
        "prepare_oauth2",
        "refresh_token",
    ));

    dag.add_edge(Edge::new(
        "prepare_oauth2",
        "request",
        "execute_oauth2",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_oauth2",
        "skip",
        "execute_oauth2",
        "skip",
    ));
    dag.add_edge(Edge::new("net_env", "net", "execute_oauth2", "res:net"));
    dag.add_edge(Edge::new(
        "execute_oauth2",
        "response",
        "parse_try_refresh",
        "response",
    ));

    // ========================================================================
    // Re-auth phase: gcloud auth login -> re-read ADC -> retry refresh
    // (guarded by needs_reauth = true from parse_try_refresh)
    // ========================================================================

    // Gcloud auth login --update-adc
    dag.add_node(Node::opaque(
        "prepare_gcloud_auth",
        vec![port("needs_reauth", "Bool")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareGcloudAuth),
    ));

    dag.add_node(Node::opaque(
        "execute_gcloud_auth",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_gcloud_auth",
        vec![port("response", "TransportResponse")],
        vec![port("ok", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseGcloudAuth),
    ));

    // Re-auth edges
    dag.add_edge(Edge::new(
        "parse_try_refresh",
        "needs_reauth",
        "prepare_gcloud_auth",
        "needs_reauth",
    ));
    dag.add_edge(Edge::new(
        "prepare_gcloud_auth",
        "request",
        "execute_gcloud_auth",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_gcloud_auth",
        "skip",
        "execute_gcloud_auth",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        "net",
        "execute_gcloud_auth",
        "res:net",
    ));
    dag.add_edge(Edge::new(
        "execute_gcloud_auth",
        "response",
        "parse_gcloud_auth",
        "response",
    ));

    // Re-read ADC after gcloud auth
    // Note: input port is "exists" to match PrepareReadAdc's expected input key.
    dag.add_node(Node::opaque(
        "prepare_reread_adc",
        vec![port("exists", "Bool")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareReadAdc),
    ));

    dag.add_node(Node::opaque(
        "execute_reread_adc",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_reread_adc",
        vec![port("response", "TransportResponse")],
        vec![
            port("client_id", "String"),
            port("client_secret", "String"),
            port("refresh_token", "String"),
        ],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseAdcCredentials),
    ));

    // Re-read edges (gcloud auth ok -> treat as "exists" for PrepareReadAdc)
    dag.add_edge(Edge::new(
        "parse_gcloud_auth",
        "ok",
        "prepare_reread_adc",
        "exists",
    ));
    dag.add_edge(Edge::new(
        "prepare_reread_adc",
        "request",
        "execute_reread_adc",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_reread_adc",
        "skip",
        "execute_reread_adc",
        "skip",
    ));
    dag.add_edge(Edge::new("net_env", "net", "execute_reread_adc", "res:net"));
    dag.add_edge(Edge::new(
        "execute_reread_adc",
        "response",
        "parse_reread_adc",
        "response",
    ));

    // Retry OAuth2 refresh with fresh credentials
    dag.add_node(Node::opaque(
        "prepare_retry_oauth2",
        vec![
            port("client_id", "String"),
            port("client_secret", "String"),
            port("refresh_token", "String"),
        ],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareOAuth2Refresh),
    ));

    dag.add_node(Node::opaque(
        "execute_retry_oauth2",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_retry_refresh",
        vec![port("response", "TransportResponse")],
        vec![
            optional("access_token", "OptionalString"),
            optional("expires_in", "OptionalInt"),
        ],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseOAuth2Refresh),
    ));

    // Retry edges
    dag.add_edge(Edge::new(
        "parse_reread_adc",
        "client_id",
        "prepare_retry_oauth2",
        "client_id",
    ));
    dag.add_edge(Edge::new(
        "parse_reread_adc",
        "client_secret",
        "prepare_retry_oauth2",
        "client_secret",
    ));
    dag.add_edge(Edge::new(
        "parse_reread_adc",
        "refresh_token",
        "prepare_retry_oauth2",
        "refresh_token",
    ));
    dag.add_edge(Edge::new(
        "prepare_retry_oauth2",
        "request",
        "execute_retry_oauth2",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_retry_oauth2",
        "skip",
        "execute_retry_oauth2",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        "net",
        "execute_retry_oauth2",
        "res:net",
    ));
    dag.add_edge(Edge::new(
        "execute_retry_oauth2",
        "response",
        "parse_retry_refresh",
        "response",
    ));

    // ========================================================================
    // Merge phase: combine try-refresh and retry-refresh results
    // ========================================================================

    dag.add_node(Node::opaque(
        "merge_auth_result",
        vec![
            optional("try_access_token", "OptionalString"),
            optional("try_expires_in", "OptionalInt"),
            optional("retry_access_token", "OptionalString"),
            optional("retry_expires_in", "OptionalInt"),
        ],
        vec![port("access_token", "String"), port("expires_in", "Int")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::MergeAuthResult),
    ));

    // Merge edges: try-refresh outputs
    dag.add_edge(Edge::new(
        "parse_try_refresh",
        "access_token",
        "merge_auth_result",
        "try_access_token",
    ));
    dag.add_edge(Edge::new(
        "parse_try_refresh",
        "expires_in",
        "merge_auth_result",
        "try_expires_in",
    ));
    // Merge edges: retry-refresh outputs
    dag.add_edge(Edge::new(
        "parse_retry_refresh",
        "access_token",
        "merge_auth_result",
        "retry_access_token",
    ));
    dag.add_edge(Edge::new(
        "parse_retry_refresh",
        "expires_in",
        "merge_auth_result",
        "retry_expires_in",
    ));

    dag
}
