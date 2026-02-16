use std::path::PathBuf;

use crate::pipeline::PipelineContext;
use daglang_derive::{derive_artifacts, DerivedArtifacts, ProgressManifest};
use daglang_emit::{emit_rust_bundle, EmissionBundle};
use daglang_lower::{lower_typed_project, LoweredOp};
use daglang_resolve::{ModuleGraph, ResolveError, ResolvedModule};
use daglang_syntax::parser;
use daglang_typecheck::{typecheck_module_graph_with_options, TypecheckOptions};
use gunbc_ir::Dag;

#[derive(Debug)]
pub struct CompileOutput {
    pub lowered_dag: Dag<LoweredOp>,
    pub derived: DerivedArtifacts,
    pub emitted: EmissionBundle,
}

pub fn build_context(input: Option<&String>) -> PipelineContext {
    let parsed = input.map(PathBuf::from);
    let (roots, target_file) = match parsed {
        Some(path) if path.extension().and_then(|ext| ext.to_str()) == Some("dag") => {
            let root = path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into()));
            (vec![root], Some(path))
        }
        Some(path) => (vec![path], None),
        None => (vec![resolve_default_root()], None),
    };

    PipelineContext { roots, target_file }
}

pub fn resolve_default_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dsl = cwd.join("dsl");
    if dsl.exists() {
        dsl
    } else {
        PathBuf::from("dsl")
    }
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
            message.push_str(&format!("  {error}\n"));
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

fn discover_module_graph_for_context(context: &PipelineContext) -> Result<ModuleGraph, String> {
    if let Some(target_file) = &context.target_file {
        let source = std::fs::read_to_string(target_file)
            .map_err(|error| format!("failed to read {}: {error}", target_file.display()))?;
        let ast = parser::parse(&source).map_err(|errors| {
            let mut message = String::from("compile diagnostics:\n");
            for error in &errors {
                message.push_str(&format!(
                    "  {}\n",
                    error.format_with_source(target_file, &source)
                ));
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

    ModuleGraph::discover(&context.roots).map_err(format_resolve_error)
}

fn format_resolve_error(error: ResolveError) -> String {
    match error {
        ResolveError::ParseErrors(files) => {
            let mut message = String::from("compile diagnostics:\n");
            for (_path, diagnostics) in files {
                for diagnostic in diagnostics {
                    message.push_str(&format!("  {}\n", diagnostic.render()));
                }
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
        message.push_str(&format!(
            "  {}: declared `{declared}` but filesystem implies `{derived}`\n",
            path.display()
        ));
    }
    Err(message)
}

fn derive_module_path(path: &std::path::Path, roots: &[PathBuf]) -> Option<Vec<String>> {
    for root in roots {
        if let Ok(relative) = path.strip_prefix(root) {
            return Some(
                relative
                    .with_extension("")
                    .components()
                    .filter_map(|segment| segment.as_os_str().to_str().map(String::from))
                    .collect(),
            );
        }
    }
    None
}

pub fn render_expand(dag: &Dag<LoweredOp>) -> String {
    let mut out = String::new();
    out.push_str("Nodes:\n");
    for node in &dag.nodes {
        let kind = match &node.body {
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable { kind, module, name }) => {
                format!("callable::{kind:?} {module}.{name}")
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline {
                module,
                name,
                stages,
            }) => format!("pipeline {module}.{name} ({stages} stages)"),
            gunbc_ir::node::NodeBody::SubDag(_) => "subdag".to_string(),
        };

        out.push_str(&format!("  - {} [{kind}]\n", node.id.0));
        if !node.inputs.is_empty() {
            out.push_str("    inputs:\n");
            for input in &node.inputs {
                out.push_str(&format!(
                    "      * {}: {} ({})\n",
                    input.name.0, input.type_id.0, input.cardinality
                ));
            }
        }
        if !node.outputs.is_empty() {
            out.push_str("    outputs:\n");
            for output in &node.outputs {
                out.push_str(&format!(
                    "      * {}: {} ({})\n",
                    output.name.0, output.type_id.0, output.cardinality
                ));
            }
        }
    }

    out.push_str("Edges:\n");
    if dag.edges.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for edge in &dag.edges {
            out.push_str(&format!(
                "  - {}.{} -> {}.{}\n",
                edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
            ));
        }
    }
    out
}

pub fn render_manifest(manifest: &ProgressManifest) -> String {
    let mut out = String::new();
    out.push_str("ProgressManifest:\n");
    out.push_str(&format!("  total_nodes: {}\n", manifest.total_nodes));
    out.push_str(&format!("  total_edges: {}\n", manifest.total_edges));
    out.push_str("  waves:\n");
    for (index, wave) in manifest.waves.iter().enumerate() {
        out.push_str(&format!("    [{index}] {}\n", wave.join(", ")));
    }
    out.push_str(&format!(
        "  entrypoint_nodes: {}\n",
        if manifest.entrypoint_nodes.is_empty() {
            "(none)".to_string()
        } else {
            manifest.entrypoint_nodes.join(", ")
        }
    ));
    out.push_str(&format!(
        "  boundary_nodes: {}\n",
        if manifest.boundary_nodes.is_empty() {
            "(none)".to_string()
        } else {
            manifest.boundary_nodes.join(", ")
        }
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
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

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }
}
