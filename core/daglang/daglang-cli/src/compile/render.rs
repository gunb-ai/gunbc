use std::collections::BTreeMap;
use std::fmt::Write;

use daglang_contract::{StageGroup, TestObligations};
use daglang_derive::DerivedArtifacts;
use daglang_lower::LoweredOp;
use gunbc_ir::Dag;
use serde_json::json;

use super::OutputFormat;

pub fn render_canonical_ir_json(dag: &Dag<LoweredOp>) -> Result<String, String> {
    daglang_lower::canonical_ir_json(dag)
        .map_err(|error| format!("failed to serialize canonical IR JSON: {error}"))
}

pub fn render_expand(dag: &Dag<LoweredOp>) -> String {
    let mut out = String::new();
    out.push_str("Nodes:\n");
    for node in &dag.nodes {
        let kind = match &node.body {
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable {
                kind, module, name, ..
            }) => {
                format!("callable::{kind:?} {module}.{name}")
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive {
                module, name, kind, ..
            }) => {
                format!("primitive::{kind:?} {module}.{name}")
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection {
                module,
                callable,
                kind,
            }) => {
                let callable_label = callable
                    .strip_prefix(&format!("{module}::"))
                    .unwrap_or(callable);
                format!("collection::{kind:?} {module}.{callable_label}")
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline {
                module,
                name,
                stages,
                ..
            }) => format!("pipeline {module}.{name} ({stages} stages)"),
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::LoopUnpack { .. }) => {
                "pattern::LoopUnpack".to_string()
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::LoopPack { .. }) => {
                "pattern::LoopPack".to_string()
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::BranchMerge { .. }) => {
                "pattern::BranchMerge".to_string()
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::UnsupportedPattern { name }) => {
                format!("unsupported_pattern::{name}")
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::ExternCall { symbol }) => {
                format!("extern_call::{symbol}")
            }
            gunbc_ir::node::NodeBody::SubDag(_) => "subdag".to_string(),
        };

        writeln!(out, "  - {} [{kind}]", node.id.0).ok();
        if !node.inputs.is_empty() {
            out.push_str("    inputs:\n");
            for input in &node.inputs {
                writeln!(
                    out,
                    "      * {}: {} ({})",
                    input.name.0, input.type_id.0, input.cardinality
                )
                .ok();
            }
        }
        if !node.outputs.is_empty() {
            out.push_str("    outputs:\n");
            for output in &node.outputs {
                writeln!(
                    out,
                    "      * {}: {} ({})",
                    output.name.0, output.type_id.0, output.cardinality
                )
                .ok();
            }
        }
    }

    out.push_str("Edges:\n");
    if dag.edges.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for edge in &dag.edges {
            writeln!(
                out,
                "  - {}.{} -> {}.{}",
                edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
            )
            .ok();
        }
    }
    out
}

pub fn render_manifest(derived: &DerivedArtifacts) -> String {
    let manifest = &derived.manifest;
    let mut out = String::new();
    out.push_str("ProgressManifest:\n");
    writeln!(out, "  total_nodes: {}", manifest.total_nodes).ok();
    writeln!(out, "  total_edges: {}", manifest.total_edges).ok();
    out.push_str("  waves:\n");
    for (index, wave) in manifest.waves.iter().enumerate() {
        writeln!(out, "    [{index}] {}", wave.join(", ")).ok();
    }
    writeln!(
        out,
        "  entrypoint_nodes: {}",
        if manifest.entrypoint_nodes.is_empty() {
            "(none)".to_string()
        } else {
            manifest.entrypoint_nodes.join(", ")
        }
    )
    .ok();
    writeln!(
        out,
        "  boundary_nodes: {}",
        if manifest.boundary_nodes.is_empty() {
            "(none)".to_string()
        } else {
            manifest.boundary_nodes.join(", ")
        }
    )
    .ok();
    out.push_str("  topology:\n");
    for node in &manifest.topology {
        writeln!(out, "    - {} (depth={})", node.id, node.depth).ok();
    }
    out.push_str("  labels:\n");
    for (node_id, label) in &manifest.labels {
        writeln!(out, "    - {node_id}: {label}").ok();
    }
    out.push_str("  subdag_boundaries:\n");
    if manifest.subdag_boundaries.is_empty() {
        out.push_str("    (none)\n");
    } else {
        for boundary in &manifest.subdag_boundaries {
            writeln!(
                out,
                "    - {} label={} inner=[{}]",
                boundary.node_id,
                boundary.label,
                boundary.inner_nodes.join(", ")
            )
            .ok();
        }
    }
    out.push_str("  parallel_groups:\n");
    for group in &manifest.parallel_groups {
        writeln!(
            out,
            "    - depth={} nodes={}",
            group.depth,
            group.nodes.join(", ")
        )
        .ok();
    }
    out.push_str("  scatter_points:\n");
    render_scatter_points_text(&mut out, &manifest.scatter_points);
    writeln!(
        out,
        "  interactive_nodes: {}",
        if manifest.interactive_nodes.is_empty() {
            "(none)".to_string()
        } else {
            manifest.interactive_nodes.join(", ")
        }
    )
    .ok();
    out.push_str("  capture_modes:\n");
    for (node_id, mode) in &manifest.capture_modes {
        writeln!(out, "    - {node_id}: {mode:?}").ok();
    }
    out.push_str("  stage_groups:\n");
    render_stage_groups_text(&mut out, &manifest.stage_groups);
    out.push_str("  resources:\n");
    if manifest.resources.is_empty() {
        out.push_str("    (none)\n");
    } else {
        for (node_id, usages) in &manifest.resources {
            let usage_text = usages
                .iter()
                .map(|usage| format!("{}:{}", usage.resource, usage.usage))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(out, "    - {node_id}: {usage_text}").ok();
        }
    }
    out.push_str(&render_obligations_text(&derived.obligations));
    out
}

fn render_stage_groups_text(out: &mut String, stage_groups: &[StageGroup]) {
    if stage_groups.is_empty() {
        out.push_str("    (none)\n");
        return;
    }
    let mut sections: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for group in stage_groups {
        let (section, stage_name) = split_stage_group_label(&group.stage_id);
        sections
            .entry(section)
            .or_default()
            .push((stage_name, group.nodes.join(", ")));
    }
    for (section, entries) in sections {
        writeln!(
            out,
            "    > [collapsed] {section} ({} stages)",
            entries.len()
        )
        .ok();
        for (stage_name, node_ids) in entries {
            writeln!(out, "      - {stage_name}: {node_ids}").ok();
        }
    }
}

fn render_scatter_points_text(out: &mut String, scatter_points: &[String]) {
    if scatter_points.is_empty() {
        out.push_str("    (none)\n");
        return;
    }
    let mut grouped = BTreeMap::<String, usize>::new();
    for scatter_point in scatter_points {
        *grouped.entry(scatter_point.clone()).or_default() += 1;
    }
    for (group, total) in grouped {
        writeln!(out, "    - {group} [0/{total}]").ok();
    }
}

fn split_stage_group_label(label: &str) -> (String, String) {
    if let Some((section, stage_name)) = label.rsplit_once(':') {
        (section.to_string(), stage_name.to_string())
    } else {
        ("ungrouped".to_string(), label.to_string())
    }
}

pub fn render_manifest_with_format(derived: &DerivedArtifacts, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => render_manifest(derived),
        OutputFormat::Json => {
            let manifest = &derived.manifest;
            let obligations = &derived.obligations;
            json!({
                "progress_manifest": manifest,
                "test_obligations": obligations
            })
            .to_string()
        }
    }
}

pub fn render_obligations(derived: &DerivedArtifacts, format: OutputFormat) -> String {
    let obligations = &derived.obligations;
    match format {
        OutputFormat::Text => render_obligations_text(obligations),
        OutputFormat::Json => json!({
            "dry_run_completion_required": obligations.dry_run_completion_required,
            "total_obligations": obligations.total_obligations,
            "transport_execution_targets": obligations.transport_execution_targets,
            "pure_node_determinism_targets": obligations.pure_node_determinism_targets,
            "service_transport_prepare_targets": obligations.service_transport_prepare_targets,
            "service_transport_execute_targets": obligations.service_transport_execute_targets,
            "service_transport_parse_targets": obligations.service_transport_parse_targets,
            "service_transport_hermetic_targets": obligations.service_transport_hermetic_targets,
            "service_transport_external_targets": obligations.service_transport_external_targets,
            "service_transport_idempotent_targets": obligations.service_transport_idempotent_targets,
            "service_transport_readonly_targets": obligations.service_transport_readonly_targets,
            "service_transport_permission_scoped_targets": obligations.service_transport_permission_scoped_targets,
            "service_param_source_targets": obligations.service_param_source_targets,
            "resource_provide_targets": obligations.resource_provide_targets,
            "resource_acquire_targets": obligations.resource_acquire_targets,
            "resource_release_targets": obligations.resource_release_targets,
            "interface_contract_verification_targets": obligations.interface_contract_verification_targets
        })
        .to_string(),
    }
}

fn render_obligations_text(obligations: &TestObligations) -> String {
    let mut out = String::new();
    out.push_str("TestObligations:\n");
    writeln!(
        out,
        "  dry_run_completion_required: {}",
        obligations.dry_run_completion_required
    )
    .ok();
    writeln!(
        out,
        "  total_obligations: {}",
        obligations.total_obligations
    )
    .ok();
    writeln!(
        out,
        "  transport_execution_targets: {}",
        obligations.transport_execution_targets
    )
    .ok();
    writeln!(
        out,
        "  pure_node_determinism_targets: {}",
        obligations.pure_node_determinism_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_prepare_targets: {}",
        obligations.service_transport_prepare_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_execute_targets: {}",
        obligations.service_transport_execute_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_parse_targets: {}",
        obligations.service_transport_parse_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_hermetic_targets: {}",
        obligations.service_transport_hermetic_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_external_targets: {}",
        obligations.service_transport_external_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_idempotent_targets: {}",
        obligations.service_transport_idempotent_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_readonly_targets: {}",
        obligations.service_transport_readonly_targets
    )
    .ok();
    writeln!(
        out,
        "  service_transport_permission_scoped_targets: {}",
        obligations.service_transport_permission_scoped_targets
    )
    .ok();
    writeln!(
        out,
        "  service_param_source_targets: {}",
        obligations.service_param_source_targets
    )
    .ok();
    writeln!(
        out,
        "  resource_provide_targets: {}",
        obligations.resource_provide_targets
    )
    .ok();
    writeln!(
        out,
        "  resource_acquire_targets: {}",
        obligations.resource_acquire_targets
    )
    .ok();
    writeln!(
        out,
        "  resource_release_targets: {}",
        obligations.resource_release_targets
    )
    .ok();
    writeln!(
        out,
        "  interface_contract_verification_targets: {}",
        obligations.interface_contract_verification_targets
    )
    .ok();
    out
}

/// Render only progress metrics (DL6).
pub fn render_progress_with_format(derived: &DerivedArtifacts, format: OutputFormat) -> String {
    let manifest = &derived.manifest;
    match format {
        OutputFormat::Text => {
            let mut out = String::new();
            out.push_str("Progress:\n");
            writeln!(out, "  total_nodes: {}", manifest.total_nodes).ok();
            writeln!(out, "  total_edges: {}", manifest.total_edges).ok();
            out.push_str("  waves:\n");
            for (index, wave) in manifest.waves.iter().enumerate() {
                writeln!(out, "    [{index}] {}", wave.join(", ")).ok();
            }
            writeln!(
                out,
                "  entrypoint_nodes: {}",
                if manifest.entrypoint_nodes.is_empty() {
                    "(none)".to_string()
                } else {
                    manifest.entrypoint_nodes.join(", ")
                }
            )
            .ok();
            writeln!(
                out,
                "  boundary_nodes: {}",
                if manifest.boundary_nodes.is_empty() {
                    "(none)".to_string()
                } else {
                    manifest.boundary_nodes.join(", ")
                }
            )
            .ok();
            out.push_str("  parallel_groups:\n");
            for group in &manifest.parallel_groups {
                writeln!(
                    out,
                    "    - depth={} nodes={}",
                    group.depth,
                    group.nodes.join(", ")
                )
                .ok();
            }
            out.push_str("  scatter_points:\n");
            render_scatter_points_text(&mut out, &manifest.scatter_points);
            writeln!(
                out,
                "  interactive_nodes: {}",
                if manifest.interactive_nodes.is_empty() {
                    "(none)".to_string()
                } else {
                    manifest.interactive_nodes.join(", ")
                }
            )
            .ok();
            out.push_str("  capture_modes:\n");
            for (node_id, mode) in &manifest.capture_modes {
                writeln!(out, "    - {node_id}: {mode:?}").ok();
            }
            out.push_str("  stage_groups:\n");
            render_stage_groups_text(&mut out, &manifest.stage_groups);
            out.push_str("  resources:\n");
            if manifest.resources.is_empty() {
                out.push_str("    (none)\n");
            } else {
                for (node_id, usages) in &manifest.resources {
                    let usage_text = usages
                        .iter()
                        .map(|u| format!("{}:{}", u.resource, u.usage))
                        .collect::<Vec<_>>()
                        .join(", ");
                    writeln!(out, "    - {node_id}: {usage_text}").ok();
                }
            }
            out.push_str(&render_obligations_text(&derived.obligations));
            out
        }
        OutputFormat::Json => json!({
            "total_nodes": manifest.total_nodes,
            "total_edges": manifest.total_edges,
            "waves": manifest.waves,
            "entrypoint_nodes": manifest.entrypoint_nodes,
            "boundary_nodes": manifest.boundary_nodes,
            "parallel_groups": manifest.parallel_groups,
            "scatter_points": manifest.scatter_points,
            "interactive_nodes": manifest.interactive_nodes,
            "capture_modes": manifest.capture_modes,
            "stage_groups": manifest.stage_groups,
            "resources": manifest.resources,
            "test_obligations": derived.obligations,
        })
        .to_string(),
    }
}

/// Render only graph topology (DL6).
pub fn render_topology_with_format(derived: &DerivedArtifacts, format: OutputFormat) -> String {
    let manifest = &derived.manifest;
    match format {
        OutputFormat::Text => {
            let mut out = String::new();
            out.push_str("Topology:\n");
            out.push_str("  nodes:\n");
            for node in &manifest.topology {
                writeln!(out, "    - {} (depth={})", node.id, node.depth).ok();
            }
            out.push_str("  labels:\n");
            for (node_id, label) in &manifest.labels {
                writeln!(out, "    - {node_id}: {label}").ok();
            }
            out.push_str("  subdag_boundaries:\n");
            if manifest.subdag_boundaries.is_empty() {
                out.push_str("    (none)\n");
            } else {
                for boundary in &manifest.subdag_boundaries {
                    writeln!(
                        out,
                        "    - {} label={} inner=[{}]",
                        boundary.node_id,
                        boundary.label,
                        boundary.inner_nodes.join(", ")
                    )
                    .ok();
                }
            }
            out
        }
        OutputFormat::Json => json!({
            "topology": manifest.topology,
            "labels": manifest.labels,
            "subdag_boundaries": manifest.subdag_boundaries,
        })
        .to_string(),
    }
}
