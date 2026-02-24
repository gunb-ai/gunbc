//! Infra discovery DAG: authenticates, queries GCP APIs in parallel, assembles InfraSpec.
//!
//! ```text
//! ADC auth (local) ──→ list_projects ──┐
//!                  ──→ list_wif_pools ──┤
//!                  ──→ list_service_accounts ──────────────┤
//!                  ──→ list_secrets ──────────────────────┤
//!                  ──→ list_buckets ──────────────────────┤
//!                  ──→ get_iam_policy ────────────────────┤
//!                                                         ▼
//!                                                assemble_infra_spec
//!                                                         ▼
//!                                                generate_config_spec
//! ```

use crate::discovery_ops::GcpDiscoveryOps;
use crate::ops::GcpOps;
use gunbc_exec::DynOp;
use gunbc_ir::build::{optional, port, resource, AccessMode};
use gunbc_ir::{Dag, Edge, Node, RESOURCE_API_NETWORK};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::NetEnv;

pub type GcpDiscoveryGraphOp = DynOp;

/// Build the infra discovery DAG.
///
/// Uses a flat `Dag::new()` + `add_node` / `add_edge` approach for simplicity
/// (the discovery DAG is wide and parallel, not deeply nested).
///
/// Entrypoints:
/// - `project`: GCP project ID to discover infrastructure for
///
/// Outputs:
/// - `config_toml`: String — generated TOML configuration
/// - `config_spec`: Json — CloudConfigSpec as JSON
/// - `infra_spec`: Json — full GcpInfraSpec as JSON
pub fn build_infra_discovery_dag() -> Dag<GcpDiscoveryGraphOp> {
    let mut dag = Dag::new();

    // Network environment
    dag.add_node(Node::opaque(
        "net_env",
        vec![],
        vec![port(NetEnv::PORT, "NetworkHandle")],
        DynOp::new(NetEnv),
    ));

    // Local auth sub-DAG (provides access_token)
    dag.add_node(Node::subdag("local_auth", build_local_auth_upsert_dag()));

    // =========================================================================
    // Parallel discovery: list_projects, list_wif_pools, list_sa, list_secrets,
    // list_buckets, get_iam_policy — all fan out from auth.access_token
    // =========================================================================

    // ----- list_projects -----
    dag.add_node(Node::opaque(
        "prepare_list_projects",
        vec![port("access_token", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(GcpDiscoveryOps::PrepareListProjects),
    ));
    dag.add_node(Node::opaque(
        "execute_list_projects",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_projects",
        vec![port("response", "TransportResponse")],
        vec![port("projects", "Json")],
        DynOp::new(GcpDiscoveryOps::ParseListProjects),
    ));
    dag.add_edge(Edge::new(
        "local_auth",
        "access_token",
        "prepare_list_projects",
        "access_token",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_projects",
        "request",
        "execute_list_projects",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_projects",
        "skip",
        "execute_list_projects",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_list_projects",
        RESOURCE_API_NETWORK,
    ));
    dag.add_edge(Edge::new(
        "execute_list_projects",
        "response",
        "parse_list_projects",
        "response",
    ));

    // ----- list_wif_pools -----
    dag.add_node(Node::opaque(
        "prepare_list_wif_pools",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(GcpDiscoveryOps::PrepareListWifPools),
    ));
    dag.add_node(Node::opaque(
        "execute_list_wif_pools",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_wif_pools",
        vec![port("response", "TransportResponse")],
        vec![port("wif_pools", "Json")],
        DynOp::new(GcpDiscoveryOps::ParseListWifPools),
    ));
    dag.add_edge(Edge::new(
        "local_auth",
        "access_token",
        "prepare_list_wif_pools",
        "access_token",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_wif_pools",
        "request",
        "execute_list_wif_pools",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_wif_pools",
        "skip",
        "execute_list_wif_pools",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_list_wif_pools",
        RESOURCE_API_NETWORK,
    ));
    dag.add_edge(Edge::new(
        "execute_list_wif_pools",
        "response",
        "parse_list_wif_pools",
        "response",
    ));

    // ----- list_service_accounts -----
    dag.add_node(Node::opaque(
        "prepare_list_sa",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(GcpDiscoveryOps::PrepareListServiceAccounts),
    ));
    dag.add_node(Node::opaque(
        "execute_list_sa",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_sa",
        vec![port("response", "TransportResponse")],
        vec![port("service_accounts", "Json")],
        DynOp::new(GcpDiscoveryOps::ParseListServiceAccounts),
    ));
    dag.add_edge(Edge::new(
        "local_auth",
        "access_token",
        "prepare_list_sa",
        "access_token",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_sa",
        "request",
        "execute_list_sa",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_sa",
        "skip",
        "execute_list_sa",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_list_sa",
        RESOURCE_API_NETWORK,
    ));
    dag.add_edge(Edge::new(
        "execute_list_sa",
        "response",
        "parse_list_sa",
        "response",
    ));

    // ----- list_secrets -----
    dag.add_node(Node::opaque(
        "prepare_list_secrets",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(GcpDiscoveryOps::PrepareListSecrets),
    ));
    dag.add_node(Node::opaque(
        "execute_list_secrets",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_secrets",
        vec![port("response", "TransportResponse")],
        vec![port("secrets", "Json")],
        DynOp::new(GcpDiscoveryOps::ParseListSecrets),
    ));
    dag.add_edge(Edge::new(
        "local_auth",
        "access_token",
        "prepare_list_secrets",
        "access_token",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_secrets",
        "request",
        "execute_list_secrets",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_secrets",
        "skip",
        "execute_list_secrets",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_list_secrets",
        RESOURCE_API_NETWORK,
    ));
    dag.add_edge(Edge::new(
        "execute_list_secrets",
        "response",
        "parse_list_secrets",
        "response",
    ));

    // ----- list_buckets -----
    dag.add_node(Node::opaque(
        "prepare_list_buckets",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(GcpDiscoveryOps::PrepareListBuckets),
    ));
    dag.add_node(Node::opaque(
        "execute_list_buckets",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_buckets",
        vec![port("response", "TransportResponse")],
        vec![port("buckets", "Json")],
        DynOp::new(GcpDiscoveryOps::ParseListBuckets),
    ));
    dag.add_edge(Edge::new(
        "local_auth",
        "access_token",
        "prepare_list_buckets",
        "access_token",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_buckets",
        "request",
        "execute_list_buckets",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_list_buckets",
        "skip",
        "execute_list_buckets",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_list_buckets",
        RESOURCE_API_NETWORK,
    ));
    dag.add_edge(Edge::new(
        "execute_list_buckets",
        "response",
        "parse_list_buckets",
        "response",
    ));

    // ----- get_iam_policy -----
    dag.add_node(Node::opaque(
        "prepare_get_iam_policy",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(GcpDiscoveryOps::PrepareGetIamPolicy),
    ));
    dag.add_node(Node::opaque(
        "execute_get_iam_policy",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        DynOp::new(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_get_iam_policy",
        vec![
            port("response", "TransportResponse"),
            port("project", "String"),
        ],
        vec![port("iam_policies", "Json")],
        DynOp::new(GcpDiscoveryOps::ParseGetIamPolicy),
    ));
    dag.add_edge(Edge::new(
        "local_auth",
        "access_token",
        "prepare_get_iam_policy",
        "access_token",
    ));
    dag.add_edge(Edge::new(
        "prepare_get_iam_policy",
        "request",
        "execute_get_iam_policy",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_get_iam_policy",
        "skip",
        "execute_get_iam_policy",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "net_env",
        NetEnv::PORT,
        "execute_get_iam_policy",
        RESOURCE_API_NETWORK,
    ));
    dag.add_edge(Edge::new(
        "execute_get_iam_policy",
        "response",
        "parse_get_iam_policy",
        "response",
    ));

    // =========================================================================
    // Assembly: combine all discovery outputs into InfraSpec
    // =========================================================================

    dag.add_node(Node::opaque(
        "assemble_infra_spec",
        vec![
            port("project", "String"),
            port("projects", "Json"),
            port("wif_pools", "Json"),
            port("wif_providers", "Json"),
            port("service_accounts", "Json"),
            port("secrets", "Json"),
            port("buckets", "Json"),
            port("iam_policies", "Json"),
        ],
        vec![port("infra_spec", "Json")],
        DynOp::new(GcpDiscoveryOps::AssembleInfraSpec),
    ));

    // Wire parse outputs -> assemble inputs
    dag.add_edge(Edge::new(
        "parse_list_projects",
        "projects",
        "assemble_infra_spec",
        "projects",
    ));
    dag.add_edge(Edge::new(
        "parse_list_wif_pools",
        "wif_pools",
        "assemble_infra_spec",
        "wif_pools",
    ));
    dag.add_edge(Edge::new(
        "parse_list_sa",
        "service_accounts",
        "assemble_infra_spec",
        "service_accounts",
    ));
    dag.add_edge(Edge::new(
        "parse_list_secrets",
        "secrets",
        "assemble_infra_spec",
        "secrets",
    ));
    dag.add_edge(Edge::new(
        "parse_list_buckets",
        "buckets",
        "assemble_infra_spec",
        "buckets",
    ));
    dag.add_edge(Edge::new(
        "parse_get_iam_policy",
        "iam_policies",
        "assemble_infra_spec",
        "iam_policies",
    ));

    // =========================================================================
    // Config generation: InfraSpec -> CloudConfigSpec TOML
    // =========================================================================

    dag.add_node(Node::opaque(
        "generate_config_spec",
        vec![port("infra_spec", "Json")],
        vec![port("config_toml", "String"), port("config_spec", "Json")],
        DynOp::new(GcpDiscoveryOps::GenerateConfigSpec),
    ));

    dag.add_edge(Edge::new(
        "assemble_infra_spec",
        "infra_spec",
        "generate_config_spec",
        "infra_spec",
    ));

    dag
}

/// Build the local-dev ADC authentication sub-DAG.
///
/// Flow: check ADC exists -> read ADC -> OAuth2 refresh -> (if expired) gcloud auth
///       -> re-read ADC -> retry refresh -> merge results.
///
/// Outputs: `access_token` (Secret), `expires_in` (Int)
pub(crate) fn build_local_auth_upsert_dag() -> Dag<DynOp> {
    let mut dag = Dag::new();

    dag.add_node(Node::opaque(
        "net_env",
        vec![],
        vec![port(NetEnv::PORT, "NetworkHandle")],
        DynOp::new(NetEnv),
    ));

    // Check phase: does ADC file exist?
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
    dag.add_edge(Edge::new("prepare_check", "request", "execute_check", "request"));
    dag.add_edge(Edge::new("prepare_check", "skip", "execute_check", "skip"));
    dag.add_edge(Edge::new("net_env", NetEnv::PORT, "execute_check", RESOURCE_API_NETWORK));
    dag.add_edge(Edge::new("execute_check", "response", "parse_check", "response"));

    // Try-refresh phase: read ADC -> parse -> OAuth2 refresh -> try parse
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

    dag.add_edge(Edge::new("parse_check", "exists", "prepare_read_adc", "exists"));
    dag.add_edge(Edge::new("prepare_read_adc", "request", "execute_read_adc", "request"));
    dag.add_edge(Edge::new("prepare_read_adc", "skip", "execute_read_adc", "skip"));
    dag.add_edge(Edge::new("net_env", NetEnv::PORT, "execute_read_adc", RESOURCE_API_NETWORK));
    dag.add_edge(Edge::new("execute_read_adc", "response", "parse_adc", "response"));
    dag.add_edge(Edge::new("parse_adc", "client_id", "prepare_oauth2", "client_id"));
    dag.add_edge(Edge::new("parse_adc", "client_secret", "prepare_oauth2", "client_secret"));
    dag.add_edge(Edge::new("parse_adc", "refresh_token", "prepare_oauth2", "refresh_token"));
    dag.add_edge(Edge::new("prepare_oauth2", "request", "execute_oauth2", "request"));
    dag.add_edge(Edge::new("prepare_oauth2", "skip", "execute_oauth2", "skip"));
    dag.add_edge(Edge::new("net_env", NetEnv::PORT, "execute_oauth2", RESOURCE_API_NETWORK));
    dag.add_edge(Edge::new("execute_oauth2", "response", "parse_try_refresh", "response"));

    // Re-auth phase: gcloud auth login -> re-read ADC -> retry refresh
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

    dag.add_edge(Edge::new("parse_try_refresh", "needs_reauth", "prepare_gcloud_auth", "needs_reauth"));
    dag.add_edge(Edge::new("prepare_gcloud_auth", "request", "execute_gcloud_auth", "request"));
    dag.add_edge(Edge::new("prepare_gcloud_auth", "skip", "execute_gcloud_auth", "skip"));
    dag.add_edge(Edge::new("net_env", NetEnv::PORT, "execute_gcloud_auth", RESOURCE_API_NETWORK));
    dag.add_edge(Edge::new("execute_gcloud_auth", "response", "parse_gcloud_auth", "response"));

    // Re-read ADC after gcloud auth
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

    dag.add_edge(Edge::new("parse_gcloud_auth", "ok", "prepare_reread_adc", "exists"));
    dag.add_edge(Edge::new("prepare_reread_adc", "request", "execute_reread_adc", "request"));
    dag.add_edge(Edge::new("prepare_reread_adc", "skip", "execute_reread_adc", "skip"));
    dag.add_edge(Edge::new("net_env", NetEnv::PORT, "execute_reread_adc", RESOURCE_API_NETWORK));
    dag.add_edge(Edge::new("execute_reread_adc", "response", "parse_reread_adc", "response"));

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

    dag.add_edge(Edge::new("parse_reread_adc", "client_id", "prepare_retry_oauth2", "client_id"));
    dag.add_edge(Edge::new("parse_reread_adc", "client_secret", "prepare_retry_oauth2", "client_secret"));
    dag.add_edge(Edge::new("parse_reread_adc", "refresh_token", "prepare_retry_oauth2", "refresh_token"));
    dag.add_edge(Edge::new("prepare_retry_oauth2", "request", "execute_retry_oauth2", "request"));
    dag.add_edge(Edge::new("prepare_retry_oauth2", "skip", "execute_retry_oauth2", "skip"));
    dag.add_edge(Edge::new("net_env", NetEnv::PORT, "execute_retry_oauth2", RESOURCE_API_NETWORK));
    dag.add_edge(Edge::new("execute_retry_oauth2", "response", "parse_retry_refresh", "response"));

    // Merge phase: combine try-refresh and retry-refresh results
    dag.add_node(Node::opaque(
        "merge_auth_result",
        vec![
            optional("try_access_token", "OptionalString"),
            optional("try_expires_in", "OptionalInt"),
            optional("retry_access_token", "OptionalString"),
            optional("retry_expires_in", "OptionalInt"),
        ],
        vec![port("access_token", "Secret"), port("expires_in", "Int")],
        DynOp::new(GcpOps::MergeAuthResult),
    ));

    dag.add_edge(Edge::new("parse_try_refresh", "access_token", "merge_auth_result", "try_access_token"));
    dag.add_edge(Edge::new("parse_try_refresh", "expires_in", "merge_auth_result", "try_expires_in"));
    dag.add_edge(Edge::new("parse_retry_refresh", "access_token", "merge_auth_result", "retry_access_token"));
    dag.add_edge(Edge::new("parse_retry_refresh", "expires_in", "merge_auth_result", "retry_expires_in"));

    dag
}
