//! DAGs for GCP WIF + Secret Manager.

use crate::ops::{GcpOps, GcpRuntimeKind};
use gunbc_exec::DynOp;
use gunbc_ir::build::{list, optional, port, resource, AccessMode};
use gunbc_ir::builder::BuilderError;
use gunbc_ir::{Dag, DagBuilder, Edge, Node, NodeRef, RESOURCE_API_NETWORK};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::NetEnv;

pub type GcpSecretManagerGraphOp = DynOp;

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
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
    let mut builder: DagBuilder<GcpSecretManagerGraphOp> = DagBuilder::new();

    let net_env = builder.add_root_node(Node::opaque(
        "net_env",
        vec![],
        vec![port(NetEnv::PORT, "NetworkHandle")],
        DynOp::new(NetEnv),
    ))?;

    // ---------------------------------------------------------------------
    // Base access token acquisition
    // ---------------------------------------------------------------------

    let access_token_node = match runtime {
        GcpRuntimeKind::GitHubActions | GcpRuntimeKind::GcpMetadata => {
            // OIDC subject token acquisition
            let subject_token_node = match runtime {
                GcpRuntimeKind::GitHubActions => {
                    let prepare = builder.add_root_node(Node::opaque(
                        "prepare_github_oidc",
                        vec![
                            port("audience", "String"),
                            optional("request_url", "OptionalString"),
                            optional("request_token", "OptionalString"),
                        ],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        DynOp::new(GcpOps::PrepareGitHubOidcRequest),
                    ))?;

                    let execute = builder.add_node_after(
                        Node::opaque(
                            "execute_github_oidc",
                            vec![
                                port("request", "TransportRequest"),
                                port("skip", "Bool"),
                                resource("api:network", "NetworkHandle", AccessMode::Read),
                            ],
                            vec![port("response", "TransportResponse")],
                            DynOp::new(TransportOps::Execute),
                        ),
                        &prepare,
                    )?;

                    let parse = builder.add_node_after(
                        Node::opaque(
                            "parse_github_oidc",
                            vec![port("response", "TransportResponse")],
                            vec![port("subject_token", "String")],
                            DynOp::new(GcpOps::ParseGitHubOidcResponse),
                        ),
                        &execute,
                    )?;

                    builder.add_edge(prepare.out("request"), execute.in_port("request"))?;
                    builder.add_edge(prepare.out("skip"), execute.in_port("skip"))?;
                    builder.add_edge(
                        net_env.out(NetEnv::PORT),
                        execute.in_port(RESOURCE_API_NETWORK),
                    )?;
                    builder.add_edge(execute.out("response"), parse.in_port("response"))?;

                    parse
                }
                GcpRuntimeKind::GcpMetadata => {
                    let prepare = builder.add_root_node(Node::opaque(
                        "prepare_metadata_oidc",
                        vec![port("audience", "String")],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        DynOp::new(GcpOps::PrepareMetadataOidcRequest),
                    ))?;

                    let execute = builder.add_node_after(
                        Node::opaque(
                            "execute_metadata_oidc",
                            vec![
                                port("request", "TransportRequest"),
                                port("skip", "Bool"),
                                resource("api:network", "NetworkHandle", AccessMode::Read),
                            ],
                            vec![port("response", "TransportResponse")],
                            DynOp::new(TransportOps::Execute),
                        ),
                        &prepare,
                    )?;

                    let parse = builder.add_node_after(
                        Node::opaque(
                            "parse_metadata_oidc",
                            vec![port("response", "TransportResponse")],
                            vec![port("subject_token", "String")],
                            DynOp::new(GcpOps::ParseMetadataOidcResponse),
                        ),
                        &execute,
                    )?;

                    builder.add_edge(prepare.out("request"), execute.in_port("request"))?;
                    builder.add_edge(prepare.out("skip"), execute.in_port("skip"))?;
                    builder.add_edge(
                        net_env.out(NetEnv::PORT),
                        execute.in_port(RESOURCE_API_NETWORK),
                    )?;
                    builder.add_edge(execute.out("response"), parse.in_port("response"))?;

                    parse
                }
                GcpRuntimeKind::LocalDev => unreachable!(),
            };

            // STS exchange (subject_token -> access_token)
            let prepare_sts = builder.add_node_after(
                Node::opaque(
                    "prepare_sts",
                    vec![port("audience", "String"), port("subject_token", "String")],
                    vec![port("request", "TransportRequest"), port("skip", "Bool")],
                    DynOp::new(GcpOps::PrepareStsExchange),
                ),
                &subject_token_node,
            )?;

            let execute_sts = builder.add_node_after(
                Node::opaque(
                    "execute_sts",
                    vec![
                        port("request", "TransportRequest"),
                        port("skip", "Bool"),
                        resource("api:network", "NetworkHandle", AccessMode::Read),
                    ],
                    vec![port("response", "TransportResponse")],
                    DynOp::new(TransportOps::Execute),
                ),
                &prepare_sts,
            )?;

            let parse_sts = builder.add_node_after(
                Node::opaque(
                    "parse_sts",
                    vec![port("response", "TransportResponse")],
                    vec![port("access_token", "String"), port("expires_in", "Int")],
                    DynOp::new(GcpOps::ParseStsExchange),
                ),
                &execute_sts,
            )?;

            builder.add_edge(
                subject_token_node.out("subject_token"),
                prepare_sts.in_port("subject_token"),
            )?;
            builder.add_edge(prepare_sts.out("request"), execute_sts.in_port("request"))?;
            builder.add_edge(prepare_sts.out("skip"), execute_sts.in_port("skip"))?;
            builder.add_edge(
                net_env.out(NetEnv::PORT),
                execute_sts.in_port(RESOURCE_API_NETWORK),
            )?;
            builder.add_edge(execute_sts.out("response"), parse_sts.in_port("response"))?;

            parse_sts
        }
        GcpRuntimeKind::LocalDev => {
            // Use the canonical upsert sub-DAG for local auth
            // (check -> create[guarded] -> resolve)

            builder.add_root_node(Node::subdag(
                "local_auth_upsert",
                build_local_auth_upsert_dag(),
            ))?
        }
    };

    // Ensure SA has required IAM roles before impersonation (local dev only).
    add_ensure_iam_nodes(&mut builder, &net_env, &access_token_node, runtime)?;

    // ---------------------------------------------------------------------
    // Service Account impersonation
    // ---------------------------------------------------------------------

    let should_impersonate = builder.add_node_after(
        Node::opaque(
            "should_impersonate",
            vec![
                port("service_account", "String"),
                optional("allow_impersonation", "OptionalBool"),
            ],
            vec![port("should", "Bool")],
            DynOp::new(GcpOps::ShouldImpersonate),
        ),
        &access_token_node,
    )?;

    let prepare_impersonate = builder.add_node_after(
        Node::opaque(
            "prepare_impersonate",
            vec![
                port("access_token", "String"),
                port("service_account", "String"),
                optional("lifetime_seconds", "OptionalInt"),
                optional("should_impersonate", "OptionalBool"),
            ],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(GcpOps::PrepareImpersonate),
        ),
        &should_impersonate,
    )?;

    let execute_impersonate = builder.add_node_after(
        Node::opaque(
            "execute_impersonate",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_impersonate,
    )?;

    let parse_impersonate = builder.add_node_after(
        Node::opaque(
            "parse_impersonate",
            vec![
                port("response", "TransportResponse"),
                optional("base_access_token", "OptionalString"),
            ],
            vec![port("access_token", "String")],
            DynOp::new(GcpOps::ParseImpersonate),
        ),
        &execute_impersonate,
    )?;

    builder.add_edge(
        access_token_node.out("access_token"),
        prepare_impersonate.in_port("access_token"),
    )?;
    builder.add_edge(
        should_impersonate.out("should"),
        prepare_impersonate.in_port("should_impersonate"),
    )?;
    builder.add_edge(
        prepare_impersonate.out("request"),
        execute_impersonate.in_port("request"),
    )?;
    builder.add_edge(
        prepare_impersonate.out("skip"),
        execute_impersonate.in_port("skip"),
    )?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_impersonate.in_port(RESOURCE_API_NETWORK),
    )?;
    builder.add_edge(
        execute_impersonate.out("response"),
        parse_impersonate.in_port("response"),
    )?;
    builder.add_edge(
        access_token_node.out("access_token"),
        parse_impersonate.in_port("base_access_token"),
    )?;

    // ---------------------------------------------------------------------
    // Secret Manager access
    // ---------------------------------------------------------------------

    let prepare_secret = builder.add_node_after(
        Node::opaque(
            "prepare_secret_access",
            vec![
                port("access_token", "String"),
                port("project", "String"),
                port("secret", "String"),
                optional("version", "OptionalString"),
            ],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(GcpOps::PrepareSecretAccess),
        ),
        &parse_impersonate,
    )?;

    let execute_secret = builder.add_node_after(
        Node::opaque(
            "execute_secret_access",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_secret,
    )?;

    let parse_secret = builder.add_node_after(
        Node::opaque(
            "parse_secret_access",
            vec![port("response", "TransportResponse")],
            vec![port("secret", "String")],
            DynOp::new(GcpOps::ParseSecretAccess),
        ),
        &execute_secret,
    )?;

    builder.add_edge(
        parse_impersonate.out("access_token"),
        prepare_secret.in_port("access_token"),
    )?;
    builder.add_edge(
        prepare_secret.out("request"),
        execute_secret.in_port("request"),
    )?;
    builder.add_edge(prepare_secret.out("skip"), execute_secret.in_port("skip"))?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_secret.in_port(RESOURCE_API_NETWORK),
    )?;
    builder.add_edge(
        execute_secret.out("response"),
        parse_secret.in_port("response"),
    )?;

    // ---------------------------------------------------------------------
    // Credential assembly
    // ---------------------------------------------------------------------

    let build_credential = builder.add_node_after(
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
            DynOp::new(GcpOps::BuildCredential),
        ),
        &parse_secret,
    )?;

    builder.add_edge(
        parse_secret.out("secret"),
        build_credential.in_port("secret"),
    )?;

    Ok(builder.build())
}

pub fn build_gcp_secret_manager_credential_graph_github(
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
    build_gcp_secret_manager_credential_graph(GcpRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-metadata",
    builder = "build_gcp_secret_manager_credential_graph_metadata()",
    returns_result
)]
pub fn build_gcp_secret_manager_credential_graph_metadata(
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
    build_gcp_secret_manager_credential_graph(GcpRuntimeKind::GcpMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-local",
    builder = "build_gcp_secret_manager_credential_graph_local()",
    returns_result
)]
pub fn build_gcp_secret_manager_credential_graph_local(
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
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
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
    let mut builder: DagBuilder<GcpSecretManagerGraphOp> = DagBuilder::new();

    let net_env = builder.add_root_node(Node::opaque(
        "net_env",
        vec![],
        vec![port(NetEnv::PORT, "NetworkHandle")],
        DynOp::new(NetEnv),
    ))?;

    // ---------------------------------------------------------------------
    // Base access token acquisition
    // ---------------------------------------------------------------------

    let access_token_node = match runtime {
        GcpRuntimeKind::GitHubActions | GcpRuntimeKind::GcpMetadata => {
            // OIDC subject token acquisition
            let subject_token_node = match runtime {
                GcpRuntimeKind::GitHubActions => {
                    let prepare = builder.add_root_node(Node::opaque(
                        "prepare_github_oidc",
                        vec![
                            port("audience", "String"),
                            optional("request_url", "OptionalString"),
                            optional("request_token", "OptionalString"),
                        ],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        DynOp::new(GcpOps::PrepareGitHubOidcRequest),
                    ))?;

                    let execute = builder.add_node_after(
                        Node::opaque(
                            "execute_github_oidc",
                            vec![
                                port("request", "TransportRequest"),
                                port("skip", "Bool"),
                                resource("api:network", "NetworkHandle", AccessMode::Read),
                            ],
                            vec![port("response", "TransportResponse")],
                            DynOp::new(TransportOps::Execute),
                        ),
                        &prepare,
                    )?;

                    let parse = builder.add_node_after(
                        Node::opaque(
                            "parse_github_oidc",
                            vec![port("response", "TransportResponse")],
                            vec![port("subject_token", "String")],
                            DynOp::new(GcpOps::ParseGitHubOidcResponse),
                        ),
                        &execute,
                    )?;

                    builder.add_edge(prepare.out("request"), execute.in_port("request"))?;
                    builder.add_edge(prepare.out("skip"), execute.in_port("skip"))?;
                    builder.add_edge(
                        net_env.out(NetEnv::PORT),
                        execute.in_port(RESOURCE_API_NETWORK),
                    )?;
                    builder.add_edge(execute.out("response"), parse.in_port("response"))?;

                    parse
                }
                GcpRuntimeKind::GcpMetadata => {
                    let prepare = builder.add_root_node(Node::opaque(
                        "prepare_metadata_oidc",
                        vec![port("audience", "String")],
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
                        DynOp::new(GcpOps::PrepareMetadataOidcRequest),
                    ))?;

                    let execute = builder.add_node_after(
                        Node::opaque(
                            "execute_metadata_oidc",
                            vec![
                                port("request", "TransportRequest"),
                                port("skip", "Bool"),
                                resource("api:network", "NetworkHandle", AccessMode::Read),
                            ],
                            vec![port("response", "TransportResponse")],
                            DynOp::new(TransportOps::Execute),
                        ),
                        &prepare,
                    )?;

                    let parse = builder.add_node_after(
                        Node::opaque(
                            "parse_metadata_oidc",
                            vec![port("response", "TransportResponse")],
                            vec![port("subject_token", "String")],
                            DynOp::new(GcpOps::ParseMetadataOidcResponse),
                        ),
                        &execute,
                    )?;

                    builder.add_edge(prepare.out("request"), execute.in_port("request"))?;
                    builder.add_edge(prepare.out("skip"), execute.in_port("skip"))?;
                    builder.add_edge(
                        net_env.out(NetEnv::PORT),
                        execute.in_port(RESOURCE_API_NETWORK),
                    )?;
                    builder.add_edge(execute.out("response"), parse.in_port("response"))?;

                    parse
                }
                GcpRuntimeKind::LocalDev => unreachable!(),
            };

            // STS exchange (subject_token -> access_token)
            let prepare_sts = builder.add_node_after(
                Node::opaque(
                    "prepare_sts",
                    vec![port("audience", "String"), port("subject_token", "String")],
                    vec![port("request", "TransportRequest"), port("skip", "Bool")],
                    DynOp::new(GcpOps::PrepareStsExchange),
                ),
                &subject_token_node,
            )?;

            let execute_sts = builder.add_node_after(
                Node::opaque(
                    "execute_sts",
                    vec![
                        port("request", "TransportRequest"),
                        port("skip", "Bool"),
                        resource("api:network", "NetworkHandle", AccessMode::Read),
                    ],
                    vec![port("response", "TransportResponse")],
                    DynOp::new(TransportOps::Execute),
                ),
                &prepare_sts,
            )?;

            let parse_sts = builder.add_node_after(
                Node::opaque(
                    "parse_sts",
                    vec![port("response", "TransportResponse")],
                    vec![port("access_token", "String"), port("expires_in", "Int")],
                    DynOp::new(GcpOps::ParseStsExchange),
                ),
                &execute_sts,
            )?;

            builder.add_edge(
                subject_token_node.out("subject_token"),
                prepare_sts.in_port("subject_token"),
            )?;
            builder.add_edge(prepare_sts.out("request"), execute_sts.in_port("request"))?;
            builder.add_edge(prepare_sts.out("skip"), execute_sts.in_port("skip"))?;
            builder.add_edge(
                net_env.out(NetEnv::PORT),
                execute_sts.in_port(RESOURCE_API_NETWORK),
            )?;
            builder.add_edge(execute_sts.out("response"), parse_sts.in_port("response"))?;

            parse_sts
        }
        GcpRuntimeKind::LocalDev => {
            // Use the canonical upsert sub-DAG for local auth
            // (check -> create[guarded] -> resolve)

            builder.add_root_node(Node::subdag(
                "local_auth_upsert",
                build_local_auth_upsert_dag(),
            ))?
        }
    };

    // Ensure SA has required IAM roles before impersonation (local dev only).
    add_ensure_iam_nodes(&mut builder, &net_env, &access_token_node, runtime)?;

    // ---------------------------------------------------------------------
    // Service Account impersonation
    // ---------------------------------------------------------------------

    let should_impersonate = builder.add_node_after(
        Node::opaque(
            "should_impersonate",
            vec![
                port("service_account", "String"),
                optional("allow_impersonation", "OptionalBool"),
            ],
            vec![port("should", "Bool")],
            DynOp::new(GcpOps::ShouldImpersonate),
        ),
        &access_token_node,
    )?;

    let prepare_impersonate = builder.add_node_after(
        Node::opaque(
            "prepare_impersonate",
            vec![
                port("access_token", "String"),
                port("service_account", "String"),
                optional("lifetime_seconds", "OptionalInt"),
                optional("should_impersonate", "OptionalBool"),
            ],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(GcpOps::PrepareImpersonate),
        ),
        &should_impersonate,
    )?;

    let execute_impersonate = builder.add_node_after(
        Node::opaque(
            "execute_impersonate",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_impersonate,
    )?;

    let parse_impersonate = builder.add_node_after(
        Node::opaque(
            "parse_impersonate",
            vec![
                port("response", "TransportResponse"),
                optional("base_access_token", "OptionalString"),
            ],
            vec![port("access_token", "String")],
            DynOp::new(GcpOps::ParseImpersonate),
        ),
        &execute_impersonate,
    )?;

    builder.add_edge(
        access_token_node.out("access_token"),
        prepare_impersonate.in_port("access_token"),
    )?;
    builder.add_edge(
        should_impersonate.out("should"),
        prepare_impersonate.in_port("should_impersonate"),
    )?;
    builder.add_edge(
        prepare_impersonate.out("request"),
        execute_impersonate.in_port("request"),
    )?;
    builder.add_edge(
        prepare_impersonate.out("skip"),
        execute_impersonate.in_port("skip"),
    )?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_impersonate.in_port(RESOURCE_API_NETWORK),
    )?;
    builder.add_edge(
        execute_impersonate.out("response"),
        parse_impersonate.in_port("response"),
    )?;
    builder.add_edge(
        access_token_node.out("access_token"),
        parse_impersonate.in_port("base_access_token"),
    )?;

    // ---------------------------------------------------------------------
    // Secret Manager upsert: check -> create -> addVersion
    // ---------------------------------------------------------------------

    let prepare_get = builder.add_node_after(
        Node::opaque(
            "prepare_secret_get",
            vec![
                port("access_token", "String"),
                port("project", "String"),
                port("secret", "String"),
            ],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(GcpOps::PrepareSecretGet),
        ),
        &parse_impersonate,
    )?;

    let execute_get = builder.add_node_after(
        Node::opaque(
            "execute_secret_get",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_get,
    )?;

    let parse_get = builder.add_node_after(
        Node::opaque(
            "parse_secret_get",
            vec![port("response", "TransportResponse")],
            vec![port("exists", "Bool")],
            DynOp::new(GcpOps::ParseSecretGet),
        ),
        &execute_get,
    )?;

    builder.add_edge(
        parse_impersonate.out("access_token"),
        prepare_get.in_port("access_token"),
    )?;
    builder.add_edge(prepare_get.out("request"), execute_get.in_port("request"))?;
    builder.add_edge(prepare_get.out("skip"), execute_get.in_port("skip"))?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_get.in_port(RESOURCE_API_NETWORK),
    )?;
    builder.add_edge(execute_get.out("response"), parse_get.in_port("response"))?;

    let prepare_create = builder.add_node_after(
        Node::opaque(
            "prepare_secret_create",
            vec![
                port("access_token", "String"),
                port("project", "String"),
                port("secret", "String"),
                port("exists", "Bool"),
            ],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(GcpOps::PrepareSecretCreate),
        ),
        &parse_get,
    )?;

    let execute_create = builder.add_node_after(
        Node::opaque(
            "execute_secret_create",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse"), port("skip", "Bool")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_create,
    )?;

    builder.add_edge(
        parse_impersonate.out("access_token"),
        prepare_create.in_port("access_token"),
    )?;
    builder.add_edge(parse_get.out("exists"), prepare_create.in_port("exists"))?;
    builder.add_edge(
        prepare_create.out("request"),
        execute_create.in_port("request"),
    )?;
    builder.add_edge(prepare_create.out("skip"), execute_create.in_port("skip"))?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_create.in_port(RESOURCE_API_NETWORK),
    )?;

    let prepare_add = builder.add_node_after(
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
            DynOp::new(GcpOps::PrepareSecretAddVersion),
        ),
        &execute_create,
    )?;

    let execute_add = builder.add_node_after(
        Node::opaque(
            "execute_secret_add_version",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_add,
    )?;

    let parse_add = builder.add_node_after(
        Node::opaque(
            "parse_secret_add_version",
            vec![port("response", "TransportResponse")],
            vec![port("version", "String")],
            DynOp::new(GcpOps::ParseSecretAddVersion),
        ),
        &execute_add,
    )?;

    builder.add_edge(
        parse_impersonate.out("access_token"),
        prepare_add.in_port("access_token"),
    )?;
    builder.add_edge(
        execute_create.out("skip"),
        prepare_add.in_port("create_done"),
    )?;
    builder.add_edge(prepare_add.out("request"), execute_add.in_port("request"))?;
    builder.add_edge(prepare_add.out("skip"), execute_add.in_port("skip"))?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_add.in_port(RESOURCE_API_NETWORK),
    )?;
    builder.add_edge(execute_add.out("response"), parse_add.in_port("response"))?;

    Ok(builder.build())
}

pub fn build_gcp_secret_manager_upsert_graph_github(
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
    build_gcp_secret_manager_upsert_graph(GcpRuntimeKind::GitHubActions)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-upsert-metadata",
    builder = "build_gcp_secret_manager_upsert_graph_metadata()",
    returns_result
)]
pub fn build_gcp_secret_manager_upsert_graph_metadata(
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
    build_gcp_secret_manager_upsert_graph(GcpRuntimeKind::GcpMetadata)
}

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gcp-wif-secret-upsert-local",
    builder = "build_gcp_secret_manager_upsert_graph_local()",
    returns_result
)]
pub fn build_gcp_secret_manager_upsert_graph_local(
) -> Result<Dag<GcpSecretManagerGraphOp>, BuilderError> {
    build_gcp_secret_manager_upsert_graph(GcpRuntimeKind::LocalDev)
}

// ---------------------------------------------------------------------------
// Local auth upsert sub-DAG (shared by credential and upsert graphs)
// ---------------------------------------------------------------------------

/// Public accessor for the local auth upsert sub-DAG (used by discovery_graph).
pub fn build_local_auth_upsert_dag_pub() -> Dag<GcpSecretManagerGraphOp> {
    build_local_auth_upsert_dag()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum EnsureIamBindingMode {
    ProjectPolicy,
    ServiceAccountPolicy,
}

/// Add IAM ensure nodes to a graph builder (local dev only).
///
/// Uses REST API (getIamPolicy + setIamPolicy) to ensure the SA has
/// `roles/secretmanager.secretAccessor` on the secrets project.
/// Fast in the common case (binding exists = single REST call, ~1s).
///
/// Tolerates PERMISSION_DENIED gracefully.
fn add_ensure_iam_nodes(
    builder: &mut DagBuilder<GcpSecretManagerGraphOp>,
    net_env: &NodeRef<GcpSecretManagerGraphOp>,
    access_token_node: &NodeRef<GcpSecretManagerGraphOp>,
    runtime: GcpRuntimeKind,
) -> Result<(), BuilderError> {
    add_ensure_iam_nodes_with_mode(
        builder,
        net_env,
        access_token_node,
        runtime,
        EnsureIamBindingMode::ProjectPolicy,
    )
}

/// Add SA-level IAM binding ensure nodes to a graph builder (local dev only).
///
/// This path uses `roles/iam.workloadIdentityUser` policy checks against the
/// service-account IAM policy and expects an additional `member` input.
#[allow(dead_code)]
pub fn add_ensure_sa_iam_nodes(
    builder: &mut DagBuilder<GcpSecretManagerGraphOp>,
    net_env: &NodeRef<GcpSecretManagerGraphOp>,
    access_token_node: &NodeRef<GcpSecretManagerGraphOp>,
    runtime: GcpRuntimeKind,
) -> Result<(), BuilderError> {
    add_ensure_iam_nodes_with_mode(
        builder,
        net_env,
        access_token_node,
        runtime,
        EnsureIamBindingMode::ServiceAccountPolicy,
    )
}

fn add_ensure_iam_nodes_with_mode(
    builder: &mut DagBuilder<GcpSecretManagerGraphOp>,
    net_env: &NodeRef<GcpSecretManagerGraphOp>,
    access_token_node: &NodeRef<GcpSecretManagerGraphOp>,
    runtime: GcpRuntimeKind,
    mode: EnsureIamBindingMode,
) -> Result<(), BuilderError> {
    if !matches!(runtime, GcpRuntimeKind::LocalDev) {
        return Ok(());
    }

    let (
        prepare_node_id,
        execute_get_node_id,
        check_node_id,
        execute_set_node_id,
        parse_node_id,
        prepare_op,
        check_op,
        parse_op,
        include_member_port,
    ) = match mode {
        EnsureIamBindingMode::ProjectPolicy => (
            "prepare_ensure_iam",
            "execute_get_iam",
            "check_iam_binding",
            "execute_set_iam",
            "parse_set_iam",
            GcpOps::PrepareEnsureIamBinding,
            GcpOps::CheckAndPrepareIamBinding,
            GcpOps::ParseSetIamBinding,
            false,
        ),
        EnsureIamBindingMode::ServiceAccountPolicy => (
            "prepare_ensure_sa_iam",
            "execute_get_sa_iam",
            "check_sa_iam_binding",
            "execute_set_sa_iam",
            "parse_set_sa_iam",
            GcpOps::PrepareEnsureSaIamBinding,
            GcpOps::CheckAndPrepareSaIamBinding,
            GcpOps::ParseSetSaIamBinding,
            true,
        ),
    };

    let mut prepare_inputs = vec![
        port("access_token", "String"),
        port("project", "String"),
        port("service_account", "String"),
    ];
    let mut prepare_outputs = vec![
        port("request", "TransportRequest"),
        port("skip", "Bool"),
        port("service_account", "String"),
        port("project", "String"),
    ];
    let mut check_inputs = vec![
        port("response", "TransportResponse"),
        port("access_token", "String"),
        port("project", "String"),
        port("service_account", "String"),
    ];
    if include_member_port {
        prepare_inputs.push(port("member", "String"));
        prepare_outputs.push(port("member", "String"));
        check_inputs.push(port("member", "String"));
    }

    let prepare_ensure_iam = builder.add_node_after(
        Node::opaque(
            prepare_node_id,
            prepare_inputs,
            prepare_outputs,
            DynOp::new(prepare_op),
        ),
        access_token_node,
    )?;

    let execute_get_iam = builder.add_node_after(
        Node::opaque(
            execute_get_node_id,
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_ensure_iam,
    )?;

    let check_iam = builder.add_node_after(
        Node::opaque(
            check_node_id,
            check_inputs,
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(check_op),
        ),
        &execute_get_iam,
    )?;

    let execute_set_iam = builder.add_node_after(
        Node::opaque(
            execute_set_node_id,
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("api:network", "NetworkHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &check_iam,
    )?;

    let parse_set_iam = builder.add_node_after(
        Node::opaque(
            parse_node_id,
            vec![port("response", "TransportResponse")],
            vec![port("ok", "Bool")],
            DynOp::new(parse_op),
        ),
        &execute_set_iam,
    )?;

    builder.add_edge(
        prepare_ensure_iam.out("request"),
        execute_get_iam.in_port("request"),
    )?;
    builder.add_edge(
        prepare_ensure_iam.out("skip"),
        execute_get_iam.in_port("skip"),
    )?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_get_iam.in_port(RESOURCE_API_NETWORK),
    )?;

    builder.add_edge(
        execute_get_iam.out("response"),
        check_iam.in_port("response"),
    )?;
    builder.add_edge(
        prepare_ensure_iam.out("service_account"),
        check_iam.in_port("service_account"),
    )?;
    builder.add_edge(
        prepare_ensure_iam.out("project"),
        check_iam.in_port("project"),
    )?;
    if include_member_port {
        builder.add_edge(
            prepare_ensure_iam.out("member"),
            check_iam.in_port("member"),
        )?;
    }

    builder.add_edge(check_iam.out("request"), execute_set_iam.in_port("request"))?;
    builder.add_edge(check_iam.out("skip"), execute_set_iam.in_port("skip"))?;
    builder.add_edge(
        net_env.out(NetEnv::PORT),
        execute_set_iam.in_port(RESOURCE_API_NETWORK),
    )?;

    builder.add_edge(
        execute_set_iam.out("response"),
        parse_set_iam.in_port("response"),
    )?;

    builder.add_edge(
        access_token_node.out("access_token"),
        prepare_ensure_iam.in_port("access_token"),
    )?;
    builder.add_edge(
        access_token_node.out("access_token"),
        check_iam.in_port("access_token"),
    )?;
    Ok(())
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
        vec![port(NetEnv::PORT, "NetworkHandle")],
        DynOp::new(NetEnv),
    ));

    // ========================================================================
    // Check phase: does ADC file exist?
    // ========================================================================

    dag.add_node(Node::opaque(
        "prepare_check",
        vec![],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(GcpOps::PrepareCheckAdc),
    ));

    dag.add_node(Node::opaque(
        "execute_check",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_check",
        vec![port("response", "TransportResponse")],
        vec![port("exists", "Bool")],
        DynOp::new(GcpOps::ParseCheckAdc),
    ));

    // Check edges
    dag.add_edge(Edge::new(
        "prepare_check",
        "request",
        "execute_check",
        "request",
    ));
    dag.add_edge(Edge::new("prepare_check", "skip", "execute_check", "skip"));
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_check",
        RESOURCE_API_NETWORK,
    ));
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
        DynOp::new(GcpOps::PrepareReadAdc),
    ));

    dag.add_node(Node::opaque(
        "execute_read_adc",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
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
        DynOp::new(GcpOps::ParseAdcCredentials),
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
        DynOp::new(GcpOps::PrepareOAuth2Refresh),
    ));

    // Step 4: Execute OAuth2 refresh
    dag.add_node(Node::opaque(
        "execute_oauth2",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
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
        DynOp::new(GcpOps::ParseTryRefresh),
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
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_read_adc",
        RESOURCE_API_NETWORK,
    ));
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
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_oauth2",
        RESOURCE_API_NETWORK,
    ));
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
        DynOp::new(GcpOps::PrepareGcloudAuth),
    ));

    dag.add_node(Node::opaque(
        "execute_gcloud_auth",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_gcloud_auth",
        vec![port("response", "TransportResponse")],
        vec![port("ok", "Bool")],
        DynOp::new(GcpOps::ParseGcloudAuth),
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
        NetEnv::PORT,
        "execute_gcloud_auth",
        RESOURCE_API_NETWORK,
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
        DynOp::new(GcpOps::PrepareReadAdc),
    ));

    dag.add_node(Node::opaque(
        "execute_reread_adc",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_reread_adc",
        vec![port("response", "TransportResponse")],
        vec![
            port("client_id", "String"),
            port("client_secret", "String"),
            port("refresh_token", "String"),
        ],
        DynOp::new(GcpOps::ParseAdcCredentials),
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
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_reread_adc",
        RESOURCE_API_NETWORK,
    ));
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
        DynOp::new(GcpOps::PrepareOAuth2Refresh),
    ));

    dag.add_node(Node::opaque(
        "execute_retry_oauth2",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_retry_refresh",
        vec![port("response", "TransportResponse")],
        vec![
            optional("access_token", "OptionalString"),
            optional("expires_in", "OptionalInt"),
        ],
        DynOp::new(GcpOps::ParseOAuth2Refresh),
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
        NetEnv::PORT,
        "execute_retry_oauth2",
        RESOURCE_API_NETWORK,
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
        DynOp::new(GcpOps::MergeAuthResult),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_builder_with_net_and_access_token() -> (
        DagBuilder<GcpSecretManagerGraphOp>,
        NodeRef<GcpSecretManagerGraphOp>,
        NodeRef<GcpSecretManagerGraphOp>,
    ) {
        let mut builder: DagBuilder<GcpSecretManagerGraphOp> = DagBuilder::new();
        let net_env = builder
            .add_root_node(Node::opaque(
                "net_env",
                vec![],
                vec![port(NetEnv::PORT, "NetworkHandle")],
                DynOp::new(NetEnv),
            ))
            .expect("net_env");
        let access_token = builder
            .add_root_node(Node::opaque(
                "access_token_source",
                vec![],
                vec![port("access_token", "String")],
                DynOp::new(GcpOps::ResolveRuntime),
            ))
            .expect("access_token_source");
        (builder, net_env, access_token)
    }

    #[test]
    fn add_ensure_sa_iam_nodes_wires_member_port_chain() {
        let (mut builder, net_env, access_token) = test_builder_with_net_and_access_token();
        add_ensure_sa_iam_nodes(
            &mut builder,
            &net_env,
            &access_token,
            GcpRuntimeKind::LocalDev,
        )
        .unwrap();
        let dag = builder.build();

        let prepare = dag
            .nodes
            .iter()
            .find(|n| n.id.0 == "prepare_ensure_sa_iam")
            .expect("prepare_ensure_sa_iam node should be present");
        assert!(
            prepare.inputs.iter().any(|p| p.name.0 == "member"),
            "prepare_ensure_sa_iam should expose member input"
        );
        let check = dag
            .nodes
            .iter()
            .find(|n| n.id.0 == "check_sa_iam_binding")
            .expect("check_sa_iam_binding node should be present");
        assert!(
            check.inputs.iter().any(|p| p.name.0 == "member"),
            "check_sa_iam_binding should consume member input"
        );
        assert!(
            dag.edges.iter().any(|edge| {
                edge.from_node.0 == "prepare_ensure_sa_iam"
                    && edge.from_port.0 == "member"
                    && edge.to_node.0 == "check_sa_iam_binding"
                    && edge.to_port.0 == "member"
            }),
            "member passthrough edge should exist for SA IAM ensure chain"
        );
    }

    #[test]
    fn add_ensure_sa_iam_nodes_is_noop_for_non_local_runtime() {
        let (mut builder, net_env, access_token) = test_builder_with_net_and_access_token();
        add_ensure_sa_iam_nodes(
            &mut builder,
            &net_env,
            &access_token,
            GcpRuntimeKind::GitHubActions,
        )
        .unwrap();
        let dag = builder.build();
        assert!(
            dag.nodes
                .iter()
                .all(|node| !node.id.0.starts_with("prepare_ensure_sa_iam")),
            "non-local runtimes should not add ensure_sa_iam nodes"
        );
    }
}
