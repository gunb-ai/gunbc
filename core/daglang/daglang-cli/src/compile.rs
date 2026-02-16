use std::fmt::Write;
use std::path::PathBuf;
use std::collections::{BTreeSet, HashMap};

use crate::path_utils;
use crate::pipeline::PipelineContext;
use daglang_derive::{derive_artifacts, DerivedArtifacts, TestObligations};
use daglang_emit::{emit_rust_bundle, EmissionBundle};
use daglang_lower::{lower_typed_project, LoweredOp};
use daglang_resolve::{ModuleGraph, ResolveError, ResolvedModule};
use daglang_syntax::diagnostic;
use daglang_syntax::parser;
use daglang_typecheck::{typecheck_module_graph_with_options, TypecheckOptions};
use gunbc_ir::{Dag, Node};
use serde_json::json;

#[derive(Debug)]
pub struct CompileOutput {
    pub lowered_dag: Dag<LoweredOp>,
    pub derived: DerivedArtifacts,
    pub emitted: EmissionBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn build_context(cwd: &std::path::Path, input: Option<&String>) -> PipelineContext {
    let parsed = input.map(|value| path_utils::normalize_cli_path(cwd, &PathBuf::from(value)));
    let (roots, target_file) = match parsed {
        Some(path) if path_utils::has_dag_extension(&path) => {
            let root = path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| cwd.to_path_buf());
            (vec![root], Some(path))
        }
        Some(path) => (vec![path], None),
        None => (vec![path_utils::resolve_default_root(cwd)], None),
    };

    PipelineContext { roots, target_file }
}

pub fn compile_from_context(context: &PipelineContext) -> Result<CompileOutput, String> {
    let module_graph = discover_module_graph_for_context(context)?;
    if context.target_file.is_none() {
        validate_module_path_consistency(&module_graph, &context.roots)?;
    }
    let typed = typecheck_module_graph_with_options(
        module_graph,
        TypecheckOptions {
            allow_unresolved_imports: context.target_file.is_some(),
        },
    )
    .map_err(|errors| {
        let mut message = String::from("typecheck errors:\n");
        for error in errors {
            writeln!(message, "  {error}").ok();
        }
        message
    })?;
    let lowered = lower_typed_project(&typed).map_err(|error| format!("lower error: {error}"))?;
    let derived = derive_artifacts(&lowered).map_err(|error| format!("derive error: {error}"))?;
    let emitted =
        emit_rust_bundle(&lowered, &derived).map_err(|error| format!("emit error: {error}"))?;

    Ok(CompileOutput {
        lowered_dag: lowered,
        derived,
        emitted,
    })
}

// Compiler pipeline: reads .dag source for single-file compilation
#[allow(clippy::disallowed_methods)]
fn discover_module_graph_for_context(context: &PipelineContext) -> Result<ModuleGraph, String> {
    if let Some(target_file) = &context.target_file {
        let source = std::fs::read_to_string(target_file)
            .map_err(|error| format!("failed to read {}: {error}", target_file.display()))?;
        let ast = parser::parse(&source).map_err(|errors| {
            let mut message = String::from("compile diagnostics:\n");
            for error in &errors {
                writeln!(message, "  {}", error.format_with_source(target_file, &source)).ok();
            }
            message
        })?;
        let module_path = ast
            .module_path
            .as_ref()
            .map(|module| module.node.segments.clone())
            .unwrap_or_else(|| {
                target_file
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|stem| vec![stem.to_string()])
                    .unwrap_or_else(|| vec!["unknown".to_string()])
            });
        return Ok(ModuleGraph {
            modules: vec![ResolvedModule {
                path: target_file.clone(),
                ast,
                module_path,
                dependencies: Vec::new(),
            }],
        });
    }

    ModuleGraph::discover_strict(&context.roots).map_err(format_resolve_error)
}

fn format_resolve_error(error: ResolveError) -> String {
    match error {
        ResolveError::ParseErrors(files) => {
            let mut message = String::from("compile diagnostics:\n");
            let diagnostics = diagnostic::normalize_diagnostics(
                files
                    .into_iter()
                    .flat_map(|(_path, diagnostics)| diagnostics)
                    .collect(),
            );
            for diagnostic in diagnostics {
                writeln!(message, "  {}", diagnostic.render()).ok();
            }
            message
        }
        other => format!("resolve error: {other}"),
    }
}

fn validate_module_path_consistency(
    graph: &ModuleGraph,
    roots: &[PathBuf],
) -> Result<(), String> {
    let mismatches = graph
        .modules
        .iter()
        .filter_map(|module| {
            let derived = derive_module_path(&module.path, roots)?;
            (derived != module.module_path).then_some((
                module.path.clone(),
                module.module_path.join("."),
                derived.join("."),
            ))
        })
        .collect::<Vec<_>>();

    if mismatches.is_empty() {
        return Ok(());
    }

    let mut message = String::from("module path mismatches:\n");
    for (path, declared, derived) in mismatches {
        writeln!(message, "  {}: declared `{declared}` but filesystem implies `{derived}`", path.display()).ok();
    }
    Err(message)
}

fn derive_module_path(path: &std::path::Path, roots: &[PathBuf]) -> Option<Vec<String>> {
    for root in roots {
        if let Ok(relative) = path.strip_prefix(root) {
            return Some(daglang_resolve::relative_path_to_module_path(relative));
        }
    }
    None
}

pub fn render_expand(dag: &Dag<LoweredOp>) -> String {
    let mut out = String::new();
    out.push_str("Nodes:\n");
    for node in &dag.nodes {
        let kind = match &node.body {
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable {
                kind,
                module,
                name,
                ..
            }) => {
                format!("callable::{kind:?} {module}.{name}")
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline {
                module,
                name,
                stages,
            }) => format!("pipeline {module}.{name} ({stages} stages)"),
            gunbc_ir::node::NodeBody::SubDag(_) => "subdag".to_string(),
        };

        writeln!(out, "  - {} [{kind}]", node.id.0).ok();
        if !node.inputs.is_empty() {
            out.push_str("    inputs:\n");
            for input in &node.inputs {
                writeln!(out, "      * {}: {} ({})", input.name.0, input.type_id.0, input.cardinality).ok();
            }
        }
        if !node.outputs.is_empty() {
            out.push_str("    outputs:\n");
            for output in &node.outputs {
                writeln!(out, "      * {}: {} ({})", output.name.0, output.type_id.0, output.cardinality).ok();
            }
        }
    }

    out.push_str("Edges:\n");
    if dag.edges.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for edge in &dag.edges {
            writeln!(out, "  - {}.{} -> {}.{}", edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0).ok();
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
        out, "  entrypoint_nodes: {}",
        if manifest.entrypoint_nodes.is_empty() {
            "(none)".to_string()
        } else {
            manifest.entrypoint_nodes.join(", ")
        }
    ).ok();
    writeln!(
        out, "  boundary_nodes: {}",
        if manifest.boundary_nodes.is_empty() {
            "(none)".to_string()
        } else {
            manifest.boundary_nodes.join(", ")
        }
    ).ok();
    out.push_str(&render_obligations_text(&derived.obligations));
    out
}

pub fn render_obligations(derived: &DerivedArtifacts, format: OutputFormat) -> String {
    let obligations = &derived.obligations;
    match format {
        OutputFormat::Text => render_obligations_text(obligations),
        OutputFormat::Json => json!({
            "dry_run_completion_required": obligations.dry_run_completion_required,
            "transport_execution_targets": obligations.transport_execution_targets,
            "pure_node_determinism_targets": obligations.pure_node_determinism_targets,
            "service_transport_prepare_targets": obligations.service_transport_prepare_targets,
            "service_transport_execute_targets": obligations.service_transport_execute_targets,
            "service_transport_parse_targets": obligations.service_transport_parse_targets,
            "service_param_source_targets": obligations.service_param_source_targets,
            "resource_provide_targets": obligations.resource_provide_targets,
            "resource_acquire_targets": obligations.resource_acquire_targets,
            "resource_release_targets": obligations.resource_release_targets
        })
        .to_string(),
    }
}

fn render_obligations_text(obligations: &TestObligations) -> String {
    let mut out = String::new();
    out.push_str("TestObligations:\n");
    writeln!(out, "  dry_run_completion_required: {}", obligations.dry_run_completion_required).ok();
    writeln!(out, "  transport_execution_targets: {}", obligations.transport_execution_targets).ok();
    writeln!(out, "  pure_node_determinism_targets: {}", obligations.pure_node_determinism_targets).ok();
    writeln!(out, "  service_transport_prepare_targets: {}", obligations.service_transport_prepare_targets).ok();
    writeln!(out, "  service_transport_execute_targets: {}", obligations.service_transport_execute_targets).ok();
    writeln!(out, "  service_transport_parse_targets: {}", obligations.service_transport_parse_targets).ok();
    writeln!(out, "  service_param_source_targets: {}", obligations.service_param_source_targets).ok();
    writeln!(out, "  resource_provide_targets: {}", obligations.resource_provide_targets).ok();
    writeln!(out, "  resource_acquire_targets: {}", obligations.resource_acquire_targets).ok();
    writeln!(out, "  resource_release_targets: {}", obligations.resource_release_targets).ok();
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
                        "parse_nodes": triplet.parse_nodes
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
        if node_output_port_type(prepare_node, edge.from_port.0.as_str()) != Some("TransportRequest")
            || node_input_port_type(execute_node, edge.to_port.0.as_str()) != Some("TransportRequest")
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

        unique.insert(TransportTriplet {
            prepare_node: edge.from_node.0.clone(),
            execute_node: edge.to_node.0.clone(),
            parse_nodes,
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
    use daglang_lower::{CallableKind, ObligationCategory};
    use gunbc_ir::{Edge, Node, Port};
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "daglang_cli_compile_{name}_{}_{}.dag",
            std::process::id(),
            nanos
        ))
    }

    fn assert_typecheck_stage_error(error: &str) {
        assert!(error.contains("typecheck errors"));
        assert!(!error.contains("lower error"));
    }

    fn assert_lower_stage_error(error: &str) {
        assert!(error.contains("lower error"));
        assert!(!error.contains("typecheck errors"));
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
        let input = root.join("sample").join("nested").join("..").join("main.dag");
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
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let context = PipelineContext {
            roots: vec![file.parent().expect("file should have parent").to_path_buf()],
            target_file: Some(file),
        };

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
    fn render_obligations_json_emits_expected_keys() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let context = PipelineContext {
            roots: vec![file.parent().expect("file should have parent").to_path_buf()],
            target_file: Some(file),
        };
        let output = compile_from_context(&context).expect("compile should succeed");

        let rendered = render_obligations(&output.derived, OutputFormat::Json);
        let parsed: Value =
            serde_json::from_str(&rendered).expect("obligations json should parse");
        assert_eq!(
            parsed.get("dry_run_completion_required").and_then(Value::as_bool),
            Some(true)
        );
        assert!(parsed.get("transport_execution_targets").is_some());
        assert!(parsed.get("pure_node_determinism_targets").is_some());
    }

    #[test]
    fn render_triplets_json_includes_makegen_transport_nodes() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let context = PipelineContext {
            roots: vec![file.parent().expect("file should have parent").to_path_buf()],
            target_file: Some(file),
        };
        let output = compile_from_context(&context).expect("compile should succeed");

        let rendered = render_triplets(&output.lowered_dag, OutputFormat::Json);
        let parsed: Value =
            serde_json::from_str(&rendered).expect("triplets json should parse");
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
    fn render_triplets_text_is_deterministic() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let context = PipelineContext {
            roots: vec![file.parent().expect("file should have parent").to_path_buf()],
            target_file: Some(file),
        };
        let output = compile_from_context(&context).expect("compile should succeed");

        let first = render_triplets(&output.lowered_dag, OutputFormat::Text);
        let second = render_triplets(&output.lowered_dag, OutputFormat::Text);
        assert_eq!(first, second, "triplet rendering should be deterministic");
    }

    #[test]
    fn render_manifest_reuses_obligations_text_block() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let context = PipelineContext {
            roots: vec![file.parent().expect("file should have parent").to_path_buf()],
            target_file: Some(file),
        };
        let output = compile_from_context(&context).expect("compile should succeed");

        let manifest = render_manifest(&output.derived);
        let obligations = render_obligations(&output.derived, OutputFormat::Text);
        assert!(
            manifest.ends_with(&obligations),
            "manifest output should embed the same obligations text renderer"
        );
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
            roots: vec![
                broken_file
                    .parent()
                    .expect("temp file should have parent")
                    .to_path_buf(),
            ],
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
    fn compile_single_file_unresolved_service_call_fails_in_lower_stage() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_lower_stage_error(&error);
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
    fn compile_single_file_unresolved_uses_fails_in_lower_stage() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_lower_stage_error(&error);
        assert!(error.contains("unresolved used resource"));
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
    fn compile_single_file_unresolved_provides_fails_in_lower_stage() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_lower_stage_error(&error);
        assert!(error.contains("unresolved provided resource"));
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
    fn compile_single_file_allows_unresolved_imports_in_relaxed_mode() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

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
    fn compile_single_file_allows_unresolved_call_targets_in_relaxed_mode() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let output = compile_from_context(&context).expect("compile should succeed");
        assert!(!output.lowered_dag.nodes.is_empty());
        assert!(output.derived.manifest.total_nodes > 0);
        assert!(!output.emitted.files.is_empty());

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
    fn compile_single_file_duplicate_service_suppresses_ambiguous_service_call() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `FsStorage`"));
        assert!(!error.contains("ambiguous service call"));

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
    fn compile_single_file_duplicate_callable_suppresses_ambiguous_call_target() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `helper`"));
        assert!(!error.contains("ambiguous call target"));

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
    fn compile_single_file_duplicate_resource_uses_suppresses_ambiguous_used_type() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `SharedResource`"));
        assert!(!error.contains("ambiguous used resource type"));

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
    fn compile_single_file_duplicate_resource_provides_suppresses_ambiguous_provided_type() {
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("duplicate definition `SharedResource`"));
        assert!(!error.contains("ambiguous provided resource type"));

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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
            target_file: Some(fixture.clone()),
        };

        let error = compile_from_context(&context).expect_err("compile should fail");
        assert_typecheck_stage_error(&error);
        assert!(error.contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
            roots: vec![fixture.parent().expect("fixture should have parent").to_path_buf()],
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
        assert!(error.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

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
        assert!(error.contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

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
