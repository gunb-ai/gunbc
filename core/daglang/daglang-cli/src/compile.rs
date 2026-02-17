use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write;
use std::path::PathBuf;

use crate::path_utils;
use crate::pipeline::PipelineContext;
use daglang_derive::{DerivedArtifacts, TestObligations};
use daglang_driver::DriverContext;
use daglang_lower::{LoweredOp, ServiceCallMetadata};
use gunbc_exec::{BoundaryMocks, ExecutionLog, ExecutionMode};
use gunbc_ir::{Dag, Node};
use serde_json::json;

pub use daglang_driver::{CheckOutput, CompileError, CompileOptions, CompileOutput};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

pub use daglang_exec_bridge::{
    makegen_check_mode_transport_mocks, makegen_dry_run_transport_mocks, makegen_entrypoint_mocks,
    resolve_lowered_dag, ResolveDagError, ResolvedOp,
};

/// Builds compile pipeline context from CLI input.
///
/// Compatibility note: paths ending in `.dag` are always treated as
/// single-file targets, even when they point to a directory.
/// Integration tests lock this behavior for lowercase/uppercase/mixed-case
/// extensions and trailing-slash variants.
pub fn build_context(cwd: &std::path::Path, input: Option<&String>) -> PipelineContext {
    let parsed = input.map(|value| path_utils::normalize_cli_path(cwd, &PathBuf::from(value)));
    let (roots, target_file) = match parsed {
        Some(path) if path_utils::is_single_file_target(&path, true) => {
            let root = path_utils::resolve_single_file_root(cwd, &path);
            (vec![root], Some(path))
        }
        Some(path) => (vec![path], None),
        None => (vec![path_utils::resolve_default_root(cwd)], None),
    };

    PipelineContext { roots, target_file }
}

pub fn compile_from_context(context: &PipelineContext) -> Result<CompileOutput, CompileError> {
    compile_from_context_with_options(
        context,
        CompileOptions {
            emit_collection_nodes: false,
        },
    )
}

pub fn compile_from_context_with_options(
    context: &PipelineContext,
    options: CompileOptions,
) -> Result<CompileOutput, CompileError> {
    daglang_driver::compile_from_context_with_options(
        &DriverContext {
            roots: context.roots.clone(),
            target_file: context.target_file.clone(),
        },
        options,
    )
}

pub fn check_from_context(context: &PipelineContext) -> Result<CheckOutput, CompileError> {
    daglang_driver::check_from_context(&DriverContext {
        roots: context.roots.clone(),
        target_file: context.target_file.clone(),
    })
}

pub fn execute_resolved_dag(
    dag: &Dag<ResolvedOp>,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, CompileError> {
    daglang_exec_bridge::execute_resolved_dag(dag, mode, input_mocks)
        .map_err(|error| CompileError::from(format!("execution error: {error}")))
}

pub fn compile_resolve_execute_from_context(
    context: &PipelineContext,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, CompileError> {
    let output = compile_from_context(context)?;
    let resolved = resolve_lowered_dag(&output.lowered_dag)
        .map_err(|error| CompileError::from(format!("resolve error: {error}")))?;
    execute_resolved_dag(&resolved, mode, input_mocks)
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
        let parent = node.parent.as_deref().unwrap_or("none");
        writeln!(
            out,
            "    - {} (depth={}, parent={parent})",
            node.id, node.depth
        )
        .ok();
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
            let parent = boundary.parent.as_deref().unwrap_or("none");
            let inner = if boundary.inner_nodes.is_empty() {
                "(none)".to_string()
            } else {
                boundary.inner_nodes.join(", ")
            };
            writeln!(
                out,
                "    - {} label={} parent={} inner_nodes={}",
                boundary.node_id, boundary.label, parent, inner
            )
            .ok();
        }
    }
    out.push_str("  parallel_groups:\n");
    for group in &manifest.parallel_groups {
        let parent = group.parent_subdag.as_deref().unwrap_or("none");
        writeln!(
            out,
            "    - depth={} parent_subdag={} nodes={}",
            group.depth,
            parent,
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

fn render_stage_groups_text(out: &mut String, stage_groups: &[daglang_derive::StageGroup]) {
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
struct TransportTriplet {
    prepare_node: String,
    execute_node: String,
    parse_nodes: Vec<String>,
    service_metadata: Option<ServiceCallMetadata>,
}

fn collect_transport_triplets(dag: &Dag<LoweredOp>) -> Vec<TransportTriplet> {
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

#[cfg(test)]
// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;
    use daglang_lower::{CallableKind, ObligationCategory, ServiceTransportClass};
    use gunbc_ir::{Edge, Node, Port};
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let fixture_root = std::env::temp_dir().join(format!(
            "daglang_cli_compile_{name}_{}_{}",
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&fixture_root).expect("failed to create temp fixture root");
        fixture_root.join(format!("{name}.dag"))
    }

    fn unique_temp_output_file(name: &str, extension: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "daglang_cli_compile_{name}_{}_{}.{}",
            std::process::id(),
            nanos,
            extension
        ))
    }

    fn workspace_dsl_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl")
    }

    fn workspace_single_file_context(relative_path: &str) -> PipelineContext {
        let root = workspace_dsl_root();
        PipelineContext {
            roots: vec![root.clone()],
            target_file: Some(root.join(relative_path)),
        }
    }

    fn assert_typecheck_stage_error(error: &CompileError) {
        assert!(error.contains("typecheck errors"));
        assert!(!error.contains("lower error"));
    }

    #[test]
    fn build_context_normalizes_absolute_directory_input_components() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let normalized_root = root.join("sample");
        std::fs::create_dir_all(&normalized_root).expect("failed to create temp directory root");
        let input = root.join("sample").join(".").join("nested").join("..");
        let input_str = input.to_string_lossy().to_string();

        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_str));
        assert_eq!(context.roots, vec![normalized_root.clone()]);
        assert!(context.target_file.is_none());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn build_context_normalizes_absolute_single_file_input_components() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_file_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let normalized_file = root.join("sample/main.dag");
        std::fs::create_dir_all(normalized_file.parent().expect("file should have parent"))
            .expect("failed to create temp file parent");
        std::fs::write(&normalized_file, "module sample.main\nfn run() -> Unit { }")
            .expect("failed to write temp dag file");
        let input = root
            .join("sample")
            .join("nested")
            .join("..")
            .join("main.dag");
        let input_str = input.to_string_lossy().to_string();

        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_str));
        assert_eq!(
            context.roots,
            vec![normalized_file
                .parent()
                .expect("file should have parent")
                .to_path_buf()]
        );
        assert_eq!(context.target_file, Some(normalized_file.clone()));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn build_context_default_root_is_cwd_dsl() {
        let cwd = std::env::temp_dir().join(format!(
            "daglang_build_context_default_root_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let context = build_context(&cwd, None);
        assert_eq!(context.roots, vec![cwd.join("dsl")]);
        assert!(context.target_file.is_none());
    }

    #[test]
    fn check_from_context_succeeds_for_valid_single_file() {
        let fixture = unique_temp_file("check_valid_single_file");
        std::fs::write(
            &fixture,
            r#"module sample.check_valid
fn run() -> Unit { }
"#,
        )
        .expect("failed to write check valid fixture");
        let cwd = std::env::temp_dir();
        let input = fixture.to_string_lossy().to_string();
        let context = build_context(&cwd, Some(&input));

        let output = check_from_context(&context).expect("check should succeed");
        assert_eq!(
            output.parsed_files, 1,
            "single-file check should report exactly one parsed file"
        );

        std::fs::remove_file(fixture).expect("failed to cleanup check valid fixture");
    }

    #[test]
    fn check_from_context_reports_typecheck_error_for_invalid_single_file() {
        let fixture = unique_temp_file("check_type_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.check_invalid
fn run() -> String { return 42 }
"#,
        )
        .expect("failed to write check invalid fixture");
        let cwd = std::env::temp_dir();
        let input = fixture.to_string_lossy().to_string();
        let context = build_context(&cwd, Some(&input));

        let error = check_from_context(&context).expect_err("check should fail");
        assert_typecheck_stage_error(&error);
        assert!(
            error.contains("type mismatch: expected `String`, got `Int`"),
            "check should surface type mismatch details: {error}"
        );

        std::fs::remove_file(fixture).expect("failed to cleanup check invalid fixture");
    }

    #[test]
    fn build_context_treats_dag_directory_input_as_single_file_target() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_dag_dir_target_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let dag_dir = root.join("bundle.dag");
        std::fs::create_dir_all(dag_dir.join("nested"))
            .expect("failed to create .dag directory fixture");
        std::fs::write(
            dag_dir.join("nested/main.dag"),
            "module sample.main\nfn run() -> Unit {}",
        )
        .expect("failed to write nested dag fixture");

        let input_str = dag_dir.to_string_lossy().to_string();
        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_str));

        assert_eq!(context.roots, vec![root.clone()]);
        assert_eq!(context.target_file, Some(dag_dir.clone()));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn build_context_normalizes_trailing_slash_for_dag_directory_target() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_dag_dir_trailing_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let dag_dir = root.join("bundle.dag");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .dag directory fixture");
        let input_with_trailing_slash = format!("{}/", dag_dir.display());

        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_with_trailing_slash));

        assert_eq!(context.roots, vec![root.clone()]);
        assert_eq!(context.target_file, Some(dag_dir.clone()));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn build_context_normalizes_trailing_slash_for_uppercase_dag_directory_target() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_uppercase_dag_dir_trailing_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let dag_dir = root.join("bundle.DAG");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory fixture");
        let input_with_trailing_slash = format!("{}/", dag_dir.display());

        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_with_trailing_slash));

        assert_eq!(context.roots, vec![root.clone()]);
        assert_eq!(context.target_file, Some(dag_dir.clone()));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn build_context_normalizes_trailing_slash_for_mixed_case_dag_directory_target() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_mixed_case_dag_dir_trailing_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let dag_dir = root.join("bundle.DaG");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory fixture");
        let input_with_trailing_slash = format!("{}/", dag_dir.display());

        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_with_trailing_slash));

        assert_eq!(context.roots, vec![root.clone()]);
        assert_eq!(context.target_file, Some(dag_dir.clone()));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn build_context_treats_uppercase_dag_directory_input_as_single_file_target() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_uppercase_dag_dir_target_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let dag_dir = root.join("bundle.DAG");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory fixture");

        let input_str = dag_dir.to_string_lossy().to_string();
        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_str));

        assert_eq!(context.roots, vec![root.clone()]);
        assert_eq!(context.target_file, Some(dag_dir.clone()));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn build_context_treats_mixed_case_dag_directory_input_as_single_file_target() {
        let root = std::env::temp_dir().join(format!(
            "daglang_build_context_mixed_case_dag_dir_target_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let dag_dir = root.join("bundle.DaG");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory fixture");

        let input_str = dag_dir.to_string_lossy().to_string();
        let cwd = std::env::temp_dir();
        let context = build_context(&cwd, Some(&input_str));

        assert_eq!(context.roots, vec![root.clone()]);
        assert_eq!(context.target_file, Some(dag_dir.clone()));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_reports_cyclic_dependency_errors() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_cycle_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("a")).expect("failed to create module a dir");
        std::fs::create_dir_all(root.join("b")).expect("failed to create module b dir");
        std::fs::write(
            root.join("a/a.dag"),
            "module cycle.a\nimport cycle.b\nfn a() -> Unit {}",
        )
        .expect("failed to write module a");
        std::fs::write(
            root.join("b/b.dag"),
            "module cycle.b\nimport cycle.a\nfn b() -> Unit {}",
        )
        .expect("failed to write module b");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let err = compile_from_context(&context).expect_err("compile should fail on cycles");
        assert!(err.contains("cyclic dependency"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_makegen_produces_non_empty_outputs() {
        let context = workspace_single_file_context("tools/makegen.dag");

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());
        let rendered = render_manifest(&output.derived);
        assert!(rendered.contains("TestObligations:"));
        assert!(rendered.contains("service_transport_prepare_targets:"));
        assert!(rendered.contains("service_param_source_targets:"));
        assert!(rendered.contains("resource_provide_targets:"));
    }

    #[test]
    fn resolve_lowered_dag_maps_makegen_nodes_to_resolved_ops() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output = compile_from_context(&context).expect("compile should succeed");

        let resolved =
            resolve_lowered_dag(&output.lowered_dag).expect("makegen dag should resolve");
        assert_eq!(resolved.nodes.len(), output.lowered_dag.nodes.len());
        assert_eq!(resolved.edges.len(), output.lowered_dag.edges.len());

        let resolved_op_for = |node_id: &str| {
            resolved
                .nodes
                .iter()
                .find(|node| node.id.0 == node_id)
                .map(|node| match &node.body {
                    gunbc_ir::node::NodeBody::Opaque(op) => op,
                    gunbc_ir::node::NodeBody::SubDag(_) => {
                        panic!("makegen fixture should not contain subdag nodes")
                    }
                })
                .expect("expected node to exist in resolved dag")
        };

        assert!(matches!(
            resolved_op_for("load_registry"),
            ResolvedOp::LoadRegistry
        ));
        assert!(matches!(resolved_op_for("fs_env"), ResolvedOp::FsEnv));
        assert!(matches!(
            resolved_op_for("tools.makegen::render_makefile"),
            ResolvedOp::RenderMakefile
        ));
        assert!(matches!(
            resolved_op_for("prepare_read_makegen"),
            ResolvedOp::PrepareReadContent
        ));
        assert!(matches!(
            resolved_op_for("execute_read_makegen"),
            ResolvedOp::ExecuteReadContent
        ));
        assert!(matches!(
            resolved_op_for("prepare_write_makegen"),
            ResolvedOp::PrepareWriteContent
        ));
        assert!(matches!(
            resolved_op_for("compare_makegen_content"),
            ResolvedOp::CompareContent
        ));
        assert!(matches!(
            resolved_op_for("execute_makegen_transport"),
            ResolvedOp::ExecuteTransport
        ));
        assert!(matches!(
            resolved_op_for("tools.makegen::makegen"),
            ResolvedOp::MakegenEntrypoint
        ));
    }

    #[test]
    fn resolve_lowered_dag_rejects_unknown_callable_module() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "sample::unknown",
            vec![],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "sample.module".to_string(),
                kind: CallableKind::Func,
                name: "unknown".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));

        let error = resolve_lowered_dag(&dag).expect_err("resolver should reject unknown module");
        assert_eq!(error.node_id, "sample::unknown");
        assert!(error.reason.contains("unsupported callable module"));
    }

    #[test]
    fn resolve_lowered_dag_rejects_pipeline_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "pipeline::ci",
            vec![],
            vec![Port::scalar("out", "String")],
            LoweredOp::Pipeline {
                module: "pipelines".to_string(),
                name: "ci".to_string(),
                stages: 3,
                stage_names: vec![
                    "cloud_env".to_string(),
                    "codegen_stage".to_string(),
                    "generate".to_string(),
                ],
            },
        ));

        let error = resolve_lowered_dag(&dag).expect_err("resolver should reject pipeline nodes");
        assert_eq!(error.node_id, "pipeline::ci");
        assert!(error.reason.contains("unsupported pipeline"));
    }

    #[test]
    fn compile_resolve_execute_makegen_real_mode_writes_output() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output_path = unique_temp_output_file("makegen_real_run", "mk");
        let output_path_str = output_path.to_string_lossy().to_string();
        let input_mocks = makegen_entrypoint_mocks(&output_path_str);

        let log =
            compile_resolve_execute_from_context(&context, ExecutionMode::Real, Some(&input_mocks))
                .expect("real execution should succeed");
        assert!(
            output_path.exists(),
            "real execution should write requested output path"
        );
        let content = std::fs::read_to_string(&output_path)
            .expect("real execution should emit readable output file");
        assert!(content.contains(".PHONY"));
        assert!(content.contains("makegen"));
        let makegen_entry = log
            .get("tools.makegen::makegen")
            .expect("execution log should include makegen entrypoint node");
        assert_eq!(
            makegen_entry.outputs.get("written"),
            Some(&gunbc_ir::Value::Bool(true)),
            "first real run should report written=true"
        );

        std::fs::remove_file(output_path).expect("failed to cleanup makegen output");
    }

    #[test]
    fn compile_resolve_execute_makegen_real_mode_reports_not_written_when_fresh() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output_path = unique_temp_output_file("makegen_real_idempotent", "mk");
        let output_path_str = output_path.to_string_lossy().to_string();
        let input_mocks = makegen_entrypoint_mocks(&output_path_str);

        compile_resolve_execute_from_context(&context, ExecutionMode::Real, Some(&input_mocks))
            .expect("first real execution should succeed");
        let first_content =
            std::fs::read_to_string(&output_path).expect("first run should write output");
        let second =
            compile_resolve_execute_from_context(&context, ExecutionMode::Real, Some(&input_mocks))
                .expect("second real execution should succeed");

        let second_entry = second
            .get("tools.makegen::makegen")
            .expect("execution log should include makegen entrypoint node");
        assert_eq!(
            second_entry.outputs.get("written"),
            Some(&gunbc_ir::Value::Bool(false)),
            "second real run should report written=false when output is unchanged"
        );
        let second_content =
            std::fs::read_to_string(&output_path).expect("second run should leave output intact");
        assert_eq!(first_content, second_content);
        assert!(
            output_path.exists(),
            "real idempotence check should preserve generated output"
        );
        std::fs::remove_file(output_path).expect("failed to cleanup makegen output");
    }

    #[test]
    fn compile_resolve_execute_makegen_dry_run_intercepts_and_skips_output_write() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output_path = unique_temp_output_file("makegen_dry_run", "mk");
        let output_path_str = output_path.to_string_lossy().to_string();
        let input_mocks = makegen_entrypoint_mocks(&output_path_str);
        let dry_run_mocks = makegen_dry_run_transport_mocks(&output_path_str);

        let log = compile_resolve_execute_from_context(
            &context,
            ExecutionMode::DryRun(dry_run_mocks),
            Some(&input_mocks),
        )
        .expect("dry-run execution should succeed");
        assert!(
            !output_path.exists(),
            "dry-run execution should not write output file"
        );
        assert!(
            log.has_intercepted(),
            "dry-run should intercept boundary nodes"
        );
        let makegen_entry = log
            .get("tools.makegen::makegen")
            .expect("execution log should include makegen entrypoint node");
        assert_eq!(
            makegen_entry.outputs.get("written"),
            Some(&gunbc_ir::Value::Bool(false)),
            "dry-run should report written=false"
        );
    }

    #[test]
    fn render_obligations_json_emits_expected_keys() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output = compile_from_context(&context).expect("compile should succeed");

        let rendered = render_obligations(&output.derived, OutputFormat::Json);
        let parsed: Value = serde_json::from_str(&rendered).expect("obligations json should parse");
        assert_eq!(
            parsed
                .get("dry_run_completion_required")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert!(parsed.get("total_obligations").is_some());
        assert!(parsed.get("transport_execution_targets").is_some());
        assert!(parsed.get("pure_node_determinism_targets").is_some());
        assert!(parsed.get("service_transport_hermetic_targets").is_some());
        assert!(parsed.get("service_transport_external_targets").is_some());
        assert!(parsed.get("service_transport_idempotent_targets").is_some());
        assert!(parsed.get("service_transport_readonly_targets").is_some());
        assert!(parsed
            .get("service_transport_permission_scoped_targets")
            .is_some());
        assert!(parsed
            .get("interface_contract_verification_targets")
            .is_some());
    }

    #[test]
    fn render_triplets_json_includes_makegen_transport_nodes() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output = compile_from_context(&context).expect("compile should succeed");

        let rendered = render_triplets(&output.lowered_dag, OutputFormat::Json);
        let parsed: Value = serde_json::from_str(&rendered).expect("triplets json should parse");
        let triplets = parsed
            .get("triplets")
            .and_then(Value::as_array)
            .expect("triplets should be an array");
        assert!(
            triplets.iter().any(|triplet| {
                triplet
                    .get("prepare_node")
                    .and_then(Value::as_str)
                    .is_some_and(|prepare| prepare == "prepare_read_makegen")
            }),
            "expected read transport triplet"
        );
        assert!(
            triplets.iter().any(|triplet| {
                triplet
                    .get("prepare_node")
                    .and_then(Value::as_str)
                    .is_some_and(|prepare| prepare == "prepare_write_makegen")
            }),
            "expected write transport triplet"
        );
    }

    #[test]
    fn render_triplets_json_includes_service_semantic_metadata_when_present() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "prepare_transport_service",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::prepare::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: Some(ServiceCallMetadata {
                    service: "FsStorage".to_string(),
                    operation: "read".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: true,
                    readonly: true,
                    permissions: vec![],
                }),
            },
        ));
        dag.add_node(Node::opaque(
            "execute_transport_service",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::execute::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: Some(ServiceCallMetadata {
                    service: "FsStorage".to_string(),
                    operation: "read".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: true,
                    readonly: true,
                    permissions: vec![],
                }),
            },
        ));
        dag.add_node(Node::opaque(
            "parse_transport_service",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("body", "String")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::parse::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: Some(ServiceCallMetadata {
                    service: "FsStorage".to_string(),
                    operation: "read".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: true,
                    readonly: true,
                    permissions: vec![],
                }),
            },
        ));
        dag.add_edge(Edge::new(
            "prepare_transport_service",
            "request",
            "execute_transport_service",
            "request",
        ));
        dag.add_edge(Edge::new(
            "execute_transport_service",
            "response",
            "parse_transport_service",
            "response",
        ));

        let rendered = render_triplets(&dag, OutputFormat::Json);
        let parsed: Value = serde_json::from_str(&rendered).expect("triplets json should parse");
        let triplets = parsed
            .get("triplets")
            .and_then(Value::as_array)
            .expect("triplets should be an array");
        let metadata = triplets
            .first()
            .and_then(|triplet| triplet.get("service_metadata"))
            .expect("triplet should include service metadata");
        assert_eq!(
            metadata.get("transport").and_then(Value::as_str),
            Some("shell_local")
        );
        assert_eq!(
            metadata.get("idempotent").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            metadata.get("readonly").and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn render_triplets_text_is_deterministic() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output = compile_from_context(&context).expect("compile should succeed");

        let first = render_triplets(&output.lowered_dag, OutputFormat::Text);
        let second = render_triplets(&output.lowered_dag, OutputFormat::Text);
        assert_eq!(first, second, "triplet rendering should be deterministic");
    }

    #[test]
    fn render_manifest_reuses_obligations_text_block() {
        let context = workspace_single_file_context("tools/makegen.dag");
        let output = compile_from_context(&context).expect("compile should succeed");

        let manifest = render_manifest(&output.derived);
        let obligations = render_obligations(&output.derived, OutputFormat::Text);
        assert!(
            manifest.ends_with(&obligations),
            "manifest output should embed the same obligations text renderer"
        );
    }

    #[test]
    fn render_manifest_groups_stage_groups_into_collapsible_sections() {
        let context = workspace_single_file_context("pipelines/ci.dag");
        let output = compile_from_context(&context).expect("compile should succeed");

        let manifest = render_manifest(&output.derived);
        assert!(
            manifest.contains("  stage_groups:\n    > [collapsed] pipelines.ci.ci"),
            "manifest text should render ci stage groups as collapsible section"
        );
        assert!(
            manifest.contains("      - cloud_env:"),
            "manifest text should render cloud_env stage inside section"
        );
        assert!(
            manifest.contains("      - bootstrap_stage:"),
            "manifest text should render bootstrap_stage inside section"
        );
    }

    #[test]
    fn render_manifest_groups_scatter_points_as_counters() {
        let root = std::env::temp_dir().join(format!(
            "daglang_manifest_scatter_points_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        let file = root.join("sample.dag");
        std::fs::write(
            &file,
            r#"module sample
fn run(values: List<String>) -> String {
  rendered = values |> map(v => v) |> join(",")
  return rendered
}
"#,
        )
        .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                emit_collection_nodes: true,
            },
        )
        .expect("compile should succeed with collection nodes");

        let manifest = render_manifest(&output.derived);
        assert!(
            manifest.contains("  scatter_points:\n    - sample.run [0/2]"),
            "manifest text should render grouped scatter counter for collection pipeline: {manifest}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn collect_transport_triplets_sorts_parse_nodes_and_ignores_non_transport_edges() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "prepare_a",
            vec![],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "prepare".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "execute_a",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "execute".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "parse_z",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("body", "String")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "parse_z".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "parse_a",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("body", "String")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "parse_a".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "non_transport_sink",
            vec![Port::scalar("value", "String")],
            vec![Port::scalar("ok", "Bool")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "sink".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));

        dag.add_edge(Edge::new("prepare_a", "request", "execute_a", "request"));
        dag.add_edge(Edge::new("execute_a", "response", "parse_z", "response"));
        dag.add_edge(Edge::new("execute_a", "response", "parse_a", "response"));
        dag.add_edge(Edge::new("parse_a", "body", "non_transport_sink", "value"));

        let triplets = collect_transport_triplets(&dag);
        assert_eq!(triplets.len(), 1, "expected exactly one transport triplet");
        let triplet = &triplets[0];
        assert_eq!(triplet.prepare_node, "prepare_a");
        assert_eq!(triplet.execute_node, "execute_a");
        assert_eq!(
            triplet.parse_nodes,
            vec!["parse_a".to_string(), "parse_z".to_string()],
            "parse nodes should be sorted and deterministic"
        );
    }

    #[test]
    fn compile_reports_pipeline_diagnostics_for_invalid_source() {
        let broken_file = unique_temp_file("broken");
        std::fs::write(&broken_file, "module broken\nfn bad( -> Unit {}")
            .expect("failed to write broken source");

        let context = PipelineContext {
            roots: vec![broken_file
                .parent()
                .expect("temp file should have parent")
                .to_path_buf()],
            target_file: Some(broken_file.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert!(error.contains("compile diagnostics"));
        assert!(error.contains(":2:"));
        assert!(!error.contains("typecheck errors"));
        assert!(!error.contains("lower error"));

        std::fs::remove_file(broken_file).expect("failed to cleanup broken source");
    }

    #[test]
    fn compile_directory_reports_module_path_mismatch() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_mismatch_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        std::fs::write(
            root.join("main.dag"),
            "module mismatch.main\nfn run() -> Unit {}",
        )
        .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let error = compile_from_context(&context).expect_err("compile should fail");
        assert!(error.contains("module path mismatches"));
        assert!(error.contains("main"));
        assert!(!error.contains("typecheck errors"));
        assert!(!error.contains("lower error"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_sorts_lex_before_parse_diagnostics() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_lex_before_parse_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
            .expect("failed to write parse-error file");
        std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
            .expect("failed to write lex-error file");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        let first_diagnostic_line = error
            .as_str()
            .lines()
            .find(|line| line.contains(".dag:"))
            .expect("expected at least one file diagnostic line");
        assert!(
            first_diagnostic_line.contains("z_lex.dag"),
            "lex diagnostics should sort before parse diagnostics: {error}"
        );
        assert!(error.contains("a_parse.dag"));
        assert!(error.contains("unexpected character '$'"));
        assert!(!error.contains("typecheck errors"));
        assert!(!error.contains("lower error"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_unresolved_service_call_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unresolved_service_call");
        std::fs::write(
            &fixture,
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
        )
        .expect("failed to write unresolved service-call fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unresolved service call"));
        assert!(error.contains("MissingStorage.read"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_unresolved_service_call_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unresolved_service_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
        )
        .expect("failed to write unresolved service-call source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unresolved service call"));
        assert!(error.contains("MissingStorage.read"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_uses_bound_resource_capability_call_succeeds() {
        let fixture = unique_temp_file("single_file_uses_bound_resource_capability_call");
        std::fs::write(
            &fixture,
            r#"module sample.resources
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}
"#,
        )
        .expect("failed to write resource-bound capability fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_uses_bound_resource_capability_call_succeeds() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_resource_bound_service_call_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}
"#,
        )
        .expect("failed to write resource-bound capability source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_unresolved_uses_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unresolved_uses");
        std::fs::write(
            &fixture,
            r#"module sample.uses
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}
"#,
        )
        .expect("failed to write unresolved uses fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown used resource type"));
        assert!(error.contains("MissingResource"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_unresolved_uses_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unresolved_uses_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}
"#,
        )
        .expect("failed to write unresolved uses source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown used resource type"));
        assert!(error.contains("MissingResource"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_uses_resource_with_runtime_config_suffix_succeeds() {
        let fixture = unique_temp_file("single_file_uses_resource_with_config_suffix");
        std::fs::write(
            &fixture,
            r#"module sample.uses
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}
"#,
        )
        .expect("failed to write configured uses fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_uses_resource_with_runtime_config_suffix_succeeds() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_uses_config_suffix_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}
"#,
        )
        .expect("failed to write configured uses source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_unresolved_provides_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unresolved_provides");
        std::fs::write(
            &fixture,
            r#"module sample.provides
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}
"#,
        )
        .expect("failed to write unresolved provides fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown provided resource type"));
        assert!(error.contains("MissingResource"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_unresolved_provides_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unresolved_provides_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}
"#,
        )
        .expect("failed to write unresolved provides source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown provided resource type"));
        assert!(error.contains("MissingResource"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_provides_resource_with_runtime_config_suffix_succeeds() {
        let fixture = unique_temp_file("single_file_provides_resource_with_config_suffix");
        std::fs::write(
            &fixture,
            r#"module sample.provides
resource ArtifactStore {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}
"#,
        )
        .expect("failed to write configured provides fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_provides_resource_with_runtime_config_suffix_succeeds() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_provides_config_suffix_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
resource ArtifactStore {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}
"#,
        )
        .expect("failed to write configured provides source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_unresolved_import_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unresolved_import");
        std::fs::write(
            &fixture,
            r#"module sample.single
import missing.dep
fn run() -> Unit {}
"#,
        )
        .expect("failed to write unresolved import fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unresolved import"));
        assert!(error.contains("missing.dep"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_unresolved_import_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unresolved_import_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
import missing.dep
fn run() -> Unit {}
"#,
        )
        .expect("failed to write unresolved import source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unresolved import"));
        assert!(error.contains("missing.dep"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_unresolved_call_target_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unresolved_call_target");
        std::fs::write(
            &fixture,
            r#"module sample.single
fn run() -> Unit {
  missing()
}
"#,
        )
        .expect("failed to write unresolved callable fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unresolved call target"));
        assert!(error.contains("missing"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_unresolved_call_target_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unresolved_call_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run() -> Unit {
  missing()
}
"#,
        )
        .expect("failed to write unresolved callable source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unresolved call target"));
        assert!(error.contains("missing"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_collection_intrinsics_typecheck_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_collection_intrinsics_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type Stage {
  success: Bool,
  skipped: Bool,
  name: String
}
fn summarize(stages: List<Stage>) -> Int {
  let passed = stages |> filter(s => s.success) |> count()
  let labels = stages |> map(s => s.name) |> join(",")
  let done = labels |> ends_with("ok")
  passed
}
"#,
        )
        .expect("failed to write collection intrinsic source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_collection_option_emits_collection_nodes() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_collection_option_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run(values: List<String>) -> String {
  rendered = values |> map(v => v) |> join(",")
  return rendered
}
"#,
        )
        .expect("failed to write collection option source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                emit_collection_nodes: true,
            },
        )
        .expect("compile should succeed with collection option");
        assert!(output.lowered_dag.nodes.iter().any(|node| {
            matches!(
                node.body,
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection {
                    kind: daglang_lower::CollectionOpKind::Map,
                    ..
                })
            )
        }));
        assert!(output.lowered_dag.nodes.iter().any(|node| {
            matches!(
                node.body,
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection {
                    kind: daglang_lower::CollectionOpKind::Join,
                    ..
                })
            )
        }));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_function_typed_parameter_calls_typecheck_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_fn_typed_param_calls_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn apply(value: Int, callback: fn(Int) -> Int) -> Int {
  callback(value)
}
"#,
        )
        .expect("failed to write callback source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_sum_variant_constructor_calls_typecheck_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_sum_variant_constructor_calls_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type CloudConfig
  = GcpConfig { project: String, region: String }
  | AwsConfig { region: String }

fn make_gcp() -> CloudConfig {
  GcpConfig(project: "gunbc", region: "us-central1")
}
"#,
        )
        .expect("failed to write constructor source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_zero_arity_variant_identifier_returns_typecheck_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_zero_arity_variant_identifier_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type Environment = Dev | Ci
fn env() -> Environment {
  Dev
}
"#,
        )
        .expect("failed to write zero-arity variant source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_lossy_match_fn_body_does_not_fail_missing_tail_mismatch() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_lossy_match_body_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type CloudConfig
  = GcpConfig { project: String }
  | AwsConfig { account: String }
type CloudProvider = Gcp | Aws

fn provider_of(config: CloudConfig) -> CloudProvider {
  match config {
    GcpConfig { ... } => Gcp
    AwsConfig { ... } => Aws
  }
}
"#,
        )
        .expect("failed to write lossy match source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_std_helper_intrinsics_typecheck_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_std_helper_intrinsics_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type DocgenSources {}

fn run(sources: DocgenSources, payload: String) -> String {
  let a = "template" |> replace_section("section", "value")
  let b = render_test_listings(sources: sources)
  let c = render_graph_structure(sources: sources)
  let d = render_source_artifacts(sources: sources)
  let e = compute_topology_diff(current: "{}", base: "{}")
  let f = render_annotated_mermaid(diff: e, topology: "{}", title: "title")
  let g = detect_runtime()
  let h = generate()
  let i = now()
  let j = build_token(
    payload: payload,
    scheme: "Bearer",
    header_name: "Authorization",
    source_id: "source",
    required_scopes: ["gist"]
  )
  a
}
"#,
        )
        .expect("failed to write helper intrinsic source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_duplicate_service_reports_ambiguous_service_call() {
        let fixture = unique_temp_file("single_file_duplicate_service");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
        )
        .expect("failed to write duplicate service fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `FsStorage`"));
        assert!(error.contains("ambiguous service call"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_duplicate_service_reports_ambiguous_service_call() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_service_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
        )
        .expect("failed to write duplicate service source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `FsStorage`"));
        assert!(error.contains("ambiguous service call"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_duplicate_callable_reports_ambiguous_call_target() {
        let fixture = unique_temp_file("single_file_duplicate_callable");
        std::fs::write(
            &fixture,
            r#"module sample.single
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
"#,
        )
        .expect("failed to write duplicate callable fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `helper`"));
        assert!(error.contains("ambiguous call target"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_duplicate_callable_reports_ambiguous_call_target() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_callable_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
"#,
        )
        .expect("failed to write duplicate callable source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `helper`"));
        assert!(error.contains("ambiguous call target"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_duplicate_resource_uses_reports_ambiguous_used_type() {
        let fixture = unique_temp_file("single_file_duplicate_resource_uses");
        std::fs::write(
            &fixture,
            r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate resource-uses fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `SharedResource`"));
        assert!(error.contains("ambiguous used resource type"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_duplicate_resource_uses_reports_ambiguous_used_type() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_resource_uses_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate resource-uses source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `SharedResource`"));
        assert!(error.contains("ambiguous used resource type"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_duplicate_resource_provides_reports_ambiguous_provided_type() {
        let fixture = unique_temp_file("single_file_duplicate_resource_provides");
        std::fs::write(
            &fixture,
            r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate resource-provides fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `SharedResource`"));
        assert!(error.contains("ambiguous provided resource type"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_duplicate_resource_provides_reports_ambiguous_provided_type() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_resource_provides_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate resource-provides source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `SharedResource`"));
        assert!(error.contains("ambiguous provided resource type"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_unresolved_service_interface_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unresolved_service_interface");
        std::fs::write(
            &fixture,
            r#"module sample.services
service FsStorage implements MissingStorage {
  operation read(path: String) -> { body: String }
}
"#,
        )
        .expect("failed to write unresolved service-interface fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`FsStorage` references unresolved interface `MissingStorage`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_unresolved_resource_interface_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unresolved_resource_interface");
        std::fs::write(
            &fixture,
            r#"module sample.resources
resource Disk implements MissingStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
        )
        .expect("failed to write unresolved resource-interface fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`Disk` references unresolved interface `MissingStorage`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_unresolved_service_interface_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unresolved_service_interface_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
service FsStorage implements MissingStorage {
  operation read(path: String) -> { body: String }
}
"#,
        )
        .expect("failed to write unresolved service-interface source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`FsStorage` references unresolved interface `MissingStorage`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_unresolved_resource_interface_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unresolved_resource_interface_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
resource Disk implements MissingStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
        )
        .expect("failed to write unresolved resource-interface source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`Disk` references unresolved interface `MissingStorage`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_duplicate_interface_reports_ambiguous_implements() {
        let fixture = unique_temp_file("single_file_duplicate_interface");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
        )
        .expect("failed to write duplicate-interface fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `Storage` in module `sample.single`"));
        assert!(error.contains("`FsStorage` references ambiguous interface `Storage`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_duplicate_interface_reports_ambiguous_implements() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_interface_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
        )
        .expect("failed to write duplicate-interface source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `Storage` in module `sample.main`"));
        assert!(error.contains("`FsStorage` references ambiguous interface `Storage`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_unit_return_without_tail_expression_succeeds() {
        let fixture = unique_temp_file("single_file_unit_without_tail");
        std::fs::write(
            &fixture,
            r#"module sample.single
fn run() -> Unit {
  let x = 42
}
"#,
        )
        .expect("failed to write Unit-return fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_unit_return_without_tail_expression_succeeds() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unit_without_tail_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run() -> Unit {
  let x = 42
}
"#,
        )
        .expect("failed to write Unit-return source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_missing_tail_non_unit_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_non_unit_without_tail");
        std::fs::write(
            &fixture,
            r#"module sample.single
fn run() -> String {
  let x = 42
}
"#,
        )
        .expect("failed to write non-Unit return fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type mismatch: expected `String`, got `Unit`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_missing_tail_non_unit_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_non_unit_without_tail_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run() -> String {
  let x = 42
}
"#,
        )
        .expect("failed to write non-Unit return source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type mismatch: expected `String`, got `Unit`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_call_arity_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_call_arity_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
"#,
        )
        .expect("failed to write call-arity fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("call arity mismatch"));
        assert!(error.contains("fmt"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_unknown_named_call_argument_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unknown_named_call_argument");
        std::fs::write(
            &fixture,
            r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
"#,
        )
        .expect("failed to write unknown-arg fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown named argument"));
        assert!(error.contains("text"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_duplicate_named_call_argument_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_duplicate_named_call_argument");
        std::fs::write(
            &fixture,
            r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(value: "a", value: "b") }
"#,
        )
        .expect("failed to write duplicate-arg fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate named argument"));
        assert!(error.contains("value"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_service_call_arity_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_service_call_arity_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read()
  return { body: response.body }
}
"#,
        )
        .expect("failed to write service call-arity fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("service call arity mismatch"));
        assert!(error.contains("FsStorage.read"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_unknown_named_service_argument_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unknown_named_service_argument");
        std::fs::write(
            &fixture,
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(name: "README.md")
  return { body: response.body }
}
"#,
        )
        .expect("failed to write unknown service-arg fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown named argument"));
        assert!(error.contains("name"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_duplicate_named_service_argument_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_duplicate_named_service_argument");
        std::fs::write(
            &fixture,
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(path: "a", path: "b")
  return { body: response.body }
}
"#,
        )
        .expect("failed to write duplicate service-arg fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate named argument"));
        assert!(error.contains("path"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_undefined_type_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_undefined_type");
        std::fs::write(
            &fixture,
            r#"module sample.types
fn run(input: MissingType) -> String { "ok" }
"#,
        )
        .expect("failed to write undefined-type fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("undefined type `MissingType"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_type_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_type_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.types
fn run() -> String { return 42 }
"#,
        )
        .expect("failed to write type-mismatch fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type mismatch: expected `String`, got `Int`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_implicit_return_type_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_implicit_return_type_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.types
fn run() -> String { 42 }
"#,
        )
        .expect("failed to write implicit-return mismatch fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type mismatch: expected `String`, got `Int`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_no_such_field_record_literal_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_no_such_field_record_literal");
        std::fs::write(
            &fixture,
            r#"module sample.types
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}
"#,
        )
        .expect("failed to write no-such-field record-literal fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type `Record` has no field `missing`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_no_such_field_named_record_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_no_such_field_named_record");
        std::fs::write(
            &fixture,
            r#"module sample.types
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
"#,
        )
        .expect("failed to write no-such-field named-record fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type `Payload` has no field `missing`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_unsatisfiable_refinement_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_unsatisfiable_refinement");
        std::fs::write(
            &fixture,
            r#"module sample.types
fn run(value: Int @range(min: 5, max: 1)) -> Int { value }
"#,
        )
        .expect("failed to write unsatisfiable-refinement fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unsatisfiable refinement on `Int`: range min 5 exceeds max 1"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_generic_arity_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_generic_arity_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.types
fn run(values: Map<String>) -> Int { 1 }
"#,
        )
        .expect("failed to write generic-arity mismatch fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("generic arity mismatch for `Map`: expected 2, got 1"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_user_defined_generic_arity_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_user_defined_generic_arity_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.types
type Box<T> = T
fn run(values: Box<String, Int>) -> String { values }
"#,
        )
        .expect("failed to write user-defined generic-arity mismatch fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("generic arity mismatch for `Box`: expected 1, got 2"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_undefined_type_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_undefined_type_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run(input: MissingType) -> String { "ok" }
"#,
        )
        .expect("failed to write undefined-type source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("undefined type `MissingType"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_type_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_type_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run() -> String { return 42 }
"#,
        )
        .expect("failed to write type-mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type mismatch: expected `String`, got `Int`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_implicit_return_type_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_implicit_return_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run() -> String { 42 }
"#,
        )
        .expect("failed to write implicit-return mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type mismatch: expected `String`, got `Int`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_no_such_field_record_literal_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_no_such_field_record_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}
"#,
        )
        .expect("failed to write no-such-field record-literal source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type `Record` has no field `missing`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_no_such_field_named_record_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_no_such_field_named_record_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
"#,
        )
        .expect("failed to write no-such-field named-record source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("type `Payload` has no field `missing`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_unsatisfiable_refinement_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unsatisfiable_refinement_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run(value: Int @range(min: 5, max: 1)) -> Int { value }
"#,
        )
        .expect("failed to write unsatisfiable-refinement source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unsatisfiable refinement on `Int`: range min 5 exceeds max 1"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_generic_arity_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_generic_arity_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run(values: Map<String>) -> Int { 1 }
"#,
        )
        .expect("failed to write generic-arity mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("generic arity mismatch for `Map`: expected 2, got 1"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_user_defined_generic_arity_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_user_defined_generic_arity_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type Box<T> = T
fn run(values: Box<String, Int>) -> String { values }
"#,
        )
        .expect("failed to write user-defined generic-arity mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("generic arity mismatch for `Box`: expected 1, got 2"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_call_arity_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_call_arity_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
"#,
        )
        .expect("failed to write call-arity mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("call arity mismatch"));
        assert!(error.contains("fmt"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_call_with_defaulted_params_succeeds() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_call_defaults_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn greet(name: String, punctuation: String = "!") -> String { name }
fn run() -> String { greet(name: "hi") }
"#,
        )
        .expect("failed to write defaulted callable source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_pattern_call_with_extra_named_wiring_args_succeeds() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_pattern_wiring_args_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
pattern ensure(should_act: Bool = true) -> { acted: Bool } {
  return { acted: should_act }
}
fn run() -> Bool {
  let result = ensure(check: true, action: false)
  result.acted
}
"#,
        )
        .expect("failed to write pattern wiring source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_generic_fn_type_params_typecheck_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_generic_fn_type_params_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn identity<T>(value: T) -> T {
  value
}
fn relay<T>(value: T) -> T {
  identity(value: value)
}
"#,
        )
        .expect("failed to write generic fn source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_generic_pattern_type_params_typecheck_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_generic_pattern_type_params_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
pattern passthrough<T: Serializable>(value: T) -> { value: T } {
  return { value: value }
}
fn relay<T>(value: T) -> T {
  let result = passthrough(value: value)
  result.value
}
"#,
        )
        .expect("failed to write generic pattern source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_named_record_literal_return_succeeds_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_named_record_literal_return_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
type StageResult {
  success: Bool,
  skipped: Bool
}
fn result() -> StageResult {
  { success: true, skipped: false }
}
"#,
        )
        .expect("failed to write named record literal source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_resource_config_named_return_succeeds_in_strict_mode() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_resource_config_named_return_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
resource GcsBucket {
  config {
    name: String,
    project: String
  }
}
fn gcp_dev_storage() -> GcsBucket.Config {
  { name: "gunbc-dev-artifacts", project: "gunbai-auto" }
}
"#,
        )
        .expect("failed to write resource config source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_unknown_named_call_argument_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unknown_named_call_arg_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
"#,
        )
        .expect("failed to write unknown named-call argument source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown named argument"));
        assert!(error.contains("text"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_duplicate_named_call_argument_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_named_call_arg_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(value: "a", value: "b") }
"#,
        )
        .expect("failed to write duplicate named-call argument source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate named argument"));
        assert!(error.contains("value"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_service_call_arity_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_service_call_arity_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read()
  return { body: response.body }
}
"#,
        )
        .expect("failed to write service call-arity mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("service call arity mismatch"));
        assert!(error.contains("FsStorage.read"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_service_call_with_defaulted_inputs_succeeds() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_service_call_defaults_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input {
      path: String,
      recursive: Bool = false
    }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String, recursive: Bool = false) -> { ok: Bool }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read(path: "/tmp")
  return { ok: response.ok }
}
"#,
        )
        .expect("failed to write defaulted service-call source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_unknown_named_service_argument_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_unknown_named_service_arg_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(name: "README.md")
  return { body: response.body }
}
"#,
        )
        .expect("failed to write unknown named service-argument source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("unknown named argument"));
        assert!(error.contains("name"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_duplicate_named_service_argument_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_named_service_arg_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(path: "a", path: "b")
  return { body: response.body }
}
"#,
        )
        .expect("failed to write duplicate named service-argument source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate named argument"));
        assert!(error.contains("path"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_duplicate_parameter_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_duplicate_parameter");
        std::fs::write(
            &fixture,
            r#"module sample.single
fn run(a: String, a: Int) -> String { a }
"#,
        )
        .expect("failed to write duplicate-parameter fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate parameter `a` in `run`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_duplicate_output_field_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_duplicate_output_field");
        std::fs::write(
            &fixture,
            r#"module sample.single
func run() -> { ok: Bool, ok: String } { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate-output fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate output field `ok` in `run`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_duplicate_uses_binding_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_duplicate_uses_binding");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate-uses fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate uses binding `fs` in `run`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_duplicate_provides_binding_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_duplicate_provides_binding");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } provides out: Storage provides out: Storage { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate-provides fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate provides binding `out` in `run`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_use_provide_binding_conflict_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_use_provide_binding_conflict");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses io: Storage provides io: Storage { return { ok: true } }
"#,
        )
        .expect("failed to write use/provide conflict fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("binding `io` is declared in both uses/provides in `run`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_missing_resource_capability_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_missing_resource_capability");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
        )
        .expect("failed to write missing-resource-capability fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(
            error.contains("resource `Disk` is missing capability `write` for interface `Storage`")
        );

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_missing_service_operation_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_missing_service_operation");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
        )
        .expect("failed to write missing-service-operation fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error
            .contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_service_interface_signature_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_interface_signature_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
}
"#,
        )
        .expect("failed to write service-signature-mismatch fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`FsStorage` does not match `Storage.read` contract"));
        assert!(error.contains("expected `String` but found `Int`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_single_file_resource_interface_signature_mismatch_fails_in_typecheck_stage() {
        let fixture = unique_temp_file("single_file_resource_signature_mismatch");
        std::fs::write(
            &fixture,
            r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}
"#,
        )
        .expect("failed to write resource-signature-mismatch fixture");

        let context = PipelineContext {
            roots: vec![fixture
                .parent()
                .expect("fixture should have parent")
                .to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`Disk` does not match `Storage.read` contract"));
        assert!(error.contains("expected `String` but found `Int`"));

        std::fs::remove_file(fixture).expect("failed to cleanup fixture");
    }

    #[test]
    fn compile_directory_duplicate_parameter_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_parameter_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
fn run(a: String, a: Int) -> String { a }
"#,
        )
        .expect("failed to write duplicate-parameter source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate parameter `a` in `run`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_duplicate_output_field_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_output_field_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
func run() -> { ok: Bool, ok: String } { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate-output source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate output field `ok` in `run`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_duplicate_uses_binding_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_uses_binding_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate-uses source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate uses binding `fs` in `run`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_duplicate_provides_binding_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_duplicate_provides_binding_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } provides out: Storage provides out: Storage { return { ok: true } }
"#,
        )
        .expect("failed to write duplicate-provides source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate provides binding `out` in `run`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_use_provide_binding_conflict_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_use_provide_binding_conflict_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses io: Storage provides io: Storage { return { ok: true } }
"#,
        )
        .expect("failed to write use/provide conflict source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("binding `io` is declared in both uses/provides in `run`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_missing_resource_capability_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_missing_resource_capability_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
        )
        .expect("failed to write missing-resource-capability source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(
            error.contains("resource `Disk` is missing capability `write` for interface `Storage`")
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_missing_service_operation_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_missing_service_operation_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
        )
        .expect("failed to write missing-service-operation source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error
            .contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_service_interface_signature_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_service_signature_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
}
"#,
        )
        .expect("failed to write service-signature-mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`FsStorage` does not match `Storage.read` contract"));
        assert!(error.contains("expected `String` but found `Int`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_resource_interface_signature_mismatch_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_resource_signature_mismatch_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}
"#,
        )
        .expect("failed to write resource-signature-mismatch source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`Disk` does not match `Storage.read` contract"));
        assert!(error.contains("expected `String` but found `Int`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_ambiguous_interface_reference_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_ambiguous_interface_reference_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/first.dag"),
            "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write first interface source");
        std::fs::write(
            root.join("sample/second.dag"),
            "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write second interface source");
        std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nservice FsStorage implements Storage { operation read(path: String) -> { body: String } }",
        )
        .expect("failed to write main source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("ambiguous interface `Storage`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_ambiguous_resource_interface_reference_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_ambiguous_resource_interface_reference_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/first.dag"),
            "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write first interface source");
        std::fs::write(
            root.join("sample/second.dag"),
            "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write second interface source");
        std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nresource Disk implements Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write main source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("`Disk` references ambiguous interface `Storage`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_ambiguous_uses_resource_type_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_ambiguous_uses_resource_type_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/one.dag"),
            "module sample.one\nresource SharedResource {}",
        )
        .expect("failed to write first resource source");
        std::fs::write(
            root.join("sample/two.dag"),
            "module sample.two\nresource SharedResource {}",
        )
        .expect("failed to write second resource source");
        std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nfunc run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }",
        )
        .expect("failed to write main source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("ambiguous used resource type `SharedResource`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_ambiguous_provides_resource_type_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_ambiguous_provides_resource_type_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/one.dag"),
            "module sample.one\nresource SharedResource {}",
        )
        .expect("failed to write first resource source");
        std::fs::write(
            root.join("sample/two.dag"),
            "module sample.two\nresource SharedResource {}",
        )
        .expect("failed to write second resource source");
        std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nfunc run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }",
        )
        .expect("failed to write main source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("ambiguous provided resource type `SharedResource`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_ambiguous_service_call_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_ambiguous_service_call_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/first.dag"),
            r#"module sample.first
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
        )
        .expect("failed to write first service source");
        std::fs::write(
            root.join("sample/second.dag"),
            r#"module sample.second
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
        )
        .expect("failed to write second service source");
        std::fs::write(
            root.join("sample/main.dag"),
            r#"module sample.main
func run(path: String) -> { body: String } {
  let response = SharedService.read(path: path)
  return { body: response.body }
}"#,
        )
        .expect("failed to write main source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("ambiguous service call `SharedService.read`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_directory_ambiguous_callable_target_fails_in_typecheck_stage() {
        let root = std::env::temp_dir().join(format!(
            "daglang_compile_ambiguous_callable_target_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/one.dag"),
            "module sample.one\nfn render(value: String) -> String { value }",
        )
        .expect("failed to write first callable source");
        std::fs::write(
            root.join("sample/two.dag"),
            "module sample.two\nfn render(value: String) -> String { value }",
        )
        .expect("failed to write second callable source");
        std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nfn run() -> String { render(value: \"ok\") }",
        )
        .expect("failed to write main source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("ambiguous call target `render`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }
}
