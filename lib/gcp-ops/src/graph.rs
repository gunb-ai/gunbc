//! DAGs for GCP WIF + Secret Manager.

use crate::ops::{GcpOps, GcpRuntimeKind};
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::build::{optional, port};
use gunbc_ir::{Dag, DagBuilder, Node, Value};
use gunbc_lib_transport::TransportOps;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum GcpSecretManagerGraphOp {
    Gcp(GcpOps),
    Transport(TransportOps),
}

impl Executable for GcpSecretManagerGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GcpSecretManagerGraphOp::Gcp(op) => op.execute(inputs),
            GcpSecretManagerGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build a GCP Secret Manager credential acquisition graph for the given runtime.
///
/// Entrypoints:
/// - `audience`: WIF provider audience
/// - `request_url`: GitHub OIDC request URL (GitHub runtime only)
/// - `request_token`: GitHub OIDC request token (GitHub runtime only)
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

    // ---------------------------------------------------------------------
    // OIDC subject token acquisition
    // ---------------------------------------------------------------------

    let subject_token_node = match runtime {
        GcpRuntimeKind::GitHubActions => {
            let prepare = builder
                .add_root_node(Node::opaque(
                    "prepare_github_oidc",
                    vec![
                        port("audience", "String"),
                        port("request_url", "String"),
                        port("request_token", "String"),
                    ],
                    vec![port("request", "TransportRequest"), port("skip", "Bool")],
                    GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareGitHubOidcRequest),
                ))
                .expect("prepare_github_oidc");

            let execute = builder
                .add_node_after(
                    Node::opaque(
                        "execute_github_oidc",
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
                        vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
                .add_edge(execute.out("response"), parse.in_port("response"))
                .expect("execute_metadata_oidc.response -> parse_metadata_oidc.response");

            parse
        }
    };

    // ---------------------------------------------------------------------
    // STS exchange (subject_token -> access_token)
    // ---------------------------------------------------------------------

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
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
        .add_edge(execute_sts.out("response"), parse_sts.in_port("response"))
        .expect("execute_sts.response -> parse_sts.response");

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
                    optional("lifetime_seconds", "Int"),
                ],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareImpersonate),
            ),
            &parse_sts,
        )
        .expect("prepare_impersonate");

    let execute_impersonate = builder
        .add_node_after(
            Node::opaque(
                "execute_impersonate",
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
            parse_sts.out("access_token"),
            prepare_impersonate.in_port("access_token"),
        )
        .expect("parse_sts.access_token -> prepare_impersonate.access_token");
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
                    optional("version", "String"),
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
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
        .add_edge(
            prepare_secret.out("skip"),
            execute_secret.in_port("skip"),
        )
        .expect("prepare_secret.skip -> execute_secret.skip");
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
                    optional("header_name", "String"),
                    port("source_id", "String"),
                ],
                vec![port("credential", "Credential")],
                GcpSecretManagerGraphOp::Gcp(GcpOps::BuildCredential),
            ),
            &parse_secret,
        )
        .expect("build_credential");

    builder
        .add_edge(parse_secret.out("secret"), build_credential.in_port("secret"))
        .expect("parse_secret.secret -> build_credential.secret");

    builder.build()
}

pub fn build_gcp_secret_manager_credential_graph_github() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_credential_graph(GcpRuntimeKind::GitHubActions)
}

pub fn build_gcp_secret_manager_credential_graph_metadata() -> Dag<GcpSecretManagerGraphOp> {
    build_gcp_secret_manager_credential_graph(GcpRuntimeKind::GcpMetadata)
}
