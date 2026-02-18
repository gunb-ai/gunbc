use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;

use daglang_lower::{LoweredOp, ServiceCallMetadata};
use gunbc_ir::{Dag, Node};
use serde_json::json;

use super::OutputFormat;

pub fn render_triplets(dag: &Dag<LoweredOp>, format: OutputFormat) -> String {
    let triplets = collect_transport_triplets(dag);
    match format {
        OutputFormat::Text => {
            let mut out = String::new();
            out.push_str("TransportTriplets:\n");
            if triplets.is_empty() {
                out.push_str("  (none)\n");
                return out;
            }
            for (index, triplet) in triplets.iter().enumerate() {
                writeln!(out, "  [{index}]").ok();
                writeln!(out, "    prepare: {}", triplet.prepare_node).ok();
                writeln!(out, "    execute: {}", triplet.execute_node).ok();
                if triplet.parse_nodes.is_empty() {
                    out.push_str("    parse: (none)\n");
                } else {
                    writeln!(out, "    parse: {}", triplet.parse_nodes.join(", ")).ok();
                }
                if let Some(metadata) = &triplet.service_metadata {
                    writeln!(out, "    transport_class: {:?}", metadata.transport).ok();
                    writeln!(out, "    service: {}", metadata.service).ok();
                    writeln!(out, "    operation: {}", metadata.operation).ok();
                    writeln!(out, "    idempotent: {}", metadata.idempotent).ok();
                    writeln!(out, "    readonly: {}", metadata.readonly).ok();
                    if metadata.permissions.is_empty() {
                        out.push_str("    permissions: (none)\n");
                    } else {
                        writeln!(out, "    permissions: {}", metadata.permissions.join(", ")).ok();
                    }
                }
            }
            out
        }
        OutputFormat::Json => {
            let triplets_json = triplets
                .iter()
                .map(|triplet| {
                    json!({
                        "prepare_node": triplet.prepare_node,
                        "execute_node": triplet.execute_node,
                        "parse_nodes": triplet.parse_nodes,
                        "service_metadata": triplet.service_metadata
                    })
                })
                .collect::<Vec<_>>();
            json!({ "triplets": triplets_json }).to_string()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TransportTriplet {
    pub(super) prepare_node: String,
    pub(super) execute_node: String,
    pub(super) parse_nodes: Vec<String>,
    pub(super) service_metadata: Option<ServiceCallMetadata>,
}

pub(super) fn collect_transport_triplets(dag: &Dag<LoweredOp>) -> Vec<TransportTriplet> {
    let node_by_id = dag
        .nodes
        .iter()
        .map(|node| (node.id.0.as_str(), node))
        .collect::<HashMap<_, _>>();
    let mut unique = BTreeSet::<TransportTriplet>::new();

    for edge in &dag.edges {
        let Some(prepare_node) = node_by_id.get(edge.from_node.0.as_str()).copied() else {
            continue;
        };
        let Some(execute_node) = node_by_id.get(edge.to_node.0.as_str()).copied() else {
            continue;
        };
        if node_output_port_type(prepare_node, edge.from_port.0.as_str())
            != Some("TransportRequest")
            || node_input_port_type(execute_node, edge.to_port.0.as_str())
                != Some("TransportRequest")
        {
            continue;
        }

        let mut parse_nodes = dag
            .edges
            .iter()
            .filter(|next_edge| next_edge.from_node.0 == edge.to_node.0)
            .filter_map(|next_edge| {
                let parse_node = node_by_id.get(next_edge.to_node.0.as_str()).copied()?;
                (node_output_port_type(execute_node, next_edge.from_port.0.as_str())
                    == Some("TransportResponse")
                    && node_input_port_type(parse_node, next_edge.to_port.0.as_str())
                        == Some("TransportResponse"))
                .then_some(next_edge.to_node.0.clone())
            })
            .collect::<Vec<_>>();
        parse_nodes.sort();
        parse_nodes.dedup();
        let service_metadata = match &execute_node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => op.service_call_metadata().cloned(),
            gunbc_ir::node::NodeBody::SubDag(_) => None,
        };

        unique.insert(TransportTriplet {
            prepare_node: edge.from_node.0.clone(),
            execute_node: edge.to_node.0.clone(),
            parse_nodes,
            service_metadata,
        });
    }

    unique.into_iter().collect()
}

fn node_input_port_type<'a>(node: &'a Node<LoweredOp>, port_name: &str) -> Option<&'a str> {
    node.inputs
        .iter()
        .find(|port| port.name.0 == port_name)
        .map(|port| port.type_id.0.as_str())
}

fn node_output_port_type<'a>(node: &'a Node<LoweredOp>, port_name: &str) -> Option<&'a str> {
    node.outputs
        .iter()
        .find(|port| port.name.0 == port_name)
        .map(|port| port.type_id.0.as_str())
}
