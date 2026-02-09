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
use crate::graph::GcpSecretManagerGraphOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::build::{port, resource, AccessMode};
use gunbc_ir::{Dag, Edge, Node, Value};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::NetEnv;
use std::collections::HashMap;

/// Union op type for the discovery DAG.
#[derive(Debug, Clone)]
pub enum GcpDiscoveryGraphOp {
    /// Discovery-specific ops (list/parse/assemble).
    Discovery(GcpDiscoveryOps),
    /// Reused GCP ops (for local auth sub-DAG).
    Gcp(GcpSecretManagerGraphOp),
    /// Transport execution.
    Transport(TransportOps),
    /// Network environment.
    NetEnv(NetEnv),
}

impl Executable for GcpDiscoveryGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GcpDiscoveryGraphOp::Discovery(op) => op.execute(inputs),
            GcpDiscoveryGraphOp::Gcp(op) => op.execute(inputs),
            GcpDiscoveryGraphOp::Transport(op) => op.execute(inputs),
            GcpDiscoveryGraphOp::NetEnv(op) => op.execute(inputs),
        }
    }
}

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
        vec![port("net", "NetworkHandle")],
        GcpDiscoveryGraphOp::NetEnv(NetEnv),
    ));

    // Local auth sub-DAG (provides access_token)
    dag.add_node(Node::subdag(
        "local_auth",
        crate::graph::build_local_auth_upsert_dag_pub()
            .map_ops(&mut GcpDiscoveryGraphOp::Gcp),
    ));

    // =========================================================================
    // Parallel discovery: list_projects, list_wif_pools, list_sa, list_secrets,
    // list_buckets, get_iam_policy — all fan out from auth.access_token
    // =========================================================================

    // ----- list_projects -----
    dag.add_node(Node::opaque(
        "prepare_list_projects",
        vec![port("access_token", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::PrepareListProjects),
    ));
    dag.add_node(Node::opaque(
        "execute_list_projects",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpDiscoveryGraphOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_projects",
        vec![port("response", "TransportResponse")],
        vec![port("projects", "Json")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::ParseListProjects),
    ));
    dag.add_edge(Edge::new("local_auth", "access_token", "prepare_list_projects", "access_token"));
    dag.add_edge(Edge::new("prepare_list_projects", "request", "execute_list_projects", "request"));
    dag.add_edge(Edge::new("prepare_list_projects", "skip", "execute_list_projects", "skip"));
    dag.add_edge(Edge::new("net_env", "net", "execute_list_projects", "res:net"));
    dag.add_edge(Edge::new("execute_list_projects", "response", "parse_list_projects", "response"));

    // ----- list_wif_pools -----
    dag.add_node(Node::opaque(
        "prepare_list_wif_pools",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::PrepareListWifPools),
    ));
    dag.add_node(Node::opaque(
        "execute_list_wif_pools",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpDiscoveryGraphOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_wif_pools",
        vec![port("response", "TransportResponse")],
        vec![port("wif_pools", "Json")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::ParseListWifPools),
    ));
    dag.add_edge(Edge::new("local_auth", "access_token", "prepare_list_wif_pools", "access_token"));
    dag.add_edge(Edge::new("prepare_list_wif_pools", "request", "execute_list_wif_pools", "request"));
    dag.add_edge(Edge::new("prepare_list_wif_pools", "skip", "execute_list_wif_pools", "skip"));
    dag.add_edge(Edge::new("net_env", "net", "execute_list_wif_pools", "res:net"));
    dag.add_edge(Edge::new("execute_list_wif_pools", "response", "parse_list_wif_pools", "response"));

    // ----- list_service_accounts -----
    dag.add_node(Node::opaque(
        "prepare_list_sa",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::PrepareListServiceAccounts),
    ));
    dag.add_node(Node::opaque(
        "execute_list_sa",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpDiscoveryGraphOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_sa",
        vec![port("response", "TransportResponse")],
        vec![port("service_accounts", "Json")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::ParseListServiceAccounts),
    ));
    dag.add_edge(Edge::new("local_auth", "access_token", "prepare_list_sa", "access_token"));
    dag.add_edge(Edge::new("prepare_list_sa", "request", "execute_list_sa", "request"));
    dag.add_edge(Edge::new("prepare_list_sa", "skip", "execute_list_sa", "skip"));
    dag.add_edge(Edge::new("net_env", "net", "execute_list_sa", "res:net"));
    dag.add_edge(Edge::new("execute_list_sa", "response", "parse_list_sa", "response"));

    // ----- list_secrets -----
    dag.add_node(Node::opaque(
        "prepare_list_secrets",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::PrepareListSecrets),
    ));
    dag.add_node(Node::opaque(
        "execute_list_secrets",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpDiscoveryGraphOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_secrets",
        vec![port("response", "TransportResponse")],
        vec![port("secrets", "Json")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::ParseListSecrets),
    ));
    dag.add_edge(Edge::new("local_auth", "access_token", "prepare_list_secrets", "access_token"));
    dag.add_edge(Edge::new("prepare_list_secrets", "request", "execute_list_secrets", "request"));
    dag.add_edge(Edge::new("prepare_list_secrets", "skip", "execute_list_secrets", "skip"));
    dag.add_edge(Edge::new("net_env", "net", "execute_list_secrets", "res:net"));
    dag.add_edge(Edge::new("execute_list_secrets", "response", "parse_list_secrets", "response"));

    // ----- list_buckets -----
    dag.add_node(Node::opaque(
        "prepare_list_buckets",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::PrepareListBuckets),
    ));
    dag.add_node(Node::opaque(
        "execute_list_buckets",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpDiscoveryGraphOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_list_buckets",
        vec![port("response", "TransportResponse")],
        vec![port("buckets", "Json")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::ParseListBuckets),
    ));
    dag.add_edge(Edge::new("local_auth", "access_token", "prepare_list_buckets", "access_token"));
    dag.add_edge(Edge::new("prepare_list_buckets", "request", "execute_list_buckets", "request"));
    dag.add_edge(Edge::new("prepare_list_buckets", "skip", "execute_list_buckets", "skip"));
    dag.add_edge(Edge::new("net_env", "net", "execute_list_buckets", "res:net"));
    dag.add_edge(Edge::new("execute_list_buckets", "response", "parse_list_buckets", "response"));

    // ----- get_iam_policy -----
    dag.add_node(Node::opaque(
        "prepare_get_iam_policy",
        vec![port("access_token", "String"), port("project", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::PrepareGetIamPolicy),
    ));
    dag.add_node(Node::opaque(
        "execute_get_iam_policy",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("net", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpDiscoveryGraphOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_get_iam_policy",
        vec![
            port("response", "TransportResponse"),
            port("project", "String"),
        ],
        vec![port("iam_policies", "Json")],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::ParseGetIamPolicy),
    ));
    dag.add_edge(Edge::new("local_auth", "access_token", "prepare_get_iam_policy", "access_token"));
    dag.add_edge(Edge::new("prepare_get_iam_policy", "request", "execute_get_iam_policy", "request"));
    dag.add_edge(Edge::new("prepare_get_iam_policy", "skip", "execute_get_iam_policy", "skip"));
    dag.add_edge(Edge::new("net_env", "net", "execute_get_iam_policy", "res:net"));
    dag.add_edge(Edge::new("execute_get_iam_policy", "response", "parse_get_iam_policy", "response"));

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
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::AssembleInfraSpec),
    ));

    // Wire parse outputs -> assemble inputs
    dag.add_edge(Edge::new("parse_list_projects", "projects", "assemble_infra_spec", "projects"));
    dag.add_edge(Edge::new("parse_list_wif_pools", "wif_pools", "assemble_infra_spec", "wif_pools"));
    dag.add_edge(Edge::new("parse_list_sa", "service_accounts", "assemble_infra_spec", "service_accounts"));
    dag.add_edge(Edge::new("parse_list_secrets", "secrets", "assemble_infra_spec", "secrets"));
    dag.add_edge(Edge::new("parse_list_buckets", "buckets", "assemble_infra_spec", "buckets"));
    dag.add_edge(Edge::new("parse_get_iam_policy", "iam_policies", "assemble_infra_spec", "iam_policies"));

    // =========================================================================
    // Config generation: InfraSpec -> CloudConfigSpec TOML
    // =========================================================================

    dag.add_node(Node::opaque(
        "generate_config_spec",
        vec![port("infra_spec", "Json")],
        vec![
            port("config_toml", "String"),
            port("config_spec", "Json"),
        ],
        GcpDiscoveryGraphOp::Discovery(GcpDiscoveryOps::GenerateConfigSpec),
    ));

    dag.add_edge(Edge::new("assemble_infra_spec", "infra_spec", "generate_config_spec", "infra_spec"));

    dag
}
