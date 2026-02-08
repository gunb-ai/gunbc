//! DAGs for GCP WIF + Secret Manager.

use crate::ops::{GcpOps, GcpRuntimeKind};
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::build::{optional, port, resource, AccessMode};
use gunbc_ir::{Dag, DagBuilder, Node, Value};
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
            let prepare_check_local = builder
                .add_root_node(Node::opaque(
                    "prepare_check_local_auth",
                    vec![],
                    vec![port("request", "TransportRequest"), port("skip", "Bool")],
                    GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareLocalAccessToken),
                ))
                .expect("prepare_check_local_auth");

            let execute_check_local = builder
                .add_node_after(
                    Node::opaque(
                        "execute_check_local_auth",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_check_local,
                )
                .expect("execute_check_local_auth");

            let parse_check_local = builder
                .add_node_after(
                    Node::opaque(
                        "parse_check_local_auth",
                        vec![port("response", "TransportResponse")],
                        vec![port("exists", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseLocalAuthCheck),
                    ),
                    &execute_check_local,
                )
                .expect("parse_check_local_auth");

            builder
                .add_edge(
                    prepare_check_local.out("request"),
                    execute_check_local.in_port("request"),
                )
                .expect("prepare_check_local_auth.request -> execute_check_local_auth.request");
            builder
                .add_edge(
                    prepare_check_local.out("skip"),
                    execute_check_local.in_port("skip"),
                )
                .expect("prepare_check_local_auth.skip -> execute_check_local_auth.skip");
            builder
                .add_edge(net_env.out("net"), execute_check_local.in_port("res:net"))
                .expect("net_env -> execute_check_local_auth.res:net");
            builder
                .add_edge(
                    execute_check_local.out("response"),
                    parse_check_local.in_port("response"),
                )
                .expect("execute_check_local_auth.response -> parse_check_local_auth.response");

            let prepare_create_local = builder
                .add_node_after(
                    Node::opaque(
                        "prepare_create_local_auth",
                        vec![
                            port("exists", "Bool"),
                            optional("interactive_allowed", "OptionalBool"),
                        ],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareLocalAuthLogin),
                    ),
                    &parse_check_local,
                )
                .expect("prepare_create_local_auth");

            let execute_create_local = builder
                .add_node_after(
                    Node::opaque(
                        "execute_create_local_auth",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_create_local,
                )
                .expect("execute_create_local_auth");

            let parse_create_local = builder
                .add_node_after(
                    Node::opaque(
                        "parse_create_local_auth",
                        vec![port("response", "TransportResponse")],
                        vec![port("ok", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseLocalAuthLogin),
                    ),
                    &execute_create_local,
                )
                .expect("parse_create_local_auth");

            builder
                .add_edge(
                    parse_check_local.out("exists"),
                    prepare_create_local.in_port("exists"),
                )
                .expect("parse_check_local_auth.exists -> prepare_create_local_auth.exists");
            builder
                .add_edge(
                    prepare_create_local.out("request"),
                    execute_create_local.in_port("request"),
                )
                .expect("prepare_create_local_auth.request -> execute_create_local_auth.request");
            builder
                .add_edge(
                    prepare_create_local.out("skip"),
                    execute_create_local.in_port("skip"),
                )
                .expect("prepare_create_local_auth.skip -> execute_create_local_auth.skip");
            builder
                .add_edge(net_env.out("net"), execute_create_local.in_port("res:net"))
                .expect("net_env -> execute_create_local_auth.res:net");
            builder
                .add_edge(
                    execute_create_local.out("response"),
                    parse_create_local.in_port("response"),
                )
                .expect("execute_create_local_auth.response -> parse_create_local_auth.response");

            let prepare_local = builder
                .add_node_after(
                    Node::opaque(
                        "prepare_local_access_token",
                        vec![optional("auth_ready", "OptionalBool")],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareLocalAccessToken),
                    ),
                    &parse_create_local,
                )
                .expect("prepare_local_access_token");

            let execute_local = builder
                .add_node_after(
                    Node::opaque(
                        "execute_local_access_token",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_local,
                )
                .expect("execute_local_access_token");

            let parse_local = builder
                .add_node_after(
                    Node::opaque(
                        "parse_local_access_token",
                        vec![port("response", "TransportResponse")],
                        vec![port("access_token", "String"), port("expires_in", "Int")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseLocalAccessToken),
                    ),
                    &execute_local,
                )
                .expect("parse_local_access_token");

            builder
                .add_edge(
                    parse_create_local.out("ok"),
                    prepare_local.in_port("auth_ready"),
                )
                .expect("parse_create_local_auth.ok -> prepare_local_access_token.auth_ready");
            builder
                .add_edge(
                    prepare_local.out("request"),
                    execute_local.in_port("request"),
                )
                .expect("prepare_local_access_token.request -> execute_local_access_token.request");
            builder
                .add_edge(prepare_local.out("skip"), execute_local.in_port("skip"))
                .expect("prepare_local_access_token.skip -> execute_local_access_token.skip");
            builder
                .add_edge(net_env.out("net"), execute_local.in_port("res:net"))
                .expect("net_env -> execute_local_access_token.res:net");
            builder
                .add_edge(
                    execute_local.out("response"),
                    parse_local.in_port("response"),
                )
                .expect("execute_local_access_token.response -> parse_local_access_token.response");

            parse_local
        }
    };

    // ---------------------------------------------------------------------
    // Service Account impersonation
    // ---------------------------------------------------------------------

    let prepare_impersonate = builder
        .add_node_after(
            Node::opaque(
                "prepare_impersonate",
                vec![
                    port("access_token", "String"),
                    port("service_account", "String"),
                    optional("lifetime_seconds", "OptionalInt"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareImpersonate),
            ),
            &access_token_node,
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
                vec![port("response", "TransportResponse")],
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
            let prepare_check_local = builder
                .add_root_node(Node::opaque(
                    "prepare_check_local_auth",
                    vec![],
                    vec![port("request", "TransportRequest"), port("skip", "Bool")],
                    GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareLocalAccessToken),
                ))
                .expect("prepare_check_local_auth");

            let execute_check_local = builder
                .add_node_after(
                    Node::opaque(
                        "execute_check_local_auth",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_check_local,
                )
                .expect("execute_check_local_auth");

            let parse_check_local = builder
                .add_node_after(
                    Node::opaque(
                        "parse_check_local_auth",
                        vec![port("response", "TransportResponse")],
                        vec![port("exists", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseLocalAuthCheck),
                    ),
                    &execute_check_local,
                )
                .expect("parse_check_local_auth");

            builder
                .add_edge(
                    prepare_check_local.out("request"),
                    execute_check_local.in_port("request"),
                )
                .expect("prepare_check_local_auth.request -> execute_check_local_auth.request");
            builder
                .add_edge(
                    prepare_check_local.out("skip"),
                    execute_check_local.in_port("skip"),
                )
                .expect("prepare_check_local_auth.skip -> execute_check_local_auth.skip");
            builder
                .add_edge(net_env.out("net"), execute_check_local.in_port("res:net"))
                .expect("net_env -> execute_check_local_auth.res:net");
            builder
                .add_edge(
                    execute_check_local.out("response"),
                    parse_check_local.in_port("response"),
                )
                .expect("execute_check_local_auth.response -> parse_check_local_auth.response");

            let prepare_create_local = builder
                .add_node_after(
                    Node::opaque(
                        "prepare_create_local_auth",
                        vec![
                            port("exists", "Bool"),
                            optional("interactive_allowed", "OptionalBool"),
                        ],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareLocalAuthLogin),
                    ),
                    &parse_check_local,
                )
                .expect("prepare_create_local_auth");

            let execute_create_local = builder
                .add_node_after(
                    Node::opaque(
                        "execute_create_local_auth",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_create_local,
                )
                .expect("execute_create_local_auth");

            let parse_create_local = builder
                .add_node_after(
                    Node::opaque(
                        "parse_create_local_auth",
                        vec![port("response", "TransportResponse")],
                        vec![port("ok", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseLocalAuthLogin),
                    ),
                    &execute_create_local,
                )
                .expect("parse_create_local_auth");

            builder
                .add_edge(
                    parse_check_local.out("exists"),
                    prepare_create_local.in_port("exists"),
                )
                .expect("parse_check_local_auth.exists -> prepare_create_local_auth.exists");
            builder
                .add_edge(
                    prepare_create_local.out("request"),
                    execute_create_local.in_port("request"),
                )
                .expect("prepare_create_local_auth.request -> execute_create_local_auth.request");
            builder
                .add_edge(
                    prepare_create_local.out("skip"),
                    execute_create_local.in_port("skip"),
                )
                .expect("prepare_create_local_auth.skip -> execute_create_local_auth.skip");
            builder
                .add_edge(net_env.out("net"), execute_create_local.in_port("res:net"))
                .expect("net_env -> execute_create_local_auth.res:net");
            builder
                .add_edge(
                    execute_create_local.out("response"),
                    parse_create_local.in_port("response"),
                )
                .expect("execute_create_local_auth.response -> parse_create_local_auth.response");

            let prepare_local = builder
                .add_node_after(
                    Node::opaque(
                        "prepare_local_access_token",
                        vec![optional("auth_ready", "OptionalBool")],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareLocalAccessToken),
                    ),
                    &parse_create_local,
                )
                .expect("prepare_local_access_token");

            let execute_local = builder
                .add_node_after(
                    Node::opaque(
                        "execute_local_access_token",
                        vec![
                            port("request", "TransportRequest"),
                            port("skip", "Bool"),
                            resource("net", "NetworkHandle", AccessMode::Read),
                        ],
                        vec![port("response", "TransportResponse")],
                        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
                    ),
                    &prepare_local,
                )
                .expect("execute_local_access_token");

            let parse_local = builder
                .add_node_after(
                    Node::opaque(
                        "parse_local_access_token",
                        vec![port("response", "TransportResponse")],
                        vec![port("access_token", "String"), port("expires_in", "Int")],
                        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseLocalAccessToken),
                    ),
                    &execute_local,
                )
                .expect("parse_local_access_token");

            builder
                .add_edge(
                    parse_create_local.out("ok"),
                    prepare_local.in_port("auth_ready"),
                )
                .expect("parse_create_local_auth.ok -> prepare_local_access_token.auth_ready");
            builder
                .add_edge(
                    prepare_local.out("request"),
                    execute_local.in_port("request"),
                )
                .expect("prepare_local_access_token.request -> execute_local_access_token.request");
            builder
                .add_edge(prepare_local.out("skip"), execute_local.in_port("skip"))
                .expect("prepare_local_access_token.skip -> execute_local_access_token.skip");
            builder
                .add_edge(net_env.out("net"), execute_local.in_port("res:net"))
                .expect("net_env -> execute_local_access_token.res:net");
            builder
                .add_edge(
                    execute_local.out("response"),
                    parse_local.in_port("response"),
                )
                .expect("execute_local_access_token.response -> parse_local_access_token.response");

            parse_local
        }
    };

    // ---------------------------------------------------------------------
    // Service Account impersonation
    // ---------------------------------------------------------------------

    let prepare_impersonate = builder
        .add_node_after(
            Node::opaque(
                "prepare_impersonate",
                vec![
                    port("access_token", "String"),
                    port("service_account", "String"),
                    optional("lifetime_seconds", "OptionalInt"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareImpersonate),
            ),
            &access_token_node,
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
                vec![port("response", "TransportResponse")],
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

pub fn build_gcp_secret_manager_upsert_graph_local() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_upsert_graph(GcpRuntimeKind::LocalDev)
}
