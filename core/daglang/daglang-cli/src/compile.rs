use std::path::PathBuf;

use crate::pipeline::PipelineContext;
use daglang_derive::{derive_artifacts, DerivedArtifacts};
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

pub fn render_manifest(derived: &DerivedArtifacts) -> String {
    let manifest = &derived.manifest;
    let obligations = &derived.obligations;
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
    out.push_str("TestObligations:\n");
    out.push_str(&format!(
        "  dry_run_completion_required: {}\n",
        obligations.dry_run_completion_required
    ));
    out.push_str(&format!(
        "  transport_execution_targets: {}\n",
        obligations.transport_execution_targets
    ));
    out.push_str(&format!(
        "  pure_node_determinism_targets: {}\n",
        obligations.pure_node_determinism_targets
    ));
    out.push_str(&format!(
        "  service_transport_prepare_targets: {}\n",
        obligations.service_transport_prepare_targets
    ));
    out.push_str(&format!(
        "  service_transport_execute_targets: {}\n",
        obligations.service_transport_execute_targets
    ));
    out.push_str(&format!(
        "  service_transport_parse_targets: {}\n",
        obligations.service_transport_parse_targets
    ));
    out.push_str(&format!(
        "  service_param_source_targets: {}\n",
        obligations.service_param_source_targets
    ));
    out.push_str(&format!(
        "  resource_provide_targets: {}\n",
        obligations.resource_provide_targets
    ));
    out.push_str(&format!(
        "  resource_acquire_targets: {}\n",
        obligations.resource_acquire_targets
    ));
    out.push_str(&format!(
        "  resource_release_targets: {}\n",
        obligations.resource_release_targets
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

    fn assert_typecheck_stage_error(error: &str) {
        assert!(error.contains("typecheck errors"));
        assert!(!error.contains("lower error"));
    }

    fn assert_lower_stage_error(error: &str) {
        assert!(error.contains("lower error"));
        assert!(!error.contains("typecheck errors"));
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
}
