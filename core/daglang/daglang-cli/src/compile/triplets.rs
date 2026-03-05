use std::fmt::Write;

use daglang_derive::DerivedArtifacts;
#[cfg(test)]
use daglang_derive::TransportTriplet;
#[cfg(test)]
use daglang_lower::LoweredOp;
#[cfg(test)]
use gunbc_ir::Dag;
use serde_json::json;

use super::OutputFormat;

pub fn render_triplets(derived: &DerivedArtifacts, format: OutputFormat) -> String {
    let triplets = &derived.transport_triplets;
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

#[cfg(test)]
pub(super) fn collect_transport_triplets(dag: &Dag<LoweredOp>) -> Vec<TransportTriplet> {
    daglang_derive::derive_transport_triplets(dag)
}
